//! Libreria: database SQLite, tag, playlist e ricerca.

use lofty::file::{AudioFile, TaggedFileExt};
use lofty::tag::Accessor;
use parking_lot::Mutex;
use rusqlite::{params, params_from_iter, Connection};
use serde::{Deserialize, Serialize};
use std::path::Path;

const EXTS: &[&str] = &[
    "mp3", "wav", "flac", "ogg", "oga", "opus", "m4a", "mp4", "aac", "aif", "aiff", "wma", "alac", "ape", "wv", "mka", "webm", "caf", "mpc",
];

/// analisi di un file salvata nel database
pub struct CachedAnalysis {
    pub bpm: f64,
    pub key: Option<crate::camelot::Key>,
    pub downbeat: usize,
    pub beats: Vec<f64>,
    pub wave: crate::analysis::Wave,
    pub loudness: f64,
    pub peak: f32,
}

pub struct Library {
    db: Mutex<Connection>,
}

#[derive(Serialize, Clone)]
pub struct LibTrack {
    pub id: i64,
    pub path: String,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub genre: String,
    pub year: i64,
    pub bpm: f64,
    pub key: String,
    pub duration: f64,
    pub rating: i64,
    pub tags: String,
    pub comment: String,
    /// analisi (bpm, chiave, beatgrid, forma d'onda) gia' in cache: il caricamento sul deck e' immediato
    pub analyzed: bool,
}

#[derive(Deserialize)]
pub struct TagUpdate {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub genre: Option<String>,
    pub rating: Option<i64>,
    pub tags: Option<String>,
    pub comment: Option<String>,
}

#[derive(Serialize)]
pub struct Playlist {
    pub id: i64,
    pub name: String,
    pub count: i64,
}

const COLS: &str = "t.id,t.path,t.title,t.artist,t.album,t.genre,t.year,t.bpm,t.key,t.duration,t.rating,t.tags,t.comment,\n    EXISTS(SELECT 1 FROM analysis a WHERE a.path=t.path AND a.version=8)";

/// cambia quando cambia l'algoritmo di analisi: le analisi vecchie vengono rifatte (tenere allineato con COLS)
pub const ANALYSIS_VERSION: i64 = 8;

fn row(r: &rusqlite::Row) -> rusqlite::Result<LibTrack> {
    Ok(LibTrack {
        id: r.get(0)?, path: r.get(1)?, title: r.get(2)?, artist: r.get(3)?, album: r.get(4)?, genre: r.get(5)?,
        year: r.get(6)?, bpm: r.get(7)?, key: r.get(8)?, duration: r.get(9)?, rating: r.get(10)?,
        tags: r.get(11)?, comment: r.get(12)?, analyzed: r.get::<_, i64>(13)? != 0,
    })
}

/// (dimensione, data di modifica) del file: serve a capire se un'analisi in cache e' ancora valida
fn file_stamp(path: &str) -> Option<(i64, i64)> {
    let m = std::fs::metadata(path).ok()?;
    let t = m.modified().ok()?.duration_since(std::time::UNIX_EPOCH).ok()?.as_secs() as i64;
    Some((m.len() as i64, t))
}

pub fn is_audio(path: &Path) -> bool {
    let ext = path.extension().and_then(|x| x.to_str()).unwrap_or("").to_lowercase();
    EXTS.contains(&ext.as_str())
}

