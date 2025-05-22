// **************
// **** NOTE ****
// **************

// These functions are designed to work with the keyboard struct. Because the struct's properties
// are known at compile time, these functions might not check certain edge cases

extern crate alloc;

use {alloc::collections::BTreeMap, core::cmp};

use rand::{Rng as _, rngs::SmallRng};

use crate::{
    cols, edge_cols, home_row, keys,
    mapped_swap::select_key,
    most_cols, most_rows, obscure_cols,
    population::SwapTable,
    row_cnts, rows, static_keys,
    structs::{Key, Slot},
    valid_locations,
};

rows!();
valid_locations!();

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

pub fn get_swappable_keys(swappable_keys: &[(u8, u8); 30]) -> Vec<Key> {
    return swappable_keys
        .iter()
        .copied()
        .map(|k| return Key::from_tuple(k))
        .collect();
}

pub fn get_static_keys(
    swappable_keys: &[Key],
    valid_locs: &[(Key, Vec<Slot>)],
) -> Vec<(Key, Vec<Slot>)> {
    return valid_locs
        .iter()
        .filter_map(|v| {
            if swappable_keys.contains(&v.0) {
                return None;
            } else {
                return Some((v.0, v.1.clone()));
            }
        })
        .collect();
}

pub fn place_keys_from_table(
    rng: &mut SmallRng,
    slots: &mut Vec<Slot>,
    keys: &mut Vec<Key>,
    swap_table: &SwapTable,
    key_slots: &mut BTreeMap<Slot, Key>,
    valid_slots: &BTreeMap<Key, Vec<Slot>>,
) -> bool {
    if slots.is_empty() && keys.is_empty() {
        return true;
    }

    debug_assert_eq!(
        slots.len(),
        keys.len(),
        "Slots and keys have different lengths in place_keys_from_table"
    );

    let slot_idx = rng.random_range(0..slots.len());
    let slot = slots[slot_idx];
    let slot_info = swap_table.get_slot_info(slot);
    let mut candidates: Vec<(Slot, Key, f64)> = Vec::new();

    for info in slot_info {
        let key = info.0;
        if !keys.contains(key) {
            continue;
        }

        let these_valid_slots: &Vec<Slot> = &valid_slots[key];
        if !these_valid_slots.contains(&slot) {
            continue;
        }

        candidates.push((slot, *info.0, info.1.get_w_avg()));
    }

    if candidates.is_empty() {
        return false;
    }

    let select_key = select_key(rng, &mut candidates);
    key_slots.insert(select_key.0, select_key.1);

    slots.remove(slot_idx);

    let key_idx = keys
        .iter()
        .position(|k| return *k == select_key.1)
        .expect("Should not have pulled a missing key");
    keys.remove(key_idx);

    if place_keys_from_table(rng, slots, keys, swap_table, key_slots, valid_slots) {
        return true;
    }

    slots.push(slot);
    keys.push(select_key.1);

    return false;
}

