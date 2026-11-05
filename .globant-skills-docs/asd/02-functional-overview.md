# 2. Functional Overview

[← Indice ASD](./ASD.md) · Stato: **Done (draft)** · Ultimo aggiornamento: 2026-11-05

## 2.1 Functional Requirements References

I requisiti formali del modulo video sono nel [PRD del video](../../docs/PRD-video.md) (FR-01…FR-15, JTBD-01…JTBD-09). Le altre aree funzionali sono descritte nel [manuale](../../docs/MANUALE.md); per esse non esiste ancora un PRD (`[TO BE DEFINED]`).

| Requirement Area | Reference | Notes |
|-----------------|-----------|-------|
| Deck, trasporto, scrub, loop, pad | `docs/MANUALE.md` → «I deck» | Due deck; il motore ne ha quattro (`NUM_DECKS`) |
| Libreria e playlist | `docs/MANUALE.md` → «Libreria e playlist» | Database SQLite `library.db` |
| Analisi (BPM, tonalità, griglia dei battiti) | `docs/MANUALE.md` → «Griglia e tonalità» | `analysis.rs`; gli anchor manuali correggono la griglia |
| Mixer, cuffie, preascolto, effetti | `docs/MANUALE.md` | |
| Stem | `docs/MANUALE.md` → «Stem» | Tempo reale o preparati |
| Automix, batteria, campionatore | `docs/MANUALE.md` | |
| Registrazione del master | `docs/MANUALE.md` → «Registrare il master» | WAV, MP3 e MP4 con video (FR-12) |
| Controller MIDI | `docs/MANUALE.md` → «Controller MIDI» | |
| Licenza e aggiornamenti | `docs/LICENZE.md`, `docs/SERVER-LICENZE.md` | |
| **Video** | `docs/PRD-video.md` (FR-01…FR-15) | Uscita su schermo/proiettore, transizioni, visualizzazioni, profili, registrazione MP4 |

## 2.2 Coverage Strategy

L'architettura copre i requisiti del video così (dettaglio in `06-software-architecture.md`):

| FR | Dove si realizza |
|----|------------------|
| FR-01, FR-13 | Libreria (`library.rs`, `EXTS`), decodifica audio (`decode.rs`), riproduzione nel motore web |
| FR-02, FR-03 | Finestra di uscita (`video-out.html`, algoritmo di sincronia); copia di lavoro (`video.rs`) |
| FR-04 | Stem (`stems.rs`) sull'audio del video |
| FR-05, FR-09 | `video.rs` (monitor, finestra), `video.js` (modalità) |
| FR-06, FR-07, FR-08 | Compositore WebGL (`video-gl.js`), visualizzazioni (`video-vis.js`), schede (`video-out.html`) |
| FR-10, FR-11 | `video-perf.js` (misure e profili), `video-out.html` (adattamento) |
| FR-12 | `video-rec.js`, `video.rs` (cattura e unione), `recorder.rs` (audio) |
| FR-14, FR-15 | Non ancora realizzati (vedi Gap Management nel PRD) |
