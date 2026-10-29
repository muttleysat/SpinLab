# Licenze e uso commerciale - verifica dei componenti

Documento interno (non fa parte del manuale per gli utenti). Aggiornato il 29 ottobre 2026 sul codice di quel giorno. **Non e' un parere legale**: dove il rischio e' reale e' scritto, e per quelle voci serve un avvocato prima di vendere.

L'elenco completo delle librerie, con i testi delle licenze, e' in `THIRD-PARTY-LICENSES.md` (si rigenera con `python tools/gen_third_party.py`).

## Riepilogo

| Componente | Licenza | Uso commerciale | Stato |
|---|---|---|---|
| 408 librerie Rust (Tauri, cpal, rustfft, rusqlite, ort...) | MIT / Apache-2.0 / BSD / ISC / Zlib / Unlicense / CC0 / Unicode | si, con l'obbligo di riportare i testi | fatto: `THIRD-PARTY-LICENSES.md` nell'installer |
| 21 librerie MPL-2.0 (Symphonia, cssparser, selectors...) | MPL-2.0 (copyleft **solo sui singoli file**) | si, se non si modificano; chi le modifica deve pubblicare le modifiche a quei file | ok: non modifichiamo nessuna di queste librerie |
| LAME (encoder MP3) | **LGPL** | si, a condizione che l'utente possa sostituire la libreria | **corretto**: era legata dentro l'eseguibile, ora e' una DLL separata (`libmp3lame.dll`), caricata a richiesta |
| FFmpeg | LGPL o **GPL** a seconda della versione | non lo distribuiamo | ok: e' facoltativo, usato solo se l'utente lo ha gia' installato; vedi sotto |
| ONNX Runtime | MIT | si | ok |
| DirectML.dll | Microsoft, ridistribuibile | si | ok (testo nei crediti) |
| Microsoft WebView2 | Microsoft, ridistribuibile | si | ok (lo installa Tauri / Windows) |
| ASIO (SDK + marchio) | Steinberg: accordo da firmare, oppure GPLv3 | si con l'accordo firmato | in corso (firma da fare) |
| Font dell'interfaccia (Bahnschrift, Segoe UI) | Microsoft, font di sistema | si: non vengono distribuiti, li usa Windows | ok |
| Marchio e icone SpinLab | nostri | si | fatto, vedi note sul nome sotto |
| **Modello HT-Demucs (stem)** | **codice MIT, pesi non chiari** | **da verificare - rischio reale** | **da decidere, vedi sotto** |

## Il problema serio: il modello degli stem (HT-Demucs)

Cosa risulta:
- Il **codice** di Demucs (Meta) e' MIT.
- Per i **pesi** del modello (il file `htdemucs.onnx` che distribuiamo) Meta non ha mai dato una licenza chiara. Il manutentore del progetto ha scritto: *«The license of the weights is not clear: MusDB contains some data that were only usable for research purpose»* e anche *«I sadly cannot help granting a license, both coming from Meta custom data, or from the original MusDB data»*.
- Il modello e' addestrato su MUSDB18-HQ (dataset **per solo uso di ricerca / non commerciale**) piu' 800 brani interni di Meta.
- Il file che usiamo e' una conversione in ONNX fatta da StemSplit (https://huggingface.co/adowu/htdemucs-onnx), che sul suo modello mette `license: mit` «matching the original HT-Demucs». Quella etichetta non puo' dare piu' diritti di quelli che aveva l'originale.
- Il progetto Demucs di Meta e' stato archiviato il 1 gennaio 2025: nessuno rispondera' a domande sulla licenza.

Quindi: l'etichetta MIT esiste, ma i pesi dipendono da dati con restrizioni non commerciali. Usarli in un prodotto venduto e' una **zona grigia**. Molte applicazioni li usano, ma il rischio e' di chi vende.

### Strade possibili

| Strada | Pro | Contro |
|---|---|---|
| **A. Tenere HT-Demucs** e basarsi sull'etichetta MIT | nessun lavoro; qualita' migliore | rischio legale non eliminato; serve almeno il parere di un avvocato |
| **B. Passare a un modello con pesi chiaramente commerciali** | rischio basso | i pesi «puliti» di qualita' simile non esistono: Spleeter (Deezer) e' dichiarato MIT, ma ha qualita' piu' bassa e non documenta i dati; i modelli BS-RoFormer ecc. dipendono da MUSDB18 come Demucs |
| **C. Licenza commerciale da un fornitore** (StemSplit, AudioShake e altri) | certezza giuridica | costo; spesso solo cloud o accordi su misura |
| **D. Addestrare un modello proprio** su musica con diritti chiari | pulito, tuo | mesi di lavoro e GPU, dati da procurarsi |
| **E. Non distribuire il modello**: l'utente lo scarica e accetta lui la licenza | sposta la distribuzione | il prodotto resta dipendente dal modello; il rischio non sparisce del tutto |

Cosa abbiamo fatto: il modello e' un singolo file sostituibile (`htdemucs.onnx`, con la sua firma SHA-256 in `tools/fetch_model.py`), quindi cambiare strada non richiede di riscrivere il programma. **Consiglio**: non vendere con HT-Demucs senza il parere di un avvocato; valutare in parallelo la strada C.

## FFmpeg

Non e' distribuito con SpinLab. SpinLab legge da solo MP3, AAC/M4A, FLAC, WAV, AIFF, OGG/Vorbis, ALAC e MKV con Symphonia (MPL-2.0). Per formati rari (WMA, Opus, AC3...) usa FFmpeg **solo se l'utente lo ha gia' installato**. Un programma esterno scelto e installato dall'utente non crea obblighi di licenza per noi.

Se un giorno volessimo includerlo: solo una versione **LGPL** (senza componenti GPL ne «nonfree»), come eseguibile separato, con testo della licenza e offerta dei sorgenti. Le versioni «full» comuni (per esempio quelle di Gyan) sono GPL e **non** vanno incluse.

## LAME / MP3

LAME e' sotto LGPL. Legarla dentro il programma lo avrebbe obbligato a permettere all'utente di ricollegarlo con un'altra versione. Ora e' un file a parte (`libmp3lame.dll`) costruito dai sorgenti ufficiali con `tools/build_lame_dll.py`: l'utente puo' sostituirlo. Testo della licenza in `licenses/LICENSE-LAME.txt`. I brevetti MP3 sono scaduti (2017).

## Da controllare comunque

- **Il nome «SpinLab»**: fare una ricerca di marchi (EUIPO / UIBM / USPTO) per software e musica prima di investire nel marchio.
- **«Camelot»** (la notazione delle tonalita'): e' di uso comune, ma verificare con chi ha registrato «Camelot Wheel» se si vuole usare il nome nel materiale commerciale. Si puo' evitare scrivendo solo i codici (8A, 9B...).
- **Marco Dufour** ha contribuito al codice (port per macOS): concordare per iscritto a chi appartiene il suo codice prima di vendere.
- **Condizioni d'uso, privacy e garanzia** (EULA): da scrivere; se il programma contatta un server per gli aggiornamenti, trattare i dati che questo comporta (indirizzo IP).
