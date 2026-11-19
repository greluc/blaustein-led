// SPDX-FileCopyrightText: 2026 the Blaustein LED contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Settings that every light of an installation shares.

/// An installation fixed at compile time.
pub trait Installation {
    /// The settings. [`FireState`](crate::FireState) and [`Follower`](crate::Follower) reject
    /// invalid ones at compile time.
    const CONFIG: InstallationConfig;
}

/// PWM timing and update tick, shared by every light of an installation.
///
/// They are kept apart from the look of a role ([`FireConfig`](crate::FireConfig)), so two roles
/// cannot disagree on them. Override fields of [`InstallationConfig::new`] with struct update
/// syntax.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InstallationConfig {
    /// PWM target frequency in Hz. 20 to 30 kHz keeps PWM inaudible and flicker-free.
    pub pwm_freq_hz: u32,
    /// Integer PWM clock divider, at least 1. Use the smallest value that keeps `top` within
    /// `u16`.
    pub pwm_divider: u8,
    /// Update interval in milliseconds, at least 1. The role presets need at most 20.
    pub tick_ms: u32,
}

impl InstallationConfig {
    /// Returns the baseline settings: 25 kHz PWM, no clock division and a 15 ms tick.
    ///
    /// A `const fn` rather than only `Default`, because installations are built in const
    /// contexts.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            pwm_freq_hz: 25_000,
            pwm_divider: 1,
            tick_ms: 15,
        }
    }

    /// Checks that the settings produce working PWM and timing.
    ///
    /// # Panics
    ///
    /// Panics with a description of the first violated constraint. Evaluated in a const context,
    /// that is a build error.
    pub const fn validate(&self) {
        assert!(
            self.pwm_divider >= 1,
            "pwm_divider must be at least 1, 0 would divide by zero"
        );
        assert!(self.pwm_freq_hz > 0, "pwm_freq_hz must be greater than 0");
        assert!(self.tick_ms > 0, "tick_ms must be greater than 0");
    }
}

impl Default for InstallationConfig {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    //! Validation of [`InstallationConfig`].

    use super::InstallationConfig;

    /// The baseline settings are valid.
    #[test]
    fn baseline_is_valid() {
        InstallationConfig::new().validate();
        assert_eq!(
            InstallationConfig::default(),
            InstallationConfig::new(),
            "Default must match new()"
        );
    }

    /// A zero clock divider is rejected.
    #[test]
    #[should_panic(expected = "pwm_divider must be at least 1")]
    fn rejects_zero_divider() {
        InstallationConfig {
            pwm_divider: 0,
            ..InstallationConfig::new()
        }
        .validate();
    }

    /// A zero PWM frequency is rejected.
    #[test]
    #[should_panic(expected = "pwm_freq_hz must be greater than 0")]
    fn rejects_zero_frequency() {
        InstallationConfig {
            pwm_freq_hz: 0,
            ..InstallationConfig::new()
        }
        .validate();
    }

    /// A zero tick is rejected.
    #[test]
    #[should_panic(expected = "tick_ms must be greater than 0")]
    fn rejects_zero_tick() {
        InstallationConfig {
            tick_ms: 0,
            ..InstallationConfig::new()
        }
        .validate();
    }
}
