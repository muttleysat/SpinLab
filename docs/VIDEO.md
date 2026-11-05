# Modulo video — documentazione tecnica

Per chi sviluppa. Spiega come è fatto il video di SpinLab, come comunicano le parti, dove stanno i valori da tarare e come estenderlo. L'uso del programma è nel [manuale](MANUALE.md#video); i requisiti e le scelte di prodotto sono nel [PRD](PRD-video.md).

Stato: funzionante su macOS (provato su Apple M3 Pro); **non ancora provato su Windows**. Vedi [Limiti noti](#limiti-noti-e-cose-da-verificare).

## Indice

1. [Idea generale](#idea-generale)
2. [Mappa dei file](#mappa-dei-file)
3. [Flusso dei dati](#flusso-dei-dati)
4. [Comandi Rust](#comandi-rust)
5. [Messaggi tra le finestre](#messaggi-tra-le-finestre)
6. [Sincronia fra audio e video](#sincronia-fra-audio-e-video)
7. [Copia di lavoro (scratch e salti)](#copia-di-lavoro-scratch-e-salti)
8. [Compositore WebGL e transizioni](#compositore-webgl-e-transizioni)
9. [Schede dei brani e visualizzazioni](#schede-dei-brani-e-visualizzazioni)
10. [Profili, misure e adattamento](#profili-misure-e-adattamento)
11. [Schermi e finestra di uscita](#schermi-e-finestra-di-uscita)
12. [Configurazione, memoria e permessi](#configurazione-memoria-e-permessi)
13. [Differenze fra Windows e macOS](#differenze-fra-windows-e-macos)
14. [Come aggiungere una transizione o una visualizzazione](#come-aggiungere-una-transizione-o-una-visualizzazione)
15. [Registrazione MP4](#registrazione-mp4)
16. [Prove e diagnostica](#prove-e-diagnostica)
17. [Limiti noti e cose da verificare](#limiti-noti-e-cose-da-verificare)

## Idea generale

Due principi guidano tutto:

- **L'audio è il master.** Il video segue la posizione che il motore audio dice per ogni deck. Se il computer fatica si degrada il video (meno fotogrammi, meno risoluzione) e mai l'audio.
- **Senza video non si paga nulla.** La modalità VIDEO è spenta finché non serve: nessuna finestra, nessun messaggio, nessun disegno. Il codice video non gira nel thread audio.

L'audio di un video passa dalla stessa pipeline dei brani (decodifica FFmpeg/Symphonia, analisi, stem): per il motore un video è un brano con un percorso diverso. Cambia solo cosa si mostra.

Architettura in due finestre, perché l'uscita è una finestra a parte da portare su un altro schermo:

```
 finestra principale (ui/index.html)                finestra di uscita (ui/video-out.html)
 ───────────────────────────────────                ──────────────────────────────────────
 deck, mixer, crossfader                            <video> ×2 (sorgenti nascoste)
 video.js ── evento "video-state" ~30/s ──────────▶ sincronizza i video con l'audio
          ── evento "video-art" (copertina) ──────▶ WebGL: texture ← video / scheda / visualizzazione
 video-perf.js (misure, profili, scheda Impost.)    compositore (video-gl.js): transizione col crossfader
          ◀── "video-quality" ogni secondo ──────── adattamento della qualità
          ◀── "video-adapt", "video-calibrated" ──
          ── "video-calibrate" ─────────────────────▶
        │
        ▼ comandi Tauri (src-tauri/src/video.rs)
 monitor, apertura/chiusura finestra, copia di lavoro, rilevamento hardware, misure con FFmpeg
```

## Mappa dei file

| File | Ruolo |
|---|---|
| `src-tauri/src/video.rs` | Comandi Rust: monitor, finestra di uscita, copia di lavoro, hardware, misure. |
| `src-tauri/src/engine.rs` | Due campi in più nello stato esposto: `Snapshot.crossfader` e `DeckState.xf_assign`. |
| `src-tauri/src/library.rs` | `EXTS` comprende anche le estensioni video. |
| `src-tauri/src/decode.rs` | `ffmpeg_path()` è ora `pub(crate)`: lo usa anche `video.rs`. |
| `ui/video.js` | Finestra principale: attivazione automatica, pulsante VIDEO, menu schermo, menu transizioni e visualizzazioni, invio dello stato, copie di lavoro, etichetta di qualità. |
| `ui/video-perf.js` | Rilevamento e misure, scelta del profilo, scheda **Impostazioni → VIDEO**, calibrazione. |
| `ui/video-out.html` | Finestra di uscita: sincronia, caricamento delle texture, ciclo di disegno, titolo, adattamento. |
| `ui/video-gl.js` | Compositore WebGL e shader delle 13 transizioni. Isolato: non conosce deck né Tauri. |
| `ui/video-vis.js` | Visualizzazioni generative (shader su un canvas a parte). |
| `ui/video-rec.js` | Registrazione MP4 lato finestra principale: coordina audio, uscita video, unione. |
| `ui/video-trans.js` | Nomi delle transizioni (`VIDEO_TRANS`) e delle immagini di ripiego (`VIDEO_VIS`), condivisi fra le due finestre. |
| `src-tauri/tauri.conf.json`, `capabilities/default.json`, `Cargo.toml` | Protocollo `asset://`, finestra `video-out`, dipendenza `sysinfo`. Vedi [Configurazione](#configurazione-memoria-e-permessi). |

L'ordine degli script in `index.html` conta: `video-trans.js`, `video-gl.js`, `video-vis.js`, `video-perf.js`, poi `video.js`. `video-gl.js` e `video-vis.js` servono anche al programma principale, per le misure.

## Flusso dei dati

**Attivazione.** `video.js` esegue `sync()` ogni 500 ms:

1. `wanted()` decide se la modalità è accesa: se l'utente non ha forzato nulla (`manual === null`) lo è quando un deck ha un file con estensione video (`VEXT`); altrimenti vale la scelta manuale.
2. Se serve e l'uscita non è aperta, la apre (`video_open`); se non serve più, la chiude (`video_close`).
3. Con l'uscita aperta, ogni 33 ms invia lo stato (`send()`).
4. Per ogni video sui deck chiede una copia di lavoro (`video_proxy`); quando un video lascia i deck, ne cancella la copia.
5. Ogni 2 secondi (un tick su 4) controlla se lo schermo scelto è comparso o scomparso.

**Stato inviato** (`video-state`, ~30 volte al secondo; la finestra principale è l'unica a conoscere i deck):

| Campo | Significato |
|---|---|
| `xf` | **Posizione della miscela video** 0..1 (0 = solo deck 1, 1 = solo deck 2). Dipende dall'impostazione `spinlab.video_mix` (`videoMix()` in `video.js`): `xfader` (predefinito) = crossfader del motore (`window.__lastXf`, letto dal poll principale, segue anche il MIDI); `channels` = `peso2 / (peso1 + peso2)` con peso = volume del canale al quadrato (`DeckState.volume`, la stessa curva del motore); se la somma è quasi zero resta l'ultimo valore. |
| `trans` | Indice della transizione scelta. |
| `vis` | Indice dell'immagine di ripiego (0 = scheda del brano, 1..10 = visualizzazioni). |
| `quality` | Profilo attivo: `{ h, fps, visH, simple, level }`. |
| `decks[i].loaded`, `.path`, `.name`, `.title`, `.artist` | Brano caricato e relativi tag. |
| `decks[i].playing`, `.held` | Riproduzione e scrub (`held` vale anche con il piatto toccato). |
| `decks[i].pos` | Posizione in secondi, già **compensata dalla latenza del dispositivo audio** (vedi sotto). |
| `decks[i].rate` | Velocità di riproduzione (`1 + tempo_pct/100`). |
| `decks[i].proxy` | Percorso della copia di lavoro, solo quando è pronta. |
| `decks[i].assign` | Assegnazione al crossfader del motore (0 A, 1 B, 2 diretto). |
| `decks[i].beat`, `.bpm`, `.level`, `.hue` | Fase del battito 0..1, BPM attuale, livello del canale 0..1, tinta dalla tonalità Camelot (1..12 → 0..1). |

Il flusso è a senso unico: la finestra di uscita non chiede nulla ai deck. Per questo l'uscita può essere chiusa e riaperta senza conseguenze per l'audio.

## Comandi Rust

Tutti in `src-tauri/src/video.rs`, registrati in `lib.rs`.

| Comando | Argomenti | Ritorna | Note |
|---|---|---|---|
| `video_monitors` | — | `Vec<MonitorInfo>` (`index`, `name`, `width`, `height`, `primary`) | Elenco dei monitor del sistema. |
| `video_open` | `monitor: Option<usize>` | `()` | Chiude l'eventuale uscita aperta e ne apre una nuova: a schermo intero senza bordi sul monitor indicato, oppure una finestra 960×540 se `None`. |
| `video_close` | — | `()` | Chiude la finestra di uscita. |
| `video_is_open` | — | `bool` | La finestra di uscita esiste ancora? Serve a sapere se l'utente l'ha chiusa a mano (x o Esc). |
| `video_proxy` | `path` | percorso della copia | Crea o ritrova la copia di lavoro; vedi sotto. Asincrono. |
| `video_cleanup` | `keep: Vec<String>` | `()` | Cancella dalla cartella temporanea le copie non elencate in `keep` (vuoto = tutte). |
| `video_hw` | — | `HwInfo` (`cpu`, `cores`, `ram_gb`, `os`, `encoders`, `ffmpeg`) | CPU e RAM con `sysinfo`; gli encoder H.264 si leggono da `ffmpeg -encoders` (ordine: videotoolbox, nvenc, qsv, amf, h264_mf, libx264). |
| `video_bench_clip` | — | `BenchClip` (`path`, `encoder`, `encode_fps`) | Crea un video di prova 1080p da 3 s (`testsrc2`) col primo encoder che funziona e misura i fotogrammi codificati al secondo. Non prova `h264_mf`. |
| `video_bench_decode` | `path` | `f64` | Decodifica il video di prova 10 volte (900 fotogrammi) con `-hwaccel auto` e ritorna la velocità in multipli del tempo reale. |
| `video_bench_clean` | — | `()` | Cancella il video di prova. |
| `video_rec_prepare` | — | `RecPaths` (`audio`, `video`) | Prepara i percorsi temporanei della registrazione (WAV e MP4 senza audio) in `spinlab_video/`. |
| `video_rec_start` | `width`, `height`, `fps` | `()` | Avvia FFmpeg per la codifica; i fotogrammi arrivano su stdin. Chiamato dalla finestra di uscita. |
| `video_rec_frame` | corpo binario (JPEG), intestazione `x-lag-ms` | `()` | Un fotogramma; la coda è di 8, gli eccedenti si scartano. |
| `video_rec_stop` | — | `RecVideoInfo` (`frames`, `dropped`, `secs`, `path`) | Chiude la codifica e aspetta FFmpeg. |
| `video_rec_mux` | `audio`, `out`, `latency_ms` | `MuxResult` (`path`, `bytes`, `offset_ms`) | Unisce video e audio in MP4 e cancella i temporanei. |

FFmpeg viene sempre lanciato come processo esterno (`decode::ffmpeg_path()`: `ffmpeg.exe` su Windows o `ffmpeg` altrove accanto all'eseguibile, cioè il sidecar **LGPL** dell'installer, altrimenti nel `PATH`), senza finestra su Windows (`CREATE_NO_WINDOW`). Come si prepara, quale licenza ha e cosa serve per rispettarla: [FFMPEG.md](FFMPEG.md).

## Messaggi tra le finestre

Eventi Tauri (`__TAURI__.event`), tutti trasmessi a tutte le finestre.

| Evento | Da → a | Payload | Quando |
|---|---|---|---|
| `video-state` | principale → uscita | vedi sopra | ogni 33 ms con l'uscita aperta |
| `video-art` | principale → uscita | `{ i, path, art }` (`art` = JPEG 512 px in data URL, o `null`) | una volta al cambio di brano su un deck (è pesante per il flusso a 30 messaggi/s) |
| `video-quality` | uscita → principale | `{ h, fps, real, off, calib, cw, ch }` | ogni secondo: risoluzione e fps in uso, fps reali, livelli di riduzione, calibrazione in corso, dimensioni del canvas |
| `video-adapt` | uscita → principale | testo | quando l'adattamento scende o risale (il principale lo mostra come avviso) |
| `video-calibrate` | principale → uscita | `{}` | l'utente preme «Ottimizza» |
| `video-calibrated` | uscita → principale | `{ level }` (0..3) | la calibrazione ha trovato il livello |
| `video-rec-start` | principale → uscita | `{ w, h, fps }` | inizio della registrazione MP4 |
| `video-rec-ready` | uscita → principale | `{ ok, error? }` | la cattura è pronta (o l'errore) |
| `video-rec-stop` | principale → uscita | `{}` | fine della registrazione |
| `video-rec-done` | uscita → principale | `{ ok, frames, dropped, skipped, error? }` | la codifica è chiusa |

## Sincronia fra audio e video

Il codice è in `apply()` di `video-out.html`. I due `<video>` sono muti e il loro tempo viene portato a quello dell'audio.

**Posizione di riferimento.** `target = pos + età_del_messaggio × rate` mentre suona. `pos` arriva già ritardata della latenza del dispositivo (`audioLatency + visOffset`, come per le waveform): il suono esce dal buffer con ritardo e l'immagine deve aspettarlo. Il cursore **Allineamento della grafica** vale quindi anche per il video.

**Scarto.** `raw = currentTime − target`. Si filtra con una media esponenziale (`ema = 0,9·ema + 0,1·raw`), perché i messaggi arrivano con qualche ms di variazione.

| Situazione | Comportamento |
|---|---|
| Fermo (`!playing`) | Pausa; se `|raw| > 0,04 s` e non c'è già un salto in corso, salta a `target`. |
| In scrub (`held`) | Come fermo, ma con `fastSeek` (salta al fotogramma intero più vicino: molto più veloce) e senza limiti di frequenza: conta solo l'ultimo punto richiesto. |
| In riproduzione, scarto > 0,5 s | Salto a `target + lat × rate`, al massimo uno ogni 800 ms. `lat` è il **ritardo di ripartenza misurato** (vedi sotto). Dopo il salto, per `lat + 400 ms` lo scarto non si giudica (`grace`). |
| In riproduzione, 0,06 s < scarto ≤ 0,5 s | Velocità corretta di ±4% (±8% se > 0,25 s) fino a rientrare sotto 15 ms (isteresi: `s.fix`). |
| Altrimenti | Velocità costante = `rate`. Cambiare velocità di continuo fa scattare i video comuni. |

**Ritardo di ripartenza.** Dopo un salto il video impiega un po' a mostrare di nuovo immagini (ricostruisce dall'ultimo fotogramma intero). L'evento `seeked` misura quanto: `lat = 0,6·lat + 0,4·misura`, limitato tra 0,05 e 1,5 s (iniziale 0,35 s). Saltare **in avanti di `lat`** fa sì che, quando riparte, il video sia già al punto giusto. Senza questo anticipo il video resta sempre indietro e salta di nuovo, senza mai scorrere: era il difetto delle prime versioni.

**Perché non c'è un unico «seek» ad alta frequenza.** Ogni salto costa centinaia di ms su file con un fotogramma intero ogni 5 s: ripeterlo ogni 33 ms non converge mai. Per questo i salti sono rari, con isteresi e con anticipo.

## Copia di lavoro (scratch e salti)

`video_proxy` crea, in `std::env::temp_dir()/spinlab_video/`, un file `<hash>.mp4`:

- **Contenuto:** solo il video (`-an`), altezza 480 (`scale=-2:480`), **tutti i fotogrammi interi** (`-g 1`), `yuv420p`, `+faststart`. I salti diventano quasi gratuiti.
- **Encoder**, nell'ordine, finché uno riesce: `h264_videotoolbox` (3 Mb/s), `h264_nvenc`, `h264_qsv`, `h264_amf` (3 Mb/s), `libx264` (`-preset ultrafast -crf 26`).
- **Nome:** FNV-1a a 64 bit di percorso (minuscolo) + dimensione + data di modifica: la stessa sorgente riusa la stessa copia (due deck con lo stesso video ne condividono una).
- **Scrittura atomica:** si scrive in `<hash>.part.mp4` e si rinomina a fine lavoro.
- **Costo misurato** (macOS, 720p H.264 da 3:50): circa 7 s e 85–170 MB secondo il bitrate; con l'encoder hardware circa 30× il tempo reale.

**Ciclo di vita.** L'originale parte subito; quando `decks[i].proxy` è valorizzato, la finestra di uscita sostituisce la sorgente nello stesso punto (`loadedmetadata` → `currentTime`), azzera `lat` e `ema` e sospende il giudizio sullo scarto per 800 ms. `video.js` chiama `video_cleanup` con le copie ancora in uso quando un video lascia i deck, e con `[]` all'avvio per ripulire quelle rimaste da una chiusura anomala. Con la modalità spenta non si crea nessuna copia.

## Compositore WebGL e transizioni

`video-gl.js` espone `VideoGL.create(canvas)`, che ritorna `null` (e imposta `VideoGL.error`) se manca WebGL o gli shader non compilano. È scritto in GLSL ES 1.00 (WebGL 1) per la massima compatibilità e non usa mipmap, quindi i campionamenti in rami non uniformi sono sicuri.

**API del compositore:** `resize(w, h)`, `upload(i, sorgente, w, h)` (carica un `<video>`, un canvas o un'immagine come texture del deck `i`; `false` se la scheda grafica rifiuta, tipicamente per CORS), `clear(i)`, `draw({ t, type, time, beat, has })`, più `sync()` (attende la fine del disegno con un `readPixels`, serve alle misure) e `info()` (nome della GPU).

**Uniform dello shader:** `uA`, `uB` (texture), `uRes`, `uSizeA`, `uSizeB` (dimensione dei contenuti, per adattarli senza deformarli con bande nere), `uT` (0 = solo deck A, 1 = solo deck B), `uTime`, `uBeat` (fase del battito), `uHasA`, `uHasB`, `uType`.

**Progresso della transizione.** `t = crossfader` (0..1): il deck 0 è sempre A (sinistra), il deck 1 sempre B. Se solo un deck ha contenuti, si mostra quello senza effetto; se nessuno, nero. L'assegnazione «diretto» del motore (`assign = 2`) non è gestita (vedi limiti).

**Le 13 transizioni** (`uType`; gli indici sono anche l'ordine di `VIDEO_TRANS`). «k» è `sin(π·t)`: 0 agli estremi, 1 a metà; serve a far partire e finire gli effetti che deformano l'immagine senza scarti.

| # | Nome | Idea |
|---|---|---|
| 0 | Dissolvenza | `mix(A, B, t)` |
| 1 | Taglio | A fino a metà corsa, poi B |
| 2 | Dissolvenza al nero | A→nero→B |
| 3 | Flash bianco | A→bianco→B |
| 4 | Wipe | bordo sfumato che scorre da sinistra |
| 5 | Zoom a spinta | A si ingrandisce, B cresce da piccolo |
| 6 | Glitch | righe spostate, canali RGB sdoppiati, blocchi che cambiano a caso (`hash` + tempo) |
| 7 | Pixel | l'immagine si scompone in blocchi e si ricompone |
| 8 | Caleidoscopio | riflessione a 6 spicchi, rotante |
| 9 | Ondulazione | onde radiali come acqua |
| 10 | Cubo 3D | due facce con prospettiva simulata in 2D |
| 11 | Vortice | rotazione più forte al centro |
| 12 | Stroboscopio sul ritmo | alterna A e B sulla fase del battito, con un lampo bianco |

Le prime cinque (0–4) sono le «leggere» disponibili con il profilo Bassa (`SIMPLE_TRANS` in `video-perf.js`, `SIMPLE` in `video-out.html`: **l'elenco è duplicato**).

**Ciclo di disegno** (`loop()` in `video-out.html`, su `requestAnimationFrame`):

1. Si calcola il livello di qualità attuale (`eff()`), si adegua la risoluzione se cambia (`sizeGL()`) e si applica il **tetto di fotogrammi** (`now − lastDraw < 1000/fps − 3 ms` → si salta il disegno).
2. Per ogni deck: se è un video e ha un fotogramma nuovo (`requestVideoFrameCallback`, in mancanza si confronta `currentTime`), carica la texture; se è un brano audio, usa la scheda o la visualizzazione.
3. `gl.draw()` con i parametri correnti. Con il profilo semplice le transizioni complesse diventano dissolvenza.

**Ripiego.** Se WebGL manca, se la scheda rifiuta una texture o se un video non si carica con l'accesso incrociato (`crossOrigin="anonymous"`; il protocollo `asset://` di Tauri risponde con l'intestazione che lo consente), la pagina passa al disegno semplice (`fallbackToDom`): i `<video>` e le schede HTML si sovrappongono con la sola trasparenza, cioè solo la dissolvenza. Il motivo compare in **H**.

## Schede dei brani e visualizzazioni

**Scheda** (`drawCard` in `video-out.html`): un canvas 2D 1280×720 diventa la texture del deck. Sfondo: la copertina ridotta a 32×18 e riscalata (sfocatura senza `ctx.filter`, che WebKit non ha) e scurita; copertina arrotondata con ombra; titolo (fino a 3 righe) e artista (fino a 2) con ritorno a capo e «…». Si ridisegna solo quando cambiano brano, titolo, artista o copertina. In modalità senza WebGL esiste una versione in HTML (`setCard`).

**Visualizzazioni** (`video-vis.js`): `VideoVis.create(w, h)` crea un canvas con un contesto WebGL proprio (`preserveDrawingBuffer: true`) e un solo fragment shader con dieci scene scelte da `uScene`. Ogni fotogramma si disegna sul canvas e si carica come texture del deck, come una scheda. Dimensione secondo il profilo (`visH`, rapporto 16:9). Ingressi:

| Uniform | Origine |
|---|---|
| `uTime` | secondi dal disegno |
| `uBeat` | `beat_phase` del deck (0..1) → l'impulso `exp(-uBeat·5)` pulsa sul battito |
| `uBar` | fase della battuta: `((pos·bpm/60) mod 4)/4` |
| `uLevel` | massimo di `level_l`/`level_r` del deck (dopo il fader del canale) |
| `uHue` | numero Camelot della tonalità attuale → 0..1 |

Scene: 0 Anelli, 1 Tunnel, 2 Onde luminose, 3 Griglia al neon, 4 Stelle, 5 Plasma, 6 Barre, 7 Esagoni, 8 Fluido, 9 Raggi (`VIDEO_VIS` = «Scheda del brano» + queste, con indice +1). Sono animazioni originali scritte per SpinLab.

**Titolo e artista** (`#title`): elemento HTML sopra il canvas, in basso a sinistra, dello stesso carattere per video e visualizzazioni. Mostra il brano del lato dominante (`op[0] ≥ op[1]`); sparisce con una dissolvenza e riappare al cambio. Non si vede sulle schede (che lo contengono). `#shade` è un'ombra scura in basso, visibile solo sulle visualizzazioni.

## Profili, misure e adattamento

### Profili

| Profilo | `h` | `fps` | `visH` | Transizioni | Livello |
|---|---|---|---|---|---|
| Bassa | 480 | 24 | 360 | solo le 5 leggere | 3 |
| Media | 720 | 30 | 540 | tutte | 2 |
| Alta | 1080 | 30 | 720 | tutte | 1 |
| Massima | 1080 | 60 | 720 | tutte | 0 |

Il «livello» indicizza `LEVELS` in `video-out.html` (`1080p60`, `1080p30`, `720p30`, `480p24`). `active()` in `video-perf.js` restituisce il profilo forzato (`spinlab.video_profile`) o quello automatico (`last.auto`), o `media` se non c'è ancora nessuna misura.

### Misure (`measure()`, `video-perf.js`)

1. **Hardware**: `video_hw` (CPU, core, RAM, encoder) e nome della GPU da WebGL (`WEBGL_debug_renderer_info`).
2. **Grafica**: due transizioni pesanti (glitch e vortice) disegnate in sequenza senza `requestAnimationFrame`, ciascuna seguita da `sync()`, con due texture 1920×1080 sintetiche. Prima 150 ms di riscaldamento a 640×360, poi 800 ms a 1080p e 800 ms a 720p; il risultato è in disegni al secondo. Le **visualizzazioni** (scene 8 e 5) si misurano per 600 ms a 960×540.
3. **Codifica**: `video_bench_clip` (fotogrammi/s dell'encoder).
4. **Decodifica**: `video_bench_decode` (multipli del tempo reale, massimo 10). Si fa con FFmpeg e non nel motore web, perché un `<video>` nascosto viene rallentato e dà valori falsi.
5. **Accelerazione** (`hwDecode`): `navigator.mediaCapabilities.decodingInfo` per H.264 1080p, campo `powerEfficient`.
6. **Stem**: `stems_cfg` (tempo reale sì/no), solo informativo.

### Scelta del profilo (`decide()`)

Soglie **provvisorie**, da tarare con prove su PC Windows di fascia bassa e media:

| Profilo | Condizione |
|---|---|
| Bassa | `shader720 < 40` |
| Media | `shader720 ≥ 40` |
| Alta | Media e `shader1080 ≥ 40`, `decodeX ≥ 2`, RAM ≥ 8 GB |
| Massima | Alta e `shader1080 ≥ 90`, `decodeX ≥ 3,5`, RAM ≥ 16 GB |

Casi prudenti: senza la misura della decodifica non si supera Media; con una misura fallita (`error`) si sceglie Media.

### Quando si misura

`auto()` parte 10 s dopo l'avvio e si ripete ogni 15 s (fino a 60 tentativi), **solo a deck fermi** perché usa la GPU. Misura se: non c'è nessun dato, `v !== 2`, la misura salvata ha `error` o `shader720 ≤ 0`, oppure è cambiata l'impronta (`cpu|core|RAM|gpu|sistema`). Il risultato si salva in `spinlab.video_perf`.

### Adattamento durante il set (`adapt()`)

Ogni secondo si confronta `fps` (disegni reali) con il tetto del livello (`e.fps`):

- sotto il 75% per 3 secondi di fila e non all'ultimo livello → **scende di un livello** (`adaptOff++`) e avvisa;
- almeno il 95% per 30 secondi e `adaptOff > 0` → **risale**.

Non valuta con la finestra non visibile (`document.visibilityState`) o senza deck caricati. Il profilo forzato fissa il livello di partenza, ma l'adattamento scende comunque se serve.

### Calibrazione (`calibrate()`, «Ottimizza»)

Richiede un video in riproduzione. L'uscita parte dal livello 0, ignorando i tetti del profilo, e scende un livello alla volta se `fps < 90%` del tetto; si ferma quando regge 3 secondi di fila (o dopo 25 secondi, o all'ultimo livello) e invia `video-calibrated`. Il secondo dopo ogni cambio di livello non conta. Il livello trovato diventa `auto` e si salva con `why` che lo indica. Se l'uscita non risponde entro 40 s la richiesta scade.

## Schermi e finestra di uscita

**Uscita coperta da altre finestre.** Il motore web non disegna una finestra completamente coperta (`document.visibilityState` diventa `hidden` e `requestAnimationFrame` si ferma): niente video, niente messaggi `video-quality` e niente cattura per la registrazione. Per questo l'uscita **in finestra** viene aperta sempre in primo piano (`always_on_top`; a schermo intero su un altro monitor non serve) e l'evento `video-pong` dice se la finestra è visibile e se sta disegnando; la registrazione, se l'uscita non disegna, lo riporta nel messaggio. Trovato con `video.log`: l'uscita rispondeva ma non disegnava perché stava dietro la finestra principale.

**Uscita chiusa a mano.** `sync()` in `video.js` chiede ogni 500 ms `video_is_open`: se la finestra non c'è più, `open` torna falso e `userClosed` blocca la riapertura automatica (la modalità automatica non deve riaprire una finestra che l'utente ha appena chiuso). Si riapre con il pulsante VIDEO, un nuovo video (se prima la modalità si era spenta) o l'avvio di una registrazione (`VideoMode.force(true)`). Prima il programma credeva l'uscita ancora aperta e la registrazione falliva con «l'uscita video non risponde». Se l'uscita non risponde, la registrazione distingue i casi con `video_is_open` e con l'evento `video-ping`/`video-pong` (l'uscita risponde se WebGL è attivo e perché no) e dà un messaggio preciso.

**Barra superiore e layout.** `#app` è una griglia con una sola colonna: se un elemento della barra superiore è più largo della finestra, la colonna `auto` si allarga e **tutto il layout** (deck, libreria) esce dalla finestra e si taglia a destra; se la sua larghezza cambia (etichetta di qualità, voce selezionata di un menu), tutto salta. Per questo: `grid-template-columns: minmax(0, 1fr)`, menu a larghezza fissa, e l'etichetta di qualità è dentro il pulsante VIDEO, fuori dal flusso (`position: absolute`).

**Chiusura del programma.** Alla chiusura della finestra principale (`CloseRequested` o `Destroyed` in `lib.rs`) `video::shutdown` distrugge la finestra di uscita e ferma un'eventuale registrazione video in corso (chiude la pipe e termina FFmpeg), così nessuna finestra o processo resta vivo. I file temporanei di una registrazione interrotta si ripuliscono all'avvio successivo.

- `video_open` costruisce la finestra con `WebviewWindowBuilder`, URL `video-out.html`, etichetta `video-out` (deve comparire in `capabilities/default.json`, altrimenti i comandi e gli eventi non funzionano).
- Con un monitor scelto: posizione e dimensioni in coordinate logiche del monitor, senza bordi, a schermo intero. Senza: finestra 960×540 con bordi.
- Lo schermo si ricorda in `spinlab.video_monitor_key` come `nome|LxA`, non come indice (l'indice cambia quando si collega o scollega qualcosa). Due monitor con lo stesso nome e la stessa risoluzione sono indistinguibili: si prende il primo.
- **Collegamento a caldo:** ogni 2 s, con l'uscita aperta, si confronta la chiave attesa con quella usata; se cambiano, si riapre l'uscita sul monitor giusto (o in finestra) con un avviso. L'audio non viene toccato.
- Tasti nella finestra: **H** (misure), **Esc** (chiude).

## Configurazione, memoria e permessi

| Dove | Cosa |
|---|---|
| `tauri.conf.json` → `app.security.assetProtocol` | `enable: true`, `scope: ["**"]`: la pagina legge i file dei video con `asset://`. **Da restringere** alle cartelle della libreria e alla cartella temporanea prima di una versione pubblica. |
| `Cargo.toml` | feature `protocol-asset` di `tauri`; dipendenza `sysinfo` (CPU e memoria). |
| `capabilities/default.json` | `windows: ["main", "video-out"]`. |
| `localStorage` | Vedi sotto. |

Chiavi di `localStorage` (finestra principale; sono per-utente e si perdono se si svuotano i dati del programma):

| Chiave | Contenuto |
|---|---|
| `spinlab.video_manual` | `"1"` sempre acceso, `"0"` sempre spento, vuoto = automatico |
| `spinlab.video_trans` | indice della transizione |
| `spinlab.video_vis` | indice dell'immagine di ripiego (0 = scheda) |
| `spinlab.video_mix` | `channels` = la miscela segue i fader dei canali; assente = crossfader |
| `spinlab.video_monitor_key` | schermo di uscita (`nome|LxA`) o vuoto |
| `spinlab.video_profile` | profilo forzato (`bassa`, `media`, `alta`, `massima`); assente = automatico |
| `spinlab.video_perf` | ultima analisi: `{ v: 2, fp, hw, res, auto, why, at }` |

## Differenze fra Windows e macOS

| Area | macOS | Windows |
|---|---|---|
| Motore web | WKWebView | WebView2 |
| Decodifica nel `<video>` | accelerata dal sistema (provato) | accelerata dal sistema (da verificare) |
| Encoder per copie e prova | `h264_videotoolbox`, poi `libx264` | `h264_nvenc`, `h264_qsv`, `h264_amf`, poi `libx264` |
| FFmpeg | `PATH` o accanto all'eseguibile | `ffmpeg.exe` accanto all'eseguibile (sidecar dell'installer) o `PATH`; senza finestra |
| `crossOrigin` e asset | verificato | **da verificare** |
| Codec leggibili nel `<video>` | H.264 provato; gli altri formati vanno verificati uno per uno | da verificare: WebView2 e WKWebView supportano insiemi diversi di codec e contenitori |

## Come aggiungere una transizione o una visualizzazione

**Nuova transizione:**

1. Aggiungi il nome in fondo a `VIDEO_TRANS` (`video-trans.js`): l'indice è il numero dello shader.
2. In `video-gl.js`, nella funzione `trans()` dello shader aggiungi un blocco `if (uType == N) { … }` che restituisce un `vec4`. Usa `fa(uv)` e `fb(uv)` per leggere i due deck (già adattati allo schermo), `uT` per il progresso e `k` per gli effetti che deformano. Evita cicli lunghi e `texture2D` fuori dagli helper.
3. Se è leggera e deve essere disponibile con il profilo Bassa, aggiungi l'indice a `SIMPLE_TRANS` (`video-perf.js`) **e** a `SIMPLE` (`video-out.html`).
4. Provala con la misura di grafica: nel pannello Impostazioni controlla che i disegni al secondo non crollino.

**Nuova visualizzazione:**

1. Aggiungi il nome in fondo a `VIDEO_VIS` (`video-trans.js`).
2. In `video-vis.js`, aggiungi un ramo a `scene()` con il nuovo `uScene` (l'ultimo ramo senza `if` è la scena 9). Ritorna un `vec3`; usa `uBeat`, `uLevel`, `uHue`, `uTime`, `pulse` (`exp(-uBeat·5)`) e `lv` (livello).
3. Tieni i cicli costanti e corti: gira a 960×540 (o `visH`) per ogni deck senza video.

## Registrazione MP4

Tre flussi, uniti alla fine. L'audio non passa mai da FFmpeg in tempo reale: si registra con il registratore esistente e si unisce dopo, così un problema del video non può rovinare l'audio.

```
 dialogo RECORD (app.js) ─▶ video-rec.js ─▶ accende l'uscita (VideoMode.force)
                                       ├─▶ video_rec_prepare  (percorsi temporanei)
                                       ├─▶ "video-rec-start"  ─▶ uscita: video_rec_start ─▶ FFmpeg (stdin = JPEG)
                                       └─▶ rec_start (WAV 24 bit)  ← segna l'istante di partenza dell'audio
 durante: uscita ─ per ogni fotogramma disegnato ─▶ video_rec_frame (JPEG) ─▶ coda (8) ─▶ FFmpeg
 stop:    rec_stop (audio) ─▶ "video-rec-stop" ─▶ uscita: video_rec_stop ─▶ video_rec_mux ─▶ MP4 finale
```

**Cattura (finestra di uscita, `captureFrame`).** Dopo ogni `gl.draw()` il canvas WebGL si copia su un canvas 2D della misura scelta: altezza e fotogrammi dal dialogo, **larghezza ricavata dalla forma del canvas di uscita** (limitata fra 1:1 e 2,4:1, dimensioni pari), così la registrazione ha la stessa forma dell'immagine che si vede e non aggiunge barre nere se la finestra o lo schermo non sono 16:9 (una forma fissa 16:9 le aggiungeva: errore trovato con una registrazione vera). La forma si fissa all'inizio: se poi la finestra cambia forma, il fotogramma si adatta con bande e si codifica con `toBlob("image/jpeg", 0,85)`. Si cattura al ritmo dei fotogrammi scelti (`next += 1000/fps`; in forte ritardo si riparte da adesso) e al massimo 3 richieste sono in volo: oltre, il fotogramma si salta e si conta. Il corpo della richiesta è binario (`invoke` con `Uint8Array`), senza JSON.

**Codifica (`rec_start_at`).** FFmpeg legge i JPEG da stdin come `image2pipe`/`mjpeg` con `-use_wallclock_as_timestamps 1` (il tempo di ogni fotogramma è quello di arrivo) e li porta a fotogrammi costanti con il filtro `fps=<f>` (i fotogrammi mancanti si ripetono), poi `scale` e `yuv420p`. Encoder: il primo che funziona fra `h264_videotoolbox`, `h264_nvenc`, `h264_qsv`, `h264_amf`, `h264_mf`, `libx264`, provato una volta con tre fotogrammi di prova (`rec_encoder`, in cache). Bitrate: 2,5 Mb/s fino a 480p, 5 Mb/s fino a 720p, 8 Mb/s oltre, ×1,5 sopra i 30 fps; `libx264` usa `-preset veryfast -crf 23`. Un fotogramma chiave ogni 2 secondi. Il registro di FFmpeg va in un file `.log` accanto al temporaneo e, se la codifica fallisce, l'ultima riga compare nell'errore.

**Coda e thread.** `video_rec_frame` mette i byte in un canale limitato (8) e ritorna subito; un thread dedicato li scrive sulla pipe. Se FFmpeg è più lento, `try_send` fallisce, il fotogramma si scarta e `dropped` aumenta: la finestra di uscita non si blocca mai.

**Sincronia: il calcolo dello spostamento (`mux_inner`).** Due correzioni:

1. **Partenza diversa.** `start_diff = t_video − t_audio`, dove `t_audio` è l'istante in cui `rec_start` ha installato la presa audio (`mark_audio_start`) e `t_video` è l'arrivo del primo fotogramma **meno il ritardo di cattura** (`x-lag-ms`, tempo fra il disegno e l'invio: copia, JPEG, `arrayBuffer`).
2. **Latenza del dispositivo audio.** L'immagine mostrata a `t` contiene l'istante `t − L` della musica (`L = audioLatency + visOffset`, la stessa compensazione usata dal video in riproduzione), mentre l'audio registrato a `t` contiene l'istante `t`: il video va spostato indietro di `L` sulla linea del tempo dell'audio.

`shift = start_diff − L`. Se `shift ≥ 0` si ritarda il video (`-itsoffset shift`), altrimenti si ritarda l'audio di `−shift`. L'unione copia il video (`-c:v copy`) e codifica l'audio in AAC a 192 kb/s (`-c:a aac`, encoder nativo di FFmpeg), con `+faststart`.

**Ciclo di vita e errori.** `video-rec.js` accende l'uscita per la sola durata della registrazione (`VideoMode.force`) e la riporta com'era. Se l'uscita non risponde entro 25 s allo stop, la codifica si chiude dal programma (`video_rec_stop`). Se l'unione fallisce, i file temporanei restano in `spinlab_video/` e il messaggio dice dove; se riesce, si cancellano. L'avvio della registrazione fallisce con un messaggio chiaro se manca WebGL, FFmpeg o un encoder.

**Test.** `video::tests::record_video_and_mux_with_audio` (`cargo test --lib video::`) esegue la pipeline vera con FFmpeg: 60 JPEG a 30 al secondo e un WAV di 2 secondi diventano un MP4 con video `h264` e audio `aac` della durata giusta, e verifica che lo spostamento calcolato coincida con la partenza ritardata misurata. Si salta da solo se mancano `ffmpeg` e `ffprobe`.

**Prestazioni.** Il costo principale è il JPEG nel motore web: a 720p 30 fps è comodo; a 1080p 60 fps su computer deboli salterà fotogrammi (che FFmpeg ripete). È per questo che la qualità della registrazione è separata da quella dell'uscita.

## Prove e diagnostica

- **Registro del video:** `video_log` scrive righe con data in `spinlab_video/video.log` (nella cartella temporanea del sistema; su macOS `$TMPDIR/spinlab_video/video.log`): apertura e chiusura dell'uscita, passi della registrazione, errori JavaScript della finestra di uscita e della principale. Serve a capire dove si ferma un'operazione senza schermate. Il file si azzera sopra 1 MB e `video_cleanup` non lo cancella.

- **Misure in diretta:** nella finestra di uscita premi **H**. Per ogni deck: dimensione del video, fotogrammi saltati/totali, sfasamento attuale e massimo (ms), velocità, «copia di lavoro sì/no», riallineamenti, ritardo di salto; più visualizzazione, uscita in uso (`720p 30 fps` e livelli di riduzione), tipo di disegno (`WebGL` o `semplice` con il motivo) e fps.
- **Errori JavaScript:** la barra errori del programma principale mostra gli errori; la finestra di uscita ha la console del motore web.
- **Controllo di sintassi senza Node:** `osascript -l JavaScript` con `new Function(testo)` su ciascun file (usato durante lo sviluppo, perché Node non c'è nell'ambiente).
- **Codice Rust:** `cargo check`, `cargo test --lib`. Il test `recorder::tests::mp3_is_written_and_valid` falliva già prima del modulo video.
- **Dati salvati:** il contenuto di `localStorage` sta in `~/Library/WebKit/<app>/WebsiteData/.../LocalStorage/localstorage.sqlite3` (valori in UTF‑16) su macOS.

**Verificato a mano (macOS, Apple M3 Pro):** riproduzione e sincronia con copia di lavoro, scrub, tutte le transizioni, le schede e le visualizzazioni, titolo e artista, misure del pannello, cambio di profilo e calibrazione.
**Verificato solo con test automatico (non nell'app):** la pipeline di registrazione MP4 (cattura da `video-out.html` e dialogo non provati a mano).
**Non verificato:** Windows (WebView2, encoder, D3D), collegamento e scollegamento a caldo di un monitor reale, più formati/codec oltre a H.264, video lunghi e carico prolungato.

## Limiti noti e cose da verificare

- Il deck 0 è sempre A e il deck 1 sempre B: l'assegnazione «diretto» del motore (`xf_assign = 2`) non è gestita.
- I formati leggibili sono quelli del motore web del sistema (H.264 sempre; altri dipendono dal computer). Per la compatibilità completa serve riprodurre con FFmpeg (domanda aperta 1 del PRD).
- Niente HDR né 10 bit; tetto di 1080p.
- Registrazione MP4: la cattura passa da JPEG nel motore web (costo di CPU) e richiede WebGL; a 1080p 60 fps su computer deboli perde fotogrammi. La sincronia è verificata sul calcolo (test) ma **non misurata con prove di lunga durata né su Windows**. Non c'è un timeout né la gestione dei processi FFmpeg orfani se il programma si chiude male.
- Le soglie dei profili sono provvisorie e vanno tarate su PC Windows.
- Il livello delle visualizzazioni è quello del canale **dopo il fader**: abbassando un deck la sua animazione si spegne anche se la musica continua.
- La perdita del contesto WebGL (`webglcontextlost`) non è gestita.
- `scope: ["**"]` del protocollo `asset://` è troppo largo per una versione pubblica.
- Se la finestra di uscita non è visibile, il motore web può rallentare il disegno e far scattare l'adattamento: l'adattamento lo ignora con `document.visibilityState`.
- `video_hw` rileva `h264_mf` (Windows) ma né la copia di lavoro né la prova di codifica lo usano ancora.
