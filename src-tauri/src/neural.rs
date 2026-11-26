//! Derivato da openDJ (https://github.com/openwave-studio/opendj, crates/engine/src/beatthis.rs), Copyright (c) gli autori di openDJ,
//! GPL-2.0-or-later; adattato per SpinLab (modifiche di Vincenzo Trombetta, 2026) e distribuito sotto GPL-3.0-or-later.
//!
//! Battiti e battute con la rete neurale Beat This! (Foscarin, Schluter, Widmer, ISMIR 2024; codice e pesi MIT) tramite il crate
//! `beat-this` (MIT, runtime rten in puro Rust). E' l'ingresso della griglia (vedi gridfit.rs): il rilevatore DSP di analysis.rs
//! resta il primo passaggio veloce, questo lo affina in sottofondo (3-6 s a brano) e sostituisce i battiti salvati.
//!
//! I modelli (mel 270 KB + beat "small" 10 MB) viaggiano nell'installer (models/beat-this); se mancano si scaricano al primo uso nella
//! cartella dei dati, con verifica SHA-256.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

const MEL_FILE: &str = "mel_spectrogram.onnx";
const BEAT_FILE: &str = "beat_this_small.onnx";
const BASE_URL: &str = "https://raw.githubusercontent.com/danigb/beat-this-rs/main/models/";
const MEL_SHA: &str = "fdd59e65c515331308e4c8841edf99972deca646bdf6197744c2a5b7755e3de9";
const BEAT_SHA: &str = "a5f8d39d989f31859454ba27afe61c5317ca95e4d9373e6853e5361b8937172f";

/// intervallo di tempo accettato (BPM): fuori si dimezza o si raddoppia
const BPM_LO: f64 = 70.0;
const BPM_HI: f64 = 200.0;

/// un modello che non si scarica o non si carica non va riprovato a ogni brano
static UNAVAILABLE: AtomicBool = AtomicBool::new(false);

/// battiti neurali di un brano
pub struct Beats {
    pub beats: Vec<f64>,
    /// indice (0..3) del battito che apre la battuta
    pub downbeat: usize,
    pub bpm: f64,
}

pub fn models_dir(dir: &Path) -> PathBuf {
    dir.join("models").join("beat-this")
}

#[cfg(not(target_os = "android"))]
mod imp {
    use super::*;
    use beat_this::{BeatThis, RtenRuntime, Runtime};

    type Bt = BeatThis<<RtenRuntime as Runtime>::Model>;
    static TRACKER: parking_lot::Mutex<Option<Bt>> = parking_lot::Mutex::new(None);

    fn sha256_hex(p: &Path) -> Option<String> {
        use sha2::{Digest, Sha256};
        let mut h = Sha256::new();
        std::io::copy(&mut std::fs::File::open(p).ok()?, &mut h).ok()?;
        Some(h.finalize().iter().map(|b| format!("{b:02x}")).collect())
    }

    /// il modello incluso nell'installer (cartella «models/beat-this» accanto al programma, o in Contents/Resources su macOS):
    /// se c'e' ed e' integro si usa quello, senza scaricare niente
    fn bundled(name: &str, sha: &str) -> Option<PathBuf> {
        let mut dirs: Vec<PathBuf> = vec![];
        if let Ok(exe) = std::env::current_exe() {
            if let Some(d) = exe.parent() {
                dirs.push(d.join("models"));
                dirs.push(d.join("resources").join("models"));
                for up in d.ancestors().take(4) {
                    dirs.push(up.join("models"));
                    dirs.push(up.join("Resources").join("models"));
                    dirs.push(up.join("src-tauri").join("models"));
                }
            }
        }
        dirs.push(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("models"));
        dirs.into_iter().map(|d| d.join("beat-this").join(name)).find(|p| p.is_file() && sha256_hex(p).as_deref() == Some(sha))
    }

    fn ensure(dir: &Path, name: &str, sha: &str) -> Result<PathBuf, String> {
        if let Some(p) = bundled(name, sha) {
            return Ok(p);
        }
        let dest = models_dir(dir).join(name);
        if dest.exists() && sha256_hex(&dest).as_deref() == Some(sha) {
            return Ok(dest);
        }
        std::fs::create_dir_all(models_dir(dir)).map_err(|e| e.to_string())?;
        let tmp = dest.with_extension("part");
        let ok = crate::stems::download(&format!("{BASE_URL}{name}"), &tmp)?;
        if !ok || sha256_hex(&tmp).as_deref() != Some(sha) {
            let _ = std::fs::remove_file(&tmp);
            return Err(format!("download di {name} non riuscito o file non valido"));
        }
        std::fs::rename(&tmp, &dest).map_err(|e| e.to_string())?;
        Ok(dest)
    }

