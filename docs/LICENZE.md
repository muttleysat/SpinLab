# Licenze e uso commerciale - verifica dei componenti

Documento interno (non fa parte del manuale per gli utenti). Aggiornato il 29 ottobre 2026 sul codice di quel giorno. **Non e' un parere legale**: dove il rischio e' reale e' scritto, e per quelle voci serve un avvocato prima di vendere.

L'elenco completo delle librerie, con i testi delle licenze, e' in `THIRD-PARTY-LICENSES.md` (si rigenera con `python tools/gen_third_party.py`).

## Riepilogo

| Componente | Licenza | Uso commerciale | Stato |
|---|---|---|---|
| 408 librerie Rust (Tauri, cpal, rustfft, rusqlite, ort...) | MIT / Apache-2.0 / BSD / ISC / Zlib / Unlicense / CC0 / Unicode | si, con l'obbligo di riportare i testi | fatto: `THIRD-PARTY-LICENSES.md` nell'installer |
| 21 librerie MPL-2.0 (Symphonia, cssparser, selectors...) | MPL-2.0 (copyleft **solo sui singoli file**) | si, se non si modificano; chi le modifica deve pubblicare le modifiche a quei file | ok: non modifichiamo nessuna di queste librerie |
| LAME (encoder MP3) | **LGPL** | si, a condizione che l'utente possa sostituire la libreria | **corretto**: era legata dentro l'eseguibile, ora e' una DLL separata (`libmp3lame.dll`), caricata a richiesta |
| FFmpeg | **LGPL** (2.1 o successiva per la build macOS; **LGPL 3** per la build Windows di BtbN, compilata con `--enable-version3`); solo build senza componenti GPL ne' non liberi | si, a condizione di testo della licenza, avviso, sorgenti e possibilita' di sostituirlo | **incluso nell'installer** come programma separato (sidecar); vedi `docs/FFMPEG.md` |
| ONNX Runtime | MIT | si | ok |
| DirectML.dll | Microsoft, ridistribuibile | si | ok (testo nei crediti) |
| Microsoft WebView2 | Microsoft, ridistribuibile | si | ok (lo installa Tauri / Windows) |
| ASIO (SDK + marchio) | Steinberg: accordo da firmare, oppure GPLv3 | si con l'accordo firmato | in corso (firma da fare) |
| Font dell'interfaccia (Bahnschrift, Segoe UI) | Microsoft, font di sistema | si: non vengono distribuiti, li usa Windows | ok |
| Marchio e icone SpinLab | nostri | si | fatto, vedi note sul nome sotto |
| **Modello HT-Demucs (stem)** | **codice MIT, pesi non chiari** | **da verificare - rischio reale** | **da decidere, vedi sotto** |

## Registro delle licenze e dei permessi da ottenere

Aggiornato il 2026-11-05 su indicazione del responsabile del prodotto. Stato di ogni voce: **Richiesta** (inviata, si attende), **Da fare**, **Da decidere**, **Fatto**. Non e' un parere legale: per le voci con rischio reale serve un avvocato prima di vendere (decisioni D-10 e D-17 in `DECISIONI.md`).

