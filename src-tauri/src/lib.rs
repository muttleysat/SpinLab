
mod analysis;
mod audio;
mod cue;
mod camelot;
mod decode;
mod engine;
mod fx;
mod lame;
mod license;
mod library;
#[cfg(not(target_os = "android"))]
mod midi;
#[cfg(target_os = "android")]
#[path = "midi_stub.rs"]
mod midi;
mod recorder;
mod preview;
mod stems;

use engine::{Mixer, Snapshot, Track, NUM_DECKS, NUM_DRUMS, NUM_PADS};
use parking_lot::Mutex;
use serde::Serialize;
use std::sync::Arc;
use library::{LibTrack, Library, Playlist, TagUpdate};
use preview::{PreviewState, Previewer};
use tauri::{Emitter, Manager, State};

struct App {
    mixer: Arc<Mutex<Mixer>>,
    audio: audio::Audio,
    dir: std::path::PathBuf,
}

#[derive(Serialize)]
struct TrackInfo {
    name: String,
    duration: f64,
    bpm: f64,
    key: Option<String>,
    key_name: Option<String>,
    wave: WaveInfo,
    beats: Vec<f64>,
    downbeat: usize,
    fixed: bool,
    norm_db: f64,
    loudness: f64,
    /// diagnostica del caricamento: tempo totale e di decodifica (ms), analisi gia' in cache o calcolata ora
    ms: u128,
    decode_ms: u128,
    cached: bool,
}

#[derive(Serialize)]
struct WaveInfo {
    rate: usize,
    lo: Vec<u8>,
    mid: Vec<u8>,
    hi: Vec<u8>,
}

fn file_title(path: &str) -> String {
    std::path::Path::new(path)
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.to_string())
}

/// avanzamento dell'analisi di un file (0..100) verso l'interfaccia: barra sulla riga del brano
fn emit_progress(app: &tauri::AppHandle, path: &str, pct: u32) {
    let _ = app.emit("analysis-track", (path.to_string(), pct));
}

/// Analisi completa di un brano gia' decodificato, salvata nel database; il 10% iniziale e' la decodifica.
/// numero di caricamenti sui deck in corso: l'analisi in background si ferma finche' non finiscono
static LOADING: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

/// Numero dell'ultima richiesta di caricamento per deck: se una richiesta lenta finisce dopo una piu' recente, viene scartata.
static LOAD_SEQ: [std::sync::atomic::AtomicU64; NUM_DECKS] = [const { std::sync::atomic::AtomicU64::new(0) }; NUM_DECKS];

struct LoadGuard;
impl LoadGuard {
    fn new() -> Self {
        LOADING.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        LoadGuard
    }
}
impl Drop for LoadGuard {
    fn drop(&mut self) {
        LOADING.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
    }
}

fn run_analysis(app: &tauri::AppHandle, lib: &Library, path: &str, data: &[f32], sr: u32, background: bool) -> analysis::Analysis {
    emit_progress(app, path, 10);
    let mut last = 10u32;
    let a = analysis::analyze_progress(data, sr, &mut |p| {
        // l'analisi in background cede il processore a chi sta caricando un brano sul deck
        while background && LOADING.load(std::sync::atomic::Ordering::SeqCst) > 0 {
            std::thread::sleep(std::time::Duration::from_millis(40));
        }
        let pct = 10 + (p * 89.0) as u32;
        if pct >= last + 2 {
            last = pct;
            emit_progress(app, path, pct);
        }
    });
    lib.analysis_put(path, &a);
    emit_progress(app, path, 100);
    a
}

#[tauri::command]
async fn load_track(
    handle: tauri::AppHandle,
    app: State<'_, App>,
    lib: State<'_, Arc<Library>>,
    hub: State<'_, Arc<stems::Hub>>,
    deck: usize,
    path: String,
) -> Result<TrackInfo, String> {
    if deck >= NUM_DECKS {
        return Err("deck non valido".into());
    }
    if !license::allowed(&app.dir) {
        return Err("Programma non registrato: apri Informazioni e inserisci la chiave di licenza".into());
    }
    let _guard = LoadGuard::new();
    let my_seq = LOAD_SEQ[deck].fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
    let t_start = std::time::Instant::now();
    let sr = app.mixer.lock().sr;
    let p = path.clone();
    let l = lib.inner().clone();
    // un brano gia' analizzato (in playlist) ha tutto in cache: resta solo la decodifica. Altrimenti si analizza ora.
    let (track, wave, loud, cache_hit, decode_ms) = tauri::async_runtime::spawn_blocking(move || -> Result<(Track, analysis::Wave, (f64, f32), bool, u128), String> {
        let cached = l.analysis_get(&p);
        let hit = cached.is_some();
        if cached.is_none() {
            emit_progress(&handle, &p, 0);
        }
        let t_dec = std::time::Instant::now();
        let data = decode::decode_file(&p, sr)?;
        let decode_ms = t_dec.elapsed().as_millis();
        let a = match cached {
            Some(c) => analysis::Analysis { beats: c.beats, bpm: c.bpm, downbeat: c.downbeat, key: c.key, wave: c.wave, loudness: c.loudness, peak: c.peak },
            None => run_analysis(&handle, &l, &p, &data, sr, false),
        };
        Ok((
            Track { name: file_title(&p), path: p.clone(), data: Arc::new(data), sr, beats: a.beats, bpm: a.bpm, key: a.key, downbeat: a.downbeat },
            a.wave,
            (a.loudness, a.peak),
            hit,
            decode_ms,
        ))
    })
    .await
    .map_err(|e| e.to_string())??;
    if LOAD_SEQ[deck].load(std::sync::atomic::Ordering::SeqCst) != my_seq {
        return Err("superato".into()); // nel frattempo e' stato chiesto un altro brano per questo deck
    }
    let saved = lib.load_grid(&path);
    let (key, key_name, name, duration) = (track.key.map(camelot::camelot_code), track.key.map(camelot::key_name), track.name.clone(), track.duration());
    let (beats, downbeat, bpm, fixed, norm_db) = {
        let mut m = app.mixer.lock();
        let d = &mut m.decks[deck];
        d.load(Arc::new(track));
        d.loudness = loud.0;
        d.peak = loud.1;
        // una griglia corretta a mano ha la precedenza sull'analisi automatica
        if let Some((b, dn, bp, fx, pins)) = saved {
            d.grid_apply_saved(b, dn, bp, fx, pins);
        }
        let g = d.grid_info().unwrap();
        // volume: allineato a quello del brano che sta suonando su un altro deck
        m.normalize_deck(deck);
        (g.beats, g.downbeat, g.bpm, g.fixed, m.decks[deck].norm_db)
    };
    // stem: la sessione del brano precedente si chiude e, se attivo, parte la separazione del nuovo
    stems_attach(app.mixer.clone(), hub.inner().clone(), app.dir.clone(), deck, path.clone());
    Ok(TrackInfo {
        name,
        duration,
        bpm,
        key,
        key_name,
        wave: WaveInfo { rate: wave.rate, lo: wave.lo, mid: wave.mid, hi: wave.hi },
        beats,
        downbeat,
        fixed,
        norm_db,
        loudness: loud.0,
        ms: t_start.elapsed().as_millis(),
        decode_ms,
        cached: cache_hit,
    })
}

