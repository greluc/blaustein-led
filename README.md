# Blaustein LED

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg?style=flat-square)](LICENSE)
[![Contributor Covenant](https://img.shields.io/badge/Contributor%20Covenant-v3.0%20adopted-ff69b4.svg?style=flat-square)](CODE_OF_CONDUCT.md)

## Table Of Contents

- [Description](#description)
- [Hardware](#hardware)
- [Configuration](#configuration)
- [Build & Flash](#build--flash)
- [Current release version](#current-release-version)
- [License](#license)
- [Contribution](#contribution)
- [Team](#team)

## Description

Smooth, configurable campfire flicker effect for an orange LED on Raspberry Pi Pico 2 (RP235x). The LED is driven from GPIO 16 using high‑frequency PWM to create a natural, breathing fire look. The effect is tuned using integer math (no floats, no allocation) and is optimized for embedded systems.

## Hardware

- Microcontroller: Raspberry Pi Pico 2 (RP235x)
- LED: Orange LED recommended (any LED works)
- Pin: GPIO16
- Wiring: GPIO16 → series resistor (330–1kΩ) → LED anode; LED cathode → GND.

Note: Ensure you use a proper series resistor. The code assumes an active‑high LED on GPIO16.

## Configuration

All parameters live in `src/main.rs` in the `FireConfig` struct and can be tweaked to taste. Defaults are chosen for a smooth, warm indoor campfire.

Parameters:

- `pwm_freq_hz` (u32): PWM frequency. 20–30 kHz keeps PWM inaudible and flicker‑free.
- `pwm_divider` (u8): Clock divider for PWM. Use the smallest value that keeps the computed `top` within `u16`.
- `min_intensity` / `max_intensity` (u8): Base brightness range (0..=255). Increase `min_intensity` if your LED is too dim.
- `breath_period_ms` (u32): Full up+down cycle of the slow “breathing” envelope in milliseconds. 3–5 seconds feels natural.
- `jitter_max` (u8): Per‑tick random jitter around the base (0..=64 typical). Larger values look more chaotic.
- `pulse_prob` (u8): Probability per tick (0..=255) of a brief flame “lick”. Small values (~2–5) are subtle.
- `pulse_boost` (u8): How much a pulse adds when it triggers.
- `pulse_decay_q8` (u8): Q8 decay factor (0..=255) for pulses per tick. 224=fast decay, 248=slower.
- `smooth_q8` (u8): Q8 smoothing factor for the output EMA. Smaller means smoother (and slower to react).
- `tick_ms` (u32): Update interval in milliseconds. 10–20 ms works well.

To change the look, modify the spawn call in `main`:

```text
spawner.spawn(
    fire_led_task(p.PWM_SLICE4, p.PIN_25, FireConfig {
        min_intensity: 16,
        max_intensity: 255,
        breath_period_ms: 4_000,
        jitter_max: 22,
        pulse_prob: 4,
        pulse_boost: 50,
        ..FireConfig::default()
    })
).unwrap();
```

## Build & Flash

- Toolchain target: `thumbv8m.main-none-eabihf`
- Runner is configured in `.cargo/config.toml` to use `picotool`.

Steps:

1. Install target (once): `rustup target add thumbv8m.main-none-eabihf`
2. Build: `cargo build --release`
3. Flash/load (via runner): `cargo run --release`

Alternatively, you can use `picotool` directly with the built `.elf` from `target/thumbv8m.main-none-eabihf/release/blaustein-led`.

## Current Release Version

v1.0.0

## License

Blaustein LED is licensed under the MIT License.
You can find the license text under [LICENSE](LICENSE).

For more information about this license,
visit [https://choosealicense.com/licenses/mit/](https://choosealicense.com/licenses/mit/).

## Contribution

- You can find the source code under [https://github.com/greluc/blaustein-led](https://github.com/greluc/blaustein-led).
- Read [CONTRIBUTING](CONTRIBUTING.md) when you wish to contribute to this project.
- Note that this project is subject to a Contributor [CODE OF CONDUCT](CODE_OF_CONDUCT.md).
  By participating in this project, you agree to abide by its terms.

## Team

- Lucas Greuloch (@greluc)

[<img src="Logo_Rust.svg" height="150"/>](https://www.rust-lang.org/)