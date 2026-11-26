# Registro delle decisioni — SpinLab DJ

Ultimo aggiornamento: 2026-11-05 · Responsabile delle decisioni: responsabile del prodotto (Vincenzo)

Questo documento raccoglie **le decisioni che spettano a te** e quelle già prese. Ogni decisione aperta ha: cosa si deve decidere, perché conta, le opzioni, la mia raccomandazione e cosa si blocca finché non è presa. Quando decidi, basta dirmi «D-xx: opzione B» e aggiorno questo registro, il PRD e l'ASD.

Collegamenti: [PRD del video](PRD-video.md) (sezione «Gap Management») · [ASD](../.globant-skills-docs/asd/ASD.md) (Open Items e Domande aperte) · [Documentazione tecnica](VIDEO.md).

**Come leggere le priorità**

- **A — Blocca la vendita:** va risolta prima di distribuire il programma a pagamento.
- **B — Blocca lo sviluppo o la documentazione:** senza una risposta non si può proseguire bene.
- **C — Può aspettare:** conviene deciderla, ma non blocca niente adesso.

---

## Decisioni aperte

### Riepilogo

| ID | Decisione | Priorità | Cosa si blocca | Stato |
|----|-----------|----------|----------------|-------|
| D-01 | Problema centrale e obiettivi misurabili (KPI) del modulo video | B | PRD §1 e §6; scelta delle priorità | Aperta |
| D-02 | Contesto di business: a chi si vende, come, quando | B | ASD §1.1; calendario delle decisioni legali | Aperta |
| D-03 | Ambito dell'ASD: solo l'app o anche server licenze e strumenti | B | Sezioni 6–9 dell'ASD | Aperta |
| D-04 | Piattaforme di destinazione (Windows, macOS, Android) | B | Prove, installer, ambito di molti requisiti | Aperta |
| D-05 | Principi architetturali del progetto | B | ASD §5 e §3 | Aperta |
| D-06 | Fader video separato dal crossfader (FR-14) | C | Niente di immediato | **Decisa**: scelta fra crossfader e fader dei canali |
| D-07 | Cartella centrale degli stem con indice nel database (FR-15) | B | Modifica a stem e database | Aperta |
| D-08 | Come ottenere la compatibilità completa dei formati (FR-13) | B | Promessa ai clienti sui formati | Aperta |
| D-09 | Quale FFmpeg distribuire (licenza) | A | Installer; dichiarazioni di licenza | **Decisa (provvisoria): build LGPL inclusa**; da provare su Windows |
| D-10 | Parere legale su codec (H.264, H.265, AAC) | A | Vendita | Aperta |
| D-11 | Testo dell'EULA sui contenuti video | A | Vendita | Aperta |
| D-12 | Restringere il permesso di lettura dei file (`asset://`) | A | Versione pubblica | Aperta |
| D-13 | Piano di prove su Windows e criteri di accettazione | B | Taratura delle soglie; affidabilità dichiarata | Aperta |
| D-14 | Quando e come unire `video-mixing` e `pulse-engine-beatgrid` a `main` | B | Rilascio; possibilità di tornare indietro | Aperta |
| D-15 | Streaming audio dalle piattaforme: avviare i contatti commerciali? | C | Nulla di tecnico | Aperta |
| D-16 | Piccole scelte di rifinitura (file di cache, formato predefinito della registrazione) | C | Niente | Aperta |
| D-17 | Licenze e modello di distribuzione | A | Distribuzione | **Decisa il 26/11/2026**: software libero GPL-3.0, gratuito, donazioni libere; aperti solo brevetti dei codec e pesi HT-Demucs |

---

### D-01 — Problema centrale e obiettivi misurabili (KPI)

- **Cosa decidere:** quale problema degli utenti risolve il video, perché adesso, e come si misura il successo.
- **Perché conta:** serve a decidere cosa vale la pena rifinire e cosa tagliare. Oggi il PRD lo segna come mancante (`[PENDING INPUT]` nelle sezioni 1 e 6).
- **Cosa mi serve da te:** una frase sul problema e 2–4 indicatori con un valore e una scadenza (per esempio: «il X% degli utenti attivi usa il video entro 3 mesi», «meno di N richieste di supporto sul video»).
- **Opzioni:** (a) me lo dici tu; (b) lo ricaviamo da interviste a utenti reali (vedi D-02); (c) restiamo senza KPI e valutiamo a occhio.
- **Raccomandazione:** (a) ora, anche provvisoria, e (b) quando ci sono utenti.
- **Impatto se si rimanda:** il PRD resta incompleto e non si può passare alla scomposizione in EPIC con `g-p-backlog-structure`.

