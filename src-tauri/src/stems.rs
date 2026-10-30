//! Separazione degli stem (batteria, basso, altro, voce) con HT-Demucs via ONNX Runtime.
//! Il modello lavora su segmenti da 7,8 s a 44,1 kHz stereo; i brani lunghi si dividono in segmenti
//! sovrapposti e si ricompongono con una finestra triangolare. Il risultato resta su disco (16 bit):
//! il deck lo rilegge e ne mescola gli stem scelti.
//!
//! Acceleratori: su Windows DirectML; su macOS di default la CPU (AMX/Accelerate) perche' il
//! CoreML EP di ORT su questo modello e' molto piu' lento — vedi `open` per i dettagli e per
//! forzare i tentativi GPU con SPIN_STEM_EP.

use ort::session::Session;
use ort::value::Tensor;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU8, AtomicUsize, Ordering};
use std::sync::Arc;

pub const SEG: usize = 343_980;
pub const SR: u32 = 44_100;

const MAGIC: &[u8; 8] = b"SPSTEM01";
/// stem compressi: 4 flussi FLAC (uno per stem) decodificabili in parallelo
const MAGIC2: &[u8; 8] = b"SPSTEM02";

/// Il modello viaggia dentro l'installer (cartella «models» accanto al programma): se c'e' si usa quello, senza scaricare niente.
/// In sviluppo si cerca in `src-tauri/models` (lo mette `tools/fetch_model.py`); in ultimo, nei dati dell'utente (dove lo scarica `ensure_model`).
pub fn model_path(dir: &Path) -> PathBuf {
    let mut dirs: Vec<PathBuf> = vec![];
    if let Ok(exe) = std::env::current_exe() {
        if let Some(d) = exe.parent() {
            dirs.push(d.join("models"));
            dirs.push(d.join("resources").join("models"));
            for up in d.ancestors().take(4) {
                dirs.push(up.join("models"));
                dirs.push(up.join("src-tauri").join("models"));
            }
        }
    }
    dirs.push(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("models"));
    // l'ordine dei nomi e' l'ordine di preferenza: su Android il modello per telefoni (piu' veloce) se c'e'
    let names: &[&str] = if cfg!(target_os = "android") { &[MODEL_FILE_MOBILE, MODEL_FILE] } else { &[MODEL_FILE] };
    for name in names {
        if let Some(p) = dirs.iter().map(|d| d.join(name)).chain(std::iter::once(dir.join("models").join(name))).find(|p| p.is_file()) {
            return p;
        }
    }
    // niente da nessuna parte: quello da scaricare
    dir.join("models").join(if cfg!(target_os = "android") && MODEL_URL_MOBILE.is_some() { MODEL_FILE_MOBILE } else { MODEL_FILE })
}

const MODEL_FILE: &str = "htdemucs.onnx";
/// Versione per telefoni, fatta da `tools/make_mobile_model.py`: pesi fp32, grafo semplificato (25 000 nodi -> 1 500)
/// e iSTFT con una IDFT invece di due ConvTranspose da 2049x4096. Stesso risultato (differenze sotto 1e-4).
/// Su Galaxy S26 Ultra (CPU, 4 thread): si apre in 0,6 s invece di 5,5 s e separa un segmento piu' in fretta.
/// int8, fp16 e NNAPI sono stati provati e scartati: int8 non guadagna niente, fp16 e NNAPI sono 10-70 volte piu' lenti.
/// Non ha ancora un indirizzo di download: finche' e' `None` l'app usa il modello normale, ma se il file e' gia'
/// nella cartella `models` (copiato a mano, vedi docs/SVILUPPO.md) lo preferisce.
const MODEL_FILE_MOBILE: &str = "htdemucs_mobile.onnx";
const MODEL_URL_MOBILE: Option<&str> = None;

/// file di cache degli stem di un brano (chiave: percorso, dimensione e data di modifica)
pub fn cache_path(dir: &Path, track: &str) -> PathBuf {
    let (size, mtime) = std::fs::metadata(track)
        .map(|m| (m.len(), m.modified().ok().and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok()).map_or(0, |d| d.as_secs())))
        .unwrap_or((0, 0));
    let mut h: u64 = 0xcbf29ce484222325;
    for b in track.to_lowercase().bytes().chain(size.to_le_bytes()).chain(mtime.to_le_bytes()) {
        h = (h ^ b as u64).wrapping_mul(0x100000001b3);
    }
    dir.join("stems").join(format!("{h:016x}.stm"))
}

/// File degli stem nella cartella «stems» accanto al brano (nome del file + dimensione, cosi' segue il brano se lo si sposta con la cartella)
pub fn library_path(track: &str) -> Option<PathBuf> {
    let p = Path::new(track);
    let size = std::fs::metadata(p).ok()?.len();
    let name = p.file_name()?.to_string_lossy();
    Some(p.parent()?.join("stems").join(format!("{name}.{size:x}.stems")))
}

/// Dove sono gli stem pronti di un brano: prima nella cartella «stems» della libreria, poi nella vecchia cache dell'app.
pub fn find_cached(dir: &Path, track: &str) -> Option<PathBuf> {
    library_path(track).filter(|p| p.exists()).or_else(|| Some(cache_path(dir, track)).filter(|p| p.exists()))
}

pub fn has_cached(dir: &Path, track: &str) -> bool {
    find_cached(dir, track).is_some()
}

/// Salva gli stem di un brano nella cartella «stems» della libreria (se non e' scrivibile, nella cache dell'app).
pub fn save_for(dir: &Path, track: &str, st: &Stems) -> Result<PathBuf, String> {
    if let Some(p) = library_path(track) {
        if save(&p, st).is_ok() {
            // una vecchia copia nella cache dell'app non serve piu'
            let _ = std::fs::remove_file(cache_path(dir, track));
            return Ok(p);
        }
    }
    let p = cache_path(dir, track);
    save(&p, st).map(|_| p)
}

/// Elimina gli stem preparati di un brano (cartella «stems» e vecchia cache).
pub fn delete_for(dir: &Path, track: &str) {
    if let Some(p) = library_path(track) {
        let _ = std::fs::remove_file(&p);
        if let Some(parent) = p.parent() {
            let _ = std::fs::remove_dir(parent); // solo se vuota
        }
    }
    let _ = std::fs::remove_file(cache_path(dir, track));
}

const MODEL_URL: &str = "https://huggingface.co/adowu/htdemucs-onnx/resolve/main/htdemucs_fp16weights.onnx";

