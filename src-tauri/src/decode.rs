//! Decodifica di qualsiasi formato audio: Symphonia (nativo) con fallback su FFmpeg.

use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::process::{Command, Stdio};

use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::DecoderOptions;
use symphonia::core::errors::Error as SymError;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

/// Ritorna audio stereo interleaved a `target_sr`.
pub fn decode_file(path: &str, target_sr: u32) -> Result<Vec<f32>, String> {
    match decode_symphonia(path) {
        Ok((data, sr)) if !data.is_empty() => Ok(resample_stereo(&data, sr, target_sr)),
        native => {
            let why = native.err().unwrap_or_else(|| "file vuoto".into());
            decode_ffmpeg(path, target_sr)
                .map_err(|e| format!("Formato non leggibile ({why}); FFmpeg: {e}"))
        }
    }
}

fn decode_symphonia(path: &str) -> Result<(Vec<f32>, u32), String> {
    let file = File::open(path).map_err(|e| e.to_string())?;
    let mss = MediaSourceStream::new(Box::new(file), Default::default());
    let mut hint = Hint::new();
    if let Some(ext) = Path::new(path).extension().and_then(|e| e.to_str()) {
        hint.with_extension(ext);
    }
    let probed = symphonia::default::get_probe()
        .format(&hint, mss, &FormatOptions::default(), &MetadataOptions::default())
        .map_err(|e| e.to_string())?;
    let mut format = probed.format;
    let track = format
        .tracks()
        .iter()
        .find(|t| t.codec_params.codec != symphonia::core::codecs::CODEC_TYPE_NULL)
        .ok_or("nessuna traccia audio")?;
    let id = track.id;
    let mut sr = track.codec_params.sample_rate.unwrap_or(44100);
    let mut decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &DecoderOptions::default())
        .map_err(|e| e.to_string())?;

    let mut out: Vec<f32> = Vec::new();
    let mut sbuf: Option<SampleBuffer<f32>> = None;
    loop {
        let packet = match format.next_packet() {
            Ok(p) => p,
            Err(SymError::IoError(_)) | Err(SymError::ResetRequired) => break,
            Err(e) => return Err(e.to_string()),
        };
        if packet.track_id() != id {
            continue;
        }
        match decoder.decode(&packet) {
            Ok(decoded) => {
                let spec = *decoded.spec();
                sr = spec.rate;
                let ch = spec.channels.count();
                if sbuf.as_ref().map_or(true, |b| b.capacity() < decoded.capacity() * ch) {
                    sbuf = Some(SampleBuffer::new(decoded.capacity() as u64, spec));
                }
                let buf = sbuf.as_mut().unwrap();
                buf.copy_interleaved_ref(decoded);
                let s = buf.samples();
                match ch {
                    1 => s.iter().for_each(|&v| out.extend_from_slice(&[v, v])),
                    2 => out.extend_from_slice(s),
                    _ => s.chunks(ch).for_each(|c| out.extend_from_slice(&[c[0], c[1]])),
                }
            }
            Err(SymError::DecodeError(_)) => continue,
            Err(e) => return Err(e.to_string()),
        }
    }
    Ok((out, sr))
}

fn decode_ffmpeg(path: &str, sr: u32) -> Result<Vec<f32>, String> {
    let mut cmd = Command::new(ffmpeg_path());
    cmd.args(["-v", "error", "-i", path, "-vn", "-f", "f32le", "-ac", "2", "-ar", &sr.to_string(), "-"])
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x0800_0000);
    }
    let mut child = cmd.spawn().map_err(|e| format!("ffmpeg non trovato ({e})"))?;
    let mut bytes = Vec::new();
    child.stdout.take().unwrap().read_to_end(&mut bytes).map_err(|e| e.to_string())?;
    let _ = child.wait();
    if bytes.len() < 8 {
        return Err("nessun audio decodificato".into());
    }
    Ok(bytes.chunks_exact(4).map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]])).collect())
}

/// Cerca ffmpeg accanto all'eseguibile (sidecar dell'installer) altrimenti nel PATH.
fn ffmpeg_path() -> std::path::PathBuf {
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let p = dir.join("ffmpeg.exe");
            if p.exists() {
                return p;
            }
        }
    }
    "ffmpeg".into()
}

/// Resampling cubico (Catmull-Rom) di audio stereo interleaved.
pub fn resample_stereo(data: &[f32], from: u32, to: u32) -> Vec<f32> {
    if from == to || data.len() < 8 {
        return data.to_vec();
    }
    let frames = data.len() / 2;
    let ratio = from as f64 / to as f64;
    let n_out = (frames as f64 / ratio) as usize;
    let mut out = Vec::with_capacity(n_out * 2);
    let at = |i: isize, c: usize| -> f32 { data[(i.clamp(0, frames as isize - 1) as usize) * 2 + c] };
    for n in 0..n_out {
        let p = n as f64 * ratio;
        let i = p.floor() as isize;
        let t = (p - i as f64) as f32;
        for c in 0..2 {
            let (a, b, cc, d) = (at(i - 1, c), at(i, c), at(i + 1, c), at(i + 2, c));
            let v = b + 0.5 * t * (cc - a + t * (2.0 * a - 5.0 * b + 4.0 * cc - d + t * (3.0 * (b - cc) + d - a)));
            out.push(v);
        }
    }
    out
}