/// Modifica della griglia di un deck; la correzione viene salvata per quel file.
#[tauri::command]
fn grid_cmd(app: State<'_, App>, lib: State<'_, Arc<Library>>, deck: usize, cmd: String, a: f64) -> Option<engine::GridInfo> {
    if deck >= NUM_DECKS {
        return None;
    }
    let mut m = app.mixer.lock();
    let d = &mut m.decks[deck];
    let path = d.track.as_ref()?.path.clone();
    if !d.grid_cmd(&cmd, a) {
        return None;
    }
    let g = d.grid_info()?;
    if cmd == "reset" {
        lib.delete_grid(&path);
    } else {
        lib.save_grid(&path, &g.beats, g.downbeat, g.bpm, g.fixed, d.grid_pins(), d.grid_auto());
    }
    Some(g)
}

/// Comandi semplici per deck: (nome, a, b)
#[tauri::command]
fn deck_cmd(app: State<'_, App>, deck: usize, cmd: String, a: f64, b: f64) {
    if deck >= NUM_DECKS {
        return;
    }
    let mut m = app.mixer.lock();
    let d = &mut m.decks[deck];
    match cmd.as_str() {
        "unload" => d.unload(),
        "play" => d.play_toggle(),
        "play_q" => d.play_quantized(),
        "cue_press" => d.cue_press(),
        "cue_release" => d.cue_release(),
        "hotcue" => d.hot_cue_press(a as usize),
        "hotcue_clear" => d.hot_cue_clear(a as usize),
        "seek" => d.set_pos_sec(a),
        "hold" => { d.held = a > 0.5; if !d.held { d.scrub_to(None); } }
        "scrub" => d.scrub_to(Some(a)),
        // muovere il pitch a mano spegne il sync
        // muovere a mano il pitch di un deck che segue spegne il suo sync (la guida invece puo' muoversi: gli altri la seguono)
        "tempo" => {
            d.set_tempo_pct(a);
            if m.is_follower(deck) {
                m.decks[deck].sync = false;
            }
        }
        "range" => d.set_range_pct(a),
        "keylock" => d.keylock = a > 0.5,
        "quantize" => d.quantize = a > 0.5,
        "key_wheel" => d.key_wheel(a as i32),
        "key_semitone" => d.key_shift = (d.key_shift + a as i32).clamp(-12, 12),
        "key_reset" => d.key_shift = 0,
        "volume" => d.volume = a as f32,
        "filter" => d.filter = a.clamp(0.0, 1.0) as f32,
        "beatjump" => d.beatjump(a),
        "roll_on" => d.roll_on(a),
        "roll_off" => d.roll_off(),
        "brake_on" => d.brake_on(a),
        "spin_on" => d.spin_on(a),
        "reverse_on" => d.reverse_on(),
        "motion_off" => d.motion_off(),
        "slip" => d.slip = a > 0.5,
        "nudge" => d.nudge = a.clamp(-0.5, 0.5),
        "loop_toggle" => d.loop_toggle(),
        "loop_in" => d.loop_in_here(),
        "loop_out" => d.loop_out_here(),
        "loop_size" => d.loop_size(a),
        "loop_halve" => d.loop_step(false),
        "loop_double" => d.loop_step(true),
        "eq" => d.eq_knobs[(a as usize).min(2)] = b as f32,
        "xf_assign" => d.xf_assign = a as u8,
        "pfl" => d.pfl = a > 0.5,
        "sync_off" => d.sync = false,
        "sync" => m.sync(deck),
        _ => {}
    }
}

#[tauri::command]
fn fx_set(app: State<'_, App>, deck: usize, kind: String, on: bool, wet: f32, param: f32, param2: Option<f32>, div: Option<u32>, feel: Option<u32>) {
    if deck >= NUM_DECKS {
        return;
    }
    let mut m = app.mixer.lock();
    let f = &mut m.decks[deck].fx;
    let new_kind = fx::FxKind::from_str(&kind);
    if new_kind != f.kind {
        f.reset_state();
    }
    f.kind = new_kind;
    f.on = on;
    if let Some(d) = div {
        f.div = (d as usize).min(fx::DIVS.len() - 1);
    }
    if let Some(fe) = feel {
        f.feel = fe.min(2) as u8;
    }
    f.wet = wet.clamp(0.0, 1.0);
    f.param = param.clamp(0.0, 1.0);
    if let Some(p) = param2 {
        f.param2 = p.clamp(0.0, 1.0);
    }
}

#[tauri::command]
fn mixer_cmd(app: State<'_, App>, cmd: String, a: f64) {
    let mut m = app.mixer.lock();
    match cmd.as_str() {
        "crossfader" => m.crossfader = a.clamp(0.0, 1.0) as f32,
        "master" => m.master = a as f32,
        "sampler_vol" => m.sampler_vol = a as f32,
        "pad_sync" => m.pad_sync = a > 0.5,
        "drum_vol" => m.drum_vol = a.clamp(0.0, 1.5) as f32,
        "hp_vol" => m.hp_vol = a.clamp(0.0, 1.0) as f32,
        "hp_mix" => m.hp_mix = a.clamp(0.0, 1.0) as f32,
        "cue_mix" => m.cue_mix = a.clamp(0.0, 1.0) as f32,
        "auto_gain" => {
            m.auto_gain = a > 0.5;
            if !m.auto_gain {
                m.decks.iter_mut().for_each(|d| d.norm_db = 0.0);
            }
        }
        _ => {}
    }
}

#[tauri::command]
async fn pad_load(app: State<'_, App>, pad: usize, path: String) -> Result<String, String> {
    if pad >= NUM_PADS {
        return Err("pad non valido".into());
    }
    let sr = app.mixer.lock().sr;
    let p = path.clone();
    let data = tauri::async_runtime::spawn_blocking(move || decode::decode_file(&p, sr))
        .await
        .map_err(|e| e.to_string())??;
    let name = file_title(&path);
    // tempo del campione (per il modo sync): analisi dei campioni non troppo lunghi; un loop ha una durata di battiti interi
    let mut is_loop = false;
    let (bpm, first_beat) = if data.len() as f64 / 2.0 / sr as f64 <= 90.0 && data.len() as f64 / 2.0 / sr as f64 >= 1.2 {
        let secs = data.len() as f64 / 2.0 / sr as f64;
        let a = analysis::analyze(&data, sr);
        if a.bpm > 0.0 && a.beats.len() >= 3 {
            let n = (secs * a.bpm / 60.0).round();
            // durata = numero intero di battiti: e' un loop (drumloop), quindi ripete da solo finche' non lo fermi
            is_loop = n >= 2.0 && (n * 60.0 / secs - a.bpm).abs() / a.bpm < 0.04;
            let bpm = if n >= 1.0 && (n * 60.0 / secs - a.bpm).abs() / a.bpm < 0.04 { n * 60.0 / secs } else { a.bpm };
            (bpm, a.beats[0] * sr as f64)
        } else {
            (0.0, 0.0)
        }
    } else {
        (0.0, 0.0)
    };
    app.mixer.lock().pads[pad].load(name.clone(), data, bpm, first_beat, is_loop);
    Ok(name)
}

#[tauri::command]
fn pad_cmd(app: State<'_, App>, pad: usize, cmd: String, a: f64) {
    let mut m = app.mixer.lock();
    match cmd.as_str() {
        "trigger" => m.pad_trigger(pad),
        "stop" => m.pad_stop(pad),
        "loop" => {
            if let Some(p) = m.pads.get_mut(pad) {
                p.looped = a > 0.5;
            }
        }
        "gain" => {
            if let Some(p) = m.pads.get_mut(pad) {
                p.gain = a as f32;
            }
        }
        "bpm" => {
            if let Some(p) = m.pads.get_mut(pad) {
                p.set_bpm(a);
            }
        }
        "bpm_half" | "bpm_double" => {
            if let Some(p) = m.pads.get_mut(pad) {
                let b = p.bpm;
                p.set_bpm(if cmd == "bpm_half" { b / 2.0 } else { b * 2.0 });
            }
        }
        _ => {}
    }
}