/// Se il modello non e' stato trovato (installazione senza il file, o sviluppo) lo scarica nei dati dell'utente:
/// con il curl del sistema su desktop, con un client HTTP interno su Android (dove curl non esiste).
pub fn ensure_model(model: &Path) -> Result<(), String> {
    if model.exists() {
        return Ok(());
    }
    let url = if model.file_name().is_some_and(|n| n == MODEL_FILE_MOBILE) {
        MODEL_URL_MOBILE.ok_or("manca l'indirizzo di download del modello per telefoni")?
    } else {
        MODEL_URL
    };
    if let Some(p) = model.parent() {
        std::fs::create_dir_all(p).map_err(|e| e.to_string())?;
    }
    let tmp = model.with_extension("part");
    let ok = download(url, &tmp).map_err(|e| format!("download del modello non riuscito: {e}"))?;
    if !ok || std::fs::metadata(&tmp).map_or(0, |m| m.len()) < 100_000_000 {
        let _ = std::fs::remove_file(&tmp);
        return Err("download del modello non riuscito (serve la connessione a internet)".into());
    }
    std::fs::rename(&tmp, model).map_err(|e| e.to_string())
}

#[cfg(not(target_os = "android"))]
fn download(url: &str, dest: &Path) -> Result<bool, String> {
    // su Windows il curl incluso si chiama curl.exe, su macOS/Linux semplicemente curl
    let curl = if cfg!(windows) { "curl.exe" } else { "curl" };
    let st = std::process::Command::new(curl)
        .args(["-L", "--fail", "-s", "-o"])
        .arg(dest)
        .arg(url)
        .status()
        .map_err(|e| e.to_string())?;
    Ok(st.success())
}

#[cfg(target_os = "android")]
fn download(url: &str, dest: &Path) -> Result<bool, String> {
    let resp = ureq::get(url).call().map_err(|e| e.to_string())?;
    let mut f = std::fs::File::create(dest).map_err(|e| e.to_string())?;
    std::io::copy(&mut resp.into_reader(), &mut f).map_err(|e| e.to_string())?;
    Ok(true)
}

/// Sessione del modello condivisa da tutti: prova gli acceleratori (DirectML su Windows, CoreML su macOS)
/// e se non partono (memoria occupata, operazioni non supportate) ripiega sulla CPU.
/// `gpu_in_use()` dice quale e' stata scelta.
static SESSION: parking_lot::Mutex<Option<Session>> = parking_lot::Mutex::new(None);
static GPU: AtomicU8 = AtomicU8::new(0); // 0 non ancora provata, 1 GPU, 2 CPU

pub fn gpu_in_use() -> bool {
    GPU.load(Ordering::SeqCst) == 1
}

/// Tentativi di accelerazione in ordine di preferenza: Windows un adattatore (DirectML),
/// macOS due (Neural Engine + GPU, poi sola GPU in caso il primo tentativo fallisca).
/// Su macOS i tentativi partono solo con SPIN_STEM_EP=gpu: vedi `open`.
#[cfg(windows)]
const GPU_ATTEMPTS: usize = 1;
#[cfg(target_os = "macos")]
const GPU_ATTEMPTS: usize = 2;
#[cfg(not(any(windows, target_os = "macos")))]
const GPU_ATTEMPTS: usize = 0;

/// Nome del tentativo `i` (solo per i log).
#[cfg(windows)]
fn gpu_label(_attempt: usize) -> &'static str {
    "DirectML"
}
#[cfg(target_os = "macos")]
fn gpu_label(attempt: usize) -> &'static str {
    if attempt == 0 { "CoreML ANE+GPU" } else { "CoreML GPU" }
}
#[cfg(not(any(windows, target_os = "macos")))]
fn gpu_label(_attempt: usize) -> &'static str {
    "GPU"
}

fn open(model: &Path) -> Result<Session, String> {
    // Su macOS il CoreML EP di ONNX Runtime con htdemucs fp16 e' ~70x piu' lento della CPU
    // (85 s contro 1,2 s per segmento: la CPU Apple usa AMX/Accelerate, il CoreML EP fa
    // continui trasferimenti e compilazioni). Default quindi: CPU esplicita su macOS.
    // SPIN_STEM_EP=gpu prova comunque gli acceleratori (diagnostica), SPIN_STEM_EP=cpu li evita ovunque.
    let ep = std::env::var("SPIN_STEM_EP").map(|v| v.to_ascii_lowercase()).unwrap_or_default();
    let default_cpu = cfg!(target_os = "macos") && ep != "gpu";
    let force_cpu = ep == "cpu" || default_cpu;
    if !force_cpu && GPU.load(Ordering::SeqCst) != 2 {
        let t = std::time::Instant::now();
        let mut last_err = String::new();
        for attempt in 0..GPU_ATTEMPTS {
            let try_gpu = || -> Result<Session, String> {
                let t_open = std::time::Instant::now();
                let mut s = open_with(model, Some(attempt as u8))?;
                eprintln!("stem: sessione {} aperta in {:?}", gpu_label(attempt), t_open.elapsed());
                // il primo passaggio compila il grafo sul dispositivo: e' li' che fallisce se manca memoria
                let t_run = std::time::Instant::now();
                run_segment(&mut s, vec![0f32; 2 * SEG])?;
                eprintln!("stem: prima inferenza {} in {:?}", gpu_label(attempt), t_run.elapsed());
                Ok(s)
            };
            match try_gpu() {
                Ok(s) => {
                    GPU.store(1, Ordering::SeqCst);
                    eprintln!("stem: GPU pronta ({}) in {:?}", gpu_label(attempt), t.elapsed());
                    return Ok(s);
                }
                Err(e) => {
                    eprintln!("stem: tentativo GPU {} non riuscito ({e})", gpu_label(attempt));
                    last_err = e;
                }
            }
        }
        eprintln!("stem: GPU non utilizzabile ({last_err}), uso la CPU");
        GPU.store(2, Ordering::SeqCst);
    }
    open_with(model, None).inspect(|_| eprintln!("stem: sessione CPU pronta ({})", model.file_name().map_or(String::new(), |n| n.to_string_lossy().into_owned())))
}

/// Abbassa la priorita' del thread che chiama (e dei thread che crea, come quelli di ONNX Runtime): la separazione
/// non deve togliere tempo all'interfaccia. L'audio non ne risente (gira in un thread a priorita' reale).
pub fn background_priority() {
    #[cfg(target_os = "android")]
    unsafe {
        // nice 5: sotto l'interfaccia (0), sopra i lavori di sfondo di Android (10) per non restare indietro
        libc::setpriority(libc::PRIO_PROCESS, libc::gettid() as libc::id_t, 5);
    }
}

