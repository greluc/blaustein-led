//! Flicker simulation for one LED channel.

use crate::config::CFG;
use crate::fixed_point::{Q8, Q16};

/// Envelope step per tick in Q16.
///
/// One 64-bit expression, so the intermediate never truncates:
/// `2 * span * 2^16 * tick_ms / breath_period_ms`.
#[expect(
    clippy::cast_possible_truncation,
    reason = "`breath_period_ms >= tick_ms` bounds the step by `2 * 255 << 16`, which fits in i32"
)]
const STEP_PER_TICK_Q16: i32 = {
    let span = CFG.max_intensity as i64 - CFG.min_intensity as i64;
    ((span << (Q16 + 1)) * CFG.tick_ms as i64 / CFG.breath_period_ms as i64) as i32
};

const _: () = assert!(
    STEP_PER_TICK_Q16 > 0,
    "breath_period_ms is too long for this intensity span: the envelope step truncates to zero \
     and the breathing would freeze"
);

/// Lower bound of the breathing envelope in Q16.
const MIN_Q16: i32 = (CFG.min_intensity as i32) << Q16;

/// Upper bound of the breathing envelope in Q16.
const MAX_Q16: i32 = (CFG.max_intensity as i32) << Q16;

/// Half of one Q8 unit; adding it before a right shift rounds to nearest.
const Q8_HALF: i32 = 1 << (Q8 - 1);

/// Offset that centers a random byte around zero.
const BYTE_CENTER: i32 = 128;

/// Upper bound of the target intensity.
const INTENSITY_MAX: i32 = 255;

/// State of the fire simulation for one LED channel.
///
/// The configuration is the global [`CFG`] constant rather than a field, so every
/// configuration-derived operand folds into an immediate.
pub struct FireState {
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
}

impl FireState {
    /// Creates a channel simulation from a non-zero PRNG seed.
    ///
    /// # Panics
    ///
    /// Panics if `seed` is zero, the fixed point of Xorshift32. Evaluate it in a `const` block
    /// to turn that into a build error.
    #[must_use]
    pub const fn new(seed: u32) -> Self {
        assert!(
            seed != 0,
            "a zero seed would keep Xorshift32 at zero forever"
        );
        Self {
            rng: seed,
            base_q16: MIN_Q16,
            step_q16: STEP_PER_TICK_Q16,
            out_q8: (CFG.min_intensity as i32) << Q8,
            pulse: 0,
        }
    }

    /// Advances the simulation by one tick and returns the new intensity.
    pub fn update(&mut self) -> u8 {
        let base = self.advance_envelope();

        // One PRNG step per tick; jitter and flare read independent bytes of the same word.
        self.rng = xorshift32(self.rng);
        let [jitter_byte, _, _, pulse_byte] = self.rng.to_be_bytes();

        let target = (base + jitter(jitter_byte) + self.advance_pulse(pulse_byte))
            .clamp(0_i32, INTENSITY_MAX);
        self.smooth_towards(target)
    }

    /// Moves the triangular breathing envelope one step and returns its integer part.
    fn advance_envelope(&mut self) -> i32 {
        self.base_q16 = (self.base_q16 + self.step_q16).clamp(MIN_Q16, MAX_Q16);
        if self.base_q16 == MIN_Q16 || self.base_q16 == MAX_Q16 {
            self.step_q16 = -self.step_q16;
        }
        self.base_q16 >> Q16
    }

    /// Triggers a flare with probability `pulse_prob / 256`, decays it and returns it.
    fn advance_pulse(&mut self, random: u8) -> i32 {
        if random < CFG.pulse_prob {
            self.pulse += i32::from(CFG.pulse_boost);
        }
        self.pulse = (self.pulse * i32::from(CFG.pulse_decay_q8)) >> Q8;
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
        self.out_q8 += (diff * i32::from(CFG.smooth_q8) + Q8_HALF) >> Q8;
        (self.out_q8 >> Q8) as u8
    }
}

/// Maps a random byte to a jitter of about `-jitter_max..=jitter_max`.
fn jitter(random: u8) -> i32 {
    let noise = i32::from(random) - BYTE_CENTER;
    (noise * (2 * i32::from(CFG.jitter_max) + 1)) >> Q8
}

/// Advances a Xorshift32 PRNG state: fast, small, and good enough for flicker.
#[must_use]
const fn xorshift32(mut state: u32) -> u32 {
    state ^= state << 13_u32;
    state ^= state >> 17_u32;
    state ^= state << 5_u32;
    state
}
