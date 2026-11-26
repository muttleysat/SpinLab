# SpinLab DJ - note per chi sviluppa

[English](SVILUPPO.en.md) · **Italiano**

Questo file NON fa parte del manuale per gli utenti (non finisce nella guida dentro il programma). Vedi anche `docs/ASIO-licenza.md` per gli obblighi legati ad ASIO.

## Compilare il programma

Serve: Git, Rust (rustup.rs) con gli strumenti C++ di Visual Studio, LLVM/clang e l'**SDK ASIO** di Steinberg (non incluso nel progetto per la sua licenza). L'SDK va estratto in `src-tauri/asio-sdk` (la cartella è ignorata da git, non va mai caricata nel repository) in modo che dentro ci siano direttamente `common`, `host`, `driver`…: il percorso in `src-tauri/.cargo/config.toml` è relativo, uguale su tutti i PC. Usare l'SDK 2.3.4 (o più recente) dal sito Steinberg. LLVM viene trovato da solo in `C:/Program Files/LLVM`: se è altrove, impostare le variabili d'ambiente `LIBCLANG_PATH` e `CLANG_PATH` (non in quel file, che vale anche su macOS). Serve anche FFmpeg nel PATH (formati audio meno comuni).

```
cargo install tauri-cli --version "^2"
git clone https://github.com/muttleysat/SpinLab.git
cd SpinLab\src-tauri
cargo tauri dev        (per provare)
cargo tauri build      (per creare l'installer)
```

I dati dell'utente (impostazioni, libreria, mappatura MIDI, modello degli stem) stanno in `%APPDATA%\com.spinlab.dj` e non sono nel repository.

**«Accesso negato (os error 5)» compilando.** Il programma è ancora aperto e Windows non lo lascia sostituire. Chiudi SpinLab (e le copie rimaste nel Gestore attività).

## Creare gli installer e pubblicare una versione

SpinLab e' software libero (GPL-3.0-or-later) e gratuito: non ci sono registrazione, server delle licenze ne' aggiornamenti automatici. Ogni
versione si pubblica come **Release su GitHub** con gli installer allegati; chi la vuole la scarica e la installa sopra la precedente.

