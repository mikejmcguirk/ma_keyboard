// TEST: No tuple should have two elements of the same value
// TEST: No capital letters should be in element 0
// TEST: No numbers should be in element 1
// TEST: If one of the elements is a letter, both elements should be letters
// TEST: If one element is a number, the other element should not be a letter
pub const KEY_TUPLES: [(char, char); 30] = [
    ('a', 'A'),
    ('b', 'B'),
    ('c', 'C'),
    ('d', 'D'),
    ('e', 'E'),
    ('f', 'F'),
    ('g', 'G'),
    ('h', 'H'),
    ('i', 'I'),
    ('j', 'J'),
    ('k', 'K'),
    ('l', 'L'),
    ('m', 'M'),
    ('n', 'N'),
    ('o', 'O'),
    ('p', 'P'),
    ('q', 'Q'),
    ('r', 'R'),
    ('s', 'S'),
    ('t', 'T'),
    ('u', 'U'),
    ('v', 'V'),
    ('w', 'W'),
    ('x', 'X'),
    ('y', 'Y'),
    ('z', 'Z'),
    ('<', ','),
    ('>', '.'),
    (';', ':'),
    ('/', '?'),
];
