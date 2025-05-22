extern crate alloc;

use {alloc::collections::BTreeMap, core::cmp, std::fs::File};

use {
    anyhow::{Result, anyhow},
    // rand::{Rng as _, SeedableRng as _, prelude::IndexedRandom as _, rngs::SmallRng},
    rand::{Rng as _, SeedableRng as _, rngs::SmallRng},
};

use crate::{
    custom_err::CorpusErr,
    // display::{update_avg, update_climb_info, update_climb_stats, update_eval, update_kb},
    display::{update_avg, update_eval, update_kb},
    keyboard::{Key, Keyboard, Slot},
    swappable_arr,
    swappable_keys,
    utils::write_err,
};

const ELITE_CNT: usize = 1;

// TODO: Need to re-think population management. Biggest issue is that progression is not
// sufficiently related to score. The probabalistic selection does not give sufficient favoritism
// to the top scorers.
// FUTURE: If user input is allowed for population management, the underlying math needs to be
// redone to check for errors
pub struct Population {
    rng: SmallRng,
    id: IdSpawner,
    pop_size: usize,
    population: Vec<Keyboard>,
    climber_cnt: usize,
    climbers: Vec<Keyboard>,
    swap_table: SwapTable,
    generation: usize,
    top_score: f64,
}

impl Population {
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
            let mut keyboard = Keyboard::create_primo(id.get());
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
            swap_table: SwapTable::new(),
            generation: 0,
            top_score: 0.0,
        });
    }

    pub fn refill_pop(&mut self) {
        self.generation += 1;

        debug_assert!(
            self.climbers.len() <= self.climber_cnt,
            "Current climbers {} is higher than the climber count {}",
            self.climbers.len(),
            self.climber_cnt
        );

        self.population.clear();
        for climber in &self.climbers {
            self.population.push(climber.clone());
        }

        let to_add = self.pop_size - self.climbers.len();
        for _ in 0..to_add {
            let new_kb = Keyboard::from_swap_table(
                &mut self.rng,
                &self.swap_table,
                self.generation,
                self.id.get(),
            );
            self.population.push(new_kb);
        }

        for p in self.population.iter_mut() {
            if p.is_elite() {
                continue;
            }

            p.shuffle(&mut self.rng, 2);
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

        return Ok(());
    }

    // TODO: Direct index access. This *should* be an iter_mut(), but that doesn't work because
    // hill climbing is not a self method
    // TODO: The population should store the average hill climbing iterations and use that for the
    // max without improvement, whether in full or some % of it
    pub fn climb_kbs(&mut self, corpus: &[String], iter: usize) -> Result<()> {
        let mut climber_score = 0.0;
        for i in 0..self.climbers.len() {
            // let climb_info: String = format!(
            //     "Keyboard: {:02}, Generation: {:05}, ID: {:07}",
            //     i.checked_add(1).expect("Too many climbers in climb_kbs"),
            //     self.climbers[i].get_generation(),
            //     self.climbers[i].get_id()
            // );
            // update_climb_info(&climb_info)?;

            self.climbers[i] = hill_climb(
                &mut self.rng,
                &self.climbers[i],
                corpus,
                iter,
                &mut self.swap_table,
            )?;

            if self.climbers[i].get_score() > self.top_score {
                self.top_score = self.climbers[i].get_score();
                update_kb(&self.climbers[i])?;
            }

            climber_score += self.climbers[i].get_score();
        }

        let avg_climber_score = climber_score / self.climbers.len() as f64;
        update_avg(avg_climber_score)?;

        // TODO: The climb method does indeed need to tell the display there is nothing to display,
        // but the climb method should not have to tell the display what needs displayed
        // update_climb_info(&" ".repeat(155))?;
        // TODO: At least for now, don't turn off climb_stats at the end of a hill climbing
        // iteration because it makes the display flicker. Might be okay to do that once the whole
        // line isn't being changed each time
        // update_climb_stats(&" ".repeat(155))?;
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
    swap_table: &mut SwapTable,
) -> Result<Keyboard> {
    // const MAX_ITER_WITHOUT_IMPROVEMENT: usize = 90;
    const MAX_ITER_WITHOUT_IMPROVEMENT: usize = 400;
    const CLAMP_VALUE: f64 = 0.999_999_999_999_999_9;

    // Iter should never be high enough for this to fail
    let mut decay_factor: f64 = 1.0 - (1.0 / iter as f64);
    decay_factor = decay_factor.min(CLAMP_VALUE);

    let mut kb: Keyboard = keyboard.clone();
    // let start = kb.get_score();

    let mut last_improvement: f64 = 0.0;
    let mut avg: f64 = 0.0;
    let mut weighted_avg: f64 = 0.0;
    let mut sum_weights: f64 = 0.0;

    // One indexed for averaging math and display
    for i in 1..=10000 {
        let kb_score = kb.get_score();

        let mut climb_kb: Keyboard = kb.clone();
        // climb_kb.shuffle(rng, 1);
        climb_kb.table_swap(rng, swap_table);
        climb_kb.eval(corpus);
        climb_kb.check_table_swap(swap_table);
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
        // let climb_stats: String = format!(
        //     "Iter: {:05}, Start: {:18}, Cur: {:18}, Best: {:18}, Avg: {:18}, Weighted: {:18}\r",
        //     i, start, climb_kb_score, kb_score, avg, weighted_avg
        // );
        // println!("{}", climb_info.len());
        // update_climb_stats(&climb_stats)?;

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

// The strong weight toward positive iterations is to give hill climbers the chance to catch up in
// later generations
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

pub struct SwapTable {
    swap_table: Vec<Vec<BTreeMap<Key, SwapScore>>>,
}

impl SwapTable {
    // TODO: Obvious issue here is we have the number row in the swap table even though we don't
    // want to use it. You could only build three rows in the table and subtract from the value of
    // the slot in get_score, but that feels like a hack
    pub fn new() -> Self {
        swappable_arr!();

        let mut swap_table: Vec<Vec<BTreeMap<Key, SwapScore>>> = Vec::new();

        for _ in 0..4 {
            let mut row: Vec<BTreeMap<Key, SwapScore>> = Vec::new();
            for _ in 0..10 {
                let mut swap_options: BTreeMap<Key, SwapScore> = BTreeMap::new();
                for key in &SWAPPABLE_KEYS {
                    swap_options.insert(Key::from_tuple(*key), SwapScore::new());
                }

                row.push(swap_options);
            }

            swap_table.push(row);
        }

        return Self { swap_table };
    }

    pub fn get_slot_info(&self, slot: Slot) -> &BTreeMap<Key, SwapScore> {
        let row = slot.get_row();
        let col = slot.get_col();

        return &self.swap_table[row][col];
    }

    // This fn is often used in iterators where a reference is provided. Passing by ref here to
    // avoid the de-referencing step
    #[expect(clippy::trivially_copy_pass_by_ref)]
    pub fn get_score(&self, slot: &Slot, key: &Key) -> f64 {
        let row = slot.get_row();
        let col = slot.get_col();

        return self.swap_table[row][col][key].get_w_avg();
    }

    pub fn update_score(&mut self, slot: Slot, key: Key, new_score: f64) {
        let row = slot.get_row();
        let col = slot.get_col();
        let mut score = self.swap_table[row][col][&key];

        score.reweight_avg(new_score);
        self.swap_table[row][col].insert(key, score);
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SwapScore {
    w_avg: f64,
    weights: f64,
}

impl SwapScore {
    pub fn new() -> Self {
        return Self {
            w_avg: 0.0,
            weights: 0.0,
        };
    }

    pub fn get_w_avg(&self) -> f64 {
        return self.w_avg;
    }

    pub fn reweight_avg(&mut self, new_score: f64) {
        let inflated_avg = self.w_avg * self.weights;
        let adj_avg = inflated_avg * 0.995;
        let adj_weight = self.weights * 0.995;

        self.weights = adj_weight + 1.0;
        self.w_avg = (adj_avg + new_score) / self.weights;
    }
}
