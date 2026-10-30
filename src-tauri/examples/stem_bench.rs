//! Misura il tempo di un segmento di HT-Demucs (7,8 s di audio) con thread e acceleratori diversi.
//! Serve a capire se gli stem tengono il passo della riproduzione su un dispositivo (Android).
//!
//! stem_bench <modello.onnx> cpu <thread> [ripetizioni]   (SPIN_PROFILE=prefisso per il profilo di ORT)
//!
//! Per tenere il passo con un deck un segmento deve durare meno di 5,85 s (il passo fra un segmento e l'altro).

use ort::session::Session;
use ort::value::Tensor;

const SEG: usize = 343_980;

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let model = a.get(1).expect("modello");
    let ep = a.get(2).map_or("cpu", |s| s.as_str());
    let threads: usize = a.get(3).and_then(|s| s.parse().ok()).unwrap_or(4);
    let runs: usize = a.get(4).and_then(|s| s.parse().ok()).unwrap_or(3);
    let env = ort::init().build().expect("ort");
    let t0 = std::time::Instant::now();
    let mut b = Session::builder(&env).unwrap().with_intra_threads(threads).unwrap();
    match ep {
        // NNAPI (provato su S26 Ultra: 37 s per segmento contro 2,6 della CPU) richiede la feature "nnapi" di ort
        _ => {
            b = b.with_auto_device(ort::session::builder::AutoDevicePolicy::PreferCPU).unwrap();
        }
    }
    // SPIN_PROFILE=prefisso: profilo di ORT (JSON) con il tempo di ogni nodo
    if let Ok(p) = std::env::var("SPIN_PROFILE") {
        b = b.with_profiling(p).unwrap();
    }
    let mut s: Session = b.commit_from_file(model).expect("apertura del modello");
    println!("ep={ep} thread={threads} apertura {:?}", t0.elapsed());
    let mix: Vec<f32> = (0..2 * SEG).map(|i| 0.3 * ((i % SEG) as f32 * 0.05).sin() + 0.1 * ((i * 7919 % 1000) as f32 / 1000.0 - 0.5)).collect();
    for k in 0..runs {
        let t = std::time::Instant::now();
        let x = Tensor::from_array(([1usize, 2, SEG], mix.clone())).unwrap();
        let out = s.run(ort::inputs!["mix" => x]).expect("inferenza");
        let (_, d) = out["stems"].try_extract_tensor::<f32>().unwrap();
        let dt = t.elapsed().as_secs_f32();
        println!("run{k}: {dt:.2} s ({:.2}x tempo reale, {} valori)", 7.8 / dt, d.len());
    }
    if let Ok(f) = s.end_profiling() {
        println!("profilo: {f}");
    }
}
