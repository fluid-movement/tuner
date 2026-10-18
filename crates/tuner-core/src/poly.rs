//! Polyphonic string pitch estimation for a strum: given a recording of all
//! strings and the instrument's tuning targets, estimate each string's
//! fundamental. Research prototype; see the strum research issue findings.

use crate::fft::Fft;
use crate::tuning::{Instrument, StringTarget, cents_between};
use num_complex::{Complex32, Complex64};

/// Blackman–Harris (4-term) coefficients: `w[n] = Σ c_m cos(2π m n / (N−1))`.
const BH: [f64; 4] = [0.358_75, -0.488_29, 0.141_28, -0.011_68];

/// Estimated pitch of one string.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StringEstimate {
    /// Estimated fundamental in Hz.
    pub frequency: f32,
    /// Offset from the target in cents.
    pub cents: f32,
    /// Level of the strongest partial used, in dB relative to the loudest
    /// partial of any string (≤ 0).
    pub level_db: f32,
    /// Inharmonicity coefficient fitted from the string's partials, or the
    /// configured default when it could not be fitted.
    pub inharmonicity: f32,
    /// Set by [`estimate_strum`] for strings with clean partials: the second
    /// largest, over those partials not shared with another string's
    /// estimated partials, of how far the peak stands above the median
    /// spectrum around it (outside the window's main lobe, out to a
    /// semitone), in dB. A sounding string makes real peaks there; a silent
    /// one only has other strings' peaks, which are excluded.
    pub prominence_db: Option<f32>,
    /// Set by [`estimate_strum`]: whether the string was played, judged from
    /// `prominence_db` against [`PRESENT_PROMINENCE_DB`]. `None` for strings
    /// without clean partials (guitar B3 and E4), whose partials all coincide
    /// with other strings' and so can't be judged on their own.
    pub present: Option<bool>,
}

/// A string with clean partials counts as played when its prominence is at
/// least this (played strings measure ≥ 39 dB, muted ones ≤ ~10 dB on
/// synthetic strums; see `examples/presence_eval.rs`).
pub const PRESENT_PROMINENCE_DB: f32 = 20.0;

/// Spectrum of a Blackman–Harris windowed, zero-padded frame.
pub struct Spectrum {
    /// Complex bins up to Nyquist.
    pub bins: Vec<Complex32>,
    /// Log magnitude (dB) per bin up to Nyquist.
    pub db: Vec<f32>,
    /// Hz per bin.
    pub bin_hz: f32,
    /// Length of the analysed frame in samples.
    pub frame_len: usize,
    pub sample_rate: f32,
}

impl Spectrum {
    pub fn new(frame: &[f32], sample_rate: f32, zero_pad: usize) -> Self {
        let n = frame.len();
        let size = (n * zero_pad).next_power_of_two();
        let mut buf: Vec<Complex32> = vec![Complex32::new(0.0, 0.0); size];
        for (i, (&x, b)) in frame.iter().zip(&mut buf).enumerate() {
            let t = std::f64::consts::TAU * i as f64 / (n - 1) as f64;
            let w: f64 = BH
                .iter()
                .enumerate()
                .map(|(m, c)| c * (m as f64 * t).cos())
                .sum();
            *b = Complex32::new(x * w as f32, 0.0);
        }
        Fft::new(size).forward(&mut buf);
        buf.truncate(size / 2);
        let db = buf
            .iter()
            .map(|c| 10.0 * (c.norm_sqr() + 1e-20).log10())
            .collect();
        Self {
            bins: buf,
            db,
            bin_hz: sample_rate / size as f32,
            frame_len: n,
            sample_rate,
        }
    }

    /// Linearly interpolated dB at `hz`.
    pub fn db_at(&self, hz: f32) -> f32 {
        let x = hz / self.bin_hz;
        let i = x.floor() as usize;
        if i + 1 >= self.db.len() {
            return f32::MIN;
        }
        let t = x - i as f32;
        self.db[i] * (1.0 - t) + self.db[i + 1] * t
    }

