//! Registrazione del programma: la chiave di licenza si scambia con il server (hosting) contro un certificato firmato Ed25519,
//! legato a questo computer. Il programma verifica la firma da solo con la chiave pubblica incorporata, quindi funziona anche offline;
//! ogni tanto ricontrolla online (certificato di 30 giorni + 14 di tolleranza). Il server e' in `server/`.

use base64::Engine;
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

/// Chiave pubblica di firma (esadecimale, 32 byte): la coppia e' generata da `tools/license_setup.py keys`.
const PUBKEY_HEX: &str = include_str!("../license_pubkey.txt");
/// Indirizzo del server delle licenze (cartella `api` dell'hosting). Si cambia a compilazione con SPINLAB_LICENSE_URL.
const API: &str = match option_env!("SPINLAB_LICENSE_URL") {
    Some(u) => u,
    None => "https://licenze.spinlab.example/api",
};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Payload {
    pub v: u32,
    pub lic: String,
    pub name: String,
    pub mid: String,
    pub iat: i64,
    pub exp: i64,
    pub grace: i64,
    pub max: u32,
}

#[derive(Serialize, Deserialize, Default)]
struct Stored {
    key: String,
    cert: String,
    /// ultimo istante visto: serve a notare un orologio rimesso indietro per allungare la licenza
    last_seen: i64,
}

#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct Status {
    /// "ok" | "grace" (da ricontrollare online a breve) | "expired" | "none" | "dev" (build di sviluppo, nessun obbligo)
    pub state: &'static str,
    pub name: String,
    pub key: String,
    pub max: u32,
    /// giorni al prossimo controllo online obbligatorio (con la tolleranza)
    pub days_left: i64,
    pub message: String,
}

fn status_none(msg: &str) -> Status {
    Status { state: "none", name: String::new(), key: String::new(), max: 0, days_left: 0, message: msg.into() }
}

fn now() -> i64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs() as i64)
}

fn path(dir: &Path) -> PathBuf {
    dir.join("license.json")
}

fn hex_decode(s: &str) -> Option<Vec<u8>> {
    let s = s.trim();
    if s.len() % 2 != 0 {
        return None;
    }
    (0..s.len()).step_by(2).map(|i| u8::from_str_radix(&s[i..i + 2], 16).ok()).collect()
}

fn b64(s: &str) -> Option<Vec<u8>> {
    base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(s.trim_end_matches('=')).ok()
}

/// Mostra solo i gruppi iniziale e finale della chiave: SPIN-AB…-…WXYZ
fn mask(key: &str) -> String {
    let p: Vec<&str> = key.split('-').collect();
    if p.len() == 5 { format!("{}-{}…-…{}", p[0], &p[1][..2.min(p[1].len())], p[4]) } else { key.into() }
}

// ---------------------------------------------------------------- identita' del computer

fn raw_machine_id() -> Result<String, String> {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        let out = std::process::Command::new("reg")
            .args(["query", r"HKLM\SOFTWARE\Microsoft\Cryptography", "/v", "MachineGuid"])
            .creation_flags(0x0800_0000)
            .output()
            .map_err(|e| e.to_string())?;
        let t = String::from_utf8_lossy(&out.stdout).to_string();
        return t.lines().find(|l| l.contains("MachineGuid")).and_then(|l| l.split_whitespace().last()).map(str::to_string).ok_or_else(|| "identificativo del computer non trovato".into());
    }
    #[cfg(target_os = "macos")]
    {
        let out = std::process::Command::new("ioreg").args(["-rd1", "-c", "IOPlatformExpertDevice"]).output().map_err(|e| e.to_string())?;
        let t = String::from_utf8_lossy(&out.stdout).to_string();
        return t.lines().find(|l| l.contains("IOPlatformUUID")).and_then(|l| l.split('"').nth(3)).map(str::to_string).ok_or_else(|| "identificativo del computer non trovato".into());
    }
    #[cfg(all(unix, not(target_os = "macos"), not(target_os = "android")))]
    {
        return std::fs::read_to_string("/etc/machine-id").map(|s| s.trim().to_string()).map_err(|e| e.to_string());
    }
    #[allow(unreachable_code)]
    Err("registrazione non disponibile su questa piattaforma".into())
}

