// SPDX-FileCopyrightText: 2026 the Blaustein LED contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Configuration of the Burg Blaustein installation.

use fire_effect::{
    Channel, FireConfig, FollowerConfig, IgnitionConfig, Installation, InstallationConfig, Light,
    Lighting, LightingConfig,
};

/// PWM timing and update tick, shared by every channel.
pub const INSTALLATION: InstallationConfig = InstallationConfig::new();

/// The look of every role, the follower, the ignition after power-on and the channel table of the
/// first controller: base castle, keep and Vorburg.
///
/// The channel table has one entry per channel, in the pin order GPIO 16 to GPIO 22, then GPIO 7
/// to GPIO 15: `Channel::new(light, trim_permille)`. `light` is a role (`Light::Hearth`, `Torch`,
/// `Candle`, `Ember`, `Ambient`) or `Light::Follows(n)`, the glow of channel `n`, which must be an
/// earlier role. `trim_permille` evens out the LED: 1000 shows the role as configured, lower
/// values dim it. Every value derived from it is a compile-time constant, and an invalid table
/// fails the build.
pub const LIGHTING: LightingConfig = LightingConfig {
    hearth: FireConfig::HEARTH,
    torch: FireConfig::TORCH,
    candle: FireConfig::CANDLE,
    ember: FireConfig::EMBER,
    ambient: FireConfig::AMBIENT,
    glow: FollowerConfig::HEARTH_GLOW,
    ignition: IgnitionConfig {
        delay_ms: 400,
        fade_ms: 2_000,
    },
    channels: &[
        Channel::new(Light::Hearth, 1_000), // 0, GPIO 16: base castle, kitchen, cooking fire
        Channel::new(Light::Follows(0), 1_000), // 1, GPIO 17: base castle, kitchen, its embers
        Channel::new(Light::Hearth, 1_000), // 2, GPIO 18: base castle, hall, fireplace flame
        Channel::new(Light::Follows(2), 1_000), // 3, GPIO 19: base castle, hall, its embers
        Channel::new(Light::Candle, 1_000), // 4, GPIO 20: base castle, hall candles, bedroom
        Channel::new(Light::Hearth, 1_000), // 5, GPIO 21: keep, living room, fire
        Channel::new(Light::Follows(5), 1_000), // 6, GPIO 22: keep, living room, its embers
        Channel::new(Light::Candle, 1_000), // 7, GPIO 7: keep, entrance level, flame bowl
        Channel::new(Light::Torch, 1_000),  // 8, GPIO 8: base castle, courtyard
        Channel::new(Light::Torch, 1_000),  // 9, GPIO 9: keep, guard platform
        Channel::new(Light::Candle, 1_000), // 10, GPIO 10: Vorburg, candles
        Channel::new(Light::Torch, 1_000),  // 11, GPIO 11: Vorburg, stone gate, left
        Channel::new(Light::Torch, 1_000),  // 12, GPIO 12: Vorburg, stone gate, right
        Channel::new(Light::Torch, 1_000),  // 13, GPIO 13: Vorburg, wall walk of the palisade
    ],
};

const _: () = LIGHTING.validate(&INSTALLATION);

/// Number of LED channels, one per entry of the channel table.
///
/// Only these channels get a simulation, a PWM slice and RAM; the remaining pins stay
/// unconfigured.
pub const CHANNELS: usize = LIGHTING.channels.len();

/// The Burg Blaustein installation.
pub struct Blaustein;

impl Installation for Blaustein {
    const CONFIG: InstallationConfig = INSTALLATION;
}

impl Lighting for Blaustein {
    const LIGHTING: LightingConfig = LIGHTING;
}
