//! Tempo di gridfit per modalita' sui battiti veri: gridfit_time <beats.json>
#[path = "../src/gridfit.rs"]
mod gridfit;
use gridfit::{fit, GridMode};
fn main() {
    let a = std::env::args().nth(1).unwrap();
    let v: Vec<serde_json::Value> = serde_json::from_str(&std::fs::read_to_string(a).unwrap()).unwrap();
    for (m, tag) in [(GridMode::Auto, "Auto"), (GridMode::Fixed, "Fisso"), (GridMode::Flexible, "Flessibile")] {
        let (mut tot, mut worst) = (0.0f64, 0.0f64);
        for t in &v {
            let b: Vec<f64> = t["b"].as_array().unwrap().iter().filter_map(|x| x.as_f64()).collect();
            let s = std::time::Instant::now();
            let _ = fit(&b, 0, m);
            let e = s.elapsed().as_secs_f64() * 1000.0;
            tot += e; worst = worst.max(e);
        }
        println!("{tag}: media {:.2} ms, massimo {:.1} ms su {} brani", tot / v.len() as f64, worst, v.len());
    }
}
