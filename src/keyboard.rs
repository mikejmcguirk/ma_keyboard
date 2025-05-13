use rand::{Rng as _, rngs::SmallRng};

const DEFAULT_KEY: (u8, u8) = (b' ', b' ');

// Finger
const INDEX: char = 'i';
const MIDDLE: char = 'm';
const RING: char = 'r';
const PINKY: char = 'p';

// Hand
const LEFT: char = 'l';
const RIGHT: char = 'r';

// Keyboard rows
const NUM: usize = 0;
const TOP: usize = 1;
const HOME: usize = 2;
const BOT: usize = 3;

// Keyboard Columns
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

const BASE_EFF: f64 = 1.0;

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

enum KeyCompare {
    Mult(f64),
    MultLeft(f64),
    Mismatch,
}

#[derive(Clone)]
pub struct Keyboard {
    kb_vec: Vec<Vec<(u8, u8)>>,
    valid_locations: Vec<((u8, u8), Vec<(usize, usize)>)>,
    slot_ascii: Vec<Option<(usize, usize)>>,
    last_key_idx: Option<(usize, usize)>,
    prev_key_idx: Option<(usize, usize)>,
    generation: usize,
    id: usize,
    evaluated: bool,
    score: f64,
    left_uses: f64,
    right_uses: f64,
    is_elite: bool,
    pos_iter: usize,
}

// TODO: Break out non-self but KB functions into their own file. Make sure it's marked explicitly
// that it's tied to keyboard so we don't put inappropriate error returns in
// TODO: A better architecture for this is to let the user bring in the valid keys from a config
// file rather than actually altering the source code. So then error propagation would be the
// better design
// TODO: The meta issue with this struct is how much it relies on the KeyTemplate enum. It's
// compile time, which is good, but it's exterior, which is bad
// TODO: Need to rebuild the qwerty creation and add dvorak
// NOTE: Any impl methods that are private and/or do not take external input assume that the struct
// data is correct. These methods are not meant to be portable
impl Keyboard {
    // TODO: No assertion message
    pub fn create_origin(id_in: usize) -> Self {
        const NUM_ROW_CAPACITY: usize = 12;
        const TOP_ROW_CAPACITY: usize = 12;
        const HOME_ROW_CAPACITY: usize = 11;
        const BOT_ROW_CAPACITY: usize = 10;

        let mut kb_vec: Vec<Vec<(u8, u8)>> = vec![
            vec![DEFAULT_KEY; NUM_ROW_CAPACITY],
            vec![DEFAULT_KEY; TOP_ROW_CAPACITY],
            vec![DEFAULT_KEY; HOME_ROW_CAPACITY],
            vec![DEFAULT_KEY; BOT_ROW_CAPACITY],
        ];
        let mut valid_locations: Vec<((u8, u8), Vec<(usize, usize)>)> = Self::get_keys();

        let mut kb_vec_cnt: usize = 0;
        for vec in &kb_vec {
            kb_vec_cnt += vec.len();
        }
        assert_eq!(kb_vec_cnt, valid_locations.len());
        for location in &valid_locations {
            assert!(location.1.len() > 0);
        }

        valid_locations.sort_by(|a, b| {
            return a
                .1
                .len()
                .partial_cmp(&b.1.len())
                .unwrap_or(std::cmp::Ordering::Equal);
        });

        assert!(Self::place_keys(&mut kb_vec, &valid_locations, 0));
        for row in &kb_vec {
            for col in row {
                assert!(*col != DEFAULT_KEY);
            }
        }

        let mut slot_ascii: Vec<Option<(usize, usize)>> = vec![None; 128];
        for i in 0..kb_vec.len() {
            for j in 0..kb_vec[i].len() {
                slot_ascii[kb_vec[i][j].0 as usize] = Some((i, j));
                slot_ascii[kb_vec[i][j].1 as usize] = Some((i, j));
            }
        }

        return Self {
            kb_vec,
            valid_locations,
            slot_ascii,
            last_key_idx: None,
            prev_key_idx: None,
            generation: 0,
            id: id_in,
            evaluated: false,
            score: 0.0,
            left_uses: 0.0,
            right_uses: 0.0,
            is_elite: false,
            pos_iter: 0,
        };
    }

