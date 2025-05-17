// **************
// **** NOTE ****
// **************

// These functions are designed to work with the keyboard struct. Because the struct's properties
// are known at compile time, these functions might not check certain edge cases

use core::cmp;

use crate::{
    keyboard::KeyCompare,
    {helper_consts, kb_helper_consts, swappable_keys},
};

helper_consts!();

/// # Panics
/// Panics if the internal key data is invalid
pub fn get_key_locations() -> Vec<((u8, u8), Vec<(usize, usize)>)> {
    let mut key_vec = get_key_vec();
    key_vec.sort_by(|a, b| {
        return a
            .1
            .len()
            .partial_cmp(&b.1.len())
            .unwrap_or(cmp::Ordering::Equal);
    });

    for key in &key_vec {
        assert!(
            usize::from(key.0.0) <= ASCII_CNT,
            "Key {} is not a valid ASCII char",
            key.0.0
        );

        assert!(
            usize::from(key.0.1) <= ASCII_CNT,
            "Key {} is not a valid ASCII char",
            key.0.1
        );

        assert!(
            !key.1.is_empty(),
            "Key {:?} has no valid locations listed",
            key.0
        );

        for location in &key.1 {
            assert!(
                (NUM_ROW..=BOT_ROW).contains(&location.0),
                "Key {:?} has an invalid row of {}",
                key.0,
                location.0
            );

            assert!(
                check_col(location.0, location.1),
                "Row {} for column {} is invalid",
                location.0,
                location.1
            );
        }
    }

    return key_vec;
}

fn get_key_vec() -> Vec<((u8, u8), Vec<(usize, usize)>)> {
    return vec![
        // Number Row
        (ONE, ONE_VALID.to_vec()),
        (TWO, TWO_VALID.to_vec()),
        (THREE, THREE_VALID.to_vec()),
        (FOUR, FOUR_VALID.to_vec()),
        (FIVE, FIVE_VALID.to_vec()),
        (SIX, SIX_VALID.to_vec()),
        (SEVEN, SEVEN_VALID.to_vec()),
        (EIGHT, EIGHT_VALID.to_vec()),
        (NINE, NINE_VALID.to_vec()),
        (ZERO, ZERO_VALID.to_vec()),
        (L_BRACKET, L_BRACKET_VALID.to_vec()),
        (R_BRACKET, R_BRACKET_VALID.to_vec()),
        // Pinky Extension Symbols
        (DASH, DASH_VALID.to_vec()),
        (EQUALS, EQUALS_VALID.to_vec()),
        (F_SLASH, F_SLASH_VALID.to_vec()),
        (BACKSLASH, BACKSLASH_VALID.to_vec()),
        (NEWLINE, NEWLINE_VALID.to_vec()),
        // Alpha Area Keys
        (COMMA, alpha_slots(COMMA_INVALID.as_ref())),
        (PERIOD, alpha_slots(PERIOD_INVALID.as_ref())),
        (SEMICOLON, alpha_slots(SEMICOLON_INVALID.as_ref())),
        (QUOTE, alpha_slots(QUOTE_INVALID.as_ref())),
        (A, alpha_slots(A_INVALID.as_ref())),
        (B, alpha_slots(B_INVALID.as_ref())),
        (C, alpha_slots(C_INVALID.as_ref())),
        (D, alpha_slots(D_INVALID.as_ref())),
        (E, alpha_slots(E_INVALID.as_ref())),
        (F, alpha_slots(F_INVALID.as_ref())),
        (G, alpha_slots(G_INVALID.as_ref())),
        (H, alpha_slots(H_INVALID.as_ref())),
        (I, alpha_slots(I_INVALID.as_ref())),
        (J, alpha_slots(J_INVALID.as_ref())),
        (K, alpha_slots(K_INVALID.as_ref())),
        (L, alpha_slots(L_INVALID.as_ref())),
        (M, alpha_slots(M_INVALID.as_ref())),
        (N, alpha_slots(N_INVALID.as_ref())),
        (O, alpha_slots(O_INVALID.as_ref())),
        (P, alpha_slots(P_INVALID.as_ref())),
        (Q, alpha_slots(Q_INVALID.as_ref())),
        (R, alpha_slots(R_INVALID.as_ref())),
        (S, alpha_slots(S_INVALID.as_ref())),
        (T, alpha_slots(T_INVALID.as_ref())),
        (U, alpha_slots(U_INVALID.as_ref())),
        (V, alpha_slots(V_INVALID.as_ref())),
        (W, alpha_slots(W_INVALID.as_ref())),
        (X, alpha_slots(X_INVALID.as_ref())),
        (Y, alpha_slots(Y_INVALID.as_ref())),
        (Z, alpha_slots(Z_INVALID.as_ref())),
    ];
}