    /// The highest local maximum within `±cents` of `hz`, refined by fitting
    /// a parabola to the dB values (exact for a Gaussian-shaped peak, close
    /// for Blackman–Harris). Returns (frequency, dB).
    pub fn peak_near(&self, hz: f32, cents: f32) -> Option<(f32, f32)> {
        let lo = ((hz * 2f32.powf(-cents / 1200.0)) / self.bin_hz)
            .floor()
            .max(1.0) as usize;
        let hi = ((hz * 2f32.powf(cents / 1200.0)) / self.bin_hz).ceil() as usize;
        let hi = hi.min(self.db.len() - 2);
        let db = &self.db;
        (lo..=hi)
            .filter(|&i| db[i] >= db[i - 1] && db[i] > db[i + 1])
            .max_by(|&a, &b| db[a].total_cmp(&db[b]))
            .map(|i| {
                let (l, m, r) = (db[i - 1], db[i], db[i + 1]);
                let denom = l - 2.0 * m + r;
                let shift = if denom < 0.0 {
                    0.5 * (l - r) / denom
                } else {
                    0.0
                };
                ((i as f32 + shift) * self.bin_hz, m - 0.25 * (l - r) * shift)
            })
    }
}

/// Settings shared by the estimators.
#[derive(Debug, Clone, Copy)]
pub struct PolyConfig {
    pub sample_rate: f32,
    /// Search range around each target, in cents.
    pub search_cents: f32,
    /// Partials considered per string.
    pub partials: u32,
    /// Inharmonicity assumed when predicting partial positions.
    pub inharmonicity: f32,
    /// Two partials of different strings closer than this (in cents, at
    /// their target positions) are treated as colliding.
    pub collision_cents: f32,
}

impl PolyConfig {
    pub fn new(sample_rate: f32, instrument: Instrument) -> Self {
        Self {
            sample_rate,
            search_cents: 60.0,
            partials: 8,
            inharmonicity: match instrument {
                Instrument::Guitar => 8e-5,
                Instrument::Ukulele => 2e-5,
            },
            collision_cents: 35.0,
        }
    }
}

fn partial(f: f32, n: u32, b: f32) -> f32 {
    let n = n as f32;
    n * f * (1.0 + b * n * n).sqrt()
}

/// For each string, which of its partials (1-based index − 1) do not collide
/// with any partial of another string at the target positions.
pub fn clean_partials(targets: &[StringTarget], config: &PolyConfig) -> Vec<Vec<bool>> {
    let b = config.inharmonicity;
    targets
        .iter()
        .enumerate()
        .map(|(i, t)| {
            (1..=config.partials)
                .map(|n| {
                    let f = partial(t.frequency, n, b);
                    !targets.iter().enumerate().any(|(j, o)| {
                        j != i
                            && (1..=config.partials * 4).any(|m| {
                                cents_between(f, partial(o.frequency, m, b)).abs()
                                    < config.collision_cents
                            })
                    })
                })
                .collect()
        })
        .collect()
}

/// Approach A: spectral comb matching against the tuning targets.
///
/// For each string, scan candidate fundamentals within the search range and
/// score each by the summed dB at its predicted partials, using only partials
/// that do not collide with other strings (all of them if none are clean).
/// The best candidate is refined from the interpolated peaks of the clean
/// partials, each divided by its partial number, weighted by level.
pub fn estimate_comb(
    frame: &[f32],
    instrument: Instrument,
    config: &PolyConfig,
) -> Vec<Option<StringEstimate>> {
    let spectrum = Spectrum::new(frame, config.sample_rate, 2);
    let targets = instrument.strings();
    comb(&spectrum, targets, &clean_partials(targets, config), config)
}

