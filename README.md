# SpinLab DJ

<img src="ui/assets/asio-logo-bw.png" alt="ASIO Compatible - ASIO is a registered trademark of Steinberg Media Technologies GmbH" width="110" align="right">

Programma per DJ con due deck, mixer, effetti, separazione in stem, batteria, campionatore, registrazione del master e supporto per qualsiasi controller MIDI.

App Tauri 2: il motore audio è in Rust (`src-tauri/src`), l'interfaccia è web (`ui/`).

## Documentazione

- **[Manuale completo](docs/MANUALE.md)**: uso del programma, controller MIDI, impostazioni audio, soluzione dei problemi.
- Dentro il programma: il pulsante **?** in alto a destra apre la stessa guida, anche senza internet.

Il manuale si scrive in `docs/MANUALE.md`. Dopo ogni modifica, per aggiornare la guida che si apre nel programma:

```
python docs/build_guida.py
```

## Compilare

Servono Git, Rust con gli strumenti C++ di Visual Studio, LLVM/clang, l'SDK ASIO di Steinberg (non incluso per la sua licenza; i percorsi si impostano in `src-tauri/.cargo/config.toml`) e FFmpeg nel PATH.

```
cargo install tauri-cli --version "^2"
cd src-tauri
cargo tauri dev        # per provare
cargo tauri build      # per creare l'installer
```

Dettagli e requisiti per macOS e Windows nel manuale, sezione «Compilare il programma».

ASIO is a registered trademark of Steinberg Media Technologies GmbH.
