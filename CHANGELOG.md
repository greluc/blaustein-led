# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project
adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Watchdog that resets the board if the effect stops updating, and a panic handler that resets
  instead of halting in deployed builds. Debug builds keep `panic-probe`, so an attached debugger
  can inspect faults.
- Compile-time validation of the configuration: invalid combinations fail the build with an
  explanatory message.
- `release-debug` build profile that keeps the symbol table defmt needs to decode RTT output.
- `rust-toolchain.toml` selecting the stable toolchain and installing the bare-metal target.
- `fire-effect` workspace crate holding the simulation, gamma curve and PWM timing without hardware
  dependencies, with 24 host-run unit tests (`cargo test-host`): configuration validation, envelope
  bounds and period, jitter range, determinism, flares, gamma curve, duty range, `top`, and a
  regression fingerprint of the baseline flicker.
- Strict quality gates: every Clippy group (`all`, `pedantic`, `nursery`, `cargo`), a curated set
  of restriction lints and all compiler warnings are denied; `rustfmt.toml` enables the strict
  formatting options (nightly `rustfmt`); `clippy.toml` and `deny.toml` (cargo-deny) configure the
  remaining checks.
- GitHub configuration: CI for formatting, Clippy, host tests and all three firmware profiles with
  a size report and the release image as artifact; repository gates for cargo-deny, gitleaks,
  actionlint, zizmor and the DCO sign-off; Dependabot for Cargo and GitHub Actions; issue and pull
  request templates; `CODEOWNERS`; a security policy.
- `CLAUDE.md` and `AGENTS.md` for coding agents, and `.editorconfig`.
- SPDX licence headers in every code and configuration file.

### Changed

- Licence: Blaustein LED is now dual licensed under `MIT OR Apache-2.0`; the licence texts are
  `LICENSE-MIT` and `LICENSE-APACHE`.
- Update all dependencies to their newest releases, including embassy-executor 0.10, embassy-rp
  0.10, cortex-m 0.7.10, cortex-m-rt 0.7.7, defmt 1.2 and defmt-rtt 1.3.1, and raise
  `rust-version` to 1.99.1.
- Drive all three LED channels from a single task and a single `Ticker` instead of two tasks using
  `Timer::after`, which removes timer drift and halves the number of wakeups.
- Make the configuration a compile-time constant, so all derived values are folded into the
  binary; `FireState` is generic over a `FireProfile` that supplies it.
- Compute the square-law gamma curve instead of keeping a table, and build releases with
  `opt-level = "z"`: the release image uses 3340 bytes of flash for code and constants.
- Add `[profile.dev]` with `opt-level = 1` and default `DEFMT_LOG` to `info`.
- Check out text files with LF line endings on every platform (`.gitattributes`).
- Rewrite `README.md`, `CONTRIBUTING.md` and the security policy for the current project, and
  replace the retired contact address.

### Removed

- The bundled Windows `pico-sdk-tools` archive in `libs/`; it only contained `pioasm` for the C
  SDK, which this firmware never uses.
- The unused `rp235x_riscv.x` linker script, the commented-out runners in `.cargo/config.toml` and
  the unrelated language templates in `.gitignore`.

### Fixed

- The breathing envelope could freeze when the integer envelope step truncated to zero, and a
  `breath_period_ms` shorter than `tick_ms` could overflow it. The step is computed in Q16 in one
  64-bit expression, which also cuts the breath period error from 0.8 % to 0.2 %, and both cases
  fail the build.
- `smooth_q8 = 0` froze the output; it now fails the build.
- The jitter rounded toward negative infinity, which darkened the flame by half a step on average
  and left a `-1` jitter on half the ticks even with `jitter_max = 0`. It now rounds to nearest and
  spans `-jitter_max..=jitter_max` symmetrically.
- The output smoothing rounded toward negative infinity, biasing the brightness slightly downward.
- The gamma rounding used `+ 32` where half the divisor (`+ 32512`) was intended, so values were
  truncated.
- `calculate_top` clamped the period to 65535 instead of 65536 counts and would divide by zero for
  `pwm_divider = 0`.
- `.text` was not 8-byte aligned, which produced a linker warning.
- The comment on `pulse_decay_q8 = 232` claimed a ~150 ms half-life; it is ~105 ms at a 15 ms
  tick.
- Replace a leftover `expect("TODO: panic message")` placeholder.
- The picotool program description still named GPIO 25.
- Align the crate version with the released `v1.0.0` tag and correct the author address.
- The README described a single LED on GPIO 16 and a `fire_led_task` signature that no longer
  exists.

## [1.0.0](https://github.com/greluc/blaustein-led/releases/tag/v1.0.0) - 2025-12-11

### Added

- Initial release.
- Smooth, configurable campfire flicker effect on GPIO 16 using PWM (Raspberry Pi Pico 2).
- `FireConfig` parameters to tune the look (breathing, jitter, pulse, smoothing, timing).
- Documentation: hardware wiring, configuration guide, and build and flash instructions.

[Unreleased]: https://github.com/greluc/blaustein-led/compare/v1.0.0...HEAD