/// Nome del file del modello in uso (per capire se la velocita' misurata vale ancora).
pub fn model_name(model: &Path) -> String {
    model.file_name().map_or(String::new(), |n| n.to_string_lossy().into_owned())
}

/// Apre il modello in anticipo (se il file c'e' gia').
pub fn warm(model: &Path) {
    if model.exists() {
        let _ = with_session(model, |_| Ok(()));
    }
}

/// Esegue `f` con la sessione condivisa (la apre alla prima chiamata).
fn with_session<R>(model: &Path, f: impl FnOnce(&mut Session) -> Result<R, String>) -> Result<R, String> {
    let mut g = SESSION.lock();
    if g.is_none() {
        *g = Some(open(model)?);
    }
    f(g.as_mut().unwrap())
}

/// `gpu`: None = CPU; Some(i) = tentativo i-esimo di accelerazione (vedi `gpu_label`).
fn open_with(model: &Path, gpu: Option<u8>) -> Result<Session, String> {
    static ENV: std::sync::OnceLock<Result<ort::environment::Environment, String>> = std::sync::OnceLock::new();
    let env = ENV.get_or_init(|| ort::init().build().map_err(|e| e.to_string())).as_ref().map_err(|e| e.clone())?;
    // su Android 4 thread separano veloci come 6 o 8 (misurato su S26 Ultra) e lasciano liberi i core per audio e interfaccia
    let mut threads = if cfg!(target_os = "android") { 4 } else { std::thread::available_parallelism().map_or(4, |n| n.get()).clamp(2, 12) };
    // prova: SPIN_STEM_THREADS oppure il file `stem_threads` accanto al modello (comodo su Android)
    if let Some(n) = std::env::var("SPIN_STEM_THREADS").ok().or_else(|| model.parent().and_then(|p| std::fs::read_to_string(p.join("stem_threads")).ok()))
        .and_then(|v| v.trim().parse::<usize>().ok()).filter(|n| (1..=16).contains(n)) {
        threads = n;
    }
    eprintln!("stem: {threads} thread");
    let mut b = Session::builder(env).map_err(|e| e.to_string())?.with_intra_threads(threads).map_err(|e| e.to_string())?;
    if let Some(attempt) = gpu {
        b = b
            .with_memory_pattern(false)
            .map_err(|e| e.to_string())?
            .with_parallel_execution(false)
            .map_err(|e| e.to_string())?;
        #[cfg(windows)]
        {
            let ep = ort::ep::DirectML::default()
                .with_performance_preference(ort::ep::directml::PerformancePreference::HighPerformance);
            b = b.with_execution_providers([ep.build()]).map_err(|e| e.to_string())?;
        }
        // su macOS la GPU e' CoreML. Con SPIN_COREML=plain si prova la configurazione semplice (diagnostica).
        #[cfg(target_os = "macos")]
        {
            let plain = std::env::var("SPIN_COREML").map(|v| v == "plain").unwrap_or(false);
            let mut ep = ort::ep::CoreML::default().with_compute_units(if attempt == 0 {
                ort::ep::coreml::ComputeUnits::All
            } else {
                ort::ep::coreml::ComputeUnits::CPUAndGPU
            });
            if !plain {
                ep = ep
                    .with_model_format(ort::ep::coreml::ModelFormat::MLProgram)
                    .with_static_input_shapes(true)
                    .with_specialization_strategy(ort::ep::coreml::SpecializationStrategy::FastPrediction);
                if let Some(cache) = model.parent().map(|p| p.join("coreml-cache")) {
                    let _ = std::fs::create_dir_all(&cache);
                    ep = ep.with_model_cache_dir(cache.to_string_lossy().into_owned());
                }
            }
            b = b.with_execution_providers([ep.build()]).map_err(|e| e.to_string())?;
        }
    } else {
        // IMPORTANTISSIMO: ort imposta di default la politica di selezione automatica "MaxEfficiency",
        // che su macOS sceglie CoreML/ANE anche senza EP espliciti (per questo modello e' lentissimo).
        // PreferCPU forza la CPU vera.
        b = b.with_auto_device(ort::session::builder::AutoDevicePolicy::PreferCPU).map_err(|e| e.to_string())?;
    }
    b.commit_from_file(model).map_err(|e| e.to_string())
}

/// Un segmento: `mix` = [L..., R...] (2*SEG), ritorna [4][2][SEG] in un solo vettore.
fn run_segment(s: &mut Session, mix: Vec<f32>) -> Result<Vec<f32>, String> {
    let t = Tensor::from_array(([1usize, 2, SEG], mix)).map_err(|e| e.to_string())?;
    let out = s.run(ort::inputs!["mix" => t]).map_err(|e| e.to_string())?;
    let (_, d) = out["stems"].try_extract_tensor::<f32>().map_err(|e| e.to_string())?;
    Ok(d.to_vec())
}

/// Stem su 4 buffer stereo interleaved a 44,1 kHz.
pub struct Stems {
    pub frames: usize,
    pub data: [Vec<i16>; 4],
}

fn to_i16(v: f32) -> i16 {
    (v.clamp(-1.0, 1.0) * 32767.0).round() as i16
}

/// Separa un file audio. `progress` riceve 0..1 e ritorna `false` per annullare.
pub fn separate(audio_path: &str, model: &Path, progress: &mut dyn FnMut(f32) -> bool) -> Result<Stems, String> {
    let mix = crate::decode::decode_file(audio_path, SR)?;
    separate_buf(&mix, model, progress)
}

/// Separa audio stereo interleaved a 44,1 kHz.
pub fn separate_buf(mix: &[f32], model: &Path, progress: &mut dyn FnMut(f32) -> bool) -> Result<Stems, String> {
    if !model.exists() {
        return Err("modello di separazione non trovato".into());
    }
    let frames = mix.len() / 2;
    if frames == 0 {
        return Err("brano vuoto".into());
    }
    let overlap = SEG / 4;
    let stride = SEG - overlap;
    let chunks = ((frames.saturating_sub(overlap)) + stride - 1) / stride;
    let chunks = chunks.max(1);
    let mut acc: Vec<Vec<f32>> = (0..4).map(|_| vec![0f32; frames * 2]).collect();
    let mut wsum = vec![0f32; frames];
    let mut planar = vec![0f32; 2 * SEG];
    for c in 0..chunks {
        if !progress(c as f32 / chunks as f32) {
            return Err("annullato".into());
        }
        let start = c * stride;
        let clen = SEG.min(frames - start);
        planar.iter_mut().for_each(|v| *v = 0.0);
        for i in 0..clen {
            planar[i] = mix[(start + i) * 2];
            planar[SEG + i] = mix[(start + i) * 2 + 1];
        }
        let out = with_session(model, |sess| run_segment(sess, planar.clone()))?;
        // finestra: dissolvenza nella zona di sovrapposizione; ai capi del brano resta piena
        let last = c + 1 == chunks;
        for i in 0..clen {
            let fin = if c == 0 { 1.0 } else { ((i + 1) as f32 / overlap as f32).min(1.0) };
            let fout = if last { 1.0 } else { ((SEG - i) as f32 / overlap as f32).min(1.0) };
            let w = fin.min(fout);
            wsum[start + i] += w;
            for s in 0..4 {
                let base = s * 2 * SEG;
                acc[s][(start + i) * 2] += out[base + i] * w;
                acc[s][(start + i) * 2 + 1] += out[base + SEG + i] * w;
            }
        }
    }
    progress(1.0);
    let mut data: [Vec<i16>; 4] = Default::default();
    for s in 0..4 {
        data[s] = (0..frames * 2).map(|k| to_i16(acc[s][k] / wsum[k / 2].max(1e-6))).collect();
    }
    Ok(Stems { frames, data })
}


