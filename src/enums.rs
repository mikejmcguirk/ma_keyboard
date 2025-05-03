use strum::EnumIter;

// TODO: Want to eventually include number keys, symbol keys, and tab/return
// But keeping at basic keys for the sake of just getting to a minimum viable start

#[derive(EnumIter, Copy, Clone, Debug, Eq, PartialEq)]
pub enum Row {
    // Number,
    Above,
    Home,
    Below,
}

// TODO: Do we need to add number returns for the fingers? Use piano numbering. Thumb is 1
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Finger {
    // Thumb,
    Index,
    Middle,
    Ring,
    Pinky,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Hand {
    Left,
    Right,
}

#[derive(EnumIter, Copy, Clone, Debug, Eq, PartialEq)]
pub enum Col {
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    // Eleven,
    // Twelve,
}

impl Col {
    pub fn get_finger(self) -> Finger {
        return match self {
            Col::One | Col::Ten => Finger::Pinky,
            Col::Two | Col::Nine => Finger::Ring,
            Col::Three | Col::Eight => Finger::Middle,
            Col::Four | Col::Seven | Col::Five | Col::Six => Finger::Index,
        };
    }

    pub fn get_hand(self) -> Hand {
        return match self {
            Col::One | Col::Two | Col::Three | Col::Four | Col::Five => Hand::Left,
            Col::Six | Col::Seven | Col::Eight | Col::Nine | Col::Ten => Hand::Right,
        };
    }

    pub fn get_center_dist(self) -> u8 {
        return match self {
            Col::One | Col::Ten => 4,
            Col::Two | Col::Nine => 3,
            Col::Three | Col::Eight => 2,
            Col::Four | Col::Seven => 1,
            Col::Five | Col::Six => 0,
        };
    }
}
