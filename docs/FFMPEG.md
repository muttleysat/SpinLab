# FFmpeg in SpinLab — licenza, inclusione e aggiornamento

Per chi sviluppa e per chi cura la parte legale. Riassume come FFmpeg è incluso nel programma, quale licenza si applica e cosa va fatto per rispettarla. Non è un parere legale: i punti da far verificare sono indicati (decisione D-10 in [DECISIONI.md](DECISIONI.md)).

## In breve

- FFmpeg è **incluso nell'installer** come programma separato (**sidecar** di Tauri): `ffmpeg.exe` su Windows, `ffmpeg` su macOS, accanto all'eseguibile di SpinLab.
- Si distribuisce **solo una build LGPL**, cioè **senza** x264, x265, fdk-aac e altri componenti GPL o non liberi. Attenzione alla versione: la build **macOS** (compilata da noi) è **LGPL 2.1 o successiva**; la build **Windows** (BtbN) è compilata con `--enable-version3` ed è quindi **LGPL 3**.
- Non si chiede nessuna licenza a nessuno: la LGPL permette di includerlo, a condizioni che si rispettano (tabella sotto).
- I **brevetti dei codec** (H.264, H.265, AAC) sono un tema diverso dalla licenza del software e restano da verificare con il parere legale (D-10).

## A cosa serve

| Uso | Dove |
|-----|------|
| Copie di lavoro dei video (scratch e salti) | `video_proxy` |
| Misure di decodifica e codifica | `video_bench_clip`, `video_bench_decode` |
| Codifica della registrazione MP4 e unione con l'audio | `video_rec_*` |
| Audio dei formati rari (e di video senza decodifica nativa) | `decode.rs` |

Come lo trova il programma (`decode::ffmpeg_path`): prima `ffmpeg.exe` (Windows) o `ffmpeg` (altrove) **accanto all'eseguibile**, altrimenti il `PATH`. In sviluppo, dove non c'è il sidecar, si usa quello del `PATH`.

## Quale build e perché

| Piattaforma | Origine | Note |
|-------------|---------|------|
| Windows 64 bit | Build LGPL di BtbN (`ffmpeg-n9.0-latest-win64-lgpl-9.0.zip`, versione `n9.0.2-47-…`), con impronta SHA-256 verificata sull'elenco pubblicato | Eseguibile unico da circa 126 MB; **LGPL 3** (`--enable-version3`); contiene gli encoder `h264_nvenc`, `h264_qsv`, `h264_amf`, `h264_mf` (da provare su un PC) |
| macOS Apple Silicon | **Compilato dai sorgenti ufficiali** di ffmpeg.org con `--disable-gpl --disable-nonfree --disable-autodetect` | Encoder `h264_videotoolbox`, `aac` e `mjpeg` nativi; nessuna libreria presa dal sistema per caso |
| macOS Intel | Stessa procedura (`macos-x64`), serve `nasm` | Non preparato |

Scelta della LGPL: obblighi leggeri (sotto) e nessun obbligo di pubblicare il codice di SpinLab. La build GPL (x264) è stata scartata; le build «full» comuni (per esempio Gyan) sono GPL e **non** vanno incluse. Una build con `--enable-nonfree` (per esempio con fdk-aac) **non è ridistribuibile**.

## Obblighi della LGPL e dove sono soddisfatti

