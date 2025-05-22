use {
    core::str,
    std::{
        env,
        fs::{self, File, ReadDir},
        io::{Write as _, stdin, stdout},
        path::Path,
        path::PathBuf,
        process::ExitCode,
    },
};

use anyhow::{Result, anyhow};

use crate::{
    display::{draw_initial, update_dvorak, update_iter, update_qwerty},
    keyboard::Keyboard,
    population::Population,
};

// FUTURE: At some point I'll come up with a way to load key settings from a config rather than
// having to edit the source code. A lot of things would then need error propagation
// FUTURE: Will have to make a decision on how to do multi-threaded RNG. Single resource so I can
// re-use the seed? Or multiple RNGs for performance? Also, do we put SmallRng in a refcell or use
// threadRNG? Issue with threadRNG is - it's the slower version from what I understand
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

    let corpus_dir = get_corpus_dir()?;
    let corpus = load_corpus(&corpus_dir)?;

    let mut population = Population::create(None, log_handle)?;

    draw_initial(&population)?;

    let mut qwerty = Keyboard::create_qwerty();
    qwerty.eval(&corpus);
    update_qwerty(qwerty.get_score())?;

    let mut dvorak = Keyboard::create_dvorak();
    dvorak.eval(&corpus);
    update_dvorak(dvorak.get_score())?;

    for iter in 1..=ITERATIONS {
        update_iter(iter)?;
        population.refill_pop();

        population.eval_gen_pop(&corpus)?;
        population.setup_climbers()?;
        population.climb_kbs(&corpus, iter)?;
    }

    println!();
    println!("Complete");
    println!();

    return Ok(ExitCode::SUCCESS);
}

fn confirm_continue() -> Option<ExitCode> {
    let mut input: String = String::new();

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

fn get_corpus_dir() -> Result<PathBuf> {
    let corpus_dir_parent: PathBuf = if cfg!(debug_assertions) {
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

    let corpus_dir: PathBuf = corpus_dir_parent.join("corpus");
    return Ok(corpus_dir);
}

// FUTURE: Will need to be updated with typing and weights for entries
fn load_corpus(corpus_dir: &PathBuf) -> Result<Vec<String>> {
    let corpus_content: ReadDir = match fs::read_dir(corpus_dir) {
        Ok(dir) => dir,
        Err(e) => {
            let err_string = format!("Unable to open {} -- {}", corpus_dir.display(), e);
            return Err(anyhow!(err_string));
        }
    };

    let mut corpus_files: Vec<String> = Vec::new();

    for entry in corpus_content {
        let file = entry?;

        let mut path = file.path();
        if path.is_file() {
            let contents = fs::read_to_string(&mut path)?;
            corpus_files.push(contents);
        }
    }

    if corpus_files.is_empty() {
        return Err(anyhow!("No corpus entries loaded"));
    }

    return Ok(corpus_files);
}
