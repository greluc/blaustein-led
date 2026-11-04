# Contributing to Blaustein LED

Thanks for taking the time to contribute. This document explains how to ask, report, propose and
change things, and which checks a change has to pass.

## Table of contents

- [Code of Conduct](#code-of-conduct)
- [Asking questions](#asking-questions)
- [Reporting a security vulnerability](#reporting-a-security-vulnerability)
- [Reporting bugs](#reporting-bugs)
- [Suggesting features](#suggesting-features)
- [Local development setup](#local-development-setup)
- [Pull requests](#pull-requests)
- [Developer Certificate of Origin (DCO)](#developer-certificate-of-origin-dco)
- [Commit messages](#commit-messages)
- [Style guides](#style-guides)
- [Before-you-push checklist](#before-you-push-checklist)
- [Releasing](#releasing)
- [Licence](#licence)

## Code of Conduct

Everyone taking part is bound by the [Code of Conduct](CODE_OF_CONDUCT.md). Report unacceptable
behaviour to [lucas.greuloch@gmail.com](mailto:lucas.greuloch@gmail.com).

## Asking questions

Search the [issues](https://github.com/greluc/blaustein-led/issues) first. If your question is not
answered there, open an issue with the label `question`.

## Reporting a security vulnerability

Never report a security problem in a public issue. Follow [.github/SECURITY.md](.github/SECURITY.md)
and use GitHub's private vulnerability reporting.

## Reporting bugs

Open a [bug report](https://github.com/greluc/blaustein-led/issues/new?template=bug_report.yml). The
form asks for what you need to give: what happened and what you expected, the steps from a fresh
clone, the version or commit, the build profile, the board and LEDs, your toolchain, any change to
`CFG`, and the build output or defmt log. If you find a closed issue that matches, open a new one
and link the old one.

## Suggesting features

Open a [feature request](https://github.com/greluc/blaustein-led/issues/new?template=feature_request.yml)
and describe the look or behaviour you are after, why the current parameters do not give it, and
how it could work.

## Local development setup

1. Install [rustup](https://rustup.rs). `rust-toolchain.toml` selects the stable toolchain, the
   `clippy`, `rustfmt` and `llvm-tools` components and the `thumbv8m.main-none-eabihf` target.
2. Install the nightly `rustfmt`, which the strict `rustfmt.toml` needs:
   `rustup toolchain install nightly --profile minimal --component rustfmt`.
3. To flash, install [picotool](https://github.com/raspberrypi/picotool).
4. For dependency changes, install [cargo-deny](https://github.com/EmbarkStudios/cargo-deny):
   `cargo install --locked cargo-deny`.

The workspace has two packages:

- `fire-effect` is the flicker simulation, gamma curve and PWM timing. It has no dependencies and
  no hardware access; `cargo test-host` and `cargo clippy-host` run it on your machine.
- `blaustein-led` (the root package) is the firmware for the RP2350. It builds for
  `thumbv8m.main-none-eabihf`, the default target in `.cargo/config.toml`.

`fire-effect` holds a regression test of the baseline flicker. A change that alters the look on
purpose updates the sum and fingerprint in `baseline_output_is_stable` and says so in the pull
request.

## Pull requests

- Branch from `main` and open the pull request against `main`. Stacked pull requests are fine;
  delete the lower branch when it is merged, so GitHub moves the next one onto `main`.
- Fill in the [pull request template](.github/PULL_REQUEST_TEMPLATE.md) and link the issue.
- Keep a pull request to one topic. Split refactors from behaviour changes.
- CI (`CI` and `Repository gates`) has to pass. A change you could not test on a board is marked as
  such in the pull request.
- A visible change adds an entry to `CHANGELOG.md` under `[Unreleased]`.

## Developer Certificate of Origin (DCO)

Every commit is signed off under the [Developer Certificate of Origin](https://developercertificate.org):
by adding the trailer you certify that you wrote the change or have the right to submit it under
the project's licences.

```bash
git commit -s
```

`-s` adds `Signed-off-by: Your Name <your@email>` from `git config user.name` and `user.email`. Do
not type the trailer by hand; it has to match the commit author, and the `DCO sign-off` gate checks
that. To add a missing sign-off, run `git commit --amend --signoff --no-edit` for the last commit or
`git rebase --signoff main` for the whole branch, then push with `--force-with-lease`.

## Commit messages

Commit messages follow [Conventional Commits](https://www.conventionalcommits.org):

```text
<type>(<optional scope>): <summary in the imperative, at most 72 characters>

<body: what changed and why, wrapped at 72 characters>

Signed-off-by: ...
```

The types in use are `feat`, `fix`, `perf`, `refactor`, `test`, `docs`, `build`, `ci` and `chore`
(dependency updates are `chore(deps)`). The history of a change belongs in the commit message and
the pull request, not in code comments.

## Style guides

### Rust

- The code follows the [Google Rust Style Guide](https://google.github.io/styleguide/rustguide.html).
- `cargo +nightly fmt --all` formats; do not skip it with `#[rustfmt::skip]` for personal taste.
- `Cargo.toml` denies all warnings, every Clippy group and a set of restriction lints. A lint is
  suppressed only where it is wrong for that spot, with `#[expect(lint, reason = "...")]`, never
  with `#[allow]`.
- No floating point, no allocation and no `unsafe` outside the documented `link_section`.
  Configuration-derived values stay compile-time constants.
- Modules are small and have one responsibility; dependencies point from the firmware to
  `fire-effect`, never back.

### Comments

- Items carry rustdoc comments (`///`, `//!`): one summary sentence, a contract sentence where a
  caller needs it, and `# Panics` / `# Errors` / `# Safety` sections where they apply.
- Short technical comments (`//`) are welcome where the code cannot say it itself: an invariant,
  a unit, a fixed-point format, why an algorithm is shaped the way it is.
- No history in comments ("previously", "now", dates, issue numbers), no commented-out code and no
  `TODO`; open an issue instead.

### Licence headers

Every code and configuration file that allows comments starts with an SPDX header, in the comment
syntax of the file:

```rust
// SPDX-FileCopyrightText: 2026 the Blaustein LED contributors
// SPDX-License-Identifier: MIT OR Apache-2.0
```

The year is the year the file was created.

### Tests

- Logic lives in `fire-effect` and has unit tests next to it (`#[cfg(test)] mod tests`).
- Tests use only integer arithmetic and give every assertion a message that says what failed.

## Before-you-push checklist

```bash
cargo +nightly fmt --all --check
```

```bash
cargo clippy --release
```

```bash
cargo clippy --profile release-debug --features debug
```

```bash
cargo clippy-host
```

```bash
cargo test-host
```

```bash
cargo deny check
```

## Releasing

1. Move the `[Unreleased]` entries of `CHANGELOG.md` into a new version section with the date.
2. Bump `version` in `[workspace.package]` of `Cargo.toml` and the version in `README.md`.
3. Merge that change, then tag the merge commit with a signed tag `vX.Y.Z` and publish a GitHub
   Release with the `blaustein-led-elf` artifact of that commit attached.

## Licence

Blaustein LED is licensed under either of the [Apache License, Version 2.0](LICENSE-APACHE) or the
[MIT licence](LICENSE-MIT) at your option. Unless you explicitly state otherwise, any contribution
intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 licence,
shall be dual licensed as above, without any additional terms or conditions.
