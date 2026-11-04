// SPDX-FileCopyrightText: 2026 the Blaustein LED contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Flicker simulation for one LED channel.

use core::marker::PhantomData;

use crate::config::{FireConfig, FireProfile};
use crate::fixed_point::{Q8, Q16};

/// Half of one Q8 unit; adding it before a right shift rounds to nearest.
const Q8_HALF: i32 = 1 << (Q8 - 1);

/// Offset that centers a random byte around zero.
const BYTE_CENTER: i32 = 128;

/// Upper bound of the target intensity.
const INTENSITY_MAX: i32 = 255;

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

/// State of the fire simulation for one LED channel.
///
/// The configuration comes from the profile `P` rather than a field, so every
/// configuration-derived operand folds into an immediate.
#[derive(Clone, Copy)]
pub struct FireState<P> {
    /// Xorshift32 state, never zero.
    rng: u32,
    /// Breathing envelope in Q16, within `MIN_Q16..=MAX_Q16`.
    base_q16: i32,
    /// Signed envelope step in Q16; the sign is the direction of travel.
    step_q16: i32,
    /// Smoothed output intensity in Q8, within `0..=255 << 8`.
    out_q8: i32,
    /// Decaying flare intensity.
    pulse: i32,
    /// Binds the state to its compile-time configuration.
    profile: PhantomData<P>,
}

impl<P: FireProfile> FireState<P> {
    /// Validated configuration; an invalid profile fails the build where it is used.
    const CONFIG: FireConfig = {
        P::CONFIG.validate();
        P::CONFIG
    };
    /// Upper bound of the breathing envelope in Q16.
    const MAX_Q16: i32 = (Self::CONFIG.max_intensity as i32) << Q16;
    /// Lower bound of the breathing envelope in Q16.
    const MIN_Q16: i32 = (Self::CONFIG.min_intensity as i32) << Q16;

    /// Creates a channel simulation from a non-zero PRNG seed.
    ///
    /// # Panics
    ///
    /// Panics if `seed` is zero, the fixed point of Xorshift32. Evaluate it in a `const` block to
    /// turn that into a build error.
    #[must_use]
    pub const fn new(seed: u32) -> Self {
        assert!(
            seed != 0,
            "a zero seed would keep Xorshift32 at zero forever"
        );
        Self {
            rng: seed,
            base_q16: Self::MIN_Q16,
            step_q16: Self::CONFIG.step_per_tick_q16(),
            out_q8: (Self::CONFIG.min_intensity as i32) << Q8,
            pulse: 0,
            profile: PhantomData,
        }
    }

    /// Creates `N` channel simulations, seeded with [`channel_seed`] of their index.
    ///
    /// Evaluate it in a `const` block, so the whole bank is built at compile time.
    #[must_use]
    pub const fn channels<const N: usize>() -> [Self; N]
    where
        P: Copy,
    {
        let mut states = [Self::new(channel_seed(0)); N];
        let mut index = 0;
        let mut channel = 0;
        while index < N {
            states[index] = Self::new(channel_seed(channel));
            index += 1;
            channel += 1;
        }
        states
    }

    /// Advances the simulation by one tick and returns the new intensity.
    pub fn update(&mut self) -> u8 {
        let base = self.advance_envelope();

        // One PRNG step per tick; jitter and flare read independent bytes of the same word.
        self.rng = xorshift32(self.rng);
        let [jitter_byte, _, _, pulse_byte] = self.rng.to_be_bytes();

        let target =
            (base + jitter(jitter_byte, Self::CONFIG.jitter_max) + self.advance_pulse(pulse_byte))
                .clamp(0_i32, INTENSITY_MAX);
        self.smooth_towards(target)
    }

    /// Moves the triangular breathing envelope one step and returns its integer part.
    fn advance_envelope(&mut self) -> i32 {
        self.base_q16 = (self.base_q16 + self.step_q16).clamp(Self::MIN_Q16, Self::MAX_Q16);
        if self.base_q16 == Self::MIN_Q16 || self.base_q16 == Self::MAX_Q16 {
            self.step_q16 = -self.step_q16;
        }
        self.base_q16 >> Q16
    }

