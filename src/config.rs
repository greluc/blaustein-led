// SPDX-FileCopyrightText: 2026 the Blaustein LED contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Fire effect configuration of the Burg Blaustein installation.

use fire_effect::{FireConfig, FireProfile};

/// Number of LEDs, 1 to 16, in the pin order GPIO 16 to GPIO 22, then GPIO 7 to GPIO 15.
///
/// Only these channels get a simulation, a PWM slice and RAM; the remaining pins stay
/// unconfigured.
pub const CHANNELS: usize = 16;

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

const _: () = CFG.validate();

/// The flame look installed at Burg Blaustein.
#[derive(Clone, Copy)]
pub struct Blaustein;

impl FireProfile for Blaustein {
    const CONFIG: FireConfig = CFG;
}