// ------------------------------------------------------------------ separazione progressiva (in tempo reale)
//
// Il brano si divide in segmenti sovrapposti (come in `separate_buf`) ma si lavora a partire dalla testina: i segmenti
// pronti entrano subito nell'audio del deck, il resto continua a suonare il brano intero finche' non arriva.
// Cambiare gli stem scelti rimescola solo cio' che e' gia' pronto: e' immediato.

pub const OVERLAP: usize = SEG / 4;
pub const STRIDE: usize = SEG - OVERLAP;

pub fn seg_count(frames: usize) -> usize {
    ((frames.saturating_sub(OVERLAP) + STRIDE - 1) / STRIDE).max(1)
}

/// peso del frame `i` del segmento `j` di `n`: dissolvenza nelle sovrapposizioni, piena ai capi del brano
fn weight(j: usize, n: usize, i: usize) -> f32 {
    let fin = if j == 0 { 1.0 } else { ((i + 1) as f32 / OVERLAP as f32).min(1.0) };
    let fout = if j + 1 == n { 1.0 } else { ((SEG - i) as f32 / OVERLAP as f32).min(1.0) };
    fin.min(fout)
}

/// uscita di un segmento: i 4 stem, stereo interleaved a 16 bit
pub struct SegOut {
    pub st: [Vec<i16>; 4],
}

struct LiveInner {
    segs: Vec<Option<Arc<SegOut>>>,
    /// brano a 44,1 kHz con gli stem scelti dove pronti, l'originale altrove
    mixed: Vec<f32>,
    /// vista corrente (alla frequenza del deck) gia' consegnata al deck
    view: Arc<Vec<f32>>,
    /// viste vecchie: si liberano qui, mai nel thread audio
    grave: Vec<Arc<Vec<f32>>>,
}

pub struct Live {
    pub path: String,
    pub deck: usize,
    pub frames: usize,
    pub n: usize,
    mix44: Vec<f32>,
    /// frequenza e lunghezza (frame) della vista del deck
    sr: u32,
    out_frames: usize,
    /// volume di ogni stem (f32 in bit): batteria, basso, altro, voce
    gains: [AtomicU32; 4],
    pub done: AtomicUsize,
    pub alive: AtomicBool,
    inner: parking_lot::Mutex<LiveInner>,
}

pub type PublishFn = dyn Fn(usize, &str, Arc<Vec<f32>>, bool) -> bool + Send + Sync;

impl Live {
    /// `mix44`: il brano a 44,1 kHz; `base`: l'audio originale del deck (alla sua frequenza `sr`).
    pub fn new(path: &str, deck: usize, mix44: Vec<f32>, sr: u32, base: Arc<Vec<f32>>) -> Live {
        let frames = mix44.len() / 2;
        let n = seg_count(frames);
        Live {
            path: path.to_string(),
            deck,
            frames,
            n,
            sr,
            out_frames: base.len() / 2,
            gains: [1.0f32, 1.0, 1.0, 1.0].map(|g| AtomicU32::new(g.to_bits())),
            done: AtomicUsize::new(0),
            alive: AtomicBool::new(true),
            inner: parking_lot::Mutex::new(LiveInner { segs: vec![None; n], mixed: mix44.clone(), view: base, grave: vec![] }),
            mix44,
        }
    }

    /// audio del segmento `j` pronto per il modello (planare, riempito di zeri in coda) e frame reali
    pub fn audio(&self, j: usize) -> (Vec<f32>, usize) {
        let start = j * STRIDE;
        let clen = SEG.min(self.frames - start);
        let mut planar = vec![0f32; 2 * SEG];
        for i in 0..clen {
            planar[i] = self.mix44[(start + i) * 2];
            planar[SEG + i] = self.mix44[(start + i) * 2 + 1];
        }
        (planar, clen)
    }

    /// segmento da fare per primo a partire dalla posizione `pos` (frame a 44,1 kHz): quello sotto la testina o il primo
    /// dopo; se davanti e' tutto pronto, il primo mancante dietro
    pub fn next_needed(&self, pos: usize) -> Option<usize> {
        let g = self.inner.lock();
        let first = (0..self.n).find(|&j| g.segs[j].is_none() && j * STRIDE + SEG > pos);
        first.or_else(|| (0..self.n).find(|&j| g.segs[j].is_none()))
    }

    /// ricalcola `mixed` per i frame [a, b) con gli stem della maschera
    fn fill(&self, g: &mut LiveInner, a: usize, b: usize) {
        let gains = self.gains();
        for f in a..b.min(self.frames) {
            let jh = (f / STRIDE).min(self.n - 1);
            let mut wsum = 0f32;
            let mut acc = [0f32; 2];
            for j in [jh.wrapping_sub(1), jh] {
                if j >= self.n || f < j * STRIDE || f >= j * STRIDE + SEG {
                    continue;
                }
                let Some(seg) = g.segs[j].as_ref() else { continue };
                let i = f - j * STRIDE;
                if i * 2 + 1 >= seg.st[0].len() {
                    continue;
                }
                let w = weight(j, self.n, i);
                for s in 0..4 {
                    if gains[s] > 0.0 {
                        acc[0] += seg.st[s][i * 2] as f32 * w * gains[s];
                        acc[1] += seg.st[s][i * 2 + 1] as f32 * w * gains[s];
                    }
                }
                wsum += w;
            }
            if wsum > 1e-6 {
                g.mixed[f * 2] = acc[0] / (wsum * 32767.0);
                g.mixed[f * 2 + 1] = acc[1] / (wsum * 32767.0);
            } else {
                g.mixed[f * 2] = self.mix44[f * 2];
                g.mixed[f * 2 + 1] = self.mix44[f * 2 + 1];
            }
        }
    }

