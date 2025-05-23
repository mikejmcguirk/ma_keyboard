extern crate alloc;

use {alloc::collections::BTreeMap, core::cmp};

use {
    anyhow::Result,
    rand::{Rng as _, SeedableRng as _, rngs::SmallRng},
};

use crate::{
    display::{update_climb_info, update_cur_avg, update_eval_dsp},
    keyboard::Keyboard,
    keys,
    structs::{IdSpawner, Key, Slot},
    swappable_keys,
};

swappable_keys!();

const MIN_POP: usize = 20;
const MAX_POP: usize = 100;
const MIN_CLIMB_PCT: f64 = 0.1;
const MAX_CLIMB_PCT: f64 = 0.4;
const ELITE_CNT: usize = 1;

const MIN_MUTATION: usize = 0;
const MAX_MUTATION: usize = 3;

const MIN_SCORE_DECAY: f64 = 0.9;
const MAX_SCORE_DECAY: f64 = 0.998;

const MIN_K_TEMP: f64 = -31.162_892_36;
const MAX_K_TEMP: f64 = -6.107_632_992;

const MUTATION_RATE: f64 = 0.05;

// TODO: Need to redo having population and climbers in One Vec. Causing problems juggling where
// the population is at any given time. Results in hacks about when certain parts of the process
// are run
// FUTURE: Consider letting populations engage in tournament selection
// FUTURE: Generation should be meta-population controlled
pub struct Population {
    id: usize,
    rng: SmallRng,
    id_spawner: IdSpawner,
    pop_cnt: usize,
    population: Vec<Keyboard>,
    climber_cnt: usize,
    elite_cnt: usize,
    mutation: usize,
    swap_table: SwapTable,
    k_temp: f64,
    score_decay: f64,
    generation: usize,
    top_score: f64,
    total_climbs: usize,
    avg_climb_iter: f64,
    climb_decay: f64,
    is_elite: bool,
}

