//! Uscita audio (cpal) su thread dedicato, riconfigurabile a caldo (dispositivo, sample rate, buffer).

use crate::engine::Mixer;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{BufferSize, FromSample, SampleFormat, SampleRate, SizedSample, StreamConfig};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::{mpsc, Arc};

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct Settings {
    /// driver audio: "asio" oppure "wasapi" (None = predefinito di Windows)
    #[serde(default)]
    pub driver: Option<String>,
    /// nome del dispositivo di uscita (None = predefinito di sistema)
    pub device: Option<String>,
    /// frequenza di campionamento (None = quella del dispositivo)
    pub sample_rate: Option<u32>,
    /// dimensione buffer in frame (None/0 = automatica)
    pub buffer: Option<u32>,
    /// dispositivo per l'anteprima dei brani in cuffia (None = lo stesso predefinito di sistema)
    #[serde(default)]
    pub preview_device: Option<String>,
}

#[derive(Serialize, Clone, Default)]
pub struct AudioInfo {
    /// blocchi audio elaborati e blocchi saltati (mixer occupato) dall'apertura del flusso
    pub rendered: u64,
    pub missed: u64,
    pub entered: u64,
    pub exited: u64,
    pub phase: u32,
    /// frequenze supportate dal dispositivo aperto (calcolate all'apertura: interrogare di nuovo un driver ASIO in uso lo ferma)
    pub rates: Vec<u32>,
    pub driver: String,
    pub device: String,
    pub sample_rate: u32,
    /// buffer richiesto (0 = automatico)
    pub buffer: u32,
    pub channels: u16,
    /// frame per callback realmente osservati
    pub actual_buffer: u32,
    pub latency_ms: f64,
    pub error: Option<String>,
}

#[derive(Serialize)]
pub struct AudioList {
    /// driver disponibili (WASAPI, ASIO se installato) e quello in uso
    pub hosts: Vec<String>,
    pub driver: String,
    pub devices: Vec<String>,
    pub default_device: String,
    pub rates: Vec<u32>,
    pub current: AudioInfo,
}

/// contatori del callback audio: servono a capire se il driver sta davvero richiamando l'audio
#[derive(Default)]
pub struct Counters {
    pub rendered: AtomicU64,
    pub missed: AtomicU64,
    /// diagnostica: callback entrati/usciti e fase in cui si trova (1 try_lock, 2 render, 3 conversione)
    pub entered: AtomicU64,
    pub exited: AtomicU64,
    pub phase: AtomicU32,
    /// cuffie sullo stesso dispositivo (canali 3-4)
    pub inline: Arc<crate::preview::InlineCue>,
}

enum Cmd {
    Open(Settings, mpsc::Sender<Result<AudioInfo, String>>),
}

pub struct Audio {
    pub inline: Arc<crate::preview::InlineCue>,
    tx: Mutex<mpsc::Sender<Cmd>>,
    pub mixer: Arc<Mutex<Mixer>>,
    info: Mutex<AudioInfo>,
    observed: Arc<AtomicU32>,
    counters: Arc<Counters>,
}

