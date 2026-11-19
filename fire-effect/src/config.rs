// SPDX-FileCopyrightText: 2026 the Blaustein LED contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The look of a light role and its validation.

use crate::fixed_point::{ONE_Q16, PERMILLE, per_tick};
use crate::installation::InstallationConfig;
use crate::intensity::Intensity;

/// A light role fixed at compile time.
pub trait FireProfile {
    /// The look. [`FireState`](crate::FireState) rejects an invalid one at compile time.
    const CONFIG: FireConfig;
}

/// The look of a light role: how bright it gets at most and how it moves.
///
/// The output starts at `peak` and four motions pull it down: three [`Layer`]s of value noise
/// (`swell`, `sway`, `flicker`) and occasional [`Gust`]s that deepen the flicker. Every depth is
/// given in thousandths of `peak`, so:
///
/// - the output never exceeds `peak`, the single bound of the brightness;
/// - it never falls below `peak * (1000 - total depth) / 1000`, where the total depth is the sum of
///   the four depths, at most 1000;
/// - changing `peak` makes the light brighter or dimmer without changing how it moves.
///
/// Start from a preset ([`FireConfig::HEARTH`], [`FireConfig::TORCH`], [`FireConfig::CANDLE`],
/// [`FireConfig::EMBER`], [`FireConfig::AMBIENT`]) and override fields with struct update syntax.
/// Times are in milliseconds and independent of the tick of the [`InstallationConfig`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FireConfig {
    /// Highest output intensity; the light never gets brighter. Must be greater than 0.
    pub peak: Intensity,
    /// Slow swell of the whole flame.
    pub swell: Layer,
    /// Faster sway on top of the swell.
    pub sway: Layer,
    /// Fast flicker; its timing also paces the flutter of gusts.
    pub flicker: Layer,
    /// Occasional gusts that deepen the flicker for a moment.
    pub gust: Gust,
}

/// One layer of smoothly interpolated value noise.
///
/// The layer glides from one random level to the next. Each glide takes a random time of about
/// `min_ms..=max_ms` and eases in and out, so the layer has no fixed period and no corners.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Layer {
    /// Largest pull below the peak in thousandths of the peak; 0 switches the layer off.
    pub depth_permille: u16,
    /// Shortest glide in milliseconds, at least two ticks.
    pub min_ms: u32,
    /// Longest glide in milliseconds, at least `min_ms`.
    pub max_ms: u32,
}

/// Occasional gusts that make the flicker deeper for a moment.
///
/// A gust starts at random, on average every `mean_interval_ms`, deepens the flicker by up to
/// `depth_permille` and fades out linearly within `duration_ms`. It follows the motion of the
/// `flicker` layer, so a gust looks like a flutter rather than a jump.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Gust {
    /// Largest extra pull at the start of a gust in thousandths of the peak; 0 switches gusts
    /// off.
    pub depth_permille: u16,
    /// Mean time between two gusts in milliseconds, at least one tick.
    pub mean_interval_ms: u32,
    /// Time in milliseconds in which a gust fades out, at least one tick.
    pub duration_ms: u32,
}

/// Validation messages of one layer, named after its field.
struct LayerMessages {
    /// `min_ms` spans less than two ticks.
    too_fast: &'static str,
    /// `max_ms` is shorter than `min_ms`.
    inverted: &'static str,
    /// `max_ms` is so long that the layer would stand still.
    too_slow: &'static str,
}

/// Builds the [`LayerMessages`] of the layer in field `$name`.
macro_rules! layer_messages {
    ($name:literal) => {
        LayerMessages {
            too_fast: concat!(
                $name,
                ".min_ms must span at least two ticks, or the layer jumps instead of gliding"
            ),
            inverted: concat!($name, ".max_ms must be at least ", $name, ".min_ms"),
            too_slow: concat!(
                $name,
                ".max_ms is too long for the tick: the glide rate truncates to zero and the layer \
                 would stand still"
            ),
        }
    };
}

impl LayerMessages {
    /// Messages of the `flicker` layer.
    const FLICKER: Self = layer_messages!("flicker");
    /// Messages of the `sway` layer.
    const SWAY: Self = layer_messages!("sway");
    /// Messages of the `swell` layer.
    const SWELL: Self = layer_messages!("swell");
}

