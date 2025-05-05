use anyhow::{Result, anyhow};

use crate::{
    constants::KEY_TUPLES,
    enums::{Col, Finger, Hand, Row},
};

// In a separate module from the Keyboard struct so private fields are fully private

#[derive(Debug, Clone, Copy)]
pub struct Key {
    base: u8,
    shift: u8,
}

impl Key {
    pub fn new(key_tuple: (u8, u8)) -> Result<Self> {
        if !KEY_TUPLES.contains(&key_tuple) {
            return Err(anyhow!("Key not contained in KEY_TUPLES"));
        }

        return Ok(Key {
            base: key_tuple.0,
            shift: key_tuple.1,
        });
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

impl KeySlot {
    pub fn new(key: Key, row: Row, col: Col, hand: Hand, finger: Finger) -> Result<Self> {
        // PERF: If this is slow, turn into a debug assertion
        let key_check: (u8, u8) = (key.get_base(), key.get_shift());
        if !KEY_TUPLES.contains(&key_check) {
            return Err(anyhow!("Invalid key"));
        }

        return Ok(KeySlot {
            key,
            row,
            col,
            hand,
            finger,
        });
    }

    pub fn get_key(&self) -> Key {
        return self.key;
    }

    pub fn set_key(&mut self, key: Key) -> Result<()> {
        // PERF: If this is slow, turn into a debug assertion
        let key_check: (u8, u8) = (key.get_base(), key.get_shift());
        if !KEY_TUPLES.contains(&key_check) {
            return Err(anyhow!("Invalid key"));
        }

        self.key = key;

        return Ok(());
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
