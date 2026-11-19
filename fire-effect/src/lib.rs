// SPDX-FileCopyrightText: 2026 the Blaustein LED contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Integer-only campfire flicker simulation.
//!
//! An [`Installation`] supplies the settings every light shares ([`InstallationConfig`]: PWM
//! timing and the tick); a [`FireProfile`] supplies the look of one light role ([`FireConfig`]).
//! [`FireState`] is generic over both, so the firmware folds every derived value into an
//! immediate, while the tests run the same code with other profiles on the host. Presets cover
//! the roles of a castle model: [`FireConfig::HEARTH`], [`FireConfig::TORCH`],
//! [`FireConfig::CANDLE`], [`FireConfig::EMBER`] and [`FireConfig::AMBIENT`].
//!
//! Each channel sums three layers of smoothly interpolated value noise (a slow swell, a faster
//! sway and a fast flicker) and occasional gusts, each moving at random times drawn from the
//! channel's own seed. No layer has a period and no two channels share a phase, so the lights
//! never move in lockstep. The result is a 16-bit perceived [`Intensity`] that never exceeds the
//! role's `peak`; [`pwm::duty_for`] maps it through a square-law gamma curve to a PWM compare
//! value.
//!
//! A [`Follower`] derives a light from another channel, such as the red glow of the embers under
//! a flame, configured by a [`FollowerProfile`].
//!
//! A [`Lighting`] installation adds a [`LightingConfig`]: the look of every role, the follower,
//! the staggered [`IgnitionConfig`] after power-on and the table of [`Channel`]s, each driven by
//! a [`Light`] and trimmed to its LED. [`Lights`] runs every channel of it; [`Identify`] blinks
//! each channel's number instead, to match wires to channels.
//!
//! [`picotool_reset`] decodes the requests of the reset interface through which picotool reboots
//! the running firmware into the USB bootloader, for the `usb-update` feature of the firmware.

#![cfg_attr(not(test), no_std)]

mod channel;
mod config;
mod fire_state;
mod fixed_point;
mod follower;
mod identify;
mod ignition;
mod installation;
mod intensity;
mod lighting;
mod noise;
pub mod picotool_reset;
pub mod pwm;
mod rng;
mod roles;
#[cfg(test)]
mod test_support;
mod tuning;

pub use channel::{Channel, Light};
pub use config::{FireConfig, FireProfile, Gust, Layer};
pub use fire_state::FireState;
pub use follower::{Follower, FollowerConfig, FollowerProfile};
pub use identify::Identify;
pub use ignition::IgnitionConfig;
pub use installation::{Installation, InstallationConfig};
pub use intensity::Intensity;
pub use lighting::{Lighting, LightingConfig, Lights};
pub use rng::channel_seed;