### D-02 — Contesto di business

- **Cosa decidere:** chi sono gli utenti e gli altri interessati, come si vende (licenza una tantum o abbonamento, sito, negozio) e con quale calendario.
- **Perché conta:** il codice non lo dice e l'ASD §1.1 non si può scrivere. Il calendario di vendita decide quando devono essere chiuse le decisioni A.
- **Opzioni:** (a) dettarmi i punti; (b) compilare insieme `docs/legale/` e il modello di licenza (ci sono già bozze di EULA e informativa).
- **Raccomandazione:** (a), anche in forma di appunti.
- **Impatto se si rimanda:** le decisioni legali (D-09…D-12) restano senza una data.

### D-03 — Ambito dell'ASD

- **Cosa decidere:** l'ASD deve descrivere solo l'app desktop o anche il **server PHP** di licenze e aggiornamenti (`server/`) e gli **strumenti** Python (`tools/`)?
- **Opzioni:** (A) solo l'app; (B) app + server licenze; (C) tutto.
- **Raccomandazione:** (B). Il server è parte del prodotto venduto (licenze e aggiornamenti firmati); gli strumenti no.
- **Impatto se si rimanda:** non posso pianificare le sezioni 6–9 dell'ASD (infrastruttura, dati, DevOps).

### D-04 — Piattaforme di destinazione

- **Cosa decidere:** su quali piattaforme SpinLab è un prodotto da vendere: Windows (principale), macOS, Android?
- **Perché conta:** il codice ha rami per Android e interfacce per telefono, ma il video li esclude. macOS oggi è l'ambiente di sviluppo e di prova.
- **Opzioni:** (A) Windows + macOS; (B) solo Windows, macOS di sviluppo; (C) anche Android.
- **Raccomandazione:** (A), con Android fuori dalla prima versione. Il video su Android resta fuori ambito.
- **Impatto se si rimanda:** gli installer, le prove (D-13) e molti requisiti non hanno un obiettivo chiaro.

### D-05 — Principi architetturali

- **Cosa decidere:** i principi che il team segue e che ogni scelta tecnica deve rispettare.
- **Proposta di partenza (da confermare o correggere):** (1) l'audio non si interrompe mai; (2) tutto funziona offline salvo licenza, aggiornamenti e primo scarico del modello; (3) degrado elegante: se il computer fatica si riduce la qualità, non la continuità; (4) Windows e macOS con le stesse funzioni; (5) le dipendenze esterne (FFmpeg, motore web) non devono rompere l'app quando mancano.
- **Impatto se si rimanda:** ASD §5 resta vuota e le sezioni NFR (§3) senza riferimento.

### D-06 — Fader video separato dal crossfader (FR-14)

> **Decisa il 2026-11-05:** non un fader video a parte, ma una **scelta nelle impostazioni video**: la miscela segue **il crossfader** (predefinito) oppure **i fader del volume dei canali** (crossfader ignorato). Un «automatico» che combina le due cose è stato scartato perché poteva creare confusione. Realizzata in `video.js` (`videoMix`), Impostazioni → VIDEO → Miscela e clic destro sul pulsante VIDEO. Il testo sotto è l'analisi originale.

- **Cosa decidere:** vuoi poter assegnare la miscela video a un controllo a parte (cursore a schermo o MIDI), così l'audio resta su un deck mentre il video passa all'altro?
- **Opzioni:** (A) sì, nella prossima fase; (B) idea futura, resta nel PRD come «Nice-to-Have»; (C) tolto.
- **Raccomandazione:** (B). Oggi il crossfader basta; il costo è modesto ma l'utilità dipende dagli utenti (D-01).
- **Impatto se si rimanda:** nessuno.

### D-07 — Cartella centrale degli stem con indice nel database (FR-15)

