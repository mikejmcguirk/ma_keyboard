// **************
// **** NOTE ****
// **************

// These functions are designed to work with the keyboard struct. Because the struct's properties
// are known at compile time, these functions might not check certain edge cases

use {core::cmp, std::collections::BTreeMap};

use crate::{
    keyboard::{Finger, Hand, Key, KeyCompare, Slot},
    {helper_consts, kb_helper_consts, swappable_keys},
};

helper_consts!();

pub fn get_valid_key_locs_sorted() -> Vec<(Key, Vec<Slot>)> {
    let mut key_locs: Vec<(Key, Vec<Slot>)> = vec![
        // Number Row
        (Key::from_tuple(ONE), make_slot_vec(&ONE_VALID)),
        (Key::from_tuple(TWO), make_slot_vec(&TWO_VALID)),
        (Key::from_tuple(THREE), make_slot_vec(&THREE_VALID)),
        (Key::from_tuple(FOUR), make_slot_vec(&FOUR_VALID)),
        (Key::from_tuple(FIVE), make_slot_vec(&FIVE_VALID)),
        (Key::from_tuple(SIX), make_slot_vec(&SIX_VALID)),
        (Key::from_tuple(SEVEN), make_slot_vec(&SEVEN_VALID)),
        (Key::from_tuple(EIGHT), make_slot_vec(&EIGHT_VALID)),
        (Key::from_tuple(NINE), make_slot_vec(&NINE_VALID)),
        (Key::from_tuple(ZERO), make_slot_vec(&ZERO_VALID)),
        (Key::from_tuple(L_BRACKET), make_slot_vec(&L_BRACKET_VALID)),
        (Key::from_tuple(R_BRACKET), make_slot_vec(&R_BRACKET_VALID)),
        // Pinky Extension Symbols
        (Key::from_tuple(DASH), make_slot_vec(&DASH_VALID)),
        (Key::from_tuple(EQUALS), make_slot_vec(&EQUALS_VALID)),
        (Key::from_tuple(F_SLASH), make_slot_vec(&F_SLASH_VALID)),
        (Key::from_tuple(BACKSLASH), make_slot_vec(&BACKSLASH_VALID)),
        (Key::from_tuple(NEWLINE), make_slot_vec(&NEWLINE_VALID)),
        // Alpha Area Keys
        (
            Key::from_tuple(COMMA),
            alpha_slots_tree(COMMA_INVALID.as_ref()),
        ),
        (
            Key::from_tuple(PERIOD),
            alpha_slots_tree(PERIOD_INVALID.as_ref()),
        ),
        (
            Key::from_tuple(SEMICOLON),
            alpha_slots_tree(SEMICOLON_INVALID.as_ref()),
        ),
        (
            Key::from_tuple(QUOTE),
            alpha_slots_tree(QUOTE_INVALID.as_ref()),
        ),
        (Key::from_tuple(A), alpha_slots_tree(A_INVALID.as_ref())),
        (Key::from_tuple(B), alpha_slots_tree(B_INVALID.as_ref())),
        (Key::from_tuple(C), alpha_slots_tree(C_INVALID.as_ref())),
        (Key::from_tuple(D), alpha_slots_tree(D_INVALID.as_ref())),
        (Key::from_tuple(E), alpha_slots_tree(E_INVALID.as_ref())),
        (Key::from_tuple(F), alpha_slots_tree(F_INVALID.as_ref())),
        (Key::from_tuple(G), alpha_slots_tree(G_INVALID.as_ref())),
        (Key::from_tuple(H), alpha_slots_tree(H_INVALID.as_ref())),
        (Key::from_tuple(I), alpha_slots_tree(I_INVALID.as_ref())),
        (Key::from_tuple(J), alpha_slots_tree(J_INVALID.as_ref())),
        (Key::from_tuple(K), alpha_slots_tree(K_INVALID.as_ref())),
        (Key::from_tuple(L), alpha_slots_tree(L_INVALID.as_ref())),
        (Key::from_tuple(M), alpha_slots_tree(M_INVALID.as_ref())),
        (Key::from_tuple(N), alpha_slots_tree(N_INVALID.as_ref())),
        (Key::from_tuple(O), alpha_slots_tree(O_INVALID.as_ref())),
        (Key::from_tuple(P), alpha_slots_tree(P_INVALID.as_ref())),
        (Key::from_tuple(Q), alpha_slots_tree(Q_INVALID.as_ref())),
        (Key::from_tuple(R), alpha_slots_tree(R_INVALID.as_ref())),
        (Key::from_tuple(S), alpha_slots_tree(S_INVALID.as_ref())),
        (Key::from_tuple(T), alpha_slots_tree(T_INVALID.as_ref())),
        (Key::from_tuple(U), alpha_slots_tree(U_INVALID.as_ref())),
        (Key::from_tuple(V), alpha_slots_tree(V_INVALID.as_ref())),
        (Key::from_tuple(W), alpha_slots_tree(W_INVALID.as_ref())),
        (Key::from_tuple(X), alpha_slots_tree(X_INVALID.as_ref())),
        (Key::from_tuple(Y), alpha_slots_tree(Y_INVALID.as_ref())),
        (Key::from_tuple(Z), alpha_slots_tree(Z_INVALID.as_ref())),
    ];

    key_locs.sort_by(|a, b| {
        return a
            .1
            .len()
            .partial_cmp(&b.1.len())
            .unwrap_or(cmp::Ordering::Equal);
    });

    return key_locs;
}

