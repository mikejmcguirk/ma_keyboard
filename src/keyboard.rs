extern crate alloc;

use alloc::collections::BTreeMap;

use rand::{Rng as _, rngs::SmallRng, seq::SliceRandom as _};

use crate::{
    base_eff,
    corpus::get_corpus,
    edge_cols,
    eval_funcs::{check_key_no_hist, compare_slots, global_adjustments},
    kb_builders::{
        check_col, get_static_keys, get_swappable_keys, get_valid_key_locs_sorted,
        place_dvorak_keys, place_keys, place_keys_from_table, place_qwerty_keys,
    },
    keys,
    mapped_swap::{get_improvement, select_key, shuffle_check},
    most_cols, most_rows,
    population::SwapTable,
    swappable_keys,
};

const ASCII_CNT: usize = 128;

most_cols!();
edge_cols!();
most_rows!();
base_eff!();
swappable_keys!();

pub enum KeyCompare {
    Mult(f64),
    Mismatch,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Hand {
    Left,
    Right,
}

impl Hand {
    /// # Panics
    /// Panics if the input col is invalid
    pub fn from_slot(slot: Slot) -> Self {
        return match slot.get_col() {
            L_PINKY..=L_EXT => Hand::Left,
            R_EXT..=R_PIPE => Hand::Right,
            _ => panic!("Col {} is invalid in get_hand", slot.get_col()),
        };
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Finger {
    Pinky,
    Ring,
    Middle,
    Index,
}

impl Finger {
    pub fn from_slot(slot: Slot) -> Self {
        return match slot.get_col() {
            L_PINKY | R_PINKY..=R_PIPE => Finger::Pinky,
            L_RING | R_RING => Finger::Ring,
            L_MIDDLE | R_MIDDLE => Finger::Middle,
            L_INDEX..=R_INDEX => Finger::Index,
            _ => panic!("Col {} is invalid in get_hand", slot.get_col()),
        };
    }
}

// FUTURE: Valid_slots is a meta-population level construct
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
    last_score: f64,
    last_swap_a: (Slot, Key),
    last_swap_b: (Slot, Key),
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
            last_score: 0.0,
            last_swap_a: (Slot::from_tuple((0, 0)), Key::from_tuple((0, 0))),
            last_swap_b: (Slot::from_tuple((0, 0)), Key::from_tuple((0, 0))),
        };
    }

