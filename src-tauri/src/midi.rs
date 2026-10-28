//! MIDI: apre tutte le porte di ingresso (e le riapre se si collega o scollega un controller).
//! Le note arrivano ai pad della batteria; ogni messaggio e' inoltrato anche all'interfaccia (evento "midi-in"),
//! che applica la mappatura del controller scelta dall'utente. L'uscita MIDI serve per i LED del controller.

use crate::engine::Mixer;
use midir::{MidiInput, MidiInputConnection, MidiOutput, MidiOutputConnection};
use parking_lot::Mutex;
use std::sync::mpsc;
use std::sync::Arc;
use tauri::Emitter;

/// nomi delle porte MIDI di ingresso attualmente collegate
pub static PORTS: Mutex<Vec<String>> = Mutex::new(Vec::new());

/// porta di uscita aperta (LED del controller) e suo nome
static OUT: Mutex<Option<(String, MidiOutputConnection)>> = Mutex::new(None);

/// un messaggio in arrivo: porta, stato (con canale), dato 1, dato 2
type Ev = (Arc<str>, u8, u8, u8);

fn handle(m: &Arc<Mutex<Mixer>>, tx: &mpsc::Sender<Ev>, port: &Arc<str>, msg: &[u8]) {
    // orologio, active sensing, sysex: non servono
    if msg.len() < 2 || msg[0] >= 0xF0 {
        return;
    }
    let (status, d1, d2) = (msg[0], msg[1], msg.get(2).copied().unwrap_or(0));
    let _ = tx.send((port.clone(), status, d1, d2));
    match status & 0xF0 {
        0x90 if d2 > 0 => m.lock().midi_note_on(d1, d2),
        0x80 | 0x90 => m.lock().midi_note_off(d1),
        0xA0 => m.lock().midi_aftertouch(Some(d1), d2),
        0xD0 => m.lock().midi_aftertouch(None, d1),
        _ => {}
    }
}

pub fn start(mixer: Arc<Mutex<Mixer>>, app: Option<tauri::AppHandle>) {
    let (tx, rx) = mpsc::channel::<Ev>();
    // inoltra all'interfaccia a gruppi (il jog manda centinaia di messaggi al secondo)
    std::thread::spawn(move || {
        while let Ok(first) = rx.recv() {
            let mut batch = vec![first];
            let t0 = std::time::Instant::now();
            while batch.len() < 512 {
                let left = std::time::Duration::from_millis(3).saturating_sub(t0.elapsed());
                match rx.recv_timeout(left) {
                    Ok(e) => batch.push(e),
                    Err(_) => break,
                }
            }
            let out: Vec<(String, u8, u8, u8)> = batch.into_iter().map(|(p, s, a, b)| (p.to_string(), s, a, b)).collect();
            if let Some(app) = &app {
                let _ = app.emit("midi-in", out);
            }
        }
    });
    std::thread::spawn(move || {
        let mut conns: Vec<(String, MidiInputConnection<()>)> = Vec::new();
        loop {
            if let Ok(probe) = MidiInput::new("spinlab-scan") {
                let ports: Vec<_> = probe.ports().into_iter().filter_map(|p| probe.port_name(&p).ok().map(|n| (p, n))).collect();
                // chiude le connessioni dei controller scollegati
                conns.retain(|(n, _)| ports.iter().any(|(_, pn)| pn == n));
                for (port, name) in &ports {
                    if conns.iter().any(|(n, _)| n == name) {
                        continue;
                    }
                    if let Ok(mut inp) = MidiInput::new("spinlab-in") {
                        // l'orologio MIDI (24 messaggi a battito) e l'active sensing sono rumore: non servono
                        inp.ignore(midir::Ignore::TimeAndActiveSense);
                        let m = mixer.clone();
                        let tx = tx.clone();
                        let label: Arc<str> = Arc::from(name.as_str());
                        if let Ok(c) = inp.connect(port, name, move |_, msg, _| handle(&m, &tx, &label, msg), ()) {
                            conns.push((name.clone(), c));
                        }
                    }
                }
                *PORTS.lock() = conns.iter().map(|(n, _)| n.clone()).collect();
            }
            // la porta di uscita scollegata si chiude da sola alla prossima scrittura
            std::thread::sleep(std::time::Duration::from_secs(2));
        }
    });
}

/// porte MIDI di uscita disponibili
pub fn out_ports() -> Vec<String> {
    MidiOutput::new("spinlab-out-scan")
        .map(|o| o.ports().into_iter().filter_map(|p| o.port_name(&p).ok()).collect())
        .unwrap_or_default()
}

/// Apre (o chiude, con nome vuoto) la porta di uscita per i LED.
pub fn out_select(name: &str) -> Result<(), String> {
    let mut g = OUT.lock();
    *g = None;
    if name.is_empty() {
        return Ok(());
    }
    let o = MidiOutput::new("spinlab-out").map_err(|e| e.to_string())?;
    let port = o.ports().into_iter().find(|p| o.port_name(p).ok().as_deref() == Some(name)).ok_or_else(|| format!("porta di uscita «{name}» non trovata"))?;
    let c = o.connect(&port, "spinlab-led").map_err(|e| e.to_string())?;
    *g = Some((name.to_string(), c));
    Ok(())
}

pub fn out_current() -> String {
    OUT.lock().as_ref().map(|(n, _)| n.clone()).unwrap_or_default()
}

/// Invia byte MIDI grezzi alla porta di uscita aperta.
pub fn send(bytes: &[u8]) {
    if bytes.is_empty() || bytes.len() > 3 {
        return;
    }
    let mut g = OUT.lock();
    if let Some((_, c)) = g.as_mut() {
        if c.send(bytes).is_err() {
            *g = None; // controller scollegato
        }
    }
}
