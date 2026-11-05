# PRD — Modulo video di SpinLab DJ

**Versione:** 2.0 · **Data:** 2026-11-05 · **Stato:** bozza per revisione
**Generato con:** skill `g-p-generate-prd` (modalità: tutto in una volta), a partire dalla bozza precedente di questo documento e dalle decisioni prese durante lo sviluppo.
**Decisioni che spettano al responsabile del prodotto:** [DECISIONI.md](DECISIONI.md) (D-01…D-16)
**Documenti collegati:** [ASD](../.globant-skills-docs/asd/ASD.md) · [Documentazione tecnica](VIDEO.md) · [Manuale](MANUALE.md#video)

Nota sulle fonti: ogni sezione deriva da materiali forniti (la bozza precedente, le istruzioni e le decisioni del responsabile del prodotto durante lo sviluppo, il codice). Quello che non si può derivare è segnato `[PENDING INPUT]` e raccolto nel rapporto finale. **Stato dei requisiti** (colonna «Stato»): *Fatto* = realizzato nel branch `video-mixing` e provato su macOS; *Parziale* = realizzato in parte; *Da fare* = non ancora realizzato.

---

## 1. Executive Summary e problema

SpinLab DJ è un programma per DJ con due deck, mixer, effetti, stem, batteria, campionatore e registrazione. Il modulo video aggiunge la possibilità di caricare **video** sui deck e di mixarli come i brani, mostrandoli su uno schermo o un proiettore, con transizioni comandate dal crossfader, immagini animate a tempo per i brani senza video e la registrazione del set in MP4. Il video deve funzionare su Windows e macOS e adattarsi a computer di fascia diversa.

**Obiettivo dichiarato dal responsabile del prodotto:** poter mixare video come si mixa la musica, con un comportamento simile a quello di programmi come VirtualDJ e djay Pro, senza compromettere l'audio.

**Problema centrale:** `[PENDING INPUT]` — i materiali non descrivono il problema di business che il modulo video risolve per gli utenti né il motivo commerciale (per esempio differenziazione rispetto ad altri programmi o richieste dei clienti).

## 2. Ambito del progetto

### In ambito

- Caricare file video sui due deck, nella libreria e nelle playlist, con audio, griglia, BPM, tonalità e stem come per un brano.
- Uscita video su finestra separata o su schermo/proiettore esterno (HDMI, DisplayPort, USB-C), con scelta dello schermo e gestione del collegamento e scollegamento.
- Transizioni video comandate dal crossfader, con menu di scelta.
- Immagine per i deck solo audio: scheda del brano o visualizzazioni animate a tempo.
- Titolo e artista sullo schermo.
- Modalità VIDEO automatica con attivazione manuale.
- Profili di qualità, analisi automatica del computer, calibrazione e adattamento durante il set.
- Scratch e salti fluidi sui video (copia di lavoro temporanea).
- Registrazione del set in MP4.
- Windows e macOS.

### Fuori ambito (prima versione)

- Streaming di video da piattaforme online.
- Editing, titoli grafici, mappatura su superfici irregolari, controllo DMX o luci.
- Più di due deck con video.
- Android.
- Video 4K e HDR (l'uscita è limitata a 1080p; l'utenza prevista usa soprattutto video convertiti da YouTube).
- Animazioni ed effetti per singolo deck oltre alle transizioni e alle visualizzazioni elencate.

## 3. Utenti e personas

Fonte: bozza precedente del PRD; da confermare (`[PENDING INPUT]` per la validazione con utenti reali).

| Persona | Contesto | Obiettivo principale |
|---------|----------|---------------------|
| **DJ da locale o evento** (primaria) | Suona dal vivo con uno schermo o un proiettore | Proiettare video o immagini a tempo, con transizioni pulite, su un computer di fascia media |
| **DJ che prepara i set a casa** (secondaria) | Prepara in anticipo la musica e gli stem | Preparare stem e copie di lavoro prima, poi suonare senza attese |
| **DJ con computer modesto** (secondaria) | Portatile datato o con grafica integrata | Usare il video a qualità ridotta senza interruzioni dell'audio |

## 4. Requisiti funzionali (FR-XX)

| ID | Requisito | Stato |
|----|-----------|-------|
| FR-01 | Il sistema deve consentire di caricare file video (mp4, m4v, mov, mkv, webm, avi, ts, mpg, mpeg) sui deck, nella libreria e nelle playlist, e analizzarne l'audio (BPM, griglia, tonalità, forma d'onda) come per un brano. | Fatto |
| FR-02 | Il sistema deve mantenere il video allineato alla posizione audio del deck (play, pausa, cue, loop, salto, cambio di velocità) con uno sfasamento massimo di 40 ms, degradando il video e mai l'audio quando il computer fatica. | Fatto (sfasamento non misurato in modo sistematico) |
| FR-03 | Il sistema deve seguire lo scrub e i salti sul video in modo fluido, preparando in sottofondo una copia di lavoro temporanea con tutti i fotogrammi interi, cancellata quando il video lascia il deck e ripulita all'avvio. | Fatto |
| FR-04 | Il sistema deve permettere di creare gli stem di un video (menu del tasto destro su libreria e playlist) calcolati sull'audio e conservati per gli usi futuri; in tempo reale sui computer adeguati. | Fatto (usa il meccanismo degli stem dei brani) |
| FR-05 | Il sistema deve mostrare l'uscita video in una finestra a parte, a schermo intero sul monitor scelto dall'utente (ricordato), con ripiego in finestra e ripresa automatica quando lo schermo si scollega e ricollega. | Fatto (collegamento a caldo non provato su monitor reale) |
| FR-06 | Il sistema deve offrire un menu di transizioni video comandate dal crossfader (almeno 12, tutte originali), compresa una a tempo con la griglia dei battiti, e limitarle alle più leggere con il profilo di qualità Bassa. | Fatto (13 transizioni) |
| FR-07 | Il sistema deve mostrare, per i deck senza video, una scheda del brano (copertina, titolo, artista) o una di dieci visualizzazioni animate a tempo (battito della griglia, livello del deck, tonalità), a scelta dell'utente, partecipando alle transizioni. | Fatto |
| FR-08 | Il sistema deve mostrare titolo e artista del brano del lato dominante in basso a sinistra sopra i video e le visualizzazioni, con comparsa morbida al cambio di brano. | Fatto |
| FR-09 | Il sistema deve attivare la modalità VIDEO da solo quando si carica un video e spegnerla quando restano solo brani audio, con pulsante per forzarla; con la modalità spenta non deve usare risorse per il video. | Fatto |
| FR-10 | Il sistema deve analizzare il computer all'avvio (hardware, grafica, decodifica, codifica), scegliere un profilo di qualità (Bassa, Media, Alta, Massima), consentire di forzarlo, calibrarlo con un video in riproduzione e mostrare la qualità reale in uso. | Fatto (soglie provvisorie) |
| FR-11 | Il sistema deve adattare la qualità dell'uscita durante il set: scendere di livello se i fotogrammi al secondo calano e risalire quando il carico cala, avvisando l'utente. | Fatto |
| FR-12 | Il sistema deve registrare il set in MP4 (H.264 + AAC), con l'immagine dell'uscita video e l'audio del master sincronizzati (sfasamento massimo 40 ms) e scelta di risoluzione e fotogrammi al secondo; la registrazione audio WAV/MP3 deve restare invariata. | Fatto (sincronia da misurare con prove di lunga durata) |
| FR-13 | Il sistema deve riprodurre la massima varietà di formati e codec video (H.264, H.265, VP8/VP9, AV1, ProRes, DNxHD, MJPEG, HAP, immagini animate, canale alpha), con decodifica hardware quando disponibile e conversione automatica dei file troppo pesanti. | Parziale (H.264 provato; il resto dipende dal motore web del sistema) |
| FR-14 | Il sistema deve permettere di scegliere se la miscela video segue il crossfader (predefinito) o i fader del volume dei due canali (crossfader ignorato), per chi mixa con i canali. | Fatto |
| FR-15 | Il sistema deve conservare gli stem in una cartella centrale configurabile, abbinati ai file per impronta del contenuto, con un indice nel database (sapere quali brani hanno gli stem, lo spazio occupato, ripulire gli orfani). | Da fare (proposta non ancora approvata) |

