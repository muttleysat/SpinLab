//! Prova di Beat This! su un file: beatthis_try <cartella modelli> <file audio> [piccolo]
//! Stampa i battiti neurali e il risultato di gridfit (Auto / Fisso / Flessibile) su di essi.
#[path = "../src/gridfit.rs"]
mod gridfit;
use beat_this::{BeatThis, RtenRuntime};
use gridfit::{fit, GridMode};

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let dir = std::path::Path::new(&a[1]);
    let small = a.get(3).is_some();
    let t0 = std::time::Instant::now();
    let mut bt = BeatThis::new(&RtenRuntime, &dir.join("mel_spectrogram.onnx"), &dir.join(if small { "beat_this_small.onnx" } else { "beat_this.onnx" })).expect("modelli");
    println!("modelli caricati in {:?}", t0.elapsed());
    let t1 = std::time::Instant::now();
    let an = bt.analyze_file(std::path::Path::new(&a[2])).expect("analisi");
    println!("analisi in {:?}: {} battiti, {} battute", t1.elapsed(), an.beats.len(), an.downbeats.len());
    let mut b: Vec<f64> = an.beats.iter().map(|&x| x as f64).collect();
    // REF=bpm del rilevatore DSP: porta il tempo vicino a quello (un battito su due, con la parita' che ha piu' battute)
    if let Some(r) = std::env::var("REF").ok().and_then(|v| v.parse::<f64>().ok()) {
        let bpm = 60.0 / { let mut g: Vec<f64> = b.windows(2).map(|w| w[1] - w[0]).collect(); g.sort_by(|a, c| a.partial_cmp(c).unwrap()); g[g.len() / 2] };
        if bpm > r * 1.4 {
            let near = |t: f64| b.iter().position(|x| (x - t).abs() < 0.07);
            let (mut odd, mut even) = (0, 0);
            for d in &an.downbeats { if let Some(k) = near(*d as f64) { if k % 2 == 1 { odd += 1 } else { even += 1 } } }
            b = b.iter().skip(usize::from(odd > even)).step_by(2).copied().collect();
            println!("tempo dimezzato: {bpm:.1} -> riferimento {r:.1}");
        }
    }
    let gaps: Vec<f64> = b.windows(2).map(|w| w[1] - w[0]).collect();
    println!("intervallo min {:.3} max {:.3} s", gaps.iter().cloned().fold(f64::MAX, f64::min), gaps.iter().cloned().fold(0.0, f64::max));
    let d0 = an.downbeats.first().and_then(|d| b.iter().position(|x| (x - *d as f64).abs() < 0.03)).unwrap_or(0);
    for (m, tag) in [(GridMode::Auto, "Auto"), (GridMode::Fixed, "Fisso"), (GridMode::Flexible, "Flessibile")] {
        match fit(&b, d0, m) {
            Some(f) => println!("{tag}: {} battiti, bpm {:.2}, {} nodi, primi: {:?}", f.beats.len(), f.bpm, f.markers.len(), f.markers.iter().take(8).map(|m| (m.0 as i64, (m.2 * 10.0).round() / 10.0)).collect::<Vec<_>>()),
            None => println!("{tag}: --"),
        }
    }
    if let Some(p) = std::env::var_os("DUMPFIT") {
        let f = fit(&b, d0, GridMode::Flexible).unwrap();
        std::fs::write(p, serde_json::to_string(&f.beats).unwrap()).unwrap();
    }
    if let Some(p) = std::env::var_os("DUMP") { std::fs::write(p, serde_json::to_string(&b).unwrap()).unwrap(); }
}
