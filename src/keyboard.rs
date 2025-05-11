use rand::{Rng as _, rngs::SmallRng};

#[derive(Clone)]
pub struct Keyboard {
    kb_vec: Vec<Vec<(u8, u8)>>,
    valid_locations: Vec<((u8, u8), Vec<(usize, usize)>)>,
    slot_ascii: Vec<Option<(usize, usize)>>,
    last_key_idx: Option<(usize, usize)>,
    prev_key_idx: Option<(usize, usize)>,
    generation: usize,
    id: usize,
    lineage: String,
    evaluated: bool,
    score: f64,
    left_uses: f64,
    right_uses: f64,
    is_elite: bool,
}

enum KeyCompare {
    Mult(f64),
    MultLeft(f64),
    MultRight(f64),
    Mismatch,
}

// TODO: A better architecture for this is to let the user bring in the valid keys from a config
// file rather than actually altering the source code. So then error propagation would be the
// better design
// TODO: The meta issue with this struct is how much it relies on the KeyTemplate enum. It's
// compile time, which is good, but it's exterior, which is bad
// TODO: Need to rebuild the qwerty creation and add dvorak
// NOTE: Any impl methods that are private and/or do not take external input assume that the struct
// data is correct. These methods are not meant to be portable
impl Keyboard {
    const DEFAULT_KEY: (u8, u8) = (b' ', b' ');
    const INDEX: char = 'i';
    const MIDDLE: char = 'm';
    const RING: char = 'r';
    const PINKY: char = 'p';
    const LEFT: char = 'l';
    const RIGHT: char = 'r';

    // TODO: No assertion message
    pub fn create_origin(id_in: usize) -> Self {
        const NUM_ROW_CAPACITY: usize = 12;
        const TOP_ROW_CAPACITY: usize = 12;
        const HOME_ROW_CAPACITY: usize = 11;
        const BOT_ROW_CAPACITY: usize = 10;

        let mut kb_vec: Vec<Vec<(u8, u8)>> = vec![
            vec![Self::DEFAULT_KEY; NUM_ROW_CAPACITY],
            vec![Self::DEFAULT_KEY; TOP_ROW_CAPACITY],
            vec![Self::DEFAULT_KEY; HOME_ROW_CAPACITY],
            vec![Self::DEFAULT_KEY; BOT_ROW_CAPACITY],
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
                assert!(*col != Self::DEFAULT_KEY);
            }
        }

        let mut slot_ascii: Vec<Option<(usize, usize)>> = vec![None; 128];
        for i in 0..kb_vec.len() {
            for j in 0..kb_vec[i].len() {
                slot_ascii[kb_vec[i][j].0 as usize] = Some((i, j));
                slot_ascii[kb_vec[i][j].1 as usize] = Some((i, j));
            }
        }

        let generation = 0;
        let id: usize = id_in;
        let lineage: String = format!("{}.{}", generation, id);