// NOTE: In order to avoid issues with high-scoring keyboards being accidently lost, the code is
// hyper-vigilant about setting and checking elite status
impl Population {
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
        let elite_cnt = ELITE_CNT;

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
            keyboard.shuffle(SWAPPABLE_KEYS.len());
            climbers.push(keyboard);
        }

        let k_temp = rng.random_range(MIN_K_TEMP..=MAX_K_TEMP);

        let score_decay = rng.random_range(MIN_SCORE_DECAY..=MAX_SCORE_DECAY);

        return Self {
            id: id_in,
            rng,
            id_spawner,
            pop_cnt,
            population,
            climber_cnt,
            elite_cnt,
            mutation,
            swap_table: SwapTable::new(),
            k_temp,
            score_decay,
            generation: 0,
            top_score: 0.0,
            total_climbs: 0,
            avg_climb_iter: 0.0,
            climb_decay: 0.0,
            is_elite: false,
        };
    }

    pub fn from_parents(parent_a: &Population, parent_b: &Population, id_in: usize) -> Self {
        let seed: [u8; 32] = rand::random();
        let mut rng = SmallRng::from_seed(seed);

        let id_spawner = IdSpawner::new();

        let top_a = parent_a.get_top_score();
        let top_b = parent_b.get_top_score();
        let top_score = top_a.max(top_b);

        let total_top = top_a + top_b;
        let top_a_pct = top_a / total_top;
        let top_b_pct = top_b / total_top;

        let pop_cnt = if rng.random_range(0.0..=1.0) <= MUTATION_RATE {
            rng.random_range(MIN_POP..=MAX_POP)
        } else if rng.random_range(0.0..=1.0) <= top_a_pct {
            parent_a.get_pop_cnt()
        } else {
            parent_b.get_pop_cnt()
        };

        let climb_pct = if rng.random_range(0.0..=1.0) <= MUTATION_RATE {
            rng.random_range(MIN_CLIMB_PCT..=MAX_CLIMB_PCT)
        } else if rng.random_range(0.0..=1.0) <= top_a_pct {
            parent_a.get_climb_pct()
        } else {
            parent_b.get_climb_pct()
        };

        let mut climber_cnt = (pop_cnt as f64 * climb_pct).round() as usize;
        let elite_cnt = ELITE_CNT;
        if climber_cnt < elite_cnt {
            climber_cnt = elite_cnt;
        }

        assert!(
            pop_cnt >= climber_cnt,
            "Climbers {climber_cnt} greater than population {pop_cnt} in from_parents"
        );

        let pop_a: &[Keyboard] = parent_a.get_population();
        let pop_b: &[Keyboard] = parent_b.get_population();
        let mut population: Vec<Keyboard> = Vec::with_capacity(pop_a.len() + pop_b.len());
        population.extend(
            pop_a
                .iter()
                .chain(pop_b.iter())
                .map(|k| return k.kb_clone()),
        );

        let mut elites: Vec<Keyboard> = Vec::new();
        let mut i = 0;
        while i < population.len() {
            if !population[i].is_elite() {
                i += 1;
                continue;
            }

            elites.push(population.swap_remove(i));
        }

        let mut full_pop_score = population
            .iter()
            .fold(0.0_f64, |acc, p| return acc + p.get_score());
        debug_assert!(full_pop_score > 0.0, "Parent populations not evaluated");

        while population.len() > (pop_cnt / 4) - elites.len() && !population.is_empty() {
            let mut checked_score: f64 = 0.0;
            let r = rng.random_range(0.0_f64..=full_pop_score);

            for (j, kb) in population.iter().enumerate() {
                checked_score += kb.get_score();
                if checked_score >= r {
                    full_pop_score -= kb.get_score();
                    population.swap_remove(j);

                    break;
                }
            }
        }

        population.append(&mut elites);
        population.sort_by(|a, b| {
            return b
                .get_score()
                .partial_cmp(&a.get_score())
                .unwrap_or(cmp::Ordering::Equal);
        });

        for p in population.iter_mut().take(elite_cnt) {
            p.set_elite();
        }

        for p in population.iter_mut().skip(elite_cnt) {
            p.unset_elite();
        }

        let top_elite_score: f64 = population
            .iter()
            .fold(0.0, |acc, p| return acc.max(p.get_score()));
        debug_assert_eq!(
            top_elite_score, top_score,
            "Elite lost after filtering population in create_child"
        );

        let mutation = if rng.random_range(0.0..=1.0) <= MUTATION_RATE {
            rng.random_range(MIN_MUTATION..=MAX_MUTATION)
        } else if rng.random_range(0.0..=1.0) <= top_a_pct {
            parent_a.get_mutation()
        } else {
            parent_b.get_mutation()
        };

        let mut swap_table = SwapTable::new();

        for j in 0_usize..4_usize {
            for k in 0_usize..10_usize {
                for key_tuple in &SWAPPABLE_KEYS {
                    let key = Key::from_tuple(*key_tuple);

                    if rng.random_range(0.0..=1.0) <= MUTATION_RATE {
                        swap_table.replace_score(j, k, key, SwapScore::new());
                        continue;
                    }

                    let swap_score_a = parent_a.get_swap_score(j, k, key);
                    let swap_score_b = parent_b.get_swap_score(j, k, key);

                    let score_a = swap_score_a.get_w_avg();
                    let score_b = swap_score_b.get_w_avg();
                    let weight_a = swap_score_a.get_weights();
                    let weight_b = swap_score_b.get_weights();

                    if rng.random_range(0.0..=1.0) >= top_a_pct {
                        let new_score = score_a;
                        let new_weight = weight_a;

                        let new_swap_score = SwapScore::from_values(new_score, new_weight);
                        swap_table.replace_score(j, k, key, new_swap_score);
                    } else {
                        let new_score = score_b;
                        let new_weight = weight_b;

                        let new_swap_score = SwapScore::from_values(new_score, new_weight);
                        swap_table.replace_score(j, k, key, new_swap_score);
                    }
                }
            }
        }

        let k_temp = if rng.random_range(0.0..=1.0) <= MUTATION_RATE {
            rng.random_range(MIN_K_TEMP..=MAX_K_TEMP)
        } else if rng.random_range(0.0..=1.0) <= top_a_pct {
            parent_a.get_k_temp()
        } else {
            parent_b.get_k_temp()
        };

        let score_decay = if rng.random_range(0.0..=1.0) <= MUTATION_RATE {
            rng.random_range(MIN_SCORE_DECAY..=MAX_SCORE_DECAY)
        } else if rng.random_range(0.0..=1.0) <= top_a_pct {
            parent_a.get_score_decay()
        } else {
            parent_b.get_score_decay()
        };

        let generation = parent_a.get_generation().max(parent_b.get_generation());

        let avg_climb_iter_a = parent_a.get_avg_climb_iter();
        let avg_climb_iter_b = parent_b.get_avg_climb_iter();
        let total_climbs_a = parent_a.get_total_climbs();
        let total_climbs_b = parent_b.get_total_climbs();

        let avg_climb_iter;
        let total_climbs;
        if avg_climb_iter_a > avg_climb_iter_b {
            avg_climb_iter = avg_climb_iter_a;
            total_climbs = total_climbs_a;
        } else {
            avg_climb_iter = avg_climb_iter_b;
            total_climbs = total_climbs_b;
        }

        let climb_decay_a = parent_a.get_climb_decay();
        let climb_decay_b = parent_b.get_climb_decay();
        let climb_decay = (climb_decay_a * top_a_pct) + (climb_decay_b * top_b_pct);

        return Self {
            id: id_in,
            rng,
            id_spawner,
            pop_cnt,
            population,
            climber_cnt,
            elite_cnt,
            mutation,
            swap_table,
            k_temp,
            score_decay,
            generation,
            top_score,
            total_climbs,
            avg_climb_iter,
            climb_decay,
            is_elite: false,
        };
    }

    pub fn refill_pop(&mut self) {
        self.generation += 1;

        debug_assert!(
            self.population.len() < self.pop_cnt,
            "Population already too big at the start of refill_pop len {}, cnt{}",
            self.population.len(),
            self.pop_cnt
        );

        let to_add = self.pop_cnt - self.population.len();
        for _ in 0..to_add {
            let new_kb = Keyboard::from_swap_table(
                &self.swap_table,
                self.generation,
                self.id_spawner.get(),
                self.k_temp,
            );
            self.population.push(new_kb);
        }

        for p in self.population.iter_mut().filter(|p| return !p.is_elite()) {
            p.shuffle(self.mutation);
        }

        debug_assert_eq!(
            self.population.len(),
            self.pop_cnt,
            "Population {} does not match the population size {}",
            self.population.len(),
            self.pop_cnt
        );
    }

    pub fn eval_gen_pop(&mut self) -> Result<()> {
        for (i, kb) in self.population.iter_mut().enumerate() {
            let display_num = i.checked_add(1).expect("Population has too many to count");
            update_eval_dsp(display_num)?;

            kb.eval();
        }

        self.population.sort_by(|a, b| {
            return b
                .get_score()
                .partial_cmp(&a.get_score())
                .unwrap_or(cmp::Ordering::Equal);
        });

        for p in self.population.iter_mut().take(self.elite_cnt) {
            p.set_elite();
        }

        for p in self.population.iter_mut().skip(self.elite_cnt) {
            p.unset_elite();
        }

        for p in self.population.iter() {
            if p.get_score() > self.top_score {
                self.top_score = p.get_score();
            }
        }

        assert_eq!(
            self.population[0].get_score(),
            self.top_score,
            "Elite lost in eval_gen_pop"
        );

        update_eval_dsp(0)?;
        return Ok(());
    }

    pub fn filter_climbers(&mut self) -> Result<()> {
        let mut climbers: Vec<Keyboard> = Vec::new();

        let mut i = 0;
        while i < self.population.len() {
            if !self.population[i].is_elite() {
                i += 1;
                continue;
            }

            climbers.push(self.population.swap_remove(i));
        }

        let mut population_score = self
            .population
            .iter()
            .fold(0.0_f64, |acc, p| return acc + p.get_score());

        while climbers.len() < self.climber_cnt && !self.population.is_empty() {
            let mut checked_score: f64 = 0.0;
            let r = self.rng.random_range(0.0_f64..=population_score);

            for (j, kb) in self.population.iter_mut().enumerate() {
                checked_score += kb.get_score();
                if checked_score >= r {
                    population_score -= kb.get_score();
                    climbers.push(self.population.swap_remove(j));

                    break;
                }
            }
        }

        debug_assert!(
            !climbers.is_empty(),
            "Climbers is zero in setup_climbers. Population {}, population size {}, Climb cnt {}",
            self.population.len(),
            self.pop_cnt,
            self.climber_cnt,
        );

        debug_assert_eq!(
            climbers.len(),
            self.climber_cnt,
            "Incorrect number of climbers. Pop len {}, Pop Cnt {}",
            self.population.len(),
            self.pop_cnt,
        );

        self.population.clear();
        self.population.append(&mut climbers);

        return Ok(());
    }

    pub fn climb_kbs(&mut self, iter: usize) -> Result<()> {
        let mut climber_score = 0.0_f64;
        self.update_climb_decay(iter);

        for i in 0..self.population.len() {
            let climb_info: String = format!(
                "Keyboard: {:02}, Generation: {:05}, ID: {:07}",
                i.checked_add(1).expect("Too many climbers in climb_kbs"),
                self.population[i].get_generation(),
                self.population[i].get_id()
            );
            update_climb_info(&climb_info)?;

            // Because climb_kbs borrows self as &mut, we can't double-borrow. Clone instead
            self.population[i] = self.climb_kb(self.population[i].kb_clone());
            climber_score += self.population[i].get_score();
        }

        let avg_climber_score = climber_score / self.population.len() as f64;
        update_cur_avg(avg_climber_score)?;

        self.population.sort_by(|a, b| {
            return b
                .get_score()
                .partial_cmp(&a.get_score())
                .unwrap_or(cmp::Ordering::Equal);
        });

        if self.population[0].get_score() >= self.top_score {
            self.top_score = self.population[0].get_score();
        }

        for p in self.population.iter_mut().take(self.elite_cnt) {
            p.set_elite();
        }

        for p in self.population.iter_mut().skip(self.elite_cnt) {
            p.unset_elite();
        }

        assert_eq!(
            self.population[0].get_score(),
            self.top_score,
            "Elite lost in climb_kbs"
        );

        return Ok(());
    }

    // NOTE: Changing one key at a time works best. If you change two keys, the algorithm will find
    // bigger changes less frequently. This causes the decay to continue for about as many
    // iterations as it would if doing only one step, but fewer improvements will be found, causing
    // the improvement at the end of the hill climbing step to be lower
    fn climb_kb(&mut self, keyboard: Keyboard) -> Keyboard {
        let mut last_improvement: f64 = 0.0;
        let mut avg_improvement: f64 = 0.0;
        let mut weighted_avg: f64 = 0.0;
        let mut sum_weights: f64 = 0.0;

        let mut kb = keyboard;

        for i in 1..=100000 {
            let mut climb_kb = kb.kb_clone();
            climb_kb.table_swap(&self.swap_table, self.k_temp);
            climb_kb.eval();
            self.update_from_swap(climb_kb.get_last_swap_info());

            let this_improvement = (climb_kb.get_score() - kb.get_score()).max(0.0);
            avg_improvement = get_new_avg(this_improvement, avg_improvement, i);

            let improvement_delta = this_improvement - last_improvement;
            last_improvement = this_improvement;

            // TODO: This should be a population setting. I think you can do the weights as an
            // enum
            let this_weight = get_weight(improvement_delta);
            sum_weights *= self.climb_decay;
            let inflated_w_avg = weighted_avg * sum_weights;
            sum_weights += this_weight;
            weighted_avg = (inflated_w_avg + this_improvement * this_weight) / sum_weights;

            if climb_kb.get_score() > kb.get_score() {
                climb_kb.add_pos_iter();
                kb = climb_kb;
            }

            // Check i > 1 to paste over an edge case where the first improvement on the first
            // iteration is smaller than the unweighted mean due to floating point imprecision
            let plateauing: bool = weighted_avg < avg_improvement && i > 1;
            let i_f64 = i as f64;
            let not_starting: bool = avg_improvement <= 0.0 && i_f64 >= self.avg_climb_iter;
            if plateauing || not_starting {
                self.total_climbs += 1;
                self.avg_climb_iter = get_new_avg(i_f64, self.avg_climb_iter, self.total_climbs);

                break;
            }
        }

        return kb;
    }

    fn update_climb_decay(&mut self, iter: usize) {
        const CLAMP_VALUE: f64 = 0.999_999_999_999_999;

        self.climb_decay = (1.0 - (1.0 / iter as f64)).min(CLAMP_VALUE);
    }

    fn update_from_swap(&mut self, swap_info: (Slot, Key, Slot, Key, f64)) {
        let last_slot_a = swap_info.0;
        let last_key_a = swap_info.1;
        let last_slot_b = swap_info.2;
        let last_key_b = swap_info.3;
        let score_diff = swap_info.4;

        self.swap_table
            .update_score(last_slot_a, last_key_a, score_diff, self.score_decay);
        self.swap_table
            .update_score(last_slot_b, last_key_b, score_diff, self.score_decay);
    }

    pub fn get_top_score(&self) -> f64 {
        return self.top_score;
    }

    pub fn get_best_kb(&self) -> &Keyboard {
        if self.population[0].get_score() == self.get_top_score() {
            return &self.population[0];
        } else {
            println!(
                "id: {}, top score self: {}, population: {:?}",
                self.id,
                self.get_top_score(),
                self.population
                    .iter()
                    .map(|c| return c.get_score())
                    .collect::<Vec<f64>>(),
            );
            panic!("Neither climbers nor population contain the best score in index zero");
        }
    }

    pub fn get_id(&self) -> usize {
        return self.id;
    }

    pub fn get_avg_climb_iter(&self) -> f64 {
        return self.avg_climb_iter;
    }

    pub fn get_pop_cnt(&self) -> usize {
        return self.pop_cnt;
    }

    pub fn get_climb_cnt(&self) -> usize {
        return self.climber_cnt;
    }

    pub fn get_climb_pct(&self) -> f64 {
        return self.climber_cnt as f64 / self.pop_cnt as f64;
    }

    pub fn get_elite_cnt(&self) -> usize {
        return self.elite_cnt;
    }

    pub fn get_generation(&self) -> usize {
        return self.generation;
    }

    fn get_population(&self) -> &[Keyboard] {
        return &self.population;
    }

    pub fn get_mutation(&self) -> usize {
        return self.mutation;
    }

    pub fn get_score_decay(&self) -> f64 {
        return self.score_decay;
    }

    pub fn get_k_temp(&self) -> f64 {
        return self.k_temp;
    }

    fn get_swap_score(&self, row: usize, col: usize, key: Key) -> SwapScore {
        return self.swap_table.get_swap_score(row, col, key);
    }

    pub fn get_total_climbs(&self) -> usize {
        return self.total_climbs;
    }

    fn get_climb_decay(&self) -> f64 {
        return self.climb_decay;
    }

    pub fn set_elite(&mut self) {
        self.is_elite = true;
    }

    pub fn unset_elite(&mut self) {
        self.is_elite = false;
    }

    // pub fn is_elite(&self) -> bool {
    //     return self.is_elite;
    // }
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
    fn new() -> Self {
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

    fn get_swap_score(&self, row: usize, col: usize, key: Key) -> SwapScore {
        return self.swap_table[row][col][&key];
    }

    pub fn update_score(&mut self, slot: Slot, key: Key, new_score: f64, decay: f64) {
        let row = slot.get_row();
        let col = slot.get_col();
        let mut score = self.swap_table[row][col][&key];

        score.reweight_avg(new_score, decay);
        self.swap_table[row][col].insert(key, score);
    }

    fn replace_score(&mut self, row: usize, col: usize, key: Key, new_score: SwapScore) {
        self.swap_table[row][col].insert(key, new_score);
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

    fn from_values(score: f64, weight: f64) -> Self {
        return Self {
            w_avg: score,
            weights: weight,
        };
    }

    pub fn get_w_avg(&self) -> f64 {
        return self.w_avg;
    }

    fn get_weights(&self) -> f64 {
        return self.w_avg;
    }

    pub fn reweight_avg(&mut self, new_score: f64, decay: f64) {
        self.weights *= decay;
        let inflated_avg = self.w_avg * self.weights;
        self.weights += 1.0_f64;

        self.w_avg = (inflated_avg + new_score) / self.weights;
    }
}
