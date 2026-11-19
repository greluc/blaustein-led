// SPDX-FileCopyrightText: 2026 the Blaustein LED contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Profiles and integer statistics shared by the tests.

use crate::config::{FireConfig, FireProfile};
use crate::fire_state::FireState;
use crate::follower::{Follower, FollowerConfig, FollowerProfile};
use crate::installation::{Installation, InstallationConfig};

/// The baseline installation with a 15 ms tick.
#[derive(Clone, Copy)]
pub struct Site;

impl Installation for Site {
    const CONFIG: InstallationConfig = InstallationConfig::new();
}

/// The hearth preset.
#[derive(Clone, Copy)]
pub struct Hearth;

impl FireProfile for Hearth {
    const CONFIG: FireConfig = FireConfig::HEARTH;
}

/// The torch preset.
#[derive(Clone, Copy)]
pub struct Torch;

impl FireProfile for Torch {
    const CONFIG: FireConfig = FireConfig::TORCH;
}

/// The candle preset.
#[derive(Clone, Copy)]
pub struct Candle;

impl FireProfile for Candle {
    const CONFIG: FireConfig = FireConfig::CANDLE;
}

/// The ember preset.
#[derive(Clone, Copy)]
pub struct Ember;

impl FireProfile for Ember {
    const CONFIG: FireConfig = FireConfig::EMBER;
}

/// The ambient preset.
#[derive(Clone, Copy)]
pub struct Ambient;

impl FireProfile for Ambient {
    const CONFIG: FireConfig = FireConfig::AMBIENT;
}

/// The hearth glow preset.
#[derive(Clone, Copy)]
pub struct HearthGlow;

impl FollowerProfile for HearthGlow {
    const CONFIG: FollowerConfig = FollowerConfig::HEARTH_GLOW;
}

/// Returns the raw outputs of `ticks` updates of a channel of `P` seeded with `seed`.
pub fn record<P: FireProfile>(seed: u32, ticks: usize) -> Vec<u16> {
    let mut state = FireState::<Site, P>::new(seed);
    (0..ticks).map(|_| state.update().get()).collect()
}

/// Returns the raw outputs of a hearth channel seeded with `seed` and of a follower of `F` fed by
/// it, over `ticks` updates.
pub fn follow<F: FollowerProfile>(seed: u32, ticks: usize) -> (Vec<u16>, Vec<u16>) {
    let mut source = FireState::<Site, Hearth>::new(seed);
    let mut follower = Follower::<Site, F>::new();
    (0..ticks)
        .map(|_| {
            let flame = source.update();
            (flame.get(), follower.update(flame).get())
        })
        .unzip()
}

/// Returns an order-sensitive FNV-1a fingerprint of `outputs`.
pub fn fingerprint(outputs: &[u16]) -> u64 {
    const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
    const FNV_PRIME: u64 = 0x0100_0000_01b3;
    outputs.iter().fold(FNV_OFFSET_BASIS, |hash, &output| {
        (hash ^ u64::from(output)).wrapping_mul(FNV_PRIME)
    })
}

/// Returns the mean of `series`, truncated.
pub fn mean(series: &[u16]) -> i64 {
    let sum: i64 = series.iter().map(|&value| i64::from(value)).sum();
    sum / i64::try_from(series.len()).expect("series length fits in i64")
}

/// Returns the mean absolute change from one value of `series` to the next, in 1/1000 steps.
pub fn mean_step_millis(series: &[u16]) -> u64 {
    let total: u64 = series
        .windows(2)
        .map(|pair| u64::from(pair[0].abs_diff(pair[1])))
        .sum();
    total * 1_000 / u64::try_from(series.len() - 1).expect("series length fits in u64")
}

/// Returns the Pearson correlation of two equally long series in permille, truncated.
///
/// Centred on their integer means: the deviations stay within `u16`, their products within
/// `u32`, and the sums over a few million values within `i64`; the product of the two variances
/// needs `u128`.
pub fn correlation_permille(first: &[u16], second: &[u16]) -> i64 {
    assert_eq!(
        first.len(),
        second.len(),
        "correlated series must be equally long"
    );
    let (first_mean, second_mean) = (mean(first), mean(second));
    let (mut covariance, mut first_variance, mut second_variance) = (0_i64, 0_i64, 0_i64);
    for (&first_value, &second_value) in first.iter().zip(second) {
        let first_deviation = i64::from(first_value) - first_mean;
        let second_deviation = i64::from(second_value) - second_mean;
        covariance += first_deviation * second_deviation;
        first_variance += first_deviation * first_deviation;
        second_variance += second_deviation * second_deviation;
    }
    let spread = (u128::try_from(first_variance).expect("a variance is non-negative")
        * u128::try_from(second_variance).expect("a variance is non-negative"))
    .isqrt();
    let spread = i128::try_from(spread).expect("the spread fits in i128");
    assert!(spread > 0, "a constant series has no correlation");
    i64::try_from(i128::from(covariance) * 1_000 / spread).expect("a correlation is within 1000")
}

/// Returns the autocorrelation of `series` at `lag` ticks in permille, truncated.
pub fn autocorrelation_permille(series: &[u16], lag: usize) -> i64 {
    correlation_permille(&series[..series.len() - lag], &series[lag..])
}
