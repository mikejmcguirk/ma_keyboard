use crate::{
    constants::KEY_TUPLES,
    enums::{Col, Finger, Row},
};

// In a separate module from the Keyboard struct so private fields are fully private

#[derive(Debug)]
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

    pub fn get_base(&self) -> char {
        return self.base;
    }

    pub fn get_shift(&self) -> char {
        return self.shift;
    }
}

#[derive(Debug)]
pub struct KeySlot {
    key: Key,
    row: Row,
    col: Col,
    finger: Finger,
    // TODO: Need to add disallowed keys
}

impl KeySlot {
    pub fn new(key: Key, row: Row, col: Col, finger: Finger) -> Self {
        return KeySlot {
            key,
            row,
            col,
            finger,
        };
    }

    pub fn get_key(&self) -> &Key {
        return &self.key;
    }

    pub fn set_key(&mut self, key: Key) {
        self.key = key;
    }
}
