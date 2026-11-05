# 1. Context

[← Indice ASD](./ASD.md) · Stato: **In Progress** · Ultimo aggiornamento: 2026-11-05

## 1.1 Business Context

> **[TO BE DEFINED]** — Il codice non dice quale problema di business risolve SpinLab, chi sono gli interessati né come verrà venduto. Domanda da porre (vedi `DISCOVERY-PLAN.md`): «Con parole tue: cos'è SpinLab e che valore dà? Chi sono gli utenti e gli altri interessati? Quali sono i piani di vendita?».

Quello che risulta dai materiali del progetto, senza interpretazioni:

- SpinLab DJ è un programma per DJ con due deck, mixer, effetti, separazione in stem, batteria, campionatore, registrazione del master, uscita video e supporto per controller MIDI (`README.md`, `docs/MANUALE.md`).
- Esiste un sistema di **licenze** (chiave di licenza, server PHP, certificato firmato Ed25519) e di **aggiornamenti** firmati (`docs/LICENZE.md`, `docs/SERVER-LICENZE.md`): il programma è pensato per essere distribuito con una licenza.
- Il progetto contiene bozze di EULA e informativa sulla privacy «da far rivedere prima della vendita» (`docs/legale/`): la vendita è un obiettivo, non ancora avviato.
- Utenti identificati per il video: DJ da locale o evento, DJ che prepara i set a casa e DJ con un computer modesto (`docs/PRD-video.md`, sezione 3).

## 1.2 Related Projects / Initiatives

| Project / Initiative | Relationship | Impact |
|---------------------|-------------|--------|
| Griglia dei battiti «Pulse Engine» (`PulseEngine/`, branch `pulse-engine-beatgrid`) | Riferimento C++ da cui sono stati portati in Rust i principi (soglia di flusso, smoothing del tempo, pesi dei battiti) | Il tracciamento dei battiti in `analysis.rs` è migliorato; il C++ non è compilato nell'app |
| Integrazione di piattaforme di streaming audio | Studio di fattibilità (nessuna decisione) | Richiede accordi di partnership con le piattaforme; nessun impatto sul codice attuale |
| Server licenze e aggiornamenti (`server/`) | Componente separato che l'app contatta per licenze e aggiornamenti | Non ancora documentato in questo ASD (domanda Q-01) |
| Versione Android («cockpit») | Il codice ha rami per Android (`cfg(target_os = "android")`, `ui/cockpit.js`, `ui/mobile.js`) | Il video è fuori ambito su Android (`docs/PRD-video.md`) |

## 1.3 Technical Context

### IT Strategy Context

Applicazione **desktop locale** (Windows come piattaforma principale, macOS in sviluppo), senza server applicativo: audio, libreria e analisi girano sul computer dell'utente. La rete serve solo per la licenza, gli aggiornamenti e, al primo uso, per scaricare il modello degli stem (circa 460 MB). Tutto il resto funziona offline.

### Related Components

| Component | Type | Impact |
|-----------|------|--------|
| FFmpeg | Programma esterno (processo), non incluso nel repository | Decodifica dei formati meno comuni, copie di lavoro dei video, misure, codifica MP4. Come sidecar `ffmpeg.exe` dell'installer o nel `PATH` |
| Motore web del sistema (WebView2 su Windows, WKWebView su macOS) | Componente del sistema operativo usato da Tauri | Mostra l'interfaccia e decodifica/disegna i video (`<video>`, WebGL) |
| Driver audio (WASAPI, ASIO, CoreAudio) | Sistema operativo / driver della scheda | Uscita audio; ASIO tramite l'SDK di Steinberg (non incluso per licenza) |
| Server licenze e aggiornamenti | Servizio web PHP | Verifica della licenza e distribuzione degli aggiornamenti firmati |
| Modello di separazione degli stem (htdemucs ONNX) | File scaricato al primo uso | Separazione in voce, batteria, basso, altro |

### Development Toolchains

| Layer | Stack |
|-------|-------|
| Backend / motore | Rust 2021, Tauri 2, cpal, Symphonia, rustfft, signalsmith-stretch, ONNX Runtime (`ort`), rusqlite, lofty, sysinfo, midir |
| Frontend | HTML, CSS e JavaScript senza framework; WebGL 1 con shader GLSL ES 1.00 |
| Shared tooling | `cargo`, `cargo tauri`; Python (strumenti in `tools/`); LLVM/clang e SDK ASIO per la compilazione su Windows |

### Technical Document References

| Documento | Contenuto |
|-----------|-----------|
| `docs/MANUALE.md` | Uso del programma (sezione Video inclusa) |
| `docs/VIDEO.md` | Documentazione tecnica del modulo video |
| `docs/PRD-video.md` | Requisiti del modulo video |
| `docs/SVILUPPO.md` | Compilazione, installer, aggiornamenti |
| `docs/LICENZE.md`, `docs/SERVER-LICENZE.md`, `docs/ASIO-licenza.md` | Licenze e server |
