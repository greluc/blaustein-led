# Security Policy

Thank you for looking at the security of Blaustein LED. This document explains how to report a
vulnerability, which versions receive fixes and what you can expect in return.

## Reporting a vulnerability

**Please do not open a public issue or pull request for anything you believe is
security-sensitive.**

The preferred channel is GitHub's **private vulnerability reporting**:

> **[Report a vulnerability](https://github.com/greluc/blaustein-led/security/advisories/new)**

A good report includes:

- A description of the issue and its impact.
- The affected version (release tag, or commit SHA for a build from source) and build profile.
- Reproduction steps or a proof of concept.
- Your assessment of the severity (a CVSS vector is welcome but not required).
- Whether you intend to publish your own write-up, and when.

If you cannot use GitHub's form, e-mail [lucas.greuloch@gmail.com](mailto:lucas.greuloch@gmail.com)
with the subject *"Security report: Blaustein LED"*.

## What to expect

Blaustein LED is a community project without a commercial SLA, but we aim for the following:

| Step | Target |
| --- | --- |
| Acknowledge receipt | within 7 days |
| Initial triage and severity assessment | within 14 days |
| Fix in `main` for High / Critical issues | within 90 days |
| Coordinated disclosure after a fixed release | within 14 days of the release |

If we cannot meet a target, we say why and propose a new date. We credit you in the advisory and
the [changelog](../CHANGELOG.md) unless you ask us not to.

## Supported versions

Only the latest release and `main` receive fixes; older releases get no backports.

| Version | Supported |
| --- | --- |
| Latest release and `main` | :white_check_mark: |
| Older releases | :x: (please upgrade) |

## Scope

In scope:

- The source code in this repository: the firmware (`src/`, `build.rs`, `memory.x`) and the
  `fire-effect` crate.
- Released firmware images.
- The GitHub Actions workflows under `.github/workflows/` and the tools they download.

Interesting classes of issue for this firmware:

- **Robustness.** A configuration that passes the compile-time validation but makes the firmware
  panic, hang or stop feeding the watchdog.
- **Memory safety.** Any way to reach undefined behaviour from safe code, or a flaw in the single
  `unsafe` attribute (`link_section` of the picotool metadata).
- **Supply chain.** Anything affecting the integrity of the dependencies (`Cargo.lock`,
  `deny.toml`) or of the CI pipeline, such as an unpinned action or download.
- **Secrets.** Any credential appearing in the repository, a workflow log or a release.

Typically out of scope:

- Vulnerabilities in third-party dependencies without a fix path in this project; please report
  those upstream first.
- Attacks that need physical access to the board, such as reflashing it over BOOTSEL or SWD.
- Reports from automated scanners without a demonstrated, reproducible impact.

## Coordinated disclosure

We follow coordinated disclosure with a default window of **90 days from our acknowledgement**, or
sooner once a fixed release is public. When the advisory is published we release the fix, publish
the GitHub Security Advisory with credit, and add a `### Security` entry to the changelog.

## Safe harbour

We will not pursue or support legal action against researchers who make a good-faith effort to
follow this policy, use only their own hardware and data, and give us reasonable time to fix the
issue before any public disclosure.