| # | Cosa | Con chi | Stato | Note |
|---|------|---------|-------|------|
| 1 | **ASIO**: accordo per l'SDK e uso del logo «ASIO Compatible» | Steinberg | **In attesa** | Senza accordo non si distribuisce un programma con ASIO (vedi `ASIO-licenza.md`) |
| 2 | **HEVC (H.265)**: brevetti del codec | Consorzio dei brevetti (da indicare) | **Richiesta** | Da chiarire se copre solo la **decodifica** (SpinLab non codifica HEVC). Indicare qui chi e da quando |
| 3 | **H.264 e AAC**: brevetti, soglie di esenzione | Consorzi dei brevetti | **Da fare** | La registrazione MP4 produce H.264 + AAC, con l'encoder AAC software di FFmpeg e gli encoder H.264 di sistema |
| 4 | **Pesi del modello degli stem (HT-Demucs)** | Avvocato o fornitore con licenza commerciale | **Da decidere** | Meta non ha chiarito la licenza dei pesi (dataset non commerciale); vedi sezione sul modello |
| 5 | **Contributo di Marco Dufour** (port per macOS) | Marco Dufour | **Da fare** | Accordo scritto su a chi appartiene il codice |
| 6 | **Marchio «SpinLab»**: ricerca di anteriorita' ed eventuale registrazione | EUIPO / UIBM / USPTO | **Da fare** | Anche «Camelot» nel materiale commerciale |
| 7 | **Certificato di firma del codice** | Autorita' di certificazione | **Da fare** | Per evitare gli avvisi di Windows (SmartScreen) |
| 8 | **Account Apple Developer** (firma e notarizzazione) | Apple | **Da fare** | Per distribuire su macOS fuori dallo store |
| 9 | **Parere dell'avvocato su EULA, informativa sulla privacy e donazioni/vendita** | Avvocato | **Da fare** | Le bozze sono in `legale/` |
| 10 | **Contratto con chi incassa** (vendita) o condizioni della piattaforma di donazioni, e profilo fiscale | Piattaforma di pagamento, commercialista | **Da decidere** | Dipende dal modello di distribuzione (D-17) |
| 11 | **Accordi con le piattaforme di streaming** (solo se si fara' lo streaming) | Beatport/Beatsource, TIDAL ecc. | **Da decidere** | Decisione D-15; non e' una questione tecnica |

**Senza nulla da chiedere** (si rispettano soltanto le licenze): FFmpeg e LAME (LGPL), le librerie MIT/Apache/BSD, le librerie MPL (non modificate), ONNX Runtime (MIT), DirectML e WebView2 (ridistribuibili). I brevetti MP3 sono scaduti. Vedi `FFMPEG.md`.

## Il problema serio: il modello degli stem (HT-Demucs)

Cosa risulta:
- Il **codice** di Demucs (Meta) e' MIT.
- Per i **pesi** del modello (il file `htdemucs.onnx` che distribuiamo) Meta non ha mai dato una licenza chiara. Il manutentore del progetto ha scritto: *«The license of the weights is not clear: MusDB contains some data that were only usable for research purpose»* e anche *«I sadly cannot help granting a license, both coming from Meta custom data, or from the original MusDB data»*.
- Il modello e' addestrato su MUSDB18-HQ (dataset **per solo uso di ricerca / non commerciale**) piu' 800 brani interni di Meta.
- Il file che usiamo e' una conversione in ONNX fatta da StemSplit (https://huggingface.co/adowu/htdemucs-onnx), che sul suo modello mette `license: mit` «matching the original HT-Demucs». Quella etichetta non puo' dare piu' diritti di quelli che aveva l'originale.
- Il progetto Demucs di Meta e' stato archiviato il 1 gennaio 2025: nessuno rispondera' a domande sulla licenza.

Quindi: l'etichetta MIT esiste, ma i pesi dipendono da dati con restrizioni non commerciali. Usarli in un prodotto venduto e' una **zona grigia**. Molte applicazioni li usano, ma il rischio e' di chi vende.

### Le due letture della licenza MIT (aggiornato il 30 ottobre 2026)

Il repository di Demucs contiene una licenza MIT (Copyright (c) Meta, Inc. and its affiliates) che permette espressamente di «use, copy, modify, merge, publish, distribute, sublicense, and/or sell». Il testo e' ora riportato nella scheda Informazioni del programma e in `THIRD-PARTY-LICENSES.md`.

- **A favore della vendita**: la MIT e' esplicita e pubblica; i dati di addestramento di norma non sono «copiati» nei pesi; il vincolo di MUSDB18 e' spesso letto come condizione d'uso del dataset, che vincola chi l'ha scaricato (Meta). Molti prodotti commerciali usano Demucs e non risultano contestazioni.
- **Contro**: quel testo e' nel repository del codice, non e' detto che copra i **pesi** (scaricati dai server di Meta); la MIT concede solo i diritti che chi la applica possiede; il manutentore ha scritto che la licenza dei pesi non e' chiara. Il rischio, se qualcuno contesta, e' di chi vende.

Conclusione: vendere con HT-Demucs e' una scelta di **rischio basso ma non nullo**. Il testo MIT e il copyright di Meta vanno sempre riportati.

### Riduzione del rischio: modello scaricato dall'utente (da fare)

Non includere `htdemucs.onnx` nell'installer: alla prima apertura dell'area stem il programma mostra la licenza e il link alla fonte, l'utente conferma e scarica (o importa) il file, controllato con la firma SHA-256. Cosi' la distribuzione dei pesi non e' nostra. Nell'EULA: l'uso del modello e' sotto la responsabilita' dell'utente; nel marketing non promettere «stem inclusi». Stato: **non ancora implementato** (oggi `tools/fetch_model.py` mette il modello nell'installer).

### Prima di vendere: parere dell'avvocato

Fargli valutare la tesi «i pesi sono coperti dalla MIT» e la formula delle eventuali donazioni (contributo volontario, senza corrispettivo).

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

E' **incluso nell'installer**, come programma separato (`ffmpeg.exe` su Windows, `ffmpeg` su macOS) accanto all'eseguibile di SpinLab, solo in versione **LGPL** (senza x264, x265, fdk-aac e altri componenti GPL o non liberi). Serve per le copie di lavoro dei video, le misure di prestazioni, la registrazione in MP4 e l'audio dei formati rari.

Cosa facciamo per rispettare la licenza (dettagli e procedura in `docs/FFMPEG.md`):

- il testo della licenza LGPL (preso dalla distribuzione ufficiale di FFmpeg) e il file con versione, origine e configurazione della build entrano nell'installer (`licenses/FFMPEG-LICENSE.txt`, `licenses/FFMPEG-BUILD.txt`);
- la scheda Informazioni e `THIRD-PARTY-LICENSES.md` dicono che SpinLab usa FFmpeg e da dove si ottengono i sorgenti;
- FFmpeg e' un file a parte, quindi l'utente puo' sostituirlo con una versione compatibile;
- l'EULA non vieta la modifica e il reverse engineering per i componenti con licenza libera (art. 4 e 6.3).

**Attenzione alla versione della LGPL.** Una build compilata con `--enable-version3` (come quella Windows di BtbN) e' **LGPL 3**, non 2.1: il testo da includere cambia (e la LGPL 3 richiama anche la GPL 3, per questo si includono tutti i testi in `licenses/ffmpeg/`) e valgono le condizioni sui brevetti della GPL 3 (art. 11, licenze brevettuali «discriminatorie»), da far verificare dal legale perche' possono interagire con le licenze dei codec (HEVC, H.264, AAC). Per questo la build **LGPL 2.1** (macOS, compilata da noi) e' preferibile; per Windows conviene compilarne una propria senza `--enable-version3` se il legale lo consiglia.

Le versioni «full» comuni (per esempio quelle di Gyan) sono GPL e **non** vanno incluse. **I brevetti dei codec (H.264, H.265, AAC) sono un tema separato dalla licenza del software**: restano da verificare con il parere legale (decisione D-10 in `docs/DECISIONI.md`).

## LAME / MP3

LAME e' sotto LGPL. Legarla dentro il programma lo avrebbe obbligato a permettere all'utente di ricollegarlo con un'altra versione. Ora e' un file a parte (`libmp3lame.dll`) costruito dai sorgenti ufficiali con `tools/build_lame_dll.py`: l'utente puo' sostituirlo. Testo della licenza in `licenses/LICENSE-LAME.txt`. I brevetti MP3 sono scaduti (2017).

## Da controllare comunque

- **Il nome «SpinLab»**: fare una ricerca di marchi (EUIPO / UIBM / USPTO) per software e musica prima di investire nel marchio.
- **«Camelot»** (la notazione delle tonalita'): e' di uso comune, ma verificare con chi ha registrato «Camelot Wheel» se si vuole usare il nome nel materiale commerciale. Si puo' evitare scrivendo solo i codici (8A, 9B...).
- **Marco Dufour** ha contribuito al codice (port per macOS): concordare per iscritto a chi appartiene il suo codice prima di vendere.
- **Condizioni d'uso, privacy e garanzia** (EULA): da scrivere; se il programma contatta un server per gli aggiornamenti, trattare i dati che questo comporta (indirizzo IP).