fn comb(
    spectrum: &Spectrum,
    targets: &[StringTarget],
    clean: &[Vec<bool>],
    config: &PolyConfig,
) -> Vec<Option<StringEstimate>> {
    let b = config.inharmonicity;
    let peak = spectrum.db.iter().copied().fold(f32::MIN, f32::max);
    let floor = peak - 80.0;

    targets
        .iter()
        .zip(clean)
        .map(|(t, clean)| {
            let any_clean = clean.iter().any(|&c| c);
            let usable = |n: u32| !any_clean || clean[n as usize - 1];

            let mut best = (f32::MIN, t.frequency);
            let steps = (2.0 * config.search_cents) as i32 * 2;
            for k in 0..=steps {
                let c = -config.search_cents + k as f32 * 0.5;
                let f = t.frequency * 2f32.powf(c / 1200.0);
                let score: f32 = (1..=config.partials)
                    .filter(|&n| usable(n))
                    .map(|n| spectrum.db_at(partial(f, n, b)).max(floor) / n as f32)
                    .sum();
                if score > best.0 {
                    best = (score, f);
                }
            }

            let f0 = best.1;
            // (partial number, peak Hz, weight)
            let mut peaks: Vec<(u32, f32, f32)> = Vec::new();
            let mut top = f32::MIN;
            for n in (1..=config.partials).filter(|&n| usable(n)) {
                if let Some((hz, db)) = spectrum.peak_near(partial(f0, n, b), 12.0) {
                    peaks.push((n, hz, 10f32.powf((db - peak) / 20.0) / n as f32));
                    top = top.max(db);
                }
            }
            let (frequency, inharmonicity) = if any_clean {
                fit_partials(&peaks, b)?
            } else {
                // Shared partials only: a fitted B would be meaningless.
                let (s, w) = peaks.iter().fold((0.0, 0.0), |(s, w), &(n, hz, wt)| {
                    (
                        s + wt * hz / (n as f32 * (1.0 + b * (n * n) as f32).sqrt()),
                        w + wt,
                    )
                });
                ((w > 0.0).then(|| s / w)?, b)
            };
            Some(StringEstimate {
                frequency,
                cents: cents_between(frequency, t.frequency),
                level_db: top - peak,
                inharmonicity,
                prominence_db: None,
                present: None,
            })
        })
        .collect()
}

/// Fits `f0` and `B` to partial peaks `p_n = n·f0·sqrt(1 + B·n²)` by
/// weighted least squares on `ln(p_n / n) ≈ ln f0 + (B/2)·n²`. With fewer
/// than two distinct partials, or an implausible fit, `B` stays at
/// `default_b`.
fn fit_partials(peaks: &[(u32, f32, f32)], default_b: f32) -> Option<(f32, f32)> {
    let weight: f64 = peaks.iter().map(|p| p.2 as f64).sum();
    if weight <= 0.0 {
        return None;
    }
    let with_b = |b: f64| -> f64 {
        peaks
            .iter()
            .map(|&(n, hz, w)| {
                w as f64 * (hz as f64 / (n as f64 * (1.0 + b * (n * n) as f64).sqrt())).ln()
            })
            .sum::<f64>()
            / weight
    };
    let distinct = peaks.iter().any(|p| p.0 != peaks[0].0);
    if distinct {
        let (mut sw, mut sx, mut sy, mut sxx, mut sxy) = (0.0f64, 0.0, 0.0, 0.0, 0.0);
        for &(n, hz, w) in peaks {
            let (w, x, y) = (w as f64, (n * n) as f64, (hz as f64 / n as f64).ln());
            sw += w;
            sx += w * x;
            sy += w * y;
            sxx += w * x * x;
            sxy += w * x * y;
        }
        let det = sw * sxx - sx * sx;
        if det.abs() > 1e-12 {
            let b = 2.0 * (sw * sxy - sx * sy) / det;
            if (0.0..=5e-4).contains(&b) {
                return Some((with_b(b).exp() as f32, b as f32));
            }
        }
    }
    Some((with_b(default_b as f64).exp() as f32, default_b))
}

