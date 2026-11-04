// SPDX-FileCopyrightText: 2025 the Blaustein LED contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Smooth, configurable campfire flicker for the LEDs of Burg Blaustein.
//!
//! Three LED channels are driven with high-frequency PWM from a single task: GPIO 16 and GPIO 17
//! on PWM slice 0 (channels A and B), GPIO 18 on PWM slice 1 (channel A). Each channel runs an
//! independent flicker simulation seeded differently so the flames do not move in lockstep.
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
use embassy_rp::peripherals::{PIN_16, PIN_17, PIN_18, PWM_SLICE0, PWM_SLICE1, WATCHDOG};
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

/// Program metadata for `picotool info`.
#[expect(
    unsafe_code,
    reason = "`memory.x` defines and keeps `.bi_entries`, so placing this static there is sound"
)]
#[unsafe(link_section = ".bi_entries")]
#[used]
static PICOTOOL_ENTRIES: [EntryAddr; 4] = [
    binary_info::rp_program_name!(c"Blaustein Fire LED"),
    binary_info::rp_program_description!(c"Smooth, configurable campfire flicker using PWM"),
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

/// Initializes the chip and spawns the fire task.
#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let peripherals = embassy_rp::init(Config::default());
    let token = fire_task(
        peripherals.WATCHDOG,
        peripherals.PWM_SLICE0,
        peripherals.PIN_16,
        peripherals.PIN_17,
        peripherals.PWM_SLICE1,
        peripherals.PIN_18,
    )
    .expect("fire_task is spawned once");
    spawner.spawn(token);
}

/// Drives all three LED channels with the campfire effect.
///
/// A single [`Ticker`] paces the loop, so the cadence stays fixed regardless of the runtime of
/// the loop body, and one task pool serves all channels.
#[embassy_executor::task]
async fn fire_task(
    watchdog: Peri<'static, WATCHDOG>,
    slice0: Peri<'static, PWM_SLICE0>,
    pin_16: Peri<'static, PIN_16>,
    pin_17: Peri<'static, PIN_17>,
    slice1: Peri<'static, PWM_SLICE1>,
    pin_18: Peri<'static, PIN_18>,
) -> ! {
    let top = pwm::calculate_top(clocks::clk_sys_freq(), CFG.pwm_freq_hz, CFG.pwm_divider);

    let mut config = PwmConfig::default();
    config.top = top;
    config.divider = CFG.pwm_divider.into();

    let mut pwm_slice0 = Pwm::new_output_ab(slice0, pin_16, pin_17, config.clone());
    let mut pwm_slice1 = Pwm::new_output_a(slice1, pin_18, config.clone());

    // Distinct seeds keep the three flames from flickering in sync.
    let mut flame_16 = const { Flame::new(0xc0ff_ee01) };
    let mut flame_17 = const { Flame::new(0x1234_5678) };
    let mut flame_18 = const { Flame::new(0xdead_beef) };

    let mut watchdog = Watchdog::new(watchdog);
    watchdog.pause_on_debug(true);
    watchdog.start(WATCHDOG_TIMEOUT);

    let mut ticker = Ticker::every(TICK);
    loop {
        // `set_config` writes both compare registers in one store, avoiding the
        // read-modify-write pairs and fallible API of the per-channel `PwmOutput` handles.
        config.compare_a = pwm::duty_for(flame_16.update(), top);
        config.compare_b = pwm::duty_for(flame_17.update(), top);
        pwm_slice0.set_config(&config);

        config.compare_a = pwm::duty_for(flame_18.update(), top);
        // Channel B of slice 1 is not routed to a pin.
        config.compare_b = 0;
        pwm_slice1.set_config(&config);

        watchdog.feed(WATCHDOG_TIMEOUT);
        ticker.next().await;
    }
}
