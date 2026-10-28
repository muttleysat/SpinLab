//! Filtri biquad (EQ a 3 bande) ed effetti: Filter, Echo, Reverb, Flanger.

use std::f32::consts::PI;

#[derive(Clone, Copy, Default)]
pub struct Biquad {
    b0: f32, b1: f32, b2: f32, a1: f32, a2: f32,
    z1: [f32; 2], z2: [f32; 2],
}

impl Biquad {
    pub fn identity() -> Self {
        Biquad { b0: 1.0, ..Default::default() }
    }
    fn set(&mut self, b0: f32, b1: f32, b2: f32, a0: f32, a1: f32, a2: f32) {
        self.b0 = b0 / a0; self.b1 = b1 / a0; self.b2 = b2 / a0;
        self.a1 = a1 / a0; self.a2 = a2 / a0;
    }
    pub fn low_shelf(&mut self, sr: f32, f: f32, db: f32) {
        let a = 10f32.powf(db / 40.0);
        let w = 2.0 * PI * f / sr;
        let (c, s) = (w.cos(), w.sin());
        let al = s / 2.0 * ((a + 1.0 / a) * (1.0 / 0.9 - 1.0) + 2.0).sqrt();
        let t = 2.0 * a.sqrt() * al;
        self.set(a * ((a + 1.0) - (a - 1.0) * c + t), 2.0 * a * ((a - 1.0) - (a + 1.0) * c),
                 a * ((a + 1.0) - (a - 1.0) * c - t), (a + 1.0) + (a - 1.0) * c + t,
                 -2.0 * ((a - 1.0) + (a + 1.0) * c), (a + 1.0) + (a - 1.0) * c - t);
    }
    pub fn high_shelf(&mut self, sr: f32, f: f32, db: f32) {
        let a = 10f32.powf(db / 40.0);
        let w = 2.0 * PI * f / sr;
        let (c, s) = (w.cos(), w.sin());
        let al = s / 2.0 * ((a + 1.0 / a) * (1.0 / 0.9 - 1.0) + 2.0).sqrt();
        let t = 2.0 * a.sqrt() * al;
        self.set(a * ((a + 1.0) + (a - 1.0) * c + t), -2.0 * a * ((a - 1.0) + (a + 1.0) * c),
                 a * ((a + 1.0) + (a - 1.0) * c - t), (a + 1.0) - (a - 1.0) * c + t,
                 2.0 * ((a - 1.0) - (a + 1.0) * c), (a + 1.0) - (a - 1.0) * c - t);
    }
    pub fn peak(&mut self, sr: f32, f: f32, q: f32, db: f32) {
        let a = 10f32.powf(db / 40.0);
        let w = 2.0 * PI * f / sr;
        let (c, s) = (w.cos(), w.sin());
        let al = s / (2.0 * q);
        self.set(1.0 + al * a, -2.0 * c, 1.0 - al * a, 1.0 + al / a, -2.0 * c, 1.0 - al / a);
    }
    pub fn lowpass(&mut self, sr: f32, f: f32, q: f32) {
        let w = 2.0 * PI * f / sr;
        let (c, s) = (w.cos(), w.sin());
        let al = s / (2.0 * q);
        self.set((1.0 - c) / 2.0, 1.0 - c, (1.0 - c) / 2.0, 1.0 + al, -2.0 * c, 1.0 - al);
    }
    pub fn highpass(&mut self, sr: f32, f: f32, q: f32) {
        let w = 2.0 * PI * f / sr;
        let (c, s) = (w.cos(), w.sin());
        let al = s / (2.0 * q);
        self.set((1.0 + c) / 2.0, -(1.0 + c), (1.0 + c) / 2.0, 1.0 + al, -2.0 * c, 1.0 - al);
    }
    #[inline]
    pub fn tick(&mut self, ch: usize, x: f32) -> f32 {
        let y = self.b0 * x + self.z1[ch];
        self.z1[ch] = self.b1 * x - self.a1 * y + self.z2[ch];
        self.z2[ch] = self.b2 * x - self.a2 * y;
        y
    }
}

/// EQ 3 bande (manopole 0..1, 0.5 = neutro, 0 = kill).
pub struct Eq3 {
    lo: Biquad, mid: Biquad, hi: Biquad,
    vals: [f32; 3],
    sr: f32,
}

fn knob_db(v: f32) -> f32 {
    if v >= 0.5 { (v - 0.5) * 2.0 * 6.0 } else if v <= 0.01 { -60.0 } else { -40.0 * (1.0 - v * 2.0).powf(0.8) }
}

