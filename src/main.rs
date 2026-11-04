// SPDX-FileCopyrightText: 2025 the Blaustein LED contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Smooth, configurable campfire flicker for the LEDs of Burg Blaustein.
//!
//! Sixteen LED channels on GPIO 7 to GPIO 22 are driven with high-frequency PWM from a single
//! task. GPIO `n` is PWM channel `n mod 16` (slice `n mod 16 / 2`, A for even `n`, B for odd), so
//! this contiguous block reaches every PWM output the RP2350A can route to a pin exactly once.
//! Each channel runs an independent flicker simulation with its own seed, so the flames do not
//! move in lockstep.
//!
//! The effect uses integer math only (no floating point, no allocation). [`config::CFG`] is a
//! compile-time constant, so every derived quantity is folded into the binary by the optimizer.

#![no_std]
#![no_main]

mod config;

#[cfg(not(feature = "debug"))]
use core::panic::PanicInfo;

#[cfg(not(feature = "debug"))]
use cortex_m::peripheral::SCB;
#[cfg(feature = "debug")]
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_rp::binary_info::{self, EntryAddr};
use embassy_rp::config::Config;
use embassy_rp::peripherals::WATCHDOG;
use embassy_rp::pwm::{Config as PwmConfig, Pwm};
use embassy_rp::watchdog::Watchdog;
use embassy_rp::{Peri, clocks};
use embassy_time::{Duration, Ticker};
use fire_effect::{FireState, pwm};
#[cfg(feature = "debug")]
use panic_probe as _;

use crate::config::{Blaustein, CFG};

/// Fire simulation of one channel with the installed flame look.
type Flame = FireState<Blaustein>;

/// PWM slices that drive LEDs; slices 8 to 11 exist only on the RP2350B.
const SLICES: usize = 8;

/// Channels of one PWM slice (A and B).
const CHANNELS_PER_SLICE: usize = 2;

/// LED channels, one per GPIO from GPIO 7 to GPIO 22.
const CHANNELS: usize = SLICES * CHANNELS_PER_SLICE;

/// Program metadata for `picotool info`.
#[expect(
    unsafe_code,
    reason = "`memory.x` defines and keeps `.bi_entries`, so placing this static there is sound"
)]
#[unsafe(link_section = ".bi_entries")]
#[used]
static PICOTOOL_ENTRIES: [EntryAddr; 4] = [
    binary_info::rp_program_name!(c"Blaustein Fire LED"),
    binary_info::rp_program_description!(
        c"Smooth, configurable campfire flicker on 16 LEDs using PWM"
    ),
    binary_info::rp_cargo_version!(),
    binary_info::rp_program_build_attribute!(),
];

/// Interval between two effect updates.
const TICK: Duration = Duration::from_millis(CFG.tick_ms as u64);

/// Watchdog period. The task feeds it once per tick; if the effect stops updating, the chip
/// resets and the flames come back on their own.
const WATCHDOG_TIMEOUT: Duration = Duration::from_millis(1_000);

const _: () = assert!(
    WATCHDOG_TIMEOUT.as_ticks() > TICK.as_ticks(),
    "the watchdog timeout must be longer than one tick, or every tick resets the chip"
);

/// Resets the chip on panic in deployed builds.
///
/// The installation is unattended, so halting would leave the LEDs dark until someone visits
/// the site. Debug builds use `panic-probe` instead, which halts for an attached debugger.
#[cfg(not(feature = "debug"))]
#[panic_handler]
fn panic(_info: &PanicInfo<'_>) -> ! {
    SCB::sys_reset()
}

/// Initializes the chip, routes GPIO 7 to GPIO 22 to their PWM slices and spawns the fire task.
#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let peripherals = embassy_rp::init(Config::default());
    let config = pwm_config();
    let slices: [Pwm<'static>; SLICES] = [
        Pwm::new_output_ab(
            peripherals.PWM_SLICE0,
            peripherals.PIN_16,
            peripherals.PIN_17,
            config.clone(),
        ),
        Pwm::new_output_ab(
            peripherals.PWM_SLICE1,
            peripherals.PIN_18,
            peripherals.PIN_19,
            config.clone(),
        ),
        Pwm::new_output_ab(
            peripherals.PWM_SLICE2,
            peripherals.PIN_20,
            peripherals.PIN_21,
            config.clone(),
        ),
        Pwm::new_output_ab(
            peripherals.PWM_SLICE3,
            peripherals.PIN_22,
            peripherals.PIN_7,
            config.clone(),
        ),
        Pwm::new_output_ab(
            peripherals.PWM_SLICE4,
            peripherals.PIN_8,
            peripherals.PIN_9,
            config.clone(),
        ),
        Pwm::new_output_ab(
            peripherals.PWM_SLICE5,
            peripherals.PIN_10,
            peripherals.PIN_11,
            config.clone(),
        ),
        Pwm::new_output_ab(
            peripherals.PWM_SLICE6,
            peripherals.PIN_12,
            peripherals.PIN_13,
            config.clone(),
        ),
        Pwm::new_output_ab(
            peripherals.PWM_SLICE7,
            peripherals.PIN_14,
            peripherals.PIN_15,
            config.clone(),
        ),
    ];
    let token = fire_task(peripherals.WATCHDOG, slices, config).expect("fire_task is spawned once");
    spawner.spawn(token);
}

/// Returns the PWM configuration shared by all slices, with both compare values at zero.
fn pwm_config() -> PwmConfig {
    let mut config = PwmConfig::default();
    config.top = pwm::calculate_top(clocks::clk_sys_freq(), CFG.pwm_freq_hz, CFG.pwm_divider);
    config.divider = CFG.pwm_divider.into();
    config
}

/// Drives all LED channels with the campfire effect.
///
/// A single [`Ticker`] paces the loop, so the cadence stays fixed regardless of the runtime of
/// the loop body, and one task pool serves all channels.
#[embassy_executor::task]
async fn fire_task(
    watchdog: Peri<'static, WATCHDOG>,
    mut slices: [Pwm<'static>; SLICES],
    mut config: PwmConfig,
) -> ! {
    let top = config.top;
    let mut flames: [Flame; CHANNELS] = const { Flame::channels() };
    let (flame_pairs, _) = flames.as_chunks_mut::<CHANNELS_PER_SLICE>();

    let mut watchdog = Watchdog::new(watchdog);
    watchdog.pause_on_debug(true);
    watchdog.start(WATCHDOG_TIMEOUT);

    let mut ticker = Ticker::every(TICK);
    loop {
        let outputs = slices.iter_mut().zip(flame_pairs.iter_mut());
        for (slice, [flame_a, flame_b]) in outputs {
            // `set_config` writes both compare registers in one store, avoiding the
            // read-modify-write pairs and fallible API of the per-channel `PwmOutput` handles.
            config.compare_a = pwm::duty_for(flame_a.update(), top);
            config.compare_b = pwm::duty_for(flame_b.update(), top);
            slice.set_config(&config);
        }

        watchdog.feed(WATCHDOG_TIMEOUT);
        ticker.next().await;
    }
}
