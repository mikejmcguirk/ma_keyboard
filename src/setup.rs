// use std::{fs::File, process::ExitCode};
use std::{
    env,
    fs::{self, File, OpenOptions, ReadDir},
    io::{self, Read},
    path::{Path, PathBuf},
    process::ExitCode,
};

use {
    anyhow::{Result, anyhow},
    rand::{Rng as _, SeedableRng as _, rngs::SmallRng},
};

use crate::structs::Keyboard;

// TODO: Come up with something to log so we can use the actual function signature/imports
// pub fn setup(handle: &mut File) -> Result<ExitCode> {
pub fn setup() -> Result<ExitCode> {
    let mut keyboard: Keyboard = Keyboard::new()?;
    keyboard.shuffle_all();
    // keyboard.print_keyslots();
    // keyboard.shuffle_some(2)?;
    // keyboard.print_keyslots();

    let corpus_dir: PathBuf = get_corpus_dir()?;
    validate_corpus_dir(&corpus_dir)?;

    let mut total_efficiency: f64 = 0.0;

    let corpus_content: ReadDir = fs::read_dir(corpus_dir)?;
    let corpus_entries: Vec<_> = corpus_content.collect::<io::Result<_>>()?;
    for file_enum in &corpus_entries {
        // Not sure what this is for now
        let file = file_enum;
        let path = file.path();

        if path.is_file() {
            let mut opened_file = File::open(&path)?;

            let mut contents = String::new();

            opened_file.read_to_string(&mut contents)?;

            // println!("{}", &contents[..20]);

            keyboard.clear_last_slot();
            for c in contents.chars() {
                total_efficiency += keyboard.get_efficiency(c);
            }
        }
    }

    println!("Starting total efficiency: {}", total_efficiency);
    let mut climb_attempts: usize = 1;
    let mut total_improvement: f64 = 0.0;
    let mut weighted_avg_rate: f64 = 0.0;
    let mut sum_weights: f64 = 0.0;
    let mut last_improvement: f64 = 0.0;
    let mut avg_improvement: f64 = 0.0;

    loop {
        let mut climb_kb: Keyboard = keyboard.clone();

        let seed: [u8; 32] = rand::random();
        let mut rng = SmallRng::from_seed(seed);
        let climb_amt: usize = rng.random_range(1..=2);
        climb_kb.shuffle_some(climb_amt)?;

        let mut new_efficiency: f64 = 0.0;

        for file_enum in &corpus_entries {
            let file = file_enum;
            let path = file.path();

            if path.is_file() {
                let mut opened_file = File::open(&path)?;

                let mut contents = String::new();

                opened_file.read_to_string(&mut contents)?;

                // println!("{}", &contents[..20]);

                keyboard.clear_last_slot();
                for c in contents.chars() {
                    new_efficiency += climb_kb.get_efficiency(c);
                }
            }
        }

        let this_change: f64 = total_efficiency - new_efficiency;
        let this_improvement: f64 = this_change.max(0.0);
        total_improvement += this_improvement;

        let this_improvement_for_avg: f64 = this_improvement / (climb_attempts as f64);
        let past_avg: f64 =
            avg_improvement * ((climb_attempts as f64 - 1.0) / climb_attempts as f64);
        avg_improvement = this_improvement_for_avg + past_avg;

        const K: f64 = 0.01;
        let delta = this_improvement - last_improvement;
        let weight: f64 = if delta > 0.0 {
            // 1.0 + K * delta.ln()
            1.0 + K * delta.sqrt()
        } else {
            1.0
        };

        const DECAY_FACTOR: f64 = 0.99;
        sum_weights *= DECAY_FACTOR;
        sum_weights += weight;
        weighted_avg_rate =
            (weighted_avg_rate * (sum_weights - weight) + this_improvement * weight) / sum_weights;

        last_improvement = this_improvement;

        println!(
            "Iter: {} -- Cur: {} -- Best: {} -- Avg: {} -- Weighted: {}",
            climb_attempts, new_efficiency, total_efficiency, avg_improvement, weighted_avg_rate
        );

        if new_efficiency < total_efficiency {
            total_efficiency = new_efficiency;
            keyboard = climb_kb.clone();
        }

        let slow_improver: bool = weighted_avg_rate < avg_improvement;
        const MAX_WITHOUT_IMPROVEMENT: usize = 50;
        let not_starting: bool =
            avg_improvement <= 0.0 && climb_attempts >= MAX_WITHOUT_IMPROVEMENT;

        // if slow_improver || not_starting {
        //     break;
        // }

        if climb_attempts >= 800 {
            break;
        }

        climb_attempts += 1;
    }

    println!("{}", total_efficiency);
    keyboard.display_keyboard();

    // TODO: Now that we have a basic keyboard, we need to read the corpus
    //
    // NOTE: Don't check clippy during these steps
    //
    // Then we can add population management. First do it by just creating random keyboards, then
    // add the lambda calcs/logic for different amounts of keys
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

// TODO: This is kind of unclear. It works in the sense of, if we can't do this then we can't get
// into the file system, but why tho
fn validate_corpus_dir(corpus_dir: &PathBuf) -> io::Result<()> {
    return fs::create_dir_all(corpus_dir);
}
