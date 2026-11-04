<!--
Thanks for the pull request. Fill in the sections below and work through the checklist as far as
it applies. A pull request that is not ready yet is opened as a Draft.
-->

## What's this about?

<!--
One to three sentences: what this changes and why (the diff shows what).
Link the issue it resolves (`Closes #123`) and related ones (`Refs #456`).
-->

## Type of change

- [ ] Bug fix
- [ ] Feature
- [ ] Change to the look of the flame (the baseline regression test is updated on purpose)
- [ ] Refactor (no visible change)
- [ ] Documentation
- [ ] Build / CI / dependencies
- [ ] Tests

## Affected parts

- [ ] Effect crate (`fire-effect/`)
- [ ] Firmware (`src/`, `build.rs`, `memory.x`)
- [ ] Configuration (`Cargo.toml`, `.cargo/`, lints, formatting, `deny.toml`)
- [ ] Repository (`.github/`, documentation)

## How was this tested?

<!--
Be concrete: which tests were added or run, and whether the image was flashed to a board (which
one) and the effect checked. If something could not be tested, say so explicitly.
-->
-

## Checklist

- [ ] The issue is linked (`Closes #...`).
- [ ] Every commit carries a `Signed-off-by:` trailer (`git commit -s`, see [CONTRIBUTING.md](../CONTRIBUTING.md#developer-certificate-of-origin-dco)).
- [ ] `cargo +nightly fmt --all --check` passes.
- [ ] `cargo clippy --release`, `cargo clippy --profile release-debug --features debug` and `cargo clippy-host` pass.
- [ ] `cargo test-host` passes; new behaviour and fixes in `fire-effect` have tests.
- [ ] `cargo deny check` passes after a dependency change.
- [ ] Comments are doc comments or short technical comments of the current state, with no history; lint suppressions use `#[expect(lint, reason = "...")]`.
- [ ] New code and configuration files start with the SPDX header.
- [ ] A visible change has an entry in `CHANGELOG.md` under `[Unreleased]`.
- [ ] `README.md` moves with the change (parameters, pins, commands).

## Reviewer notes

<!--
What should the reviewer look at closely? Which trade-offs were made on purpose? What is
explicitly not part of this pull request?
-->