## 5. Requisiti non funzionali

| Area | Requisito | Fonte |
|------|-----------|-------|
| Priorità dell'audio | L'audio non deve mai interrompersi per colpa del video; se il computer fatica si degrada il video | Principio guida del PRD precedente |
| Sincronia | Sfasamento audio-video ≤ 40 ms in riproduzione e in registrazione | PRD precedente |
| Scalabilità | Funzionare su configurazioni diverse con un profilo scelto in automatico in base all'hardware, modificabile dall'utente | Istruzioni del responsabile del prodotto |
| Risorse a riposo | Con la modalità VIDEO spenta nessun costo di CPU o memoria per il video | PRD precedente |
| Compatibilità | Windows e macOS con le stesse funzioni; Windows è la piattaforma di riferimento per le prove di prestazioni | Istruzioni del responsabile del prodotto |
| Prestazioni (obiettivo) | Base: 720p a 30 fps con stem preparati; medio: 1080p a 30 fps con transizioni; alto: 1080p a 60 fps con stem in tempo reale | PRD precedente; da confermare con prove su Windows |
| Sicurezza | La pagina non deve poter leggere file fuori dalla libreria e dalla cartella temporanea (oggi `asset://` ha ambito `**`) | Revisione tecnica (FC-01 dell'ASD) |
| Affidabilità | Nessun file temporaneo che resti dopo l'uso; recupero all'avvio dopo una chiusura anomala | Istruzioni del responsabile del prodotto (copia temporanea cancellata) |
| Accessibilità, localizzazione | `[PENDING INPUT]` | — |

## 6. Criteri di successo del business (KPI)

`[PENDING INPUT]` — I materiali non contengono metriche, valori obiettivo o tempi. Domande per la riunione successiva nel rapporto finale.

## 7. Jobs to be Done (JTBD-XX)

- **JTBD-01 (lavoro funzionale principale):** Quando suono un set in un locale con uno schermo o un proiettore, voglio che l'immagine segua la musica e il mio mixaggio, così il pubblico vive uno spettacolo audio-visivo coerente senza altri programmi.
- **JTBD-02:** Quando passo da un brano all'altro, voglio cambiare anche l'immagine con il crossfader, così le immagini cambiano con la stessa naturalezza dell'audio.
- **JTBD-03:** Quando suono brani senza video, voglio un'immagine bella sullo schermo (copertina e titolo o un'animazione), così lo schermo non resta mai vuoto.
- **JTBD-04:** Quando faccio scratch o salto in un video, voglio che l'immagine segua subito la testina, così un video si usa come un brano.
- **JTBD-05:** Quando uso un computer modesto o sotto carico, voglio che il programma adatti da solo la qualità dell'immagine, così l'audio non salta mai.
- **JTBD-06:** Quando finisco un set, voglio salvarlo come video con il suono, così posso conservarlo o condividerlo.
- **JTBD-07:** Quando preparo i set a casa, voglio preparare in anticipo stem e copie dei video, così in serata suono senza attese.
- **JTBD-08:** Quando carico un video scaricato da internet in un formato comune, voglio che funzioni subito, così non devo convertire i file a mano.
- **JTBD-09:** Quando collego o scollego uno schermo durante il set, voglio che lo spettacolo continui, così non si ferma nulla.

