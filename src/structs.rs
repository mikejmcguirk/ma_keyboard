use std::collections::HashMap;

use {
    anyhow::{Result, anyhow},
    strum::IntoEnumIterator as _,
};

use crate::{
    constants::KEY_TUPLES,
    enums::{Col, Finger, Row},
    kb_components::{Key, KeySlot},
};

pub struct Keyboard {
    keyslots: Vec<KeySlot>,
    // Between the base and shift layer, there are enough possible keypresses to justify this
    slot_ref: HashMap<char, usize>,
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
                let finger: Finger = col.get_finger();

                let slot: KeySlot = KeySlot::new(key, row, col, finger);
                keyslots.push(slot);
            }
        }

        let mut slot_ref: HashMap<char, usize> = HashMap::new();
        for i in 0..keyslots.len() {
            let key: &Key = keyslots[i].get_key();

            slot_ref.insert(key.get_base(), i);
            slot_ref.insert(key.get_shift(), i);
        }

        return Ok(Keyboard { keyslots, slot_ref });
    }

    pub fn print_keyslots(&self) {
        println!("{:?}", self.keyslots);
    }
}
