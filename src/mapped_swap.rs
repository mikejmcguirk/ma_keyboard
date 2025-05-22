use rand::{Rng as _, rngs::SmallRng};

use crate::keyboard::{Key, Slot};

pub fn select_key(rng: &mut SmallRng, values: &mut [(Slot, Key, f64)]) -> (Slot, Key, f64) {
    debug_assert!(
        !values.is_empty(),
        "Should always be candidates in select_swap"
    );

    apply_minmax(values);
    let var = get_variance(values);
    let temp = get_temp(var);
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
            v.2 = 0.0;
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

    let mut var = 0.0;
    for v in values {
        var += (v.2 - mean).powi(2);
    }

    var /= values.len() as f64;

    return var;
}

// This function assumes we are working with min/maxed weighted averages. This means the
// temperature values required to produce sharper probability distributions will be low
pub fn get_temp(var: f64) -> f64 {
    const DECAY_MIN: f64 = 0.01;
    const DECAY_MAX_PART: f64 = 0.14;
    // When normalized variance is 0.05, temp should be 0.08
    const K_TEMP: f64 = -13.862943611198906;

    debug_assert!((0.0..=0.25).contains(&var), "Var {var} invalid in get_temp");

    return DECAY_MIN + DECAY_MAX_PART * (K_TEMP * var).exp();
}

pub fn apply_softmax(values: &mut [(Slot, Key, f64)], temp: f64) {
    // NOTE: Negative temps will invert the probability curve
    debug_assert_ne!(temp, 0.0, "Temp is zero in apply_softmax");
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

    let mut total_scaled = 0.0;
    for c in values.iter_mut() {
        c.2 = (c.2 / temp).exp();
        total_scaled += c.2;
    }

    // Check if total_scaled is zero due to underflow
    if total_scaled == 0.0 {
        let uniform_prob = 1.0 / values.len() as f64;
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
    let r = rng.random_range(0.0..=1.0);

    for v in values {
        checked_score += v.2;
        if checked_score >= r {
            return *v;
        }
    }

    return *values.last().expect("Values is empty in mapped_roulette");
}
