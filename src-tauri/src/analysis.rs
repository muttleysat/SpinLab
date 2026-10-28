//! Analisi del brano: beatgrid fluida (tracking dinamico + aggancio ai transitori reali),
//! tonalità e forma d'onda a 3 bande.

use crate::camelot::Key;
use crate::fx::Biquad;
use rustfft::{num_complex::Complex, FftPlanner};
use std::f32::consts::PI;

/// risoluzione della forma d'onda: campioni di picco al secondo
pub const PEAK_RATE: usize = 400;
/// frazione della finestra di energia da sottrarre per arrivare all'inizio reale del transitorio
const ATTACK_OFFSET: f64 = 0.17;
/// peso del flusso sotto i 160 Hz: la grancassa definisce il battito più delle note
const LOW_WEIGHT: f32 = 20.0;
const SMOOTH_HALF: usize = 3;
const SMOOTH_SIGMA: f64 = 1.2;

pub struct Analysis {
    /// posizioni dei battiti in secondi: ogni battito è agganciato al transitorio reale
    pub beats: Vec<f64>,
    pub bpm: f64,
    /// indice (0..3) del battito che apre la battuta
    pub downbeat: usize,
    pub key: Option<Key>,
    pub wave: Wave,
    /// loudness integrata in LUFS (ITU-R BS.1770) e picco massimo: servono alla normalizzazione del volume
    pub loudness: f64,
    pub peak: f32,
}

/// Forma d'onda a 3 bande (bassi / medi / alti), 0..255 per campione.
pub struct Wave {
    pub rate: usize,
    pub lo: Vec<u8>,
    pub mid: Vec<u8>,
    pub hi: Vec<u8>,
}

pub fn analyze(stereo: &[f32], sr: u32) -> Analysis {
    analyze_progress(stereo, sr, &mut |_| {})
}

/// Come `analyze`, ma segnala l'avanzamento (0..1) a `progress`.
pub fn analyze_progress(stereo: &[f32], sr: u32, progress: &mut dyn FnMut(f32)) -> Analysis {
    progress(0.0);
    let mono: Vec<f32> = stereo.chunks_exact(2).map(|f| 0.5 * (f[0] + f[1])).collect();
    let wave = waveform(stereo, sr);
    progress(0.08);
    let (beats, bpm, downbeat) = track_beats(&mono, sr, &mut |p| progress(0.08 + 0.72 * p));
    progress(0.8);
    let key = detect_key(&mono, sr);
    progress(0.92);
    let (loudness, peak) = loudness(stereo, sr);
    progress(1.0);
    Analysis { beats, bpm, downbeat, key, wave, loudness, peak }
}

// ------------------------------------------------------------ loudness

struct Bq {
    b: [f64; 3],
    a: [f64; 2],
    z: [f64; 2],
}
impl Bq {
    fn run(&mut self, x: f64) -> f64 {
        let y = self.b[0] * x + self.z[0];
        self.z[0] = self.b[1] * x - self.a[0] * y + self.z[1];
        self.z[1] = self.b[2] * x - self.a[1] * y;
        y
    }
}

/// Filtro di ponderazione K (ITU-R BS.1770): shelf alto a 1,68 kHz + passa-alto a 38 Hz.
fn k_weighting(sr: u32) -> [Bq; 2] {
    let fs = sr as f64;
    let (g, q, f0) = (3.999843853973347f64, 0.7071752369554196f64, 1681.974450955533f64);
    let a = 10f64.powf(g / 40.0);
    let w0 = 2.0 * std::f64::consts::PI * f0 / fs;
    let al = w0.sin() / (2.0 * q);
    let t = 2.0 * a.sqrt() * al;
    let (c, a0) = (w0.cos(), (a + 1.0) - (a - 1.0) * w0.cos() + 2.0 * a.sqrt() * al);
    let shelf = Bq {
        b: [a * ((a + 1.0) + (a - 1.0) * c + t) / a0, -2.0 * a * ((a - 1.0) + (a + 1.0) * c) / a0, a * ((a + 1.0) + (a - 1.0) * c - t) / a0],
        a: [2.0 * ((a - 1.0) - (a + 1.0) * c) / a0, ((a + 1.0) - (a - 1.0) * c - t) / a0],
        z: [0.0; 2],
    };
    let (q2, f2) = (0.5003270373238773f64, 38.13547087602444f64);
    let w = 2.0 * std::f64::consts::PI * f2 / fs;
    let al2 = w.sin() / (2.0 * q2);
    let a02 = 1.0 + al2;
    let hp = Bq {
        b: [(1.0 + w.cos()) / 2.0 / a02, -(1.0 + w.cos()) / a02, (1.0 + w.cos()) / 2.0 / a02],
        a: [-2.0 * w.cos() / a02, (1.0 - al2) / a02],
        z: [0.0; 2],
    };
    [shelf, hp]
}

