//! Adattamento della griglia ai battiti rilevati, con tre modalita' per brano:
//! - Auto: un solo tempo finche' i battiti non si scostano di piu' di 30 ms per almeno 32 battiti; allora un nuovo tratto;
//! - Fisso: un solo tempo e una sola fase per tutto il brano (retta robusta sui battiti, tempo arrotondato se possibile);
//! - Flessibile: segue un batterista vero battuta per battuta (punti di nodo sulle battute, tolleranza 15 ms).
//!
//! Il risultato e' sempre una lista di battiti consecutivi (i battiti mancanti nel rilevamento sono riempiti, quelli in piu' scartati),
//! cosi' il motore, che lavora sull'elenco dei battiti, non cambia.

/// Scostamento oltre il quale un battito e' "fuori" dalla retta del tratto (s)
const TOLERANCE: f64 = 0.030;
/// Battiti guardati in avanti per decidere se il tempo e' cambiato
const RUN: usize = 8;
/// Lunghezza minima di un tratto (battiti)
const MIN_SECTION: usize = 32;
/// Due tratti con periodi che differiscono meno di cosi' (frazione) sono lo stesso tempo
const SAME_TEMPO: f64 = 0.002;
/// Battiti recenti con cui si numerano i battiti uno dopo l'altro
const LOCAL: usize = 16;
/// Con meno battiti la griglia non si adatta: resta quella rilevata
const MIN_BEATS: usize = 16;
/// Flessibile: scostamento massimo di una battuta dalla retta fra due nodi (s) e battute minime di un tratto
const FLEX_TOLERANCE: f64 = 0.015;
const FLEX_MIN_BARS: usize = 2;
/// Arrotondamento del tempo a BPM interi o mezzi: massimo scarto dal tempo misurato e peggioramento ammesso dell'adattamento (s)
const ROUND_RANGE: f64 = 0.1;
const ROUND_SLACK: f64 = 0.003;
/// Etichette BPM: battiti minimi fra due etichette e variazione minima (BPM)
const LABEL_MIN_BEATS: f64 = 64.0;
const LABEL_MIN_CHANGE: f64 = 0.6;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum GridMode {
    Auto,
    Fixed,
    Flexible,
}

impl GridMode {
    pub fn from_id(v: u8) -> GridMode {
        match v {
            1 => GridMode::Fixed,
            2 => GridMode::Flexible,
            _ => GridMode::Auto,
        }
    }
    pub fn id(self) -> u8 {
        match self {
            GridMode::Auto => 0,
            GridMode::Fixed => 1,
            GridMode::Flexible => 2,
        }
    }
}

pub struct Fitted {
    /// battiti consecutivi (s)
    pub beats: Vec<f64>,
    /// tempo del tratto piu' lungo
    pub bpm: f64,
    /// indice (in `beats`) del primo battito di una battuta, modulo 4
    pub downbeat: usize,
    /// cambi di tempo: (indice del battito in `beats`, secondi, BPM del tratto che inizia li')
    pub markers: Vec<(f64, f64, f64)>,
}

/// retta t = off + per * n
#[derive(Clone, Copy)]
struct Line {
    per: f64,
    off: f64,
}

impl Line {
    fn at(&self, n: f64) -> f64 {
        self.off + self.per * n
    }
}

fn median(v: &mut [f64]) -> f64 {
    if v.is_empty() {
        return 0.0;
    }
    v.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let m = v.len() / 2;
    if v.len() % 2 == 1 { v[m] } else { 0.5 * (v[m - 1] + v[m]) }
}

/// minimi quadrati su (n, t)
fn lsq(p: &[(f64, f64)]) -> Option<Line> {
    if p.len() < 2 {
        return None;
    }
    let k = p.len() as f64;
    let (mn, mt) = (p.iter().map(|x| x.0).sum::<f64>() / k, p.iter().map(|x| x.1).sum::<f64>() / k);
    let (mut sxy, mut sxx) = (0.0, 0.0);
    for &(n, t) in p {
        sxy += (n - mn) * (t - mt);
        sxx += (n - mn) * (n - mn);
    }
    if sxx <= 0.0 {
        return None;
    }
    let per = sxy / sxx;
    if !(per.is_finite() && per > 0.0) {
        return None;
    }
    Some(Line { per, off: mt - per * mn })
}