impl FireConfig {
    /// Checks that the look works at the tick of `installation`, and checks the installation.
    ///
    /// # Panics
    ///
    /// Panics with a description of the first violated constraint. Evaluated in a const context,
    /// that is a build error.
    pub const fn validate(&self, installation: &InstallationConfig) {
        installation.validate();
        assert!(
            self.peak.get() > 0,
            "peak must be greater than 0, or the light stays dark"
        );
        assert!(
            self.total_depth_permille() <= PERMILLE,
            "the depths of swell, sway, flicker and gust must add up to at most 1000 permille, or \
             the light could be pulled below dark"
        );
        let tick_ms = installation.tick_ms;
        if self.swell.is_on() {
            self.swell.validate(tick_ms, &LayerMessages::SWELL);
        }
        if self.sway.is_on() {
            self.sway.validate(tick_ms, &LayerMessages::SWAY);
        }
        if self.flicker_moves() {
            self.flicker.validate(tick_ms, &LayerMessages::FLICKER);
        }
        if self.gust.is_on() {
            self.gust.validate(tick_ms);
        }
    }

    /// Returns whether the flicker layer has to move: for its own depth or for gusts.
    #[must_use]
    pub(crate) const fn flicker_moves(&self) -> bool {
        self.flicker.is_on() || self.gust.is_on()
    }

    /// Returns the sum of all depths in thousandths of the peak.
    #[must_use]
    pub(crate) const fn total_depth_permille(&self) -> u32 {
        self.swell.depth_permille as u32
            + self.sway.depth_permille as u32
            + self.flicker.depth_permille as u32
            + self.gust.depth_permille as u32
    }
}

impl Layer {
    /// A switched-off layer.
    pub const OFF: Self = Self {
        depth_permille: 0,
        min_ms: 0,
        max_ms: 0,
    };

    /// Returns whether the layer pulls the light at all.
    #[must_use]
    pub(crate) const fn is_on(&self) -> bool {
        self.depth_permille > 0
    }

    /// Checks the glide times at a tick of `tick_ms`.
    const fn validate(&self, tick_ms: u32, messages: &LayerMessages) {
        // Halving `min_ms` instead of doubling `tick_ms` cannot overflow.
        assert!(self.min_ms / 2 >= tick_ms, "{}", messages.too_fast);
        assert!(self.max_ms >= self.min_ms, "{}", messages.inverted);
        assert!(
            per_tick(ONE_Q16, tick_ms, self.max_ms) > 0,
            "{}",
            messages.too_slow
        );
    }
}

impl Gust {
    /// Gusts switched off.
    pub const OFF: Self = Self {
        depth_permille: 0,
        mean_interval_ms: 0,
        duration_ms: 0,
    };

    /// Returns whether gusts happen at all.
    #[must_use]
    pub(crate) const fn is_on(&self) -> bool {
        self.depth_permille > 0
    }

    /// Checks the gust timing at a tick of `tick_ms`.
    const fn validate(&self, tick_ms: u32) {
        assert!(
            self.mean_interval_ms >= tick_ms,
            "gust.mean_interval_ms must span at least one tick"
        );
        assert!(
            per_tick(ONE_Q16, tick_ms, self.mean_interval_ms) > 0,
            "gust.mean_interval_ms is too long for the tick: the chance per tick truncates to \
             zero and gusts would never come"
        );
        assert!(
            self.duration_ms >= tick_ms,
            "gust.duration_ms must span at least one tick"
        );
        assert!(
            per_tick(u16::MAX as u32, tick_ms, self.duration_ms) > 0,
            "gust.duration_ms is too long for the tick: the fade per tick truncates to zero and a \
             gust would never end"
        );
    }
}

#[cfg(test)]
mod tests {
    //! Validation of [`FireConfig`].

    use super::{FireConfig, Gust, Layer};
    use crate::installation::InstallationConfig;
    use crate::intensity::Intensity;

    /// The baseline installation with a 15 ms tick.
    const INSTALLATION: InstallationConfig = InstallationConfig::new();

    /// Every preset is valid at every tick from 1 to 20 ms.
    #[test]
    fn presets_are_valid_up_to_20_ms() {
        for tick_ms in 1..=20 {
            let installation = InstallationConfig {
                tick_ms,
                ..INSTALLATION
            };
            for preset in [
                FireConfig::HEARTH,
                FireConfig::TORCH,
                FireConfig::CANDLE,
                FireConfig::EMBER,
                FireConfig::AMBIENT,
            ] {
                preset.validate(&installation);
            }
        }
    }

    /// Switched-off layers and gusts need no timing.
    #[test]
    fn switched_off_motions_need_no_timing() {
        FireConfig {
            peak: Intensity::FULL,
            swell: Layer::OFF,
            sway: Layer::OFF,
            flicker: Layer::OFF,
            gust: Gust::OFF,
        }
        .validate(&INSTALLATION);
    }