/// Impronta del computer: SHA-256 dell'identificativo di sistema (il server non vede mai l'identificativo vero).
pub fn machine_id() -> Result<String, String> {
    static ID: OnceLock<Result<String, String>> = OnceLock::new();
    ID.get_or_init(|| {
        let raw = raw_machine_id()?;
        let mut h = Sha256::new();
        h.update(b"spinlab-v1:");
        h.update(raw.trim().to_lowercase().as_bytes());
        Ok(h.finalize().iter().map(|b| format!("{b:02x}")).collect())
    })
    .clone()
}

fn label() -> String {
    std::env::var("COMPUTERNAME").or_else(|_| std::env::var("HOSTNAME")).unwrap_or_default()
}

// ---------------------------------------------------------------- certificato

/// Controlla firma e computer di un certificato `base64url(payload).base64url(firma)`.
pub fn verify_cert(cert: &str, machine: &str, pubkey_hex: &str) -> Result<Payload, String> {
    let (p, s) = cert.split_once('.').ok_or("certificato non valido")?;
    let (payload, sig) = (b64(p).ok_or("certificato non valido")?, b64(s).ok_or("certificato non valido")?);
    let pk: [u8; 32] = hex_decode(pubkey_hex).and_then(|v| v.try_into().ok()).ok_or("chiave pubblica non valida")?;
    let vk = VerifyingKey::from_bytes(&pk).map_err(|_| "chiave pubblica non valida")?;
    let sig = Signature::from_slice(&sig).map_err(|_| "firma non valida")?;
    vk.verify(&payload, &sig).map_err(|_| "firma non valida")?;
    let pl: Payload = serde_json::from_slice(&payload).map_err(|_| "certificato non valido")?;
    if pl.v != 1 || pl.mid != machine {
        return Err("la licenza e' di un altro computer".into());
    }
    Ok(pl)
}

/// Stato della licenza in un certo istante (separato dal resto per poterlo provare).
fn evaluate(pl: &Payload, key: &str, t: i64, last_seen: i64) -> Status {
    let base = |state, days, msg: &str| Status { state, name: pl.name.clone(), key: mask(key), max: pl.max, days_left: days, message: msg.into() };
    if t + 86_400 < last_seen {
        return base("expired", 0, "L'orologio del computer e' indietro: collegati a internet per ricontrollare la licenza");
    }
    let day = |s: i64| (s + 86_399) / 86_400;
    if t <= pl.exp {
        base("ok", day(pl.exp + pl.grace - t), "")
    } else if t <= pl.exp + pl.grace {
        base("grace", day(pl.exp + pl.grace - t), "Collegati a internet per rinnovare la registrazione")
    } else {
        base("expired", 0, "La registrazione va ricontrollata: collegati a internet")
    }
}

fn load(dir: &Path) -> Option<Stored> {
    serde_json::from_slice(&std::fs::read(path(dir)).ok()?).ok()
}
fn save(dir: &Path, s: &Stored) {
    let _ = std::fs::create_dir_all(dir);
    let _ = std::fs::write(path(dir), serde_json::to_vec_pretty(s).unwrap_or_default());
}

/// Stato attuale, senza rete. Tiene aggiornato l'ultimo istante visto.
pub fn status(dir: &Path) -> Status {
    // il porting mobile non ha (ancora) un sistema di registrazione; le build di sviluppo non hanno obblighi (SPINLAB_FORCE_LICENSE=1 per provare)
    if cfg!(any(target_os = "android", target_os = "ios")) || (cfg!(debug_assertions) && std::env::var_os("SPINLAB_FORCE_LICENSE").is_none()) {
        return Status { state: "dev", name: String::new(), key: String::new(), max: 0, days_left: 0, message: "Versione di sviluppo".into() };
    }
    let (Some(mut st), Ok(mid)) = (load(dir), machine_id()) else { return status_none("") };
    match verify_cert(&st.cert, &mid, PUBKEY_HEX) {
        Ok(pl) => {
            let t = now();
            let s = evaluate(&pl, &st.key, t, st.last_seen);
            if t > st.last_seen {
                st.last_seen = t;
                save(dir, &st);
            }
            s
        }
        Err(e) => status_none(&e),
    }
}

/// Il programma puo' lavorare? (registrato, o in tolleranza, o build di sviluppo)
pub fn allowed(dir: &Path) -> bool {
    matches!(status(dir).state, "ok" | "grace" | "dev")
}

// ---------------------------------------------------------------- server