### Tracciabilità FR ↔ JTBD

**FR → JTBD**

| FR ID | Titolo | Soddisfatto da JTBD |
|-------|--------|---------------------|
| FR-01 | Video sui deck con analisi dell'audio | JTBD-01, JTBD-08 |
| FR-02 | Sincronia fra audio e video | JTBD-01, JTBD-04 |
| FR-03 | Scratch, salti e copia di lavoro | JTBD-04, JTBD-07 |
| FR-04 | Stem dei video | JTBD-01, JTBD-07 |
| FR-05 | Uscita su schermo esterno | JTBD-01, JTBD-09 |
| FR-06 | Transizioni video | JTBD-02 |
| FR-07 | Scheda e visualizzazioni per i brani senza video | JTBD-03 |
| FR-08 | Titolo e artista sullo schermo | JTBD-01, JTBD-03 |
| FR-09 | Modalità VIDEO automatica | JTBD-01, JTBD-05 |
| FR-10 | Analisi del computer e profili | JTBD-05 |
| FR-11 | Adattamento durante il set | JTBD-05 |
| FR-12 | Registrazione MP4 | JTBD-06 |
| FR-13 | Massima compatibilità dei formati | JTBD-08 |
| FR-14 | Miscela video che segue crossfader o fader dei canali | JTBD-02 |
| FR-15 | Cartella centrale degli stem | JTBD-07 |

**JTBD → FR**

| JTBD ID | Lavoro | Coperto da FR |
|---------|--------|---------------|
| JTBD-01 | Immagine che segue musica e mixaggio | FR-01, FR-02, FR-04, FR-05, FR-08, FR-09 |
| JTBD-02 | Transizione con il crossfader | FR-06, FR-14 |
| JTBD-03 | Schermo mai vuoto con brani senza video | FR-07, FR-08 |
| JTBD-04 | Scratch e salti sul video | FR-02, FR-03 |
| JTBD-05 | Qualità adattata al computer | FR-09, FR-10, FR-11 |
| JTBD-06 | Salvare il set come video | FR-12 |
| JTBD-07 | Preparare in anticipo | FR-03, FR-04, FR-15 |
| JTBD-08 | Video comuni che funzionano subito | FR-01, FR-13 |
| JTBD-09 | Lo spettacolo continua se lo schermo cambia | FR-05 |