// This does not really need to call separate functions and flatten the results, but keeping this
// around as a template for when I want to start mixing and matching different groups of keys
fn alpha_slots_tree(exclusions: &[(usize, usize)]) -> Vec<Slot> {
    let slot_groups = vec![top_row_tree(), home_row_tree(), bottom_row_tree()];

    let mut slot_groups_flat: Vec<Slot> = slot_groups.into_iter().flatten().collect();
    let slot_exclusions: Vec<Slot> = exclusions
        .iter()
        .map(|x| return Slot::from_tuple(*x))
        .collect();
    slot_groups_flat.retain(|x| return !slot_exclusions.contains(x));

    return slot_groups_flat;
}

// NOTE: Even though the functions below return constant arrays, leaving them wrapped in Vecs so
// the function signatures don't have to be changed if the constants are changed

fn top_row_tree() -> Vec<Slot> {
    return make_slot_vec(&DEFAULT_TOP_ROW);
}

fn home_row_tree() -> Vec<Slot> {
    return make_slot_vec(&DEFAULT_HOME_ROW);
}

fn bottom_row_tree() -> Vec<Slot> {
    return make_slot_vec(&DEFAULT_BOT_ROW);
}

fn make_slot_vec(input: &[(usize, usize)]) -> Vec<Slot> {
    return input.iter().map(|i| return Slot::from_tuple(*i)).collect();
}

pub fn check_col(row: usize, col: usize) -> bool {
    return match row {
        NUM_ROW => (0..=NUM_ROW_CNT).contains(&col),
        TOP_ROW => (0..=TOP_ROW_CNT).contains(&col),
        HOME_ROW => (0..=HOME_ROW_CNT).contains(&col),
        BOT_ROW => (0..=BOT_ROW_CNT).contains(&col),
        _ => false,
    };
}

pub fn place_keys(
    slots: &mut BTreeMap<Slot, Key>,
    valid_slots: &Vec<(Key, Vec<Slot>)>,
    idx: usize,
) -> bool {
    if idx == valid_slots.len() {
        return true;
    }

    for slot in &valid_slots[idx].1 {
        if slots.contains_key(slot) {
            continue;
        }

        let key: Key = valid_slots[idx].0;
        slots.insert(*slot, key);

        let next_idx = idx + 1;
        if place_keys(slots, valid_slots, next_idx) {
            return true;
        }

        slots.remove(slot);
    }

    return false;
}

pub fn global_adjustments(slot: Slot) -> f64 {
    let mut mult = BASE_EFF;
    let finger = Finger::from_slot(slot);
    let row = slot.get_row();

    // The algo doesn't intrinsically know these fingers are less dexterous
    // Top row pinky gets extra deduction because it requires hand movement
    let ring_or_pinky = finger == Finger::Ring || finger == Finger::Pinky;
    if (ring_or_pinky && row == BOT_ROW) || (finger == Finger::Ring && row == TOP_ROW) {
        mult *= D_LO_B;
    } else if finger == Finger::Pinky && row == TOP_ROW {
        mult *= D_ME_B;
    }

    return mult;
}