/// Retta robusta: una stima con le mediane, poi i minimi quadrati solo sui battiti vicini alla stima (se sono piu' della meta')
fn robust_line(p: &[(f64, f64)]) -> Option<Line> {
    if p.len() < 2 {
        return None;
    }
    let h = (p.len() / 2).max(1);
    let mut sl: Vec<f64> = (0..p.len() - h).filter(|&i| p[i + h].0 > p[i].0).map(|i| (p[i + h].1 - p[i].1) / (p[i + h].0 - p[i].0)).collect();
    let per = median(&mut sl);
    if !(per.is_finite() && per > 0.0) {
        return lsq(p);
    }
    let mut of: Vec<f64> = p.iter().map(|&(n, t)| t - per * n).collect();
    let guess = Line { per, off: median(&mut of) };
    let inl: Vec<(f64, f64)> = p.iter().copied().filter(|&(n, t)| (t - guess.at(n)).abs() <= 2.0 * TOLERANCE).collect();
    if inl.len() * 2 > p.len() {
        lsq(&inl).or(Some(guess))
    } else {
        lsq(p).or(Some(guess))
    }
}

fn median_dist(l: &Line, p: &[(f64, f64)]) -> f64 {
    let mut d: Vec<f64> = p.iter().map(|&(n, t)| (t - l.at(n)).abs()).collect();
    median(&mut d)
}

/// Tempo a BPM interi (poi a mezzi) se l'adattamento non peggiora
fn rounded(l: Line, p: &[(f64, f64)]) -> Line {
    let bpm = 60.0 / l.per;
    let base = median_dist(&l, p);
    for cand in [bpm.round(), (bpm * 2.0).round() / 2.0] {
        if cand <= 0.0 || (cand - bpm).abs() > ROUND_RANGE {
            continue;
        }
        let per = 60.0 / cand;
        let mut of: Vec<f64> = p.iter().map(|&(n, t)| t - per * n).collect();
        let r = Line { per, off: median(&mut of) };
        if median_dist(&r, p) <= base + ROUND_SLACK {
            return r;
        }
    }
    l
}

/// Numera i battiti: ogni battito prende il numero intero piu' vicino sulla retta dei battiti recenti. Un battito mancante lascia un
/// buco nei numeri (poi riempito), uno in piu' (stesso numero del precedente) si scarta tenendo il piu' vicino alla retta.
fn number(beats: &[f64]) -> Vec<(i64, f64)> {
    let mut gaps: Vec<f64> = beats.windows(2).map(|w| w[1] - w[0]).filter(|g| *g > 0.0).collect();
    let p0 = median(&mut gaps);
    if beats.len() < 2 || p0 <= 0.0 {
        return vec![];
    }
    let mut out: Vec<(i64, f64)> = Vec::with_capacity(beats.len());
    for &t in beats {
        let Some(&(last_n, last_t)) = out.last() else {
            out.push((0, t));
            continue;
        };
        let rec = &out[out.len().saturating_sub(LOCAL)..];
        let pts: Vec<(f64, f64)> = rec.iter().map(|&(n, t)| (n as f64, t)).collect();
        let line = if pts.len() >= 8 { lsq(&pts).filter(|l| l.per > 0.6 * p0 && l.per < 1.6 * p0) } else { None };
        let l = line.unwrap_or_else(|| {
            let mut of: Vec<f64> = pts.iter().map(|&(n, t)| t - p0 * n).collect();
            Line { per: p0, off: median(&mut of) }
        });
        let n = ((t - l.off) / l.per).round() as i64;
        if n > last_n {
            out.push((n, t));
        } else if n == last_n && (t - l.at(n as f64)).abs() < (last_t - l.at(n as f64)).abs() {
            *out.last_mut().unwrap() = (n, t);
        }
    }
    out
}

