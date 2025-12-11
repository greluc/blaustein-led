#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_rp::Peri;
use embassy_rp::peripherals::{PIN_16, PWM_SLICE0};
use embassy_rp::pwm::{Config, Pwm, SetDutyCycle};
use embassy_time::Timer;
// Always include a panic handler; in debug builds we also enable defmt RTT logging.
#[cfg(feature = "debug")]
use defmt_rtt as _;
use panic_probe as _;

// Program metadata for `picotool info`.
// This isn't needed, but it's recommended to have these minimal entries.
#[unsafe(link_section = ".bi_entries")]
#[used]
pub static PICOTOOL_ENTRIES: [embassy_rp::binary_info::EntryAddr; 4] = [
    embassy_rp::binary_info::rp_program_name!(c"Blaustein Fire LED"),
    embassy_rp::binary_info::rp_program_description!(
        c"Smooth, configurable campfire flicker on GPIO 25 using PWM"
    ),
    embassy_rp::binary_info::rp_cargo_version!(),
    embassy_rp::binary_info::rp_program_build_attribute!(),
];

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    spawner
        .spawn(fire_led_task(p.PWM_SLICE0, p.PIN_16, FireConfig {
            max_intensity: 64,
            jitter_max: 25,
            pulse_prob: 64,
            ..FireConfig::default()
        }))
        .unwrap();
}

/// Fire effect configuration parameters.
///
/// These parameters shape the "look & feel" of the LED flame.
/// All values are integer and no-std friendly. Adjust to taste.
#[derive(Clone, Copy)]
pub struct FireConfig {
    /// PWM target frequency (Hz). 20-30 kHz keeps flicker invisible.
    pub pwm_freq_hz: u32,
    /// Integer clock divider. Use the smallest value that keeps `top` within u16.
    pub pwm_divider: u8,
    /// Minimum and maximum base intensity (0..=255).
    pub min_intensity: u8,
    pub max_intensity: u8,
    /// Full breath period in milliseconds (one up+down cycle).
    pub breath_period_ms: u32,
    /// Random per-tick jitter amplitude added around the base (0..=64 typical).
    pub jitter_max: u8,
    /// Probability (0..=255) to start a short flare on a tick. ~3 = ~1% at 15ms.
    pub pulse_prob: u8,
    /// Pulse strength added to intensity when a flare triggers.
    pub pulse_boost: u8,
    /// Q8 decay factor per tick for pulses (0..=255). 224≈fast, 248≈slower.
    pub pulse_decay_q8: u8,
    /// Q8 smoothing factor per tick for the output EMA. Smaller = smoother.
    /// new = old + ((target-old)*smooth_q8 >> 8)
    pub smooth_q8: u8,
    /// Update interval in milliseconds.
    pub tick_ms: u32,
}

impl Default for FireConfig {
    fn default() -> Self {
        Self {
            pwm_freq_hz: 25_000,
            pwm_divider: 1,
            min_intensity: 20,
            max_intensity: 255,
            breath_period_ms: 3_500,
            jitter_max: 18,
            pulse_prob: 3,
            pulse_boost: 40,
            pulse_decay_q8: 232,  // ~150ms half-life at 15ms tick
            smooth_q8: 20,        // output reacts in a few hundred ms
            tick_ms: 15,
        }
    }
}

