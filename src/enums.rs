use strum::EnumIter;

// TODO: Want to eventually include number keys, symbol keys, and tab/return
// But keeping at basic keys for the sake of just getting to a minimum viable start

#[derive(EnumIter, Copy, Clone, Debug)]
pub enum Row {
    // Number,
    Above,
    Home,
    Below,
}

// TODO: Do we need to add number returns for the fingers? Use piano numbering. Thumb is 1
#[derive(Debug)]
pub enum Finger {
    // Thumb,
    Index,
    Middle,
    Ring,
    Pinky,
}

#[derive(EnumIter, Copy, Clone, Debug)]
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
}
