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
- Update `cortex-m-rt` to 0.7.7 and refresh transitive patch/minor releases in `Cargo.lock` (bitflags, cc, cfg-if, either, find-msvc-tools, generator, indexmap, lazy_static, libc, log, siphasher, smallvec, thiserror, unicode-ident, zerocopy). `embedded-storage` 0.3.2 and `embedded-storage-async` 0.4.2 stay out because they require Rust 1.87, above the declared `rust-version` of 1.85.
- Raise `rust-version` to 1.99.1, the current stable toolchain, and update `cortex-m` to 0.7.10, `defmt` to 1.2.0 and `defmt-rtt` to 1.3.1. `Cargo.lock` now also picks up `embedded-storage` 0.3.2, `embedded-storage-async` 0.4.2, `fixed` 1.31.0, `heapless` 0.9.3 and current patch releases of the remaining transitive crates.
- Add `.gitattributes` so all text files are checked out with LF line endings on every platform and the bundled archive is treated as binary.
- Correct the author address in `Cargo.toml`.
- Split `src/main.rs` into single-purpose modules: `config` (configuration and its validation), `fire_state` (per-channel simulation), `pwm` (gamma table, duty and `top`), and `fixed_point`.
- Deny all Clippy groups (`all`, `pedantic`, `nursery`, `cargo`), a curated set of restriction lints and all compiler warnings; add a strict `rustfmt.toml` (formatting needs the nightly `rustfmt`).
- Simplify the breathing envelope to a signed step that flips at the bounds, read jitter and flare bytes by destructuring `to_be_bytes`, and compute `top` with `saturating_sub` and `u16::try_from` instead of a clamp and a cast.
- Seed the channels in `const` blocks, so a zero seed fails the build; replace `#[allow]` with `#[expect(…, reason = …)]`; let the fire task return `!`.
- Use the `cargo::` build-script syntax and `fs::write` in `build.rs`.
- Remove the unused `rp235x_riscv.x` linker script, the commented-out runners in `.cargo/config.toml`, and the unrelated language templates from `.gitignore`.

### Fixed

- A `breath_period_ms` shorter than `tick_ms` could overflow the envelope step, and `smooth_q8 = 0` froze the output; both now fail the build.
- The comment on `pulse_decay_q8 = 232` claimed a ~150 ms half-life; it is ~105 ms at a 15 ms tick.

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