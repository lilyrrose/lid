//! Fast and customizable ID generator.
//!
//! # Quick Start
//!
//! The easiest way to use LID is by using the [`generate_lid()`] function.
//! This uses a global [LID] instance behind a [Mutex].
//!
//! You may also change the alphabet used by switching up the feature flags.
//! The available features are: base32, base36, and base62.
//! NOTE: When using base62, the default ID size will change to 20 bytes.
//! If not using the base62 feature, the default ID size will be 28 bytes.
//!
//! ```
//! use lid::{LID, generate_lid};
//!
//! println!("{}", generate_lid());
//!
//! // Or, create your own instance.
//! let mut lid = LID::<12, 8>::new(); // This will give you a 20 byte ID.
//! println!("{}", lid.generate());
//! ```

#![warn(clippy::pedantic)]
#![allow(clippy::cast_possible_truncation)]

#[cfg(not(any(feature = "base32", feature = "base36", feature = "base62")))]
compile_error!("You must enable one of the features! base32 is the default.");

#[cfg(any(
    all(feature = "base32", feature = "base36"),
    all(feature = "base32", feature = "base62"),
    all(feature = "base36", feature = "base32"),
    all(feature = "base36", feature = "base62"),
    all(feature = "base62", feature = "base32"),
    all(feature = "base62", feature = "base36"),
))]
compile_error!("You must only have one of the features enabled! base32 is the default.");

use rand::{
    distributions::{Distribution, Uniform},
    rngs::OsRng,
    Rng,
};
use spin::mutex::Mutex;

#[cfg(feature = "base32")]
mod base32 {
    pub const BASE: u64 = 32;

    lazy_static::lazy_static! {
        pub static ref BASE_ALPHABET: Vec<u8> = "ABCDEFGHIJKLMNOPQRSTUVWXYZ234567".as_bytes().to_vec();
    }
}

#[cfg(feature = "base36")]
mod base36 {
    pub const BASE: u64 = 36;

    lazy_static::lazy_static! {
        pub static ref BASE_ALPHABET: Vec<u8> = "ABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890".as_bytes().to_vec();
    }
}

#[cfg(feature = "base62")]
mod base62 {
    pub const BASE: u64 = 62;

    lazy_static::lazy_static! {
        pub static ref BASE_ALPHABET: Vec<u8> = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890".as_bytes().to_vec();
    }
}

#[cfg(feature = "base32")]
use base32::{BASE, BASE_ALPHABET};

#[cfg(feature = "base36")]
use base36::{BASE, BASE_ALPHABET};

#[cfg(feature = "base62")]
use base62::{BASE, BASE_ALPHABET};

const MIN_INCREMENT: u64 = 100;
const MAX_INCREMENT: u64 = 1000;

lazy_static::lazy_static! {
    static ref GLOBAL_LID: Mutex<LID> = Mutex::new(LID::new());
}

// Base62 has to have smaller defaults because MAX_SEQUENCE is too big otherwise.
#[cfg(feature = "base62")]
pub struct LID<const PREFIX_LENGTH: usize = 12, const SEQUENCE_LENGTH: usize = 8> {
    prefix: Vec<u8>,
    sequence: u64,
    increment: u64,
    inner_buffer: Vec<u8>,
}

#[cfg(not(feature = "base62"))]
/// The combined total of `PREFIX_LENGTH` and `SEQUENCE_LENGTH` is the length of the ID.
/// By default, this is 28 bytes.
pub struct LID<const PREFIX_LENGTH: usize = 16, const SEQUENCE_LENGTH: usize = 12> {
    prefix: Vec<u8>,
    sequence: u64,
    increment: u64,
    inner_buffer: Vec<u8>,
}

impl<const PREFIX_LENGTH: usize, const SEQUENCE_LENGTH: usize> LID<PREFIX_LENGTH, SEQUENCE_LENGTH> {
    const MAX_SEQUENCE: u64 = BASE.pow(SEQUENCE_LENGTH as u32);
    const ID_LENGTH: usize = PREFIX_LENGTH + SEQUENCE_LENGTH;

    #[must_use]
    pub fn new() -> Self {
        let mut lid = Self {
            prefix: vec![0; PREFIX_LENGTH],
            sequence: 0,
            increment: 0,
            inner_buffer: vec![0; Self::ID_LENGTH],
        };
        lid.reset();
        lid.new_prefix();
        lid
    }

    fn reset(&mut self) {
        self.sequence = OsRng.gen_range(0..Self::MAX_SEQUENCE);
        self.increment = OsRng.gen_range(MIN_INCREMENT..MAX_INCREMENT);
    }

    fn new_prefix(&mut self) {
        let between = Uniform::from(0..BASE);
        for i in 0..PREFIX_LENGTH {
            self.prefix[i] = BASE_ALPHABET[between.sample(&mut OsRng) as usize];
        }
    }

    fn new_sequence(&mut self) {
        self.sequence = (self.sequence + self.increment) % Self::MAX_SEQUENCE;
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

    /// Generates a new ID.
    pub fn generate(&mut self) -> String {
        self.new_sequence();
        self.inner_buffer[..PREFIX_LENGTH].copy_from_slice(&self.prefix);
        Self::copy_sequence_into(&mut self.inner_buffer[PREFIX_LENGTH..], self.sequence);
        // Safety: The alphabet used ensures that the bytes are valid UTF-8.
        unsafe { String::from_utf8_unchecked(self.inner_buffer.clone()) }
    }
}

impl<const PREFIX_LENGTH: usize, const SEQUENCE_LENGTH: usize> Default
    for LID<PREFIX_LENGTH, SEQUENCE_LENGTH>
{
    fn default() -> Self {
        Self::new()
    }
}

/// Generates an ID using the global LID instance.
/// This is slightly slower than creating your own [LID] instance due to using a [Mutex].
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
