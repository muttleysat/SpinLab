//! Uscita video (prototipo): finestra a parte, a schermo intero su un monitor scelto, che mostra i video dei due deck.
//! Il video segue sempre l'audio: la posizione arriva dall'interfaccia principale e la finestra di uscita si allinea (vedi ui/video-out.html).
//! Tutto cio' che riguarda il video sta qui e in ui/video*.{js,html}: se la modalita' VIDEO e' spenta non viene eseguito nulla.

use serde::Serialize;
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};

pub const WINDOW: &str = "video-out";

#[derive(Serialize)]
pub struct MonitorInfo {
    pub index: usize,
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub primary: bool,
}

/// Monitor collegati (HDMI, DisplayPort, proiettori...) con nome e risoluzione.
#[tauri::command]
pub fn video_monitors(app: AppHandle) -> Vec<MonitorInfo> {
    let primary = app.primary_monitor().ok().flatten().map(|m| (m.position().x, m.position().y));
    app.available_monitors()
        .unwrap_or_default()
        .iter()
        .enumerate()
        .map(|(index, m)| MonitorInfo {
            index,
            name: m.name().cloned().unwrap_or_else(|| format!("Schermo {}", index + 1)),
            width: m.size().width,
            height: m.size().height,
            primary: Some((m.position().x, m.position().y)) == primary,
        })
        .collect()
}

/// Apre (o riporta in primo piano) la finestra di uscita sul monitor indicato; `None` = finestra normale sullo schermo corrente.
#[tauri::command]
pub fn video_open(app: AppHandle, monitor: Option<usize>) -> Result<(), String> {
    if let Some(w) = app.get_webview_window(WINDOW) {
        let _ = w.close();
    }
    let mut b = WebviewWindowBuilder::new(&app, WINDOW, WebviewUrl::App("video-out.html".into()))
        .title("SpinLab — Uscita video")
        .inner_size(960.0, 540.0)
        .decorations(true);
    if let Some(i) = monitor {
        if let Some(m) = app.available_monitors().map_err(|e| e.to_string())?.get(i) {
            let scale = m.scale_factor();
            let (p, s) = (m.position().to_logical::<f64>(scale), m.size().to_logical::<f64>(scale));
            b = b.position(p.x, p.y).inner_size(s.width, s.height).decorations(false).fullscreen(true);
        }
    }
    b.build().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn video_close(app: AppHandle) {
    if let Some(w) = app.get_webview_window(WINDOW) {
        let _ = w.close();
    }
}

// ---------------------------------------------------------------- copia di lavoro per lo scratch
// Nei file comuni c'e' un fotogramma intero ogni qualche secondo: ogni salto (scratch, allineamento) costa centinaia di ms.
// Si crea in sottofondo una copia 480p con tutti i fotogrammi interi (salti quasi gratuiti) in una cartella temporanea;
// si cancella quando il video non e' piu' su nessun deck e, per le chiusure anomale, all'avvio.

fn proxy_dir() -> std::path::PathBuf {
    std::env::temp_dir().join("spinlab_video")
}

fn proxy_name(path: &str) -> String {
    let (size, mtime) = std::fs::metadata(path)
        .map(|m| (m.len(), m.modified().ok().and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok()).map_or(0, |d| d.as_secs())))
        .unwrap_or((0, 0));
    let mut h: u64 = 0xcbf29ce484222325;
    for b in path.to_lowercase().bytes().chain(size.to_le_bytes()).chain(mtime.to_le_bytes()) {
        h = (h ^ b as u64).wrapping_mul(0x100000001b3);
    }
    format!("{h:016x}.mp4")
}

