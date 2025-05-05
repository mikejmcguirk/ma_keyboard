// TEST: No tuple should have two elements of the same value
// TEST: No capital letters should be in element 0
// TEST: No numbers should be in element 1
// TEST: If one of the elements is a letter, both elements should be letters
// TEST: If one element is a number, the other element should not be a letter
pub const KEY_TUPLES: [(u8, u8); 30] = [
    (b'a', b'A'),
    (b'b', b'B'),
    (b'c', b'C'),
    (b'd', b'D'),
    (b'e', b'E'),
    (b'f', b'F'),
    (b'g', b'G'),
    (b'h', b'H'),
    (b'i', b'I'),
    (b'j', b'J'),
    (b'k', b'K'),
    (b'l', b'L'),
    (b'm', b'M'),
    (b'n', b'N'),
    (b'o', b'O'),
    (b'p', b'P'),
    (b'q', b'Q'),
    (b'r', b'R'),
    (b's', b'S'),
    (b't', b'T'),
    (b'u', b'U'),
    (b'v', b'V'),
    (b'w', b'W'),
    (b'x', b'X'),
    (b'y', b'Y'),
    (b'z', b'Z'),
    (b',', b'>'),
    (b'.', b'>'),
    (b';', b':'),
    (b'\'', b'"'),
];