impl Audio {
    pub fn start(settings: &Settings) -> Result<Audio, String> {
        let (tx, rx) = mpsc::channel::<Cmd>();
        let (first_tx, first_rx) = mpsc::channel::<Result<(AudioInfo, Arc<Mutex<Mixer>>, Arc<AtomicU32>, Arc<Counters>), String>>();
        let first = settings.clone();
        let counters = Arc::new(Counters::default());
        let thread_counters = counters.clone();
        std::thread::spawn(move || {
            let counters = thread_counters;
            let observed = Arc::new(AtomicU32::new(0));
            let mut mixer: Option<Arc<Mutex<Mixer>>> = None;
            let mut stream: Option<cpal::Stream> = None;
            let mut pending = Some(first);
            let mut first_tx = Some(first_tx);
            loop {
                let (st, reply): (Settings, Option<mpsc::Sender<Result<AudioInfo, String>>>) = match pending.take() {
                    Some(s) => (s, None),
                    None => match rx.recv() {
                        Ok(Cmd::Open(s, r)) => (s, Some(r)),
                        Err(_) => return,
                    },
                };
                drop(stream.take()); // chiude il flusso precedente prima di riaprire
                let res = match attempt(&st, &mut mixer, &observed, &counters) {
                    Ok(r) => Ok(r),
                    Err(e) => {
                        // mai restare muti: se il driver scelto non parte (o non richiama l'audio) si torna a Windows (WASAPI)
                        let what = st.driver.as_deref().map_or("l'uscita scelta".to_string(), |d| d.to_uppercase());
                        let fb = Settings { driver: None, device: None, sample_rate: None, buffer: None, preview_device: st.preview_device.clone() };
                        match attempt(&fb, &mut mixer, &observed, &counters) {
                            Ok((s, mut info)) => {
                                info.error = Some(format!("{what} non utilizzabile ({e}): uso Windows (WASAPI)"));
                                Ok((s, info))
                            }
                            Err(e2) => Err(format!("{e}; anche WASAPI non parte: {e2}")),
                        }
                    }
                };
                match res {
                    Ok((s, info)) => {
                        stream = Some(s);
                        if let Some(r) = reply {
                            let _ = r.send(Ok(info.clone()));
                        }
                        if let Some(f) = first_tx.take() {
                            let _ = f.send(Ok((info, mixer.clone().unwrap(), observed.clone(), counters.clone())));
                        }
                    }
                    Err(e) => {
                        if let Some(r) = reply {
                            let _ = r.send(Err(e.clone()));
                        }
                        if let Some(f) = first_tx.take() {
                            let _ = f.send(Err(e));
                            return;
                        }
                    }
                }
                let _ = &stream;
            }
        });
        let (info, mixer, observed, counters) = first_rx.recv().map_err(|e| e.to_string())??;
        let _ = &counters;
        Ok(Audio { inline: counters.inline.clone(), tx: Mutex::new(tx), mixer, info: Mutex::new(info), observed, counters })
    }

    pub fn reconfigure(&self, st: Settings) -> Result<AudioInfo, String> {
        let (rtx, rrx) = mpsc::channel();
        self.tx.lock().send(Cmd::Open(st, rtx)).map_err(|e| e.to_string())?;
        let info = rrx.recv().map_err(|e| e.to_string())??;
        *self.info.lock() = info.clone();
        Ok(info)
    }

    pub fn current(&self) -> AudioInfo {
        let mut i = self.info.lock().clone();
        i.actual_buffer = self.observed.load(Ordering::Relaxed);
        i.rendered = self.counters.rendered.load(Ordering::Relaxed);
        i.missed = self.counters.missed.load(Ordering::Relaxed);
        i.entered = self.counters.entered.load(Ordering::Relaxed);
        i.exited = self.counters.exited.load(Ordering::Relaxed);
        i.phase = self.counters.phase.load(Ordering::Relaxed);
        let frames = if i.buffer > 0 { i.buffer } else { i.actual_buffer };
        i.latency_ms = frames as f64 / i.sample_rate.max(1) as f64 * 1000.0;
        i
    }

