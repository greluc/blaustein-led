// SPDX-FileCopyrightText: 2026 the Blaustein LED contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Smoothly interpolated value noise, the building block of every layer of the flicker.
//!
//! A layer glides from one random level to the next. Each glide takes a random time and eases in
//! and out along a smoothstep curve, so the motion is continuous in level and slope, has no fixed
//! period and is band-limited by the range of glide times. Levels are Q16 shares of the layer's
//! top: `0` pulls the light nowhere, `u16::MAX` pulls it the full depth of the layer.

use crate::fixed_point::{ONE_Q16, Q16, per_tick};
use crate::rng::Rng;

/// Three in Q16, the constant of the factor `3 - 2x` of smoothstep.
const THREE_Q16: u64 = 3 << Q16;

/// Range of glide rates a layer picks from, in Q16 glides per tick.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rates {
    /// Rate of the longest glide.
    slowest: u16,
    /// Distance from the slowest to the fastest rate.
    span: u16,
}

impl Rates {
    /// Rates of a layer that stands still.
    pub const STILL: Self = Self {
        slowest: 0,
        span: 0,
    };

    /// Returns the rates of glides lasting `min_ms..=max_ms` at a tick of `tick_ms`.
    ///
    /// # Panics
    ///
    /// Panics if `min_ms` spans less than two ticks or `max_ms` is shorter than `min_ms`;
    /// [`FireConfig::validate`](crate::FireConfig::validate) reports both with a clearer message.
    #[must_use]
    #[expect(
        clippy::cast_possible_truncation,
        reason = "a glide of at least two ticks advances at most half a Q16 unit, which fits in \
                  u16"
    )]
    pub const fn from_interval(min_ms: u32, max_ms: u32, tick_ms: u32) -> Self {
        assert!(
            min_ms / 2 >= tick_ms,
            "a glide must span at least two ticks"
        );
        assert!(
            max_ms >= min_ms,
            "the longest glide must not be shorter than the shortest"
        );
        let slowest = per_tick(ONE_Q16, tick_ms, max_ms) as u16;
        let fastest = per_tick(ONE_Q16, tick_ms, min_ms) as u16;
        Self {
            slowest,
            span: fastest - slowest,
        }
    }

    /// Maps a random number uniformly onto `slowest..=slowest + span`.
    ///
    /// `random * (span + 1) >> 16` is at most `span`, so the sum is at most the fastest rate.
    const fn pick(self, random: u16) -> u16 {
        self.slowest + ((random as u32 * (self.span as u32 + 1)) >> Q16) as u16
    }
}

/// State of one layer of value noise.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ValueNoise {
    /// Level at the start of the current glide.
    from: u16,
    /// Level at the end of the current glide.
    to: u16,
    /// Progress through the current glide in Q16.
    phase: u16,
    /// Phase advance per tick in Q16.
    rate: u16,
}

impl ValueNoise {
    /// Starts a layer at a random point of a random glide, so differently seeded layers start at
    /// unrelated phases.
    #[must_use]
    pub const fn new(rng: &mut Rng, rates: Rates) -> Self {
        let [from, to] = rng.next_pair();
        let [phase, pick] = rng.next_pair();
        Self {
            from,
            to,
            phase,
            rate: rates.pick(pick),
        }
    }

    /// Advances the layer by one tick and returns its level.
    ///
    /// When a glide ends, the next one starts from its end towards a new random level with a new
    /// random rate; the phase carries over, so no tick is lost.
    pub const fn advance(&mut self, rng: &mut Rng, rates: Rates) -> u16 {
        let (phase, finished) = self.phase.overflowing_add(self.rate);
        self.phase = phase;
        if finished {
            let [to, pick] = rng.next_pair();
            self.from = self.to;
            self.to = to;
            self.rate = rates.pick(pick);
        }
        lerp(self.from, self.to, smoothstep(self.phase))
    }
}

/// Returns `3x² - 2x³` for `x` in Q16, truncated: eases from 0 to just below 1.
///
/// Exact in 64 bits (`x²(3 - 2x) < 2^50`), so the curve is monotonic and its largest value is
/// `u16::MAX`. On the RP2350 the product is a single `UMULL` whose high word is the result.
#[expect(
    clippy::cast_possible_truncation,
    reason = "`x < 1` keeps the curve below one, so it fits in u16"
)]
const fn smoothstep(x: u16) -> u16 {
    let squared = x as u32 * x as u32;
    ((squared as u64 * (THREE_Q16 - 2 * x as u64)) >> (2 * Q16)) as u16
}

/// Returns the point a Q16 share `weight` of the way from `from` to `to`, truncated.
///
/// The weighted sum is at most `u16::MAX << 16`, so it fits in `u32` and the result in `u16`.
const fn lerp(from: u16, to: u16, weight: u16) -> u16 {
    let weight = weight as u32;
    ((from as u32 * (ONE_Q16 - weight) + to as u32 * weight) >> Q16) as u16
}

#[cfg(test)]
mod tests {
    //! Smoothstep, interpolation, glide rates and the layer state.