    pub fn create_qwerty() -> Self {
        let mut key_slots: BTreeMap<Slot, Key> = BTreeMap::new();
        let valid_key_locs_sorted: Vec<(Key, Vec<Slot>)> = get_valid_key_locs_sorted();
        place_qwerty_keys(&mut key_slots);
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
            id: 0,
            evaluated: false,
            score: 0.0,
            left_uses: 0.0,
            right_uses: 0.0,
            is_elite: false,
            pos_iter: 0,
            last_score: 0.0,
            last_swap_a: (Slot::from_tuple((0, 0)), Key::from_tuple((0, 0))),
            last_swap_b: (Slot::from_tuple((0, 0)), Key::from_tuple((0, 0))),
        };
    }

    pub fn create_dvorak() -> Self {
        let mut key_slots: BTreeMap<Slot, Key> = BTreeMap::new();
        let valid_key_locs_sorted: Vec<(Key, Vec<Slot>)> = get_valid_key_locs_sorted();
        place_dvorak_keys(&mut key_slots);
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
            id: 0,
            evaluated: false,
            score: 0.0,
            left_uses: 0.0,
            right_uses: 0.0,
            is_elite: false,
            pos_iter: 0,
            last_score: 0.0,
            last_swap_a: (Slot::from_tuple((0, 0)), Key::from_tuple((0, 0))),
            last_swap_b: (Slot::from_tuple((0, 0)), Key::from_tuple((0, 0))),
        };
    }

    pub fn from_swap_table(
        rng: &mut SmallRng,
        swap_table: &SwapTable,
        gen_in: usize,
        id_in: usize,
    ) -> Self {
        let valid_key_locs_sorted: Vec<(Key, Vec<Slot>)> = get_valid_key_locs_sorted();
        let mut swappable_keys: Vec<Key> = get_swappable_keys(&SWAPPABLE_KEYS);
        let static_keys = get_static_keys(&swappable_keys, &valid_key_locs_sorted);

        let mut key_slots: BTreeMap<Slot, Key> = BTreeMap::new();
        assert!(place_keys(&mut key_slots, &static_keys, 0), "place_keys");

        let valid_slots: BTreeMap<Key, Vec<Slot>> = valid_key_locs_sorted.into_iter().collect();
        let mut swappable_slots: Vec<Slot> = (1..=3)
            .flat_map(|i| return (0..=9).map(move |j| return Slot::from_tuple((i, j))))
            .collect();

        loop {
            if place_keys_from_table(
                rng,
                &mut swappable_slots,
                &mut swappable_keys,
                swap_table,
                &mut key_slots,
                &valid_slots,
            ) {
                break;
            }
        }

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
            generation: gen_in,
            id: id_in,
            evaluated: false,
            score: 0.0,
            left_uses: 0.0,
            right_uses: 0.0,
            is_elite: false,
            pos_iter: 0,
            last_score: 0.0,
            last_swap_a: (Slot::from_tuple((0, 0)), Key::from_tuple((0, 0))),
            last_swap_b: (Slot::from_tuple((0, 0)), Key::from_tuple((0, 0))),
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
            let row_a = rng.random_range(TOP_ROW..=BOT_ROW);
            let col_a = rng.random_range(L_PINKY..=R_PINKY);
            let slot_a = Slot::from_tuple((row_a, col_a));
            let key_a = self.key_slots[&slot_a];

            if let Some(vec) = self.valid_slots.get_mut(&key_a) {
                vec.shuffle(rng);
                if vec.len() == 1 {
                    continue;
                }
            } else {
                panic!("Valid slots not found for key {:?}", key_a);
            }

            for slot in &self.valid_slots[&key_a] {
                let row_b = slot.get_row();
                let col_b = slot.get_col();
                let slot_b = Slot::from_tuple((row_b, col_b));
                let key_b = self.key_slots[&slot_b];
                if !shuffle_check(&self.valid_slots, slot_a, key_a, slot_b, key_b) {
                    continue;
                }

                self.swap_keys(slot_a, key_a, slot_b, key_b);

                break;
            }

            debug_assert_ne!(
                self.key_slots[&slot_a], key_a,
                "ERROR: Key {:?} at {},{} not changed",
                key_a, row_a, col_a
            );
        }
    }

    // FUTURE: Right now the kb swap functions and the swap map build explicitly exclude anything
    // outside the alpha area. This works until we want to start locking individual keys
    pub fn table_swap(&mut self, rng: &mut SmallRng, swap_table: &SwapTable) {
        self.evaluated = false;
        self.last_score = self.score;

        let mut base_a: Vec<(Slot, Key, f64)> = self
            .key_slots
            .iter()
            .filter(|&(slot, key)| {
                let invalid_location = slot.get_row() < TOP_ROW || slot.get_col() > R_PINKY;
                let static_key = self.valid_slots[key].len() == 1;
                if invalid_location && static_key {
                    return false;
                }

                return true;
            })
            .map(|(slot, key)| {
                let score = swap_table.get_score(slot, key);
                return (*slot, *key, score);
            })
            .collect();

        let select_a = select_key(rng, &mut base_a);
        let select_a_score = swap_table.get_score(&select_a.0, &select_a.1);

        let mut base_b: Vec<(Slot, Key, f64)> = self
            .key_slots
            .iter()
            .filter(|&(slot_b, key_b)| {
                let slot_a = select_a.0;
                let key_a = select_a.1;
                let invalid_slot = slot_b.get_row() < TOP_ROW || slot_b.get_col() > R_PINKY;
                let bad_shuffle_check =
                    !shuffle_check(&self.valid_slots, slot_a, key_a, *slot_b, *key_b);

                if bad_shuffle_check || invalid_slot {
                    return false;
                }

                return true;
            })
            .map(|(slot_b, key_b)| {
                let slot_a = select_a.0;
                let key_a = select_a.1;
                let improvement =
                    get_improvement(swap_table, select_a_score, slot_a, key_a, slot_b, key_b);

                return (*slot_b, *key_b, improvement);
            })
            .collect();

        let select_b = select_key(rng, &mut base_b);
        self.swap_keys(select_a.0, select_a.1, select_b.0, select_b.1);
    }

    fn swap_keys(&mut self, slot_a: Slot, key_a: Key, slot_b: Slot, key_b: Key) {
        self.last_swap_a = (slot_a, key_a);
        self.last_swap_b = (slot_b, key_b);

        self.key_slots.insert(slot_a, key_b);
        self.slot_ascii[usize::from(key_b.get_base())] = Some(slot_a);
        self.slot_ascii[usize::from(key_b.get_shift())] = Some(slot_a);

        self.key_slots.insert(slot_b, key_a);
        self.slot_ascii[usize::from(key_a.get_base())] = Some(slot_b);
        self.slot_ascii[usize::from(key_a.get_shift())] = Some(slot_b);
    }

    // For any slot/key pair in the swap map, a higher weighted average means improvement has been
    // seen when the key leaves the slot. (We can more reliably know which key/slot positions are
    // bad than which ones are good). Therefore, when scoring a swap, the update is made on the
    // key's starting point rather than where it ended up
    // FUTURE: It would be cool if this were called automatically after evaluating. But that would
    // require either passing the swap map into the eval function or making the keyboard store a
    // reference to the swap map, which I think gets into lifetimes. Half of the logic is already
    // done for this though because eval terminates early if there hasn't been a layout change
    // since the last run
    pub fn check_table_swap(&self, swap_table: &mut SwapTable) {
        let last_slot_a = self.last_swap_a.0;
        let last_key_a = self.last_swap_a.1;
        let last_slot_b = self.last_swap_b.0;
        let last_key_b = self.last_swap_b.1;
        let score_diff = self.score - self.last_score;

        swap_table.update_score(last_slot_a, last_key_a, score_diff);
        swap_table.update_score(last_slot_b, last_key_b, score_diff);
    }

    // NOTE: A single major efficiency penalty at any point in the algorithm can cause the entire
    // layout to change. Be careful over-indexing for any particular factor
    fn get_efficiency(&mut self, this_slot: Slot) -> f64 {
        let mut eff = BASE_EFF;

        let this_hand = Hand::from_slot(this_slot);
        if this_hand == Hand::Right {
            self.right_uses += 1.0_f64;
        } else {
            self.left_uses += 1.0_f64;
        }

        eff *= global_adjustments(this_slot);

        let last_compare: Option<KeyCompare> = self
            .last_slot_idx
            .map(|last_slot| return compare_slots(this_slot, last_slot, true));
        if let Some(key_compare) = last_compare {
            match key_compare {
                KeyCompare::Mult(x) => return eff * x,
                KeyCompare::Mismatch => {}
            }
        }

        let prev_compare: Option<KeyCompare> = self
            .prev_slot_idx
            .map(|prev_slot| return compare_slots(this_slot, prev_slot, false));
        if let Some(key_compare) = prev_compare {
            match key_compare {
                KeyCompare::Mult(x) => return eff * x,
                KeyCompare::Mismatch => {}
            }
        }

        eff *= check_key_no_hist(this_slot);

        return eff;
    }

    pub fn eval(&mut self) {
        if self.evaluated {
            return;
        }

        let corpus = get_corpus();
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

    // FUTURE: Very inefficient
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

        let display_chars: Vec<Vec<char>> = vec![num_row, top_row, home_row, bot_row];
        return display_chars;
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
