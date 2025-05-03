use crate::{
    constants::KEY_TUPLES,
    enums::{Col, Finger, Hand, Row},
};

// In a separate module from the Keyboard struct so private fields are fully private

#[derive(Debug, Clone, Copy)]
pub struct Key {
    base: char,
    shift: char,
}

impl Key {
    pub fn new(key_tuple: (char, char)) -> Self {
        // TODO: I think right now I have debug_assertions on for profiling, which in this case
        // would actually cause a significant performance hit
        // On the other hand, if we only expect the original make keyboard method to be run once,
        // then just having this check in production is not an issue, because this should not run
        // in a hot loop. So better to code defensively
        debug_assert!(KEY_TUPLES.contains(&key_tuple));

        return Key {
            base: key_tuple.0,
            shift: key_tuple.1,
        };
    }

    pub fn get_base(self) -> char {
        return self.base;
    }

    pub fn get_shift(self) -> char {
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

impl KeySlot {
    pub fn new(key: Key, row: Row, col: Col, hand: Hand, finger: Finger) -> Self {
        return KeySlot {
            key,
            row,
            col,
            hand,
            finger,
        };
    }

    pub fn get_key(&self) -> Key {
        return self.key;
    }

    pub fn set_key(&mut self, key: Key) {
        self.key = key;
    }

    pub fn get_row(&self) -> Row {
        return self.row;
    }

    pub fn get_col(&self) -> Col {
        return self.col;
    }

    pub fn get_hand(&self) -> Hand {
        return self.hand;
    }

    pub fn get_finger(&self) -> Finger {
        return self.finger;
    }
}
