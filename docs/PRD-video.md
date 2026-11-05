# PRD — Modulo video di SpinLab DJ

Stato: bozza per revisione · Piattaforme: **Windows (primaria) e macOS** · Ultimo aggiornamento: 2026-11-05

## 1. Obiettivo

Permettere di caricare file video sui deck e mixarli come i brani: stessa griglia, stessi stem, stesso scratch, con transizioni video comandate dal crossfader e uscita su uno schermo/proiettore a parte. Il set (audio + video mixato) si può registrare in MP4.

Principio guida: **l'audio resta il master**. Il video segue sempre l'audio e non lo rallenta mai. Se il computer fatica, si degrada il video, mai l'audio.

### Fuori ambito (prima versione)
- Streaming di video da piattaforme online.
- Editing, titoli, mappatura su superfici irregolari (video mapping), controllo DMX/luci.
- Effetti video per singolo deck oltre a quelli elencati nelle transizioni.
- Android.

## 2. Utenti

| Utente | Bisogno |
|---|---|
| DJ da locale/evento | Proiettare clip o video musicali a tempo, con transizioni pulite, su PC di fascia media |
| DJ che prepara i set a casa | Preparare in anticipo stem e copie di lavoro, poi suonare senza attese |
| DJ con PC modesto | Usare il video a bassa qualità senza interruzioni audio |

## 3. Cosa esiste già (verificato nel codice)

- 4 deck nel motore (`NUM_DECKS`) e analisi di BPM, griglia, tonalità e forma d'onda.
- Decodifica audio tramite **FFmpeg a riga di comando**, cercato accanto all'eseguibile (`ffmpeg.exe` come sidecar dell'installer) o nel PATH ([decode.rs](../src-tauri/src/decode.rs)).
- Stem: tempo reale o preparati in anticipo, con misura della velocità del PC; file salvati in una cartella `stems` accanto al brano ([stems.rs](../src-tauri/src/stems.rs)).
- Registrazione del master in WAV/MP3 ([recorder.rs](../src-tauri/src/recorder.rs)).
- Database SQLite `library.db` con libreria, playlist, griglie e analisi ([library.rs](../src-tauri/src/library.rs)). Gli stem non vi sono registrati.
- Interfaccia web in Tauri 2 (WebView2 su Windows, WKWebView su macOS).

## 4. Requisiti funzionali

Priorità: **P0** necessario per la versione 1, **P1** subito dopo, **P2** rimandabile.

### 4.1 Caricamento e libreria
- **V-1 (P0)** I formati video entrano nella libreria e nelle playlist, riconosciuti dall'estensione (mp4, mov, mkv, webm, avi, m4v). FFmpeg decide se il contenuto è leggibile; i file non supportati mostrano un messaggio chiaro.
- **V-2 (P0)** Un video si carica su un deck come un brano. L'audio del video alimenta analisi, griglia, BPM, tonalità e waveform con la pipeline esistente, senza codice nuovo.
- **V-3 (P1)** Icona/miniatura distinta per i video nella libreria (primo fotogramma o fotogramma a metà).
- **V-4 (P1)** Video senza audio: il deck è caricabile solo con BPM e griglia manuali (inseriti o tap tempo).

### 4.2 Riproduzione e sincronia
- **V-5 (P0)** Il video è agganciato alla posizione audio del deck: play, pausa, cue, loop, salto e cambio di velocità (pitch) si riflettono sul video. Lo sfasamento A/V massimo tollerato è **40 ms**.
- **V-6 (P0)** Se il decoder non regge, salta fotogrammi invece di rallentare l'audio.
- **V-7 (P1)** Scratch e jog: il video segue la testina avanti e indietro (vedi 5.2).

### 4.3 Stem
- **V-8 (P0)** Gli stem di un video si creano e si leggono come per i brani, dalla stessa voce di menu «Crea stem» (libreria e playlist, tasto destro). Si calcolano sull'audio del video.
- **V-9 (P0)** I PC adeguati separano gli stem in tempo reale mentre il video suona; gli altri usano solo gli stem preparati. Stessa logica e stessa misura di velocità di oggi.
- **V-10 (P0)** Gli stem preparati restano salvati per usi futuri (a differenza della copia di lavoro del video).

### 4.4 Transizioni e uscita
- **V-11 (P0)** Una finestra di uscita video, a schermo intero su qualsiasi monitor/proiettore selezionato, separata dall'interfaccia.
- **V-12 (P0)** Il crossfader audio comanda anche la miscela video fra i due deck assegnati, con almeno: dissolvenza, taglio, dissolvenza al nero.
- **V-13 (P1)** Libreria di transizioni (wipe, zoom, glitch, pixel) con scelta dell'utente e durata; intensità legata alla posizione del fader.
- **V-14 (P1)** Anteprima per ogni deck e per l'uscita nell'interfaccia, in bassa risoluzione.
- **V-15 (P1)** Con più di due video attivi (4 deck) l'uscita usa i due deck assegnati al crossfader; gli altri restano nell'anteprima.
- **V-16 (P2)** Logo/immagine di sfondo quando non c'è un video attivo.

