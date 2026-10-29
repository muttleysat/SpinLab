//! Encoder MP3: LAME caricata a runtime come DLL separata (libmp3lame.dll).
//!
//! LAME e' sotto licenza LGPL: tenerla in un file a parte, sostituibile dall'utente, e' il modo normale di usarla in un programma
//! commerciale (la stessa scelta di Audacity). Il file si costruisce dai sorgenti ufficiali con `tools/build_lame_dll.py`.
//! Se la DLL non c'e' la registrazione in WAV funziona lo stesso e per l'MP3 si mostra un errore chiaro.

use libloading::Library;
use std::os::raw::{c_float, c_int, c_uchar, c_void};
use std::path::PathBuf;

type Init = unsafe extern "C" fn() -> *mut c_void;
type Close = unsafe extern "C" fn(*mut c_void) -> c_int;
type SetInt = unsafe extern "C" fn(*mut c_void, c_int) -> c_int;
type InitParams = unsafe extern "C" fn(*mut c_void) -> c_int;
type Encode = unsafe extern "C" fn(*mut c_void, *const c_float, c_int, *mut c_uchar, c_int) -> c_int;
type Flush = unsafe extern "C" fn(*mut c_void, *mut c_uchar, c_int) -> c_int;

pub struct Lame {
    handle: *mut c_void,
    close: Close,
    encode: Encode,
    flush: Flush,
    buf: Vec<u8>,
    // la libreria deve restare caricata finche' esiste l'encoder: va per ultima, cosi' si scarica dopo il resto
    _lib: Library,
}

// il puntatore di LAME e' usato da un solo thread alla volta (il thread di scrittura della registrazione)
unsafe impl Send for Lame {}

fn candidates() -> Vec<PathBuf> {
    let name = if cfg!(windows) {
        "libmp3lame.dll"
    } else if cfg!(target_os = "macos") {
        "libmp3lame.dylib"
    } else {
        "libmp3lame.so.0"
    };
    let mut v = vec![];
    if let Ok(exe) = std::env::current_exe() {
        if let Some(d) = exe.parent() {
            v.push(d.join(name));
            v.push(d.join("libs").join(name));
            // `cargo tauri dev` e i test girano da target/<profilo>: la DLL sta nella cartella libs del progetto
            for up in d.ancestors().take(4) {
                v.push(up.join("libs").join(name));
                v.push(up.join("src-tauri").join("libs").join(name));
            }
        }
    }
    v.push(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("libs").join(name));
    v.push(PathBuf::from(name)); // percorsi di ricerca di sistema
    v
}

fn load() -> Result<Library, String> {
    for p in candidates() {
        if p.components().count() > 1 && !p.exists() {
            continue;
        }
        if let Ok(l) = unsafe { Library::new(&p) } {
            return Ok(l);
        }
    }
    Err("encoder MP3 non trovato (manca libmp3lame.dll accanto al programma): registra in WAV oppure reinstalla SpinLab".into())
}

impl Lame {
    /// `kbps`: 128, 192, 256 o 320. Frequenza di campionamento e canali (sempre 2, interleaved) come il resto della registrazione.
    pub fn new(sr: u32, kbps: u32) -> Result<Lame, String> {
        let lib = load()?;
        unsafe {
            let init: Init = *lib.get::<Init>(b"lame_init\0").map_err(|e| e.to_string())?;
            let close: Close = *lib.get::<Close>(b"lame_close\0").map_err(|e| e.to_string())?;
            let set_sr: SetInt = *lib.get::<SetInt>(b"lame_set_in_samplerate\0").map_err(|e| e.to_string())?;
            let set_ch: SetInt = *lib.get::<SetInt>(b"lame_set_num_channels\0").map_err(|e| e.to_string())?;
            let set_br: SetInt = *lib.get::<SetInt>(b"lame_set_brate\0").map_err(|e| e.to_string())?;
            let set_q: SetInt = *lib.get::<SetInt>(b"lame_set_quality\0").map_err(|e| e.to_string())?;
            let params: InitParams = *lib.get::<InitParams>(b"lame_init_params\0").map_err(|e| e.to_string())?;
            let encode: Encode = *lib.get::<Encode>(b"lame_encode_buffer_interleaved_ieee_float\0").map_err(|e| e.to_string())?;
            let flush: Flush = *lib.get::<Flush>(b"lame_encode_flush_nogap\0").map_err(|e| e.to_string())?;
            let h = init();
            if h.is_null() {
                return Err("encoder MP3 non inizializzato".into());
            }
            let ok = set_sr(h, sr as c_int) == 0 && set_ch(h, 2) == 0 && set_br(h, kbps as c_int) == 0 && set_q(h, 5) == 0 && params(h) >= 0;
            if !ok {
                close(h);
                return Err("encoder MP3: impostazioni non accettate".into());
            }
            Ok(Lame { handle: h, close, encode, flush, buf: Vec::new(), _lib: lib })
        }
    }

    /// Codifica un blocco di campioni float interleaved (L, R, L, R...) fra -1 e 1; restituisce i byte MP3 prodotti.
    pub fn encode(&mut self, block: &[f32]) -> Result<&[u8], String> {
        let frames = block.len() / 2;
        // dimensione consigliata dalla documentazione di LAME: 1,25 campioni per canale + 7200 byte
        self.buf.resize(frames * 5 / 4 + 7200, 0);
        let n = unsafe { (self.encode)(self.handle, block.as_ptr(), frames as c_int, self.buf.as_mut_ptr(), self.buf.len() as c_int) };
        if n < 0 {
            return Err(format!("errore dell'encoder MP3 ({n})"));
        }
        Ok(&self.buf[..n as usize])
    }

    /// Svuota i campioni rimasti dentro l'encoder (da chiamare alla fine).
    pub fn finish(&mut self) -> Result<&[u8], String> {
        self.buf.resize(8192, 0);
        let n = unsafe { (self.flush)(self.handle, self.buf.as_mut_ptr(), self.buf.len() as c_int) };
        if n < 0 {
            return Err(format!("errore dell'encoder MP3 ({n})"));
        }
        Ok(&self.buf[..n as usize])
    }
}

impl Drop for Lame {
    fn drop(&mut self) {
        unsafe { (self.close)(self.handle) };
    }
}