    /// Triggers a flare with probability `pulse_prob / 256`, decays it and returns it.
    fn advance_pulse(&mut self, random: u8) -> i32 {
        if random < Self::CONFIG.pulse_prob {
            self.pulse += i32::from(Self::CONFIG.pulse_boost);
        }
        self.pulse = (self.pulse * i32::from(Self::CONFIG.pulse_decay_q8)) >> Q8;
        self.pulse
    }

    /// Moves the output a `smooth_q8 / 256` fraction towards `target` and returns it.
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "the EMA never overshoots its target in 0..=255, so `out_q8 >> 8` is in 0..=255"
    )]
    fn smooth_towards(&mut self, target: i32) -> u8 {
        let diff = (target << Q8) - self.out_q8;
        self.out_q8 += (diff * i32::from(Self::CONFIG.smooth_q8) + Q8_HALF) >> Q8;
        (self.out_q8 >> Q8) as u8
    }
}

/// Maps a random byte to a jitter in `-jitter_max..=jitter_max`, rounded to nearest.
fn jitter(random: u8, jitter_max: u8) -> i32 {
    let noise = i32::from(random) - BYTE_CENTER;
    (noise * (2 * i32::from(jitter_max) + 1) + Q8_HALF) >> Q8
}

/// Advances a Xorshift32 PRNG state: fast, small, and good enough for flicker.
#[must_use]
const fn xorshift32(mut state: u32) -> u32 {
    state ^= state << 13_u32;
    state ^= state >> 17_u32;
    state ^= state << 5_u32;
    state
}

#[cfg(test)]
mod tests {
    //! Behaviour of [`FireState`] under different profiles.

    use core::ops::RangeInclusive;

    use super::{FireState, channel_seed, jitter, xorshift32};
    use crate::config::{FireConfig, FireProfile};

    /// Seed of the first channel in the firmware.
    const SEED: u32 = 0xc0ff_ee01;

    /// Ticks to simulate; long enough for dozens of breath cycles.
    const TICKS: u32 = 100_000;

    /// The baseline configuration.
    #[derive(Clone, Copy)]
    struct Baseline;

    impl FireProfile for Baseline {
        const CONFIG: FireConfig = FireConfig::new();
    }

    /// No jitter and no flares, so the output follows the breathing envelope alone.
    struct Calm;

    impl FireProfile for Calm {
        const CONFIG: FireConfig = FireConfig {
            jitter_max: 0,
            pulse_prob: 0,
            ..FireConfig::new()
        };
    }

    /// Like [`Calm`], but a flare starts on almost every tick.
    struct Flaring;

    impl FireProfile for Flaring {
        const CONFIG: FireConfig = FireConfig {
            pulse_prob: u8::MAX,
            ..<Calm as FireProfile>::CONFIG
        };
    }

    /// Sums the outputs of `ticks` updates.
    fn output_sum<P: FireProfile>(state: &mut FireState<P>, ticks: u32) -> u64 {
        (0..ticks).map(|_| u64::from(state.update())).sum()
    }

    /// Returns an order-sensitive FNV-1a fingerprint of the outputs of `ticks` updates.
    fn output_fingerprint<P: FireProfile>(state: &mut FireState<P>, ticks: u32) -> u64 {
        const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
        const FNV_PRIME: u64 = 0x0100_0000_01b3;
        (0..ticks).fold(FNV_OFFSET_BASIS, |hash, _| {
            (hash ^ u64::from(state.update())).wrapping_mul(FNV_PRIME)
        })
    }