    fn get_keys() -> Vec<((u8, u8), Vec<(usize, usize)>)> {
        return vec![
            ((b'1', b'!'), vec![(NUM, L_PINKY)]),
            ((b'2', b'@'), vec![(NUM, L_RING)]),
            ((b'3', b'#'), vec![(NUM, L_MIDDLE)]),
            ((b'4', b'$'), vec![(NUM, L_INDEX)]),
            ((b'5', b'%'), vec![(NUM, L_EXT)]),
            ((b'6', b'^'), vec![(NUM, R_EXT)]),
            ((b'7', b'&'), vec![(NUM, R_INDEX)]),
            ((b'8', b'*'), vec![(NUM, R_MIDDLE)]),
            ((b'9', b'('), vec![(NUM, R_RING)]),
            ((b'0', b')'), vec![(NUM, R_PINKY)]),
            ((b'[', b'{'), vec![(NUM, R_SYMBOL)]),
            ((b']', b'}'), vec![(NUM, R_NETHER)]),
            ((b',', b'<'), Self::alpha_slots(&vec![(BOT, R_MIDDLE)])),
            ((b'.', b'>'), Self::alpha_slots(&vec![(BOT, R_RING)])),
            ((b'-', b'_'), vec![(TOP, R_SYMBOL)]),
            ((b'=', b'+'), vec![(TOP, R_NETHER)]),
            ((b'/', b'?'), vec![(HOME, R_SYMBOL)]),
            (
                (b';', b':'),
                Self::alpha_slots(&vec![(TOP, R_PINKY), (HOME, R_PINKY), (BOT, R_PINKY)]),
            ),
            (
                (b'\'', b'"'),
                Self::alpha_slots(&vec![(TOP, R_PINKY), (HOME, R_PINKY), (BOT, R_PINKY)]),
            ),
            (
                (b'a', b'A'),
                Self::alpha_slots(&vec![(TOP, L_PINKY), (HOME, L_PINKY), (BOT, L_PINKY)]),
            ),
            (
                (b'b', b'B'),
                Self::alpha_slots(&vec![(TOP, L_EXT), (HOME, L_EXT), (BOT, L_EXT)]),
            ),
            (
                (b'c', b'C'),
                Self::alpha_slots(&vec![(TOP, L_MIDDLE), (HOME, L_MIDDLE), (BOT, L_MIDDLE)]),
            ),
            (
                (b'd', b'D'),
                Self::alpha_slots(&vec![(TOP, L_MIDDLE), (HOME, L_MIDDLE), (BOT, L_MIDDLE)]),
            ),
            (
                (b'e', b'E'),
                Self::alpha_slots(&vec![(TOP, L_MIDDLE), (HOME, L_MIDDLE), (BOT, L_MIDDLE)]),
            ),
            (
                (b'f', b'F'),
                Self::alpha_slots(&vec![(TOP, L_INDEX), (HOME, L_INDEX), (BOT, L_INDEX)]),
            ),
            (
                (b'g', b'G'),
                Self::alpha_slots(&vec![(TOP, L_EXT), (HOME, L_EXT), (BOT, L_EXT)]),
            ),
            (
                (b'h', b'H'),
                Self::alpha_slots(&vec![(TOP, R_EXT), (HOME, R_EXT), (BOT, R_EXT)]),
            ),
            (
                (b'i', b'I'),
                Self::alpha_slots(&vec![(TOP, R_MIDDLE), (HOME, R_MIDDLE), (BOT, R_MIDDLE)]),
            ),
            (
                (b'j', b'J'),
                Self::alpha_slots(&vec![(TOP, R_INDEX), (HOME, R_INDEX), (BOT, R_INDEX)]),
            ),
            (
                (b'k', b'K'),
                Self::alpha_slots(&vec![(TOP, R_MIDDLE), (HOME, R_MIDDLE), (BOT, R_MIDDLE)]),
            ),
            (
                (b'l', b'L'),
                Self::alpha_slots(&vec![(TOP, R_RING), (HOME, R_RING), (BOT, R_RING)]),
            ),
            (
                (b'm', b'M'),
                Self::alpha_slots(&vec![(TOP, R_INDEX), (HOME, R_INDEX), (BOT, R_INDEX)]),
            ),
            (
                (b'n', b'N'),
                Self::alpha_slots(&vec![(TOP, R_EXT), (HOME, R_EXT), (BOT, R_EXT)]),
            ),
            (
                (b'o', b'O'),
                Self::alpha_slots(&vec![(TOP, R_RING), (HOME, R_RING), (BOT, R_RING)]),
            ),
            (
                (b'p', b'P'),
                Self::alpha_slots(&vec![(TOP, R_PINKY), (HOME, R_PINKY), (BOT, R_PINKY)]),
            ),
            (
                (b'q', b'Q'),
                Self::alpha_slots(&vec![(TOP, L_PINKY), (HOME, L_PINKY), (BOT, L_PINKY)]),
            ),
            (
                (b'r', b'R'),
                Self::alpha_slots(&vec![(TOP, L_INDEX), (HOME, L_INDEX), (BOT, L_INDEX)]),
            ),
            (
                (b's', b'S'),
                Self::alpha_slots(&vec![(TOP, L_RING), (HOME, L_RING), (BOT, L_RING)]),
            ),
            (
                (b't', b'T'),
                Self::alpha_slots(&vec![(TOP, L_EXT), (HOME, L_EXT), (BOT, L_EXT)]),
            ),
            (
                (b'u', b'U'),
                Self::alpha_slots(&vec![(TOP, R_INDEX), (HOME, R_INDEX), (BOT, R_INDEX)]),
            ),
            (
                (b'v', b'V'),
                Self::alpha_slots(&vec![(TOP, L_INDEX), (HOME, L_INDEX), (BOT, L_INDEX)]),
            ),
            (
                (b'w', b'W'),
                Self::alpha_slots(&vec![(TOP, L_RING), (HOME, L_RING), (BOT, L_RING)]),
            ),
            (
                (b'x', b'X'),
                Self::alpha_slots(&vec![(TOP, L_RING), (HOME, L_RING), (BOT, L_RING)]),
            ),
            (
                (b'y', b'Y'),
                Self::alpha_slots(&vec![(TOP, R_EXT), (HOME, R_EXT), (BOT, R_EXT)]),
            ),
            (
                (b'z', b'Z'),
                Self::alpha_slots(&vec![(TOP, L_PINKY), (HOME, L_PINKY), (BOT, L_PINKY)]),
            ),
        ];
    }

