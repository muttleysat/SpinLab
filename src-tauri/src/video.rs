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
