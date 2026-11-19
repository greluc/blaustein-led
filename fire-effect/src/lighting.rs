// SPDX-FileCopyrightText: 2026 the Blaustein LED contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The lights of an installation, from the look of every role to the channel table, and the bank
//! that runs every channel.

use core::array;
use core::marker::PhantomData;

use crate::channel::{Channel, Light, validate_table};
use crate::config::{FireConfig, FireProfile};
use crate::fire_state::FireState;
use crate::follower::{Follower, FollowerConfig, FollowerProfile};
use crate::ignition::{Ignition, IgnitionConfig};
use crate::installation::{Installation, InstallationConfig};
use crate::intensity::Intensity;
use crate::rng::boot_seed;

/// The lights of an installation fixed at compile time.
pub trait Lighting: Installation {
    /// The lights. [`Lights`] rejects invalid ones at compile time.
    const LIGHTING: LightingConfig;
}

/// The lights of an installation: the look of every role, the follower, the ignition after
/// power-on and the channel table.
///
/// Every look is validated, whether a channel uses it or not, so switching a channel to another
/// role never uncovers an invalid look.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LightingConfig {
    /// Look of every [`Light::Hearth`] channel.
    pub hearth: FireConfig,
    /// Look of every [`Light::Torch`] channel.
    pub torch: FireConfig,
    /// Look of every [`Light::Candle`] channel.
    pub candle: FireConfig,
    /// Look of every [`Light::Ember`] channel.
    pub ember: FireConfig,
    /// Look of every [`Light::Ambient`] channel.
    pub ambient: FireConfig,
    /// Parameters of every [`Light::Follows`] channel.
    pub glow: FollowerConfig,
    /// How the channels light after power-on.
    pub ignition: IgnitionConfig,
    /// One entry per LED channel, in the order of the outputs.
    pub channels: &'static [Channel],
}

impl LightingConfig {
    /// Checks every look, the follower, the channel table and the ignition at the tick of
    /// `installation`, and checks the installation.
    ///
    /// # Panics
    ///
    /// Panics with a description of the first violated constraint. Evaluated in a const context,
    /// that is a build error.
    pub const fn validate(&self, installation: &InstallationConfig) {
        self.hearth.validate(installation);
        self.torch.validate(installation);
        self.candle.validate(installation);
        self.ember.validate(installation);
        self.ambient.validate(installation);
        self.glow.validate(installation);
        validate_table(self.channels);
        self.ignition
            .validate(installation.tick_ms, self.channels.len());
    }
}

