// SPDX-FileCopyrightText: 2026 the Blaustein LED contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Flicker simulation for one LED channel.

use core::marker::PhantomData;

use crate::config::FireProfile;
use crate::fixed_point::Q16;
use crate::installation::Installation;
use crate::intensity::Intensity;
use crate::noise::ValueNoise;
use crate::rng::{Rng, channel_seed};
use crate::tuning::Tuning;

/// State of the fire simulation for one LED channel.
///
/// The installation `S` supplies the tick and the profile `P` the look. Both come from types
/// rather than fields, so every derived constant folds into an immediate. The state holds no
/// shared phase: a channel starts at a random point of every layer, drawn from its seed.
#[derive(Clone, Copy, Debug)]
pub struct FireState<S, P> {
    /// Source of every random choice of this channel.
    rng: Rng,
    /// The slow swell.
    swell: ValueNoise,
    /// The faster sway.
    sway: ValueNoise,
    /// The fast flicker, which also shapes the gusts.
    flicker: ValueNoise,
    /// Envelope of the current gust, fading from `u16::MAX` to 0.
    gust: u16,
    /// Binds the state to its compile-time installation and look.
    profile: PhantomData<(S, P)>,
}

impl<S: Installation, P: FireProfile> FireState<S, P> {
    /// Per-tick constants; an invalid installation or look fails the build where it is used.
    const TUNING: Tuning = Tuning::new(&S::CONFIG, &P::CONFIG);

    /// Creates a channel simulation from a non-zero PRNG seed.
    ///
    /// Usable at compile time and at run time, so seeds may come from [`channel_seed`] or from a
    /// hardware random source.
    ///
    /// # Panics
    ///
    /// Panics if `seed` is zero, the fixed point of Xorshift32. Evaluate it in a `const` block to
    /// turn that into a build error.
    #[must_use]
    pub const fn new(seed: u32) -> Self {
        let mut rng = Rng::new(seed);
        let swell = ValueNoise::new(&mut rng, Self::TUNING.swell.rates);
        let sway = ValueNoise::new(&mut rng, Self::TUNING.sway.rates);
        let flicker = ValueNoise::new(&mut rng, Self::TUNING.flicker.rates);
        Self {
            rng,
            swell,
            sway,
            flicker,
            gust: 0,
            profile: PhantomData,
        }
    }

    /// Creates `N` channel simulations, seeded with [`channel_seed`] of their index.
    ///
    /// Evaluate it in a `const` block, so the whole bank is built at compile time.
    #[must_use]
    pub const fn channels<const N: usize>() -> [Self; N]
    where
        S: Copy,
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
    ///
    /// The result lies within `peak - total depth ..= peak` of the profile's look.
    #[expect(
        clippy::cast_possible_truncation,
        reason = "the pulls add up to at most `peak`, so the difference lies in 0..=peak, within \
                  u16"
    )]
    pub fn update(&mut self) -> Intensity {
        let tuning = Self::TUNING;
        let swell = self.swell.advance(&mut self.rng, tuning.swell.rates);
        let sway = self.sway.advance(&mut self.rng, tuning.sway.rates);
        let flicker = self.flicker.advance(&mut self.rng, tuning.flicker.rates);
        let gust = self.advance_gust();
        // Every pull is below its depth and the depths add up to at most `peak`, so this never
        // underflows.
        let dip = pull(tuning.swell.depth, swell)
            + pull(tuning.sway.depth, sway)
            + pull(tuning.flicker.depth, flicker)
            + pull(pull(tuning.gust_depth, gust), flicker);
        Intensity::new((tuning.peak - dip) as u16)
    }

    /// Starts a gust with the configured chance, otherwise fades the current one, and returns
    /// the envelope.
    fn advance_gust(&mut self) -> u16 {
        let [roll, _] = self.rng.next_pair();
        self.gust = if u32::from(roll) < Self::TUNING.gust_chance {
            u16::MAX
        } else {
            self.gust.saturating_sub(Self::TUNING.gust_fade)
        };
        self.gust
    }
}

/// Returns the pull of a motion at `level`, a Q16 share of its largest pull `depth`, truncated.
///
/// `depth` is at most `u16::MAX`, so the product fits in `u32`, and the result is below `depth`
/// unless both are zero.
const fn pull(depth: u32, level: u16) -> u32 {
    (depth * level as u32) >> Q16
}

#[cfg(test)]
mod tests {
    //! Behaviour of [`FireState`]: seeding, bounds, independence of channels and the pinned look.

    use super::FireState;
    use crate::config::{FireConfig, FireProfile, Gust, Layer};
    use crate::installation::Installation;
    use crate::intensity::Intensity;
    use crate::rng::channel_seed;
    use crate::test_support::{
        Ambient, Candle, Ember, Hearth, HearthGlow, Site, Torch, autocorrelation_permille,
        correlation_permille, fingerprint, follow, record,
    };

    /// Seed of the first channel in the firmware.
    const SEED: u32 = 0xc0ff_ee01;

