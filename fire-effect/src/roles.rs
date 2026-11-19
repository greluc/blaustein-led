// SPDX-FileCopyrightText: 2026 the Blaustein LED contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Presets for the light roles of a castle model.
//!
//! The numbers are starting points, tuned in a host simulation and not yet on the real model:
//! adjust them there. Every preset is valid at every tick up to 20 ms.

use crate::config::{FireConfig, Gust, Layer};
use crate::follower::FollowerConfig;
use crate::intensity::Intensity;

impl FireConfig {
    /// Constant dim level with a few percent of unrest, for room light.
    pub const AMBIENT: Self = Self {
        peak: Intensity::from_permille(250),
        swell: Layer {
            depth_permille: 30,
            min_ms: 4_000,
            max_ms: 10_000,
        },
        sway: Layer {
            depth_permille: 20,
            min_ms: 1_000,
            max_ms: 3_000,
        },
        flicker: Layer::OFF,
        gust: Gust::OFF,
    };
    /// Almost steady with a rare short twitch, for a candle or an oil lamp.
    pub const CANDLE: Self = Self {
        peak: Intensity::from_permille(500),
        swell: Layer {
            depth_permille: 40,
            min_ms: 2_000,
            max_ms: 5_000,
        },
        sway: Layer {
            depth_permille: 30,
            min_ms: 400,
            max_ms: 1_200,
        },
        flicker: Layer {
            depth_permille: 20,
            min_ms: 60,
            max_ms: 150,
        },
        gust: Gust {
            depth_permille: 400,
            mean_interval_ms: 20_000,
            duration_ms: 250,
        },
    };
    /// Very slow, deep glow with a small swing and no fast flicker, for embers or a forge.
    pub const EMBER: Self = Self {
        peak: Intensity::from_permille(400),
        swell: Layer {
            depth_permille: 150,
            min_ms: 3_000,
            max_ms: 8_000,
        },
        sway: Layer {
            depth_permille: 80,
            min_ms: 1_000,
            max_ms: 2_500,
        },
        flicker: Layer::OFF,
        gust: Gust::OFF,
    };
    /// Irregular swell over about 1 to 3 s with a lively flicker, for a fireplace or a cooking
    /// fire.
    pub const HEARTH: Self = Self {
        peak: Intensity::from_permille(900),
        swell: Layer {
            depth_permille: 350,
            min_ms: 1_000,
            max_ms: 3_000,
        },
        sway: Layer {
            depth_permille: 200,
            min_ms: 250,
            max_ms: 700,
        },
        flicker: Layer {
            depth_permille: 180,
            min_ms: 45,
            max_ms: 120,
        },
        gust: Gust {
            depth_permille: 120,
            mean_interval_ms: 8_000,
            duration_ms: 400,
        },
    };
    /// Fast and restless with a small swing and occasional gusts, for a torch.
    pub const TORCH: Self = Self {
        peak: Intensity::from_permille(750),
        swell: Layer {
            depth_permille: 100,
            min_ms: 800,
            max_ms: 2_000,
        },
        sway: Layer {
            depth_permille: 150,
            min_ms: 150,
            max_ms: 400,
        },
        flicker: Layer {
            depth_permille: 200,
            min_ms: 40,
            max_ms: 80,
        },
        gust: Gust {
            depth_permille: 300,
            mean_interval_ms: 6_000,
            duration_ms: 600,
        },
    };
}

impl FollowerConfig {
    /// Red glow of the embers under a hearth flame: dim, slow and following the flame's level.
    pub const HEARTH_GLOW: Self = Self {
        floor: Intensity::from_permille(60),
        gain_permille: 600,
        smoothing_ms: 1_500,
    };
}

#[cfg(test)]
mod tests {
    //! Bounds and character of the presets.

    use crate::config::FireProfile;
    use crate::fixed_point::PERMILLE;
    use crate::rng::channel_seed;
    use crate::test_support::{Ambient, Candle, Ember, Hearth, Torch, mean_step_millis, record};

    /// Channels each preset is checked on.
    const CHANNELS: u32 = 4;

    /// Ticks per channel: 25 minutes at a 15 ms tick.
    const TICKS: usize = 100_000;

    /// Restlessness below which a light has no fast flicker: 0.1 % of its peak per tick.
    const STEADY_PER_MILLION: u64 = 1_000;

    /// Asserts that every output of `P` stays within its bounds on several channels, and returns
    /// the mean change per tick in millionths of the peak.
    fn checked_restlessness<P: FireProfile>(name: &str) -> u64 {
        let peak = P::CONFIG.peak.get();
        let floor_permille = u64::from(PERMILLE - P::CONFIG.total_depth_permille());
        let mut restlessness = 0;
        for channel in 0..CHANNELS {
            let outputs = record::<P>(channel_seed(channel), TICKS);
            for (tick, &output) in outputs.iter().enumerate() {
                assert!(
                    output <= peak,
                    "{name}: {output} exceeds the peak {peak} at {tick}"
                );
                assert!(
                    u64::from(output) * u64::from(PERMILLE) >= u64::from(peak) * floor_permille,
                    "{name}: {output} falls below {floor_permille} permille of {peak} at {tick}"
                );
            }
            restlessness += mean_step_millis(&outputs) * 1_000 / u64::from(peak);
        }
        restlessness / u64::from(CHANNELS)
    }

    /// Every preset stays within its bounds and keeps its character: the torch is the most
    /// restless, the hearth livelier than the candle, and the ember and the ambient light have no
    /// fast flicker, the ambient light the least.
    #[test]
    fn presets_keep_their_bounds_and_character() {
        let torch = checked_restlessness::<Torch>("torch");
        let hearth = checked_restlessness::<Hearth>("hearth");
        let candle = checked_restlessness::<Candle>("candle");
        let ember = checked_restlessness::<Ember>("ember");
        let ambient = checked_restlessness::<Ambient>("ambient");
        assert!(
            torch > hearth,
            "the torch ({torch}) must be more restless than the hearth ({hearth})"
        );
        assert!(
            hearth > candle,
            "the hearth ({hearth}) must be livelier than the candle ({candle})"
        );
        assert!(
            candle > ember,
            "the candle ({candle}) must flicker more than the ember ({ember})"
        );
        assert!(
            ember < STEADY_PER_MILLION && ambient < ember,
            "ember ({ember}) and ambient ({ambient}) must not flicker, ambient the least"
        );
    }
}
