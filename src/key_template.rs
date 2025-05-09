use strum::EnumIter;

#[derive(EnumIter, Debug, Clone, Copy)]
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
            KeyTemplate::Plus => b'=',
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

    // TODO: If we put J in a corner, that his implications for Vim controls
    #[expect(clippy::match_same_arms)]
    pub fn get_valid_locations(self) -> Vec<(usize, usize)> {
        return match self {
            KeyTemplate::One => vec![(0, 0)],
            KeyTemplate::Two => vec![(0, 1)],
            KeyTemplate::Three => vec![(0, 2)],
            KeyTemplate::Four => vec![(0, 3)],
            KeyTemplate::Five => vec![(0, 4)],
            KeyTemplate::Six => vec![(0, 5)],
            KeyTemplate::Seven => vec![(0, 6)],
            KeyTemplate::Eight => vec![(0, 7)],
            KeyTemplate::Nine => vec![(0, 8)],
            KeyTemplate::Zero => vec![(0, 9)],
            KeyTemplate::LBracket => vec![(0, 10)],
            KeyTemplate::RBracket => vec![(0, 11)],
            KeyTemplate::Comma => vec![(1, 0)],
            KeyTemplate::Period => vec![(1, 1)],
            KeyTemplate::Minus => vec![(1, 10)],
            KeyTemplate::Plus => vec![(1, 11)],
            KeyTemplate::ForwardSlash => vec![(2, 10)],
            KeyTemplate::SemiColon => vec![(1, 5), (3, 4)],
            KeyTemplate::Quote => vec![(1, 5), (3, 4)],
            KeyTemplate::Q => not_home(&vec![(1, 0)]),
            KeyTemplate::W => not_home(&vec![(1, 1)]),
            KeyTemplate::E => vec![(2, 6), (2, 7)],
            KeyTemplate::R => alpha_slots(&vec![(1, 3)]),
            KeyTemplate::T => alpha_slots(&vec![(1, 4)]),
            KeyTemplate::Y => alpha_slots(&vec![(1, 5)]),
            KeyTemplate::U => alpha_slots(&vec![(1, 6)]),
            KeyTemplate::I => alpha_slots(&vec![(1, 7)]),
            KeyTemplate::O => alpha_slots(&vec![(1, 8)]),
            KeyTemplate::P => not_home(&vec![(1, 9)]),
            KeyTemplate::A => major_home_slots(&vec![(2, 0)]),
            KeyTemplate::S => alpha_slots(&vec![(2, 1)]),
            KeyTemplate::D => alpha_slots(&vec![(2, 2)]),
            KeyTemplate::F => alpha_slots(&vec![(2, 3)]),
            KeyTemplate::G => alpha_slots(&vec![(2, 4)]),
            KeyTemplate::H => alpha_slots(&vec![(2, 5)]),
            KeyTemplate::J => vec![(1, 8), (1, 9), (3, 0), (3, 9)],
            KeyTemplate::K => alpha_slots(&vec![(2, 7)]),
            KeyTemplate::L => alpha_slots(&vec![(2, 8)]),
            KeyTemplate::Z => not_home(&vec![(3, 0)]),
            KeyTemplate::X => not_home(&vec![(3, 1)]),
            KeyTemplate::C => alpha_slots(&vec![(3, 2)]),
            KeyTemplate::V => not_home(&vec![(3, 3)]),
            KeyTemplate::B => not_home(&vec![(3, 4)]),
            KeyTemplate::N => alpha_slots(&vec![(3, 5)]),
            KeyTemplate::M => alpha_slots(&vec![(3, 6)]),
        };
    }

    pub fn get_starting_location(self) -> (usize, usize) {
        return match self {
            KeyTemplate::One => (0, 0),
            KeyTemplate::Two => (0, 1),
            KeyTemplate::Three => (0, 2),
            KeyTemplate::Four => (0, 3),
            KeyTemplate::Five => (0, 4),
            KeyTemplate::Six => (0, 5),
            KeyTemplate::Seven => (0, 6),
            KeyTemplate::Eight => (0, 7),
            KeyTemplate::Nine => (0, 8),
            KeyTemplate::Zero => (0, 9),
            KeyTemplate::LBracket => (0, 10),
            KeyTemplate::RBracket => (0, 11),
            KeyTemplate::Comma => (1, 0),
            KeyTemplate::Period => (1, 1),
            KeyTemplate::A => (2, 2),
            KeyTemplate::B => (1, 3),
            KeyTemplate::C => (1, 4),
            KeyTemplate::SemiColon => (1, 5),
            KeyTemplate::D => (1, 6),
            KeyTemplate::E => (2, 7),
            KeyTemplate::F => (1, 8),
            KeyTemplate::G => (1, 2),
            KeyTemplate::Minus => (1, 10),
            KeyTemplate::Plus => (1, 11),
            KeyTemplate::H => (2, 0),
            KeyTemplate::I => (2, 1),
            KeyTemplate::J => (1, 9),
            KeyTemplate::K => (2, 3),
            KeyTemplate::L => (2, 4),
            KeyTemplate::M => (2, 5),
            KeyTemplate::N => (2, 6),
            KeyTemplate::O => (1, 7),
            KeyTemplate::P => (3, 1),
            KeyTemplate::Q => (3, 0),
            KeyTemplate::ForwardSlash => (2, 10),
            KeyTemplate::R => (2, 9),
            KeyTemplate::S => (2, 8),
            KeyTemplate::T => (3, 2),
            KeyTemplate::U => (3, 3),
            KeyTemplate::Quote => (3, 4),
            KeyTemplate::V => (3, 5),
            KeyTemplate::W => (3, 6),
            KeyTemplate::X => (3, 7),
            KeyTemplate::Y => (3, 8),
            KeyTemplate::Z => (3, 9),
        };
    }
}