impl Library {
    pub fn open(path: &Path) -> Result<Self, String> {
        let db = Connection::open(path).map_err(|e| e.to_string())?;
        db.execute_batch(
            "PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;
             CREATE TABLE IF NOT EXISTS tracks(
               id INTEGER PRIMARY KEY, path TEXT UNIQUE NOT NULL, title TEXT, artist TEXT, album TEXT, genre TEXT,
               year INTEGER DEFAULT 0, bpm REAL DEFAULT 0, key TEXT DEFAULT '', duration REAL DEFAULT 0,
               rating INTEGER DEFAULT 0, tags TEXT DEFAULT '', comment TEXT DEFAULT '', added INTEGER DEFAULT (strftime('%s','now')));
             CREATE TABLE IF NOT EXISTS playlists(id INTEGER PRIMARY KEY, name TEXT NOT NULL);
             CREATE TABLE IF NOT EXISTS playlist_tracks(
               playlist_id INTEGER REFERENCES playlists(id) ON DELETE CASCADE,
               track_id INTEGER REFERENCES tracks(id) ON DELETE CASCADE,
               position INTEGER NOT NULL);
             CREATE TABLE IF NOT EXISTS grids(path TEXT PRIMARY KEY, beats TEXT NOT NULL, downbeat INTEGER, bpm REAL, fixed INTEGER);
             CREATE TABLE IF NOT EXISTS analysis(
               path TEXT PRIMARY KEY, size INTEGER, mtime INTEGER, version INTEGER, bpm REAL, key_pc INTEGER, key_minor INTEGER,
               downbeat INTEGER, beats TEXT, wave_rate INTEGER, lo BLOB, mid BLOB, hi BLOB);
             CREATE INDEX IF NOT EXISTS pt_idx ON playlist_tracks(playlist_id, position);",
        )
        .map_err(|e| e.to_string())?;
        // ancoraggi manuali (indici e tempi dei marker verificati) e griglia automatica originale: fanno da riferimento
        // per tarare l'algoritmo; le colonne mancano nei database creati con versioni precedenti
        for col in ["pins TEXT", "anchors TEXT", "auto_beats TEXT"] {
            let _ = db.execute(&format!("ALTER TABLE grids ADD COLUMN {col}"), []);
        }
        for col in ["loudness REAL", "peak REAL"] {
            let _ = db.execute(&format!("ALTER TABLE analysis ADD COLUMN {col}"), []);
        }
        Ok(Library { db: Mutex::new(db) })
    }

    /// Analisi in cache per un file, valida solo se file e versione non sono cambiati.
    pub fn analysis_get(&self, path: &str) -> Option<CachedAnalysis> {
        let (size, mtime) = file_stamp(path)?;
        let db = self.db.lock();
        db.query_row(
            "SELECT bpm,key_pc,key_minor,downbeat,beats,wave_rate,lo,mid,hi,loudness,peak FROM analysis WHERE path=? AND size=? AND mtime=? AND version=?",
            params![path, size, mtime, ANALYSIS_VERSION],
            |r| {
                let pc: Option<i64> = r.get(1)?;
                let beats: String = r.get(4)?;
                Ok(CachedAnalysis {
                    bpm: r.get(0)?,
                    key: pc.map(|pc| crate::camelot::Key { pc: pc as u8, minor: r.get::<_, i64>(2).unwrap_or(0) != 0 }),
                    downbeat: r.get::<_, i64>(3)? as usize,
                    beats: serde_json::from_str(&beats).unwrap_or_default(),
                    wave: crate::analysis::Wave { rate: r.get::<_, i64>(5)? as usize, lo: r.get(6)?, mid: r.get(7)?, hi: r.get(8)? },
                    loudness: r.get::<_, Option<f64>>(9)?.unwrap_or(-70.0),
                    peak: r.get::<_, Option<f32>>(10)?.unwrap_or(1.0),
                })
            },
        )
        .ok()
    }
    /// Dimentica l'analisi salvata: la prossima richiesta di analisi rifa' il brano da capo.
    pub fn analysis_forget(&self, path: &str) {
        let _ = self.db.lock().execute("DELETE FROM analysis WHERE path=?", params![path]);
    }

    pub fn analysis_put(&self, path: &str, a: &crate::analysis::Analysis) {
        let Some((size, mtime)) = file_stamp(path) else { return };
        let beats = serde_json::to_string(&a.beats).unwrap_or_default();
        let db = self.db.lock();
        let _ = db.execute(
            "INSERT OR REPLACE INTO analysis(path,size,mtime,version,bpm,key_pc,key_minor,downbeat,beats,wave_rate,lo,mid,hi,loudness,peak)
             VALUES(?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)",
            params![
                path, size, mtime, ANALYSIS_VERSION, a.bpm, a.key.map(|k| k.pc as i64), a.key.map_or(0, |k| k.minor as i64),
                a.downbeat as i64, beats, a.wave.rate as i64, a.wave.lo, a.wave.mid, a.wave.hi, a.loudness, a.peak
            ],
        );
        let key = a.key.map(crate::camelot::camelot_code).unwrap_or_default();
        let _ = db.execute("UPDATE tracks SET bpm=?, key=? WHERE path=?", params![a.bpm, key, path]);
    }

