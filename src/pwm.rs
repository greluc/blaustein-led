//! PWM timing and gamma-corrected duty values.

use embassy_rp::clocks;

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

/// Returns the PWM `top` value for the desired frequency and integer clock divider.
///
/// A slice counts `0..=top`, so `top` is one less than the period in counts. The period
/// saturates at the 16-bit counter range.
///
/// # Panics
///
/// Panics if `desired_freq_hz` or `divider` is zero.
#[must_use]
pub fn calculate_top(desired_freq_hz: u32, divider: u8) -> u16 {
    let period_counts = clocks::clk_sys_freq() / (desired_freq_hz * u32::from(divider));
    u16::try_from(period_counts.saturating_sub(1)).unwrap_or(u16::MAX)
}
