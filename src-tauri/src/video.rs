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
