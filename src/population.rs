use {core::cmp, std::fs::File};

use {
    anyhow::{Result, anyhow},
    rand::{Rng as _, SeedableRng as _, prelude::IndexedRandom as _, rngs::SmallRng},
};

use crate::{
    custom_err::CorpusErr,
    display::{update_avg, update_climb_info, update_climb_stats, update_eval, update_kb},
    keyboard::Keyboard,
    utils::write_err,
};

const ELITE_CNT: usize = 1;

// TODO: Add a swap history/swap table so swaps can be probabalistically constrained to ones likely
// to improve the keyboard. Do this after a full clippy pass of the non-swap code
// TODO: Maybe add some sort of health check function to make sure all the keyboards keep the same
// parameters. Unsure to what extent this is a population vs keyboard problem
// TODO: When incorporating multi-threading for the keyboards, unsure if we can pass a SmallRng
// somehow, or make it global with a ref_cell, or if we should use thread_rng. From what I
// understand, thread_rng uses the cryptographically secure RNG, which is slower
// Importantly though, we want to be able to store and re-use the RNG seed, so whatever our RNG
// solution is cannot have multiple seeds
pub struct Population {
    rng: SmallRng,
    id: IdSpawner,
    pop_size: usize,
    population: Vec<Keyboard>,
    climber_cnt: usize,
    climbers: Vec<Keyboard>,
    generation: usize,
    top_score: f64,
}

impl Population {
    // TODO: Long function
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
        let elite_cnt: usize = ELITE_CNT;
        // Should be impossible to fail due to compile time constraints
        debug_assert!(
            elite_cnt <= climber_cnt,
            "Elite count {elite_cnt} is higher than climber count {climber_cnt}"
        );

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
            generation: 0,
            top_score: 0.0,
        });
    }

    // TODO: The checked math is fine for now, but not robust if user input is allowed
    // PERF: For simplicity, we are currently using push/drain/clear on the various Vecs to manage
    // their contents. If this is slow, move to simply reading and writing to it directly. This
    // *should* be possible without unsafe
    // PERF: Instead of clearing and rebuilding the population each time, it is theoretically
    // faster to hold the climbers in there and iterate over a slice of the population, though that
    // would also require rebuilding how the climbers are created. In practice, I have not seen
    // this step take a lot of time in profiling, so probably irrelevant
    pub fn mutate_climbers(&mut self, amts: [usize; 4]) {
        self.generation
            .checked_add(1)
            .expect("Too many generations");

        debug_assert!(
            self.climbers.len() <= self.climber_cnt,
            "Current climbers {} is higher than the climber count {}",
            self.climbers.len(),
            self.climber_cnt
        );

        self.population.clear();
        let tot_score = self
            .climbers
            .iter()
            .fold(0.0_f64, |acc, c| return acc + c.get_score());

        let to_add = self
            .pop_size
            .checked_sub(self.climbers.len())
            .expect("Climbers greater than population size");

        for _ in 0..to_add {
            let parent = {
                let r = self.rng.random_range(0.0_f64..=tot_score);
                let mut checked_score: f64 = 0.0;

                self.climbers
                    .iter()
                    .find(|climber| {
                        checked_score += climber.get_score();
                        return checked_score > r;
                    })
                    .unwrap_or_else(|| {
                        return self.climbers.last().expect("Climbers should not be empty");
                    })
            };

            let this_amt = *amts
                .choose(&mut self.rng)
                .expect("Amts should not be empty");
            let mut new_kb = Keyboard::mutate_from(parent, self.generation, self.id.get());
            new_kb.shuffle(&mut self.rng, this_amt);

            self.population.push(new_kb);
        }

        assert_eq!(
            self.population.len(),
            self.pop_size,
            "Population {} does not match the population size {}",
            self.population.len(),
            self.pop_size
        );
    }

    pub fn eval_gen_pop(&mut self, corpus: &[String]) -> Result<(), CorpusErr> {
        if corpus.is_empty() {
            return Err(CorpusErr::EmptyCorpus);
        }

        for (i, kb) in self.population.iter_mut().enumerate() {
            let display_num = i.checked_add(1).expect("Population has too many to count");
            update_eval(display_num)?;

            kb.eval(corpus);
        }

        update_eval(0)?;
        return Ok(());
    }

    // TODO: Long function
    pub fn setup_climbers(&mut self) -> Result<()> {
        self.climbers.clear();
        self.population.sort_by(|a, b| {
            return b
                .get_score()
                .partial_cmp(&a.get_score())
                .unwrap_or(cmp::Ordering::Equal);
        });

        self.climbers.extend(self.population.drain(..ELITE_CNT));
        let elite = self.climbers.first().expect("Elite climber not drained");
        if elite.get_score() > self.top_score {
            self.top_score = elite.get_score();
            update_kb(elite)?;
        }

        let mut population_score = self
            .population
            .iter()
            .fold(0.0_f64, |acc, p| return acc + p.get_score());

        while self.climbers.len() < self.climber_cnt && !self.population.is_empty() {
            let mut checked_score: f64 = 0.0;
            let r = self.rng.random_range(0.0_f64..=population_score);

            for (i, kb) in self.population.iter_mut().enumerate() {
                checked_score += kb.get_score();
                if checked_score >= r {
                    population_score -= kb.get_score();
                    self.climbers.extend(self.population.drain(i..=i));

                    break;
                }
            }
        }

        for climber in self.climbers.iter_mut().take(ELITE_CNT) {
            climber.set_elite();
        }

        for climber in self.climbers.iter_mut().skip(ELITE_CNT) {
            climber.unset_elite();
        }

        let mut climber_score: f64 = 0.0;
        for climber in &self.climbers {
            climber_score += climber.get_score();
        }

        // climbers.len() should never be big enough for this to fail
        let avg_climber_score = climber_score / self.climbers.len() as f64;
        update_avg(avg_climber_score)?;

        return Ok(());
    }

    // TODO: Direct index access. This *should* be an iter_mut(), but that doesn't work because
    // hill climbing is not a self method
    pub fn climb_kbs(&mut self, corpus: &[String], iter: usize) -> Result<()> {
        for i in 0..self.climbers.len() {
            let climb_info: String = format!(
                "Keyboard: {:02}, Generation: {:05}, ID: {:07}",
                i.checked_add(1).expect("Too many climbers in climb_kbs"),
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
        self.next_id
            .checked_add(1)
            .expect("Too many kbs created when getting ID");

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

    // Iter should never be high enough for this to fail
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
