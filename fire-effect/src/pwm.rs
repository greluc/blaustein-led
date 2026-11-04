// SPDX-FileCopyrightText: 2026 the Blaustein LED contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Gamma-corrected PWM duty values and PWM timing.

use crate::fixed_point::Q16;

/// Square of the highest intensity; normalizes the gamma curve to full scale.
const MAX_SQUARED: u32 = 255 * 255;

/// Returns the square-law brightness of `intensity` in Q16, rounded to nearest.
///
/// Computed instead of looked up: one multiply and one hardware divide cost less than the
/// 512-byte table they replace. `255² * u16::MAX` still fits in u32.
fn gamma_q16(intensity: u8) -> u32 {
    let squared = u32::from(intensity) * u32::from(intensity);
    (squared * u32::from(u16::MAX) + MAX_SQUARED / 2) / MAX_SQUARED
}

/// Maps an intensity to a PWM compare value in `0..=top`.
#[must_use]
pub fn duty_for(intensity: u8, top: u16) -> u16 {
    ((gamma_q16(intensity) * (u32::from(top) + 1)) >> Q16) as u16
}

/// Returns the PWM `top` value for a slice clocked at `clock_hz` with an integer divider.
///
/// A slice counts `0..=top`, so `top` is one less than the period in counts. The period
/// saturates at the 16-bit counter range.
///
/// # Panics
///
/// Panics if `desired_freq_hz` or `divider` is zero.
#[must_use]
pub fn calculate_top(clock_hz: u32, desired_freq_hz: u32, divider: u8) -> u16 {
    let period_counts = clock_hz / (desired_freq_hz * u32::from(divider));
    u16::try_from(period_counts.saturating_sub(1)).unwrap_or(u16::MAX)
}

#[cfg(test)]
mod tests {
    //! Gamma table, duty mapping and PWM timing.

    use super::{calculate_top, duty_for, gamma_q16};

    /// System clock of the RP2350 at its default configuration.
    const CLK_SYS_HZ: u32 = 150_000_000;

    /// `top` values spanning the full counter range.
    const TOPS: [u16; 5] = [0, 1, 255, 5_999, u16::MAX];

    /// The gamma curve runs from zero to full scale without decreasing.
    #[test]
    fn gamma_is_monotonic_from_zero_to_full_scale() {
        assert_eq!(gamma_q16(0), 0, "gamma must start at zero");
        assert_eq!(
            gamma_q16(u8::MAX),
            u32::from(u16::MAX),
            "gamma must end at full scale"
        );
        let mut previous = 0;
        for intensity in 0..=u8::MAX {
            let gamma = gamma_q16(intensity);
            assert!(gamma >= previous, "gamma decreased at {intensity}");
            previous = gamma;
        }
    }

    /// The gamma curve matches the rounded square law computed in 64 bits.
    #[test]
    fn gamma_matches_square_law() {
        let max_squared = u64::from(u8::MAX) * u64::from(u8::MAX);
        for intensity in 0..=u8::MAX {
            let squared = u64::from(intensity) * u64::from(intensity);
            let expected = (squared * u64::from(u16::MAX) + max_squared / 2) / max_squared;
            assert_eq!(
                u64::from(gamma_q16(intensity)),
                expected,
                "gamma differs at {intensity}"
            );
        }
    }

    /// Duty values stay within `0..=top` and reach both ends.
    #[test]
    fn duty_spans_zero_to_top() {
        for top in TOPS {
            assert_eq!(duty_for(0, top), 0, "intensity 0 must switch the LED off");
            assert_eq!(duty_for(u8::MAX, top), top, "intensity 255 must reach top");
            for intensity in 0..=u8::MAX {
                assert!(duty_for(intensity, top) <= top, "duty exceeded top {top}");
            }
        }
    }

    /// `top` matches the period for the default PWM frequency.
    #[test]
    fn top_matches_period() {
        assert_eq!(
            calculate_top(CLK_SYS_HZ, 25_000, 1),
            5_999,
            "6000 counts at 25 kHz"
        );
        assert_eq!(
            calculate_top(CLK_SYS_HZ, 25_000, 2),
            2_999,
            "the divider halves the period"
        );
    }

    /// `top` saturates at both ends of the counter range.
    #[test]
    fn top_saturates() {
        assert_eq!(
            calculate_top(65_536, 1, 1),
            u16::MAX,
            "a full 16-bit period fits exactly"
        );
        assert_eq!(
            calculate_top(CLK_SYS_HZ, 1_000, 1),
            u16::MAX,
            "long periods saturate"
        );
        assert_eq!(
            calculate_top(1_000, 25_000, 1),
            0,
            "a clock below the frequency gives 0"
        );
    }

    /// `top` rejects a zero frequency.
    #[test]
    #[should_panic(expected = "divide by zero")]
    fn top_rejects_zero_frequency() {
        let _top = calculate_top(CLK_SYS_HZ, 0, 1);
    }
}
