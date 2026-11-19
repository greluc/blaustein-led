// SPDX-FileCopyrightText: 2026 the Blaustein LED contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The PWM outputs of the LED channels: their slices, phases and duty values.
//!
//! GPIO `n` is PWM channel `n mod 16` (slice `n mod 16 / 2`, A for even `n`, B for odd), so GPIO 7
//! to GPIO 22 reach every PWM output the RP2350A can route to a pin exactly once.

use core::array;

use embassy_rp::pwm::{ChannelAPin, ChannelBPin, Config as PwmConfig, Pwm, PwmBatch, Slice};
use embassy_rp::{Peri, clocks};
use fire_effect::{Intensity, pwm};

use crate::config::{CHANNELS, INSTALLATION};

/// PWM slices the RP2350A routes to pins; slices 8 to 11 exist only on the RP2350B.
pub const MAX_SLICES: usize = 8;

/// Channels of one PWM slice (A and B).
const CHANNELS_PER_SLICE: usize = 2;

/// Most LED channels the Pico 2 drives with hardware PWM.
const MAX_CHANNELS: usize = MAX_SLICES * CHANNELS_PER_SLICE;

/// PWM slices in use; the last one drives only channel A when `CHANNELS` is odd.
const SLICES: usize = CHANNELS.div_ceil(CHANNELS_PER_SLICE);

const _: () = assert!(
    CHANNELS <= MAX_CHANNELS,
    "the channel table must have at most 16 entries, the PWM outputs of the Pico 2"
);

/// Returns the PWM configuration of a running slice, with both compare values at zero.
#[must_use]
pub fn config() -> PwmConfig {
    let mut config = PwmConfig::default();
    // `set_config` writes the enable flag on every tick; it must keep a running slice running.
    config.enable = true;
    config.top = pwm::calculate_top(
        clocks::clk_sys_freq(),
        INSTALLATION.pwm_freq_hz,
        INSTALLATION.pwm_divider,
    );
    config.divider = INSTALLATION.pwm_divider.into();
    config
}

/// Routes the channels of PWM slice `index` that carry an LED and returns its driver, stopped,
/// with its LEDs dark and its counter at zero.
///
/// Returns `None` for a slice beyond `CHANNELS`; its pins are dropped unconfigured.
pub fn slice<S: Slice>(
    index: usize,
    slice: Peri<'static, S>,
    pin_a: Peri<'static, impl ChannelAPin<S>>,
    pin_b: Peri<'static, impl ChannelBPin<S>>,
    config: &PwmConfig,
) -> Option<Pwm<'static>> {
    let mut stopped = config.clone();
    stopped.enable = false;
    let first_channel = index * CHANNELS_PER_SLICE;
    if first_channel + 1 < CHANNELS {
        Some(Pwm::new_output_ab(slice, pin_a, pin_b, stopped))
    } else if first_channel < CHANNELS {
        Some(Pwm::new_output_a(slice, pin_a, stopped))
    } else {
        None
    }
}

/// The PWM slices of the LED channels, running with staggered phases.
pub struct Outputs {
    /// The slices in use, in channel order.
    slices: [Pwm<'static>; SLICES],
    /// Configuration of a running slice; only its compare values change from tick to tick.
    config: PwmConfig,
}

impl Outputs {
    /// Spreads the counters of the slices in use evenly over the PWM period and starts them in
    /// the same clock cycle, so their LEDs switch on at different instants of the period.
    ///
    /// `routed` holds the stopped slices from [`slice()`], in slice order; `config` is the
    /// configuration from [`config()`].
    ///
    /// # Panics
    ///
    /// Panics if a slice below `SLICES` is missing from `routed`.
    pub fn start(mut routed: [Option<Pwm<'static>>; MAX_SLICES], config: PwmConfig) -> Self {
        let slices: [Pwm<'static>; SLICES] = array::from_fn(|index| {
            routed[index]
                .take()
                .expect("every slice below SLICES drives at least one channel")
        });
        for (index, slice) in slices.iter().enumerate() {
            slice.set_counter(pwm::phase_offset(index, SLICES, config.top));
        }
        // One write to the shared enable register starts every slice in the same cycle, so the
        // offsets between the counters hold for as long as the slices run.
        PwmBatch::set_enabled(true, |batch| {
            for slice in &slices {
                batch.enable(slice);
            }
        });
        Self { slices, config }
    }

    /// Sets every channel to its intensity through the gamma curve.
    ///
    /// `set_config` writes both compare registers in one store and leaves the counter alone;
    /// `config` keeps the slice enabled and its divider and `top` unchanged, so the phases hold.
    pub fn show(&mut self, intensities: &[Intensity; CHANNELS]) {
        let top = self.config.top;
        let (pairs, lone) = intensities.as_chunks::<CHANNELS_PER_SLICE>();
        for (slice, &[intensity_a, intensity_b]) in self.slices.iter_mut().zip(pairs) {
            self.config.compare_a = pwm::duty_for(intensity_a, top);
            self.config.compare_b = pwm::duty_for(intensity_b, top);
            slice.set_config(&self.config);
        }
        // With an odd `CHANNELS`, the last slice drives only channel A.
        if let ([.., last_slice], &[intensity]) = (self.slices.as_mut_slice(), lone) {
            self.config.compare_a = pwm::duty_for(intensity, top);
            self.config.compare_b = 0;
            last_slice.set_config(&self.config);
        }
    }
}