- **Cosa decidere:** approvare, rimandare o togliere la proposta di una cartella unica (configurabile) per gli stem, con abbinamento per impronta del contenuto e una tabella nel database.
- **Perché conta:** oggi gli stem stanno in una cartella `stems` accanto a ogni brano e si perdono se il file viene rinominato o spostato. Occupano molto (circa 4× il brano).
- **Opzioni:** (A) approvata, con la cartella dati dell'app come predefinita e migrazione di quelli esistenti; (B) rimandata; (C) tolta (si resta con le cartelle accanto ai brani).
- **Raccomandazione:** (A). Risolve il problema dei brani spostati e permette di mostrare lo spazio occupato.
- **Impatto se si rimanda:** gli utenti rifanno gli stem dopo ogni spostamento di file.

### D-08 — Compatibilità completa dei formati (FR-13)

- **Cosa decidere:** come arrivare a leggere i formati oltre H.264 (H.265, VP9, AV1, ProRes…).
- **Stato:** oggi i video si riproducono nel motore web del sistema: H.264 funziona (provato), gli altri dipendono dal computer.
- **Opzioni:**
  - **A. Riprodurre con FFmpeg** invece del motore web: compatibilità massima, ma molto più lavoro e una finestra di uscita da rifare.
  - **B. Conversione automatica in ingresso:** se un file non si legge nel motore web, FFmpeg lo converte in sottofondo in un H.264 leggero e si riproduce quello. Meno lavoro, riusa la copia di lavoro già esistente.
  - **C. Lasciare com'è** e avvisare l'utente di convertire a mano.
- **Raccomandazione:** (B) ora, valutando (A) solo se serve davvero.
- **Impatto se si rimanda:** non si può promettere «legge tutti i formati» nelle comunicazioni di vendita.

### D-09 — Quale FFmpeg distribuire

> **Decisa il 2026-11-05 (provvisoria):** si include FFmpeg nell'installer come programma separato, **solo in versione LGPL**, con le licenze nei documenti. Realizzata in `tools/fetch_ffmpeg.py`, `tauri.release.conf.json`, EULA §4 e §6.3, scheda Informazioni e [FFMPEG.md](FFMPEG.md). Restano: la prova su Windows (D-13) e il parere sui brevetti (D-10). **Attenzione:** la build Windows di BtbN è **LGPL 3** (`--enable-version3`), quella macOS compilata da noi è LGPL 2.1; la LGPL 3 richiama le condizioni sui brevetti della GPL 3, da far verificare insieme alle licenze dei codec (D-10). Alternativa: compilare per Windows una build LGPL 2.1 propria. Il testo qui sotto è l'analisi che ha portato alla scelta.

- **Cosa decidere:** la build di FFmpeg da includere nell'installer.
- **Opzioni:** (A) **build LGPL** con gli encoder di sistema (VideoToolbox, NVENC, Quick Sync, AMF, Media Foundation): obblighi leggeri, ma senza x264 non c'è un encoder software garantito; (B) **build GPL con x264**: ripiego software sempre presente, ma obblighi più pesanti (licenza GPL, sorgenti, dichiarazioni in `THIRD-PARTY-LICENSES.md`); (C) non distribuire FFmpeg e chiedere all'utente di installarlo (come oggi nella documentazione).
- **Raccomandazione:** (A), dopo il parere legale (D-10). Su Windows Media Foundation offre un encoder anche senza scheda dedicata.
- **Impatto se si rimanda:** l'installer non si può chiudere; senza FFmpeg mancano copie di lavoro, misure e registrazione MP4.

### D-10 — Parere legale su codec e brevetti