    use super::{Rates, ValueNoise, lerp, smoothstep};
    use crate::rng::Rng;

    /// Tick of the tests in milliseconds.
    const TICK_MS: u32 = 15;

    /// Glides of 45 to 120 ms, three to eight ticks.
    const RATES: Rates = Rates::from_interval(45, 120, TICK_MS);

    /// Smoothstep rises monotonically from 0 to `u16::MAX` and is point-symmetric about the
    /// middle.
    #[test]
    fn smoothstep_eases_monotonically() {
        assert_eq!(smoothstep(0), 0, "smoothstep starts at 0");
        assert_eq!(
            smoothstep(u16::MAX),
            u16::MAX,
            "smoothstep ends at full scale"
        );
        assert_eq!(smoothstep(32_768), 32_768, "smoothstep passes the middle");
        let mut previous = 0;
        for x in 1..=u16::MAX {
            let eased = smoothstep(x);
            assert!(eased >= previous, "smoothstep decreased at {x}");
            let mirrored = smoothstep(0_u16.wrapping_sub(x));
            assert!(
                (u32::from(eased) + u32::from(mirrored)).abs_diff(1 << 16) <= 1,
                "smoothstep is not symmetric at {x}: {eased} + {mirrored}"
            );
            previous = eased;
        }
    }

    /// Smoothstep starts and ends flat, so glides join without a kink.
    #[test]
    fn smoothstep_is_flat_at_both_ends() {
        let start = smoothstep(256);
        let end = u16::MAX - smoothstep(u16::MAX - 256);
        assert!(start <= 3, "smoothstep starts too steeply: {start}");
        assert!(end <= 3, "smoothstep ends too steeply: {end}");
    }

    /// Interpolation hits its start, its middle and stays between its ends.
    #[test]
    fn lerp_stays_between_its_ends() {
        assert_eq!(lerp(100, 900, 0), 100, "weight 0 returns the start");
        assert_eq!(
            lerp(100, 900, 32_768),
            500,
            "weight one half returns the middle"
        );
        assert_eq!(
            lerp(u16::MAX, u16::MAX, u16::MAX),
            u16::MAX,
            "a glide that stays at full scale keeps it"
        );
        for weight in (0..=u16::MAX).step_by(97) {
            let point = lerp(900, 100, weight);
            assert!(
                (100..=900).contains(&point),
                "lerp left its ends at {weight}"
            );
        }
    }

    /// Rates span exactly the configured glide times.
    #[test]
    fn rates_span_the_glide_times() {
        assert_eq!(
            RATES.pick(0),
            8_192,
            "slowest rate is one eighth glide per tick"
        );
        assert_eq!(
            RATES.pick(u16::MAX),
            21_845,
            "fastest rate is one third glide per tick"
        );
        assert_eq!(
            Rates::STILL.pick(u16::MAX),
            0,
            "a still layer never advances"
        );
    }

    /// A glide shorter than two ticks is rejected.
    #[test]
    #[should_panic(expected = "a glide must span at least two ticks")]
    fn rejects_jumping_glides() {
        let _rates = Rates::from_interval(TICK_MS, 100, TICK_MS);
    }

    /// Inverted glide times are rejected.
    #[test]
    #[should_panic(expected = "the longest glide must not be shorter")]
    fn rejects_inverted_glides() {
        let _rates = Rates::from_interval(100, 50, TICK_MS);
    }

    /// Every glide lasts between the shortest and the longest glide time, and the level stays
    /// between the ends of its glide.
    #[test]
    fn glides_last_within_their_range() {
        let mut rng = Rng::new(0xc0ff_ee01);
        let mut layer = ValueNoise::new(&mut rng, RATES);
        let mut since_start = 0_u32;
        let mut glides = 0_u32;
        let mut first = true;
        for _ in 0..100_000_u32 {
            let previous_phase = layer.phase;
            let level = layer.advance(&mut rng, RATES);
            let (low, high) = (layer.from.min(layer.to), layer.from.max(layer.to));
            assert!(
                (low..=high).contains(&level),
                "level {level} left its glide {low}..={high}"
            );
            since_start += 1;
            if layer.phase < previous_phase {
                // The first glide started at a random phase, so only later glides are measured.
                if !first {
                    assert!(
                        (3..=8).contains(&since_start),
                        "a glide lasted {since_start} ticks, outside 45 to 120 ms"
                    );
                    glides += 1;
                }
                first = false;
                since_start = 0;
            }
        }
        assert!(glides > 10_000, "only {glides} glides in 100000 ticks");
    }

    /// A still layer keeps its level.
    #[test]
    fn still_layer_keeps_its_level() {
        let mut rng = Rng::new(7);
        let mut layer = ValueNoise::new(&mut rng, Rates::STILL);
        let level = layer.advance(&mut rng, Rates::STILL);
        for _ in 0..1_000_u32 {
            assert_eq!(
                layer.advance(&mut rng, Rates::STILL),
                level,
                "a still layer moved"
            );
        }
    }
}
