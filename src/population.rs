extern crate alloc;

use {alloc::collections::BTreeMap, core::cmp};

use {
    anyhow::Result,
    rand::{Rng as _, SeedableRng as _, rngs::SmallRng},
};

use crate::{
    custom_err::CorpusErr,
    display::{update_avg, update_climb_info, update_eval, update_kb},
    keyboard::{Key, Keyboard, Slot},
    keys,
    structs::IdSpawner,
    swappable_keys,
};

swappable_keys!();

const MIN_POP: usize = 20;
const MAX_POP: usize = 50;
const MIN_CLIMB_PCT: f64 = 0.1;
const MAX_CLIMB_PCT: f64 = 0.5;
const MIN_ELITES: usize = 1;
const MAX_ELITES: usize = 2;

const MIN_MUTATION: usize = 0;
const MAX_MUTATION: usize = 3;

// TODO: Implement the hyper-heuristic idea. Includes:
// - Weighted avg for swap table
// - Temperature goal (0.05 - 0.15)
// - Climb decay
// - Climb improvement weight
// - Probabalistic selection or tournament(usize)
// - Variable population feels overly-complex but maybe
pub struct Population {
    id: usize,
    rng: SmallRng,
    id_spawner: IdSpawner,
    pop_cnt: usize,
    population: Vec<Keyboard>,
    climber_cnt: usize,
    climbers: Vec<Keyboard>,
    elite_cnt: usize,
    mutation: usize,
    swap_table: SwapTable,
    generation: usize,
    top_score: f64,
}

impl Population {
    // TODO: Need an method to re-roll population, climbers, and elites
    // FUTURE: Sloppy, but don't want to get into deep refactor without knowing how the
    // meta-population management will be handled
    // FUTURE: Could do bigger populations and/or more climbers after multi-threading
    // FUTURE: Add an option to cull some bottom % of the population
    // FUTURE: Add an option to do tournament mode for thinning population
    pub fn create(id_in: usize) -> Self {
        let seed: [u8; 32] = rand::random();
        let mut rng = SmallRng::from_seed(seed);

        let mut id_spawner = IdSpawner::new();

        let pop_cnt = rng.random_range(MIN_POP..=MAX_POP);
        let climb_pct = rng.random_range(MIN_CLIMB_PCT..=MAX_CLIMB_PCT);
        let climber_cnt = (pop_cnt as f64 * climb_pct).round() as usize;
        let elite_cnt = rng.random_range(MIN_ELITES..=MAX_ELITES);

        assert!(
            elite_cnt <= climber_cnt,
            "Elite count {elite_cnt} is higher than climber count {climber_cnt}"
        );

        let population: Vec<Keyboard> = Vec::with_capacity(pop_cnt);
        let mut climbers: Vec<Keyboard> = Vec::with_capacity(climber_cnt);

        let mutation = rng.random_range(MIN_MUTATION..=MAX_MUTATION);

        // New population members are created at the beginning of each iteration, so fill the
        // climbers now
        for _ in 0..climber_cnt {
            let mut keyboard = Keyboard::create_primo(id_spawner.get());
            keyboard.shuffle(&mut rng, SWAPPABLE_KEYS.len());
            climbers.push(keyboard);
        }

        return Self {
            id: id_in,
            rng,
            id_spawner,
            pop_cnt,
            population,
            climber_cnt,
            climbers,
            elite_cnt,
            mutation,
            swap_table: SwapTable::new(),
            generation: 0,
            top_score: 0.0,
        };
    }