/// Somma degli scarti al quadrato dalla retta ai minimi quadrati
fn sse(p: &[(f64, f64)]) -> f64 {
    match lsq(p) {
        Some(l) => p.iter().map(|&(n, t)| (t - l.at(n)).powi(2)).sum(),
        None => f64::MAX / 4.0,
    }
}

/// Tratti a tempo costante (Auto): intervalli [inizio, fine) sull'elenco numerato
fn sections(p: &[(f64, f64)]) -> Vec<(usize, usize)> {
    let n = p.len();
    let mut starts = vec![0usize];
    let mut s = 0usize;
    let mut i = s + MIN_SECTION;
    while i + RUN <= n {
        let Some(line) = robust_line(&p[s..i]) else { break };
        let ahead = |len: usize| -> f64 {
            let e = (i + len).min(n);
            median_dist(&line, &p[i..e])
        };
        if ahead(RUN) > TOLERANCE && ahead(32) > TOLERANCE {
            // il tempo e' cambiato: il punto di stacco e' dove la somma degli errori delle due rette e' minima
            let lo = (i.saturating_sub(16)).max(s + 16);
            let hi = (i + RUN).min(n.saturating_sub(RUN));
            let end = (i + 32).min(n);
            let cut = (lo..=hi.max(lo)).filter(|&c| c + 4 <= end).min_by(|&a, &b| {
                let (ea, eb) = (sse(&p[s..a]) + sse(&p[a..end]), sse(&p[s..b]) + sse(&p[b..end]));
                ea.partial_cmp(&eb).unwrap_or(std::cmp::Ordering::Equal)
            });
            match cut {
                Some(c) if c > s => {
                    starts.push(c);
                    s = c;
                    i = s + MIN_SECTION;
                    continue;
                }
                _ => {}
            }
        }
        i += 1;
    }
    let mut secs: Vec<(usize, usize)> = starts.iter().enumerate().map(|(k, &a)| (a, starts.get(k + 1).copied().unwrap_or(n))).collect();
    // tratti con lo stesso tempo (entro 0,2%): si uniscono
    let mut k = 0;
    while k + 1 < secs.len() {
        let (a, b) = (robust_line(&p[secs[k].0..secs[k].1]), robust_line(&p[secs[k + 1].0..secs[k + 1].1]));
        let same = match (a, b) {
            (Some(a), Some(b)) => ((a.per - b.per) / a.per).abs() < SAME_TEMPO,
            _ => true,
        };
        if same {
            secs[k].1 = secs[k + 1].1;
            secs.remove(k + 1);
        } else {
            k += 1;
        }
    }
    secs
}

/// Nodi (numero di battito, secondi, BPM del tratto che segue) -> battiti da `n0` a `n1`. Fra due nodi il tempo e' costante,
/// dopo l'ultimo si prosegue al suo tempo, e un numero intero di battiti sta sempre esattamente fra due nodi.
fn render(knots: &[(f64, f64)], n0: i64, n1: i64) -> Vec<f64> {
    let mut out = Vec::with_capacity((n1 - n0 + 1).max(0) as usize);
    let mut k = 0usize;
    for n in n0..=n1 {
        let x = n as f64;
        while k + 2 < knots.len() && x >= knots[k + 1].0 {
            k += 1;
        }
        let (a, b) = if knots.len() >= 2 { (knots[k], knots[k + 1]) } else { (knots[0], (knots[0].0 + 1.0, knots[0].1 + 0.5)) };
        out.push(a.1 + (x - a.0) * (b.1 - a.1) / (b.0 - a.0));
    }
    out
}