fn explain(code: &str) -> String {
    match code {
        "chiave_non_valida" => "La chiave non e' scritta correttamente (SPIN-XXXXX-XXXXX-XXXXX-XXXXX)",
        "chiave_sconosciuta" => "Chiave non riconosciuta",
        "licenza_revocata" => "Questa licenza non e' piu' valida",
        "limite_computer" => "La licenza e' gia' attiva sul numero massimo di computer: disattivala su uno di quelli, oppure scrivici",
        "non_attivata" => "Questo computer non e' registrato con questa licenza",
        "troppi_tentativi" => "Troppi tentativi: riprova tra un'ora",
        _ => "Risposta inattesa dal server",
    }
    .into()
}

/// Chiamata al server: Ok(corpo JSON) oppure Err((codice HTTP o 0 se manca la rete, messaggio, codice dell'errore del server)).
fn call(endpoint: &str, body: serde_json::Value) -> Result<serde_json::Value, (u16, String, String)> {
    let url = format!("{}/{endpoint}.php", API.trim_end_matches('/'));
    let r = ureq::AgentBuilder::new().timeout(std::time::Duration::from_secs(20)).build().post(&url).send_json(body);
    match r {
        Ok(resp) => resp.into_json::<serde_json::Value>().map_err(|_| (0, "Risposta inattesa dal server".into(), String::new())),
        Err(ureq::Error::Status(code, resp)) => {
            let e = resp.into_json::<serde_json::Value>().ok().and_then(|j| j["error"].as_str().map(str::to_string)).unwrap_or_default();
            Err((code, explain(&e), e))
        }
        Err(_) => Err((0, "Server non raggiungibile: controlla la connessione a internet".into(), String::new())),
    }
}

fn take_cert(dir: &Path, key: &str, j: &serde_json::Value) -> Result<Status, String> {
    let cert = j["cert"].as_str().ok_or("Risposta inattesa dal server")?;
    let mid = machine_id()?;
    verify_cert(cert, &mid, PUBKEY_HEX)?;
    save(dir, &Stored { key: key.into(), cert: cert.into(), last_seen: now() });
    Ok(status(dir))
}

/// Registra il programma su questo computer con una chiave di licenza.
pub fn activate(dir: &Path, key: &str) -> Result<Status, String> {
    let mid = machine_id()?;
    let key = key.trim().to_uppercase();
    let j = call("activate", serde_json::json!({ "key": key, "machine": mid, "label": label(), "version": env!("CARGO_PKG_VERSION") })).map_err(|e| e.1)?;
    take_cert(dir, &key, &j)
}

/// Rinnova il certificato. Senza rete non cambia nulla; se il server dice che la licenza non vale piu', il programma si de-registra.
pub fn refresh(dir: &Path) -> Result<Status, String> {
    let Some(st) = load(dir) else { return Ok(status(dir)) };
    let mid = machine_id()?;
    match call("refresh", serde_json::json!({ "key": st.key, "machine": mid, "label": label(), "version": env!("CARGO_PKG_VERSION") })) {
        Ok(j) => take_cert(dir, &st.key, &j),
        Err((code, msg, e)) => {
            if matches!(e.as_str(), "licenza_revocata" | "non_attivata" | "chiave_sconosciuta") {
                let _ = std::fs::remove_file(path(dir));
            }
            if code == 0 { Ok(status(dir)) } else { Err(msg) }
        }
    }
}

