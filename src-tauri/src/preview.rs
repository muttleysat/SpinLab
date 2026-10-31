//! Anteprima dei brani (ascolto in cuffia): uscita audio separata dal mixer, su un dispositivo a scelta.
//! Il flusso dell'anteprima non passa dal master, quindi non si sente nel mix.

use crate::cue::{CueBus, Puller};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, SampleFormat, SizedSample, StreamConfig};
use parking_lot::Mutex;
use serde::Serialize;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{mpsc, Arc};

#[derive(Default)]
struct PState {
    /// audio stereo interleaved alla frequenza del dispositivo di anteprima
    data: Option<Arc<Vec<f32>>>,
    pos: f64,
    playing: bool,
    path: String,
    gain: f32,
    /// dopo un salto di posizione la dissolvenza in entrata riparte: niente click
    reset_fade: bool,
}

#[derive(Serialize)]
pub struct PreviewState {
    pub path: String,
    pub playing: bool,
    pub pos: f64,
    pub duration: f64,
}

enum Cmd {
    Open(Option<String>, mpsc::Sender<Result<u32, String>>),
}

/// Valore speciale del dispositivo cuffie: stesso dispositivo dell'uscita principale, sui canali 3-4
/// (per i controller con scheda audio integrata: master sui canali 1-2, cuffie sui canali 3-4).
pub const SAME_DEVICE: &str = "@main";

/// Genera l'audio delle cuffie (preascolto dei canali + anteprima dei brani): lo usano sia l'uscita
/// dedicata sia, in modo "stesso dispositivo", il flusso principale.
pub struct CueRender {
    st: Arc<Mutex<PState>>,
    bus: Arc<CueBus>,
    fade: f32,
    puller: Puller,
    cue: Vec<f32>,
}

impl CueRender {
    fn new(st: Arc<Mutex<PState>>, bus: Arc<CueBus>) -> Self {
        CueRender { st, bus, fade: 0.0, puller: Puller::default(), cue: Vec::new() }
    }

    /// riempie `tmp` (stereo interleaved, `frames` frame) alla frequenza `sr`
    pub fn render(&mut self, frames: usize, sr: u32, tmp: &mut Vec<f32>) {
        // preascolto dei canali (PFL) e mix cuffie dal mixer
        tmp.resize(frames * 2, 0.0);
        self.cue.resize(frames * 2, 0.0);
        self.puller.pull(&self.bus, &mut self.cue, sr);
        tmp.copy_from_slice(&self.cue);
        if let Some(mut s) = self.st.try_lock() {
            if s.reset_fade {
                self.fade = 0.0;
                s.reset_fade = false;
            }
            if let (true, Some(d)) = (s.playing, s.data.clone()) {
                let total = d.len() / 2;
                for i in 0..frames {
                    let p = s.pos as usize;
                    if p >= total {
                        s.playing = false;
                        break;
                    }
                    self.fade = (self.fade + 1.0 / 256.0).min(1.0);
                    // ultimo mezzo secondo: dissolvenza in uscita
                    let tail = ((total - p) as f32 / 4096.0).min(1.0);
                    let g = s.gain * self.fade * tail;
                    tmp[i * 2] += d[p * 2] * g;
                    tmp[i * 2 + 1] += d[p * 2 + 1] * g;
                    s.pos += 1.0;
                }
            } else {
                self.fade = 0.0;
            }
        }
        for v in tmp.iter_mut() {
            *v = v.clamp(-1.0, 1.0);
        }
    }
}

/// Punto di aggancio fra il flusso principale e le cuffie "sullo stesso dispositivo".
#[derive(Default)]
pub struct InlineCue {
    /// il flusso principale deve scrivere le cuffie sui canali 3-4
    pub on: std::sync::atomic::AtomicBool,
    /// canali e frequenza del flusso principale aperto
    pub channels: AtomicU32,
    pub sr: AtomicU32,
    r: Mutex<Option<CueRender>>,
}

impl InlineCue {
    /// chiamato dal thread audio principale: scrive le cuffie in `tmp`; false se non attivo
    pub fn render(&self, frames: usize, sr: u32, tmp: &mut Vec<f32>) -> bool {
        if !self.on.load(Ordering::Relaxed) {
            return false;
        }
        match self.r.try_lock() {
            Some(mut g) => match g.as_mut() {
                Some(r) => {
                    r.render(frames, sr, tmp);
                    true
                }
                None => false,
            },
            None => false,
        }
    }
}

pub struct Previewer {
    tx: Mutex<mpsc::Sender<Cmd>>,
    state: Arc<Mutex<PState>>,
    sr: AtomicU32,
    /// nome del dispositivo scelto (None = predefinito di sistema)
    pub device: Mutex<Option<String>>,
}

