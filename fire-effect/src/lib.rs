//! Integer-only campfire flicker simulation.
//!
//! A [`FireProfile`] supplies a [`FireConfig`] at compile time. [`FireState`] is generic over the
//! profile, so the firmware folds every configuration-derived value into an immediate, while the
//! tests run the same code with other profiles on the host.

#![cfg_attr(not(test), no_std)]

mod config;
mod fire_state;
mod fixed_point;
pub mod pwm;

pub use config::{FireConfig, FireProfile};
pub use fire_state::FireState;