/// Crea (o ritrova) la copia di lavoro di un video e ne ritorna il percorso. Prova gli encoder hardware del sistema prima di quello software.
#[tauri::command]
pub async fn video_proxy(path: String) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || -> Result<String, String> {
        let dir = proxy_dir();
        std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        let out = dir.join(proxy_name(&path));
        if std::fs::metadata(&out).map_or(false, |m| m.len() > 0) {
            return Ok(out.to_string_lossy().into_owned());
        }
        let part = out.with_extension("part.mp4");
        let encoders: [&[&str]; 5] = [
            &["-c:v", "h264_videotoolbox", "-b:v", "3M"],
            &["-c:v", "h264_nvenc", "-b:v", "3M"],
            &["-c:v", "h264_qsv", "-b:v", "3M"],
            &["-c:v", "h264_amf", "-b:v", "3M"],
            &["-c:v", "libx264", "-preset", "ultrafast", "-crf", "26"],
        ];
        for enc in encoders {
            let mut cmd = std::process::Command::new(crate::decode::ffmpeg_path());
            cmd.args(["-v", "error", "-y", "-i", &path, "-map", "0:v:0", "-an", "-vf", "scale=-2:480", "-g", "1", "-pix_fmt", "yuv420p"])
                .args(enc)
                .args(["-movflags", "+faststart", "-f", "mp4"])
                .arg(&part)
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null());
            #[cfg(windows)]
            {
                use std::os::windows::process::CommandExt;
                cmd.creation_flags(0x0800_0000);
            }
            if cmd.status().map_or(false, |s| s.success()) && std::fs::metadata(&part).map_or(false, |m| m.len() > 0) {
                std::fs::rename(&part, &out).map_err(|e| e.to_string())?;
                return Ok(out.to_string_lossy().into_owned());
            }
            let _ = std::fs::remove_file(&part);
        }
        Err("impossibile creare la copia di lavoro del video".into())
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Cancella le copie di lavoro che non servono piu' (`keep` = quelle ancora in uso; vuoto = tutte).
#[tauri::command]
pub fn video_cleanup(keep: Vec<String>) {
    if let Ok(rd) = std::fs::read_dir(proxy_dir()) {
        for e in rd.flatten() {
            let p = e.path().to_string_lossy().into_owned();
            if !keep.contains(&p) {
                let _ = std::fs::remove_file(e.path());
            }
        }
    }
}

// ---------------------------------------------------------------- configurazione automatica (rilevamento e misure)

#[derive(Serialize)]
pub struct HwInfo {
    pub cpu: String,
    pub cores: usize,
    pub ram_gb: f64,
    pub os: String,
    /// encoder H.264 presenti in FFmpeg (hardware e software), nell'ordine di preferenza
    pub encoders: Vec<String>,
    pub ffmpeg: bool,
}

fn run_ffmpeg(args: &[&str]) -> Option<std::process::Output> {
    let mut cmd = std::process::Command::new(crate::decode::ffmpeg_path());
    cmd.args(args).stdin(std::process::Stdio::null());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x0800_0000);
    }
    cmd.output().ok()
}

/// Rileva CPU, memoria, sistema e gli encoder H.264 disponibili (quelli hardware prima).
#[tauri::command]
pub async fn video_hw() -> HwInfo {
    tauri::async_runtime::spawn_blocking(|| {
        let mut sys = sysinfo::System::new();
        sys.refresh_memory();
        sys.refresh_cpu();
        let cpu = sys.cpus().first().map(|c| c.brand().trim().to_string()).filter(|b| !b.is_empty()).unwrap_or_else(|| "sconosciuta".into());
        let (mut encoders, mut ffmpeg) = (vec![], false);
        if let Some(o) = run_ffmpeg(&["-hide_banner", "-encoders"]) {
            ffmpeg = o.status.success();
            let list = String::from_utf8_lossy(&o.stdout).to_string();
            for e in ["h264_videotoolbox", "h264_nvenc", "h264_qsv", "h264_amf", "h264_mf", "libx264"] {
                if list.lines().any(|l| l.split_whitespace().nth(1) == Some(e)) {
                    encoders.push(e.to_string());
                }
            }
        }
        HwInfo {
            cpu,
            cores: std::thread::available_parallelism().map_or(1, |n| n.get()),
            ram_gb: (sys.total_memory() as f64 / 1_073_741_824.0 * 10.0).round() / 10.0,
            os: format!("{} {}", std::env::consts::OS, std::env::consts::ARCH),
            encoders,
            ffmpeg,
        }
    })
    .await
    .unwrap_or(HwInfo { cpu: String::new(), cores: 1, ram_gb: 0.0, os: String::new(), encoders: vec![], ffmpeg: false })
}

