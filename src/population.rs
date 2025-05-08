use std::{
    fs::File,
    io::{Write as _, stdout},
};

use {
    anyhow::{Result, anyhow},
    rand::{Rng as _, SeedableRng as _, rngs::SmallRng},
};

use crate::{custom_err::CorpusErr, key::Key, keyboard::Keyboard, utils::write_err};

const MIN_ELITES: usize = 1;

// TODO: Maybe add some sort of health check function to make sure all the keyboards keep the same
// parameters. Unsure to what extent this is a population vs keyboard problem
// TODO: When incorporating multi-threading for the keyboards, unsure if we can pass a SmallRng
// somehow, or make it global with a ref_cell, or if we should use thread_rng. From what I
// understand, thread_rng uses the cryptographically secure RNG, which is slower
// Importantly though, we want to be able to store and re-use the RNG seed, so whatever our RNG
// solution is cannot have multiple seeds
// PERF: When we get to multithreading stage, I think trying to process multiple keyboards at once
// is too much nonsense. I think we're better off handing the corpus strings off to worker threads
// or using Rayon or Tokio for parallel processing
// NOTE: This struct manages the population of keyboards, not any particular keyboard. It also is
// not concerned with to what end the population is managed for
pub struct Population {
    rng: SmallRng,
    id: IdSpawner,
    pop_size: usize,
    population: Vec<Keyboard>,
    climber_cnt: usize,
    climbers: Vec<Keyboard>,
    elite_cnt: usize,
    cull_cnt: usize,
    generation: usize,
}

impl Population {
    // TODO: Long function
    // TODO: Current code cannot robustly handle input options
    // TODO: long function signature
    pub fn create(size: Option<usize>, corpus: &[String], log_handle: &mut File) -> Result<Self> {
        const DEFAULT_POPULATION: usize = 100;
        const DEFAULT_CLIMB_PCT: f64 = 0.2;
        const MIN_CLIMBERS: f64 = 1.0;

        let seed: [u8; 32] = rand::random();
        let seed_string: String = format!("{seed:?}");
        write_err(log_handle, &seed_string)?;
        let mut rng = SmallRng::from_seed(seed);

        let mut id: IdSpawner = IdSpawner::new();

        let pop_cnt: usize = size.unwrap_or(DEFAULT_POPULATION);
        if pop_cnt == 0 {
            return Err(anyhow!("Population size cannot be zero"));
        }

        let gen_pop: Vec<Keyboard> = Vec::with_capacity(pop_cnt);
        println!("Population initialized with a size of {pop_cnt}");

        // let climb_pct: f64 = climb_pct_in.unwrap_or(DEFAULT_CLIMB_PCT);
        // if climb_pct <= 0.0 {
        //     return Err(anyhow!(
        //         "ERROR: Climb % ({:.2}%) must be greater than zero",
        //         climb_pct * 100.0
        //     ));
        // } else if climb_pct > 1.0 {
        //     return Err(anyhow!(
        //         "ERROR: Climb % ({:.2}%) cannot be greater than 100%",
        //         climb_pct * 100.0
        //     ));
        // }

        // let climbers: usize = (pop_size as f64 * climb_pct).max(1.0) as usize;
        let climber_cnt: usize = (pop_cnt as f64 * DEFAULT_CLIMB_PCT).max(MIN_CLIMBERS) as usize;
        let mut climbers: Vec<Keyboard> = Vec::with_capacity(climber_cnt);
        if climber_cnt > pop_cnt {
            return Err(anyhow!(
                "Climbers {climber_cnt} cannot be greater than total population ({pop_cnt})"
            ));
        }
        println!("Population will have {climber_cnt} climbers");

        let elite_cnt: usize = (climber_cnt as f64 * 0.2).max(MIN_ELITES as f64) as usize;
        if elite_cnt > climber_cnt {
            return Err(anyhow!(
                "Elite count ({elite_cnt}) cannot be higher than climber count ({climber_cnt})"
            ));
        }
        println!("Population will have {elite_cnt} elites");

        let cull_cnt: usize = (pop_cnt as f64 * 0.04).max(1.0) as usize;
        if cull_cnt + elite_cnt > pop_cnt {
            return Err(anyhow!(
                "ERROR: Elites ({}) and amount to cull ({}) cannot be greater than the total ({})",
                elite_cnt,
                cull_cnt,
                pop_cnt
            ));
        }

        if pop_cnt - cull_cnt < climber_cnt {
            return Err(anyhow!(
                "Population ({}) less group to cull ({}) is less than climber count ({})",
                pop_cnt,
                cull_cnt,
                climber_cnt
            ));
        }

        println!("The bottom {cull_cnt} keyboards will be eliminated each iteration");

        // At the end of the last iteration, it is not necessary to mutate the climbers. Therefore,
        // mutating climbers is done at the beginning of each iteration. Even though creating our
        // initial population as climbers rather than general population is unintuitive, it lets us
        // transition into the main loop logic without creating a special case for the first
        // iteration
        // TODO: The hard-coded shuffle value needs a place to live
        for _ in 0..climber_cnt {
            let mut keyboard: Keyboard = Keyboard::create_origin(id.get());
            keyboard.shuffle(&mut rng, 30);
            // For probabalistic selection when they are mutated to fill out the population. The
            // keyboards are able to store if they have evaluated since they were last changed
            keyboard.eval(corpus);
            climbers.push(keyboard);
        }

        return Ok(Self {
            rng,
            id,
            pop_size: pop_cnt,
            population: gen_pop,
            climber_cnt,
            climbers,
            elite_cnt,
            cull_cnt,
            generation: 0,
        });
    }

