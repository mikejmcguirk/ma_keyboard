use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io;

use strum::EnumIter;

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

impl Finger {
    pub fn get_num(self) -> u8 {
        return match self {
            Finger::Index => 2,
            Finger::Middle => 3,
            Finger::Ring => 4,
            Finger::Pinky => 5,
        };
    }
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

    // pub fn get_center_dist(self) -> u8 {
    //     return match self {
    //         Col::One | Col::Ten => 4,
    //         Col::Two | Col::Nine => 3,
    //         Col::Three | Col::Eight => 2,
    //         Col::Four | Col::Seven => 1,
    //         Col::Five | Col::Six => 0,
    //     };
    // }
}

#[derive(Debug)]
pub enum MyError {
    EmptyCorpus,
    Io(io::Error),
}

impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MyError::EmptyCorpus => return write!(f, "No files in corpus"),
            MyError::Io(e) => return write!(f, "IO error: {}", e,),
        }
    }
}

impl Error for MyError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            MyError::EmptyCorpus => return None,
            MyError::Io(e) => return Some(e),
        }
    }
}

impl From<io::Error> for MyError {
    fn from(error: io::Error) -> Self {
        return MyError::Io(error);
    }
}
