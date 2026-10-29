# SpinLab DJ — Manuale

![ASIO Compatible - ASIO is a registered trademark of Steinberg Media Technologies GmbH](../ui/assets/asio-logo-bw.png)

SpinLab DJ è un programma per DJ con due deck, mixer, effetti, separazione in stem, batteria, campionatore, registrazione del master e supporto per qualsiasi controller MIDI. Questo manuale spiega come usarlo, come collegare un controller e come risolvere i problemi più comuni.

> I tasti e i comandi sono descritti come compaiono sullo schermo. Passando il mouse sopra quasi ogni pulsante compare un suggerimento.

## Primo avvio

1. **Scegli la scheda audio.** Apri **SETTINGS** (o l'ingranaggio in alto a destra) → scheda **AUDIO**. Se hai una scheda con driver ASIO (per esempio Focusrite) scegli driver **ASIO®**: la latenza è molto più bassa e i pad della batteria rispondono subito. Altrimenti lascia **Windows (WASAPI)**. Premi **Applica**.
2. **Scegli le cuffie.** Nello stesso pannello, «Uscita cuffie». Vedi [Impostazioni audio](#impostazioni-audio).
3. **Aggiungi la musica.** Nella colonna **ESPLORA** a sinistra scegli una cartella: i brani compaiono nella tabella. Vengono analizzati (BPM, tonalità, griglia dei battiti) in sottofondo.
4. **Carica un brano.** Seleziona un brano e premi **LOAD** nella sua riga, oppure trascinalo su un deck.

---

## La schermata

- **Barra in alto.** Navigazione (**LIBRARY**, **MIXER**, **RECORD**, **SETTINGS**), indicatore **REC** quando registri, carico del processore (**CPU**), orologio, comandi delle cuffie (pulsanti 1 e 2, **CUE MIX**, **PHONES**) e volume **MASTER**.
- **Centro.** I due deck con le waveform e, in mezzo, il mixer a due canali con il crossfader.
- **In basso a sinistra.** Esplora cartelle e playlist.
- **In basso al centro.** La libreria: ricerca, tabella dei brani, scheda **TAG & INFO**.
- **In basso a destra.** Il pannello con **AUTOMIX** (in arrivo), **SAMPLER**, **DRUMS**, **STEMS** e, sotto, gli **FX**.

La grafica si adatta alla finestra: se la riduci molto, si rimpicciolisce in blocco.

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
- **Tasto destro su un brano.** Carica sul deck 1 o 2, aggiungi a una playlist (esistente o nuova con la selezione), analizza (BPM, tonalità, griglia), crea o elimina gli stem preparati, rimuovi dalla playlist (se ne hai una aperta).

---

## I deck

Ogni deck mostra due waveform: quella grande (zoom, scorre) e quella piccola (panoramica dell'intero brano), il piatto, i pulsanti e il fader del pitch.

### Trasporto

- **PLAY** e **CUE**. CUE tenuto premuto fa partire il brano dal punto di cue finché non rilasci.
- **LOAD** carica il brano selezionato nella libreria; il tasto accanto svuota il deck.
- **Il pitch** si regola con il fader a destra. **±8** (o il valore indicato) cambia l'escursione: 4, 8, 16, 50, 100%. Doppio clic sul fader lo riporta a zero.
- **SYNC** allinea tempo e fase al deck che fa da guida. Resta attivo anche dopo aver caricato un altro brano.

### Scrub

Tieni premuto il mouse sulla waveform grande e trascina: il brano segue come un disco (un trascinamento a sinistra avanza). Anche il piatto si può girare con il mouse. Sulla waveform piccola un clic porta la testina in quel punto.

### Griglia e tonalità

- **GRID** apre la correzione della griglia: spostamento fine (Maiusc = 1 ms), dimezza o raddoppia il BPM, «1» sulla testina, ancora sul marker, azzeramento. **FLUIDA** segue le variazioni di tempo del brano, **FISSA** usa un BPM costante.
- **QUANT** fa agganciare cue, loop, roll e hot cue ai battiti della griglia.
- **KEY** apre la tonalità: **KEY LOCK** tiene la tonalità quando cambi il pitch, il passo Camelot ±1 e le correzioni di mezzo tono.

### I quattro pad

Il pulsante sopra i pad cambia la loro funzione:

| Modalità | Cosa fanno i 4 pad |
|---|---|
| **HOT CUE 1–4** e **5–8** | Un clic salva il punto o ci salta. Maiusc + clic, oppure tasto destro, lo cancella. |
| **BEAT JUMP** | Salta di −2, −1, +1, +2 volte la misura scelta con ‹ ›. |
| **LOOP ROLL** | Ripete un tratto finché tieni premuto. I pad fanno 1×, 2×, 4× e 8× la lunghezza base (da 1/16 a 1 battito, con ‹ ›). |
| **FRENO · SPIN · REV · SLIP** | **BRAKE**: il disco si ferma finché tieni premuto. **SPIN**: spinback. **REV**: reverse finché premi. **SLIP**: acceso/spento. |

**SLIP**: durante roll, freno e reverse il brano continua «in silenzio» e al rilascio ti ritrovi dove sarebbe arrivato. **Q** attiva la quantizzazione.

### Loop

- **‹ ›** dimezza e raddoppia il loop, il numero al centro lo accende o spegne.
- **IN** e **OUT** impostano inizio e fine; OUT lo attiva.

---

## Il mixer, le cuffie e il preascolto

Ogni canale ha fader, EQ a tre bande, filtro e un indicatore di livello dopo il fader. Il **crossfader** è in basso; doppio clic lo centra.

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

Gli stem separano un brano in **voce**, **batteria**, **basso** e **altro**. Nel pannello **STEMS** ogni stem ha un fader, un pulsante muto (**M**) e un solo (**S**). I preset **ACAPELLA**, **INSTRUMENTAL** e **ALL** sistemano tutto con un clic.

SpinLab funziona in due modi:

- **Tempo reale.** Gli stem si separano mentre il brano suona, con il processore. Non serve preparare niente.
- **Preparati.** Gli stem si calcolano prima e si salvano. Il caricamento è praticamente immediato e la qualità è massima. Sono la scelta per i PC più lenti.

In **Impostazioni → STEM** scegli la modalità: **Automatico** (in base alla velocità misurata del PC), **Sempre in tempo reale** o **Solo stem preparati**. Il pulsante **Misura la velocità di questo PC** ripete la misura. Per il tempo reale serve almeno 2× la velocità di riproduzione, perché due deck separano insieme.

**Preparare gli stem.** Tasto destro su un brano (anche più brani) o su una playlist → **Crea stem**. Si salvano in una cartella `stems` accanto ai brani, compressi senza perdita di qualità (circa il 45% della dimensione originale). **Elimina gli stem preparati** li cancella.

Il modello di separazione è uno solo e viene scaricato da internet al primo uso (circa 460 MB). In «Solo stem preparati», un brano senza stem mostra un avviso che invita a prepararli.

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

## Registrare il master

**RECORD** in alto apre la finestra di registrazione. Registra tutto ciò che esce dal master: deck con effetti, campionatore e batteria.

1. Scegli il formato: WAV 16 o 24 bit, oppure MP3 a 128, 192, 256 o 320 kbps.
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

Per chi **compila** il programma (non per chi lo usa) serve l'SDK ASIO di Steinberg: contiene solo i file necessari a «parlare» con i driver e non è incluso nel progetto per la sua licenza. L'installer non contiene driver. Se un giorno SpinLab verrà distribuito pubblicamente con il supporto ASIO, bisognerà rispettare la licenza di Steinberg per l'SDK.

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

**«Accesso negato (os error 5)» compilando.** Il programma è ancora aperto e Windows non lo lascia sostituire. Chiudi SpinLab (e le copie rimaste nel Gestore attività).

**Gli stem sono lenti o non partono.** Vedi **Impostazioni → STEM**: con «Solo stem preparati» prepara prima i brani (tasto destro → Crea stem). Il primo uso scarica il modello e richiede internet.

---

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

---

## Crediti

ASIO is a trademark of Steinberg Media Technologies GmbH, registered in Europe and other countries.

![ASIO Compatible](../ui/assets/asio-logo-bw.png)
