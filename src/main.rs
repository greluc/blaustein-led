#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_rp::Peri;
use embassy_rp::peripherals::{PIN_16, PIN_17, PIN_18, PWM_SLICE0, PWM_SLICE1};
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

    // Common configuration for the fire effect
    let cfg = FireConfig {
        max_intensity: 128,
        jitter_max: 25,
        pulse_prob: 48,
        breath_period_ms: 5000,
        ..FireConfig::default()
    };

    // Slice 0 controls PIN_16 (Channel A) and PIN_17 (Channel B)
    // We spawn a task that updates both channels independently.
    spawner
        .spawn(fire_task_slice0(
            p.PWM_SLICE0,
            p.PIN_16,
            p.PIN_17,
            cfg,
        ))
        .unwrap();

    // Slice 1 controls PIN_18 (Channel A)
    spawner
        .spawn(fire_task_slice1(
            p.PWM_SLICE1,
            p.PIN_18,
            cfg,
        ))
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
            pulse_decay_q8: 232, // ~150ms half-life at 15ms tick
            smooth_q8: 20,       // output reacts in a few hundred ms
            tick_ms: 15,
        }
    }
}

/// Encapsulates the state of a single fire effect simulation.
/// Allows multiple independent fire effects (LEDs) to run side-by-side.
struct FireState {
    rng: u32,
    base_q8: i32,
    rising: bool,
    out_q8: i32,
    pulse: i32,
    cfg: FireConfig,
    // Precomputed derived values
    step_q8: i32,
    max_q8: i32,
    min_q8: i32,
    jitter_range: i32,
}

impl FireState {
    fn new(cfg: FireConfig, seed: u32) -> Self {
        let span_i32 = cfg.max_intensity as i32 - cfg.min_intensity as i32;
        // step per millisecond in Q8: 2*span / period
        let step_q8 = (((span_i32 as i64) << 9) / (cfg.breath_period_ms as i64)) as i32;
        let min_q8 = (cfg.min_intensity as i32) << 8;
        let max_q8 = (cfg.max_intensity as i32) << 8;
        
        Self {
            rng: seed,
            base_q8: min_q8,
            rising: true,
            out_q8: min_q8,
            pulse: 0,
            cfg,
            step_q8,
            max_q8,
            min_q8,
            jitter_range: cfg.jitter_max as i32,
        }
    }

    /// Advances the simulation by one tick and returns the new intensity (0..=255).
    fn update(&mut self) -> u8 {
        // --- base triangular "breathing" envelope ---
        if self.rising {
            self.base_q8 += self.step_q8 * (self.cfg.tick_ms as i32);
            if self.base_q8 >= self.max_q8 {
                self.base_q8 = self.max_q8;
                self.rising = false;
            }
        } else {
            self.base_q8 -= self.step_q8 * (self.cfg.tick_ms as i32);
            if self.base_q8 <= self.min_q8 {
                self.base_q8 = self.min_q8;
                self.rising = true;
            }
        }
        let base = self.base_q8 >> 8;

        // --- random jitter around base ---
        self.rng = xorshift32(self.rng);
        // Scale signed 8-bit noise to approximately [-jitter_max, +jitter_max]
        let n = (self.rng as u8 as i16) - 128; // -128..127
        let jitter = (n as i32 * (2 * self.jitter_range + 1)) >> 8;

        // --- sporadic short pulses ---
        self.rng = xorshift32(self.rng);
        if (self.rng as u8) < self.cfg.pulse_prob {
            self.pulse += self.cfg.pulse_boost as i32;
        }
        // Exponential-like decay
        self.pulse = (self.pulse * (self.cfg.pulse_decay_q8 as i32)) >> 8;

        // Combine and clamp target intensity to 0..255
        let mut target = base + jitter + self.pulse;
        if target < 0 {
            target = 0;
        } else if target > 255 {
            target = 255;
        }

        // --- smooth the output ---
        let target_q8 = target << 8;
        self.out_q8 += ((target_q8 - self.out_q8) * (self.cfg.smooth_q8 as i32)) >> 8;

        (self.out_q8 >> 8) as u8
    }
}

/// Helper to create a gamma-corrected duty cycle lookup table.
fn create_lut(top_counts: u16) -> [u16; 256] {
    let mut duty_lut = [0u16; 256];
    for i in 0..=255 {
        let y = (i as u32) * (i as u32); // 0..65025
        duty_lut[i as usize] = (((y * (top_counts as u32)) + 32) / 65025) as u16;
    }
    duty_lut
}

/// Task that generates a smooth, warm campfire effect on Slice 0 (GPIO 16 & 17).
#[embassy_executor::task]
async fn fire_task_slice0(
    slice: Peri<'static, PWM_SLICE0>,
    pin_a: Peri<'static, PIN_16>,
    pin_b: Peri<'static, PIN_17>,
    cfg: FireConfig,
) {
    let mut c = Config::default();
    c.top = calculate_top(cfg.pwm_freq_hz, cfg.pwm_divider);
    c.divider = cfg.pwm_divider.into();
    let top = c.top;

    let mut pwm = Pwm::new_output_ab(slice, pin_a, pin_b, c.clone());
    let lut = create_lut(top);
    
    // Initialize two fire simulators with different seeds so they don't flicker in sync.
    let mut sim_a = FireState::new(cfg, 0xC0FFEE01);
    let mut sim_b = FireState::new(cfg, 0x12345678);

    loop {
        let int_a = sim_a.update();
        let int_b = sim_b.update();
        
        // Update both channels
        c.compare_a = lut[int_a as usize];
        c.compare_b = lut[int_b as usize];
        pwm.set_config(&c);
        
        Timer::after_millis(cfg.tick_ms as u64).await;
    }
}

/// Task that generates a smooth, warm campfire effect on Slice 1 (GPIO 18).
#[embassy_executor::task]
async fn fire_task_slice1(
    slice: Peri<'static, PWM_SLICE1>,
    pin_a: Peri<'static, PIN_18>,
    cfg: FireConfig,
) {
    let mut c = Config::default();
    c.top = calculate_top(cfg.pwm_freq_hz, cfg.pwm_divider);
    c.divider = cfg.pwm_divider.into();
    let top = c.top;

    let mut pwm = Pwm::new_output_a(slice, pin_a, c.clone());
    let lut = create_lut(top);
    
    // Use a third unique seed
    let mut sim = FireState::new(cfg, 0xDEADBEEF);

    loop {
        let intensity = sim.update();
        
        // Single channel update
        pwm.set_duty_cycle(lut[intensity as usize]).expect("TODO: panic message");
        
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
