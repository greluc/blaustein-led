// SPDX-FileCopyrightText: 2025 the Blaustein LED contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Firmware for the campfire lights of Burg Blaustein.
//!
//! Up to sixteen LED channels on GPIO 7 to GPIO 22 are driven with high-frequency PWM from a
//! single task. Each channel runs the role or follower its entry in the channel table assigns,
//! with its own seed and no shared phase, so the flames do not move in lockstep. After power-on
//! the channels light one after another; the `identify` feature replaces the fire with every
//! channel blinking its own number.
//!
//! The effect uses integer math only (no floating point, no allocation). The configuration in
//! [`config`] is a compile-time constant, so every derived quantity is folded into the binary by
//! the optimizer.

#![no_std]
#![no_main]

mod config;
mod effect;
mod outputs;

use core::ffi::CStr;
#[cfg(not(feature = "debug"))]
use core::panic::PanicInfo;

#[cfg(not(feature = "debug"))]
use cortex_m::peripheral::SCB;
#[cfg(feature = "debug")]
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_rp::binary_info::{self, EntryAddr};
use embassy_rp::config::Config;
use embassy_rp::watchdog::Watchdog;
use embassy_time::{Duration, Ticker};
#[cfg(feature = "debug")]
use panic_probe as _;

use crate::config::INSTALLATION;
use crate::outputs::Outputs;

/// Program description for `picotool info`.
#[cfg(not(feature = "identify"))]
const DESCRIPTION: &CStr = c"Smooth, configurable campfire flicker on up to 16 LEDs using PWM";

/// Program description for `picotool info`.
#[cfg(feature = "identify")]
const DESCRIPTION: &CStr = c"Identify build: every LED blinks its channel number";

/// Program metadata for `picotool info`.
#[expect(
    unsafe_code,
    reason = "`memory.x` defines and keeps `.bi_entries`, so placing this static there is sound"
)]
#[unsafe(link_section = ".bi_entries")]
#[used]
static PICOTOOL_ENTRIES: [EntryAddr; 4] = [
    binary_info::rp_program_name!(c"Blaustein Fire LED"),
    binary_info::rp_program_description!(DESCRIPTION),
    binary_info::rp_cargo_version!(),
    binary_info::rp_program_build_attribute!(),
];

/// Interval between two effect updates.
const TICK: Duration = Duration::from_millis(INSTALLATION.tick_ms as u64);

/// Watchdog period. It runs from the start of `main`, and the task feeds it once per tick; if
/// anything stops the effect, the chip resets and the lights come back on their own.
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

/// Initializes the chip, starts the watchdog, routes the pins of the LED channels to their PWM
/// slices and spawns the light task.
#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let peripherals = embassy_rp::init(Config::default());
    let mut watchdog = Watchdog::new(peripherals.WATCHDOG);
    watchdog.pause_on_debug(true);
    watchdog.start(WATCHDOG_TIMEOUT);

    let config = outputs::config();
    let routed = [
        outputs::slice(
            0,
            peripherals.PWM_SLICE0,
            peripherals.PIN_16,
            peripherals.PIN_17,
            &config,
        ),
        outputs::slice(
            1,
            peripherals.PWM_SLICE1,
            peripherals.PIN_18,
            peripherals.PIN_19,
            &config,
        ),
        outputs::slice(
            2,
            peripherals.PWM_SLICE2,
            peripherals.PIN_20,
            peripherals.PIN_21,
            &config,
        ),
        outputs::slice(
            3,
            peripherals.PWM_SLICE3,
            peripherals.PIN_22,
            peripherals.PIN_7,
            &config,
        ),
        outputs::slice(
            4,
            peripherals.PWM_SLICE4,
            peripherals.PIN_8,
            peripherals.PIN_9,
            &config,
        ),
        outputs::slice(
            5,
            peripherals.PWM_SLICE5,
            peripherals.PIN_10,
            peripherals.PIN_11,
            &config,
        ),
        outputs::slice(
            6,
            peripherals.PWM_SLICE6,
            peripherals.PIN_12,
            peripherals.PIN_13,
            &config,
        ),
        outputs::slice(
            7,
            peripherals.PWM_SLICE7,
            peripherals.PIN_14,
            peripherals.PIN_15,
            &config,
        ),
    ];
    let outputs = Outputs::start(routed, config);
    let token = light_task(watchdog, outputs).expect("light_task is spawned once");
    spawner.spawn(token);
}

/// Shows the effect of this build on every LED channel and feeds the watchdog.
///
/// A single [`Ticker`] paces the loop, so the cadence stays fixed regardless of the runtime of
/// the loop body, and one task serves all channels.
#[embassy_executor::task]
async fn light_task(mut watchdog: Watchdog, mut outputs: Outputs) -> ! {
    let mut effect = effect::start();
    let mut ticker = Ticker::every(TICK);
    loop {
        outputs.show(&effect.update());
        watchdog.feed(WATCHDOG_TIMEOUT);
        ticker.next().await;
    }
}
