#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::Peri;
use embassy_rp::peripherals::{PIN_25, PWM_SLICE4};
use embassy_rp::pwm::{Config, Pwm, SetDutyCycle};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

// Program metadata for `picotool info`.
// This isn't needed, but it's recommended to have these minimal entries.
#[unsafe(link_section = ".bi_entries")]
#[used]
pub static PICOTOOL_ENTRIES: [embassy_rp::binary_info::EntryAddr; 4] = [
    embassy_rp::binary_info::rp_program_name!(c"Blinky Example"),
    embassy_rp::binary_info::rp_program_description!(
        c"This example tests the RP Pico on board LED, connected to gpio 25"
    ),
    embassy_rp::binary_info::rp_cargo_version!(),
    embassy_rp::binary_info::rp_program_build_attribute!(),
];

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    spawner
        .spawn(pwm_set_dutycycle(p.PWM_SLICE4, p.PIN_25))
        .unwrap();
}

/// Demonstrates PWM by setting a duty cycle.
///
/// Using GP25 in Slice4, make sure to use an appropriate resistor.
#[embassy_executor::task]
async fn pwm_set_dutycycle(slice4: Peri<'static, PWM_SLICE4>, pin25: Peri<'static, PIN_25>) {
    let desired_freq_hz = 25_000;
    let divider = 16u8;

    let mut c = Config::default();
    c.top = calculate_top(desired_freq_hz, divider);
    c.divider = divider.into();

    let mut pwm = Pwm::new_output_b(slice4, pin25, c.clone());

    loop {
        // 100% duty cycle, fully on
        pwm.set_duty_cycle_fully_on().unwrap();
        Timer::after_millis(100).await;

        // 66% duty cycle. Expressed as a percentage.
        pwm.set_duty_cycle_percent(66).unwrap();
        Timer::after_millis(100).await;

        // 25% duty cycle. Expressed as 32768/4 = 8192.
        pwm.set_duty_cycle(c.top / 4).unwrap();
        Timer::after_millis(100).await;

        // 0% duty cycle, fully off.
        pwm.set_duty_cycle_fully_off().unwrap();
        Timer::after_millis(100).await;
    }
}

/// Calculates the PWM timer top value based on the desired frequency and clock divider.
///
/// This function determines the top value of the PWM timer based on the
/// desired frequency (in Hz) and the divider value. The top value defines
/// the period of the PWM cycle, with the counter incrementing from 0 to
/// the top value before wrapping around to 0. The duration of one PWM cycle
/// is determined by this behavior.
///
/// # Arguments
///
/// * `desired_freq_hz` - The target frequency for the PWM (in Hz).
/// * `divider` - The clock divider applied to the timer.
///
/// # Returns
///
/// * `u16` - The calculated top value for the PWM timer.
///
/// # Calculation
///
/// The calculation uses the system clock frequency and computes the desired
/// timer period as follows:
///
/// ```text
/// period = CLK_SYS_FREQ / (desired_freq_hz * divider) - 1
/// ```
/// Where `CLK_SYS_FREQ` is the system clock frequency retrieved using
/// `embassy_rp::clocks::clk_sys_freq()`.
///
/// # Example
///
/// ```rust
/// let desired_frequency = 1000; // Target frequency in Hz
/// let divider = 4; // Clock divider
/// let top_value = calculate_top(desired_frequency, divider);
/// println!("The calculated top value is: {}", top_value);
/// ```
///
/// # Notes
///
/// - Ensure the desired frequency and divider values are valid and lead
///   to a feasible top value within the range of a `u16`.
/// - The function assumes that `clk_sys_freq` returns the correct system
///   clock frequency in Hz.
fn calculate_top(desired_freq_hz: u32, divider: u8) -> u16 {
    let clock_freq_hz = embassy_rp::clocks::clk_sys_freq();
    (clock_freq_hz / (desired_freq_hz * divider as u32)) as u16 - 1
}
