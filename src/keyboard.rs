extern crate alloc;

use {alloc::collections::BTreeMap, std::collections::HashMap};

use rand::{Rng as _, rngs::SmallRng, seq::SliceRandom as _};

use crate::{
    kb_consts, kb_helper_consts,
    kb_helpers::{
        check_col, check_key_no_hist, compare_slots, get_valid_key_locs_sorted,
        global_adjustments, place_keys,
    },
    population::SwapScore,
};

kb_consts!();

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
            last_score: kb.last_score,
            last_swap_a: kb.last_swap_a,
            last_swap_b: kb.last_swap_b,
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

    // TODO: Right now the kb swap functions and the swap map build explicitly exclude anything
    // outside the alpha area. This works until we want to start locking individual keys
    // TODO: This function does not check valid key locations. Like above, this only works for now
    pub fn mapped_swap(&mut self, rng: &mut SmallRng, swap_map: &HashMap<(Slot, Key), SwapScore>) {
        self.evaluated = false;
        self.last_score = self.score;

        let mut min_out = f64::MAX;
        let mut max_out = f64::MIN;
        let out_raw: Vec<(Slot, Key, f64)> = self
            .key_slots
            .iter()
            .filter(|(slot, _)| return slot.get_row() >= 1 && slot.get_col() <= 9)
            .map(|(slot, key)| {
                let score = swap_map[&(*slot, *key)].get_w_avg();
                min_out = min_out.min(score);
                max_out = max_out.max(score);

                return (*slot, *key, score);
            })
            .collect();

        debug_assert!(!out_raw.is_empty(), "Should always be candidates out");

        let mut total_normalized = 0.0;
        let out_normalized: Vec<(Slot, Key, f64)> = out_raw
            .iter()
            .map(|c| {
                if max_out > min_out {
                    let raw_score = c.2;
                    let normalized_score = (raw_score - min_out) / (max_out - min_out);
                    total_normalized += normalized_score;

                    return (c.0, c.1, normalized_score);
                } else {
                    return (c.0, c.1, 0.0);
                }
            })
            .collect();

        let range_out = max_out - min_out;
        let mean_out = out_raw.iter().map(|c| return c.2).sum::<f64>() / out_raw.len() as f64;
        let variance_out = out_raw
            .iter()
            .map(|c| return (c.2 - mean_out).powi(2))
            .sum::<f64>()
            / out_raw.len() as f64;
        // .25 is highly spread out
        let norm_variance_out = if range_out > 0.0 {
            variance_out / (range_out * range_out)
        } else {
            0.0
        };

        // Low temperature, but looking at the Softmax probability distributions, this is what it
        // seems to take to get it to not chose keys essentially at random
        // let temp_out = (norm_variance_out * 2.0).max(0.01);
        // let temp_out = (norm_variance_out * 4.0).max(0.01);
        // let temp_out = 1.0 - (3.96 * norm_variance_out);
        // let temp_out = 0.5 - (1.96 * norm_variance_out);
        let temp_out = 0.26 - norm_variance_out;

        let mut total_exp_out = 0.0;
        let exp_out_candidates: Vec<(Slot, Key, f64)> = out_normalized
            .iter()
            .map(|c| {
                let this_base_score = c.2;
                let this_exp_score = (this_base_score / temp_out).exp();
                total_exp_out += this_exp_score;

                return (c.0, c.1, this_exp_score);
            })
            .collect();

        let exp_normalized_out: Vec<(Slot, Key, f64)> = exp_out_candidates
            .into_iter()
            .map(|(slot, key, exp_score)| return (slot, key, exp_score / total_exp_out))
            .collect();

        let mut checked_score_out: f64 = 0.0;
        let r_out = rng.random_range(0.0..=1.0);
        // println!("r_out {r_out}");
        let out_selection: (Slot, Key, f64) = {
            *exp_normalized_out
                .iter()
                .find(|c| {
                    checked_score_out += c.2;
                    // println!("checked score out {checked_score_out}, r_out {r_out}");
                    return checked_score_out > r_out;
                })
                .unwrap_or_else(|| {
                    return exp_normalized_out
                        .last()
                        .expect("Normalized cancidates should not be empty");
                })
        };

        let mut min_in = f64::MAX;
        let mut max_in = f64::MIN;
        let in_raw: Vec<(Slot, Key, f64)> = self
            .key_slots
            .iter()
            .filter(|(slot_in, key_in)| {
                let key_out = out_selection.1;
                return (slot_in.get_row() >= 1 && slot_in.get_col() <= 9) && **key_in != key_out;
            })
            .map(|(slot_in, key_in)| {
                let slot_out = out_selection.0;
                let key_out = out_selection.1;
                let score_out = swap_map[&(*slot_in, key_out)].get_w_avg();
                let score_in = swap_map[&(slot_out, *key_in)].get_w_avg();
                let total_score = score_out + score_in;

                min_in = min_in.min(total_score);
                max_in = max_in.max(total_score);

                return (*slot_in, *key_in, total_score);
            })
            .collect();

        let in_normalized: Vec<(Slot, Key, f64)> = in_raw
            .iter()
            .map(|c| {
                if max_in > min_in {
                    let raw_score = c.2;
                    let normalized_score = (raw_score - min_in) / (max_in - min_in);
                    // total_normalized += normalized_score;

                    return (c.0, c.1, normalized_score);
                } else {
                    return (c.0, c.1, 0.0);
                }
            })
            .collect();

        let range_in = max_in - min_in;
        let mean_in = in_raw.iter().map(|c| return c.2).sum::<f64>() / in_raw.len() as f64;
        let variance_in = in_raw
            .iter()
            .map(|c| return (c.2 - mean_in).powi(2))
            .sum::<f64>()
            / in_raw.len() as f64;
        // .25 is highly spread out
        let norm_variance_in = if range_in > 0.0 {
            variance_in / (range_in * range_in)
        } else {
            0.0
        };
        let temp_in = 0.26 - norm_variance_in;
        // let temp_in = (norm_variance_in * 2.0).max(0.01);
        // let temp_in = (norm_variance_in * 4.0).max(0.01);
        // let temp_in = 1.0 - (3.96 * norm_variance_in);
        // let temp_in = 0.5 - (1.96 * norm_variance_in);
        // println!(
        //     "norm variance {}, temp {}, out slot {:?}",
        //     norm_variance_in, temp_in, out_selection.0
        // );

        let mut total_exp_in = 0.0;
        let exp_in_candidates: Vec<(Slot, Key, f64)> = in_normalized
            .iter()
            .map(|c| {
                let this_base_score = c.2;
                let this_exp_score = (this_base_score / temp_in).exp();
                // println!("base score {this_base_score}");
                // println!("temp {temp}");
                // println!("base / temp {}", (this_base_score / temp));
                // println!("this exp {this_exp_score}");
                total_exp_in += this_exp_score;

                return (c.0, c.1, this_exp_score);
            })
            .collect();

        // println!("total exp in {total_exp_in}");
        let exp_normalized_in: Vec<(Slot, Key, f64)> = exp_in_candidates
            .into_iter()
            .map(|(slot, key, exp_score)| return (slot, key, exp_score / total_exp_in))
            .collect();
        // println!("{:?}", exp_normalized_in);

        let mut checked_score_in: f64 = 0.0;
        let r_in = rng.random_range(0.0..=1.0);
        let in_selection: (Slot, Key, f64) = {
            *exp_normalized_in
                .iter()
                .find(|c| {
                    checked_score_in += c.2;
                    return checked_score_out > r_in;
                })
                .unwrap_or_else(|| {
                    return exp_normalized_in
                        .last()
                        .expect("Normalized cancidates should not be empty");
                })
        };

        let out_slot = out_selection.0;
        let out_key = out_selection.1;
        let in_slot = in_selection.0;
        let in_key = in_selection.1;

        self.last_swap_a = (out_slot, out_key);
        self.last_swap_b = (in_slot, in_key);

        self.key_slots.insert(out_slot, in_key);
        self.slot_ascii[usize::from(in_key.get_base())] = Some(out_slot);
        self.slot_ascii[usize::from(in_key.get_shift())] = Some(out_slot);

        self.key_slots.insert(in_slot, out_key);
        self.slot_ascii[usize::from(out_key.get_base())] = Some(in_slot);
        self.slot_ascii[usize::from(out_key.get_shift())] = Some(in_slot);
    }

    pub fn check_swap(&self, swap_map: &mut HashMap<(Slot, Key), SwapScore>, iter: usize) {
        let last_slot_a = self.last_swap_a.0;
        let last_key_a = self.last_swap_a.1;
        let last_slot_b = self.last_swap_b.0;
        let last_key_b = self.last_swap_b.1;

        let mut score_a = swap_map[&(last_slot_a, last_key_a)];
        let mut score_b = swap_map[&(last_slot_b, last_key_b)];

        let score_diff = self.score - self.last_score;
        score_a.reweight_avg(score_diff, iter as f64);
        score_b.reweight_avg(score_diff, iter as f64);

        swap_map.insert((self.last_swap_a.0, self.last_swap_a.1), score_a);
        swap_map.insert((self.last_swap_b.0, self.last_swap_b.1), score_b);

        // if iter % 100 == 0 {
        //     println!("{:#?}", swap_map);
        // }
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
