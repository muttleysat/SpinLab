//! Diagnostica delle cuffie: elenca le uscite WASAPI e apre quella indicata misurando il blocco richiesto dal driver.
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
fn main() {
    let host = cpal::default_host();
    println!("host {:?}", host.id());
    for d in host.output_devices().unwrap() {
        let n = d.name().unwrap_or_default();
        match d.default_output_config() {
            Ok(c) => println!("- {n}: {} ch {} Hz {:?} buf {:?}", c.channels(), c.sample_rate().0, c.sample_format(), c.buffer_size()),
            Err(e) => println!("- {n}: ERRORE {e}"),
        }
    }
    let Some(want) = std::env::args().nth(1) else { return };
    let d = host.output_devices().unwrap().find(|d| d.name().ok().as_deref() == Some(want.as_str())).expect("dispositivo non trovato");
    let c = d.default_output_config().unwrap();
    let cfg = cpal::StreamConfig { channels: c.channels(), sample_rate: c.sample_rate(), buffer_size: cpal::BufferSize::Default };
    let t0 = std::time::Instant::now();
    let n = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let n2 = n.clone();
    let ch = c.channels() as usize;
    let s = d.build_output_stream(&cfg, move |data: &mut [f32], _| {
        let k = n2.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        if k < 6 { eprintln!("callback {k}: {} frame a +{:?}", data.len() / ch, t0.elapsed()); }
        data.iter_mut().for_each(|v| *v = 0.0);
    }, |e| eprintln!("errore {e}"), None).expect("apertura");
    s.play().unwrap();
    std::thread::sleep(std::time::Duration::from_secs(2));
    println!("callback in 2 s: {}", n.load(std::sync::atomic::Ordering::SeqCst));
}
