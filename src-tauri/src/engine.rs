//! Motore: deck (con keylock e cambio tonalità), mixer, sampler.

use crate::camelot::{self, Key};
use crate::fx::{Eq3, Fx};
use serde::Serialize;
use signalsmith_stretch::Stretch;
use crate::cue::CueBus;
use std::sync::Arc;

pub const NUM_DECKS: usize = 4;

/// Movimenti del disco: freno, spinback, retromarcia e ripartenza.
/// Porta una lunghezza in battiti a un valore leggibile: la potenza di due piu' vicina (entro il 4%), altrimenti un intero
/// (entro il 3%), altrimenti due decimali. Cosi' un loop segnato a mano mostra 4 e non 4,01.
/// millisecondi dall'avvio del programma (orologio monotono)
pub fn now_ms() -> f64 {
    static START: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();
    START.get_or_init(std::time::Instant::now).elapsed().as_secs_f64() * 1000.0
}

pub fn snap_beats(x: f64) -> f64 {
    const NICE: [f64; 12] = [0.03125, 0.0625, 0.125, 0.25, 0.5, 1.0, 2.0, 4.0, 8.0, 16.0, 32.0, 64.0];
    let rel = |n: f64| (n - x).abs() / x.max(1e-9);
    let best = NICE.iter().copied().min_by(|a, b| rel(*a).partial_cmp(&rel(*b)).unwrap()).unwrap();
    if rel(best) < 0.04 {
        best
    } else if x >= 1.0 && (x - x.round()).abs() / x < 0.03 {
        x.round()
    } else {
        (x * 100.0).round() / 100.0
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Motion {
    None,
    Brake,
    Reverse,
    /// (tempo trascorso, durata)
    Spin(f64, f64),
    Resume,
}
/// loudness di riferimento (LUFS) quando nessun altro brano sta suonando
pub const REFERENCE_LUFS: f64 = -14.0;
pub const NUM_PADS: usize = 8;
/// Velocita' del rientro automatico del pitch, in punti percentuali al secondo: +8% torna a 0 in 16 s, +2% in 4 s.
pub const TEMPO_RAMP_PCT_PER_S: f64 = 0.5;

pub struct Track {
    pub name: String,
    pub path: String,
    pub data: Arc<Vec<f32>>, // stereo interleaved @ sr del motore, condiviso fra le varianti di griglia
    pub sr: u32,
    pub beats: Vec<f64>,
    pub bpm: f64,
    pub key: Option<Key>,
    /// indice (0..3) del battito che apre la battuta
    pub downbeat: usize,
}

impl Track {
    /// stessa traccia con una griglia diversa (l'audio non viene copiato)
    pub fn with_grid(&self, beats: Vec<f64>, downbeat: usize, bpm: f64) -> Track {
        Track { name: self.name.clone(), path: self.path.clone(), data: self.data.clone(), sr: self.sr, beats, bpm, key: self.key, downbeat }
    }
    pub fn frames(&self) -> usize {
        self.data.len() / 2
    }
    pub fn duration(&self) -> f64 {
        self.frames() as f64 / self.sr as f64
    }
    /// indice del battito che precede (o coincide con) t
    fn beat_index(&self, t: f64) -> Option<usize> {
        if self.beats.len() < 2 {
            return None;
        }
        Some(self.beats.partition_point(|&b| b <= t).saturating_sub(1).min(self.beats.len() - 2))
    }
    /// BPM locale (la griglia è fluida: segue il tempo reale del brano)
    pub fn local_bpm(&self, t: f64) -> f64 {
        match self.beat_index(t) {
            Some(i) => {
                let lo = i.saturating_sub(3);
                let hi = (i + 4).min(self.beats.len() - 1);
                60.0 * (hi - lo) as f64 / (self.beats[hi] - self.beats[lo])
            }
            None => self.bpm,
        }
    }
    /// fase nel battito 0..1
    pub fn beat_phase(&self, t: f64) -> Option<f64> {
        let i = self.beat_index(t)?;
        Some(((t - self.beats[i]) / (self.beats[i + 1] - self.beats[i])).clamp(0.0, 1.0))
    }
    pub fn time_at_phase(&self, t: f64, phase: f64) -> Option<f64> {
        let i = self.beat_index(t)?;
        Some(self.beats[i] + phase * (self.beats[i + 1] - self.beats[i]))
    }
    /// posizione in battiti (indice del battito + fase): continua anche fuori dai capi della griglia
    pub fn beat_pos(&self, t: f64) -> Option<f64> {
        let i = self.beat_index(t)?;
        Some(i as f64 + (t - self.beats[i]) / (self.beats[i + 1] - self.beats[i]))
    }
    /// istante (secondi) di una posizione in battiti
    pub fn time_at_beat_pos(&self, b: f64) -> Option<f64> {
        if self.beats.len() < 2 { return None; }
        let last = self.beats.len() - 1;
        let i = b.floor().clamp(0.0, (last - 1) as f64) as usize;
        Some(self.beats[i] + (b - i as f64) * (self.beats[i + 1] - self.beats[i]))
    }
    pub fn nearest_beat(&self, t: f64) -> Option<f64> {
        let i = self.beat_index(t)?;
        let (a, b) = (self.beats[i], self.beats[i + 1]);
        Some(if t - a < b - t { a } else { b })
    }
}

#[inline]
pub fn cubic(p: &[f32], i: isize, t: f32, c: usize, frames: usize) -> f32 {
    let g = |k: isize| -> f32 {
        let k = k.clamp(0, frames as isize - 1) as usize;
        p[k * 2 + c]
    };
    let (a, b, cc, d) = (g(i - 1), g(i), g(i + 1), g(i + 2));
    b + 0.5 * t * (cc - a + t * (2.0 * a - 5.0 * b + 4.0 * cc - d + t * (3.0 * (b - cc) + d - a)))
}

pub struct Deck {
    sr: u32,
    pub track: Option<Arc<Track>>,
    pub pos: f64, // in frame
    pub playing: bool,
    preview: bool,
    pub tempo: f64, // frazione, es. 0.04 = +4%
    /// rientro graduale del pitch in corso: tempo (frazione) verso cui si va
    pub tempo_ramp: Option<f64>,
    /// tasto SYNC acceso: il deck insegue tempo e fase del deck guida. Resta acceso finche' non lo spegni tu
    pub sync: bool,
    sync_prev_playing: bool,
    /// partenza in attesa del prossimo battito del deck guida (SYNC acceso, play premuto su un battito, per esempio sul cue)
    pub armed: bool,
    arm_pos: f64,
    pub range: f64, // 0.04, 0.08, 0.16, 0.5, 1.0
    pub keylock: bool,
    pub key_shift: i32,
    pub cue_point: f64, // sec
    pub hot_cues: [Option<f64>; 8],
    pub quantize: bool,
    /// griglia analizzata (per il reset) e griglia di lavoro modificabile
    grid_orig: (Vec<f64>, usize, f64),
    grid_base: Vec<f64>,
    grid_down: usize,
    grid_bpm: f64,
    /// modalita' della griglia (0 auto, 1 fisso, 2 flessibile), vedi gridfit.rs
    pub grid_mode: u8,
    /// etichette BPM dove il tempo cambia: (secondi, BPM)
    grid_labels: Vec<(f64, f64)>,
    /// battiti fissati a mano con «anchor_here» (indici in grid_base); non vengono più spostati
    grid_pins: Vec<usize>,
    pub volume: f32,
    pub xf_assign: u8, // 0=A 1=B 2=thru
    pub eq: Eq3,
    pub eq_knobs: [f32; 3],
    pub fx: Fx,
    stretch: Stretch,
    stretch_mode: bool,
    need_preroll: bool,
    last_transpose: f32,
    in_buf: Vec<f32>,
    /// anticipo (in frame) con cui leggiamo l'input dello stretcher per compensarne la latenza:
    /// `pos` e' il puntatore di lettura, la posizione ascoltata e' `pos - lead`
    lead: f64,
    /// deck tenuto fermo (scrubbing col mouse): silenzioso e fermo
    pub held: bool,
    /// preascolto in cuffia (PFL): il segnale prima del fader di canale va al bus delle cuffie
    pub pfl: bool,
    pfl_buf: Vec<f32>,
    /// movimenti: velocita' di riproduzione extra (1 = normale, 0 = fermo, negativa = al contrario), durata del freno,
    /// e SLIP: mentre dura un effetto il brano "continua" in silenzio e al rilascio ci si ritrova dove sarebbe arrivato
    mod_rate: f64,
    /// pitch bend momentaneo (jog del controller, -0.5..0.5): accelera o rallenta senza toccare il tempo impostato
    pub nudge: f64,
    pub motion: Motion,
    pub brake_dur: f64,
    pub slip: bool,
    slip_sec: Option<f64>,
    pub rolling: bool,
    roll_prev: (bool, f64, f64),
    /// filtro del canale (0 = passa-basso al massimo, 0,5 = neutro, 1 = passa-alto al massimo)
    pub filter: f32,
    cfilt: crate::fx::ColorFilter,
    /// loop: estremi in frame del brano, attivo, e lunghezza scelta in battiti
    pub loop_in: f64,
    pub loop_out: f64,
    pub loop_on: bool,
    pub loop_beats: f64,
    /// stem: audio alternativo (somma degli stem scelti) letto al posto del brano; maschera bit0 batteria, bit1 basso, bit2 altro, bit3 voce
    view: Option<Arc<Vec<f32>>>,
    pub stem_mask: u8,
    /// volume di ogni stem (batteria, basso, altro, voce): 1 per tutti = brano intero
    pub stem_gains: [f32; 4],
    /// cambio di stem: l'audio precedente sfuma nel nuovo (dissolvenza incrociata sull'ingresso, senza fermare nulla)
    prev_view: Option<Arc<Vec<f32>>>,
    xf: f32,
    /// loudness (LUFS) e picco del brano caricato, e guadagno di normalizzazione applicato (dB)
    pub loudness: f64,
    pub peak: f32,
    pub norm_db: f64,
    /// scrub stile vinile: posizione (frame) verso cui la puntina viene trascinata mentre il deck e' tenuto
    scrub_target: Option<f64>,
    /// velocita' attuale della puntina (frame per frame): cambia dolcemente, mai a scatti da un blocco all'altro
    scrub_v: f64,
    pub level: f32,
    /// livello (picco) sinistro e destro di cio' che il canale manda al master, dopo il fader: i meter dei canali
    pub lvl: [f32; 2],
    gain_prev: f32,
}

impl Deck {
    pub fn new(sr: u32, xf_assign: u8) -> Self {
        Deck {
            sr, track: None, pos: 0.0, playing: false, preview: false, tempo: 0.0, tempo_ramp: None, sync: false, sync_prev_playing: false, armed: false, arm_pos: 0.0, range: 0.08,
            keylock: true, key_shift: 0, cue_point: 0.0, hot_cues: [None; 8], quantize: true,
            grid_orig: (vec![], 0, 0.0), grid_base: vec![], grid_down: 0, grid_bpm: 0.0, grid_mode: 0, grid_labels: vec![], grid_pins: vec![],
            volume: 0.85, xf_assign, eq: Eq3::new(sr as f32), eq_knobs: [0.5; 3], fx: Fx::new(sr as f32),
            stretch: Stretch::preset_default(2, sr), stretch_mode: false, need_preroll: true,
            last_transpose: 1e9, in_buf: Vec::new(), lead: 0.0, held: false, pfl: false, pfl_buf: Vec::new(), mod_rate: 1.0, nudge: 0.0, motion: Motion::None, brake_dur: 1.5, slip: true, slip_sec: None, rolling: false, roll_prev: (false, 0.0, 0.0), filter: 0.5, cfilt: crate::fx::ColorFilter::new(), loop_in: 0.0, loop_out: 0.0, loop_on: false, loop_beats: 4.0, view: None, stem_mask: 15, stem_gains: [1.0; 4], prev_view: None, xf: 0.0, loudness: 0.0, peak: 1.0, norm_db: 0.0, scrub_target: None, scrub_v: 0.0, level: 0.0, gain_prev: 0.0, lvl: [0.0; 2],
        }
    }

    /// Svuota il deck: ferma la riproduzione e toglie il brano (all'inizio di un nuovo caricamento).
    pub fn unload(&mut self) {
        self.track = None;
        self.reset_motion();
        self.stem_gains = [1.0; 4];
        self.loop_on = false;
        self.loop_in = 0.0;
        self.loop_out = 0.0;
        self.view = None;
        self.stem_mask = 15;
        self.playing = false;
        self.preview = false;
        self.pos = 0.0;
        self.sync_prev_playing = false;
        self.scrub_target = None;
        self.held = false;
        self.grid_orig = (vec![], 0, 0.0);
        self.grid_base.clear();
        self.grid_pins.clear();
        self.cue_point = 0.0;
        self.hot_cues = [None; 8];
        self.loudness = 0.0;
        self.peak = 1.0;
        self.norm_db = 0.0;
        self.lead = 0.0;
        self.need_preroll = true;
        self.stretch_mode = false;
    }

    pub fn load(&mut self, t: Arc<Track>) {
        self.tempo_ramp = None;
        self.grid_orig = (t.beats.clone(), t.downbeat, t.bpm);
        self.grid_base = t.beats.clone();
        self.grid_down = t.downbeat;
        self.grid_bpm = t.bpm;
        self.grid_mode = 0;
        self.grid_pins.clear();
        self.sync_prev_playing = false;
        self.loudness = 0.0;
        self.peak = 1.0;
        self.norm_db = 0.0;
        self.track = Some(t);
        self.reset_motion();
        self.stem_gains = [1.0; 4];
        self.loop_on = false;
        self.loop_in = 0.0;
        self.loop_out = 0.0;
        self.view = None;
        self.stem_mask = 15;
        self.pos = 0.0;
        self.playing = false;
        self.preview = false;
        self.cue_point = 0.0;
        self.hot_cues = [None; 8];
        self.key_shift = 0;
        self.lead = 0.0;
        self.stretch_mode = false;
        self.need_preroll = true;
        self.rebuild_grid();
    }

    /// Imposta gli stem ascoltati: `view` = audio gia' mescolato (None = brano intero). Il suono entra con una breve rampa.
    pub fn set_stems(&mut self, gains: [f32; 4], view: Option<Arc<Vec<f32>>>) {
        self.stem_gains = gains;
        self.stem_mask = (0..4).fold(0u8, |m, s| if gains[s] > 0.001 { m | (1 << s) } else { m });
        self.set_view(view, true);
    }

    /// gli stem sono modificati rispetto al brano intero?
    pub fn stems_active(&self) -> bool {
        self.stem_gains.iter().any(|g| *g < 0.999)
    }

    /// Cambia l'audio letto al posto del brano. `fade`: rampa breve in entrata (cambio di stem). Senza `fade` lo scambio
    /// e' silenzioso: serve quando la separazione in corso aggiunge segmenti pronti piu' avanti nel brano.
    pub fn set_view(&mut self, view: Option<Arc<Vec<f32>>>, fade: bool) {
        if fade {
            self.prev_view = self.view.clone().or_else(|| self.track.as_ref().map(|t| t.data.clone()));
            self.xf = 0.0;
        }
        self.view = view;
    }

    pub fn rate(&self) -> f64 {
        (1.0 + self.tempo).clamp(0.05, 2.0)
    }
    /// posizione effettivamente ascoltata (gia' compensata della latenza dello stretcher)
    pub fn pos_sec(&self) -> f64 {
        (self.pos - self.lead) / self.sr as f64
    }
    fn use_stretch(&self) -> bool {
        self.keylock || self.key_shift != 0
    }
    /// ritardo dello stretcher in frame di input: (in/r + out) in tempo di uscita, per r
    fn target_lead(&self) -> f64 {
        if self.use_stretch() {
            self.stretch.input_latency() as f64 + self.stretch.output_latency() as f64 * self.rate()
        } else {
            0.0
        }
    }
    pub fn set_pos_sec(&mut self, s: f64) {
        if let Some(t) = &self.track {
            let heard = (s * self.sr as f64).clamp(0.0, t.frames() as f64 - 1.0);
            self.lead = self.target_lead();
            self.pos = heard + self.lead;
            self.need_preroll = true;
        }
    }
    /// BPM effettivo in uscita
    pub fn bpm_now(&self) -> f64 {
        self.track.as_ref().map_or(0.0, |t| t.local_bpm(self.pos_sec()) * self.rate())
    }
    /// posizione corrente in battiti (per gli effetti a tempo)
    fn beat_pos_now(&self) -> f64 {
        let t = self.pos_sec();
        self.track.as_ref().and_then(|tr| tr.beat_pos(t)).unwrap_or_else(|| t * self.bpm_now().max(1.0) / 60.0 / self.rate().max(0.05))
    }
    pub fn beat_len(&self) -> f32 {
        let b = self.bpm_now();
        if b > 0.0 { (60.0 / b) as f32 } else { 0.5 }
    }

    /// durante lo scrub: sposta la "puntina" verso `sec`; None a fine scrub
    pub fn scrub_to(&mut self, sec: Option<f64>) {
        self.scrub_target = sec.map(|s| s * self.sr as f64);
        if sec.is_none() {
            self.scrub_v = 0.0;
        }
    }
    pub fn set_tempo_pct(&mut self, pct: f64) {
        self.tempo_ramp = None; // muovere il pitch a mano interrompe il rientro automatico
        self.tempo = (pct / 100.0).clamp(-self.range, self.range).max(-0.95);
    }
    /// Avvia il rientro graduale del pitch verso `target_pct` (di solito 0): il tempo si avvicina al bersaglio a velocita'
    /// costante (`TEMPO_RAMP_PCT_PER_S`), quindi un pitch piccolo rientra presto e uno grande piu' lentamente, senza scatti udibili.
    pub fn start_tempo_ramp(&mut self, target_pct: f64) {
        self.tempo_ramp = Some((target_pct / 100.0).clamp(-self.range, self.range).max(-0.95));
    }
    pub fn cancel_tempo_ramp(&mut self) {
        self.tempo_ramp = None;
    }
    /// Un passo del rientro: da chiamare a ogni blocco audio con la durata del blocco in secondi.
    pub fn step_tempo_ramp(&mut self, secs: f64) {
        if let Some(target) = self.tempo_ramp {
            let max = TEMPO_RAMP_PCT_PER_S / 100.0 * secs;
            let d = target - self.tempo;
            if d.abs() <= max {
                self.tempo = target;
                self.tempo_ramp = None;
            } else {
                self.tempo += d.signum() * max;
            }
        }
    }
    pub fn set_range_pct(&mut self, pct: f64) {
        self.range = pct / 100.0;
        self.tempo = self.tempo.clamp(-self.range, self.range);
    }

    /// semitoni totali percepiti rispetto all'originale
    pub fn effective_semitones(&self) -> i32 {
        let vari = if self.keylock { 0.0 } else { 12.0 * self.rate().log2() };
        self.key_shift + vari.round() as i32
    }
    pub fn key_now(&self) -> Option<Key> {
        self.track.as_ref()?.key.map(|k| camelot::transpose(k, self.effective_semitones()))
    }
    pub fn key_wheel(&mut self, delta: i32) {
        let Some(base) = self.track.as_ref().and_then(|t| t.key) else { return };
        let cur = camelot::camelot_number(camelot::transpose(base, self.effective_semitones())) as i32;
        let target = ((cur - 1 + delta).rem_euclid(12) + 1) as u8;
        let vari = if self.keylock { 0 } else { (12.0 * self.rate().log2()).round() as i32 };
        let k0 = camelot::transpose(base, vari);
        self.key_shift = camelot::shift_to(k0, target);
    }

    // ---- trasporto
    pub fn play_toggle(&mut self) {
        if self.track.is_none() { return; }
        if self.armed {
            self.armed = false;
            return;
        }
        self.reset_motion();
        if self.preview { self.preview = false; return; }
        self.playing = !self.playing;
        self.need_preroll = true;
    }
    /// Play sul prossimo battito del deck guida (partenza quantizzata): usato da AUTOMIX per far partire il brano sull'«uno» della battuta.
    /// Il play normale (`play_toggle`) parte subito. Senza guida in riproduzione o senza SYNC parte subito anche questo.
    pub fn play_quantized(&mut self) {
        if self.track.is_none() { return; }
        if self.armed {
            self.armed = false;
            return;
        }
        self.reset_motion();
        if self.preview || self.playing { return; }
        self.armed = true;
        self.arm_pos = self.track.as_ref().and_then(|t| t.nearest_beat(self.pos_sec())).unwrap_or(self.pos_sec());
        self.set_pos_sec(self.arm_pos);
        self.sync_prev_playing = false;
        self.need_preroll = true;
    }
    pub fn cue_press(&mut self) {
        if self.track.is_none() { return; }
        self.armed = false;
        if self.playing && !self.preview {
            self.playing = false;
            self.set_pos_sec(self.cue_point);
        } else if (self.pos_sec() - self.cue_point).abs() < 0.03 {
            self.playing = true;
            self.preview = true;
            self.need_preroll = true;
        } else {
            self.cue_point = self.snap(self.pos_sec());
            self.set_pos_sec(self.cue_point);
        }
    }
    pub fn cue_release(&mut self) {
        if self.preview {
            self.preview = false;
            self.playing = false;
            self.set_pos_sec(self.cue_point);
        }
    }
    fn snap(&self, t: f64) -> f64 {
        if self.quantize {
            self.track.as_ref().and_then(|tr| tr.nearest_beat(t)).unwrap_or(t)
        } else {
            t
        }
    }
    pub fn hot_cue_press(&mut self, i: usize) {
        if self.track.is_none() || i >= 8 { return; }
        match self.hot_cues[i] {
            None => self.hot_cues[i] = Some(self.snap(self.pos_sec())),
            Some(t) => {
                self.set_pos_sec(t);
                self.playing = true;
            }
        }
    }
    /// durata di un battito nel brano (secondi di brano, indipendente dal pitch)
    fn track_beat(&self) -> f64 {
        let b = self.track.as_ref().map_or(0.0, |t| t.local_bpm(self.pos_sec()));
        if b > 0.0 { 60.0 / b } else { 0.5 }
    }
    /// loop automatico da qui per `loop_beats` battiti (partendo dal battito piu' vicino se QUANTIZE e' acceso)
    pub fn loop_auto(&mut self) {
        if self.track.is_none() { return; }
        let start = self.snap(self.pos_sec());
        self.loop_in = start * self.sr as f64;
        self.loop_out = self.loop_end_sec(start, self.loop_beats) * self.sr as f64;
        self.loop_on = true;
    }
    /// fine (secondi) di un loop che parte a `start` e dura `beats` battiti, misurata sulla griglia (segue anche i cambi di tempo)
    fn loop_end_sec(&self, start: f64, beats: f64) -> f64 {
        self.track.as_ref().and_then(|t| t.beat_pos(start).and_then(|b| t.time_at_beat_pos(b + beats))).unwrap_or(start + beats * self.track_beat())
    }
    /// lunghezza in battiti fra due istanti, misurata sulla griglia e portata a un valore "pulito" (1/8, 1/4, 1/2, 1, 2, 4... o un intero)
    fn loop_len_beats(&self, a: f64, b: f64) -> f64 {
        let raw = match self.track.as_ref().and_then(|t| Some((t.beat_pos(a)?, t.beat_pos(b)?))) {
            Some((x, y)) => y - x,
            None => (b - a) / self.track_beat(),
        };
        snap_beats(raw).max(0.03125)
    }
    /// interruttore del loop: se non ce n'e' uno definito ne crea uno automatico
    pub fn loop_toggle(&mut self) {
        if self.track.is_none() { return; }
        if self.loop_on {
            self.loop_on = false;
        } else if self.loop_out > self.loop_in + 64.0 && self.pos_sec() * (self.sr as f64) <= self.loop_out {
            self.loop_on = true;
        } else {
            self.loop_auto();
        }
    }
    pub fn loop_in_here(&mut self) {
        if self.track.is_none() { return; }
        self.loop_in = self.snap(self.pos_sec()) * self.sr as f64;
        if self.loop_out <= self.loop_in + 64.0 {
            self.loop_out = self.loop_end_sec(self.loop_in / self.sr as f64, self.loop_beats) * self.sr as f64;
            self.loop_on = false;
        }
    }
    /// OUT: chiude il loop che si sta definendo. Con un loop gia' attivo la testina gira dentro il loop, e prenderla come nuova fine
    /// lo farebbe collassare su se stesso (4 battiti -> 1, senza piu' modo di allungarlo): come sui CDJ, OUT in loop ne esce
    pub fn loop_out_here(&mut self) {
        if self.track.is_none() { return; }
        if self.loop_on {
            self.loop_on = false;
            return;
        }
        let o = self.snap(self.pos_sec()) * self.sr as f64;
        if o > self.loop_in + 64.0 {
            self.loop_out = o;
            self.loop_on = true;
            self.loop_beats = self.loop_len_beats(self.loop_in / self.sr as f64, o / self.sr as f64);
        }
    }
    /// cambia la lunghezza del loop (battiti); se e' attivo lo accorcia o allunga restando sullo stesso inizio
    pub fn loop_size(&mut self, beats: f64) {
        self.loop_beats = beats.clamp(0.03125, 64.0);
        if self.loop_on {
            self.loop_out = self.loop_end_sec(self.loop_in / self.sr as f64, self.loop_beats) * self.sr as f64;
        }
    }

    /// dimezza (`up` = false) o raddoppia il loop. Una lunghezza segnata a mano (3, 5, 6 battiti...) va alla potenza di due piu' vicina
    /// nel verso scelto, invece di dare 1,5 o 0,75: le lunghezze restano sempre 1/32 ... 64
    pub fn loop_step(&mut self, up: bool) {
        let b = self.loop_beats.max(0.03125);
        let e = b.log2();
        let next = if (e - e.round()).abs() < 1e-6 { if up { b * 2.0 } else { b / 2.0 } } else if up { 2f64.powf(e.ceil()) } else { 2f64.powf(e.floor()) };
        self.loop_size(next);
    }

    // ---- movimenti sul brano

    fn reset_motion(&mut self) {
        self.mod_rate = 1.0;
        self.motion = Motion::None;
        self.slip_sec = None;
        if self.rolling {
            self.rolling = false;
            (self.loop_on, self.loop_in, self.loop_out) = self.roll_prev;
        }
    }
    fn slip_begin(&mut self) {
        if self.slip_sec.is_none() {
            self.slip_sec = Some(self.pos_sec());
        }
    }
    fn slip_end_if_idle(&mut self) {
        if !self.rolling && self.motion == Motion::None {
            self.slip_sec = None;
        }
    }
    /// Beatjump: salta di `beats` battiti (anche negativi) restando in fase con la griglia. Un loop attivo si sposta con la testina.
    pub fn beatjump(&mut self, beats: f64) {
        let Some(t) = self.track.clone() else { return };
        let now = self.pos_sec();
        let target = t.beat_pos(now).and_then(|b| t.time_at_beat_pos(b + beats)).unwrap_or(now + beats * 60.0 / t.bpm.max(1.0));
        let target = target.clamp(0.0, (t.duration() - 0.05).max(0.0));
        let delta = (target - now) * self.sr as f64;
        if self.loop_on {
            self.loop_in += delta;
            self.loop_out += delta;
        }
        if let Some(s) = self.slip_sec.as_mut() {
            *s += target - now;
        }
        self.set_pos_sec(target);
    }
    /// Loop roll: finche' e' tenuto, ripete `beats` battiti (partendo dalla griglia di quella lunghezza se QUANTIZE e' acceso).
    /// Con SLIP al rilascio il brano riprende da dove sarebbe arrivato senza il loop.
    pub fn roll_on(&mut self, beats: f64) {
        let Some(t) = self.track.clone() else { return };
        if beats <= 0.0 { return; }
        if !self.rolling {
            self.roll_prev = (self.loop_on, self.loop_in, self.loop_out);
            self.rolling = true;
            self.slip_begin();
        }
        let now = self.pos_sec();
        let (start, end) = match t.beat_pos(now) {
            Some(b) => {
                let b0 = if self.quantize { (b / beats).floor() * beats } else { b };
                (t.time_at_beat_pos(b0).unwrap_or(now), t.time_at_beat_pos(b0 + beats).unwrap_or(now + beats * 0.5))
            }
            None => (now, now + beats * 60.0 / t.bpm.max(1.0)),
        };
        self.loop_in = start * self.sr as f64;
        self.loop_out = end * self.sr as f64;
        self.loop_on = true;
    }
    pub fn roll_off(&mut self) {
        if !self.rolling { return; }
        self.rolling = false;
        (self.loop_on, self.loop_in, self.loop_out) = self.roll_prev;
        if self.slip {
            if let Some(s) = self.slip_sec { self.set_pos_sec(s); }
        }
        self.slip_end_if_idle();
    }
    /// Freno: il disco rallenta fino a fermarsi in `dur` secondi (la tonalita' scende come su un vinile). Resta fermo finche' non rilasci.
    pub fn brake_on(&mut self, dur: f64) {
        if self.track.is_none() || !self.playing { return; }
        self.slip_begin();
        self.brake_dur = dur.clamp(0.1, 6.0);
        self.motion = Motion::Brake;
    }
    /// Spinback: il disco gira all'indietro veloce e rallenta fino a fermarsi in `dur` secondi, poi il brano riparte.
    pub fn spin_on(&mut self, dur: f64) {
        if self.track.is_none() || !self.playing { return; }
        self.slip_begin();
        self.motion = Motion::Spin(0.0, dur.clamp(0.3, 6.0));
    }
    /// Reverse: finche' e' tenuto il brano va al contrario.
    pub fn reverse_on(&mut self) {
        if self.track.is_none() || !self.playing { return; }
        self.slip_begin();
        self.motion = Motion::Reverse;
    }
    /// Rilascio di freno / reverse: il disco riparte (con SLIP dal punto dove sarebbe arrivato).
    pub fn motion_off(&mut self) {
        if matches!(self.motion, Motion::Brake | Motion::Reverse) {
            self.resume_motion();
        }
    }
    fn resume_motion(&mut self) {
        if self.slip {
            if let Some(s) = self.slip_sec {
                self.set_pos_sec(s);
                self.mod_rate = 0.0;
            }
        }
        self.motion = Motion::Resume;
    }
    /// Un passo di movimento per ogni blocco audio di `n` frame.
    fn motion_step(&mut self, n: usize) {
        let dt = n as f64 / self.sr as f64;
        let base = self.rate();
        if let Some(s) = self.slip_sec.as_mut() {
            *s += dt * base;
        }
        match self.motion {
            Motion::None => {}
            Motion::Brake => self.mod_rate = (self.mod_rate - dt / self.brake_dur).max(0.0),
            Motion::Reverse => self.mod_rate = (self.mod_rate - dt * 25.0).max(-1.0),
            Motion::Spin(t, dur) => {
                let t = t + dt;
                if t >= dur {
                    self.resume_motion();
                } else {
                    let k = 1.0 - t / dur;
                    self.mod_rate = -4.0 * k * k;
                    self.motion = Motion::Spin(t, dur);
                }
            }
            Motion::Resume => {
                self.mod_rate += dt / 0.12;
                if self.mod_rate >= 1.0 {
                    self.mod_rate = 1.0;
                    self.motion = Motion::None;
                    self.slip_end_if_idle();
                }
            }
        }
    }

    pub fn hot_cue_clear(&mut self, i: usize) {
        if i < 8 { self.hot_cues[i] = None; }
    }

    /// Allinea tempo e fase al deck master
    pub fn sync_to(&mut self, master_bpm: f64, master_phase: Option<f64>) {
        let Some(t) = self.track.clone() else { return };
        let mine = t.local_bpm(self.pos_sec());
        if mine <= 0.0 || master_bpm <= 0.0 { return; }
        let mut target = master_bpm;
        // evita errori di ottava (es. 70 vs 140 bpm)
        while target / mine > 1.5 { target /= 2.0; }
        while target / mine < 0.67 { target *= 2.0; }
        let want = target / mine - 1.0;
        if want.abs() > self.range {
            self.range = [0.04, 0.08, 0.16, 0.5, 1.0].into_iter().find(|&r| r >= want.abs()).unwrap_or(1.0);
        }
        self.tempo = want.clamp(-self.range, self.range);
        if let (Some(ph), true) = (master_phase, self.playing) {
            if let Some(nt) = t.time_at_phase(self.pos_sec(), ph) {
                self.set_pos_sec(nt);
            }
        }
    }

    // ---- griglia: modifica manuale
    /// ricostruisce la traccia con la griglia della modalita' corrente (Auto / Fisso / Flessibile, vedi gridfit.rs).
    /// Una griglia con battiti fissati a mano resta com'e' (salvo in modalita' Fisso, che la riduce a un solo tempo).
    fn rebuild_grid(&mut self) {
        let Some(t) = self.track.clone() else { return };
        let mode = crate::gridfit::GridMode::from_id(self.grid_mode);
        let manual = !self.grid_pins.is_empty() && mode != crate::gridfit::GridMode::Fixed;
        let fitted = if manual { None } else { crate::gridfit::fit(&self.grid_base, self.grid_down, mode) };
        match fitted {
            Some(f) => {
                self.grid_labels = crate::gridfit::tempo_labels(&f);
                self.track = Some(Arc::new(t.with_grid(f.beats, f.downbeat, f.bpm)));
            }
            None => {
                self.grid_labels.clear();
                self.track = Some(Arc::new(t.with_grid(self.grid_base.clone(), self.grid_down, self.grid_bpm)));
            }
        }
    }
    pub fn grid_pins(&self) -> &[usize] {
        &self.grid_pins
    }
    /// griglia calcolata automaticamente al caricamento (prima delle correzioni manuali)
    pub fn grid_auto(&self) -> &[f64] {
        &self.grid_orig.0
    }
    pub fn grid_info(&self) -> Option<GridInfo> {
        let t = self.track.as_ref()?;
        Some(GridInfo { beats: t.beats.clone(), base: self.grid_base.clone(), base_downbeat: self.grid_down, downbeat: t.downbeat, bpm: t.bpm, mode: self.grid_mode, labels: self.grid_labels.clone() })
    }
    pub fn grid_apply_saved(&mut self, beats: Vec<f64>, downbeat: usize, bpm: f64, mode: u8, pins: Vec<usize>) {
        if beats.len() < 2 {
            return;
        }
        self.grid_base = beats;
        self.grid_down = downbeat % 4;
        self.grid_bpm = bpm;
        self.grid_mode = mode.min(2);
        self.grid_pins = pins.into_iter().filter(|&p| p < self.grid_base.len()).collect();
        self.rebuild_grid();
    }
    /// battiti nuovi (rete neurale) per un brano la cui griglia non e' stata toccata a mano: sostituiscono quelli di lavoro e
    /// quelli del reset, mantenendo la modalita'
    pub fn grid_refine(&mut self, beats: Vec<f64>, downbeat: usize, bpm: f64) {
        if beats.len() < 2 || self.track.is_none() {
            return;
        }
        self.grid_orig = (beats.clone(), downbeat % 4, bpm);
        self.grid_base = beats;
        self.grid_down = downbeat % 4;
        self.grid_bpm = bpm;
        self.grid_pins.clear();
        self.rebuild_grid();
    }
    /// comandi di modifica della griglia; ritorna true se la griglia e' cambiata
    pub fn grid_cmd(&mut self, cmd: &str, a: f64) -> bool {
        if self.track.is_none() || self.grid_base.len() < 2 {
            return false;
        }
        match cmd {
            "halve" => {
                self.grid_base = self.grid_base.iter().step_by(2).copied().collect();
                self.grid_down = (self.grid_down / 2) % 4;
                self.grid_bpm /= 2.0;
                self.grid_pins.clear();
            }
            "double" => {
                let mut v = Vec::with_capacity(self.grid_base.len() * 2);
                for w in self.grid_base.windows(2) {
                    v.push(w[0]);
                    v.push(0.5 * (w[0] + w[1]));
                }
                v.push(*self.grid_base.last().unwrap());
                self.grid_base = v;
                self.grid_down = (self.grid_down * 2) % 4;
                self.grid_bpm *= 2.0;
                self.grid_pins.clear();
            }
            "mode" => self.grid_mode = (a as i64).clamp(0, 2) as u8,
            "reset" => {
                self.grid_base = self.grid_orig.0.clone();
                self.grid_down = self.grid_orig.1;
                self.grid_bpm = self.grid_orig.2;
                self.grid_mode = 0;
                self.grid_pins.clear();
            }
            // il battito piu' vicino alla testina viene portato esattamente sulla testina
            "anchor_here" => {
                let t = self.pos_sec();
                let n = self.grid_base.len();
                let i = self.grid_base.partition_point(|&b| b <= t).saturating_sub(1);
                let j = if i + 1 < n && (self.grid_base[i + 1] - t).abs() < (t - self.grid_base[i]).abs() { i + 1 } else { i };
                let d = t - self.grid_base[j];
                let prev = self.grid_pins.iter().copied().filter(|&p| p < j).max();
                let next = self.grid_pins.iter().copied().filter(|&p| p > j).min();
                // prima: la correzione svanisce verso il vincolo precedente (o dopo 3 battiti); dopo: verso il successivo (o resta)
                let back = prev.map_or(3, |p| j - p);
                for k in j.saturating_sub(back)..j {
                    self.grid_base[k] += d * (k + back - j) as f64 / back as f64;
                }
                for k in j..n {
                    let w = match next {
                        Some(q) if k >= q => 0.0,
                        Some(q) => (q - k) as f64 / (q - j) as f64,
                        None => 1.0,
                    };
                    self.grid_base[k] += d * w;
                }
                if !self.grid_pins.contains(&j) {
                    self.grid_pins.push(j);
                }
            }
            _ => return false,
        }
        self.rebuild_grid();
        true
    }

    pub fn render(&mut self, out: &mut [f32], target_gain: f32) {
        out.iter_mut().for_each(|v| *v = 0.0);
        if self.pfl {
            self.pfl_buf.clear();
            self.pfl_buf.resize(out.len(), 0.0);
        }
        self.level *= 0.9;
        // i picchi scendono nel tempo (meta' ogni 80 ms) e non a ogni blocco: con buffer piccoli (ASIO) non si perderebbero fra una lettura e l'altra
        let fall = 0.5f32.powf(out.len() as f32 / 2.0 / self.sr.max(1) as f32 / 0.08);
        self.lvl[0] *= fall;
        self.lvl[1] *= fall;
        let Some(track) = self.track.clone() else { return };
        let data: Arc<Vec<f32>> = self.view.clone().unwrap_or_else(|| track.data.clone());
        let prev = self.prev_view.clone();
        let xf_len = (self.sr as f32 * 0.09).max(1.0);
        let xf0 = self.xf;
        let lp_on = self.loop_on && self.loop_out > self.loop_in + 64.0;
        let (lp_in, lp_out) = (self.loop_in, self.loop_out);
        let lp_len = (lp_out - lp_in).max(1.0);
        let scrubbing = self.held && self.scrub_target.is_some();
        if (!self.playing || self.held) && !scrubbing {
            return;
        }
        let n = out.len() / 2;
        let frames = track.frames();
        if !scrubbing { self.motion_step(n); }
        let r = self.rate() * self.mod_rate * (1.0 + self.nudge);
        let use_stretch = self.use_stretch() && !scrubbing && self.motion == Motion::None;

        if scrubbing {
            // il suono segue il mouse come un disco: la velocita' e' proporzionale alla distanza che resta da percorrere
            // (anche all'indietro); a mouse fermo la puntina si ferma e l'audio si spegne
            if self.stretch_mode {
                self.pos -= self.lead;
                self.lead = 0.0;
                self.stretch_mode = false;
            }
            self.need_preroll = true;
            let tgt = self.scrub_target.unwrap_or(self.pos).clamp(0.0, frames as f64 - 1.0);
            // la velocita' desiderata e' proporzionale alla distanza che resta (anche all'indietro) e la velocita' vera la insegue con un
            // piccolo ritardo, campione per campione: cosi' non ci sono scalini di velocita' ai bordi dei blocchi (quelli si sentono come un ronzio)
            let sr = self.sr as f64;
            // un salto lungo (clic sulla panoramica) non si percorre: la puntina si posa subito nel punto indicato
            if (tgt - self.pos).abs() > 2.0 * sr {
                self.pos = tgt;
                self.scrub_v = 0.0;
            }
            let (chase, slew) = (1.0 / (0.02 * sr), 1.0 / (0.006 * sr));
            let mut v = self.scrub_v;
            for i in 0..n {
                let want = ((tgt - self.pos) * chase).clamp(-5.0, 5.0);
                v += (want - v) * slew;
                let ip = self.pos.floor() as isize;
                let t = (self.pos - ip as f64) as f32;
                let g = (v.abs() as f32 * 3.0).min(1.0);
                if ip >= 0 && (ip as usize) < frames {
                    out[i * 2] = cubic(&data, ip, t, 0, frames) * g;
                    out[i * 2 + 1] = cubic(&data, ip, t, 1, frames) * g;
                }
                self.pos = (self.pos + v).clamp(0.0, frames as f64 - 1.0);
            }
            self.scrub_v = v;
        } else if use_stretch {
            let transpose = if self.keylock { self.key_shift as f32 } else { self.key_shift as f32 + 12.0 * (r as f32).log2() };
            if (transpose - self.last_transpose).abs() > 1e-4 {
                self.stretch.set_transpose_factor_semitones(transpose, None);
                self.last_transpose = transpose;
            }
            let lead = self.target_lead();
            if !self.stretch_mode || self.need_preroll {
                // cambio modalita' o salto: mantieni la posizione ascoltata e ricarica lo stretcher
                let heard = self.pos - self.lead;
                self.lead = lead;
                self.pos = heard + lead;
                self.stretch.reset();
                let p = self.pos.floor() as isize;
                // preroll: lo stretcher gira su audio che precede il punto di ripartenza e se ne scarta l'uscita, cosi' anche i buffer di
                // sintesi sono a regime. Con il solo `seek` l'uscita partiva con la rampa d'ingresso delle finestre e l'attacco
                // (il kick di un hot cue) usciva smussato e a un terzo dell'ampiezza.
                let n_pre = lead.ceil() as usize + 512;
                let q0 = p - (n_pre as f64 * r).round() as isize;
                let pre_in_len = (p - q0).max(0) as usize;
                let mut pre_in = vec![0f32; pre_in_len * 2];
                for k in 0..pre_in_len {
                    let f = q0 + k as isize;
                    if f >= 0 && (f as usize) < frames {
                        pre_in[k * 2] = data[f as usize * 2];
                        pre_in[k * 2 + 1] = data[f as usize * 2 + 1];
                    }
                }
                let mut pre_out = vec![0f32; n_pre * 2];
                self.stretch.process(&pre_in, &mut pre_out);
                self.stretch_mode = true;
                self.need_preroll = false;
            } else {
                self.lead = lead;
            }
            let start = self.pos.floor() as isize;
            let end = (self.pos + n as f64 * r).floor() as isize;
            let in_len = (end - start).max(0) as usize;
            self.in_buf.clear();
            self.in_buf.resize(in_len * 2, 0.0);
            for k in 0..in_len {
                let mut f = start + k as isize;
                if lp_on && f as f64 >= lp_out {
                    f = (lp_in + (f as f64 - lp_in) % lp_len).floor() as isize;
                }
                if f >= 0 && (f as usize) < frames {
                    let (mut l, mut rr) = (data[f as usize * 2], data[f as usize * 2 + 1]);
                    if let Some(p) = prev.as_ref().filter(|p| (f as usize) * 2 + 1 < p.len()) {
                        let g = (xf0 + (k as f32 / in_len.max(1) as f32) * (n as f32 / xf_len)).min(1.0);
                        l = p[f as usize * 2] + (l - p[f as usize * 2]) * g;
                        rr = p[f as usize * 2 + 1] + (rr - p[f as usize * 2 + 1]) * g;
                    }
                    self.in_buf[k * 2] = l;
                    self.in_buf[k * 2 + 1] = rr;
                }
            }
            self.stretch.process(&self.in_buf, &mut *out);
            self.pos += n as f64 * r;
            if lp_on && self.pos >= lp_out {
                self.pos = lp_in + (self.pos - lp_in) % lp_len;
            }
        } else {
            if self.stretch_mode {
                // uscita dalla modalita' stretch: mantieni la posizione ascoltata
                self.pos -= self.lead;
                self.lead = 0.0;
                self.stretch_mode = false;
            }
            self.need_preroll = false;
            for i in 0..n {
                let ip = self.pos.floor() as isize;
                let t = (self.pos - ip as f64) as f32;
                if ip >= 0 && (ip as usize) < frames {
                    let (mut l, mut rr) = (cubic(&data, ip, t, 0, frames), cubic(&data, ip, t, 1, frames));
                    if let Some(p) = prev.as_ref().filter(|p| p.len() / 2 >= frames) {
                        let g = (xf0 + i as f32 / xf_len).min(1.0);
                        l = cubic(p, ip, t, 0, frames) + (l - cubic(p, ip, t, 0, frames)) * g;
                        rr = cubic(p, ip, t, 1, frames) + (rr - cubic(p, ip, t, 1, frames)) * g;
                    }
                    out[i * 2] = l;
                    out[i * 2 + 1] = rr;
                }
                self.pos += r;
                if lp_on && self.pos >= lp_out {
                    self.pos -= lp_len;
                }
                if self.pos < 0.0 {
                    self.pos = 0.0;
                }
            }
        }
        if !scrubbing && self.pos - self.lead >= frames as f64 - 1.0 {
            self.pos = frames as f64 - 1.0 + self.lead;
            self.playing = false;
        }

        if prev.is_some() {
            self.xf += n as f32 / xf_len;
            if self.xf >= 1.0 {
                self.prev_view = None;
                self.xf = 0.0;
            }
        }

        self.eq.set(self.eq_knobs);
        self.eq.process(out);
        self.cfilt.process(out, self.sr as f32, self.filter, true);
        let bl = self.beat_len();
        let bp = self.beat_pos_now();
        self.fx.process(out, bl, bp);

        // preascolto: segnale dopo EQ ed effetti e dopo la normalizzazione, prima del fader di canale e del crossfader
        if self.pfl {
            let ng = 10f32.powf(self.norm_db as f32 / 20.0);
            for (p, o) in self.pfl_buf.iter_mut().zip(out.iter()) {
                *p = *o * ng;
            }
        }
        // volume con rampa per evitare zipper noise
        let g0 = self.gain_prev;
        let step = (target_gain - g0) / n as f32;
        let mut peak = 0f32;
        let (mut pl, mut pr) = (0f32, 0f32);
        for i in 0..n {
            let g = g0 + step * i as f32;
            out[i * 2] *= g;
            out[i * 2 + 1] *= g;
            pl = pl.max(out[i * 2].abs());
            pr = pr.max(out[i * 2 + 1].abs());
        }
        peak = peak.max(pl).max(pr);
        self.gain_prev = target_gain;
        self.level = self.level.max(peak);
        // meter del canale: dopo il fader (e il crossfader), cioe' quello che questo canale manda al master
        self.lvl[0] = self.lvl[0].max(pl);
        self.lvl[1] = self.lvl[1].max(pr);
    }

    /// Ricostruisce il deck per un nuovo sample rate (cambio dispositivo audio).
    pub fn change_sr(&mut self, new_sr: u32) {
        if new_sr == self.sr {
            return;
        }
        let heard = self.pos_sec();
        self.view = None;
        self.stem_mask = 15;
        self.stem_gains = [1.0; 4];
        if let Some(t) = self.track.take() {
            let data = crate::decode::resample_stereo(&t.data, t.sr, new_sr);
            self.track = Some(Arc::new(Track { name: t.name.clone(), path: t.path.clone(), data: Arc::new(data), sr: new_sr, beats: t.beats.clone(), bpm: t.bpm, key: t.key, downbeat: t.downbeat }));
        }
        self.sr = new_sr;
        self.eq = Eq3::new(new_sr as f32);
        let mut fx = Fx::new(new_sr as f32);
        fx.kind = self.fx.kind;
        fx.on = self.fx.on;
        fx.wet = self.fx.wet;
        fx.param = self.fx.param;
        fx.param2 = self.fx.param2;
        fx.div = self.fx.div;
        fx.feel = self.fx.feel;
        self.fx = fx;
        self.cfilt = crate::fx::ColorFilter::new();
        self.loop_on = false;
        self.stretch = Stretch::preset_default(2, new_sr);
        self.stretch_mode = false;
        self.last_transpose = 1e9;
        self.lead = 0.0;
        self.pos = heard * new_sr as f64;
        self.need_preroll = true;
    }
}

// ---------------------------------------------------------------- sampler

#[derive(Default)]
pub struct Pad {
    pub name: String,
    pub data: Option<Arc<Vec<f32>>>,
    pos: f64,
    pub playing: bool,
    pub looped: bool,
    pub gain: f32,
    /// tempo del campione (0 = sconosciuto) e posizione del suo primo battito (frame)
    pub bpm: f64,
    pub first_beat: f64,
    /// in attesa del prossimo battito della guida per partire (modo sync)
    pub armed: bool,
    prev_phase: Option<f64>,
    /// in dissolvenza di uscita: tra pochi ms smette di suonare
    stopping: bool,
    env: f32,
    stretch: Option<Stretch>,
    stretch_mode: bool,
    need_preroll: bool,
    lead: f64,
    in_buf: Vec<f32>,
    tmp: Vec<f32>,
}

impl Pad {
    fn spb(&self, sr: u32) -> f64 {
        if self.bpm > 0.0 { 60.0 / self.bpm * sr as f64 } else { 0.0 }
    }
    fn begin(&mut self, pos: f64) {
        self.pos = pos;
        self.playing = true;
        self.stopping = false;
        self.armed = false;
        self.env = 0.0;
        self.need_preroll = true;
        self.stretch_mode = false;
        self.lead = 0.0;
    }
    pub fn stop(&mut self) {
        self.armed = false;
        if self.playing {
            self.stopping = true;
        }
    }

    /// Somma il pad in `out`. Con `sync_on` e una guida (bpm, fase del battito) il campione si adegua: parte sul battito,
    /// ne segue il tempo con il time-stretch (senza cambiare tonalita') e la fase con una piccola correzione di velocita',
    /// come il sync fra deck.
    fn render(&mut self, out: &mut [f32], sr: u32, vol: f32, sync_on: bool, lead_ref: Option<(f64, f64)>) {
        let Some(data) = self.data.clone() else { return };
        let total = data.len() / 2;
        if total == 0 {
            return;
        }
        let n = out.len() / 2;
        let spb = self.spb(sr);
        let have_ref = sync_on && lead_ref.map_or(false, |(b, _)| b > 0.0);
        if self.armed {
            match lead_ref {
                Some((_, ph)) if have_ref => {
                    let wrapped = self.prev_phase.map_or(false, |pp| ph < pp - 0.5);
                    self.prev_phase = Some(ph);
                    if wrapped {
                        // il battito e' appena scattato: parte dal suo primo battito, avanzato della fase gia' trascorsa
                        let start = if spb > 0.0 { self.first_beat + ph * spb } else { 0.0 };
                        self.begin(start.rem_euclid(total as f64));
                    }
                }
                _ => self.begin(0.0),
            }
        }
        if !self.playing {
            return;
        }
        let synced = have_ref && self.bpm > 0.0;
        let mut r = 1.0f64;
        if synced {
            let (mbpm, mph) = lead_ref.unwrap();
            let (mut target, mine) = (mbpm, self.bpm);
            let mut octave = false;
            while target / mine > 1.5 { target /= 2.0; octave = true; }
            while target / mine < 0.67 { target *= 2.0; octave = true; }
            r = target / mine;
            if !octave && spb > 0.0 {
                let heard = self.pos - self.lead - self.first_beat;
                let pph = (heard / spb).rem_euclid(1.0);
                let mut e = pph - mph;
                if e > 0.5 { e -= 1.0 } else if e < -0.5 { e += 1.0 }
                if e.abs() > 0.002 {
                    r *= 1.0 - (0.5 * e).clamp(-0.04, 0.04);
                }
            }
        }
        self.tmp.clear();
        self.tmp.resize(n * 2, 0.0);
        if synced {
            let st = self.stretch.get_or_insert_with(|| Stretch::preset_default(2, sr));
            let lead = st.input_latency() as f64 + st.output_latency() as f64 * r;
            if !self.stretch_mode || self.need_preroll {
                let heard = if self.stretch_mode { self.pos - self.lead } else { self.pos };
                self.lead = lead;
                self.pos = heard + lead;
                st.reset();
                let lat = st.input_latency();
                let p = self.pos.floor() as isize;
                let s = (p - lat as isize).max(0) as usize;
                let e = (p.max(0) as usize).min(total);
                if e > s {
                    st.seek(&data[s * 2..e * 2], r);
                }
                self.stretch_mode = true;
                self.need_preroll = false;
            } else {
                self.lead = lead;
            }
            let start = self.pos.floor() as isize;
            let end = (self.pos + n as f64 * r).floor() as isize;
            let in_len = (end - start).max(0) as usize;
            self.in_buf.clear();
            self.in_buf.resize(in_len * 2, 0.0);
            for k in 0..in_len {
                let mut f = start + k as isize;
                if self.looped {
                    f = f.rem_euclid(total as isize);
                }
                if f >= 0 && (f as usize) < total {
                    self.in_buf[k * 2] = data[f as usize * 2];
                    self.in_buf[k * 2 + 1] = data[f as usize * 2 + 1];
                }
            }
            st.process(&self.in_buf, &mut self.tmp[..]);
            self.pos += n as f64 * r;
            if self.looped {
                while self.pos >= total as f64 {
                    self.pos -= total as f64;
                }
            } else if self.pos - self.lead >= total as f64 {
                self.stopping = true;
            }
        } else {
            if self.stretch_mode {
                // uscita dal sync: resta la posizione ascoltata
                self.pos = (self.pos - self.lead).max(0.0);
                self.lead = 0.0;
                self.stretch_mode = false;
            }
            for i in 0..n {
                if self.pos as usize >= total {
                    if self.looped { self.pos = 0.0 } else { self.stopping = true; break; }
                }
                let ip = self.pos as usize;
                self.tmp[i * 2] = data[ip * 2];
                self.tmp[i * 2 + 1] = data[ip * 2 + 1];
                self.pos += 1.0;
            }
        }
        // entrata e uscita con una rampa di pochi ms: niente click quando parte o si ferma
        let step = 1.0 / (0.004 * sr as f32);
        let g = self.gain * vol;
        for (i, f) in out.chunks_exact_mut(2).enumerate() {
            let tgt = if self.stopping { 0.0 } else { 1.0 };
            self.env += (tgt - self.env).clamp(-step, step);
            f[0] += self.tmp[i * 2] * g * self.env;
            f[1] += self.tmp[i * 2 + 1] * g * self.env;
        }
        if self.stopping && self.env <= 0.0 {
            self.playing = false;
            self.stopping = false;
        }
    }
}


// ---------------------------------------------------------------- batteria (campioni singoli suonati dal vivo)

pub const NUM_DRUMS: usize = 16;

#[derive(Clone, Copy, Default)]
struct DVoice {
    pos: f64,
    active: bool,
    releasing: bool,
    env: f32,
    gain: f32,
}

pub struct DrumPad {
    pub name: String,
    pub path: String,
    pub data: Option<Arc<Vec<f32>>>,
    /// volume 0..1,5; intonazione in semitoni; gruppo choke (0 = nessuno: un pad del gruppo zittisce gli altri, come hi-hat chiuso/aperto)
    pub gain: f32,
    pub tune: f32,
    pub group: u8,
    /// gate: suona solo finche' il pad e' premuto (altrimenti suona per intero)
    pub gate: bool,
    /// aftertouch (pressione sul pad dopo il colpo): cosa controlla (0 niente, 1 volume, 2 intonazione, 3 entrambi), valore ricevuto 0..1 e valore smussato
    pub at_mode: u8,
    pub at: f32,
    at_s: f32,
    voices: [DVoice; 4],
}

impl Default for DrumPad {
    fn default() -> Self {
        DrumPad { name: String::new(), path: String::new(), data: None, gain: 1.0, tune: 0.0, group: 0, gate: false, at_mode: 1, at: 0.0, at_s: 0.0, voices: [DVoice::default(); 4] }
    }
}

/// quanto cambia il suono con la pressione al massimo: +60% di volume, +7 semitoni
const AT_VOL: f32 = 0.6;
const AT_SEMI: f32 = 7.0;

impl DrumPad {
    pub fn load(&mut self, name: String, path: String, data: Vec<f32>) {
        self.name = name;
        self.path = path;
        self.data = Some(Arc::new(data));
        self.voices = [DVoice::default(); 4];
    }
    pub fn clear(&mut self) {
        *self = DrumPad::default();
    }
    /// pressione 0..1 sul pad (ignorata se il pad non suona: serve a non lasciare valori vecchi per il colpo dopo)
    pub fn pressure(&mut self, p: f32) {
        self.at = if self.playing() { p.clamp(0.0, 1.0) } else { 0.0 };
    }
    pub fn playing(&self) -> bool {
        self.voices.iter().any(|v| v.active && !v.releasing)
    }
    fn release_all(&mut self) {
        for v in self.voices.iter_mut() {
            if v.active {
                v.releasing = true;
            }
        }
    }
    /// Fa partire una voce; le precedenti sfumano in pochi ms (come una drum machine: la ripetizione taglia la coda).
    fn start(&mut self, vel: f32) {
        self.release_all();
        self.at = 0.0;
        self.at_s = 0.0;
        let slot = self.voices.iter().position(|v| !v.active).unwrap_or(0);
        self.voices[slot] = DVoice { pos: 0.0, active: true, releasing: false, env: 0.0, gain: vel };
    }
    fn render(&mut self, out: &mut [f32], sr: u32, vol: f32) {
        let Some(data) = self.data.clone() else { return };
        let total = data.len() / 2;
        let rate0 = 2f64.powf(self.tune as f64 / 12.0);
        let up = 1.0 / (0.001 * sr as f32);
        let down = 1.0 / (0.004 * sr as f32);
        let g0 = self.gain * vol;
        // la pressione cambia in modo graduale (circa 10 ms) per non fare scalini nel suono
        let frames = (out.len() / 2).max(1);
        let a0 = self.at_s;
        let a1 = a0 + (self.at - a0) * (1.0 - (1.0 - 1.0 / (0.01 * sr as f32)).powi(frames as i32));
        self.at_s = a1;
        let (at_vol, at_pitch) = (self.at_mode & 1 != 0, self.at_mode & 2 != 0);
        for v in self.voices.iter_mut() {
            if !v.active {
                continue;
            }
            for (fi, f) in out.chunks_exact_mut(2).enumerate() {
                let a = a0 + (a1 - a0) * (fi as f32 / frames as f32);
                let rate = if at_pitch && a > 0.0005 { rate0 * 2f64.powf((a * AT_SEMI) as f64 / 12.0) } else { rate0 };
                let ip = v.pos as usize;
                if ip + 1 >= total {
                    v.active = false;
                    break;
                }
                let t = (v.pos - ip as f64) as f32;
                let l = data[ip * 2] + (data[ip * 2 + 2] - data[ip * 2]) * t;
                let r = data[ip * 2 + 1] + (data[ip * 2 + 3] - data[ip * 2 + 1]) * t;
                if v.releasing {
                    v.env -= down;
                    if v.env <= 0.0 {
                        v.active = false;
                        break;
                    }
                } else {
                    v.env = (v.env + up).min(1.0);
                }
                let g = g0 * v.gain * v.env * if at_vol { 1.0 + AT_VOL * a } else { 1.0 };
                f[0] += l * g;
                f[1] += r * g;
                v.pos += rate;
            }
        }
    }
}

// ---------------------------------------------------------------- mixer

pub struct Mixer {
    pub sr: u32,
    pub decks: [Deck; NUM_DECKS],
    pub pads: [Pad; NUM_PADS],
    pub crossfader: f32, // 0..1
    pub master: f32,
    pub sampler_vol: f32,
    pub master_level: [f32; 2],
    /// normalizzazione del volume: un brano appena caricato viene portato alla stessa loudness di quello che suona
    pub auto_gain: bool,
    /// deck guida del sync: gli altri deck con SYNC acceso inseguono il suo tempo e la sua fase
    pub leader: Option<usize>,
    leader_prev_phase: Option<f64>,
    /// cuffie: volume, mix fra preascolto (0) e master (1), e bus verso l'uscita cuffie
    pub hp_vol: f32,
    pub hp_mix: f32,
    /// CUE MIX: manopola fra i canali in preascolto, 0 = solo il lato A (deck 1/3), 1 = solo il lato B (deck 2/4), al centro entrambi
    pub cue_mix: f32,
    pub cue_bus: Arc<CueBus>,
    hp_buf: Vec<f32>,
    scratch: Vec<f32>,
    /// carico del thread audio: tempo di calcolo / durata del blocco (media mobile)
    pub cpu: f32,
    /// campionatore in sync: i campioni partono sul battito e seguono tempo e fase del deck guida
    pub pad_sync: bool,
    /// batteria: 16 pad con campioni singoli, volume, nota MIDI di ogni pad e stato dell'apprendimento MIDI
    pub drums: [DrumPad; NUM_DRUMS],
    pub drum_vol: f32,
    pub drum_notes: [u8; NUM_DRUMS],
    pub learn: Option<usize>,
    pub last_note: i16,
    pub note_seq: u32,
    /// registrazione dell'uscita master (se attiva): cattura tutto, deck con effetti, campionatore e batteria
    pub rec: Option<crate::recorder::Tap>,
}

impl Mixer {
    pub fn new(sr: u32) -> Self {
        Mixer {
            sr,
            decks: [Deck::new(sr, 0), Deck::new(sr, 1), Deck::new(sr, 0), Deck::new(sr, 1)],
            pads: Default::default(),
            crossfader: 0.5,
            master: 0.8,
            sampler_vol: 0.8,
            master_level: [0.0; 2],
            auto_gain: true,
            leader: None,
            leader_prev_phase: None,
            hp_vol: 0.8,
            hp_mix: 0.0,
            cue_mix: 0.5,
            cue_bus: Arc::new(CueBus::new(sr)),
            hp_buf: Vec::new(),
            scratch: Vec::new(),
            cpu: 0.0,
            pad_sync: false,
            drums: Default::default(),
            drum_vol: 0.9,
            drum_notes: std::array::from_fn(|i| 36 + i as u8),
            learn: None,
            last_note: -1,
            note_seq: 0,
            rec: None,
        }
    }

    pub fn render(&mut self, out: &mut [f32]) {
        let t0 = std::time::Instant::now();
        self.render_inner(out);
        let budget = (out.len() / 2) as f32 / self.sr.max(1) as f32;
        let load = t0.elapsed().as_secs_f32() / budget.max(1e-6);
        self.cpu += (load - self.cpu) * 0.1;
    }

    fn render_inner(&mut self, out: &mut [f32]) {
        let block_secs = out.len() as f64 / 2.0 / self.sr.max(1) as f64;
        for d in self.decks.iter_mut() {
            d.step_tempo_ramp(block_secs);
        }
        self.update_sync();
        out.iter_mut().for_each(|v| *v = 0.0);
        if self.scratch.len() < out.len() {
            self.scratch.resize(out.len(), 0.0);
        }
        let x = self.crossfader * std::f32::consts::FRAC_PI_2;
        let (ga, gb) = (x.cos(), x.sin());
        for d in self.decks.iter_mut() {
            let xf = match d.xf_assign { 0 => ga, 1 => gb, _ => 1.0 };
            let g = d.volume * d.volume * xf * 10f32.powf(d.norm_db as f32 / 20.0);
            let buf = &mut self.scratch[..out.len()];
            d.render(buf, g);
            for (o, s) in out.iter_mut().zip(buf.iter()) {
                *o += *s;
            }
        }
        // riferimento per il sync dei campioni: tempo e fase del deck che suona (la guida, se c'e')
        let pad_ref = if self.pad_sync { self.beat_reference() } else { None };
        let (sr, vol, sync_on) = (self.sr, self.sampler_vol, self.pad_sync);
        for p in self.pads.iter_mut() {
            if !p.playing && !p.armed { continue; }
            p.render(out, sr, vol, sync_on, pad_ref);
        }
        let dv = self.drum_vol;
        for p in self.drums.iter_mut() {
            if p.data.is_some() { p.render(out, sr, dv); }
        }
        let (mut pl, mut pr) = (0f32, 0f32);
        for f in out.chunks_exact_mut(2) {
            for (c, v) in f.iter_mut().enumerate() {
                let mut y = *v * self.master;
                if y.abs() > 0.9 {
                    y = y.signum() * (0.9 + 0.1 * ((y.abs() - 0.9) / 0.1).tanh());
                }
                *v = y;
                if c == 0 { pl = pl.max(y.abs()) } else { pr = pr.max(y.abs()) }
            }
        }
        let fall = 0.5f32.powf(out.len() as f32 / 2.0 / self.sr.max(1) as f32 / 0.08);
        self.master_level[0] = (self.master_level[0] * fall).max(pl);
        self.master_level[1] = (self.master_level[1] * fall).max(pr);
        // registrazione: il segnale finale del master, dopo il limitatore, uguale a quello che va alle casse
        if let Some(r) = &self.rec {
            r.push(out);
        }
        self.render_cue(out);
    }

    /// Bus cuffie: somma del preascolto dei deck con PFL acceso, mescolata al master secondo `hp_mix`.
    fn render_cue(&mut self, master: &[f32]) {
        let any = self.decks.iter().any(|d| d.pfl);
        if !any && self.hp_mix < 0.001 {
            self.cue_bus.clear();
            return;
        }
        let n = master.len();
        self.hp_buf.clear();
        self.hp_buf.resize(n, 0.0);
        let (cue_g, mst_g) = if any { (1.0 - self.hp_mix, self.hp_mix) } else { (0.0, self.hp_mix) };
        // CUE MIX: al centro i due lati sono entrambi a piena voce, verso un estremo l'altro lato si abbassa fino al silenzio
        let (ga, gb) = ((2.0 * (1.0 - self.cue_mix)).min(1.0), (2.0 * self.cue_mix).min(1.0));
        for d in self.decks.iter().filter(|d| d.pfl && d.pfl_buf.len() == n) {
            let g = match d.xf_assign { 0 => ga, 1 => gb, _ => 1.0 };
            for (h, p) in self.hp_buf.iter_mut().zip(d.pfl_buf.iter()) {
                *h += *p * g;
            }
        }
        let v = self.hp_vol * self.hp_vol;
        for (h, m) in self.hp_buf.iter_mut().zip(master.iter()) {
            *h = ((*h * cue_g + *m * mst_g) * v).clamp(-1.0, 1.0);
        }
        self.cue_bus.push(&self.hp_buf);
    }

    pub fn change_sr(&mut self, new_sr: u32) {
        if new_sr == self.sr {
            return;
        }
        let old = self.sr;
        for d in self.decks.iter_mut() {
            d.change_sr(new_sr);
        }
        for p in self.pads.iter_mut() {
            p.change_sr(old, new_sr);
        }
        for p in self.drums.iter_mut() {
            if let Some(d) = p.data.take() {
                p.data = Some(Arc::new(crate::decode::resample_stereo(&d, old, new_sr)));
            }
        }
        self.sr = new_sr;
        self.cue_bus.set_sr(new_sr);
        self.scratch.clear();
    }

    /// Un pad della batteria: `vel` 0..1.
    pub fn drum_trigger(&mut self, i: usize, vel: f32) {
        if i >= NUM_DRUMS || self.drums[i].data.is_none() { return; }
        let g = self.drums[i].group;
        if g > 0 {
            for (j, p) in self.drums.iter_mut().enumerate() {
                if j != i && p.group == g { p.release_all(); }
            }
        }
        self.drums[i].start(vel.clamp(0.05, 1.0));
    }
    pub fn drum_release(&mut self, i: usize) {
        if i < NUM_DRUMS && self.drums[i].gate { self.drums[i].release_all(); }
    }
    /// Nota MIDI in ingresso: se si sta apprendendo la assegna al pad scelto, altrimenti suona il pad collegato alla nota.
    pub fn midi_note_on(&mut self, note: u8, vel: u8) {
        self.last_note = note as i16;
        self.note_seq = self.note_seq.wrapping_add(1);
        if let Some(i) = self.learn.take() {
            // una nota guida un solo pad: se era di un altro, quel pad resta senza nota (valore 255)
            for (j, n) in self.drum_notes.iter_mut().enumerate() {
                if j != i && *n == note { *n = 255; }
            }
            self.drum_notes[i] = note;
            return;
        }
        if let Some(i) = self.drum_notes.iter().position(|&n| n == note) {
            // curva di velocity un po' morbida: i colpi leggeri restano udibili
            let v = (vel as f32 / 127.0).powf(0.8);
            self.drum_trigger(i, v);
        }
    }
    pub fn midi_note_off(&mut self, note: u8) {
        if let Some(i) = self.drum_notes.iter().position(|&n| n == note) {
            self.drum_release(i);
            self.drums[i].at = 0.0;
        }
    }
    /// Aftertouch: con `note` e' la pressione di un singolo pad, senza e' quella di canale (vale per tutti i pad che suonano).
    pub fn midi_aftertouch(&mut self, note: Option<u8>, pressure: u8) {
        let p = pressure as f32 / 127.0;
        match note {
            Some(n) => {
                if let Some(i) = self.drum_notes.iter().position(|&x| x == n) {
                    self.drums[i].pressure(p);
                }
            }
            None => self.drums.iter_mut().for_each(|d| d.pressure(p)),
        }
    }

    /// (bpm effettivo, fase del battito) del deck guida o, in mancanza, del primo deck in riproduzione
    fn beat_reference(&self) -> Option<(f64, f64)> {
        let playing = |i: usize| self.decks[i].playing && self.decks[i].track.is_some();
        let m = self.leader.filter(|&l| playing(l)).or_else(|| (0..NUM_DECKS).find(|&i| playing(i)))?;
        let d = &self.decks[m];
        let ph = d.track.as_ref().and_then(|t| t.beat_phase(d.pos_sec()))?;
        Some((d.bpm_now(), ph))
    }
    /// Un clic fa partire il campione e un altro clic lo ferma. In modo sync parte sul prossimo battito del deck che suona.
    pub fn pad_trigger(&mut self, i: usize) {
        let sync = self.pad_sync && self.beat_reference().is_some();
        if let Some(p) = self.pads.get_mut(i) {
            if p.data.is_none() { return; }
            if p.armed { p.armed = false; return; }
            if p.playing && !p.stopping { p.stop(); return; }
            if sync { p.armed = true; p.prev_phase = None; } else { p.begin(0.0); }
        }
    }
    pub fn pad_stop(&mut self, i: usize) {
        if let Some(p) = self.pads.get_mut(i) { p.stop(); }
    }

    /// Deck guida del sync. Puo' esserci il SYNC acceso su tutti i deck: uno guida, gli altri seguono.
    /// Guida un deck in riproduzione senza SYNC (quello che il DJ muove a mano); se tutti hanno il SYNC resta
    /// la guida attuale, altrimenti il primo deck in riproduzione. Senza deck in riproduzione resta
    /// l'ultima guida come riferimento di solo tempo.
    fn pick_leader(&mut self) -> Option<usize> {
        let valid = |d: &Deck| d.track.is_some() && d.playing;
        let free: Vec<usize> = (0..NUM_DECKS).filter(|&i| valid(&self.decks[i]) && !self.decks[i].sync).collect();
        let any: Vec<usize> = (0..NUM_DECKS).filter(|&i| valid(&self.decks[i])).collect();
        let cur = self.leader.filter(|&l| valid(&self.decks[l]));
        let pick = match cur {
            Some(l) if !self.decks[l].sync || free.is_empty() => Some(l),
            _ => free.first().or(any.first()).copied(),
        };
        if pick.is_some() {
            self.leader = pick;
        }
        pick
    }

    pub fn is_follower(&self, deck: usize) -> bool {
        deck < NUM_DECKS && self.decks[deck].sync && self.leader != Some(deck)
    }

    /// Sync continuo: i deck con SYNC acceso inseguono tempo e fase della guida a ogni blocco audio.
    /// Il tempo e' quello della guida (con la griglia fluida segue le sue variazioni); la fase si corregge con una
    /// piccola variazione di velocita' proporzionale all'errore, senza salti udibili. Quando un deck parte
    /// (o viene caricato e fatto partire) si allinea subito al battito della guida.
    fn update_sync(&mut self) {
        let playing_leader = self.pick_leader();
        let Some(m) = playing_leader.or(self.leader) else {
            for d in self.decks.iter_mut() { d.sync_prev_playing = d.playing; if d.armed { d.armed = false; d.playing = true; d.sync_prev_playing = true; } }
            self.leader_prev_phase = None;
            return;
        };
        let (mbpm, mph, mplay) = {
            let md = &self.decks[m];
            (md.bpm_now(), md.track.as_ref().and_then(|t| t.beat_phase(md.pos_sec())), md.playing)
        };
        // il battito della guida e' appena scattato? (la fase e' tornata verso 0)
        let beat_wrapped = match (mph, self.leader_prev_phase) { (Some(p), Some(q)) => p < q - 0.5, _ => false };
        self.leader_prev_phase = mph;
        for i in 0..NUM_DECKS {
            let d = &mut self.decks[i];
            if d.armed {
                // partenza quantizzata: aspetta il battito della guida, poi parte esattamente dal punto scelto (per esempio il cue)
                if !mplay || mph.is_none() || !d.sync {
                    d.armed = false;
                    d.playing = true;
                    d.sync_prev_playing = true;
                } else if beat_wrapped && i != m {
                    d.armed = false;
                    let adv = mph.unwrap_or(0.0) * 60.0 / d.track.as_ref().map_or(120.0, |t| t.local_bpm(d.arm_pos).max(1.0));
                    d.set_pos_sec(d.arm_pos + adv);
                    d.playing = true;
                    d.sync_prev_playing = true;
                }
                continue;
            }
            d.sync_prev_playing = d.playing;
            if !d.sync || i == m { continue; }
            let Some(t) = d.track.clone() else { continue };
            let pos = d.pos_sec();
            let mine = t.local_bpm(pos);
            if mine <= 0.0 || mbpm <= 0.0 { continue; }
            let mut target = mbpm;
            let mut octave = false;
            while target / mine > 1.5 { target /= 2.0; octave = true; }
            while target / mine < 0.67 { target *= 2.0; octave = true; }
            let r0 = target / mine;
            let mut r = r0;
            // la fase si porta in linea con una piccola variazione di velocita' proporzionale all'errore; il play non aspetta il battito
            // e non salta in avanti: il suono parte nell'istante in cui si preme
            if let (Some(mp), Some(dp), true, true, false) = (mph, t.beat_phase(pos), mplay, d.playing, octave) {
                let mut e = dp - mp;
                if e > 0.5 { e -= 1.0 } else if e < -0.5 { e += 1.0 }
                if e.abs() > 0.002 {
                    r *= 1.0 - (0.5 * e).clamp(-0.04, 0.04);
                }
            }
            let want = r - 1.0;
            if want.abs() > d.range {
                d.range = [0.04, 0.08, 0.16, 0.5, 1.0].into_iter().find(|&x| x >= want.abs()).unwrap_or(1.0);
            }
            d.tempo = want.clamp(-d.range, d.range).max(-0.95);
        }
    }

    /// Normalizza il volume del deck appena caricato: lo porta alla loudness con cui si sente il brano che suona
    /// su un altro deck; se nessuno suona usa il riferimento (-14 LUFS). Il guadagno e' limitato a -12/+9 dB
    /// e non porta mai il picco oltre il fondo scala.
    pub fn normalize_deck(&mut self, deck: usize) {
        if deck >= NUM_DECKS { return; }
        let (l, peak) = (self.decks[deck].loudness, self.decks[deck].peak);
        if !self.auto_gain || l <= -69.0 {
            self.decks[deck].norm_db = 0.0;
            return;
        }
        let target = (0..NUM_DECKS)
            .filter(|&i| i != deck && self.decks[i].playing && self.decks[i].track.is_some() && self.decks[i].loudness > -69.0)
            .map(|i| self.decks[i].loudness + self.decks[i].norm_db)
            .next()
            .unwrap_or(REFERENCE_LUFS);
        let peak_room = if peak > 1e-6 { -20.0 * (peak as f64).log10() } else { 0.0 };
        self.decks[deck].norm_db = (target - l).clamp(-12.0, 9.0).min(peak_room.max(0.0));
    }

    /// Accende il SYNC su un deck: resta acceso (anche cambiando brano) finche' non lo spegni.
    /// Se un altro deck sta suonando, il deck si allinea subito a lui.
    pub fn sync(&mut self, deck: usize) {
        if deck >= NUM_DECKS { return; }
        self.decks[deck].tempo_ramp = None; // il sync prende il tempo della guida: il rientro non ha piu' senso
        self.decks[deck].sync = true;
        let Some(m) = self.pick_leader() else { return };
        if m == deck { return; }
        let (bpm, phase) = {
            let md = &self.decks[m];
            (md.bpm_now(), md.track.as_ref().and_then(|t| t.beat_phase(md.pos_sec())))
        };
        self.decks[deck].sync_to(bpm, phase);
    }

    pub fn snapshot(&self) -> Snapshot {
        Snapshot {
            t_ms: now_ms(),
            decks: self.decks.iter().map(|d| {
                let t = d.track.as_ref();
                DeckState {
                    loaded: t.is_some(),
                    name: t.map(|t| t.name.clone()).unwrap_or_default(),
                    playing: d.playing,
                    pos: d.pos_sec(),
                    duration: t.map_or(0.0, |t| t.duration()),
                    bpm_orig: t.map_or(0.0, |t| t.bpm),
                    bpm_now: d.bpm_now(),
                    tempo_pct: d.tempo * 100.0,
                    volume: d.volume,
                    tempo_ramp: d.tempo_ramp.is_some(),
                    range_pct: d.range * 100.0,
                    keylock: d.keylock,
                    key_orig: t.and_then(|t| t.key).map(camelot::camelot_code),
                    key_orig_name: t.and_then(|t| t.key).map(camelot::key_name),
                    key_now: d.key_now().map(camelot::camelot_code),
                    key_now_name: d.key_now().map(camelot::key_name),
                    key_shift: d.key_shift,
                    cue: d.cue_point,
                    hot_cues: d.hot_cues,
                    quantize: d.quantize,
                    level: d.level,
                    level_l: d.lvl[0],
                    level_r: d.lvl[1],
                    held: d.held,
                    grid_fixed: d.grid_mode == 1,
                    grid_mode: d.grid_mode,
                    sync: d.sync,
                    pfl: d.pfl,
                    stems: d.stem_mask,
                    stem_gains: d.stem_gains,
                    slip: d.slip,
                    armed: d.armed,
                    rolling: d.rolling,
                    motion: match d.motion { Motion::None => 0, Motion::Brake => 1, Motion::Spin(..) => 2, Motion::Reverse => 3, Motion::Resume => 4 },
                    filter: d.filter,
                    loop_on: d.loop_on,
                    loop_beats: d.loop_beats,
                    loop_in: d.loop_in / d.sr as f64,
                    loop_out: d.loop_out / d.sr as f64,
                    path: t.map_or(String::new(), |t| t.path.clone()),
                    xf_assign: d.xf_assign,
                    norm_db: d.norm_db,
                    loudness: d.loudness,
                    downbeat: t.map_or(0, |t| t.downbeat),
                    beat_phase: t.and_then(|t| t.beat_phase(d.pos_sec())).unwrap_or(0.0),
                }
            }).collect(),
            pads: self.pads.iter().map(|p| PadState { name: p.name.clone(), loaded: p.data.is_some(), playing: p.playing && !p.stopping, armed: p.armed, looped: p.looped, bpm: p.bpm }).collect(),
            pad_sync: self.pad_sync,
            drums: self.drums.iter().enumerate().map(|(i, p)| DrumState { name: p.name.clone(), loaded: p.data.is_some(), playing: p.playing(), gain: p.gain, tune: p.tune, group: p.group, gate: p.gate, note: self.drum_notes[i], at_mode: p.at_mode, at: p.at }).collect(),
            drum_vol: self.drum_vol,
            rec_on: self.rec.is_some(),
            rec_secs: self.rec.as_ref().map_or(0.0, |r| r.frames.load(std::sync::atomic::Ordering::Relaxed) as f64 / self.sr.max(1) as f64),
            rec_dropped: self.rec.as_ref().map_or(0, |r| r.dropped.load(std::sync::atomic::Ordering::Relaxed)),
            learn: self.learn,
            last_note: self.last_note,
            note_seq: self.note_seq,
            crossfader: self.crossfader,
            master_level: self.master_level,
            auto_gain: self.auto_gain,
            hp_vol: self.hp_vol,
            hp_mix: self.hp_mix,
            cue_mix: self.cue_mix,
            cpu: self.cpu,
        }
    }
}

#[derive(Serialize)]
pub struct DeckState {
    pub loaded: bool,
    pub name: String,
    pub playing: bool,
    pub pos: f64,
    pub duration: f64,
    pub bpm_orig: f64,
    pub bpm_now: f64,
    pub tempo_pct: f64,
    /// fader del volume del canale (0..1): la miscela video puo' seguire questo invece del crossfader
    pub volume: f32,
    /// true mentre il pitch rientra da solo verso lo 0
    pub tempo_ramp: bool,
    pub range_pct: f64,
    pub keylock: bool,
    pub key_orig: Option<String>,
    pub key_orig_name: Option<String>,
    pub key_now: Option<String>,
    pub key_now_name: Option<String>,
    pub key_shift: i32,
    pub cue: f64,
    pub hot_cues: [Option<f64>; 8],
    pub quantize: bool,
    pub level: f32,
    /// picco sinistro e destro del canale (dopo il fader)
    pub level_l: f32,
    pub level_r: f32,
    pub held: bool,
    pub grid_fixed: bool,
    pub grid_mode: u8,
    pub sync: bool,
    pub pfl: bool,
    /// stem ascoltati (bit0 batteria, bit1 basso, bit2 altro, bit3 voce) e brano caricato
    pub stems: u8,
    pub stem_gains: [f32; 4],
    pub slip: bool,
    pub armed: bool,
    pub rolling: bool,
    /// 0 normale, 1 freno, 2 spinback, 3 reverse, 4 ripartenza
    pub motion: u8,
    pub filter: f32,
    pub loop_on: bool,
    pub loop_beats: f64,
    pub loop_in: f64,
    pub loop_out: f64,
    pub path: String,
    /// assegnazione al crossfader (0 = A, 1 = B, 2 = diretto)
    pub xf_assign: u8,
    pub norm_db: f64,
    pub loudness: f64,
    pub downbeat: usize,
    pub beat_phase: f64,
}

#[derive(Serialize)]
pub struct GridInfo {
    /// battiti mostrati (dopo l'adattamento della modalita')
    pub beats: Vec<f64>,
    /// battiti di lavoro (rilevati o corretti a mano), da cui la modalita' ricalcola la griglia: e' questa che si salva
    pub base: Vec<f64>,
    /// indice in `base` del battito che apre la battuta
    pub base_downbeat: usize,
    pub downbeat: usize,
    pub bpm: f64,
    pub mode: u8,
    pub labels: Vec<(f64, f64)>,
}

#[derive(Serialize)]
pub struct PadState {
    pub name: String,
    pub loaded: bool,
    pub playing: bool,
    pub armed: bool,
    pub looped: bool,
    pub bpm: f64,
}

#[derive(Serialize)]
pub struct Snapshot {
    /// istante (ms dall'avvio del motore) in cui e' stato preso lo stato: l'interfaccia lo usa al posto dell'istante di arrivo,
    /// che include il ritardo variabile della risposta e farebbe tornare indietro la posizione della waveform
    pub t_ms: f64,
    pub decks: Vec<DeckState>,
    pub pads: Vec<PadState>,
    pub master_level: [f32; 2],
    pub auto_gain: bool,
    pub hp_vol: f32,
    pub hp_mix: f32,
    pub cue_mix: f32,
    pub cpu: f32,
    pub pad_sync: bool,
    pub drums: Vec<DrumState>,
    pub drum_vol: f32,
    pub rec_on: bool,
    pub rec_secs: f64,
    pub rec_dropped: u64,
    pub learn: Option<usize>,
    pub last_note: i16,
    pub note_seq: u32,
    /// posizione del crossfader (0..1): serve anche alla miscela video
    pub crossfader: f32,
}

#[derive(Serialize)]
pub struct DrumState {
    pub name: String,
    pub loaded: bool,
    pub playing: bool,
    pub gain: f32,
    pub tune: f32,
    pub group: u8,
    pub gate: bool,
    pub note: u8,
    pub at_mode: u8,
    pub at: f32,
}

impl Pad {
    pub fn change_sr(&mut self, from: u32, to: u32) {
        if let Some(d) = self.data.take() {
            self.data = Some(Arc::new(crate::decode::resample_stereo(&d, from, to)));
            self.first_beat *= to as f64 / from as f64;
            self.stretch = None;
            self.stretch_mode = false;
            self.pos = 0.0;
            self.playing = false;
            self.stopping = false;
            self.armed = false;
        }
    }
    pub fn load(&mut self, name: String, data: Vec<f32>, bpm: f64, first_beat: f64, looped: bool) {
        self.looped = looped;
        self.name = name;
        self.data = Some(Arc::new(data));
        self.pos = 0.0;
        self.playing = false;
        self.stopping = false;
        self.armed = false;
        self.stretch_mode = false;
        self.gain = 1.0;
        self.bpm = bpm;
        self.first_beat = first_beat;
    }
    /// cambia il tempo del campione senza spostarne la posizione di partenza
    pub fn set_bpm(&mut self, bpm: f64) {
        self.bpm = if bpm >= 30.0 && bpm <= 300.0 { bpm } else { 0.0 };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sine_track(sr: u32, secs: f64, hz: f64) -> Arc<Track> {
        let n = (secs * sr as f64) as usize;
        let mut data = Vec::with_capacity(n * 2);
        for i in 0..n {
            let v = (2.0 * std::f64::consts::PI * hz * i as f64 / sr as f64).sin() as f32 * 0.5;
            data.push(v);
            data.push(v);
        }
        Arc::new(Track { name: "t".into(), path: String::new(), data: Arc::new(data), sr, beats: vec![], bpm: 120.0, key: Some(Key { pc: 9, minor: true }), downbeat: 0 })
    }

    /// frequenza stimata dagli attraversamenti dello zero
    fn freq(buf: &[f32], sr: u32) -> f64 {
        let m: Vec<f32> = buf.chunks(2).map(|f| f[0]).collect();
        let z = m.windows(2).filter(|w| w[0] <= 0.0 && w[1] > 0.0).count();
        z as f64 * sr as f64 / m.len() as f64
    }

    fn run(d: &mut Deck, blocks: usize) -> Vec<f32> {
        let mut all = vec![];
        let mut b = vec![0f32; 1024];
        for _ in 0..blocks {
            d.render(&mut b, 1.0);
            all.extend_from_slice(&b);
        }
        all
    }

    #[test]
    fn keylock_keeps_pitch_varispeed_changes_it() {
        let sr = 44100;
        let mut d = Deck::new(sr, 0);
        d.load(sine_track(sr, 30.0, 440.0));
        d.playing = true;
        d.set_tempo_pct(8.0);
        d.range = 0.08;
        d.keylock = true;
        run(&mut d, 20); // assestamento
        let out = run(&mut d, 40);
        let f = freq(&out, sr);
        assert!((f - 440.0).abs() < 6.0, "keylock: {f}");
        assert!(out.iter().all(|v| v.is_finite()));
        assert!(d.pos > 0.0);

        d.keylock = false;
        run(&mut d, 20);
        let f = freq(&run(&mut d, 40), sr);
        assert!((f - 440.0 * 1.08).abs() < 8.0, "varispeed: {f}");
    }

    #[test]
    fn key_shift_and_camelot() {
        let sr = 44100;
        let mut d = Deck::new(sr, 0);
        d.load(sine_track(sr, 30.0, 440.0));
        d.keylock = true;
        d.key_shift = 2; // +2 semitoni
        d.playing = true;
        run(&mut d, 20);
        let f = freq(&run(&mut d, 40), sr);
        assert!((f - 440.0 * 2f64.powf(2.0 / 12.0)).abs() < 8.0, "shift: {f}");
        // Am (8A) -> Bm = 10A
        assert_eq!(camelot::camelot_code(d.key_now().unwrap()), "10A");
        d.key_shift = 0;
        d.key_wheel(1); // 8A -> 9A (Em = +7 = -5 semitoni)
        assert_eq!(camelot::camelot_code(d.key_now().unwrap()), "9A");
        assert_eq!(d.key_shift, -5);
    }

    #[test]
    fn measure_stretch_latency() {
        let sr = 44100;
        for (keylock, shift, tempo) in [(false, 0, 0.0), (true, 0, 0.0), (true, 0, 8.0), (true, 2, 0.0), (true, 0, -8.0)] {
            let n = sr as usize * 12;
            let mut data = vec![0f32; n * 2];
            let click = sr as usize * 6;
            for k in 0..64 { data[(click + k) * 2] = 1.0; data[(click + k) * 2 + 1] = 1.0; }
            let t = Arc::new(Track { name: "c".into(), path: String::new(), data: Arc::new(data), sr, beats: vec![], bpm: 120.0, key: None, downbeat: 0 });
            let mut d = Deck::new(sr, 0);
            d.load(t);
            d.keylock = keylock;
            d.key_shift = shift;
            d.range = 0.5;
            d.set_tempo_pct(tempo);
            d.set_pos_sec(5.0);
            d.playing = true;
            let mut b = vec![0f32; 256];
            let mut frames = 0usize;
            let mut hit = None;
            for _ in 0..2000 {
                d.render(&mut b, 1.0);
                if let Some(i) = b.iter().position(|v| v.abs() > 0.2) { hit = Some(frames + i / 2); break; }
                frames += 128;
            }
            println!("in_lat={} out_lat={}", d.stretch.input_latency(), d.stretch.output_latency());
            let expected = sr as f64 * 1.0 / d.rate();
            let hit = hit.unwrap_or(0) as f64;
            println!("keylock={keylock} shift={shift} tempo={tempo}: click at {hit} frames, expected {expected:.0}, late by {:.1} ms", (hit - expected) / sr as f64 * 1000.0);
        }
    }

    #[test]
    fn grid_editing() {
        let sr = 44100;
        let mut d = Deck::new(sr, 0);
        let mut t = Arc::try_unwrap(sine_track(sr, 30.0, 440.0)).ok().unwrap();
        // 120 bpm con un po' di rumore, 4 battiti per battuta a partire da 0.1 s
        t.beats = (0..50).map(|i| 0.1 + i as f64 * 0.5 + if i % 2 == 0 { 0.004 } else { -0.004 }).collect();
        t.bpm = 120.0;
        t.downbeat = 1;
        d.load(Arc::new(t));
        d.grid_cmd("double", 0.0);
        let t = d.track.as_ref().unwrap();
        assert_eq!(t.beats.len(), 99);
        assert!((t.bpm - 240.0).abs() < 1e-9);
        d.grid_cmd("halve", 0.0);
        assert_eq!(d.track.as_ref().unwrap().beats.len(), 50);
        // griglia FISSA: intervalli perfettamente uguali, bpm 120
        d.grid_cmd("mode", 1.0);
        let t = d.track.as_ref().unwrap();
        let iv: Vec<f64> = t.beats.windows(2).map(|w| w[1] - w[0]).collect();
        assert!(iv.iter().all(|v| (v - iv[0]).abs() < 1e-9), "intervalli non uniformi");
        assert!((t.bpm - 120.0).abs() < 0.01, "bpm {}", t.bpm);
        d.grid_cmd("mode", 0.0);
        // ancora: il battito piu' vicino alla testina va esattamente sulla testina, i vincoli gia' dati restano
        // l'ancora agisce sulla base della griglia (i battiti prima dell'adattamento di gridfit): si verifica li'
        let before: Vec<f64> = d.grid_base.clone();
        let j = (0..before.len()).min_by(|&a, &b| (before[a] - 10.43).abs().partial_cmp(&(before[b] - 10.43).abs()).unwrap()).unwrap();
        d.set_pos_sec(10.43);
        assert!(d.grid_cmd("anchor_here", 0.0));
        let b = &d.grid_base;
        assert!((b[j] - 10.43).abs() < 1e-9, "ancora {}", b[j]);
        assert!((b[0] - before[0]).abs() < 1e-9 && (b[j - 4] - before[j - 4]).abs() < 1e-9);
        assert!((b[j + 10] - before[j + 10] - (10.43 - before[j])).abs() < 1e-9);
        d.set_pos_sec(7.0);
        d.grid_cmd("anchor_here", 0.0);
        d.set_pos_sec(10.43);
        d.grid_cmd("anchor_here", 0.0);
        let b = &d.track.as_ref().unwrap().beats;
        assert!(b.windows(2).all(|w| w[1] > w[0]));
        d.grid_cmd("reset", 0.0);
        let t = d.track.as_ref().unwrap();
        assert!((t.beats[0] - 0.104).abs() < 0.01 && t.downbeat == 1 && d.grid_mode == 0);
    }

    /// lo stato espone il fader del volume di ogni canale (serve alla miscela video che segue i fader dei canali)
    #[test]
    fn snapshot_exposes_channel_volume() {
        let mut m = Mixer::new(44100);
        m.decks[0].volume = 0.3;
        m.decks[1].volume = 0.9;
        let s = m.snapshot();
        assert!((s.decks[0].volume - 0.3).abs() < 1e-6 && (s.decks[1].volume - 0.9).abs() < 1e-6);
    }

    /// rientro graduale del pitch: parte da +6%, torna a 0 a velocita' costante (0,5 punti al secondo = 12 s), senza scatti e si ferma da solo
    #[test]
    fn tempo_ramp_returns_to_zero_gradually() {
        let sr = 44100;
        let mut m = Mixer::new(sr);
        m.decks[0].load(sine_track(sr, 120.0, 440.0));
        m.decks[0].playing = true;
        m.decks[0].set_tempo_pct(6.0);
        m.decks[0].start_tempo_ramp(0.0);
        assert!(m.decks[0].tempo_ramp.is_some());
        let mut buf = vec![0f32; 1024];
        let block = 512.0 / sr as f64;
        let mut prev = m.decks[0].tempo;
        let mut max_step = 0f64;
        let mut secs_to_zero = None;
        for blk in 0..(sr as usize * 20 / 512) {
            m.render(&mut buf);
            let t = m.decks[0].tempo;
            assert!(t <= prev + 1e-12, "il pitch non deve risalire durante il rientro");
            max_step = max_step.max(prev - t);
            prev = t;
            if secs_to_zero.is_none() && t == 0.0 {
                secs_to_zero = Some((blk + 1) as f64 * block);
            }
        }
        let secs = secs_to_zero.expect("il pitch deve arrivare a 0");
        assert!((secs - 12.0).abs() < 0.2, "rientro in {secs} s invece di 12");
        // il passo massimo per blocco e' quello della velocita' costante: nessun salto
        assert!(max_step <= TEMPO_RAMP_PCT_PER_S / 100.0 * block * 1.001, "passo troppo grande: {max_step}");
        assert!(m.decks[0].tempo_ramp.is_none() && m.decks[0].tempo == 0.0, "a fine rientro si ferma");
    }

    /// un pitch piccolo rientra presto, uno grande piu' lentamente (stessa velocita'); funziona anche verso il basso
    #[test]
    fn tempo_ramp_speed_is_constant_for_any_offset() {
        let sr = 44100;
        for (start, expect) in [(2.0, 4.0), (-4.0, 8.0), (8.0, 16.0)] {
            let mut m = Mixer::new(sr);
            m.decks[0].load(sine_track(sr, 120.0, 440.0));
            m.decks[0].playing = true;
            m.decks[0].set_tempo_pct(start);
            m.decks[0].start_tempo_ramp(0.0);
            let mut buf = vec![0f32; 1024];
            let mut blocks = 0;
            while m.decks[0].tempo_ramp.is_some() && blocks < sr as usize * 30 / 512 {
                m.render(&mut buf);
                blocks += 1;
            }
            let secs = blocks as f64 * 512.0 / sr as f64;
            assert!((secs - expect).abs() < 0.2, "da {start}%: {secs} s invece di {expect}");
            assert_eq!(m.decks[0].tempo, 0.0);
        }
    }

    /// il rientro si interrompe se muovi il pitch a mano, se lo ferma il pulsante, se carichi un brano o se accendi il sync
    #[test]
    fn tempo_ramp_is_cancelled_by_user_actions() {
        let sr = 44100;
        let mut m = Mixer::new(sr);
        m.decks[0].load(sine_track(sr, 120.0, 440.0));
        m.decks[1].load(sine_track(sr, 120.0, 440.0));
        m.decks[0].playing = true;
        m.decks[1].playing = true;
        let mut buf = vec![0f32; 1024];
        // a mano
        m.decks[0].set_tempo_pct(5.0);
        m.decks[0].start_tempo_ramp(0.0);
        for _ in 0..100 { m.render(&mut buf); }
        let mid = m.decks[0].tempo;
        assert!(mid < 0.05 && mid > 0.0);
        m.decks[0].set_tempo_pct(3.0);
        assert!(m.decks[0].tempo_ramp.is_none());
        for _ in 0..100 { m.render(&mut buf); }
        assert!((m.decks[0].tempo - 0.03).abs() < 1e-9, "dopo l'intervento manuale il pitch deve restare dov'e'");
        // fermato dal pulsante: resta dove si trova
        m.decks[0].start_tempo_ramp(0.0);
        for _ in 0..100 { m.render(&mut buf); }
        m.decks[0].cancel_tempo_ramp();
        let stop = m.decks[0].tempo;
        for _ in 0..100 { m.render(&mut buf); }
        assert_eq!(m.decks[0].tempo, stop);
        // nuovo brano
        m.decks[0].start_tempo_ramp(0.0);
        m.decks[0].load(sine_track(sr, 120.0, 440.0));
        assert!(m.decks[0].tempo_ramp.is_none());
        // sync
        m.decks[1].set_tempo_pct(4.0);
        m.decks[1].start_tempo_ramp(0.0);
        m.sync(1);
        assert!(m.decks[1].tempo_ramp.is_none());
    }

    /// sync continuo: tempo e fase del master vengono inseguiti anche quando il master cambia velocita'
    #[test]
    fn continuous_sync_follows_master() {
        let sr = 44100;
        let mut m = Mixer::new(sr);
        let mk = |bpm: f64, first: f64| {
            let mut t = Arc::try_unwrap(sine_track(sr, 120.0, 440.0)).ok().unwrap();
            let iv = 60.0 / bpm;
            t.beats = (0..(110.0 / iv) as usize).map(|i| first + i as f64 * iv).collect();
            t.bpm = bpm;
            Arc::new(t)
        };
        m.decks[0].load(mk(120.0, 0.0));
        m.decks[1].load(mk(100.0, 0.13));
        m.decks[0].playing = true;
        m.decks[1].playing = true;
        m.decks[0].set_pos_sec(5.0);
        m.decks[1].set_pos_sec(8.3);
        m.sync(1);
        assert!(m.decks[1].sync && m.leader == Some(0));
        let mut buf = vec![0f32; 1024];
        let mut worst = 0f64;
        for blk in 0..(sr as usize * 40 / 512) {
            if blk == sr as usize * 10 / 512 {
                m.decks[0].set_tempo_pct(6.0); // il DJ cambia il pitch del master a meta' mix
            }
            m.render(&mut buf);
            if blk > sr as usize * 14 / 512 {
                let (a, b) = (&m.decks[0], &m.decks[1]);
                let pa = a.track.as_ref().unwrap().beat_phase(a.pos_sec()).unwrap();
                let pb = b.track.as_ref().unwrap().beat_phase(b.pos_sec()).unwrap();
                let mut e = pa - pb;
                if e > 0.5 { e -= 1.0 } else if e < -0.5 { e += 1.0 }
                worst = worst.max(e.abs());
            }
        }
        let (a, b) = (&m.decks[0], &m.decks[1]);
        assert!((a.bpm_now() - b.bpm_now()).abs() < 0.05, "bpm {} vs {}", a.bpm_now(), b.bpm_now());
        assert!(worst < 0.02, "errore di fase massimo {worst:.3} battiti");
        // muovere il pitch del deck sincronizzato spegne il sync
        m.decks[1].sync = false;
        assert!(m.snapshot().decks[1].sync == false);
    }

    /// scrub: con il deck fermo il suono segue la puntina (anche all'indietro) e tace quando il mouse si ferma
    #[test]
    fn scrub_is_audible_while_paused() {
        let sr = 44100;
        let mut d = Deck::new(sr, 2);
        d.load(sine_track(sr, 30.0, 440.0));
        d.set_pos_sec(10.0);
        d.held = true;
        let mut buf = vec![0f32; 1024];
        d.scrub_to(Some(10.5));
        let mut peak = 0f32;
        for _ in 0..20 {
            d.render(&mut buf, 1.0);
            peak = peak.max(buf.iter().fold(0f32, |m, v| m.max(v.abs())));
        }
        assert!(peak > 0.05, "nessun suono durante lo scrub: {peak}");
        assert!(d.pos_sec() > 10.2, "la puntina non si e' mossa: {}", d.pos_sec());
        d.scrub_to(Some(9.0)); // all'indietro
        for _ in 0..60 {
            d.render(&mut buf, 1.0);
        }
        assert!(d.pos_sec() < 9.2, "scrub all'indietro: {}", d.pos_sec());
        for _ in 0..200 {
            d.render(&mut buf, 1.0);
        }
        let tail = buf.iter().fold(0f32, |m, v| m.max(v.abs()));
        assert!(tail < 0.01, "a mouse fermo deve tacere: {tail}");
        assert!(!d.playing);
    }

    /// scrub col jog: la velocita' cambia con gradualita' anche quando la posizione arriva a scatti (come i messaggi MIDI del piatto)
    #[test]
    fn scrub_speed_changes_smoothly() {
        let sr = 44100;
        let mut d = Deck::new(sr, 2);
        d.load(sine_track(sr, 60.0, 440.0));
        d.set_pos_sec(20.0);
        d.held = true;
        let mut buf = vec![0f32; 256];
        let mut tgt = 20.0f64;
        let mut speeds = vec![];
        for blk in 0..400 {
            if blk % 3 == 0 { tgt += 0.03; d.scrub_to(Some(tgt)); } // ~ un aggiornamento ogni 17 ms, 1,7x
            let p0 = d.pos_sec();
            d.render(&mut buf, 1.0);
            speeds.push((d.pos_sec() - p0) * sr as f64 / 128.0);
        }
        let steady = &speeds[150..];
        let jump = steady.windows(2).fold(0f64, |m, w| m.max((w[1] - w[0]).abs()));
        println!("salto massimo di velocita' tra blocchi: {jump:.3}");
        assert!(jump < 0.3, "la velocita' salta di {jump} tra due blocchi");
        let peak = buf.iter().fold(0f32, |m, v| m.max(v.abs()));
        assert!(peak > 0.1 && peak <= 1.0, "livello dello scrub: {peak}");
    }

    /// clic sulla panoramica: la puntina salta nel punto indicato invece di correre fin li'
    #[test]
    fn scrub_long_jump_lands_immediately() {
        let sr = 44100;
        let mut d = Deck::new(sr, 2);
        d.load(sine_track(sr, 300.0, 440.0));
        d.set_pos_sec(10.0);
        d.held = true;
        let mut buf = vec![0f32; 256];
        d.scrub_to(Some(200.0));
        for _ in 0..4 { d.render(&mut buf, 1.0); }
        assert!((d.pos_sec() - 200.0).abs() < 1.0, "dopo il salto la testina e' a {}", d.pos_sec());
        d.scrub_to(Some(40.0));
        for _ in 0..4 { d.render(&mut buf, 1.0); }
        assert!((d.pos_sec() - 40.0).abs() < 1.0, "salto all'indietro: {}", d.pos_sec());
    }

    /// normalizzazione: il brano nuovo prende la loudness di quello che suona, entro i limiti di guadagno
    #[test]
    fn normalize_matches_playing_deck() {
        let sr = 44100;
        let mut m = Mixer::new(sr);
        m.decks[0].load(sine_track(sr, 10.0, 440.0));
        m.decks[0].loudness = -10.0;
        m.decks[0].peak = 0.8;
        m.decks[0].playing = true;
        m.decks[1].load(sine_track(sr, 10.0, 440.0));
        m.decks[1].loudness = -16.0;
        m.decks[1].peak = 0.3;
        m.normalize_deck(1);
        assert!((m.decks[1].norm_db - 6.0).abs() < 1e-9, "{}", m.decks[1].norm_db);
        // il guadagno non porta il picco sopra il fondo scala
        m.decks[1].peak = 0.7;
        m.normalize_deck(1);
        assert!(m.decks[1].norm_db <= -20.0 * 0.7f64.log10() + 1e-6);
        // deck 0 normalizzato a +3 dB: il nuovo brano deve arrivare a -10 + 3 = -7 LUFS
        m.decks[0].norm_db = 3.0;
        m.decks[1].peak = 0.1;
        m.decks[1].loudness = -12.0;
        m.normalize_deck(1);
        assert!((m.decks[1].norm_db - 5.0).abs() < 1e-9, "{}", m.decks[1].norm_db);
        // nessun brano in riproduzione: riferimento -14 LUFS; troppo forte viene abbassato
        m.decks[0].playing = false;
        m.decks[1].loudness = -6.0;
        m.normalize_deck(1);
        assert!((m.decks[1].norm_db + 8.0).abs() < 1e-9);
        m.auto_gain = false;
        m.normalize_deck(1);
        assert_eq!(m.decks[1].norm_db, 0.0);
    }

    /// il SYNC resta acceso su entrambi i deck, anche dopo aver caricato un altro brano
    #[test]
    fn sync_stays_on_for_both_decks_and_across_loads() {
        let sr = 44100;
        let mut m = Mixer::new(sr);
        let mk = |bpm: f64, first: f64| {
            let mut t = Arc::try_unwrap(sine_track(sr, 120.0, 440.0)).ok().unwrap();
            let iv = 60.0 / bpm;
            t.beats = (0..(110.0 / iv) as usize).map(|i| first + i as f64 * iv).collect();
            t.bpm = bpm;
            Arc::new(t)
        };
        m.decks[0].load(mk(120.0, 0.0));
        m.decks[1].load(mk(100.0, 0.2));
        m.sync(0);
        m.sync(1);
        assert!(m.decks[0].sync && m.decks[1].sync, "accendere un deck non deve spegnere l'altro");
        m.decks[0].playing = true;
        let mut buf = vec![0f32; 1024];
        for _ in 0..50 { m.render(&mut buf); }
        m.decks[1].set_pos_sec(7.3);
        m.decks[1].playing = true;
        let mut worst = 0f64;
        for blk in 0..(sr as usize * 30 / 512) {
            m.render(&mut buf);
            if blk > sr as usize * 12 / 512 {
                let (a, b) = (&m.decks[0], &m.decks[1]);
                let pa = a.track.as_ref().unwrap().beat_phase(a.pos_sec()).unwrap();
                let pb = b.track.as_ref().unwrap().beat_phase(b.pos_sec()).unwrap();
                let mut e = pa - pb;
                if e > 0.5 { e -= 1.0 } else if e < -0.5 { e += 1.0 }
                worst = worst.max(e.abs());
            }
        }
        assert!(worst < 0.02, "fase {worst}");
        assert!((m.decks[0].bpm_now() - m.decks[1].bpm_now()).abs() < 0.05);
        assert_eq!(m.leader, Some(0), "la guida non deve cambiare a ogni blocco");
        // il deck 1 cambia brano: svuotato e ricaricato, il SYNC resta acceso e alla partenza si riallinea
        m.decks[1].unload();
        assert!(m.decks[1].sync);
        m.decks[1].load(mk(90.0, 0.37));
        assert!(m.decks[1].sync, "il caricamento non deve spegnere il sync");
        m.decks[1].playing = true;
        for _ in 0..(sr as usize * 12 / 512) { m.render(&mut buf); }
        let (a, b) = (&m.decks[0], &m.decks[1]);
        let pa = a.track.as_ref().unwrap().beat_phase(a.pos_sec()).unwrap();
        let pb = b.track.as_ref().unwrap().beat_phase(b.pos_sec()).unwrap();
        let mut e = pa - pb;
        if e > 0.5 { e -= 1.0 } else if e < -0.5 { e += 1.0 }
        assert!(e.abs() < 0.03, "dopo il nuovo caricamento: {e}");
        assert!((a.bpm_now() - b.bpm_now()).abs() < 0.05);
        // la guida si ferma: l'altro deck diventa guida senza spegnere il sync di nessuno
        m.decks[0].playing = false;
        m.render(&mut buf);
        assert_eq!(m.leader, Some(1));
        assert!(m.decks[0].sync && m.decks[1].sync);
    }

    /// preascolto: il canale arriva alle cuffie anche con il fader a zero e senza sentirsi nel master
    #[test]
    fn pfl_goes_to_headphones_pre_fader() {
        let sr = 44100;
        let mut m = Mixer::new(sr);
        m.decks[0].load(sine_track(sr, 30.0, 440.0));
        m.decks[0].playing = true;
        m.decks[0].volume = 0.0; // fader chiuso: niente nel master
        m.hp_mix = 0.0;
        m.hp_vol = 1.0;
        let mut buf = vec![0f32; 1024];
        let mut puller = crate::cue::Puller::default();
        let mut hp = vec![0f32; 1024];
        let (mut master_peak, mut hp_peak) = (0f32, 0f32);
        // senza PFL le cuffie sono mute (mix a 0)
        for _ in 0..40 { m.render(&mut buf); puller.pull(&m.cue_bus, &mut hp, sr); }
        assert!(hp.iter().all(|v| v.abs() < 1e-6), "cuffie mute senza PFL");
        m.decks[0].pfl = true;
        for _ in 0..80 {
            m.render(&mut buf);
            master_peak = master_peak.max(buf.iter().fold(0f32, |a, v| a.max(v.abs())));
            puller.pull(&m.cue_bus, &mut hp, sr);
            hp_peak = hp_peak.max(hp.iter().fold(0f32, |a, v| a.max(v.abs())));
        }
        assert!(master_peak < 1e-4, "il master non deve sentire il canale: {master_peak}");
        assert!(hp_peak > 0.2, "il preascolto non arriva alle cuffie: {hp_peak}");
        // mix al 100%: nelle cuffie c'e' il master (qui muto)
        m.hp_mix = 1.0;
        for _ in 0..40 { m.render(&mut buf); puller.pull(&m.cue_bus, &mut hp, sr); }
        let tail = hp.iter().fold(0f32, |a, v| a.max(v.abs()));
        assert!(tail < 1e-3, "con il mix sul master il canale non deve sentirsi: {tail}");
    }

    /// CUE MIX: la manopola dosa in cuffia il lato A (deck 1/3) e il lato B (deck 2/4)
    #[test]
    fn cue_mix_blends_channels() {
        let sr = 44100;
        let mut m = Mixer::new(sr);
        // deck 0 (lato A) a 440 Hz e deck 1 (lato B) a 880 Hz, entrambi in PFL con il fader chiuso
        m.decks[0].load(sine_track(sr, 30.0, 440.0));
        m.decks[1].load(sine_track(sr, 30.0, 880.0));
        for i in 0..2 {
            m.decks[i].playing = true;
            m.decks[i].volume = 0.0;
            m.decks[i].pfl = true;
        }
        m.hp_vol = 1.0;
        let mut buf = vec![0f32; 1024];
        let mut hp = vec![0f32; 1024];
        let mut measure = |m: &mut Mixer, mix: f32| -> (f32, f32) {
            m.cue_mix = mix;
            let mut puller = crate::cue::Puller::default();
            // ampiezza di ciascun tono (correlazione in quadratura: non dipende dalla fase)
            let mut acc = [0f64; 4];
            for blk in 0..120 {
                m.render(&mut buf);
                puller.pull(&m.cue_bus, &mut hp, sr);
                if blk > 60 {
                    for (k, f) in hp.chunks(2).enumerate() {
                        let t = (blk * 512 + k) as f64 / sr as f64;
                        for (j, hz) in [440.0f64, 880.0].iter().enumerate() {
                            let ph = 2.0 * std::f64::consts::PI * hz * t;
                            acc[j * 2] += f[0] as f64 * ph.sin();
                            acc[j * 2 + 1] += f[0] as f64 * ph.cos();
                        }
                    }
                }
            }
            let amp = |s: f64, c: f64| ((s * s + c * c).sqrt() / 15000.0) as f32;
            (amp(acc[0], acc[1]), amp(acc[2], acc[3]))
        };
        let _ = &mut measure;
        let (a0, b0) = measure(&mut m, 0.0);
        assert!(a0 > 0.5 && b0 < 0.1, "solo lato A: {a0} {b0}");
        let (a1, b1) = measure(&mut m, 1.0);
        assert!(b1 > 0.5 && a1 < 0.1, "solo lato B: {a1} {b1}");
        let (ac, bc) = measure(&mut m, 0.5);
        assert!(ac > 0.5 && bc > 0.5, "al centro entrambi: {ac} {bc}");
    }

    /// stem: il deck legge il buffer alternativo al posto del brano, con una breve rampa in entrata, e torna al brano intero
    #[test]
    fn stems_view_replaces_track_audio() {
        let sr = 44100;
        let mut d = Deck::new(sr, 0);
        d.load(sine_track(sr, 30.0, 440.0));
        d.playing = true;
        d.keylock = false;
        let mut buf = vec![0f32; 1024];
        let rms = |b: &[f32]| (b.iter().map(|v| v * v).sum::<f32>() / b.len() as f32).sqrt();
        for _ in 0..12 { d.render(&mut buf, 1.0); }
        assert!(rms(&buf) > 0.05, "il brano intero deve suonare");
        // buffer alternativo di soli zero (stem vuoto): silenzio
        d.set_stems(crate::stems::mask_gains(8), Some(Arc::new(vec![0f32; 30 * sr as usize * 2])));
        for _ in 0..12 { d.render(&mut buf, 1.0); }
        assert!(rms(&buf) < 1e-4, "lo stem vuoto deve tacere: {}", rms(&buf));
        assert_eq!(d.stem_mask, 8);
        d.set_stems([1.0; 4], None);
        for _ in 0..12 { d.render(&mut buf, 1.0); }
        assert!(rms(&buf) > 0.05, "il brano intero deve tornare");
        // il passaggio e' una dissolvenza incrociata: nessuno scatto fra campioni vicini mentre si cambia stem
        let mut d = Deck::new(sr, 0);
        d.load(sine_track(sr, 30.0, 440.0));
        d.playing = true;
        d.keylock = false;
        let mut prev = 0f32;
        let mut max_step = 0f32;
        let other: Vec<f32> = (0..30 * sr as usize).flat_map(|i| { let v = 0.5 * (2.0 * std::f32::consts::PI * 440.0 * i as f32 / sr as f32 + 2.0).sin(); [v, v] }).collect();
        for blk in 0..24 {
            if blk == 8 { d.set_stems(crate::stems::mask_gains(8), Some(Arc::new(other.clone()))); }
            d.render(&mut buf, 1.0);
            for f in buf.chunks(2) {
                max_step = max_step.max((f[0] - prev).abs());
                prev = f[0];
            }
        }
        assert!(max_step < 0.1, "scatto nel cambio di stem: {max_step}");
    }

    /// loop: la posizione resta dentro l'intervallo e l'audio e' continuo (varispeed e keylock)
    #[test]
    fn loop_keeps_playhead_inside() {
        let sr = 44100;
        for keylock in [false, true] {
            let mut d = Deck::new(sr, 0);
            d.load(sine_track(sr, 60.0, 440.0));
            d.keylock = keylock;
            d.playing = true;
            d.quantize = false;
            d.set_pos_sec(10.0);
            let mut buf = vec![0f32; 1024];
            for _ in 0..4 { d.render(&mut buf, 1.0); }
            d.loop_beats = 2.0; // sine_track ha 120 bpm: 1 s
            d.loop_auto();
            let (a, b) = (d.loop_in / sr as f64, d.loop_out / sr as f64);
            assert!((b - a - 1.0).abs() < 0.01, "lunghezza loop {}", b - a);
            let mut max_step = 0f32;
            let mut prev = 0f32;
            for k in 0..400 {
                d.render(&mut buf, 1.0);
                let p = d.pos_sec();
                if k > 40 { assert!(p > a - 0.5 && p < b + 0.5, "posizione fuori dal loop: {p} ({a}..{b}) keylock={keylock}"); }
                for f in buf.chunks(2) { if k > 40 { max_step = max_step.max((f[0] - prev).abs()); } prev = f[0]; }
            }
            assert!(max_step < 0.3, "scatto nel loop: {max_step} keylock={keylock}");
            d.loop_toggle();
            assert!(!d.loop_on);
        }
    }

    fn tone(sr: u32, secs: f64, hz: f64) -> Vec<f32> {
        (0..(secs * sr as f64) as usize).flat_map(|i| { let v = (2.0 * std::f64::consts::PI * hz * i as f64 / sr as f64).sin() as f32 * 0.5; [v, v] }).collect()
    }

    /// campionatore: un clic fa partire il campione, un altro lo ferma (senza click)
    #[test]
    fn pad_click_toggles_play_and_stop() {
        let sr = 44100;
        let mut m = Mixer::new(sr);
        m.pads[0].load("p".into(), tone(sr, 3.0, 330.0), 0.0, 0.0, false);
        let rms = |b: &[f32]| (b.iter().map(|v| v * v).sum::<f32>() / b.len() as f32).sqrt();
        let mut buf = vec![0f32; 1024];
        m.pad_trigger(0);
        for _ in 0..10 { m.render(&mut buf); }
        assert!(m.pads[0].playing && rms(&buf) > 0.05, "il campione deve suonare");
        let mut prev = buf[buf.len() - 2];
        m.pad_trigger(0);
        let mut max_step = 0f32;
        for _ in 0..10 { m.render(&mut buf); for f in buf.chunks(2) { max_step = max_step.max((f[0] - prev).abs()); prev = f[0]; } }
        assert!(!m.pads[0].playing && rms(&buf) < 1e-6, "il secondo clic deve fermarlo");
        assert!(max_step < 0.1, "scatto allo stop: {max_step}");
        m.pad_trigger(0);
        for _ in 0..4 { m.render(&mut buf); }
        assert!(m.pads[0].playing, "un altro clic lo fa ripartire");
    }

    /// campionatore in sync: parte sul battito del deck e segue fase e variazioni di tempo
    #[test]
    fn pad_sync_follows_deck_tempo_and_phase() {
        let sr = 44100;
        let mut m = Mixer::new(sr);
        let n = 60.0;
        let track = Track { name: "t".into(), path: String::new(), data: Arc::new(tone(sr, n, 220.0)), sr, beats: (0..118).map(|k| k as f64 * 0.5).collect(), bpm: 120.0, key: None, downbeat: 0 };
        m.decks[0].load(Arc::new(track));
        m.decks[0].playing = true;
        // loop di 4 battiti a 100 bpm (2,4 s)
        m.pads[0].load("p".into(), tone(sr, 2.4, 330.0), 100.0, 0.0, true);
        m.pads[0].looped = true;
        m.pad_sync = true;
        m.pad_trigger(0);
        assert!(m.pads[0].armed && !m.pads[0].playing, "in attesa del battito");
        let mut buf = vec![0f32; 1024];
        let phase_err = |m: &Mixer| -> f64 {
            let p = &m.pads[0];
            let spb = 60.0 / p.bpm * sr as f64;
            let pph = ((p.pos - p.lead - p.first_beat) / spb).rem_euclid(1.0);
            let d = &m.decks[0];
            let mph = d.track.as_ref().unwrap().beat_phase(d.pos_sec()).unwrap();
            let mut e = pph - mph;
            if e > 0.5 { e -= 1.0 } else if e < -0.5 { e += 1.0 }
            e
        };
        let mut started = false;
        for _ in 0..300 { m.render(&mut buf); started |= m.pads[0].playing; }
        assert!(started, "il campione deve partire da solo");
        let e = phase_err(&m);
        assert!(e.abs() < 0.05, "fase del campione rispetto al deck: {e}");
        // il deck accelera del 6%: il campione deve seguirlo
        m.decks[0].tempo = 0.06;
        for _ in 0..800 { m.render(&mut buf); }
        let e = phase_err(&m);
        assert!(e.abs() < 0.06, "dopo il cambio di tempo: {e}");
        assert!(m.pads[0].playing);
    }

    /// batteria: un colpo suona per intero, la ripetizione taglia la coda senza click, il gruppo choke zittisce l'altro pad
    #[test]
    fn drums_trigger_retrigger_choke_and_gate() {
        let sr = 44100;
        let mut m = Mixer::new(sr);
        let rms = |b: &[f32]| (b.iter().map(|v| v * v).sum::<f32>() / b.len() as f32).sqrt();
        m.drums[0].load("kick".into(), String::new(), tone(sr, 0.5, 60.0));
        m.drums[1].load("hat".into(), String::new(), tone(sr, 2.0, 3000.0));
        m.drums[2].load("open".into(), String::new(), tone(sr, 2.0, 5000.0));
        let mut buf = vec![0f32; 512];
        // un pad senza campione non suona
        m.drum_trigger(7, 1.0);
        m.render(&mut buf);
        assert!(rms(&buf) < 1e-6);
        m.drum_trigger(0, 1.0);
        for _ in 0..4 { m.render(&mut buf); }
        assert!(rms(&buf) > 0.05, "il kick deve suonare");
        for _ in 0..100 { m.render(&mut buf); }
        assert!(!m.drums[0].playing() && rms(&buf) < 1e-6, "dopo 0,5 s il kick e' finito da solo");
        // ripetizione veloce: nessuno scatto
        let mut prev = 0f32;
        let mut max_step = 0f32;
        for k in 0..60 {
            if k % 6 == 0 { m.drum_trigger(0, 1.0); }
            m.render(&mut buf);
            for f in buf.chunks(2) { if k > 3 { max_step = max_step.max((f[0] - prev).abs()); } prev = f[0]; }
        }
        assert!(max_step < 0.2, "scatto alla ripetizione: {max_step}");
        // choke: hat chiuso (gruppo 1) zittisce l'aperto (gruppo 1)
        m.drums[1].group = 1; m.drums[2].group = 1;
        m.drum_trigger(2, 1.0);
        for _ in 0..4 { m.render(&mut buf); }
        assert!(m.drums[2].playing());
        m.drum_trigger(1, 1.0);
        for _ in 0..6 { m.render(&mut buf); }
        assert!(!m.drums[2].playing(), "l'hat aperto deve essere zittito");
        // gate: suona finche' e' premuto
        m.drums[1].gate = true;
        m.drum_trigger(1, 1.0);
        for _ in 0..80 { m.render(&mut buf); }
        assert!(m.drums[1].playing(), "con il gate suona finche' tieni premuto");
        m.drum_release(1);
        for _ in 0..40 { m.render(&mut buf); }
        assert!(!m.drums[1].playing() && rms(&buf) < 1e-6, "al rilascio smette");
    }

    /// MIDI: le note 36.. suonano i pad; l'apprendimento assegna al pad la prossima nota ricevuta
    #[test]
    fn aftertouch_raises_volume_and_pitch_of_held_pad() {
        let sr = 44100;
        let mut m = Mixer::new(sr);
        m.drums[0].load("tone".into(), String::new(), tone(sr, 3.0, 440.0));
        let mut buf = vec![0f32; 4096];
        let level = |m: &mut Mixer, buf: &mut Vec<f32>| { for _ in 0..8 { m.render(buf); } (buf.iter().map(|x| x * x).sum::<f32>() / buf.len() as f32).sqrt() };
        m.midi_note_on(36, 100);
        let plain = level(&mut m, &mut buf);
        m.midi_aftertouch(Some(36), 127);
        let pressed = level(&mut m, &mut buf);
        assert!(pressed > plain * 1.4, "con la pressione massima il volume sale: {plain} -> {pressed}");
        m.midi_aftertouch(None, 0);
        let relaxed = level(&mut m, &mut buf);
        assert!((relaxed - plain).abs() < plain * 0.1, "senza pressione torna com'era");
        // intonazione: la posizione di lettura avanza piu' in fretta
        m.drums[0].at_mode = 2;
        m.midi_note_off(36);
        m.midi_note_on(36, 100);
        m.midi_aftertouch(Some(36), 127);
        for _ in 0..8 { m.render(&mut buf); }
        assert!(m.drums[0].voices.iter().any(|v| v.active && v.pos > 8.0 * 2048.0 * 1.3), "con aftertouch l'intonazione sale");
        // pad fermo: la pressione non resta in memoria per il colpo successivo
        m.drums[0].at_mode = 1;
        m.drum_notes[1] = 37;
        m.midi_aftertouch(Some(37), 100);
        assert_eq!(m.drums[1].at, 0.0);
    }

    #[test]
    fn midi_notes_play_and_learn_drum_pads() {
        let sr = 44100;
        let mut m = Mixer::new(sr);
        m.drums[0].load("kick".into(), String::new(), tone(sr, 0.5, 60.0));
        m.drums[3].load("snare".into(), String::new(), tone(sr, 0.5, 200.0));
        let mut buf = vec![0f32; 512];
        m.midi_note_on(39, 100); // 36 + 3
        for _ in 0..3 { m.render(&mut buf); }
        assert!(m.drums[3].playing() && !m.drums[0].playing());
        // apprendimento: il pad 0 prende la nota 60 (e la toglie al pad che l'aveva)
        m.drum_notes[5] = 60;
        m.learn = Some(0);
        m.midi_note_on(60, 90);
        assert_eq!(m.drum_notes[0], 60);
        assert_eq!(m.drum_notes[5], 255);
        assert!(m.learn.is_none());
        assert!(!m.drums[0].playing(), "la nota usata per imparare non suona");
        m.midi_note_on(60, 90);
        for _ in 0..3 { m.render(&mut buf); }
        assert!(m.drums[0].playing(), "dopo l'apprendimento la nota suona il pad 0");
    }

    /// la registrazione cattura l'uscita master intera: deck, campionatore e batteria insieme
    #[test]
    fn recording_captures_decks_sampler_and_drums() {
        let sr = 44100;
        let mut m = Mixer::new(sr);
        let path = std::env::temp_dir().join("spinlab_rec_mix_test.wav");
        let _ = std::fs::remove_file(&path);
        let (rec, tap) = crate::recorder::start(&path, crate::recorder::Format::Wav16, sr).unwrap();
        m.rec = Some(tap);
        let mut buf = vec![0f32; 512];
        let mut rendered: Vec<f32> = vec![];
        let mut capture = |m: &mut Mixer, blocks: usize, rendered: &mut Vec<f32>| {
            for _ in 0..blocks { m.render(&mut buf); rendered.extend_from_slice(&buf); }
        };
        // 1) silenzio, 2) solo il campionatore, 3) solo la batteria, 4) deck con filtro attivo
        capture(&mut m, 20, &mut rendered);
        let n_silence = rendered.len();
        m.pads[0].load("p".into(), tone(sr, 1.0, 330.0), 0.0, 0.0, false);
        m.pad_trigger(0);
        capture(&mut m, 40, &mut rendered);
        let n_pad = rendered.len();
        m.pad_trigger(0);
        capture(&mut m, 10, &mut rendered);
        m.drums[0].load("k".into(), String::new(), tone(sr, 0.4, 100.0));
        m.drum_trigger(0, 1.0);
        capture(&mut m, 40, &mut rendered);
        let n_drum = rendered.len();
        m.decks[0].load(sine_track(sr, 20.0, 220.0));
        m.decks[0].playing = true;
        m.decks[0].filter = 0.2;
        capture(&mut m, 40, &mut rendered);
        m.rec = None; // come fa il comando di stop
        let fin = rec.finish().unwrap();
        assert_eq!(fin.dropped, 0);
        let mut r = hound::WavReader::open(&path).unwrap();
        let samples: Vec<i16> = r.samples::<i16>().map(|s| s.unwrap()).collect();
        assert_eq!(samples.len(), rendered.len(), "il file deve avere esattamente cio' che e' uscito dal master");
        let rms = |a: usize, b: usize| (samples[a..b].iter().map(|v| (*v as f32 / 32768.0).powi(2)).sum::<f32>() / (b - a) as f32).sqrt();
        assert!(rms(0, n_silence) < 1e-4, "silenzio all'inizio");
        assert!(rms(n_silence + 2000, n_pad) > 0.05, "il campionatore deve essere nel file");
        assert!(rms(n_pad + 8000, n_drum) > 0.05, "la batteria deve essere nel file");
        assert!(rms(n_drum + 4000, samples.len()) > 0.01, "il deck con il filtro deve essere nel file");
        // il file coincide con il segnale renderizzato (16 bit)
        let err = samples.iter().zip(rendered.iter()).map(|(a, b)| (*a as f32 / 32767.0 - b.clamp(-1.0, 1.0)).abs()).fold(0f32, f32::max);
        assert!(err < 1.5 / 32767.0, "differenza dal segnale master: {err}");
        let _ = std::fs::remove_file(&path);
    }

    /// brano con la griglia a 120 bpm (un battito ogni 0,5 s)
    fn grid_deck(sr: u32) -> Deck {
        let mut d = Deck::new(sr, 0);
        let track = Track { name: "g".into(), path: String::new(), data: Arc::new(tone(sr, 120.0, 220.0)), sr, beats: (0..238).map(|k| k as f64 * 0.5).collect(), bpm: 120.0, key: None, downbeat: 0 };
        d.load(Arc::new(track));
        d.playing = true;
        d.keylock = false;
        d.quantize = true;
        d
    }
    fn run_blocks(d: &mut Deck, secs: f64, sr: u32) {
        let mut buf = vec![0f32; 512];
        for _ in 0..((secs * sr as f64) / 256.0).round() as usize { d.render(&mut buf, 1.0); }
    }

    /// hot cue su un kick: il transiente non deve perdere energia, con e senza key lock
    #[test]
    fn hot_cue_keeps_the_kick_attack() {
        let sr = 44100u32;
        // kick = seno a 60 Hz con attacco immediato e coda di 150 ms, uno ogni mezzo secondo
        let n = (sr as f64 * 60.0) as usize;
        let mut data = vec![0f32; n * 2];
        for b in 0..120 {
            let s0 = (b as f64 * 0.5 * sr as f64) as usize;
            for k in 0..(0.15 * sr as f64) as usize {
                let t = k as f64 / sr as f64;
                let v = ((2.0 * std::f64::consts::PI * 60.0 * t).sin() * (-t * 25.0).exp()) as f32 * 0.9;
                if s0 + k < n { data[(s0 + k) * 2] = v; data[(s0 + k) * 2 + 1] = v; }
            }
        }
        for keylock in [false, true] {
            let mut d = Deck::new(sr, 0);
            let track = Track { name: "k".into(), path: String::new(), data: Arc::new(data.clone()), sr, beats: (0..119).map(|k| k as f64 * 0.5).collect(), bpm: 120.0, key: None, downbeat: 0 };
            d.load(Arc::new(track));
            d.keylock = keylock;
            d.quantize = true;
            d.hot_cues[0] = Some(20.0);
            d.set_pos_sec(7.3);
            d.playing = true;
            run_blocks(&mut d, 0.5, sr);
            d.hot_cue_press(0);
            let mut out = Vec::new();
            let mut buf = vec![0f32; 512];
            for _ in 0..32 { buf.iter_mut().for_each(|x| *x = 0.0); d.render(&mut buf, 1.0); out.extend_from_slice(&buf); }
            // primi 20 ms di uscita dopo il salto, contro lo stesso kick in riproduzione continua
            let w = (0.020 * sr as f64) as usize * 2;
            let peak = out[..w].iter().fold(0f32, |m, v| m.max(v.abs()));
            // riferimento: stesso kick attraversato in riproduzione continua, senza salto
            let mut c = Deck::new(sr, 0);
            c.load(Arc::new(Track { name: "k".into(), path: String::new(), data: Arc::new(data.clone()), sr, beats: vec![], bpm: 120.0, key: None, downbeat: 0 }));
            c.keylock = keylock; c.set_pos_sec(19.0); c.playing = true;
            let mut cont = Vec::new();
            for _ in 0..((1.2 * sr as f64) / 256.0) as usize { buf.iter_mut().for_each(|x| *x = 0.0); c.render(&mut buf, 1.0); cont.extend_from_slice(&buf); }
            let a0 = (1.0 * sr as f64) as usize * 2;
            let base = cont[a0..a0 + w].iter().fold(0f32, |m, v| m.max(v.abs()));
            assert!(peak > base * 0.95, "keylock={keylock}: il kick perde l'attacco, picco {peak} contro {base} in riproduzione continua");
        }
    }

    /// beatjump: salta di battiti interi restando in fase, e porta con se' il loop
    #[test]
    fn beatjump_keeps_phase_and_moves_loop() {
        let sr = 44100;
        let mut d = grid_deck(sr);
        d.playing = false;
        d.set_pos_sec(10.2);
        d.beatjump(4.0);
        assert!((d.pos_sec() - 12.2).abs() < 0.01, "avanti di 4 battiti: {}", d.pos_sec());
        d.beatjump(-1.0);
        assert!((d.pos_sec() - 11.7).abs() < 0.01, "indietro di 1 battito: {}", d.pos_sec());
        d.beatjump(0.5);
        assert!((d.pos_sec() - 11.95).abs() < 0.01, "mezzo battito: {}", d.pos_sec());
        d.set_pos_sec(20.0);
        d.loop_beats = 2.0;
        d.loop_auto();
        let (a, b) = (d.loop_in, d.loop_out);
        d.beatjump(8.0);
        assert!((d.loop_in - a - 4.0 * sr as f64).abs() < 1.0 && (d.loop_out - b - 4.0 * sr as f64).abs() < 1.0, "il loop segue il salto");
        d.beatjump(-1000.0);
        assert!(d.pos_sec() >= 0.0);
    }

    /// loop roll: ripete finche' e' tenuto; con SLIP al rilascio si ritorna dove il brano sarebbe arrivato
    #[test]
    fn roll_with_slip_resumes_on_the_virtual_position() {
        let sr = 44100;
        for slip in [true, false] {
            let mut d = grid_deck(sr);
            d.slip = slip;
            d.set_pos_sec(10.0);
            run_blocks(&mut d, 0.1, sr);
            let start = d.pos_sec();
            d.roll_on(0.5); // mezzo battito = 0,25 s
            let (a, b) = (d.loop_in / sr as f64, d.loop_out / sr as f64);
            assert!((b - a - 0.25).abs() < 0.01, "lunghezza del roll {}", b - a);
            run_blocks(&mut d, 1.0, sr);
            let p = d.pos_sec();
            assert!(p > a - 0.1 && p < b + 0.1, "durante il roll la testina resta nel loop: {p} ({a}..{b})");
            d.roll_off();
            assert!(!d.loop_on);
            if slip {
                assert!((d.pos_sec() - (start + 1.0)).abs() < 0.06, "al rilascio riprende dal punto virtuale: {} (atteso {})", d.pos_sec(), start + 1.0);
            } else {
                assert!(d.pos_sec() < b + 0.1, "senza slip riprende dal loop");
            }
            run_blocks(&mut d, 0.3, sr);
            assert!(d.pos_sec() > p || slip, "dopo il rilascio il brano riparte");
        }
    }

    /// freno, spinback e reverse: la velocita' cambia davvero e al rilascio (slip) si torna sul punto virtuale
    #[test]
    fn brake_spin_and_reverse_motions() {
        let sr = 44100;
        // freno: si ferma, resta fermo, riparte
        let mut d = grid_deck(sr);
        d.set_pos_sec(10.0);
        run_blocks(&mut d, 0.1, sr);
        let t0 = d.pos_sec();
        d.brake_on(0.5);
        run_blocks(&mut d, 0.8, sr);
        let stopped = d.pos_sec();
        run_blocks(&mut d, 0.3, sr);
        assert!((d.pos_sec() - stopped).abs() < 0.002, "fermo");
        assert!(stopped - t0 < 0.4, "il freno ha fermato il disco dopo meno di 0,4 s di avanzamento: {}", stopped - t0);
        d.motion_off();
        run_blocks(&mut d, 0.5, sr);
        assert_eq!(d.motion, Motion::None);
        let virt = t0 + 0.8 + 0.3 + 0.0;
        assert!(d.pos_sec() > virt - 0.1, "con slip riprende dal punto virtuale: {} (atteso almeno {})", d.pos_sec(), virt);
        // spinback: va all'indietro e poi riparte
        let mut d = grid_deck(sr);
        d.set_pos_sec(30.0);
        run_blocks(&mut d, 0.1, sr);
        let t0 = d.pos_sec();
        d.spin_on(1.0);
        run_blocks(&mut d, 0.2, sr);
        assert!(d.pos_sec() < t0 - 0.2, "lo spinback va all'indietro: {} -> {}", t0, d.pos_sec());
        run_blocks(&mut d, 1.2, sr);
        assert_eq!(d.motion, Motion::None, "lo spinback finisce da solo");
        assert!((d.pos_sec() - (t0 + 1.4)).abs() < 0.3, "riparte vicino al punto virtuale: {} (atteso ~{})", d.pos_sec(), t0 + 1.4);
        // reverse
        let mut d = grid_deck(sr);
        d.set_pos_sec(40.0);
        run_blocks(&mut d, 0.1, sr);
        let t0 = d.pos_sec();
        d.reverse_on();
        run_blocks(&mut d, 1.0, sr);
        assert!(d.pos_sec() < t0 - 0.5, "reverse: {} -> {}", t0, d.pos_sec());
        d.motion_off();
        run_blocks(&mut d, 0.4, sr);
        assert_eq!(d.motion, Motion::None);
        assert!(d.pos_sec() > t0 + 0.9, "dopo il reverse si riprende dal punto virtuale: {}", d.pos_sec());
    }

    /// i valori del loop sono battiti puliti (4, 2, 1/2...) anche segnando IN e OUT a mano su una griglia irregolare
    /// OUT con il loop attivo ne esce senza toccare i punti; un nuovo OUT (dopo IN) lo ridefinisce
    #[test]
    fn loop_out_while_looping_exits_instead_of_collapsing() {
        let sr = 44100; let mut d = grid_deck(sr);
        d.set_pos_sec(10.0); d.loop_in_here(); run_blocks(&mut d, 2.0, sr); d.loop_out_here();
        assert!(d.loop_on && d.loop_beats == 4.0);
        let out = d.loop_out;
        for k in 0..4 { run_blocks(&mut d, 0.37 + 0.11 * k as f64, sr); d.loop_out_here(); d.loop_toggle(); }
        assert!(d.loop_on && d.loop_out == out && d.loop_beats == 4.0, "il loop non deve accorciarsi: {} battiti", d.loop_beats);
        d.loop_out_here();
        assert!(!d.loop_on && d.loop_out == out, "OUT in loop ne esce senza cambiare i punti");
        run_blocks(&mut d, 3.0, sr);
        assert!(d.pos_sec() > out / sr as f64 + 1.0, "uscito dal loop il brano prosegue oltre la fine: {}", d.pos_sec());
        d.loop_step(false); d.loop_step(true); d.loop_step(true);
        assert_eq!(d.loop_beats, 8.0);
    }

    #[test]
    fn loop_halve_and_double_stay_on_powers_of_two() {
        let mut d = grid_deck(44100);
        for (start, halved, doubled) in [(4.0, 2.0, 8.0), (3.0, 2.0, 4.0), (6.0, 4.0, 8.0), (5.0, 4.0, 8.0), (0.75, 0.5, 1.0), (0.03125, 0.03125, 0.0625), (64.0, 32.0, 64.0)] {
            d.loop_beats = start; d.loop_step(false);
            assert_eq!(d.loop_beats, halved, "dimezzare {start}");
            d.loop_beats = start; d.loop_step(true);
            assert_eq!(d.loop_beats, doubled, "raddoppiare {start}");
        }
    }

    #[test]
    fn loop_lengths_are_clean_beat_values() {
        let sr = 44100;
        let mut d = Deck::new(sr, 0);
        // griglia a 120 bpm con piccole irregolarita' come in un brano suonato dal vivo
        let beats: Vec<f64> = (0..238).map(|k| k as f64 * 0.5 + 0.006 * (k as f64 * 1.7).sin()).collect();
        d.load(Arc::new(Track { name: "g".into(), path: String::new(), data: Arc::new(tone(sr, 120.0, 220.0)), sr, beats, bpm: 120.0, key: None, downbeat: 0 }));
        // il deck adatta la griglia ai battiti rilevati (gridfit, modo AUTO): i loop vanno confrontati con la griglia adattata
        let b2 = d.track.as_ref().unwrap().beats.clone();
        d.playing = false;
        d.quantize = true;
        for (n, len) in [(4usize, 4.0f64), (2, 2.0), (1, 1.0), (8, 8.0), (16, 16.0)] {
            d.loop_on = false; d.loop_out = 0.0; d.loop_in = 0.0;
            d.set_pos_sec(b2[40] + 0.01);
            d.loop_in_here();
            d.set_pos_sec(b2[40 + n] + 0.01);
            d.loop_out_here();
            assert!(d.loop_on);
            assert_eq!(d.loop_beats, len, "loop di {n} battiti segnato a mano");
        }
        // dimezza e raddoppia: restano valori puliti e la fine segue la griglia
        d.loop_size(d.loop_beats / 2.0);
        assert_eq!(d.loop_beats, 8.0);
        assert!((d.loop_out / sr as f64 - b2[48]).abs() < 0.002, "fine del loop sulla griglia: {} vs {}", d.loop_out / sr as f64, b2[48]);
        d.loop_size(d.loop_beats * 2.0);
        assert_eq!(d.loop_beats, 16.0);
        // loop automatico da 4 battiti = 4 battiti di griglia
        d.loop_on = false;
        d.set_pos_sec(b2[60] + 0.02);
        d.loop_beats = 4.0;
        d.loop_auto();
        assert!((d.loop_out / sr as f64 - b2[64]).abs() < 0.002);
        assert_eq!(snap_beats(4.01), 4.0);
        assert_eq!(snap_beats(2.0001), 2.0);
        assert_eq!(snap_beats(0.49), 0.5);
        assert_eq!(snap_beats(6.0), 6.0);
        assert_eq!(snap_beats(3.37), 3.37);
    }

    /// SYNC + cue: premere play su un battito non salta avanti di mezzo battito: aspetta il battito della guida e parte dal cue
    #[test]
    fn plain_play_starts_at_once_and_quantized_play_waits_for_the_leader_beat() {
        let sr = 44100;
        let mk = || {
            let track = Track { name: "g".into(), path: String::new(), data: Arc::new(tone(sr, 120.0, 220.0)), sr, beats: (0..238).map(|k| k as f64 * 0.5).collect(), bpm: 120.0, key: None, downbeat: 0 };
            Arc::new(track)
        };
        let setup = || {
            let mut m = Mixer::new(sr);
            m.decks[0].load(mk());
            m.decks[0].keylock = false;
            m.decks[0].set_pos_sec(20.25); // fase 0,5 del battito
            m.decks[0].playing = true;
            m.decks[1].load(mk());
            m.decks[1].keylock = false;
            m.decks[1].quantize = true;
            m.decks[1].set_pos_sec(10.0);
            m.decks[1].cue_point = 10.0;
            m.decks[1].sync = true;
            m
        };
        let mut buf = vec![0f32; 256];
        let ph = |d: &Deck| d.track.as_ref().unwrap().beat_phase(d.pos_sec()).unwrap();

        // play normale: parte subito, dal punto in cui si e', senza attendere il battito ne' saltare avanti
        let mut m = setup();
        for _ in 0..8 { m.render(&mut buf); }
        m.decks[1].play_toggle();
        for _ in 0..4 { m.render(&mut buf); }
        assert!(m.decks[1].playing && !m.decks[1].armed, "deve partire subito");
        let p = m.decks[1].pos_sec();
        assert!((10.0..10.02).contains(&p), "parte dal punto scelto, senza saltare: {p}");
        // con il SYNC la fase si porta in linea da sola (variazione di velocita' limitata al 4%: mezzo battito richiede qualche secondo)
        for _ in 0..(20 * sr as usize / 128) { m.render(&mut buf); }
        let mut e = ph(&m.decks[1]) - ph(&m.decks[0]);
        if e > 0.5 { e -= 1.0 } else if e < -0.5 { e += 1.0 }
        assert!(e.abs() < 0.04, "dopo qualche secondo i due deck sono in fase: {e}");

        // play quantizzato: aspetta il battito della guida e parte esattamente dal punto scelto
        let mut m = setup();
        for _ in 0..8 { m.render(&mut buf); }
        m.decks[1].play_quantized();
        for _ in 0..4 { m.render(&mut buf); }
        assert!(m.decks[1].armed && !m.decks[1].playing, "in attesa del battito");
        let mut blocks = 0;
        while m.decks[1].armed && blocks < 400 { m.render(&mut buf); blocks += 1; }
        assert!(m.decks[1].playing, "deve partire da solo");
        // il buffer ha 256 campioni = 128 frame per blocco
        let waited = (blocks + 4) as f64 * 128.0 / sr as f64;
        assert!((0.2..0.32).contains(&waited), "attesa fino al battito: {waited}");
        let p = m.decks[1].pos_sec();
        assert!((p - 10.0).abs() < 0.03, "parte dal cue, non mezzo battito dopo: {p}");
        for _ in 0..200 { m.render(&mut buf); }
        let mut e = ph(&m.decks[1]) - ph(&m.decks[0]);
        if e > 0.5 { e -= 1.0 } else if e < -0.5 { e += 1.0 }
        assert!(e.abs() < 0.04, "i due deck sono in fase: {e}");
        // un secondo clic su play durante l'attesa la annulla
        let mut d = Deck::new(sr, 0);
        d.load(mk());
        d.armed = true;
        d.play_toggle();
        assert!(!d.armed && !d.playing);
    }

    /// meter di canale: livelli sinistro e destro dopo il fader, che reagiscono al volume e cadono nel tempo
    #[test]
    fn channel_meters_follow_the_fader_and_fall() {
        let sr = 44100;
        let mut d = Deck::new(sr, 0);
        // sinistra forte (0,5), destra debole (0,1)
        let data: Vec<f32> = (0..30 * sr as usize).flat_map(|i| { let v = (2.0 * std::f32::consts::PI * 220.0 * i as f32 / sr as f32).sin(); [0.5 * v, 0.1 * v] }).collect();
        d.load(Arc::new(Track { name: "l".into(), path: String::new(), data: Arc::new(data), sr, beats: vec![], bpm: 120.0, key: None, downbeat: 0 }));
        d.keylock = false;
        d.playing = true;
        let mut buf = vec![0f32; 256];
        for _ in 0..100 { d.render(&mut buf, 1.0); }
        assert!((d.lvl[0] - 0.5).abs() < 0.03 && (d.lvl[1] - 0.1).abs() < 0.02, "fader aperto: {:?}", d.lvl);
        // fader a meta' (guadagno 0,25): il meter scende di conseguenza
        for _ in 0..200 { d.render(&mut buf, 0.25); }
        assert!((d.lvl[0] - 0.125).abs() < 0.02, "fader a meta': {}", d.lvl[0]);
        // fader a zero: il meter si svuota anche se il brano suona
        for _ in 0..400 { d.render(&mut buf, 0.0); }
        assert!(d.lvl[0] < 0.003 && d.lvl[1] < 0.003, "fader chiuso: {:?}", d.lvl);
        // brano fermo: il picco cade nel tempo
        for _ in 0..100 { d.render(&mut buf, 1.0); }
        d.playing = false;
        for _ in 0..300 { d.render(&mut buf, 1.0); }
        assert!(d.lvl[0] < 0.1, "il picco scende quando il segnale finisce: {}", d.lvl[0]);
    }

    /// Costo del motore per blocco audio con e senza SYNC (con SYNC il tempo e' sempre un po' diverso e si usa lo stiramento temporale).
    /// cargo test --release bench_render_sync -- --ignored --nocapture
    #[test]
    #[ignore]
    fn bench_render_sync() {
        let sr = 44100;
        let noisy = |bpm: f64| {
            let mut x = 12345u32;
            let n = 120 * sr as usize;
            let data: Vec<f32> = (0..n).flat_map(|i| {
                x = x.wrapping_mul(1664525).wrapping_add(1013904223);
                let r = ((x >> 16) as f32 / 32768.0 - 1.0) * 0.1;
                let v = (2.0 * std::f32::consts::PI * 220.0 * i as f32 / sr as f32).sin() * 0.3 + r;
                [v, v]
            }).collect();
            let per = 60.0 / bpm;
            Arc::new(Track { name: "b".into(), path: String::new(), data: Arc::new(data), sr, beats: (0..(120.0 / per) as usize).map(|k| k as f64 * per).collect(), bpm, key: None, downbeat: 0 })
        };
        for (label, sync) in [("senza SYNC (tempo 0)", false), ("con SYNC (tempo -3%, stiramento)", true)] {
            let mut m = Mixer::new(sr);
            m.decks[0].load(noisy(130.0));
            m.decks[1].load(noisy(134.0));
            m.decks[0].playing = true;
            m.decks[1].playing = true;
            if sync { m.decks[1].sync = true; m.decks[0].sync = false; }
            let mut buf = vec![0f32; 1024];
            for _ in 0..200 { m.render(&mut buf); } // assestamento
            let mut times = vec![];
            for _ in 0..3000 {
                let t0 = std::time::Instant::now();
                m.render(&mut buf);
                times.push(t0.elapsed().as_secs_f64() * 1000.0);
            }
            times.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let avg = times.iter().sum::<f64>() / times.len() as f64;
            let block_ms = 512.0 / sr as f64 * 1000.0;
            println!("{label}: blocco da {block_ms:.1} ms -> medio {avg:.2} ms ({:.0}% di un core), 99% {:.2} ms, massimo {:.2} ms, tempo deck2 {:+.2}%", avg / block_ms * 100.0, times[times.len() * 99 / 100], times[times.len() - 1], m.decks[1].tempo * 100.0);
        }
    }
}