pub fn place_qwerty_keys(key_slots: &mut BTreeMap<Slot, Key>) {
    key_slots.insert(Slot::from_tuple((0, 0)), Key::from_tuple(ONE));
    key_slots.insert(Slot::from_tuple((0, 1)), Key::from_tuple(TWO));
    key_slots.insert(Slot::from_tuple((0, 2)), Key::from_tuple(THREE));
    key_slots.insert(Slot::from_tuple((0, 3)), Key::from_tuple(FOUR));
    key_slots.insert(Slot::from_tuple((0, 4)), Key::from_tuple(FIVE));
    key_slots.insert(Slot::from_tuple((0, 5)), Key::from_tuple(SIX));
    key_slots.insert(Slot::from_tuple((0, 6)), Key::from_tuple(SEVEN));
    key_slots.insert(Slot::from_tuple((0, 7)), Key::from_tuple(EIGHT));
    key_slots.insert(Slot::from_tuple((0, 8)), Key::from_tuple(NINE));
    key_slots.insert(Slot::from_tuple((0, 9)), Key::from_tuple(ZERO));
    key_slots.insert(Slot::from_tuple((0, 10)), Key::from_tuple(DASH));
    key_slots.insert(Slot::from_tuple((0, 11)), Key::from_tuple(EQUALS));

    key_slots.insert(Slot::from_tuple((1, 0)), Key::from_tuple(Q));
    key_slots.insert(Slot::from_tuple((1, 1)), Key::from_tuple(W));
    key_slots.insert(Slot::from_tuple((1, 2)), Key::from_tuple(E));
    key_slots.insert(Slot::from_tuple((1, 3)), Key::from_tuple(R));
    key_slots.insert(Slot::from_tuple((1, 4)), Key::from_tuple(T));
    key_slots.insert(Slot::from_tuple((1, 5)), Key::from_tuple(Y));
    key_slots.insert(Slot::from_tuple((1, 6)), Key::from_tuple(U));
    key_slots.insert(Slot::from_tuple((1, 7)), Key::from_tuple(I));
    key_slots.insert(Slot::from_tuple((1, 8)), Key::from_tuple(O));
    key_slots.insert(Slot::from_tuple((1, 9)), Key::from_tuple(P));
    key_slots.insert(Slot::from_tuple((1, 10)), Key::from_tuple(L_BRACKET));
    key_slots.insert(Slot::from_tuple((1, 11)), Key::from_tuple(R_BRACKET));
    key_slots.insert(Slot::from_tuple((1, 12)), Key::from_tuple(BACKSLASH));

    key_slots.insert(Slot::from_tuple((2, 0)), Key::from_tuple(A));
    key_slots.insert(Slot::from_tuple((2, 1)), Key::from_tuple(S));
    key_slots.insert(Slot::from_tuple((2, 2)), Key::from_tuple(D));
    key_slots.insert(Slot::from_tuple((2, 3)), Key::from_tuple(F));
    key_slots.insert(Slot::from_tuple((2, 4)), Key::from_tuple(G));
    key_slots.insert(Slot::from_tuple((2, 5)), Key::from_tuple(H));
    key_slots.insert(Slot::from_tuple((2, 6)), Key::from_tuple(J));
    key_slots.insert(Slot::from_tuple((2, 7)), Key::from_tuple(K));
    key_slots.insert(Slot::from_tuple((2, 8)), Key::from_tuple(L));
    key_slots.insert(Slot::from_tuple((2, 9)), Key::from_tuple(SEMICOLON));
    key_slots.insert(Slot::from_tuple((2, 10)), Key::from_tuple(QUOTE));
    key_slots.insert(Slot::from_tuple((2, 11)), Key::from_tuple(NEWLINE));

    key_slots.insert(Slot::from_tuple((3, 0)), Key::from_tuple(Z));
    key_slots.insert(Slot::from_tuple((3, 1)), Key::from_tuple(X));
    key_slots.insert(Slot::from_tuple((3, 2)), Key::from_tuple(C));
    key_slots.insert(Slot::from_tuple((3, 3)), Key::from_tuple(V));
    key_slots.insert(Slot::from_tuple((3, 4)), Key::from_tuple(B));
    key_slots.insert(Slot::from_tuple((3, 5)), Key::from_tuple(N));
    key_slots.insert(Slot::from_tuple((3, 6)), Key::from_tuple(M));
    key_slots.insert(Slot::from_tuple((3, 7)), Key::from_tuple(COMMA));
    key_slots.insert(Slot::from_tuple((3, 8)), Key::from_tuple(PERIOD));
    key_slots.insert(Slot::from_tuple((3, 9)), Key::from_tuple(F_SLASH));
}