pub fn compare_slots(this_slot: Slot, last_slot: Slot, is_bigram: bool) -> KeyCompare {
    let this_hand = Hand::from_slot(this_slot);
    let last_hand = Hand::from_slot(last_slot);
    if this_hand != last_hand {
        return KeyCompare::Mismatch;
    }

    let mut mult = BASE_EFF;
    mult *= check_index_ext(this_slot, last_slot, is_bigram);
    mult *= check_pinky_ext(this_slot, last_slot, is_bigram);
    mult *= check_num_ext(this_slot, last_slot, is_bigram);

    let this_finger = Finger::from_slot(this_slot);
    let last_finger = Finger::from_slot(last_slot);
    let finger_match: bool = this_finger == last_finger;
    let this_row: usize = this_slot.get_row();
    let last_row: usize = last_slot.get_row();
    let row_match: bool = this_row == last_row;

    if finger_match {
        mult *= get_base_sf_penalty(is_bigram);
        mult *= get_col_sf_penalty(this_slot, last_slot, is_bigram);

        if !row_match {
            mult *= get_row_mult(this_slot, last_slot, is_bigram);
        }

        return KeyCompare::Mult(mult);
    }

    if !row_match {
        mult *= get_row_mult(this_slot, last_slot, is_bigram);

        mult *= check_combo(this_slot, last_slot, is_bigram);
        mult *= check_scissor(this_slot, last_slot, is_bigram);

        return KeyCompare::Mult(mult);
    }

    mult *= check_roll(this_slot, last_slot, is_bigram);

    return KeyCompare::Mult(mult);
}

fn get_row_mult(this_slot: Slot, last_slot: Slot, is_bigram: bool) -> f64 {
    let this_hand = Hand::from_slot(this_slot);
    let last_hand = Hand::from_slot(last_slot);
    debug_assert_eq!(
        this_hand,
        last_hand,
        "Cols {} and {} are on different hands",
        this_slot.get_col(),
        last_slot.get_col()
    );

    let mut mult = BASE_EFF;
    // The slope of the keys works against the left hand
    if this_hand == Hand::Left && is_bigram {
        mult *= D_LO_B;
    } else if this_hand == Hand::Left && !is_bigram {
        mult *= D_LO_S;
    }

    let this_row = this_slot.get_row();
    let last_row = last_slot.get_row();
    debug_assert_ne!(
        this_row, last_row,
        "Rows {this_row} and {last_row} are the same in get_row_mult"
    );

    let row_diff = this_row.abs_diff(last_row);
    return match (row_diff, is_bigram) {
        (1, true) => mult * D_LO_B,
        (2, true) => mult * D_ME_B,
        (3, true) => mult * D_HI_B,
        (1, false) => mult * D_LO_S,
        (2, false) => mult * D_ME_S,
        (3, false) => mult * D_HI_S,
        _ => BASE_EFF,
    };
}