// This does not really need to call separate functions and flatten the results, but keeping this
// around as a template for when I want to start mixing and matching different groups of keys
fn alpha_slots(exclusions: &[(usize, usize)]) -> Vec<(usize, usize)> {
    let slot_groups = vec![top_row(), home_row(), bottom_row()];

    let mut slot_groups_flat: Vec<(usize, usize)> = slot_groups.into_iter().flatten().collect();
    slot_groups_flat.retain(|x| return !exclusions.contains(x));

    return slot_groups_flat;
}

// NOTE: Even though the functions below return constant arrays, leaving them wrapped in Vecs so
// the function signatures don't have to be changed if the constants are changed

fn top_row() -> Vec<(usize, usize)> {
    return DEFAULT_TOP_ROW.to_vec();
}

fn home_row() -> Vec<(usize, usize)> {
    return DEFAULT_HOME_ROW.to_vec();
}

fn bottom_row() -> Vec<(usize, usize)> {
    return DEFAULT_BOT_ROW.to_vec();
}

fn check_col(row: usize, col: usize) -> bool {
    return match row {
        NUM_ROW => (0..=NUM_ROW_CNT).contains(&col),
        TOP_ROW => (0..=TOP_ROW_CNT).contains(&col),
        HOME_ROW => (0..=HOME_ROW_CNT).contains(&col),
        BOT_ROW => (0..=BOT_ROW_CNT).contains(&col),
        _ => false,
    };
}

pub fn place_keys(
    kb_vec: &mut Vec<Vec<(u8, u8)>>,
    keys: &Vec<((u8, u8), Vec<(usize, usize)>)>,
    idx: usize,
) -> bool {
    if idx == keys.len() {
        return true;
    }

    for location in &keys[idx].1 {
        let (row, col) = *location;
        if kb_vec[row][col] != SPACE {
            continue;
        }

        kb_vec[row][col] = keys[idx].0;

        let next_idx = idx + 1;
        if place_keys(kb_vec, keys, next_idx) {
            return true;
        }

        kb_vec[row][col] = SPACE;
    }

    return false;
}

pub fn check_spaces(kb_vec: &[Vec<(u8, u8)>]) -> Vec<(usize, usize)> {
    return kb_vec
        .iter()
        .enumerate()
        .flat_map(|(i, row)| {
            return row.iter().enumerate().filter_map(move |(j, &col)| {
                if col == SPACE {
                    return Some((i, j));
                }

                return None;
            });
        })
        .collect();
}

/// # Panics
/// Panics if the input col is invalid
pub fn get_hand(col: usize) -> char {
    return match col {
        L_PINKY..=L_EXT => LEFT,
        R_EXT..=R_PIPE => RIGHT,
        _ => panic!("Col {col} is invalid in get_hand"),
    };
}

/// # Panics
/// Panics if the input col is invalid
pub fn get_finger(col: usize) -> char {
    return match col {
        L_PINKY | R_PINKY..=R_PIPE => 'p',
        L_RING | R_RING => 'r',
        L_MIDDLE | R_MIDDLE => 'm',
        L_INDEX..=R_INDEX => 'i',
        _ => panic!("Col {col} is invalid in get_hand"),
    };
}

