//! A small in-place radix-2 FFT for power-of-two sizes.
//!
//! The analysis only needs forward transforms of a few fixed power-of-two
//! sizes; this keeps the WASM module far smaller than a general FFT library
//! (whose planner pulls in many algorithms).

use num_complex::Complex32;

/// Precomputed twiddle factors and bit-reversal table for one size.
#[derive(Debug, Clone)]
pub struct Fft {
    size: usize,
    /// `e^{-2πik/size}` for `k < size / 2`.
    twiddles: Vec<Complex32>,
    /// Index permutation for the bit-reversed input order.
    reversed: Vec<u32>,
}

impl Fft {
    /// # Panics
    ///
    /// If `size` is not a power of two (or is less than 2).
    pub fn new(size: usize) -> Self {
        assert!(
            size >= 2 && size.is_power_of_two(),
            "FFT size must be a power of two"
        );
        let bits = size.trailing_zeros();
        let twiddles = (0..size / 2)
            .map(|k| {
                // Computed in f64 so large sizes keep full f32 accuracy.
                let angle = -std::f64::consts::TAU * k as f64 / size as f64;
                Complex32::new(angle.cos() as f32, angle.sin() as f32)
            })
            .collect();
        let reversed = (0..size as u32)
            .map(|i| i.reverse_bits() >> (32 - bits))
            .collect();
        Self {
            size,
            twiddles,
            reversed,
        }
    }

    pub fn size(&self) -> usize {
        self.size
    }

    /// Forward transform in place: `X[k] = Σ x[n]·e^{-2πikn/N}`.
    ///
    /// # Panics
    ///
    /// If `data.len()` differs from the planned size.
    pub fn forward(&self, data: &mut [Complex32]) {
        assert_eq!(data.len(), self.size, "wrong FFT input length");
        for (i, &j) in self.reversed.iter().enumerate() {
            let j = j as usize;
            if i < j {
                data.swap(i, j);
            }
        }
        let mut half = 1;
        while half < self.size {
            let stride = self.size / (2 * half);
            for start in (0..self.size).step_by(2 * half) {
                for k in 0..half {
                    let w = self.twiddles[k * stride];
                    let a = data[start + k];
                    let b = data[start + k + half] * w;
                    data[start + k] = a + b;
                    data[start + k + half] = a - b;
                }
            }
            half *= 2;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Deterministic pseudo-random values in -1..1.
    fn values(n: usize, mut seed: u32) -> Vec<Complex32> {
        let mut next = move || {
            seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            (seed >> 8) as f32 / (1u32 << 23) as f32 - 1.0
        };
        (0..n).map(|_| Complex32::new(next(), next())).collect()
    }

    fn naive_dft(x: &[Complex32]) -> Vec<Complex32> {
        let n = x.len();
        (0..n)
            .map(|k| {
                let (mut re, mut im) = (0.0f64, 0.0f64);
                for (t, v) in x.iter().enumerate() {
                    let angle = -std::f64::consts::TAU * (k * t % n) as f64 / n as f64;
                    let (s, c) = angle.sin_cos();
                    re += v.re as f64 * c - v.im as f64 * s;
                    im += v.re as f64 * s + v.im as f64 * c;
                }
                Complex32::new(re as f32, im as f32)
            })
            .collect()
    }

    #[test]
    fn matches_a_naive_dft() {
        for (n, seed) in [(2, 1), (8, 2), (64, 3), (1024, 4)] {
            let x = values(n, seed);
            let expected = naive_dft(&x);
            let mut got = x.clone();
            Fft::new(n).forward(&mut got);
            let scale = (n as f32).sqrt();
            for (k, (g, e)) in got.iter().zip(&expected).enumerate() {
                assert!((g - e).norm() <= 1e-4 * scale, "n={n} bin {k}: {g} vs {e}");
            }
        }
    }

    #[test]
    fn sine_lands_in_its_bin_at_large_sizes() {
        let n = 65_536;
        let bin = 1_234;
        let mut x: Vec<Complex32> = (0..n)
            .map(|i| {
                let a = std::f64::consts::TAU * (bin * i) as f64 / n as f64;
                Complex32::new(a.cos() as f32, 0.0)
            })
            .collect();
        Fft::new(n).forward(&mut x);
        let peak = x[..n / 2]
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.norm().total_cmp(&b.1.norm()))
            .unwrap();
        assert_eq!(peak.0, bin);
        assert!((peak.1.norm() - n as f32 / 2.0).abs() < 1e-3 * n as f32);
    }

    #[test]
    #[should_panic(expected = "power of two")]
    fn rejects_other_sizes() {
        Fft::new(1000);
    }
}