fn check_index_ext(this_slot: Slot, last_slot: Slot, is_bigram: bool) -> f64 {
    debug_assert_eq!(
        Hand::from_slot(this_slot),
        Hand::from_slot(last_slot),
        "Cols {} and {} are on different hands",
        this_slot.get_col(),
        last_slot.get_col()
    );

    return match (
        this_slot.get_row(),
        this_slot.get_col(),
        last_slot.get_row(),
        last_slot.get_col(),
        is_bigram,
    ) {
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

fn check_pinky_ext(this_slot: Slot, last_slot: Slot, is_bigram: bool) -> f64 {
    debug_assert_eq!(
        Hand::from_slot(this_slot),
        Hand::from_slot(last_slot),
        "Cols {} and {} are on different hands",
        this_slot.get_col(),
        last_slot.get_col()
    );

    return match (
        this_slot.get_row(),
        this_slot.get_col(),
        last_slot.get_row(),
        last_slot.get_col(),
        is_bigram,
    ) {
        // '
        (HOME_ROW, R_SYMBOL, _, _, true) | (_, _, HOME_ROW, R_SYMBOL, true) => D_LO_B,
        (HOME_ROW, R_SYMBOL, _, _, false) | (_, _, HOME_ROW, R_SYMBOL, false) => D_LO_S,
        // [ or \n
        (TOP_ROW, R_SYMBOL, _, _, true)
        | (_, _, TOP_ROW, R_SYMBOL, true)
        | (HOME_ROW, R_NETHER, _, _, true)
        | (_, _, HOME_ROW, R_NETHER, true) => D_ME_B,
        (TOP_ROW, R_SYMBOL, _, _, false)
        | (_, _, TOP_ROW, R_SYMBOL, false)
        | (HOME_ROW, R_NETHER, _, _, false)
        | (_, _, HOME_ROW, R_NETHER, false) => D_ME_S,
        // ]
        (TOP_ROW, R_NETHER, _, _, true) | (_, _, TOP_ROW, R_NETHER, true) => D_HI_B,
        (TOP_ROW, R_NETHER, _, _, false) | (_, _, TOP_ROW, R_NETHER, false) => D_HI_S,
        // -/= or |
        (NUM_ROW, R_SYMBOL | R_NETHER, _, _, true)
        | (_, _, NUM_ROW, R_SYMBOL | R_NETHER, true)
        | (TOP_ROW, R_PIPE, _, _, true)
        | (_, _, TOP_ROW, R_PIPE, true) => D_BU_B,
        (NUM_ROW, R_SYMBOL | R_NETHER, _, _, false)
        | (_, _, NUM_ROW, R_SYMBOL | R_NETHER, false)
        | (TOP_ROW, R_PIPE, _, _, false)
        | (_, _, TOP_ROW, R_PIPE, false) => D_BU_S,
        _ => 1.0,
    };
}

fn check_num_ext(this_slot: Slot, last_slot: Slot, is_bigram: bool) -> f64 {
    debug_assert_eq!(
        Hand::from_slot(this_slot),
        Hand::from_slot(last_slot),
        "Cols {} and {} are on different hands",
        this_slot.get_col(),
        last_slot.get_col()
    );

    return match (this_slot.get_row(), last_slot.get_row(), is_bigram) {
        (NUM_ROW, _, true) | (_, NUM_ROW, true) => D_BU_B,
        (NUM_ROW, _, false) | (_, NUM_ROW, false) => D_BU_S,
        _ => BASE_EFF,
    };
}

fn check_roll(this_slot: Slot, last_slot: Slot, is_bigram: bool) -> f64 {
    debug_assert_eq!(
        Hand::from_slot(this_slot),
        Hand::from_slot(last_slot),
        "cols {} and {} are on different hands",
        this_slot.get_col(),
        last_slot.get_col()
    );

    debug_assert_ne!(
        this_slot.get_col(),
        last_slot.get_col(),
        "{} and {} are the same col",
        this_slot.get_col(),
        last_slot.get_col()
    );

    let this_dist = get_center_dist(this_slot);
    let last_dist = get_center_dist(last_slot);
    if this_dist >= last_dist {
        return BASE_EFF;
    }

    if is_bigram {
        return I_LO_B;
    }

    return I_LO_S;
}

fn get_base_sf_penalty(is_last: bool) -> f64 {
    if is_last {
        return D_LO_B;
    }

    return D_LO_S;
}

fn get_col_sf_penalty(this_slot: Slot, last_slot: Slot, last: bool) -> f64 {
    let this_col = this_slot.get_col();
    let last_col = last_slot.get_col();

    debug_assert_eq!(
        Finger::from_slot(this_slot),
        Finger::from_slot(last_slot),
        "ERROR: Cols {this_col} and {last_col} are on different fingers when getting SF penalty",
    );

    let col_diff = this_col.abs_diff(last_col);
    return match (col_diff, last) {
        (1, true) => D_ME_B,
        (2, true) => D_HI_B,
        (3, true) => D_BU_B,
        (1, false) => D_ME_S,
        (2, false) => D_HI_S,
        (3, false) => D_BU_S,
        _ => 1.0,
    };
}

fn get_center_dist(slot: Slot) -> usize {
    debug_assert!(
        (L_PINKY..=R_PIPE).contains(&slot.get_col()),
        "Col {} invalid in get_center_dist",
        slot.get_col()
    );

    return if slot.get_col() <= L_EXT {
        L_EXT
            .checked_sub(slot.get_col())
            .expect("{L_EXT} must be greater than {col}")
    } else {
        slot.get_col()
            .checked_sub(R_EXT)
            .expect("{col} must be greater than {R_EXT}")
    };
}

/// # Panics
/// Panics if the rows of each key are the same
fn check_combo(this_slot: Slot, last_slot: Slot, is_bigram: bool) -> f64 {
    let this_row: usize = this_slot.get_row();
    let last_row: usize = last_slot.get_row();
    let this_finger = Finger::from_slot(this_slot);
    let last_finger = Finger::from_slot(last_slot);
    debug_assert_ne!(
        this_finger,
        last_finger,
        "Cols {} and {} are on the same finger",
        this_slot.get_col(),
        last_slot.get_col()
    );

    let (top, bot): (Finger, Finger) = match this_row.cmp(&last_row) {
        cmp::Ordering::Greater => (this_finger, last_finger),
        cmp::Ordering::Less => (last_finger, this_finger),
        cmp::Ordering::Equal => panic!("Trying to get combo of equal rows"),
    };

    if bot == Finger::Index
        || top == Finger::Middle
        || (top == Finger::Ring && bot == Finger::Pinky)
    {
        if is_bigram {
            return I_LO_B;
        } else {
            return I_LO_S;
        }
    }

    if is_bigram {
        return D_ME_B;
    }

    return D_ME_S;
}

// NOTE: I've seen "non-adjacent" scissors described before, but that should be possible to
// handle using the normal rules
fn check_scissor(this_slot: Slot, last_slot: Slot, is_bigram: bool) -> f64 {
    debug_assert_eq!(
        Hand::from_slot(this_slot),
        Hand::from_slot(last_slot),
        "Same hands when checking for scissor",
    );

    let this_col: usize = this_slot.get_col();
    let last_col: usize = last_slot.get_col();
    debug_assert!(
        this_col.abs_diff(last_col) > 0,
        "{this_col} and {last_col} are the same in check_scissor"
    );

    if this_col.abs_diff(last_col) > 1 {
        return 1.0;
    }

    let this_row: usize = this_slot.get_row();
    let last_row: usize = last_slot.get_row();

    debug_assert_ne!(this_row, last_row, "Same rows when checking for scissor",);

    let hand = Hand::from_slot(this_slot);
    // Left-handed scissors are penalized beyond the base left-hand movement deduction because,
    // unlike right-handed scissors, you have to actually rock your hand to hit them
    return match (this_row.abs_diff(last_row), hand, is_bigram) {
        (2, Hand::Right, true) => D_ME_B,
        (2, Hand::Right, false) => D_ME_S,
        (3, Hand::Right, true) | (2, Hand::Left, true) => D_HI_B,
        (3, Hand::Right, false) | (2, Hand::Left, false) => D_HI_S,
        (3, Hand::Left, true) => D_BU_B,
        (3, Hand::Left, false) => D_BU_S,
        _ => 1.0,
    };
}

pub fn check_key_no_hist(slot: Slot) -> f64 {
    let mut mult = BASE_EFF;

    let row = slot.get_row();
    debug_assert!(
        (NUM_ROW..=BOT_ROW).contains(&slot.get_row()),
        "Row {row} is invalid when checking home distance",
    );

    let row_dist = row.abs_diff(HOME_ROW);
    if row_dist == 1 {
        return mult * D_LO_B;
    } else if row_dist == 2 {
        return mult * D_ME_B;
    }

    if row_dist > 0 && Hand::from_slot(slot) == Hand::Left {
        mult *= D_LO_B;
    }

    let col = slot.get_col();
    if col == L_EXT || col == R_EXT {
        mult *= D_LO_B;
    } else if col == R_SYMBOL {
        mult *= D_ME_B;
    } else if col == R_NETHER {
        mult *= D_HI_B;
    } else if col == R_PIPE {
        mult *= D_BU_B;
    }

    return mult;
}
