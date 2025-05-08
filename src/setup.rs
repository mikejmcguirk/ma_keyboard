use std::{
    env,
    fs::{self, File, ReadDir},
    io::Read,
    path::PathBuf,
    process::ExitCode,
};

use anyhow::{Result, anyhow};

use crate::population::Population;

// TODO: Will have to make a decision on how to do multi-threaded RNG. Single resource so I can
// re-use the seed? Or multiple RNGs for performance? Also, do we put SmallRng in a refcell or use
// threadRNG? Issue with threadRNG is - it's the slower version from what I understand
// TODO: Keeping the setup naming for now. At some point we're going to add arg processing and then
// it would make more sense to do that here and then break out actually running the training in its
// own file
// TODO: Run qwerty and dvorak controls for scoring
// TODO: Args:
// TODO: write seed to log not error
// - Population size
// - Layout to rate
// - Save file to load
// - Read from config file
// - The input options will have restrictions on what is possible. Should be possible to print them
// - Amount of elites
// - Amount to cull
pub fn setup(log_handle: &mut File) -> Result<ExitCode> {
    const ITERATIONS: usize = 500;

    let corpus_dir: PathBuf = get_corpus_dir()?;
    let corpus: Vec<String> = load_corpus(&corpus_dir)?;

    // let mut qwerty: Keyboard = Keyboard::make_qwerty(0);
    // qwerty.eval(&corpus);
    // qwerty.display_keyboard();
    // println!("Qwerty score: {}", qwerty.get_score());

    let mut population: Population = Population::create(None, &corpus, log_handle)?;

    let decay_start: f64 = 30.0;
    let small_decay_target: f64 = 2.0;
    let med_decay_target: f64 = 3.0;
    let large_decay_target: f64 = 4.0;

    for iter in 1..=ITERATIONS {
        println!();
        println!("Iteration {}", iter);
        println!();

        let iter_decay: f64 = iter as f64 - 1.0;
        let small_decay: f64 = decay_value(decay_start, iter_decay, small_decay_target);
        let small_decay_usize: usize = small_decay as usize;
        let med_decay: f64 = decay_value(decay_start, iter_decay, med_decay_target);
        let med_decay_usize: usize = med_decay as usize;
        let large_decay: f64 = decay_value(decay_start, iter_decay, large_decay_target);
        let large_decay_usize: usize = large_decay as usize;

        population.mutate_climbers([
            small_decay_usize,
            small_decay_usize,
            med_decay_usize,
            large_decay_usize,
        ])?;

        population.eval_gen_pop(&corpus)?;
        population.setup_climbers()?;
        population.climb_kbs(&corpus, iter)?;
    }

    // println!("Qwerty score: {}", qwerty.get_score());
    population.print_results();

    return Ok(ExitCode::SUCCESS);
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

// TODO: Will need to be updated with typing and weights for entries
// TODO: Do we just consume corpus_dir here?
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

        let path = file.path();
        if path.is_file() {
            let mut opened_file = File::open(&path)?;

            let mut contents = String::new();
            opened_file.read_to_string(&mut contents)?;
            corpus_files.push(contents);
        }
    }

    if corpus_files.len() < 1 {
        return Err(anyhow!("No corpus entries loaded"));
    } else {
        return Ok(corpus_files);
    }
}

fn decay_value(start: f64, iter: f64, target: f64) -> f64 {
    const K: f64 = 0.1;
    let z = target + (start - target) * (-K * iter).exp();
    return z.max(target); // Clamp to ensure z >= z_min
}