// ---------------------------------------------------------------- batteria

#[derive(Serialize, serde::Deserialize, Default, Clone)]
struct KitPad {
    path: String,
    gain: f32,
    tune: f32,
    group: u8,
    gate: bool,
    note: u8,
    #[serde(default = "default_at_mode")]
    at_mode: u8,
}

fn default_at_mode() -> u8 { 1 }

/// Salva le note MIDI assegnate ai pad. I campioni NON si ricordano: ad ogni avvio la batteria parte vuota.
fn save_kit(mixer: &Arc<Mutex<Mixer>>, dir: &std::path::Path) {
    let kit: Vec<KitPad> = {
        let m = mixer.lock();
        (0..NUM_DRUMS).map(|i| KitPad { path: String::new(), gain: 1.0, tune: 0.0, group: 0, gate: false, note: m.drum_notes[i], at_mode: m.drums[i].at_mode }).collect()
    };
    let _ = std::fs::write(dir.join("drums.json"), serde_json::to_string_pretty(&kit).unwrap_or_default());
}

/// All'avvio ricarica solo le note MIDI (anche da vecchi file che contenevano i percorsi dei campioni, ignorati).
fn load_kit(mixer: &Arc<Mutex<Mixer>>, dir: &std::path::Path) {
    let Some(kit) = std::fs::read_to_string(dir.join("drums.json")).ok().and_then(|t| serde_json::from_str::<Vec<KitPad>>(&t).ok()) else { return };
    let mut m = mixer.lock();
    for (i, k) in kit.into_iter().enumerate().take(NUM_DRUMS) {
        m.drum_notes[i] = k.note;
        m.drums[i].at_mode = k.at_mode & 3;
    }
}

#[tauri::command]
async fn drum_load(app: State<'_, App>, pad: usize, path: String) -> Result<String, String> {
    if pad >= NUM_DRUMS {
        return Err("pad non valido".into());
    }
    let sr = app.mixer.lock().sr;
    let p = path.clone();
    let data = tauri::async_runtime::spawn_blocking(move || decode::decode_file(&p, sr)).await.map_err(|e| e.to_string())??;
    if data.len() as f64 / 2.0 / sr as f64 > 30.0 {
        return Err("campione troppo lungo per un pad della batteria (massimo 30 s): usa il campionatore".into());
    }
    let name = file_title(&path);
    app.mixer.lock().drums[pad].load(name.clone(), path, data);
    save_kit(&app.mixer, &app.dir);
    Ok(name)
}

#[tauri::command]
fn drum_cmd(app: State<'_, App>, pad: usize, cmd: String, a: f64) {
    if pad >= NUM_DRUMS {
        return;
    }
    let mut save = true;
    {
        let mut m = app.mixer.lock();
        match cmd.as_str() {
            "trigger" => { m.drum_trigger(pad, a as f32); save = false; }
            "release" => { m.drum_release(pad); save = false; }
            "gain" => m.drums[pad].gain = a.clamp(0.0, 1.5) as f32,
            "tune" => m.drums[pad].tune = a.clamp(-24.0, 24.0) as f32,
            "group" => m.drums[pad].group = a.clamp(0.0, 8.0) as u8,
            "gate" => m.drums[pad].gate = a > 0.5,
            "aftertouch" => m.drums[pad].at_mode = (a as u8).min(3),
            "clear" => { m.drums[pad].clear(); m.drum_notes[pad] = 36 + pad as u8; }
            "note" => m.drum_notes[pad] = a.clamp(0.0, 255.0) as u8,
            "learn" => { m.learn = if a > 0.5 { Some(pad) } else { None }; save = false; }
            _ => save = false,
        }
    }
    if save {
        save_kit(&app.mixer, &app.dir);
    }
}

/// salva note MIDI e impostazioni dopo un apprendimento
#[tauri::command]
fn drum_save(app: State<'_, App>) {
    save_kit(&app.mixer, &app.dir);
}

/// controller MIDI collegati
#[tauri::command]
fn midi_ports() -> Vec<String> {
    midi::PORTS.lock().clone()
}

/// porte MIDI di uscita (per i LED del controller) e quella aperta
#[derive(Serialize)]
struct MidiOutInfo {
    ports: Vec<String>,
    current: String,
}

#[tauri::command]
fn midi_out_info() -> MidiOutInfo {
    MidiOutInfo { ports: midi::out_ports(), current: midi::out_current() }
}

#[tauri::command]
fn midi_out_select(name: String) -> Result<(), String> {
    midi::out_select(&name)
}

#[tauri::command]
fn midi_send(bytes: Vec<u8>) {
    midi::send(&bytes);
}

/// mappatura dei controller (JSON gestito dall'interfaccia)
#[tauri::command]
fn midi_map_get(a: State<'_, App>) -> String {
    std::fs::read_to_string(a.dir.join("midimap.json")).unwrap_or_default()
}

#[tauri::command]
fn midi_map_set(a: State<'_, App>, json: String) -> Result<(), String> {
    std::fs::write(a.dir.join("midimap.json"), json).map_err(|e| e.to_string())
}

// ---------------------------------------------------------------- registrazione

#[derive(Default)]
struct RecState(Mutex<Option<recorder::Recorder>>);

#[derive(Serialize)]
struct RecResult {
    path: String,
    secs: f64,
    dropped: u64,
    bytes: u64,
}

/// Avvia la registrazione dell'uscita master (tutto: deck, effetti, campionatore, batteria) nel file scelto.
/// `format`: wav16, wav24, mp3-128, mp3-192, mp3-256, mp3-320.
#[tauri::command]
fn rec_start(app: State<'_, App>, rec: State<'_, RecState>, path: String, format: String) -> Result<(), String> {
    let mut g = rec.0.lock();
    if g.is_some() {
        return Err("c'e' gia' una registrazione in corso".into());
    }
    let sr = app.mixer.lock().sr;
    let (r, tap) = recorder::start(std::path::Path::new(&path), recorder::Format::parse(&format), sr)?;
    app.mixer.lock().rec = Some(tap);
    *g = Some(r);
    Ok(())
}

/// Ferma la registrazione e chiude il file.
#[tauri::command]
fn rec_stop(app: State<'_, App>, rec: State<'_, RecState>) -> Result<RecResult, String> {
    let r = rec.0.lock().take().ok_or("nessuna registrazione in corso")?;
    app.mixer.lock().rec = None; // il canale si chiude e il thread scrive la fine del file
    let f = r.finish()?;
    Ok(RecResult { path: f.path.to_string_lossy().into_owned(), secs: f.secs, dropped: f.dropped, bytes: f.bytes })
}

#[tauri::command]
fn get_state(app: State<'_, App>) -> Snapshot {
    app.mixer.lock().snapshot()
}

// ---------------------------------------------------------------- libreria