/// Solves the complex system `a·x = b` (row-major, n×n) in place by Gaussian
/// elimination with partial pivoting; the solution replaces `b`. Returns
/// `None` if the system is singular.
fn solve(a: &mut [Complex64], b: &mut [Complex64], n: usize) -> Option<()> {
    for col in 0..n {
        let pivot = (col..n).max_by(|&i, &j| {
            a[i * n + col]
                .norm_sqr()
                .total_cmp(&a[j * n + col].norm_sqr())
        })?;
        if a[pivot * n + col].norm_sqr() < 1e-24 {
            return None;
        }
        for k in 0..n {
            a.swap(col * n + k, pivot * n + k);
        }
        b.swap(col, pivot);
        for row in col + 1..n {
            let f = a[row * n + col] / a[col * n + col];
            for k in col..n {
                let v = a[col * n + k];
                a[row * n + k] -= f * v;
            }
            let v = b[col];
            b[row] -= f * v;
        }
    }
    for col in (0..n).rev() {
        let mut v = b[col];
        for k in col + 1..n {
            v -= a[col * n + k] * b[k];
        }
        b[col] = v / a[col * n + col];
    }
    Some(())
}

/// Spectrum, at bins `lo..=hi` of `spectrum`, of a unit complex exponential
/// at `hz` with damping `d` (per second) under the Blackman–Harris window.
/// Exact: the window is a sum of cosines, so each term is a geometric series
/// `Σ e^{sn} = (1 − e^{sN}) / (1 − e^{s})`. Successive bins differ by a
/// constant phase step, so the terms follow a recurrence instead of needing
/// complex exponentials per bin.
fn kernel_row(spectrum: &Spectrum, lo: usize, hi: usize, hz: f64, d: f64) -> Vec<Complex64> {
    let fs = spectrum.sample_rate as f64;
    let n = spectrum.frame_len as f64;
    let bin_w = std::f64::consts::TAU * spectrum.bin_hz as f64 / fs;
    let theta = std::f64::consts::TAU / (n - 1.0);
    let one = Complex64::new(1.0, 0.0);
    let step = Complex64::from_polar(1.0, -bin_w);
    let step_n = Complex64::from_polar(1.0, -bin_w * n);

    // For each window term m and sign: e^{s} and e^{sN} at bin `lo`.
    let mut terms: Vec<(f64, Complex64, Complex64)> = Vec::with_capacity(7);
    let base = Complex64::new(-d / fs, std::f64::consts::TAU * hz / fs - bin_w * lo as f64);
    for (m, &c) in BH.iter().enumerate() {
        let shifts: &[f64] = if m == 0 { &[0.0] } else { &[1.0, -1.0] };
        let weight = if m == 0 { c } else { c / 2.0 };
        for &sign in shifts {
            let s = base + Complex64::new(0.0, sign * theta * m as f64);
            terms.push((weight, s.exp(), (s * n).exp()));
        }
    }
    (lo..=hi)
        .map(|_| {
            let mut v = Complex64::new(0.0, 0.0);
            for (w, e1, en) in &mut terms {
                let denom = one - *e1;
                v += if denom.norm_sqr() < 1e-24 {
                    Complex64::new(n * *w, 0.0)
                } else {
                    (one - *en) / denom * *w
                };
                *e1 *= step;
                *en *= step_n;
            }
            v
        })
        .collect()
}

/// A least-squares fit of a band of spectrum bins with some components at
/// fixed frequencies; the fixed part of the normal equations is computed once
/// so trying positions for one extra component is cheap.
struct BandFit {
    x: Vec<Complex64>,
    energy: f64,
    /// Residual energy with the known components alone.
    base_residual: f64,
    known: Vec<Vec<Complex64>>,
    gram: Vec<Complex64>,
    rhs: Vec<Complex64>,
}

impl BandFit {
    fn new(spectrum: &Spectrum, lo: usize, hi: usize, known_hz: &[f64], d: f64) -> Self {
        let x: Vec<Complex64> = spectrum.bins[lo..=hi]
            .iter()
            .map(|c| Complex64::new(c.re as f64, c.im as f64))
            .collect();
        let energy = x.iter().map(|c| c.norm_sqr()).sum();
        let known: Vec<Vec<Complex64>> = known_hz
            .iter()
            .map(|&f| kernel_row(spectrum, lo, hi, f, d))
            .collect();
        let k = known.len();
        let mut gram = vec![Complex64::new(0.0, 0.0); k * k];
        let mut rhs = vec![Complex64::new(0.0, 0.0); k];
        for p in 0..k {
            rhs[p] = dot(&known[p], &x);
            for q in 0..k {
                gram[p * k + q] = dot(&known[p], &known[q]);
            }
        }
        let mut fit = Self {
            x,
            energy,
            base_residual: energy,
            known,
            gram,
            rhs,
        };
        if k > 0 {
            let mut g = fit.gram.clone();
            regularize(&mut g, k);
            let mut amps = fit.rhs.clone();
            if solve(&mut g, &mut amps, k).is_some() {
                let explained: f64 = amps
                    .iter()
                    .zip(&fit.rhs)
                    .map(|(a, r)| (a.conj() * r).re)
                    .sum();
                fit.base_residual = energy - explained;
            }
        }
        fit
    }

