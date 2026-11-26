# SpinLab DJ

<img src="ui/assets/asio-logo-bw.png" alt="ASIO Compatible - ASIO is a registered trademark of Steinberg Media Technologies GmbH" width="110" align="right">

Programma per DJ con due deck, mixer, effetti, separazione in stem, batteria, campionatore, **uscita video** per schermi e proiettori (con transizioni, visualizzazioni e registrazione del set in MP4), registrazione del master, **skin** e colori a scelta e supporto per qualsiasi controller MIDI. Per Windows e macOS.

**Versione attuale: 0.2.0** per Windows e macOS (Apple Silicon) — scarica dalle [Release](https://github.com/muttleysat/SpinLab/releases); le novità sono in [CHANGELOG.md](CHANGELOG.md).

L'installer contiene già tutto (modello degli stem, modelli dei battiti, FFmpeg): dopo l'installazione non viene scaricato niente e il programma funziona senza internet.

## Cosa sa fare

### Deck
- Due deck con forma d'onda a colori a scorrimento e panoramica dell'intero brano; scrub con il mouse sulla forma d'onda o sul piatto (scratch).
- **PLAY**, **CUE** (tenuto premuto fa ascoltare dal punto di cue), **SYNC** di tempo e fase che resta attivo cambiando brano.
- Pitch con escursione **±4, 8, 16, 50, 100%**; **→0** riporta il pitch a zero gradualmente, senza scatti.
- **KEY LOCK** (tonalità ferma cambiando velocità), cambio di tonalità per semitoni e per passi della ruota Camelot.
- **8 hot cue** per deck, **BEAT JUMP**, **LOOP ROLL**, **freno, spinback, reverse** e **SLIP**; **QUANTIZE** sui battiti.
- Loop con dimezza/raddoppia, IN e OUT.
- Conto alla rovescia che lampeggia in rosso negli ultimi 20 secondi del brano.
- Forme d'onda verticali tra i fader, per lo scratch.

### Analisi e griglia dei battiti
- Analisi automatica di **BPM, tonalità (Camelot) e griglia dei battiti**, in sottofondo.
- Griglia affinata dalla rete neurale **Beat This!**, anche per i brani suonati dal vivo.
- Modalità per brano: **AUTO**, **FISSA** (tempo costante) e **FLESS.** (segue un batterista vero battuta per battuta).
- Correzioni rapide: dimezza o raddoppia il BPM, **ANCHOR** per rimettere la griglia sul battito in un punto preciso, ripristino.

### Mixer
- Due canali con fader, **EQ a tre bande**, **filtro** e meter a LED; **crossfader**.
- **AUTO GAIN**: ogni brano caricato prende la stessa sonorità di quello che suona.
- **Cuffie** con preascolto (PFL) per canale, **CUE MIX** e volume dedicato; uscita cuffie su un'altra scheda o sui canali 3-4 della stessa.

### Effetti
- **ECHO, PING-PONG, REVERB, FLANGER, PHASER, BITCRUSH, GATER, SCATTER, NOISE** a tempo, con passo e feel (dritto, terzina, puntato).
- Effetti sul brano: **BEAT JUMP, BRAKE, SPINBACK, REVERSE**.

### Stem
- Separazione di ogni brano in **voce, batteria, basso e altri strumenti** con il modello HT-Demucs.
- **In tempo reale** mentre il brano suona oppure **preparati** in anticipo (caricamento immediato, qualità massima), anche per un'intera playlist.
- Volume, muto e solo per ogni stem; preset **ACAPELLA**, **INSTRUMENTAL**, **ALL**.

### AUTOMIX
- Suona da solo i brani della libreria con transizioni a tempo: ordine come in elenco, per BPM o **intelligente** (tonalità e tempo più vicini).
- Transizioni da 4 a 32 battute, stile **scambio dei bassi**, dissolvenza o taglio rapido; coda modificabile e salvata fra le sessioni.

### Batteria e campionatore
- **DRUMS**: 16 pad con campioni, dalla tastiera, dal mouse o dai pad MIDI; gate, aftertouch, intonazione, gruppi choke.
- **SAMPLER**: 8 pad per suoni lunghi e loop, con **SYNC** sul tempo del deck che suona.

### Video
- **Video sui due deck**, a tempo con la musica: si caricano, si fermano, si spostano e si scratchano come un brano.
- Uscita in una finestra o **a schermo intero su un secondo schermo o proiettore**, con collegamento a caldo.
- **13 transizioni** (dissolvenza, taglio, dissolvenza al nero, flash, wipe, zoom, glitch, pixel, caleidoscopio, ondulazione, cubo 3D, vortice, stroboscopio a tempo) guidate dal crossfader o dai fader dei canali.
- Per i brani senza video: **scheda con copertina, titolo e artista** oppure **10 visualizzazioni animate** a tempo di musica.
- Qualità adattata al computer, con profili da 480p a 1080p 60 e adattamento durante il set.
- Il deck con un video diventa uno schermo con l'anteprima.

### Registrazione
- Registrazione del master in **WAV** (16 o 24 bit) o **MP3** (fino a 320 kbps).
- Registrazione del set in **MP4 con il video** (H.264 + AAC).
- Avvio e stop con un clic dal pulsante ● in alto.

### Libreria
- Esplora cartelle, playlist, ricerca per titolo, artista, tag, tonalità e BPM.
- Tag, valutazione a stelle e commenti; **anteprima in cuffia** dei brani con scrub.
- Riquadro con le informazioni del brano selezionato.

### Controller MIDI
- Funziona con **qualsiasi controller MIDI**: assegnazione con **Learn**, più assegnazioni per comando, **SHIFT**, livelli dei pad, jog con calibrazione, encoder, effetti e **LED**.

### Aspetto
- Tre **skin**: **Dark Club** (grafite e luci viola), **Industrial** (metallo scuro e luci ambra), **Retro Hardware** (alluminio spazzolato, tasti crema, accenti rossi), con piatti motorizzati che girano alla velocità reale.
- **Colore del tema a scelta** con selettore e anteprima dal vivo dell'interfaccia.
- Layout dedicato per i **monitor ultrawide** (32:9): deck e mixer a sinistra, libreria a tutta altezza.

### Audio
- **ASIO** su Windows per la latenza più bassa (oltre a WASAPI); **Core Audio** su Mac.
- Buffer e frequenza di campionamento regolabili.

### Altro
- Guida completa dentro il programma (pulsante **?**), anche senza internet.
- Aggiornamenti automatici (Windows).

## Documentazione

- **[Manuale completo](docs/MANUALE.md)**: uso del programma, controller MIDI, impostazioni audio, soluzione dei problemi.
- **[Novità](CHANGELOG.md)**: cosa è cambiato in ogni versione.
- **[FFmpeg](docs/FFMPEG.md)**: licenza, inclusione nell'installer e aggiornamento di FFmpeg.
- **[Decisioni](docs/DECISIONI.md)**: le decisioni aperte che spettano al responsabile del prodotto (con opzioni e raccomandazioni) e quelle già prese.
- **[Video](docs/VIDEO.md)**: documentazione tecnica del modulo video (architettura, messaggi, sincronia, profili); i requisiti sono in [PRD-video.md](docs/PRD-video.md).
- Dentro il programma: il pulsante **?** in alto a destra apre la stessa guida, anche senza internet.

Il manuale si scrive in `docs/MANUALE.md`. Dopo ogni modifica, per aggiornare la guida che si apre nel programma:

```
python docs/build_guida.py
```

## Installare

- **Windows**: installer `SpinLab_<versione>_x64-setup.exe`; le versioni installate si aggiornano da sole.
- **macOS** (Apple Silicon): `SpinLab_<versione>_aarch64.dmg` dalle [Release di GitHub](https://github.com/muttleysat/SpinLab/releases). Il programma non è ancora firmato da Apple: la prima volta aprilo con clic destro → **Apri**, oppure da Impostazioni di Sistema → Privacy e sicurezza → **Apri comunque**.

## Compilare

Su Windows servono Git, Rust con gli strumenti C++ di Visual Studio, LLVM/clang, l'SDK ASIO di Steinberg (non incluso per la sua licenza; i percorsi si impostano in `src-tauri/.cargo/config.toml`) e FFmpeg nel PATH. Su macOS bastano Git, Rust e gli strumenti da riga di comando di Xcode.

Per l'installer completo (modello degli stem, FFmpeg, licenze) prima `python tools/fetch_model.py` e `python tools/fetch_ffmpeg.py`, poi `cargo tauri build --config tauri.release.conf.json` (vedi `docs/SVILUPPO.md`).

```
cargo install tauri-cli --version "^2"
cd src-tauri
cargo tauri dev        # per provare
cargo tauri build      # per creare l'installer
```

Dettagli per chi sviluppa in `docs/SVILUPPO.md`.

ASIO is a trademark of Steinberg Media Technologies GmbH, registered in Europe and other countries.
