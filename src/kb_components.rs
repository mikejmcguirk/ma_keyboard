use rand::{Rng, rngs::SmallRng};

use crate::{
    custom_err::KeySetError,
    enums::{Col, Finger, Hand, ListType, Row},
    key_template::KeyTemplate,
};

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
// TODO: long function signature
impl KeySlot {
    pub fn from_components(
        key: Key,
        loc_info: LocInfo,
        hand_info: HandInfo,
        key_list: KeyList,
    ) -> Self {
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

    pub fn get_row(&self) -> Row {
        return self.loc_info.get_row();
    }

    pub fn get_col(&self) -> Col {
        return self.loc_info.get_col();
    }

    pub fn get_hand(&self) -> Hand {
        return self.hand_info.get_hand();
    }

    pub fn get_finger(&self) -> Finger {
        return self.hand_info.get_finger();
    }

    pub fn get_key(&self) -> Key {
        return self.key;
    }

    pub fn check_key(&self, key: Key) -> Result<(), KeySetError> {
        if self.key_list.is_valid_key(key) {
            return Ok(());
        }

        if !self.key_list.is_valid_key(self.key) {
            return Err(KeySetError::HasInvalid);
        }

        if self.key_list.contains_one_valid_key() {
            return Err(KeySetError::HasOnlyValid);
        }

        return Err(KeySetError::InvalidInput);
    }

    pub fn set_key(&mut self, key: Key) -> Result<(), KeySetError> {
        if let Err(e) = self.check_key(key) {
            return Err(e);
        }

        self.key = key;
        return Ok(());
    }
}

#[derive(Clone)]
pub struct UpdatedKey {
    base: u8,
    shift: u8,
    current_location: (usize, usize),
    valid_locations: Vec<(usize, usize)>,
    is_static: bool,
}

impl UpdatedKey {
    pub fn from_template(template: KeyTemplate) -> Self {
        let base: u8 = template.get_base();
        let shift: u8 = template.get_shift();
        let current_location: (usize, usize) = template.get_starting_location();
        let valid_locations: Vec<(usize, usize)> = template.get_valid_locations();

        let is_static = if valid_locations.len() == 1 {
            if current_location != valid_locations[0] {
                panic!("Template has mismatch between current and valid location for static key");
            }
            true
        } else {
            false
        };

        if valid_locations.len() == 0 {
            panic!("Template provided no valid locations");
        }

        return Self {
            base,
            shift,
            current_location,
            valid_locations,
            is_static,
        };
    }

    pub fn get_base(&self) -> u8 {
        return self.base;
    }

    pub fn get_shift(&self) -> u8 {
        return self.shift;
    }

    pub fn is_static(&self) -> bool {
        return self.is_static;
    }

    pub fn get_cnt_valid_locations(&self) -> usize {
        return self.valid_locations.len();
    }

    pub fn get_current_location(&self) -> (usize, usize) {
        return self.current_location;
    }

    pub fn get_valid_locations(&self) -> &[(usize, usize)] {
        return &self.valid_locations;
    }

    pub fn shuffle_valid_locations(&mut self, rng: &mut SmallRng) {
        for i in 0..self.valid_locations.len() - 1 {
            let j = rng.random_range((i + 1)..self.valid_locations.len());

            self.valid_locations.swap(i, j);
        }
    }

    pub fn set_cur_location(&mut self, new_location: (usize, usize)) {
        self.current_location = new_location;
    }

    pub fn get_valid_location_at_idx(&self, idx: usize) -> (usize, usize) {
        return self.valid_locations[idx];
    }
}

pub struct SimpleKeySlot {
    key: Key,
    hand: Hand,
    finger: Finger,
}
