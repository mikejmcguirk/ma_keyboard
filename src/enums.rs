#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Row {
    Num,
    Above,
    Home,
    Below,
}

impl Row {
    pub fn get_num(self) -> u8 {
        return match self {
            Row::Num => 4,
            Row::Above => 3,
            Row::Home => 2,
            Row::Below => 1,
        };
    }
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

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
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
    Eleven,
    Twelve,
}

impl Col {
    pub fn get_finger(self) -> Finger {
        return match self {
            Col::One | Col::Ten | Col::Eleven | Col::Twelve => Finger::Pinky,
            Col::Two | Col::Nine => Finger::Ring,
            Col::Three | Col::Eight => Finger::Middle,
            Col::Four | Col::Seven | Col::Five | Col::Six => Finger::Index,
        };
    }

    pub fn get_hand(self) -> Hand {
        return match self {
            Col::One | Col::Two | Col::Three | Col::Four | Col::Five => Hand::Left,
            Col::Six
            | Col::Seven
            | Col::Eight
            | Col::Nine
            | Col::Ten
            | Col::Eleven
            | Col::Twelve => Hand::Right,
        };
    }

    // pub fn get_num(self) -> usize {
    //     return match self {
    //         Col::One => 1,
    //         Col::Two => 2,
    //         Col::Three => 3,
    //         Col::Four => 4,
    //         Col::Five => 5,
    //         Col::Six => 6,
    //         Col::Seven => 7,
    //         Col::Eight => 8,
    //         Col::Nine => 9,
    //         Col::Ten => 10,
    //         Col::Eleven => 11,
    //         Col::Twelve => 12,
    //     };
    // }

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

// TODO: Move errors into their own file

#[derive(Debug, Clone, Copy)]
pub enum KeyTemplate {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
    Comma,
    Period,
    SemiColon,
    ForwardSlash,
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Zero,
    Minus,
    Plus,
    LBracket,
    RBracket,
    Quote,
}

impl KeyTemplate {
    pub fn get_base(self) -> u8 {
        return match self {
            KeyTemplate::A => b'a',
            KeyTemplate::B => b'b',
            KeyTemplate::C => b'c',
            KeyTemplate::D => b'd',
            KeyTemplate::E => b'e',
            KeyTemplate::F => b'f',
            KeyTemplate::G => b'g',
            KeyTemplate::H => b'h',
            KeyTemplate::I => b'i',
            KeyTemplate::J => b'j',
            KeyTemplate::K => b'k',
            KeyTemplate::L => b'l',
            KeyTemplate::M => b'm',
            KeyTemplate::N => b'n',
            KeyTemplate::O => b'o',
            KeyTemplate::P => b'p',
            KeyTemplate::Q => b'q',
            KeyTemplate::R => b'r',
            KeyTemplate::S => b's',
            KeyTemplate::T => b't',
            KeyTemplate::U => b'u',
            KeyTemplate::V => b'v',
            KeyTemplate::W => b'w',
            KeyTemplate::X => b'x',
            KeyTemplate::Y => b'y',
            KeyTemplate::Z => b'z',
            KeyTemplate::Comma => b',',
            KeyTemplate::Period => b'.',
            KeyTemplate::SemiColon => b';',
            KeyTemplate::ForwardSlash => b'/',
            KeyTemplate::One => b'1',
            KeyTemplate::Two => b'2',
            KeyTemplate::Three => b'3',
            KeyTemplate::Four => b'4',
            KeyTemplate::Five => b'5',
            KeyTemplate::Six => b'6',
            KeyTemplate::Seven => b'7',
            KeyTemplate::Eight => b'8',
            KeyTemplate::Nine => b'9',
            KeyTemplate::Zero => b'0',
            KeyTemplate::Minus => b'-',
            KeyTemplate::Plus => b'+',
            KeyTemplate::LBracket => b'[',
            KeyTemplate::RBracket => b']',
            KeyTemplate::Quote => b'\'',
        };
    }

    pub fn get_shift(self) -> u8 {
        return match self {
            KeyTemplate::A => b'A',
            KeyTemplate::B => b'B',
            KeyTemplate::C => b'C',
            KeyTemplate::D => b'D',
            KeyTemplate::E => b'E',
            KeyTemplate::F => b'F',
            KeyTemplate::G => b'G',
            KeyTemplate::H => b'H',
            KeyTemplate::I => b'I',
            KeyTemplate::J => b'J',
            KeyTemplate::K => b'K',
            KeyTemplate::L => b'L',
            KeyTemplate::M => b'M',
            KeyTemplate::N => b'N',
            KeyTemplate::O => b'O',
            KeyTemplate::P => b'P',
            KeyTemplate::Q => b'Q',
            KeyTemplate::R => b'R',
            KeyTemplate::S => b'S',
            KeyTemplate::T => b'T',
            KeyTemplate::U => b'U',
            KeyTemplate::V => b'V',
            KeyTemplate::W => b'W',
            KeyTemplate::X => b'X',
            KeyTemplate::Y => b'Y',
            KeyTemplate::Z => b'Z',
            KeyTemplate::Comma => b'<',
            KeyTemplate::Period => b'>',
            KeyTemplate::SemiColon => b':',
            KeyTemplate::ForwardSlash => b'?',
            KeyTemplate::One => b'!',
            KeyTemplate::Two => b'@',
            KeyTemplate::Three => b'#',
            KeyTemplate::Four => b'$',
            KeyTemplate::Five => b'%',
            KeyTemplate::Six => b'^',
            KeyTemplate::Seven => b'&',
            KeyTemplate::Eight => b'*',
            KeyTemplate::Nine => b'(',
            KeyTemplate::Zero => b')',
            KeyTemplate::Minus => b'_',
            KeyTemplate::Plus => b'+',
            KeyTemplate::LBracket => b'{',
            KeyTemplate::RBracket => b'}',
            KeyTemplate::Quote => b'"',
        };
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ListType {
    Allow,
    Deny,
}
