// SPDX-FileCopyrightText: 2026 the Blaustein LED contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The effect this build shows: the fire of the channel table, or with the `identify` feature
//! every channel blinking its own number.

#[cfg(not(feature = "identify"))]
use embassy_rp::clocks::RoscRng;
#[cfg(feature = "identify")]
use fire_effect::Identify;
#[cfg(not(feature = "identify"))]
use fire_effect::Lights;

use crate::config::{Blaustein, CHANNELS};

/// The fire of every channel, as set in [`LIGHTING`](crate::config::LIGHTING).
#[cfg(not(feature = "identify"))]
pub type Effect = Lights<Blaustein, CHANNELS>;

/// Every channel blinks its own 1-based number, to match wires to channels.
#[cfg(feature = "identify")]
pub type Effect = Identify<Blaustein, CHANNELS>;

/// Starts the fire with 32 random bits mixed into the seeds, so every power-on looks different.
///
/// The bits come from 32 reads of the ring oscillator's random bit, a fixed number of register
/// reads that cannot block or fail. If the oscillator were stopped, the bits would be constant
/// and the seeds would still be valid and distinct, only the same at every power-on.
#[cfg(not(feature = "identify"))]
pub fn start() -> Effect {
    Lights::new(RoscRng.next_u32())
}

/// Starts the identify pattern at the first flash of every channel.
#[cfg(feature = "identify")]
pub const fn start() -> Effect {
    Identify::new()
}
