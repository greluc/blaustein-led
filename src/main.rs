//! Smooth, configurable campfire flicker for the LEDs of Burg Blaustein.
//!
//! Three LED channels are driven with high-frequency PWM from a single task:
//! GPIO 16 and GPIO 17 on PWM slice 0 (channels A and B), GPIO 18 on PWM slice 1
//! (channel A). Each channel runs an independent flicker simulation seeded
//! differently so the flames do not move in lockstep.
//!
//! The whole effect uses integer math only (no floating point, no allocation).
//! [`FireConfig`] is a compile-time constant, so every derived quantity is folded
//! into the binary by the optimizer and costs neither RAM nor runtime cycles.

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_rp::Peri;
use embassy_rp::peripherals::{PIN_16, PIN_17, PIN_18, PWM_SLICE0, PWM_SLICE1, WATCHDOG};
use embassy_rp::pwm::{Config, Pwm};
use embassy_rp::watchdog::Watchdog;
use embassy_time::{Duration, Ticker};

#[cfg(feature = "debug")]
use {defmt_rtt as _, panic_probe as _};

/// Panic handler for deployed builds.
///
/// The installation is unattended, so halting on panic would leave the LEDs dark
/// until someone physically visits the site. Resetting gets the effect running
/// again within milliseconds. Debug builds instead use `panic-probe`, which halts
/// so an attached debugger can inspect the fault.
#[cfg(not(feature = "debug"))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    cortex_m::peripheral::SCB::sys_reset()
}

/// Program metadata for `picotool info`.
// `link_section` is an unsafe attribute in edition 2024: placing a static in a
// custom section is only sound because `memory.x` defines and KEEPs `.bi_entries`.
#[allow(unsafe_code)]
#[unsafe(link_section = ".bi_entries")]
#[used]
pub static PICOTOOL_ENTRIES: [embassy_rp::binary_info::EntryAddr; 4] = [
    embassy_rp::binary_info::rp_program_name!(c"Blaustein Fire LED"),
    embassy_rp::binary_info::rp_program_description!(
        c"Smooth, configurable campfire flicker using PWM"
    ),
    embassy_rp::binary_info::rp_cargo_version!(),
    embassy_rp::binary_info::rp_program_build_attribute!(),
];

/// Active fire effect configuration.
///
/// This is a `const`, not a runtime value: the optimizer folds every derived
/// quantity (envelope step, thresholds, decay factors) into immediates.
const CFG: FireConfig = FireConfig {
    max_intensity: 128,
    jitter_max: 25,
    pulse_prob: 48,
    breath_period_ms: 5_000,
    ..FireConfig::new()
};

/// Watchdog period. The task feeds it once per tick; if the effect ever stops
/// updating, the chip resets and the flames come back on their own.
const WATCHDOG_TIMEOUT: Duration = Duration::from_millis(1_000);

/// Compile-time validation of [`CFG`].
///
/// These conditions were previously unchecked and could silently produce a dead
/// or panicking build; now an invalid configuration fails to compile.
const _: () = {
    assert!(
        CFG.max_intensity > CFG.min_intensity,
        "max_intensity must be greater than min_intensity"
    );
    assert!(
        CFG.pwm_divider >= 1,
        "pwm_divider must be at least 1 (0 would divide by zero here)"
    );
    assert!(CFG.pwm_freq_hz > 0, "pwm_freq_hz must be greater than 0");
    assert!(CFG.tick_ms > 0, "tick_ms must be greater than 0");
    assert!(
        STEP_PER_TICK_Q16 > 0,
        "breath_period_ms is too long for this intensity span: \
         the envelope step truncates to zero and the breathing would freeze"
    );
    assert!(
        WATCHDOG_TIMEOUT.as_millis() > CFG.tick_ms as u64,
        "watchdog timeout must be longer than one tick"
    );
};

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    // A single task drives all three channels from one timer: half the wakeups
    // of two independent tasks, and only one task pool in .bss.
    //
    // Since embassy-executor 0.10 the fallible step is building the token (the
    // pool holds one instance), while `spawn` itself is infallible. This is the
    // only spawn, so the pool cannot be exhausted.
    let token = fire_task(
        p.WATCHDOG,
        p.PWM_SLICE0,
        p.PIN_16,
        p.PIN_17,
        p.PWM_SLICE1,
        p.PIN_18,
    )
    .expect("fire_task pool exhausted");
    spawner.spawn(token);
}