impl Eq3 {
    pub fn new(sr: f32) -> Self {
        let mut e = Eq3 { lo: Biquad::identity(), mid: Biquad::identity(), hi: Biquad::identity(), vals: [-1.0; 3], sr };
        e.set([0.5; 3]);
        e
    }
    pub fn set(&mut self, v: [f32; 3]) {
        if v == self.vals { return; }
        self.vals = v;
        self.lo.low_shelf(self.sr, 250.0, knob_db(v[0]));
        self.mid.peak(self.sr, 1200.0, 0.7, knob_db(v[1]));
        self.hi.high_shelf(self.sr, 4000.0, knob_db(v[2]));
    }
    pub fn process(&mut self, buf: &mut [f32]) {
        if self.vals == [0.5; 3] { return; }
        for f in buf.chunks_exact_mut(2) {
            for c in 0..2 {
                f[c] = self.hi.tick(c, self.mid.tick(c, self.lo.tick(c, f[c])));
            }
        }
    }
}

/// Filtro a variabili di stato (topologia TPT): uscite passa-basso e passa-alto dallo stesso stato.
#[derive(Clone, Copy, Default)]
struct Svf {
    ic1: f32,
    ic2: f32,
}

impl Svf {
    /// `g` = tan(pi*fc/sr), `k` = 1/Q. Ritorna (passa-basso, passa-alto).
    #[inline]
    fn tick(&mut self, x: f32, g: f32, k: f32) -> (f32, f32) {
        let a1 = 1.0 / (1.0 + g * (g + k));
        let a2 = g * a1;
        let a3 = g * a2;
        let v3 = x - self.ic2;
        let v1 = a1 * self.ic1 + a2 * v3;
        let v2 = self.ic2 + a2 * self.ic1 + a3 * v3;
        self.ic1 = 2.0 * v1 - self.ic1;
        self.ic2 = 2.0 * v2 - self.ic2;
        (v2, x - k * v1 - v2)
    }
}

/// Filtro DJ (passa-basso a sinistra, passa-alto a destra del centro): due filtri a variabili di stato in serie, stabili
/// anche se frequenza e risonanza si muovono, che al centro sono fuori banda (trasparenti).
pub struct ColorFilter {
    svf: [[Svf; 2]; 2],
    f_lp: f32,
    f_hp: f32,
    q_lp: f32,
    q_hp: f32,
    /// entrata/uscita graduale (niente scatto quando lo si accende o spegne)
    amt: f32,
}

impl ColorFilter {
    pub fn new() -> Self {
        ColorFilter { svf: Default::default(), f_lp: 20000.0, f_hp: 20.0, q_lp: 0.707, q_hp: 0.707, amt: 0.0 }
    }

    /// ancora in funzione (o in dissolvenza di uscita)?
    pub fn active(&self) -> bool {
        self.amt > 0.0
    }