impl Previewer {
    /// Avvia il thread dell'uscita di anteprima. Se il dispositivo non si apre l'anteprima resta spenta (sr = 0).
    pub fn start(device: Option<String>, bus: Arc<CueBus>, inline: Arc<InlineCue>) -> Previewer {
        let (tx, rx) = mpsc::channel::<Cmd>();
        let state = Arc::new(Mutex::new(PState { gain: 1.0, ..Default::default() }));
        let st = state.clone();
        *inline.r.lock() = Some(CueRender::new(state.clone(), bus.clone()));
        std::thread::spawn(move || {
            let host = cpal::default_host();
            let mut stream: Option<cpal::Stream> = None;
            while let Ok(Cmd::Open(dev, reply)) = rx.recv() {
                drop(stream.take());
                inline.on.store(false, Ordering::SeqCst);
                if dev.as_deref() == Some(SAME_DEVICE) {
                    // niente flusso dedicato: il principale scrive le cuffie sui canali 3-4 (serve un'uscita con almeno 4 canali)
                    let (ch, sr) = (inline.channels.load(Ordering::SeqCst), inline.sr.load(Ordering::SeqCst));
                    if ch >= 4 && sr > 0 {
                        inline.on.store(true, Ordering::SeqCst);
                        let _ = reply.send(Ok(sr));
                    } else {
                        let _ = reply.send(Err(format!("l'uscita principale ha solo {ch} canali: per le cuffie sui canali 3-4 ne servono almeno 4")));
                    }
                    continue;
                }
                match open(&host, dev.as_deref(), st.clone(), bus.clone()) {
                    Ok((s, sr)) => {
                        stream = Some(s);
                        let _ = reply.send(Ok(sr));
                    }
                    Err(e) => {
                        let _ = reply.send(Err(e));
                    }
                }
            }
        });
        let p = Previewer { tx: Mutex::new(tx), state, sr: AtomicU32::new(0), device: Mutex::new(device.clone()) };
        let _ = p.reopen(device);
        p
    }

    /// Riapre l'uscita su un altro dispositivo. Ritorna la frequenza di campionamento.
    pub fn reopen(&self, device: Option<String>) -> Result<u32, String> {
        let (rtx, rrx) = mpsc::channel();
        self.tx.lock().send(Cmd::Open(device.clone(), rtx)).map_err(|e| e.to_string())?;
        let res = rrx.recv().map_err(|e| e.to_string())?;
        *self.device.lock() = device;
        match res {
            Ok(sr) => {
                self.sr.store(sr, Ordering::SeqCst);
                // l'audio gia' in memoria e' alla vecchia frequenza: si ferma
                let mut s = self.state.lock();
                s.playing = false;
                s.data = None;
                Ok(sr)
            }
            Err(e) => {
                self.sr.store(0, Ordering::SeqCst);
                Err(e)
            }
        }
    }

    pub fn sample_rate(&self) -> u32 {
        self.sr.load(Ordering::SeqCst)
    }

    /// Avvia l'anteprima di un brano gia' decodificato alla frequenza `sample_rate()`.
    pub fn play(&self, path: &str, data: Vec<f32>, start_frac: f64, gain: f32) {
        let frames = data.len() / 2;
        let mut s = self.state.lock();
        s.data = Some(Arc::new(data));
        s.pos = (start_frac.clamp(0.0, 0.98) * frames as f64).floor();
        s.path = path.to_string();
        s.gain = gain;
        s.playing = true;
        s.reset_fade = true;
    }
    pub fn stop(&self) {
        self.state.lock().playing = false;
    }
    pub fn seek(&self, frac: f64) {
        let mut s = self.state.lock();
        let frames = s.data.as_ref().map_or(0, |d| d.len() / 2);
        s.pos = (frac.clamp(0.0, 0.98) * frames as f64).floor();
        s.reset_fade = true;
    }
    pub fn state(&self) -> PreviewState {
        let s = self.state.lock();
        let sr = self.sample_rate().max(1) as f64;
        PreviewState {
            path: s.path.clone(),
            playing: s.playing,
            pos: s.pos / sr,
            duration: s.data.as_ref().map_or(0.0, |d| (d.len() / 2) as f64 / sr),
        }
    }
}

fn open(host: &cpal::Host, device: Option<&str>, st: Arc<Mutex<PState>>, bus: Arc<CueBus>) -> Result<(cpal::Stream, u32), String> {
    let default = host.default_output_device();
    let dev = match crate::audio::chosen(device) {
        Some(n) => host.output_devices().map_err(|e| e.to_string())?.find(|d| d.name().ok().as_deref() == Some(n)).ok_or_else(|| format!("dispositivo «{n}» non trovato"))?,
        None => default.ok_or("nessun dispositivo audio di uscita")?,
    };
    let dcfg = dev.default_output_config().map_err(|e| e.to_string())?;
    let sr = dcfg.sample_rate().0;
    let channels = dcfg.channels();
    let cfg = StreamConfig { channels, sample_rate: dcfg.sample_rate(), buffer_size: cpal::BufferSize::Default };
    let stream = match dcfg.sample_format() {
        SampleFormat::F32 => build::<f32>(&dev, &cfg, channels as usize, st, bus, sr),
        SampleFormat::I16 => build::<i16>(&dev, &cfg, channels as usize, st, bus, sr),
        SampleFormat::U16 => build::<u16>(&dev, &cfg, channels as usize, st, bus, sr),
        SampleFormat::I32 => build::<i32>(&dev, &cfg, channels as usize, st, bus, sr),
        f => Err(format!("formato audio non supportato: {f:?}")),
    }?;
    stream.play().map_err(|e| e.to_string())?;
    Ok((stream, sr))
}

fn build<T: SizedSample + FromSample<f32>>(
    dev: &cpal::Device,
    cfg: &StreamConfig,
    ch: usize,
    st: Arc<Mutex<PState>>,
    bus: Arc<CueBus>,
    sr: u32,
) -> Result<cpal::Stream, String> {
    let mut r = CueRender::new(st, bus);
    let mut tmp = Vec::<f32>::new();
    dev.build_output_stream(
        cfg,
        move |data: &mut [T], _| {
            let frames = data.len() / ch;
            r.render(frames, sr, &mut tmp);
            for (i, f) in data.chunks_mut(ch).enumerate() {
                for (c, s) in f.iter_mut().enumerate() {
                    *s = T::from_sample(if c < 2 { tmp[i * 2 + c] } else { 0.0 });
                }
            }
        },
        |e| eprintln!("errore audio anteprima: {e}"),
        None,
    )
    .map_err(|e| e.to_string())
}
