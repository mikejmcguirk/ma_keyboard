use core::cmp;

use rand::{Rng as _, rngs::SmallRng};

use crate::{
    keyboard::Keyboard,
    keys,
    population::{
        ELITE_CNT, MAX_CLIMB_PCT, MAX_K_TEMP, MAX_MUTATION, MAX_POP, MAX_SCORE_DECAY,
        MIN_CLIMB_PCT, MIN_K_TEMP, MIN_MUTATION, MIN_POP, MIN_SCORE_DECAY, MUTATION_RATE,
        Population, SwapScore, SwapTable,
    },
    structs::Key,
    swappable_keys,
};

swappable_keys!();

pub fn pop_cnt_from_parents(
    rng: &mut SmallRng,
    parent_a: &Population,
    parent_b: &Population,
    top_a_pct: f64,
) -> usize {
    return if rng.random_range(0.0..=1.0) <= MUTATION_RATE {
        rng.random_range(MIN_POP..=MAX_POP)
    } else if rng.random_range(0.0..=1.0) <= top_a_pct {
        parent_a.get_pop_cnt()
    } else {
        parent_b.get_pop_cnt()
    };
}

pub fn climb_cnt_from_parents(
    rng: &mut SmallRng,
    parent_a: &Population,
    parent_b: &Population,
    top_a_pct: f64,
    pop_cnt: usize,
) -> usize {
    let climb_pct = if rng.random_range(0.0_f64..=1.0_f64) <= MUTATION_RATE {
        rng.random_range(MIN_CLIMB_PCT..=MAX_CLIMB_PCT)
    } else if rng.random_range(0.0_f64..=1.0_f64) <= top_a_pct {
        parent_a.get_climb_pct()
    } else {
        parent_b.get_climb_pct()
    };

    let climber_cnt = (pop_cnt as f64 * climb_pct).round() as usize;

    return climber_cnt.max(ELITE_CNT);
}

pub fn new_pop_from_parents(
    rng: &mut SmallRng,
    parent_a: &Population,
    parent_b: &Population,
    pop_cnt: usize,
    top_score: f64,
) -> Vec<Keyboard> {
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
    debug_assert!(full_pop_score > 0.0_f64, "Parent populations not evaluated");

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

    for p in population.iter_mut().take(ELITE_CNT) {
        p.set_elite();
    }

    for p in population.iter_mut().skip(ELITE_CNT) {
        p.unset_elite();
    }

    let top_elite_score: f64 = population
        .iter()
        .fold(0.0_f64, |acc, p| return acc.max(p.get_score()));
    debug_assert_eq!(
        top_elite_score, top_score,
        "Elite lost after filtering population in create_child"
    );

    return population;
}

pub fn mutation_from_parents(
    rng: &mut SmallRng,
    parent_a: &Population,
    parent_b: &Population,
    top_a_pct: f64,
) -> usize {
    return if rng.random_range(0.0..=1.0) <= MUTATION_RATE {
        rng.random_range(MIN_MUTATION..=MAX_MUTATION)
    } else if rng.random_range(0.0..=1.0) <= top_a_pct {
        parent_a.get_mutation()
    } else {
        parent_b.get_mutation()
    };
}

pub fn swap_table_from_parents(
    rng: &mut SmallRng,
    parent_a: &Population,
    parent_b: &Population,
    top_a_pct: f64,
) -> SwapTable {
    let mut swap_table = SwapTable::new();

    for j in 0_usize..4_usize {
        for k in 0_usize..10_usize {
            for key_tuple in &SWAPPABLE_KEYS {
                let key = Key::from_tuple(*key_tuple);

                if rng.random_range(0.0_f64..=1.0_f64) <= MUTATION_RATE {
                    swap_table.replace_score(j, k, key, SwapScore::new());
                    continue;
                }

                let swap_score_a = parent_a.get_swap_score(j, k, key);
                let swap_score_b = parent_b.get_swap_score(j, k, key);

                let score_a = swap_score_a.get_w_avg();
                let score_b = swap_score_b.get_w_avg();
                let weight_a = swap_score_a.get_weights();
                let weight_b = swap_score_b.get_weights();

                if rng.random_range(0.0_f64..=1.0_f64) >= top_a_pct {
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

    return swap_table;
}

pub fn k_temp_from_parents(
    rng: &mut SmallRng,
    parent_a: &Population,
    parent_b: &Population,
    top_a_pct: f64,
) -> f64 {
    return if rng.random_range(0.0..=1.0) <= MUTATION_RATE {
        rng.random_range(MIN_K_TEMP..=MAX_K_TEMP)
    } else if rng.random_range(0.0..=1.0) <= top_a_pct {
        parent_a.get_k_temp()
    } else {
        parent_b.get_k_temp()
    };
}

pub fn score_decay_from_parents(
    rng: &mut SmallRng,
    parent_a: &Population,
    parent_b: &Population,
    top_a_pct: f64,
) -> f64 {
    return if rng.random_range(0.0..=1.0) <= MUTATION_RATE {
        rng.random_range(MIN_SCORE_DECAY..=MAX_SCORE_DECAY)
    } else if rng.random_range(0.0..=1.0) <= top_a_pct {
        parent_a.get_score_decay()
    } else {
        parent_b.get_score_decay()
    };
}

pub fn avg_climb_iter_from_parents(parent_a: &Population, parent_b: &Population) -> (f64, usize) {
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

    return (avg_climb_iter, total_climbs);
}
