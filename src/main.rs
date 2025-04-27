#![warn(clippy::all)]
#![warn(clippy::cargo)]
#![warn(clippy::complexity)]
#![warn(clippy::correctness)]
#![warn(clippy::pedantic)]
#![warn(clippy::perf)]
#![warn(clippy::restriction)]
#![warn(clippy::style)]
#![warn(clippy::suspicious)]
// Always Allow (General)
#![allow(clippy::allow_attributes_without_reason)] // This is what comments are for
#![allow(clippy::arbitrary_source_item_ordering)] // Promotes illogical organization
#![allow(clippy::blanket_clippy_restriction_lints)] // I do what I want
#![allow(clippy::integer_division)] // I know
#![allow(clippy::integer_division_remainder_used)] // I know
#![allow(clippy::min_ident_chars)] // i, j, e, and so on are fine
#![allow(clippy::missing_docs_in_private_items)] // Flags crates I can't edit, and just too much
#![allow(clippy::module_name_repetitions)] // Promotes contrived naming
#![allow(clippy::multiple_crate_versions)] // I can't do anything about this
#![allow(clippy::multiple_unsafe_ops_per_block)]
#![allow(clippy::needless_return)] // I like explicit returns
#![allow(clippy::question_mark_used)] // Good for concision
#![allow(clippy::redundant_else)] // Conflicts with else_if_without_else
#![allow(clippy::redundant_type_annotations)] // I like explicit types
#![allow(clippy::semicolon_outside_block)] // I prefer inside
#![allow(clippy::separated_literal_suffix)] // I prefer separated
#![allow(clippy::similar_names)] // Promotes contrived naming
#![allow(clippy::single_char_lifetime_names)] // Multi-char names lead to bloated code
#![allow(clippy::single_call_fn)] // Good for code organization
#![allow(clippy::struct_field_names)] // Promotes contrived naming
#![allow(clippy::too_many_lines)] // Unneeded
#![allow(clippy::undocumented_unsafe_blocks)]
// Always Allow (Project Specific)
#![allow(clippy::exhaustive_enums)] // This isn't a library
#![allow(clippy::exhaustive_structs)] // This isn't a library
#![allow(clippy::missing_errors_doc)] // This isn't a library
#![allow(clippy::print_stderr)] // This is a terminal application
#![allow(clippy::print_stdout)] // This is a terminal application

// Allow these when just trying to get code down
#![allow(clippy::allow_attributes)]
#![allow(clippy::absolute_paths)]
#![allow(clippy::arithmetic_side_effects)]
#![allow(clippy::as_conversions)]
#![allow(clippy::assertions_on_result_states)] // Better than unwrap
#![allow(clippy::bool_to_int_with_if)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::create_dir)]
#![allow(clippy::default_numeric_fallback)]
#![allow(clippy::double_must_use)]
#![allow(clippy::else_if_without_else)]
#![allow(clippy::excessive_precision)] // Creates problems when using literals
#![allow(clippy::expect_used)]
#![allow(clippy::explicit_iter_loop)]
#![allow(clippy::field_reassign_with_default)]
#![allow(clippy::indexing_slicing)]
#![allow(clippy::items_after_statements)]
#![allow(clippy::len_zero)]
#![allow(clippy::let_underscore_must_use)]
#![allow(clippy::let_underscore_untyped)]
#![allow(clippy::manual_let_else)]
#![allow(clippy::manual_string_new)]
#![allow(clippy::missing_assert_message)]
#![allow(clippy::missing_asserts_for_indexing)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::new_without_default)]
#![allow(clippy::panic_in_result_fn)]
#![allow(clippy::pathbuf_init_then_push)]
#![allow(clippy::pattern_type_mismatch)]
#![allow(clippy::question_mark)]
#![allow(clippy::similar_names)]
#![allow(clippy::std_instead_of_alloc)]
#![allow(clippy::std_instead_of_core)]
#![allow(clippy::str_to_string)]
#![allow(clippy::too_many_lines)] // Encourages premature factoring
#![allow(clippy::type_complexity)] // Encourages premature factoring
#![allow(clippy::unnecessary_wraps)]
#![allow(clippy::uninlined_format_args)] // Trips when debug formatting is used
#![allow(clippy::unreadable_literal)] // Creates problems pasting literals
#![allow(clippy::unused_trait_names)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::useless_format)]
#![allow(clippy::use_debug)]

mod setup;

fn main() {
    println!("Hello, world!");
}
