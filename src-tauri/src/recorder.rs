//! Registrazione dell'uscita master su file WAV (16 o 24 bit) o MP3. Il thread audio manda solo i blocchi su un canale
//! senza mai aspettare; un thread a parte li scrive su disco, cosi' il disco lento non provoca scatti.

use crate::lame::Lame;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{sync_channel, Receiver, SyncSender};
use std::sync::Arc;
use std::thread::JoinHandle;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Format {
    Wav16,
    Wav24,
    /// kbps
    Mp3(u32),
}

impl Format {
    pub fn parse(s: &str) -> Format {
        match s {
            "wav24" => Format::Wav24,
            "mp3-128" => Format::Mp3(128),
            "mp3-192" => Format::Mp3(192),
            "mp3-256" => Format::Mp3(256),
            "mp3-320" | "mp3" => Format::Mp3(320),
            _ => Format::Wav16,
        }
    }
    #[cfg(test)]
    pub fn ext(&self) -> &'static str {
        match self {
            Format::Mp3(_) => "mp3",
            _ => "wav",
        }
    }
}

/// Lato audio: il mixer ci scrive i blocchi gia' mixati (stereo interleaved) e aggiorna i contatori.
pub struct Tap {
    pub tx: SyncSender<Vec<f32>>,
    pub frames: Arc<AtomicU64>,
    pub dropped: Arc<AtomicU64>,
}

impl Tap {
    pub fn push(&self, block: &[f32]) {
        match self.tx.try_send(block.to_vec()) {
            Ok(()) => {
                self.frames.fetch_add((block.len() / 2) as u64, Ordering::Relaxed);
            }
            Err(_) => {
                self.dropped.fetch_add(1, Ordering::Relaxed);
            }
        }
    }
}

/// Lato gestione: tiene il thread che scrive il file.
pub struct Recorder {
    pub path: PathBuf,
    pub sr: u32,
    pub frames: Arc<AtomicU64>,
    pub dropped: Arc<AtomicU64>,
    handle: JoinHandle<Result<(), String>>,
}

pub struct Finished {
    pub path: PathBuf,
    pub secs: f64,
    pub dropped: u64,
    pub bytes: u64,
}

/// Apre il file e avvia il thread di scrittura. Ritorna il registratore e il `Tap` da dare al mixer.
pub fn start(path: &Path, fmt: Format, sr: u32) -> Result<(Recorder, Tap), String> {
    if let Some(dir) = path.parent() {
        if !dir.as_os_str().is_empty() {
            std::fs::create_dir_all(dir).map_err(|e| format!("cartella non utilizzabile: {e}"))?;
        }
    }
    let (tx, rx) = sync_channel::<Vec<f32>>(512);
    let frames = Arc::new(AtomicU64::new(0));
    let dropped = Arc::new(AtomicU64::new(0));
    let p = path.to_path_buf();
    // il file si crea subito: se il percorso non e' scrivibile l'errore si vede prima di partire
    let sink = Sink::open(&p, fmt, sr)?;
    let handle = std::thread::spawn(move || write_loop(sink, rx));
    Ok((Recorder { path: p, sr, frames: frames.clone(), dropped: dropped.clone(), handle }, Tap { tx, frames, dropped }))
}

impl Recorder {
    /// Chiude la registrazione: il `Tap` deve essere gia' stato tolto dal mixer (cosi' il canale si chiude).
    pub fn finish(self) -> Result<Finished, String> {
        self.handle.join().map_err(|_| "il thread di scrittura e' terminato in modo anomalo".to_string())??;
        let bytes = std::fs::metadata(&self.path).map(|m| m.len()).unwrap_or(0);
        Ok(Finished { secs: self.frames.load(Ordering::Relaxed) as f64 / self.sr.max(1) as f64, dropped: self.dropped.load(Ordering::Relaxed), path: self.path, bytes })
    }
}

enum Sink {
    Wav(hound::WavWriter<std::io::BufWriter<std::fs::File>>, Format),
    Mp3(Lame, std::io::BufWriter<std::fs::File>),
}