    pub fn run(dir: &Path, stereo: &[f32], sr: u32) -> Option<Beats> {
        if UNAVAILABLE.load(Ordering::SeqCst) {
            return None;
        }
        let mut g = TRACKER.lock();
        if g.is_none() {
            // rten userebbe tutti i core fisici e soffocherebbe interfaccia e audio (waveform a scatti, stem piu' lenti): pochi thread,
            // l'analisi e' un lavoro di sottofondo e puo' metterci qualche secondo in piu'. Va impostato prima della prima inferenza.
            if std::env::var_os("RTEN_NUM_THREADS").is_none() {
                std::env::set_var("RTEN_NUM_THREADS", "3");
            }
            let loaded = ensure(dir, MEL_FILE, MEL_SHA).and_then(|m| {
                let b = ensure(dir, BEAT_FILE, BEAT_SHA)?;
                Bt::new(&RtenRuntime, &m, &b).map_err(|e| e.to_string())
            });
            match loaded {
                Ok(bt) => *g = Some(bt),
                Err(e) => {
                    eprintln!("beat this: {e}");
                    UNAVAILABLE.store(true, Ordering::SeqCst);
                    return None;
                }
            }
        }
        let bt = g.as_mut()?;
        let mono: Vec<f32> = stereo.chunks_exact(2).map(|f| 0.5 * (f[0] + f[1])).collect();
        let t = std::time::Instant::now();
        let an = match bt.analyze_owned(mono, sr) {
            Ok(a) => a,
            Err(e) => {
                eprintln!("beat this: analisi non riuscita ({e})");
                return None;
            }
        };
        eprintln!("[{:.0}] beat this: {} battiti in {:.1} s", crate::engine::now_ms(), an.beats.len(), t.elapsed().as_secs_f32());
        let beats: Vec<f64> = an.beats.iter().map(|&x| x as f64).collect();
        let downs: Vec<f64> = an.downbeats.iter().map(|&x| x as f64).collect();
        finish(beats, &downs)
    }
}

#[cfg(target_os = "android")]
mod imp {
    use super::*;
    pub fn run(_dir: &Path, _stereo: &[f32], _sr: u32) -> Option<Beats> {
        None
    }
}

/// Analizza `stereo` (interleaved, `sr` Hz). `_dsp_bpm` non serve piu' (l'ottava del tempo la sceglie la rete).
pub fn refine(dir: &Path, stereo: &[f32], sr: u32, _dsp_bpm: f64) -> Option<Beats> {
    imp::run(dir, stereo, sr)
}

fn median(v: &mut [f64]) -> f64 {
    if v.is_empty() {
        return 0.0;
    }
    v.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    v[v.len() / 2]
}

fn bpm_of(beats: &[f64]) -> f64 {
    let mut g: Vec<f64> = beats.windows(2).map(|w| w[1] - w[0]).filter(|x| *x > 0.0).collect();
    let m = median(&mut g);
    if m > 0.0 { 60.0 / m } else { 0.0 }
}

/// Tempo entro 70..200 BPM: l'ottava la sceglie la rete (un brano a 82 BPM puo' uscire a 165, come in opendj); solo fuori da questo
/// intervallo si tiene un battito su due (con la parita' che ha piu'
/// battute) o aggiunge un battito a meta' fra due. Poi ricava l'indice del battito che apre la battuta.
fn finish(mut beats: Vec<f64>, downs: &[f64]) -> Option<Beats> {
    if beats.len() < 16 {
        return None;
    }
    let near = |b: &[f64], t: f64| -> Option<usize> {
        let i = b.partition_point(|&x| x < t);
        let c = [i.checked_sub(1), (i < b.len()).then_some(i)];
        c.into_iter().flatten().min_by(|&a, &c| (b[a] - t).abs().partial_cmp(&(b[c] - t).abs()).unwrap_or(std::cmp::Ordering::Equal)).filter(|&k| (b[k] - t).abs() < 0.07)
    };
    {
        for _ in 0..2 {
            let bpm = bpm_of(&beats);
            if bpm > BPM_HI {
                // un battito su due, dalla parita' con piu' inizi di battuta
                let odd = downs.iter().filter_map(|&d| near(&beats, d)).filter(|k| k % 2 == 1).count();
                let even = downs.iter().filter_map(|&d| near(&beats, d)).filter(|k| k % 2 == 0).count();
                let start = usize::from(odd > even);
                beats = beats.iter().skip(start).step_by(2).copied().collect();
            } else if bpm > 0.0 && bpm < BPM_LO {
                let mut v = Vec::with_capacity(beats.len() * 2);
                for w in beats.windows(2) {
                    v.push(w[0]);
                    v.push(0.5 * (w[0] + w[1]));
                }
                v.push(*beats.last()?);
                beats = v;
            } else {
                break;
            }
        }
    }
    let bpm = bpm_of(&beats);
    if bpm <= 0.0 || beats.len() < 16 {
        return None;
    }
    // indice della battuta: la fase (0..3) piu' votata dagli inizi di battuta
    let mut votes = [0usize; 4];
    for &d in downs {
        if let Some(k) = near(&beats, d) {
            votes[k % 4] += 1;
        }
    }
    let downbeat = (0..4).max_by_key(|&k| votes[k]).unwrap_or(0);
    Some(Beats { beats, downbeat, bpm })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn halves_a_double_time_track_and_keeps_bar_phase() {
        // 240 BPM rilevati, riferimento 120: un battito su due, la parita' e' quella degli inizi di battuta
        let b: Vec<f64> = (0..80).map(|i| 0.1 + i as f64 * 0.25).collect();
        let downs: Vec<f64> = (0..10).map(|k| b[1 + k * 8]).collect(); // sui battiti dispari
        let r = finish(b, &downs).unwrap();
        assert!((r.bpm - 120.0).abs() < 0.5, "bpm {}", r.bpm);
        assert!((r.beats[0] - 0.35).abs() < 1e-9, "primo {}", r.beats[0]);
    }

    #[test]
    fn doubles_a_half_time_track() {
        let b: Vec<f64> = (0..40).map(|i| 0.1 + i as f64 * 1.0).collect(); // 60 BPM, riferimento 120
        let r = finish(b, &[]).unwrap();
        assert!((r.bpm - 120.0).abs() < 0.5, "bpm {}", r.bpm);
    }

    #[test]
    fn leaves_a_matching_tempo_alone() {
        let b: Vec<f64> = (0..40).map(|i| 0.1 + i as f64 * 0.5).collect();
        let r = finish(b.clone(), &[b[3], b[7]]).unwrap();
        assert_eq!(r.beats.len(), 40);
        assert_eq!(r.downbeat, 3);
    }
}
