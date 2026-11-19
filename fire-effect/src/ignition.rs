// SPDX-FileCopyrightText: 2026 the Blaustein LED contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Staggered ignition: after power-on the channels light one after another, each fading in.

use crate::intensity::Intensity;

/// Longest fade in ticks, so that an intensity times the ticks elapsed in a fade fits in `u32`.
const MAX_FADE_TICKS: u32 = 1 << 16;

/// How the channels light after power-on.
///
/// Channel `n` (counted from 0) stays dark for `n * delay_ms`, then its intensity rises linearly
/// to its full output within `fade_ms`. A follower comes later in the table than the channel it
/// follows, so it never glows before its flame. Both times are rounded down to whole ticks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IgnitionConfig {
    /// Time from the start of one channel to the start of the next in milliseconds; 0 lights
    /// every channel at once.
    pub delay_ms: u32,
    /// Time in which a channel fades in from dark in milliseconds, at least one tick.
    pub fade_ms: u32,
}

impl IgnitionConfig {
    /// Checks the timing for `channels` channels at a tick of `tick_ms`, which must be greater
    /// than 0.
    ///
    /// # Panics
    ///
    /// Panics with a description of the first violated constraint. Evaluated in a const context,
    /// that is a build error.
    pub const fn validate(&self, tick_ms: u32, channels: usize) {
        let fade = self.fade_ms / tick_ms;
        assert!(fade > 0, "ignition.fade_ms must span at least one tick");
        assert!(
            fade <= MAX_FADE_TICKS,
            "ignition.fade_ms is too long for the tick: a fade spans at most 65536 ticks"
        );
        let last_start = (channels as u64).saturating_sub(1) * (self.delay_ms / tick_ms) as u64;
        assert!(
            last_start + fade as u64 <= u32::MAX as u64,
            "ignition.delay_ms is too long for this many channels: the last channel would light \
             after the tick counter saturates"
        );
    }
}

/// Per-tick constants of an [`IgnitionConfig`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Ignition {
    /// Ticks from the start of one channel to the start of the next.
    delay: u32,
    /// Ticks of a fade, within `1..=MAX_FADE_TICKS`.
    fade: u32,
}

impl Ignition {
    /// Derives the per-tick constants of `config` for `channels` channels at a tick of `tick_ms`.
    ///
    /// # Panics
    ///
    /// Panics if `config` is invalid, see [`IgnitionConfig::validate`]. Evaluated in a const
    /// context, that is a build error.
    #[must_use]
    pub const fn new(config: IgnitionConfig, tick_ms: u32, channels: usize) -> Self {
        config.validate(tick_ms, channels);
        Self {
            delay: config.delay_ms / tick_ms,
            fade: config.fade_ms / tick_ms,
        }
    }

    /// Returns `intensity` scaled by the envelope of `channel` at `tick` ticks after power-on.
    ///
    /// The envelope is dark up to and including the channel's start tick, `channel * delay`,
    /// rises linearly from there, truncated, and passes `intensity` unchanged from `fade` ticks
    /// after the start on.
    #[must_use]
    #[expect(
        clippy::cast_possible_truncation,
        reason = "within a fade `elapsed < fade <= 65536`, so `elapsed` fits in u32, the product \
                  in u32 and the quotient, at most `intensity`, in u16"
    )]
    pub const fn apply(self, channel: usize, tick: u32, intensity: Intensity) -> Intensity {
        // In 64 bits, so the start of a channel beyond the table cannot overflow.
        let start = channel as u64 * self.delay as u64;
        let elapsed = (tick as u64).saturating_sub(start);
        if elapsed >= self.fade as u64 {
            intensity
        } else {
            Intensity::new((intensity.get() as u32 * elapsed as u32 / self.fade) as u16)
        }
    }
}

#[cfg(test)]
mod tests {
    //! Timing, shape and validation of the ignition.

    use super::{Ignition, IgnitionConfig};
    use crate::intensity::Intensity;

    /// Tick of the tests in milliseconds.
    const TICK_MS: u32 = 15;

    /// Channels of a full bank.
    const CHANNELS: usize = 16;