    /// `param` 0..1 (0,5 = neutro). `on` false = dissolvenza fino al bypass.
    pub fn process(&mut self, buf: &mut [f32], sr: f32, param: f32, on: bool) {
        let p = if (param - 0.5).abs() < 0.012 { 0.5 } else { param };
        let d_lp = ((0.5 - p) * 2.0).clamp(0.0, 1.0);
        let d_hp = ((p - 0.5) * 2.0).clamp(0.0, 1.0);
        let t_lp = (20000.0f32 * (60.0f32 / 20000.0).powf(d_lp)).min(sr * 0.45);
        let t_hp = 20.0f32 * (8000.0f32 / 20.0).powf(d_hp);
        // risonanza solo quando il filtro lavora davvero
        let q_lp = 0.707 + (1.6 - 0.707) * d_lp;
        let q_hp = 0.707 + (1.6 - 0.707) * d_hp;
        if on && p == 0.5 && (self.f_lp / t_lp - 1.0).abs() < 0.003 && self.f_hp / t_hp - 1.0 < 0.003 && (self.q_lp - q_lp).abs() < 0.01 && (self.q_hp - q_hp).abs() < 0.01 {
            // neutro e a regime: bypass senza elaborare
            self.f_lp = t_lp;
            self.f_hp = t_hp;
            self.q_lp = q_lp;
            self.q_hp = q_hp;
            // fuori dal bypass si rientra con una breve dissolvenza: gli stati del filtro sono fermi
            self.amt = 0.0;
            return;
        }
        // le frequenze si muovono in scala logaritmica con costante di tempo ~15 ms
        let k = 1.0 - (-1.0 / (0.015 * sr)).exp();
        let up = if on { 1.0 / (0.01 * sr) } else { -1.0 / (0.01 * sr) };
        for f in buf.chunks_exact_mut(2) {
            self.f_lp *= (t_lp / self.f_lp).powf(k);
            self.f_hp *= (t_hp / self.f_hp).powf(k);
            // anche la risonanza si muove piano: un salto di Q sposta lo stato del filtro e fa scattare
            self.q_lp += (q_lp - self.q_lp) * k;
            self.q_hp += (q_hp - self.q_hp) * k;
            self.amt = (self.amt + up).clamp(0.0, 1.0);
            let g_lp = (PI * self.f_lp / sr).tan();
            let g_hp = (PI * self.f_hp / sr).tan();
            for c in 0..2 {
                let dry = f[c];
                let lo = self.svf[0][c].tick(dry, g_lp, 1.0 / self.q_lp).0;
                let wet = self.svf[1][c].tick(lo, g_hp, 1.0 / self.q_hp).1;
                f[c] = dry + (wet - dry) * self.amt;
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FxKind { Filter, Echo, Reverb, Flanger, PingPong, Phaser, Crush, Gater, Scatter, Noise }

/// durate dei passi in battiti (TIME): 1/16 .. 4 battiti
pub const DIVS: [f64; 7] = [0.0625, 0.125, 0.25, 0.5, 1.0, 2.0, 4.0];

impl FxKind {
    pub fn from_str(s: &str) -> FxKind {
        match s {
            "echo" => FxKind::Echo, "reverb" => FxKind::Reverb, "flanger" => FxKind::Flanger, "pingpong" => FxKind::PingPong,
            "phaser" => FxKind::Phaser, "crush" => FxKind::Crush, "gater" => FxKind::Gater, "scatter" => FxKind::Scatter,
            "noise" => FxKind::Noise, _ => FxKind::Filter,
        }
    }
}

struct Comb { buf: Vec<f32>, i: usize, store: f32 }
struct Allpass { buf: Vec<f32>, i: usize }

pub struct Fx {
    pub kind: FxKind,
    pub on: bool,
    pub wet: f32,   // 0..1
    pub param: f32, // 0..1 (significato dipende dall'effetto)
    sr: f32,
    /// secondo parametro (echo: feedback, flanger: feedback, ...)
    pub param2: f32,
    /// TIME: indice in DIVS; FEEL: 0 dritto, 1 terzina (x2/3), 2 puntato (x1,5)
    pub div: usize,
    pub feel: u8,
    pp_lp: [f32; 2],
    ph_z: [[f32; 4]; 2],
    ph_fb: [f32; 2],
    cr_hold: [f32; 2],
    cr_cnt: f32,
    gate_g: f32,
    sc: [Vec<f32>; 2],
    sc_w: usize,
    sc_idx: i64,
    sc_mode: u8,
    sc_start: usize,
    sc_len: usize,
    rng: u32,
    nz: [Svf; 2],
    cf: ColorFilter,
    fl_last: [f32; 2],
    delay: [Vec<f32>; 2],
    dpos: usize,
    combs: [Vec<Comb>; 2],
    aps: [Vec<Allpass>; 2],
    lfo: f32,
    fl: [Vec<f32>; 2],
    flpos: usize,
}

impl Fx {
    pub fn new(sr: f32) -> Self {
        let scale = sr / 44100.0;
        let ct = [1116, 1188, 1277, 1356, 1422, 1491, 1557, 1617];
        let at = [556, 441, 341, 225];
        let mk = |off: usize| -> (Vec<Comb>, Vec<Allpass>) {
            (ct.iter().map(|&t| Comb { buf: vec![0.0; ((t + off) as f32 * scale) as usize], i: 0, store: 0.0 }).collect(),
             at.iter().map(|&t| Allpass { buf: vec![0.0; ((t + off) as f32 * scale) as usize], i: 0 }).collect())
        };
        let (c0, a0) = mk(0);
        let (c1, a1) = mk(23);
        Fx {
            kind: FxKind::Filter, on: false, wet: 0.5, param: 0.5, sr,
            param2: 0.5, cf: ColorFilter::new(), fl_last: [0.0; 2],
            div: 3, feel: 0, pp_lp: [0.0; 2], ph_z: [[0.0; 4]; 2], ph_fb: [0.0; 2], cr_hold: [0.0; 2], cr_cnt: 0.0, gate_g: 1.0,
            sc: [vec![0.0; (sr * 4.0) as usize], vec![0.0; (sr * 4.0) as usize]], sc_w: 0, sc_idx: i64::MIN, sc_mode: 0, sc_start: 0, sc_len: 0,
            rng: 0x1234_5678, nz: [Svf::default(); 2],
            delay: [vec![0.0; (sr * 2.0) as usize], vec![0.0; (sr * 2.0) as usize]],
            dpos: 0, combs: [c0, c1], aps: [a0, a1], lfo: 0.0,
            fl: [vec![0.0; 2048], vec![0.0; 2048]], flpos: 0,
        }
    }

    /// `beat_len`: durata di un battito in secondi; `beat_pos`: posizione in battiti all'inizio del blocco (aggancia gli
    /// effetti a tempo alla griglia)
    pub fn process(&mut self, buf: &mut [f32], beat_len: f32, beat_pos: f64) {
        let beat_len = if beat_len > 0.05 && beat_len < 4.0 { beat_len } else { 0.5 };
        let inc = 1.0 / (beat_len as f64 * self.sr as f64);
        if self.kind == FxKind::Filter {
            // il filtro resta in esecuzione finche' la dissolvenza di spegnimento non e' finita
            if self.on || self.cf.active() { self.cf.process(buf, self.sr, self.param, self.on); }
            return;
        }
        if !self.on { return; }
        match self.kind {
            FxKind::Filter => {}
            FxKind::Echo => self.echo(buf, beat_len),
            FxKind::Reverb => self.reverb(buf),
            FxKind::Flanger => self.flanger(buf),
            FxKind::PingPong => self.pingpong(buf, beat_len),
            FxKind::Phaser => self.phaser(buf, beat_pos, inc),
            FxKind::Crush => self.crush(buf),
            FxKind::Gater => self.gater(buf, beat_pos, inc),
            FxKind::Scatter => self.scatter(buf, beat_pos, inc),
            FxKind::Noise => self.noise(buf),
        }
    }

    fn step_beats(&self) -> f64 {
        DIVS[self.div.min(DIVS.len() - 1)] * match self.feel { 1 => 2.0 / 3.0, 2 => 1.5, _ => 1.0 }
    }

    fn rnd(&mut self) -> f32 {
        self.rng ^= self.rng << 13;
        self.rng ^= self.rng >> 17;
        self.rng ^= self.rng << 5;
        (self.rng as f32 / u32::MAX as f32) * 2.0 - 1.0
    }

    /// Azzera code e stati (cambio di effetto): niente code vecchie che riemergono.
    pub fn reset_state(&mut self) {
        for c in 0..2 {
            self.delay[c].iter_mut().for_each(|v| *v = 0.0);
            self.sc[c].iter_mut().for_each(|v| *v = 0.0);
            self.pp_lp[c] = 0.0;
            self.ph_z[c] = [0.0; 4];
            self.ph_fb[c] = 0.0;
            self.cr_hold[c] = 0.0;
            self.nz[c] = Svf::default();
        }
        self.sc_idx = i64::MIN;
        self.sc_mode = 0;
        self.gate_g = 1.0;
    }

    /// Eco con tempo sincronizzato (TIME e FEEL): DEPTH = tono delle ripetizioni, FEEDBACK = quante ne restano.
    fn echo(&mut self, buf: &mut [f32], beat_len: f32) {
        let n = self.delay[0].len();
        let len = (self.step_beats() * beat_len as f64 * self.sr as f64) as usize;
        let len = len.clamp(64, n - 1);
        let fb = 0.1 + 0.8 * self.param2;
        let a = 0.08 + 0.92 * self.param;
        for f in buf.chunks_exact_mut(2) {
            let rp = (self.dpos + n - len) % n;
            for c in 0..2 {
                let d = self.delay[c][rp];
                self.pp_lp[c] += (d - self.pp_lp[c]) * a;
                self.delay[c][self.dpos] = f[c] + self.pp_lp[c] * fb;
                f[c] += d * self.wet;
            }
            self.dpos = (self.dpos + 1) % n;
        }
    }

    /// Eco ping-pong: le ripetizioni rimbalzano da sinistra a destra.
    fn pingpong(&mut self, buf: &mut [f32], beat_len: f32) {
        let n = self.delay[0].len();
        let len = ((self.step_beats() * beat_len as f64 * self.sr as f64) as usize).clamp(64, n - 1);
        let fb = 0.1 + 0.8 * self.param2;
        let a = 0.08 + 0.92 * self.param;
        for f in buf.chunks_exact_mut(2) {
            let rp = (self.dpos + n - len) % n;
            let (dl, dr) = (self.delay[0][rp], self.delay[1][rp]);
            self.pp_lp[0] += (dr - self.pp_lp[0]) * a;
            self.pp_lp[1] += (dl - self.pp_lp[1]) * a;
            let m = 0.5 * (f[0] + f[1]);
            self.delay[0][self.dpos] = m + self.pp_lp[0] * fb;
            self.delay[1][self.dpos] = self.pp_lp[1] * fb;
            f[0] += dl * self.wet;
            f[1] += dr * self.wet;
            self.dpos = (self.dpos + 1) % n;
        }
    }

    /// Phaser: 4 filtri passa-tutto con LFO agganciato ai battiti (un ciclo = 4 passi).
    fn phaser(&mut self, buf: &mut [f32], mut bp: f64, inc: f64) {
        let period = self.step_beats() * 4.0;
        let depth = 0.2 + 0.8 * self.param;
        let fb = 0.7 * self.param2;
        for f in buf.chunks_exact_mut(2) {
            let lfo = (0.5 - 0.5 * (2.0 * std::f64::consts::PI * bp / period).cos()) as f32;
            let fc = 300.0 * 2f32.powf(lfo * depth * 4.5);
            let t = (PI * fc / self.sr).tan();
            let a = (t - 1.0) / (t + 1.0);
            for c in 0..2 {
                let mut x = f[c] + self.ph_fb[c] * fb;
                for s in 0..4 {
                    let y = a * x + self.ph_z[c][s];
                    self.ph_z[c][s] = x - a * y;
                    x = y;
                }
                self.ph_fb[c] = x;
                f[c] = (f[c] + self.wet * x) / (1.0 + 0.5 * self.wet);
            }
            bp += inc;
        }
    }

    /// Bitcrusher: BITS riduce la risoluzione, RATE tiene il campione per piu' tempo.
    fn crush(&mut self, buf: &mut [f32]) {
        let q = 2f32.powf(16.0 - 14.0 * self.param - 1.0);
        let hold = 1.0 + 31.0 * self.param2;
        for f in buf.chunks_exact_mut(2) {
            self.cr_cnt += 1.0;
            if self.cr_cnt >= hold {
                self.cr_cnt -= hold;
                for c in 0..2 {
                    self.cr_hold[c] = (f[c] * q).round() / q;
                }
            }
            for c in 0..2 {
                f[c] = f[c] * (1.0 - self.wet) + self.cr_hold[c] * self.wet;
            }
        }
    }

    /// Gater / trans: il suono si apre e si chiude a tempo (passo = TIME/FEEL), agganciato ai battiti.
    /// DEPTH = quanto si chiude, LENGTH = per quanta parte del passo resta aperto.
    fn gater(&mut self, buf: &mut [f32], mut bp: f64, inc: f64) {
        let step = self.step_beats();
        let duty = 0.1 + 0.8 * self.param2 as f64;
        let closed = 1.0 - self.param;
        let k = 1.0 - (-1.0 / (0.002 * self.sr)).exp();
        for f in buf.chunks_exact_mut(2) {
            let ph = (bp / step).rem_euclid(1.0);
            let target = if ph < duty { 1.0 } else { closed };
            self.gate_g += (target - self.gate_g) * k;
            let g = 1.0 - self.wet + self.wet * self.gate_g;
            f[0] *= g;
            f[1] *= g;
            bp += inc;
        }
    }

    /// Scatter: a ogni passo (TIME/FEEL) decide se lasciar passare il suono o ripetere / dimezzare / invertire / tagliare
    /// l'ultimo passo. AMOUNT = quanto spesso interviene, PATTERN = il disegno casuale (stesso valore = stessa sequenza).
    fn scatter(&mut self, buf: &mut [f32], mut bp: f64, inc: f64) {
        let n = self.sc[0].len();
        let step = self.step_beats();
        let step_len = ((step / inc) as usize).clamp(64, n / 2);
        let fade = (0.002 * self.sr) as usize;
        let seed = ((self.param2 * 1000.0) as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
        for f in buf.chunks_exact_mut(2) {
            let s = bp / step;
            let idx = s.floor() as i64;
            let i = (((s - s.floor()) * step_len as f64) as usize).min(step_len - 1);
            if idx != self.sc_idx {
                self.sc_idx = idx;
                let mut h = (idx as u64).wrapping_mul(0xBF58_476D_1CE4_E5B9) ^ seed;
                h ^= h >> 29;
                h = h.wrapping_mul(0x94D0_49BB_1331_11EB);
                h ^= h >> 32;
                let r = (h % 1000) as f32 / 1000.0;
                self.sc_mode = if r >= self.param { 0 } else { 1 + ((h / 1000) % 4) as u8 };
                self.sc_start = (self.sc_w + n - step_len) % n;
                self.sc_len = step_len;
            }
            let live = [f[0], f[1]];
            for c in 0..2 {
                self.sc[c][self.sc_w] = live[c];
            }
            self.sc_w = (self.sc_w + 1) % n;
            let len = self.sc_len.max(2);
            let out = match self.sc_mode {
                1 => { let p = (self.sc_start + i % len) % n; [self.sc[0][p], self.sc[1][p]] }
                2 => { let h = len / 2; let p = (self.sc_start + h + i % h.max(1)) % n; [self.sc[0][p], self.sc[1][p]] }
                3 => { let p = (self.sc_start + len - 1 - i % len) % n; [self.sc[0][p], self.sc[1][p]] }
                4 => [0.0, 0.0],
                _ => live,
            };
            // piccola dissolvenza ai bordi del passo: niente click
            let g = if self.sc_mode == 0 { 1.0 } else { (i as f32 / fade.max(1) as f32).min((step_len - i) as f32 / fade.max(1) as f32).min(1.0) };
            for c in 0..2 {
                let o = out[c] * g + live[c] * (1.0 - g);
                f[c] = live[c] * (1.0 - self.wet) + o * self.wet;
            }
            bp += inc;
        }
    }

    /// Noise sweep: rumore bianco filtrato che si aggiunge al suono. CUTOFF = frequenza (passa-basso sotto il centro,
    /// passa-alto sopra), RESO = risonanza, MIX = livello.
    fn noise(&mut self, buf: &mut [f32]) {
        let fc = (100.0 * 2f32.powf(7.0 * self.param)).min(self.sr * 0.45);
        let k = 1.0 / (0.6 + 4.0 * self.param2);
        let g = (PI * fc / self.sr).tan();
        let mix = self.param;
        for f in buf.chunks_exact_mut(2) {
            for c in 0..2 {
                let x = self.rnd() * 0.5;
                let (lp, hp) = self.nz[c].tick(x, g, k);
                f[c] += (lp * (1.0 - mix) + hp * mix) * self.wet * 0.7;
            }
        }
    }

    fn reverb(&mut self, buf: &mut [f32]) {
        let room = 0.7 + self.param * 0.28;
        let damp = 0.3;
        for f in buf.chunks_exact_mut(2) {
            let input = (f[0] + f[1]) * 0.015;
            for c in 0..2 {
                let mut out = 0.0;
                for cb in self.combs[c].iter_mut() {
                    let o = cb.buf[cb.i];
                    cb.store = o * (1.0 - damp) + cb.store * damp;
                    cb.buf[cb.i] = input + cb.store * room;
                    cb.i = (cb.i + 1) % cb.buf.len();
                    out += o;
                }
                for ap in self.aps[c].iter_mut() {
                    let b = ap.buf[ap.i];
                    let o = -out + b;
                    ap.buf[ap.i] = out + b * 0.5;
                    ap.i = (ap.i + 1) % ap.buf.len();
                    out = o;
                }
                f[c] += out * self.wet * 3.0;
            }
        }
    }

    fn flanger(&mut self, buf: &mut [f32]) {
        let rate = 0.1 + self.param * 2.0;
        let n = self.fl[0].len();
        for f in buf.chunks_exact_mut(2) {
            self.lfo += 2.0 * PI * rate / self.sr;
            if self.lfo > 2.0 * PI { self.lfo -= 2.0 * PI; }
            let d = (0.0015 + 0.0012 * self.lfo.sin()) * self.sr;
            for c in 0..2 {
                self.fl[c][self.flpos] = f[c] + self.fl_last[c] * (0.7 * self.param2);
                let rp = (self.flpos as f32 + n as f32 - d) % n as f32;
                let (i0, fr) = (rp as usize, rp.fract());
                let dl = self.fl[c][i0] * (1.0 - fr) + self.fl[c][(i0 + 1) % n] * fr;
                self.fl_last[c] = dl;
                f[c] = f[c] * (1.0 - 0.5 * self.wet) + dl * self.wet;
            }
            self.flpos = (self.flpos + 1) % n;
        }
    }
}

#[cfg(test)]
mod filter_tests {
    use super::*;

    /// girare di scatto la manopola del filtro, passare da passa-basso a passa-alto e accenderlo/spegnerlo non deve produrre scatti
    #[test]
    fn filter_moves_without_clicks() {
        let sr = 44100.0;
        let mut fx = Fx::new(sr);
        fx.kind = FxKind::Filter;
        fx.on = true;
        let mut phase = 0f32;
        let mut prev = 0f32;
        let mut max_step = 0f32;
        let mut block = |fx: &mut Fx, n: usize, max_step: &mut f32| {
            let mut b: Vec<f32> = (0..n).flat_map(|_| { phase += 2.0 * PI * 300.0 / sr; let v = 0.5 * phase.sin(); [v, v] }).collect();
            fx.process(&mut b, 0.5, 0.0);
            for f in b.chunks(2) {
                *max_step = max_step.max((f[0] - prev).abs());
                prev = f[0];
            }
        };
        // il seno a 300 Hz ha un passo massimo di 0,5*2pi*300/44100 ~ 0,021
        for (i, p) in [0.5f32, 0.1, 0.9, 0.3, 0.7, 0.02, 0.98, 0.5].iter().enumerate() {
            fx.param = *p;
            if i == 5 { fx.on = false; }
            if i == 6 { fx.on = true; }
            for _ in 0..8 { block(&mut fx, 512, &mut max_step); }
        }
        assert!(max_step < 0.08, "scatto nel filtro: {max_step}");
    }
}

#[cfg(test)]
mod timed_fx_tests {
    use super::*;

    const SR: f32 = 44100.0;

    /// Fa passare `blocks` blocchi da 512 frame con la posizione in battiti che avanza a 120 bpm (0,5 s a battito).
    fn run(fx: &mut Fx, input: &[f32]) -> Vec<f32> {
        let mut out = Vec::with_capacity(input.len());
        let beat = 0.5f32;
        let mut pos = 0usize;
        for chunk in input.chunks(1024) {
            let mut b = chunk.to_vec();
            let bp = pos as f64 / 2.0 / (beat as f64 * SR as f64);
            fx.process(&mut b, beat, bp);
            out.extend_from_slice(&b);
            pos += chunk.len();
        }
        out
    }
    fn sine(frames: usize, hz: f32, amp: f32) -> Vec<f32> {
        (0..frames).flat_map(|i| { let v = amp * (2.0 * PI * hz * i as f32 / SR).sin(); [v, v] }).collect()
    }
    fn rms(x: &[f32]) -> f32 {
        (x.iter().map(|v| v * v).sum::<f32>() / x.len().max(1) as f32).sqrt()
    }
    fn fx(kind: FxKind) -> Fx {
        let mut f = Fx::new(SR);
        f.kind = kind;
        f.on = true;
        f
    }

    /// ogni effetto: uscita finita e limitata, e con MIX a zero il suono passa invariato (tranne il rumore che si somma)
    #[test]
    fn all_timed_effects_are_stable_and_transparent_when_dry() {
        for kind in [FxKind::Echo, FxKind::PingPong, FxKind::Phaser, FxKind::Crush, FxKind::Gater, FxKind::Scatter, FxKind::Noise] {
            for (param, p2) in [(0.0, 0.0), (0.5, 0.5), (1.0, 1.0)] {
                let mut f = fx(kind);
                f.param = param; f.param2 = p2; f.wet = 1.0;
                let y = run(&mut f, &sine(44100 * 3, 440.0, 0.5));
                assert!(y.iter().all(|v| v.is_finite() && v.abs() < 4.0), "{kind:?} param {param}: uscita non valida");
            }
            let mut f = fx(kind);
            f.wet = 0.0;
            let x = sine(44100, 440.0, 0.5);
            let y = run(&mut f, &x);
            let err = x.iter().zip(y.iter()).map(|(a, b)| (a - b).abs()).fold(0f32, f32::max);
            assert!(err < 1e-4, "{kind:?} con MIX a zero deve lasciar passare il suono: {err}");
        }
    }

    /// il tempo dell'eco segue TIME e FEEL: 1/2 battito a 120 bpm = 0,25 s; in terzina 2/3 di quello
    #[test]
    fn echo_time_follows_time_and_feel() {
        for (div, feel, expect) in [(3usize, 0u8, 0.25f64), (3, 1, 0.25 * 2.0 / 3.0), (3, 2, 0.375), (4, 0, 0.5), (2, 0, 0.125)] {
            let mut f = fx(FxKind::Echo);
            f.div = div; f.feel = feel; f.wet = 1.0; f.param = 1.0; f.param2 = 0.0;
            let mut x = vec![0f32; 44100 * 2];
            x[0] = 1.0; x[1] = 1.0;
            let y = run(&mut f, &x);
            let first = y.chunks(2).enumerate().skip(100).find(|(_, v)| v[0].abs() > 0.3).map(|(i, _)| i).unwrap();
            assert!((first as f64 / SR as f64 - expect).abs() < 0.001, "div {div} feel {feel}: eco a {} s, atteso {expect}", first as f64 / SR as f64);
        }
    }

    /// ping-pong: la prima ripetizione e' a sinistra dopo un passo, la seconda a destra dopo due
    #[test]
    fn pingpong_bounces_between_channels() {
        let mut f = fx(FxKind::PingPong);
        f.div = 3; f.wet = 1.0; f.param = 1.0; f.param2 = 0.6;
        let mut x = vec![0f32; 44100 * 2];
        x[0] = 1.0; x[1] = 1.0;
        let y = run(&mut f, &x);
        let step = (0.25 * SR) as usize;
        let at = |n: usize, c: usize| y[(n * step) * 2 + c].abs();
        assert!(at(1, 0) > 0.3 && at(1, 1) < 0.05, "1a ripetizione a sinistra: L {} R {}", at(1, 0), at(1, 1));
        assert!(at(2, 1) > 0.1 && at(2, 0) < 0.05, "2a ripetizione a destra: L {} R {}", at(2, 0), at(2, 1));
    }

    /// gater: aperto per la prima parte del passo e chiuso dopo, agganciato ai battiti
    #[test]
    fn gater_cuts_in_time() {
        let mut f = fx(FxKind::Gater);
        f.div = 3; f.feel = 0; f.wet = 1.0; f.param = 1.0; f.param2 = 0.5; // passo 0,25 s, aperto 50%
        let y = run(&mut f, &sine(44100 * 2, 440.0, 0.5));
        let step = (0.25 * SR) as usize;
        let window = |a: f32, b: f32, n: usize| { let i0 = n * step + (a * step as f32) as usize; let i1 = n * step + (b * step as f32) as usize; rms(&y[i0 * 2..i1 * 2]) };
        for n in 2..6 {
            assert!(window(0.1, 0.4, n) > 0.3, "parte aperta");
            assert!(window(0.6, 0.95, n) < 0.03, "parte chiusa");
        }
    }

    /// bitcrusher: pochi livelli distinti; scatter: AMOUNT 0 lascia passare, AMOUNT 1 cambia il suono ma resta in tempo
    #[test]
    fn crush_scatter_phaser_and_noise_do_their_job() {
        let x = sine(44100 * 2, 300.0, 0.5);
        let mut f = fx(FxKind::Crush);
        f.wet = 1.0; f.param = 1.0; f.param2 = 0.0;
        let y = run(&mut f, &x);
        let mut levels: Vec<i32> = y.iter().map(|v| (v * 1000.0).round() as i32).collect();
        levels.sort(); levels.dedup();
        assert!(levels.len() <= 6, "livelli distinti: {}", levels.len());
        let mut f = fx(FxKind::Scatter);
        f.wet = 1.0; f.param = 0.0;
        let y = run(&mut f, &x);
        assert!(x.iter().zip(y.iter()).all(|(a, b)| (a - b).abs() < 1e-5), "AMOUNT 0: nessun intervento");
        f.param = 1.0;
        let y = run(&mut f, &x);
        let diff = x.iter().zip(y.iter()).filter(|(a, b)| (**a - **b).abs() > 0.05).count();
        assert!(diff > x.len() / 10, "scatter deve cambiare il suono: {diff}");
        let mut f = fx(FxKind::Phaser);
        f.wet = 1.0; f.param = 1.0;
        let y = run(&mut f, &x);
        assert!(x.iter().zip(y.iter()).any(|(a, b)| (a - b).abs() > 0.1), "il phaser deve cambiare il suono");
        let mut f = fx(FxKind::Noise);
        f.wet = 1.0; f.param = 0.7;
        let y = run(&mut f, &vec![0f32; 44100]);
        assert!(rms(&y) > 0.01 && rms(&y) < 0.8, "il rumore c'e' ed e' limitato: {}", rms(&y));
    }
}
