use core::cmp;

use crate::{
    base_eff, edge_cols, home_row,
    keyboard::{Finger, Hand, KeyCompare},
    most_rows, obscure_cols, scoring,
    structs::Slot,
};

scoring!();
most_rows!();
home_row!();
// cols!();
edge_cols!();
obscure_cols!();

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
    let this_row = this_slot.get_row();
    let last_row = last_slot.get_row();
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
    let this_row = this_slot.get_row();
    let last_row = last_slot.get_row();
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

    let this_col = this_slot.get_col();
    let last_col = last_slot.get_col();
    debug_assert!(
        this_col.abs_diff(last_col) > 0,
        "{this_col} and {last_col} are the same in check_scissor"
    );

    if this_col.abs_diff(last_col) > 1 {
        return 1.0;
    }

    let this_row = this_slot.get_row();
    let last_row = last_slot.get_row();

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
