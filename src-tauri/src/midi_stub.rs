//! MIDI non disponibile su Android (midir non lo supporta): stessa interfaccia di `midi.rs`, nessuna porta.

use crate::engine::Mixer;
use parking_lot::Mutex;
use std::sync::Arc;

pub static PORTS: Mutex<Vec<String>> = Mutex::new(Vec::new());

pub fn start(_mixer: Arc<Mutex<Mixer>>, _app: Option<tauri::AppHandle>) {}

pub fn out_ports() -> Vec<String> {
    Vec::new()
}

pub fn out_select(_name: &str) -> Result<(), String> {
    Ok(())
}

pub fn out_current() -> String {
    String::new()
}

pub fn send(_bytes: &[u8]) {}