#[tauri::command]
async fn lib_scan(app: tauri::AppHandle, lib: State<'_, Arc<Library>>, folder: String) -> Result<usize, String> {
    let l = lib.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        l.scan_folder(&folder, |n| {
            let _ = app.emit("lib-progress", n);
        })
    })
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
fn lib_add_files(lib: State<'_, Arc<Library>>, paths: Vec<String>) -> Vec<i64> {
    // le cartelle trascinate vengono esplorate (sottocartelle comprese)
    let files: Vec<String> = paths.into_iter().flat_map(|p| if std::path::Path::new(&p).is_dir() { audio_files(&p, true) } else { vec![p] }).collect();
    files.iter().filter_map(|p| lib.add_file(p).ok()).collect()
}

#[tauri::command]
fn lib_search(lib: State<'_, Arc<Library>>, query: String, playlist: Option<i64>, sort: String, desc: bool) -> Vec<LibTrack> {
    lib.search(&query, playlist, &sort, desc)
}

#[tauri::command]
fn lib_update_tags(lib: State<'_, Arc<Library>>, id: i64, update: TagUpdate) -> Result<(), String> {
    lib.update_tags(id, update)
}

#[tauri::command]
fn lib_remove(lib: State<'_, Arc<Library>>, ids: Vec<i64>) {
    lib.remove_tracks(&ids)
}

#[tauri::command]
fn playlists(lib: State<'_, Arc<Library>>) -> Vec<Playlist> {
    lib.playlists()
}

#[tauri::command]
fn playlist_create(lib: State<'_, Arc<Library>>, name: String) -> Result<i64, String> {
    lib.playlist_create(&name)
}

#[tauri::command]
fn playlist_rename(lib: State<'_, Arc<Library>>, id: i64, name: String) {
    lib.playlist_rename(id, &name)
}

#[tauri::command]
fn playlist_delete(lib: State<'_, Arc<Library>>, id: i64) {
    lib.playlist_delete(id)
}

#[tauri::command]
fn playlist_add(app: tauri::AppHandle, lib: State<'_, Arc<Library>>, q: State<'_, Arc<AnalysisQueue>>, id: i64, tracks: Vec<i64>) {
    lib.playlist_add(id, &tracks);
    queue_analysis(&app, q.inner(), lib.inner(), lib.paths_of(&tracks));
}

#[tauri::command]
fn playlist_remove(lib: State<'_, Arc<Library>>, id: i64, tracks: Vec<i64>) {
    lib.playlist_remove(id, &tracks)
}

#[tauri::command]
fn playlist_move(lib: State<'_, Arc<Library>>, id: i64, track: i64, to: i64) {
    lib.playlist_move(id, track, to)
}

/// Copertina incorporata nel file (se c'e'), ridotta a `size` pixel e codificata come data URL JPEG.
#[tauri::command]
async fn track_art(path: String, size: u32) -> Option<String> {
    tauri::async_runtime::spawn_blocking(move || -> Option<String> {
        use base64::Engine as _;
        use lofty::file::TaggedFileExt;
        let f = lofty::read_from_path(&path).ok()?;
        let tag = f.primary_tag().or_else(|| f.first_tag())?;
        let pic = tag.pictures().first()?;
        let img = image::load_from_memory(pic.data()).ok()?;
        let size = size.clamp(16, 512);
        let thumb = img.thumbnail(size, size).to_rgb8();
        let mut out = std::io::Cursor::new(Vec::new());
        image::DynamicImage::ImageRgb8(thumb).write_to(&mut out, image::ImageFormat::Jpeg).ok()?;
        Some(format!("data:image/jpeg;base64,{}", base64::engine::general_purpose::STANDARD.encode(out.into_inner())))
    })
    .await
    .ok()
    .flatten()
}

// ---------------------------------------------------------------- stem (voce / strumentale)

/// Coda di separazione degli stem: un solo brano alla volta (usa tutti i core), i richiesti dal deck passano avanti.
#[derive(Default)]
struct StemQueue {
    pending: Mutex<std::collections::VecDeque<String>>,
    running: std::sync::atomic::AtomicBool,
}

fn queue_stems(app: &tauri::AppHandle, q: &Arc<StemQueue>, dir: &std::path::Path, paths: Vec<String>, front: bool) {
    use std::sync::atomic::Ordering::SeqCst;
    {
        let mut p = q.pending.lock();
        for path in paths {
            if stems::has_cached(dir, &path) {
                let _ = app.emit("stems-ready", (path, true));
            } else if let Some(i) = p.iter().position(|x| *x == path) {
                if front {
                    p.remove(i);
                    p.push_front(path);
                }
            } else if front {
                p.push_front(path);
            } else {
                p.push_back(path);
            }
        }
    }
    if q.pending.lock().is_empty() || q.running.swap(true, SeqCst) {
        return;
    }
    let (q, app, dir) = (q.clone(), app.clone(), dir.to_path_buf());
    std::thread::spawn(move || {
        loop {
            let Some(path) = q.pending.lock().pop_front() else { break };
            let _ = app.emit("stems-progress", (path.clone(), 0u32));
            let model = stems::model_path(&dir);
            let mut last = 0u32;
            let r = stems::ensure_model(&model).and_then(|_| stems::separate(&path, &model, &mut |p| {
                // chi sta caricando un brano sul deck ha la precedenza
                while LOADING.load(SeqCst) > 0 {
                    std::thread::sleep(std::time::Duration::from_millis(40));
                }
                let pct = (p * 100.0) as u32;
                if pct != last {
                    last = pct;
                    let _ = app.emit("stems-progress", (path.clone(), pct));
                }
                true
            }).and_then(|st| stems::save_for(&dir, &path, &st).map(|_| ())));
            if let Err(e) = &r {
                let _ = app.emit("stems-error", (path.clone(), e.clone()));
            }
            let _ = app.emit("stems-ready", (path, r.is_ok()));
        }
        q.running.store(false, SeqCst);
    });
}

/// Separa gli stem di alcuni brani (ids della libreria) in background.
#[tauri::command]
fn stems_queue(app: tauri::AppHandle, a: State<'_, App>, lib: State<'_, Arc<Library>>, q: State<'_, Arc<StemQueue>>, ids: Vec<i64>, front: bool) {
    let paths = lib.paths_of(&ids);
    queue_stems(&app, &q, &a.dir, paths, front);
}

/// Prepara gli stem di tutti i brani di una playlist.
#[tauri::command]
fn stems_queue_playlist(app: tauri::AppHandle, a: State<'_, App>, lib: State<'_, Arc<Library>>, q: State<'_, Arc<StemQueue>>, id: i64) {
    let paths = lib.playlist_paths(id);
    queue_stems(&app, &q, &a.dir, paths, false);
}

/// Separa il brano di un percorso (dal deck).
#[tauri::command]
fn stems_queue_path(app: tauri::AppHandle, a: State<'_, App>, q: State<'_, Arc<StemQueue>>, path: String) {
    queue_stems(&app, &q, &a.dir, vec![path], true);
}

/// Quali brani hanno gia' gli stem pronti.
#[tauri::command]
fn stems_have(a: State<'_, App>, paths: Vec<String>) -> Vec<bool> {
    paths.iter().map(|p| stems::has_cached(&a.dir, p)).collect()
}

/// Come lavorano gli stem su questo PC: 0 automatico (in base alla velocita' misurata), 1 sempre in tempo reale, 2 solo stem preparati.
static STEM_MODE: std::sync::atomic::AtomicU8 = std::sync::atomic::AtomicU8::new(0);
/// velocita' di separazione misurata (secondi di musica al secondo, bit di f32; 0 = mai misurata)
static STEM_SPEED: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
/// sotto questa velocita' il tempo reale non regge (due deck che separano insieme dimezzano la velocita' di ciascuno)
const REALTIME_MIN_SPEED: f32 = 2.0;