/// Task that generates a smooth, warm campfire effect on GPIO 16 using PWM.
///
/// Uses only integer math, no_alloc, and runs entirely async on Embassy.
#[embassy_executor::task]
async fn fire_led_task(
    slice0: Peri<'static, PWM_SLICE0>,
    pin16: Peri<'static, PIN_16>,
    cfg: FireConfig,
) {
    let mut c = Config::default();
    c.top = calculate_top(cfg.pwm_freq_hz, cfg.pwm_divider);
    c.divider = cfg.pwm_divider.into();
    let top_counts = c.top;

    // Channel A is GPIO 16 on RP235x.
    let mut pwm = Pwm::new_output_a(slice0, pin16, c);

    // Precompute gamma≈2 duty lookup to avoid per-tick division.
    let mut duty_lut = [0u16; 256];
    for i in 0..=255 {
        let y = (i as u32) * (i as u32); // 0..65025
        duty_lut[i as usize] = (((y * (top_counts as u32)) + 32) / 65025) as u16;
    }

    // PRNG state (Xorshift32) and flicker state.
    let mut rng: u32 = 0xC0FFEE01;
    // Triangular "breathing" envelope using fixed-point incremental ramp (Q8).
    let span_i32 = (cfg.max_intensity as i32 - cfg.min_intensity as i32) as i32;
    // step per millisecond in Q8: 2*span / period
    let step_q8: i32 = (((span_i32 as i64) << 9) / (cfg.breath_period_ms as i64)) as i32;
    let mut base_q8: i32 = (cfg.min_intensity as i32) << 8; // start at min
    let max_q8: i32 = (cfg.max_intensity as i32) << 8;
    let min_q8: i32 = (cfg.min_intensity as i32) << 8;
    let mut rising = true;

    let mut out_q8: i32 = (cfg.min_intensity as i32) << 8; // smoothed output, Q8
    let mut pulse: i32 = 0; // current pulse boost, integer

    loop {
        // --- base triangular "breathing" envelope (incremental, no div in loop) ---
        if rising {
            base_q8 += step_q8 * (cfg.tick_ms as i32);
            if base_q8 >= max_q8 {
                base_q8 = max_q8;
                rising = false;
            }
        } else {
            base_q8 -= step_q8 * (cfg.tick_ms as i32);
            if base_q8 <= min_q8 {
                base_q8 = min_q8;
                rising = true;
            }
        }
        let base = base_q8 >> 8;

        // --- random jitter around base ---
        rng = xorshift32(rng);
        let jitter_range = cfg.jitter_max as i32;
        // Scale signed 8-bit noise to approximately [-jitter_max, +jitter_max] without modulo.
        let n = (rng as u8 as i16) - 128; // -128..127
        let jitter = ((n as i32 * (2 * jitter_range + 1)) >> 8);

        // --- sporadic short pulses (flames licking higher) ---
        rng = xorshift32(rng);
        if (rng as u8) < cfg.pulse_prob {
            pulse += cfg.pulse_boost as i32;
        }
        // Exponential-like decay in Q8 domain
        pulse = (pulse * (cfg.pulse_decay_q8 as i32)) >> 8;

        // Combine and clamp target intensity to 0..255
        let mut target = base + jitter + pulse;
        if target < 0 {
            target = 0;
        } else if target > 255 {
            target = 255;
        }

        // --- smooth the output ---
        let target_q8 = target << 8;
        out_q8 += ((target_q8 - out_q8) * (cfg.smooth_q8 as i32)) >> 8;

        // Map 0..255 (from Q8) through gamma≈2 curve to PWM duty counts
        let intensity = (out_q8 >> 8) as u8;
        let duty = duty_lut[intensity as usize];
        // Apply PWM duty (errors are infallible for RP PWM)
        let _ = pwm.set_duty_cycle(duty);

        Timer::after_millis(cfg.tick_ms as u64).await;
    }
}

/// Simple Xorshift32 PRNG: fast, small, good enough for flicker.
#[inline]
fn xorshift32(mut x: u32) -> u32 {
    x ^= x << 13;
    x ^= x >> 17;
    x ^= x << 5;
    x
}

/// Map 0..=255 intensity to PWM counts in 0..=top using an approximate
/// gamma 2.0 curve for more natural perceived brightness.
#[inline]
fn gamma2_map_to_top(intensity: u8, top: u16) -> u16 {
    let i = intensity as u32;
    let y = i * i; // 0..65025
    // duty = round((y / 255^2) * top)
    (((y * (top as u32)) + 32) / 65025) as u16
}

/// Calculates the PWM timer top value based on the desired frequency and clock divider.
///
/// See notes: choose a small divider (often 1) to maximize duty resolution at a given
/// PWM frequency, while keeping `top` within u16.
fn calculate_top(desired_freq_hz: u32, divider: u8) -> u16 {
    let clock_freq_hz = embassy_rp::clocks::clk_sys_freq();
    let period_counts = clock_freq_hz / (desired_freq_hz * divider as u32);
    let period_counts = core::cmp::min(period_counts, u16::MAX as u32) as u16;
    core::cmp::max(1, period_counts) - 1
}