    /// Tracce note per percorso, nello stesso ordine dei percorsi (quelle sconosciute vengono aggiunte).
    pub fn tracks_for_paths(&self, paths: &[String]) -> Vec<LibTrack> {
        let mut out = Vec::with_capacity(paths.len());
        for p in paths {
            let known = self.db.lock().query_row("SELECT 1 FROM tracks WHERE path=?", [p], |_| Ok(())).is_ok();
            if !known && self.add_file(p).is_err() {
                continue;
            }
            let db = self.db.lock();
            if let Ok(t) = db.query_row(&format!("SELECT {COLS} FROM tracks t WHERE t.path=?"), [p], row) {
                out.push(t);
            }
        }
        out
    }
    pub fn paths_of(&self, ids: &[i64]) -> Vec<String> {
        let db = self.db.lock();
        ids.iter().filter_map(|id| db.query_row("SELECT path FROM tracks WHERE id=?", [id], |r| r.get(0)).ok()).collect()
    }
    pub fn playlist_paths(&self, pid: i64) -> Vec<String> {
        let db = self.db.lock();
        let Ok(mut st) = db.prepare("SELECT t.path FROM tracks t JOIN playlist_tracks pt ON pt.track_id=t.id WHERE pt.playlist_id=? ORDER BY pt.position") else {
            return vec![];
        };
        st.query_map([pid], |r| r.get(0)).map(|r| r.filter_map(|x| x.ok()).collect()).unwrap_or_default()
    }
    /// Percorsi di tutti i brani che stanno in almeno una playlist.
    pub fn all_playlist_paths(&self) -> Vec<String> {
        let db = self.db.lock();
        let Ok(mut st) = db.prepare("SELECT DISTINCT t.path FROM tracks t JOIN playlist_tracks pt ON pt.track_id=t.id") else { return vec![] };
        st.query_map([], |r| r.get(0)).map(|r| r.filter_map(|x| x.ok()).collect()).unwrap_or_default()
    }
    /// Toglie dal database i brani che non stanno in nessuna playlist (i file restano sul disco; le analisi restano in cache).
    pub fn prune_orphans(&self) {
        let _ = self.db.lock().execute("DELETE FROM tracks WHERE id NOT IN (SELECT track_id FROM playlist_tracks)", []);
    }

    /// Scansiona ricorsivamente una cartella; ritorna il numero di brani aggiunti.
    pub fn scan_folder(&self, folder: &str, mut progress: impl FnMut(usize)) -> usize {
        let mut added = 0;
        let mut seen = 0;
        let walk = walkdir::WalkDir::new(folder).into_iter().filter_entry(|e| !(e.depth() > 0 && e.file_type().is_dir() && e.file_name().eq_ignore_ascii_case("stems")));
        for e in walk.filter_map(|e| e.ok()) {
            let p = e.path();
            let ext = p.extension().and_then(|x| x.to_str()).unwrap_or("").to_lowercase();
            if !e.file_type().is_file() || !EXTS.contains(&ext.as_str()) {
                continue;
            }
            seen += 1;
            if self.add_file(&p.to_string_lossy()).is_ok() {
                added += 1;
            }
            if seen % 20 == 0 {
                progress(seen);
            }
        }
        progress(seen);
        added
    }