impl Sink {
    fn open(path: &Path, fmt: Format, sr: u32) -> Result<Sink, String> {
        match fmt {
            Format::Wav16 | Format::Wav24 => {
                let spec = hound::WavSpec { channels: 2, sample_rate: sr, bits_per_sample: if fmt == Format::Wav24 { 24 } else { 16 }, sample_format: hound::SampleFormat::Int };
                let w = hound::WavWriter::create(path, spec).map_err(|e| format!("non riesco a creare il file: {e}"))?;
                Ok(Sink::Wav(w, fmt))
            }
            Format::Mp3(kbps) => {
                let enc = Lame::new(sr, kbps)?;
                let f = std::fs::File::create(path).map_err(|e| format!("non riesco a creare il file: {e}"))?;
                Ok(Sink::Mp3(enc, std::io::BufWriter::new(f)))
            }
        }
    }

    fn write(&mut self, block: &[f32]) -> Result<(), String> {
        match self {
            Sink::Wav(w, fmt) => {
                for &v in block {
                    let v = v.clamp(-1.0, 1.0);
                    if *fmt == Format::Wav24 {
                        w.write_sample((v * 8_388_607.0).round() as i32).map_err(|e| e.to_string())?;
                    } else {
                        w.write_sample((v * 32767.0).round() as i16).map_err(|e| e.to_string())?;
                    }
                }
                Ok(())
            }
            Sink::Mp3(enc, f) => {
                let bytes = enc.encode(block)?;
                f.write_all(bytes).map_err(|e| e.to_string())
            }
        }
    }

    fn finish(self) -> Result<(), String> {
        match self {
            Sink::Wav(w, _) => w.finalize().map_err(|e| e.to_string()),
            Sink::Mp3(mut enc, mut f) => {
                f.write_all(enc.finish()?).map_err(|e| e.to_string())?;
                f.flush().map_err(|e| e.to_string())
            }
        }
    }
}

fn write_loop(mut sink: Sink, rx: Receiver<Vec<f32>>) -> Result<(), String> {
    let mut err = None;
    while let Ok(block) = rx.recv() {
        if err.is_none() {
            if let Err(e) = sink.write(&block) {
                err = Some(e);
            }
        }
    }
    // il canale si e' chiuso: la registrazione e' finita, si chiude il file
    let fin = sink.finish();
    match err {
        Some(e) => Err(e),
        None => fin,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tone(frames: usize) -> Vec<f32> {
        (0..frames).flat_map(|i| { let v = 0.4 * (2.0 * std::f32::consts::PI * 440.0 * i as f32 / 44100.0).sin(); [v, -v] }).collect()
    }

    fn run(fmt: Format, name: &str) -> Finished {
        let path = std::env::temp_dir().join(format!("spinlab_rec_test_{name}.{}", fmt.ext()));
        let _ = std::fs::remove_file(&path);
        let (rec, tap) = start(&path, fmt, 44100).unwrap();
        for _ in 0..200 {
            tap.push(&tone(441));
        }
        drop(tap);
        rec.finish().unwrap()
    }

    #[test]
    fn wav_16_and_24_have_the_right_length_and_content() {
        for (fmt, name, bits) in [(Format::Wav16, "w16", 16u16), (Format::Wav24, "w24", 24)] {
            let f = run(fmt, name);
            assert!((f.secs - 2.0).abs() < 0.001, "durata {}", f.secs);
            assert_eq!(f.dropped, 0);
            let mut r = hound::WavReader::open(&f.path).unwrap();
            assert_eq!(r.spec().channels, 2);
            assert_eq!(r.spec().bits_per_sample, bits);
            assert_eq!(r.duration(), 200 * 441);
            let peak = r.samples::<i32>().map(|s| s.unwrap().abs()).max().unwrap();
            let full = if bits == 16 { 32767.0 } else { 8_388_607.0 };
            assert!((peak as f32 / full - 0.4).abs() < 0.01, "picco {peak}");
            let _ = std::fs::remove_file(&f.path);
        }
    }

    #[test]
    fn mp3_is_written_and_valid() {
        let f = run(Format::Mp3(192), "m192");
        assert!(f.bytes > 20_000, "file troppo piccolo: {}", f.bytes);
        let data = std::fs::read(&f.path).unwrap();
        // 2 s a 192 kbps = ~48 kB; frame MP3 che inizia con la sincronizzazione 0xFFEx
        assert!((30_000..80_000).contains(&data.len()), "dimensione {}", data.len());
        assert!(data.windows(2).any(|w| w[0] == 0xFF && (w[1] & 0xE0) == 0xE0), "nessun frame MP3");
        let _ = std::fs::remove_file(&f.path);
    }
}
