#[macro_export]
macro_rules! kb_helper_consts {
    () => {
        const ASCII_CNT: usize = 128;

        // Default Key
        const SPACE: (u8, u8) = (b' ', b' ');

        const NUM_ROW_CNT: usize = 12;
        const TOP_ROW_CNT: usize = 13;
        const HOME_ROW_CNT: usize = 12;
        const BOT_ROW_CNT: usize = 10;

        // Hands
        const RIGHT: char = 'r';

        const BASE_EFF: f64 = 1.0;
    };
}

#[macro_export]
macro_rules! swappable_keys {
    () => {
        const COMMA: (u8, u8) = (b',', b'<');
        const PERIOD: (u8, u8) = (b'.', b'>');
        const SEMICOLON: (u8, u8) = (b';', b':');
        const QUOTE: (u8, u8) = (b'\'', b'"');
        const A: (u8, u8) = (b'a', b'A');
        const B: (u8, u8) = (b'b', b'B');
        const C: (u8, u8) = (b'c', b'C');
        const D: (u8, u8) = (b'd', b'D');
        const E: (u8, u8) = (b'e', b'E');
        const F: (u8, u8) = (b'f', b'F');
        const G: (u8, u8) = (b'g', b'G');
        const H: (u8, u8) = (b'h', b'H');
        const I: (u8, u8) = (b'i', b'I');
        const J: (u8, u8) = (b'j', b'J');
        const K: (u8, u8) = (b'k', b'K');
        const L: (u8, u8) = (b'l', b'L');
        const M: (u8, u8) = (b'm', b'M');
        const N: (u8, u8) = (b'n', b'N');
        const O: (u8, u8) = (b'o', b'O');
        const P: (u8, u8) = (b'p', b'P');
        const Q: (u8, u8) = (b'q', b'Q');
        const R: (u8, u8) = (b'r', b'R');
        const S: (u8, u8) = (b's', b'S');
        const T: (u8, u8) = (b't', b'T');
        const U: (u8, u8) = (b'u', b'U');
        const V: (u8, u8) = (b'v', b'V');
        const W: (u8, u8) = (b'w', b'W');
        const X: (u8, u8) = (b'x', b'X');
        const Y: (u8, u8) = (b'y', b'Y');
        const Z: (u8, u8) = (b'z', b'Z');
    };
}

#[macro_export]
macro_rules! swappable_arr {
    () => {
        swappable_keys!();

        const SWAPPABLE_KEYS: [(u8, u8); 30] = [
            COMMA, PERIOD, SEMICOLON, QUOTE, A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R,
            S, T, U, V, W, X, Y, Z,
        ];
    };
}