fn stem_speed() -> f32 {
    f32::from_bits(STEM_SPEED.load(std::sync::atomic::Ordering::SeqCst))
}

/// Gli stem in tempo reale sono consentiti? Se il PC e' lento si usano solo quelli preparati prima.
fn realtime_ok() -> bool {
    match STEM_MODE.load(std::sync::atomic::Ordering::SeqCst) {
        1 => true,
        2 => false,
        _ => stem_speed() <= 0.0 || stem_speed() >= REALTIME_MIN_SPEED,
    }
}

fn stem_cfg_path(dir: &std::path::Path) -> std::path::PathBuf {
    dir.join("stems.json")
}

fn stem_cfg_load(dir: &std::path::Path) {
    use std::sync::atomic::Ordering::SeqCst;
    if let Some(v) = std::fs::read_to_string(stem_cfg_path(dir)).ok().and_then(|t| serde_json::from_str::<serde_json::Value>(&t).ok()) {
        STEM_MODE.store(v["mode"].as_u64().unwrap_or(0).min(2) as u8, SeqCst);
        // la velocita' vale solo per il modello con cui e' stata misurata (es. arriva il modello per telefoni): se no si rimisura
        let same = v["model"].as_str().map_or(true, |m| m == stems::model_name(&stems::model_path(dir)));
        let speed = if same { v["speed"].as_f64().unwrap_or(0.0) as f32 } else { 0.0 };
        STEM_SPEED.store(speed.to_bits(), SeqCst);
    }
}

fn stem_cfg_save(dir: &std::path::Path) {
    use std::sync::atomic::Ordering::SeqCst;
    let v = serde_json::json!({ "mode": STEM_MODE.load(SeqCst), "speed": stem_speed(), "model": stems::model_name(&stems::model_path(dir)) });
    let _ = std::fs::write(stem_cfg_path(dir), v.to_string());
}

