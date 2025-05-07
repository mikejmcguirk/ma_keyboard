use crate::enums::{Col, Finger, Hand, KeySetError, KeyTemplate, ListType, Row};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Key {
    base: u8,
    shift: u8,
}

// NOTE: In order to ensure keys are accurate throughout the lifetime of the program, only create
// new from the template enum and keep the fields private
impl Key {
    pub fn from_template(key_template: KeyTemplate) -> Key {
        return Key {
            base: key_template.get_base(),
            shift: key_template.get_shift(),
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
pub struct LocInfo {
    row: Row,
    col: Col,
}

impl LocInfo {
    pub fn from_row_col(row: Row, col: Col) -> Self {
        return Self { row, col };
    }

    pub fn get_row(self) -> Row {
        return self.row;
    }

    pub fn get_col(self) -> Col {
        return self.col;
    }
}

#[derive(Debug, Clone, Copy)]
pub struct HandInfo {
    hand: Hand,
    finger: Finger,
}

impl HandInfo {
    pub fn from_hand_finger(hand: Hand, finger: Finger) -> Self {
        return Self { hand, finger };
    }

    pub fn get_hand(self) -> Hand {
        return self.hand;
    }

    pub fn get_finger(self) -> Finger {
        return self.finger;
    }
}

#[derive(Debug, Clone)]
pub struct KeyList {
    list: Vec<Key>,
    list_type: ListType,
}

impl KeyList {
    // pub fn new() -> Self {
    //     return Self {
    //         list: Vec::new(),
    //         list_type: ListType::Deny,
    //     };
    // }

    pub fn from_vec(list: Vec<Key>, list_type: ListType) -> Self {
        return Self { list, list_type };
    }

    pub fn contains_one_valid_key(&self) -> bool {
        return self.list_type == ListType::Allow && self.list.len() == 1;
    }

    pub fn is_valid_key(&self, key: Key) -> bool {
        let found: bool = self.list.contains(&key);

        if self.list_type == ListType::Deny && found {
            return false;
        }

        if self.list_type == ListType::Allow && !found {
            return false;
        }

        return true;
    }
}

// NOTE: You could derive the hand and finger programmatically from the row and column, but this
// costs more execution time than a simple memory read
// NOTE: This needs a global lock bool for number keys. And then, to see if it's locked, you can
// incorporate that into a search of the keylist
#[derive(Debug, Clone)]
pub struct KeySlot {
    key: Key,
    loc_info: LocInfo,
    hand_info: HandInfo,
    key_list: KeyList,
    // TODO: Need to add disallowed keys
}

// TODO: Similar to the above, the KeySlot must assume that the key data is valid. We can check
// that the key should not be in this slot for a reason like "we don't want a symbol key on a
// home row index finger", but we have to assume that the key itself is something that should
// exist. How to deal with this gets back to what is in the comment above Key
// TODO: Long signature. The solution here is probably to make row/col a struct, hand/finger a
// struct, and list/allow into a struct, so similar things are grouped together.
// get_hand().get_hand() will be kinda silly, but ah well
impl KeySlot {
    pub fn new(key: Key, loc_info: LocInfo, hand_info: HandInfo, key_list: KeyList) -> Self {
        return Self {
            key,
            loc_info,
            hand_info,
            key_list,
        };
    }

    // pub fn get_key(&self) -> Key {
    //     return self.key;
    // }
    //
    // pub fn set_key(&mut self, key: Key) {
    //     self.key = key;
    // }

    pub fn get_loc_info(&self) -> LocInfo {
        return self.loc_info;
    }

    pub fn get_hand_info(&self) -> HandInfo {
        return self.hand_info;
    }

    pub fn get_key(&self) -> Key {
        return self.key;
    }

    // TODO: Panic/assert
    pub fn set_key(&mut self, key: Key) -> Result<(), KeySetError> {
        if self.key_list.contains_one_valid_key() {
            assert!(self.key_list.is_valid_key(self.key), "assertion failed");

            return Err(KeySetError::SingleKeySlot);
        }

        if !self.key_list.is_valid_key(key) {
            return Err(KeySetError::InvalidKey);
        }

        self.key = key;

        return Ok(());
    }
}