    pub fn add_file(&self, path: &str) -> Result<i64, String> {
        let (mut title, mut artist, mut album, mut genre, mut year, mut dur) =
            (String::new(), String::new(), String::new(), String::new(), 0i64, 0f64);
        let mut bpm = 0f64;
        if let Ok(f) = lofty::read_from_path(path) {
            dur = f.properties().duration().as_secs_f64();
            if let Some(t) = f.primary_tag().or_else(|| f.first_tag()) {
                title = t.title().map(|s| s.to_string()).unwrap_or_default();
                artist = t.artist().map(|s| s.to_string()).unwrap_or_default();
                album = t.album().map(|s| s.to_string()).unwrap_or_default();
                genre = t.genre().map(|s| s.to_string()).unwrap_or_default();
                year = t.year().unwrap_or(0) as i64;
                if let Some(b) = t.get_string(&lofty::tag::ItemKey::IntegerBpm) {
                    bpm = b.parse().unwrap_or(0.0);
                }
            }
        }
        if title.is_empty() {
            title = Path::new(path).file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
        }
        let db = self.db.lock();
        db.execute(
            "INSERT INTO tracks(path,title,artist,album,genre,year,bpm,duration) VALUES(?,?,?,?,?,?,?,?)
             ON CONFLICT(path) DO UPDATE SET duration=excluded.duration",
            params![path, title, artist, album, genre, year, bpm, dur],
        )
        .map_err(|e| e.to_string())?;
        // il brano puo' essere stato tolto dalla libreria all'avvio (prune_orphans) mentre l'analisi e' rimasta in cache:
        // ripristina bpm e tonalita' calcolati, altrimenti sembrerebbe da rianalizzare
        if let Ok((abpm, pc, minor)) = db.query_row(
            "SELECT bpm,key_pc,key_minor FROM analysis WHERE path=? AND version=?",
            params![path, ANALYSIS_VERSION],
            |r| Ok((r.get::<_, f64>(0)?, r.get::<_, Option<i64>>(1)?, r.get::<_, Option<i64>>(2)?.unwrap_or(0))),
        ) {
            let key = pc.map(|pc| crate::camelot::camelot_code(crate::camelot::Key { pc: pc as u8, minor: minor != 0 })).unwrap_or_default();
            let _ = db.execute("UPDATE tracks SET bpm=?, key=? WHERE path=?", params![abpm, key, path]);
        }
        db.query_row("SELECT id FROM tracks WHERE path=?", [path], |r| r.get(0)).map_err(|e| e.to_string())
    }

    /// Ricerca testuale: ogni parola deve comparire in titolo/artista/album/genere/tag/commento/chiave/bpm.
    /// Con `playlist` cerca solo dentro quella playlist.
    pub fn search(&self, query: &str, playlist: Option<i64>, sort: &str, desc: bool) -> Vec<LibTrack> {
        let mut sql = format!("SELECT {COLS} FROM tracks t ");
        let mut args: Vec<String> = vec![];
        let mut wh: Vec<String> = vec![];
        if let Some(p) = playlist {
            sql += "JOIN playlist_tracks pt ON pt.track_id=t.id ";
            wh.push(format!("pt.playlist_id={p}"));
        }
        for w in query.split_whitespace() {
            wh.push(
                "(t.title||' '||t.artist||' '||t.album||' '||t.genre||' '||t.tags||' '||t.comment||' '||t.key||' '||CAST(CAST(t.bpm AS INTEGER) AS TEXT)) LIKE ?"
                    .into(),
            );
            args.push(format!("%{w}%"));
        }
        if !wh.is_empty() {
            sql += &format!("WHERE {} ", wh.join(" AND "));
        }
        let col = match sort {
            "artist" => "t.artist",
            "bpm" => "t.bpm",
            "key" => "t.key",
            "duration" => "t.duration",
            "album" => "t.album",
            "genre" => "t.genre",
            "rating" => "t.rating",
            "added" => "t.added",
            "title" => "t.title",
            _ => {
                if playlist.is_some() {
                    "pt.position"
                } else {
                    "t.title"
                }
            }
        };
        sql += &format!("ORDER BY {col} COLLATE NOCASE {} LIMIT 5000", if desc { "DESC" } else { "ASC" });
        let db = self.db.lock();
        let Ok(mut st) = db.prepare(&sql) else { return vec![] };
        st.query_map(params_from_iter(args.iter()), row)
            .map(|r| r.filter_map(|x| x.ok()).collect())
            .unwrap_or_default()
    }

    pub fn update_tags(&self, id: i64, u: TagUpdate) -> Result<(), String> {
        let db = self.db.lock();
        macro_rules! set {
            ($f:ident, $col:literal) => {
                if let Some(v) = &u.$f {
                    db.execute(concat!("UPDATE tracks SET ", $col, "=? WHERE id=?"), params![v, id]).map_err(|e| e.to_string())?;
                }
            };
        }
        set!(title, "title");
        set!(artist, "artist");
        set!(album, "album");
        set!(genre, "genre");
        set!(rating, "rating");
        set!(tags, "tags");
        set!(comment, "comment");
        Ok(())
    }