// Notes on key locations:
// - For now, all alpha keys will be allowed on all valid alpha spots. Some ideas to specify this
// depending on training results:
//  - Limit E to right index and middle
//  - Limit ENIARLTOSU to home keys + top middles
//  An issue with this though is, ultimately, typing is more about patterns than the individual
//  keys. Intuitively, J should go on a pinky key, but maybe in combination with everything else it
//  fits better on a ring finger
//  In general, manual edits should be made as they are found through training, in order to hone
//  the population in on better solutions
//  TODO: One thing to keep in mind is, the maximum possible shuffles might be limited by the
//  number of locked keys. The code will need to handle that
// A counter to this though - The layout should be easy to learn and immediately feel comfortable.
// R and L being pinky and ring top row on Dvorak is... a choice

fn alpha_slots(exclusions: &[(usize, usize)]) -> Vec<(usize, usize)> {
    let slot_groups: Vec<Vec<(usize, usize)>> =
        vec![top_row_alpha(), home_row_alpha(), bottom_row_alpha()];

    let mut slot_groups_flat: Vec<(usize, usize)> = slot_groups.into_iter().flatten().collect();
    slot_groups_flat.retain(|x| return !exclusions.contains(x));

    return slot_groups_flat;
}

fn major_home_slots(exclusions: &[(usize, usize)]) -> Vec<(usize, usize)> {
    let mut slots = vec![
        (2, 0),
        (2, 1),
        (2, 2),
        (2, 3),
        (2, 6),
        (2, 7),
        (2, 8),
        (2, 9),
    ];

    slots.retain(|x| return !exclusions.contains(x));

    return slots;
}

fn not_home(exclusions: &[(usize, usize)]) -> Vec<(usize, usize)> {
    let slot_groups: Vec<Vec<(usize, usize)>> = vec![top_row_alpha(), bottom_row_alpha()];

    let mut slot_groups_flat: Vec<(usize, usize)> = slot_groups.into_iter().flatten().collect();
    slot_groups_flat.retain(|x| return !exclusions.contains(x));

    return slot_groups_flat;
}

fn top_row_alpha() -> Vec<(usize, usize)> {
    return vec![
        // The below slots are omitted to handle the , and . keys
        // (1, 0),
        // (1, 1),
        (1, 2),
        (1, 3),
        (1, 4),
        // (1, 5) is skipped so this can hold a symbol key
        (1, 6),
        (1, 7),
        (1, 8),
        (1, 9),
    ];
}

fn home_row_alpha() -> Vec<(usize, usize)> {
    return vec![
        (2, 0),
        (2, 1),
        (2, 2),
        (2, 3),
        (2, 4),
        (2, 5),
        (2, 6),
        (2, 7),
        (2, 8),
        (2, 9),
    ];
}

fn bottom_row_alpha() -> Vec<(usize, usize)> {
    return vec![
        (3, 0),
        (3, 1),
        (3, 2),
        (3, 3),
        // (3, 4) skipped so this can hold a symbol key
        (3, 5),
        (3, 6),
        (3, 7),
        (3, 8),
        (3, 9),
    ];
}
