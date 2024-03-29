//! Fast and customizable ID generator.
//!
//! # Quick Start
//!
//! The easiest way to use LID is by using the `easy` feature and using [`generate_distributed`] or [`generate_random`].
//! These use a static [LID] instance backed by a [Mutex].
//!
//! You may also change the alphabet used by switching up the feature flags.
//! The available features are: base32, base36, and base62.
//! NOTE: When using base62, the default ID size will change to 20 bytes.
//! If not using the base62 feature, the default ID size will be 28 bytes.
//!
//! # Unsafe
//! By default this crate uses unsafe when converting from &[u8] to String, this is safe due to the alphabets always being UTF8.
//! If you would like, you can change [`generate`] to return a [`Result<String, FromUtf8Error>`] by enabling the `no-unsafe` feature.
//!
//! # Customization
//!
//! You can always customize your ID size with const generics:
//! ```
//! use lid::LID;
//!
//! let mut lid = LID::<12, 8>::default(); // This will give you a 20 byte ID.
//! println!("{:?}", lid.generate());
//! ```
//!
//! You can also customize the 'randomness' of the IDs generated by changing the `MIN_INCREMENT` and `MAX_INCREMENT` generic values.
//! ```
//! use lid::LID;
//!
//! let mut lid = LID::<6, 9, 1000, 1_000_000>::default();
//! println!("{:?}", lid.generate());
//! ```
//!

#![warn(clippy::pedantic)]
#![allow(clippy::cast_possible_truncation)]

#[cfg(not(any(feature = "base32", feature = "base36", feature = "base62")))]
compile_error!("You must enable one of the alphabet related features! 'base32' is the default.");

#[cfg(any(
    all(feature = "base32", feature = "base36"),
    all(feature = "base32", feature = "base62"),
    all(feature = "base36", feature = "base32"),
    all(feature = "base36", feature = "base62"),
    all(feature = "base62", feature = "base32"),
    all(feature = "base62", feature = "base36"),
))]
compile_error!("You must only have one of the alphabet related features enabled!");

use rand::{
    distributions::{Distribution, Uniform},
    rngs::OsRng,
    Rng,
};

pub mod configs {
    use super::LID;

    #[must_use]
    pub fn new_distributed() -> LID {
        LID::default()
    }

    #[must_use]
    pub fn new_random<const PREFIX_LENGTH: usize, const SEQUENCE_LENGTH: usize>(
    ) -> LID<PREFIX_LENGTH, SEQUENCE_LENGTH, 10_000_000, { u64::MAX }> {
        LID::default()
    }
}

#[cfg(feature = "easy")]
pub mod easy {
    use lazy_static::lazy_static;
    use spin::Mutex;

    use crate::{
        configs::{new_distributed, new_random},
        LID,
    };

    lazy_static! {
        static ref DISTRIBUTED_INST: Mutex<LID> = Mutex::new(new_distributed());
        static ref RANDOM_INST: Mutex<LID<12, 8, 10_000_000, { u64::MAX }>> =
            Mutex::new(new_random());
    }

    #[must_use]
    #[cfg(not(feature = "no-unsafe"))]
    pub fn generate_distributed() -> String {
        DISTRIBUTED_INST.lock().generate()
    }

    #[must_use]
    #[cfg(feature = "no-unsafe")]
    pub fn generate_distributed() -> Result<String, std::string::FromUtf8Error> {
        DISTRIBUTED_INST.lock().generate()
    }

    #[must_use]
    #[cfg(not(feature = "no-unsafe"))]
    pub fn generate_random() -> String {
        RANDOM_INST.lock().generate()
    }

    #[must_use]
    #[cfg(feature = "no-unsafe")]
    pub fn generate_random() -> Result<String, std::string::FromUtf8Error> {
        RANDOM_INST.lock().generate()
    }
}

#[cfg(feature = "base32")]
pub const BASE_ALPHABET: &[u8] = "ABCDEFGHIJKLMNOPQRSTUVWXYZ234567".as_bytes();

#[cfg(feature = "base36")]
pub const BASE_ALPHABET: &[u8] = "ABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890".as_bytes();

#[cfg(feature = "base62")]
pub const BASE_ALPHABET: &[u8] =
    "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890".as_bytes();

const BASE: u64 = BASE_ALPHABET.len() as u64;

