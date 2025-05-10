use std::ptr;

use {
    rand::{Rng as _, rngs::SmallRng},
    strum::IntoEnumIterator,
};

use crate::{
    key::{Key, LightKey},
    key_template::KeyTemplate,
};

#[derive(Clone)]
pub struct Keyboard {
    kb_vec: Vec<Vec<Key>>,
    slot_ascii: Vec<Option<(usize, usize)>>,
    last_key_idx: Option<(usize, usize)>,
    generation: usize,
    id: usize,
    lineage: String,
    evaluated: bool,
    score: f64,
    same_row_streak: f64,
    home_row_streak: f64,
    is_elite: bool,
}

// TODO: The meta issue with this struct is how much it relies on the KeyTemplate enum. It's
// compile time, which is good, but it's exterior, which is bad
// TODO: Need to rebuild the qwerty creation and add dvorak
impl Keyboard {
    const DEFAULT_KEY: (u8, u8) = (b' ', b' ');

    pub fn create_origin(id_in: usize) -> Self {
        const NUM_ROW_CAPACITY: usize = 12;
        const TOP_ROW_CAPACITY: usize = 12;
        const HOME_ROW_CAPACITY: usize = 11;
        const BOT_ROW_CAPACITY: usize = 10;

        let mut kb_tuple_vec: Vec<Vec<(u8, u8)>> = vec![
            vec![Self::DEFAULT_KEY; NUM_ROW_CAPACITY],
            vec![Self::DEFAULT_KEY; TOP_ROW_CAPACITY],
            vec![Self::DEFAULT_KEY; HOME_ROW_CAPACITY],
            vec![Self::DEFAULT_KEY; BOT_ROW_CAPACITY],
        ];
        let mut tuple_keys: Vec<((u8, u8), Vec<(usize, usize)>)> = Self::get_keys();

        let mut kb_tuple_vec_len: usize = 0;
        for vec in &kb_tuple_vec {
            kb_tuple_vec_len += vec.len();
        }
        debug_assert_eq!(kb_tuple_vec_len, tuple_keys.len());

        tuple_keys.sort_by(|a, b| {
            return a
                .1
                .len()
                .partial_cmp(&b.1.len())
                .unwrap_or(std::cmp::Ordering::Equal);
        });

        let placed: bool = Self::place_keys(&mut kb_tuple_vec, &tuple_keys, 0);
        debug_assert!(placed);

        tuple_keys.retain(|x| return x.1.len() > 1);

        let mut kb_vec: Vec<Vec<Key>> = vec![
            Vec::with_capacity(NUM_ROW_CAPACITY),
            Vec::with_capacity(TOP_ROW_CAPACITY),
            Vec::with_capacity(HOME_ROW_CAPACITY),
            Vec::with_capacity(BOT_ROW_CAPACITY),
        ];

        // SAFETY: Compile time values from within the struct
        unsafe {
            kb_vec[0].set_len(NUM_ROW_CAPACITY);
            kb_vec[1].set_len(TOP_ROW_CAPACITY);
            kb_vec[2].set_len(HOME_ROW_CAPACITY);
            kb_vec[3].set_len(BOT_ROW_CAPACITY);
        }

        let mut key_list: Vec<LightKey> = Vec::new();
        for template in KeyTemplate::iter() {
            key_list.push(LightKey::from_template(template));
        }

        let ptr: *mut Vec<Key> = kb_vec.as_mut_ptr();
        for template in KeyTemplate::iter() {
            let location: (usize, usize) = template.get_starting_location();
            let this_key = Key::from_template(template);

            // SAFETY: The indexes come from within the Keyboard struct and are built at compile
            // time
            unsafe {
                let row_ptr: *mut Vec<Key> = ptr.add(location.0);
                let inner_vec: &mut Vec<Key> = &mut *row_ptr;
                let elem_ptr: *mut Key = inner_vec.as_mut_ptr();
                elem_ptr.add(location.1).write(this_key);
            }
        }

        let mut slot_ascii: Vec<Option<(usize, usize)>> = vec![None; 128];
        for i in 0..kb_vec.len() {
            for j in 0..kb_vec[i].len() {
                slot_ascii[kb_vec[i][j].get_base() as usize] = Some((i, j));
                slot_ascii[kb_vec[i][j].get_shift() as usize] = Some((i, j));
            }
        }

        let generation = 0;
        let id: usize = id_in;
        let lineage: String = format!("{}.{}", generation, id);

        return Self {
            kb_vec,
            slot_ascii,
            last_key_idx: None,
            generation,
            id,
            lineage,
            evaluated: false,
            score: 0.0,
            same_row_streak: 1.0,
            home_row_streak: 1.0,
            is_elite: false,
        };
    }

