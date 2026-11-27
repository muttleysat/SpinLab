//! Copia di openDJ (https://github.com/openwave-studio/opendj, crates/engine/src/analysis.rs (intervallo dei tempi e tempi dei generi), commit d7432b2), Copyright (c) gli autori di
//! openDJ, GPL-2.0-or-later; distribuito in SpinLab sotto GPL-3.0-or-later. Il codice e' quello di openDJ senza modifiche:
//! la griglia di SpinLab deve dare gli stessi risultati.
//!
#![allow(dead_code)]

use std::sync::atomic::{AtomicU64, Ordering};

use crate::gridfit::GridMode;

pub const DEFAULT_BPM_RANGE: (f64, f64) = (60.0, 180.0);
static BPM_RANGE: AtomicU64 = AtomicU64::new(pack_range(DEFAULT_BPM_RANGE));

const fn pack_range((lo, hi): (f64, f64)) -> u64 {
    ((lo as u64) << 32) | hi as u64
}

/// The tempo range tracks are analysed in, in whole BPM.
pub fn bpm_range() -> (f64, f64) {
    let v = BPM_RANGE.load(Ordering::Relaxed);
    ((v >> 32) as f64, (v & 0xffff_ffff) as f64)
}

/// Sets the tempo range for tracks analysed from now on. The top must be
/// nearly twice the bottom, so every tempo has a place in it.
pub fn set_bpm_range(lo: f64, hi: f64) {
    let lo = lo.clamp(40.0, 200.0).round();
    let hi = hi.round().max((lo * 1.9).ceil()).min(400.0);
    BPM_RANGE.store(pack_range((lo, hi)), Ordering::Relaxed);
}
/// What analysis is told about a track before it listens: a tempo from tags
/// (see [`tag_tempo`]) and the track's genre, which picks the octave
/// tempos are read in (see [`genre_range`]). `None` and `Some(bpm)` convert
/// to a hint without a genre.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TempoHint {
    pub bpm: Option<f64>,
    pub genre: String,
    /// How the grid follows the tempo (see [`GridMode`]).
    pub mode: GridMode,
}

impl From<Option<f64>> for TempoHint {
    fn from(bpm: Option<f64>) -> Self {
        TempoHint {
            bpm,
            ..TempoHint::default()
        }
    }
}

impl TempoHint {
    /// The tempo range to read this track's tempo in: its genre's, or the
    /// one set with [`set_bpm_range`].
    pub fn range(&self) -> (f64, f64) {
        genre_range(&self.genre).unwrap_or_else(bpm_range)
    }

    /// The tempo the detector leans towards: the genre's, or 120 BPM.
    fn prior(&self) -> f64 {
        genre_tempo(&self.genre).unwrap_or(PRIOR_BPM)
    }
}

/// Typical tempos of genres whose tempo is clear, as (words in the genre,
/// tempo). The first match wins, so narrower genres come first. Genres that
/// span many tempos (pop, rock, electronic, dance) aren't here.
const GENRE_TEMPOS: &[(&[&str], f64)] = &[
    (
        &[
            "drum & bass",
            "drum and bass",
            "drum n bass",
            "drum'n'bass",
            "dnb",
            "d&b",
            "jungle",
            "neurofunk",
        ],
        172.0,
    ),
    (&["footwork", "juke"], 160.0),
    (&["hardstyle", "rawstyle"], 150.0),
    (&["gabber", "hardcore", "frenchcore"], 170.0),
    (&["dubstep", "riddim", "brostep"], 140.0),
    (&["grime"], 140.0),
    (&["psytrance", "psy-trance", "psy trance", "goa"], 142.0),
    (&["trance"], 136.0),
    (&["uk garage", "2-step", "2 step", "speed garage"], 132.0),
    (&["breakbeat", "breaks"], 130.0),
    (&["techno"], 130.0),
    (&["moombahton"], 108.0),
    (&["reggaeton", "dembow", "dancehall"], 95.0),
    (
        &["hip hop", "hip-hop", "hiphop", "rap", "r&b", "rnb", "trap"],
        90.0,
    ),
    (&["house", "garage", "disco", "big room", "electro"], 124.0),
    (
        &[
            "ambient",
            "downtempo",
            "chillout",
            "chill out",
            "lounge",
            "trip hop",
            "trip-hop",
            "lo-fi",
            "lofi",
        ],
        85.0,
    ),
];

/// The tempo range for tracks of `genre`: the octave around the genre's
/// typical tempo, so half and double tempo readings fold to the one the
/// genre is known by (drum & bass at 174 rather than 87, hip-hop at 90
/// rather than 180). None for an unknown or empty genre.
pub fn genre_range(genre: &str) -> Option<(f64, f64)> {
    genre_tempo(genre).map(genre_range_of)
}

/// The typical tempo of `genre`, from [`GENRE_TEMPOS`].
fn genre_tempo(genre: &str) -> Option<f64> {
    let genre = genre.to_lowercase();
    let has = |word: &str| {
        genre.match_indices(word).any(|(i, _)| {
            let before = genre[..i].chars().next_back();
            let after = genre[i + word.len()..].chars().next();
            // Whole words only, so "rap" isn't found in "trap" or "scraps".
            !before.is_some_and(char::is_alphanumeric) && !after.is_some_and(char::is_alphanumeric)
        })
    };
    GENRE_TEMPOS
        .iter()
        .find(|(words, _)| words.iter().any(|w| has(w)))
        .map(|&(_, tempo)| tempo)
}

/// The octave around `tempo`, in whole BPM.
fn genre_range_of(tempo: f64) -> (f64, f64) {
    let lo = (tempo / std::f64::consts::SQRT_2).round();
    (lo, 2.0 * lo)
}

/// Centre and width (standard deviation) of the tempo detector's lean.
const PRIOR_BPM: f64 = 120.0;

/// The tempo from tags in `bpm_hint`, if it is a plausible one; the analysis
/// then keeps it, steady.
pub fn tag_tempo(bpm_hint: Option<f64>) -> Option<f64> {
    bpm_hint.filter(|b| (40.0..=250.0).contains(b))
}
