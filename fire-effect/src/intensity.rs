// SPDX-FileCopyrightText: 2026 the Blaustein LED contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Perceived light intensity, the output of the simulation.

use crate::fixed_point::PERMILLE;

/// Perceived light intensity as a 16-bit fraction of full scale.
///
/// `0` is dark and `u16::MAX` is full scale. The scale follows what the eye sees, not the light
/// output: [`pwm::duty_for`](crate::pwm::duty_for) maps it through the square-law gamma curve to a
/// PWM compare value, so half of full scale gives a quarter of the light.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Intensity(u16);

impl Intensity {
    /// Full scale.
    pub const FULL: Self = Self(u16::MAX);
    /// Dark.
    pub const OFF: Self = Self(0);

    /// Wraps a raw intensity in steps of `1 / 65535` of full scale.
    #[must_use]
    pub const fn new(raw: u16) -> Self {
        Self(raw)
    }

    /// Returns `permille` thousandths of full scale, rounded to nearest.
    ///
    /// # Panics
    ///
    /// Panics if `permille` exceeds 1000. Evaluated in a const context, that is a build error.
    #[must_use]
    #[expect(
        clippy::cast_possible_truncation,
        reason = "`permille <= 1000` bounds the quotient by `u16::MAX`"
    )]
    pub const fn from_permille(permille: u16) -> Self {
        assert!(
            permille as u32 <= PERMILLE,
            "an intensity is at most 1000 permille of full scale"
        );
        Self(((permille as u32 * u16::MAX as u32 + PERMILLE / 2) / PERMILLE) as u16)
    }

    /// Returns the raw intensity in steps of `1 / 65535` of full scale.
    #[must_use]
    pub const fn get(self) -> u16 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    //! Construction of [`Intensity`].

    use super::Intensity;

    /// Permille values map to the ends of the scale and round to nearest in between.
    #[test]
    fn permille_spans_the_scale() {
        assert_eq!(
            Intensity::from_permille(0),
            Intensity::OFF,
            "0 permille is dark"
        );
        assert_eq!(
            Intensity::from_permille(1_000),
            Intensity::FULL,
            "1000 permille is full scale"
        );
        assert_eq!(
            Intensity::from_permille(500).get(),
            32_768,
            "500 permille rounds 32767.5 up"
        );
        let mut previous = Intensity::OFF;
        for permille in 1..=1_000 {
            let intensity = Intensity::from_permille(permille);
            assert!(intensity > previous, "intensity did not rise at {permille}");
            previous = intensity;
        }
    }

    /// More than 1000 permille is rejected.
    #[test]
    #[should_panic(expected = "an intensity is at most 1000 permille")]
    fn rejects_more_than_full_scale() {
        let _intensity = Intensity::from_permille(1_001);
    }
}
