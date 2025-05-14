use std::fs::File;

use {
    anyhow::{Result, anyhow},
    rand::{Rng as _, SeedableRng as _, rngs::SmallRng},
};

use crate::{
    custom_err::CorpusErr,
    display::{update_avg, update_climb_info, update_climb_stats, update_eval, update_kb},
    keyboard::Keyboard,
    utils::write_err,
};

// TODO: Add a swap history/swap table so swaps can be probabalistically constrained to ones likely
// to improve the keyboard. Do this after a full clippy pass of the non-swap code
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
    generation: usize,
    top_score: f64,
}

impl Population {
    const ELITE_CNT: usize = 1;
    // TODO: Long function
    // TODO: Current code cannot robustly handle input options
    // TODO: long function signature
    // TODO: This will eventually take user input, so keep the error return
    pub fn create(size: Option<usize>, log_handle: &mut File) -> Result<Self> {
        const DEFAULT_POPULATION: usize = 100;
        const DEFAULT_CLIMB_CNT: usize = 20;
        const MIN_CLIMBERS: usize = 1;

        let seed: [u8; 32] = rand::random();
        let seed_string: String = format!("{seed:?}");
        write_err(log_handle, &seed_string)?;
        let mut rng = SmallRng::from_seed(seed);

        let mut id = IdSpawner::new();

        let pop_cnt: usize = size.unwrap_or(DEFAULT_POPULATION);
        if pop_cnt == 0 {
            return Err(anyhow!("Population size cannot be zero"));
        }
        let gen_pop: Vec<Keyboard> = Vec::with_capacity(pop_cnt);

        // TODO: The input should just be the population total and the climber total. No need for
        // goofy % math

        let climber_cnt: usize = DEFAULT_CLIMB_CNT;
        if climber_cnt > pop_cnt {
            return Err(anyhow!(
                "Climbers {climber_cnt} cannot be greater than total population ({pop_cnt})"
            ));
        } else if climber_cnt < MIN_CLIMBERS {
            return Err(anyhow!(
                "Climbers {climber_cnt} less than the minimum ({MIN_CLIMBERS})"
            ));
        }
        let mut climbers: Vec<Keyboard> = Vec::with_capacity(climber_cnt);

        // Having multiple elites kills genetic diversity
        let elite_cnt: usize = Self::ELITE_CNT;
        // Should be impossible to fail due to compile time constraints
        assert!(elite_cnt <= climber_cnt);

        // At the end of the last iteration, it is not necessary to mutate the climbers. Therefore,
        // mutating climbers is done at the beginning of each iteration. Even though creating our
        // initial population as climbers rather than general population is unintuitive, it lets us
        // transition into the main loop logic without creating a special case for the first
        // iteration
        // TODO: The hard-coded shuffle value needs a place to live
        for _ in 0..climber_cnt {
            let mut keyboard = Keyboard::create_origin(id.get());
            keyboard.shuffle(&mut rng, 30);
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
            generation: 0,
            top_score: 0.0,
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
    // TODO: Format string in keyboard evaluation should be based on digits in total population
    // size
    // TODO: The probabalistic selection logic has to be something that can be outlined
    // PERF: We generate a random starting selection so the edge case doesn't always default to the
    // strongest member. Might be extra
    // PERF: For simplicity, we are currently using push/drain/clear on the various Vecs to manage
    // their contents. If this is slow, move to simply reading and writing to it directly. This
    // *should* be possible without unsafe
    pub fn mutate_climbers(&mut self, amts: [usize; 4]) {
        self.generation += 1;

        assert!(self.climbers.len() <= self.climber_cnt);

        self.population.clear();
        let mut climber_score: f64 = 0.0;
        for climber in &self.climbers {
            climber_score += climber.get_score();
            self.population.push(climber.clone());
        }

        let to_add: usize = self.pop_size - self.climbers.len();
        for _ in 0..to_add {
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

            let mut new_kb =
                Keyboard::mutate_from(&self.climbers[idx], self.generation, self.id.get());
            let this_amt_idx: usize = self.rng.random_range(0..amts.len());
            let this_amt: usize = amts[this_amt_idx];
            new_kb.shuffle(&mut self.rng, this_amt);

            self.population.push(new_kb);
        }

        assert_eq!(self.population.len(), self.pop_size);
    }

    // TODO: Long function signature
    pub fn eval_gen_pop(&mut self, corpus: &[String]) -> Result<(), CorpusErr> {
        if corpus.is_empty() {
            return Err(CorpusErr::EmptyCorpus);
        }

        for i in 0..self.population.len() {
            update_eval(i + 1)?;
            self.population[i].eval(corpus);
        }

        update_eval(0)?;
        return Ok(());
    }

    // TODO: Since we're just going with one elite, we only need to pull one out. And because only
    // one elite creates more diversity, checking for duplicates in the general population is a
    // waste of time, so that can be stripped out. I'm also not against getting rid of the code to
    // cut the bottom n, since it adds complexity to the setup.
    // TODO: Long function
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

        self.climbers.clear();
        // Pull out elite
        self.climbers.extend(self.population.drain(..1));
        if self.climbers[0].get_score() > self.top_score {
            self.top_score = self.climbers[0].get_score();
            update_kb(&self.climbers[0])?;
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
            self.climbers
                .extend(self.population.drain(selection..=selection));
        }

        let mut elites_set: usize = 0;
        for climber in self.climbers.iter_mut() {
            if elites_set < Self::ELITE_CNT {
                climber.set_elite();
                elites_set += 1;
            } else {
                climber.unset_elite();
            }
        }

        let mut climber_score: f64 = 0.0;
        for climber in &self.climbers {
            climber_score += climber.get_score();
        }
        let avg_climber_score = climber_score / self.climbers.len() as f64;
        update_avg(avg_climber_score)?;

        return Ok(());
    }

    // TODO: Long function signature
    pub fn climb_kbs(&mut self, corpus: &[String], iter: usize) -> Result<()> {
        for i in 0..self.climbers.len() {
            let climb_info: String = format!(
                "Keyboard: {:02}, Generation: {:05}, ID: {:07}",
                i + 1,
                self.climbers[i].get_generation(),
                self.climbers[i].get_id()
            );
            update_climb_info(&climb_info)?;

            self.climbers[i] = hill_climb(&mut self.rng, &self.climbers[i], corpus, iter)?;

            if self.climbers[0].get_score() > self.top_score {
                self.top_score = self.climbers[0].get_score();
                update_kb(&self.climbers[0])?;
            }
        }

        // TODO: The climb method does indeed need to tell the display there is nothing to display,
        // but the climb method should not have to tell the display what needs displayed
        update_climb_info(&" ".repeat(155))?;
        // TODO: At least for now, don't turn off climb_stats at the end of a hill climbing
        // iteration because it makes the display flicker. Might be okay to do that once the whole
        // line isn't being changed each time
        update_climb_stats(&" ".repeat(155))?;
        return Ok(());
    }

    pub fn get_pop_size(&self) -> usize {
        return self.pop_size;
    }

    pub fn get_climb_cnt(&self) -> usize {
        return self.climber_cnt;
    }

    pub fn get_elite_cnt(&self) -> usize {
        return self.elite_cnt;
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

// TODO: After the swap map is added, test whether allowing the elite to climb at all is good
// TODO: Re-introduce an annealing type concept back into here. We are not seeing reliable enough
// cycling
// PERF: Some of this calculation is per iteration and could be sectioned out
// TODO: Function too long. But don't want to chip away too much without knowing how it will be
// displayed
// TODO: Long function signature
// NOTE: Changing one key at a time works best. If you change two keys, the algorithm will find
// bigger changes less frequently. This causes the decay to continue for about as many
// iterations as it would if doing only one step, but fewer improvements will be found,
// causing the improvement at the end of the hill climbing step to be lower
pub fn hill_climb(
    rng: &mut SmallRng,
    keyboard: &Keyboard,
    corpus: &[String],
    iter: usize,
) -> Result<Keyboard> {
    const MAX_ITER_WITHOUT_IMPROVEMENT: usize = 90;
    const CLAMP_VALUE: f64 = 0.999_999_999_999_999_9;

    let mut decay_factor: f64 = 1.0 - (1.0 / iter as f64);
    decay_factor = decay_factor.min(CLAMP_VALUE);
    if keyboard.is_elite() {
        // Promotes more reliable global exploration
        decay_factor *= decay_factor.powf(5.0);
    }

    let mut kb: Keyboard = keyboard.clone();
    let start = kb.get_score();

    let mut last_improvement: f64 = 0.0;
    let mut avg: f64 = 0.0;
    let mut weighted_avg: f64 = 0.0;
    let mut sum_weights: f64 = 0.0;

    // One indexed for averaging math and display
    for i in 1..=10000 {
        let kb_score = kb.get_score();

        let mut climb_kb: Keyboard = kb.clone();
        climb_kb.shuffle(rng, 1);
        climb_kb.eval(corpus);
        let climb_kb_score = climb_kb.get_score();

        let this_change = climb_kb_score - kb_score;
        let this_improvement = (this_change).max(0.0);

        avg = get_new_avg(this_improvement, avg, i);

        let delta: f64 = this_improvement - last_improvement;
        last_improvement = this_improvement;
        let weight = get_weight(delta);

        sum_weights *= decay_factor;
        let weighted_avg_for_new: f64 = weighted_avg * sum_weights;
        sum_weights += weight;
        weighted_avg = (weighted_avg_for_new + this_improvement * weight) / sum_weights;

        // TODO: Have hard coded blank value when this isn't active, but need more principled
        // method
        let climb_stats: String = format!(
            "Iter: {:05}, Start: {:18}, Cur: {:18}, Best: {:18}, Avg: {:18}, Weighted: {:18}\r",
            i, start, climb_kb_score, kb_score, avg, weighted_avg
        );
        // println!("{}", climb_info.len());
        update_climb_stats(&climb_stats)?;

        if climb_kb_score > kb_score {
            climb_kb.add_pos_iter();
            kb = climb_kb;
        }

        // NOTE: The i > 1 condition pastes over an edge case where the first improvement on the
        // first iteration is smaller than the unweighted mean due to floating point imprecision
        let plateauing: bool = weighted_avg < avg && i > 1;
        let not_starting: bool = avg <= 0.0 && i >= MAX_ITER_WITHOUT_IMPROVEMENT;
        if plateauing || not_starting {
            break;
        }
    }

    return Ok(kb);
}

// TODO: How do make the division work with f64. Do we try to fix the truncating behavior?
fn get_new_avg(new_value: f64, old_avg: f64, new_count: usize) -> f64 {
    let new_value_for_new_avg: f64 = new_value / (new_count as f64);
    let old_avg_for_new_avg: f64 = old_avg * ((new_count as f64 - 1.0) / new_count as f64);

    return new_value_for_new_avg + old_avg_for_new_avg;
}

// The strong weight for positive iterations is necessary for hill climbers to catch up to the
// elite in later iterations. This comes with the trade-off risk that an early elite can entrench
// itself in a local optima. This risk is mitigated by giving the elite a punishing decay factor in
// early iterations. However, the elite is not double-punished with a reduced weight for positive
// iterations. This allows for excellent genomes to still have the opportunity to climb.
fn get_weight(delta: f64) -> f64 {
    const K: f64 = 0.01;

    if delta <= 0.0_f64 {
        return 1.0;
    }

    return 1.0 + K * delta.powf(0.9);

    // Alternatives:
    // return 1.0 + K * delta.sqrt();
    // return 1.0 + K * delta.ln();
    // return 1.0 + K * delta.powf(0.0001);
}
