//! Prova di gridfit sui battiti veri della libreria: gridfit_real <beats.json>
#[path = "../src/gridfit.rs"]
mod gridfit;
use gridfit::{fit, tempo_labels, GridMode};

fn main() {
    let a = std::env::args().nth(1).expect("json dei battiti");
    let v: Vec<serde_json::Value> = serde_json::from_str(&std::fs::read_to_string(a).unwrap()).unwrap();
    let (mut n_multi, mut n_total, mut worst_all) = (0, 0, 0.0f64);
    for t in &v {
        let b: Vec<f64> = t["b"].as_array().unwrap().iter().filter_map(|x| x.as_f64()).collect();
        let d = t["d"].as_u64().unwrap_or(0) as usize;
        let name = t["p"].as_str().unwrap_or("?");
        let mut line = format!("{:<44.44} in {:>4} bpm {:>6.2} |", name, b.len(), t["bpm"].as_f64().unwrap_or(0.0));
        for (m, tag) in [(GridMode::Auto, "A"), (GridMode::Fixed, "F"), (GridMode::Flexible, "X")] {
            match fit(&b, d, m) {
                Some(f) => {
                    // scarto massimo dai battiti rilevati (sul battito piu' vicino)
                    let dev = b.iter().map(|&x| f.beats.iter().map(|&y| (x - y).abs()).fold(f64::MAX, f64::min)).fold(0.0, f64::max);
                    line += &format!(" {tag}: n{:>4} bpm{:>7.2} mk{:>3} dev{:>4.0}ms lb{} |", f.beats.len(), f.bpm, f.markers.len(), dev * 1000.0, tempo_labels(&f).len());
                    if std::env::var("SHOW").map_or(false, |v| name.contains(&v)) && tag == "A" { println!("   markers: {:?}", f.markers.iter().map(|m| (m.0 as i64, (m.1 * 10.0).round() / 10.0, (m.2 * 100.0).round() / 100.0)).collect::<Vec<_>>()); }
                    if tag == "A" { n_total += 1; if f.markers.len() > 2 { n_multi += 1; } worst_all = worst_all.max(dev); }
                }
                None => line += &format!(" {tag}: -- |"),
            }
        }
        println!("{line}");
    }
    println!("Auto con piu' tratti: {n_multi}/{n_total}, scarto massimo {:.0} ms", worst_all * 1000.0);
}