#[derive(Serialize)]
pub struct BenchClip {
    pub path: String,
    pub encoder: String,
    /// fotogrammi codificati al secondo (1080p): dice quanto regge la registrazione MP4
    pub encode_fps: f64,
}

/// Crea un breve video di prova 1080p (3 s) con il miglior encoder disponibile e misura quanto e' stato veloce.
/// La finestra lo riproduce per misurare la decodifica; poi lo si cancella con `video_bench_clean`.
#[tauri::command]
pub async fn video_bench_clip() -> Result<BenchClip, String> {
    tauri::async_runtime::spawn_blocking(|| -> Result<BenchClip, String> {
        let dir = proxy_dir();
        std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        let out = dir.join("bench-clip.mp4");
        let _ = std::fs::remove_file(&out);
        let encoders: [(&str, &[&str]); 5] = [
            ("h264_videotoolbox", &["-b:v", "8M"]),
            ("h264_nvenc", &["-b:v", "8M"]),
            ("h264_qsv", &["-b:v", "8M"]),
            ("h264_amf", &["-b:v", "8M"]),
            ("libx264", &["-preset", "veryfast", "-crf", "23"]),
        ];
        let frames = 90.0;
        for (name, extra) in encoders {
            let t0 = std::time::Instant::now();
            let mut args: Vec<&str> = vec!["-v", "error", "-y", "-f", "lavfi", "-i", "testsrc2=size=1920x1080:rate=30:duration=3", "-an", "-pix_fmt", "yuv420p", "-c:v", name];
            args.extend_from_slice(extra);
            let o = out.to_string_lossy().into_owned();
            args.extend_from_slice(&["-movflags", "+faststart", &o]);
            let ok = run_ffmpeg(&args).map_or(false, |r| r.status.success());
            if ok && std::fs::metadata(&out).map_or(false, |m| m.len() > 0) {
                return Ok(BenchClip { path: o, encoder: name.to_string(), encode_fps: frames / t0.elapsed().as_secs_f64().max(0.001) });
            }
            let _ = std::fs::remove_file(&out);
        }
        Err("nessun encoder H.264 disponibile (FFmpeg mancante?)".into())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub fn video_bench_clean() {
    let _ = std::fs::remove_file(proxy_dir().join("bench-clip.mp4"));
}

/// Velocita' di decodifica del video di prova, in multipli del tempo reale (30 s di video decodificati e scartati, con l'accelerazione del
/// sistema se c'e'). Misurarla con FFmpeg e' piu' affidabile che nel motore web, dove un video nascosto viene rallentato.
#[tauri::command]
pub async fn video_bench_decode(path: String) -> Result<f64, String> {
    tauri::async_runtime::spawn_blocking(move || -> Result<f64, String> {
        let t0 = std::time::Instant::now();
        let o = run_ffmpeg(&["-v", "error", "-hwaccel", "auto", "-stream_loop", "9", "-i", &path, "-map", "0:v:0", "-f", "null", "-"]).ok_or("ffmpeg non trovato")?;
        if !o.status.success() {
            return Err("decodifica di prova non riuscita".into());
        }
        // 3 s a 30 fps ripetuti 10 volte = 900 fotogrammi
        Ok((900.0 / t0.elapsed().as_secs_f64().max(0.001)) / 30.0)
    })
    .await
    .map_err(|e| e.to_string())?
}

// ---------------------------------------------------------------- registrazione del set in MP4
// Il video composto sulla finestra di uscita si cattura come JPEG (uno per fotogramma) e si manda a FFmpeg, che lo codifica in H.264 con
// l'encoder hardware del sistema (o x264). L'audio del master si registra come sempre in un WAV temporaneo. A fine registrazione i due
// flussi si uniscono in un MP4 (audio AAC), con la correzione dei ritardi fra i due: l'istante di partenza di audio e video e la latenza
// del dispositivo audio (l'immagine e' ritardata di tanto per coincidere col suono che si sente).

use parking_lot::Mutex;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{sync_channel, SyncSender, TrySendError};
use std::sync::Arc;
use std::time::Instant;

struct RecVideo {
    tx: Option<SyncSender<Vec<u8>>>,
    writer: Option<std::thread::JoinHandle<()>>,
    child: std::process::Child,
    path: PathBuf,
    log: PathBuf,
    t0: Option<Instant>,
    frames: u64,
    dropped: Arc<AtomicU64>,
    fps: u32,
}

/// Registrazione video in corso.
static VREC: Mutex<Option<RecVideo>> = Mutex::new(None);
/// Istante in cui e' partita la registrazione audio (messo da `rec_start`).
static AUDIO_T0: Mutex<Option<Instant>> = Mutex::new(None);
/// Percorsi temporanei preparati per la prossima registrazione (audio, video).
static VPREP: Mutex<Option<(PathBuf, PathBuf)>> = Mutex::new(None);
/// Ultima registrazione video terminata: serve all'unione con l'audio.
static VLAST: Mutex<Option<(PathBuf, Option<Instant>)>> = Mutex::new(None);

/// Chiamata da `rec_start`: segna l'istante di partenza della registrazione audio.
pub fn mark_audio_start() {
    *AUDIO_T0.lock() = Some(Instant::now());
}

#[derive(Serialize)]
pub struct RecPaths {
    pub audio: String,
    pub video: String,
}

/// Prepara i file temporanei di una registrazione con video (WAV per l'audio, MP4 senza audio per il video).
#[tauri::command]
pub fn video_rec_prepare() -> Result<RecPaths, String> {
    let dir = proxy_dir();
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let id = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_millis());
    let (a, v) = (dir.join(format!("rec-{id}.wav")), dir.join(format!("rec-{id}-video.mp4")));
    *VPREP.lock() = Some((a.clone(), v.clone()));
    Ok(RecPaths { audio: a.to_string_lossy().into_owned(), video: v.to_string_lossy().into_owned() })
}

/// Primo encoder H.264 che funziona su questo computer (hardware prima, poi x264). Si prova una volta sola.
fn rec_encoder() -> Result<&'static str, String> {
    static ENC: std::sync::OnceLock<Option<&'static str>> = std::sync::OnceLock::new();
    ENC.get_or_init(|| {
        ["h264_videotoolbox", "h264_nvenc", "h264_qsv", "h264_amf", "h264_mf", "libx264"].into_iter().find(|e| {
            run_ffmpeg(&["-v", "error", "-f", "lavfi", "-i", "color=c=black:s=1280x720:r=30", "-frames:v", "3", "-pix_fmt", "yuv420p", "-c:v", e, "-f", "null", "-"]).map_or(false, |o| o.status.success())
        })
    })
    .ok_or_else(|| "nessun encoder H.264 disponibile (FFmpeg mancante?)".to_string())
}

fn rec_bitrate_k(h: u32, fps: u32) -> u32 {
    let base = match h {
        0..=480 => 2500,
        481..=720 => 5000,
        _ => 8000,
    };
    if fps > 30 { base * 3 / 2 } else { base }
}

/// Avvia la codifica: FFmpeg legge i JPEG da stdin con la data di arrivo come tempo e produce un video a fotogrammi costanti.
fn rec_start_at(out: &Path, w: u32, h: u32, fps: u32) -> Result<(), String> {
    let mut g = VREC.lock();
    if g.is_some() {
        return Err("c'e' gia' una registrazione video in corso".into());
    }
    let (w, h, fps) = (w.max(2) & !1, h.max(2) & !1, fps.clamp(1, 120));
    let enc = rec_encoder()?;
    let log = out.with_extension("log");
    let mut cmd = std::process::Command::new(crate::decode::ffmpeg_path());
    cmd.args(["-v", "error", "-y", "-use_wallclock_as_timestamps", "1", "-f", "image2pipe", "-c:v", "mjpeg", "-i", "-"])
        .args(["-vf", &format!("fps={fps},scale={w}:{h}:flags=bicubic,format=yuv420p"), "-c:v", enc]);
    let k = format!("{}k", rec_bitrate_k(h, fps));
    match enc {
        "libx264" => { cmd.args(["-preset", "veryfast", "-crf", "23"]); }
        _ => { cmd.args(["-b:v", &k]); }
    }
    cmd.args(["-g", &(fps * 2).to_string(), "-movflags", "+faststart"])
        .arg(out)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::null())
        .stderr(std::fs::File::create(&log).map(std::process::Stdio::from).unwrap_or_else(|_| std::process::Stdio::null()));
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x0800_0000);
    }
    let mut child = cmd.spawn().map_err(|e| format!("ffmpeg non avviato: {e}"))?;
    let mut stdin = child.stdin.take().ok_or("stdin di ffmpeg non disponibile")?;
    let (tx, rx) = sync_channel::<Vec<u8>>(8);
    let writer = std::thread::spawn(move || {
        while let Ok(frame) = rx.recv() {
            if stdin.write_all(&frame).is_err() {
                break;
            }
        }
    });
    *g = Some(RecVideo { tx: Some(tx), writer: Some(writer), child, path: out.to_path_buf(), log, t0: None, frames: 0, dropped: Arc::new(AtomicU64::new(0)), fps });
    Ok(())
}