> **Aggiornamento 2026-11-05:** hai richiesto una licenza per **HEVC** (da chiarire: copre solo la decodifica?). Restano da affrontare **H.264 e AAC**, usati dalla registrazione MP4. Registro completo in [LICENZE.md](LICENZE.md#registro-delle-licenze-e-dei-permessi-da-ottenere).

- **Cosa decidere:** chi prepara e quando il parere su H.264, H.265 e AAC (decodifica e codifica) per un prodotto a pagamento.
- **Opzioni:** (a) consulente legale esterno; (b) verifica interna con la documentazione dei fornitori; (c) rinuncia a H.265 e usa solo H.264 e AAC.
- **Raccomandazione:** (a) prima della vendita; nel frattempo evitare di promettere formati specifici.
- **Impatto se si rimanda:** rischio legale alla vendita.

### D-11 — Testo dell'EULA sui contenuti video

- **Cosa decidere:** approvare una clausola che dica che i video proiettati, registrati e pubblicati sono responsabilità dell'utente.
- **Stato:** l'EULA è una bozza «da far rivedere prima della vendita» (`docs/legale/EULA.md`); la voce sui video manca.
- **Opzioni:** (a) la preparo io come bozza da far rivedere; (b) la scrive il legale.
- **Raccomandazione:** (a) come punto di partenza per D-10.
- **Impatto se si rimanda:** l'EULA resta incompleta alla vendita.

### D-12 — Restringere il permesso di lettura dei file (`asset://`)

- **Cosa decidere:** dare il via a limitare il protocollo `asset://` alle cartelle della libreria e alla cartella temporanea. Oggi l'ambito è `**`: la pagina potrebbe leggere qualsiasi file.
- **Perché conta:** è un punto di sicurezza (FC-01 dell'ASD).
- **Opzioni:** (A) subito; (B) prima della versione pubblica; (C) lasciare così (sconsigliato).
- **Raccomandazione:** (B), cioè entro la prima distribuzione ad altri; non blocca l'uso personale.
- **Impatto se si rimanda:** rischio di sicurezza in distribuzione.

### D-13 — Prove su Windows e criteri di accettazione

- **Cosa decidere:** su quali computer provare (almeno uno di fascia bassa e uno media), chi le fa e con quali criteri numerici per dichiarare il video «pronto».
- **Proposta di criteri:** sfasamento audio-video ≤ 40 ms; nessuna interruzione dell'audio in 30 minuti con due video; registrazione MP4 di 30 minuti sincronizzata; adattamento che interviene senza interrompere l'audio.
- **Opzioni:** (a) tu hai o procuri i computer, io preparo la lista delle prove; (b) rimandiamo le prove e dichiariamo Windows «non verificato».
- **Raccomandazione:** (a). Windows è la piattaforma di riferimento e oggi non è provata.
- **Impatto se si rimanda:** le soglie dei profili restano ipotesi e non si può promettere nulla su Windows.

### D-14 — Unire i branch a `main`

- **Cosa decidere:** quando e in che ordine portare `pulse-engine-beatgrid` e `video-mixing` su `main`, e come mantenere la possibilità di tornare indietro.
- **Stato:** `video-mixing` contiene già tutto il lavoro di `pulse-engine-beatgrid` (23 commit in più di `main`). `main` è intatto.
- **Opzioni:** (A) una pull request unica da `video-mixing`, dopo le prove; (B) prima la sola griglia (`pulse-engine-beatgrid`), poi il video; (C) restare sui branch finché non c'è un rilascio.
- **Raccomandazione:** (B): la griglia è più piccola e già in uso; il video entra dopo le prove su Windows (D-13). Prima di unire, un tag su `main` per il ritorno.
- **Impatto se si rimanda:** i branch si allontanano da `main` e unirli diventa più rischioso.

### D-15 — Streaming audio dalle piattaforme

- **Cosa decidere:** avviare o no i contatti commerciali con Beatport/Beatsource, TIDAL e altri per un'integrazione di streaming.
- **Stato:** richiede accordi di partnership con le piattaforme; non è una questione tecnica.
- **Opzioni:** (a) avvia i contatti; (b) rimanda a dopo la prima vendita; (c) non fare streaming.
- **Raccomandazione:** (b), poi (a) se gli utenti lo chiedono.
- **Impatto se si rimanda:** nessuno sul codice.

### D-16 — Piccole scelte

- **Cartella `.comment-md-cache/`:** aggiungerla a `.gitignore` (cache dell'editor). Raccomandazione: sì.
- **Formato predefinito della registrazione:** WAV 16 bit (oggi) o MP4? Raccomandazione: lasciare WAV.
- **Qualità predefinita della registrazione MP4:** «come il profilo» (oggi) o 720p 30 fps fisso? Raccomandazione: come il profilo.

---

### D-17 — Licenze e permessi da ottenere, e modello di distribuzione

**Decisa il 26 novembre 2026** (vedi «Decisioni già prese»): SpinLab è **software libero (GPL-3.0-or-later) e gratuito, con donazioni libere**. Restano aperti solo il parere sui brevetti dei codec (D-10) e la licenza dei pesi di HT-Demucs (`docs/LICENZE.md`).

---

## Cose che puoi fare tu (non sono decisioni)

| # | Azione | Serve per |
|---|--------|-----------|
| 1 | Provare a mano la **registrazione MP4** per qualche minuto (RECORD → MP4) e dirmi se audio e immagine restano a tempo | Verificare FR-12 (oggi c'è solo un test automatico) |
| 2 | Provare un **monitor esterno**: scollegarlo e ricollegarlo a musica in corso | Verificare FR-05 |
| 3 | Provare su un **PC Windows** (anche solo di fascia media) e mandarmi cosa scrive Impostazioni → VIDEO | D-13; taratura delle soglie |
| 4 | Continuare a usare l'**anchor** sui brani in cui la griglia sbaglia; con circa dieci esempi di generi diversi posso cercare una regola | Migliorare la fase della griglia |
| 5 | Rispondere alle domande D-01…D-05 anche in modo breve | Completare PRD e ASD |

---

## Decisioni già prese

| Data | Decisione | Motivo | Dove è applicata |
|------|-----------|--------|------------------|
| 2026-11-26 | **Software libero (GPL-3.0-or-later) e gratuito, con donazioni libere via PayPal**; tolti registrazione, server delle licenze e aggiornamenti automatici; versioni pubblicate come Release su GitHub | La griglia deriva da openDJ (GPL-2.0-or-later); scelta del responsabile del prodotto | `LICENSE`, `NOTICE`, finestra Informazioni, `docs/legale/` |
| 2026-11 | Sviluppo del video su un branch separato (`video-mixing`) con `main` intatto | Poter sempre tornare indietro | Flusso di lavoro |
| 2026-11 | **Solo 2 deck** per il video | Semplicità; il motore ne ha 4 ma il video no | PRD §2, FR-01 |
| 2026-11 | **Modalità VIDEO automatica**, con pulsante per forzarla | Nessun costo senza video | FR-09 |
| 2026-11 | **Composizione con WebGL** nella finestra di uscita (opzione A), compositore isolato | Rapidità e portabilità; sostituibile con uno nativo | `video-gl.js`, ASD §6.2 |
| 2026-11 | **Niente 4K né HDR** | Gli utenti useranno soprattutto video convertiti da YouTube | PRD §2 |
| 2026-11 | **Transizioni sul crossfader**, con scelta alternativa dei **fader dei canali** (niente modalità automatica) | Chi mixa con i canali lascia il crossfader fermo; l'automatico poteva creare confusione | FR-06, FR-14 |
| 2026-11 | **Scheda del brano o 10 visualizzazioni** per i deck senza video; titolo e artista sempre in basso a sinistra | Schermo mai vuoto | FR-07, FR-08 |
| 2026-11 | **Copia di lavoro temporanea** per scratch e salti, cancellata quando il video lascia il deck | Niente spazio occupato; stem invece persistenti | FR-03 |
| 2026-11 | **Configurazione automatica per hardware** con profili, calibrazione e adattamento; pannello in **Impostazioni → VIDEO** | Scalabilità su computer diversi | FR-10, FR-11 |
| 2026-11 | **Registrazione MP4** con audio WAV temporaneo e unione finale | L'audio non passa mai da FFmpeg in tempo reale | FR-12 |
| 2026-11-05 | **FFmpeg incluso nell'installer, solo build LGPL** (sidecar), con licenza, avviso e sorgenti documentati | Il video dipende da FFmpeg; la LGPL ha obblighi leggeri e non obbliga a pubblicare il codice | `docs/FFMPEG.md`, `tools/fetch_ffmpeg.py` |
| 2026-11 | Uso delle **skill Globant** per la documentazione | Documentazione chiara e completa | `.globant-skills-docs/`, PRD v2.0 |
| 2026-11 | Commit solo su richiesta, sempre su `video-mixing` | Controllo del lavoro | Flusso di lavoro |