    /// Residual energy and the amplitude of `extra` when it is added.
    fn with(&self, extra: &[Complex64]) -> Option<(f64, Complex64)> {
        let k = self.known.len();
        let size = k + 1;
        let mut gram = vec![Complex64::new(0.0, 0.0); size * size];
        for p in 0..k {
            gram[p * size..p * size + k].copy_from_slice(&self.gram[p * k..p * k + k]);
            let c = dot(&self.known[p], extra);
            gram[p * size + k] = c;
            gram[k * size + p] = c.conj();
        }
        gram[k * size + k] = dot(extra, extra);
        let mut rhs = self.rhs.clone();
        rhs.push(dot(extra, &self.x));
        let mut amps = rhs.clone();
        regularize(&mut gram, size);
        solve(&mut gram, &mut amps, size)?;
        let explained: f64 = amps.iter().zip(&rhs).map(|(a, r)| (a.conj() * r).re).sum();
        Some((self.energy - explained, amps[k]))
    }
}

/// The value at which the cumulative weight reaches half, so that one bad
/// candidate among three cannot move the result.
fn weighted_median(values: &mut [(f64, f64)]) -> Option<f64> {
    let total: f64 = values.iter().map(|v| v.1).sum();
    if total <= 0.0 {
        return None;
    }
    values.sort_by(|a, b| a.0.total_cmp(&b.0));
    let mut acc = 0.0;
    values
        .iter()
        .find(|v| {
            acc += v.1;
            acc >= total / 2.0
        })
        .map(|v| v.0)
}

/// Adds a small ridge to the diagonal of a Gram matrix. Without it, two
/// nearly coincident components get huge opposite-phase amplitudes that fit
/// noise or model error instead of the signal.
fn regularize(gram: &mut [Complex64], n: usize) {
    let mean = (0..n).map(|i| gram[i * n + i].re).sum::<f64>() / n as f64;
    for i in 0..n {
        gram[i * n + i] += Complex64::new(1e-3 * mean, 0.0);
    }
}

/// `Σ conj(a)·b`.
fn dot(a: &[Complex64], b: &[Complex64]) -> Complex64 {
    a.iter().zip(b).map(|(x, y)| x.conj() * y).sum()
}

/// Approach B: the comb estimate for strings with clean partials, then each
/// string without clean partials is resolved band by band. In a band around
/// one of its partials, the partials of already-estimated strings are known
/// components at positions predicted from their fitted fundamental and
/// inharmonicity; the unknown partial's frequency is the one giving the
/// smallest least-squares residual of the exact windowed-spectrum model,
/// found coarse to fine. Damping is searched over a few values.
pub fn estimate_resolved(
    frame: &[f32],
    instrument: Instrument,
    config: &PolyConfig,
) -> Vec<Option<StringEstimate>> {
    resolve(
        &Spectrum::new(frame, config.sample_rate, 2),
        instrument,
        config,
    )
}