    // NOTE: The amts argument is used to determine how many keyslots are shuffled on the keyboard.
    // Because the amts are type usize, no amt can produce invalid behavior from the standpoint of
    // the population. To see if certain amts are invalid, the keyboard struct must be checked
    // NOTE: An error is returned if climbers is zero because this should never be able to happen
    // NOTE: Probabilistically selecting which climber to mutate based on score tips the scales
    // toward the higher scoring keyboards. This is a trade-off aimed at increasing the value of
    // the mutation phase in later generations as the population converges on an optimal solution.
    // This creates the downside of hurting population diversity in the earlier stages, though the
    // amount of keys to shuffle can be increased to compensate
    // TODO: When incrementing generations, should return an error if max usize is exceeded
    // The caller can then do what it wants with that. The mutate function should not clear the
    // climbers
    // TODO: Contains a clone
    // TODO: Format string in keyboard evaluation should be based on digits in total population
    // size
    // PERF: We generate a random starting selection so the edge case doesn't always default to the
    // strongest member. Might be extra
    // PERF: For simplicity, we are currently using push/drain/clear on the various Vecs to manage
    // their contents. If this is slow, move to simply reading and writing to it directly. This
    // *should* be possible without unsafe
    pub fn mutate_climbers(&mut self, amts: [usize; 4]) -> Result<()> {
        self.generation += 1;

        if self.climbers.len() == 0 {
            return Err(anyhow!("ERROR: No climbers to mutate!"));
        }

        self.population.clear();
        let mut climber_score: f64 = 0.0;
        for climber in &self.climbers {
            climber_score += climber.get_score();
            self.population.push(climber.clone());
        }

        for _ in 0..(self.pop_size - self.climbers.len()) {
            let mut idx: usize = self.rng.random_range(0..self.climbers.len());
            let mut checked_score: f64 = 0.0;
            let r: f64 = self.rng.random_range(0.0..=climber_score);

            for i in 0..self.climbers.len() {
                checked_score += self.climbers[i].get_score();
                if checked_score >= r {
                    idx = i;
                    break;
                }
            }

            let this_amt_idx: usize = self.rng.random_range(0..amts.len());
            let this_amt: usize = amts[this_amt_idx];
            let mut new_kb: Keyboard =
                Keyboard::mutate_from(&self.climbers[idx], self.generation, self.id.get());
            new_kb.shuffle(&mut self.rng, this_amt);

            self.population.push(new_kb);
        }

        if self.population.len() != self.pop_size {
            return Err(anyhow!(
                "ERROR: Total population of {} does not match expected {}",
                self.population.len(),
                self.pop_size
            ));
        }

        return Ok(());
    }

