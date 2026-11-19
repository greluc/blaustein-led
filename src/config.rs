// SPDX-FileCopyrightText: 2026 the Blaustein LED contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Fire effect configuration of the Burg Blaustein installation.

use fire_effect::{FireConfig, FireProfile, Installation, InstallationConfig};

/// Number of LEDs, 1 to 16, in the pin order GPIO 16 to GPIO 22, then GPIO 7 to GPIO 15.
///
/// Only these channels get a simulation, a PWM slice and RAM; the remaining pins stay
/// unconfigured.
pub const CHANNELS: usize = 16;

/// PWM timing and update tick, shared by every channel.
pub const INSTALLATION: InstallationConfig = InstallationConfig::new();

/// Look of every channel.
///
/// A `const`, so the optimizer folds every derived quantity (glide rates, depths, gust chance)
/// into immediates.
pub const CFG: FireConfig = FireConfig::HEARTH;

const _: () = CFG.validate(&INSTALLATION);

/// The Burg Blaustein installation.
#[derive(Clone, Copy)]
pub struct Blaustein;

impl Installation for Blaustein {
    const CONFIG: InstallationConfig = INSTALLATION;
}

/// The flame look of every channel.
#[derive(Clone, Copy)]
pub struct Campfire;

impl FireProfile for Campfire {
    const CONFIG: FireConfig = CFG;
}
