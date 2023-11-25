#![warn(clippy::pedantic)]
#![allow(clippy::cast_possible_truncation)]

use rand::{
    distributions::{Distribution, Uniform},
    rngs::OsRng,
    Rng,
};
use spin::mutex::Mutex;

const BASE: u64 = 36;

// I decided 28 total bytes is good enough and leaves us with an extremely improbable collision chance.
// When modifying, keep in mind that SEQUENCE_LENGTH is prng while PREFIX_LENGTH uses OsRng.
const PREFIX_LENGTH: usize = 16;
const SEQUENCE_LENGTH: usize = 12;

const MAX_SEQUENCE: u64 = BASE.pow(SEQUENCE_LENGTH as u32);

const MIN_INCREMENT: u64 = 100;
const MAX_INCREMENT: u64 = 1000;

const ID_LENGTH: usize = PREFIX_LENGTH + SEQUENCE_LENGTH;

lazy_static::lazy_static! {
    static ref BASE_ALPHABET: Vec<u8> = "ABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890".as_bytes().to_vec();
    static ref GLOBAL_LID: Mutex<LID> = Mutex::new(LID::new());
}

pub struct LID {
    prefix: Vec<u8>,
    sequence: u64,
    increment: u64,
    inner_buffer: Vec<u8>,
}

impl LID {
    #[must_use]
    pub fn new() -> Self {
        let mut lid = Self {
            prefix: vec![0; PREFIX_LENGTH],
            sequence: 0,
            increment: 0,
            inner_buffer: vec![0; ID_LENGTH],
        };
        lid.reset();
        lid.new_prefix();
        lid
    }

    fn reset(&mut self) {
        self.sequence = OsRng.gen_range(0..MAX_SEQUENCE);
        self.increment = OsRng.gen_range(MIN_INCREMENT..MAX_INCREMENT);
    }

    fn new_prefix(&mut self) {
        let between = Uniform::from(0..BASE);
        for i in 0..PREFIX_LENGTH {
            self.prefix[i] = BASE_ALPHABET[between.sample(&mut OsRng) as usize];
        }
    }

    fn new_sequence(&mut self) {
        self.sequence = (self.sequence + self.increment) % MAX_SEQUENCE;
        if self.sequence == 0 {
            self.new_prefix();
        }
    }

    fn copy_sequence_into(buffer: &mut [u8], mut sequence: u64) {
        for digit in buffer.iter_mut().rev() {
            *digit = BASE_ALPHABET[(sequence % BASE) as usize];
            sequence /= BASE;
        }
    }

    pub fn generate(&mut self) -> String {
        self.new_sequence();
        self.inner_buffer[..PREFIX_LENGTH].copy_from_slice(&self.prefix);
        Self::copy_sequence_into(&mut self.inner_buffer[PREFIX_LENGTH..], self.sequence);
        unsafe { String::from_utf8_unchecked(self.inner_buffer.clone()) }
    }
}

impl Default for LID {
    fn default() -> Self {
        Self::new()
    }
}

#[must_use]
pub fn generate_lid() -> String {
    GLOBAL_LID.lock().generate()
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    #[test]
    fn test_uniqueness() {
        let mut ids = HashSet::new();
        let num_iterations = 10_000_000;

        for _ in 0..num_iterations {
            let id = super::generate_lid();
            assert!(!ids.contains(&id), "Duplicate ID found: {id}");
            ids.insert(id);
        }

        assert_eq!(
            ids.len(),
            num_iterations,
            "Number of unique IDs does not match the number of iterations"
        );
    }
}