// No blanket adjustment for any particular row. The specific code for bigrams and the
// additional code for single keys both deduct for row movement, which necessarily results in
// the algo favoring the home row
pub fn get_single_key_mult(key: (usize, usize)) -> f64 {
    let row = key.0;
    let col = key.1;
    let finger = get_finger(col);
    let mut mult = BASE_EFF;

    // Do a blanket downward adjustment rather than micro-correct in the finger comparisons
    // The ring and pinky are mostly treated the same due to different preferences per typist.
    // However, the pinky top row is given an extra penalty because the whole hand has to be
    // moved to hit it
    let ring_or_pinky = finger == RING || finger == PINKY;
    if (ring_or_pinky && row == BOT_ROW) || (finger == RING && row == TOP_ROW) {
        mult *= D_ME_B;
    } else if finger == PINKY && row == TOP_ROW {
        mult *= D_HI_B;
    }

    // The algo is too willing to put high-usage keys here
    mult *= match (row, col) {
        (HOME_ROW, 4 | 5) => D_LO_B,
        (TOP_ROW, 4) | (BOT_ROW, 5) => D_ME_B,
        (BOT_ROW, 4) | (TOP_ROW, 5) => D_HI_B,
        _ => BASE_EFF,
    };

    return mult;
}

pub fn compare_keys(
    this_key: (usize, usize),
    last_key: (usize, usize),
    is_bigram: bool,
) -> KeyCompare {
    let this_col: usize = this_key.1;
    let last_col: usize = last_key.1;
    let this_hand = get_hand(this_col);
    let last_hand = get_hand(last_col);
    if this_hand != last_hand {
        return KeyCompare::Mismatch;
    }

    let mut mult: f64 = BASE_EFF;
    mult *= check_index_ext(this_key, last_key, is_bigram);
    mult *= check_pinky_ext(this_key, last_key, is_bigram);

    let this_row: usize = this_key.0;
    let last_row: usize = last_key.0;
    let row_match: bool = this_row == last_row;
    if !row_match {
        mult *= get_row_mult(this_row, last_row, is_bigram);

        // The slope of the keyboard columns goes against the shape of the left hand
        if this_hand == LEFT && is_bigram {
            mult *= D_LO_B;
        } else if this_hand == LEFT && !is_bigram {
            mult *= D_LO_S;
        }
    }

    let this_finger = get_finger(this_col);
    let last_finger = get_finger(last_col);
    let finger_match: bool = this_finger == last_finger;
    if finger_match {
        mult *= get_base_sf_penalty(is_bigram);
        mult *= get_col_sf_penalty(this_col, last_col, is_bigram);

        let is_ring_or_pinky = this_finger == RING || this_finger == PINKY;
        if is_ring_or_pinky && is_bigram {
            mult *= D_ME_B;
        } else if is_ring_or_pinky && !is_bigram {
            mult *= D_ME_S;
        }

        return KeyCompare::Mult(mult);
    }

    if row_match {
        mult *= check_roll(this_col, last_col, is_bigram);
        return KeyCompare::Mult(mult);
    }

    mult *= check_combo(this_key, last_key, is_bigram);
    mult *= check_scissor(this_key, last_key, is_bigram);

    return KeyCompare::Mult(mult);
}

// NOTE: Assumes that both keys are on the same hand
fn get_row_mult(this_row: usize, last_row: usize, is_bigram: bool) -> f64 {
    let row_diff = this_row.abs_diff(last_row);

    return match (row_diff, is_bigram) {
        (1, true) => D_LO_B,
        (2, true) => D_ME_B,
        (3, true) => D_HI_B,
        (1, false) => D_LO_S,
        (2, false) => D_ME_S,
        (3, false) => D_HI_S,
        _ => 1.0,
    };
}

