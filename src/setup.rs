// use std::{fs::File, process::ExitCode};
use std::{
    env,
    fs::{self, File, ReadDir},
    // fs::{self, File, OpenOptions, ReadDir},
    io::{self, Read},
    // path::{Path, PathBuf},
    path::PathBuf,
    process::ExitCode,
};

use {
    anyhow::{Result, anyhow},
    rand::{Rng as _, SeedableRng as _, rngs::SmallRng},
};

use crate::structs::Keyboard;

// TODO: Come up with something to log so we can use the actual function signature/imports
// pub fn setup(handle: &mut File) -> Result<ExitCode> {
// TODO: Because we want to have the option to just clone keyboards, we need a globally available
// RNG. If you clone the keyboard, you need to manually advance the RNG state of the original
// keyboard or else you get the same random rolls each time. This is complicated and it is
// wasteful. For the multi-threaded case, we would want to have each thread use its own RNG. This
// is both faster (no holding threads for RNG) and less complex (no resource sharing). It seems
// like this can be done either with ThreadRng or by using thread_local to put SmallRng into a ref
// cell. Will work through specifics as we get there
pub fn setup() -> Result<ExitCode> {
    let seed: [u8; 32] = rand::random();
    let mut rng = SmallRng::from_seed(seed);

    let corpus_dir: PathBuf = get_corpus_dir()?;
    // validate_corpus_dir(&corpus_dir)?;
    let corpus: Vec<String> = load_corpus(&corpus_dir)?;

    let mut keyboard: Keyboard = Keyboard::new()?;
    keyboard.shuffle_all(&mut rng)?;

    keyboard.evaluate(&corpus)?;
    println!("Starting total efficiency: {}", keyboard.get_score());
    keyboard = climb_kb(&mut rng, keyboard, &corpus)?;

    println!("Final Score: {}", keyboard.get_score());
    keyboard.display_keyboard();

    // NOTE: Don't check clippy during these steps
    //
    // Then we can add population management. First do it by just creating random keyboards, then
    // add the lambda calcs/logic for different amounts of keys. Or in different terms, first just
    // be able to hill climb all the keyboards, then worry about looping through multiple
    // generations
    //
    // Add a display. Should show enough stats that you can see the progression. A visual chart
    // like the ones I've seen in the training videos would be nice but is a stretch goal. It
    // should also show the layout of the highest scoring keyboard. The biggest thing with the
    // stats is being able to intuit the amount of convergence
    //
    // We also need a method to save and load data. Including a way to press a key in the middle of
    // the program to finish the current iteration and save. It would be helpful to be able to scan
    // a directory and be able to find the best keyboard in one of the save files, but I'm not sure
    // if that's a Rust thing or a Python thing. The best and average each iteration should also be
    // saved so they can be visualized later.
    //
    // NOTE: Make sure save files are in gitignore
    //
    // The visualization/saving part is very feature heavy, but it lets us complete the pipeline
    // and the data will likely be helpful later for testing.
    //
    // From there the rest of the details should be able to be filled in.

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

    // let corpus_iter: Vec<_> = corpus_content.collect::<io::Result<_>>()?;
    let mut corpus_files: Vec<String> = Vec::new();

    for entry in corpus_content {
        let file = entry?;

        let path = file.path();
        if path.is_file() {
            let mut opened_file = File::open(&path)?;

            let mut contents = String::new();
            opened_file.read_to_string(&mut contents)?;
            // println!("{}", &contents[..20]);
            corpus_files.push(contents);
        }
    }

    if corpus_files.len() < 1 {
        return Err(anyhow!("No corpus entries loaded"));
    } else {
        return Ok(corpus_files);
    }
}

// You could, in theory, have keyboard do this as a method on itself, but having that object store
// its own state + a hypothetical future state feels contrived
fn climb_kb(rng: &mut SmallRng, keyboard: Keyboard, corpus: &[String]) -> Result<Keyboard> {
    const DECAY_FACTOR: f64 = 0.99;
    const MAX_ITER_WITHOUT_IMPROVEMENT: usize = 50;

    let mut kb: Keyboard = keyboard;

    let mut last_improvement: f64 = 0.0;
    let mut avg: f64 = 0.0;
    let mut weighted_avg: f64 = 0.0;
    let mut sum_weights: f64 = 0.0;

    // One indexed for averaging math and display
    for i in 1..=1000 {
        let kb_score: f64 = kb.get_score();

        let climb_amt: usize = rng.random_range(1..=2);
        let mut climb_kb: Keyboard = kb.clone();
        climb_kb.shuffle_some(rng, climb_amt)?;
        climb_kb.evaluate(corpus)?;
        let climb_kb_score: f64 = climb_kb.get_score();

        let this_change = kb_score - climb_kb_score;
        let this_improvement: f64 = (this_change).max(0.0);

        avg = get_new_avg(this_improvement, avg, i);

        let delta = this_improvement - last_improvement;
        last_improvement = this_improvement;
        let weight: f64 = get_weight(delta);

        sum_weights *= DECAY_FACTOR;
        let weighted_avg_for_new: f64 = weighted_avg * sum_weights;
        sum_weights += weight;
        weighted_avg = (weighted_avg_for_new + this_improvement * weight) / sum_weights;

        // TODO: Debug only
        println!(
            "Iter: {} -- Cur: {} -- Best: {} -- Avg: {} -- Weighted: {}",
            i, climb_kb_score, kb_score, avg, weighted_avg
        );

        if climb_kb_score < kb_score {
            kb = climb_kb;
        }

        let plateauing: bool = weighted_avg < avg;
        let not_starting: bool = avg <= 0.0 && i >= MAX_ITER_WITHOUT_IMPROVEMENT;
        if plateauing || not_starting {
            break;
        }
    }

    return Ok(kb);
}

fn get_new_avg(new_value: f64, old_avg: f64, new_count: usize) -> f64 {
    let new_value_for_new_avg: f64 = new_value / (new_count as f64);
    let old_avg_for_new_avg: f64 = old_avg * ((new_count as f64 - 1.0) / new_count as f64);

    return new_value_for_new_avg + old_avg_for_new_avg;
}

fn get_weight(delta: f64) -> f64 {
    const K: f64 = 0.01;

    return if delta > 0.0 {
        1.0 + K * delta.sqrt()
        // 1.0 + K * delta.ln() // Less scaling for positive values
    } else {
        1.0
    };
}
