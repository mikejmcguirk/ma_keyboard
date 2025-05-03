use std::collections::HashMap;

use {
    anyhow::{Result, anyhow},
    rand::{Rng as _, SeedableRng as _, rngs::SmallRng},
    strum::IntoEnumIterator as _,
};

use crate::{
    constants::KEY_TUPLES,
    enums::{Col, Finger, Hand, Row},
    kb_components::{Key, KeySlot},
};

#[derive(Clone)]
pub struct Keyboard {
    keyslots: Vec<KeySlot>,
    // Between the base and shift layer, there are enough possible keypresses to justify this
    slot_ref: HashMap<char, usize>,
    last_slot: Option<KeySlot>,
}

impl Keyboard {
    // TODO: I'm not actually sure if new() needs to be run more than once. Even for creating the
    // initial keyboards, it's probably faster to just run clone(). And then for subsequent
    // keyboards, we know we are cloning pre-existing ones. So I think any performance optimization
    // here at the expense of safety and clarity might be an unnecessary flex
    // PERF: This can be optimized by pre-allocating keyslots and unsafely writing to it

    // TODO: At some point this logic will need to handle keys that are not totally randomized. As
    // much of this logic as possible should be tied to the enums. The key though is it needs to
    // flow intuitively. Right now, col.get_finger() intuitively makes sense because we know each
    // keyboard column has a finger mapped to it. You don't really need to jump to definition to
    // understand it
    pub fn new() -> Result<Self> {
        let mut keyslots: Vec<KeySlot> = Vec::new();
        let mut kt_idx: usize = 0;

        // TODO: Do some kind of checked index access for key_tuple_idx
        // TODO: Add a check or debug_assert that key_tuple_idx matches the len of
        // KEY_TUPLES. We need the number of keys to match exactly the amount of slots to fill
        // Add documentation for this behavior as well, since it corrolates a couple different
        // pieces of code
        for row in Row::iter() {
            for col in Col::iter() {
                let Some(key_tuple): Option<&(char, char)> = KEY_TUPLES.get(kt_idx) else {
                    // TODO: Terrible error message
                    return Err(anyhow!("Bad read from KEY_TUPLES"));
                };

                kt_idx += 1;
                let key: Key = Key::new(*key_tuple);
                let hand: Hand = col.get_hand();
                let finger: Finger = col.get_finger();

                let slot: KeySlot = KeySlot::new(key, row, col, hand, finger);
                keyslots.push(slot);
            }
        }

        let mut slot_ref: HashMap<char, usize> = HashMap::new();
        for i in 0..keyslots.len() {
            let key: Key = keyslots[i].get_key();

            slot_ref.insert(key.get_base(), i);
            slot_ref.insert(key.get_shift(), i);
        }

        return Ok(Keyboard {
            keyslots,
            slot_ref,
            last_slot: None,
        });
    }

    pub fn shuffle_all(&mut self) {
        // TODO: This should be attached to the keyboard
        let seed: [u8; 32] = rand::random();
        let mut rng = SmallRng::from_seed(seed);

        for i in 0..self.keyslots.len() {
            let j: usize = rng.random_range(i..self.keyslots.len());

            let key_i: Key = self.keyslots[i].get_key();
            let key_j: Key = self.keyslots[j].get_key();

            // TODO: This needs to be made into some kind of grouped logic
            self.keyslots[i].set_key(key_j);
            self.slot_ref.insert(key_j.get_base(), i);
            self.slot_ref.insert(key_j.get_shift(), i);

            // TODO: This needs to be made into some kind of grouped logic
            self.keyslots[j].set_key(key_i);
            self.slot_ref.insert(key_i.get_base(), j);
            self.slot_ref.insert(key_i.get_shift(), j);
        }
    }