/// Defines a profile type that takes its look from a field of the [`LightingConfig`] of `L`.
macro_rules! role_look {
    ($(#[$doc:meta])* $name:ident, $field:ident) => {
        $(#[$doc])*
        struct $name<L>(PhantomData<L>);

        impl<L: Lighting> FireProfile for $name<L> {
            const CONFIG: FireConfig = L::LIGHTING.$field;
        }
    };
}

role_look!(
    /// The hearth look of the installation `L`.
    HearthLook,
    hearth
);
role_look!(
    /// The torch look of the installation `L`.
    TorchLook,
    torch
);
role_look!(
    /// The candle look of the installation `L`.
    CandleLook,
    candle
);
role_look!(
    /// The ember look of the installation `L`.
    EmberLook,
    ember
);
role_look!(
    /// The ambient look of the installation `L`.
    AmbientLook,
    ambient
);

/// The follower of the installation `L`.
struct GlowLook<L>(PhantomData<L>);

impl<L: Lighting> FollowerProfile for GlowLook<L> {
    const CONFIG: FollowerConfig = L::LIGHTING.glow;
}

/// State of one channel.
enum State<L> {
    /// A [`Light::Hearth`] channel.
    Hearth(FireState<L, HearthLook<L>>),
    /// A [`Light::Torch`] channel.
    Torch(FireState<L, TorchLook<L>>),
    /// A [`Light::Candle`] channel.
    Candle(FireState<L, CandleLook<L>>),
    /// A [`Light::Ember`] channel.
    Ember(FireState<L, EmberLook<L>>),
    /// A [`Light::Ambient`] channel.
    Ambient(FireState<L, AmbientLook<L>>),
    /// A [`Light::Follows`] channel and the index of the channel it follows.
    Glow(Follower<L, GlowLook<L>>, usize),
}

impl<L: Lighting> State<L> {
    /// Starts a channel driven by `light`; a role draws its randomness from `seed`.
    ///
    /// # Panics
    ///
    /// Panics if `light` is a role and `seed` is zero.
    const fn new(light: Light, seed: u32) -> Self {
        match light {
            Light::Hearth => Self::Hearth(FireState::new(seed)),
            Light::Torch => Self::Torch(FireState::new(seed)),
            Light::Candle => Self::Candle(FireState::new(seed)),
            Light::Ember => Self::Ember(FireState::new(seed)),
            Light::Ambient => Self::Ambient(FireState::new(seed)),
            Light::Follows(source) => Self::Glow(Follower::new(), source),
        }
    }

    /// Advances the channel by one tick and returns its intensity.
    ///
    /// `earlier` holds this tick's outputs of the channels before this one, which a follower
    /// reads its source from.
    fn update(&mut self, earlier: &[Intensity]) -> Intensity {
        match self {
            Self::Hearth(flame) => flame.update(),
            Self::Torch(flame) => flame.update(),
            Self::Candle(flame) => flame.update(),
            Self::Ember(flame) => flame.update(),
            Self::Ambient(flame) => flame.update(),
            Self::Glow(glow, source) => glow.update(earlier[*source]),
        }
    }
}

/// Every channel of an installation `L`, `N` of them: its role or follower, the ignition after
/// power-on and its trim.
///
/// Each tick, the channels advance in table order. A follower reads the output of its source
/// after the ignition and before the trim, so its embers heat up with the flame as it lights,
/// and the trim of the flame's LED does not reach it. A follower comes later in the table than
/// its source, so its own ignition starts later and it never glows before its flame.
pub struct Lights<L, const N: usize> {
    /// State of every channel, in table order.
    channels: [State<L>; N],
    /// Ticks since power-on, saturating; paces the ignition.
    tick: u32,
}

impl<L: Lighting, const N: usize> Lights<L, N> {
    /// Per-tick constants of the ignition; an invalid installation fails the build where it is
    /// used.
    const IGNITION: Ignition = Ignition::new(L::LIGHTING.ignition, L::CONFIG.tick_ms, N);
    /// Checks the installation and that `N` matches its channel table.
    const VALID: () = {
        L::LIGHTING.validate(&L::CONFIG);
        assert!(
            N == L::LIGHTING.channels.len(),
            "N must be the number of entries of the channel table"
        );
    };

    /// Starts every channel dark, with `entropy` mixed into the seeds of the roles.
    ///
    /// Every value of `entropy` gives valid seeds that differ between the channels; zero gives
    /// the compile-time seeds of [`channel_seed`](crate::channel_seed). Pass hardware randomness
    /// so every power-on looks different.
    #[must_use]
    pub fn new(entropy: u32) -> Self {
        let () = Self::VALID;
        let mut channel = 0;
        let channels = array::from_fn(|index| {
            let state = State::new(
                L::LIGHTING.channels[index].light(),
                boot_seed(channel, entropy),
            );
            channel += 1;
            state
        });
        Self { channels, tick: 0 }
    }

    /// Advances every channel by one tick and returns its intensity.
    pub fn update(&mut self) -> [Intensity; N] {
        let mut outputs = [Intensity::OFF; N];
        for (index, state) in self.channels.iter_mut().enumerate() {
            let intensity = state.update(&outputs[..index]);
            outputs[index] = Self::IGNITION.apply(index, self.tick, intensity);
        }
        for (output, channel) in outputs.iter_mut().zip(L::LIGHTING.channels) {
            *output = channel.trim(*output);
        }
        self.tick = self.tick.saturating_add(1);
        outputs
    }
}

#[cfg(test)]
mod tests {
    //! The bank of channels: roles, followers, ignition and trim working together.

    use super::{Lighting, LightingConfig, Lights};
    use crate::channel::{Channel, Light};
    use crate::config::FireConfig;
    use crate::fire_state::FireState;
    use crate::follower::{Follower, FollowerConfig};
    use crate::ignition::IgnitionConfig;
    use crate::installation::{Installation, InstallationConfig};
    use crate::intensity::Intensity;
    use crate::rng::{boot_seed, channel_seed};
    use crate::test_support::{Candle, Hearth, HearthGlow, Site, Torch};

    /// Ticks from the start of one channel to the start of the next.
    const DELAY_TICKS: u32 = 20;

    /// Ticks of a fade.
    const FADE_TICKS: u32 = 40;

    /// Channels of the test installations.
    const CHANNELS: usize = 5;

    /// Ticks after which every channel of the test installations is lit, with some margin.
    const LIT: u32 = 6 * DELAY_TICKS + FADE_TICKS;

    /// Ticks in which a follower forgets how it was lit: twenty time constants of the glow.
    const SETTLE: u32 = 20 * 1_500 / 15;

    /// Entropy of the test power-on.
    const ENTROPY: u32 = 0x2468_ace1;

    /// The role channels of [`Castle`].
    const ROLES: [usize; 3] = [0, 2, 3];

    /// The follower channels of [`Castle`].
    const FOLLOWERS: [usize; 2] = [1, 4];

    /// Lights of the test installations, with the presets and the given channel table.
    const fn lighting(channels: &'static [Channel]) -> LightingConfig {
        LightingConfig {
            hearth: FireConfig::HEARTH,
            torch: FireConfig::TORCH,
            candle: FireConfig::CANDLE,
            ember: FireConfig::EMBER,
            ambient: FireConfig::AMBIENT,
            glow: FollowerConfig::HEARTH_GLOW,
            ignition: IgnitionConfig {
                delay_ms: DELAY_TICKS * 15,
                fade_ms: FADE_TICKS * 15,
            },
            channels,
        }
    }

    /// A hearth with its glow, a torch, a candle and a follower of the candle, untrimmed.
    struct Castle;

    impl Installation for Castle {
        const CONFIG: InstallationConfig = InstallationConfig::new();
    }

    impl Lighting for Castle {
        const LIGHTING: LightingConfig = lighting(&[
            Channel::new(Light::Hearth, 1_000),
            Channel::new(Light::Follows(0), 1_000),
            Channel::new(Light::Torch, 1_000),
            Channel::new(Light::Candle, 1_000),
            Channel::new(Light::Follows(3), 1_000),
        ]);
    }

    /// [`Castle`] with every LED trimmed to its own share.
    struct TrimmedCastle;

    impl Installation for TrimmedCastle {
        const CONFIG: InstallationConfig = InstallationConfig::new();
    }

    impl Lighting for TrimmedCastle {
        const LIGHTING: LightingConfig = lighting(&TRIMMED);
    }

    /// The channel table of [`TrimmedCastle`].
    const TRIMMED: [Channel; CHANNELS] = [
        Channel::new(Light::Hearth, 500),
        Channel::new(Light::Follows(0), 900),
        Channel::new(Light::Torch, 1),
        Channel::new(Light::Candle, 999),
        Channel::new(Light::Follows(3), 250),
    ];

    /// Returns the outputs of every tick of `ticks`, one array per tick.
    fn record<L: Lighting>(ticks: u32) -> Vec<[Intensity; CHANNELS]> {
        let mut lights = Lights::<L, CHANNELS>::new(ENTROPY);
        (0..ticks).map(|_| lights.update()).collect()
    }

    /// After the ignition, every channel shows its role or follower exactly, seeded from the
    /// entropy.
    #[test]
    fn channels_run_their_roles_once_lit() {
        let seed = |channel| boot_seed(channel, ENTROPY);
        let mut hearth = FireState::<Site, Hearth>::new(seed(0));
        let mut glow = Follower::<Site, HearthGlow>::new();
        let mut torch = FireState::<Site, Torch>::new(seed(2));
        let mut candle = FireState::<Site, Candle>::new(seed(3));
        let mut candle_glow = Follower::<Site, HearthGlow>::new();
        let mut lights = Lights::<Castle, CHANNELS>::new(ENTROPY);
        for tick in 0..LIT + SETTLE + 5_000 {
            let outputs = lights.update();
            let flame = hearth.update();
            let candle_flame = candle.update();
            let expected = [
                flame,
                glow.update(flame),
                torch.update(),
                candle_flame,
                candle_glow.update(candle_flame),
            ];
            if tick > LIT {
                for channel in ROLES {
                    assert_eq!(
                        outputs[channel], expected[channel],
                        "channel {channel} diverged from its role at tick {tick}"
                    );
                }
            }
            // A follower was fed its source while it faded in, so it only converges on the
            // reference within the rounding of the smoothing.
            if tick > LIT + SETTLE {
                for channel in FOLLOWERS {
                    assert!(
                        outputs[channel].get().abs_diff(expected[channel].get()) <= 1,
                        "follower {channel} diverged at tick {tick}"
                    );
                }
            }
        }
    }

    /// During the ignition every channel is dark until its turn, and no follower glows while the
    /// channel it follows is still dark.
    #[test]
    fn ignition_lights_channels_in_turn() {
        let delay = usize::try_from(DELAY_TICKS).expect("a delay fits in usize");
        for (tick, outputs) in (0_u32..).zip(record::<Castle>(LIT)) {
            for (channel, (output, start)) in
                outputs.iter().zip((0_u32..).step_by(delay)).enumerate()
            {
                if tick <= start {
                    assert_eq!(
                        *output,
                        Intensity::OFF,
                        "channel {channel} glows at tick {tick} before its turn at {start}"
                    );
                }
            }
            for (follower, source) in FOLLOWERS.into_iter().zip([0, 3]) {
                assert!(
                    outputs[follower] == Intensity::OFF || outputs[source] > Intensity::OFF,
                    "follower {follower} glows at tick {tick} while channel {source} is dark"
                );
            }
        }
    }

    /// The trim scales the output of its own channel and of nothing else: the followers see the
    /// untrimmed flame.
    #[test]
    fn trim_scales_only_its_own_channel() {
        let plain = record::<Castle>(LIT + 5_000);
        let trimmed = record::<TrimmedCastle>(LIT + 5_000);
        for (tick, (plain, trimmed)) in plain.iter().zip(&trimmed).enumerate() {
            for (channel, ((&untrimmed, &output), entry)) in
                plain.iter().zip(trimmed).zip(&TRIMMED).enumerate()
            {
                assert_eq!(
                    output,
                    entry.trim(untrimmed),
                    "channel {channel} at tick {tick} is not its trimmed output"
                );
            }
        }
    }

    /// A look no channel uses is validated as well.
    #[test]
    #[should_panic(expected = "peak must be greater than 0")]
    fn rejects_invalid_unused_look() {
        LightingConfig {
            ambient: FireConfig {
                peak: Intensity::OFF,
                ..FireConfig::AMBIENT
            },
            ..Castle::LIGHTING
        }
        .validate(&InstallationConfig::new());
    }

    /// The follower is validated.
    #[test]
    #[should_panic(expected = "smoothing_ms is too long for the tick")]
    fn rejects_invalid_glow() {
        LightingConfig {
            glow: FollowerConfig {
                smoothing_ms: u32::MAX,
                ..FollowerConfig::HEARTH_GLOW
            },
            ..Castle::LIGHTING
        }
        .validate(&InstallationConfig::new());
    }

    /// The channel table is validated.
    #[test]
    #[should_panic(expected = "a channel cannot follow itself")]
    fn rejects_invalid_table() {
        const SELF_FOLLOWER: [Channel; 1] = [Channel::new(Light::Follows(0), 1_000)];
        LightingConfig {
            channels: &SELF_FOLLOWER,
            ..Castle::LIGHTING
        }
        .validate(&InstallationConfig::new());
    }

    /// The ignition is validated at the tick of the installation.
    #[test]
    #[should_panic(expected = "ignition.fade_ms must span at least one tick")]
    fn rejects_invalid_ignition() {
        LightingConfig {
            ignition: IgnitionConfig {
                delay_ms: 0,
                fade_ms: 14,
            },
            ..Castle::LIGHTING
        }
        .validate(&InstallationConfig::new());
    }

    /// Zero entropy seeds the roles with their compile-time channel seeds; other entropy changes
    /// the look.
    #[test]
    fn entropy_reseeds_the_roles() {
        let mut fixed = Lights::<Castle, CHANNELS>::new(0);
        let mut reference = FireState::<Site, Hearth>::new(channel_seed(0));
        let mut changed = Lights::<Castle, CHANNELS>::new(ENTROPY);
        let mut differs = false;
        for tick in 0..LIT + 1_000 {
            let flame = reference.update();
            let outputs = fixed.update();
            if tick > LIT {
                assert_eq!(
                    outputs[0], flame,
                    "zero entropy changed the hearth at {tick}"
                );
            }
            differs |= changed.update()[0] != outputs[0];
        }
        assert!(differs, "entropy did not change the hearth");
    }
}