    pub fn refill_pop(&mut self) {
        self.generation += 1;

        self.population.clear();
        for c in &self.climbers {
            let climbers_moved = self.population.len() == self.climber_cnt;
            let pop_filled = self.population.len() == self.pop_cnt;
            if climbers_moved || pop_filled {
                break;
            }

            self.population.push(c.clone());
        }

        // If the new climber_cnt is <= the old one, that number should be moved. If the new
        // climber_cnt is >=, all current climbers should be moved.
        assert!(
            self.population.len() == self.climber_cnt
                || self.population.len() == self.climbers.len(),
            "Not enough climbers moved in refill_pop"
        );

        let to_add = self.pop_cnt - self.population.len();
        for _ in 0..to_add {
            let new_kb = Keyboard::from_swap_table(
                &mut self.rng,
                &self.swap_table,
                self.generation,
                self.id_spawner.get(),
            );
            self.population.push(new_kb);
        }

        for p in self.population.iter_mut().filter(|p| return !p.is_elite()) {
            p.shuffle(&mut self.rng, self.mutation);
        }

        assert_eq!(
            self.population.len(),
            self.pop_cnt,
            "Population {} does not match the population size {}",
            self.population.len(),
            self.pop_cnt
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

        self.climbers
            .extend(self.population.drain(..self.elite_cnt));
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

        for climber in self.climbers.iter_mut().take(self.elite_cnt) {
            climber.set_elite();
        }

        for climber in self.climbers.iter_mut().skip(self.elite_cnt) {
            climber.unset_elite();
        }

        return Ok(());
    }

    pub fn climb_kbs(&mut self, corpus: &[String], iter: usize) -> Result<()> {
        let mut climber_score = 0.0_f64;
        for i in 0..self.climbers.len() {
            let climb_info: String = format!(
                "Keyboard: {:02}, Generation: {:05}, ID: {:07}",
                i.checked_add(1).expect("Too many climbers in climb_kbs"),
                self.climbers[i].get_generation(),
                self.climbers[i].get_id()
            );
            update_climb_info(&climb_info)?;

            self.climbers[i] = hill_climb(
                &mut self.rng,
                &self.climbers[i],
                corpus,
                iter,
                &mut self.swap_table,
            );

            if self.climbers[i].get_score() > self.top_score {
                self.top_score = self.climbers[i].get_score();
                update_kb(&self.climbers[i])?;
            }

            climber_score += self.climbers[i].get_score();
        }

        let avg_climber_score = climber_score / self.climbers.len() as f64;
        update_avg(avg_climber_score)?;

        return Ok(());
    }

    pub fn get_id(&self) -> usize {
        return self.id;
    }

    pub fn get_pop_cnt(&self) -> usize {
        return self.pop_cnt;
    }

    pub fn get_climb_cnt(&self) -> usize {
        return self.climber_cnt;
    }

    pub fn get_elite_cnt(&self) -> usize {
        return self.elite_cnt;
    }

    pub fn get_mutation(&self) -> usize {
        return self.mutation;
    }

    // TODO: Maybe don't need this. If you're creating an offspring maybe you would just create a
    // new one with properties of the parent
    // pub fn set_new_pop_cnt(&mut self, new_cnt: usize) {
    //     self.pop_cnt = new_cnt;
    //
    //     if self.climber_cnt > self.pop_cnt {
    //         self.climber_cnt = self.pop_cnt;
    //     }
    //
    //     if self.elite_cnt > self.pop_cnt {
    //         self.elite_cnt = self.pop_cnt;
    //     }
    // }

    pub fn randomize_pop_cnt(&mut self) {
        self.pop_cnt = self.rng.random_range(MIN_POP..=MAX_POP);

        if self.climber_cnt > self.pop_cnt {
            self.climber_cnt = self.pop_cnt;
        }

        assert!(
            self.pop_cnt >= self.elite_cnt,
            "elite_cnt higher than pop_cnt in randomize_pop_cnt"
        );
    }

    pub fn randomize_climber_cnt(&mut self) {
        let climb_pct = self.rng.random_range(MIN_CLIMB_PCT..=MAX_CLIMB_PCT);
        self.climber_cnt = (self.pop_cnt as f64 * climb_pct).round() as usize;

        assert!(
            self.climber_cnt <= self.pop_cnt,
            "Climber cnt higher than pop cnt in randomize_climber_cnt"
        );

        assert!(
            self.climber_cnt >= self.elite_cnt,
            "Climber cnt higher than elite_cnt in randomize_climber_cnt"
        );
    }

    pub fn randomize_elite_cnt(&mut self) {
        self.elite_cnt = self.rng.random_range(MIN_ELITES..=MAX_ELITES);
        assert!(
            self.elite_cnt <= self.pop_cnt,
            "elite_cnt > pop_cnt in randomize_elite_cnt"
        );

        assert!(
            self.climber_cnt >= self.elite_cnt,
            "Climber cnt higher than elite_cnt in randomize_elite_cnt"
        );
    }

    pub fn randomize_mutation(&mut self) {
        self.mutation = self.rng.random_range(MIN_MUTATION..=MAX_MUTATION);
    }
}

// FUTURE: It would be better if max_iter_without_improvement were based on the average iters per
// hill climb, but right now it would be contrived to pass around that data
// PERF: Some of this calculation is per iteration and could be sectioned out
// FUTURE: Don't love the shape of this function, but need to see how meta-population shakes out
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
) -> Keyboard {
    const CLAMP_VALUE: f64 = 0.999_999_999_999_999_9;

    let max_iter_without_improvement = iter / 2;
    let mut decay_factor: f64 = 1.0 - (1.0 / iter as f64);
    decay_factor = decay_factor.min(CLAMP_VALUE);

    let mut kb: Keyboard = keyboard.clone();

    let mut last_improvement: f64 = 0.0;
    let mut avg: f64 = 0.0;
    let mut weighted_avg: f64 = 0.0;
    let mut sum_weights: f64 = 0.0;

    // One indexed for averaging math and display
    for i in 1..=10000 {
        let kb_score = kb.get_score();

        let mut climb_kb: Keyboard = kb.clone();
        climb_kb.table_swap(rng, swap_table);
        climb_kb.eval(corpus);
        climb_kb.check_table_swap(swap_table);
        let climb_kb_score = climb_kb.get_score();

        let this_change = climb_kb_score - kb_score;
        let this_improvement = (this_change).max(0.0);

        avg = get_new_avg(this_improvement, avg, i);

        let delta = this_improvement - last_improvement;
        last_improvement = this_improvement;
        let weight = get_weight(delta);

        sum_weights *= decay_factor;
        let weighted_avg_for_new = weighted_avg * sum_weights;
        sum_weights += weight;
        weighted_avg = (weighted_avg_for_new + this_improvement * weight) / sum_weights;

        if climb_kb_score > kb_score {
            climb_kb.add_pos_iter();
            kb = climb_kb;
        }

        // NOTE: The i > 1 condition pastes over an edge case where the first improvement on the
        // first iteration is smaller than the unweighted mean due to floating point imprecision
        let plateauing: bool = weighted_avg < avg && i > 1;
        let not_starting: bool = avg <= 0.0 && i >= max_iter_without_improvement;
        if plateauing || not_starting {
            break;
        }
    }

    return kb;
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
    // FUTURE: Obvious issue here is we have the number row in the swap table even though we don't
    // want to use it. You could only build three rows in the table and subtract from the value of
    // the slot in get_score, but that feels like a hack
    pub fn new() -> Self {
        let mut swap_table: Vec<Vec<BTreeMap<Key, SwapScore>>> = Vec::new();

        for _ in 0_usize..4_usize {
            let mut row: Vec<BTreeMap<Key, SwapScore>> = Vec::new();
            for _ in 0_usize..10_usize {
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
        let adj_avg = inflated_avg * 0.995_f64;
        let adj_weight = self.weights * 0.995_f64;

        self.weights = adj_weight + 1.0_f64;
        self.w_avg = (adj_avg + new_score) / self.weights;
    }
}