    /// Griglia corretta a mano per un file (sovrascrive l'analisi automatica al prossimo caricamento).
    pub fn save_grid(&self, path: &str, beats: &[f64], downbeat: usize, bpm: f64, fixed: bool, pins: &[usize], auto: &[f64]) {
        let js = serde_json::to_string(beats).unwrap_or_default();
        let pins_js = serde_json::to_string(pins).unwrap_or_default();
        let anchors: Vec<f64> = pins.iter().filter_map(|&i| beats.get(i).copied()).collect();
        let anchors_js = serde_json::to_string(&anchors).unwrap_or_default();
        let auto_js = serde_json::to_string(auto).unwrap_or_default();
        let _ = self.db.lock().execute(
            "INSERT INTO grids(path,beats,downbeat,bpm,fixed,pins,anchors,auto_beats) VALUES(?,?,?,?,?,?,?,?)
             ON CONFLICT(path) DO UPDATE SET beats=excluded.beats, downbeat=excluded.downbeat, bpm=excluded.bpm, fixed=excluded.fixed,
               pins=excluded.pins, anchors=excluded.anchors, auto_beats=excluded.auto_beats",
            params![path, js, downbeat as i64, bpm, fixed as i64, pins_js, anchors_js, auto_js],
        );
    }
    pub fn load_grid(&self, path: &str) -> Option<(Vec<f64>, usize, f64, bool, Vec<usize>)> {
        let db = self.db.lock();
        db.query_row("SELECT beats,downbeat,bpm,fixed,pins FROM grids WHERE path=?", [path], |r| {
            let js: String = r.get(0)?;
            let pins: Option<String> = r.get(4)?;
            Ok((
                serde_json::from_str::<Vec<f64>>(&js).unwrap_or_default(),
                r.get::<_, i64>(1)? as usize,
                r.get(2)?,
                r.get::<_, i64>(3)? != 0,
                pins.and_then(|p| serde_json::from_str::<Vec<usize>>(&p).ok()).unwrap_or_default(),
            ))
        })
        .ok()
    }
    pub fn delete_grid(&self, path: &str) {
        let _ = self.db.lock().execute("DELETE FROM grids WHERE path=?", [path]);
    }

    pub fn remove_tracks(&self, ids: &[i64]) {
        let db = self.db.lock();
        for id in ids {
            let _ = db.execute("DELETE FROM tracks WHERE id=?", [id]);
        }
    }