**Verifica:** ogni FR ha almeno un JTBD e ogni JTBD ha almeno un FR; non ci sono lacune di tracciabilità. Le voci FR-14 e FR-15 sono comunque legate a un solo JTBD e vanno confermate (vedi rapporto sulle lacune).

## 8. Mappa delle dipendenze

**Esterne**

| Dipendenza | Uso | Note |
|------------|-----|------|
| FFmpeg (processo esterno) | Copie di lavoro, misure, codifica MP4, audio dei formati rari | Incluso nell'installer, build LGPL (D-09); vedi [FFMPEG.md](FFMPEG.md) |
| Motore web del sistema (WKWebView, WebView2) | Decodifica e disegno dei video (`<video>`, WebGL) | Codec leggibili diversi per piattaforma; Windows non provato |
| Encoder e decoder hardware (VideoToolbox, NVENC, Quick Sync, AMF, Media Foundation) | Prestazioni di codifica e decodifica | Opzionali, con ripiego software |
| Sistema operativo (monitor, finestre, file temporanei) | Uscita a schermo intero, ripiego | |
| Tauri 2 | Finestre, eventi, protocollo `asset://` | |

**Interne**

| Dipendenza | Uso |
|------------|-----|
| Motore audio (posizione, crossfader, livelli) | Il video segue l'audio; lo stato esposto è esteso con `crossfader` e `xf_assign` |
| Analisi e griglia dei battiti | Transizione a tempo e visualizzazioni |
| Stem | FR-04 |
| Registrazione audio | FR-12 (WAV temporaneo del master) |
| Libreria e database | I file video entrano come brani |

## 9. MVP, Nice-to-Have e Roadmap

**MVP (realizzato):** video sui deck con audio e griglia; sincronia; uscita su schermo con scelta; dissolvenza col crossfader (FR-01, FR-02, FR-05, FR-06 base).

**Nice-to-Have:** cartella centrale degli stem (FR-15); riproduzione con FFmpeg per la compatibilità totale (FR-13); video 4K e HDR (fuori ambito oggi).

**Roadmap per fasi**

| Fase | Contenuto | Stato |
|------|-----------|-------|
| 0 — Prototipo | Due video, dissolvenza, finestra di uscita, misure | Fatto |
| 1 — Base | FR-01, FR-02, FR-04, FR-05, FR-06 (base), FR-08, FR-09, FR-10, FR-11, titolo e schede | Fatto |
| 2 — Scratch e stem | FR-03 | Fatto; FR-15 da fare |
| 3 — Transizioni e visualizzazioni | FR-06 (13 transizioni), FR-07 | Fatto |
| 4 — Registrazione | FR-12 | Fatto |
| 5 — Consolidamento | Prove su Windows, taratura delle soglie, riproduzione con FFmpeg (FR-13), FR-15, restringere `asset://` | Da fare |

## 10. Vincoli normativi e obbligatori

| Tema | Vincolo | Stato |
|------|---------|-------|
| Diritti d'autore dei contenuti | I video musicali e i filmati sono protetti: proiezione, registrazione e pubblicazione sono responsabilità dell'utente; l'EULA lo dichiara (anche per i video) | EULA in bozza (`docs/legale/EULA.md`): voce sui video da aggiungere |
| Brevetti sui codec (H.264, H.265, AAC) | Distribuire decodifica e codifica può richiedere licenze o dichiarazioni | Parere legale prima della vendita |
| Licenza di FFmpeg | Si include solo una build **LGPL** (senza x264/x265), come programma separato, con licenza, avviso e sorgenti documentati ([FFMPEG.md](FFMPEG.md)); una build GPL o nonfree non è ammessa | Deciso (D-09); da provare su Windows |
| Privacy | Il video non invia dati all'esterno (tutto locale) | Informativa in bozza (`docs/legale/PRIVACY.md`): `[PENDING INPUT]` su altri obblighi |
| Standard o certificazioni di settore | `[PENDING INPUT]` | — |

## 11. Team suggerito o assegnato

I materiali non indicano persone. Ruoli necessari, derivati dall'ambito:

- Sviluppatore Rust (nucleo, FFmpeg, registrazione).
- Sviluppatore front-end / grafica (WebGL, shader, interfaccia).
- Esperto di audio e sincronia (latenza, griglia).
- QA su Windows e macOS, comprese prove su computer di fascia bassa.
- Responsabile legale / licenze (codec, FFmpeg, EULA).
- Responsabile del prodotto.