/// Loudness integrata (LUFS) con gate assoluto a -70 e relativo a -10 LU, e picco massimo.
pub fn loudness(stereo: &[f32], sr: u32) -> (f64, f32) {
    let peak = stereo.iter().fold(0f32, |m, v| m.max(v.abs()));
    let frames = stereo.len() / 2;
    let blk = (sr as usize * 4 / 10).max(1);
    let hop = (sr as usize / 10).max(1);
    if frames < blk {
        return (-70.0, peak);
    }
    let mut ch = [k_weighting(sr), k_weighting(sr)];
    // energia per campione dopo la ponderazione, somma dei due canali
    let mut e = Vec::with_capacity(frames);
    for f in stereo.chunks_exact(2) {
        let mut s = 0.0;
        for c in 0..2 {
            let mut y = f[c] as f64;
            y = ch[c][0].run(y);
            y = ch[c][1].run(y);
            s += y * y;
        }
        e.push(s);
    }
    let mut pre = vec![0f64; frames + 1];
    for i in 0..frames {
        pre[i + 1] = pre[i] + e[i];
    }
    let blocks: Vec<f64> = (0..=(frames - blk) / hop).map(|k| (pre[k * hop + blk] - pre[k * hop]) / blk as f64).collect();
    let lk = |ms: f64| -0.691 + 10.0 * ms.max(1e-12).log10();
    let above: Vec<f64> = blocks.iter().copied().filter(|&b| lk(b) > -70.0).collect();
    if above.is_empty() {
        return (-70.0, peak);
    }
    let rel = lk(above.iter().sum::<f64>() / above.len() as f64) - 10.0;
    let gated: Vec<f64> = above.into_iter().filter(|&b| lk(b) > rel).collect();
    if gated.is_empty() {
        return (-70.0, peak);
    }
    (lk(gated.iter().sum::<f64>() / gated.len() as f64), peak)
}

// ------------------------------------------------------------ forma d'onda

fn waveform(stereo: &[f32], sr: u32) -> Wave {
    let frames = stereo.len() / 2;
    let per = (sr as f64 / PEAK_RATE as f64).max(1.0);
    let bins = (frames as f64 / per) as usize;
    let sr_f = sr as f32;
    // lo < 200 Hz, mid 200..3500, hi > 3500 (filtri a 4° ordine)
    let (mut lp1, mut lp2) = (Biquad::identity(), Biquad::identity());
    let (mut hp1, mut hp2) = (Biquad::identity(), Biquad::identity());
    let (mut mh1, mut mh2) = (Biquad::identity(), Biquad::identity());
    let (mut ml1, mut ml2) = (Biquad::identity(), Biquad::identity());
    lp1.lowpass(sr_f, 200.0, 0.707);
    lp2.lowpass(sr_f, 200.0, 0.707);
    hp1.highpass(sr_f, 3500.0, 0.707);
    hp2.highpass(sr_f, 3500.0, 0.707);
    mh1.highpass(sr_f, 200.0, 0.707);
    mh2.highpass(sr_f, 200.0, 0.707);
    ml1.lowpass(sr_f, 3500.0, 0.707);
    ml2.lowpass(sr_f, 3500.0, 0.707);
    let mut lo = vec![0f32; bins];
    let mut mid = vec![0f32; bins];
    let mut hi = vec![0f32; bins];
    for f in 0..frames {
        let b = (f as f64 / per) as usize;
        if b >= bins {
            break;
        }
        let x = 0.5 * (stereo[f * 2] + stereo[f * 2 + 1]);
        let l = lp2.tick(0, lp1.tick(0, x));
        let h = hp2.tick(0, hp1.tick(0, x));
        let m = ml2.tick(0, ml1.tick(0, mh2.tick(0, mh1.tick(0, x))));
        lo[b] = lo[b].max(l.abs());
        mid[b] = mid[b].max(m.abs());
        hi[b] = hi[b].max(h.abs());
    }
    let norm = |v: Vec<f32>| -> Vec<u8> {
        let mut s: Vec<f32> = v.iter().step_by(4).copied().collect();
        s.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let top = s.get(s.len().saturating_sub(1) * 995 / 1000).copied().unwrap_or(1.0).max(1e-4);
        v.iter().map(|&x| ((x / top).min(1.0).sqrt() * 255.0) as u8).collect()
    };
    Wave { rate: PEAK_RATE, lo: norm(lo), mid: norm(mid), hi: norm(hi) }
}

// ----------------------------------------------------------------- beatgrid

fn hann(n: usize) -> Vec<f32> {
    (0..n).map(|i| 0.5 - 0.5 * (2.0 * PI * i as f32 / n as f32).cos()).collect()
}

/// Envelope di onset grossolana (flusso spettrale logaritmico). Ritorna (envelope, fps, n/2).
/// L'indice `i` corrisponde al tempo (i*hop + n/2)/sr.
fn onset_envelope(mono: &[f32], sr: u32) -> (Vec<f32>, f32, usize) {
    let hop = (sr / 200).max(1) as usize; // 5 ms
    let n = 1024usize;
    let fps = sr as f32 / hop as f32;
    let fft = FftPlanner::<f32>::new().plan_fft_forward(n);
    let win = hann(n);
    let max_bin = ((9000.0 / (sr as f32 / n as f32)) as usize).min(n / 2);
    let low_bins = (160.0 / (sr as f32 / n as f32)) as usize;
    let mut prev = vec![0f32; max_bin];
    let mut env = Vec::with_capacity(mono.len() / hop);
    let mut buf = vec![Complex::new(0f32, 0f32); n];
    let mut pos = 0;
    while pos + n <= mono.len() {
        for i in 0..n {
            buf[i] = Complex::new(mono[pos + i] * win[i], 0.0);
        }
        fft.process(&mut buf);
        let mut flux = 0f32;
        for k in 1..max_bin {
            let m = (1.0 + 100.0 * buf[k].norm() / n as f32 * 8.0).ln();
            let w = if k <= low_bins { LOW_WEIGHT } else { 1.0 };
            flux += w * (m - prev[k]).max(0.0);
            prev[k] = m;
        }
        env.push(flux);
        pos += hop;
    }
    let w = fps as usize;
    let mut out = vec![0f32; env.len()];
    let mut acc = 0f64;
    for i in 0..env.len() {
        acc += env[i] as f64;
        if i >= 2 * w {
            acc -= env[i - 2 * w] as f64;
        }
        let lo = i.saturating_sub(2 * w);
        let mean = acc / (i - lo + 1) as f64;
        out[i] = (env[i] as f64 - mean).max(0.0) as f32;
    }
    let sd = (out.iter().map(|v| v * v).sum::<f32>() / out.len().max(1) as f32).sqrt().max(1e-6);
    out.iter_mut().for_each(|v| *v /= sd);
    (out, fps, n / 2)
}