    pub fn playlists(&self) -> Vec<Playlist> {
        let db = self.db.lock();
        let Ok(mut st) = db.prepare(
            "SELECT p.id,p.name,(SELECT COUNT(*) FROM playlist_tracks WHERE playlist_id=p.id) FROM playlists p ORDER BY p.name COLLATE NOCASE",
        ) else {
            return vec![];
        };
        st.query_map([], |r| Ok(Playlist { id: r.get(0)?, name: r.get(1)?, count: r.get(2)? }))
            .map(|r| r.filter_map(|x| x.ok()).collect())
            .unwrap_or_default()
    }
    pub fn playlist_create(&self, name: &str) -> Result<i64, String> {
        let db = self.db.lock();
        db.execute("INSERT INTO playlists(name) VALUES(?)", [name]).map_err(|e| e.to_string())?;
        Ok(db.last_insert_rowid())
    }
    pub fn playlist_rename(&self, id: i64, name: &str) {
        let _ = self.db.lock().execute("UPDATE playlists SET name=? WHERE id=?", params![name, id]);
    }
    pub fn playlist_delete(&self, id: i64) {
        let db = self.db.lock();
        let _ = db.execute("DELETE FROM playlist_tracks WHERE playlist_id=?", [id]);
        let _ = db.execute("DELETE FROM playlists WHERE id=?", [id]);
    }
    pub fn playlist_add(&self, pid: i64, ids: &[i64]) {
        let db = self.db.lock();
        let mut pos: i64 = db
            .query_row("SELECT COALESCE(MAX(position),-1)+1 FROM playlist_tracks WHERE playlist_id=?", [pid], |r| r.get(0))
            .unwrap_or(0);
        for id in ids {
            let dup = db
                .query_row("SELECT 1 FROM playlist_tracks WHERE playlist_id=? AND track_id=?", params![pid, id], |_| Ok(()))
                .is_ok();
            if dup {
                continue;
            }
            let _ = db.execute("INSERT INTO playlist_tracks(playlist_id,track_id,position) VALUES(?,?,?)", params![pid, id, pos]);
            pos += 1;
        }
    }
    pub fn playlist_remove(&self, pid: i64, ids: &[i64]) {
        let db = self.db.lock();
        for id in ids {
            let _ = db.execute("DELETE FROM playlist_tracks WHERE playlist_id=? AND track_id=?", params![pid, id]);
        }
    }
    /// sposta un brano alla posizione `to` (riordino con drag & drop)
    pub fn playlist_move(&self, pid: i64, track: i64, to: i64) {
        let db = self.db.lock();
        let Ok(mut st) = db.prepare("SELECT track_id FROM playlist_tracks WHERE playlist_id=? ORDER BY position") else { return };
        let mut order: Vec<i64> = st.query_map([pid], |r| r.get(0)).map(|r| r.filter_map(|x| x.ok()).collect()).unwrap_or_default();
        drop(st);
        if let Some(i) = order.iter().position(|&t| t == track) {
            order.remove(i);
            order.insert((to.max(0) as usize).min(order.len()), track);
            for (p, t) in order.iter().enumerate() {
                let _ = db.execute(
                    "UPDATE playlist_tracks SET position=? WHERE playlist_id=? AND track_id=?",
                    params![p as i64, pid, t],
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn analysis_cache_roundtrip_and_playlists() {
        let dir = std::env::temp_dir().join(format!("spinlab_test_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let wav = dir.join("a.wav");
        std::fs::write(&wav, b"RIFF....WAVE").unwrap();
        let path = wav.to_string_lossy().to_string();
        let lib = Library::open(&dir.join("t.db")).unwrap();
        assert!(lib.analysis_get(&path).is_none());
        let a = crate::analysis::Analysis {
            beats: vec![0.5, 1.0, 1.5], bpm: 120.0, downbeat: 2, weights: vec![0.75, 0.5, 1.0], bpm_curve: vec![120.0; 3], key: Some(crate::camelot::Key { pc: 9, minor: true }),
            wave: crate::analysis::Wave { rate: 400, lo: vec![1, 2], mid: vec![3, 4], hi: vec![5, 6] },
            loudness: -12.5, peak: 0.9,
        };
        let id = lib.add_file(&path).unwrap_or_else(|_| 0);
        lib.analysis_put(&path, &a);
        let c = lib.analysis_get(&path).expect("in cache");
        assert_eq!((c.bpm, c.downbeat, c.beats.len()), (120.0, 2, 3));
        assert_eq!(c.key, Some(crate::camelot::Key { pc: 9, minor: true }));
        assert_eq!(c.wave.lo, vec![1, 2]);
        assert!((c.loudness + 12.5).abs() < 1e-9 && (c.peak - 0.9).abs() < 1e-6);
        // se il file cambia, l'analisi non vale piu'
        std::fs::write(&wav, b"RIFF....WAVE-modificato").unwrap();
        assert!(lib.analysis_get(&path).is_none());
        // brani fuori dalle playlist vengono tolti; quelli in playlist restano e non si duplicano
        let t = lib.tracks_for_paths(&[path.clone()]);
        assert_eq!(t.len(), 1);
        let _ = id;
        let pid = lib.playlist_create("p").unwrap();
        lib.playlist_add(pid, &[t[0].id]);
        lib.playlist_add(pid, &[t[0].id]);
        assert_eq!(lib.playlist_paths(pid).len(), 1);
        lib.prune_orphans();
        assert_eq!(lib.tracks_for_paths(&[path.clone()]).len(), 1);
        lib.playlist_remove(pid, &[t[0].id]);
        lib.prune_orphans();
        assert!(lib.db.lock().query_row("SELECT COUNT(*) FROM tracks", [], |r| r.get::<_, i64>(0)).unwrap() == 0);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