/// Libera il posto sul server e toglie la registrazione da questo computer.
pub fn deactivate(dir: &Path) -> Result<Status, String> {
    let Some(st) = load(dir) else { return Ok(status_none("")) };
    let mid = machine_id()?;
    match call("deactivate", serde_json::json!({ "key": st.key, "machine": mid })) {
        Ok(_) => {}
        Err((_, msg, e)) if e != "chiave_sconosciuta" => return Err(msg),
        Err(_) => {}
    }
    let _ = std::fs::remove_file(path(dir));
    Ok(status_none(""))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{Signer, SigningKey};

    fn keys() -> (SigningKey, String) {
        let sk = SigningKey::from_bytes(&[7u8; 32]);
        let hex: String = sk.verifying_key().to_bytes().iter().map(|b| format!("{b:02x}")).collect();
        (sk, hex)
    }
    fn cert(sk: &SigningKey, pl: &Payload) -> String {
        let j = serde_json::to_vec(pl).unwrap();
        let e = base64::engine::general_purpose::URL_SAFE_NO_PAD;
        format!("{}.{}", e.encode(&j), e.encode(sk.sign(&j).to_bytes()))
    }
    fn pl(mid: &str) -> Payload {
        Payload { v: 1, lic: "SPIN-AAAAA-BBBBB-CCCCC-DDDDD".into(), name: "Mario".into(), mid: mid.into(), iat: 1000, exp: 1000 + 30 * 86400, grace: 14 * 86400, max: 2 }
    }

    #[test]
    fn valid_certificate_is_accepted_and_tampering_is_not() {
        let (sk, pk) = keys();
        let c = cert(&sk, &pl("m1"));
        assert_eq!(verify_cert(&c, "m1", &pk).unwrap().name, "Mario");
        assert!(verify_cert(&c, "m2", &pk).is_err(), "altro computer");
        let (p, s) = c.split_once('.').unwrap();
        let mut bad = b64(p).unwrap();
        bad[10] ^= 1;
        let e = base64::engine::general_purpose::URL_SAFE_NO_PAD;
        assert!(verify_cert(&format!("{}.{s}", e.encode(bad)), "m1", &pk).is_err(), "contenuto alterato");
        let other = SigningKey::from_bytes(&[9u8; 32]);
        assert!(verify_cert(&cert(&other, &pl("m1")), "m1", &pk).is_err(), "firmato con un'altra chiave");
    }

    #[test]
    fn validity_windows() {
        let p = pl("m1");
        let k = "SPIN-AAAAA-BBBBB-CCCCC-DDDDD";
        assert_eq!(evaluate(&p, k, 2000, 0).state, "ok");
        assert_eq!(evaluate(&p, k, p.exp + 100, 0).state, "grace");
        assert_eq!(evaluate(&p, k, p.exp + p.grace + 100, 0).state, "expired");
        assert_eq!(evaluate(&p, k, 2000, 2000 + 3 * 86400).state, "expired", "orologio indietro di 3 giorni");
        assert_eq!(evaluate(&p, k, 2000, 2000 + 3600).state, "ok", "una piccola differenza e' tollerata");
        assert!(evaluate(&p, k, 2000, 0).key.contains('…'), "la chiave non si mostra intera");
    }

    #[test]
    fn machine_id_is_stable_hash() {
        let a = machine_id().unwrap();
        assert_eq!(a.len(), 64);
        assert_eq!(a, machine_id().unwrap());
    }

    #[test]
    fn built_in_public_key_is_well_formed() {
        let b = hex_decode(PUBKEY_HEX).unwrap();
        assert_eq!(b.len(), 32);
        assert!(VerifyingKey::from_bytes(&b.try_into().unwrap()).is_ok());
    }

    /// Prova completa contro il server finto: SPINLAB_LICENSE_URL=http://127.0.0.1:8765/api cargo test ... -- --ignored license_live
    /// (con `python tools/mock_license_server.py` acceso). Va con SPINLAB_FORCE_LICENSE=1 perche' le build di sviluppo non hanno obblighi.
    #[test]
    #[ignore]
    fn license_live_activate_refresh_deactivate() {
        assert!(API.starts_with("http://127.0.0.1"), "compila con SPINLAB_LICENSE_URL del server finto");
        let dir = std::env::temp_dir().join("spinlab-license-live");
        let _ = std::fs::remove_dir_all(&dir);
        assert_eq!(status(&dir).state, "none");
        assert!(activate(&dir, "SPIN-ZZZZZ-ZZZZZ-ZZZZZ-ZZZZZ").is_err(), "chiave sconosciuta");
        let st = activate(&dir, " spin-aaaaa-bbbbb-ccccc-ddddd ").unwrap();
        assert_eq!((st.state, st.name.as_str(), st.max), ("ok", "Mario Rossi", 2));
        assert!(allowed(&dir) && status(&dir).days_left >= 43);
        assert_eq!(refresh(&dir).unwrap().state, "ok");
        // un certificato manomesso non vale
        let mut stored: Stored = load(&dir).unwrap();
        stored.cert.push('x');
        save(&dir, &stored);
        assert_eq!(status(&dir).state, "none");
        let _ = activate(&dir, "SPIN-AAAAA-BBBBB-CCCCC-DDDDD").unwrap();
        assert_eq!(deactivate(&dir).unwrap().state, "none");
        assert!(!path(&dir).exists());
    }
}
