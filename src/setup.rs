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

use crate::{
    population::{IdSpawner, Population},
    structs::Keyboard,
};

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
// TODO: Run qwerty and dvorak controls for scoring
// TODO: Args:
// - Population size
// - Layout to rate
// - Save file to load
pub fn setup() -> Result<ExitCode> {
    // POP: Added to pop
    let seed: [u8; 32] = rand::random();
    let mut rng = SmallRng::from_seed(seed);

    // POP: Don't put corpus in population, at least for now
    let corpus_dir: PathBuf = get_corpus_dir()?;
    let corpus: Vec<String> = load_corpus(&corpus_dir)?;

    // POP: This has been added to population
    let mut id: IdSpawner = IdSpawner::new();

    // POP: Added to pop
    let mut starting_pop: Vec<Keyboard> = Vec::new();
    for _ in 0..20 {
        let mut keyboard: Keyboard = Keyboard::make_origin(id.get())?;
        keyboard.shuffle_all(&mut rng)?;
        starting_pop.push(keyboard);
    }

    // POP: Because decay is tied to iterations, it should not be something population is
    // responsible for handling
    let decay_start: f64 = 30.0;
    let small_decay_target: f64 = 2.0;
    let med_decay_target: f64 = 3.0;
    let large_decay_target: f64 = 4.0;

    for iter in 1..=500 {
        println!();
        println!("Iteration {}", iter);
        println!();

        // POP: Because decay is tied to iterations, it should not be something population is
        // responsible for handling
        let iter_decay: f64 = iter as f64 - 1.0;
        let small_decay: f64 = decay_value(decay_start, iter_decay, small_decay_target);
        let small_decay_usize: usize = small_decay as usize;
        let med_decay: f64 = decay_value(decay_start, iter_decay, med_decay_target);
        let med_decay_usize: usize = med_decay as usize;
        let large_decay: f64 = decay_value(decay_start, iter_decay, large_decay_target);
        let large_decay_usize: usize = large_decay as usize;
        // println!("Current small decay: {}", small_decay);
        // println!("Current med decay: {}", med_decay);
        // println!("Current large decay: {}", large_decay);
        // println!("Current decay: {}", decay_usize);

        // POP: cur_starting_pop not needed in fixed pop struct logic
        let cur_starting_pop = starting_pop.len();
        for i in 0..cur_starting_pop {
            // POP: This should not be in pop
            let small_amt: usize = rng.random_range(small_decay_usize..=small_decay_usize);
            let med_amt: usize = rng.random_range(small_decay_usize..=small_decay_usize);
            let large_amt: usize = rng.random_range(med_decay_usize..=med_decay_usize);
            let huge_amt: usize = rng.random_range(large_decay_usize..=large_decay_usize);

            // POP: This part is in pop
            let mut small_kb: Keyboard = Keyboard::mutate_kb(&starting_pop[i], iter, id.get());
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

        // POP: In pop
        for i in 0..starting_pop.len() {
            print!("Evaluating Keyboard {:03}\r", i + 1);
            stdout().flush()?;
            starting_pop[i].eval(&corpus);
        }

        starting_pop.sort_by(|a, b| {
            return b
                .get_score()
                .partial_cmp(&a.get_score())
                .unwrap_or(std::cmp::Ordering::Equal);
        });

        let mut selections: Vec<Keyboard> =
            starting_pop.drain(..4.min(starting_pop.len())).collect();

        let mut duplicates: Vec<usize> = Vec::new();
        // TODO: horrible hard codes
        // ALso, this is not principled logic
        for i in 0..3 {
            for j in i + 1..4 {
                if duplicates.contains(&j) {
                    continue;
                }

                let slots_i = selections[i].get_keyslots();
                let slots_j = selections[j].get_keyslots();

                let mut unique_found: bool = false;
                for k in 0..slots_i.len() {
                    let i_base = slots_i[k].get_key().get_base();
                    let j_base = slots_j[k].get_key().get_base();
                    if i_base != j_base {
                        unique_found = true;
                        break;
                    }
                }

                if unique_found {
                    continue;
                }

                duplicates.push(j);
            }
        }

        // TODO: A complication here is, it's hard to follow what sorts are doing what
        if duplicates.len() > 0 {
            duplicates.sort_by(|a, b| {
                return b.cmp(a);
            });

            for &i in duplicates.iter() {
                selections.drain(i..=i);
            }

            // TODO: This should, instead, be a probabalistic selection to promote diversity
            selections.extend(starting_pop.drain(..duplicates.len().min(starting_pop.len())));
        }

        for selection in selections.iter_mut() {
            selection.set_elite();
        }

        starting_pop.drain(starting_pop.len().saturating_sub(4)..);

        // println!();
        // println!(
        //     "Starting pop length after removing top and bottom 4: {}",
        //     starting_pop.len()
        // );

        let mut pop_score: f64 = 0.0;
        for member in &starting_pop {
            pop_score += member.get_score();
        }

        // TODO: An example of how hard codes can kill us - if we subtract more than the
        // population, we get a bad index because we're trying to work with empty starting
        // population
        // Another edge case example - Population score less than zero breaking the rng. Should not
        // happen with current hard codes, but not impossible either
        for _ in 0..16 {
            let mut winner: usize = 0;
            let mut checked_score: f64 = 0.0;
            let r: f64 = rng.random_range(0.0..=pop_score);

            for i in 0..starting_pop.len() {
                checked_score += starting_pop[i].get_score();
                if checked_score >= r {
                    winner = i;
                    break;
                }
            }

            pop_score -= starting_pop[winner].get_score();
            // TODO: Instead of drain, perhaps swap and pop
            // Is this bad though because it doesn't preserve ordering?
            selections.extend(starting_pop.drain(winner..=winner));
        }

        // println!(
        //     "Starting pop length after removing next 15: {}",
        //     starting_pop.len()
        // );
        // println!("Selection count: {}", selections.len());

        // TODO: Right now this is just for aesthetic purposes
        selections.sort_by(|a, b| {
            return b
                .get_score()
                .partial_cmp(&a.get_score())
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
            selections[i] = hill_climb(&mut rng, &selections[i], &corpus, iter)?;
        }

        starting_pop.clear();
        for selection in &selections {
            // TODO: probably not the final form of the code, but clone here still bad
            starting_pop.push(selection.clone());
        }

        // println!("Starting pop after climbing: {}", starting_pop.len());
    }

    starting_pop.sort_by(|a, b| {
        return b
            .get_score()
            .partial_cmp(&a.get_score())
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
fn hill_climb(
    rng: &mut SmallRng,
    keyboard: &Keyboard,
    corpus: &[String],
    iter: usize,
) -> Result<Keyboard> {
    // const DECAY_FACTOR: f64 = 0.999;
    let mut decay_factor: f64 = 1.0 - (1.0 / iter as f64);
    // TODO: This should be a hard code
    let clamp_value: f64 = 1.0 - (2.0_f64).powf(-53.0);
    decay_factor = decay_factor.min(clamp_value);
    if keyboard.is_elite() {
        decay_factor *= decay_factor.powf(3.0);
    }
    println!("Climb Decay: {}", decay_factor);

    if keyboard.is_elite() {
        let r: f64 = rng.random_range(0.0..=1.0);
        if r >= decay_factor {
            return Ok(keyboard.clone());
        }
    }

    const MAX_ITER_WITHOUT_IMPROVEMENT: usize = 90;

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
    for i in 1..=10000 {
        let kb_score: f64 = kb.get_score();

        // Doing steps of one change works best. If you change two keys, the algorithm will find
        // bigger changes less frequently. This causes the decay to continue four about as many
        // iterations as it would if doing only one step, but fewer improvements will be found,
        // causing the improvement at the end of the hill climbing step to be lower
        let mut climb_kb: Keyboard = Keyboard::copy_kb(&kb);
        climb_kb.shuffle_some(rng, 1)?;
        climb_kb.eval(corpus);
        let climb_kb_score: f64 = climb_kb.get_score();

        let this_change = climb_kb_score - kb_score;
        let this_improvement: f64 = (this_change).max(0.0);

        avg = get_new_avg(this_improvement, avg, i);

        let delta = this_improvement - last_improvement;
        last_improvement = this_improvement;
        let weight: f64 = get_weight(delta, kb.is_elite());

        sum_weights *= decay_factor;
        let weighted_avg_for_new: f64 = weighted_avg * sum_weights;
        sum_weights += weight;
        weighted_avg = (weighted_avg_for_new + this_improvement * weight) / sum_weights;

        // TODO: Debug only
        print!(
            "Iter: {} -- Start: {} -- Cur: {} -- Best: {} -- Avg: {} -- Weighted: {}\r",
            i, start, climb_kb_score, kb_score, avg, weighted_avg
        );
        stdout().flush()?;

        if climb_kb_score > kb_score {
            kb = climb_kb;
        }

        // NOTE: An edge case can occur where, if the first improvement is on the first iteration,
        // the weighted average can be smaller than the unweighted due to floating point
        // imprecision
        // We get around this with an iteration minimum, but it does paste over the underlying
        // issue
        // TODO: Is there a better solution?
        let plateauing: bool = weighted_avg < avg && i > 1;
        let not_starting: bool = avg <= 0.0 && i >= MAX_ITER_WITHOUT_IMPROVEMENT;
        if plateauing || not_starting {
            break;
        }
    }

    // TODO: For debugging
    println!();
    if kb.is_elite() {
        kb.display_keyboard();
    }

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
        // return 1.0 + K * delta.powf(0.3); // Even less scaling for positive values
        return 1.0 + K * delta.powf(0.0001); // Even less scaling for positive values
    }

    return 1.0 + K * delta.sqrt();
}

fn decay_value(start: f64, iter: f64, target: f64) -> f64 {
    const K: f64 = 0.1;
    let z = target + (start - target) * (-K * iter).exp();
    return z.max(target); // Clamp to ensure z >= z_min
}
