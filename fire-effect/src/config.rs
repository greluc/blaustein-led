//! Fire effect parameters and their validation.

use crate::fixed_point::Q16;

/// A fire effect configuration fixed at compile time.
pub trait FireProfile {
    /// The parameters. [`FireState`](crate::FireState) rejects invalid ones at compile time.
    const CONFIG: FireConfig;
}

/// Fire effect parameters.
///
/// These shape the look of the LED flame. All values are integers. Override fields of
/// [`FireConfig::new`] with struct update syntax.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FireConfig {
    /// PWM target frequency in Hz. 20-30 kHz keeps flicker invisible.
    pub pwm_freq_hz: u32,
    /// Integer clock divider, at least 1. Use the smallest value that keeps `top` within `u16`.
    pub pwm_divider: u8,
    /// Minimum base intensity (0..=255).
    pub min_intensity: u8,
    /// Maximum base intensity (0..=255). Must be greater than `min_intensity`.
    pub max_intensity: u8,
    /// Full breath period in milliseconds (one up and down cycle), at least one tick.
    pub breath_period_ms: u32,
    /// Random per-tick jitter amplitude around the base (0..=64 typical).
    pub jitter_max: u8,
    /// Probability (0..=255) to start a short flare on a tick. At a 15 ms tick, 3 is about 1 %
    /// (~0.8 flares/s) and 48 is about 19 % (~12 flares/s, a visibly busy fire).
    pub pulse_prob: u8,
    /// Intensity added when a flare triggers.
    pub pulse_boost: u8,
    /// Q8 decay factor per tick for flares (0..=255). 224 decays fast, 248 slower.
    pub pulse_decay_q8: u8,
    /// Q8 smoothing factor per tick for the output EMA, at least 1. Smaller is smoother:
    /// `new = old + ((target - old) * smooth_q8 >> 8)`.
    pub smooth_q8: u8,
    /// Update interval in milliseconds.
    pub tick_ms: u32,
}

impl FireConfig {
    /// Returns the baseline configuration.
    ///
    /// A `const fn` rather than only `Default`, because profiles are built in const contexts.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            pwm_freq_hz: 25_000,
            pwm_divider: 1,
            min_intensity: 20,
            max_intensity: 255,
            breath_period_ms: 3_500,
            jitter_max: 18,
            pulse_prob: 3,
            pulse_boost: 40,
            // About 105 ms half-life at a 15 ms tick.
            pulse_decay_q8: 232,
            // The output settles within a few hundred milliseconds.
            smooth_q8: 20,
            tick_ms: 15,
        }
    }

    /// Checks that the parameters produce a working effect.
    ///
    /// # Panics
    ///
    /// Panics with a description of the first violated constraint. Evaluated in a const context,
    /// that is a build error.
    pub const fn validate(&self) {
        assert!(
            self.max_intensity > self.min_intensity,
            "max_intensity must be greater than min_intensity"
        );
        assert!(
            self.pwm_divider >= 1,
            "pwm_divider must be at least 1, 0 would divide by zero"
        );
        assert!(self.pwm_freq_hz > 0, "pwm_freq_hz must be greater than 0");
        assert!(self.tick_ms > 0, "tick_ms must be greater than 0");
        assert!(
            self.breath_period_ms >= self.tick_ms,
            "breath_period_ms must span at least one tick, or the envelope step overflows"
        );
        assert!(
            self.smooth_q8 > 0,
            "smooth_q8 must be greater than 0, or the output never changes"
        );
        assert!(
            self.step_per_tick_q16() > 0_i32,
            "breath_period_ms is too long for this intensity span: the envelope step truncates to \
             zero and the breathing would freeze"
        );
    }

    /// Returns the breathing envelope step per tick in Q16.
    ///
    /// One 64-bit expression, so the intermediate never truncates:
    /// `2 * span * 2^16 * tick_ms / breath_period_ms`.
    #[must_use]
    #[expect(
        clippy::cast_possible_truncation,
        reason = "`breath_period_ms >= tick_ms` bounds the step by `2 * 255 << 16`, which fits in \
                  i32"
    )]
    pub(crate) const fn step_per_tick_q16(&self) -> i32 {
        let span = self.max_intensity as i64 - self.min_intensity as i64;
        ((span << (Q16 + 1)) * self.tick_ms as i64 / self.breath_period_ms as i64) as i32
    }
}

impl Default for FireConfig {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    //! Validation and derived values of [`FireConfig`].

    use super::FireConfig;

    /// The baseline configuration is valid.
    #[test]
    fn baseline_is_valid() {
        FireConfig::new().validate();
        assert_eq!(
            FireConfig::default(),
            FireConfig::new(),
            "Default must match new()"
        );
    }

    /// The envelope step of the baseline matches the closed-form value.
    #[test]
    fn baseline_step_matches_formula() {
        // (255 - 20) << 17 * 15 / 3500, truncated.
        assert_eq!(
            FireConfig::new().step_per_tick_q16(),
            132_008_i32,
            "baseline step changed"
        );
    }

    /// An empty intensity span is rejected.
    #[test]
    #[should_panic(expected = "max_intensity must be greater than min_intensity")]
    fn rejects_empty_span() {
        FireConfig {
            min_intensity: 100,
            max_intensity: 100,
            ..FireConfig::new()
        }
        .validate();
    }

    /// A zero clock divider is rejected.
    #[test]
    #[should_panic(expected = "pwm_divider must be at least 1")]
    fn rejects_zero_divider() {
        FireConfig {
            pwm_divider: 0,
            ..FireConfig::new()
        }
        .validate();
    }

    /// A zero PWM frequency is rejected.
    #[test]
    #[should_panic(expected = "pwm_freq_hz must be greater than 0")]
    fn rejects_zero_frequency() {
        FireConfig {
            pwm_freq_hz: 0,
            ..FireConfig::new()
        }
        .validate();
    }

    /// A zero tick is rejected.
    #[test]
    #[should_panic(expected = "tick_ms must be greater than 0")]
    fn rejects_zero_tick() {
        FireConfig {
            tick_ms: 0,
            ..FireConfig::new()
        }
        .validate();
    }

    /// A breath period shorter than one tick is rejected.
    #[test]
    #[should_panic(expected = "breath_period_ms must span at least one tick")]
    fn rejects_breath_shorter_than_tick() {
        FireConfig {
            breath_period_ms: 10,
            tick_ms: 15,
            ..FireConfig::new()
        }
        .validate();
    }

    /// A zero smoothing factor is rejected.
    #[test]
    #[should_panic(expected = "smooth_q8 must be greater than 0")]
    fn rejects_zero_smoothing() {
        FireConfig {
            smooth_q8: 0,
            ..FireConfig::new()
        }
        .validate();
    }

    /// A breath period so long that the step truncates to zero is rejected.
    #[test]
    #[should_panic(expected = "breath_period_ms is too long")]
    fn rejects_frozen_envelope() {
        FireConfig {
            min_intensity: 0,
            max_intensity: 1,
            breath_period_ms: 4_000_000,
            ..FireConfig::new()
        }
        .validate();
    }
}