/// Flessibile: un nodo su ogni battuta, tolti quelli che stanno sulla retta fra i vicini
fn flexible_knots(p: &[(f64, f64)], bar_res: i64) -> Vec<(f64, f64)> {
    let (n_first, n_last) = (p[0].0 as i64, p[p.len() - 1].0 as i64);
    // numero del primo battito di battuta a o dopo il primo battito
    let mut b0 = n_first - (n_first - bar_res).rem_euclid(4);
    if b0 < n_first - 3 {
        b0 += 4;
    }
    // tempo di ogni battuta: retta sui battiti delle battute vicine (da quella prima a quella dopo)
    let mut bars: Vec<(f64, f64)> = vec![];
    let mut nb = b0;
    while nb <= n_last {
        let seg: Vec<(f64, f64)> = p.iter().copied().filter(|&(n, _)| n >= (nb - 4) as f64 && n <= (nb + 4) as f64).collect();
        if seg.len() >= 2 {
            if let Some(l) = lsq(&seg) {
                bars.push((nb as f64, l.at(nb as f64)));
            }
        }
        nb += 4;
    }
    if bars.len() < 3 {
        return bars;
    }
    // il tratto si allunga finche' ogni battuta sta entro 15 ms dalla retta fra i due nodi; una battuta storta (un fill) non basta
    // a chiuderlo, due di fila si'
    let bar_dev = |a: usize, e: usize, j: usize| -> f64 {
        let (ka, ke) = (bars[a], bars[e]);
        let l = Line { per: (ke.1 - ka.1) / (ke.0 - ka.0), off: 0.0 };
        let at = |n: f64| ka.1 + l.per * (n - ka.0);
        let (lo, hi) = (bars[j].0, bars[j].0 + 4.0);
        let mut d: Vec<f64> = p.iter().filter(|&&(n, _)| n >= lo && n < hi).map(|&(n, t)| (t - at(n)).abs()).collect();
        median(&mut d)
    };
    let mut knots = vec![bars[0]];
    let mut a = 0usize;
    while a + 1 < bars.len() {
        let mut e = (a + FLEX_MIN_BARS).min(bars.len() - 1);
        let mut best = a + 1;
        while e < bars.len() {
            let mut bad_run = 0;
            let mut ok = true;
            for j in a..e {
                if bar_dev(a, e, j) > FLEX_TOLERANCE {
                    bad_run += 1;
                    if bad_run >= 2 {
                        ok = false;
                        break;
                    }
                } else {
                    bad_run = 0;
                }
            }
            if !ok {
                break;
            }
            best = e;
            e += 1;
        }
        knots.push(bars[best]);
        a = best;
    }
    knots
}

