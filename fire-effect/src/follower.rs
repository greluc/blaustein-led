// SPDX-FileCopyrightText: 2026 the Blaustein LED contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! A light that follows another channel, such as the red glow of the embers under a flame.

use core::marker::PhantomData;

use crate::fixed_point::{ONE_Q16, Q16, per_tick, permille_of};
use crate::installation::{Installation, InstallationConfig};
use crate::intensity::Intensity;

/// Half of one Q16 unit; adding it before a right shift rounds to nearest.
const Q16_HALF: u32 = 1 << (Q16 - 1);

/// A follower fixed at compile time.
pub trait FollowerProfile {
    /// The parameters. [`Follower`] rejects invalid ones at compile time.
    const CONFIG: FollowerConfig;
}

/// Parameters of a light that follows the intensity of another channel, heavily smoothed.
///
/// The follower smooths its source with a first-order low-pass and outputs
/// `floor + smoothed source * gain_permille / 1000`, saturating at full scale. Its output stays
/// within `floor..=floor + source peak * gain_permille / 1000` and never exceeds full scale.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FollowerConfig {
    /// Output while the source is dark; the lower bound of the follower.
    pub floor: Intensity,
    /// Share of the smoothed source added to `floor`, in thousandths; above 1000 amplifies.
    pub gain_permille: u16,
    /// Time constant of the smoothing in milliseconds: after it, the follower has covered about
    /// 63 % of a step of its source. 0 follows without smoothing.
    pub smoothing_ms: u32,
}

impl FollowerConfig {
    /// Checks that the parameters work at the tick of `installation`, and checks the
    /// installation.
    ///
    /// # Panics
    ///
    /// Panics with a description of the first violated constraint. Evaluated in a const context,
    /// that is a build error.
    pub const fn validate(&self, installation: &InstallationConfig) {
        installation.validate();
        assert!(
            self.coefficient_q16(installation.tick_ms) > 0,
            "smoothing_ms is too long for the tick: the smoothing step truncates to zero and the \
             follower would never move"
        );
    }

    /// Returns the share of the remaining distance to the source covered per tick, in Q16.
    ///
    /// The backward-Euler step of a first-order low-pass, `tick / (smoothing + tick)`, which
    /// stays within `0..=1` for every time constant.
    const fn coefficient_q16(self, tick_ms: u32) -> u64 {
        per_tick(ONE_Q16, tick_ms, self.smoothing_ms.saturating_add(tick_ms))
    }
}

/// State of a light that follows another channel.
///
/// Feed it the current intensity of its source once per tick with [`Follower::update`]. It
/// starts as if its source had been dark, so it rises from `floor` like embers heating up.
#[derive(Clone, Copy, Debug)]
pub struct Follower<S, F> {
    /// Smoothed intensity of the source in Q16.16, within `0..=u16::MAX << 16`.
    smoothed: u32,
    /// Binds the state to its compile-time installation and parameters.
    profile: PhantomData<(S, F)>,
}

impl<S: Installation, F: FollowerProfile> Follower<S, F> {
    /// Smoothing coefficient per tick in Q16; an invalid configuration fails the build where it
    /// is used.
    #[expect(
        clippy::cast_possible_wrap,
        reason = "`tick / (smoothing + tick)` is at most one, 65536 in Q16, which fits in i64"
    )]
    const COEFFICIENT_Q16: i64 = {
        F::CONFIG.validate(&S::CONFIG);
        F::CONFIG.coefficient_q16(S::CONFIG.tick_ms) as i64
    };

    /// Creates a follower whose source has been dark.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            smoothed: 0,
            profile: PhantomData,
        }
    }

    /// Advances the follower by one tick towards the current intensity of its source and
    /// returns its own intensity.
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "each step covers at most the distance to the source, so `smoothed` stays \
                  between its old value and `source << 16`, within u32"
    )]
    pub fn update(&mut self, source: Intensity) -> Intensity {
        let target = i64::from(source.get()) << Q16;
        let current = i64::from(self.smoothed);
        // Flooring a share of at most one never overshoots the target in either direction.
        self.smoothed = (current + (((target - current) * Self::COEFFICIENT_Q16) >> Q16)) as u32;
        let level = ((self.smoothed + Q16_HALF) >> Q16) as u16;
        let lifted = u32::from(F::CONFIG.floor.get()) + permille_of(level, F::CONFIG.gain_permille);
        Intensity::new(u16::try_from(lifted).unwrap_or(u16::MAX))
    }
}