/// Un fotogramma JPEG; `lag_ms` e' il tempo passato fra il disegno e l'invio (cattura e codifica JPEG): il fotogramma e' di quell'istante.
/// Se FFmpeg non riesce a stare dietro il fotogramma si perde (e si conta) invece di bloccare la finestra di uscita.
fn rec_push(frame: Vec<u8>, lag_ms: f64) -> Result<(), String> {
    let mut g = VREC.lock();
    let r = g.as_mut().ok_or("nessuna registrazione video in corso")?;
    if r.t0.is_none() {
        r.t0 = Some(Instant::now() - std::time::Duration::from_secs_f64((lag_ms / 1000.0).clamp(0.0, 2.0)));
    }
    match r.tx.as_ref().ok_or("registrazione chiusa")?.try_send(frame) {
        Ok(()) => r.frames += 1,
        Err(TrySendError::Full(_)) => { r.dropped.fetch_add(1, Ordering::Relaxed); }
        Err(TrySendError::Disconnected(_)) => return Err("la codifica video si e' fermata".into()),
    }
    Ok(())
}

#[derive(Serialize)]
pub struct RecVideoInfo {
    pub frames: u64,
    pub dropped: u64,
    pub secs: f64,
    pub path: String,
}

/// Chiude la codifica e aspetta FFmpeg.
fn rec_stop_inner() -> Result<RecVideoInfo, String> {
    let mut r = VREC.lock().take().ok_or("nessuna registrazione video in corso")?;
    r.tx = None; // il canale si chiude: il thread di scrittura finisce e chiude stdin
    if let Some(w) = r.writer.take() {
        let _ = w.join();
    }
    let st = r.child.wait().map_err(|e| e.to_string())?;
    if !st.success() || std::fs::metadata(&r.path).map_or(true, |m| m.len() == 0) {
        let why = std::fs::read_to_string(&r.log).unwrap_or_default();
        return Err(format!("la codifica video non e' riuscita{}", if why.trim().is_empty() { String::new() } else { format!(": {}", why.trim().lines().last().unwrap_or("")) }));
    }
    let _ = std::fs::remove_file(&r.log);
    *VLAST.lock() = Some((r.path.clone(), r.t0));
    Ok(RecVideoInfo { frames: r.frames, dropped: r.dropped.load(Ordering::Relaxed), secs: r.frames as f64 / r.fps.max(1) as f64, path: r.path.to_string_lossy().into_owned() })
}