### 4.4 bis Uscita su schermo esterno, brani senza video e modalità VIDEO
- **V-24 (P0)** **Interruttore «VIDEO» nell'intestazione.** Spento (predefinito): nessun codice video attivo, nessuna finestra di uscita, nessun costo di CPU/RAM; l'app si comporta come oggi. Acceso: si abilitano i video sui deck, la finestra di uscita e le transizioni. L'impostazione si ricorda fra le sessioni.
- **V-25 (P0)** **Scelta dello schermo di uscita**: elenco dei monitor rilevati (HDMI, DisplayPort, USB-C, proiettore), a schermo intero senza bordi sul monitor scelto, con frequenza di aggiornamento nativa. La scelta si ricorda.
- **V-26 (P0)** **Scollegamento/collegamento a caldo**: se lo schermo esterno si scollega, l'uscita passa a una finestra di anteprima senza fermare audio né video; se si ricollega, torna a schermo intero. Se non c'è uno schermo esterno, l'uscita resta in una finestra.
- **V-27 (P0)** **Brano audio mescolato a un video (o a un altro audio).** Un deck con solo audio produce un'immagine di ripiego, scelta nelle impostazioni, e partecipa alle transizioni come qualunque altro deck: (a) **titolo, artista e copertina** dai tag, (b) **visualizzazione generativa** (vedi V-28), (c) **logo/immagine dell'utente**, (d) **nero**. Predefinito: (a).
- **V-28 (P1)** **Visualizzazioni generative**: circa **dieci animazioni vettoriali/shader originali**, incluse nell'app, tra cui l'utente sceglie (anteprima nelle impostazioni, cambio anche durante il set). Si muovono con il suono: le pulsazioni seguono la **griglia dei battiti**, l'intensità segue l'energia dello stem della batteria e il colore la tonalità o il genere. Funzionano anche con VIDEO acceso e **nessun video caricato**, per chi ha solo il proiettore. Devono essere tutte opera originale (nessun materiale di terzi).
- **V-29 (P1)** **Solo audio con proiettore collegato.** Con VIDEO acceso e nessun video, l'uscita mostra la visualizzazione scelta a pieno schermo. Con VIDEO spento il proiettore non viene toccato. Il programma non accende mai l'uscita da solo: quando rileva un secondo schermo propone una volta sola di usarlo.
- **V-30 (P1)** **Animazioni dell'utente**: oltre alle dieci incluse, l'utente può caricare i propri video o animazioni in loop e usarli come immagine di ripiego o come sfondo per le visualizzazioni. Valgono le stesse regole di compatibilità di V-31.
- **V-31 (P0)** **Compatibilità massima con i formati video.** Si accetta tutto ciò che FFmpeg sa decodificare. Obiettivo minimo, con esito verificato su Windows e macOS: H.264, H.265/HEVC, VP8, VP9, AV1, MPEG-2, MPEG-4, ProRes, DNxHD, MJPEG, HAP, in contenitori mp4, mov, mkv, webm, avi, m4v, ts; GIF/WebP/APNG animati e sequenze di immagini; video con **canale alpha** (ProRes 4444, VP9/WebM con trasparenza) per sovrapposizioni.
  - Decodifica hardware quando il codec è supportato dal sistema, ripiego software in tutti gli altri casi: un file non è mai rifiutato solo perché manca l'accelerazione.
  - **Controllo al caricamento**: se un file non si legge o è troppo pesante per il PC (es. 4K HEVC su una macchina debole), il programma lo dice e propone la conversione automatica in un formato leggero e compatibile, in sottofondo, senza toccare l'originale.
  - Frame rate variabili e diversi da quello di uscita (23,976 / 25 / 29,97 / 50 / 60) gestiti senza strappi.