    /// Ticks of a long run: 50 minutes at a 15 ms tick.
    const LONG_RUN: usize = 200_000;

    /// Channels of a full bank.
    const BANK: usize = 16;

    /// Largest correlation between any two channels of a bank over a long run, in permille.
    ///
    /// The slowest roles glide for up to 10 s, so even independent channels correlate by chance
    /// by up to about 0.1 over 50 minutes; channels in lockstep correlate by 0.65 or more.
    const MAX_CORRELATION_PERMILLE: i64 = 150;

    /// Largest mean of the absolute correlations of all channel pairs of a bank, in permille.
    const MAX_MEAN_CORRELATION_PERMILLE: i64 = 50;

    /// Ticks of the pinned runs.
    const PINNED_RUN: usize = 100_000;

    /// Distance from an end of the scale that counts as reaching it: one sixteenth of full scale.
    const EDGE: u16 = 4_096;

    /// Largest autocorrelation beyond the longest glide, in permille.
    const MAX_AUTOCORRELATION_PERMILLE: i64 = 150;

    /// Every depth at its largest, at full scale: the extreme of the bounds.
    #[derive(Clone, Copy)]
    struct Extreme;

    impl FireProfile for Extreme {
        const CONFIG: FireConfig = FireConfig {
            peak: Intensity::FULL,
            swell: Layer {
                depth_permille: 250,
                min_ms: 30,
                max_ms: 60,
            },
            sway: Layer {
                depth_permille: 250,
                min_ms: 30,
                max_ms: 30,
            },
            flicker: Layer {
                depth_permille: 250,
                min_ms: 30,
                max_ms: 45,
            },
            gust: Gust {
                depth_permille: 250,
                mean_interval_ms: 15,
                duration_ms: 15,
            },
        };
    }

    /// The hearth look at the faintest possible peak.
    #[derive(Clone, Copy)]
    struct Faint;

    impl FireProfile for Faint {
        const CONFIG: FireConfig = FireConfig {
            peak: Intensity::new(1),
            ..FireConfig::HEARTH
        };
    }

    /// The hearth look at a quarter of its peak.
    #[derive(Clone, Copy)]
    struct DimHearth;

    impl FireProfile for DimHearth {
        const CONFIG: FireConfig = FireConfig {
            peak: Intensity::new(FireConfig::HEARTH.peak.get() / 4),
            ..FireConfig::HEARTH
        };
    }

    /// Returns the outputs of a bank of `BANK` channels over `ticks` ticks, one series each.
    fn record_bank<P: FireProfile + Copy>(ticks: usize) -> Vec<Vec<u16>> {
        let mut bank: [FireState<Site, P>; BANK] = FireState::channels();
        let mut series: Vec<Vec<u16>> = (0..BANK).map(|_| Vec::with_capacity(ticks)).collect();
        for _ in 0..ticks {
            for (state, outputs) in bank.iter_mut().zip(&mut series) {
                outputs.push(state.update().get());
            }
        }
        series
    }

    /// Asserts that no two channels of a bank of `P` move together, and that the channels are
    /// uncorrelated on average.
    fn assert_channels_independent<P: FireProfile + Copy>(name: &str) {
        let series = record_bank::<P>(LONG_RUN);
        let (mut total, mut pairs) = (0_i64, 0_i64);
        for (first, first_series) in series.iter().enumerate() {
            for (second, second_series) in series.iter().enumerate().skip(first + 1) {
                let correlation = correlation_permille(first_series, second_series);
                assert!(
                    correlation.abs() < MAX_CORRELATION_PERMILLE,
                    "{name}: channels {first} and {second} correlate by {correlation} permille"
                );
                total += correlation.abs();
                pairs += 1;
            }
        }
        let mean = total / pairs;
        assert!(
            mean < MAX_MEAN_CORRELATION_PERMILLE,
            "{name}: channel pairs correlate by {mean} permille on average"
        );
    }

    /// Asserts that a channel of `P` has no period: its autocorrelation stays low at every lag
    /// from twice its longest swell glide to 30 s, in steps of 5 ticks.
    fn assert_no_period<P: FireProfile>(name: &str) {
        let tick_ms = <Site as Installation>::CONFIG.tick_ms;
        let shortest = (2 * P::CONFIG.swell.max_ms / tick_ms) as usize;
        let longest = (30_000 / tick_ms) as usize;
        let series = record::<P>(SEED, LONG_RUN);
        for lag in (shortest..=longest).step_by(5) {
            let correlation = autocorrelation_permille(&series, lag);
            assert!(
                correlation.abs() < MAX_AUTOCORRELATION_PERMILLE,
                "{name}: autocorrelation at {} ms is {correlation} permille",
                lag * tick_ms as usize
            );
        }
    }

