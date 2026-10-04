# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Watchdog that resets the board if the effect stops updating, and a panic handler that resets instead of halting in deployed builds. Debug builds keep `panic-probe` so an attached debugger can inspect faults.
- Compile-time validation of `FireConfig`: invalid combinations now fail the build with an explanatory message.
- `release-debug` build profile that keeps the symbol table `defmt` needs to decode RTT output.
- `rust-toolchain.toml` pinning the stable channel and installing the bare-metal target automatically.
- Lint configuration (`clippy::all`, `missing_docs`, `unsafe_code = "deny"`).

### Changed

- Update the release link in CHANGELOG.md.
- Update dependencies, including major upgrades to embassy-executor 0.10 and embassy-rp 0.10.
- Drive all three LED channels from a single task and a single `Ticker` instead of two tasks using `Timer::after`, which removes timer drift and halves the number of wakeups.
- Make `FireConfig` a compile-time constant so all derived values are folded by the optimizer.
- Move the gamma table to flash as a shared constant instead of building a copy in RAM per task.
- Set `opt-level = "s"` for release builds; `opt-level = 3` nearly doubled `.text` for no benefit in this workload.
- Add `[profile.dev]` with `opt-level = 1`.
- Default `DEFMT_LOG` to `info` so the `debug` feature actually produces output.
- Update the bundled Windows `pico-sdk-tools` archive in `libs/` from 2.3.0 to 2.3.1 (`pioasm` only; `picotool` is not bundled).

### Fixed

- Breathing envelope could freeze permanently when `min_intensity`/`max_intensity`/`breath_period_ms` made the integer envelope step truncate to zero. The step is now computed in Q16 in a single 64-bit expression, which also cuts the breath period error from 0.8 % to 0.2 %.
- `.text` was not 8-byte aligned, which produced a linker warning.
- `calculate_top` clamped the period to 65535 instead of 65536 counts, losing one count at the extreme, and would divide by zero for `pwm_divider = 0`.
- Gamma table rounding used `+ 32` where half of the divisor (`+ 32512`) was intended, so values were effectively truncated.
- Output smoothing rounded toward negative infinity, biasing brightness slightly downward.
- Replace a leftover `expect("TODO: panic message")` placeholder.
- Correct the `picotool` program description, which still named GPIO 25.
- Align the crate version in `Cargo.toml` with the released `v1.0.0` tag.
- Correct README: it described a single LED on GPIO 16 and referenced a `fire_led_task` signature that no longer exists.


## [1.0.0](https://github.com/greluc/blaustein-led/releases/tag/v1.0.0) – 2025-12-11

### Added

- Initial release.
- Smooth, configurable campfire flicker effect on GPIO 16 using PWM (Raspberry Pi Pico 2).
- `FireConfig` parameters to tune look & feel (breathing, jitter, pulse, smoothing, timing).
- Documentation update: hardware wiring, configuration guide, and build/flash instructions.