    pub fn shuffle_some(&mut self, amt: usize) -> Result<()> {
        if amt > self.keyslots.len() {
            return Err(anyhow!("Amount is greater than valid keys"));
        }

        // TODO: Like before, should be attached to the struct
        let seed: [u8; 32] = rand::random();
        let mut rng = SmallRng::from_seed(seed);

        let mut already_shuffled: Vec<usize> = Vec::new();

        for _ in 0..amt {
            let i: usize = loop {
                let idx = rng.random_range(0..self.keyslots.len());
                if !already_shuffled.contains(&idx) {
                    already_shuffled.push(idx);
                    break idx;
                }
            };

            let j: usize = loop {
                let idx = rng.random_range(0..self.keyslots.len());
                if !already_shuffled.contains(&idx) {
                    already_shuffled.push(idx);
                    break idx;
                }
            };

            let key_i: Key = self.keyslots[i].get_key();
            let key_j: Key = self.keyslots[j].get_key();

            // TODO: This needs to be made into some kind of grouped logic
            self.keyslots[i].set_key(key_j);
            self.slot_ref.insert(key_j.get_base(), i);
            self.slot_ref.insert(key_j.get_shift(), i);

            // TODO: This needs to be made into some kind of grouped logic
            self.keyslots[j].set_key(key_i);
            self.slot_ref.insert(key_i.get_base(), j);
            self.slot_ref.insert(key_i.get_shift(), j);
        }

        return Ok(());
    }

    pub fn clear_last_slot(&mut self) {
        self.last_slot = None;
    }

    pub fn get_efficiency(&mut self, input: char) -> f64 {
        const DEFAULT_EFFICIENCY: f64 = 1.0;

        let slot_idx: &usize = match self.slot_ref.get(&input) {
            Some(slot) => slot,
            None => return 0.0, // Not a valid key, don't affect score
        };

        let mut efficiency: f64 = DEFAULT_EFFICIENCY;

        let row: Row = self.keyslots[*slot_idx].get_row();

        // I agree with Dvorak. The top row is easier to hit than the bottom
        if row == Row::Above {
            efficiency *= 1.08;
        }
        if row == Row::Below {
            efficiency *= 1.16;
        }

        // Penalize index finger extensions
        let col: Col = self.keyslots[*slot_idx].get_col();
        if col == Col::Five || col == Col::Six {
            efficiency *= 1.1;
        }

        // Because the keyboard columns slope down-right, this goes against the grain of the left
        // hand, so we penalize it here. But, only slightly because left-handed typists are out
        // there and on account of personal preference
        if col == Col::One
            || col == Col::Two
            || col == Col::Three
            || col == Col::Four
            || col == Col::Five
        {
            efficiency *= 1.05;
        }

        // The ring and pinky fingers are penalized evenly due to variance in personal preference.
        // Neither the index nor middle finger are preferenced for the same reason
        let finger: Finger = self.keyslots[*slot_idx].get_finger();
        if finger == Finger::Ring || finger == Finger::Pinky {
            efficiency *= 1.1;
        }

        if let Some(last_slot) = self.last_slot {
            let last_row: Row = last_slot.get_row();

            // Two row skip
            let hand: Hand = self.keyslots[*slot_idx].get_hand();
            let last_hand: Hand = last_slot.get_hand();

            if hand == last_hand
                && ((last_row == Row::Above && row == Row::Below)
                    || (last_row == Row::Below && row == Row::Above))
            {
                efficiency *= 1.25;
            }

            let last_finger: Finger = last_slot.get_finger();
            // Slow, causes pain
            if finger == last_finger && hand == last_hand {
                efficiency *= 1.15;
            }

            let last_col: Col = last_slot.get_col();

            // Needs refining
            if hand == last_hand && col.get_center_dist() <= last_col.get_center_dist() {
                efficiency *= 1.15;
            }
        }

        self.last_slot = Some(self.keyslots[*slot_idx]);

        return efficiency;
    }

    pub fn display_keyboard(&self) {
        let mut main_vec: Vec<Vec<char>> = Vec::new();
        let mut above_vec: Vec<char> = Vec::new();
        let mut home_vec: Vec<char> = Vec::new();
        let mut below_vec: Vec<char> = Vec::new();

        for slot in &self.keyslots {
            let this_row = slot.get_row();
            let this_key = slot.get_key().get_base();

            if this_row == Row::Above {
                above_vec.push(this_key);
            } else if this_row == Row::Home {
                home_vec.push(this_key);
            } else if this_row == Row::Below {
                below_vec.push(this_key);
            }
        }

        println!("{:?}", above_vec);
        println!("{:?}", home_vec);
        println!("{:?}", below_vec);
    }
}