    /// The installation is validated along with the look.
    #[test]
    #[should_panic(expected = "tick_ms must be greater than 0")]
    fn rejects_invalid_installation() {
        FireConfig::HEARTH.validate(&InstallationConfig {
            tick_ms: 0,
            ..INSTALLATION
        });
    }

    /// A dark peak is rejected.
    #[test]
    #[should_panic(expected = "peak must be greater than 0")]
    fn rejects_dark_peak() {
        FireConfig {
            peak: Intensity::OFF,
            ..FireConfig::HEARTH
        }
        .validate(&INSTALLATION);
    }

    /// Depths of 1001 permille, which could pull the light below dark, are rejected; 1000 pass.
    #[test]
    #[should_panic(expected = "must add up to at most 1000 permille")]
    fn rejects_excess_depth() {
        let full = FireConfig {
            swell: Layer {
                depth_permille: 400,
                ..FireConfig::HEARTH.swell
            },
            sway: Layer {
                depth_permille: 300,
                ..FireConfig::HEARTH.sway
            },
            flicker: Layer {
                depth_permille: 200,
                ..FireConfig::HEARTH.flicker
            },
            gust: Gust {
                depth_permille: 100,
                ..FireConfig::HEARTH.gust
            },
            ..FireConfig::HEARTH
        };
        full.validate(&INSTALLATION);
        FireConfig {
            gust: Gust {
                depth_permille: 101,
                ..full.gust
            },
            ..full
        }
        .validate(&INSTALLATION);
    }

    /// A glide shorter than two ticks is rejected and named after its layer.
    #[test]
    #[should_panic(expected = "sway.min_ms must span at least two ticks")]
    fn rejects_jumping_layer() {
        FireConfig {
            sway: Layer {
                min_ms: 29,
                ..FireConfig::HEARTH.sway
            },
            ..FireConfig::HEARTH
        }
        .validate(&INSTALLATION);
    }

    /// Inverted glide times are rejected and named after their layer.
    #[test]
    #[should_panic(expected = "swell.max_ms must be at least swell.min_ms")]
    fn rejects_inverted_layer() {
        FireConfig {
            swell: Layer {
                max_ms: FireConfig::HEARTH.swell.min_ms - 1,
                ..FireConfig::HEARTH.swell
            },
            ..FireConfig::HEARTH
        }
        .validate(&INSTALLATION);
    }

    /// A glide so long that the layer would stand still is rejected.
    #[test]
    #[should_panic(expected = "flicker.max_ms is too long for the tick")]
    fn rejects_frozen_layer() {
        FireConfig {
            flicker: Layer {
                max_ms: 15 * 65_536 + 1,
                ..FireConfig::HEARTH.flicker
            },
            ..FireConfig::HEARTH
        }
        .validate(&INSTALLATION);
    }

    /// Gusts need a moving flicker layer even when the flicker itself is off.
    #[test]
    #[should_panic(expected = "flicker.min_ms must span at least two ticks")]
    fn gusts_need_flicker_timing() {
        FireConfig {
            flicker: Layer::OFF,
            ..FireConfig::TORCH
        }
        .validate(&INSTALLATION);
    }

    /// A gust interval shorter than a tick is rejected.
    #[test]
    #[should_panic(expected = "gust.mean_interval_ms must span at least one tick")]
    fn rejects_constant_gusts() {
        FireConfig {
            gust: Gust {
                mean_interval_ms: 14,
                ..FireConfig::TORCH.gust
            },
            ..FireConfig::TORCH
        }
        .validate(&INSTALLATION);
    }

    /// A gust interval so long that gusts never come is rejected.
    #[test]
    #[should_panic(expected = "gust.mean_interval_ms is too long for the tick")]
    fn rejects_absent_gusts() {
        FireConfig {
            gust: Gust {
                mean_interval_ms: 15 * 65_536 + 1,
                ..FireConfig::TORCH.gust
            },
            ..FireConfig::TORCH
        }
        .validate(&INSTALLATION);
    }

    /// A gust shorter than a tick is rejected.
    #[test]
    #[should_panic(expected = "gust.duration_ms must span at least one tick")]
    fn rejects_instant_gust() {
        FireConfig {
            gust: Gust {
                duration_ms: 14,
                ..FireConfig::TORCH.gust
            },
            ..FireConfig::TORCH
        }
        .validate(&INSTALLATION);
    }

    /// A gust so long that it never fades is rejected.
    #[test]
    #[should_panic(expected = "gust.duration_ms is too long for the tick")]
    fn rejects_endless_gust() {
        FireConfig {
            gust: Gust {
                duration_ms: 15 * 65_535 + 1,
                ..FireConfig::TORCH.gust
            },
            ..FireConfig::TORCH
        }
        .validate(&INSTALLATION);
    }
}