#[tauri::command]
pub fn video_rec_start(width: u32, height: u32, fps: u32) -> Result<(), String> {
    let (_, v) = VPREP.lock().clone().ok_or("registrazione non preparata")?;
    rec_start_at(&v, width, height, fps)
}

/// Un fotogramma JPEG nel corpo della richiesta (binario, senza passare da JSON); intestazione `x-lag-ms` con il ritardo di cattura.
#[tauri::command]
pub fn video_rec_frame(request: tauri::ipc::Request<'_>) -> Result<(), String> {
    let lag = request.headers().get("x-lag-ms").and_then(|v| v.to_str().ok()).and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0);
    match request.body() {
        tauri::ipc::InvokeBody::Raw(b) => rec_push(b.clone(), lag),
        _ => Err("fotogramma non valido".into()),
    }
}

#[tauri::command]
pub async fn video_rec_stop() -> Result<RecVideoInfo, String> {
    tauri::async_runtime::spawn_blocking(rec_stop_inner).await.map_err(|e| e.to_string())?
}

#[derive(Serialize)]
pub struct MuxResult {
    pub path: String,
    pub bytes: u64,
    /// spostamento applicato fra audio e video, in ms (positivo = il video parte dopo l'audio)
    pub offset_ms: f64,
}

/// Unisce il video catturato e l'audio registrato in un MP4 (H.264 + AAC) e cancella i file temporanei.
/// `latency_ms`: ritardo con cui l'immagine e' stata mostrata rispetto al suono registrato (latenza del dispositivo + allineamento scelto).
fn mux_inner(audio: &str, out: &str, latency_ms: f64) -> Result<MuxResult, String> {
    let (video, vt0) = VLAST.lock().clone().ok_or("nessun video registrato da unire")?;
    let at0 = *AUDIO_T0.lock();
    // quando e' partito il video rispetto all'audio (secondi); se manca uno dei due istanti si assume 0
    let start_diff = match (vt0, at0) {
        (Some(v), Some(a)) => if v >= a { v.duration_since(a).as_secs_f64() } else { -a.duration_since(v).as_secs_f64() },
        _ => 0.0,
    };
    // il fotogramma mostrato a t contiene l'istante t - latenza della musica: va spostato indietro di tanto sulla linea del tempo dell'audio
    let shift = start_diff - latency_ms / 1000.0;
    let (v_off, a_off) = if shift >= 0.0 { (shift, 0.0) } else { (0.0, -shift) };
    let _ = std::fs::remove_file(out);
    let o = run_ffmpeg(&[
        "-v", "error", "-y",
        "-itsoffset", &format!("{v_off:.3}"), "-i", &video.to_string_lossy(),
        "-itsoffset", &format!("{a_off:.3}"), "-i", audio,
        "-map", "0:v:0", "-map", "1:a:0", "-c:v", "copy", "-c:a", "aac", "-b:a", "192k", "-movflags", "+faststart", out,
    ])
    .ok_or("ffmpeg non trovato")?;
    if !o.status.success() || std::fs::metadata(out).map_or(true, |m| m.len() == 0) {
        return Err(format!("unione di audio e video non riuscita (i file temporanei sono in {}): {}", proxy_dir().display(), String::from_utf8_lossy(&o.stderr).trim().lines().last().unwrap_or("")));
    }
    let _ = std::fs::remove_file(&video);
    let _ = std::fs::remove_file(audio);
    Ok(MuxResult { path: out.to_string(), bytes: std::fs::metadata(out).map(|m| m.len()).unwrap_or(0), offset_ms: shift * 1000.0 })
}