/// Misura quanto e' veloce la separazione su questo PC (stesso percorso dell'uso reale: CPU o GPU come per gli stem).
fn stem_bench(dir: &std::path::Path) -> Result<f32, String> {
    let model = stems::model_path(dir);
    stems::ensure_model(&model)?;
    let mut x = 12345u32;
    let planar: Vec<f32> = (0..2 * stems::SEG)
        .map(|i| {
            x = x.wrapping_mul(1664525).wrapping_add(1013904223);
            ((x >> 9) as f32 / 4_194_304.0 - 1.0) * 0.2 + (i as f32 * 0.03).sin() * 0.3
        })
        .collect();
    // la prima esecuzione apre il modello: si misura la seconda
    stems::infer_segment(&model, planar.clone(), stems::SEG)?;
    while LOADING.load(std::sync::atomic::Ordering::SeqCst) > 0 {
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    // il migliore di due: una sola misura e' rumorosa (su Android oscilla del 10-15% e la soglia e' vicina)
    let mut secs = f64::MAX;
    for _ in 0..2 {
        let t = std::time::Instant::now();
        stems::infer_segment(&model, planar.clone(), stems::SEG)?;
        secs = secs.min(t.elapsed().as_secs_f64().max(1e-3));
    }
    let speed = (stems::STRIDE as f64 / stems::SR as f64 / secs) as f32;
    eprintln!("stem: bench {speed:.2}x tempo reale ({secs:.2} s per segmento)");
    STEM_SPEED.store(speed.to_bits(), std::sync::atomic::Ordering::SeqCst);
    stem_cfg_save(dir);
    Ok(speed)
}

#[derive(Serialize)]
struct StemCfg {
    /// "auto", "realtime" oppure "prepared"
    mode: &'static str,
    /// velocita' misurata (0 = mai misurata)
    speed: f32,
    /// gli stem in tempo reale sono attivi adesso
    realtime: bool,
    min_speed: f32,
}

fn stem_cfg_now() -> StemCfg {
    StemCfg {
        mode: match STEM_MODE.load(std::sync::atomic::Ordering::SeqCst) { 1 => "realtime", 2 => "prepared", _ => "auto" },
        speed: stem_speed(),
        realtime: realtime_ok(),
        min_speed: REALTIME_MIN_SPEED,
    }
}

#[tauri::command]
fn stems_cfg() -> StemCfg {
    stem_cfg_now()
}

#[tauri::command]
fn stems_cfg_set(a: State<'_, App>, mode: String) -> StemCfg {
    STEM_MODE.store(match mode.as_str() { "realtime" => 1, "prepared" => 2, _ => 0 }, std::sync::atomic::Ordering::SeqCst);
    stem_cfg_save(&a.dir);
    stem_cfg_now()
}

/// Misura la velocita' degli stem su questo PC e applica la modalita' automatica.
#[tauri::command]
async fn stems_bench_cmd(a: State<'_, App>) -> Result<StemCfg, String> {
    let dir = a.dir.clone();
    tauri::async_runtime::spawn_blocking(move || stem_bench(&dir)).await.map_err(|e| e.to_string())??;
    Ok(stem_cfg_now())
}

/// Elimina gli stem preparati di alcuni brani (ids della libreria).
#[tauri::command]
fn stems_delete(app: tauri::AppHandle, a: State<'_, App>, lib: State<'_, Arc<Library>>, ids: Vec<i64>) {
    for path in lib.paths_of(&ids) {
        stems::delete_for(&a.dir, &path);
        let _ = app.emit("stems-ready", (path, false));
    }
}

/// separazione degli stem dei brani sui deck mentre si caricano (se spenta parte solo quando serve)
static AUTO_STEM: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(true);

fn stem_publish(mixer: Arc<Mutex<Mixer>>) -> impl Fn(usize, &str, Arc<Vec<f32>>, bool) -> bool + Send + Sync {
    move |deck, path, view, fade| {
        let mut m = mixer.lock();
        let d = &mut m.decks[deck];
        // la vista vale solo se il deck ha ancora lo stesso brano e gli stem sono attivi
        if d.track.as_ref().map(|t| t.path.as_str()) == Some(path) && d.stems_active() {
            d.set_view(Some(view), fade);
            true
        } else {
            false
        }
    }
}

/// Prepara la separazione progressiva del brano sul deck (decodifica a 44,1 kHz e registra la sessione).
fn stems_live(mixer: &Arc<Mutex<Mixer>>, hub: &Arc<stems::Hub>, deck: usize) -> Result<Arc<stems::Live>, String> {
    let (path, sr, base) = {
        let m = mixer.lock();
        let Some(t) = m.decks[deck].track.clone() else { return Err("nessun brano".into()) };
        (t.path.clone(), m.sr, t.data.clone())
    };
    if let Some(l) = hub.live[deck].lock().clone() {
        if l.path == path && l.alive.load(std::sync::atomic::Ordering::SeqCst) {
            return Ok(l);
        }
    }
    let mix44 = decode::decode_file(&path, stems::SR)?;
    let live = Arc::new(stems::Live::new(&path, deck, mix44, sr, base));
    if let Some(old) = hub.live[deck].lock().replace(live.clone()) {
        old.alive.store(false, std::sync::atomic::Ordering::SeqCst);
    }
    hub.wake();
    Ok(live)
}

/// Un brano appena caricato su un deck: scarta la sessione del brano precedente e, se serve, prepara la separazione in anticipo.
fn stems_attach(mixer: Arc<Mutex<Mixer>>, hub: Arc<stems::Hub>, dir: std::path::PathBuf, deck: usize, path: String) {
    if let Some(old) = hub.live[deck].lock().take() {
        old.alive.store(false, std::sync::atomic::Ordering::SeqCst);
    }
    *hub.cached[deck].lock() = None;
    if !AUTO_STEM.load(std::sync::atomic::Ordering::SeqCst) || !realtime_ok() || stems::has_cached(&dir, &path) {
        return;
    }
    std::thread::spawn(move || {
        // il brano potrebbe essere stato cambiato nel frattempo
        if mixer.lock().decks[deck].track.as_ref().map(|t| t.path.as_str()) == Some(path.as_str()) {
            let _ = stems_live(&mixer, &hub, deck);
        }
    });
}

/// Accende o spegne la separazione automatica dei brani caricati.
#[tauri::command]
fn stems_auto(app: State<'_, App>, hub: State<'_, Arc<stems::Hub>>, on: bool) {
    AUTO_STEM.store(on, std::sync::atomic::Ordering::SeqCst);
    if on && realtime_ok() {
        for d in 0..NUM_DECKS {
            let path = app.mixer.lock().decks[d].track.as_ref().map(|t| t.path.clone());
            if let Some(path) = path {
                if hub.live[d].lock().is_none() && !stems::has_cached(&app.dir, &path) {
                    let (mx, hb) = (app.mixer.clone(), hub.inner().clone());
                    std::thread::spawn(move || { let _ = stems_live(&mx, &hb, d); });
                }
            }
        }
    }
}

/// Sceglie gli stem ascoltati su un deck: un volume per stem (batteria, basso, altro, voce); tutti a 1 = brano intero.
/// Il cambio e' immediato: gli stem gia' separati entrano subito, il resto si separa partendo dalla testina.
#[tauri::command]
async fn deck_stems(app: State<'_, App>, hub: State<'_, Arc<stems::Hub>>, deck: usize, gains: Vec<f32>) -> Result<(), String> {
    if deck >= NUM_DECKS || gains.len() != 4 {
        return Err("deck o volumi non validi".into());
    }
    let gains = [0, 1, 2, 3].map(|s| gains[s].clamp(0.0, 1.0));
    let (path, sr, frames) = {
        let m = app.mixer.lock();
        let Some(t) = m.decks[deck].track.clone() else { return Err("nessun brano".into()) };
        (t.path.clone(), m.sr, t.frames())
    };
    let publish = stem_publish(app.mixer.clone());
    if gains.iter().all(|g| *g >= 0.999) {
        app.mixer.lock().decks[deck].set_stems([1.0; 4], None);
        if let Some(l) = hub.live[deck].lock().clone() {
            l.set_gains([1.0; 4], &publish);
        }
        return Ok(());
    }
    // i pulsanti si accendono subito; l'audio cambia appena la vista e' pronta
    {
        let mut m = app.mixer.lock();
        let d = &mut m.decks[deck];
        d.stem_gains = gains;
        d.stem_mask = (0..4).fold(0u8, |mk, s| if gains[s] > 0.001 { mk | (1 << s) } else { mk });
    }
    let (mixer, hubc, dir) = (app.mixer.clone(), hub.inner().clone(), app.dir.clone());
    tauri::async_runtime::spawn_blocking(move || -> Result<(), String> {
        // 1) stem gia' interi in cache
        let cached = {
            let c = hubc.cached[deck].lock().clone();
            match c {
                Some((p, st)) if p == path => Some(st),
                _ => None,
            }
        };
        let cached = match cached {
            Some(st) => Some(st),
            None => {
                if let Some(file) = stems::find_cached(&dir, &path) {
                    let st = Arc::new(stems::load(&file)?);
                    *hubc.cached[deck].lock() = Some((path.clone(), st.clone()));
                    Some(st)
                } else {
                    None
                }
            }
        };
        if let Some(st) = cached {
            let view = stems::mix(&st, gains, sr, frames);
            let mut m = mixer.lock();
            if m.decks[deck].track.as_ref().map(|t| t.path.as_str()) == Some(path.as_str()) && m.decks[deck].stem_gains == gains {
                m.decks[deck].set_stems(gains, Some(Arc::new(view)));
            }
            return Ok(());
        }
        // 2) separazione progressiva (solo se questo PC regge il tempo reale)
        if !realtime_ok() {
            return Err("Stem non ancora preparati: su questo PC vanno preparati prima (clic destro sul brano → Crea stem)".into());
        }
        let live = stems_live(&mixer, &hubc, deck)?;
        live.set_gains(gains, &publish);
        hubc.wake();
        Ok(())
    })
    .await
    .map_err(|e| e.to_string())?
}

// ---------------------------------------------------------------- analisi in background

/// Coda di analisi: i brani messi in playlist vengono analizzati (bpm, chiave, beatgrid, forma d'onda) e salvati nel
/// database, cosi' il caricamento sul deck non deve rifare l'analisi.
#[derive(Default)]
struct AnalysisQueue {
    pending: Mutex<std::collections::VecDeque<String>>,
    workers: std::sync::atomic::AtomicUsize,
    total: std::sync::atomic::AtomicUsize,
    done: std::sync::atomic::AtomicUsize,
}

fn queue_analysis(app: &tauri::AppHandle, q: &Arc<AnalysisQueue>, lib: &Arc<Library>, paths: Vec<String>) {
    use std::sync::atomic::Ordering::SeqCst;
    {
        let mut p = q.pending.lock();
        for path in paths {
            if lib.analysis_get(&path).is_none() && !p.contains(&path) {
                p.push_back(path);
                q.total.fetch_add(1, SeqCst);
            }
        }
    }
    let want = std::thread::available_parallelism().map_or(2, |n| n.get().saturating_sub(1).clamp(1, 3));
    while q.workers.load(SeqCst) < want && !q.pending.lock().is_empty() {
        q.workers.fetch_add(1, SeqCst);
        let (q, lib, app) = (q.clone(), lib.clone(), app.clone());
        std::thread::spawn(move || {
            loop {
                let Some(path) = q.pending.lock().pop_front() else { break };
                if lib.analysis_get(&path).is_none() {
                    emit_progress(&app, &path, 0);
                    match decode::decode_file(&path, 44100) {
                        Ok(data) => {
                            run_analysis(&app, &lib, &path, &data, 44100, true);
                        }
                        Err(_) => emit_progress(&app, &path, 100),
                    }
                }
                let done = q.done.fetch_add(1, SeqCst) + 1;
                let _ = app.emit("analysis-progress", (done, q.total.load(SeqCst)));
            }
            if q.workers.fetch_sub(1, SeqCst) == 1 {
                // ultimo worker: azzera i contatori per il prossimo giro
                q.total.store(0, SeqCst);
                q.done.store(0, SeqCst);
                let _ = app.emit("analysis-progress", (0usize, 0usize));
            }
        });
    }
}

/// Analizza brani (ids della libreria) in background.
#[tauri::command]
fn analyze_tracks(app: tauri::AppHandle, lib: State<'_, Arc<Library>>, q: State<'_, Arc<AnalysisQueue>>, ids: Vec<i64>) {
    let paths = lib.paths_of(&ids);
    queue_analysis(&app, q.inner(), lib.inner(), paths);
}

/// Analizza tutti i brani di una playlist.
#[tauri::command]
fn analyze_playlist(app: tauri::AppHandle, lib: State<'_, Arc<Library>>, q: State<'_, Arc<AnalysisQueue>>, id: i64) {
    let paths = lib.playlist_paths(id);
    queue_analysis(&app, q.inner(), lib.inner(), paths);
}

// ---------------------------------------------------------------- esplora risorse

#[derive(Serialize)]
struct FsNode {
    name: String,
    path: String,
}

/// Unita' del computer e cartelle utente piu' comuni.
#[tauri::command]
fn fs_roots() -> Vec<FsNode> {
    let mut out = vec![];
    if let Some(home) = std::env::var_os("USERPROFILE").or_else(|| std::env::var_os("HOME")) {
        let home = std::path::PathBuf::from(home);
        for (label, sub) in [("Musica", "Music"), ("Desktop", "Desktop"), ("Download", "Downloads"), ("Documenti", "Documents")] {
            let p = home.join(sub);
            if p.is_dir() {
                out.push(FsNode { name: label.into(), path: p.to_string_lossy().into_owned() });
            }
        }
    }
    // su Android HOME non esiste: le cartelle dell'utente stanno nell'archivio condiviso
    #[cfg(target_os = "android")]
    for (label, sub) in [("Musica", "Music"), ("Download", "Download"), ("Documenti", "Documents"), ("Archivio interno", "")] {
        let p = std::path::Path::new("/storage/emulated/0").join(sub);
        if p.is_dir() {
            out.push(FsNode { name: label.into(), path: p.to_string_lossy().trim_end_matches('/').to_string() });
        }
    }
    #[cfg(windows)]
    for c in b'A'..=b'Z' {
        let root = format!("{}:\\", c as char);
        if std::path::Path::new(&root).exists() {
            out.push(FsNode { name: format!("Unità ({}:)", c as char), path: root });
        }
    }
    #[cfg(not(windows))]
    out.push(FsNode { name: "/".into(), path: "/".into() });
    out
}

fn hidden(e: &std::fs::DirEntry) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if let Ok(m) = e.metadata() {
            return m.file_attributes() & 0x6 != 0; // nascosto o di sistema
        }
    }
    e.file_name().to_string_lossy().starts_with('.')
}