### 4.5 Registrazione
- **V-17 (P1)** Registrare il set in **MP4 (H.264 + AAC)**: video miscelato (come sull'uscita) più audio del master, senza sfasamenti superiori a 40 ms per tutta la durata.
- **V-18 (P1)** Scelta di risoluzione e frame rate di registrazione, indipendenti dall'uscita.
- **V-19 (P0)** La registrazione audio WAV/MP3 resta disponibile e non cambia.

### 4.6 Impostazioni e prestazioni
- **V-20 (P0)** Profili qualità (Bassa 480p/24 fps · Media 720p/30 · Alta 1080p/30 · Massima 1080p/60 o 4K) scelti dall'utente.
- **V-21 (P1)** Al primo avvio il programma propone un profilo in base a CPU, GPU, RAM e presenza di decodifica/codifica hardware.
- **V-22 (P0)** Interruttore per disattivare del tutto il video: chi non lo usa non paga nulla in CPU/RAM.
- **V-23 (P0)** Degrado automatico: se il carico video supera la soglia, il programma abbassa prima il frame rate di uscita, poi la risoluzione, e avvisa l'utente. L'audio ha sempre priorità.

## 5. Scelte tecniche

### 5.1 Decodifica
- FFmpeg resta l'unico decoder (già presente e distribuito). Per la riproduzione si usa la **decodifica hardware** quando c'è: D3D11VA / DXVA2 su Windows, VideoToolbox su macOS; software come ripiego.
- Nota: finora FFmpeg è usato come processo esterno. Per i video serve un flusso continuo di fotogrammi sincronizzato con l'audio: decisione da prendere (domanda aperta 1) tra processo esterno con pipe e collegamento diretto alla libreria FFmpeg.

### 5.2 Scratch: copia di lavoro e cache
Il problema: nei file comuni (H.264/VP9/AV1) un fotogramma intero compare ogni 2-5 s, quindi muoversi rapidamente avanti e indietro richiede di ridecodificare dall'ultimo.

Soluzione a tre livelli, dal più semplice:
1. **Cache di fotogrammi decodificati** attorno alla testina: scratch istantaneo vicino al punto, breve ritardo sui salti lunghi.
2. **Copia di lavoro temporanea**: al caricamento si crea in sottofondo una copia 480p con tutti i fotogrammi interi (MJPEG, funziona identico su Windows e macOS con FFmpeg standard). Il video parte subito dall'originale e passa alla copia quando è pronta. Quando nessun deck usa più il video, la copia viene **cancellata**. Se due deck caricano lo stesso video, la copia è una sola.
3. All'avvio il programma svuota la cartella delle copie temporanee rimaste da una chiusura anomala.

Misure su un video di prova (720p H.264, 20 s, Mac): copia MJPEG 0,4 s e 10,5 MB (circa 32 MB al minuto), circa 50 volte più veloce del video. Da verificare su Windows e su sorgenti 1080p/4K/VP9/AV1, dove i tempi possono essere 2-5 volte superiori.

### 5.3 Composizione e uscita
- Composizione su GPU (shader): lato interfaccia con WebGL nella finestra di uscita (più rapido da realizzare) oppure finestra grafica nativa in Rust (più robusto con video pesanti). **Da decidere con un prototipo** (domanda aperta 2).
- La finestra di uscita è una seconda finestra Tauri a schermo intero sul monitor scelto; la selezione del monitor funziona su entrambe le piattaforme.

### 5.4 Registrazione MP4
- Codifica con encoder hardware dove disponibili: **Media Foundation / NVENC / Quick Sync / AMF su Windows**, VideoToolbox su macOS; ripiego software solo se la licenza lo consente (vedi 8).
- Audio dal master già esistente, video dalla composizione; i due flussi sono uniti con timestamp comuni dal clock audio.

### 5.5 Stem e database
- **Cartella centrale degli stem**, configurabile (predefinita: cartella dati dell'app, non la directory del programma, perché su Windows `Program Files` non è scrivibile e un aggiornamento la cancellerebbe).
- Abbinamento brano/video ↔ stem per **impronta del contenuto** (dimensione più alcuni blocchi del file), non per percorso: sopravvive a rinomina e spostamento. Nome leggibile: `nomefile.<impronta>.stems`.
- Nuova tabella `stems` nel database esistente (impronta, percorso, dimensione, ultimo uso, tipo audio/video): sapere subito quali brani hanno gli stem, mostrare lo spazio occupato, ripulire i più vecchi e gli orfani. Un pulsante sposta gli stem già preparati nella cartella centrale; la ricerca prova comunque la cartella accanto al file e la vecchia cache.
- Nessuna dicitura «vd» obbligatoria: il nome contiene già l'estensione. Si può aggiungere come marcatore opzionale.
- Nota collegata: `prune_orphans` toglie dalla libreria i brani senza playlist ma non pulisce gli stem; l'indice li renderebbe individuabili.

## 6. Parità Windows / macOS

| Area | Windows | macOS |
|---|---|---|
| Decodifica | D3D11VA/DXVA2, ripiego software | VideoToolbox, ripiego software |
| Codifica MP4 | Media Foundation, NVENC, Quick Sync, AMF | VideoToolbox |
| Copia di lavoro (MJPEG) | FFmpeg software, identico | FFmpeg software, identico |
| Interfaccia / WebGL | WebView2 | WKWebView |
| Finestra di uscita | Seconda finestra Tauri, scelta monitor | Idem |
| FFmpeg | `ffmpeg.exe` come sidecar nell'installer | Sidecar o PATH |
| Stem | Cartella dati dell'app (es. `%APPDATA%`) | `~/Library/Application Support` |

Tutte le funzioni di questo documento devono funzionare su entrambe. La piattaforma di riferimento per i test di prestazioni è **Windows**.

## 7. Prestazioni (obiettivi, da confermare con un prototipo)

| Profilo | Hardware indicativo | Obiettivo |
|---|---|---|
| Base | Portatile di qualche anno, 8 GB, grafica integrata | 720p/30, stem preparati, registrazione solo audio |
| Medio | Ultimi 5 anni, 16 GB | 1080p/30, transizioni, registrazione MP4 con encoder hardware |
| Alto | GPU dedicata / Apple Silicon recente | 1080p/60 o 4K, stem in tempo reale |

I requisiti minimi ufficiali si scrivono solo dopo aver misurato la fase 1 su almeno un PC Windows di fascia bassa e uno di fascia media.

## 8. Aspetti legali

- **Diritti dei contenuti:** l'uso, la registrazione e la pubblicazione di video protetti è responsabilità dell'utente. L'EULA ([docs/legale/EULA.md](legale/EULA.md)) deve dirlo in modo esplicito e includere i video.
- **Codec:** l'encoder H.264 software più diffuso (x264) è GPL: se usato va dichiarato in [THIRD-PARTY-LICENSES.md](../THIRD-PARTY-LICENSES.md) e comporta obblighi. Gli encoder hardware del sistema evitano il problema. La compatibilità di distribuzione di H.264/AAC va verificata con un consulente prima della vendita.
- **FFmpeg distribuito:** scegliere una build LGPL e verificare che l'installer rispetti gli obblighi di licenza. La compatibilità massima (V-31) include codec con brevetti (H.264, H.265): decodificarli in un prodotto venduto può richiedere licenze o dichiarazioni, da verificare con un consulente prima del lancio.

## 9. Fasi e criteri di accettazione

| Fase | Contenuto | Si considera finita quando |
|---|---|---|
| 0 — Prototipo | Un video su due deck, dissolvenza, finestra di uscita; misure su un PC Windows di fascia bassa e uno medio | Risolte le domande aperte 1 e 2; misure scritte |
| 1 — Base | V-1, V-2, V-5, V-6, V-8–V-12, V-19–V-27, V-31 | Due video 720p suonano per 30 min senza interruzioni audio; sfasamento A/V < 40 ms |
| 2 — Scratch e stem | V-7, cache e copia di lavoro, cartella centrale degli stem, tabella nel database | Scratch fluido su un video H.264 1080p; stem ritrovati dopo aver rinominato e spostato il file |
| 3 — Transizioni e visualizzazioni | V-13–V-16, V-28–V-30 | Almeno 6 transizioni comandate dal fader, 10 visualizzazioni selezionabili, caricamento di animazioni dell'utente |
| 4 — Registrazione | V-17, V-18 | MP4 di 30 min riproducibile, audio e video sincronizzati |

## 10. Rischi

| Rischio | Impatto | Mitigazione |
|---|---|---|
| Sincronia A/V su PC lenti | Alto | Audio come master, salto dei fotogrammi, degrado automatico |
| Scratch lento su video comuni | Medio | Cache e copia di lavoro |
| Registrazione MP4 pesante | Alto | Encoder hardware, profili, registrazione a risoluzione indipendente |
| Spazio disco (stem, copie) | Medio | Copie temporanee, cartella centrale con pulizia, indicatore dello spazio |
| Licenze codec | Alto per la vendita | Encoder di sistema, parere legale prima del lancio |
| Concorrenza (djay Pro, Resolume, VirtualDJ) | Medio | Puntare su semplicità e su stem + griglia già presenti |

## 11. Domande aperte

1. FFmpeg come processo esterno con pipe, o collegamento diretto alla libreria (più lavoro, più controllo e prestazioni)?
2. Composizione: WebGL nella finestra di uscita o finestra grafica nativa in Rust?
3. Il numero di deck con video è 2 o fino a 4?
4. Le transizioni video seguono sempre il crossfader audio o si possono assegnare a un controllo separato?
5. Quali encoder hardware garantire su Windows come requisito minimo (Media Foundation come base)?
6. Dove deve stare la cartella centrale degli stem per chi ha più dischi?
7. L'MP4 registrato deve includere la traccia degli stem separati o solo il master?
