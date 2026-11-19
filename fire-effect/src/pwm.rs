// SPDX-FileCopyrightText: 2026 the Blaustein LED contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Gamma-corrected PWM duty values and PWM timing.

use crate::intensity::Intensity;

/// Full-scale intensity; the square law is normalized to it.
const FULL: u32 = u16::MAX as u32;

/// Returns `value / 65535`, rounded half up.
///
/// `value` must stay at most `65535²`, so the sum fits in `u32`. Dividing by a constant compiles
/// to a multiply.
const fn div_full_rounded(value: u32) -> u32 {
    (value + FULL / 2) / FULL
}

/// Returns the square-law brightness of `intensity` on the intensity scale,
/// `round(intensity² / 65535)`; full scale maps to `u16::MAX`.
const fn gamma(intensity: Intensity) -> u32 {
    let raw = intensity.get() as u32;
    div_full_rounded(raw * raw)
}

/// Maps a linear intensity to a PWM compare value in `0..=top` through a square-law gamma curve.
///
/// Rounding rule: `duty = round(round(i² / 65535) * top / 65535)` for the raw intensity `i`,
/// both roundings half up. It is monotonic, maps dark to 0 and full scale to `top`, and deviates
/// from the exact `top * (i / 65535)²` by less than one count. Both steps are a multiply and a
/// division by a constant, so no table is needed.
#[must_use]
#[expect(
    clippy::cast_possible_truncation,
    reason = "`gamma <= 65535` keeps the result at most `top`"
)]
pub const fn duty_for(intensity: Intensity, top: u16) -> u16 {
    div_full_rounded(gamma(intensity) * top as u32) as u16
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
    //! Gamma curve, duty mapping and PWM timing.

    use super::{calculate_top, duty_for, gamma};
    use crate::intensity::Intensity;

    /// System clock of the RP2350 at its default configuration.
    const CLK_SYS_HZ: u32 = 150_000_000;

    /// `top` values spanning the full counter range.
    const TOPS: [u16; 6] = [0, 1, 255, 2_999, 5_999, u16::MAX];

    /// Square of full scale, the denominator of the exact square law.
    const FULL_SQUARED: u64 = 65_535 * 65_535;

    /// Returns every intensity from dark to full scale.
    fn every_intensity() -> impl Iterator<Item = Intensity> {
        (0..=u16::MAX).map(Intensity::new)
    }

    /// Returns `value / 65535` rounded half up, in 64 bits.
    fn reference_div_full(value: u64) -> u64 {
        (value + 65_535 / 2) / 65_535
    }

    /// The gamma curve runs from zero to full scale without decreasing.
    #[test]
    fn gamma_is_monotonic_from_zero_to_full_scale() {
        assert_eq!(gamma(Intensity::OFF), 0, "gamma must start at zero");
        assert_eq!(
            gamma(Intensity::FULL),
            u32::from(u16::MAX),
            "gamma must end at full scale"
        );
        let mut previous = 0;
        for intensity in every_intensity() {
            let brightness = gamma(intensity);
            assert!(brightness >= previous, "gamma decreased at {intensity:?}");
            previous = brightness;
        }
    }

    /// The gamma curve matches the rounded square law computed in 64 bits.
    #[test]
    fn gamma_matches_square_law() {
        for intensity in every_intensity() {
            let expected = reference_div_full(u64::from(intensity.get()).pow(2));
            assert_eq!(
                u64::from(gamma(intensity)),
                expected,
                "gamma differs at {intensity:?}"
            );
        }
    }

    /// For every intensity and every `top`, the duty is monotonic, within `0..=top`, reaches both
    /// ends and follows the stated rounding rule.
    #[test]
    fn duty_follows_the_rounding_rule() {
        for top in TOPS {
            assert_eq!(
                duty_for(Intensity::OFF, top),
                0,
                "dark must switch the LED off"
            );
            assert_eq!(
                duty_for(Intensity::FULL, top),
                top,
                "full scale must reach top {top}"
            );
            let mut previous = 0;
            for intensity in every_intensity() {
                let duty = duty_for(intensity, top);
                let brightness = reference_div_full(u64::from(intensity.get()).pow(2));
                let rule = reference_div_full(brightness * u64::from(top));
                assert_eq!(
                    u64::from(duty),
                    rule,
                    "duty breaks the rule at {intensity:?}, top {top}"
                );
                assert!(
                    duty >= previous,
                    "duty decreased at {intensity:?}, top {top}"
                );
                assert!(duty <= top, "duty exceeded top {top} at {intensity:?}");
                previous = duty;
            }
        }
    }

    /// The duty deviates from the exact square law by less than one count.
    #[test]
    fn duty_is_within_one_count_of_the_square_law() {
        for top in TOPS {
            for intensity in every_intensity() {
                // Compare `duty` with `top * i² / 65535²` scaled by `65535²` to stay in integers.
                let exact = u64::from(top) * u64::from(intensity.get()).pow(2);
                let duty = u64::from(duty_for(intensity, top)) * FULL_SQUARED;
                assert!(
                    duty.abs_diff(exact) < FULL_SQUARED,
                    "duty is a count or more off the square law at {intensity:?}, top {top}"
                );
            }
        }
    }

    /// At 25 kHz, adjacent intensities move the duty by at most one count, so dim lights fade
    /// without visible steps of the intensity.
    #[test]
    fn adjacent_intensities_move_the_duty_by_one_count_at_most() {
        let top = calculate_top(CLK_SYS_HZ, 25_000, 1);
        for raw in 0..u16::MAX {
            let step = duty_for(Intensity::new(raw + 1), top) - duty_for(Intensity::new(raw), top);
            assert!(
                step <= 1,
                "duty jumped by {step} counts above intensity {raw}"
            );
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