    /// copia i frame [a, b) di `mixed` nella nuova vista del deck (ricampionando se serve)
    fn to_view(&self, g: &LiveInner, v: &mut [f32], a: usize, b: usize) {
        if self.sr == SR {
            let b = b.min(v.len() / 2);
            if a < b {
                v[a * 2..b * 2].copy_from_slice(&g.mixed[a * 2..b * 2]);
            }
            return;
        }
        let ratio = SR as f64 / self.sr as f64;
        let ja = (a as f64 / ratio).floor() as usize;
        let jb = ((b as f64 / ratio).ceil() as usize).min(self.out_frames);
        for j in ja..jb {
            let pos = j as f64 * ratio;
            let ip = pos.floor() as isize;
            let t = (pos - ip as f64) as f32;
            v[j * 2] = crate::engine::cubic(&g.mixed, ip, t, 0, self.frames);
            v[j * 2 + 1] = crate::engine::cubic(&g.mixed, ip, t, 1, self.frames);
        }
    }

    /// Consegna al deck la vista aggiornata nelle regioni indicate. `fade`: rampa breve contro i click (cambio stem).
    fn publish(&self, g: &mut LiveInner, regions: &[(usize, usize)], publish: &PublishFn, fade: bool) {
        let mut v: Vec<f32> = (*g.view).clone();
        v.resize(self.out_frames * 2, 0.0);
        for &(a, b) in regions {
            self.to_view(g, &mut v, a, b);
        }
        let nv = Arc::new(v);
        if publish(self.deck, &self.path, nv.clone(), fade) {
            let old = std::mem::replace(&mut g.view, nv);
            g.grave.push(old);
            while g.grave.len() > 3 {
                g.grave.remove(0);
            }
        }
    }

    /// arriva un segmento: entra nell'audio del deck se gli stem sono attivi
    pub fn on_segment(&self, j: usize, out: SegOut, publish: &PublishFn) {
        let mut g = self.inner.lock();
        g.segs[j] = Some(Arc::new(out));
        self.done.fetch_add(1, Ordering::SeqCst);
        if self.active() {
            let (a, b) = (j * STRIDE, (j * STRIDE + SEG).min(self.frames));
            self.fill(&mut g, a, b);
            self.publish(&mut g, &[(a, b)], publish, false);
        }
    }

    /// cambio degli stem scelti: rimescola subito tutto cio' che e' pronto
    pub fn set_gains(&self, gains: [f32; 4], publish: &PublishFn) {
        for s in 0..4 {
            self.gains[s].store(gains[s].clamp(0.0, 1.0).to_bits(), Ordering::SeqCst);
        }
        if !self.active() {
            return;
        }
        let mut g = self.inner.lock();
        self.fill(&mut g, 0, self.frames);
        let all = [(0, self.frames)];
        self.publish(&mut g, &all, publish, true);
    }

    pub fn gains(&self) -> [f32; 4] {
        [0, 1, 2, 3].map(|s| f32::from_bits(self.gains[s].load(Ordering::SeqCst)))
    }

    /// gli stem sono diversi dal brano intero?
    pub fn active(&self) -> bool {
        self.gains().iter().any(|g| *g < 0.999)
    }

    pub fn complete(&self) -> bool {
        self.done.load(Ordering::SeqCst) >= self.n
    }

    /// tutti i segmenti pronti: ricostruisce i 4 stem interi (per la cache su disco)
    pub fn assemble(&self) -> Option<Stems> {
        let g = self.inner.lock();
        if g.segs.iter().any(|s| s.is_none()) {
            return None;
        }
        let mut data: [Vec<i16>; 4] = [0; 4].map(|_| vec![0i16; self.frames * 2]);
        for f in 0..self.frames {
            let jh = (f / STRIDE).min(self.n - 1);
            let mut wsum = 0f32;
            let mut acc = [[0f32; 2]; 4];
            for j in [jh.wrapping_sub(1), jh] {
                if j >= self.n || f < j * STRIDE || f >= j * STRIDE + SEG {
                    continue;
                }
                let seg = g.segs[j].as_ref()?;
                let i = f - j * STRIDE;
                let w = weight(j, self.n, i);
                for s in 0..4 {
                    acc[s][0] += seg.st[s][i * 2] as f32 * w;
                    acc[s][1] += seg.st[s][i * 2 + 1] as f32 * w;
                }
                wsum += w;
            }
            let wsum = wsum.max(1e-6);
            for s in 0..4 {
                data[s][f * 2] = (acc[s][0] / wsum).round() as i16;
                data[s][f * 2 + 1] = (acc[s][1] / wsum).round() as i16;
            }
        }
        Some(Stems { frames: self.frames, data })
    }
}

/// Converte l'uscita grezza del modello ([4][2][SEG]) nei primi `clen` frame di ogni stem, interleaved a 16 bit.
pub fn seg_out(out: &[f32], clen: usize) -> SegOut {
    let mut st: [Vec<i16>; 4] = Default::default();
    for s in 0..4 {
        let base = s * 2 * SEG;
        st[s] = (0..clen * 2).map(|k| to_i16(out[base + (k & 1) * SEG + k / 2])).collect();
    }
    SegOut { st }
}

/// Esegue un segmento con la sessione condivisa.
pub fn infer_segment(model: &Path, planar: Vec<f32>, clen: usize) -> Result<SegOut, String> {
    let out = with_session(model, |s| run_segment(s, planar))?;
    Ok(seg_out(&out, clen))
}

/// Chi lavora per i deck: il thread che sceglie quale segmento separare e le funzioni che parlano con il mixer.
pub struct Hub {
    pub live: [parking_lot::Mutex<Option<Arc<Live>>>; 4],
    pub cached: [parking_lot::Mutex<Option<(String, Arc<Stems>)>>; 4],
    wake: (parking_lot::Mutex<bool>, parking_lot::Condvar),
}

pub struct Callbacks {
    pub publish: Box<PublishFn>,
    /// posizione (secondi), riproduzione in corso e maschera stem del deck
    pub deck_pos: Box<dyn Fn(usize) -> Option<(f64, bool, bool)> + Send + Sync>,
    /// deck, percorso, percentuale, GPU
    pub progress: Box<dyn Fn(usize, &str, u32, bool) + Send + Sync>,
    /// true mentre un brano si sta caricando: la separazione aspetta
    pub busy: Box<dyn Fn() -> bool + Send + Sync>,
    /// separazione completa di un brano (salvata in cache)
    pub finished: Box<dyn Fn(&str, bool) + Send + Sync>,
}