    /// A bank of channels matches simulations created one by one from their channel seeds.
    #[test]
    fn channels_are_seeded_by_index() {
        let mut bank: [FireState<Site, Hearth>; BANK] = const { FireState::channels() };
        for (channel, state) in (0_u32..).zip(&mut bank) {
            let mut single = FireState::<Site, Hearth>::new(channel_seed(channel));
            for _ in 0..10_000_u32 {
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
        let _state: FireState<Site, Hearth> = FireState::new(0);
    }

    /// The same seed reproduces the same flicker, different seeds diverge.
    #[test]
    fn seeds_determine_the_sequence() {
        let first = record::<Hearth>(SEED, 10_000);
        assert_eq!(
            first,
            record::<Hearth>(SEED, 10_000),
            "equal seeds must give equal output"
        );
        assert_ne!(
            first,
            record::<Hearth>(0x1234_5678, 10_000),
            "different seeds must give different output"
        );
    }

    /// With depths of 1000 permille at full scale, the output reaches close to both ends of the
    /// scale without overflowing.
    #[test]
    fn extreme_depths_stay_within_the_scale() {
        let outputs = record::<Extreme>(SEED, LONG_RUN);
        let lowest = outputs.iter().min().copied();
        let highest = outputs.iter().max().copied();
        assert!(
            lowest < Some(EDGE),
            "the extreme look never got dark: {lowest:?}"
        );
        assert!(
            highest > Some(u16::MAX - EDGE),
            "the extreme look never got bright: {highest:?}"
        );
    }

    /// The faintest peak of one step is never exceeded.
    #[test]
    fn faint_peak_is_never_exceeded() {
        let outputs = record::<Faint>(SEED, LONG_RUN);
        assert!(
            outputs.iter().all(|&output| output <= 1),
            "a peak of 1 was exceeded"
        );
    }

    /// Changing the peak scales the light without changing its motion: the same seed gives the
    /// same flicker, scaled to within the rounding of the four pulls.
    #[test]
    fn peak_scales_without_changing_the_motion() {
        let bright = record::<Hearth>(SEED, LONG_RUN);
        let dim = record::<DimHearth>(SEED, LONG_RUN);
        let bright_peak = u64::from(FireConfig::HEARTH.peak.get());
        let dim_peak = u64::from(<DimHearth as FireProfile>::CONFIG.peak.get());
        // Each of the four pulls and the four depths rounds by less than one step.
        let tolerance = 8 * (bright_peak + dim_peak);
        for (tick, (&high, &low)) in bright.iter().zip(&dim).enumerate() {
            let deviation = (u64::from(high) * dim_peak).abs_diff(u64::from(low) * bright_peak);
            assert!(
                deviation <= tolerance,
                "tick {tick}: {high} at peak {bright_peak} and {low} at peak {dim_peak} differ in \
                 shape"
            );
        }
    }

    /// No two channels of a bank move together, for any role.
    #[test]
    fn channels_are_independent() {
        assert_channels_independent::<Hearth>("hearth");
        assert_channels_independent::<Torch>("torch");
        assert_channels_independent::<Candle>("candle");
        assert_channels_independent::<Ember>("ember");
        assert_channels_independent::<Ambient>("ambient");
    }

    /// No role has a dominant period.
    #[test]
    fn roles_have_no_period() {
        assert_no_period::<Hearth>("hearth");
        assert_no_period::<Torch>("torch");
        assert_no_period::<Candle>("candle");
        assert_no_period::<Ember>("ember");
        assert_no_period::<Ambient>("ambient");
    }

    /// The look of every preset is unchanged; update the values only for intended changes to the
    /// look.
    ///
    /// Each preset is pinned by the sum of its outputs (its average brightness) and an
    /// order-sensitive fingerprint (its motion); the hearth glow follows the hearth.
    #[test]
    fn baseline_output_is_stable() {
        let pins = [
            (
                "hearth",
                record::<Hearth>(SEED, PINNED_RUN),
                3_735_887_954,
                0x91c1_057e_43f0_56a5,
            ),
            (
                "torch",
                record::<Torch>(SEED, PINNED_RUN),
                3_777_922_164,
                0x00c2_0dac_f60e_4f21,
            ),
            (
                "candle",
                record::<Candle>(SEED, PINNED_RUN),
                3_127_402_344,
                0x558a_812e_103a_c6b9,
            ),
            (
                "ember",
                record::<Ember>(SEED, PINNED_RUN),
                2_333_883_063,
                0x9b3f_25ce_5636_2a50,
            ),
            (
                "ambient",
                record::<Ambient>(SEED, PINNED_RUN),
                1_597_073_837,
                0xf934_c33e_ecbf_c38a,
            ),
            (
                "hearth glow",
                follow::<HearthGlow>(SEED, PINNED_RUN).1,
                2_631_991_342,
                0x2e32_8f03_cf8e_9123,
            ),
        ];
        for (name, outputs, sum, print) in pins {
            let actual_sum: u64 = outputs.iter().map(|&output| u64::from(output)).sum();
            assert_eq!(actual_sum, sum, "average brightness of the {name} changed");
            assert_eq!(fingerprint(&outputs), print, "the {name} flicker changed");
        }
    }
}
