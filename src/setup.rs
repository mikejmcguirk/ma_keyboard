use {
    core::str,
    std::{
        fs::File,
        io::{Write as _, stdin, stdout},
        path::Path,
        process::ExitCode,
    },
};

use anyhow::Result;

use crate::{
    corpus::initialize_corpus,
    display::{initial_dsp, update_dvorak, update_iter, update_pop_dsp, update_qwerty},
    keyboard::Keyboard,
    population::Population,
    structs::IdSpawner,
    utils::write_log,
};

// FUTURE: At some point I'll come up with a way to load key settings from a config rather than
// having to edit the source code. A lot of things would then need error propagation
// FUTURE: Keeping the setup naming for now. At some point we're going to add arg processing and then
// it would make more sense to do that here and then break out actually running the training in its
// own file
// FUTURE: Args:
// - Save file to load
// - Read from config file
// - The input options will have restrictions on what is possible. Should be possible to print them
pub fn setup(log_handle: &mut File, log_dir: &Path) -> Result<ExitCode> {
    const ITERATIONS: usize = 2000;
    const PROG_NAME: &str = "MA Keyboard Generator";
    // SAFETY: PROG_NAME is defined at compile time
    const NAME_DASHES: &str = unsafe { str::from_utf8_unchecked(&[b'='; PROG_NAME.len()]) };

    let message = "Initializing...";
    write_log(log_handle, &message)?;

    println!();
    println!("{NAME_DASHES}");
    println!("{PROG_NAME}");
    println!("{NAME_DASHES}");
    println!();
    println!("Log Path: {}", log_dir.display());
    println!();

    if let Some(exit_code) = confirm_continue() {
        return Ok(exit_code);
    }

    initialize_corpus()?;

    let mut id_spawner = IdSpawner::new();
    let mut population = Population::create(id_spawner.get());

    initial_dsp(&population)?;

    let mut qwerty = Keyboard::create_qwerty();
    qwerty.eval();
    update_qwerty(qwerty.get_score())?;

    let mut dvorak = Keyboard::create_dvorak();
    dvorak.eval();
    update_dvorak(dvorak.get_score())?;

    for iter in 1..=ITERATIONS {
        update_iter(iter)?;
        population.randomize_pop_cnt();
        population.randomize_climber_cnt();
        population.randomize_elite_cnt();
        population.randomize_mutation();
        update_pop_dsp(&population)?;
        population.refill_pop();

        population.eval_gen_pop()?;
        population.setup_climbers()?;
        population.climb_kbs(iter)?;
    }

    println!();
    println!("Complete");
    println!();

    return Ok(ExitCode::SUCCESS);
}

fn confirm_continue() -> Option<ExitCode> {
    let mut input = String::new();

    loop {
        print!("Continue? [Y/N]: ");
        if let Err(e) = stdout().flush() {
            eprintln!("Failed to flush stdout: {e}");
            return Some(ExitCode::FAILURE);
        }
        if let Err(e) = stdin().read_line(&mut input) {
            eprintln!("Failed to read input: {e}");
            return Some(ExitCode::FAILURE);
        }

        match input.trim().to_lowercase().as_str() {
            "y" | "yes" => {
                println!();
                return None;
            }
            "n" | "no" => {
                println!("User chose to exit");
                println!();
                return Some(ExitCode::from(2));
            }
            _ => println!("Invalid input. Please enter 'Y' or 'N'"),
        }
        input.clear();
        println!();
    }
}
