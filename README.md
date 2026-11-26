# SpinLab DJ

<img src="ui/assets/asio-logo-bw.png" alt="ASIO Compatible - ASIO is a registered trademark of Steinberg Media Technologies GmbH" width="110" align="right">

Programma per DJ con due deck, mixer, effetti, separazione in stem, batteria, campionatore, **uscita video** per schermi e proiettori (con transizioni, visualizzazioni e registrazione del set in MP4), registrazione del master, **skin** e colori a scelta e supporto per qualsiasi controller MIDI. Per Windows e macOS.

**Versione attuale: 0.2.0** — le novità sono in [CHANGELOG.md](CHANGELOG.md).

App Tauri 2: il motore audio è in Rust (`src-tauri/src`), l'interfaccia è web (`ui/`).

## Documentazione

- **[Manuale completo](docs/MANUALE.md)**: uso del programma, controller MIDI, impostazioni audio, soluzione dei problemi.
- **[Novità](CHANGELOG.md)**: cosa è cambiato in ogni versione.
- **[FFmpeg](docs/FFMPEG.md)**: licenza, inclusione nell'installer e aggiornamento di FFmpeg.
- **[Decisioni](docs/DECISIONI.md)**: le decisioni aperte che spettano al responsabile del prodotto (con opzioni e raccomandazioni) e quelle già prese.
- **[Video](docs/VIDEO.md)**: documentazione tecnica del modulo video (architettura, messaggi, sincronia, profili); i requisiti sono in [PRD-video.md](docs/PRD-video.md).
- Dentro il programma: il pulsante **?** in alto a destra apre la stessa guida, anche senza internet.

Il manuale si scrive in `docs/MANUALE.md`. Dopo ogni modifica, per aggiornare la guida che si apre nel programma:

```
python docs/build_guida.py
```

## Installare

- **Windows**: installer `SpinLab_<versione>_x64-setup.exe`; le versioni installate si aggiornano da sole.
- **macOS** (Apple Silicon): `SpinLab_<versione>_aarch64.dmg` dalle [Release di GitHub](https://github.com/muttleysat/SpinLab/releases). Il programma non è ancora firmato da Apple: la prima volta aprilo con clic destro → **Apri**, oppure da Impostazioni di Sistema → Privacy e sicurezza → **Apri comunque**.

## Compilare

Su Windows servono Git, Rust con gli strumenti C++ di Visual Studio, LLVM/clang, l'SDK ASIO di Steinberg (non incluso per la sua licenza; i percorsi si impostano in `src-tauri/.cargo/config.toml`) e FFmpeg nel PATH. Su macOS bastano Git, Rust e gli strumenti da riga di comando di Xcode.

Per l'installer completo (modello degli stem, FFmpeg, licenze) prima `python tools/fetch_model.py` e `python tools/fetch_ffmpeg.py`, poi `cargo tauri build --config tauri.release.conf.json` (vedi `docs/SVILUPPO.md`).

```
cargo install tauri-cli --version "^2"
cd src-tauri
cargo tauri dev        # per provare
cargo tauri build      # per creare l'installer
```

Dettagli per chi sviluppa in `docs/SVILUPPO.md`.

ASIO is a trademark of Steinberg Media Technologies GmbH, registered in Europe and other countries.