/// Sottocartelle di una cartella.
#[tauri::command]
fn fs_list(path: String) -> Vec<FsNode> {
    let Ok(rd) = std::fs::read_dir(&path) else { return vec![] };
    let mut v: Vec<FsNode> = rd
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir() && !hidden(e))
        .map(|e| FsNode { name: e.file_name().to_string_lossy().into_owned(), path: e.path().to_string_lossy().into_owned() })
        .collect();
    v.sort_by_key(|n| n.name.to_lowercase());
    v
}

fn audio_files(folder: &str, recursive: bool) -> Vec<String> {
    let mut v: Vec<String> = if recursive {
        walkdir::WalkDir::new(folder)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file() && library::is_audio(e.path()))
            .map(|e| e.path().to_string_lossy().into_owned())
            .collect()
    } else {
        std::fs::read_dir(folder)
            .map(|rd| {
                rd.filter_map(|e| e.ok())
                    .filter(|e| e.path().is_file() && !hidden(e) && library::is_audio(&e.path()))
                    .map(|e| e.path().to_string_lossy().into_owned())
                    .collect()
            })
            .unwrap_or_default()
    };
    v.sort_by_key(|p| p.to_lowercase());
    v
}

/// Brani audio contenuti in una cartella (senza sottocartelle).
#[tauri::command]
async fn fs_tracks(lib: State<'_, Arc<Library>>, path: String) -> Result<Vec<LibTrack>, String> {
    let l = lib.inner().clone();
    tauri::async_runtime::spawn_blocking(move || l.tracks_for_paths(&audio_files(&path, false))).await.map_err(|e| e.to_string())
}

/// Aggiunge a una playlist tutti i brani di una cartella, sottocartelle comprese, e li fa analizzare.
#[tauri::command]
async fn playlist_add_folder(
    app: tauri::AppHandle,
    lib: State<'_, Arc<Library>>,
    q: State<'_, Arc<AnalysisQueue>>,
    id: i64,
    folder: String,
) -> Result<usize, String> {
    let l = lib.inner().clone();
    let tracks = tauri::async_runtime::spawn_blocking(move || {
        let t = l.tracks_for_paths(&audio_files(&folder, true));
        let ids: Vec<i64> = t.iter().map(|t| t.id).collect();
        l.playlist_add(id, &ids);
        t
    })
    .await
    .map_err(|e| e.to_string())?;
    queue_analysis(&app, q.inner(), lib.inner(), tracks.iter().map(|t| t.path.clone()).collect());
    Ok(tracks.len())
}

// ---------------------------------------------------------------- anteprima (cuffia)

