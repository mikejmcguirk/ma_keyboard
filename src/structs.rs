use crate::{
    kb_builders::check_col,
    keyboard::{ASCII_CNT, BOT_ROW, NUM_ROW},
};

pub struct IdSpawner {
    next_id: usize,
}

impl IdSpawner {
    pub fn new() -> Self {
        return Self { next_id: 0 };
    }

    // PERF: I want to return 0 as the first id but maybe this is an extravagance
    pub fn get(&mut self) -> usize {
        let to_return: usize = self.next_id;
        self.next_id += 1;

        return to_return;
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