    fn alpha_slots(exclusions: &[(usize, usize)]) -> Vec<(usize, usize)> {
        let slot_groups: Vec<Vec<(usize, usize)>> =
            vec![Self::top_row(), Self::home_row(), Self::bottom_row()];

        let mut slot_groups_flat: Vec<(usize, usize)> =
            slot_groups.into_iter().flatten().collect();
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

    fn place_keys(
        kb_vec: &mut Vec<Vec<(u8, u8)>>,
        keys: &Vec<((u8, u8), Vec<(usize, usize)>)>,
        idx: usize,
    ) -> bool {
        if idx == keys.len() {
            return true;
        }

        assert!(keys[idx].1.len() > 0);

        for placement in &keys[idx].1 {
            let (row, col) = *placement;
            if kb_vec[row][col] != DEFAULT_KEY {
                continue;
            }

            kb_vec[row][col] = keys[idx].0;

            if Self::place_keys(kb_vec, keys, idx + 1) {
                return true;
            } else {
                kb_vec[row][col] = DEFAULT_KEY;
            }
        }

        return false;
    }

    pub fn mutate_from(kb: &Keyboard, gen_input: usize, id_in: usize) -> Self {
        return Self {
            kb_vec: kb.kb_vec.clone(),
            valid_locations: kb.valid_locations.clone(),
            slot_ascii: kb.slot_ascii.clone(),
            last_key_idx: None,
            prev_key_idx: None,
            generation: gen_input,
            id: id_in,
            evaluated: kb.evaluated,
            score: kb.get_score(),
            left_uses: 0.0,
            right_uses: 0.0,
            is_elite: false,
            pos_iter: kb.pos_iter,
        };
    }

    // NOTE: This function assumes the keys are properly setup such that there is always at least
    // one valid option to shuffle to
    // TODO: Not assertion messages
    pub fn shuffle(&mut self, rng: &mut SmallRng, amt: usize) {
        // The number row and the side symbol keys are purposefully avoided
        const MIN_ROW: usize = 1;
        const MAX_ROW: usize = 4;
        const MIN_COL: usize = 0;
        const MAX_COL: usize = 10;

        self.evaluated = false;

        let mut s: usize = 0;
        while s < amt {
            let row_x: usize = rng.random_range(MIN_ROW..MAX_ROW);
            let col_x: usize = rng.random_range(MIN_COL..MAX_COL);
            let key_x: (u8, u8) = self.kb_vec[row_x][col_x];
            let idx_x: usize = Self::get_loc_idx(key_x, &mut self.valid_locations);

            debug_assert!(!self.valid_locations[idx_x].1.is_empty());
            if self.valid_locations[idx_x].1.len() == 1 {
                continue;
            }

            for i in 0..self.valid_locations[idx_x].1.len() - 1 {
                let j: usize = rng.random_range(1..self.valid_locations[idx_x].1.len());
                self.valid_locations[idx_x].1.swap(i, j);
            }

            for i in 0..self.valid_locations[idx_x].1.len() {
                let row_y: usize = self.valid_locations[idx_x].1[i].0;
                let col_y: usize = self.valid_locations[idx_x].1[i].1;
                let key_y: (u8, u8) = self.kb_vec[row_y][col_y];
                let idx_y: usize = Self::get_loc_idx(key_y, &mut self.valid_locations);

                debug_assert!(!self.valid_locations[idx_y].1.is_empty());
                if self.valid_locations[idx_y].1.len() == 1 {
                    continue;
                }

                if self.valid_locations[idx_y].1.len() == 1
                    || !self.valid_locations[idx_y].1.contains(&(row_x, col_x))
                {
                    continue;
                }

                self.kb_vec[row_x][col_x] = key_y;
                self.kb_vec[row_y][col_y] = key_x;

                self.slot_ascii[key_y.0 as usize] = Some((row_x, col_x));
                self.slot_ascii[key_y.1 as usize] = Some((row_x, col_x));
                self.slot_ascii[key_x.0 as usize] = Some((row_y, col_y));
                self.slot_ascii[key_x.1 as usize] = Some((row_y, col_y));

                s += 1;
                break;
            }
        }
    }

    // TODO: I think this panic message is fine
    fn get_loc_idx(
        key: (u8, u8),
        valid_locations: &mut [((u8, u8), Vec<(usize, usize)>)],
    ) -> usize {
        for i in 0..valid_locations.len() {
            if valid_locations[i].0 == key {
                return i;
            }
        }

        panic!("Did not find {:?} in valid locations", key);
    }

    // TODO: Unsure of how to handle space and return
    // NOTE: A single major efficiency penalty at any point in the algorithm can cause the entire
    // layout to change. Be careful over-indexing for any particular factor
    fn get_efficiency(&mut self, this_key: (usize, usize)) -> f64 {
        let mut eff: f64 = BASE_EFF;

        let this_col: usize = this_key.1;
        let this_hand: char = Self::get_hand(this_col);
        if this_hand == RIGHT {
            self.right_uses += 1.0;
        } else {
            self.left_uses += 1.0;
        }

        eff *= Self::get_single_key_mult(this_key);

        let last_compare: Option<KeyCompare> = self
            .last_key_idx
            .map(|last_key| return Self::compare_keys(this_key, last_key, true));
        if let Some(key_compare) = last_compare {
            match key_compare {
                KeyCompare::Mult(x) => return eff * x,
                KeyCompare::MultLeft(x) => return eff * x * 0.8,
                KeyCompare::Mismatch => {}
            }
        }

        let prev_compare: Option<KeyCompare> = self
            .prev_key_idx
            .map(|prev_key| return Self::compare_keys(this_key, prev_key, true));
        if let Some(key_compare) = prev_compare {
            match key_compare {
                KeyCompare::Mult(x) => return eff * x,
                KeyCompare::MultLeft(x) => return eff * x * 0.8,
                KeyCompare::Mismatch => {}
            }
        }

        let this_row: usize = this_key.0;
        let dist_from_home: usize = this_row.abs_diff(2);
        if dist_from_home == 1 {
            eff *= 0.8;
        } else if dist_from_home == 2 {
            eff *= 0.6;
        }

        if this_hand == 'l' && dist_from_home > 0 {
            eff *= 0.8;
        }

        return eff;
    }

    pub fn eval(&mut self, corpus: &[String]) {
        if self.evaluated {
            return;
        }

        self.score = 0.0;
        self.last_key_idx = None;
        self.prev_key_idx = None;
        self.left_uses = 0.0;
        self.right_uses = 0.0;

        for entry in corpus {
            for b in entry.as_bytes() {
                let this_key: (usize, usize) =
                    if let Some(&Some(key)) = self.slot_ascii.get(*b as usize) {
                        key
                    } else {
                        self.prev_key_idx = self.last_key_idx;
                        self.last_key_idx = None;
                        continue;
                    };

                self.score += self.get_efficiency(this_key);

                self.prev_key_idx = self.last_key_idx;
                self.last_key_idx = Some(this_key);
            }
        }

        if self.left_uses < self.right_uses {
            self.score *= self.left_uses / self.right_uses;
        } else {
            self.score *= self.right_uses / self.left_uses;
        }

        self.evaluated = true;
    }

    fn get_hand(col: usize) -> char {
        return match col {
            0..=4 => 'l',
            5..=11 => 'r',
            _ => 'u',
        };
    }

    fn get_finger(col: usize) -> char {
        return match col {
            0 | 9..=11 => 'p',
            1 | 8 => 'r',
            2 | 7 => 'm',
            3..=6 => 'i',
            _ => 'u',
        };
    }

    // No blanket adjustment for any particular row. The specific code for bigrams and the
    // additional code for single keys both deduct for row movement, which necessarily results in
    // the algo favoring the home row
    fn get_single_key_mult(key: (usize, usize)) -> f64 {
        let row: usize = key.0;
        let col: usize = key.1;
        let finger: char = Self::get_finger(col);
        let mut mult: f64 = BASE_EFF;

        // Do a blanket downward adjustment rather than micro-correct in the finger comparisons
        // The ring and pinky are mostly treated the same due to different preferences per typist.
        // However, the pinky top row is given an extra penalty because the whole hand has to be
        // moved to hit it
        if finger == RING || finger == PINKY {
            if row == HOME {
                mult *= D_LO_B;
            } else if (row == BOT) || (row == TOP && finger == RING) {
                mult *= D_ME_B;
            } else if row == TOP && finger == PINKY {
                mult *= D_HI_B;
            }
        }

        // The algo is too willing to put high-usage keys here
        mult *= match (row, col) {
            (TOP, 4) => D_ME_B,
            (HOME, 4) => D_LO_B,
            (BOT, 4) => D_HI_B,
            (TOP, 5) => D_HI_B,
            (HOME, 5) => D_LO_B,
            (BOT, 5) => D_ME_B,
            _ => BASE_EFF,
        };

        return mult;
    }

    fn compare_keys(key_x: (usize, usize), key_y: (usize, usize), is_last: bool) -> KeyCompare {
        let key_x_col: usize = key_x.1;
        let key_y_col: usize = key_y.1;
        let key_x_hand: char = Self::get_hand(key_x_col);
        let key_y_hand: char = Self::get_hand(key_y_col);
        if key_x_hand != key_y_hand {
            return KeyCompare::Mismatch;
        }

        let key_x_row: usize = key_x.0;
        let key_y_row: usize = key_y.0;
        let mut eff: f64 = BASE_EFF;

        let row_eff: f64 = Self::get_row_mult(key_x_row, key_y_row, is_last);
        eff *= row_eff;

        // NOTE: These extension effeciencies are meant to track the impact of moving the index or
        // the pinky off the home columns on the entire hand
        let index_ext_eff: f64 = Self::get_index_eff(key_x, key_y, is_last);
        eff *= index_ext_eff;
        eff *= Self::get_pinky_eff(key_x, key_y, is_last);

        let key_x_finger: char = Self::get_finger(key_x_col);
        let key_y_finger: char = Self::get_finger(key_y_col);
        let mut index_mv_eff: f64 = 1.0;
        if key_x_finger == key_y_finger {
            eff *= Self::get_base_sf_penalty(is_last);
            index_mv_eff = Self::get_repeat_col_mult(key_x_col, key_y_col, is_last);
            eff *= index_mv_eff;
        } else if key_x_row != key_y_row {
            // No need here to save a value to check left hand efficiency. This branch requires a
            // row move, which has already been checked
            eff *= Self::check_combo(key_x, key_y, is_last);
            eff *= Self::check_scissor(key_x, key_y, is_last);
        }

        let did_mv: bool = row_eff < 1.0 || index_ext_eff < 1.0 || index_mv_eff < 1.0;
        if key_x_hand == LEFT && did_mv {
            return KeyCompare::MultLeft(eff);
        }

        return KeyCompare::Mult(eff);
    }

    fn check_combo(this_key: (usize, usize), that_key: (usize, usize), is_last: bool) -> f64 {
        let this_row: usize = this_key.0;
        let that_row: usize = that_key.0;
        let this_finger: char = Self::get_finger(this_key.1);
        let that_finger: char = Self::get_finger(that_key.1);

        let (top, bot): (char, char) = if this_row > that_row {
            (this_finger, that_finger)
        } else if that_row > this_row {
            (that_finger, this_finger)
        } else {
            panic!("Trying to get combo of equal rows");
        };

        if bot == INDEX || top == MIDDLE || (top == RING && bot == PINKY) {
            return 1.0;
        } else if is_last {
            return 0.6;
        } else {
            return 0.8;
        }
    }

    fn get_center_distance(row: usize) -> usize {
        return if row <= 4 {
            4 - row
        } else {
            row - 5
        };
    }

    // NOTE: Assumes that both keys are on the same hand
    fn get_row_mult(this_row: usize, that_row: usize, is_last: bool) -> f64 {
        let row_diff: usize = this_row.abs_diff(that_row);

        return match (row_diff, is_last) {
            (0, true) => 1.0,
            (1, true) => 0.8,
            (2, true) => 0.6,
            (3, true) => 0.2,
            (0, false) => 1.0,
            (1, false) => 0.9,
            (2, false) => 0.7,
            (3, false) => 0.5,
            _ => 1.0,
        };
    }

    // NOTE: Assumes that both keys are on the same hand
    fn get_index_eff(this: (usize, usize), last: (usize, usize), is_bigram: bool) -> f64 {
        if !((4..=5).contains(&this.1) || (4..=5).contains(&last.1)) {
            return BASE_EFF;
        }

        return match (this.0, this.1, last.0, last.1, is_bigram) {
            // T (Not penalized. No more movement than hitting R)
            (TOP, 4, _, _, true) | (_, _, TOP, 4, true) => BASE_EFF,
            (TOP, 4, _, _, false) | (_, _, TOP, 4, false) => BASE_EFF,
            // G
            (HOME, 4, _, _, true) | (_, _, HOME, 4, true) => D_LO_B,
            (HOME, 4, _, _, false) | (_, _, HOME, 4, false) => D_LO_S,
            // B
            (BOT, 4, _, _, true) | (_, _, BOT, 4, true) => D_HI_B,
            (BOT, 4, _, _, false) | (_, _, BOT, 4, false) => D_HI_S,
            // 5 (Not penalized, no more movement than hitting 4)
            (NUM, 4, _, _, true) | (_, _, NUM, 4, true) => BASE_EFF,
            (NUM, 4, _, _, false) | (_, _, NUM, 4, false) => BASE_EFF,
            // Y
            (TOP, 5, _, _, true) | (_, _, TOP, 5, true) => D_HI_B,
            (TOP, 5, _, _, false) | (_, _, TOP, 5, false) => D_HI_S,
            // H
            (HOME, 5, _, _, true) | (_, _, HOME, 5, true) => D_LO_B,
            (HOME, 5, _, _, false) | (_, _, HOME, 5, false) => D_LO_S,
            // N
            (BOT, 5, _, _, true) | (_, _, BOT, 5, true) => D_ME_B,
            (BOT, 5, _, _, false) | (_, _, BOT, 5, false) => D_ME_S,
            // 6
            (NUM, 5, _, _, true) | (_, _, NUM, 5, true) => D_BU_B,
            (NUM, 5, _, _, false) | (_, _, NUM, 5, false) => D_BU_S,
            _ => BASE_EFF,
        };
    }

    // NOTE: Assumes that both keys are on the same hand
    fn get_pinky_eff(this: (usize, usize), that: (usize, usize), is_last: bool) -> f64 {
        if !((10..=11).contains(&this.1) || (10..=11).contains(&that.1)) {
            return 1.0;
        }

        return match (this.0, this.1, that.0, that.1, is_last) {
            (NUM, 10, _, _, true) | (_, _, NUM, 10, true) => 0.2,
            (NUM, 11, _, _, true) | (_, _, NUM, 11, true) => 0.2,
            (TOP, 10, _, _, true) | (_, _, TOP, 10, true) => 0.6,
            (TOP, 11, _, _, true) | (_, _, TOP, 11, true) => 0.4,
            (HOME, 10, _, _, true) | (_, _, HOME, 10, true) => 0.8,
            (NUM, 10, _, _, false) | (_, _, NUM, 10, false) => 0.2,
            (NUM, 11, _, _, false) | (_, _, NUM, 11, false) => 0.2,
            (TOP, 10, _, _, false) | (_, _, TOP, 10, false) => 0.8,
            (TOP, 11, _, _, false) | (_, _, TOP, 11, false) => 0.7,
            (HOME, 10, _, _, false) | (_, _, HOME, 10, false) => 0.9,
            _ => 1.0,
        };
    }

    fn get_base_sf_penalty(is_last: bool) -> f64 {
        if is_last {
            return 0.6;
        } else {
            return 0.8;
        }
    }

    // NOTE: These are more severely devalued because the algorithm inherently likes to put
    // important keys here
    // TODO: But there maybe an alternative solution, if we factor in the move for the stretch back
    // as well
    // TODO: Does this need to handle YB specifically?
    fn get_repeat_col_mult(this_col: usize, last_col: usize, last: bool) -> f64 {
        let this_center_dist: usize = Self::get_center_distance(this_col);
        let last_center_dist: usize = Self::get_center_distance(last_col);
        let center_diff: usize = this_center_dist.abs_diff(last_center_dist);

        return match (center_diff, last) {
            (1, true) => 0.8,
            (2, true) => 0.6,
            (1, false) => 0.9,
            (2, false) => 0.8,
            _ => 1.0,
        };
    }

    // NOTE: This function assumes we've already verified that we're on the same hand
    // NOTE: I've seen "non-adjacent" scissors described before, but that should be possible to
    // handle using the normal rules
    fn check_scissor(this_key: (usize, usize), that_key: (usize, usize), is_last: bool) -> f64 {
        let this_col: usize = this_key.1;
        let that_col: usize = that_key.1;
        if this_col.abs_diff(that_col) > 1 {
            return 1.0;
        }

        let this_row: usize = this_key.0;
        let that_row: usize = that_key.0;
        let hand: char = Self::get_hand(this_col);
        // Left-handed scissors are punished beyond the base left-hand movement deduction because,
        // unlike right-handed scissors, you have to actually rock your hand to hit them
        return match (this_row.abs_diff(that_row), hand, is_last) {
            (2, RIGHT, true) => 0.6,
            (3, RIGHT, true) => 0.4,
            (2, RIGHT, false) => 0.8,
            (3, RIGHT, false) => 0.7,
            (2, LEFT, true) => 0.4,
            (3, LEFT, true) => 0.2,
            (2, LEFT, false) => 0.7,
            (3, LEFT, false) => 0.6,
            _ => 1.0,
        };
    }

    // TODO: Very inefficient
    pub fn get_display_chars(&self) -> Vec<Vec<char>> {
        let mut display_chars: Vec<Vec<char>> = Vec::new();

        for row in &self.kb_vec {
            let mut chars: Vec<char> = Vec::new();
            for element in row {
                let char = element.0 as char;
                chars.push(char);
            }

            display_chars.push(chars);
        }

        return display_chars;
    }

    // Info display
    pub fn get_score(&self) -> f64 {
        return self.score;
    }

    pub fn get_generation(&self) -> usize {
        return self.generation;
    }

    pub fn get_id(&self) -> usize {
        return self.id;
    }

    pub fn is_elite(&self) -> bool {
        return self.is_elite;
    }

    pub fn set_elite(&mut self) {
        self.is_elite = true;
    }

    pub fn unset_elite(&mut self) {
        self.is_elite = false;
    }

    pub fn get_pos_iter(&self) -> usize {
        return self.pos_iter;
    }

    pub fn add_pos_iter(&mut self) {
        self.pos_iter += 1;
    }
}
