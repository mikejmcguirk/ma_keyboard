#![warn(clippy::all)]
#![warn(clippy::cargo)]
#![warn(clippy::complexity)]
#![warn(clippy::correctness)]
#![warn(clippy::pedantic)]
#![warn(clippy::perf)]
#![warn(clippy::restriction)]
#![warn(clippy::style)]
#![warn(clippy::suspicious)]
// Always Allow
#![allow(clippy::allow_attributes_without_reason)] // This is what comments are for
#![allow(clippy::arbitrary_source_item_ordering)] // Promotes illogical organization
#![allow(clippy::blanket_clippy_restriction_lints)] // I do what I want
#![allow(clippy::integer_division)] // I know
#![allow(clippy::integer_division_remainder_used)] // I know
#![allow(clippy::min_ident_chars)] // i, j, e, and so on are fine
#![allow(clippy::missing_docs_in_private_items)] // Flags crates I can't edit, and just too much
#![allow(clippy::module_name_repetitions)] // Promotes contrived naming
#![allow(clippy::multiple_crate_versions)] // I can't do anything about this
#![allow(clippy::needless_return)] // I like explicit returns
#![allow(clippy::question_mark_used)] // Good for concision
#![allow(clippy::semicolon_outside_block)] // I prefer inside
#![allow(clippy::separated_literal_suffix)] // I prefer separated
#![allow(clippy::similar_names)] // Promotes contrived naming
#![allow(clippy::single_char_lifetime_names)] // Multi-char names lead to bloated code
#![allow(clippy::single_call_fn)] // Good for code organization
#![allow(clippy::struct_field_names)] // Promotes contrived naming
#![allow(clippy::type_complexity)] // Creating type defs is not helpful
// Always Allow (Project Specific)
#![allow(clippy::exhaustive_enums)] // This isn't a library
#![allow(clippy::exhaustive_structs)] // This isn't a library
#![allow(clippy::missing_errors_doc)] // This isn't a library
#![allow(clippy::print_stderr)] // This is a terminal application
#![allow(clippy::print_stdout)]
// To address
#![allow(clippy::allow_attributes)]
#![allow(clippy::absolute_paths)]
#![allow(clippy::bool_to_int_with_if)]
#![allow(clippy::comparison_chain)]
#![allow(clippy::create_dir)]
#![allow(clippy::default_numeric_fallback)]
#![allow(clippy::double_must_use)]
#![allow(clippy::excessive_precision)] // Don't need nags when pasting literals
#![allow(clippy::explicit_iter_loop)]
#![allow(clippy::field_reassign_with_default)]
#![allow(clippy::identity_op)]
#![allow(clippy::items_after_statements)]
#![allow(clippy::len_zero)]
#![allow(clippy::let_underscore_must_use)]
#![allow(clippy::let_underscore_untyped)]
#![allow(clippy::manual_assert)]
#![allow(clippy::manual_let_else)]
#![allow(clippy::manual_string_new)]
#![allow(clippy::match_on_vec_items)]
#![allow(clippy::match_same_arms)]
#![allow(clippy::missing_assert_message)]
#![allow(clippy::missing_asserts_for_indexing)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::needless_continue)]
#![allow(clippy::needless_range_loop)]
#![allow(clippy::new_without_default)]
#![allow(clippy::pathbuf_init_then_push)]
#![allow(clippy::pattern_type_mismatch)]
#![allow(clippy::question_mark)]
#![allow(clippy::redundant_type_annotations)]
#![allow(clippy::std_instead_of_alloc)]
#![allow(clippy::std_instead_of_core)]
#![allow(clippy::str_to_string)]
#![allow(clippy::undocumented_unsafe_blocks)]
#![allow(clippy::unnecessary_safety_comment)]
#![allow(clippy::unnecessary_wraps)]
#![allow(clippy::unreadable_literal)] // Creates problems pasting literals
#![allow(clippy::unused_self)]
#![allow(clippy::unused_trait_names)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::useless_format)]
#![allow(clippy::useless_vec)]
#![allow(clippy::vec_init_then_push)]
#![allow(clippy::verbose_file_reads)]
// To Check
#![allow(clippy::arithmetic_side_effects)]
#![allow(clippy::as_conversions)]
#![allow(clippy::assertions_on_result_states)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::else_if_without_else)]
#![allow(clippy::expect_used)]
#![allow(clippy::float_arithmetic)]
#![allow(clippy::float_cmp)]
#![allow(clippy::indexing_slicing)]
#![allow(clippy::missing_trait_methods)] // Can suggest unstable methods/code
#![allow(clippy::multiple_unsafe_ops_per_block)]
#![allow(clippy::panic)]
#![allow(clippy::panic_in_result_fn)]
#![allow(clippy::redundant_else)]
#![allow(clippy::similar_names)]
#![allow(clippy::string_slice)]
#![allow(clippy::too_many_lines)] // Encourages premature factoring
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::unwrap_in_result)]
#![allow(clippy::use_debug)]

mod corpus;
mod display;
mod eval_funcs;
mod kb_builders;
mod keyboard;
mod macros;
mod mapped_swap;
mod meta_pop;
mod pop_helpers;
mod population;
mod setup;
mod structs;
mod utils;

use std::{
    env,
    fs::{self, File, OpenOptions},
    path::{Path, PathBuf},
    process::ExitCode,
};

use anyhow::{Result, anyhow};

use crate::{setup::setup, utils::write_log};

fn main() -> ExitCode {
    let log_dir: PathBuf = match create_log_dir() {
        Ok(dir) => dir,
        Err(e) => {
            eprintln!("Unable to setup log dir: {e}");
            return ExitCode::FAILURE;
        }
    };

    let mut log_handle: File = match setup_log_handle(&log_dir) {
        Ok(handle) => handle,
        Err(e) => {
            eprintln!("Unable to setup logger: {e}");
            return ExitCode::FAILURE;
        }
    };

    match setup(&mut log_handle, &log_dir) {
        Ok(code) => return code,
        Err(e) => {
            if let Err(log_err) = write_log(&mut log_handle, &e) {
                eprintln!("{log_err}");
            }

            return ExitCode::FAILURE;
        }
    }
}

fn create_log_dir() -> Result<PathBuf> {
    let log_dir_parent: PathBuf = if cfg!(debug_assertions) {
        let cargo_root: String = env::var("CARGO_MANIFEST_DIR")?;
        cargo_root.into()
    } else {
        let exe_path = env::current_exe()?;
        if let Some(parent) = exe_path.parent() {
            parent.into()
        } else {
            return Err(anyhow!("No parent for {}", exe_path.display()));
        }
    };

    let log_dir: PathBuf = log_dir_parent.join("log");
    fs::create_dir_all(&log_dir)?;

    return Ok(log_dir);
}

fn setup_log_handle(log_dir: &Path) -> Result<File> {
    let err_file = get_log_file(log_dir)?;

    let handle: File = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&err_file)?;

    return Ok(handle);
}

fn get_log_file(log_dir: &Path) -> Result<PathBuf> {
    for i in 0_u8..=99_u8 {
        const MAX_FILE_SIZE: u64 = 1024 * 1024;

        let err_file = log_dir.join(format!("{i:02}.log"));
        if !err_file.exists() {
            return Ok(err_file);
        }

        let metadata = fs::metadata(&err_file)?;
        if metadata.len() < MAX_FILE_SIZE {
            return Ok(err_file);
        }
    }

    return Ok(log_dir.join("00.log"));
}