fn resolve(
    spectrum: &Spectrum,
    instrument: Instrument,
    config: &PolyConfig,
) -> Vec<Option<StringEstimate>> {
    let targets = instrument.strings();
    let clean = clean_partials(targets, config);
    let mut estimates = comb(spectrum, targets, &clean, config);
    let b = config.inharmonicity as f64;
    let peak = spectrum.db.iter().copied().fold(f32::MIN, f32::max);

    // Strings without clean partials, lowest first so later ones can treat
    // earlier ones as known.
    let mut order: Vec<usize> = (0..targets.len())
        .filter(|&i| clean[i].iter().all(|&c| !c))
        .collect();
    order.sort_by(|&x, &y| targets[x].frequency.total_cmp(&targets[y].frequency));
    let mut known: Vec<usize> = (0..targets.len()).filter(|i| !order.contains(i)).collect();

    for &u in &order {
        let target = targets[u].frequency as f64;
        // (fundamental implied by one partial, weight)
        let mut candidates: Vec<(f64, f64)> = Vec::new();
        let mut top = f32::MIN;
        for n in 1..=3u32 {
            let scale = n as f64 * (1.0 + b * (n * n) as f64).sqrt();
            let centre = target * scale;
            let range = 2f64.powf((config.search_cents as f64 + 25.0) / 1200.0);
            let (lo_hz, hi_hz) = (centre / range, centre * range);
            let lo = (lo_hz / spectrum.bin_hz as f64).floor() as usize;
            let hi =
                ((hi_hz / spectrum.bin_hz as f64).ceil() as usize).min(spectrum.bins.len() - 1);

            // Known partials in (or just outside) the band.
            let margin = 2f64.powf(15.0 / 1200.0);
            let known_hz: Vec<f64> = known
                .iter()
                .filter_map(|&s| estimates[s])
                .flat_map(|e| {
                    (1..=config.partials * 4)
                        .map(move |m| partial(e.frequency, m, e.inharmonicity) as f64)
                })
                .filter(|&f| f > lo_hz / margin && f < hi_hz * margin)
                .collect();

            let at = |cents: f64| centre * 2f64.powf(cents / 1200.0);
            // (residual, frequency, energy explained beyond the known partials)
            let mut best = (f64::MAX, centre, 0.0f64);
            for &d in &[0.0, 1.5, 4.0] {
                let fit = BandFit::new(spectrum, lo, hi, &known_hz, d);
                let try_at = |cents: f64| fit.with(&kernel_row(spectrum, lo, hi, at(cents), d));
                // Coarse 2-cent grid, then 0.25-cent steps around each of the
                // three lowest local minima (the residual can have several).
                let search = config.search_cents as f64;
                let coarse: Vec<(f64, f64)> = (0..=(search as i32))
                    .map(|k| -search + 2.0 * k as f64)
                    .filter_map(|c| try_at(c).map(|(res, _)| (res, c)))
                    .collect();
                let mut minima: Vec<(f64, f64)> = (0..coarse.len())
                    .filter(|&i| {
                        (i == 0 || coarse[i].0 <= coarse[i - 1].0)
                            && (i + 1 == coarse.len() || coarse[i].0 <= coarse[i + 1].0)
                    })
                    .map(|i| coarse[i])
                    .collect();
                minima.sort_by(|a, b| a.0.total_cmp(&b.0));
                for &(_, centre_cents) in minima.iter().take(3) {
                    for k in -8..=8 {
                        let c = centre_cents + k as f64 * 0.25;
                        if let Some((res, _)) = try_at(c)
                            && res < best.0
                        {
                            best = (res, at(c), (fit.base_residual - res).max(0.0));
                        }
                    }
                }
            }
            if best.0 < f64::MAX {
                candidates.push((best.1 / scale, best.2));
                top = top.max(10.0 * (best.2 as f32 + 1e-20).log10());
            }
        }
        if let Some(f) = weighted_median(&mut candidates) {
            let frequency = f as f32;
            estimates[u] = Some(StringEstimate {
                frequency,
                cents: cents_between(frequency, targets[u].frequency),
                level_db: top - peak,
                inharmonicity: config.inharmonicity,
                prominence_db: None,
                present: None,
            });
        }
        known.push(u);
    }
    estimates
}

