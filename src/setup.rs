use std::{
    env,
    fs::{self, File, ReadDir},
    path::PathBuf,
    process::ExitCode,
};

use {
    anyhow::{Result, anyhow},
    rand::{Rng as _, SeedableRng as _, rngs::SmallRng},
};

use crate::{display::Display, population::Population};

// TODO: The second rng is quite bad
// TODO: A better architecture for this is to let the user bring in the valid keys from a config
// file rather than actually altering the source code. So then error propagation would be the
// better design
// TODO: I think display being globally available context acutally makes sense, but not totally
// sure how to do it
// TODO: THe mutation amounts need to go back to ranges, given that we have further segmented out
// the hill climbing as its own thing and we have reduced to one elite
// TODO: Create qwerty and dvorak controls
// TODO: Will have to make a decision on how to do multi-threaded RNG. Single resource so I can
// re-use the seed? Or multiple RNGs for performance? Also, do we put SmallRng in a refcell or use
// threadRNG? Issue with threadRNG is - it's the slower version from what I understand
// TODO: Keeping the setup naming for now. At some point we're going to add arg processing and then
// it would make more sense to do that here and then break out actually running the training in its
// own file
// TODO: Run qwerty and dvorak controls for scoring
// TODO: Args:
// TODO: write seed to log not error
// TODO: The usize conversions on the decays are still bad
// - Population size
// - Layout to rate
// - Save file to load
// - Read from config file
// - The input options will have restrictions on what is possible. Should be possible to print them
// - Amount of elites
// - Amount to cull
pub fn setup(log_handle: &mut File) -> Result<ExitCode> {
    const ITERATIONS: usize = 100;

    let seed: [u8; 32] = rand::random();
    // let seed_string: String = format!("{seed:?}");
    // write_err(log_handle, &seed_string)?;
    let mut rng = SmallRng::from_seed(seed);

    let corpus_dir = get_corpus_dir()?;
    let corpus = load_corpus(&corpus_dir)?;

    let mut population = Population::create(None, log_handle)?;

    let mut display = Display::new();
    display.draw_initial(&population);

    let decay_start: f64 = 30.0;

    let small_value_target: f64 = 2.0;
    let med_range_bot_target: f64 = 3.0;
    let med_range_top_target: f64 = 12.0;
    let large_range_bot_target: f64 = 13.0;
    let large_range_top_target: f64 = 21.0;
    let huge_range_bot_target: f64 = 22.0;
    let huge_top_value: f64 = 30.0;

    // TODO: You can wrap these up in debug_asserts to make sure they don't get wrong, but we have
    // to assume the usize and f64 values are small enough here to work
    #[expect(clippy::as_conversions)]
    #[expect(clippy::cast_possible_truncation)]
    #[expect(clippy::cast_precision_loss)]
    #[expect(clippy::cast_sign_loss)]
    for iter in 1..=ITERATIONS {
        display.update_iter(iter);
        let iter_decay: f64 = iter as f64 - 1.0;

        let small_value = decay_value(decay_start, iter_decay, small_value_target);
        let med_bot_value = decay_value(decay_start, iter_decay, med_range_bot_target);
        let med_top_value = decay_value(decay_start, iter_decay, med_range_top_target);
        let large_bot_value = decay_value(decay_start, iter_decay, large_range_bot_target);
        let large_top_value = decay_value(decay_start, iter_decay, large_range_top_target);
        let huge_bot_value = decay_value(decay_start, iter_decay, huge_range_bot_target);

        let small_value_usize: usize = small_value.round() as usize;
        let med_bot_usize: usize = med_bot_value.round() as usize;
        let med_top_usize: usize = med_top_value.round() as usize;
        let large_bot_usize: usize = large_bot_value.round() as usize;
        let large_top_usize: usize = large_top_value.round() as usize;
        let huge_bot_usize: usize = huge_bot_value.round() as usize;
        let huge_top_usize: usize = huge_top_value.round() as usize;

        let med_value = rng.random_range(med_bot_usize..=med_top_usize);
        let large_value = rng.random_range(large_bot_usize..=large_top_usize);
        let huge_value = rng.random_range(huge_bot_usize..=huge_top_usize);

        display.update_mut_values(
            small_value_usize,
            small_value_usize,
            med_bot_usize,
            med_top_usize,
            large_bot_usize,
            large_top_usize,
            huge_bot_usize,
            huge_top_usize,
        );

        population.mutate_climbers([small_value_usize, med_value, large_value, huge_value]);

        population.eval_gen_pop(&corpus, &mut display)?;
        population.setup_climbers(&mut display)?;
        population.climb_kbs(&corpus, iter, &mut display)?;
    }

    // TODO: use display to tell the user the program's complete

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

fn decay_value(start: f64, iter: f64, target: f64) -> f64 {
    const K: f64 = 0.1;
    let z = target + (start - target) * (-K * iter).exp();
    return z.max(target); // Clamp to ensure z >= z_min
}
