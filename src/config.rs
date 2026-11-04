//! Fire effect configuration and its compile-time validation.

/// Active fire effect configuration.
///
/// A `const`, so the optimizer folds every derived quantity (envelope step, thresholds, decay
/// factors) into immediates.
pub const CFG: FireConfig = FireConfig {
    max_intensity: 128,
    jitter_max: 25,
    pulse_prob: 48,
    breath_period_ms: 5_000,
    ..FireConfig::new()
};

const _: () = {
    assert!(
        CFG.max_intensity > CFG.min_intensity,
        "max_intensity must be greater than min_intensity"
    );
    assert!(
        CFG.pwm_divider >= 1,
        "pwm_divider must be at least 1, 0 would divide by zero"
    );
    assert!(CFG.pwm_freq_hz > 0, "pwm_freq_hz must be greater than 0");
    assert!(CFG.tick_ms > 0, "tick_ms must be greater than 0");
    assert!(
        CFG.breath_period_ms >= CFG.tick_ms,
        "breath_period_ms must span at least one tick, or the envelope step overflows"
    );
    assert!(
        CFG.smooth_q8 > 0,
        "smooth_q8 must be greater than 0, or the output never changes"
    );
};

/// Fire effect parameters.
///
/// These shape the look of the LED flame. All values are integers. Override fields of
/// [`FireConfig::new`] with struct update syntax.
#[derive(Clone, Copy, Debug)]
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
    /// A `const fn` rather than only `Default`, because [`CFG`] is built in a const context.
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
}

impl Default for FireConfig {
    fn default() -> Self {
        Self::new()
    }
}
