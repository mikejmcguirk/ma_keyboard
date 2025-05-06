use anyhow::Result;

use crate::enums::{Col, Finger, Hand, Row};

// In a separate module from the Keyboard struct so private fields are fully private

#[derive(Debug, Clone, Copy)]
pub struct Key {
    base: u8,
    shift: u8,
}

// TODO: It is not invalid, from the standpoint of the key, if you give it a tuple like ('a', 'a').
// The key struct, correctly, does not allow the key to be modified, but the source of the initial
// keys needs to be controlled some other way. The key itself should not be an enum, because using
// match statements on those values is cumbersome, but maybe that's how the sources should be
// constructed. Feeding a struct here is also a possibility. Because this struct is used for
// creating new keyboards, it creates illogical error returns on functions that do not involve
// runtime input if you check for errors here
impl Key {
    pub fn new(key_tuple: (u8, u8)) -> Key {
        return Key {
            base: key_tuple.0,
            shift: key_tuple.1,
        };
    }

    pub fn get_base(self) -> u8 {
        return self.base;
    }

    pub fn get_shift(self) -> u8 {
        return self.shift;
    }
}

#[derive(Debug, Clone, Copy)]
pub struct KeySlot {
    key: Key,
    row: Row,
    col: Col,
    hand: Hand,
    finger: Finger,
    // TODO: Need to add disallowed keys
}

// TODO: Similar to the above, the KeySlot must assume that the key data is valid. We can check
// that the key should not be in this slot for a reason like "we don't want a symbol key on a
// home row index finger", but we have to assume that the key itself is something that should
// exist. How to deal with this gets back to what is in the comment above Key
impl KeySlot {
    pub fn new(key: Key, row: Row, col: Col, hand: Hand, finger: Finger) -> Self {
        return Self {
            key,
            row,
            col,
            hand,
            finger,
        };
    }

    pub fn get_key(self) -> Key {
        return self.key;
    }

    pub fn set_key(&mut self, key: Key) -> Result<()> {
        self.key = key;
        return Ok(());
    }

    pub fn get_row(self) -> Row {
        return self.row;
    }

    pub fn get_col(self) -> Col {
        return self.col;
    }

    pub fn get_hand(self) -> Hand {
        return self.hand;
    }

    pub fn get_finger(self) -> Finger {
        return self.finger;
    }
}