pub fn place_dvorak_keys(key_slots: &mut BTreeMap<Slot, Key>) {
    key_slots.insert(Slot::from_tuple((0, 0)), Key::from_tuple(ONE));
    key_slots.insert(Slot::from_tuple((0, 1)), Key::from_tuple(TWO));
    key_slots.insert(Slot::from_tuple((0, 2)), Key::from_tuple(THREE));
    key_slots.insert(Slot::from_tuple((0, 3)), Key::from_tuple(FOUR));
    key_slots.insert(Slot::from_tuple((0, 4)), Key::from_tuple(FIVE));
    key_slots.insert(Slot::from_tuple((0, 5)), Key::from_tuple(SIX));
    key_slots.insert(Slot::from_tuple((0, 6)), Key::from_tuple(SEVEN));
    key_slots.insert(Slot::from_tuple((0, 7)), Key::from_tuple(EIGHT));
    key_slots.insert(Slot::from_tuple((0, 8)), Key::from_tuple(NINE));
    key_slots.insert(Slot::from_tuple((0, 9)), Key::from_tuple(ZERO));
    key_slots.insert(Slot::from_tuple((0, 10)), Key::from_tuple(L_BRACKET));
    key_slots.insert(Slot::from_tuple((0, 11)), Key::from_tuple(R_BRACKET));

    key_slots.insert(Slot::from_tuple((1, 0)), Key::from_tuple(QUOTE));
    key_slots.insert(Slot::from_tuple((1, 1)), Key::from_tuple(COMMA));
    key_slots.insert(Slot::from_tuple((1, 2)), Key::from_tuple(PERIOD));
    key_slots.insert(Slot::from_tuple((1, 3)), Key::from_tuple(P));
    key_slots.insert(Slot::from_tuple((1, 4)), Key::from_tuple(Y));
    key_slots.insert(Slot::from_tuple((1, 5)), Key::from_tuple(F));
    key_slots.insert(Slot::from_tuple((1, 6)), Key::from_tuple(G));
    key_slots.insert(Slot::from_tuple((1, 7)), Key::from_tuple(C));
    key_slots.insert(Slot::from_tuple((1, 8)), Key::from_tuple(R));
    key_slots.insert(Slot::from_tuple((1, 9)), Key::from_tuple(L));
    key_slots.insert(Slot::from_tuple((1, 10)), Key::from_tuple(F_SLASH));
    key_slots.insert(Slot::from_tuple((1, 11)), Key::from_tuple(EQUALS));
    key_slots.insert(Slot::from_tuple((1, 12)), Key::from_tuple(BACKSLASH));

    key_slots.insert(Slot::from_tuple((2, 0)), Key::from_tuple(A));
    key_slots.insert(Slot::from_tuple((2, 1)), Key::from_tuple(O));
    key_slots.insert(Slot::from_tuple((2, 2)), Key::from_tuple(E));
    key_slots.insert(Slot::from_tuple((2, 3)), Key::from_tuple(U));
    key_slots.insert(Slot::from_tuple((2, 4)), Key::from_tuple(I));
    key_slots.insert(Slot::from_tuple((2, 5)), Key::from_tuple(D));
    key_slots.insert(Slot::from_tuple((2, 6)), Key::from_tuple(H));
    key_slots.insert(Slot::from_tuple((2, 7)), Key::from_tuple(T));
    key_slots.insert(Slot::from_tuple((2, 8)), Key::from_tuple(N));
    key_slots.insert(Slot::from_tuple((2, 9)), Key::from_tuple(S));
    key_slots.insert(Slot::from_tuple((2, 10)), Key::from_tuple(DASH));
    key_slots.insert(Slot::from_tuple((2, 11)), Key::from_tuple(NEWLINE));

    key_slots.insert(Slot::from_tuple((3, 0)), Key::from_tuple(SEMICOLON));
    key_slots.insert(Slot::from_tuple((3, 1)), Key::from_tuple(Q));
    key_slots.insert(Slot::from_tuple((3, 2)), Key::from_tuple(J));
    key_slots.insert(Slot::from_tuple((3, 3)), Key::from_tuple(K));
    key_slots.insert(Slot::from_tuple((3, 4)), Key::from_tuple(X));
    key_slots.insert(Slot::from_tuple((3, 5)), Key::from_tuple(B));
    key_slots.insert(Slot::from_tuple((3, 6)), Key::from_tuple(M));
    key_slots.insert(Slot::from_tuple((3, 7)), Key::from_tuple(W));
    key_slots.insert(Slot::from_tuple((3, 8)), Key::from_tuple(V));
    key_slots.insert(Slot::from_tuple((3, 9)), Key::from_tuple(Z));
}
