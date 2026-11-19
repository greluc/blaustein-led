# CLAUDE.md

Instructions for coding agents (Claude Code, Codex and others) working in this repository.
[CONTRIBUTING.md](CONTRIBUTING.md) is the source for human contributors; this file adds what an
agent needs and does not repeat it.

## Project

Firmware for the campfire LEDs of Burg Blaustein on a Raspberry Pi Pico 2 (RP2350), written in Rust
with Embassy. A Cargo workspace with two packages:

- `fire-effect/`: `no_std` library without dependencies. `InstallationConfig` and `Installation`
  (PWM and tick), `FireConfig` (the look of a role), its presets and compile-time validation,
  `FireProfile`, the generic `FireState<S, P>` simulation, the `Follower`, the 16-bit `Intensity`,
  `LightingConfig` and `Lighting` (role looks, follower, `IgnitionConfig`, the table of `Channel`s
  driven by a `Light`, and their validation), the `Lights<L, N>` bank that runs every channel with
  ignition and trim, the `Identify` pattern, the seed mixing, the gamma curve, the PWM `top`
  calculation and the slice phases. All logic and all tests live here.
- `/` (`blaustein-led`): the binary for `thumbv8m.main-none-eabihf`. `src/config.rs` holds the
  installation-wide `INSTALLATION`, the installed `LIGHTING` with the channel table, `CHANNELS`
  (1 to 16, the length of the table) and the `Blaustein` installation; `src/outputs.rs` routes,
  phases and drives the PWM slices in use (channels on GPIO 7 to GPIO 22, GPIO `n` = channel
  `n mod 16`); `src/effect.rs` picks the effect of the build (`Lights`, seeded from the ring
  oscillator, or `Identify` with the `identify` feature); `src/main.rs` wires the pins, the
  watchdog, the panic handler, the picotool metadata and the task.

## Commands

| Purpose | Command |
| --- | --- |
| Format | `cargo +nightly fmt --all` |
| Lint the firmware | `cargo clippy --release`, `cargo clippy --release --features identify` and `cargo clippy --profile release-debug --features debug` |
| Lint `fire-effect` with tests | `cargo clippy-host` |
| Test | `cargo test-host` |
| Dependencies | `cargo deny check` |
| Build | `cargo build --release` and `cargo build --release --features identify` |
| Image size | `llvm-size -A target/thumbv8m.main-none-eabihf/release/blaustein-led` (from the `llvm-tools` component) |

The default target is the RP2350, so host tests need the `test-host` / `clippy-host` aliases from
`.cargo/config.toml`. All of these must pass before a commit.

## Rules

- **Style.** The [Google Rust Style Guide](https://google.github.io/styleguide/rustguide.html)
  applies; deviations need the maintainer's approval and a note here.
- **Comments.** Rustdoc comments on items, short technical comments where the code cannot say it
  itself. No history ("previously", dates, issue numbers), no commented-out code, no `TODO`.
- **Lints.** Never weaken the lint tables in `Cargo.toml`. Suppress a single finding with
  `#[expect(lint, reason = "...")]` only where the lint is wrong for that spot.
- **Firmware constraints.** No floating point, no allocation, no new `unsafe`. Values derived from
  the configuration stay compile-time constants; check the release image size after changes to the
  hot path and state it in the pull request.
- **Behaviour.** `baseline_output_is_stable` in `fire-effect/src/fire_state.rs` pins the look of the
  flame. Update its values only for an intended change and say so in the commit message.
- **Licence headers.** New code and configuration files start with the SPDX header
  (`SPDX-FileCopyrightText: <year> the Blaustein LED contributors`,
  `SPDX-License-Identifier: MIT OR Apache-2.0`).
- **Modularity.** Small modules with one responsibility; the firmware depends on `fire-effect`,
  never the other way round.
- **No shortcuts.** No dead code, duplication, magic numbers or swallowed errors. A clean fix that
  is out of scope becomes a follow-up issue, not a comment.

## Git

- Branch from `main`; never commit to `main` directly.
- Every commit is signed off with `git commit -s` (DCO gate); never type the trailer by hand.
- Commit messages follow Conventional Commits and carry the history of the change.
- Add an entry under `[Unreleased]` in `CHANGELOG.md` for every visible change.
- A stacked pull request is retargeted to `main` once the lower one is merged; check that its
  changes actually reached `main`.
