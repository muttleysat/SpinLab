# SpinLab DJ — Manuale

![ASIO Compatible - ASIO is a registered trademark of Steinberg Media Technologies GmbH](../ui/assets/asio-logo-bw.png)

SpinLab DJ è un programma per DJ con due deck, mixer, effetti, separazione in stem, batteria, campionatore, uscita video per schermi e proiettori, registrazione del master e supporto per qualsiasi controller MIDI. Questo manuale spiega come usarlo, come collegare un controller e come risolvere i problemi più comuni.

> I tasti e i comandi sono descritti come compaiono sullo schermo. Passando il mouse sopra quasi ogni pulsante compare un suggerimento.

## Primo avvio

All'avvio compare la schermata di SpinLab con una barra di avanzamento: sparisce da sola quando il motore audio e la libreria sono pronti.

1. **Scegli la scheda audio.** Apri **SETTINGS** (o l'ingranaggio in alto a destra) → scheda **AUDIO**. Se hai una scheda con driver ASIO (per esempio Focusrite) scegli driver **ASIO®**: la latenza è molto più bassa e i pad della batteria rispondono subito. Altrimenti lascia **Windows (WASAPI)**. Premi **Applica**.
2. **Scegli le cuffie.** Nello stesso pannello, «Uscita cuffie». Vedi [Impostazioni audio](#impostazioni-audio).
3. **Aggiungi la musica.** Nella colonna **ESPLORA** a sinistra scegli una cartella: i brani compaiono nella tabella. Vengono analizzati (BPM, tonalità, griglia dei battiti) in sottofondo.
4. **Carica un brano.** Seleziona un brano e premi **LOAD** nella sua riga, oppure trascinalo su un deck.

---

## La schermata

- **Barra in alto.** Navigazione (**LIBRARY**, **MIXER**, **RECORD**, **SETTINGS**), indicatore **REC** quando registri, carico del processore (**CPU**), orologio, comandi delle cuffie (pulsanti 1 e 2, **CUE MIX**, **PHONES**) e volume **MASTER**. A destra del volume ci sono la **i** (informazioni su SpinLab, versione e crediti) e il **?** (questa guida).
- **Centro.** I due deck con le waveform e, in mezzo, il mixer a due canali con il crossfader.
- **In basso a sinistra.** Esplora cartelle e playlist.
- **In basso al centro.** La libreria: ricerca, tabella dei brani, scheda **TAG & INFO**.
- **In basso a destra.** Il pannello con **AUTOMIX**, **SAMPLER**, **DRUMS**, **STEMS** e, sotto, gli **FX**.

La grafica si adatta alla finestra: se la riduci molto, si rimpicciolisce in blocco. Su un **monitor ultrawide** (dal formato 2,4:1 in su, per esempio 32:9) la disposizione cambia da sola: vedi [Aspetto: skin, colori e monitor ultrawide](#aspetto-skin-colori-e-monitor-ultrawide).

**Scorciatoie**

| Tasto | Cosa fa |
|---|---|
| Barra spaziatrice | Play / pausa del deck sotto il mouse (altrimenti dell'ultimo usato) |
| Freccia su / giù | Sfoglia i brani della libreria |
| Invio | Carica il brano selezionato sul deck in uso |
| Canc | Toglie i brani selezionati dalla playlist aperta |
| Z X C V · A S D F · Q W E R · 1 2 3 4 | Pad della batteria (con la scheda DRUMS aperta) |
| Maiusc + clic su un pad della batteria | Colpo più leggero |

---

## Libreria e playlist

- **Esplora cartelle.** Scegli una cartella: SpinLab ne legge i brani e li analizza. La sottocartella `stems` (vedi [Stem](#stem)) viene ignorata.
- **Ricerca.** Il campo in alto cerca per titolo, artista, tag, tonalità e BPM. Le colonne si ordinano con un clic sull'intestazione.
- **Anteprima in cuffia.** Il pulsante a forma di triangolo accanto al brano lo ascolta nelle cuffie senza passare dal master. Maiusc + clic parte da metà brano. Clic o trascinamento sulla riga per spostarsi.
- **Playlist.** **+** accanto a PLAYLIST ne crea una. Trascina i brani su una playlist per aggiungerli. Tasto destro su una playlist: rinomina, elimina, **Crea stem di tutta la playlist**. Aggiungere un brano a una playlist ne avvia l'analisi completa.
- **TAG & INFO.** Il pulsante con la «i» accanto alla ricerca apre il pannello per titolo, artista, album, genere, tag, valutazione a stelle e commento.
- **Doppio clic su un brano** lo carica sul primo deck libero.
- **Vista LIBRARY.** Con **LIBRARY** nella barra in alto la libreria prende più spazio. Nel pannello di destra le sezioni (stem, sampler, drum, automix) restano della loro misura e sotto compare il **riquadro del brano selezionato**: copertina, titolo, artista, BPM, tonalità, durata, album, genere, tag, valutazione, se è analizzato, formato, commento e percorso del file. Con più brani selezionati ne indica il numero.
- **Tasto destro su un brano.** Carica sul deck 1 o 2, aggiungi a una playlist (esistente o nuova con la selezione), analizza (BPM, tonalità, griglia), crea o elimina gli stem preparati, rimuovi dalla playlist (se ne hai una aperta).

---

## I deck

Ogni deck mostra due waveform: quella grande (zoom, scorre) e quella piccola (panoramica dell'intero brano), il piatto, i pulsanti e il fader del pitch.

- **Il conto alla rovescia** in alto a destra (il tempo che manca alla fine) **lampeggia in rosso negli ultimi 20 secondi** del brano, per avvisare che sta per finire. Con «Riduci movimento» attivo nel sistema resta rosso fisso.
- **Il piatto** si illumina secondo lo stato del deck: tenue senza brano, fermo in pausa, con la luce che corre insieme al disco mentre suona, più acceso durante lo scratch e color ambra quando tieni premuto CUE mentre il brano suona. Nella skin Dark Club il braccio del giradischi appoggia la puntina sul disco quando carichi un brano e avanza verso il centro mentre il brano procede.

### Trasporto

- **PLAY** e **CUE**. CUE tenuto premuto fa partire il brano dal punto di cue finché non rilasci.
- **LOAD** carica il brano selezionato nella libreria; il tasto accanto svuota il deck.
- **Il pitch** si regola con il fader a destra. **±8** (o il valore indicato) cambia l'escursione: 4, 8, 16, 50, 100%. Doppio clic sul fader lo riporta a zero di colpo.
- **→0** (sotto il valore del pitch) lo riporta a zero **gradualmente**, senza scatti udibili: il pitch si avvicina a 0 a velocità costante di 0,5 punti percentuali al secondo (da +8% servono 16 secondi, da +2% quattro). Il pulsante si accende mentre il rientro è in corso e un secondo clic lo ferma dove si trova. Muovere il fader a mano, caricare un brano o accendere il SYNC lo interrompe; se il deck stava seguendo il SYNC, il SYNC si spegne, come quando si muove il pitch a mano. Il pulsante è spento quando il pitch è già a 0.
- **SYNC** allinea tempo e fase al deck che fa da guida. Resta attivo anche dopo aver caricato un altro brano.

### Scrub

Tieni premuto il mouse sulla waveform grande e trascina: il brano segue come un disco (un trascinamento a sinistra avanza). Anche il piatto si può girare con il mouse. Sulla waveform piccola un clic porta la testina in quel punto.

### Griglia e tonalità

La griglia dei battiti viene calcolata dall'analisi e, quando possibile, affinata da una rete neurale (Beat This!) che riconosce i battiti anche nei brani suonati dal vivo.

- **⚓ (ancora)**, sul deck sopra **GRID**: porta esattamente sulla testina il marker più vicino. Serve per correggere un punto dove la griglia si è spostata; la correzione si attenua verso le ancore vicine, quindi se ne possono mettere più d'una nello stesso brano.
- **GRID** apre la finestra della griglia, salvata per quel brano:
  - **½** dimezza il BPM (un marker ogni due), **×2** lo raddoppia, **↺** torna alla griglia calcolata automaticamente.
  - **AUTO** usa un solo tempo finché i battiti non se ne allontanano a lungo, poi apre un nuovo tratto: va bene per quasi tutto. **FISSA** usa un tempo e una fase costanti per tutto il brano (musica elettronica, brani fatti al computer). **FLESS.** segue un batterista vero battuta per battuta (rock, funk, disco anni '70, dal vivo).
  - **QUANTIZE** come il pulsante QUANT.
- **QUANT** fa agganciare cue, loop, roll e hot cue ai battiti della griglia.
- **KEY** apre la tonalità: **KEY LOCK** tiene la tonalità quando cambi il pitch, il passo Camelot ±1 e le correzioni di mezzo tono.

### I pad

Il pulsante sopra i pad cambia la loro funzione:

| Modalità | Cosa fanno i pad |
|---|---|
| **HOT CUE 1–8** | Otto pad su due file. Un clic salva il punto o ci salta. Maiusc + clic, oppure tasto destro, lo cancella. Un hot cue vuoto è spento; uno memorizzato si accende del suo colore e fa un breve lampo quando lo richiami. |
| **BEAT JUMP** | Salta di −2, −1, +1, +2 volte la misura scelta con ‹ ›. |
| **LOOP ROLL** | Ripete un tratto finché tieni premuto. I pad fanno 1×, 2×, 4× e 8× la lunghezza base (da 1/16 a 1 battito, con ‹ ›). |
| **FRENO · SPIN · REV · SLIP** | **BRAKE**: il disco si ferma finché tieni premuto. **SPIN**: spinback. **REV**: reverse finché premi. **SLIP**: acceso/spento. |

Nelle modalità diverse dagli hot cue si usano i primi quattro pad.

**SLIP**: durante roll, freno e reverse il brano continua «in silenzio» e al rilascio ti ritrovi dove sarebbe arrivato. **Q** attiva la quantizzazione.

### Loop

- **‹ ›** dimezza e raddoppia il loop, il numero al centro lo accende o spegne.
- **IN** e **OUT** impostano inizio e fine; OUT lo attiva.

---

## Il mixer, le cuffie e il preascolto

Ogni canale ha fader, EQ a tre bande (**HI**, **MID**, **LOW**), **FILTER** e un indicatore di livello dopo il fader. Le manopole si girano trascinando in su e in giù (con Maiusc la regolazione è fine) o con la rotella; il doppio clic le riporta al valore iniziale. L'arco colorato attorno alla manopola mostra il valore. Il **crossfader** è in basso; doppio clic lo centra.

**AUTO GAIN** porta un brano appena caricato alla stessa sonorità di quello che sta suonando. **AUTO STEM** separa in anticipo gli stem di ogni brano caricato.

**Cuffie.** I pulsanti **1** e **2** in alto accendono il preascolto (PFL) dei canali. **CUE MIX** dosa tra i due lati, **PHONES** è il volume. Il PFL si accende anche dall'icona delle cuffie sul piatto.

---

## Effetti

Il pannello **FX** lavora sul deck scelto con i pulsanti **DECK** (1 / 2) sopra i pad; resta allineato alla striscia del canale.

- **Effetti sul segnale:** ECHO, PING-PONG, REVERB, FLANGER, PHASER, BITCRUSH, GATER, SCATTER, NOISE.
- **Effetti sul brano:** BEAT JUMP, BRAKE, SPINBACK, REVERSE. Agiscono sul disco e non sul segnale.
- **TIME** e **FEEL** (dritto, terzina, puntato) fissano il passo degli effetti a tempo.
- **I tre pomelli** cambiano nome a seconda dell'effetto. Per esempio nell'echo sono **TONE**, **FEEDBACK** e **MIX**.
- Il pulsante accanto al nome dell'effetto lo accende. Per freno e reverse, tenendo premuto più di 0,35 secondi l'effetto dura quanto la pressione.

---

## Stem

Gli stem separano un brano in **voce**, **batteria**, **basso** e **altro**. Nel pannello **STEMS** ogni stem ha una manopola del volume (la ghiera a segmenti si accende in proporzione), un pulsante muto (**M**) e un solo (**S**). Un clic sull'icona o sul nome dello stem lo esclude o lo riammette. I preset **ACAPELLA**, **INSTRUMENTAL** e **ALL** sistemano tutto con un clic.

SpinLab funziona in due modi:

- **Tempo reale.** Gli stem si separano mentre il brano suona, con il processore. Non serve preparare niente.
- **Preparati.** Gli stem si calcolano prima e si salvano. Il caricamento è praticamente immediato e la qualità è massima. Sono la scelta per i PC più lenti.

In **Impostazioni → STEM** scegli la modalità: **Automatico** (in base alla velocità misurata del PC), **Sempre in tempo reale** o **Solo stem preparati**. Il pulsante **Misura la velocità di questo PC** ripete la misura. Per il tempo reale serve almeno 2× la velocità di riproduzione, perché due deck separano insieme.

**Preparare gli stem.** Tasto destro su un brano (anche più brani) o su una playlist → **Crea stem**. Si salvano in una cartella `stems` accanto ai brani, compressi senza perdita di qualità (circa il 45% della dimensione originale). **Elimina gli stem preparati** li cancella.

Il modello di separazione è uno solo e viene scaricato da internet al primo uso (circa 460 MB). In «Solo stem preparati», un brano senza stem mostra un avviso che invita a prepararli.

---

## AUTOMIX

AUTOMIX suona da solo i brani della libreria, uno dopo l'altro, con transizioni a tempo come farebbe un DJ. Si trova nella scheda **AUTOMIX** del pannello in basso a destra.

**Come si usa**
1. Scegli quali brani suonare: apri una playlist, una cartella o fai una ricerca. AUTOMIX usa i brani che vedi nella tabella, a partire da quello selezionato.
2. Scegli **Ordine**, **Transizione** e **Stile** (sotto).
3. Premi **AVVIA AUTOMIX**. Se un deck sta già suonando, quel brano continua e i successivi partono alla sua fine.
4. **FERMA AUTOMIX** lo ferma: i deck restano come sono. **Salta** passa subito al brano successivo con una transizione.

**Le impostazioni**
- **Ordine.** *Come in elenco* (quello della tabella), *BPM crescente* o *BPM decrescente* (i brani senza BPM vanno in fondo), *Intelligente* (dopo ogni brano sceglie quello con tonalità, codice Camelot, e tempo più vicini, per mix più armonici) oppure *Casuale*.
- **La coda si può modificare.** Sotto le impostazioni c'è l'elenco dei prossimi brani. Su ogni riga: ⤒ lo mette per primo, ▲ e ▼ lo spostano, ✕ lo toglie. Da fermo, *ripristina* annulla le modifiche; cambiando l'ordine la coda si rifà da capo.
- **Transizione.** Quante battute dura il mix: 4, 8, 16 o 32. A 120 BPM, 16 battute sono circa 32 secondi.
- **Stile.** *Scambio dei bassi*: il nuovo brano entra senza bassi e a metà transizione i bassi passano da un brano all'altro, così non si sommano due bassi. *Dissolvenza semplice*: solo il crossfader. *Taglio rapido*: due battute, senza allineare il tempo.
- **Ripeti.** Quando la coda finisce, ricomincia da capo.

**Cosa fa il programma**
- Circa 75 secondi prima della fine del brano che suona, carica il successivo sull'altro deck.
- Allinea tempo e battiti con **SYNC**. Se i due brani hanno tempi troppo diversi (oltre il 12%, dopo aver tenuto conto del raddoppio e del dimezzamento) non li sincronizza e fa una transizione rapida.
- A transizione finita svuota il deck vecchio, e il nuovo brano torna con calma al suo tempo originale.

**Cose da sapere**
- I brani devono essere stati analizzati (tempo e tonalità). Senza analisi la transizione è una semplice dissolvenza senza sincronizzazione.
- Se sposti a mano il crossfader o i bassi durante un mix, AUTOMIX li rimette a posto al passo successivo.
- Dal controller MIDI si può assegnare **AUTOMIX avvia / ferma** e **AUTOMIX: salta**.

---

## Batteria e campionatore

### Batteria (DRUMS)

16 pad con campioni singoli, da suonare con il mouse, con la tastiera (Z X C V · A S D F · Q W E R · 1 2 3 4) o con un controller. La batteria parte vuota a ogni avvio; si ricordano solo le note MIDI dei pad.

- **Carica un campione:** clic su un pad vuoto, oppure trascina un file sul pad. Massimo 30 secondi.
- **Tasto destro su un pad:** MIDI learn, modo **gate** (suona finché tieni premuto), **aftertouch**, intonazione (±12 semitoni), **gruppo choke** (per esempio hi-hat chiuso e aperto che si zittiscono a vicenda), volume, nota MIDI, svuota.
- **Aftertouch.** Premendo il pad dopo il colpo, il suono cresce. La destinazione è per pad: spento, volume (fino a +60%), intonazione (fino a +7 semitoni) o entrambi. Funziona con l'aftertouch per singolo pad e con quello di canale. Una barra colorata sul pad mostra la pressione.
- **Note MIDI di default:** i 16 pad rispondono alle note 36–51.

### Campionatore (SAMPLER)

8 pad per suoni più lunghi. Trascina un file (o un brano della libreria) su un pad. Clic: parte, altro clic: si ferma. Tasto destro: loop, BPM, sostituisci. Tasti **1–8** della tastiera. Con **SYNC** i campioni partono sul battito del deck che suona e seguono tempo e fase.

---

## Video

SpinLab può mostrare un'immagine su uno schermo esterno o su un proiettore mentre suoni: i **video** che carichi sui deck, oppure, per i brani senza video, la **copertina con titolo e artista** o una **visualizzazione animata** a tempo di musica. I due deck si mescolano con il crossfader, con una transizione che scegli tu.

Il video non cambia il modo di suonare: l'audio comanda sempre. Un video si carica, si ferma, si sposta e si scratcha come un brano, e la griglia dei battiti, il BPM, la tonalità e gli stem funzionano sul suo audio come per qualunque altro file.

### Attivare la modalità VIDEO

La modalità si accende **da sola** quando carichi un video su un deck e si spegne quando sui deck restano solo brani audio. Con la modalità spenta il programma non usa risorse per il video e non apre nessuna finestra.

Il pulsante **VIDEO**, in alto vicino all'orologio, la forza a mano: a ogni clic passi alla scelta successiva tra **automatica**, **sempre accesa** (utile per vedere le visualizzazioni anche con soli brani audio) e **sempre spenta**. Quando è accesa il pulsante si illumina e accanto compaiono i menu delle transizioni e delle visualizzazioni e l'indicazione della qualità in uso.

### Il deck con un video

Quando un deck ha un video, il deck cambia aspetto da solo: al posto delle forme d'onda e del piatto compare lo **schermo con l'anteprima del video**, muta e a tempo con la musica. La **panoramica del brano** sta nella fascia sopra i pad, accanto al pulsante del pitch, e si usa come sempre (clic o trascinamento per spostarsi). Trascinando in orizzontale sullo schermo si fa lo **scratch** (tutta la larghezza vale 4 secondi). Il deck con un brano audio resta classico; quando togli il video, il deck torna come prima.

### Formati supportati

La libreria riconosce **mp4, m4v, mov, mkv, webm, avi, ts, mpg, mpeg**. I video scaricati da YouTube e convertiti in **mp4 (H.264)** sono il caso più semplice e funzionano senza altro.

- L'**immagine** viene riprodotta dal motore web del programma: i formati che il sistema (Windows o macOS) non sa leggere non si vedono. H.264 è sempre il più sicuro; per altri codec (H.265, VP9, AV1…) dipende dal computer.
- L'**audio** si decodifica come quello di un brano. Per i formati meno comuni serve FFmpeg, come per i brani: vedi **Soluzione dei problemi**.
- L'uscita è limitata a 1080p: i video 4K, se il sistema li legge, vengono mostrati a una risoluzione più bassa, e l'HDR non è gestito.

### L'uscita su schermo o proiettore

La finestra di uscita si apre da sola con la modalità VIDEO. Per mandarla su un monitor esterno, un proiettore o un televisore (HDMI, DisplayPort, USB-C), fai **clic destro sul pulsante VIDEO** e scegli lo schermo dall'elenco; **Finestra normale** la tiene in una finestra sul monitor del programma. La scelta si ricorda.

- Se lo schermo scelto si **scollega** mentre suoni, l'uscita passa in una finestra normale e compare un avviso: musica e video non si fermano. Quando lo **ricolleghi** torna a schermo intero.
- Nella finestra di uscita, **H** mostra o nasconde le misure (fotogrammi saltati, sfasamento rispetto all'audio, qualità in uso) ed **Esc** la chiude.

### Transizioni

Il menu a tendina accanto al pulsante VIDEO sceglie l'effetto con cui la miscela passa da un deck all'altro. Di default la miscela segue il **crossfader**; chi mixa con i fader del volume dei canali e lascia il crossfader fermo può scegliere **i fader dei canali** in **Impostazioni → VIDEO → Miscela** (o dal clic destro sul pulsante VIDEO): in quel caso il crossfader è ignorato e il video passa da un deck all'altro in base al volume dei due canali (se uno è a zero si vede solo l'altro; con i due volumi uguali, la miscela a metà; con entrambi a zero l'immagine resta com'era). L'effetto segue la posizione della miscela: a sinistra si vede il deck 1, a destra il deck 2, in mezzo la transizione. Si può cambiare anche durante il set.

Le transizioni sono: **Dissolvenza**, **Taglio**, **Dissolvenza al nero**, **Flash bianco**, **Wipe**, **Zoom a spinta**, **Glitch**, **Pixel**, **Caleidoscopio**, **Ondulazione**, **Cubo 3D**, **Vortice** e **Stroboscopio sul ritmo**. L'ultima alterna i due deck a tempo con la **griglia dei battiti**, quindi funziona meglio con brani la cui griglia è giusta.

Se un solo deck ha qualcosa da mostrare, si vede quello senza effetti. Con la qualità **Bassa** (vedi sotto) restano disponibili solo le prime cinque, le più leggere; le altre compaiono disattivate.

### Brani senza video: scheda e visualizzazioni

Un deck che ha un brano senza video partecipa comunque alla miscela. Il secondo menu accanto al pulsante VIDEO sceglie cosa mostrare:

- **Scheda del brano**: la copertina (se il file ne ha una) con titolo e artista, su uno sfondo che ne riprende i colori.
- **Dieci visualizzazioni**: Anelli, Tunnel, Onde luminose, Griglia al neon, Stelle, Plasma, Barre, Esagoni, Fluido e Raggi. Pulsano sul **battito** della griglia, reagiscono al **livello** del deck e prendono il colore dalla **tonalità** del brano.

Le visualizzazioni sono animazioni astratte create per SpinLab. Si possono usare anche senza nessun video, con la modalità VIDEO accesa a mano, per riempire uno schermo mentre suoni solo musica.

### Titolo e artista

Sopra i video e le visualizzazioni, in basso a sinistra, compaiono **titolo** e **artista** del brano del lato dominante del crossfader, con una comparsa morbida al cambio di brano. Sulle visualizzazioni una leggera ombra scura in basso tiene il testo leggibile. La scheda del brano contiene già titolo e artista.

### Scratch, salti e la copia di lavoro

I video comuni hanno un'immagine completa solo ogni qualche secondo: per mostrare un punto qualsiasi il programma deve ricostruirla, e questo rallenta scratch e salti. Per questo, quando carichi un video, SpinLab ne prepara in sottofondo una **copia di lavoro** leggera (480p) in cui ogni fotogramma è completo:

- Il video parte subito dall'originale; quando la copia è pronta la sostituisce senza che tu faccia nulla.
- La copia sta in una cartella temporanea e si **cancella da sola** quando il video lascia il deck. All'avvio il programma elimina quelle rimaste da una chiusura anomala. L'originale non viene mai modificato.
- Serve FFmpeg. Una copia occupa circa 85 MB per un video di quattro minuti e si prepara in pochi secondi.

### Stem sui video

Gli stem dei video si creano come per i brani: tasto destro sul video nella libreria o in una playlist → **Crea stem**. Si calcolano sull'audio e restano salvati per gli usi futuri. Per le modalità e la preparazione vedi **Stem**.

### Qualità e prestazioni

In **Impostazioni → VIDEO** scegli la qualità dell'uscita:

| Profilo | Risoluzione e fotogrammi | Transizioni |
|---|---|---|
| **Bassa** | fino a 480p, 24 fps | solo le cinque più leggere |
| **Media** | fino a 720p, 30 fps | tutte |
| **Alta** | fino a 1080p, 30 fps | tutte |
| **Massima** | fino a 1080p, 60 fps | tutte |

- **Automatico** (consigliato). Qualche secondo dopo l'avvio, a deck fermi, il programma analizza il computer (processore, memoria, scheda grafica, velocità della grafica, della decodifica e della codifica video) e sceglie un profilo di partenza. La misura si ripete da sola solo se cambia l'hardware o se la prima è fallita.
- **Ottimizza con un video in riproduzione.** Avvia un video su un deck, premi il pulsante e per circa 10 secondi il programma prova la qualità più alta e scende finché l'uscita è fluida; il risultato diventa il profilo automatico.
- **Durante il set.** Se il computer fatica, l'uscita scende da sola di un livello e compare un avviso; se il carico cala per un po', risale. L'audio non viene mai toccato.
- **Scelta a mano.** Un profilo scelto da menu resta valido finché non torni su **Automatico**: finché è forzato, la misura non lo cambia.

Per sapere a che qualità sei, guarda l'etichetta accanto al pulsante VIDEO (per esempio «1080p 60», arancione se l'uscita è stata ridotta dal carico) oppure la riga **Uscita in questo momento** nella scheda VIDEO. La scheda mostra anche i dati dell'analisi del computer.

Il profilo limita risoluzione e fotogrammi al secondo dell'uscita **a schermo intero**: su una piccola finestra, o con un video a 25 fps, profili diversi sembrano uguali.

### Registrare il set in MP4

Oltre all'audio, SpinLab può registrare **l'immagine dell'uscita video** (con transizioni, visualizzazioni, titolo e artista) e salvarla insieme al suono del master in un solo file **MP4** (video H.264, audio AAC).

1. **RECORD** → nel menu **Formato** scegli **MP4 con video (H.264 + AAC)**.
2. **Qualità del video**: *Come il profilo video in uso* (consigliato) oppure 480p 24 fps, 720p 30 fps, 1080p 30 fps, 1080p 60 fps. La qualità della registrazione è indipendente da quella dell'uscita: puoi proiettare a una risoluzione e registrare a un'altra. Il video registrato ha la **stessa forma dell'immagine sull'uscita** (16:9, 16:10…), senza barre nere aggiunte: se il video originale è 4:3 vedrai comunque le bande ai lati, che sono parte dell'immagine. Il 1080p a 60 fps richiede un computer potente.
3. Scegli dove salvare il file e premi **Avvia registrazione**. L'uscita video si accende da sola per tutta la durata (anche se sui deck ci sono solo brani audio: vedrai la scheda o la visualizzazione scelta).
4. **Ferma** con un clic sull'indicatore **REC** o su **STOP REC**. Il file va preparato (si uniscono audio e video): può servire qualche secondo, e compare un avviso con il percorso e la dimensione.

Cose da sapere:

- Serve **FFmpeg** (come per le copie di lavoro); se manca, la registrazione non parte e un messaggio lo spiega.
- La registrazione cattura quello che si vede sull'uscita, quindi richiede il **disegno WebGL** attivo (la modalità semplice senza WebGL non è registrabile).
- Se il computer non tiene il ritmo, alcuni fotogrammi si perdono (il video si vede un po' a scatti) ma **l'audio non si interrompe**; a fine registrazione compare un avviso con il numero di fotogrammi persi.
- Se qualcosa va storto dopo la registrazione (per esempio non riesce l'unione), l'audio resta salvato in un file temporaneo e il messaggio indica dove.
- I file intermedi si cancellano da soli quando il file finale è pronto.

### Cosa non c'è ancora

- Il video su più di due deck e un fader video separato dal crossfader.

I video musicali e i filmati sono coperti da diritti d'autore: ciò che proietti, registri o pubblichi è responsabilità tua.

### Se qualcosa non va con il video

**Lo schermo resta nero o compare un errore.** Il formato non è letto dal sistema. Premi **H** nella finestra di uscita: se vedi un errore sul deck, converti il video in mp4 (H.264).

**Il video va a scatti o si riallinea di continuo.** Attendi qualche secondo dopo il caricamento: la copia di lavoro deve finire (in **H** compare «copia di lavoro: sì»). Se dopo non basta, abbassa la qualità in **Impostazioni → VIDEO** o premi **Ottimizza**.

**Il video non è a tempo con la musica.** Una differenza fissa tra audio e immagine si regola con il cursore **Allineamento della grafica** (**Impostazioni → AUDIO**), lo stesso delle waveform.

**Non vedo lo schermo esterno nell'elenco.** Collegalo prima di aprire il menu e controlla che il sistema lo riconosca. Due monitor identici (stesso nome e stessa risoluzione) non si distinguono: SpinLab sceglie il primo.

**Le transizioni sono disattivate o ce n'è una sola.** Con il profilo **Bassa** restano solo le più leggere. Scegli un altro profilo nelle Impostazioni.

**La registrazione MP4 non parte o il video è a scatti.** Controlla che FFmpeg sia disponibile e che l'uscita video mostri l'immagine (premi **H** nella finestra di uscita: «disegno: WebGL»). Con un computer lento scegli una qualità più bassa nel dialogo di registrazione.

**Nel pannello compaiono misure a zero o «non misurata».** La prima analisi non è riuscita (di solito manca FFmpeg o la scheda grafica non è stata letta). Riavvia il programma: si ripete da sola; oppure scegli un profilo a mano.

---

## Aspetto: skin, colori e monitor ultrawide

Tutto si sceglie in **Impostazioni → INTERFACCIA** e si applica subito; la scelta si ricorda.

### Skin

| Skin | Aspetto |
|---|---|
| **Dark Club** (predefinita) | Grafite e alluminio con luci viola, piatto con bordo forato e braccio del giradischi. |
| **Industrial** | Metallo scuro, lamiera forata e mandorlata, luci ambra; piatto motorizzato con l'anello di luce ambra. |
| **Retro Hardware** | Alluminio spazzolato, tasti color crema, accenti rossi; un tasto attivo resta premuto e la sua scritta si illumina. |

Nelle skin Industrial e Retro il piatto gira come un piatto a motore: l'anello esterno, le viti e il mozzo girano alla velocità reale, mentre i riflessi del metallo restano fermi.

### Colore del tema

Ogni skin ha il suo colore (viola, ambra o rosso), che vale per luci, manopole, piatti, forme d'onda e hot cue. Per cambiarlo scegli la tinta sulla **barra colorata** e la sfumatura nel **quadrato** sopra (come in Photoshop), oppure scrivi il codice esadecimale. Sotto vedi un'**anteprima reale** dell'interfaccia che cambia mentre scegli. **↺ Ripristina colore predefinito** torna al colore della skin. PLAY verde, CUE ambra e i meter verde-giallo-rosso restano fissi perché hanno un significato.

### Monitor ultrawide

Quando la finestra è almeno 2,4 volte più larga che alta (per esempio un monitor 32:9 a schermo intero), SpinLab usa una disposizione dedicata:

- **a sinistra** deck e mixer, nelle stesse proporzioni di un monitor normale;
- **sotto i deck**, affiancati: gli **stem** (sempre visibili), il pannello scelto fra **SAMPLER**, **DRUMS** e **AUTOMIX** e gli **FX**;
- **a destra** cartelle, playlist ed elenco dei brani a tutta altezza.

In questa disposizione la libreria è sempre aperta, quindi la vista LIBRARY non serve. Se stringi la finestra torna la disposizione normale. Sui monitor tradizionali non cambia nulla.

---

## Registrare il master

**RECORD** in alto apre la finestra della registrazione, dove scegli formato, qualità del video e cartella e premi **Avvia registrazione**: la finestra si chiude e compare l'indicatore **REC** con il tempo. Registra tutto ciò che esce dal master: deck con effetti, campionatore e batteria. **Dopo la prima volta** puoi avviare e fermare anche con il pulsante tondo **●** in alto, accanto all'orologio: usa le stesse impostazioni e un nome automatico (`SpinLab AAAA-MM-GG HH.MM.SS`) nella stessa cartella, senza riaprire la finestra. Durante la registrazione il pulsante diventa l'indicatore **REC** (un clic lo ferma) e il pulsante RECORD diventa **STOP REC**.

1. Scegli il formato: WAV 16 o 24 bit, MP3 a 128, 192, 256 o 320 kbps, oppure **MP4 con video** (vedi [Registrare il set in MP4](#registrare-il-set-in-mp4)).
2. Scegli dove salvare il file.
3. **Avvia registrazione.** Compare l'indicatore **REC** con il tempo. Un clic sull'indicatore, o su **STOP REC**, ferma la registrazione.

Non si possono cambiare le impostazioni audio mentre si registra.

---

## Impostazioni audio

**SETTINGS** → scheda **AUDIO**.

- **Driver audio.** **Windows (WASAPI)** funziona con qualsiasi scheda. **ASIO** usa il driver della scheda e dà latenze molto più basse, ma la scheda diventa dedicata a SpinLab.
- **Uscita principale (master).** Con ASIO l'elenco mostra i driver ASIO installati. Per usare la scheda integrata del PC scegli il driver **Windows (WASAPI)**.
- **Frequenza e buffer.** Un buffer piccolo dà meno latenza ma più rischio di scatti: se senti click, aumentalo. 128 frame è un buon punto di partenza con ASIO.
- **Uscita cuffie.** È l'uscita del preascolto e dell'anteprima dei brani. L'elenco mostra i dispositivi Windows. In alternativa **Stesso dispositivo dell'uscita principale, canali 3-4**: master sui canali 1-2 e cuffie sui canali 3-4 della stessa scheda (serve un'uscita con almeno 4 canali).

### ASIO: da dove viene il driver

**SpinLab non installa nessun driver ASIO.** ASIO è un sistema di driver a bassa latenza: il driver lo scrive e lo installa il **produttore della scheda audio** (per esempio Focusrite, Behringer, RME, MOTU, Denon DJ). Quando installi la scheda, il suo driver si registra in Windows, e SpinLab ne mostra l'elenco nel menu «Uscita audio principale». Ogni utente usa quindi il driver ASIO della **propria** scheda, già installato sul suo PC.

- **La scheda ha un driver ASIO** (di solito sì, nelle schede audio esterne e nelle interfacce per DJ e musica): scegli driver **ASIO** e la scheda dall'elenco.
- **La scheda integrata del PC o altre schede senza driver ASIO**: usa **Windows (WASAPI)**. Funziona sempre, con una latenza più alta. Se vuoi comunque ASIO su una scheda qualsiasi esistono driver universali di terze parti (per esempio ASIO4ALL o FlexASIO): se li installi, compaiono nell'elenco come gli altri.
- **Con ASIO la scheda è dedicata a SpinLab**: altri programmi non possono usarla nello stesso momento.

**Dopo «Applica»**, l'area sotto mostra l'uscita realmente attiva e la latenza stimata. Se l'uscita scelta non parte, SpinLab torna a Windows (WASAPI) e ti dice il motivo.

**Esempi di configurazione**

| Situazione | Master | Cuffie |
|---|---|---|
| Scheda esterna con ASIO e controller | ASIO → la scheda | Dispositivo Windows delle cuffie del controller (di solito compare con un nome tipo «Line 3/4 (nome del controller)») |
| Tutto sul controller | ASIO o WASAPI del controller | Stesso dispositivo, canali 3-4 |
| Solo PC | WASAPI | Un secondo dispositivo, o nessuno |

---

## Controller MIDI

SpinLab funziona con qualsiasi controller MIDI. Le funzioni si assegnano nel pannello **SETTINGS → CONTROLLER MIDI** con il MIDI learn. La mappatura si salva da sola e si esporta e importa come testo.

### Il pannello

- **Collegati** elenca i controller trovati. **Ultimo messaggio** mostra in tempo reale quello che arriva: serve a capire cosa manda un controllo.
- **Uscita LED** è la porta per accendere i LED del controller.
- **Jog per giro**: quanti messaggi manda il jog per un giro completo del piatto. **Calibra il jog** lo misura da solo: premi, gira il piatto di un giro, premi di nuovo.
- **Analizza un controllo**: per 8 secondi registra tutti i messaggi. Gira un controllo in un senso e poi nell'altro; il riepilogo mostra quale messaggio manda e con che valori. Utile per capire un encoder o la jog.
- **Ignora un tasto…**: per i sensori di tocco (per esempio un encoder che manda un messaggio quando lo sfiori). Premi, tocca il sensore: quel messaggio non comanda più nulla e il learn lo salta.
- **Cerca una funzione** filtra l'elenco. Le funzioni sono raggruppate per deck e per mixer, libreria ed effetti.
- **Esporta / Importa / Cancella tutto** per copiare o azzerare la mappatura.

### Assegnare una funzione

1. Premi **Learn** sulla riga della funzione.
2. Premi il tasto, gira la manopola o muovi il fader sul controller.
3. Il Learn riconosce da solo il tipo di controllo: pulsante, fader a 7 bit, encoder relativo (complemento a 2, offset 64, bit di segno) o valore assoluto che gira (tipico di una jog). Se sbaglia, cambia il **Modo** nel menu della riga.
4. Per **togliere** un'assegnazione usa la ✕ della riga. Esc annulla un Learn in corso.

Per i controlli continui (fader, manopole, encoder) il Learn aspetta un movimento vero: il rumore o i sensori che mandano sempre lo stesso valore non vengono assegnati.

### Più assegnazioni

- Una funzione può avere **più assegnazioni**: premi **+ Altro** per aggiungerne un'altra (per esempio due tasti che fanno la stessa cosa).
- Un **messaggio** può comandare **più funzioni** (per esempio un solo clic che fa scattare due cose). Se capita, la riga mostra un avviso «⚠ anche …» che elenca le altre funzioni: serve a scoprire assegnazioni vecchie dimenticate.

### SHIFT

Assegna prima il tasto **SHIFT** del controller. Per fare una combinazione:

1. Premi **Learn** su una funzione.
2. **Tieni premuto SHIFT** sul controller.
3. Premi il tasto da combinare (per esempio un pad) e rilascia.

La funzione risulta «SHIFT + pad». Lo stesso pad, senza SHIFT, può comandare un'altra funzione. Se non hai ancora assegnato SHIFT, tieni premuto il tasto e premi l'altro: SpinLab capisce da solo che il primo è SHIFT. Puoi assegnare anche due tasti SHIFT (uno per deck).

### Livelli dei pad

Se il controller ha tasti di modalità per i pad (per esempio HOT CUE, LOOP, ROLL, BANK), SpinLab li usa come **livelli**:

1. Nel gruppo del deck assegna **Livello dei pad: Hot cue / Loop / Roll / Bank** ai quattro tasti di modalità.
2. Premi **Learn** su una funzione, premi il tasto di modalità (per esempio LOOP), poi il pad.
3. L'assegnazione vale solo in quel livello. Il menu **Livello** sulla riga permette di correggere.

Se in un livello non ci sono assegnazioni per un pad, vale quella «Ogni livello». I livelli sono indipendenti per deck, e all'avvio i pad sono in Hot cue.

### Jog, encoder e tasti speciali

- **Jog.** Con il piatto toccato fa scratch; a brano in play e piatto non toccato, fa un piccolo pitch bend. Assegna la rotazione a **Jog (scratch e pitch bend)** e il tocco a **Jog: tocco del piatto**.
- **Encoder come pulsante.** Su una funzione a pulsante (per esempio **Loop × 2**) puoi usare la rotazione di un encoder: il Learn la riconosce e imposta «Encoder: solo rotazione oraria». Per l'altra direzione usa il menu Modo.
- **Loop IN / OUT con un solo tasto:** primo clic = IN, secondo = OUT, terzo = esce dal loop.
- **Roll:** lunghezza più lunga / più corta / encoder.
- **Libreria:** scorri i brani (un encoder per ogni deck, anche a 10 brani per scatto) e carica il brano scelto.

### Effetti dal controller

Il gruppo **Effetti del controller** pensa a controller con un solo set di comandi effetti:

- **Assegna l'effetto al deck 1 / 2.** Premuti insieme (entro 0,4 secondi) valgono per entrambi i deck.
- **Effetto acceso / spento** sui deck assegnati.
- **Parametro (encoder)** con **clic dell'encoder** che alterna tra il primo e il secondo parametro (per esempio TONE e FEEDBACK). Il nome sottolineato nel pannello FX mostra quale è attivo.
- **MIX dry / wet** (manopola).
- **Effetto successivo / precedente** o encoder di scelta dell'effetto.

Gli scatti dell'encoder hanno un'accelerazione e una piccola rampa per non sentire a gradini. Gli scatti subito prima di un clic vengono scartati, perché sono solo lo spostamento causato dalla pressione.

### LED

Scegli la porta in **Uscita LED**. Ogni assegnazione di un tasto ha due valori (acceso e spento) che SpinLab manda al controller quando la funzione cambia stato. Funziona solo se il controller accetta messaggi di LED via MIDI.

---

## Soluzione dei problemi

**Il driver ASIO non parte e resto in Windows (WASAPI).**
Leggi il messaggio sotto «Applica». Cause frequenti: un altro programma tiene il driver aperto (un'altra copia di SpinLab, il pannello della scheda, una DAW); un controller con scheda audio collegato può avere il suo driver ASIO caricato; SpinLab apre solo il driver che scegli e non tocca gli altri. Chiudi le copie in più di SpinLab dal Gestore attività.

**Click o interruzioni nell'audio.** Aumenta il buffer. Con ASIO prova 256.

**Le cuffie non suonano.**
- L'uscita cuffie va scelta tra i dispositivi Windows. Il preascolto ha i suoi pulsanti PFL (1 e 2 in alto, o l'icona sul piatto) e il volume **PHONES**.
- Per «Stesso dispositivo, canali 3-4» l'uscita principale deve avere almeno 4 canali.

**Il jog va in una sola direzione o a scatti.** Il modo è sbagliato. Rifai il Learn girando il piatto per un secondo, oppure scegli *Assoluto, usato come differenza* nel menu Modo. Se è troppo veloce, usa **Calibra il jog**.

**Lo scratch è distorto.** Ricalibra «Jog per giro». Un valore troppo basso fa correre il disco.

**Il Learn assegna da solo.** Il controller sta mandando messaggi senza che tu tocchi niente (rumore, sensori). Il Learn dei controlli continui aspetta un movimento vero. Se continua, usa **Ignora un tasto…**.

**Un encoder muove due cose.** Guarda gli avvisi «⚠ anche …» nella lista: una vecchia assegnazione sullo stesso messaggio muove altre funzioni. Cancellala con la ✕.

**Il video non si vede, va a scatti o non è a tempo.** Vedi [Video](#video), in particolare «Se qualcosa non va con il video».

**Gli stem sono lenti o non partono.** Vedi **Impostazioni → STEM**: con «Solo stem preparati» prepara prima i brani (tasto destro → Crea stem). Il primo uso scarica il modello e richiede internet.

---

---

## Crediti

ASIO is a trademark of Steinberg Media Technologies GmbH, registered in Europe and other countries.

![ASIO Compatible](../ui/assets/asio-logo-bw.png)