fn check_index_ext(this: (usize, usize), last: (usize, usize), is_bigram: bool) -> f64 {
    debug_assert_eq!(
        get_hand(this.1),
        get_hand(last.1),
        "Rows {} and {} are on different hands",
        this.1,
        last.1
    );

    return match (this.0, this.1, last.0, last.1, is_bigram) {
        // T and 5 (Not penalized. No more movement than hitting R and 4)
        (TOP_ROW | NUM_ROW, L_EXT, _, _, true | false)
        | (_, _, TOP_ROW | NUM_ROW, L_EXT, true | false) => BASE_EFF,
        // G, H, and N
        (HOME_ROW, L_EXT | R_EXT, _, _, true)
        | (_, _, HOME_ROW, L_EXT | R_EXT, true)
        | (BOT_ROW, R_EXT, _, _, true)
        | (_, _, BOT_ROW, R_EXT, true) => D_LO_B,
        (HOME_ROW, L_EXT | R_EXT, _, _, false)
        | (_, _, HOME_ROW, L_EXT | R_EXT, false)
        | (BOT_ROW, R_EXT, _, _, false)
        | (_, _, BOT_ROW, R_EXT, false) => D_LO_S,
        // B and Y
        (BOT_ROW, L_EXT, _, _, true)
        | (_, _, BOT_ROW, L_EXT, true)
        | (TOP_ROW, R_EXT, _, _, true)
        | (_, _, TOP_ROW, R_EXT, true) => D_HI_B,
        (BOT_ROW, L_EXT, _, _, false)
        | (_, _, BOT_ROW, L_EXT, false)
        | (TOP_ROW, R_EXT, _, _, false)
        | (_, _, TOP_ROW, R_EXT, false) => D_HI_S,
        // 6
        (NUM_ROW, R_EXT, _, _, true) | (_, _, NUM_ROW, R_EXT, true) => D_BU_B,
        (NUM_ROW, R_EXT, _, _, false) | (_, _, NUM_ROW, R_EXT, false) => D_BU_S,
        _ => BASE_EFF,
    };
}

fn check_pinky_ext(this: (usize, usize), last: (usize, usize), is_bigram: bool) -> f64 {
    debug_assert_eq!(
        get_hand(this.1),
        get_hand(last.1),
        "Rows {} and {} are on different hands",
        this.1,
        last.1
    );

    return match (this.0, this.1, last.0, last.1, is_bigram) {
        (NUM_ROW, R_SYMBOL | R_NETHER, _, _, true)
        | (_, _, NUM_ROW, R_SYMBOL | R_NETHER, true)
        | (TOP_ROW, R_PIPE, _, _, true)
        | (_, _, TOP_ROW, R_PIPE, true) => D_BU_B,
        (TOP_ROW, R_SYMBOL, _, _, true)
        | (_, _, TOP_ROW, R_SYMBOL, true)
        | (HOME_ROW, R_NETHER, _, _, true)
        | (_, _, HOME_ROW, R_NETHER, true) => D_ME_B,
        (TOP_ROW, R_NETHER, _, _, true) | (_, _, TOP_ROW, R_NETHER, true) => D_HI_B,
        (HOME_ROW, R_SYMBOL, _, _, true) | (_, _, HOME_ROW, R_SYMBOL, true) => D_LO_B,
        (NUM_ROW, R_SYMBOL | R_NETHER, _, _, false)
        | (_, _, NUM_ROW, R_SYMBOL | R_NETHER, false)
        | (TOP_ROW, R_PIPE, _, _, false)
        | (_, _, TOP_ROW, R_PIPE, false) => D_BU_S,
        (TOP_ROW, R_SYMBOL, _, _, false)
        | (_, _, TOP_ROW, R_SYMBOL, false)
        | (HOME_ROW, R_NETHER, _, _, false)
        | (_, _, HOME_ROW, R_NETHER, false) => D_ME_S,
        (TOP_ROW, R_NETHER, _, _, false) | (_, _, TOP_ROW, R_NETHER, false) => D_HI_S,
        (HOME_ROW, R_SYMBOL, _, _, false) | (_, _, HOME_ROW, R_SYMBOL, false) => D_LO_S,
        _ => 1.0,
    };
}

fn check_roll(this_col: usize, last_col: usize, is_bigram: bool) -> f64 {
    debug_assert_eq!(
        get_hand(this_col),
        get_hand(last_col),
        "cols {this_col} and {last_col} are on different hands"
    );

    debug_assert_ne!(
        this_col, last_col,
        "{this_col} and {last_col} are the same col"
    );

    let this_dist = get_center_dist(this_col);
    let last_dist = get_center_dist(last_col);
    if this_dist >= last_dist {
        return BASE_EFF;
    }

    if is_bigram {
        return I_ME_B;
    }

    return I_ME_S;
}