    /// Returns the breathing range of the [`Calm`] profile.
    fn calm_range() -> RangeInclusive<i32> {
        let config = <Calm as FireProfile>::CONFIG;
        i32::from(config.min_intensity)..=i32::from(config.max_intensity)
    }

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
        for _ in 0..TICKS {
            state = xorshift32(state);
            assert_ne!(state, 0, "Xorshift32 left its non-zero cycle");
        }
    }

    /// Jitter spans `-jitter_max..=jitter_max` symmetrically and is zero when disabled.
    #[test]
    fn jitter_spans_configured_amplitude() {
        assert_eq!(jitter(0, 25), -25_i32, "lowest byte maps to -jitter_max");
        assert_eq!(
            jitter(u8::MAX, 25),
            25_i32,
            "highest byte maps to jitter_max"
        );
        assert_eq!(jitter(128, 25), 0_i32, "center byte maps to zero");
        for random in 0..=u8::MAX {
            assert_eq!(jitter(random, 0), 0_i32, "disabled jitter must stay zero");
        }
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

    /// A bank of channels matches simulations created one by one from their channel seeds.
    #[test]
    fn channels_are_seeded_by_index() {
        let mut bank: [FireState<Baseline>; 16] = const { FireState::channels() };
        for (channel, state) in (0_u32..).zip(&mut bank) {
            let mut single = FireState::<Baseline>::new(channel_seed(channel));
            for _ in 0..TICKS {
                assert_eq!(
                    state.update(),
                    single.update(),
                    "channel {channel} diverged"
                );
            }
        }
    }

    /// A zero seed is rejected.
    #[test]
    #[should_panic(expected = "a zero seed")]
    fn rejects_zero_seed() {
        let _state: FireState<Baseline> = FireState::new(0);
    }

    /// The envelope stays within its bounds and reaches both of them.
    #[test]
    fn envelope_stays_within_bounds() {
        let range = calm_range();
        let mut state = FireState::<Calm>::new(SEED);
        let (mut reached_min, mut reached_max) = (false, false);
        for _ in 0..TICKS {
            let base = state.advance_envelope();
            assert!(range.contains(&base), "envelope left {range:?}: {base}");
            reached_min |= base == *range.start();
            reached_max |= base == *range.end();
        }
        assert!(
            reached_min && reached_max,
            "envelope must reach both bounds"
        );
    }

    /// One envelope cycle lasts `breath_period_ms` to within one tick.
    #[test]
    fn envelope_period_matches_breath_period() {
        let config = <Calm as FireProfile>::CONFIG;
        let peak = *calm_range().end();
        let mut state = FireState::<Calm>::new(SEED);
        let mut peaks = (1..=TICKS).filter(|_| state.advance_envelope() == peak);
        let first = peaks.next().expect("envelope reaches its peak");
        let second = peaks.next().expect("envelope reaches its peak twice");
        let period_ms = (second - first) * config.tick_ms;
        assert!(
            period_ms.abs_diff(config.breath_period_ms) <= config.tick_ms,
            "breath period is {period_ms} ms, expected {} ms",
            config.breath_period_ms
        );
    }

    /// Without noise the smoothed output never leaves the envelope range.
    #[test]
    fn calm_output_stays_within_envelope() {
        let range = calm_range();
        let mut state = FireState::<Calm>::new(SEED);
        for _ in 0..TICKS {
            let output = i32::from(state.update());
            assert!(range.contains(&output), "output left {range:?}: {output}");
        }
    }

    /// The same seed reproduces the same flicker, different seeds diverge.
    #[test]
    fn seeds_determine_the_sequence() {
        let mut first = FireState::<Baseline>::new(SEED);
        let mut again = FireState::<Baseline>::new(SEED);
        let mut other = FireState::<Baseline>::new(0x1234_5678);
        let mut diverged = false;
        for _ in 0..TICKS {
            let output = first.update();
            assert_eq!(output, again.update(), "equal seeds must give equal output");
            diverged |= output != other.update();
        }
        assert!(diverged, "different seeds must give different output");
    }

    /// Flares raise the average brightness.
    #[test]
    fn flares_raise_brightness() {
        let calm = output_sum(&mut FireState::<Calm>::new(SEED), TICKS);
        let flaring = output_sum(&mut FireState::<Flaring>::new(SEED), TICKS);
        assert!(
            flaring > calm,
            "flares must brighten the fire: {flaring} <= {calm}"
        );
    }

    /// The baseline flicker is unchanged; update the values only for intended changes to the look.
    #[test]
    fn baseline_output_is_stable() {
        let sum = output_sum(&mut FireState::<Baseline>::new(SEED), TICKS);
        assert_eq!(
            sum, 13_945_072,
            "average brightness of the baseline flicker changed"
        );
        let fingerprint = output_fingerprint(&mut FireState::<Baseline>::new(SEED), TICKS);
        assert_eq!(
            fingerprint, 0x7702_a701_be67_ba17,
            "baseline flicker changed"
        );
    }
}