    pub fn eval_gen_pop(&mut self, corpus: &[String]) -> Result<(), CorpusErr> {
        if corpus.is_empty() {
            return Err(CorpusErr::EmptyCorpus);
        }

        for i in 0..self.population.len() {
            print!("Evaluating Keyboard {:03}\r", i + 1);
            stdout().flush()?; // MyError handles io errors
            self.population[i].eval(corpus);
        }

        println!();

        return Ok(());
    }

    // TODO: Re-arranging the population Vec is probably not the best way to do this
    // TODO: Long function
    // TODO: The saturating sub is extra
    // TODO: The score averages are only useful for debugging
    // TODO: Duplicates are not being removed. I guess just do the manual comparison. To test, use
    // a small corpus and high iterations
    // NOTE: Removing duplicates can cause the amount of available climbers to be below what is
    // intended. This is allowed to happen without error because the population is replenished
    // during the mutation phase
    // NOTE: This method assumes that the amount of elites, culls, and climbers is properly setup
    pub fn setup_climbers(&mut self) -> Result<()> {
        self.population.sort_by(|a, b| {
            return b
                .get_score()
                .partial_cmp(&a.get_score())
                .unwrap_or(std::cmp::Ordering::Equal);
        });

        self.population
            .drain(self.population.len().saturating_sub(self.cull_cnt)..);

        self.climbers.clear();
        // Add the first elite
        self.climbers.extend(self.population.drain(..1));

        // Add elites deterministically
        let mut dup_elites: usize = 0;
        for _ in 0..self.elite_cnt - 1 {
            let mut diff_found: bool = false;
            let candidate_vec: &[Vec<Key>] = self.population[0].get_vec_ref();
            let flat_candidate: Vec<&Key> = candidate_vec.iter().flatten().collect();

            for climber in &self.climbers {
                let climber_vec: &[Vec<Key>] = climber.get_vec_ref();
                let flat_climber: Vec<&Key> = climber_vec.iter().flatten().collect();

                debug_assert_eq!(flat_candidate.len(), flat_climber.len());

                for i in 0..flat_climber.len() {
                    if flat_climber[i].get_base() != flat_candidate[i].get_base() {
                        diff_found = true;
                        break;
                    }
                }
            }

            if diff_found {
                self.climbers.extend(self.population.drain(..1));
            } else {
                self.population.drain(..1);
                dup_elites += 1;
            }
        }

        // Add remaining climbers probabalistically
        let mut population_score: f64 = 0.0;
        for member in &self.population {
            population_score += member.get_score();
        }

        while self.climbers.len() < self.climber_cnt && self.population.len() > 0 {
            let mut selection: usize = 0;
            let mut checked_score: f64 = 0.0;
            let r: f64 = self.rng.random_range(0.0..=population_score);

            for i in 0..self.population.len() {
                checked_score += self.population[i].get_score();
                if checked_score >= r {
                    selection = i;
                    break;
                }
            }

            population_score -= self.population[selection].get_score();

            let mut diff_found: bool = false;
            let candidate_vec: &[Vec<Key>] = self.population[0].get_vec_ref();
            let flat_candidate: Vec<&Key> = candidate_vec.iter().flatten().collect();

            for climber in &self.climbers {
                let climber_vec: &[Vec<Key>] = climber.get_vec_ref();
                let flat_climber: Vec<&Key> = climber_vec.iter().flatten().collect();

                debug_assert_eq!(flat_candidate.len(), flat_climber.len());

                for i in 0..flat_climber.len() {
                    if flat_climber[i].get_base() != flat_candidate[i].get_base() {
                        diff_found = true;
                        break;
                    }
                }
            }

            if diff_found {
                self.climbers
                    .extend(self.population.drain(selection..=selection));
            } else {
                self.population.drain(selection..=selection);
            }
        }

        let this_elite_cnt: usize = self.elite_cnt - dup_elites;
        let mut elites_set: usize = 0;
        for climber in self.climbers.iter_mut() {
            if elites_set < this_elite_cnt {
                climber.set_elite();
                elites_set += 1;
            } else {
                climber.unset_elite();
            }
        }

        println!(
            "{} climbers containing {this_elite_cnt} elites",
            self.climbers.len()
        );

        println!("Top Score: {}", self.climbers[0].get_score());

        let mut selection_score: f64 = 0.0;
        for climber in &self.climbers {
            selection_score += climber.get_score();
        }
        let avg_selection_score = selection_score / self.climbers.len() as f64;
        println!("Average Score: {}", avg_selection_score);

        return Ok(());
    }

