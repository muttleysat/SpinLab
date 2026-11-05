# Prova di SpinLab su Windows — passo passo

Per la prova del modulo video (branch `video-mixing`) su un PC Windows. Serve a verificare FFmpeg incluso, le misure di prestazioni, la registrazione MP4 e le funzioni nuove. Esito da mandare a Claude (vedi «Cosa mandare»).

## 1. Prima di cominciare

Servono gli strumenti per compilare SpinLab (vedi `docs/SVILUPPO.md`): Git, Rust (rustup.rs) con gli strumenti C++ di Visual Studio, LLVM/clang e l'SDK ASIO di Steinberg (i percorsi sono in `src-tauri/.cargo/config.toml`). Per lo script di FFmpeg serve anche Python 3 e la rete.

## 2. Il codice

```
git fetch
git checkout video-mixing
git pull
```

## 3. FFmpeg

Due modi, a scelta.

**A. Con lo script** (dalla cartella del progetto):

```
python tools\fetch_ffmpeg.py windows
```

Se `python` non esiste prova `py tools\fetch_ffmpeg.py windows`. Scarica la build LGPL di FFmpeg, controlla l'impronta SHA-256 e la mette in `src-tauri\binaries\ffmpeg-x86_64-pc-windows-msvc.exe` (circa 126 MB).

**B. Copiandolo dal Mac:** il file è nel Mac in `src-tauri/binaries/ffmpeg-x86_64-pc-windows-msvc.exe`.

**Dove metterlo per la prova con `cargo tauri dev`:** in sviluppo Tauri non lo copia da solo. Copia il file come `ffmpeg.exe` accanto all'eseguibile di sviluppo (`src-tauri\target\debug\ffmpeg.exe`) oppure in una cartella che sia nel `PATH`.

**Controllo:**

```
ffmpeg.exe -hide_banner -encoders | findstr h264
```

Deve comparire almeno uno fra `h264_nvenc`, `h264_qsv`, `h264_amf`, `h264_mf`. Se non compare nessuno, annota cosa scrive e mandamelo.

## 4. Avvio

```
cd src-tauri
cargo tauri dev
```

## 5. Cosa provare, in ordine

1. **Analisi del computer.** Aspetta una decina di secondi a deck fermi. Apri **Impostazioni → VIDEO** e leggi: scheda grafica, grafica 1080p e 720p, decodifica, codifica MP4, profilo scelto. Poi carica un video, premi play e premi **Ottimizza con un video in riproduzione** (circa 10 secondi).
2. **Video mp4.** Caricalo su un deck: deve partire, restare a tempo con la musica, seguire lo scratch e i salti. Prova alcune transizioni dal menu e una visualizzazione per un brano senza video. Nella finestra video premi **H** per le misure.
3. **Schermo esterno** (se c'è): clic destro sul pulsante VIDEO, scegli lo schermo; poi scollegalo e ricollegalo a musica in corso.
4. **Registrazione MP4.** Da **RECORD** scegli «MP4 con video», qualità e cartella e premi Avvia; per le volte dopo basta il pulsante tondo ● accanto all'orologio. Registra 3-5 minuti con un video in riproduzione. Ferma e controlla che il file si apra, che audio e immagine siano a tempo e che non ci siano barre nere strane.
5. **Chiusura.** Chiudi il programma con un video in riproduzione: anche la finestra video deve chiudersi.
6. **Miscela con i fader dei canali.** In **Impostazioni → VIDEO → Miscela** scegli «i fader dei canali», lascia il crossfader al centro e muovi i due fader del volume: l'immagine deve passare da un video all'altro.
7. **Pitch.** Imposta il pitch a +5% e premi **→0** sotto il valore: deve tornare a 0 gradualmente.

## 6. Cosa mandare a Claude

- Il **testo di Impostazioni → VIDEO** (hardware, misure, profilo).
- Il **registro del video**: apri `%TEMP%\spinlab_video\video.log` (per esempio con `notepad %TEMP%\spinlab_video\video.log`) e copia il contenuto. `%TEMP%` di solito è `C:\Users\<nome>\AppData\Local\Temp`.
- Il **testo della finestra video con H** durante la riproduzione.
- Eventuali **messaggi d'errore** (anche quelli rossi in basso o nella finestra di registrazione).
- Il modello di PC: scheda grafica, memoria, versione di Windows.

## 7. Problemi comuni

| Sintomo | Cosa controllare |
|---------|------------------|
| «ffmpeg non trovato» | Il file si chiama proprio `ffmpeg.exe` e sta accanto a `spinlab.exe` (cartella `target\debug`) o nel `PATH`. Riavvia il programma dopo averlo copiato |
| Windows avvisa che il file non è sicuro (SmartScreen) | `ffmpeg.exe` non è firmato: «Ulteriori informazioni → Esegui comunque». Annotalo, per l'installer servirà la firma del codice |
| La registrazione dice che l'uscita non disegna | La finestra video non deve essere nascosta; riprova con la finestra in vista |
| Nessun encoder H.264 | Aggiorna i driver della scheda grafica e rifai il controllo del punto 3 |
| Lo script non scarica | Controlla la rete; riprova; in alternativa copia il file dal Mac (modo B) |

L'installer completo con FFmpeg incluso (`cargo tauri build` con `src-tauri/tauri.release.conf.json`) non è ancora stato provato: se ti va, tienilo per un secondo momento.