    pub fn list(&self, driver: Option<String>) -> AudioList {
        let cur = self.current();
        let want = driver.as_deref().unwrap_or(&cur.driver).to_string();
        let hosts: Vec<String> = cpal::available_hosts().iter().map(|h| host_label(*h)).collect();
        let active_here = want.eq_ignore_ascii_case(&cur.driver);
        // ATTENZIONE: con ASIO elencare i dispositivi con cpal CARICA ogni driver installato, compreso quello in uso, e lo ferma
        // (l'audio muore). I nomi si leggono dal registro con asio-sys; le frequenze del dispositivo aperto sono quelle
        // memorizzate all'apertura e per gli altri non si chiede nulla.
        if want.eq_ignore_ascii_case("asio") {
            let devices = asio_sys::Asio::new().driver_names();
            let default_device = if active_here && !cur.device.is_empty() { cur.device.clone() } else { devices.first().cloned().unwrap_or_default() };
            let rates = if active_here { cur.rates.clone() } else { vec![] };
            return AudioList { hosts, driver: "ASIO".into(), devices, default_device, rates, current: cur };
        }
        let host = pick_host(Some(&want));
        let devices: Vec<String> = host.output_devices().map(|d| d.filter_map(|x| x.name().ok()).collect()).unwrap_or_default();
        let default_device = host.default_output_device().and_then(|d| d.name().ok()).unwrap_or_default();
        let name = if cur.device.is_empty() || !active_here { default_device.clone() } else { cur.device.clone() };
        let mut rates = vec![];
        if let Some(dev) = host.output_devices().ok().and_then(|mut d| d.find(|x| x.name().ok().as_deref() == Some(&name))) {
            if let Ok(cfgs) = dev.supported_output_configs() {
                let cfgs: Vec<_> = cfgs.filter(|c| c.channels() >= 2).collect();
                for r in [44100u32, 48000, 88200, 96000, 176400, 192000] {
                    if cfgs.iter().any(|c| c.min_sample_rate().0 <= r && c.max_sample_rate().0 >= r) {
                        rates.push(r);
                    }
                }
            }
        }
        AudioList { hosts, driver: host_label(host.id()), devices, default_device, rates, current: cur }
    }
}

/// Nome breve di un driver: "WASAPI", "ASIO"...
fn host_label(id: cpal::HostId) -> String {
    id.name().split_whitespace().next().unwrap_or("").to_uppercase()
}

/// Sceglie il driver: "asio" se richiesto e installato, altrimenti quello predefinito di Windows (WASAPI).
fn pick_host(driver: Option<&str>) -> cpal::Host {
    if let Some(d) = driver {
        if let Some(id) = cpal::available_hosts().into_iter().find(|h| host_label(*h).eq_ignore_ascii_case(d)) {
            if let Ok(h) = cpal::host_from_id(id) {
                return h;
            }
        }
    }
    cpal::default_host()
}

/// Apre l'uscita e controlla che il driver richiami davvero l'audio (un ASIO che si apre ma resta fermo renderebbe tutto muto).
fn attempt(
    st: &Settings,
    mixer: &mut Option<Arc<Mutex<Mixer>>>,
    observed: &Arc<AtomicU32>,
    counters: &Arc<Counters>,
) -> Result<(cpal::Stream, AudioInfo), String> {
    let host = pick_host(st.driver.as_deref());
    counters.rendered.store(0, Ordering::SeqCst);
    counters.missed.store(0, Ordering::SeqCst);
    let (s, info) = open(&host, st, mixer, observed.clone(), counters.clone())?;
    for _ in 0..14 {
        std::thread::sleep(std::time::Duration::from_millis(50));
        if counters.rendered.load(Ordering::SeqCst) > 0 {
            return Ok((s, info));
        }
    }
    Err("il driver non richiama l'audio".into())
}

