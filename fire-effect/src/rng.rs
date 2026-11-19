// SPDX-FileCopyrightText: 2026 the Blaustein LED contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Pseudo-random numbers and channel seeds.

/// First multiplier of the MurmurHash3 32-bit finalizer.
const FMIX_MULTIPLIER_1: u32 = 0x85eb_ca6b;

/// Second multiplier of the MurmurHash3 32-bit finalizer.
const FMIX_MULTIPLIER_2: u32 = 0xc2b2_ae35;

/// Returns the PRNG seed of a channel: distinct per channel and never zero.
///
/// The MurmurHash3 finalizer is a bijection on `u32` that maps only 0 to 0, so mixing
/// `channel + 1` gives a distinct non-zero seed for every channel below `u32::MAX`.
#[must_use]
pub const fn channel_seed(channel: u32) -> u32 {
    let mut hash = channel.wrapping_add(1);
    hash ^= hash >> 16_u32;
    hash = hash.wrapping_mul(FMIX_MULTIPLIER_1);
    hash ^= hash >> 13_u32;
    hash = hash.wrapping_mul(FMIX_MULTIPLIER_2);
    hash ^= hash >> 16_u32;
    hash
}

/// Xorshift32 pseudo-random number generator: fast, small, and good enough for flicker.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rng(u32);

impl Rng {
    /// Creates a generator from a non-zero seed.
    ///
    /// # Panics
    ///
    /// Panics if `seed` is zero, the fixed point of Xorshift32. Evaluated in a const context,
    /// that is a build error.
    #[must_use]
    pub const fn new(seed: u32) -> Self {
        assert!(
            seed != 0,
            "a zero seed would keep Xorshift32 at zero forever"
        );
        Self(seed)
    }

    /// Advances the generator and returns the new state split into its high and low half.
    #[expect(
        clippy::cast_possible_truncation,
        reason = "the halves of a u32 are taken apart on purpose"
    )]
    pub const fn next_pair(&mut self) -> [u16; 2] {
        self.0 = xorshift32(self.0);
        [(self.0 >> 16_u32) as u16, self.0 as u16]
    }
}

/// Advances a Xorshift32 state.
#[must_use]
const fn xorshift32(mut state: u32) -> u32 {
    state ^= state << 13_u32;
    state ^= state >> 17_u32;
    state ^= state << 5_u32;
    state
}

#[cfg(test)]
mod tests {
    //! Xorshift32 and channel seeds.

    use super::{Rng, channel_seed, xorshift32};

    /// Xorshift32 matches its reference sequence and does not reach zero.
    #[test]
    fn xorshift_matches_reference() {
        assert_eq!(xorshift32(1), 270_369, "first Xorshift32 value from seed 1");
        assert_eq!(
            xorshift32(270_369),
            67_634_689,
            "second Xorshift32 value from seed 1"
        );
        let mut state = 1_u32;
        for _ in 0..100_000_u32 {
            state = xorshift32(state);
            assert_ne!(state, 0, "Xorshift32 left its non-zero cycle");
        }
    }

    /// A pair holds the high and the low half of the next state.
    #[test]
    fn pair_splits_the_state() {
        let mut rng = Rng::new(1);
        assert_eq!(rng.next_pair(), [0x0004, 0x2021], "270369 is 0x0004_2021");
    }

    /// A zero seed is rejected.
    #[test]
    #[should_panic(expected = "a zero seed")]
    fn rejects_zero_seed() {
        let _rng = Rng::new(0);
    }

    /// Channel seeds are non-zero and distinct.
    #[test]
    fn channel_seeds_are_distinct_and_non_zero() {
        let mut seeds: Vec<u32> = (0..1_024).map(channel_seed).collect();
        assert!(!seeds.contains(&0), "a channel seed is zero");
        seeds.sort_unstable();
        seeds.dedup();
        assert_eq!(seeds.len(), 1_024, "two channels share a seed");
    }
}