| Obbligo | Come lo soddisfiamo |
|---------|---------------------|
| Includere il testo della licenza | `licenses/FFMPEG-LICENSE.txt` (quella della build: LGPL 2.1 su macOS, LGPL 3 su Windows) e tutti i testi in `licenses/ffmpeg/`, copiati dai sorgenti ufficiali dallo script (non scritti a mano) |
| Avviso che si usa FFmpeg | Scheda **Informazioni** del programma, `THIRD-PARTY-LICENSES.md`, condizioni d'uso §5 (`docs/legale/EULA.md`) |
| Fornire i sorgenti della versione distribuita (o un'offerta scritta) | `licenses/FFMPEG-BUILD.txt` (versione, origine, impronte, configurazione); i sorgenti sono su ffmpeg.org/releases. **Indirizzo per richiedere i sorgenti su altro supporto: `[TO BE DEFINED]`** |
| Permettere di sostituire la libreria | FFmpeg è un file separato accanto al programma: si può sostituire con una versione compatibile |
| (LGPL 3) Includere anche il testo della GPL 3, a cui la LGPL 3 rimanda | `licenses/ffmpeg/COPYING.GPLv3`, preso dai sorgenti ufficiali |
| Non vietare modifica e reverse engineering per la parte LGPL | SpinLab e' software libero (GPL-3.0): modifica e studio sono permessi per tutto il programma |
| Non usare il nome come approvazione | La scheda Informazioni non presenta SpinLab come approvato dal progetto FFmpeg |

## Preparare e aggiornare FFmpeg per l'installer

```
python3 tools/fetch_ffmpeg.py windows        # su qualunque computer: scarica e verifica la build LGPL
python3 tools/fetch_ffmpeg.py macos-arm64    # su un Mac: compila dai sorgenti ufficiali (qualche minuto)
```

Il risultato sta in `src-tauri/binaries/` (ignorata da git, i file sono grandi): `ffmpeg-<triple>[.exe]`, `licenses/FFMPEG-LICENSE.txt`, `licenses/FFMPEG-BUILD.txt`. La compilazione dell'installer con `src-tauri/tauri.release.conf.json` li include (`externalBin` e `resources`). **Le build di sviluppo non richiedono il sidecar.**

Per aggiornare la versione: cambia `SRC_VERSION` (e `SRC_SHA256`, che lo script stampa la prima volta) o l'URL di Windows in `tools/fetch_ffmpeg.py`, rilancia lo script, poi **ripeti le verifiche** qui sotto.

### Verifiche prima di ogni rilascio

1. `ffmpeg -hide_banner -buildconf` **non deve** contenere `--enable-gpl`, `--enable-nonfree`, `libx264`, `libx265`, `fdk`. `--enable-version3` rende la build LGPL 3 (accettabile, ma cambia i testi e le condizioni: vedi sopra). Lo script di compilazione per macOS controlla tutto da solo; per Windows lo script rileva `--enable-version3` e lo scrive in `FFMPEG-BUILD.txt`.
2. `ffmpeg -hide_banner -encoders` deve mostrare almeno un encoder H.264 di sistema (`h264_videotoolbox` su Mac; `h264_nvenc`, `h264_qsv`, `h264_amf` o `h264_mf` su Windows), `aac` e `mjpeg`.
3. Il test `cargo test --lib video::` (registrazione MP4 con FFmpeg vero) deve passare **con quel sidecar** (mettilo in testa al `PATH`).
4. L'installer contiene `licenses/FFMPEG-LICENSE.txt` e `licenses/FFMPEG-BUILD.txt`.

## Cose da non fare

- Non includere build GPL (x264/x265) né `nonfree`.
- Non copiare a mano il testo della licenza: si prende dalla distribuzione ufficiale.
- Non togliere FFmpeg dal file separato (collegarlo dentro l'eseguibile cambierebbe gli obblighi).
- Non promettere formati o codec specifici nelle comunicazioni di vendita prima del parere sui brevetti.

## Punti aperti

| Punto | Stato |
|-------|-------|
| Brevetti di H.264, H.265, AAC per un prodotto distribuito | Parere legale (D-10); gli encoder di sistema riducono il rischio, non lo eliminano. **La LGPL 3 (build Windows) richiama le condizioni sui brevetti della GPL 3**: chiedere al legale se interagiscono con le licenze dei consorzi, o compilare per Windows una build LGPL 2.1 propria |
| Prova della build Windows (encoder disponibili, avvio da `Program Files`, firma del file) | Da fare su un PC Windows (D-13) |
| Dimensione dell'installer (la build LGPL di Windows è un eseguibile grande) | Da misurare; si può ridurre con una build più mirata |
| Se l'utente ha un proprio FFmpeg nel `PATH` | Il programma usa prima quello incluso; un FFmpeg dell'utente (anche GPL) resta una sua scelta e responsabilità |
| Indirizzo per l'offerta scritta dei sorgenti | `[TO BE DEFINED]` |
