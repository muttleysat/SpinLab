//! Chiavi musicali e ruota Camelot.

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Key {
    /// pitch class 0..11 (C=0)
    pub pc: u8,
    pub minor: bool,
}

/// Numero Camelot (1..12) di una chiave.
pub fn camelot_number(k: Key) -> u8 {
    let major_pc = if k.minor { (k.pc + 3) % 12 } else { k.pc } as u32;
    (((major_pc * 7) % 12 + 7) % 12 + 1) as u8
}

pub fn camelot_code(k: Key) -> String {
    format!("{}{}", camelot_number(k), if k.minor { 'A' } else { 'B' })
}

const NAMES: [&str; 12] = ["C", "Db", "D", "Eb", "E", "F", "F#", "G", "Ab", "A", "Bb", "B"];

pub fn key_name(k: Key) -> String {
    format!("{}{}", NAMES[k.pc as usize % 12], if k.minor { "m" } else { "" })
}

pub fn transpose(k: Key, semitones: i32) -> Key {
    Key { pc: (k.pc as i32 + semitones).rem_euclid(12) as u8, minor: k.minor }
}

/// Semitoni (-6..=5) per portare la chiave `k` sul numero Camelot `number` (stesso modo).
pub fn shift_to(k: Key, number: u8) -> i32 {
    (-6..=5)
        .find(|&s| camelot_number(transpose(k, s)) == number)
        .unwrap_or(0)
}