#[tauri::command]
pub async fn video_rec_mux(audio: String, out: String, latency_ms: f64) -> Result<MuxResult, String> {
    tauri::async_runtime::spawn_blocking(move || mux_inner(&audio, &out, latency_ms)).await.map_err(|e| e.to_string())?
}

#[cfg(test)]
mod tests {
    use super::*;

    fn have(bin: &str) -> bool {
        std::process::Command::new(bin).arg("-version").stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null()).status().map_or(false, |s| s.success())
    }

    /// Divide un flusso MJPEG nei suoi JPEG (inizio FFD8, fine FFD9).
    fn split_jpegs(b: &[u8]) -> Vec<Vec<u8>> {
        let mut out = vec![];
        let mut i = 0;
        while i + 1 < b.len() {
            if b[i] == 0xFF && b[i + 1] == 0xD8 {
                let mut j = i + 2;
                while j + 1 < b.len() && !(b[j] == 0xFF && b[j + 1] == 0xD9) {
                    j += 1;
                }
                out.push(b[i..(j + 2).min(b.len())].to_vec());
                i = j + 2;
            } else {
                i += 1;
            }
        }
        out
    }

    fn probe(path: &str, entries: &str, stream: &str) -> String {
        let o = std::process::Command::new("ffprobe").args(["-v", "error", "-select_streams", stream, "-show_entries", entries, "-of", "csv=p=0", path]).output().unwrap();
        String::from_utf8_lossy(&o.stdout).trim().to_string()
    }

    /// Registrazione completa con FFmpeg vero: 60 fotogrammi JPEG a 30 al secondo (2 s) + un WAV di 2 s -> un MP4 con video H.264 e audio AAC
    /// della durata giusta, e con lo spostamento calcolato dalla partenza ritardata del video (misurata nel test).
    #[test]
    fn record_video_and_mux_with_audio() {
        if !have("ffmpeg") || !have("ffprobe") {
            eprintln!("ffmpeg/ffprobe assenti: prova saltata");
            return;
        }
        let dir = std::env::temp_dir().join("spinlab_video_test");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        // fotogrammi di prova
        let mj = run_ffmpeg(&["-v", "error", "-f", "lavfi", "-i", "testsrc2=size=640x360:rate=30:duration=2", "-f", "image2pipe", "-c:v", "mjpeg", "-q:v", "5", "-"]).unwrap();
        let frames = split_jpegs(&mj.stdout);
        assert!(frames.len() >= 55, "pochi fotogrammi di prova: {}", frames.len());
        // audio di 2 s (sinusoide) in un WAV
        let wav = dir.join("a.wav");
        {
            let spec = hound::WavSpec { channels: 2, sample_rate: 44100, bits_per_sample: 16, sample_format: hound::SampleFormat::Int };
            let mut w = hound::WavWriter::create(&wav, spec).unwrap();
            for n in 0..(44100 * 2) {
                let v = ((n as f32 * 440.0 * std::f32::consts::TAU / 44100.0).sin() * 8000.0) as i16;
                w.write_sample(v).unwrap();
                w.write_sample(v).unwrap();
            }
            w.finalize().unwrap();
        }
        let vid = dir.join("v.mp4");
        mark_audio_start();
        let audio_t0 = Instant::now();
        std::thread::sleep(std::time::Duration::from_millis(120)); // il video parte dopo l'audio (piu' il tempo di avvio di FFmpeg)
        rec_start_at(&vid, 640, 360, 30).unwrap();
        let t0 = Instant::now();
        for (i, f) in frames.iter().take(60).enumerate() {
            rec_push(f.clone(), 0.0).unwrap();
            let due = std::time::Duration::from_secs_f64((i + 1) as f64 / 30.0);
            if let Some(w) = due.checked_sub(t0.elapsed()) {
                std::thread::sleep(w);
            }
        }
        let info = rec_stop_inner().unwrap();
        assert!(info.frames >= 55 && info.dropped == 0, "fotogrammi {} persi {}", info.frames, info.dropped);
        let out = dir.join("out.mp4");
        let m = mux_inner(&wav.to_string_lossy(), &out.to_string_lossy(), 0.0).unwrap();
        assert!(m.bytes > 1000);
        let expected = t0.duration_since(audio_t0).as_secs_f64() * 1000.0;   // dalla partenza dell'audio al primo fotogramma
        assert!((m.offset_ms - expected).abs() < 40.0, "spostamento {} ms, atteso {expected}", m.offset_ms);
        let o = out.to_string_lossy();
        assert_eq!(probe(&o, "stream=codec_name", "v:0"), "h264");
        assert_eq!(probe(&o, "stream=codec_name", "a:0"), "aac");
        let dur: f64 = probe(&o, "format=duration", "v:0").parse().unwrap_or(0.0);
        assert!((1.8..2.6).contains(&dur), "durata {dur}");
        // i temporanei sono stati cancellati
        assert!(!vid.exists() && !wav.exists());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
