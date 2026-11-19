// SPDX-FileCopyrightText: 2026 the Blaustein LED contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The channel table: what drives each LED channel and how its LED is trimmed.

use crate::fixed_point::{PERMILLE, permille_of};
use crate::intensity::Intensity;

/// What drives a channel: one of the light roles, or another channel.
///
/// The roles take their look from the [`LightingConfig`](crate::LightingConfig) of the
/// installation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Light {
    /// A fireplace or a cooking fire.
    Hearth,
    /// A torch.
    Torch,
    /// A candle or an oil lamp.
    Candle,
    /// Embers or a forge.
    Ember,
    /// Room light.
    Ambient,
    /// The glow of the installation's follower, driven by the channel with this index.
    ///
    /// That channel must come earlier in the table, so it is updated first within a tick, and
    /// must not be a follower itself.
    Follows(usize),
}

/// One LED channel: what drives it and the trim of its LED.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Channel {
    /// What drives the channel.
    light: Light,
    /// Share of the output that reaches the LED, in thousandths, within `1..=1000`.
    trim_permille: u16,
}

impl Channel {
    /// Returns a channel driven by `light` whose output is scaled to `trim_permille` thousandths.
    ///
    /// The trim evens out LEDs of the same type without touching the role: 1000 shows the role as
    /// configured, lower values dim the LED without changing how it moves. It scales the
    /// intensity, so 500 looks about half as bright and gives a quarter of the light output.
    ///
    /// # Panics
    ///
    /// Panics if `trim_permille` is 0 or above 1000. Evaluated in a const context, that is a
    /// build error that points at the entry.
    #[must_use]
    pub const fn new(light: Light, trim_permille: u16) -> Self {
        assert!(
            trim_permille > 0,
            "trim_permille must be greater than 0, or the LED stays dark"
        );
        assert!(
            trim_permille as u32 <= PERMILLE,
            "trim_permille must be at most 1000: a trim only dims a role; raise the role's peak \
             for more light"
        );
        Self {
            light,
            trim_permille,
        }
    }

    /// Returns what drives the channel.
    #[must_use]
    pub(crate) const fn light(self) -> Light {
        self.light
    }

    /// Returns `intensity` scaled by the trim, truncated; a trim of 1000 returns it unchanged.
    #[must_use]
    #[expect(
        clippy::cast_possible_truncation,
        reason = "a trim of at most 1000 permille keeps the result at most `intensity`"
    )]
    pub(crate) const fn trim(self, intensity: Intensity) -> Intensity {
        Intensity::new(permille_of(intensity.get(), self.trim_permille) as u16)
    }
}

/// Checks that `channels` is not empty and that every follower follows an earlier channel that
/// is not a follower itself.
///
/// # Panics
///
/// Panics with a description of the first violated constraint. Evaluated in a const context,
/// that is a build error.
pub const fn validate_table(channels: &[Channel]) {
    assert!(
        !channels.is_empty(),
        "the channel table needs at least one channel"
    );
    let mut index = 0;
    while index < channels.len() {
        if let Light::Follows(source) = channels[index].light {
            assert!(source != index, "a channel cannot follow itself");
            assert!(
                source < channels.len(),
                "a channel follows a channel beyond the end of the table"
            );
            assert!(
                source < index,
                "a channel can only follow an earlier channel, which is updated before it within \
                 a tick"
            );
            assert!(
                !matches!(channels[source].light, Light::Follows(_)),
                "a channel cannot follow a follower; let it follow the flame directly"
            );
        }
        index += 1;
    }
}

#[cfg(test)]
mod tests {
    //! Trim of a [`Channel`] and validation of a channel table.

    use super::{Channel, Light, validate_table};
    use crate::intensity::Intensity;

    /// Returns every intensity from dark to full scale.
    fn every_intensity() -> impl Iterator<Item = Intensity> {
        (0..=u16::MAX).map(Intensity::new)
    }

    /// A trim of 1000 passes every intensity unchanged.
    #[test]
    fn full_trim_is_the_identity() {
        let channel = Channel::new(Light::Hearth, 1_000);
        for intensity in every_intensity() {
            assert_eq!(
                channel.trim(intensity),
                intensity,
                "a trim of 1000 changed {intensity:?}"
            );
        }
    }

    /// A trim scales every intensity by its share, truncated, without exceeding it, and keeps
    /// the order of intensities.
    #[test]
    fn trim_scales_by_its_share() {
        for trim_permille in [1, 250, 500, 999] {
            let channel = Channel::new(Light::Candle, trim_permille);
            let mut previous = Intensity::OFF;
            for intensity in every_intensity() {
                let trimmed = channel.trim(intensity);
                let expected = u32::from(intensity.get()) * u32::from(trim_permille) / 1_000;
                assert_eq!(
                    u32::from(trimmed.get()),
                    expected,
                    "trim {trim_permille} of {intensity:?}"
                );
                assert!(
                    trimmed >= previous,
                    "trim {trim_permille} reversed the order at {intensity:?}"
                );
                previous = trimmed;
            }
        }
    }

    /// A trim of 0 is rejected.
    #[test]
    #[should_panic(expected = "trim_permille must be greater than 0")]
    fn rejects_dark_trim() {
        let _channel = Channel::new(Light::Torch, 0);
    }

    /// A trim above 1000 is rejected.
    #[test]
    #[should_panic(expected = "trim_permille must be at most 1000")]
    fn rejects_boosting_trim() {
        let _channel = Channel::new(Light::Torch, 1_001);
    }

    /// Roles and followers of earlier roles in any order are valid.
    #[test]
    fn accepts_followers_of_earlier_roles() {
        validate_table(&[
            Channel::new(Light::Hearth, 1_000),
            Channel::new(Light::Follows(0), 1_000),
            Channel::new(Light::Torch, 500),
            Channel::new(Light::Candle, 1),
            Channel::new(Light::Follows(2), 1_000),
            Channel::new(Light::Follows(0), 800),
            Channel::new(Light::Ember, 1_000),
            Channel::new(Light::Ambient, 1_000),
        ]);
    }

    /// An empty table is rejected.
    #[test]
    #[should_panic(expected = "needs at least one channel")]
    fn rejects_empty_table() {
        validate_table(&[]);
    }

    /// A channel that follows itself is rejected.
    #[test]
    #[should_panic(expected = "cannot follow itself")]
    fn rejects_self_follower() {
        validate_table(&[
            Channel::new(Light::Hearth, 1_000),
            Channel::new(Light::Follows(1), 1_000),
        ]);
    }

    /// A channel that follows a later channel is rejected.
    #[test]
    #[should_panic(expected = "can only follow an earlier channel")]
    fn rejects_follower_of_later_channel() {
        validate_table(&[
            Channel::new(Light::Follows(1), 1_000),
            Channel::new(Light::Hearth, 1_000),
        ]);
    }

    /// A channel that follows a channel beyond the table is rejected.
    #[test]
    #[should_panic(expected = "beyond the end of the table")]
    fn rejects_follower_out_of_range() {
        validate_table(&[
            Channel::new(Light::Hearth, 1_000),
            Channel::new(Light::Follows(2), 1_000),
        ]);
    }

    /// A channel that follows a follower is rejected.
    #[test]
    #[should_panic(expected = "cannot follow a follower")]
    fn rejects_follower_of_follower() {
        validate_table(&[
            Channel::new(Light::Hearth, 1_000),
            Channel::new(Light::Follows(0), 1_000),
            Channel::new(Light::Follows(1), 1_000),
        ]);
    }
}