/// Funzione di onset fine (1 ms): salita dell'energia a breve termine.
/// `on[k]` è riferita alla fine della finestra: tempo = (k*hop + win)/sr.
struct Fine {
    on: Vec<f32>,
    hop: usize,
    win: usize,
}

fn fine_onset(mono: &[f32], sr: u32) -> Fine {
    let hop = (sr / 1000).max(1) as usize;
    let win = hop * 10;
    let mut pre = vec![0f64; mono.len() + 1];
    let mut prev = 0f32;
    for (i, &x) in mono.iter().enumerate() {
        let d = x - 0.9 * prev; // passa-alto leggero per enfatizzare i transitori
        prev = x;
        pre[i + 1] = pre[i] + (d * d) as f64;
    }
    let count = mono.len().saturating_sub(win) / hop;
    let e: Vec<f32> = (0..count)
        .map(|k| {
            let s = k * hop;
            (((pre[s + win] - pre[s]) / win as f64) as f32 + 1e-9).ln()
        })
        .collect();
    let lag = 4;
    let mut on = vec![0f32; count];
    for k in lag..count {
        on[k] = (e[k] - e[k - lag]).max(0.0);
    }
    Fine { on, hop, win }
}

fn track_beats(mono: &[f32], sr: u32, progress: &mut dyn FnMut(f32)) -> (Vec<f64>, f64, usize) {
    let (o, fps, half) = onset_envelope(mono, sr);
    progress(0.35);
    if o.len() < (fps * 8.0) as usize {
        return (vec![], 0.0, 0);
    }
    let coarse_t = |i: f64| -> f64 { (i * (sr as f64 / fps as f64) + half as f64) / sr as f64 };

    // --- tempo globale: autocorrelazione; un periodo vero ha picchi anche ai suoi multipli (2L, 3L, 4L),
    // quindi si valuta la media dei supporti: elimina le confusioni con terzine e mezzi tempi
    let n = o.len();
    let lag_lo = (fps * 60.0 / 190.0) as usize;
    let lag_hi = (fps * 60.0 / 62.0) as usize;
    let lag_all = lag_hi * 4 + 2;
    let mut ac = vec![0f32; lag_all + 1];
    for lag in lag_lo.saturating_sub(1)..=lag_all.min(n - 1) {
        let mut sum = 0f32;
        for t in 0..n - lag {
            sum += o[t] * o[t + lag];
        }
        ac[lag] = sum / (n - lag) as f32;
    }
    let center = fps * 60.0 / 105.0;
    let at = |x: f32| -> f32 {
        let i = x.floor() as usize;
        if i + 1 >= ac.len() { return 0.0; }
        let f = x - i as f32;
        ac[i] * (1.0 - f) + ac[i + 1] * f
    };
    let mut best = lag_lo;
    let mut best_v = f32::MIN;
    for lag in lag_lo..=lag_hi {
        let l = lag as f32;
        let support = (at(l) + 0.9 * at(2.0 * l) + 0.8 * at(3.0 * l) + 0.7 * at(4.0 * l)) / 3.4;
        let r = (l / center).log2();
        let v = support * (-0.5 * (r / 1.1) * (r / 1.1)).exp();
        if v > best_v {
            best_v = v;
            best = lag;
        }
    }
    let (a, b, c) = (ac[best - 1], ac[best], ac[best + 1]);
    let denom = a - 2.0 * b + c;
    let period = best as f32 + if denom.abs() > 1e-9 { 0.5 * (a - c) / denom } else { 0.0 };

    progress(0.5);
    // --- DP di Ellis: la tolleranza sull'intervallo lascia "fluttuare" la griglia
    let lambda = 60.0f32;
    let p = period;
    let mut cscore = vec![0f32; n];
    let mut back = vec![-1i32; n];
    for t in 0..n {
        let lo = (t as f32 - 2.0 * p).ceil().max(0.0) as i64;
        let hi = (t as f32 - p / 2.0).floor() as i64;
        let mut bs = 0f32;
        let mut bi = -1i32;
        if hi >= lo {
            bs = f32::MIN;
            for q in lo..=hi {
                let d = (t as i64 - q) as f32 / p;
                let sc = cscore[q as usize] - lambda * d.ln().powi(2);
                if sc > bs {
                    bs = sc;
                    bi = q as i32;
                }
            }
        }
        if bi < 0 || bs < 0.0 {
            cscore[t] = o[t];
            back[t] = -1;
        } else {
            cscore[t] = o[t] + bs;
            back[t] = bi;
        }
    }
    let tail = (n as f32 - 1.5 * p).max(0.0) as usize;
    let mut cur = (tail..n).max_by(|&x, &y| cscore[x].partial_cmp(&cscore[y]).unwrap()).unwrap_or(n - 1) as i32;
    let mut idx = vec![];
    while cur >= 0 {
        idx.push(cur as usize);
        cur = back[cur as usize];
    }
    idx.reverse();
    if idx.len() < 8 {
        return (vec![], 60.0 * fps as f64 / period as f64, 0);
    }
    let coarse: Vec<f64> = idx.iter().map(|&i| coarse_t(i as f64)).collect();

    progress(0.65);
    // --- aggancio ai transitori reali nel dominio del tempo (±60 ms, risoluzione 1 ms)
    let fine = fine_onset(mono, sr);
    let fine_t = |k: usize| -> f64 { (k * fine.hop + fine.win) as f64 / sr as f64 };
    let khz = sr as f64 / fine.hop as f64;
    let mut refined: Vec<Option<(f64, f32)>> = Vec::with_capacity(coarse.len());
    for &tc in &coarse {
        let k0 = (((tc - 0.06) * khz) as isize).max(1) as usize;
        let k1 = (((tc + 0.06) * khz) as usize).min(fine.on.len().saturating_sub(1));
        if k1 <= k0 + 2 {
            refined.push(None);
            continue;
        }
        let (mut km, mut vm) = (k0, 0f32);
        for k in k0..=k1 {
            if fine.on[k] > vm {
                vm = fine.on[k];
                km = k;
            }
        }
        // primo aumento significativo nei 30 ms che precedono il massimo: e' l'attacco del transitorio
        let mut ks = km;
        for k in km.saturating_sub(30).max(k0)..=km {
            if fine.on[k] >= 0.5 * vm {
                ks = k;
                break;
            }
        }
        refined.push(Some((fine_t(ks) - (fine.win as f64 / sr as f64) * ATTACK_OFFSET, vm)));
    }
    let mut strengths: Vec<f32> = refined.iter().flatten().map(|r| r.1).collect();
    strengths.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let thr = strengths.get(strengths.len() / 2).copied().unwrap_or(0.0) * 0.35;
    let mut beats: Vec<f64> = coarse.clone();
    let ok: Vec<bool> = refined.iter().map(|r| r.map_or(false, |(_, v)| v >= thr)).collect();
    for i in 0..beats.len() {
        if ok[i] {
            beats[i] = refined[i].unwrap().0;
        }
    }
    // battiti senza transitorio (breakdown): interpola tra i vicini agganciati
    for i in 0..beats.len() {
        if ok[i] {
            continue;
        }
        let a = (0..i).rev().find(|&j| ok[j]);
        let b = (i + 1..beats.len()).find(|&j| ok[j]);
        match (a, b) {
            (Some(a), Some(b)) => beats[i] = beats[a] + (beats[b] - beats[a]) * (i - a) as f64 / (b - a) as f64,
            (Some(a), None) => beats[i] = beats[a] + (coarse[i] - coarse[a]),
            (None, Some(b)) => beats[i] = beats[b] - (coarse[b] - coarse[i]),
            _ => {}
        }
    }
    for i in 1..beats.len() {
        if beats[i] <= beats[i - 1] + 0.05 {
            beats[i] = beats[i - 1] + (coarse[i] - coarse[i - 1]).max(0.05);
        }
    }

    progress(0.85);
    // --- griglia fluida ma regolare: regressione lineare locale pesata (±6 battiti). Segue le derive
    // di tempo vere (batteristi, vinili) ma elimina swing e micro-scarti dei singoli colpi.
    {
        let src = beats.clone();
        let nb = src.len();
        for i in 0..nb {
            let lo = i.saturating_sub(SMOOTH_HALF);
            let hi = (i + SMOOTH_HALF).min(nb - 1);
            let (mut sw, mut sx, mut sy, mut sxx, mut sxy) = (0f64, 0f64, 0f64, 0f64, 0f64);
            for j in lo..=hi {
                let x = j as f64 - i as f64;
                let w = (-0.5 * (x / SMOOTH_SIGMA) * (x / SMOOTH_SIGMA)).exp();
                sw += w;
                sx += w * x;
                sy += w * src[j];
                sxx += w * x * x;
                sxy += w * x * src[j];
            }
            let den = sw * sxx - sx * sx;
            beats[i] = if den.abs() > 1e-9 { (sxx * sy - sx * sxy) / den } else { src[i] };
        }
    }

    // --- bpm globale: regressione lineare se stabile, altrimenti mediana
    let m = beats.len() as f64;
    let (sx, sy): (f64, f64) = beats.iter().enumerate().fold((0.0, 0.0), |a, (i, &t)| (a.0 + i as f64, a.1 + t));
    let (mx, my) = (sx / m, sy / m);
    let (mut sxy, mut sxx) = (0.0, 0.0);
    for (i, &t) in beats.iter().enumerate() {
        sxy += (i as f64 - mx) * (t - my);
        sxx += (i as f64 - mx).powi(2);
    }
    let slope = sxy / sxx;
    let rms = (beats.iter().enumerate().map(|(i, &t)| (t - (my + slope * (i as f64 - mx))).powi(2)).sum::<f64>() / m).sqrt();
    let mut ivs: Vec<f64> = beats.windows(2).map(|w| w[1] - w[0]).collect();
    ivs.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let med = ivs[ivs.len() / 2];
    let mut bpm = if rms < 0.02 { 60.0 / slope } else { 60.0 / med };
    while bpm < 75.0 {
        bpm *= 2.0;
    }
    while bpm > 175.0 {
        bpm /= 2.0;
    }
    // la griglia deve avere un marker per ogni battito del bpm mostrato
    let want = 60.0 / bpm;
    let cur = med;
    if cur > want * 1.7 {
        // griglia a mezzo tempo: aggiungi i punti intermedi
        let mut dense = Vec::with_capacity(beats.len() * 2);
        for w in beats.windows(2) {
            dense.push(w[0]);
            dense.push(0.5 * (w[0] + w[1]));
        }
        dense.push(*beats.last().unwrap());
        beats = dense;
    } else if cur < want * 0.6 {
        // griglia a doppio tempo: tieni un battito su due
        beats = beats.iter().step_by(2).copied().collect();
    }

    // --- battuta: la fase (0..3) con più energia sui battiti = tempo forte
    let mut phase_sum = [0f64; 4];
    let mut phase_cnt = [0f64; 4];
    for (i, &t) in beats.iter().enumerate() {
        let k = ((t * khz) as usize + 6).min(fine.on.len().saturating_sub(1));
        if k > 4 {
            phase_sum[i % 4] += fine.on[k - 4..=k].iter().cloned().fold(0.0, f32::max) as f64;
            phase_cnt[i % 4] += 1.0;
        }
    }
    let mean: Vec<f64> = (0..4).map(|i| phase_sum[i] / phase_cnt[i].max(1.0)).collect();
    let bestp = (0..4).max_by(|&a, &b| mean[a].partial_cmp(&mean[b]).unwrap()).unwrap_or(0);

    // --- estendi la griglia fino all'inizio e alla fine del brano
    let dur = mono.len() as f64 / sr as f64;
    // il prolungamento all'indietro si ferma all'inizio della musica (niente marker nel silenzio/rumore iniziale)
    let music_start = {
        let blk = (sr as usize / 100).max(1);
        let rms: Vec<f32> = mono.chunks(blk).map(|c| (c.iter().map(|v| v * v).sum::<f32>() / c.len() as f32).sqrt()).collect();
        let mut sorted = rms.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let thr = sorted[sorted.len() / 2] * 0.25;
        rms.iter().position(|&r| r > thr).map_or(0.0, |i| i as f64 * blk as f64 / sr as f64)
    };
    let lead = beats.iter().take_while(|&&b| b < music_start - 0.05).count();
    if lead > 0 && beats.len() - lead >= 8 {
        beats.drain(..lead);
    }
    let first_iv = (beats[1] - beats[0]).max(0.2);
    let mut shift = 0;
    while beats[0] - first_iv > music_start - 0.05 {
        beats.insert(0, beats[0] - first_iv);
        shift += 1;
    }
    let k = beats.len();
    let last_iv = (beats[k - 1] - beats[k - 2]).max(0.2);
    while *beats.last().unwrap() + last_iv < dur {
        let l = *beats.last().unwrap();
        beats.push(l + last_iv);
    }
    // se la musica parte con un attacco netto, il primo battito apre la battuta
    let bestp = if shift == 0 && (beats[0] - music_start).abs() < 0.1 { 0 } else { bestp };
    let downbeat = (bestp + 4 - shift % 4) % 4;
    (beats, bpm, downbeat)
}

