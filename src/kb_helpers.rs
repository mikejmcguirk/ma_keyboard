// **** NOTE ****
// These functions are designed to work with the keyboard struct. Because the struct's properties
// are known at compile time, these functions might not check certain edge cases

use crate::{helper_consts, row_consts};

helper_consts!();

pub fn get_keys() -> Vec<((u8, u8), Vec<(usize, usize)>)> {
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

fn alpha_slots(exclusions: &[(usize, usize)]) -> Vec<(usize, usize)> {
    let slot_groups: Vec<Vec<(usize, usize)>> = vec![top_row(), home_row(), bottom_row()];

    let mut slot_groups_flat: Vec<(usize, usize)> = slot_groups.into_iter().flatten().collect();
    slot_groups_flat.retain(|x| return !exclusions.contains(x));

    return slot_groups_flat;
}

fn top_row() -> Vec<(usize, usize)> {
    return vec![
        (1, 0),
        (1, 1),
        (1, 2),
        (1, 3),
        (1, 4),
        // (1, 5) is skipped so this can hold a symbol key
        (1, 5),
        (1, 6),
        (1, 7),
        (1, 8),
        (1, 9),
    ];
}

fn home_row() -> Vec<(usize, usize)> {
    return vec![
        (2, 0),
        (2, 1),
        (2, 2),
        (2, 3),
        (2, 4),
        (2, 5),
        (2, 6),
        (2, 7),
        (2, 8),
        (2, 9),
    ];
}

fn bottom_row() -> Vec<(usize, usize)> {
    return vec![
        (3, 0),
        (3, 1),
        (3, 2),
        (3, 3),
        // (3, 4) skipped so this can hold a symbol key
        (3, 4),
        (3, 5),
        (3, 6),
        (3, 7),
        (3, 8),
        (3, 9),
    ];
}