    fn get_keys() -> Vec<((u8, u8), Vec<(usize, usize)>)> {
        return vec![
            ((b'a', b'A'), vec![(0, 0)]),
            ((b'b', b'B'), vec![(0, 1)]),
            ((b'c', b'C'), vec![(0, 2)]),
            ((b'd', b'D'), vec![(0, 3)]),
            ((b'e', b'E'), vec![(0, 4)]),
            ((b'f', b'F'), vec![(0, 5)]),
            ((b'g', b'G'), vec![(0, 6)]),
            ((b'h', b'H'), vec![(0, 7)]),
            ((b'i', b'I'), vec![(0, 8)]),
            ((b'j', b'J'), vec![(0, 9)]),
            ((b'k', b'K'), vec![(0, 10)]),
            ((b'l', b'L'), vec![(0, 11)]),
            ((b'm', b'M'), vec![(1, 0)]),
            ((b'n', b'N'), vec![(1, 1)]),
            ((b'o', b'O'), vec![(1, 10)]),
            ((b'p', b'P'), vec![(1, 11)]),
            ((b'q', b'Q'), vec![(2, 10)]),
            ((b'r', b'R'), vec![(1, 5), (3, 4)]),
            ((b's', b'S'), vec![(1, 5), (3, 4)]),
            ((b't', b'T'), Self::not_home(&vec![(1, 0)])),
            ((b'u', b'U'), Self::not_home(&vec![(1, 1)])),
            ((b'v', b'V'), vec![(2, 6), (2, 7)]),
            ((b'w', b'W'), Self::alpha_slots(&vec![(1, 3)])),
            ((b'x', b'X'), Self::alpha_slots(&vec![(1, 4)])),
            ((b'y', b'Y'), Self::alpha_slots(&vec![(1, 5)])),
            ((b'z', b'Z'), Self::alpha_slots(&vec![(1, 6)])),
            ((b',', b'<'), Self::alpha_slots(&vec![(1, 7)])),
            ((b'.', b'>'), Self::alpha_slots(&vec![(1, 8)])),
            ((b';', b':'), Self::not_home(&vec![(1, 9)])),
            ((b'/', b'?'), Self::major_home_slots(&vec![(2, 0)])),
            ((b'1', b'!'), Self::alpha_slots(&vec![(2, 1)])),
            ((b'2', b'@'), Self::alpha_slots(&vec![(2, 2)])),
            ((b'3', b'#'), Self::alpha_slots(&vec![(2, 3)])),
            ((b'4', b'$'), Self::alpha_slots(&vec![(2, 4)])),
            ((b'5', b'%'), Self::alpha_slots(&vec![(2, 5)])),
            ((b'6', b'^'), vec![(1, 8), (1, 9), (3, 0), (3, 9)]),
            ((b'7', b'&'), Self::alpha_slots(&vec![(2, 7)])),
            ((b'8', b'*'), Self::alpha_slots(&vec![(2, 8)])),
            ((b'9', b'('), Self::not_home(&vec![(3, 0)])),
            ((b'0', b')'), Self::not_home(&vec![(3, 1)])),
            ((b'-', b'_'), Self::alpha_slots(&vec![(3, 2)])),
            ((b'=', b'+'), Self::not_home(&vec![(3, 3)])),
            ((b'[', b'{'), Self::not_home(&vec![(3, 4)])),
            ((b']', b'}'), Self::alpha_slots(&vec![(3, 5)])),
            ((b'\'', b'"'), Self::alpha_slots(&vec![(3, 6)])),
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

    fn not_home(exclusions: &[(usize, usize)]) -> Vec<(usize, usize)> {
        let slot_groups: Vec<Vec<(usize, usize)>> = vec![Self::top_row(), Self::bottom_row()];

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

    fn major_home_slots(exclusions: &[(usize, usize)]) -> Vec<(usize, usize)> {
        let mut slots = vec![
            (2, 0),
            (2, 1),
            (2, 2),
            (2, 3),
            (2, 6),
            (2, 7),
            (2, 8),
            (2, 9),
        ];

        slots.retain(|x| return !exclusions.contains(x));

        return slots;
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

    fn place_keys(
        kb_vec: &mut Vec<Vec<(u8, u8)>>,
        keys: &Vec<((u8, u8), Vec<(usize, usize)>)>,
        idx: usize,
    ) -> bool {
        debug_assert!(keys[idx].1.len() > 0);

        for placement in &keys[idx].1 {
            let row: usize = placement.0;
            let col: usize = placement.1;

            if kb_vec[row][col] != Self::DEFAULT_KEY {
                continue;
            }

            kb_vec[row][col] = keys[idx].0;

            if idx == keys.len() - 1 {
                return true;
            }

            if Self::place_keys(kb_vec, keys, idx + 1) {
                return true;
            }

            kb_vec[row][col] = Self::DEFAULT_KEY;
        }

        return false;
    }

    pub fn mutate_from(kb: &Keyboard, gen_input: usize, id_in: usize) -> Self {
        let kb_vec: Vec<Vec<Key>> = kb.get_kb_vec().to_vec();
        let slot_ascii: Vec<Option<(usize, usize)>> = kb.get_slot_ascii().to_vec();
        let last_key_idx: Option<(usize, usize)> = None;

        let generation: usize = gen_input;
        let id: usize = id_in;
        let lineage: String = format!("{}-{}.{}", kb.get_lineage(), generation, id);

        let evaluated: bool = kb.get_eval_status();
        let score: f64 = kb.get_score();
        let is_elite: bool = kb.is_elite();

        return Self {
            kb_vec,
            slot_ascii,
            last_key_idx,
            generation,
            id,
            lineage,
            evaluated,
            score,
            same_row_streak: 1.0,
            home_row_streak: 1.0,
            is_elite,
        };
    }

    // NOTE: This function assumes the keys are properly setup such that there is always at least
    // one valid option to shuffle to
    pub fn shuffle(&mut self, rng: &mut SmallRng, amt: usize) {
        self.evaluated = false;

        let mut shuffled: usize = 0;
        while shuffled < amt {
            // The top row and the side symbol keys are purposefully avoided
            let row = rng.random_range(1..4);
            let col = rng.random_range(0..10);
            if self.kb_vec[row][col].is_static() {
                continue;
            }

            self.kb_vec[row][col].shuffle_valid_locations(rng);
            let cnt_valid = self.kb_vec[row][col].get_cnt_valid_locations();

            for i in 0..cnt_valid {
                let test = self.kb_vec[row][col].get_valid_location_at_idx(i);
                if test.0 == row && test.1 == col {
                    continue;
                }

                if self.kb_vec[test.0][test.1]
                    .get_valid_locations()
                    .contains(&(row, col))
                {
                    unsafe {
                        let ptr = self.kb_vec.as_mut_ptr();
                        let row1_ptr = ptr.add(row);
                        let row2_ptr = ptr.add(test.0);
                        let elem1_ptr = (*row1_ptr).as_mut_ptr().add(col);
                        let elem2_ptr = (*row2_ptr).as_mut_ptr().add(test.1);

                        ptr::swap(elem1_ptr, elem2_ptr);
                    }

                    self.slot_ascii[self.kb_vec[row][col].get_base() as usize] = Some((row, col));
                    self.slot_ascii[self.kb_vec[row][col].get_shift() as usize] = Some((row, col));
                    self.slot_ascii[self.kb_vec[test.0][test.1].get_base() as usize] =
                        Some((test.0, test.1));
                    self.slot_ascii[self.kb_vec[test.0][test.1].get_shift() as usize] =
                        Some((test.0, test.1));

                    shuffled += 1;
                    break;
                }
            }
        }
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

                self.same_row_streak = 1.0;
                self.home_row_streak = 1.0;
                return 0.0;
            };

        let this_row: usize = key_idx.0;
        let this_col: usize = key_idx.1;

        let mut efficiency: f64 = DEFAULT_EFFICIENCY;

        // Finger Rules
        if (2..=7).contains(&this_col) {
            efficiency *= 1.2;
        } else {
            efficiency *= 0.8;
        }

        if (this_col == 2 || this_col == 7) && this_row <= 1 {
            efficiency *= 1.20;
        }

        // Row Rules
        if this_row == 0 {
            efficiency *= 0.6;
        } else if this_row == 1 {
            efficiency *= 1.0;
        } else if this_row == 2 {
            efficiency *= 1.2;
        } else if this_row == 3 {
            efficiency *= 0.8;
        }

        // Handle Symbol Keys
        if this_col == 10 && this_row == 2 {
            efficiency *= 0.80;
        } else if (this_col == 10 && this_row <= 1) || this_col == 1 {
            efficiency *= 0.60;
        }

        let last_key: (usize, usize) = if let Some(key) = self.last_key_idx {
            key
        } else {
            self.last_key_idx = Some(key_idx);
            self.same_row_streak = 1.0;
            return efficiency;
        };

        let last_row: usize = last_key.0;
        let last_col: usize = last_key.1;

        let row_dist: usize = last_row.abs_diff(this_row);
        let col_dist: usize = last_col.abs_diff(this_col);

        let this_hand_right: bool = this_col >= 5;
        let last_hand_right: bool = last_col >= 5;

        if this_row == last_row {
            self.same_row_streak *= 1.2;
            efficiency *= self.same_row_streak;
        } else {
            self.same_row_streak = 1.0;
        }

        if this_row == 2 && last_row == 2 {
            self.home_row_streak *= 1.2;
            efficiency *= self.home_row_streak;
        } else {
            self.home_row_streak = 1.0;
        }

        if this_hand_right == last_hand_right {
            // Scisors
            if row_dist == 1 && col_dist >= 2 {
                efficiency *= 0.7;
            }
        }

        if last_col == this_col || (last_col >= 9 && this_col >= 9) {
            efficiency *= 0.8;
        }

        self.last_key_idx = Some(key_idx);

        return efficiency;
    }

    pub fn eval(&mut self, corpus: &[String]) {
        if self.evaluated {
            return;
        }

        self.score = 0.0;
        self.last_key_idx = None;
        self.same_row_streak = 1.0;
        self.home_row_streak = 1.0;

        for entry in corpus {
            for b in entry.as_bytes() {
                self.score += self.get_efficiency(*b);
            }
        }

        self.evaluated = true;
    }

    // TODO: Better, but will still need to be redone for final display
    pub fn display_keyboard(&self) {
        for row in &self.kb_vec {
            let mut chars: Vec<char> = Vec::new();
            for element in row {
                let char = element.get_base() as char;
                chars.push(char);
            }
            println!("{:?}", chars);
        }
    }

    // Pieces for mutation
    fn get_kb_vec(&self) -> &[Vec<Key>] {
        return &self.kb_vec;
    }

    fn get_slot_ascii(&self) -> &[Option<(usize, usize)>] {
        return &self.slot_ascii;
    }

    // Info display
    pub fn get_lineage(&self) -> &str {
        return &self.lineage;
    }

    pub fn get_eval_status(&self) -> bool {
        return self.evaluated;
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
    pub fn get_vec_ref(&self) -> &[Vec<Key>] {
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