## 12. Glossario

| Termine | Definizione |
|---------|-------------|
| Adattamento | Riduzione o ripristino automatico della qualità dell'uscita in base al carico |
| Calibrazione | Prova sul campo, con un video in riproduzione, che trova il livello di qualità più alto che regge |
| Copia di lavoro | Copia temporanea a bassa risoluzione con tutti i fotogrammi interi, usata per scratch e salti |
| Crossfader | Cursore che passa dal deck sinistro al destro, comanda anche la miscela video |
| Deck | Giradischi virtuale su cui si carica un brano o un video |
| FFmpeg | Programma esterno per elaborare audio e video |
| Fotogramma intero | Fotogramma che si può mostrare senza dipendere dai precedenti |
| Griglia dei battiti | Posizioni dei battiti di un brano, usate per sincronia e transizioni a tempo |
| Profilo di qualità | Livello di risoluzione, fotogrammi al secondo e transizioni dell'uscita (Bassa, Media, Alta, Massima) |
| Scheda del brano | Immagine di ripiego con copertina, titolo e artista |
| Stem | Parte separata di un brano (voce, batteria, basso, altro) |
| Uscita video | Finestra, anche a schermo intero su un altro monitor, che mostra l'immagine |
| Visualizzazione | Animazione astratta a tempo di musica per i deck senza video |
| WebGL | Tecnologia del motore web per disegnare con la scheda grafica |

---

# Gap Management

## Informazioni mancanti (`[PENDING INPUT]`)

> Le decisioni corrispondenti, con opzioni e raccomandazioni, sono nel [registro delle decisioni](DECISIONI.md): §1 e §6 → D-01; §3 → D-01/D-02; §10 → D-09, D-10, D-11; FR-13 → D-08; FR-14 → D-06; FR-15 → D-07; sicurezza di `asset://` → D-12; prove su Windows → D-13.

| Campo | Perché serve all'ingegneria | Domanda per la prossima riunione |
|-------|-----------------------------|----------------------------------|
| §1 Problema centrale | Priorizzare e valutare il valore delle funzioni | Qual è il problema degli utenti che il video risolve? Perché adesso, e rispetto a quali alternative? |
| §3 Validazione delle personas | Le personas vengono dalla bozza, non da utenti reali | Ci sono utenti o clienti reali da intervistare? Quali profili hanno chiesto il video? |
| §5 Accessibilità e localizzazione | Impatto su interfaccia e testi | Servono altre lingue? Requisiti di accessibilità? |
| §6 KPI | Definire quando il modulo è «riuscito» | Quali metriche contano (utilizzo del video, conversioni, supporto)? Con quali valori e tempi? |
| §10 Standard e altri obblighi | Evitare rilavorazioni legali | Altri obblighi (privacy, licenze di settore)? Chi cura il parere sui codec? |

## Lacune di tracciabilità

Nessun FR o JTBD senza controparte. Da confermare: FR-14 (JTBD-02) e FR-15 (JTBD-07) sono soddisfatti da un solo lavoro e **non ancora approvati come requisiti** (FR-15 era una proposta; FR-14 è nel PRD precedente come opzione).

## Ordine del giorno della prossima riunione

1. Confermare o ridefinire il problema centrale e i KPI (sezioni 1 e 6).
2. Decidere FR-14 e FR-15: approvarli, rimandarli o toglierli.
3. Decidere la strada per FR-13: riproduzione con FFmpeg o conversione automatica in ingresso (domanda aperta del progetto).
4. Fissare le prove su Windows (macchina di fascia bassa e media) e i criteri di accettazione numerici (sincronia, fluidità).
5. Parere su codec e licenza di FFmpeg e aggiornamento dell'EULA.
6. Restringere l'ambito del protocollo `asset://` prima della vendita.

# Riepilogo e prossimo passo

- **Completezza:** 9 sezioni su 12 complete; la 1 (problema), la 6 (KPI) e la 10 (altri obblighi) hanno campi `[PENDING INPUT]`; la 3 e la 5 sono complete ma da validare.
- **Pronto per la scomposizione in EPIC?** Quasi: il perimetro funzionale è chiaro e in gran parte realizzato. Prima conviene chiudere i punti 1, 2 e 3 dell'ordine del giorno; poi si può usare `g-p-backlog-structure` per trasformare i requisiti rimasti (FR-13, FR-14, FR-15, consolidamento) in EPIC.