        return Self {
            kb_vec,
            valid_locations,
            slot_ascii,
            last_key_idx: None,
            prev_key_idx: None,
            generation,
            id,
            lineage,
            evaluated: false,
            score: 0.0,
            left_uses: 0.0,
            right_uses: 0.0,
            is_elite: false,
        };
    }

    fn get_keys() -> Vec<((u8, u8), Vec<(usize, usize)>)> {
        return vec![
            ((b'1', b'!'), vec![(0, 0)]),
            ((b'2', b'@'), vec![(0, 1)]),
            ((b'3', b'#'), vec![(0, 2)]),
            ((b'4', b'$'), vec![(0, 3)]),
            ((b'5', b'%'), vec![(0, 4)]),
            ((b'6', b'^'), vec![(0, 5)]),
            ((b'7', b'&'), vec![(0, 6)]),
            ((b'8', b'*'), vec![(0, 7)]),
            ((b'9', b'('), vec![(0, 8)]),
            ((b'0', b')'), vec![(0, 9)]),
            ((b'[', b'{'), vec![(0, 10)]),
            ((b']', b'}'), vec![(0, 11)]),
            // ((b',', b'<'), vec![(1, 0), (3, 0)]),
            ((b',', b'<'), Self::alpha_slots(&vec![(3, 7)])),
            // ((b'.', b'>'), vec![(1, 1), (3, 1)]),
            ((b'.', b'>'), Self::alpha_slots(&vec![(3, 8)])),
            ((b'-', b'_'), vec![(1, 10)]),
            ((b'=', b'+'), vec![(1, 11)]),
            ((b'/', b'?'), vec![(2, 10)]),
            // ((b';', b':'), vec![(3, 4)]),
            ((b';', b':'), Self::alpha_slots(&vec![(2, 9)])),
            // ((b'\'', b'"'), vec![(1, 5)]),
            ((b'\'', b'"'), Self::alpha_slots(&vec![(2, 10)])),
            (
                (b'a', b'A'),
                Self::alpha_slots(&vec![(1, 0), (2, 0), (3, 0)]),
            ),
            (
                (b'b', b'B'),
                Self::alpha_slots(&vec![(1, 4), (2, 4), (3, 4)]),
            ),
            (
                (b'c', b'C'),
                Self::alpha_slots(&vec![(1, 2), (2, 2), (3, 2)]),
            ),
            (
                (b'd', b'D'),
                Self::alpha_slots(&vec![(1, 2), (2, 2), (3, 2)]),
            ),
            (
                (b'e', b'E'),
                Self::alpha_slots(&vec![(1, 2), (2, 2), (3, 2)]),
            ),
            (
                (b'f', b'F'),
                Self::alpha_slots(&vec![(1, 3), (2, 3), (3, 3)]),
            ),
            (
                (b'g', b'G'),
                Self::alpha_slots(&vec![(1, 4), (2, 4), (3, 4)]),
            ),
            (
                (b'h', b'H'),
                Self::alpha_slots(&vec![(1, 5), (2, 5), (3, 5)]),
            ),
            (
                (b'i', b'I'),
                Self::alpha_slots(&vec![(1, 7), (2, 7), (3, 7)]),
            ),
            (
                (b'j', b'J'),
                Self::alpha_slots(&vec![(1, 6), (2, 6), (3, 6)]),
            ),
            (
                (b'k', b'K'),
                Self::alpha_slots(&vec![(1, 7), (2, 7), (3, 7)]),
            ),
            (
                (b'l', b'L'),
                Self::alpha_slots(&vec![(1, 8), (2, 8), (3, 8)]),
            ),
            (
                (b'm', b'M'),
                Self::alpha_slots(&vec![(1, 6), (2, 6), (3, 6)]),
            ),
            (
                (b'n', b'N'),
                Self::alpha_slots(&vec![(1, 5), (2, 5), (3, 5)]),
            ),
            (
                (b'o', b'O'),
                Self::alpha_slots(&vec![(1, 8), (2, 8), (3, 8)]),
            ),
            (
                (b'p', b'P'),
                Self::alpha_slots(&vec![(1, 9), (2, 9), (3, 9)]),
            ),
            (
                (b'q', b'Q'),
                Self::alpha_slots(&vec![(1, 0), (2, 0), (3, 0)]),
            ),
            (
                (b'r', b'R'),
                Self::alpha_slots(&vec![(1, 3), (2, 3), (3, 3)]),
            ),
            (
                (b's', b'S'),
                Self::alpha_slots(&vec![(1, 1), (2, 1), (3, 1)]),
            ),
            (
                (b't', b'T'),
                Self::alpha_slots(&vec![(1, 4), (2, 4), (3, 4)]),
            ),
            (
                (b'u', b'U'),
                Self::alpha_slots(&vec![(1, 6), (2, 6), (3, 6)]),
            ),
            (
                (b'v', b'V'),
                Self::alpha_slots(&vec![(1, 3), (2, 3), (3, 3)]),
            ),
            (
                (b'w', b'W'),
                Self::alpha_slots(&vec![(1, 1), (2, 1), (3, 1)]),
            ),
            (
                (b'x', b'X'),
                Self::alpha_slots(&vec![(1, 1), (2, 1), (3, 1)]),
            ),
            (
                (b'y', b'Y'),
                Self::alpha_slots(&vec![(1, 5), (2, 5), (3, 5)]),
            ),
            (
                (b'z', b'Z'),
                Self::alpha_slots(&vec![(1, 0), (2, 0), (3, 0)]),
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

    // fn not_home(exclusions: &[(usize, usize)]) -> Vec<(usize, usize)> {
    //     let slot_groups: Vec<Vec<(usize, usize)>> = vec![Self::top_row(), Self::bottom_row()];
    //
    //     let mut slot_groups_flat: Vec<(usize, usize)> =
    //         slot_groups.into_iter().flatten().collect();
    //     slot_groups_flat.retain(|x| return !exclusions.contains(x));
    //
    //     return slot_groups_flat;
    // }

    // fn power_ten(exclusions: &[(usize, usize)]) -> Vec<(usize, usize)> {
    //     let mut power_ten = vec![
    //         (2, 0),
    //         (2, 1),
    //         (2, 2),
    //         (2, 3),
    //         (2, 6),
    //         (2, 7),
    //         (2, 8),
    //         (2, 9),
    //     ];
    //
    //     power_ten.retain(|x| return !exclusions.contains(x));
    //
    //     return power_ten;
    // }

    fn top_row() -> Vec<(usize, usize)> {
        return vec![
            // Omitted due to , and . keys
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

    // fn get_k() -> Vec<(usize, usize)> {
    //     return vec![(3, 1), (3, 2), (3, 3), (1, 9), (3, 9)];
    // }

    // TODO: No assertion message
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
            if kb_vec[row][col] != Self::DEFAULT_KEY {
                continue;
            }

            kb_vec[row][col] = keys[idx].0;

            if Self::place_keys(kb_vec, keys, idx + 1) {
                return true;
            } else {
                kb_vec[row][col] = Self::DEFAULT_KEY;
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
            lineage: format!("{}-{}.{}", kb.get_lineage(), gen_input, id_in),
            evaluated: kb.evaluated,
            score: kb.get_score(),
            left_uses: 0.0,
            right_uses: 0.0,
            is_elite: false,
        };
    }

    // NOTE: This function assumes the keys are properly setup such that there is always at least
    // one valid option to shuffle to
    // TODO: Not assertion messages
    pub fn shuffle(&mut self, rng: &mut SmallRng, amt: usize) {
        // The top row and the side symbol keys are purposefully avoided
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

    // TODO: No panic message
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
    // TODO: How to factor out...
    fn get_efficiency(&mut self, this_key_idx: u8) -> f64 {
        const DEFAULT_EFFICIENCY: f64 = 1.0;

        let this_key: (usize, usize) =
            if let Some(&Some(slot)) = self.slot_ascii.get(this_key_idx as usize) {
                slot
            } else {
                self.prev_key_idx = self.last_key_idx;
                self.last_key_idx = None;

                return 0.0;
            };

        let this_row: usize = this_key.0;
        let this_col: usize = this_key.1;
        let this_hand: char = Self::get_hand(this_col);
        let mut eff: f64 = DEFAULT_EFFICIENCY;

        if this_hand == Self::RIGHT {
            self.right_uses += 1.0;
        } else {
            self.left_uses += 1.0;
        }

        eff *= Self::get_single_key_mult(this_key);

        let last_compare: Option<KeyCompare> = self
            .last_key_idx
            .map(|last_key| return Self::compare_keys(this_key, last_key, true));

        let has_bigram: bool = !(matches!(last_compare, None | Some(KeyCompare::Mismatch)));
        let prev_compare: Option<KeyCompare> = if has_bigram {
            None
        } else {
            self.prev_key_idx
                .map(|prev_key| return Self::compare_keys(this_key, prev_key, false))
        };

        let has_skipgram: bool = !(matches!(prev_compare, None | Some(KeyCompare::Mismatch)));

        self.prev_key_idx = self.last_key_idx;
        self.last_key_idx = Some(this_key);

        let mut has_left_move: bool = false;
        // TODO: Outline this probably
        // TODO: This should also early return when complete
        if !(has_bigram || has_skipgram) {
            let dist_from_home: usize = this_row.abs_diff(2);

            if dist_from_home == 1 {
                eff *= 0.8;
            } else if dist_from_home == 2 {
                eff *= 0.6;
            }

            if this_hand == 'l' && dist_from_home > 0 {
                // TODO: tecnically correct but confusing naming
                has_left_move = true;
            }
        }

        if has_bigram {
            match last_compare {
                Some(KeyCompare::Mult(x)) => eff *= x,
                Some(KeyCompare::MultLeft(x)) => {
                    eff *= x;
                    has_left_move = true;
                }
                _ => {}
            }
        } else if has_skipgram {
            match prev_compare {
                Some(KeyCompare::Mult(x)) => eff *= x,
                Some(KeyCompare::MultLeft(x)) => {
                    eff *= x;
                    has_left_move = true;
                }
                _ => {}
            }
        }

        // The diagonal of the left keys goes against the shape of the hand
        if has_left_move && has_skipgram {
            eff *= 0.9;
        } else if has_left_move {
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
                self.score += self.get_efficiency(*b);
            }
        }

        let lr_diff: f64 = (self.left_uses - self.right_uses).abs();
        let max_usage: f64 = self.left_uses.max(self.right_uses);
        let diff_pct: f64 = (max_usage - lr_diff) / max_usage;
        self.score *= diff_pct;

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

    // NOTE: The algorithm cannot inherently know that the ring and pinky are less dexterous than
    // the index and middle fingers. Rather than make micro adjustments through, just do a blanket
    // deduction here. The deduction for ring and pinky is small because, if it is too large, the
    // algorithm will start creating SFBs of common pairs on the index and middle fingers to avoid
    // the ring and pinky fingers. The deduction is also the same for both the ring and middle
    // fingers, because some people have a different one of the two they prefer
    // TODO: One edit you could make the the ring/pinky adjustment is, add a larger downgrade if
    // they move
    // NOTE: Because the algorithm does not inherently know that the bottom and number rows are
    // harder to reach than the home and top alpha rows, a deduction is added here. Like the
    // ring/pinky deduction, it is not overly large in order to avoid the algorithm putting keys in
    // convoluted places. Because the algorithm later penalizes moving off the home row, a
    // generalized favoritism for it is not added here
    fn get_single_key_mult(key: (usize, usize)) -> f64 {
        let row: usize = key.0;
        let col: usize = key.1;
        let finger: char = Self::get_finger(col);

        let mut eff_mult: f64 = 1.0;

        if finger == Self::RING || finger == Self::PINKY {
            eff_mult *= 0.8;
        }

        if row == 3 {
            eff_mult *= 0.8;
        } else if row == 0 {
            eff_mult *= 0.6;
        }

        return eff_mult;
    }

    fn compare_keys(key_x: (usize, usize), key_y: (usize, usize), is_last: bool) -> KeyCompare {
        let key_x_col: usize = key_x.1;
        let key_x_hand: char = Self::get_hand(key_x_col);
        let key_y_col: usize = key_y.1;
        let key_y_hand: char = Self::get_hand(key_y_col);
        if key_x_hand != key_y_hand {
            return KeyCompare::Mismatch;
        }

        let key_x_row: usize = key_x.0;
        let key_y_row: usize = key_y.0;
        let key_x_finger: char = Self::get_finger(key_x_col);
        let key_y_finger: char = Self::get_finger(key_y_col);
        let mut mult: f64 = 1.0;

        let row_eff: f64 = Self::get_repeat_row_mult(key_x_row, key_y_row, is_last);
        mult *= row_eff;
        // NOTE: These extension effeciencies are meant to track the impact of moving the index or
        // the pinky off the home columns on the entire hand
        let index_ext_eff: f64 = Self::handle_index(key_x, key_y, is_last);
        mult *= index_ext_eff;
        mult *= Self::handle_right_pinky(key_x, key_y, is_last);

        let mut index_mv_eff: f64 = 1.0;
        if key_x_finger != key_y_finger {
            mult *= Self::get_base_sf_penalty(is_last);

            index_mv_eff = Self::get_repeat_col_mult(key_x_col, key_y_col, is_last);
            mult *= index_mv_eff;
        } else if key_x_row == key_y_row {
            mult *= Self::check_roll(key_x_col, key_y_col, is_last);
        } else {
            // No need here to save a value to check left hand efficiency. This branch requires a
            // row move, which has already been checked
            mult *= Self::check_combo(key_x, key_y, is_last);
            mult *= Self::check_scissor(key_x_col, key_y_col, key_x_row, key_y_row, is_last);
        }

        let did_mv: bool = row_eff < 1.0 || index_ext_eff < 1.0 || index_mv_eff < 1.0;
        if key_x_hand == Self::LEFT && did_mv {
            return KeyCompare::MultLeft(mult);
        }

        return KeyCompare::Mult(mult);
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

        if bot == Self::INDEX || top == Self::MIDDLE || (top == Self::RING && bot == Self::PINKY) {
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

    fn get_repeat_row_mult(this_row: usize, last_row: usize, last: bool) -> f64 {
        let row_diff: usize = this_row.abs_diff(last_row);

        return match (row_diff, last) {
            (0, true) => 0.8,
            (1, true) => 0.6,
            (2, true) => 0.4,
            (3, true) => 0.2,
            (0, false) => 0.9,
            (1, false) => 0.8,
            (2, false) => 0.7,
            (3, false) => 0.6,
            _ => 1.0,
        };
    }

    fn handle_index(this: (usize, usize), last: (usize, usize), is_last: bool) -> f64 {
        if !((4..=5).contains(&this.1) || (4..=5).contains(&last.1)) {
            return 1.0;
        }

        // These motions are straightforward
        let this_t: bool = this.0 == 1 && this.1 == 4;
        let last_t: bool = last.0 == 1 && last.1 == 4;
        let this_v: bool = this.0 == 3 && this.1 == 3;
        let last_v: bool = last.0 == 3 && last.1 == 3;
        let tv: bool = (this_t && last_v) || (this_v && last_t);
        let this_u: bool = this.0 == 1 && this.1 == 6;
        let last_u: bool = last.0 == 1 && last.1 == 6;
        let this_n: bool = this.0 == 3 && this.1 == 5;
        let last_n: bool = last.0 == 3 && last.1 == 5;
        let un: bool = (this_u && last_n) || (this_n && last_u);
        if tv || un {
            return 1.0;
        }

        if this_t || last_t || this_n || last_n {
            if is_last {
                return 0.6;
            } else {
                return 0.8;
            }
        }

        // B or Y
        let is_b: bool = this.1 == 4 && this.0 == 3 || last.1 == 4 && last.0 == 3;
        let is_y: bool = this.1 == 5 && this.0 == 1 || last.1 == 5 && last.0 == 1;
        if is_b || is_y {
            if is_last {
                return 0.2;
            } else {
                return 0.6;
            }
        }

        // B or Y
        let is_g: bool = this.1 == 4 && this.0 == 2 || last.1 == 4 && last.0 == 2;
        let is_h: bool = this.1 == 5 && this.0 == 2 || last.1 == 5 && last.0 == 2;
        if is_g || is_h {
            if is_last {
                return 0.8;
            } else {
                return 0.9;
            }
        }

        return 1.0;
    }

    fn handle_right_pinky(this: (usize, usize), last: (usize, usize), is_last: bool) -> f64 {
        if !((10..=11).contains(&this.1) || (10..=11).contains(&last.1)) {
            return 1.0;
        }

        return match (this.0, this.1, last.0, last.1, is_last) {
            (0, 10, _, _, true) => 0.2,
            (0, 11, _, _, true) => 0.2,
            (1, 10, _, _, true) => 0.6,
            (1, 11, _, _, true) => 0.4,
            (2, 10, _, _, true) => 0.8,
            (0, 10, _, _, false) => 0.6,
            (0, 11, _, _, false) => 0.6,
            (1, 10, _, _, false) => 0.8,
            (1, 11, _, _, false) => 0.7,
            (2, 10, _, _, false) => 0.9,
            (_, _, 0, 10, true) => 0.2,
            (_, _, 0, 11, true) => 0.2,
            (_, _, 1, 10, true) => 0.6,
            (_, _, 1, 11, true) => 0.4,
            (_, _, 2, 10, true) => 0.8,
            (_, _, 0, 10, false) => 0.6,
            (_, _, 0, 11, false) => 0.6,
            (_, _, 1, 10, false) => 0.8,
            (_, _, 1, 11, false) => 0.7,
            (_, _, 2, 10, false) => 0.9,
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

    // TODO: Oddity - Middle > Index rolls on the right hand aren't the best but they're fine on
    // the left
    fn check_roll(this_col: usize, last_col: usize, last: bool) -> f64 {
        let this_center_dist: usize = Self::get_center_distance(this_col);
        let last_center_dist: usize = Self::get_center_distance(last_col);

        return match (this_center_dist < last_center_dist, last) {
            (true, true) => 1.2,
            (true, false) => 1.1,
            _ => 1.0,
        };
    }

    // NOTE: This function assumes we've already verified that we're on the same hand
    // NOTE: I've seen "non-adjacent" scissors described before, but that should be possible to
    // handle using the normal rules
    // TODO: Long function signature
    fn check_scissor(
        this_col: usize,
        last_col: usize,
        this_row: usize,
        last_row: usize,
        last: bool,
    ) -> f64 {
        if this_col.abs_diff(last_col) > 1 {
            return 1.0;
        }

        return match (this_row.abs_diff(last_row), last) {
            (2, true) => 0.6,
            (3, true) => 0.4,
            (2, false) => 0.8,
            (3, false) => 0.7,
            _ => 1.0,
        };
    }

    // TODO: Better, but will still need to be redone for final display
    pub fn display_keyboard(&self) {
        for row in &self.kb_vec {
            let mut chars: Vec<char> = Vec::new();
            for element in row {
                let char = element.0 as char;
                chars.push(char);
            }
            println!("{:?}", chars);
        }
    }

    // Info display
    pub fn get_lineage(&self) -> &str {
        return &self.lineage;
    }

    pub fn get_score(&self) -> f64 {
        return self.score;
    }

    pub fn get_generation(&self) -> usize {
        return self.generation;
    }

    pub fn get_id(&self) -> usize {
        return self.id;
    }

    // For population management
    pub fn get_vec_ref(&self) -> &[Vec<(u8, u8)>] {
        return &self.kb_vec;
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
}