/// Fire effect configuration parameters.
///
/// These parameters shape the "look & feel" of the LED flame.
/// All values are integer and no-std friendly. Adjust to taste.
#[derive(Clone, Copy)]
pub struct FireConfig {
    /// PWM target frequency (Hz). 20-30 kHz keeps flicker invisible.
    pub pwm_freq_hz: u32,
    /// Integer clock divider. Must be at least 1. Use the smallest value that
    /// keeps `top` within u16.
    pub pwm_divider: u8,
    /// Minimum base intensity (0..=255).
    pub min_intensity: u8,
    /// Maximum base intensity (0..=255). Must be greater than `min_intensity`.
    pub max_intensity: u8,
    /// Full breath period in milliseconds (one up+down cycle).
    pub breath_period_ms: u32,
    /// Random per-tick jitter amplitude added around the base (0..=64 typical).
    pub jitter_max: u8,
    /// Probability (0..=255) to start a short flare on a tick, evaluated once
    /// per tick. At the default 15 ms tick, 3 is roughly 1% (~0.8 flares/s) and
    /// 48 is roughly 19% (~12 flares/s, a visibly busy fire).
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

impl FireConfig {
    /// Baseline configuration; use struct update syntax to override fields.
    ///
    /// This is a `const fn` rather than a `Default` impl because [`CFG`] is built
    /// in a const context, and `Default::default()` cannot be called there.
    #[must_use]
    pub const fn new() -> Self {
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

impl Default for FireConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Envelope step per tick in Q16, evaluated at compile time.
///
/// Computed as one expression in 64-bit math so the intermediate never
/// truncates: `2 * span * 2^16 * tick_ms / breath_period_ms`. The Q16 scale (up
/// from Q8) keeps the rounding error of the breath period well under 0.5%.
const STEP_PER_TICK_Q16: i32 = {
    let span = (CFG.max_intensity as i64) - (CFG.min_intensity as i64);
    (((span << 17) * CFG.tick_ms as i64) / CFG.breath_period_ms as i64) as i32
};

/// Lower bound of the breathing envelope, in Q16.
const MIN_Q16: i32 = (CFG.min_intensity as i32) << 16;
/// Upper bound of the breathing envelope, in Q16.
const MAX_Q16: i32 = (CFG.max_intensity as i32) << 16;

/// Gamma (square-law) brightness table, normalized to Q16.
///
/// Lives in `.rodata` (flash) and is shared by all channels. The previous
/// runtime-built table cost 512 bytes of RAM *per task*; scaling this constant
/// by `top` at the point of use costs one multiply and one shift instead.
const GAMMA_Q16: [u16; 256] = {
    let mut table = [0u16; 256];
    let mut i = 0usize;
    while i < 256 {
        // 64-bit intermediates leave plenty of headroom; the `+ 32512` is a
        // proper round-to-nearest (half of the 65025 divisor).
        let squared = (i * i) as u64; // 0..=65025
        table[i] = ((squared * 65535 + 32512) / 65025) as u16;
        i += 1;
    }
    table
};

/// Maps an intensity to a PWM compare value for the given `top`.
///
/// The result never exceeds `top`: `GAMMA_Q16` peaks at `u16::MAX`, and
/// `65535 * 65536 >> 16 == 65535`, so the product also stays inside `u32`.
#[inline]
fn duty_for(intensity: u8, top: u16) -> u16 {
    ((u32::from(GAMMA_Q16[intensity as usize]) * (u32::from(top) + 1)) >> 16) as u16
}

/// State of a single fire effect simulation.
///
/// One instance per LED channel, so the channels flicker independently. The
/// configuration is deliberately *not* stored here: it is the global [`CFG`]
/// constant, which keeps this struct at 16 bytes and lets the optimizer fold
/// every configuration-derived operand into an immediate.
struct FireState {
    rng: u32,
    base_q16: i32,
    out_q8: i32,
    pulse: i32,
    rising: bool,
}

impl FireState {
    const fn new(seed: u32) -> Self {
        Self {
            rng: seed,
            base_q16: MIN_Q16,
            out_q8: (CFG.min_intensity as i32) << 8,
            pulse: 0,
            rising: true,
        }
    }

    /// Advances the simulation by one tick and returns the new intensity (0..=255).
    fn update(&mut self) -> u8 {
        // --- base triangular "breathing" envelope ---
        if self.rising {
            self.base_q16 += STEP_PER_TICK_Q16;
            if self.base_q16 >= MAX_Q16 {
                self.base_q16 = MAX_Q16;
                self.rising = false;
            }
        } else {
            self.base_q16 -= STEP_PER_TICK_Q16;
            if self.base_q16 <= MIN_Q16 {
                self.base_q16 = MIN_Q16;
                self.rising = true;
            }
        }
        let base = self.base_q16 >> 16;

        // One PRNG step per tick; the jitter and pulse decisions read
        // independent byte fields of the same word.
        self.rng = xorshift32(self.rng);
        let rand = self.rng;

        // --- random jitter around base ---
        // Scale signed 8-bit noise to approximately [-jitter_max, +jitter_max].
        let noise = i16::from((rand >> 24) as u8) - 128; // -128..=127
        let jitter = (i32::from(noise) * (2 * i32::from(CFG.jitter_max) + 1)) >> 8;

        // --- sporadic short pulses ---
        if rand & 0xFF < u32::from(CFG.pulse_prob) {
            self.pulse += i32::from(CFG.pulse_boost);
        }
        // Exponential-like decay
        self.pulse = (self.pulse * i32::from(CFG.pulse_decay_q8)) >> 8;

        // Combine and clamp target intensity to 0..255
        let target = (base + jitter + self.pulse).clamp(0, 255);

        // --- smooth the output ---
        // The `+ 128` makes the shift round to nearest; a bare arithmetic shift
        // rounds toward negative infinity and biases the output downward.
        let diff = (target << 8) - self.out_q8;
        self.out_q8 += (diff * i32::from(CFG.smooth_q8) + 128) >> 8;

        (self.out_q8 >> 8) as u8
    }
}

/// Drives all three LED channels with a warm campfire effect.
///
/// GPIO 16 and 17 share PWM slice 0 (channels A and B), GPIO 18 uses slice 1
/// (channel A). A single [`Ticker`] paces the update loop: unlike
/// `Timer::after`, it keeps a fixed cadence instead of accumulating the runtime
/// of the loop body as drift.
#[embassy_executor::task]
async fn fire_task(
    watchdog: Peri<'static, WATCHDOG>,
    slice0: Peri<'static, PWM_SLICE0>,
    pin_16: Peri<'static, PIN_16>,
    pin_17: Peri<'static, PIN_17>,
    slice1: Peri<'static, PWM_SLICE1>,
    pin_18: Peri<'static, PIN_18>,
) {
    let top = calculate_top(CFG.pwm_freq_hz, CFG.pwm_divider);

    let mut config = Config::default();
    config.top = top;
    config.divider = CFG.pwm_divider.into();

    let mut pwm_slice0 = Pwm::new_output_ab(slice0, pin_16, pin_17, config.clone());
    let mut pwm_slice1 = Pwm::new_output_a(slice1, pin_18, config.clone());

    // Distinct seeds keep the three flames from flickering in sync.
    let mut sim = [
        FireState::new(0xC0FF_EE01),
        FireState::new(0x1234_5678),
        FireState::new(0xDEAD_BEEF),
    ];

    let mut watchdog = Watchdog::new(watchdog);
    // Do not reset while halted in a debugger.
    watchdog.pause_on_debug(true);
    watchdog.start(WATCHDOG_TIMEOUT);

    let mut ticker = Ticker::every(Duration::from_millis(u64::from(CFG.tick_ms)));
    loop {
        // `set_config` writes both compare registers in a single store. It also
        // rewrites div/top/csr, which is a handful of redundant stores per tick
        // but avoids the read-modify-write pairs and fallible API of the
        // per-channel `PwmOutput` handles.
        config.compare_a = duty_for(sim[0].update(), top);
        config.compare_b = duty_for(sim[1].update(), top);
        pwm_slice0.set_config(&config);

        config.compare_a = duty_for(sim[2].update(), top);
        config.compare_b = 0; // channel B of slice 1 is not routed to a pin
        pwm_slice1.set_config(&config);

        watchdog.feed(WATCHDOG_TIMEOUT);
        ticker.next().await;
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

/// Calculates the PWM timer top value for the desired frequency and clock divider.
///
/// Choose a small divider (often 1) to maximize duty resolution at a given PWM
/// frequency, while keeping `top` within u16. `divider` is validated to be
/// non-zero at compile time, so the division below cannot trap.
fn calculate_top(desired_freq_hz: u32, divider: u8) -> u16 {
    let clock_freq_hz = embassy_rp::clocks::clk_sys_freq();
    let period_counts = clock_freq_hz / (desired_freq_hz * u32::from(divider));
    // A slice counts 0..=top, so a full 16-bit period is 65536 counts.
    // Clamping to u16::MAX here would lose one count.
    let period_counts = period_counts.clamp(1, 1 << 16);
    (period_counts - 1) as u16
}