fn open(
    host: &cpal::Host,
    st: &Settings,
    mixer: &mut Option<Arc<Mutex<Mixer>>>,
    observed: Arc<AtomicU32>,
    counters: Arc<Counters>,
) -> Result<(cpal::Stream, AudioInfo), String> {
    // il dispositivo predefinito si cerca solo se serve: con ASIO caricarlo occupa il primo driver installato e impedisce di aprire gli altri
    let dev = match &st.device {
        Some(n) => host
            .output_devices()
            .map_err(|e| e.to_string())?
            .find(|d| d.name().ok().as_deref() == Some(n))
            // niente ripiego silenzioso: con ASIO il «predefinito» e' il primo driver installato e si finirebbe su una scheda diversa da quella scelta
            .ok_or_else(|| format!("dispositivo «{n}» non trovato o non utilizzabile (occupato da un altro programma?)"))?,
        None => host.default_output_device().ok_or("nessun dispositivo audio di uscita")?,
    };
    let name = dev.name().unwrap_or_default();
    let dcfg = dev.default_output_config().map_err(|e| e.to_string())?;
    let sr = st.sample_rate.unwrap_or(dcfg.sample_rate().0);

    // scegli la configurazione supportata che contiene `sr`, preferendo f32 e il minor numero di canali >= 2
    let mut chosen = None;
    let mut rates: Vec<u32> = vec![];
    if let Ok(cfgs) = dev.supported_output_configs() {
        let all: Vec<_> = cfgs.collect();
        for r in [44100u32, 48000, 88200, 96000, 176400, 192000] {
            if all.iter().any(|c| c.channels() >= 2 && c.min_sample_rate().0 <= r && c.max_sample_rate().0 >= r) {
                rates.push(r);
            }
        }
        let cfgs = all.into_iter();
        // con le cuffie sullo stesso dispositivo servono almeno 4 canali (se non ci sono si ripiega su 2 e le cuffie segnalano l'errore)
        let fits = |c: &cpal::SupportedStreamConfigRange| c.min_sample_rate().0 <= sr && c.max_sample_rate().0 >= sr;
        let want_ch = if st.preview_device.as_deref() == Some(crate::preview::SAME_DEVICE) && cfgs.clone().any(|c| c.channels() >= 4 && fits(&c)) { 4 } else { 2 };
        let mut v: Vec<_> = cfgs.filter(|c| c.channels() >= want_ch && fits(c)).collect();
        v.sort_by_key(|c| (c.sample_format() != SampleFormat::F32, c.channels()));
        chosen = v.into_iter().next();
    }
    let (fmt, channels) = match &chosen {
        Some(c) => (c.sample_format(), c.channels()),
        None if st.sample_rate.is_some() => return Err(format!("{sr} Hz non supportato da «{name}»")),
        None => (dcfg.sample_format(), dcfg.channels()),
    };

    let m = mixer.get_or_insert_with(|| Arc::new(Mutex::new(Mixer::new(sr)))).clone();
    m.lock().change_sr(sr);
    counters.inline.channels.store(channels as u32, Ordering::SeqCst);
    counters.inline.sr.store(sr, Ordering::SeqCst);

    let want = st.buffer.filter(|&b| b > 0);
    let try_open = |buf: BufferSize| -> Result<cpal::Stream, String> {
        let cfg = StreamConfig { channels, sample_rate: SampleRate(sr), buffer_size: buf };
        let ch = channels as usize;
        let (m2, o2, c2) = (m.clone(), observed.clone(), counters.clone());
        match fmt {
            SampleFormat::F32 => build::<f32>(&dev, &cfg, ch, m2, o2, c2),
            SampleFormat::I16 => build::<i16>(&dev, &cfg, ch, m2, o2, c2),
            SampleFormat::U16 => build::<u16>(&dev, &cfg, ch, m2, o2, c2),
            SampleFormat::I32 => build::<i32>(&dev, &cfg, ch, m2, o2, c2),
            f => Err(format!("formato audio non supportato: {f:?}")),
        }
    };
    let (stream, used_buf, warn) = match want {
        Some(b) => match try_open(BufferSize::Fixed(b)) {
            Ok(s) => (s, b, None),
            Err(e) => (try_open(BufferSize::Default)?, 0, Some(format!("buffer di {b} frame non accettato ({e}); uso quello automatico"))),
        },
        None => (try_open(BufferSize::Default)?, 0, None),
    };
    stream.play().map_err(|e| e.to_string())?;
    Ok((
        stream,
        AudioInfo { rendered: 0, missed: 0, entered: 0, exited: 0, phase: 0, rates, driver: host_label(host.id()), device: name, sample_rate: sr, buffer: used_buf, channels, actual_buffer: 0, latency_ms: 0.0, error: warn },
    ))
}