Prima volta sul computer: `python tools/fetch_model.py` (mette in `src-tauri/models` il modello degli stem e i due modelli dei battiti, da
dove l'installer li include: il programma installato non scarica niente) e `python tools/fetch_ffmpeg.py` (FFmpeg LGPL, vedi `docs/FFMPEG.md`).
La DLL dell'encoder MP3 e' gia' in `src-tauri/libs` (si ricostruisce con `tools/build_lame_dll.py`).

Per ogni versione:

1. Cambia il numero di versione in `src-tauri/tauri.conf.json` e in `src-tauri/Cargo.toml`, e aggiungi le novita' in `CHANGELOG.md`.
2. Se sono cambiate le dipendenze: `python tools/gen_third_party.py` (aggiorna `THIRD-PARTY-LICENSES.md`, che finisce nell'installer).
3. Crea gli installer:
   - **Windows**, dalla cartella `src-tauri`: `cargo tauri build --config tauri.release.conf.json` → `target/release/bundle/nsis` (e `msi`).
   - **macOS**: `zsh tools/build_mac.sh` → `src-tauri/target/release/bundle/dmg/SpinLab_<versione>_aarch64.dmg` (con lo SHA-256).
   Con `tauri.release.conf.json` l'installer include modelli, FFmpeg, DLL e licenze. Su Windows `tauri.windows.conf.json` aggiunge le
   librerie Visual C++ (`src-tauri/libs/vcredist`) e WebView2 offline: l'installer basta da solo anche su un Windows appena installato.
4. Su GitHub: **Releases → Draft a new release**, tag `v<versione>`, descrizione (modello in `docs/release/`), allega gli installer, **Publish**.

## Icone

Le icone dell'app si generano da un'unica immagine quadrata 1024 x 1024 (con trasparenza):

```
cd src-tauri
cargo tauri icon percorso\icona.png
```

## Aggiornare la guida dentro il programma

Il manuale si scrive in `docs/MANUALE.md`. Dopo ogni modifica:

```
python docs/build_guida.py
```

Genera `ui/guida.html`, che il programma mostra dal pulsante **?**.

## Stem su Android

Sul telefono gli stem si separano con la CPU (4 thread). Misure su Galaxy S26 Ultra con il modello normale: 2,6-3,0 s per un segmento da 7,8 s, cioè 1,95-2,2 volte il tempo reale. L'app accetta il tempo reale dalle 2 volte in su (`REALTIME_MIN_SPEED`), quindi con il modello normale il telefono è proprio sulla soglia.

Per questo su Android l'app preferisce `htdemucs_mobile.onnx`, se c'è: è lo stesso modello reso più leggero. Si apre in 0,65 s invece di 6,4; la separazione ha la stessa velocità. Con questo modello la misura all'avvio dell'app ha dato 2,63 volte il tempo reale, quindi il tempo reale resta acceso. Si crea dal modello normale:

```
pip install onnx onnxslim onnxruntime
python tools/make_mobile_model.py htdemucs.onnx htdemucs_mobile.onnx --verify
```

Non ha ancora un indirizzo di download (`MODEL_URL_MOBILE` in `src-tauri/src/stems.rs`): per ora si copia a mano su un telefono con la build di debug installata:

```
adb push htdemucs_mobile.onnx /data/local/tmp/
adb shell run-as com.spinlab.dj mkdir -p models
adb shell run-as com.spinlab.dj cp /data/local/tmp/htdemucs_mobile.onnx models/
```

La cartella dei dati dell'app è `/data/user/0/com.spinlab.dj`, quella in cui parte `run-as`. Al riavvio l'app rimisura la velocità perché il modello è cambiato.

**Prova nell'app (S26 Ultra, un deck, brano di 342 s).** La separazione parte al caricamento (AUTO STEM) e resta davanti alla testina: a 21 s di riproduzione gli stem erano pronti fino a 49 s, a 75 s fino a 142 s. Togliere la voce a 22 s ha avuto effetto subito. A regime, con il telefono caldo (38 °C), un segmento costa circa 3,4 s: 1,7 volte il tempo reale. Basta per un deck; due deck che separano insieme da zero non ce la fanno (la priorità va al deck che si ascolta).

Per pilotare l'app dal Mac: `adb forward tcp:9333 localabstract:webview_devtools_remote_<pid>` e poi i DevTools di Chrome (`chrome://inspect`) o il protocollo CDP. La cattura dello schermo dei DevTools non mostra i canvas: per le schermate usare `adb exec-out screencap -p`.

Provati e scartati sul telefono: NNAPI (37 s per segmento), fp16 (186 s), int8 dinamico (nessun guadagno). Per misurare un modello direttamente sul telefono c'è `src-tauri/examples/stem_bench.rs` (istruzioni in testa al file).

## Video

Il modulo video (finestra di uscita, transizioni, visualizzazioni, profili di qualità) è descritto in [VIDEO.md](VIDEO.md); i requisiti e le scelte di prodotto sono in [PRD-video.md](PRD-video.md). Per svilupparlo e provarlo:

- serve **FFmpeg** (per le copie di lavoro dei video, le misure di decodifica e codifica, la registrazione MP4 e l'audio dei formati meno comuni): in sviluppo basta quello del `PATH`; nell'installer è incluso come sidecar **LGPL** (vedi [FFMPEG.md](FFMPEG.md) e `tools/fetch_ffmpeg.py`);
- il codice sta in `src-tauri/src/video.rs` e in `ui/video*.js`, `ui/video-out.html`;
- nella finestra di uscita **H** mostra le misure in diretta.
