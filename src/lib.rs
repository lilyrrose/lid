#![warn(clippy::pedantic)]
#![allow(clippy::cast_possible_truncation)]

use rand::{
    distributions::{Distribution, Uniform},
    rngs::OsRng,
    Rng,
};
use std::sync::{Arc, Mutex};

const BASE: u64 = 32;

// I decided 28 total bytes is good enough and leaves us with an improbable collision chance.
// When modifying, keep in mind that SEQUENCE_LENGTH is prng while PREFIX_LENGTH uses OsRng.
const PREFIX_LENGTH: usize = 16;
const SEQUENCE_LENGTH: usize = 12;

const MAX_SEQUENCE: u64 = BASE.pow(SEQUENCE_LENGTH as u32);

const MIN_INCREMENT: u64 = 100;
const MAX_INCREMENT: u64 = 1000;

const ID_LENGTH: usize = PREFIX_LENGTH + SEQUENCE_LENGTH;

lazy_static::lazy_static! {
    static ref BASE_ALPHABET: Vec<u8> = "ABCDEFGHIJKLMNOPQRSTUVWXYZ234567".as_bytes().to_vec();
    static ref GLOBAL_OID: Arc<Mutex<Oid>> = Arc::new(Mutex::new(Oid::new()));
}

pub struct Oid {
    prefix: Vec<u8>,
    sequence: u64,
    increment: u64,
    inner_buffer: Vec<u8>,
}

impl Oid {
    #[must_use]
    pub fn new() -> Self {
        let mut oid = Self {
            prefix: vec![0; PREFIX_LENGTH],
            sequence: 0,
            increment: 0,
            inner_buffer: vec![0; ID_LENGTH],
        };
        oid.reset();
        oid.new_prefix();
        oid
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

impl Default for Oid {
    fn default() -> Self {
        Self::new()
    }
}

#[must_use]
pub fn generate_oid() -> String {
    GLOBAL_OID
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .generate()
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    #[test]
    fn test_uniqueness() {
        let mut ids = HashSet::new();
        let num_iterations = 10_000_000;

        for _ in 0..num_iterations {
            let id = super::generate_oid();
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
