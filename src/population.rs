use std::io::{Write as _, stdout};

use {
    anyhow::{Result, anyhow},
    // rand::{Rng as _, SeedableRng as _, rngs::SmallRng},
    rand::{SeedableRng as _, rngs::SmallRng},
};

use crate::{enums::MyError, structs::Keyboard};

// TODO: I am not convinced that this architecture actually works with multi-threading, but I would
// rather work from this position than try to unspool the rough code
// NOTE: This struct is responsible for the management of the keyboard population, not to what end
// the population should be managed. Any error checking/input checking it performs should be for
// the sake of its own internal logic only
pub struct Population {
    rng: SmallRng,
    id: IdSpawner,
    pop_size: usize,
    gen_pop: Vec<Keyboard>,
    climber_cnt: usize,
    climbers: Vec<Keyboard>,
    generation: usize,
}

impl Population {
    // TODO: Long function
    pub fn create(size: Option<usize>) -> Result<Self> {
        const DEFAULT_POPULATION: usize = 100;
        const DEFAULT_CLIMB_PCT: f64 = 0.2;
        const MIN_CLIMBERS: f64 = 1.0;

        let seed: [u8; 32] = rand::random();
        let mut rng = SmallRng::from_seed(seed);

        let mut id: IdSpawner = IdSpawner::new();

        let pop_size: usize = size.unwrap_or(DEFAULT_POPULATION);
        if pop_size == 0 {
            return Err(anyhow!("Population size cannot be zero"));
        } else if pop_size % 5 != 0 {
            return Err(anyhow!(
                "ERROR: Population size {pop_size} is not a multiple of five"
            ));
        }

        let gen_pop: Vec<Keyboard> = Vec::with_capacity(pop_size);
        println!("Population initialized with a size of {pop_size}");

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
        let climber_cnt: usize = (pop_size as f64 * DEFAULT_CLIMB_PCT).max(MIN_CLIMBERS) as usize;
        let mut climbers: Vec<Keyboard> = Vec::with_capacity(climber_cnt);
        if pop_size / climber_cnt != 5 {
            return Err(anyhow!(
                "Climbers {climber_cnt} must be 20% of the general population ({pop_size})"
            ));
        }
        println!("Population will have {climber_cnt} climbers");

        // At the end of the last iteration, it is not necessary to mutate the climbers. Therefore,
        // mutating climbers is done at the beginning of each iteration. Even though creating our
        // initial population as climbers rather than general population is unintuitive, it lets us
        // transition into the main loop logic without creating a special case for the first
        // iteration
        for _ in 0..climber_cnt {
            // TODO: It is unintuitive that this would error
            let mut keyboard: Keyboard = Keyboard::make_origin(id.get())?;
            // TODO: shuffle_all should be run automatically when making an original keyboard, but
            // want to wait on the architecture to settle in before doing this
            keyboard.shuffle_all(&mut rng)?;
            climbers.push(keyboard);
        }

        return Ok(Self {
            rng,
            id,
            pop_size,
            gen_pop,
            climber_cnt,
            climbers,
            generation: 0,
        });
    }

    // NOTE: The amts argument is used to determine how many keyslots are shuffled on the keyboard.
    // Because the amts are type usize, no amt can produce invalid behavior from the standpoint of
    // the population. To see if certain amts are invalid, the keyboard struct must be checked
    pub fn mutate_climbers(&mut self, amts: (usize, usize, usize)) -> Result<()> {
        self.gen_pop.clear();
        // TODO: Extreme edge case, but this should return an error if the max usize is exceeded.
        // The caller should be able to handle this and print out the results
        self.generation += 1; // Assumes that generation is initialized at zero

        let small_amt: usize = amts.0;
        let med_amt: usize = amts.1;
        let large_amt: usize = amts.2;

        for kb in &self.climbers {
            let mut small_kb_1: Keyboard = Keyboard::mutate_kb(kb, self.generation, self.id.get());
            // TODO: shuffle_some should be a part of mutate_kb
            small_kb_1.shuffle_some(&mut self.rng, small_amt)?;
            self.gen_pop.push(small_kb_1);

            let mut small_kb_2: Keyboard = Keyboard::mutate_kb(kb, self.generation, self.id.get());
            small_kb_2.shuffle_some(&mut self.rng, small_amt)?;
            self.gen_pop.push(small_kb_2);

            let mut med_kb: Keyboard = Keyboard::mutate_kb(kb, self.generation, self.id.get());
            med_kb.shuffle_some(&mut self.rng, med_amt)?;
            self.gen_pop.push(med_kb);

            let mut large_kb: Keyboard = Keyboard::mutate_kb(kb, self.generation, self.id.get());
            large_kb.shuffle_some(&mut self.rng, large_amt)?;
            self.gen_pop.push(large_kb);

            // TODO: Clone bad
            self.gen_pop.push(kb.clone());
        }

        if self.gen_pop.len() != self.pop_size {
            return Err(anyhow!(
                "ERROR: Total population of {} does not match expected {}",
                self.gen_pop.len(),
                self.pop_size
            ));
        }

        return Ok(());
    }

    pub fn eval_gen_pop(&mut self, corpus: &[String]) -> Result<(), MyError> {
        if corpus.is_empty() {
            return Err(MyError::EmptyCorpus);
        }

        for i in 0..self.gen_pop.len() {
            // TODO: Format string should be based on digits in total population size
            print!("Evaluating Keyboard {:03}\r", i + 1);
            stdout().flush()?; // MyError handles io errors
            self.gen_pop[i].eval(corpus);
        }

        println!();

        return Ok(());
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
