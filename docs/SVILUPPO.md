# SpinLab DJ - note per chi sviluppa

Questo file NON fa parte del manuale per gli utenti (non finisce nella guida dentro il programma). Vedi anche `docs/ASIO-licenza.md` per gli obblighi legati ad ASIO.

## Compilare il programma

Serve: Git, Rust (rustup.rs) con gli strumenti C++ di Visual Studio, LLVM/clang e l'**SDK ASIO** di Steinberg (non incluso nel progetto per la sua licenza). Sul PC di sviluppo i percorsi dell'SDK e di LLVM sono in `src-tauri/.cargo/config.toml`: sul nuovo PC vanno modificati. Serve anche FFmpeg nel PATH (formati audio meno comuni).

```
cargo install tauri-cli --version "^2"
git clone https://github.com/muttleysat/SpinLab.git
cd SpinLab\src-tauri
cargo tauri dev        (per provare)
cargo tauri build      (per creare l'installer)
```

I dati dell'utente (impostazioni, libreria, mappatura MIDI, modello degli stem) stanno in `%APPDATA%\com.spinlab.dj` e non sono nel repository.

**«Accesso negato (os error 5)» compilando.** Il programma è ancora aperto e Windows non lo lascia sostituire. Chiudi SpinLab (e le copie rimaste nel Gestore attività).

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
