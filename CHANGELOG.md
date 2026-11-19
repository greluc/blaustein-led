# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project
adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Documentation of the installation: `docs/lighting-plan.md` with the analysis of 2.0.0, the
  lighting concept, a draft channel plan for the full castle and every source that still has to
  be verified; `docs/hardware.md` with the electrical design, the power supply with a magnetic
  coupling, the installation rules and a bill of materials; `docs/research-brief.md` with the
  research a session with internet access still has to do.
- Presets for the light roles of a castle model: `FireConfig::HEARTH` (fireplace, cooking fire),
  `FireConfig::TORCH`, `FireConfig::CANDLE` (candle, oil lamp), `FireConfig::EMBER` (embers,
  forge) and `FireConfig::AMBIENT` (room light). They are starting points tuned in a host
  simulation and still have to be tuned on the real model.
- `Follower`: a light driven by another channel, configured by a `FollowerConfig` with its own
  floor, gain and smoothing time constant, such as `FollowerConfig::HEARTH_GLOW` for the red glow
  of the embers under a hearth flame. It takes the source's intensity each tick and needs 4 bytes
  of RAM.
- `InstallationConfig` and the `Installation` trait for the settings every light shares (PWM
  frequency, divider, tick), kept apart from the look of a role so two roles cannot disagree on
  them.
- `Intensity`, a 16-bit linear intensity newtype.
- Host tests that every pair of channels is uncorrelated (each below 0.15, on average below 0.05,
  over 50 minutes), that no role has a dominant period (autocorrelation below 0.15 from twice the
  longest swell up to 30 s), that every role stays within its bounds and keeps its character, that
  `peak` bounds the output and scales it without changing the motion, that the follower is
  smoother than its source, stays in bounds and tracks its level, and an exhaustive test of the
  gamma curve and duty mapping over all 65,536 intensities for six `top` values. The baseline
  regression test pins every preset and the hearth glow.

### Changed

- The README recommends about 1 kΩ per amber, orange or red LED and about 330 Ω per warm white
  LED instead of 330 Ω to 1 kΩ, and a budget of 4 mA per pin and 40 mA in total: 330 Ω on sixteen
  orange LEDs would peak at about 63 mA.
- The look of the flame changed. Three layers of smoothly interpolated value noise (a slow swell,
  a faster sway and a fast flicker), each gliding to a new random level after a random time, and
  occasional gusts that deepen the flicker replace the triangular breathing envelope, the jitter,
  the flares and the output smoothing. `FireConfig` is now the look of a role: `peak`, the
  `swell`, `sway` and `flicker` layers and `gust`; the PWM frequency, divider and tick moved to
  `InstallationConfig`, and `FireState<S, P>` takes the installation `S` next to the profile `P`.
- The firmware runs the hearth preset on every channel. It averages 33.6 % PWM duty, close to the
  32.4 % of the previous installed look, with a deeper and irregular swell and a livelier flicker.
  `src/config.rs` holds `INSTALLATION` next to `CFG`.
- The simulation returns a 16-bit `Intensity` instead of `u8`, and `pwm::duty_for` maps it with a
  stated rounding rule, `round(round(i² / 65535) * top / 65535)`: monotonic, dark maps to 0, full
  scale to `top`, and within one count of the exact square law.
- A channel needs 32 instead of 20 bytes of RAM. With sixteen channels the release image uses 5504
  bytes of flash for code and constants and 1072 bytes of static RAM, with one channel 4512 and
  480.

### Fixed

- All channels breathed together: every channel started the same deterministic envelope at the
  same point, so any two channels correlated by 0.65 (0.99 with the baseline configuration) and the
  whole castle swelled in a 5 s rhythm. Every channel now starts at a random point of every layer
  and moves at random times, with no period: over 50 minutes, two channels of the hearth correlate
  by at most 0.04 and by 0.01 on average.
- `max_intensity` bounded only the envelope; flares and jitter pushed the installed look up to 250
  of 255 although it was set to 128. `peak` now bounds the output by construction.
- Dim lights faded in visible steps, because the intensity had 8 bits before the gamma curve: from
  intensity 20 to 21 the duty jumped by 11 %. With 16 bits, adjacent intensities move the duty by
  at most one count at 25 kHz.

## [2.0.0](https://github.com/greluc/blaustein-led/releases/tag/v2.0.0) - 2026-11-04

### Added

- Up to sixteen LED channels instead of three: GPIO 7 to GPIO 22 reach every PWM output of the
  RP2350A once, each with its own flame. The existing LEDs on GPIO 16, 17 and 18 stay where they
  are. `CHANNELS` in `src/config.rs` (1 to 16) sets how many are used; only those get a
  simulation, a PWM slice and RAM, and a value outside the range fails the build.
  Seeds come from `channel_seed`, a MurmurHash3 mix of the channel index, so every channel flickers
  differently; the first three channels therefore flicker differently than before.
- Watchdog that resets the board if the effect stops updating, and a panic handler that resets
  instead of halting in deployed builds. Debug builds keep `panic-probe`, so an attached debugger
  can inspect faults.
- Compile-time validation of the configuration: invalid combinations fail the build with an
  explanatory message.
- `release-debug` build profile that keeps the symbol table defmt needs to decode RTT output.
- `rust-toolchain.toml` selecting the stable toolchain and installing the bare-metal target.
- `fire-effect` workspace crate holding the simulation, gamma curve and PWM timing without hardware
  dependencies, with 26 host-run unit tests (`cargo test-host`): configuration validation, envelope
  bounds and period, jitter range, determinism, channel seeds, flares, gamma curve, duty range, `top`, and a
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
- Project logo, a castle tower with a flame burning in its window (`docs/design/logo.svg`), shown
  at the top of the README, and a GitHub social preview image (`docs/design/social-preview.png`).
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
  `opt-level = "z"`: with three channels the release image used 3340 bytes of flash for code and constants, with sixteen it uses 5124.
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

[Unreleased]: https://github.com/greluc/blaustein-led/compare/v2.0.0...HEAD
