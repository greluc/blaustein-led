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

Smooth, configurable campfire flicker effect for three orange LEDs on Raspberry Pi Pico 2 (RP235x). The LEDs are driven from GPIO 16, 17 and 18 using high‑frequency PWM to create a natural, breathing fire look. Each channel runs its own simulation with a different seed, so the flames do not move in lockstep.

The effect is tuned using integer math (no floats, no allocation) and is optimized for embedded systems: the configuration is a compile‑time constant, the gamma table lives in flash rather than RAM, and all three channels are driven from a single task and a single timer. A watchdog restarts the board if the effect ever stops updating.

## Hardware

- Microcontroller: Raspberry Pi Pico 2 (RP235x)
- LEDs: Orange LEDs recommended (any LED works)
- Pins: GPIO16 and GPIO17 (PWM slice 0, channels A and B), GPIO18 (PWM slice 1, channel A)
- Wiring, per channel: GPIO → series resistor (330–1kΩ) → LED anode; LED cathode → GND.

Note: Ensure you use a proper series resistor on every channel. The code assumes active‑high LEDs.

## Configuration

All parameters live in `src/main.rs` in the `FireConfig` struct and can be tweaked to taste. Defaults are chosen for a smooth, warm indoor campfire.

Parameters:

- `pwm_freq_hz` (u32): PWM frequency. 20–30 kHz keeps PWM inaudible and flicker‑free.
- `pwm_divider` (u8): Clock divider for PWM. Use the smallest value that keeps the computed `top` within `u16`.
- `min_intensity` / `max_intensity` (u8): Base brightness range (0..=255). Increase `min_intensity` if your LED is too dim.
- `breath_period_ms` (u32): Full up+down cycle of the slow “breathing” envelope in milliseconds. 3–5 seconds feels natural.
- `jitter_max` (u8): Per‑tick random jitter around the base (0..=64 typical). Larger values look more chaotic.
- `pulse_prob` (u8): Probability per tick (0..=255) of a brief flame “lick”. Small values (~2–5) are subtle (~1 % per tick); the shipped value of 48 is about 19 % per tick, a visibly busy fire.
- `pulse_boost` (u8): How much a pulse adds when it triggers.
- `pulse_decay_q8` (u8): Q8 decay factor (0..=255) for pulses per tick. 224=fast decay, 248=slower.
- `smooth_q8` (u8): Q8 smoothing factor for the output EMA. Smaller means smoother (and slower to react).
- `tick_ms` (u32): Update interval in milliseconds. 10–20 ms works well.

To change the look, edit the `CFG` constant in `src/main.rs`:

```rust
const CFG: FireConfig = FireConfig {
    min_intensity: 16,
    max_intensity: 255,
    breath_period_ms: 4_000,
    jitter_max: 22,
    pulse_prob: 4,
    pulse_boost: 50,
    ..FireConfig::new()
};
```

`CFG` is a `const`, so the compiler folds every derived value into the binary — the configuration costs no RAM and no runtime cycles. Invalid combinations (for example a `breath_period_ms` so long that the envelope step truncates to zero and the breathing would freeze) fail the build with an explanatory message instead of producing dead firmware.

## Build & Flash

- Toolchain target: `thumbv8m.main-none-eabihf`
- `rust-toolchain.toml` pins the stable channel and installs the target automatically, so no manual `rustup target add` is needed.
- Runner is configured in `.cargo/config.toml` to use `picotool`.

Steps:

1. Build: `cargo build --release`
2. Flash/load (via runner): `cargo run --release`

Alternatively, you can use `picotool` directly with the built `.elf` from `target/thumbv8m.main-none-eabihf/release/blaustein-led`.

### Build profiles

| Profile | Command | Purpose |
| --- | --- | --- |
| `release` | `cargo build --release` | Deployment. No defmt stack; panics reset the board so the installation recovers unattended. |
| `release-debug` | `cargo build --profile release-debug --features debug` | Optimized build that keeps the symbol table `defmt` needs to decode RTT output. Panics halt for an attached debugger. |
| `dev` | `cargo build` | `opt-level = 1`, full debug info. |

Set the log level for `debug` builds with `DEFMT_LOG` (defaults to `info` in `.cargo/config.toml`).

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