/// Adatta la griglia. `base`: battiti rilevati (o corretti a mano); `down`: indice in `base` di un battito che apre la battuta.
/// None se i battiti sono troppo pochi: resta la griglia rilevata.
pub fn fit(base: &[f64], down: usize, mode: GridMode) -> Option<Fitted> {
    let p_i = number(base);
    if p_i.len() < MIN_BEATS {
        return None;
    }
    let p: Vec<(f64, f64)> = p_i.iter().map(|&(n, t)| (n as f64, t)).collect();
    let (n0, n1) = (p_i[0].0, p_i[p_i.len() - 1].0);
    // fase della battuta: numero del battito (piu' vicino nel tempo) che apre la battuta
    let td = base.get(down.min(base.len().saturating_sub(1))).copied().unwrap_or(base[0]);
    let nd = p_i.iter().min_by(|a, b| (a.1 - td).abs().partial_cmp(&(b.1 - td).abs()).unwrap_or(std::cmp::Ordering::Equal)).map_or(0, |x| x.0);
    let bar_res = nd.rem_euclid(4);

    // nodi (numero, secondi)
    let mut knots: Vec<(f64, f64)> = match mode {
        GridMode::Fixed => {
            let l = rounded(robust_line(&p)?, &p);
            vec![(n0 as f64, l.at(n0 as f64)), (n1 as f64, l.at(n1 as f64))]
        }
        GridMode::Auto => {
            let secs = sections(&p);
            let mut k: Vec<(f64, f64)> = vec![];
            for &(a, b) in &secs {
                let l = rounded(robust_line(&p[a..b])?, &p[a..b]);
                let n = p[a].0;
                k.push((n, l.at(n)));
            }
            // dopo l'ultimo nodo si prosegue al tempo dell'ultimo tratto
            let &(a, b) = secs.last()?;
            let l = rounded(robust_line(&p[a..b])?, &p[a..b]);
            k.push((n1 as f64, l.at(n1 as f64)));
            k
        }
        GridMode::Flexible => {
            let mut k = flexible_knots(&p, bar_res);
            if k.len() < 2 {
                let l = rounded(robust_line(&p)?, &p);
                k = vec![(n0 as f64, l.at(n0 as f64)), (n1 as f64, l.at(n1 as f64))];
            } else {
                // dopo l'ultimo nodo il tempo e' quello dell'ultimo tratto: si porta il nodo fino all'ultimo battito
                let (a, b) = (k[k.len() - 2], k[k.len() - 1]);
                if (n1 as f64) > b.0 {
                    k.push((n1 as f64, b.1 + (n1 as f64 - b.0) * (b.1 - a.1) / (b.0 - a.0)));
                }
            }
            k
        }
    };
    // i nodi fuori dall'intervallo dei battiti numerati restano (servono all'interpolazione)
    knots.dedup_by(|b, a| b.0 <= a.0);
    if knots.len() < 2 {
        return None;
    }
    let beats = render(&knots, n0, n1);
    if beats.len() < 2 || beats.windows(2).any(|w| !(w[1] > w[0])) {
        return None;
    }
    // BPM di ogni tratto e tratto piu' lungo
    let mut markers: Vec<(f64, f64, f64)> = vec![];
    let (mut best_len, mut bpm) = (-1.0f64, 60.0 / ((beats[beats.len() - 1] - beats[0]) / (beats.len() - 1) as f64));
    for w in knots.windows(2) {
        let b = 60.0 * (w[1].0 - w[0].0) / (w[1].1 - w[0].1);
        if w[1].0 - w[0].0 > best_len {
            best_len = w[1].0 - w[0].0;
            bpm = b;
        }
        markers.push((w[0].0 - n0 as f64, w[0].1, b));
    }
    let downbeat = (bar_res - n0).rem_euclid(4) as usize;
    Some(Fitted { beats, bpm, downbeat, markers })
}

