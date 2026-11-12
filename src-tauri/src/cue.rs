//! Bus delle cuffie: il mixer vi scrive il preascolto (PFL) dei deck e l'uscita cuffie lo legge.
//! Le due uscite audio hanno clock e frequenze diverse, quindi in mezzo c'e' un buffer con
//! ricampionamento lineare e un piccolo servo che tiene il livello di riempimento stabile senza salti.

use parking_lot::Mutex;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicU32, Ordering};

pub struct CueBus {
    q: Mutex<VecDeque<f32>>,
    /// frequenza di campionamento dei dati scritti dal mixer
    sr: AtomicU32,
}

impl CueBus {
    pub fn new(sr: u32) -> Self {
        CueBus { q: Mutex::new(VecDeque::new()), sr: AtomicU32::new(sr) }
    }
    pub fn set_sr(&self, sr: u32) {
        self.sr.store(sr, Ordering::SeqCst);
        self.q.lock().clear();
    }
    pub fn sr(&self) -> u32 {
        self.sr.load(Ordering::SeqCst)
    }
    /// scrive audio stereo interleaved (chiamato dal thread audio principale)
    pub fn push(&self, data: &[f32]) {
        if let Some(mut q) = self.q.try_lock() {
            q.extend(data.iter().copied());
            // limite di sicurezza: oltre 150 ms di ritardo si scarta il piu' vecchio
            let cap = self.sr() as usize * 3 / 10; // frame*2 = 150 ms
            if q.len() > cap {
                let drop = q.len() - cap;
                q.drain(..drop);
            }
        }
    }
    pub fn clear(&self) {
        if let Some(mut q) = self.q.try_lock() {
            q.clear();
        }
    }
}

/// Lato lettura: ricampiona dal bus alla frequenza dell'uscita cuffie.
#[derive(Default)]
pub struct Puller {
    /// posizione frazionaria dentro il primo frame del bus
    frac: f64,
    primed: bool,
}

impl Puller {
    /// Riempie `out` (stereo interleaved a `out_sr`) con l'audio del bus; silenzio se il bus e' vuoto.
    pub fn pull(&mut self, bus: &CueBus, out: &mut [f32], out_sr: u32) {
        out.iter_mut().for_each(|v| *v = 0.0);
        let Some(mut q) = bus.q.try_lock() else { return };
        let frames = out.len() / 2;
        if frames == 0 || out_sr == 0 {
            return;
        }
        let base = bus.sr() as f64 / out_sr as f64;
        let have = (q.len() / 2) as f64;
        // riserva: abbastanza audio da non restare a secco fra un callback e l'altro (~4 blocchi)
        let target = (frames as f64 * base * 4.0).max(256.0);
        if !self.primed {
            if have < target {
                return;
            }
            self.primed = true;
            self.frac = 0.0;
        }
        if have < frames as f64 * base + 2.0 {
            // sottoflusso: aspetta che il buffer si riempia di nuovo
            self.primed = false;
            return;
        }
        // ritardo accumulato (uscita cuffie ferma o in ritardo per un po'): il servo da solo smaltirebbe 0,5 s in due minuti,
        // quindi sopra i ~60 ms si salta subito al livello normale (un piccolo salto, meglio di un ritardo che resta)
        let max_have = (target * 2.0).max(frames as f64 * base * 2.0 + out_sr as f64 * 0.06 * base);
        if have > max_have {
            let drop = ((have - target) as usize) * 2;
            let n = drop.min(q.len());
            q.drain(..n);
            self.frac = 0.0;
        }
        let have = (q.len() / 2) as f64;
        // servo: buffer troppo pieno -> leggi un po' piu' in fretta, troppo vuoto -> piu' piano
        let adj = ((have - target) / target).clamp(-1.0, 1.0) * 0.004;
        let ratio = base * (1.0 + adj);
        let mut pos = self.frac;
        for f in 0..frames {
            let i = pos.floor() as usize;
            let t = (pos - i as f64) as f32;
            for c in 0..2 {
                let a = q[i * 2 + c];
                let b = q.get((i + 1) * 2 + c).copied().unwrap_or(a);
                out[f * 2 + c] = a + (b - a) * t;
            }
            pos += ratio;
        }
        let used = pos.floor() as usize;
        let n = (used * 2).min(q.len());
        q.drain(..n);
        self.frac = pos - used as f64;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// due clock diversi (48 kHz scrive, 44,1 kHz legge): il suono arriva continuo, senza vuoti, e il buffer non cresce
    #[test]
    fn drops_accumulated_delay_quickly() {
        let bus = CueBus::new(44100);
        let mut puller = Puller::default();
        let mut out = vec![0f32; 441 * 2];
        // il lato cuffie resta fermo 250 ms mentre il mixer continua a scrivere
        for _ in 0..25 {
            bus.push(&vec![0.3f32; 441 * 2]);
        }
        // poi riprende a ritmo normale: dopo pochi blocchi il ritardo deve essere sceso sotto i 60 ms
        for _ in 0..10 {
            bus.push(&vec![0.3f32; 441 * 2]);
            puller.pull(&bus, &mut out, 44100);
        }
        let delay_ms = bus.q.lock().len() as f64 / 2.0 / 44.1;
        assert!(delay_ms < 60.0, "ritardo rimasto {delay_ms} ms");
    }

    #[test]
    fn resamples_between_clocks_without_gaps_or_growth() {
        let bus = CueBus::new(48000);
        let mut puller = Puller::default();
        let (wsr, rsr) = (48000.0f64, 44100.0f64);
        let (wblk, rblk) = (512usize, 441usize);
        let mut t_w = 0.0f64; // secondi scritti
        let mut t_r = 0.0f64; // secondi letti
        let mut n_w = 0usize;
        let mut out = vec![0f32; rblk * 2];
        let mut silent_after_prime = 0;
        let mut prev_last = 0f32;
        let mut max_jump = 0f32;
        let mut max_len = 0usize;
        let mut started = false;
        // simula 20 s di tempo reale: avanza il clock piu' indietro
        while t_w.min(t_r) < 20.0 {
            if t_w <= t_r {
                let blk: Vec<f32> = (0..wblk).flat_map(|i| { let v = 0.5 * (2.0 * std::f64::consts::PI * 440.0 * (n_w + i) as f64 / wsr).sin() as f32; [v, v] }).collect();
                bus.push(&blk);
                n_w += wblk;
                t_w += wblk as f64 / wsr;
            } else {
                puller.pull(&bus, &mut out, 44100);
                t_r += rblk as f64 / rsr;
                let energy: f32 = out.iter().map(|v| v.abs()).sum();
                if energy > 1.0 {
                    started = true;
                    max_jump = max_jump.max((out[0] - prev_last).abs());
                    for f in out.chunks(2).collect::<Vec<_>>().windows(2) {
                        max_jump = max_jump.max((f[1][0] - f[0][0]).abs());
                    }
                    prev_last = out[rblk * 2 - 2];
                } else if started {
                    silent_after_prime += 1;
                }
                max_len = max_len.max(bus.q.lock().len() / 2);
            }
        }
        assert!(started, "il bus non ha mai prodotto suono");
        assert_eq!(silent_after_prime, 0, "vuoti nel suono dopo la partenza");
        // seno a 440 Hz, campionato a 44,1 kHz: il salto massimo fra campioni vicini e' 0,5*2pi*440/44100 ~ 0,031
        assert!(max_jump < 0.05, "discontinuita' {max_jump}");
        assert!(max_len < 48000 / 4, "il buffer cresce: {max_len} frame");
    }
}
