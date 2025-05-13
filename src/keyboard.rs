use rand::{Rng as _, rngs::SmallRng};

use crate::{
    kb_consts, kb_helper_consts,
    kb_helpers::{check_spaces, get_key_locations, place_keys},
};

kb_consts!();

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

impl Keyboard {
    // slot_ascii has a compile time length of 128. Every char in valid_locations is checked with
    // an assertion in get_key_locations that it is 127 or less
    #[expect(clippy::indexing_slicing)]
    pub fn create_origin(id_in: usize) -> Self {
        let mut kb_vec = vec![
            vec![SPACE; NUM_ROW_CNT],
            vec![SPACE; TOP_ROW_CNT],
            vec![SPACE; HOME_ROW_CNT],
            vec![SPACE; BOT_ROW_CNT],
        ];

        let valid_locations = get_key_locations();
        let kb_vec_cnt: usize = kb_vec.iter().map(|v| return v.len()).sum();
        assert_eq!(
            kb_vec_cnt,
            valid_locations.len(),
            "The amount of keys to assign ({}) is not equal to the number of keyboard slots ({})",
            kb_vec_cnt,
            valid_locations.len()
        );

        place_keys(&mut kb_vec, &valid_locations, 0);
        let space_keys = check_spaces(&kb_vec);
        assert!(
            space_keys.is_empty(),
            "The following kb_vec values contain spaces: {:?}",
            space_keys
        );

        let mut slot_ascii = vec![None; ASCII_CNT];
        kb_vec.iter().enumerate().for_each(|(i, row)| {
            row.iter().enumerate().for_each(|(j, &(base, shift))| {
                slot_ascii[usize::from(base)] = Some((i, j));
                slot_ascii[usize::from(shift)] = Some((i, j));
            });
        });

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
            if row == HOME_ROW {
                mult *= D_LO_B;
            } else if (row == BOT_ROW) || (row == TOP_ROW && finger == RING) {
                mult *= D_ME_B;
            } else if row == TOP_ROW && finger == PINKY {
                mult *= D_HI_B;
            }
        }

        // The algo is too willing to put high-usage keys here
        mult *= match (row, col) {
            (TOP_ROW, 4) => D_ME_B,
            (HOME_ROW, 4) => D_LO_B,
            (BOT_ROW, 4) => D_HI_B,
            (TOP_ROW, 5) => D_HI_B,
            (HOME_ROW, 5) => D_LO_B,
            (BOT_ROW, 5) => D_ME_B,
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
            (TOP_ROW, 4, _, _, true) | (_, _, TOP_ROW, 4, true) => BASE_EFF,
            (TOP_ROW, 4, _, _, false) | (_, _, TOP_ROW, 4, false) => BASE_EFF,
            // G
            (HOME_ROW, 4, _, _, true) | (_, _, HOME_ROW, 4, true) => D_LO_B,
            (HOME_ROW, 4, _, _, false) | (_, _, HOME_ROW, 4, false) => D_LO_S,
            // B
            (BOT_ROW, 4, _, _, true) | (_, _, BOT_ROW, 4, true) => D_HI_B,
            (BOT_ROW, 4, _, _, false) | (_, _, BOT_ROW, 4, false) => D_HI_S,
            // 5 (Not penalized, no more movement than hitting 4)
            (NUM_ROW, 4, _, _, true) | (_, _, NUM_ROW, 4, true) => BASE_EFF,
            (NUM_ROW, 4, _, _, false) | (_, _, NUM_ROW, 4, false) => BASE_EFF,
            // Y
            (TOP_ROW, 5, _, _, true) | (_, _, TOP_ROW, 5, true) => D_HI_B,
            (TOP_ROW, 5, _, _, false) | (_, _, TOP_ROW, 5, false) => D_HI_S,
            // H
            (HOME_ROW, 5, _, _, true) | (_, _, HOME_ROW, 5, true) => D_LO_B,
            (HOME_ROW, 5, _, _, false) | (_, _, HOME_ROW, 5, false) => D_LO_S,
            // N
            (BOT_ROW, 5, _, _, true) | (_, _, BOT_ROW, 5, true) => D_ME_B,
            (BOT_ROW, 5, _, _, false) | (_, _, BOT_ROW, 5, false) => D_ME_S,
            // 6
            (NUM_ROW, 5, _, _, true) | (_, _, NUM_ROW, 5, true) => D_BU_B,
            (NUM_ROW, 5, _, _, false) | (_, _, NUM_ROW, 5, false) => D_BU_S,
            _ => BASE_EFF,
        };
    }

    // NOTE: Assumes that both keys are on the same hand
    fn get_pinky_eff(this: (usize, usize), that: (usize, usize), is_last: bool) -> f64 {
        if !((10..=11).contains(&this.1) || (10..=11).contains(&that.1)) {
            return 1.0;
        }

        return match (this.0, this.1, that.0, that.1, is_last) {
            (NUM_ROW, 10, _, _, true) | (_, _, NUM_ROW, 10, true) => 0.2,
            (NUM_ROW, 11, _, _, true) | (_, _, NUM_ROW, 11, true) => 0.2,
            (TOP_ROW, 10, _, _, true) | (_, _, TOP_ROW, 10, true) => 0.6,
            (TOP_ROW, 11, _, _, true) | (_, _, TOP_ROW, 11, true) => 0.4,
            (HOME_ROW, 10, _, _, true) | (_, _, HOME_ROW, 10, true) => 0.8,
            (NUM_ROW, 10, _, _, false) | (_, _, NUM_ROW, 10, false) => 0.2,
            (NUM_ROW, 11, _, _, false) | (_, _, NUM_ROW, 11, false) => 0.2,
            (TOP_ROW, 10, _, _, false) | (_, _, TOP_ROW, 10, false) => 0.8,
            (TOP_ROW, 11, _, _, false) | (_, _, TOP_ROW, 11, false) => 0.7,
            (HOME_ROW, 10, _, _, false) | (_, _, HOME_ROW, 10, false) => 0.9,
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