/// [`estimate_resolved`] on `frame` (the analysis window after a strum),
/// plus [`StringEstimate::prominence_db`] and [`StringEstimate::present`]:
/// whether each string with clean partials was actually played.
pub fn estimate_strum(
    frame: &[f32],
    instrument: Instrument,
    config: &PolyConfig,
) -> Vec<Option<StringEstimate>> {
    let spectrum = Spectrum::new(frame, config.sample_rate, 2);
    let mut estimates = resolve(&spectrum, instrument, config);
    let clean = clean_partials(instrument.strings(), config);
    // Every estimated partial of every string, to tell which peaks another
    // string already explains.
    let all_partials: Vec<(usize, f32)> = estimates
        .iter()
        .enumerate()
        .filter_map(|(i, e)| e.map(|e| (i, e)))
        .flat_map(|(i, e)| {
            (1..=config.partials * 4).map(move |m| (i, partial(e.frequency, m, e.inharmonicity)))
        })
        .collect();
    // Within the Blackman–Harris main lobe (±4 bins of the analysed length)
    // two partials are one peak.
    let lobe_hz = 4.0 * config.sample_rate / frame.len() as f32;
    let lobe_bins = (lobe_hz / spectrum.bin_hz).ceil() as usize + 1;
    let peak_db = |hz: f32| {
        let lo = (hz * 2f32.powf(-15.0 / 1200.0) / spectrum.bin_hz).floor() as usize;
        let hi = ((hz * 2f32.powf(15.0 / 1200.0) / spectrum.bin_hz).ceil() as usize)
            .min(spectrum.db.len() - 1);
        (lo..=hi).map(|i| spectrum.db[i]).fold(f32::MIN, f32::max)
    };
    for (index, (est, clean)) in estimates.iter_mut().zip(&clean).enumerate() {
        let Some(e) = est else { continue };
        if !clean.iter().any(|&c| c) {
            continue;
        }
        let explained_elsewhere = |hz: f32| {
            all_partials
                .iter()
                .any(|&(j, other)| j != index && (other - hz).abs() <= lobe_hz)
        };
        let mut prominences: Vec<f32> = (1..=config.partials)
            .filter(|&n| clean[n as usize - 1])
            .map(|n| partial(e.frequency, n, e.inharmonicity))
            .filter(|&hz| !explained_elsewhere(hz))
            .map(|hz| {
                let centre = (hz / spectrum.bin_hz).round() as usize;
                // Background: beyond the window's main lobe, out to a
                // semitone or two lobes, whichever is wider.
                let reach = ((hz * (2f32.powf(100.0 / 1200.0) - 1.0)) / spectrum.bin_hz)
                    .ceil()
                    .max(2.0 * lobe_bins as f32) as usize;
                let mut around: Vec<f32> = (centre.saturating_sub(reach)..=centre + reach)
                    .filter(|&i| i.abs_diff(centre) > lobe_bins && i < spectrum.db.len())
                    .map(|i| spectrum.db[i])
                    .collect();
                around.sort_by(f32::total_cmp);
                peak_db(hz) - around.get(around.len() / 2).copied().unwrap_or(f32::MAX)
            })
            .collect();
        // The second strongest: pluck-position nulls can silence one partial
        // of a played string, while a muted string's estimate rarely lines up
        // with two foreign peaks.
        prominences.sort_by(|a, b| b.total_cmp(a));
        let prominence = prominences
            .get(1)
            .or(prominences.first())
            .copied()
            .unwrap_or(0.0);
        e.prominence_db = Some(prominence);
        e.present = Some(prominence >= PRESENT_PROMINENCE_DB);
    }
    estimates
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::TAU;

    #[test]
    fn guitar_collisions_are_found() {
        let config = PolyConfig::new(48_000.0, Instrument::Guitar);
        let clean = clean_partials(Instrument::Guitar.strings(), &config);
        // E4 (index 5) lies on E2's harmonic series: no clean partial.
        assert!(clean[5].iter().all(|&c| !c));
        // E2's fundamental and 2nd partial are clean; its 3rd sits on B3.
        assert!(clean[0][0] && clean[0][1] && !clean[0][2]);
    }

    #[test]
    fn kernel_matches_the_fft() {
        let rate = 48_000.0;
        let n = 4096;
        let f = 1234.567f64;
        let d = 3.0; // per second
        let frame: Vec<f32> = (0..n)
            .map(|i| {
                let t = i as f64 / rate as f64;
                ((-d * t).exp() * (std::f64::consts::TAU * f * t).cos()) as f32
            })
            .collect();
        let s = Spectrum::new(&frame, rate, 2);
        let centre = (f / s.bin_hz as f64).round() as usize;
        // A real cosine is half a positive and half a negative exponential.
        let predicted = kernel_row(&s, centre - 5, centre + 5, f, d);
        for (k, p) in predicted.iter().enumerate() {
            let measured = s.bins[centre - 5 + k];
            let p = p * 0.5;
            let tol = 1e-2 * s.bins[centre].norm();
            assert!(
                (p.re as f32 - measured.re).abs() < tol,
                "bin {k}: {p} vs {measured}"
            );
            assert!(
                (p.im as f32 - measured.im).abs() < tol,
                "bin {k}: {p} vs {measured}"
            );
        }
    }

    #[test]
    fn resolves_e4_hidden_under_e2_harmonics() {
        use crate::synth::{PluckedString, Rng, render};
        let rate = 48_000.0;
        let mut rng = Rng::new(3);
        let string = |frequency: f32, onset: f32| PluckedString {
            frequency,
            level: 0.15,
            onset,
            inharmonicity: 8e-5,
            decay: 1.0,
            rolloff: 0.9,
            pluck_position: 0.17,
            polarization_cents: 0.0,
        };
        let mut strum: Vec<PluckedString> = Instrument::Guitar
            .strings()
            .iter()
            .enumerate()
            .map(|(i, t)| string(t.frequency, 0.02 * i as f32))
            .collect();
        // E4 12 cents flat, everything else in tune.
        strum[5].frequency *= 2f32.powf(-12.0 / 1200.0);
        let signal = render(&strum, rate, 1.0, 0.001, &mut rng);
        let start = (0.2 * rate) as usize;
        let config = PolyConfig::new(rate, Instrument::Guitar);
        let est = estimate_resolved(&signal[start..start + 32_768], Instrument::Guitar, &config);
        let e4 = est[5].expect("E4");
        assert!(
            (e4.cents + 12.0).abs() < 2.0,
            "E4 at {:+.2} cents",
            e4.cents
        );
    }

    #[test]
    fn muted_strings_are_not_present() {
        use crate::synth::{Rng, StrumOptions, highpass, random_strum, render};
        let rate = 48_000.0;
        for (instrument, muted) in [
            (Instrument::Guitar, 2usize), // D3
            (Instrument::Guitar, 0),      // E2
            (Instrument::Ukulele, 1),     // C4
        ] {
            let mut rng = Rng::new(7 + muted as u64);
            let opts = StrumOptions {
                max_detune_cents: 10.0,
                ..StrumOptions::for_instrument(instrument)
            };
            let mut strum = random_strum(instrument, &opts, &mut rng);
            strum[muted].level = 0.0;
            let last = strum.iter().map(|s| s.onset).fold(0.0, f32::max);
            let mut x = render(&strum, rate, last + 0.9, 0.002, &mut rng);
            highpass(&mut x, 100.0, rate);
            let start = ((last + 0.08) * rate) as usize;
            let est = estimate_strum(
                &x[start..start + 32_768],
                instrument,
                &PolyConfig::new(rate, instrument),
            );
            for (i, e) in est.iter().enumerate() {
                let e = e.expect("estimate");
                let expected =
                    if clean_partials(instrument.strings(), &PolyConfig::new(rate, instrument))[i]
                        .iter()
                        .any(|&c| c)
                    {
                        Some(i != muted)
                    } else {
                        None
                    };
                assert_eq!(
                    e.present, expected,
                    "{instrument:?} string {i} (muted {muted}): {:?}",
                    e.prominence_db
                );
            }
        }
    }

    #[test]
    fn spectrum_peak_is_accurate_for_a_sine() {
        let rate = 48_000.0;
        let f = 123.4;
        let frame: Vec<f32> = (0..32_768)
            .map(|i| (TAU * f * i as f32 / rate).sin())
            .collect();
        let s = Spectrum::new(&frame, rate, 2);
        let (hz, _) = s.peak_near(123.0, 30.0).unwrap();
        assert!(cents_between(hz, f).abs() < 0.2, "{hz}");
    }
}
