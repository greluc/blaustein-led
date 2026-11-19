// SPDX-FileCopyrightText: 2026 the Blaustein LED contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The identify pattern: every channel blinks its own number, so wires can be matched to
//! channels during the installation.

use core::array;
use core::marker::PhantomData;

use crate::installation::Installation;
use crate::intensity::Intensity;

/// Time a flash is lit in milliseconds.
const FLASH_MS: u32 = 150;

/// Dark time between two flashes of a channel in milliseconds.
const GAP_MS: u32 = 250;

/// Dark time after the flashes of the last channel, before the pattern repeats, in milliseconds.
const PAUSE_MS: u32 = 2_000;

/// Timing of the identify pattern in ticks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Pattern {
    /// Ticks a flash is lit, at least one.
    flash: u32,
    /// Ticks from the start of one flash to the start of the next, more than `flash`.
    period: u32,
    /// Ticks of one repetition: a period per channel and the pause.
    cycle: u32,
}

impl Pattern {
    /// Derives the timing for `channels` channels at a tick of `tick_ms`, which must be greater
    /// than 0.
    ///
    /// # Panics
    ///
    /// Panics if a tick is too long for the flashes or the gaps between them, or a repetition
    /// does not fit in `u32` ticks. Evaluated in a const context, that is a build error.
    #[expect(
        clippy::cast_possible_truncation,
        reason = "the cycle is checked to fit in u32 before the cast"
    )]
    const fn new(tick_ms: u32, channels: usize) -> Self {
        let flash = FLASH_MS / tick_ms;
        let gap = GAP_MS / tick_ms;
        assert!(
            flash > 0 && gap > 0,
            "tick_ms must be at most 150 for the identify pattern, or its flashes vanish or merge"
        );
        let period = flash + gap;
        let cycle = channels as u64 * period as u64 + (PAUSE_MS / tick_ms) as u64;
        assert!(
            cycle <= u32::MAX as u64,
            "the identify pattern of this many channels does not fit in the tick counter"
        );
        Self {
            flash,
            period,
            cycle: cycle as u32,
        }
    }

    /// Returns whether `channel`, counted from 0, is lit `tick` ticks into a repetition.
    ///
    /// Channel `n` flashes `n + 1` times from the start of the repetition, then stays dark.
    const fn is_lit(self, channel: usize, tick: u32) -> bool {
        (tick / self.period) as usize <= channel && tick % self.period < self.flash
    }
}

/// Every channel of an installation `S`, `N` of them, blinks its own number.
///
/// Channel `n`, counted from 0, flashes `n + 1` times at full scale, then stays dark until the
/// pattern repeats. All channels start every repetition together, so their flashes line up and
/// can be counted side by side.
#[derive(Clone, Copy, Debug)]
pub struct Identify<S, const N: usize> {
    /// Ticks into the current repetition.
    tick: u32,
    /// Binds the pattern to the tick of its installation.
    installation: PhantomData<S>,
}

impl<S: Installation, const N: usize> Identify<S, N> {
    /// Timing in ticks; an invalid installation fails the build where it is used.
    const PATTERN: Pattern = {
        S::CONFIG.validate();
        Pattern::new(S::CONFIG.tick_ms, N)
    };

    /// Starts the pattern at the first flash of every channel.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            tick: 0,
            installation: PhantomData,
        }
    }

    /// Advances the pattern by one tick and returns the intensity of every channel.
    pub fn update(&mut self) -> [Intensity; N] {
        let pattern = Self::PATTERN;
        let tick = self.tick;
        self.tick = (tick + 1) % pattern.cycle;
        array::from_fn(|channel| {
            if pattern.is_lit(channel, tick) {
                Intensity::FULL
            } else {
                Intensity::OFF
            }
        })
    }
}

impl<S: Installation, const N: usize> Default for Identify<S, N> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    //! Timing of the identify pattern.

    use super::{Identify, Pattern};
    use crate::intensity::Intensity;
    use crate::test_support::Site;

    /// Channels of a full bank.
    const CHANNELS: usize = 16;

    /// Returns the outputs of every channel over `ticks` ticks, one series per channel.
    fn record(ticks: u32) -> Vec<Vec<bool>> {
        let mut identify = Identify::<Site, CHANNELS>::new();
        let mut series = vec![Vec::new(); CHANNELS];
        for _ in 0..ticks {
            for (outputs, output) in series.iter_mut().zip(identify.update()) {
                assert!(
                    output == Intensity::FULL || output == Intensity::OFF,
                    "identify must switch between dark and full scale, not {output:?}"
                );
                outputs.push(output == Intensity::FULL);
            }
        }
        series
    }

    /// Returns the lengths of the lit runs of `series`.
    fn flashes(series: &[bool]) -> Vec<usize> {
        series
            .split(|&lit| !lit)
            .map(<[bool]>::len)
            .filter(|&length| length > 0)
            .collect()
    }

    /// At a 15 ms tick a flash lasts 10 ticks, a period 26 ticks and a repetition of 16 channels
    /// 549 ticks, about 8.2 s.
    #[test]
    fn timing_follows_the_tick() {
        assert_eq!(
            Pattern::new(15, CHANNELS),
            Pattern {
                flash: 10,
                period: 26,
                cycle: 16 * 26 + 133,
            },
            "timing of the pattern at 15 ms"
        );
    }

    /// In every repetition channel `n` flashes `n + 1` times with the same length and every
    /// channel starts together.
    #[test]
    fn every_channel_blinks_its_number() {
        let pattern = Pattern::new(15, CHANNELS);
        let series = record(pattern.cycle);
        let flash = usize::try_from(pattern.flash).expect("a flash fits in usize");
        for (channel, outputs) in series.iter().enumerate() {
            assert!(outputs[0], "channel {channel} is dark at the start");
            assert_eq!(
                flashes(outputs),
                vec![flash; channel + 1],
                "flashes of channel {channel}"
            );
        }
    }

    /// The pattern repeats after one cycle and ends each repetition with a dark pause.
    #[test]
    fn pattern_repeats_after_a_pause() {
        let pattern = Pattern::new(15, CHANNELS);
        let cycle = usize::try_from(pattern.cycle).expect("a cycle fits in usize");
        let pause = usize::try_from(2_000 / 15_u32).expect("a pause fits in usize");
        let series = record(3 * pattern.cycle);
        for (channel, outputs) in series.iter().enumerate() {
            assert_eq!(
                outputs[..cycle],
                outputs[cycle..2 * cycle],
                "channel {channel} changed between repetitions"
            );
            assert!(
                outputs[cycle - pause..cycle].iter().all(|&lit| !lit),
                "channel {channel} is lit during the pause"
            );
        }
    }

    /// A tick too long for the flashes is rejected.
    #[test]
    #[should_panic(expected = "tick_ms must be at most 150 for the identify pattern")]
    fn rejects_long_tick() {
        let _pattern = Pattern::new(151, CHANNELS);
    }
}
