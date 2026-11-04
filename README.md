<p align="center">
  <img src="docs/design/logo.svg" alt="Blaustein LED logo: a castle tower with a flame burning in its window" width="128" height="128">
</p>

# Blaustein LED

[![CI](https://github.com/greluc/blaustein-led/actions/workflows/ci.yml/badge.svg)](https://github.com/greluc/blaustein-led/actions/workflows/ci.yml)
[![Repository gates](https://github.com/greluc/blaustein-led/actions/workflows/repository-gates.yml/badge.svg)](https://github.com/greluc/blaustein-led/actions/workflows/repository-gates.yml)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)
[![Contributor Covenant](https://img.shields.io/badge/Contributor%20Covenant-3.0-ff69b4.svg)](CODE_OF_CONDUCT.md)

Blaustein LED is the firmware for the campfire lights of Burg Blaustein: a smooth, configurable
flicker on up to sixteen LEDs, driven by a Raspberry Pi Pico 2 (RP2350). Each LED runs its own simulation
with its own seed, so the flames never move in lockstep, and a watchdog brings the effect back on
its own if it ever stops.

The current release is **[1.0.0](https://github.com/greluc/blaustein-led/releases/tag/v1.0.0)**.
Blaustein LED is written in Rust with [Embassy](https://embassy.dev) and licensed under either of
the [Apache License, Version 2.0](LICENSE-APACHE) or the [MIT licence](LICENSE-MIT) at your option.
The source lives at [github.com/greluc/blaustein-led](https://github.com/greluc/blaustein-led).

## What it does

- **Flame.** A slow triangular "breathing" envelope, per-tick random jitter, sporadic flares that
  decay exponentially, and an output EMA that smooths it all, mapped through a square-law gamma
  curve to 25 kHz PWM.
- **One to sixteen channels, one task.** GPIO 7 to GPIO 22 reach all sixteen PWM outputs of the
  Pico 2, each with its own flame. `CHANNELS` sets how many are in use; only those get a
  simulation, a PWM slice and RAM. One task and one `Ticker` drive them all, so the cadence never
  drifts.
- **Integer only.** No floating point and no allocation. The configuration is a compile-time
  constant, so every derived value folds into the binary. The release image uses about 4.1 KB of
  flash and 464 bytes of static RAM for one LED, and 5.1 KB and 880 bytes for sixteen.
- **Unattended.** In deployed builds a panic resets the chip instead of halting, and a watchdog
  resets it if the effect stops updating. Invalid configurations fail the build.

## Hardware

- Board: Raspberry Pi Pico 2 (RP2350A).
- LEDs: orange LEDs look best; any LED works.
- Pins: GPIO 7 to GPIO 22, one LED each. GPIO `n` is PWM channel `n mod 16`, so this block uses
  every PWM output of the RP2350A exactly once. `CHANNELS` LEDs are used in the order GPIO 16 to
  GPIO 22, then GPIO 7 to GPIO 15 (slice by slice, A before B):

  | PWM slice | 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 |
  | --- | --- | --- | --- | --- | --- | --- | --- | --- |
  | Channel A | GPIO 16 | GPIO 18 | GPIO 20 | GPIO 22 | GPIO 8 | GPIO 10 | GPIO 12 | GPIO 14 |
  | Channel B | GPIO 17 | GPIO 19 | GPIO 21 | GPIO 7 | GPIO 9 | GPIO 11 | GPIO 13 | GPIO 15 |

- Wiring per channel: GPIO → series resistor (330 Ω to 1 kΩ) → LED anode; LED cathode → GND.

Use a series resistor on every channel. The firmware assumes active-high LEDs. A GPIO drives only
a few milliamperes, and the sum over all pins is limited as well; with more than a handful of LEDs,
or for brighter ones, switch each LED through a small NPN transistor or N-channel MOSFET (or use an
LED driver) and feed it from the 5 V or 3.3 V supply instead of the pin.

## Configuration

The parameters are the fields of `FireConfig` in
[`fire-effect/src/config.rs`](fire-effect/src/config.rs); the installed values are the `CFG`
constant in [`src/config.rs`](src/config.rs). The baseline (`FireConfig::new()`) is tuned for a
smooth, warm indoor campfire.

| Parameter | Type | Meaning |
| --- | --- | --- |
| `pwm_freq_hz` | `u32` | PWM frequency. 20 to 30 kHz keeps PWM inaudible and flicker-free. |
| `pwm_divider` | `u8` | PWM clock divider, at least 1. Use the smallest value that keeps `top` within `u16`. |
| `min_intensity` / `max_intensity` | `u8` | Breathing range (0..=255); `max_intensity` must be greater. Raise `min_intensity` if the LED is too dim. |
| `breath_period_ms` | `u32` | One full up and down cycle of the breathing, at least one tick. 3 to 5 s feels natural. |
| `jitter_max` | `u8` | Random per-tick jitter around the breathing (0..=64 typical). Larger looks more chaotic. |
| `pulse_prob` | `u8` | Chance per tick (0..=255) of a short flare. 2 to 5 is subtle (about 1 %); 48 is about 19 %, a busy fire. |
| `pulse_boost` | `u8` | Intensity a flare adds. |
| `pulse_decay_q8` | `u8` | Q8 decay factor of a flare per tick. 224 decays fast, 248 slower. |
| `smooth_q8` | `u8` | Q8 smoothing factor of the output, at least 1. Smaller is smoother and slower to react. |
| `tick_ms` | `u32` | Update interval in milliseconds. 10 to 20 ms works well. |

Set the number of LEDs with `CHANNELS` (1 to 16) in the same file. One LED uses GPIO 16, three use
GPIO 16 to 18, sixteen use GPIO 7 to 22; a value outside 1 to 16 fails the build:

```rust
pub const CHANNELS: usize = 3;
```

To change the look, edit `CFG`:

```rust
pub const CFG: FireConfig = FireConfig {
    min_intensity: 16,
    max_intensity: 255,
    breath_period_ms: 4_000,
    jitter_max: 22,
    pulse_prob: 4,
    pulse_boost: 50,
    ..FireConfig::new()
};
```

An invalid combination, for example a `breath_period_ms` so long that the envelope step truncates
to zero, fails the build with an explanatory message instead of producing dead firmware.

## Build and flash

Requires [rustup](https://rustup.rs) and, to flash, [picotool](https://github.com/raspberrypi/picotool)
on your `PATH`. `rust-toolchain.toml` selects the stable toolchain (1.99.1 or newer) and installs
the `thumbv8m.main-none-eabihf` target on first use.

1. Hold BOOTSEL while plugging in the Pico 2.
2. Build and flash in one step: `cargo run --release`.

`cargo run` uses the `picotool` runner from [`.cargo/config.toml`](.cargo/config.toml). To flash by
hand, build with `cargo build --release` and load
`target/thumbv8m.main-none-eabihf/release/blaustein-led` with `picotool load -x -t elf`. Every CI
run also uploads that image as the `blaustein-led-elf` artifact.

| Profile | Command | Purpose |
| --- | --- | --- |
| `release` | `cargo build --release` | Deployment. No defmt stack; a panic resets the board so the installation recovers unattended. |
| `release-debug` | `cargo build --profile release-debug --features debug` | Optimised build that keeps the symbol table defmt needs to decode RTT output. A panic halts for an attached debugger. |
| `dev` | `cargo build` | `opt-level = 1`, full debug info. |

Set the log level of `debug` builds with `DEFMT_LOG` (default `info`, set in `.cargo/config.toml`).

## Development

| Step | Command |
| --- | --- |
| Format | `cargo +nightly fmt --all` |
| Lint the firmware | `cargo clippy --release` and `cargo clippy --profile release-debug --features debug` |
| Lint the effect crate and its tests | `cargo clippy-host` |
| Test | `cargo test-host` |
| Check dependencies | `cargo deny check` |

`Cargo.toml` denies every Clippy group (`all`, `pedantic`, `nursery`, `cargo`), a set of
restriction lints that fit an integer-only `no_std` firmware, and all compiler warnings.
`rustfmt.toml` enables the strict formatting options, most of which need the nightly `rustfmt`;
the code itself builds on stable. CI runs all of the above on every pull request.

### Project layout

| Path | Content |
| --- | --- |
| [`fire-effect/`](fire-effect/) | The flicker simulation, gamma curve and PWM timing. No hardware dependencies, so its tests run on the host (`cargo test-host`, an alias for `cargo test --package fire-effect --target host-tuple`). |
| [`src/main.rs`](src/main.rs) | The firmware: wires the simulation to the PWM slices, the watchdog and the panic handler. |
| [`src/config.rs`](src/config.rs) | The installed `CFG`. |
| [`memory.x`](memory.x), [`build.rs`](build.rs) | Memory layout of the RP2350 and the linker setup. |
| [`.github/`](.github/) | CI, repository gates, Dependabot, issue and pull request templates, security policy. |
| [`docs/design/`](docs/design/) | The logo (`logo.svg`) and the GitHub social preview (`social-preview.svg`, rendered to `social-preview.png`). |

## Documentation

| Topic | Document |
| --- | --- |
| Contributing, checks and commit rules | [CONTRIBUTING.md](CONTRIBUTING.md) |
| Reporting a vulnerability | [.github/SECURITY.md](.github/SECURITY.md) |
| Code of Conduct | [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) |
| Release notes | [CHANGELOG.md](CHANGELOG.md) |
| Instructions for coding agents | [CLAUDE.md](CLAUDE.md) |

## Contributing

Read [CONTRIBUTING.md](CONTRIBUTING.md) first: the checks, the tests and the sign-off of every
commit. Security issues are reported as described in [.github/SECURITY.md](.github/SECURITY.md).
Everyone taking part is bound by the [Code of Conduct](CODE_OF_CONDUCT.md).

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  <https://www.apache.org/licenses/LICENSE-2.0>)
- MIT licence ([LICENSE-MIT](LICENSE-MIT) or <https://opensource.org/licenses/MIT>)

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in
the work by you, as defined in the Apache-2.0 licence, shall be dual licensed as above, without any
additional terms or conditions.

[<img src="Logo_Rust.svg" alt="Rust logo" height="96">](https://www.rust-lang.org/)
