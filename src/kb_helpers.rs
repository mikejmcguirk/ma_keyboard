// **************
// **** NOTE ****
// **************

// These functions are designed to work with the keyboard struct. Because the struct's properties
// are known at compile time, these functions might not check certain edge cases

use core::cmp;

use crate::{helper_consts, kb_helper_consts};

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
                "Column {} for row {} is invalid",
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

// Because this function contains recursion, I don't want to hurt its readability. assertion checks
// must have already been performed
#[expect(clippy::arithmetic_side_effects)]
#[expect(clippy::indexing_slicing)]
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
        } else {
            kb_vec[row][col] = SPACE;
        }
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
                } else {
                    return None;
                }
            });
        })
        .collect();
}

/// # Panics
/// Any key sent to this function must be found in the valid locations Vec. Any key found in
/// the valid locations Vec must have at least one valid location
pub fn get_loc_idx(key: (u8, u8), valid_locations: &[((u8, u8), Vec<(usize, usize)>)]) -> usize {
    for (i, location) in valid_locations.iter().enumerate() {
        if location.0 == key {
            debug_assert!(
                !location.1.is_empty(),
                "Valid location index for {:?} is empty",
                key
            );

            return i;
        }
    }

    panic!("Did not find {:?} in valid locations", key);
}
