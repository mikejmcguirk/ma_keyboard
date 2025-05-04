// use std::{fs::File, process::ExitCode};
use std::{
    env,
    fs::{self, File, ReadDir},
    // fs::{self, File, OpenOptions, ReadDir},
    // io::{self, Read, Write as _, stdout},
    io::{Read, Write as _, stdout},
    // path::{Path, PathBuf},
    path::PathBuf,
    process::ExitCode,
};

use {
    anyhow::{Result, anyhow},
    rand::{Rng as _, SeedableRng as _, rngs::SmallRng},
};

use crate::structs::{IdSpawner, Keyboard};

// TODO: Come up with something to log so we can use the actual function signature/imports
// pub fn setup(handle: &mut File) -> Result<ExitCode> {
// TODO: Because we want to have the option to just clone keyboards, we need a globally available
// RNG. If you clone the keyboard, you need to manually advance the RNG state of the original
// keyboard or else you get the same random rolls each time. This is complicated and it is
// wasteful. For the multi-threaded case, we would want to have each thread use its own RNG. This
// is both faster (no holding threads for RNG) and less complex (no resource sharing). It seems
// like this can be done either with ThreadRng or by using thread_local to put SmallRng into a ref
// cell. Will work through specifics as we get there
// TODO: Because we can make keyboards "from_kb", it is possible to re-seed rng on the new keyboard
// when creating it, which reduces the argument for global or thread local rng. In theory,
// re-seeding is a cost, but it would make rng state easier to track and potentially less
// convoluted than a thread local rng
pub fn setup() -> Result<ExitCode> {
    let seed: [u8; 32] = rand::random();
    let mut rng = SmallRng::from_seed(seed);

    let corpus_dir: PathBuf = get_corpus_dir()?;
    let corpus: Vec<String> = load_corpus(&corpus_dir)?;

    let mut id: IdSpawner = IdSpawner::new();

    let mut starting_pop: Vec<Keyboard> = Vec::new();
    for _ in 0..20 {
        let mut keyboard: Keyboard = Keyboard::make_origin(id.get())?;
        // TODO: This should be a part of make_origin. But I want to let the architecture of how we
        // restrict key movements settle in more before doing this. The obvious answer to how this
        // is done is to place the keys into random slots from the beginning, but I don't know what
        // the final key/slot logic will be yet
        keyboard.shuffle_all(&mut rng)?;
        starting_pop.push(keyboard);
    }

    println!("Initial starting pop length: {}", starting_pop.len());

    // TODO: Right now, all of the logic for how the population is managed is based on hard coded
    // assumptions. It should be possible to change the populaion size and have the reproduction of
    // the keyboards algorithmically flow from that
    for iter in 1..=20 {
        println!();
        println!("Iteration {}", iter);
        println!();

        // TODO: Test if len updates as the loop runs. Storing it separately is extra
        let cur_starting_pop = starting_pop.len();
        for i in 0..cur_starting_pop {
            // TODO: These should all be lambda ranges
            let small_amt: usize = rng.random_range(3..=9);
            let med_amt: usize = rng.random_range(10..=16);
            let large_amt: usize = rng.random_range(17..=23);
            let huge_amt: usize = rng.random_range(24..=30);

            let mut small_kb: Keyboard = Keyboard::mutate_kb(&starting_pop[i], iter, id.get());
            // TODO: Similar to the origin keyboard, this should also be integrated into the mutate
            // function, but holding off for the same reasons
            small_kb.shuffle_some(&mut rng, small_amt)?;
            starting_pop.push(small_kb);

            let mut med_kb: Keyboard = Keyboard::mutate_kb(&starting_pop[i], iter, id.get());
            med_kb.shuffle_some(&mut rng, med_amt)?;
            starting_pop.push(med_kb);

            let mut large_kb: Keyboard = Keyboard::mutate_kb(&starting_pop[i], iter, id.get());
            large_kb.shuffle_some(&mut rng, large_amt)?;
            starting_pop.push(large_kb);

            let mut huge_kb: Keyboard = Keyboard::mutate_kb(&starting_pop[i], iter, id.get());
            huge_kb.shuffle_some(&mut rng, huge_amt)?;
            starting_pop.push(huge_kb);
        }

        println!(
            "Starting pop length after new additions: {}",
            starting_pop.len()
        );

        println!();

        for i in 0..starting_pop.len() {
            print!("Evaluating Keyboard {:03}\r", i + 1);
            stdout().flush()?;

            // TODO: add_generation should be tied to some other thing
            starting_pop[i].evaluate(&corpus)?;
        }

        println!();

        starting_pop.sort_by(|a, b| {
            return a
                .get_score()
                .partial_cmp(&b.get_score())
                .unwrap_or(std::cmp::Ordering::Equal);
        });

        let mut selections: Vec<Keyboard> =
            starting_pop.drain(..4.min(starting_pop.len())).collect();

        for selection in selections.iter_mut() {
            selection.set_elite();
        }

        starting_pop.drain(starting_pop.len().saturating_sub(4)..);

        println!();
        println!(
            "Starting pop length after removing top and bottom 4: {}",
            starting_pop.len()
        );

        for _ in 0..16 {
            // TODO: The idea here is, if we pull 19, we're basically guaranteeing getting the top
            // quintille. You could improve this by dividing the remaining population by 5 each
            // iteration in order to get a more accurate tournament size. But I'm still not sure how
            // much I like this given that tournaments are usually smaller
            // We should also have a check to make sure the tournament size is not bigger than the
            // remaining starting population
            let tournament_size: usize = rng.random_range(2..=18);
            let mut found_idx: Vec<usize> = Vec::new();
            loop {
                let this_idx: usize = rng.random_range(0..starting_pop.len());
                if found_idx.contains(&this_idx) {
                    continue;
                }

                found_idx.push(this_idx);

                if found_idx.len() == tournament_size {
                    break;
                }
            }

            let winner: usize = match found_idx.iter().min() {
                Some(min) => *min,
                None => {
                    return Err(anyhow!("Tournament vector is empy"));
                }
            };

            selections.extend(starting_pop.drain(winner..=winner));
        }

        println!(
            "Starting pop length after removing next 15: {}",
            starting_pop.len()
        );
        println!("Selection count: {}", selections.len());

        // TODO: Right now this is just for aesthetic purposes
        selections.sort_by(|a, b| {
            return a
                .get_score()
                .partial_cmp(&b.get_score())
                .unwrap_or(std::cmp::Ordering::Equal);
        });

        // TODO: Super nasty hard code
        for i in 4..20 {
            selections[i].unset_elite();
        }

        let mut selection_score: f64 = 0.0;
        for selection in &selections {
            selection_score += selection.get_score();
        }

        let avg_selection_score: f64 = selection_score / 20.0;
        println!("Avg Selection Score: {}", avg_selection_score);

        for i in 0..20 {
            println!();
            println!("Climbing Keyboard {}", i + 1,);
            println!(
                "Gen {}, Id {}, Lineage: {}",
                selections[i].get_generation(),
                selections[i].get_id(),
                selections[i].get_lineage()
            );
            selections[i] = hill_climb(&mut rng, &selections[i], &corpus)?;
        }

        starting_pop.clear();
        for selection in &selections {
            // TODO: probably not the final form of the code, but clone here still bad
            starting_pop.push(selection.clone());
        }

        println!("Starting pop after climbing: {}", starting_pop.len());
    }

    starting_pop.sort_by(|a, b| {
        return a
            .get_score()
            .partial_cmp(&b.get_score())
            .unwrap_or(std::cmp::Ordering::Equal);
    });

    println!();

    for i in 0..20 {
        println!("Results: Keyboard {}", i + 1);
        println!(
            "Gen {}, Id {}, Lineage: {}",
            starting_pop[i].get_generation(),
            starting_pop[i].get_id(),
            starting_pop[i].get_lineage()
        );
        println!("Score: {}", starting_pop[i].get_score());
        println!("Layout:");
        starting_pop[i].display_keyboard();
        println!();
    }

    // NOTE: Don't check clippy during these steps
    //
    // Refactor the population management logic
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

// TODO: Function too long
// You could, in theory, have keyboard do this as a method on itself, but having that object store
// its own state + a hypothetical future state feels contrived
fn hill_climb(rng: &mut SmallRng, keyboard: &Keyboard, corpus: &[String]) -> Result<Keyboard> {
    const DECAY_FACTOR: f64 = 0.99;
    const MAX_ITER_WITHOUT_IMPROVEMENT: usize = 60;

    // TODO: I'm not sure if this is actually better than cloning, though the intention is more
    // explicit
    let mut kb: Keyboard = Keyboard::copy_kb(keyboard);
    let start: f64 = kb.get_score();

    let mut last_improvement: f64 = 0.0;
    let mut avg: f64 = 0.0;
    let mut weighted_avg: f64 = 0.0;
    let mut sum_weights: f64 = 0.0;

    // For debugging
    // println!();

    // One indexed for averaging math and display
    for i in 1..=1000 {
        let kb_score: f64 = kb.get_score();

        // Doing steps of one change works best. If you change two keys, the algorithm will find
        // bigger changes less frequently. This causes the decay to continue four about as many
        // iterations as it would if doing only one step, but fewer improvements will be found,
        // causing the improvement at the end of the hill climbing step to be lower
        let mut climb_kb: Keyboard = Keyboard::copy_kb(&kb);
        climb_kb.shuffle_some(rng, 1)?;
        climb_kb.evaluate(corpus)?;
        let climb_kb_score: f64 = climb_kb.get_score();

        let this_change = kb_score - climb_kb_score;
        let this_improvement: f64 = (this_change).max(0.0);

        avg = get_new_avg(this_improvement, avg, i);

        let delta = this_improvement - last_improvement;
        last_improvement = this_improvement;
        let weight: f64 = get_weight(delta, kb.is_elite());

        sum_weights *= DECAY_FACTOR;
        let weighted_avg_for_new: f64 = weighted_avg * sum_weights;
        sum_weights += weight;
        weighted_avg = (weighted_avg_for_new + this_improvement * weight) / sum_weights;

        // TODO: Debug only
        print!(
            "Iter: {} -- Start: {} -- Cur: {} -- Best: {} -- Avg: {} -- Weighted: {}\r",
            i, start, climb_kb_score, kb_score, avg, weighted_avg
        );
        stdout().flush()?;

        if climb_kb_score < kb_score {
            kb = climb_kb;
        }

        // NOTE: An edge case can occur where, if the first improvement is on the first iteration,
        // the weighted average can be smaller than the unweighted due to floating point
        // imprecision
        // We get around this by doing a minimum of 20 iterations, but it does paste over the
        // underlying issue
        // TODO: Is there a better solution?
        let plateauing: bool = weighted_avg < avg && i > 20;
        let not_starting: bool = avg <= 0.0 && i >= MAX_ITER_WITHOUT_IMPROVEMENT;
        if plateauing || not_starting {
            break;
        }
    }

    // TODO: For debugging
    println!();

    return Ok(kb);
}

fn get_new_avg(new_value: f64, old_avg: f64, new_count: usize) -> f64 {
    let new_value_for_new_avg: f64 = new_value / (new_count as f64);
    let old_avg_for_new_avg: f64 = old_avg * ((new_count as f64 - 1.0) / new_count as f64);

    return new_value_for_new_avg + old_avg_for_new_avg;
}

fn get_weight(delta: f64, is_old: bool) -> f64 {
    const K: f64 = 0.01;

    if delta <= 0.0 {
        return 1.0;
    }

    if is_old {
        // return 1.0 + K * delta.ln(); // Less scaling for positive values
        return 1.0 + K * delta.powf(0.3); // Even less scaling for positive values
    }

    return 1.0 + K * delta.sqrt();
}