impl<S: Installation, F: FollowerProfile> Default for Follower<S, F> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    //! Smoothing, bounds and tracking of [`Follower`].

    use super::{Follower, FollowerConfig, FollowerProfile};
    use crate::installation::InstallationConfig;
    use crate::intensity::Intensity;
    use crate::test_support::{HearthGlow, Site, follow, mean, mean_step_millis};

    /// Seed of the source channel.
    const SEED: u32 = 0xc0ff_ee01;

    /// Ticks of a long run: 50 minutes at a 15 ms tick.
    const LONG_RUN: usize = 200_000;

    /// Largest deviation of the glow's mean from its floor plus the gained mean of its flame:
    /// 0.15 % of full scale, for the warm-up from dark and the rounding.
    const MAX_LEVEL_DEVIATION: u64 = 100;

    /// Unity gain, no floor and a 1.5 s time constant, for exact step responses.
    struct Unity;

    impl FollowerProfile for Unity {
        const CONFIG: FollowerConfig = FollowerConfig {
            floor: Intensity::OFF,
            gain_permille: 1_000,
            smoothing_ms: 1_500,
        };
    }

    /// A gain of 4 that drives the follower into saturation.
    struct Loud;

    impl FollowerProfile for Loud {
        const CONFIG: FollowerConfig = FollowerConfig {
            floor: Intensity::from_permille(100),
            gain_permille: 4_000,
            smoothing_ms: 300,
        };
    }

    /// After one time constant a step is covered by about 63 %, and the follower settles on the
    /// source exactly.
    #[test]
    fn step_response_matches_the_time_constant() {
        let mut follower = Follower::<Site, Unity>::new();
        let time_constant = 1_500_u32 / 15;
        let mut output = Intensity::OFF;
        for _ in 0..time_constant {
            output = follower.update(Intensity::FULL);
        }
        // 1 - 1/e is 0.632 of full scale.
        assert!(
            (40_800..=42_000).contains(&output.get()),
            "after one time constant the follower reached {}",
            output.get()
        );
        for _ in 0..50 * time_constant {
            output = follower.update(Intensity::FULL);
        }
        assert_eq!(
            output,
            Intensity::FULL,
            "the follower must settle on its source"
        );
        for _ in 0..50 * time_constant {
            output = follower.update(Intensity::OFF);
        }
        assert_eq!(
            output,
            Intensity::OFF,
            "the follower must settle on a dark source"
        );
    }

    /// The glow stays within its floor and its floor plus the gained source peak.
    #[test]
    fn glow_stays_within_its_bounds() {
        let config = FollowerConfig::HEARTH_GLOW;
        let (source, glow) = follow::<HearthGlow>(SEED, LONG_RUN);
        let source_peak = source.iter().max().copied().expect("the run is not empty");
        let ceiling = u32::from(config.floor.get())
            + u32::from(source_peak) * u32::from(config.gain_permille) / 1_000;
        for (tick, &output) in glow.iter().enumerate() {
            assert!(
                (config.floor.get()..).contains(&output) && u32::from(output) <= ceiling,
                "tick {tick}: glow {output} left {}..={ceiling}",
                config.floor.get()
            );
        }
    }

    /// The glow moves far less from tick to tick than its flame.
    #[test]
    fn glow_is_smoother_than_its_source() {
        let (source, glow) = follow::<HearthGlow>(SEED, LONG_RUN);
        let source_step = mean_step_millis(&source);
        let glow_step = mean_step_millis(&glow);
        assert!(
            glow_step * 20 < source_step,
            "the glow moves {glow_step}/1000 per tick, more than a twentieth of its flame's \
             {source_step}/1000"
        );
    }

    /// On average the glow sits at its floor plus the gained average of its flame.
    #[test]
    fn glow_tracks_the_level_of_its_source() {
        let config = FollowerConfig::HEARTH_GLOW;
        let (source, glow) = follow::<HearthGlow>(SEED, LONG_RUN);
        let expected =
            i64::from(config.floor.get()) + mean(&source) * i64::from(config.gain_permille) / 1_000;
        let deviation = mean(&glow).abs_diff(expected);
        assert!(
            deviation < MAX_LEVEL_DEVIATION,
            "the glow averages {}, expected {expected}",
            mean(&glow)
        );
    }

    /// A large gain saturates at full scale instead of wrapping.
    #[test]
    fn loud_follower_saturates() {
        let (_, loud) = follow::<Loud>(SEED, LONG_RUN);
        assert!(
            loud.contains(&u16::MAX),
            "a gain of 4 on a bright flame must reach full scale"
        );
        let floor = <Loud as FollowerProfile>::CONFIG.floor.get();
        assert!(
            loud.iter().all(|&output| output >= floor),
            "the loud follower fell below its floor"
        );
    }

    /// A time constant so long that the follower would never move is rejected.
    #[test]
    #[should_panic(expected = "smoothing_ms is too long for the tick")]
    fn rejects_frozen_follower() {
        FollowerConfig {
            smoothing_ms: 15 * 65_536,
            ..FollowerConfig::HEARTH_GLOW
        }
        .validate(&InstallationConfig::new());
    }

    /// A time constant of zero follows the source unsmoothed.
    #[test]
    fn zero_smoothing_follows_directly() {
        let config = FollowerConfig {
            smoothing_ms: 0,
            ..FollowerConfig::HEARTH_GLOW
        };
        config.validate(&InstallationConfig::new());
        assert_eq!(
            config.coefficient_q16(15),
            1 << 16_u32,
            "no smoothing covers the whole step"
        );
    }
}