fn build<T: SizedSample + FromSample<f32>>(
    dev: &cpal::Device,
    cfg: &StreamConfig,
    ch: usize,
    mixer: Arc<Mutex<Mixer>>,
    observed: Arc<AtomicU32>,
    counters: Arc<Counters>,
) -> Result<cpal::Stream, String> {
    let mut tmp: Vec<f32> = Vec::new();
    let mut hp: Vec<f32> = Vec::new();
    let sr = cfg.sample_rate.0;
    dev.build_output_stream(
        cfg,
        move |data: &mut [T], _| {
            let frames = data.len() / ch;
            counters.entered.fetch_add(1, Ordering::Relaxed);
            counters.phase.store(1, Ordering::Relaxed);
            observed.store(frames as u32, Ordering::Relaxed);
            tmp.resize(frames * 2, 0.0);
            match mixer.try_lock() {
                Some(mut m) => {
                    counters.phase.store(2, Ordering::Relaxed);
                    m.render(&mut tmp);
                    counters.rendered.fetch_add(1, Ordering::Relaxed);
                }
                None => {
                    tmp.iter_mut().for_each(|v| *v = 0.0);
                    counters.missed.fetch_add(1, Ordering::Relaxed);
                    counters.rendered.fetch_add(1, Ordering::Relaxed);
                }
            }
            counters.phase.store(3, Ordering::Relaxed);
            // cuffie sullo stesso dispositivo: canali 3-4
            let with_hp = ch >= 4 && counters.inline.render(frames, sr, &mut hp);
            for (i, f) in data.chunks_mut(ch).enumerate() {
                for (c, s) in f.iter_mut().enumerate() {
                    *s = T::from_sample(if c < 2 { tmp[i * 2 + c] } else if with_hp && c < 4 { hp[i * 2 + c - 2] } else { 0.0 });
                }
            }
            counters.exited.fetch_add(1, Ordering::Relaxed);
        },
        |e| eprintln!("errore audio: {e}"),
        None,
    )
    .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// elenca i driver e i dispositivi; con SPIN_ASIO_OPEN=1 apre davvero il driver ASIO con un buffer piccolo
    #[test]
    #[ignore]
    fn list_hosts_and_try_asio() {
        println!("driver disponibili: {:?}", cpal::available_hosts().iter().map(|h| host_label(*h)).collect::<Vec<_>>());
        let host = pick_host(Some("asio"));
        println!("scelto: {}", host_label(host.id()));
        let devs: Vec<String> = host.output_devices().map(|d| d.filter_map(|x| x.name().ok()).collect()).unwrap_or_default();
        println!("dispositivi: {devs:?}");
        if std::env::var("SPIN_ASIO_OPEN").is_ok() {
            for buf in [64u32, 128, 256] {
                let st = Settings { driver: Some("asio".into()), device: None, sample_rate: Some(44100), buffer: Some(buf), preview_device: None };
                let mut mixer = None;
                let observed_n = Arc::new(AtomicU32::new(0));
                match open(&host, &st, &mut mixer, observed_n.clone(), Arc::new(Counters::default())) {
                    Ok((s, info)) => {
                        let m = mixer.clone().unwrap();
                        let n = (info.sample_rate as usize) / 4;
                        m.lock().drums[0].load("t".into(), String::new(), (0..n * 2).map(|i| ((i / 2) as f32 * 0.05).sin() * 0.2).collect());
                        std::thread::sleep(std::time::Duration::from_millis(200));
                        m.lock().drum_trigger(0, 1.0);
                        std::thread::sleep(std::time::Duration::from_millis(900));
                        let playing = m.lock().drums[0].playing();
                        println!("buffer {buf}: aperto {} {} Hz, buffer {} ({:?}); blocchi osservati: {} frame; pad ancora acceso dopo 0,9 s: {playing}", info.device, info.sample_rate, info.buffer, info.error, observed_n.load(Ordering::Relaxed));
                        drop(s);
                    }
                    Err(e) => println!("buffer {buf}: errore {e}"),
                }
            }
        }
    }

    /// prova ad aprire ogni driver ASIO installato per nome (stampa l'errore vero di ciascuno)
    #[test]
    #[ignore]
    fn try_each_named_asio_driver() {
        let names = asio_sys::Asio::new().driver_names();
        println!("driver ASIO nel registro: {names:?}");
        let host = pick_host(Some("asio"));
        for n in names {
            let st = Settings { driver: Some("asio".into()), device: Some(n.clone()), sample_rate: None, buffer: Some(128), preview_device: None };
            let mut mixer = None;
            let t = std::time::Instant::now();
            match attempt(&st, &mut mixer, &Arc::new(AtomicU32::new(0)), &Arc::new(Counters::default())) {
                Ok((s, info)) => { println!("«{n}»: OK {} Hz, {} canali, buffer {} in {:?}", info.sample_rate, info.channels, info.buffer, t.elapsed()); drop(s); }
                Err(e) => println!("«{n}»: ERRORE {e} (dopo {:?})", t.elapsed()),
            }
        }
        let _ = host;
    }

    /// l'anteprima (WASAPI) e' gia' aperta sulla stessa scheda quando parte l'ASIO: l'audio deve girare lo stesso
    #[test]
    #[ignore]
    fn asio_with_wasapi_preview_open() {
        let bus = Arc::new(crate::cue::CueBus::new(44100));
        let pv = crate::preview::Previewer::start(None, bus, Default::default());
        println!("anteprima WASAPI aperta: sr {}", pv.sample_rate());
        std::thread::sleep(std::time::Duration::from_millis(500));
        let host = pick_host(Some("asio"));
        let st = Settings { driver: Some("asio".into()), device: None, sample_rate: None, buffer: None, preview_device: None };
        let mut mixer = None;
        let obs = Arc::new(AtomicU32::new(0));
        match open(&host, &st, &mut mixer, obs.clone(), Arc::new(Counters::default())) {
            Ok((s, info)) => {
                let m = mixer.clone().unwrap();
                let n = (info.sample_rate as usize) / 4;
                m.lock().drums[0].load("t".into(), String::new(), (0..n * 2).map(|i| ((i / 2) as f32 * 0.05).sin() * 0.2).collect());
                std::thread::sleep(std::time::Duration::from_millis(300));
                m.lock().drum_trigger(0, 1.0);
                std::thread::sleep(std::time::Duration::from_millis(1200));
                println!("ASIO {} {} Hz buffer {} ({}): callback {} frame, pad acceso dopo 1,2 s: {}", info.device, info.sample_rate, info.buffer, info.actual_buffer, obs.load(Ordering::Relaxed), m.lock().drums[0].playing());
                drop(s);
            }
            Err(e) => println!("ASIO errore: {e}"),
        }
    }

    /// come nell'app: parte con WASAPI e poi si passa a ASIO a caldo (e di nuovo indietro)
    #[test]
    #[ignore]
    fn switch_wasapi_to_asio_at_runtime() {
        let a = Audio::start(&Settings::default()).expect("avvio");
        let bus = a.mixer.lock().cue_bus.clone();
        let pv = crate::preview::Previewer::start(None, bus, Default::default());
        let check = |label: &str, a: &Audio| {
            let n = 11025usize;
            a.mixer.lock().drums[0].load("t".into(), String::new(), (0..n * 2).map(|i| ((i / 2) as f32 * 0.05).sin() * 0.2).collect());
            std::thread::sleep(std::time::Duration::from_millis(300));
            a.mixer.lock().drum_trigger(0, 1.0);
            std::thread::sleep(std::time::Duration::from_millis(1200));
            let i = a.current();
            println!("{label}: {} {} Hz, callback {} frame, pad acceso dopo 1,2 s: {}", i.device, i.sample_rate, i.actual_buffer, a.mixer.lock().drums[0].playing());
        };
        check("WASAPI", &a);
        let r = a.reconfigure(Settings { driver: Some("asio".into()), ..Default::default() });
        println!("passaggio a ASIO: {:?}", r.as_ref().map(|i| (i.driver.clone(), i.device.clone(), i.sample_rate)).map_err(|e| e.clone()));
        let t = std::time::Instant::now();
        let r2 = pv.reopen(None);
        println!("anteprima riaperta in {:?}: {:?}", t.elapsed(), r2);
        check("ASIO", &a);
        let _ = a.reconfigure(Settings { driver: Some("wasapi".into()), ..Default::default() });
        check("di nuovo WASAPI", &a);
    }

    /// se il driver scelto non parte l'uscita ripiega su WASAPI e l'audio continua a girare
    #[test]
    #[ignore]
    fn failed_asio_falls_back_to_wasapi() {
        let a = Audio::start(&Settings::default()).expect("avvio");
        let r = a.reconfigure(Settings { driver: Some("asio".into()), sample_rate: Some(7000), ..Default::default() }).expect("deve ripiegare, non fallire");
        println!("dopo il guasto: {} / {} errore: {:?}", r.driver, r.device, r.error);
        assert_eq!(r.driver, "WASAPI");
        assert!(r.error.is_some());
        std::thread::sleep(std::time::Duration::from_millis(500));
        let i = a.current();
        println!("blocchi elaborati: {}", i.rendered);
        assert!(i.rendered > 10, "l'audio deve girare");
        // e l'ASIO buono funziona
        let r = a.reconfigure(Settings { driver: Some("asio".into()), buffer: Some(128), ..Default::default() }).expect("asio");
        println!("ASIO: {} {:?} buffer {}", r.device, r.error, r.buffer);
        std::thread::sleep(std::time::Duration::from_millis(500));
        let i = a.current();
        println!("ASIO blocchi: {} (saltati {})", i.rendered, i.missed);
        assert_eq!(r.driver, "ASIO");
        assert!(i.rendered > 50);
    }

    /// l'ASIO deve continuare a richiamare l'audio per tutta la durata, non solo all'inizio
    #[test]
    #[ignore]
    fn asio_keeps_running() {
        let a = Audio::start(&Settings { driver: Some("asio".into()), buffer: Some(128), ..Default::default() }).expect("avvio");
        for k in 0..8 {
            std::thread::sleep(std::time::Duration::from_millis(1000));
            let i = a.current();
            println!("t={}s driver {} blocchi {} saltati {}", k + 1, i.driver, i.rendered, i.missed);
        }
    }

    /// bisezione: ASIO insieme agli altri componenti dell'app (SPIN_WITH = midi,preview,stems)
    #[test]
    #[ignore]
    fn asio_with_components() {
        let with = std::env::var("SPIN_WITH").unwrap_or_default();
        let a = Audio::start(&Settings { driver: Some("asio".into()), buffer: Some(128), ..Default::default() }).expect("avvio");
        let bus = a.mixer.lock().cue_bus.clone();
        let _pv = if with.contains("preview") { Some(crate::preview::Previewer::start(None, bus, Default::default())) } else { None };
        if with.contains("midi") { crate::midi::start(a.mixer.clone(), None); }
        if with.contains("stems") {
            let dir = std::path::PathBuf::from(std::env::var("APPDATA").unwrap()).join("com.spinlab.dj");
            std::thread::spawn(move || crate::stems::warm(&crate::stems::model_path(&dir)));
        }
        for k in 0..8 {
            std::thread::sleep(std::time::Duration::from_millis(1000));
            let i = a.current();
            println!("[{with}] t={}s blocchi {}", k + 1, i.rendered);
        }
    }
}
