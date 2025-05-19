extern crate alloc;

use alloc::collections::BTreeMap;
// use {alloc::collections::BTreeMap, std::collections::HashMap};

use rand::{Rng as _, rngs::SmallRng, seq::SliceRandom as _};

use crate::{
    kb_consts, kb_helper_consts,
    kb_helpers::{
        check_col, check_key_no_hist, compare_keys, get_hand, get_single_key_mult,
        get_valid_key_locs_sorted, place_keys,
    },
};

// TODO: Some of this stuff should be removed as we factor out the scoring
kb_consts!();

pub enum KeyCompare {
    Mult(f64),
    Mismatch,
}

#[derive(Clone)]
pub struct Keyboard {
    key_slots: BTreeMap<Slot, Key>,
    valid_slots: BTreeMap<Key, Vec<Slot>>,
    slot_ascii: Vec<Option<Slot>>,
    last_slot_idx: Option<Slot>,
    prev_slot_idx: Option<Slot>,
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
    /// # Panics
    /// The specs to build the keyboard properly are defined at compile time. If the specs are
    /// incorrect, this function or one of its sub-functions will panic
    pub fn create_primo(id_in: usize) -> Self {
        let mut key_slots: BTreeMap<Slot, Key> = BTreeMap::new();
        let valid_key_locs_sorted: Vec<(Key, Vec<Slot>)> = get_valid_key_locs_sorted();
        assert!(
            place_keys(&mut key_slots, &valid_key_locs_sorted, 0),
            "Unable to place all keys"
        );

        let valid_slots: BTreeMap<Key, Vec<Slot>> = valid_key_locs_sorted.into_iter().collect();

        let mut slot_ascii: Vec<Option<Slot>> = vec![None; ASCII_CNT];
        for (slot, key) in &key_slots {
            slot_ascii[usize::from(key.get_base())] = Some(*slot);
            slot_ascii[usize::from(key.get_shift())] = Some(*slot);
        }

        return Self {
            key_slots,
            valid_slots,
            slot_ascii,
            last_slot_idx: None,
            prev_slot_idx: None,
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
            key_slots: kb.key_slots.clone(),
            valid_slots: kb.valid_slots.clone(),
            slot_ascii: kb.slot_ascii.clone(),
            last_slot_idx: None,
            prev_slot_idx: None,
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

    // FUTURE: Right now, shuffling is restricted using constants. If we start adding unmovable
    // keys to the alpha area, perhaps we break key_slots into movable and unmovable keys. This
    // will speed up shuffling, but potentially slow down the ASCII lookup since two BTrees need to
    // be checked
    // FUTURE: It would be better if the shuffle amounts were read from a config
    // PERF: When getting this_key and that_key, the borrows are de-referenced and moved out of
    // scope so that the borrow checker doesn't complain when doing the swap. If this is a
    // performance issue, could hold onto the borrows until it is actually time to swap, then
    // figure out how to handle
    /// # Panics
    /// This function panics under the following conditions:
    /// - A valid key slot does not exist for the selected row and column
    /// - A key has no valid locations
    /// - If a valid key cannot be found in any of the valid slots
    /// - The swap key does not have any valid slots
    pub fn shuffle(&mut self, rng: &mut SmallRng, cnt: usize) {
        self.evaluated = false;

        for _ in 0..cnt {
            let this_row = rng.random_range(TOP_ROW..=BOT_ROW);
            let this_col = rng.random_range(L_PINKY..=R_PINKY);
            let this_slot = Slot::from_tuple((this_row, this_col));
            let this_key = self.key_slots[&this_slot];

            if let Some(vec) = self.valid_slots.get_mut(&this_key) {
                vec.shuffle(rng);
                if vec.len() == 1 {
                    continue;
                }
            } else {
                panic!("Valid slots not found for key {:?}", this_key);
            }

            let these_valid_slots = &self.valid_slots[&this_key];
            for slot in these_valid_slots {
                let that_row = slot.get_row();
                let that_col = slot.get_col();
                let that_slot = Slot::from_tuple((that_row, that_col));
                let that_key = self.key_slots[&that_slot];
                if this_key == that_key
                    || !self.valid_slots[&that_key].contains(&this_slot)
                    || self.valid_slots[&that_key].len() == 1
                {
                    continue;
                }

                self.key_slots.insert(this_slot, that_key);
                self.slot_ascii[usize::from(that_key.get_base())] = Some(this_slot);
                self.slot_ascii[usize::from(that_key.get_shift())] = Some(this_slot);

                self.key_slots.insert(that_slot, this_key);
                self.slot_ascii[usize::from(this_key.get_base())] = Some(that_slot);
                self.slot_ascii[usize::from(this_key.get_shift())] = Some(that_slot);

                break;
            }

            debug_assert_ne!(
                self.key_slots[&this_slot], this_key,
                "ERROR: Key {:?} at {},{} not changed",
                this_key, this_row, this_col
            );
        }
    }

    // pub fn mapped_swap(&mut self, swap_map: &HashMap<((usize, usize), (u8, u8)), (f64, f64)>) {
    //     let mut raw_out_scores: BTreeMap<(usize, usize), f64> = BTreeMap::new();
    //
    //     for i in 1..self.kb_vec.len() {
    //         for j in 0..self.kb_vec[i].len() {
    //             let key = self.kb_vec[i][j];
    //             let score = swap_map[&((i, j), key)].0;
    //             // let score: f64 = swap_map[]
    //             raw_out_scores.insert((i, j), score);
    //         }
    //     }
    // }

    // NOTE: A single major efficiency penalty at any point in the algorithm can cause the entire
    // layout to change. Be careful over-indexing for any particular factor
    fn get_efficiency(&mut self, this_key: Slot) -> f64 {
        let mut eff = BASE_EFF;

        let this_col = this_key.get_col();
        let this_hand = get_hand(this_col);
        if this_hand == RIGHT {
            self.right_uses += 1.0_f64;
        } else {
            self.left_uses += 1.0_f64;
        }

        eff *= get_single_key_mult(this_key);

        let last_compare: Option<KeyCompare> = self
            .last_slot_idx
            .map(|last_key| return compare_keys(this_key, last_key, true));
        if let Some(key_compare) = last_compare {
            match key_compare {
                KeyCompare::Mult(x) => return eff * x,
                KeyCompare::Mismatch => {}
            }
        }

        let prev_compare: Option<KeyCompare> = self
            .prev_slot_idx
            .map(|prev_key| return compare_keys(this_key, prev_key, false));
        if let Some(key_compare) = prev_compare {
            match key_compare {
                KeyCompare::Mult(x) => return eff * x,
                KeyCompare::Mismatch => {}
            }
        }

        eff *= check_key_no_hist(this_key);

        return eff;
    }

    pub fn eval(&mut self, corpus: &[String]) {
        if self.evaluated {
            return;
        }

        self.score = 0.0_f64;
        self.last_slot_idx = None;
        self.prev_slot_idx = None;
        self.left_uses = 0.0_f64;
        self.right_uses = 0.0_f64;

        for entry in corpus {
            for b in entry.as_bytes() {
                let this_key: Slot = if let Some(&Some(key)) = self.slot_ascii.get(usize::from(*b))
                {
                    key
                } else {
                    self.prev_slot_idx = self.last_slot_idx;
                    self.last_slot_idx = None;
                    continue;
                };

                self.score += self.get_efficiency(this_key);

                self.prev_slot_idx = self.last_slot_idx;
                self.last_slot_idx = Some(this_key);
            }
        }

        if self.left_uses < self.right_uses {
            self.score *= self.left_uses / self.right_uses;
        } else {
            self.score *= self.right_uses / self.left_uses;
        }

        self.evaluated = true;
    }

    // TODO: Very inefficient
    pub fn get_display_chars(&self) -> Vec<Vec<char>> {
        let mut num_row: Vec<char> = Vec::new();
        let mut top_row: Vec<char> = Vec::new();
        let mut home_row: Vec<char> = Vec::new();
        let mut bot_row: Vec<char> = Vec::new();

        for (slot, key) in &self.key_slots {
            if slot.get_row() == 0 {
                num_row.push(char::from(key.get_base()));
            } else if slot.get_row() == 1 {
                top_row.push(char::from(key.get_base()));
            } else if slot.get_row() == 2 {
                home_row.push(char::from(key.get_base()));
            } else if slot.get_row() == 3 {
                bot_row.push(char::from(key.get_base()));
            }
        }

        let mut display_chars: Vec<Vec<char>> = Vec::new();
        display_chars.push(num_row);
        display_chars.push(top_row);
        display_chars.push(home_row);
        display_chars.push(bot_row);

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
        if let Some(iter) = self.pos_iter.checked_add(1) {
            self.pos_iter = iter;
        } else {
            self.pos_iter = 0;
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Slot {
    row: usize,
    col: usize,
}

impl Slot {
    // PERF: If this is used in a hot loop, change to debug_assert
    pub fn from_tuple(source: (usize, usize)) -> Self {
        assert!(
            (NUM_ROW..=BOT_ROW).contains(&source.0),
            "Source row ({}) < num_row ({}) or > bottom row ({}) in slot.from_tuple",
            NUM_ROW,
            BOT_ROW,
            source.0,
        );

        assert!(
            check_col(source.0, source.1),
            "Col {} invalid for row {} in slot.from_tuple",
            source.0,
            source.1
        );

        return Self {
            row: source.0,
            col: source.1,
        };
    }

    pub fn get_row(&self) -> usize {
        return self.row;
    }

    pub fn get_col(&self) -> usize {
        return self.col;
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Key {
    base: u8,
    shift: u8,
}

impl Key {
    // PERF: If this is run in a hot loop, change to debug_assert
    pub fn from_tuple(source: (u8, u8)) -> Self {
        assert!(
            usize::from(source.0) <= ASCII_CNT,
            "Base key {} is not a valid ASCII char in Key::from_tuple",
            source.0
        );

        assert!(
            usize::from(source.1) <= ASCII_CNT,
            "Shift key {} is not a valid ASCII char in Key::from_tuple",
            source.0
        );

        return Self {
            base: source.0,
            shift: source.1,
        };
    }

    pub fn get_base(self) -> u8 {
        return self.base;
    }

    pub fn get_shift(self) -> u8 {
        return self.shift;
    }
}
