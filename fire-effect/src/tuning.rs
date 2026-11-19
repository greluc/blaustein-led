// SPDX-FileCopyrightText: 2026 the Blaustein LED contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Per-tick constants of a role, derived from its look and the tick of the installation.

use crate::config::{FireConfig, Layer};
use crate::fixed_point::{ONE_Q16, per_tick, permille_of};
use crate::installation::InstallationConfig;
use crate::noise::Rates;

/// Per-tick constants of one layer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LayerTuning {
    /// Largest pull below the peak in intensity units.
    pub depth: u32,
    /// Glide rates; [`Rates::STILL`] for a layer that does not move.
    pub rates: Rates,
}

/// Per-tick constants of a role.
///
/// The depths are shares of `peak`, truncated, so they add up to at most `peak` whenever the
/// depths of the look add up to at most 1000 permille.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Tuning {
    /// Highest output intensity.
    pub peak: u32,
    /// The slow swell.
    pub swell: LayerTuning,
    /// The faster sway.
    pub sway: LayerTuning,
    /// The fast flicker, which also paces the gusts.
    pub flicker: LayerTuning,
    /// Largest extra pull of the flicker at the start of a gust in intensity units.
    pub gust_depth: u32,
    /// Chance per tick that a gust starts, in 65536ths.
    pub gust_chance: u32,
    /// Fade of the gust envelope per tick, in 65535ths of a full gust.
    pub gust_fade: u16,
}

impl Tuning {
    /// Derives the per-tick constants of `look` at the tick of `installation`.
    ///
    /// # Panics
    ///
    /// Panics if the look or the installation is invalid, see [`FireConfig::validate`]. Evaluated
    /// in a const context, that is a build error.
    #[must_use]
    #[expect(
        clippy::cast_possible_truncation,
        reason = "validation keeps the gust chance within 65536 and the fade within u16::MAX"
    )]
    pub const fn new(installation: &InstallationConfig, look: &FireConfig) -> Self {
        look.validate(installation);
        let tick_ms = installation.tick_ms;
        let peak = look.peak.get();
        let gust = &look.gust;
        let (gust_chance, gust_fade) = if gust.is_on() {
            (
                per_tick(ONE_Q16, tick_ms, gust.mean_interval_ms) as u32,
                per_tick(u16::MAX as u32, tick_ms, gust.duration_ms) as u16,
            )
        } else {
            (0, 0)
        };
        Self {
            peak: peak as u32,
            swell: LayerTuning::new(&look.swell, peak, tick_ms, look.swell.is_on()),
            sway: LayerTuning::new(&look.sway, peak, tick_ms, look.sway.is_on()),
            flicker: LayerTuning::new(&look.flicker, peak, tick_ms, look.flicker_moves()),
            gust_depth: permille_of(peak, gust.depth_permille),
            gust_chance,
            gust_fade,
        }
    }
}

impl LayerTuning {
    /// Derives the constants of `layer` for a role with `peak`; a layer that does not move gets
    /// [`Rates::STILL`].
    const fn new(layer: &Layer, peak: u16, tick_ms: u32, moves: bool) -> Self {
        Self {
            depth: permille_of(peak, layer.depth_permille),
            rates: if moves {
                Rates::from_interval(layer.min_ms, layer.max_ms, tick_ms)
            } else {
                Rates::STILL
            },
        }
    }
}

#[cfg(test)]
mod tests {
    //! Derivation of [`Tuning`].

    use super::Tuning;
    use crate::config::{FireConfig, Gust, Layer};
    use crate::installation::InstallationConfig;
    use crate::intensity::Intensity;
    use crate::noise::Rates;

    /// A look with round numbers.
    const LOOK: FireConfig = FireConfig {
        peak: Intensity::new(60_000),
        swell: Layer {
            depth_permille: 400,
            min_ms: 1_000,
            max_ms: 3_000,
        },
        sway: Layer::OFF,
        flicker: Layer {
            depth_permille: 100,
            min_ms: 45,
            max_ms: 120,
        },
        gust: Gust {
            depth_permille: 500,
            mean_interval_ms: 6_000,
            duration_ms: 600,
        },
    };

    /// Depths, rates, chance and fade match their closed forms at a 15 ms tick.
    #[test]
    fn constants_match_their_closed_forms() {
        let tuning = Tuning::new(&InstallationConfig::new(), &LOOK);
        assert_eq!(tuning.peak, 60_000, "peak is the raw intensity");
        assert_eq!(tuning.swell.depth, 24_000, "400 permille of 60000");
        assert_eq!(
            tuning.swell.rates,
            Rates::from_interval(1_000, 3_000, 15),
            "swell rates follow the glide times"
        );
        assert_eq!(tuning.sway.depth, 0, "a switched-off layer has no depth");
        assert_eq!(
            tuning.sway.rates,
            Rates::STILL,
            "a switched-off layer stands still"
        );
        assert_eq!(tuning.flicker.depth, 6_000, "100 permille of 60000");
        assert_eq!(tuning.gust_depth, 30_000, "500 permille of 60000");
        assert_eq!(tuning.gust_chance, 163, "65536 * 15 / 6000, truncated");
        assert_eq!(tuning.gust_fade, 1_638, "65535 * 15 / 600, truncated");
        let total =
            tuning.swell.depth + tuning.sway.depth + tuning.flicker.depth + tuning.gust_depth;
        assert_eq!(
            total, tuning.peak,
            "depths of 1000 permille add up to the peak"
        );
    }

    /// Gusts keep the flicker moving when the flicker itself is off.
    #[test]
    fn gusts_move_the_flicker() {
        let look = FireConfig {
            flicker: Layer {
                depth_permille: 0,
                ..LOOK.flicker
            },
            ..LOOK
        };
        let tuning = Tuning::new(&InstallationConfig::new(), &look);
        assert_eq!(
            tuning.flicker.depth, 0,
            "the flicker adds no depth of its own"
        );
        assert_eq!(
            tuning.flicker.rates,
            Rates::from_interval(45, 120, 15),
            "the flicker still moves for the gusts"
        );
    }

    /// Without gusts there is no chance and no fade.
    #[test]
    fn switched_off_gusts_never_start() {
        let look = FireConfig {
            gust: Gust::OFF,
            ..LOOK
        };
        let tuning = Tuning::new(&InstallationConfig::new(), &look);
        assert_eq!(tuning.gust_chance, 0, "gusts never start");
        assert_eq!(tuning.gust_depth, 0, "gusts pull nothing");
    }
}
