use core::cmp;

use {
    anyhow::Result,
    rand::{Rng as _, SeedableRng as _, rngs::SmallRng},
};

use crate::{
    display::{update_best_kb, update_best_pop_dsp, update_cur_pop_dsp},
    population::Population,
    structs::IdSpawner,
};

pub struct MetaPopulation {
    rng: SmallRng,
    id_spawner: IdSpawner,
    collection: Vec<Population>,
    generation: usize,
    top_score: f64,
    elite_cnt: usize,
    pop_size: usize,
    to_remove: usize,
}

impl MetaPopulation {
    pub fn create() -> Self {
        let seed: [u8; 32] = rand::random();
        let rng = SmallRng::from_seed(seed);

        let mut id_spawner = IdSpawner::new();

        let pop_size = 20;

        let mut collection = Vec::new();
        for _ in 0..pop_size {
            let new_pop = Population::create(id_spawner.get());
            // new_pop.refill_pop();
            collection.push(new_pop);
        }

        let generation = 0;

        let top_score = 0.0;

        return Self {
            rng,
            id_spawner,
            collection,
            generation,
            top_score,
            elite_cnt: 1,
            pop_size,
            to_remove: 10,
        };
    }

    pub fn run_generation(&mut self) -> Result<()> {
        debug_assert!(self.collection.len() > 0, "len zero in run_generation");
        self.generation += 1;

        for p in self.collection.iter_mut() {
            update_cur_pop_dsp(p)?;

            p.refill_pop();
            p.eval_gen_pop()?;
            p.filter_climbers()?;
            p.climb_kbs(self.generation)?;

            if p.get_top_score() >= self.top_score {
                self.top_score = p.get_top_score();
                update_best_pop_dsp(p)?;
                update_best_kb(p.get_best_kb())?;
            }
        }

        return Ok(());
    }

    pub fn purge(&mut self) {
        self.collection.sort_by(|a, b| {
            return b
                .get_top_score()
                .partial_cmp(&a.get_top_score())
                .unwrap_or(cmp::Ordering::Equal);
        });

        for c in self.collection.iter_mut().take(self.elite_cnt) {
            c.set_elite();
        }

        for c in self.collection.iter_mut().skip(self.elite_cnt) {
            c.unset_elite();
        }

        let mut elites: Vec<Population> = Vec::new();
        for i in 0..self.elite_cnt {
            elites.push(self.collection.swap_remove(i));
        }

        let mut total_top_score = self
            .collection
            .iter()
            .fold(0.0_f64, |acc, c| return acc + c.get_top_score());

        let mut removed = 0;
        while removed < self.to_remove {
            let mut checked_score: f64 = 0.0;
            let r = self.rng.random_range(0.0_f64..=total_top_score);

            for (i, p) in self.collection.iter_mut().enumerate() {
                checked_score += p.get_top_score();
                if checked_score >= r {
                    total_top_score -= p.get_top_score();
                    self.collection.swap_remove(i);

                    removed += 1;
                    break;
                }
            }
        }

        for i in 0..elites.len() {
            self.collection.push(elites.swap_remove(i));
        }
    }

    pub fn reproduce(&mut self) {
        debug_assert_eq!(
            self.collection.len(),
            self.pop_size - self.to_remove,
            "Improper amount removed in to_reproduce",
        );

        let mut already_reproduced: Vec<Population> = Vec::new();
        let mut parents: Vec<Population> = Vec::new();
        let mut children: Vec<Population> = Vec::new();

        while (self.collection.len() + already_reproduced.len() + children.len()) < self.pop_size {
            if self.collection.len() <= 1 {
                self.collection.append(&mut already_reproduced);
            }

            let mut total_top_score = self
                .collection
                .iter()
                .fold(0.0_f64, |acc, c| return acc + c.get_top_score());

            let mut a = self.collection.len();
            let mut checked_score: f64 = 0.0;
            let r_a = self.rng.random_range(0.0_f64..=total_top_score);

            for (i, p) in self.collection.iter_mut().enumerate() {
                checked_score += p.get_top_score();
                if checked_score >= r_a {
                    total_top_score -= p.get_top_score();

                    a = i;
                    break;
                }
            }

            parents.push(self.collection.swap_remove(a));
            let mut b = self.collection.len();

            checked_score = 0.0;
            let r_b = self.rng.random_range(0.0_f64..=total_top_score);

            for (i, p) in self.collection.iter_mut().enumerate() {
                checked_score += p.get_top_score();
                if checked_score >= r_b {
                    b = i;
                    break;
                }
            }

            parents.push(self.collection.swap_remove(b));
            debug_assert_eq!(parents.len(), 2, "In reproduce");

            let child = Population::from_parents(&parents[0], &parents[1], self.id_spawner.get());

            children.push(child);
            already_reproduced.push(parents.swap_remove(1));
            already_reproduced.push(parents.swap_remove(0));
        }

        self.collection.append(&mut already_reproduced);
        self.collection.append(&mut children);
        debug_assert_eq!(self.collection.len(), self.pop_size, "in reproduce");
    }
}