/// Etichette BPM da mostrare sulla forma d'onda: dove il tempo cambia davvero (almeno 0,6 BPM e almeno 64 battiti dall'etichetta
/// precedente), come (secondi, BPM). Un brano a tempo fisso non ne ha.
pub fn tempo_labels(f: &Fitted) -> Vec<(f64, f64)> {
    let mut out: Vec<(f64, f64)> = vec![];
    if f.markers.len() < 2 {
        return out;
    }
    let mut last_bpm = f.markers[0].2;
    let mut last_n = f.markers[0].0 - LABEL_MIN_BEATS; // la prima etichetta non e' limitata dalla distanza
    let first = (f.markers[0].1, f.markers[0].2);
    for (k, &(n, t, bpm)) in f.markers.iter().enumerate().skip(1) {
        if bpm.round() == last_bpm.round() || (bpm - last_bpm).abs() < LABEL_MIN_CHANGE {
            continue;
        }
        let cand = n.max(last_n + LABEL_MIN_BEATS);
        let next_n = f.markers.get(k + 1).map_or(f64::MAX, |m| m.0);
        let end_n = (f.beats.len() - 1) as f64;
        if cand >= next_n || cand > end_n {
            continue;
        }
        let ts = if cand <= n { t } else { f.beats[(cand as usize).min(f.beats.len() - 1)] };
        out.push((ts, bpm));
        last_bpm = bpm;
        last_n = cand;
    }
    if !out.is_empty() {
        out.insert(0, first);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lcg(seed: &mut u64) -> f64 {
        *seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        ((*seed >> 33) as f64 / (1u64 << 31) as f64) - 0.5
    }

    /// battiti di un brano a tempo costante, con scarto casuale (+-`jit` s)
    fn steady(bpm: f64, n: usize, jit: f64) -> Vec<f64> {
        let mut s = 7u64;
        (0..n).map(|i| 0.4 + i as f64 * 60.0 / bpm + lcg(&mut s) * 2.0 * jit).collect()
    }

    /// batterista: tempo che oscilla fra 116 e 124 BPM con ciclo di 64 battiti
    fn drummer(n: usize) -> (Vec<f64>, Vec<f64>) {
        let mut t = 0.4;
        let (mut det, mut truth) = (vec![], vec![]);
        for i in 0..n {
            truth.push(t);
            det.push(t);
            let bpm = 120.0 + 4.0 * (2.0 * std::f64::consts::PI * i as f64 / 64.0).sin();
            t += 60.0 / bpm;
        }
        (det, truth)
    }

    fn worst(a: &[f64], b: &[f64]) -> f64 {
        a.iter().zip(b).map(|(x, y)| (x - y).abs()).fold(0.0, f64::max)
    }

    #[test]
    fn fixed_ignores_strays_and_rounds_tempo() {
        let mut b = steady(124.0, 200, 0.004);
        b[50] += 0.12; // un battito storto
        let f = fit(&b, 0, GridMode::Fixed).unwrap();
        assert!((f.bpm - 124.0).abs() < 0.05, "bpm {}", f.bpm);
        assert_eq!(f.markers.len(), 1);
        assert!(tempo_labels(&f).is_empty());
    }

    #[test]
    fn missing_beats_are_filled_and_extra_dropped() {
        let mut b = steady(120.0, 120, 0.002);
        b.remove(40);
        b.remove(40); // due battiti mancanti di fila
        b.insert(80, b[79] + 0.07); // un battito in piu' vicino a quello prima
        let f = fit(&b, 0, GridMode::Fixed).unwrap();
        assert_eq!(f.beats.len(), 120, "battiti {}", f.beats.len());
    }

    #[test]
    fn auto_keeps_one_tempo_on_jittery_steady_track() {
        let b = steady(124.0, 400, 0.012);
        let f = fit(&b, 0, GridMode::Auto).unwrap();
        assert!(f.markers.len() <= 2, "tratti {}", f.markers.len());
    }

    #[test]
    fn auto_finds_a_tempo_change() {
        let mut b: Vec<f64> = (0..128).map(|i| 0.4 + i as f64 * 60.0 / 120.0).collect();
        let t0 = b[127];
        b.extend((1..128).map(|i| t0 + i as f64 * 60.0 / 130.0));
        let f = fit(&b, 0, GridMode::Auto).unwrap();
        assert!(f.markers.len() >= 2, "tratti {}", f.markers.len());
        assert!((f.markers[0].2 - 120.0).abs() < 0.5 && (f.markers.last().unwrap().2 - 130.0).abs() < 0.5, "{:?}", f.markers);
        assert!(worst(&f.beats, &b) < 0.03, "scarto {}", worst(&f.beats, &b));
        assert!(!tempo_labels(&f).is_empty());
    }

    #[test]
    fn flexible_follows_a_live_drummer() {
        let (det, truth) = drummer(256);
        let fl = fit(&det, 0, GridMode::Flexible).unwrap();
        let au = fit(&det, 0, GridMode::Auto).unwrap();
        let (ef, ea) = (worst(&fl.beats, &truth), worst(&au.beats, &truth));
        assert!(ef < 0.02, "flessibile {ef}");
        assert!(ef <= ea + 1e-9, "flessibile {ef} contro auto {ea}");
    }

    #[test]
    fn downbeat_phase_is_kept() {
        let b = steady(128.0, 100, 0.002);
        let f = fit(&b, 3, GridMode::Fixed).unwrap();
        assert_eq!(f.downbeat, 3);
    }

    #[test]
    fn too_few_beats_keep_the_detected_grid() {
        assert!(fit(&steady(120.0, 10, 0.0), 0, GridMode::Auto).is_none());
    }
}