impl Hub {
    pub fn new() -> Arc<Hub> {
        Arc::new(Hub {
            live: Default::default(),
            cached: Default::default(),
            wake: (parking_lot::Mutex::new(false), parking_lot::Condvar::new()),
        })
    }

    pub fn wake(&self) {
        *self.wake.0.lock() = true;
        self.wake.1.notify_all();
    }

    /// Avvia il thread che separa i segmenti: sceglie sempre quello che serve prima a chi sta ascoltando.
    pub fn spawn(self: &Arc<Hub>, dir: PathBuf, cb: Callbacks) {
        let hub = self.clone();
        std::thread::spawn(move || {
            background_priority();
            let model = model_path(&dir);
            loop {
                // sceglie il lavoro piu' urgente fra i deck
                let mut best: Option<(Arc<Live>, usize, (u8, f64))> = None;
                for d in 0..4 {
                    let Some(live) = hub.live[d].lock().clone() else { continue };
                    if !live.alive.load(Ordering::SeqCst) || live.complete() {
                        continue;
                    }
                    let Some((sec, playing, active)) = (cb.deck_pos)(d) else { continue };
                    let pos = (sec.max(0.0) * SR as f64) as usize;
                    let Some(j) = live.next_needed(pos) else { continue };
                    // chi ascolta gli stem ora ha la precedenza, poi chi li ha scelti ma e' fermo, poi il lavoro in anticipo
                    let class = match (active, playing) {
                        (true, true) => 0,
                        (true, false) => 1,
                        (false, true) => 2,
                        (false, false) => 3,
                    };
                    let dist = (j * STRIDE) as f64 - pos as f64;
                    let key = (class, dist.max(0.0));
                    if best.as_ref().map_or(true, |b| key < b.2) {
                        best = Some((live, j, key));
                    }
                }
                let Some((live, j, _)) = best else {
                    let mut w = hub.wake.0.lock();
                    if !*w {
                        hub.wake.1.wait_for(&mut w, std::time::Duration::from_millis(300));
                    }
                    *w = false;
                    continue;
                };
                if (cb.busy)() {
                    std::thread::sleep(std::time::Duration::from_millis(60));
                    continue;
                }
                if let Err(e) = ensure_model(&model) {
                    (cb.finished)(&live.path, false);
                    eprintln!("stem: {e}");
                    live.alive.store(false, Ordering::SeqCst);
                    continue;
                }
                let (planar, clen) = live.audio(j);
                match infer_segment(&model, planar, clen) {
                    Ok(out) => {
                        if !live.alive.load(Ordering::SeqCst) {
                            continue;
                        }
                        live.on_segment(j, out, &*cb.publish);
                        let pct = (live.done.load(Ordering::SeqCst) * 100 / live.n) as u32;
                        (cb.progress)(live.deck, &live.path, pct, gpu_in_use());
                        if live.complete() {
                            if let Some(st) = live.assemble() {
                                let ok = save_for(&dir, &live.path, &st).is_ok();
                                (cb.finished)(&live.path, ok);
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("stem: {e}");
                        live.alive.store(false, Ordering::SeqCst);
                        (cb.finished)(&live.path, false);
                    }
                }
            }
        });
    }
}

fn flac_encode(pcm: &[i16]) -> Result<Vec<u8>, String> {
    use flacenc::component::BitRepr;
    use flacenc::error::Verify;
    let cfg = flacenc::config::Encoder::default().into_verified().map_err(|e| format!("{e:?}"))?;
    let samples: Vec<i32> = pcm.iter().map(|&v| v as i32).collect();
    let src = flacenc::source::MemSource::from_samples(&samples, 2, 16, SR as usize);
    let stream = flacenc::encode_with_fixed_block_size(&cfg, src, cfg.block_size).map_err(|e| format!("{e:?}"))?;
    let mut sink = flacenc::bitsink::ByteSink::new();
    stream.write(&mut sink).map_err(|e| format!("{e:?}"))?;
    Ok(sink.as_slice().to_vec())
}

fn flac_decode(bytes: Vec<u8>, frames: usize) -> Result<Vec<i16>, String> {
    let mut r = claxon::FlacReader::new(std::io::Cursor::new(bytes)).map_err(|e| format!("lettura FLAC: {e}"))?;
    if r.streaminfo().channels != 2 {
        return Err("flusso FLAC non stereo".into());
    }
    let mut out: Vec<i16> = Vec::with_capacity(frames * 2);
    let mut blocks = r.blocks();
    let mut buf = Vec::new();
    loop {
        match blocks.read_next_or_eof(buf) {
            Ok(Some(b)) => {
                for i in 0..b.duration() {
                    out.push(b.sample(0, i) as i16);
                    out.push(b.sample(1, i) as i16);
                }
                buf = b.into_buffer();
            }
            Ok(None) => break,
            Err(e) => return Err(format!("decodifica FLAC: {e}")),
        }
    }
    out.resize(frames * 2, 0);
    Ok(out)
}

pub fn save(path: &Path, st: &Stems) -> Result<(), String> {
    if let Some(p) = path.parent() {
        std::fs::create_dir_all(p).map_err(|e| e.to_string())?;
    }
    let tmp = path.with_extension("tmp");
    let r = (|| -> Result<(), String> {
        // i 4 stem si comprimono in parallelo
        let blobs: Vec<Result<Vec<u8>, String>> = std::thread::scope(|sc| {
            let hs: Vec<_> = (0..4).map(|s| sc.spawn(move || flac_encode(&st.data[s]))).collect();
            hs.into_iter().map(|h| h.join().unwrap_or_else(|_| Err("compressione interrotta".into()))).collect()
        });
        let blobs: Vec<Vec<u8>> = blobs.into_iter().collect::<Result<_, _>>()?;
        let mut f = std::io::BufWriter::new(std::fs::File::create(&tmp).map_err(|e| e.to_string())?);
        f.write_all(MAGIC2).map_err(|e| e.to_string())?;
        f.write_all(&(st.frames as u64).to_le_bytes()).map_err(|e| e.to_string())?;
        for b in &blobs {
            f.write_all(&(b.len() as u64).to_le_bytes()).map_err(|e| e.to_string())?;
        }
        for b in &blobs {
            f.write_all(b).map_err(|e| e.to_string())?;
        }
        f.flush().map_err(|e| e.to_string())
    })();
    if let Err(e) = r {
        let _ = std::fs::remove_file(&tmp);
        return Err(e);
    }
    std::fs::rename(&tmp, path).map_err(|e| e.to_string())
}

pub fn load(path: &Path) -> Result<Stems, String> {
    let mut f = std::fs::File::open(path).map_err(|e| e.to_string())?;
    let mut head = [0u8; 16];
    f.read_exact(&mut head).map_err(|e| e.to_string())?;
    let frames = u64::from_le_bytes(head[8..16].try_into().unwrap()) as usize;
    if &head[..8] == MAGIC2 {
        let mut lens = [0u8; 32];
        f.read_exact(&mut lens).map_err(|e| e.to_string())?;
        let lens: Vec<usize> = lens.chunks_exact(8).map(|b| u64::from_le_bytes(b.try_into().unwrap()) as usize).collect();
        let mut blobs = Vec::new();
        for l in &lens {
            let mut b = vec![0u8; *l];
            f.read_exact(&mut b).map_err(|e| e.to_string())?;
            blobs.push(b);
        }
        // i 4 flussi si decodificano insieme: il caricamento e' veloce anche per brani lunghi
        let res: Vec<Result<Vec<i16>, String>> = std::thread::scope(|sc| {
            let hs: Vec<_> = blobs.into_iter().map(|b| sc.spawn(move || flac_decode(b, frames))).collect();
            hs.into_iter().map(|h| h.join().unwrap_or_else(|_| Err("decodifica interrotta".into()))).collect()
        });
        let mut data: [Vec<i16>; 4] = Default::default();
        for (s, r) in res.into_iter().enumerate() {
            data[s] = r?;
        }
        return Ok(Stems { frames, data });
    }
    if &head[..8] != MAGIC {
        return Err("file stem non valido".into());
    }
    let mut data: [Vec<i16>; 4] = Default::default();
    for s in 0..4 {
        let mut raw = vec![0u8; frames * 4];
        f.read_exact(&mut raw).map_err(|e| e.to_string())?;
        data[s] = raw.chunks_exact(2).map(|b| i16::from_le_bytes([b[0], b[1]])).collect();
    }
    Ok(Stems { frames, data })
}

/// Maschera di bit (1 batteria, 2 basso, 4 altro, 8 voce) come volumi 0/1.
pub fn mask_gains(mask: u8) -> [f32; 4] {
    [0, 1, 2, 3].map(|s| if mask & (1 << s) != 0 { 1.0 } else { 0.0 })
}

/// Somma gli stem con i loro volumi (batteria, basso, altro, voce) e ricampiona a `sr`.
/// Ritorna `frames_out` x 2 interleaved; zero-pad o taglio a `frames_out` per allinearsi al brano.
pub fn mix(st: &Stems, gains: [f32; 4], sr: u32, frames_out: usize) -> Vec<f32> {
    let mut out = vec![0f32; st.frames * 2];
    for s in 0..4 {
        if gains[s] > 0.0 {
            let g = gains[s] / 32767.0;
            for (o, v) in out.iter_mut().zip(st.data[s].iter()) {
                *o += *v as f32 * g;
            }
        }
    }
    let mut out = if sr == SR { out } else { crate::decode::resample_stereo(&out, SR, sr) };
    out.resize(frames_out * 2, 0.0);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dir() -> PathBuf {
        PathBuf::from(std::env::var("APPDATA").unwrap()).join("com.spinlab.dj")
    }

    /// separazione di un brano reale: SPIN_STEM=percorso. Verifica che somma degli stem ~ mix e che voce/strumentale siano diversi
    #[test]
    #[ignore]
    fn separate_real() {
        let Ok(p) = std::env::var("SPIN_STEM") else { return };
        let mix = crate::decode::decode_file(&p, SR).unwrap();
        let frames = mix.len() / 2;
        let secs: usize = std::env::var("SPIN_SECS").ok().and_then(|s| s.parse().ok()).unwrap_or(30);
        let start = (frames / 3).min(frames.saturating_sub(secs * SR as usize)) * 2;
        let seg = &mix[start..(start + secs * SR as usize * 2).min(mix.len())];
        let t0 = std::time::Instant::now();
        let st = separate_buf(seg, &model_path(&dir()), &mut |_| true).unwrap();
        println!("{} s separati in {:?}", seg.len() as f32 / 2.0 / SR as f32, t0.elapsed());
        let sum = mix_all(&st);
        let mut err = 0f64;
        let mut en = 0f64;
        for (a, b) in seg.iter().zip(sum.iter()) {
            err += ((a - b) as f64).powi(2);
            en += (*a as f64).powi(2);
        }
        let rms = |s: usize| (st.data[s].iter().map(|v| (*v as f64 / 32767.0).powi(2)).sum::<f64>() / st.data[s].len() as f64).sqrt();
        println!("errore ricostruzione {:.1} dB; rms drums {:.3} bass {:.3} other {:.3} vocals {:.3}", 10.0 * (err / en).log10(), rms(0), rms(1), rms(2), rms(3));
        assert!(err / en < 0.05, "la somma degli stem non ricostruisce il mix");
        let path = dir().join("stems").join("test.stm");
        save(&path, &st).unwrap();
        let back = load(&path).unwrap();
        assert_eq!(back.frames, st.frames);
        assert_eq!(back.data[3], st.data[3]);
    }

    fn mix_all(st: &Stems) -> Vec<f32> {
        mix(st, [1.0; 4], SR, st.frames)
    }

    /// la finestra non lascia buchi ne' amplifica: un segnale costante passa invariato a ogni numero di segmenti
    /// prova della GPU: SPIN_GPU = "cpu" oppure numero del tentativo (0 = acceleratore preferito)
    #[test]
    #[ignore]
    fn gpu_spike() {
        let dev = match std::env::var("SPIN_GPU").unwrap_or("0".into()).as_str() { "cpu" => None, n => Some(n.parse::<u8>().unwrap_or(0)) };
        let model = std::env::var("SPIN_MODEL").map(PathBuf::from).unwrap_or(model_path(&dir()));
        let t0 = std::time::Instant::now();
        let mut s = open_with(&model, dev).expect("apri");
        println!("apertura {:?}", t0.elapsed());
        let mix: Vec<f32> = (0..2 * SEG).map(|i| 0.3 * ((i % SEG) as f32 * 0.05).sin()).collect();
        for k in 0..4 {
            let t0 = std::time::Instant::now();
            let o = run_segment(&mut s, mix.clone()).expect("run");
            println!("run{k} {:?} len={}", t0.elapsed(), o.len());
        }
    }

    /// salva/ricarica nel formato compresso: identico campione per campione, piu' piccolo del grezzo, caricamento rapido
    #[test]
    fn compressed_file_roundtrip_is_lossless_and_small() {
        let frames = SR as usize * 20;
        let mut data: [Vec<i16>; 4] = Default::default();
        for s in 0..4 {
            data[s] = (0..frames * 2).map(|i| (((i / 2) as f32 * (0.02 + s as f32 * 0.013)).sin() * 9000.0 + ((i * 7919) % 200) as f32 - 100.0) as i16).collect();
        }
        let st = Stems { frames, data };
        let p = std::env::temp_dir().join("spinlab_test.stems");
        let t0 = std::time::Instant::now();
        save(&p, &st).unwrap();
        let t_save = t0.elapsed();
        let t1 = std::time::Instant::now();
        let back = load(&p).unwrap();
        let t_load = t1.elapsed();
        let size = std::fs::metadata(&p).unwrap().len();
        let _ = std::fs::remove_file(&p);
        println!("20 s: salvataggio {t_save:?}, caricamento {t_load:?}, {size} byte contro {} grezzi", frames * 16);
        assert_eq!(back.frames, frames);
        for s in 0..4 {
            assert_eq!(back.data[s], st.data[s], "stem {s} diverso");
        }
        assert!((size as usize) < frames * 16);
    }

    #[test]
    fn window_weights_cover_everything() {
        let overlap = SEG / 4;
        let stride = SEG - overlap;
        for frames in [SEG - 5, SEG, SEG + 1, stride * 3 + 100, stride * 4] {
            let chunks = (((frames.saturating_sub(overlap)) + stride - 1) / stride).max(1);
            let mut w = vec![0f32; frames];
            for c in 0..chunks {
                let start = c * stride;
                let clen = SEG.min(frames - start);
                for i in 0..clen {
                    let fin = if c == 0 { 1.0 } else { ((i + 1) as f32 / overlap as f32).min(1.0) };
                    let fout = if c + 1 == chunks { 1.0 } else { ((SEG - i) as f32 / overlap as f32).min(1.0) };
                    w[start + i] += fin.min(fout);
                }
            }
            assert!(w.iter().all(|v| *v > 0.2), "buco per frames={frames}");
        }
    }

    /// separazione progressiva: gli stem pronti entrano nella vista, il resto resta il brano originale; la maschera rimescola subito
    #[test]
    fn live_mixes_ready_segments_and_original_elsewhere() {
        let frames = STRIDE * 2 + OVERLAP + 5000;
        let mix44 = vec![0.25f32; frames * 2];
        let base = Arc::new(mix44.clone());
        let live = Live::new("x", 0, mix44, SR, base);
        assert_eq!(live.n, 3);
        let published: Arc<parking_lot::Mutex<Option<Arc<Vec<f32>>>>> = Default::default();
        let pc = published.clone();
        let publish: Box<PublishFn> = Box::new(move |_, _, v, _| { *pc.lock() = Some(v); true });
        // segmento 0: voce a 0,5, il resto a 0,1 per ognuno degli altri tre
        let mk = |v: f32, o: f32, len: usize| SegOut { st: [vec![to_i16(o); len * 2], vec![to_i16(o); len * 2], vec![to_i16(o); len * 2], vec![to_i16(v); len * 2]] };
        live.set_gains(mask_gains(8), &*publish); // solo voce, nessun segmento pronto: tutto originale
        live.on_segment(0, mk(0.5, 0.1, SEG), &*publish);
        let v = published.lock().clone().unwrap();
        assert!((v[1000 * 2] - 0.5).abs() < 0.01, "voce nel segmento pronto: {}", v[2000]);
        assert!((v[(STRIDE + 100000) * 2] - 0.25).abs() < 1e-6, "originale oltre il segmento pronto");
        live.set_gains(mask_gains(7), &*publish); // strumentale: 3 stem da 0,1
        let v = published.lock().clone().unwrap();
        assert!((v[1000 * 2] - 0.3).abs() < 0.01, "strumentale: {}", v[2000]);
        assert_eq!(live.next_needed(0), Some(1));
        assert_eq!(live.next_needed(STRIDE * 2 + OVERLAP + 100), Some(2));
        live.set_gains([1.0; 4], &*publish);
        assert!(!live.active());
        // volume dello stem: la voce a meta' si somma allo strumentale a 0,1+0,1+0,1
        live.set_gains([1.0, 1.0, 1.0, 0.5], &*publish);
        let v = published.lock().clone().unwrap();
        assert!((v[1000 * 2] - 0.55).abs() < 0.01, "gain voce: {}", v[2000]);
    }

    /// prova completa su un brano reale (SPIN_STEM): partendo da 60 s la voce deve arrivare entro pochi secondi
    #[test]
    #[ignore]
    fn hub_real_time() {
        let Ok(p) = std::env::var("SPIN_STEM") else { return };
        let mix44 = crate::decode::decode_file(&p, SR).unwrap();
        let base = Arc::new(mix44.clone());
        let live = Arc::new(Live::new(&p, 0, mix44.clone(), SR, base));
        let hub = Hub::new();
        let t0 = std::time::Instant::now();
        let first: Arc<parking_lot::Mutex<Option<(std::time::Duration, Arc<Vec<f32>>)>>> = Default::default();
        let (f2, t1) = (first.clone(), t0);
        let done = Arc::new(AtomicBool::new(false));
        let d2 = done.clone();
        *hub.live[0].lock() = Some(live.clone());
        hub.spawn(dir(), Callbacks {
            publish: Box::new(move |_, _, v, _| { f2.lock().get_or_insert((t1.elapsed(), v)); true }),
            deck_pos: Box::new(|_| Some((60.0, true, true))),
            progress: Box::new(|_, _, pct, gpu| println!("{pct}% gpu={gpu}")),
            busy: Box::new(|| false),
            finished: Box::new(move |_, ok| { println!("finito ok={ok}"); d2.store(true, Ordering::SeqCst); }),
        });
        live.set_gains(mask_gains(8), &|_, _, _, _| true);
        for _ in 0..600 {
            if first.lock().is_some() { break; }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
        let (dt, v) = first.lock().clone().expect("nessuna vista in 60 s");
        let a = 60 * SR as usize * 2;
        let rms = |x: &[f32]| (x.iter().map(|v| v * v).sum::<f32>() / x.len() as f32).sqrt();
        println!("prima vista dopo {dt:?}; rms originale {:.3} voce {:.3}", rms(&mix44[a..a + 88200]), rms(&v[a..a + 88200]));
        assert!(rms(&v[a..a + 88200]) < rms(&mix44[a..a + 88200]));
        assert!(dt.as_secs_f32() < 15.0);
        live.alive.store(false, Ordering::SeqCst);
    }
}
