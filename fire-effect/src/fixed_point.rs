// SPDX-FileCopyrightText: 2026 the Blaustein LED contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Fixed-point formats and unit conversions shared by the simulation and the PWM mapping.

/// Fractional bits of a Q16 value.
pub const Q16: u32 = 16;

/// One in Q16.
pub const ONE_Q16: u32 = 1 << Q16;

/// Thousandths in one whole; the unit of depths and gains in the configuration.
pub const PERMILLE: u32 = 1_000;

/// Returns `permille` thousandths of `whole`, truncated.
///
/// Both operands are 16 bits wide, so the product always fits in `u32`.
#[must_use]
pub const fn permille_of(whole: u16, permille: u16) -> u32 {
    whole as u32 * permille as u32 / PERMILLE
}

/// Returns the share of `whole` that one tick of `tick_ms` covers within `span_ms`, truncated.
///
/// Computed in 64 bits, so `whole * tick_ms` never overflows; callers check the result against
/// the range of the type they store it in.
///
/// # Panics
///
/// Panics if `span_ms` is zero.
#[must_use]
pub const fn per_tick(whole: u32, tick_ms: u32, span_ms: u32) -> u64 {
    whole as u64 * tick_ms as u64 / span_ms as u64
}
