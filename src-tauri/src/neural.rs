//! Derivato da openDJ (https://github.com/openwave-studio/opendj, crates/engine/src/beatthis.rs), Copyright (c) gli autori di openDJ,
//! GPL-2.0-or-later; adattato per SpinLab (modifiche di Vincenzo Trombetta, 2026) e distribuito sotto GPL-3.0-or-later.
//!
//! Battiti e battute con la rete neurale Beat This! (Foscarin, Schluter, Widmer, ISMIR 2024; codice e pesi MIT) tramite il crate
//! `beat-this` (MIT, runtime rten in puro Rust). E' l'ingresso della griglia (vedi gridfit.rs): il rilevatore DSP di analysis.rs
//! resta il primo passaggio veloce, questo lo affina in sottofondo (3-6 s a brano) e sostituisce i battiti salvati.
//!
//! I modelli (mel 270 KB + beat "small" 10 MB) viaggiano nell'installer (models/beat-this); se mancano si scaricano al primo uso nella
//! cartella dei dati, con verifica SHA-256.
//!
//! `in_range` e `refit` sono copiati da openDJ senza modifiche: dai battiti e dagli inizi di battuta della rete si ottiene la stessa
//! griglia di openDJ (scelta dell'ottava nell'intervallo di tempo, poi gridfit.rs).

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

const MEL_FILE: &str = "mel_spectrogram.onnx";
const BEAT_FILE: &str = "beat_this_small.onnx";
const BASE_URL: &str = "https://raw.githubusercontent.com/danigb/beat-this-rs/main/models/";
const MEL_SHA: &str = "fdd59e65c515331308e4c8841edf99972deca646bdf6197744c2a5b7755e3de9";
const BEAT_SHA: &str = "a5f8d39d989f31859454ba27afe61c5317ca95e4d9373e6853e5361b8937172f";

/// un modello che non si scarica o non si carica non va riprovato a ogni brano
static UNAVAILABLE: AtomicBool = AtomicBool::new(false);

/// battiti e inizi di battuta (secondi) trovati dalla rete in un brano, come li tiene openDJ
pub struct Beats {
    pub beats: Vec<f64>,
    pub downbeats: Vec<f64>,
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
        let downbeats: Vec<f64> = an.downbeats.iter().map(|&x| x as f64).collect();
        Some(Beats { beats, downbeats })
    }
}

#[cfg(target_os = "android")]
mod imp {
    use super::*;
    pub fn run(_dir: &Path, _stereo: &[f32], _sr: u32) -> Option<Beats> {
        None
    }
}

/// Analizza `stereo` (interleaved, `sr` Hz): battiti e inizi di battuta della rete.
pub fn refine(dir: &Path, stereo: &[f32], sr: u32) -> Option<Beats> {
    imp::run(dir, stereo, sr)
}

/// La griglia dai battiti della rete, come `beatthis::refit` di openDJ: tempo dei battiti riportato nell'intervallo `range`
/// (quello del genere del brano o 60-180 BPM), poi gridfit nella modalita' del brano.
pub fn refit(beats: &[f64], downbeats: &[f64], range: (f64, f64), mode: crate::gridfit::GridMode) -> Option<crate::gridfit::Fitted> {
    let (beats, bpm) = in_range(beats.to_vec(), downbeats, range);
    crate::gridfit::fit_mode(&beats, Some(downbeats), bpm, mode)
}

/// The tempo of `beats` (from the median gap) and the beats at that tempo,
/// halved or doubled into `range`: halving keeps the beats that fall on
/// more downbeats, doubling adds one between each two.
fn in_range(mut beats: Vec<f64>, downbeats: &[f64], (lo, hi): (f64, f64)) -> (Vec<f64>, f64) {
    let tempo = |beats: &[f64]| {
        let mut gaps: Vec<f64> = beats.windows(2).map(|w| w[1] - w[0]).collect();
        if gaps.is_empty() {
            return 0.0;
        }
        let mid = gaps.len() / 2;
        60.0 / *gaps.select_nth_unstable_by(mid, f64::total_cmp).1
    };
    let mut bpm = tempo(&beats);
    // Twice is enough for any range at least an octave wide.
    for _ in 0..2 {
        if bpm > hi {
            let on_downbeat = |&b: &f64| downbeats.iter().any(|d| (d - b).abs() < 0.07);
            let odd = beats
                .iter()
                .skip(1)
                .step_by(2)
                .filter(|b| on_downbeat(b))
                .count();
            let even = beats.iter().step_by(2).filter(|b| on_downbeat(b)).count();
            beats = beats
                .into_iter()
                .skip(usize::from(odd > even))
                .step_by(2)
                .collect();
        } else if bpm > 0.0 && bpm < lo {
            let mids: Vec<f64> = beats.windows(2).map(|w| (w[0] + w[1]) / 2.0).collect();
            beats.extend(mids);
            beats.sort_by(f64::total_cmp);
        } else {
            break;
        }
        bpm = tempo(&beats);
    }
    (beats, bpm)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn folds_the_beats_tempo_into_the_range() {
        let at = |bpm: f64, n: usize| {
            (0..n)
                .map(|k| 0.3 + k as f64 * 60.0 / bpm)
                .collect::<Vec<_>>()
        };
        let (beats, bpm) = in_range(at(67.2, 100), &[], (60.0, 180.0));
        assert_eq!(beats.len(), 100);
        assert!((bpm - 67.2).abs() < 0.01, "{bpm}");
        // Too fast: every other beat, the ones on downbeats.
        let fast = at(200.0, 100);
        let downbeats: Vec<f64> = fast.iter().skip(1).step_by(8).copied().collect();
        let (beats, bpm) = in_range(fast.clone(), &downbeats, (60.0, 180.0));
        assert!((bpm - 100.0).abs() < 0.01, "{bpm}");
        assert_eq!(beats[0], fast[1]);
        // Too slow: a beat between each two.
        let (beats, bpm) = in_range(at(62.0, 50), &[], (70.0, 180.0));
        assert!((bpm - 124.0).abs() < 0.01, "{bpm}");
        assert_eq!(beats.len(), 99);
    }
}