    pub fn climb_kbs(&mut self, corpus: &[String], iter: usize) -> Result<()> {
        for i in 0..self.climbers.len() {
            println!();
            println!("Climbing Keyboard {}", i + 1,);
            println!(
                "Gen {}, Id {}, Lineage: {}",
                self.climbers[i].get_generation(),
                self.climbers[i].get_id(),
                self.climbers[i].get_lineage()
            );

            self.climbers[i] = hill_climb(&mut self.rng, &self.climbers[i], corpus, iter)?;
        }

        return Ok(());
    }

    pub fn print_results(&mut self) {
        self.climbers.sort_by(|a, b| {
            return b
                .get_score()
                .partial_cmp(&a.get_score())
                .unwrap_or(std::cmp::Ordering::Equal);
        });

        println!();

        for i in 0..self.climbers.len() {
            println!("Results: Keyboard {}", i + 1);
            println!(
                "Gen {}, Id {}, Lineage: {}",
                self.climbers[i].get_generation(),
                self.climbers[i].get_id(),
                self.climbers[i].get_lineage()
            );
            println!("Score: {}", self.climbers[i].get_score());
            println!("Layout:");
            self.climbers[i].display_keyboard();
            println!();
        }
    }
}

pub struct IdSpawner {
    next_id: usize,
}

impl IdSpawner {
    pub fn new() -> Self {
        return Self { next_id: 0 };
    }

    // PERF: I want to return 0 as the first id but maybe this is an extravagance
    pub fn get(&mut self) -> usize {
        let to_return: usize = self.next_id;
        self.next_id += 1;

        return to_return;
    }
}

// TODO: Function too long
pub fn hill_climb(
    rng: &mut SmallRng,
    keyboard: &Keyboard,
    corpus: &[String],
    iter: usize,
) -> Result<Keyboard> {
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
            println!("Score: {}", keyboard.get_score());
            keyboard.display_keyboard();
            return Ok(keyboard.clone());
        }
    }

    const MAX_ITER_WITHOUT_IMPROVEMENT: usize = 90;

    // TODO: I'm not sure if this is actually better than cloning, though the intention is more
    // explicit
    let mut kb: Keyboard = keyboard.clone();
    let start: f64 = kb.get_score();

    let mut last_improvement: f64 = 0.0;
    let mut avg: f64 = 0.0;
    let mut weighted_avg: f64 = 0.0;
    let mut sum_weights: f64 = 0.0;

    // One indexed for averaging math and display
    for i in 1..=10000 {
        let kb_score: f64 = kb.get_score();

        // Doing steps of one change works best. If you change two keys, the algorithm will find
        // bigger changes less frequently. This causes the decay to continue for about as many
        // iterations as it would if doing only one step, but fewer improvements will be found,
        // causing the improvement at the end of the hill climbing step to be lower
        let mut climb_kb: Keyboard = kb.clone();
        climb_kb.shuffle(rng, 1);
        climb_kb.eval(corpus);
        let climb_kb_score: f64 = climb_kb.get_score();

        let this_change = climb_kb_score - kb_score;
        let this_improvement: f64 = (this_change).max(0.0);

        avg = get_new_avg(this_improvement, avg, i);

        let delta: f64 = this_improvement - last_improvement;
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

// TODO: How do make the division work with f64. Do we try to fix the truncating behavior?
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
        return 1.0 + K * delta.powf(0.0001); // Even less scaling for positive values
    }

    return 1.0 + K * delta.sqrt();
}