/// Ascolta un brano sull'uscita di anteprima. `start` e' la posizione di partenza (0..1). Se il brano e' gia'
/// analizzato la sua loudness viene portata a -14 LUFS, cosi' le anteprime hanno tutte lo stesso volume.
#[tauri::command]
async fn preview_play(lib: State<'_, Arc<Library>>, pv: State<'_, Arc<Previewer>>, path: String, start: f64) -> Result<(), String> {
    let sr = pv.sample_rate();
    if sr == 0 {
        return Err("Nessuna uscita per l'anteprima: scegli un dispositivo in Impostazioni audio".into());
    }
    let (l, p, pvc) = (lib.inner().clone(), path.clone(), pv.inner().clone());
    tauri::async_runtime::spawn_blocking(move || -> Result<(), String> {
        let data = decode::decode_file(&p, sr)?;
        let gain = l
            .analysis_get(&p)
            .filter(|a| a.loudness > -69.0)
            .map_or(0.8, |a| {
                let g = 10f64.powf((engine::REFERENCE_LUFS - a.loudness).clamp(-12.0, 9.0) / 20.0) as f32;
                (g * 0.8).min(0.8 / a.peak.max(0.01))
            });
        pvc.play(&p, data, start, gain);
        Ok(())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
fn preview_stop(pv: State<'_, Arc<Previewer>>) {
    pv.stop()
}

#[tauri::command]
fn preview_seek(pv: State<'_, Arc<Previewer>>, frac: f64) {
    pv.seek(frac)
}

#[tauri::command]
fn preview_state(pv: State<'_, Arc<Previewer>>) -> PreviewState {
    pv.state()
}

// ---------------------------------------------------------------- impostazioni audio

fn settings_path(app: &App) -> std::path::PathBuf {
    app.dir.join("audio.json")
}

/// file passati da riga di comando (o "Apri con"): vengono caricati sui deck all'avvio
#[tauri::command]
fn startup_files() -> Vec<String> {
    std::env::args().skip(1).filter(|a| std::path::Path::new(a).is_file()).collect()
}

#[derive(Serialize)]
struct AudioListFull {
    #[serde(flatten)]
    list: audio::AudioList,
    /// dispositivo scelto per l'anteprima (None = predefinito) e se l'uscita e' attiva
    preview_device: Option<String>,
    preview_ok: bool,
    /// dispositivi Windows (WASAPI) per le cuffie: l'anteprima usa sempre quelli, anche se l'uscita principale e' ASIO
    preview_devices: Vec<String>,
}

/// stato dell'uscita attiva, senza interrogare i driver (con ASIO farlo ferma l'audio)
#[tauri::command]
fn audio_current(app: State<'_, App>) -> audio::AudioInfo {
    app.audio.current()
}

#[tauri::command]
fn audio_info(app: State<'_, App>, pv: State<'_, Arc<Previewer>>, driver: Option<String>) -> AudioListFull {
    let preview_devices = audio::device_names(&cpal::default_host());
    AudioListFull { list: app.audio.list(driver), preview_device: pv.device.lock().clone(), preview_ok: pv.sample_rate() > 0, preview_devices }
}

#[tauri::command]
async fn audio_set(app: State<'_, App>, pv: State<'_, Arc<Previewer>>, settings: audio::Settings) -> Result<audio::AudioInfo, String> {
    if app.mixer.lock().rec.is_some() {
        return Err("Ferma la registrazione prima di cambiare le impostazioni audio".into());
    }
    // la riconfigurazione puo' ricampionare i brani caricati: fuori dal thread UI
    let info = app.audio.reconfigure(settings.clone())?;
    // uscita di anteprima: se non si apre resta spenta, senza compromettere l'uscita principale
    let preview_err = pv.reopen(settings.preview_device.clone().filter(|d| !d.is_empty())).err();
    let mut settings = settings;
    if settings.driver.as_deref().map_or(false, |d| !d.eq_ignore_ascii_case(&info.driver)) {
        settings.driver = None;
        settings.device = None;
        settings.sample_rate = None;
        settings.buffer = None;
    }
    let _ = std::fs::write(settings_path(&app), serde_json::to_string_pretty(&settings).unwrap_or_default());
    let mut info = info;
    if let Some(e) = preview_err {
        info.error = Some(format!("uscita anteprima: {e}"));
    }
    Ok(info)
}

// ---------------------------------------------------------------- registrazione (licenza)

#[tauri::command]
fn license_status(app: State<'_, App>) -> license::Status {
    license::status(&app.dir)
}
#[tauri::command]
async fn license_activate(app: State<'_, App>, key: String) -> Result<license::Status, String> {
    let dir = app.dir.clone();
    tauri::async_runtime::spawn_blocking(move || license::activate(&dir, &key)).await.map_err(|e| e.to_string())?
}
#[tauri::command]
async fn license_refresh(app: State<'_, App>) -> Result<license::Status, String> {
    let dir = app.dir.clone();
    tauri::async_runtime::spawn_blocking(move || license::refresh(&dir)).await.map_err(|e| e.to_string())?
}
#[tauri::command]
async fn license_deactivate(app: State<'_, App>) -> Result<license::Status, String> {
    let dir = app.dir.clone();
    tauri::async_runtime::spawn_blocking(move || license::deactivate(&dir)).await.map_err(|e| e.to_string())?
}


#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default().plugin(tauri_plugin_dialog::init());
    // gli aggiornamenti automatici esistono solo su desktop
    #[cfg(desktop)]
    let builder = builder.plugin(tauri_plugin_updater::Builder::new().build());
    builder
        .setup(|app| {
            // Gli stati leggeri si registrano per primi: l'interfaccia puo' chiamare i comandi mentre il resto di
            // `setup` (audio, libreria) e' ancora in corso, soprattutto su Android dove la libreria e' lenta da aprire.
            let q = Arc::new(AnalysisQueue::default());
            app.manage(q.clone());
            app.manage(Arc::new(StemQueue::default()));
            app.manage(RecState::default());
            // separazione degli stem in tempo reale per i deck (il thread parte piu' sotto, a mixer pronto)
            let hub = stems::Hub::new();
            app.manage(hub.clone());
            let dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&dir)?;
            stem_cfg_load(&dir);
            let saved: audio::Settings = std::fs::read_to_string(dir.join("audio.json"))
                .ok()
                .and_then(|t| serde_json::from_str(&t).ok())
                .unwrap_or_default();
            // se le impostazioni salvate non funzionano piu' (dispositivo scollegato) usa quelle predefinite
            let aud = audio::Audio::start(&saved).or_else(|_| audio::Audio::start(&audio::Settings::default()))?;
            let mixer = aud.mixer.clone();
            app.manage(App { mixer, audio: aud, dir: dir.clone() });
            // batteria: ricarica il kit salvato e apre l'ingresso MIDI
            {
                let st = app.state::<App>();
                let (mx, d2) = (st.mixer.clone(), st.dir.clone());
                midi::start(mx.clone(), Some(app.handle().clone()));
                std::thread::spawn(move || load_kit(&mx, &d2));
            }
            let bus = app.state::<App>().mixer.lock().cue_bus.clone();
            app.manage(Arc::new(Previewer::start(saved.preview_device.clone().filter(|d| !d.is_empty()), bus, app.state::<App>().audio.inline.clone())));
            let lib = Library::open(&dir.join("library.db"))?;
            lib.prune_orphans();
            app.manage(Arc::new(lib));
            {
                let mx = app.state::<App>().mixer.clone();
                let (mx2, mx3, h) = (mx.clone(), mx.clone(), app.handle().clone());
                let (h2, h3) = (h.clone(), h.clone());
                hub.spawn(dir.clone(), stems::Callbacks {
                    publish: Box::new(stem_publish(mx)),
                    deck_pos: Box::new(move |d| {
                        let m = mx2.lock();
                        let dk = &m.decks[d];
                        dk.track.as_ref().map(|_| (dk.pos_sec(), dk.playing, dk.stems_active()))
                    }),
                    progress: Box::new(move |_, path, pct, gpu| { let _ = h2.emit("stems-progress", (path.to_string(), pct, gpu)); }),
                    busy: Box::new(|| LOADING.load(std::sync::atomic::Ordering::SeqCst) > 0),
                    finished: Box::new(move |path, ok| { let _ = h3.emit("stems-ready", (path.to_string(), ok)); }),
                });
                let _ = &mx3;
                // il modello si carica subito in background: al primo uso degli stem e' gia' pronto
                let model = stems::model_path(&dir);
                let d4 = dir.clone();
                std::thread::spawn(move || {
                    stems::background_priority();
                    stems::warm(&model);
                    // prima misura della velocita' (poi si rifa solo su richiesta dalle impostazioni)
                    if stem_speed() <= 0.0 && model.exists() {
                        if let Err(e) = stem_bench(&d4) {
                            eprintln!("stem: bench non riuscito: {e}");
                        }
                    }
                });
            }
            // i brani gia' in playlist ma non ancora analizzati vengono analizzati subito, in background
            let (h, l) = (app.handle().clone(), app.state::<Arc<Library>>().inner().clone());
            let paths = l.all_playlist_paths();
            queue_analysis(&h, &q, &l, paths);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            load_track, deck_cmd, fx_set, mixer_cmd, pad_load, pad_cmd, get_state, lib_scan, lib_add_files,
            lib_search, lib_update_tags, lib_remove, playlists, playlist_create, playlist_rename, playlist_delete,
            playlist_add, playlist_remove, playlist_move, audio_info, audio_set, startup_files, grid_cmd,
            fs_roots, fs_list, fs_tracks, playlist_add_folder, analyze_tracks, analyze_playlist,
            preview_play, preview_stop, preview_seek, preview_state,
            stems_queue, stems_queue_playlist, stems_queue_path, stems_have, stems_cfg, stems_cfg_set, stems_bench_cmd, stems_delete, deck_stems, stems_auto, track_art,
            drum_load, drum_cmd, drum_save, midi_ports, midi_out_info, midi_out_select, midi_send, midi_map_get, midi_map_set, audio_current, rec_start, rec_stop,
            license_status, license_activate, license_refresh, license_deactivate
        ])
        .run(tauri::generate_context!())
        .expect("errore avvio SpinLab");
}
