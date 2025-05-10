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
            ((b',', b'<'), vec![(1, 0), (3, 0)]),
            ((b'.', b'>'), vec![(1, 1), (3, 1)]),
            ((b'-', b'_'), vec![(1, 10)]),
            ((b'=', b'+'), vec![(1, 11)]),
            ((b'/', b'?'), vec![(2, 10)]),
            ((b';', b':'), vec![(3, 4)]),
            ((b'\'', b'"'), vec![(1, 5)]),
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
    fn get_efficiency(&mut self, input: u8) -> f64 {
        const DEFAULT_EFFICIENCY: f64 = 1.0;

        let key_idx: (usize, usize) =
            if let Some(&Some(slot)) = self.slot_ascii.get(input as usize) {
                slot
            } else {
                self.last_key_idx = None;
                self.prev_key_idx = self.last_key_idx;

                return 0.0;
            };

        let this_row: usize = key_idx.0;
        let this_col: usize = key_idx.1;
        let this_finger: char = Self::get_finger(this_col);

        let mut efficiency: f64 = DEFAULT_EFFICIENCY;

        // let this_hand_right: bool = this_col >= 5;
        let this_hand: char = Self::get_hand(this_col);
        if this_hand == 'r' {
            self.right_uses += 1.0;
        } else {
            self.left_uses += 1.0;
        }

        // Generally punish ring/pinky usage
        if this_finger == 'r' || this_finger == 'p' {
            efficiency *= 0.6;
        }

        // Generally disfavor the bottom row
        if this_row == 3 {
            efficiency *= 0.1;
        }

        // Generally favor the home row
        if this_row == 2 {
            efficiency *= 2.0;
        }

        if this_row == 1 {
            efficiency *= 0.6;
        }

        if (4..=5).contains(&this_col) {
            efficiency *= 0.2;
        }

        // Avoid using the same finger twice in a row
        if let Some(last_key) = self.last_key_idx {
            let last_col: usize = last_key.1;
            let last_finger: char = Self::get_finger(last_col);

            if this_finger == last_finger {
                efficiency *= 0.4;
            }
        }

        // Further penalize same finger usage if it moves rows
        let mut row_move_deduction: f64 = 1.0;
        if let Some(last_key) = self.last_key_idx {
            let last_row: usize = last_key.0;
            let last_col: usize = last_key.1;
            let last_finger: char = Self::get_finger(last_col);

            if this_finger == last_finger {
                let row_diff = last_row.abs_diff(this_row);
                if row_diff == 1 {
                    row_move_deduction = 0.8;
                } else if row_diff == 2 {
                    row_move_deduction = 0.6;
                } else if row_diff == 3 {
                    row_move_deduction = 0.4;
                }
            }
        } else if let Some(prev_key) = self.prev_key_idx {
            let prev_row: usize = prev_key.0;
            let prev_col: usize = prev_key.1;
            let prev_finger: char = Self::get_finger(prev_col);

            if this_finger == prev_finger {
                let row_diff = prev_row.abs_diff(this_row);
                if row_diff == 1 {
                    row_move_deduction = 0.9;
                } else if row_diff == 2 {
                    row_move_deduction = 0.8;
                } else if row_diff == 3 {
                    row_move_deduction = 0.7;
                }
            }
        } else if this_row.abs_diff(2) == 1 {
            row_move_deduction = 0.8;
        } else if this_row.abs_diff(2) == 2 {
            row_move_deduction = 0.6;
        }

        efficiency *= row_move_deduction;

        let mut index_move_deduction = 1.0;
        if this_finger == 'i' {
            if let Some(last_key) = self.last_key_idx {
                let last_row: usize = last_key.0;
                let last_col: usize = last_key.1;

                if (3..5).contains(&this_col)
                    && (3..5).contains(&last_col)
                    && this_col.abs_diff(last_col) == 1
                {
                    let b: bool =
                        (this_row == 3 && this_col == 4) || (last_row == 3 && last_col == 4);
                    let tv: bool =
                        (this_row == 1 && this_col == 4 && last_row == 3 && last_col == 3)
                            || (this_row == 3 && this_col == 3 && last_row == 1 && last_col == 4);

                    if b {
                        index_move_deduction = 0.4;
                    } else if !tv {
                        index_move_deduction = 0.8;
                    }
                }

                if (5..7).contains(&this_col)
                    && (5..7).contains(&last_col)
                    && this_col.abs_diff(last_col) == 1
                {
                    let un: bool =
                        (this_row == 1 && this_col == 6 && last_row == 3 && last_col == 5)
                            || (this_row == 3 && this_col == 5 && last_row == 1 && last_col == 6);
                    let y: bool =
                        (this_row == 1 && this_col == 5) || (last_row == 1 && last_col == 5);

                    if y {
                        index_move_deduction = 0.4;
                    } else if !un {
                        index_move_deduction = 0.8;
                    }
                }

                efficiency *= index_move_deduction;
            } else if let Some(prev_key) = self.prev_key_idx {
                let prev_row: usize = prev_key.0;
                let prev_col: usize = prev_key.1;

                if (3..5).contains(&this_col)
                    && (3..5).contains(&prev_col)
                    && this_col.abs_diff(prev_col) == 1
                {
                    let b: bool =
                        (this_row == 3 && this_col == 4) || (prev_row == 3 && prev_col == 4);
                    let tv: bool =
                        (this_row == 1 && this_col == 4 && prev_row == 3 && prev_col == 3)
                            || (this_row == 3 && this_col == 3 && prev_row == 1 && prev_col == 4);

                    if b {
                        index_move_deduction *= 0.7;
                    } else if !tv {
                        index_move_deduction = 0.9;
                    }
                }

                if (5..7).contains(&this_col)
                    && (5..7).contains(&prev_col)
                    && this_col.abs_diff(prev_col) == 1
                {
                    let un: bool =
                        (this_row == 1 && this_col == 6 && prev_row == 3 && prev_col == 5)
                            || (this_row == 3 && this_col == 5 && prev_row == 1 && prev_col == 6);
                    let y: bool =
                        (this_row == 1 && this_col == 5) || (prev_row == 1 && prev_col == 5);

                    if y {
                        index_move_deduction = 0.7;
                    } else if !un {
                        index_move_deduction = 0.9;
                    }
                }

                efficiency *= index_move_deduction;
            }
        }

        // The keyboard is sloped against the natural shape of the left hand
        if this_hand == 'l' && row_move_deduction < 1.0 {
            efficiency *= 0.8;
        }

        // Scissors and rolls
        if let Some(last_key) = self.last_key_idx {
            let last_row: usize = last_key.0;
            let last_col: usize = last_key.1;
            let last_finger: char = Self::get_finger(last_col);
            let last_hand: char = Self::get_hand(last_col);

            if this_hand == last_hand && last_row != this_row && last_finger != this_finger {
                let is_good_combo: bool = if last_row > this_row {
                    Self::is_good_combo(last_finger, this_finger)
                } else {
                    Self::is_good_combo(this_finger, last_finger)
                };

                if this_row > last_row {
                    efficiency *= 0.8;
                }

                if !is_good_combo {
                    efficiency *= 0.8;
                }

                if is_good_combo
                    && Self::get_center_distance(this_row) < Self::get_center_distance(last_row)
                {
                    efficiency *= 1.2;
                }

                if last_row == this_row
                    && Self::get_center_distance(this_row) < Self::get_center_distance(last_row)
                {
                    efficiency *= 1.8;
                }

                if last_row == this_row
                    && Self::get_center_distance(this_row) > Self::get_center_distance(last_row)
                {
                    efficiency *= 1.2;
                }

                //Scissor
                // TODO: Not totally sure how to grade this. Even good combo scissors on the
                // ring/pinky tend to be tough, but they're okay with the middle and index if it's
                // a good combo
                // 3 row scissors don't need extra deduction because we already have addressed row
                //   jumps
                if last_col.abs_diff(this_col) == 1 && last_row.abs_diff(this_row) > 1 {
                    if is_good_combo {
                        efficiency *= 0.6;
                    } else {
                        efficiency *= 0.1;
                    }
                }

                // TODO: This does not handle a case like DT
                if last_col.abs_diff(this_col) == 1
                    && last_row.abs_diff(this_row) == 1
                    && !is_good_combo
                {
                    efficiency *= 0.8;
                }
            }
        } else if let Some(prev_key) = self.prev_key_idx {
            let prev_row: usize = prev_key.0;
            let prev_col: usize = prev_key.1;
            let prev_finger: char = Self::get_finger(prev_col);
            let prev_hand: char = Self::get_hand(prev_col);

            if this_hand == prev_hand && prev_row != this_row && prev_finger != this_finger {
                let is_good_combo: bool = if prev_row > this_row {
                    Self::is_good_combo(prev_finger, this_finger)
                } else {
                    Self::is_good_combo(this_finger, prev_finger)
                };

                if this_row > prev_row {
                    efficiency *= 0.9;
                }

                if !is_good_combo {
                    efficiency *= 0.9;
                }

                if is_good_combo
                    && Self::get_center_distance(this_row) < Self::get_center_distance(prev_row)
                {
                    efficiency *= 1.1;
                }

                if prev_row == this_row
                    && Self::get_center_distance(this_row) < Self::get_center_distance(prev_row)
                {
                    efficiency *= 1.4;
                }

                if prev_row == this_row
                    && Self::get_center_distance(this_row) > Self::get_center_distance(prev_row)
                {
                    efficiency *= 1.1;
                }
                //Scissor
                // TODO: Not totally sure how to grade this. Even good combo scissors on the
                // ring/pinky tend to be tough, but they're okay with the middle and index if it's
                // a good combo
                // 3 row scissors don't need extra deduction because we already have addressed row
                //   jumps
                if prev_col.abs_diff(this_col) == 1 && prev_row.abs_diff(this_row) > 1 {
                    if is_good_combo {
                        efficiency *= 0.8;
                    } else {
                        efficiency *= 0.5;
                    }
                }

                // TODO: This does not handle a case like DT
                if prev_col.abs_diff(this_col) == 1
                    && prev_row.abs_diff(this_row) == 1
                    && !is_good_combo
                {
                    efficiency *= 0.9;
                }
            }
        }

        self.last_key_idx = Some(key_idx);
        self.prev_key_idx = self.last_key_idx;

        return efficiency;
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

        // println!("left: {}, right: {}", self.left_uses, self.right_uses);
        let lr_diff: f64 = (self.left_uses - self.right_uses).abs();
        let max_usage: f64 = self.left_uses.max(self.right_uses);
        let diff_pct: f64 = (max_usage - lr_diff) / max_usage;
        // println!(
        //     "diff: {lr_diff}, max: {max_usage}, diff: {diff_pct}, score:{}",
        //     self.score
        // );
        self.score *= diff_pct;
        // println!("New score: {}", self.score);
        // println!();

        self.evaluated = true;
    }

    fn get_hand(col: usize) -> char {
        return match col {
            0..=4 => 'l',
            5..=11 => 'r',
            _ => 'u',
        };
    }
    // TODO: Make an enum?
    fn get_finger(col: usize) -> char {
        return match col {
            0 | 9..=11 => 'p',
            1 | 8 => 'r',
            2 | 7 => 'm',
            3..=6 => 'i',
            _ => 'u',
        };
    }

    fn is_good_combo(top: char, bot: char) -> bool {
        return bot == 'i' || top == 'm' || (top == 'r' && bot == 'p');
    }

    fn get_center_distance(row: usize) -> usize {
        return if row <= 4 {
            4 - row
        } else {
            row - 5
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