#[macro_export]
macro_rules! helper_consts {
    () => {
        kb_helper_consts!();

        // =======================
        // ==== Keyboard Info ====
        // =======================
        // Hands
        const LEFT: char = 'l';

        // Columns
        const L_PINKY: usize = 0;
        const L_RING: usize = 1;
        const L_MIDDLE: usize = 2;
        const L_INDEX: usize = 3;
        const L_EXT: usize = 4;
        const R_EXT: usize = 5;
        const R_INDEX: usize = 6;
        const R_MIDDLE: usize = 7;
        const R_RING: usize = 8;
        const R_PINKY: usize = 9;
        const R_SYMBOL: usize = 10;
        const R_NETHER: usize = 11;
        const R_PIPE: usize = 12;

        // Rows
        const NUM_ROW: usize = 0;
        const TOP_ROW: usize = 1;
        const HOME_ROW: usize = 2;
        const BOT_ROW: usize = 3;

        // Fingers
        const INDEX: char = 'i';
        const MIDDLE: char = 'm';
        const RING: char = 'r';
        const PINKY: char = 'p';

        // Increase, Medium, Bigram or Single Key
        const I_ME_B: f64 = 1.4;
        // Increase, Medium, Skipgram
        const I_ME_S: f64 = 1.2;
        // Increase, Low, Bigram or Single Key
        // const I_LO_B: f64 = 1.2;
        // Increase, Low, Skipgram
        // const I_LO_S: f64 = 1.1;
        // Deduct, Low, Bigram or Single key
        const D_LO_B: f64 = 0.8;
        // Deduct, Low, Skipgram
        const D_LO_S: f64 = 0.8;
        // Deduct, Medium, Bigram or Single key
        const D_ME_B: f64 = 0.6;
        // Deduct, Medium, Skipgram
        const D_ME_S: f64 = 0.8;
        // Deduct, High, Bigram or Single key
        const D_HI_B: f64 = 0.4;
        // Deduct, High, Skipgram
        const D_HI_S: f64 = 0.7;
        // Deduct, Brutal, Bigram or Single key
        const D_BU_B: f64 = 0.2;
        // Deduct, Brutal, Skipgram
        const D_BU_S: f64 = 0.6;

        // ==================
        // ==== Key Info ====
        // ==================

        // Keys
        const NEWLINE: (u8, u8) = (b'\n', b'\n');
        const BACKSLASH: (u8, u8) = (b'\\', b'|');
        const ONE: (u8, u8) = (b'1', b'!');
        const TWO: (u8, u8) = (b'2', b'@');
        const THREE: (u8, u8) = (b'3', b'#');
        const FOUR: (u8, u8) = (b'4', b'$');
        const FIVE: (u8, u8) = (b'5', b'%');
        const SIX: (u8, u8) = (b'6', b'^');
        const SEVEN: (u8, u8) = (b'7', b'&');
        const EIGHT: (u8, u8) = (b'8', b'*');
        const NINE: (u8, u8) = (b'9', b'(');
        const ZERO: (u8, u8) = (b'0', b')');
        const L_BRACKET: (u8, u8) = (b'[', b'{');
        const R_BRACKET: (u8, u8) = (b']', b'}');
        const DASH: (u8, u8) = (b'-', b'_');
        const EQUALS: (u8, u8) = (b'=', b'+');
        const F_SLASH: (u8, u8) = (b'/', b'?');
        swappable_keys!();

        // Number Row - Static
        const ONE_VALID: [(usize, usize); 1] = [(NUM_ROW, L_PINKY)];
        const TWO_VALID: [(usize, usize); 1] = [(NUM_ROW, L_RING)];
        const THREE_VALID: [(usize, usize); 1] = [(NUM_ROW, L_MIDDLE)];
        const FOUR_VALID: [(usize, usize); 1] = [(NUM_ROW, L_INDEX)];
        const FIVE_VALID: [(usize, usize); 1] = [(NUM_ROW, L_EXT)];
        const SIX_VALID: [(usize, usize); 1] = [(NUM_ROW, R_EXT)];
        const SEVEN_VALID: [(usize, usize); 1] = [(NUM_ROW, R_INDEX)];
        const EIGHT_VALID: [(usize, usize); 1] = [(NUM_ROW, R_MIDDLE)];
        const NINE_VALID: [(usize, usize); 1] = [(NUM_ROW, R_RING)];
        const ZERO_VALID: [(usize, usize); 1] = [(NUM_ROW, R_PINKY)];
        const L_BRACKET_VALID: [(usize, usize); 1] = [(NUM_ROW, R_SYMBOL)];
        const R_BRACKET_VALID: [(usize, usize); 1] = [(NUM_ROW, R_NETHER)];

        // Pinky Extension Symbol Keys - Static
        const DASH_VALID: [(usize, usize); 1] = [(TOP_ROW, R_SYMBOL)];
        const EQUALS_VALID: [(usize, usize); 1] = [(TOP_ROW, R_NETHER)];
        const F_SLASH_VALID: [(usize, usize); 1] = [(HOME_ROW, R_SYMBOL)];
        const NEWLINE_VALID: [(usize, usize); 1] = [(HOME_ROW, R_NETHER)];
        const BACKSLASH_VALID: [(usize, usize); 1] = [(TOP_ROW, R_PIPE)];

        // ALpha Keys - Dynamic
        const Q_INVALID: [(usize, usize); 3] = [
            (TOP_ROW, L_PINKY),
            (HOME_ROW, L_PINKY),
            (BOT_ROW, L_PINKY)
        ];
        const A_INVALID: [(usize, usize); 3] = [
            (TOP_ROW, L_PINKY),
            (HOME_ROW, L_PINKY),
            (BOT_ROW, L_PINKY)
        ];
        const Z_INVALID: [(usize, usize); 3] = [
            (TOP_ROW, L_PINKY),
            (HOME_ROW, L_PINKY),
            (BOT_ROW, L_PINKY)
        ];
        const W_INVALID: [(usize, usize); 3] = [
            (TOP_ROW, L_RING),
            (HOME_ROW, L_RING),
            (BOT_ROW, L_RING)
        ];
        const S_INVALID: [(usize, usize); 3] = [
            (TOP_ROW, L_RING),
            (HOME_ROW, L_RING),
            (BOT_ROW, L_RING)
        ];
        const X_INVALID: [(usize, usize); 3] = [
            (TOP_ROW, L_RING),
            (HOME_ROW, L_RING),
            (BOT_ROW, L_RING)
        ];
        const E_INVALID: [(usize, usize); 3] = [
            (TOP_ROW, L_MIDDLE),
            (HOME_ROW, L_MIDDLE),
            (BOT_ROW, L_MIDDLE),
        ];
        const D_INVALID: [(usize, usize); 3] = [
            (TOP_ROW, L_MIDDLE),
            (HOME_ROW, L_MIDDLE),
            (BOT_ROW, L_MIDDLE),
        ];
        const C_INVALID: [(usize, usize); 3] = [
            (TOP_ROW, L_MIDDLE),
            (HOME_ROW, L_MIDDLE),
            (BOT_ROW, L_MIDDLE),
        ];
        const R_INVALID: [(usize, usize); 3] = [
            (TOP_ROW, L_INDEX),
            (HOME_ROW, L_INDEX),
            (BOT_ROW, L_INDEX)
        ];
        const F_INVALID: [(usize, usize); 3] = [
            (TOP_ROW, L_INDEX),
            (HOME_ROW, L_INDEX),
            (BOT_ROW, L_INDEX)
        ];
        const V_INVALID: [(usize, usize); 3] = [
            (TOP_ROW, L_INDEX),
            (HOME_ROW, L_INDEX),
            (BOT_ROW, L_INDEX)
        ];
        const T_INVALID: [(usize, usize); 3] = [
            (TOP_ROW, L_EXT),
            (HOME_ROW, L_EXT),
            (BOT_ROW, L_EXT)
        ];
        const G_INVALID: [(usize, usize); 3] = [
            (TOP_ROW, L_EXT),
            (HOME_ROW, L_EXT),
            (BOT_ROW, L_EXT)
        ];
        const B_INVALID: [(usize, usize); 3] = [
            (TOP_ROW, L_EXT),
            (HOME_ROW, L_EXT),
            (BOT_ROW, L_EXT)
        ];
        const Y_INVALID: [(usize, usize); 3] = [
            (TOP_ROW, R_EXT),
            (HOME_ROW, R_EXT),
            (BOT_ROW, R_EXT)
        ];
        const H_INVALID: [(usize, usize); 3] = [
            (TOP_ROW, R_EXT),
            (HOME_ROW, R_EXT),
            (BOT_ROW, R_EXT)
        ];
        const N_INVALID: [(usize, usize); 3] = [
            (TOP_ROW, R_EXT),
            (HOME_ROW, R_EXT),
            (BOT_ROW, R_EXT)
        ];
        const U_INVALID: [(usize, usize); 3] = [
            (TOP_ROW, R_INDEX),
            (HOME_ROW, R_INDEX),
            (BOT_ROW, R_INDEX)
        ];
        const J_INVALID: [(usize, usize); 3] = [
            (TOP_ROW, R_INDEX),
            (HOME_ROW, R_INDEX),
            (BOT_ROW, R_INDEX)
        ];
        const M_INVALID: [(usize, usize); 3] = [
            (TOP_ROW, R_INDEX),
            (HOME_ROW, R_INDEX),
            (BOT_ROW, R_INDEX)
        ];
        const I_INVALID: [(usize, usize); 3] = [
            (TOP_ROW, R_MIDDLE),
            (HOME_ROW, R_MIDDLE),
            (BOT_ROW, R_MIDDLE),
        ];
        const K_INVALID: [(usize, usize); 3] = [
            (TOP_ROW, R_MIDDLE),
            (HOME_ROW, R_MIDDLE),
            (BOT_ROW, R_MIDDLE),
        ];
        const COMMA_INVALID: [(usize, usize); 6] = [
            (TOP_ROW, R_PINKY),
            (HOME_ROW, R_PINKY),
            (BOT_ROW, R_PINKY),
            (TOP_ROW, R_MIDDLE),
            (HOME_ROW, R_MIDDLE),
            (BOT_ROW, R_MIDDLE),
        ];
        const O_INVALID: [(usize, usize); 3] = [
            (TOP_ROW, R_RING),
            (HOME_ROW, R_RING),
            (BOT_ROW, R_RING)
        ];
        const L_INVALID: [(usize, usize); 3] = [
            (TOP_ROW, R_RING),
            (HOME_ROW, R_RING),
            (BOT_ROW, R_RING)
        ];
        const PERIOD_INVALID: [(usize, usize); 6] = [
            (TOP_ROW, L_PINKY),
            (HOME_ROW, L_PINKY),
            (BOT_ROW, L_PINKY),
            (TOP_ROW, R_RING),
            (HOME_ROW, R_RING),
            (BOT_ROW, R_RING),
        ];
        const P_INVALID: [(usize, usize); 3] = [
            (TOP_ROW, R_PINKY),
            (HOME_ROW, R_PINKY),
            (BOT_ROW, R_PINKY)
        ];
        const SEMICOLON_INVALID: [(usize, usize); 3] = [
            (TOP_ROW, R_PINKY),
            (HOME_ROW, R_PINKY),
            (BOT_ROW, R_PINKY)
        ];
        const QUOTE_INVALID: [(usize, usize); 3] = [
            (TOP_ROW, R_PINKY),
            (HOME_ROW, R_PINKY),
            (BOT_ROW, R_PINKY)
        ];

        const DEFAULT_TOP_ROW: [(usize, usize); 10] = [
            (TOP_ROW, L_PINKY),
            (TOP_ROW, L_RING),
            (TOP_ROW, L_MIDDLE),
            (TOP_ROW, L_INDEX),
            (TOP_ROW, L_EXT),
            (TOP_ROW, R_EXT),
            (TOP_ROW, R_INDEX),
            (TOP_ROW, R_MIDDLE),
            (TOP_ROW, R_RING),
            (TOP_ROW, R_PINKY),
        ];

        const DEFAULT_HOME_ROW: [(usize, usize); 10] = [
            (HOME_ROW, L_PINKY),
            (HOME_ROW, L_RING),
            (HOME_ROW, L_MIDDLE),
            (HOME_ROW, L_INDEX),
            (HOME_ROW, L_EXT),
            (HOME_ROW, R_EXT),
            (HOME_ROW, R_INDEX),
            (HOME_ROW, R_MIDDLE),
            (HOME_ROW, R_RING),
            (HOME_ROW, R_PINKY),
        ];

        const DEFAULT_BOT_ROW: [(usize, usize); 10] = [
            (BOT_ROW, L_PINKY),
            (BOT_ROW, L_RING),
            (BOT_ROW, L_MIDDLE),
            (BOT_ROW, L_INDEX),
            (BOT_ROW, L_EXT),
            (BOT_ROW, R_EXT),
            (BOT_ROW, R_INDEX),
            (BOT_ROW, R_MIDDLE),
            (BOT_ROW, R_RING),
            (BOT_ROW, R_PINKY),
        ];

    };
}