// Base62 has to have a smaller default length because MAX_SEQUENCE is too big otherwise.
#[cfg(feature = "base62")]
pub struct LID<
    const PREFIX_LENGTH: usize = 12,
    const SEQUENCE_LENGTH: usize = 8,
    const MIN_INCREMENT: u64 = 100,
    const MAX_INCREMENT: u64 = 1000,
> {
    prefix: Vec<u8>,
    sequence: u64,
    increment: u64,
    inner_buffer: Vec<u8>,
}

#[cfg(not(feature = "base62"))]
/// The combined total of `PREFIX_LENGTH` and `SEQUENCE_LENGTH` is the length of the ID.
/// By default, this is 28 bytes.
pub struct LID<
    const PREFIX_LENGTH: usize = 16,
    const SEQUENCE_LENGTH: usize = 12,
    const MIN_INCREMENT: u64 = 100,
    const MAX_INCREMENT: u64 = 1000,
> {
    prefix: Vec<u8>,
    sequence: u64,
    increment: u64,
    inner_buffer: Vec<u8>,
}

impl<
        const PREFIX_LENGTH: usize,
        const SEQUENCE_LENGTH: usize,
        const MIN_INCREMENT: u64,
        const MAX_INCREMENT: u64,
    > LID<PREFIX_LENGTH, SEQUENCE_LENGTH, MIN_INCREMENT, MAX_INCREMENT>
{
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
    #[cfg(not(feature = "no-unsafe"))]
    pub fn generate(&mut self) -> String {
        self.new_sequence();
        self.inner_buffer[..PREFIX_LENGTH].copy_from_slice(&self.prefix);
        Self::copy_sequence_into(&mut self.inner_buffer[PREFIX_LENGTH..], self.sequence);

        // Safety: The alphabet used ensures that the bytes are valid UTF-8.
        unsafe { String::from_utf8_unchecked(self.inner_buffer.clone()) }
    }

    /// Generates a new ID.
    #[cfg(feature = "no-unsafe")]
    pub fn generate(&mut self) -> Result<String, std::string::FromUtf8Error> {
        self.new_sequence();
        self.inner_buffer[..PREFIX_LENGTH].copy_from_slice(&self.prefix);
        Self::copy_sequence_into(&mut self.inner_buffer[PREFIX_LENGTH..], self.sequence);

        String::from_utf8(self.inner_buffer.clone())
    }
}

impl<
        const PREFIX_LENGTH: usize,
        const SEQUENCE_LENGTH: usize,
        const MIN_INCREMENT: u64,
        const MAX_INCREMENT: u64,
    > Default for LID<PREFIX_LENGTH, SEQUENCE_LENGTH, MIN_INCREMENT, MAX_INCREMENT>
{
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use crate::*;

    #[test]
    #[cfg(not(feature = "no-unsafe"))]
    fn test_uniqueness() {
        let mut lid = configs::new_distributed();

        let mut ids = HashSet::new();
        let num_iterations = 10_000_000;

        for _ in 0..num_iterations {
            let id = lid.generate();
            assert!(!ids.contains(&id), "Duplicate ID found: {id}");
            ids.insert(id);
        }

        assert_eq!(
            ids.len(),
            num_iterations,
            "Number of unique IDs does not match the number of iterations"
        );
    }

    #[test]
    #[cfg(feature = "no-unsafe")]
    fn test_uniqueness() -> Result<(), Box<dyn std::error::Error>> {
        let mut lid = configs::new_distributed();

        let mut ids = HashSet::new();
        let num_iterations = 10_000_000;

        for _ in 0..num_iterations {
            let id = lid.generate()?;
            assert!(!ids.contains(&id), "Duplicate ID found: {id}");
            ids.insert(id);
        }

        assert_eq!(
            ids.len(),
            num_iterations,
            "Number of unique IDs does not match the number of iterations"
        );

        Ok(())
    }

    #[test]
    #[cfg(all(feature = "easy", not(feature = "no-unsafe")))]
    fn test_easy() {
        use self::easy::generate_distributed;

        let _ = generate_distributed();
    }

    #[test]
    #[cfg(all(feature = "easy", feature = "no-unsafe"))]
    fn test_easy() -> Result<(), Box<dyn std::error::Error>> {
        use self::easy::generate_distributed;

        let _ = generate_distributed()?;
        Ok(())
    }
}
