extern crate alloc;

use alloc::collections::BTreeMap;

use rand::{Rng as _, rngs::SmallRng};

use crate::{
    population::SwapTable,
    structs::{Key, Slot},
};

// PERF: valid_loc_a doesn't need to be checked when running a basic shuffle
pub fn shuffle_check(
    valid_slots: &BTreeMap<Key, Vec<Slot>>,
    slot_a: Slot,
    key_a: Key,
    slot_b: Slot,
    key_b: Key,
) -> bool {
    let different_keys = key_a != key_b;
    let valid_loc_a = valid_slots[&key_a].contains(&slot_b);
    let valid_loc_b = valid_slots[&key_b].contains(&slot_a);
    let swappable_key_b = !valid_slots[&key_b].is_empty();

    return different_keys && valid_loc_a && valid_loc_b && swappable_key_b;
}

// Takes in references because this is called from an iterator
#[expect(clippy::trivially_copy_pass_by_ref)]
pub fn get_improvement(
    swap_table: &SwapTable,
    select_a_score: f64,
    slot_a: Slot,
    key_a: Key,
    slot_b: &Slot,
    key_b: &Key,
) -> f64 {
    let candidate_score_b = swap_table.get_score(slot_b, key_b);
    let new_select_score_a = swap_table.get_score(&slot_a, key_b);
    let new_candidate_score_b = swap_table.get_score(slot_b, &key_a);

    let total_cur_score = select_a_score + candidate_score_b;
    let total_new_score = new_select_score_a + new_candidate_score_b;

    // A higher score in the swap map for a particular key and slot means the overall
    // keyboard score improves when the key leaves the slot. We therefore want to move
    // keys to lower scoring slots where they are less likely to want to move
    let improvement = total_cur_score - total_new_score;
    return improvement;
}

pub fn select_key(
    rng: &mut SmallRng,
    values: &mut [(Slot, Key, f64)],
    k_temp: f64,
) -> (Slot, Key, f64) {
    debug_assert!(
        !values.is_empty(),
        "Should always be candidates in select_swap"
    );

    apply_minmax(values);
    let var = get_variance(values);
    let temp = get_temp(var, k_temp);
    apply_softmax(values, temp);

    let selection = mapped_roulette(rng, values);

    return selection;
}

pub fn apply_minmax(values: &mut [(Slot, Key, f64)]) {
    debug_assert!(!values.is_empty(), "Values vec is empty in apply_minmax");
    debug_assert!(
        !values.iter().any(|v| return v.2.is_nan()),
        "Input values contain at least one NaN in apply_minmax"
    );

    debug_assert!(
        values.iter().all(|v| return v.2.is_finite()),
        "Input values contain at least one infinite number in apply_minmax"
    );

    let min_val: f64 = values
        .iter()
        .fold(f64::INFINITY, |acc, v| return v.2.min(acc));

    let max_val: f64 = values
        .iter()
        .fold(f64::NEG_INFINITY, |acc, v| return acc.max(v.2));

    if max_val > min_val {
        for v in values.iter_mut() {
            v.2 = (v.2 - min_val) / (max_val - min_val);
        }
    } else {
        for v in values.iter_mut() {
            v.2 = 0.0_f64;
        }
    }
}

pub fn get_variance(values: &[(Slot, Key, f64)]) -> f64 {
    debug_assert!(!values.is_empty(), "Values vec is empty in apply_softmax");
    debug_assert!(
        !values.iter().any(|v| return v.2.is_nan()),
        "Input values contain at least one NaN in apply_softmax"
    );

    debug_assert!(
        values.iter().all(|v| return v.2.is_finite()),
        "Input values contain at least one infinite number in apply_softmax"
    );

    let mean = values.iter().map(|v| return v.2).sum::<f64>() / values.len() as f64;

    let mut var = 0.0_f64;
    for v in values {
        var += (v.2 - mean).powi(2);
    }

    var /= values.len() as f64;

    return var;
}

// This function assumes we are working with min/maxed weighted averages. This means the
// temperature values required to produce sharper probability distributions will be low
pub fn get_temp(var: f64, k_temp: f64) -> f64 {
    const DECAY_MIN: f64 = 0.01;
    const DECAY_MAX_PART: f64 = 0.19;

    debug_assert!(
        (0.0_f64..=0.25_f64).contains(&var),
        "Var {var} invalid in get_temp"
    );

    return DECAY_MIN + DECAY_MAX_PART * (k_temp * var).exp();
}

pub fn apply_softmax(values: &mut [(Slot, Key, f64)], temp: f64) {
    // NOTE: Negative temps will invert the probability curve
    debug_assert_ne!(temp, 0.0_f64, "Temp is zero in apply_softmax");
    debug_assert!(!values.is_empty(), "Values vec is empty in apply_softmax");
    debug_assert!(
        !values.iter().any(|v| return v.2.is_nan()),
        "Input values contain at least one NaN in apply_softmax"
    );

    debug_assert!(
        values.iter().all(|v| return v.2.is_finite()),
        "Input values contain at least one infinite number in apply_softmax"
    );

    #[cfg(debug_assertions)]
    {
        let max_value: f64 = values
            .iter()
            .fold(f64::NEG_INFINITY, |acc, v| return acc.max(v.2));

        debug_assert!(
            max_value / temp <= f64::MAX.ln(),
            "Max value {} / temperature {} > f64::MAX.ln() {} in apply_softmax",
            max_value,
            temp,
            f64::MAX.ln()
        );
    }

    let mut total_scaled = 0.0_f64;
    for c in values.iter_mut() {
        c.2 = (c.2 / temp).exp();
        total_scaled += c.2;
    }

    // Check if total_scaled is zero due to underflow
    if total_scaled == 0.0_f64 {
        let uniform_prob = 1.0_f64 / values.len() as f64;
        for c in values.iter_mut() {
            c.2 = uniform_prob;
        }
    } else {
        for c in values.iter_mut() {
            c.2 /= total_scaled;
        }
    }
}

fn mapped_roulette(rng: &mut SmallRng, values: &[(Slot, Key, f64)]) -> (Slot, Key, f64) {
    debug_assert!(
        !values.iter().any(|v| return v.2.is_nan()),
        "Input values contain at least one NaN in apply_softmax"
    );

    debug_assert!(
        values.iter().all(|v| return v.2.is_finite()),
        "Input values contain at least one infinite number in apply_softmax"
    );

    let mut checked_score: f64 = 0.0;
    let r = rng.random_range(0.0_f64..=1.0_f64);

    for v in values {
        checked_score += v.2;
        if checked_score >= r {
            return *v;
        }
    }

    return *values.last().expect("Values is empty in mapped_roulette");
}