    /// A start every 20 ticks and a fade of 40 ticks; the odd milliseconds round down.
    const CONFIG: IgnitionConfig = IgnitionConfig {
        delay_ms: 20 * TICK_MS + 14,
        fade_ms: 40 * TICK_MS + 1,
    };

    /// Ticks from the start of one channel to the start of the next in [`CONFIG`].
    const DELAY_TICKS: u32 = 20;

    /// Ticks of a fade in [`CONFIG`].
    const FADE_TICKS: u32 = 40;

    /// Each channel stays dark up to its start tick, then rises monotonically along the linear
    /// envelope and reaches its full output after the fade.
    #[test]
    fn channels_light_in_turn_and_fade_in() {
        let ignition = Ignition::new(CONFIG, TICK_MS, CHANNELS);
        let full = Intensity::new(60_000);
        for (channel, start) in (0..CHANNELS).zip((0..).step_by(DELAY_TICKS as usize)) {
            let mut previous = Intensity::OFF;
            for tick in 0..=start + FADE_TICKS + 5 {
                let output = ignition.apply(channel, tick, full);
                let elapsed = tick.saturating_sub(start).min(FADE_TICKS);
                let expected = u32::from(full.get()) * elapsed / FADE_TICKS;
                assert_eq!(
                    u32::from(output.get()),
                    expected,
                    "channel {channel} at tick {tick}"
                );
                assert!(
                    output >= previous,
                    "channel {channel} dimmed at tick {tick}"
                );
                previous = output;
            }
            assert_eq!(
                previous, full,
                "channel {channel} never reached full output"
            );
        }
    }

    /// A later channel is never brighter than an earlier one during the ignition, so a follower
    /// never glows before the channel it follows.
    #[test]
    fn later_channels_never_lead() {
        let ignition = Ignition::new(CONFIG, TICK_MS, CHANNELS);
        let channels = u32::try_from(CHANNELS).expect("16 channels fit in u32");
        let last_lit = (channels - 1) * DELAY_TICKS + FADE_TICKS;
        for tick in 0..=last_lit {
            for channel in 1..CHANNELS {
                let earlier = ignition.apply(channel - 1, tick, Intensity::FULL);
                let later = ignition.apply(channel, tick, Intensity::FULL);
                assert!(
                    later <= earlier,
                    "channel {channel} leads channel {} at tick {tick}",
                    channel - 1
                );
            }
        }
    }

    /// Without a delay every channel fades in at once.
    #[test]
    fn zero_delay_lights_every_channel_at_once() {
        let ignition = Ignition::new(
            IgnitionConfig {
                delay_ms: 0,
                ..CONFIG
            },
            TICK_MS,
            CHANNELS,
        );
        for tick in 0..=FADE_TICKS {
            let first = ignition.apply(0, tick, Intensity::FULL);
            for channel in 1..CHANNELS {
                assert_eq!(
                    ignition.apply(channel, tick, Intensity::FULL),
                    first,
                    "channel {channel} differs from channel 0 at tick {tick}"
                );
            }
        }
    }

    /// A fade shorter than a tick is rejected.
    #[test]
    #[should_panic(expected = "ignition.fade_ms must span at least one tick")]
    fn rejects_instant_fade() {
        IgnitionConfig {
            fade_ms: TICK_MS - 1,
            ..CONFIG
        }
        .validate(TICK_MS, CHANNELS);
    }

    /// A fade longer than 65536 ticks is rejected; 65536 ticks pass.
    #[test]
    #[should_panic(expected = "ignition.fade_ms is too long for the tick")]
    fn rejects_endless_fade() {
        let longest = IgnitionConfig {
            fade_ms: 65_536 * TICK_MS,
            ..CONFIG
        };
        longest.validate(TICK_MS, CHANNELS);
        IgnitionConfig {
            fade_ms: 65_537 * TICK_MS,
            ..longest
        }
        .validate(TICK_MS, CHANNELS);
    }

    /// A delay that would light the last channel after the tick counter saturates is rejected.
    #[test]
    #[should_panic(expected = "ignition.delay_ms is too long for this many channels")]
    fn rejects_endless_ignition() {
        IgnitionConfig {
            delay_ms: u32::MAX,
            fade_ms: TICK_MS,
        }
        .validate(1, CHANNELS);
    }
}