// ------------------------------------------------------------------ tonalità

const KS_MAJOR: [f32; 12] = [6.35, 2.23, 3.48, 2.33, 4.38, 4.09, 2.52, 5.19, 2.39, 3.66, 2.29, 2.88];
const KS_MINOR: [f32; 12] = [6.33, 2.68, 3.52, 5.38, 2.60, 3.53, 2.54, 4.75, 3.98, 2.69, 3.34, 3.17];

fn detect_key(mono: &[f32], sr: u32) -> Option<Key> {
    let n = 8192usize;
    if mono.len() < n * 4 {
        return None;
    }
    let fft = FftPlanner::<f32>::new().plan_fft_forward(n);
    let win = hann(n);
    let total = (mono.len() - n) / n;
    let step = (total / 150).max(1);
    let mut chroma = [0f32; 12];
    let mut buf = vec![Complex::new(0f32, 0f32); n];
    let bin_hz = sr as f32 / n as f32;
    for fi in (0..total).step_by(step) {
        let s = fi * n;
        for i in 0..n {
            buf[i] = Complex::new(mono[s + i] * win[i], 0.0);
        }
        fft.process(&mut buf);
        for k in (80.0 / bin_hz) as usize..(2000.0 / bin_hz) as usize {
            let midi = 69.0 + 12.0 * (k as f32 * bin_hz / 440.0).log2();
            let r = midi.round();
            if (midi - r).abs() < 0.35 {
                chroma[(r as i32).rem_euclid(12) as usize] += buf[k].norm().sqrt();
            }
        }
    }
    let corr = |profile: &[f32; 12], rot: usize| -> f32 {
        let cm = chroma.iter().sum::<f32>() / 12.0;
        let pm = profile.iter().sum::<f32>() / 12.0;
        let (mut num, mut dc, mut dp) = (0.0, 0.0, 0.0);
        for i in 0..12 {
            let c = chroma[(i + rot) % 12] - cm;
            let p = profile[i] - pm;
            num += c * p;
            dc += c * c;
            dp += p * p;
        }
        num / (dc * dp).sqrt().max(1e-9)
    };
    let mut best = (f32::MIN, Key { pc: 0, minor: false });
    for pc in 0..12 {
        for (minor, prof) in [(false, &KS_MAJOR), (true, &KS_MINOR)] {
            let c = corr(prof, pc);
            if c > best.0 {
                best = (c, Key { pc: pc as u8, minor });
            }
        }
    }
    Some(best.1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::camelot;

    /// kick a tempo variabile (deriva lenta) + accordo La minore. Ritorna (audio, tempi veri dei kick)
    fn synth(bpm: f64, drift: f64, secs: f64, sr: u32) -> (Vec<f32>, Vec<f64>) {
        let n = (secs * sr as f64) as usize;
        let mut out = vec![0f32; n * 2];
        let mut kicks = vec![];
        let mut t = 0.37; // primo kick non a zero
        while t < secs {
            kicks.push(t);
            let local = bpm * (1.0 + drift * (t / secs * std::f64::consts::PI * 2.0).sin());
            t += 60.0 / local;
        }
        for &k in &kicks {
            let s0 = (k * sr as f64) as usize;
            for j in 0..(sr as usize / 4) {
                if s0 + j >= n {
                    break;
                }
                let ph = j as f64 / sr as f64;
                let v = ((-ph * 25.0).exp() * (2.0 * std::f64::consts::PI * (50.0 + 90.0 * (-ph * 30.0).exp()) * ph).sin() * 0.8) as f32;
                out[(s0 + j) * 2] += v;
                out[(s0 + j) * 2 + 1] += v;
            }
        }
        for i in 0..n {
            let t = i as f64 / sr as f64;
            let chord: f64 = [(110.0, 2.0), (220.0, 2.0), (261.63, 0.7), (329.63, 1.0)]
                .iter()
                .map(|(f, a)| a * (2.0 * std::f64::consts::PI * f * t).sin())
                .sum::<f64>()
                * 0.05;
            out[i * 2] += chord as f32;
            out[i * 2 + 1] += chord as f32;
        }
        (out, kicks)
    }

    /// distanza media/massima tra ogni kick vero (dopo i primi 4 s) e il marker più vicino
    fn alignment_error(beats: &[f64], kicks: &[f64]) -> (f64, f64) {
        let ds: Vec<f64> = kicks
            .iter()
            .filter(|&&k| k > 4.0 && k < kicks[kicks.len() - 1] - 2.0)
            .map(|&k| beats.iter().map(|&b| (b - k).abs()).fold(f64::MAX, f64::min))
            .collect();
        (ds.iter().sum::<f64>() / ds.len() as f64, ds.iter().cloned().fold(0.0, f64::max))
    }

    #[test]
    fn steady_bpm_key_and_alignment() {
        let sr = 44100;
        let (audio, kicks) = synth(126.0, 0.0, 40.0, sr);
        let a = analyze(&audio, sr);
        println!("bpm={} beats={} key={:?}", a.bpm, a.beats.len(), a.key.map(camelot::camelot_code));
        assert!((a.bpm - 126.0).abs() < 0.3, "bpm {}", a.bpm);
        assert_eq!(a.key.map(camelot::camelot_code).as_deref(), Some("8A"));
        let err = alignment_error(&a.beats, &kicks);
        for &k in kicks.iter().skip(4).take(12) {
            let nb = a.beats.iter().cloned().fold((f64::MAX, 0.0), |acc: (f64, f64), b| if (b - k).abs() < acc.0 { ((b - k).abs(), b - k) } else { acc });
            println!("kick {:.3} -> marker offset {:+.1} ms", k, nb.1 * 1000.0);
        }
        println!("steady: errore medio {:.1} ms, max {:.1} ms", err.0 * 1000.0, err.1 * 1000.0);
        assert!(err.0 < 0.008, "marker non allineati: {:.1} ms", err.0 * 1000.0);
    }

    #[test]
    fn fluid_grid_follows_drift() {
        let sr = 44100;
        let (audio, kicks) = synth(120.0, 0.06, 60.0, sr); // ±6% di deriva
        let a = analyze(&audio, sr);
        let err = alignment_error(&a.beats, &kicks);
        println!("fluida: errore medio {:.1} ms, max {:.1} ms", err.0 * 1000.0, err.1 * 1000.0);
        assert!(err.0 < 0.012, "la griglia non segue il tempo: {:.1} ms", err.0 * 1000.0);
    }

    #[test]
    #[ignore]
    fn real_files() {
        let files = std::env::var("SPIN_FILES").unwrap_or_default();
        for f in files.split('|').filter(|f| !f.is_empty()) {
            let t0 = std::time::Instant::now();
            let data = crate::decode::decode_file(f, 44100).expect("decode");
            let t1 = t0.elapsed();
            let a = analyze(&data, 44100);
            let ivs: Vec<f64> = a.beats.windows(2).map(|w| w[1] - w[0]).collect();
            let mean = ivs.iter().sum::<f64>() / ivs.len() as f64;
            let sd = (ivs.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / ivs.len() as f64).sqrt();
            let (mn, mx) = ivs.iter().fold((f64::MAX, 0.0f64), |a, &v| (a.0.min(v), a.1.max(v)));
            println!("{} | {:.1}s | bpm {:.2} | key {:?} | beats {} | intervallo medio {:.1} ms sd {:.1} ms (min {:.0} max {:.0}) | downbeat {} | decode {:?} totale {:?}",
                std::path::Path::new(f).file_name().unwrap().to_string_lossy(), data.len() as f64 / 2.0 / 44100.0, a.bpm,
                a.key.map(camelot::camelot_code), a.beats.len(), mean * 1000.0, sd * 1000.0, mn * 1000.0, mx * 1000.0, a.downbeat, t1, t0.elapsed());
            // profilo del tempo locale ogni 10 s
            let mut prof = String::new();
            let mut t = 5.0;
            while t < data.len() as f64 / 2.0 / 44100.0 - 5.0 {
                let i = a.beats.partition_point(|&b| b < t).clamp(4, a.beats.len() - 5);
                prof += &format!("{:.0}s:{:.1} ", t, 60.0 * 8.0 / (a.beats[i + 4] - a.beats[i - 4]));
                t += 10.0;
            }
            println!("   tempo locale: {prof}");
            {
                // scarto marker - picco di onset piu' vicino (±60 ms), mediana e percentili, in ms
                let mono: Vec<f32> = data.chunks_exact(2).map(|f| 0.5 * (f[0] + f[1])).collect();
                let fine = fine_onset(&mono, 44100);
                let mut offs: Vec<f64> = vec![];
                for &b in a.beats.iter().filter(|&&b| b > 3.0 && b < data.len() as f64 / 88200.0 - 3.0) {
                    let k0 = (b * 1000.0) as usize - 60 + 10;
                    let k1 = (b * 1000.0) as usize + 60 + 10;
                    let (mut km, mut vm) = (k0, 0f32);
                    for k in k0..k1.min(fine.on.len()) {
                        if fine.on[k] > vm { vm = fine.on[k]; km = k; }
                    }
                    offs.push((b * 1000.0) - (km as f64 - 10.0));
                }
                offs.sort_by(|a, b| a.partial_cmp(b).unwrap());
                let q = |p: f64| offs[((offs.len() - 1) as f64 * p) as usize];
                println!("   marker - picco onset (ms): p10 {:.0} p25 {:.0} mediana {:.0} p75 {:.0} p90 {:.0}", q(0.1), q(0.25), q(0.5), q(0.75), q(0.9));
            }
            if let Ok(r) = std::env::var("SPIN_ANCHORS") {
                // tempi (s) in cui l'utente sa che cade un battito: scarto dal marker piu' vicino
                for t in r.split(',').filter_map(|x| x.parse::<f64>().ok()) {
                    let (i, b) = a.beats.iter().enumerate().min_by(|x, y| (x.1 - t).abs().partial_cmp(&(y.1 - t).abs()).unwrap()).unwrap();
                    println!("   ancora {t:.3}s -> marker #{i} a {b:.3}s (scarto {:+.0} ms)", (b - t) * 1000.0);
                }
            }
            if let Ok(r) = std::env::var("SPIN_RANGE") {
                let v: Vec<f64> = r.split(',').map(|x| x.parse().unwrap()).collect();
                let ivs: Vec<String> = a.beats.windows(2).filter(|w| w[0] >= v[0] && w[0] < v[1]).map(|w| format!("{:.0}", (w[1] - w[0]) * 1000.0)).collect();
                println!("   intervalli ms {}-{}s: {}", v[0], v[1], ivs.join(" "));
            }
            if let Ok(dir) = std::env::var("SPIN_DUMP") {
                let name = std::path::Path::new(f).file_stem().unwrap().to_string_lossy().to_string();
                let js = format!("{{\"beats\":{:?},\"bpm\":{}}}", a.beats, a.bpm);
                let _ = std::fs::write(format!("{dir}/{name}.json"), js);
            }
        }
    }

    // ---------- diagnostica: immagine PNG con forma d'onda, onset fine e marker
    fn crc32(data: &[u8]) -> u32 {
        let mut c = 0xFFFF_FFFFu32;
        for &b in data {
            c ^= b as u32;
            for _ in 0..8 {
                c = if c & 1 != 0 { 0xEDB8_8320 ^ (c >> 1) } else { c >> 1 };
            }
        }
        !c
    }
    fn write_png(path: &str, w: usize, h: usize, rgb: &[u8]) {
        let mut raw = Vec::with_capacity((w * 3 + 1) * h);
        for y in 0..h {
            raw.push(0);
            raw.extend_from_slice(&rgb[y * w * 3..(y + 1) * w * 3]);
        }
        let mut z = vec![0x78, 0x01];
        let mut i = 0;
        while i < raw.len() {
            let n = (raw.len() - i).min(65535);
            z.push(if i + n == raw.len() { 1 } else { 0 });
            z.extend_from_slice(&(n as u16).to_le_bytes());
            z.extend_from_slice(&(!(n as u16)).to_le_bytes());
            z.extend_from_slice(&raw[i..i + n]);
            i += n;
        }
        let (mut a, mut b) = (1u32, 0u32);
        for &v in &raw {
            a = (a + v as u32) % 65521;
            b = (b + a) % 65521;
        }
        z.extend_from_slice(&((b << 16) | a).to_be_bytes());
        let mut out = vec![0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
        let mut chunk = |ty: &[u8], data: &[u8]| {
            out.extend_from_slice(&(data.len() as u32).to_be_bytes());
            let mut body = ty.to_vec();
            body.extend_from_slice(data);
            out.extend_from_slice(&body);
            out.extend_from_slice(&crc32(&body).to_be_bytes());
        };
        let mut ihdr = vec![];
        ihdr.extend_from_slice(&(w as u32).to_be_bytes());
        ihdr.extend_from_slice(&(h as u32).to_be_bytes());
        ihdr.extend_from_slice(&[8, 2, 0, 0, 0]);
        chunk(b"IHDR", &ihdr);
        chunk(b"IDAT", &z);
        chunk(b"IEND", &[]);
        std::fs::write(path, out).unwrap();
    }

    /// SPIN_FILES + SPIN_PNG=<cartella> + SPIN_SEGS="t0,t1,t2" (secondi, finestre di 12 s)
    #[test]
    #[ignore]
    fn plot_segments() {
        let f = std::env::var("SPIN_FILES").unwrap();
        let dir = std::env::var("SPIN_PNG").unwrap();
        let segs: Vec<f64> = std::env::var("SPIN_SEGS").unwrap().split(',').map(|x| x.parse().unwrap()).collect();
        let data = crate::decode::decode_file(&f, 44100).unwrap();
        let a = analyze(&data, 44100);
        let mono: Vec<f32> = data.chunks_exact(2).map(|f| 0.5 * (f[0] + f[1])).collect();
        let fine = fine_onset(&mono, 44100);
        let win: f64 = std::env::var("SPIN_WIN").ok().and_then(|v| v.parse().ok()).unwrap_or(12.0);
        let (w, rowh) = (1800usize, 150usize);
        let h = rowh * segs.len();
        let mut img = vec![12u8; w * h * 3];
        let mut put = |x: usize, y: usize, c: [u8; 3]| {
            if x < w && y < h {
                let i = (y * w + x) * 3;
                img[i..i + 3].copy_from_slice(&c);
            }
        };
        let mut sorted: Vec<f32> = fine.on.iter().step_by(7).cloned().collect();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let fmax = sorted[sorted.len() * 995 / 1000].max(1e-6);
        for (si, &t0) in segs.iter().enumerate() {
            let y0 = si * rowh;
            for x in 0..w {
                let ta = t0 + x as f64 / w as f64 * win;
                let tb = t0 + (x + 1) as f64 / w as f64 * win;
                let (sa, sb) = ((ta * 44100.0) as usize, ((tb * 44100.0) as usize).min(mono.len()));
                let m = mono.get(sa..sb).map_or(0.0, |s| s.iter().fold(0f32, |m, v| m.max(v.abs())));
                let hh = (m * 0.95 * (rowh as f32 * 0.32)) as usize;
                for dy in 0..=hh {
                    put(x, y0 + rowh * 40 / 100 - dy.min(rowh * 40 / 100), [70, 140, 220]);
                    put(x, y0 + rowh * 40 / 100 + dy.min(rowh * 40 / 100), [70, 140, 220]);
                }
                let (ka, kb) = (((ta * 1000.0) as usize).saturating_sub(10), ((tb * 1000.0) as usize).saturating_sub(10));
                let om = fine.on.get(ka..kb.max(ka + 1).min(fine.on.len())).map_or(0.0, |s| s.iter().cloned().fold(0.0, f32::max)) / fmax;
                let oh = (om.min(1.0) * rowh as f32 * 0.26) as usize;
                for dy in 0..oh {
                    put(x, y0 + rowh - 2 - dy, [230, 170, 40]);
                }
            }
            for (bi, &b) in a.beats.iter().enumerate() {
                if b >= t0 && b < t0 + win {
                    let x = ((b - t0) / win * w as f64) as usize;
                    let bar = (bi + 4 - a.downbeat % 4) % 4 == 0;
                    for y in 0..rowh {
                        put(x, y0 + y, if bar { [255, 255, 255] } else { [255, 60, 110] });
                    }
                }
            }
            for x in 0..w {
                put(x, y0, [60, 60, 60]);
            }
        }
        write_png(&format!("{dir}/plot.png"), w, h, &img);
        println!("bpm {:.2} beats {}", a.bpm, a.beats.len());
    }

    #[test]
    fn loudness_scales_with_level_and_ignores_silence() {
        let sr = 44100u32;
        let sine = |a: f32, secs: f32| -> Vec<f32> {
            (0..(secs * sr as f32) as usize).flat_map(|i| { let v = a * (2.0 * PI * 1000.0 * i as f32 / sr as f32).sin(); [v, v] }).collect()
        };
        let (l1, p1) = loudness(&sine(0.1, 8.0), sr);
        let (l2, _) = loudness(&sine(0.2, 8.0), sr);
        assert!((l2 - l1 - 6.02).abs() < 0.1, "{l1} {l2}");
        assert!((p1 - 0.1).abs() < 1e-3);
        // seno stereo identico sui due canali, picco -20 dBFS: BS.1770 da circa -20 LUFS
        assert!((l1 + 20.0).abs() < 0.5, "{l1}");
        // il silenzio in mezzo non cambia la misura (gate)
        let mut v = sine(0.1, 8.0);
        v.extend(std::iter::repeat(0.0).take(sr as usize * 2 * 20));
        let (l3, _) = loudness(&v, sr);
        assert!((l3 - l1).abs() < 0.3, "{l1} {l3}");
    }

    /// SPIN_FILES: tempi di decodifica (e resampling) usati dal caricamento sul deck
    #[test]
    #[ignore]
    fn load_timing() {
        for f in std::env::var("SPIN_FILES").unwrap_or_default().split('|').filter(|f| !f.is_empty()) {
            for sr in [44100u32, 48000] {
                let t0 = std::time::Instant::now();
                let d = crate::decode::decode_file(f, sr).unwrap();
                let t1 = t0.elapsed();
                let w = waveform(&d, sr);
                let t2 = t0.elapsed() - t1;
                let js = serde_json::to_string(&(w.lo.clone(), w.mid.clone(), w.hi.clone())).unwrap();
                println!("{} @{sr}: decode {:?}, waveform {:?}, json onda {} KB in {:?}", std::path::Path::new(f).file_name().unwrap().to_string_lossy(), t1, t2, js.len() / 1024, t0.elapsed() - t1 - t2);
            }
        }
    }
}
