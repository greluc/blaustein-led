//! Gamma-corrected PWM duty values and PWM timing.

use crate::fixed_point::Q16;

/// Number of intensity levels.
const LEVELS: usize = 256;

/// Gamma (square-law) brightness table normalized to Q16, in flash and shared by all channels.
#[expect(
    clippy::cast_possible_truncation,
    reason = "each entry is at most `u16::MAX * 255² / 255²`, which fits in u16"
)]
const GAMMA_Q16: [u16; LEVELS] = {
    const MAX_SQUARED: u64 = ((LEVELS - 1) * (LEVELS - 1)) as u64;
    let mut table = [0_u16; LEVELS];
    let mut index = 0;
    while index < LEVELS {
        let squared = (index * index) as u64;
        // Adding half the divisor rounds to nearest.
        table[index] = ((squared * u16::MAX as u64 + MAX_SQUARED / 2) / MAX_SQUARED) as u16;
        index += 1;
    }
    table
};

/// Maps an intensity to a PWM compare value in `0..=top`.
#[must_use]
pub fn duty_for(intensity: u8, top: u16) -> u16 {
    let gamma = u32::from(GAMMA_Q16[usize::from(intensity)]);
    ((gamma * (u32::from(top) + 1)) >> Q16) as u16
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

    use super::{GAMMA_Q16, calculate_top, duty_for};

    /// System clock of the RP2350 at its default configuration.
    const CLK_SYS_HZ: u32 = 150_000_000;

    /// `top` values spanning the full counter range.
    const TOPS: [u16; 5] = [0, 1, 255, 5_999, u16::MAX];

    /// The gamma table runs from zero to full scale without decreasing.
    #[test]
    fn gamma_is_monotonic_from_zero_to_full_scale() {
        assert_eq!(GAMMA_Q16.first(), Some(&0), "gamma must start at zero");
        assert_eq!(
            GAMMA_Q16.last(),
            Some(&u16::MAX),
            "gamma must end at full scale"
        );
        assert!(GAMMA_Q16.is_sorted(), "gamma must not decrease");
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