fn get_base_sf_penalty(is_last: bool) -> f64 {
    if is_last {
        return D_ME_B;
    }

    return D_ME_S;
}

fn get_col_sf_penalty(this_col: usize, last_col: usize, last: bool) -> f64 {
    debug_assert_eq!(
        get_finger(this_col),
        get_finger(last_col),
        "ERROR: {this_col} and {last_col} are on different fingers when getting SF penalty",
    );

    let col_diff = this_col.abs_diff(last_col);
    return match (col_diff, last) {
        (1, true) => D_LO_B,
        (2, true) => D_ME_B,
        (3, true) => D_HI_B,
        (4, true) => D_BU_B,
        (1, false) => D_LO_S,
        (2, false) => D_ME_S,
        (3, false) => D_HI_S,
        (4, false) => D_BU_S,
        _ => 1.0,
    };
}

fn get_center_dist(col: usize) -> usize {
    debug_assert!(
        (L_PINKY..=R_PIPE).contains(&col),
        "Col {col} invalid in get_center_dist"
    );

    return if col <= L_EXT {
        L_EXT
            .checked_sub(col)
            .expect("{L_EXT} must be greater than {col}")
    } else {
        col.checked_sub(R_EXT)
            .expect("{col} must be greater than {R_EXT}")
    };
}

/// # Panics
/// Panics if the rows of each key are the same
fn check_combo(this_key: (usize, usize), last_key: (usize, usize), is_bigram: bool) -> f64 {
    let this_row: usize = this_key.0;
    let last_row: usize = last_key.0;
    let this_finger = get_finger(this_key.1);
    let last_finger = get_finger(last_key.1);

    let (top, bot): (char, char) = match this_row.cmp(&last_row) {
        cmp::Ordering::Greater => (this_finger, last_finger),
        cmp::Ordering::Less => (last_finger, this_finger),
        cmp::Ordering::Equal => panic!("Trying to get combo of equal rows"),
    };

    if bot == INDEX || top == MIDDLE || (top == RING && bot == PINKY) {
        return BASE_EFF;
    } else if is_bigram {
        return D_ME_B;
    }

    return D_ME_S;
}

// NOTE: I've seen "non-adjacent" scissors described before, but that should be possible to
// handle using the normal rules
fn check_scissor(this_key: (usize, usize), last_key: (usize, usize), is_bigram: bool) -> f64 {
    let this_col: usize = this_key.1;
    let last_col: usize = last_key.1;

    debug_assert_eq!(
        get_hand(this_col),
        get_hand(last_col),
        "Same hands when checking for scissor",
    );

    if this_col.abs_diff(last_col) > 1 {
        return 1.0;
    }

    let this_row: usize = this_key.0;
    let last_row: usize = last_key.0;

    debug_assert_ne!(this_row, last_row, "Same rows when checking for scissor",);

    let hand = get_hand(this_col);
    // Left-handed scissors are penalized beyond the base left-hand movement deduction because,
    // unlike right-handed scissors, you have to actually rock your hand to hit them
    return match (this_row.abs_diff(last_row), hand, is_bigram) {
        (2, RIGHT, true) => D_ME_B,
        (2, RIGHT, false) => D_ME_S,
        (3, RIGHT, true) | (2, LEFT, true) => D_HI_B,
        (3, RIGHT, false) | (2, LEFT, false) => D_HI_S,
        (3, LEFT, true) => D_BU_B,
        (3, LEFT, false) => D_BU_S,
        _ => 1.0,
    };
}

pub fn check_key_no_hist(key: (usize, usize)) -> f64 {
    let mut mult = BASE_EFF;

    let col = key.1;
    if get_hand(col) == LEFT {
        mult *= D_LO_B;
    }

    let row = key.0;
    let dist = row.abs_diff(HOME_ROW);
    debug_assert!(
        (NUM_ROW..=BOT_ROW).contains(&key.0),
        "Row {row} is invalid when checking home distance",
    );

    if dist == 1 {
        return mult * D_LO_B;
    }
    if dist == 2 {
        return mult * D_ME_B;
    }

    return mult;
}
