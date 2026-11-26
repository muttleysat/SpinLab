# Novità di SpinLab DJ

## 0.2.0 — 26 novembre 2026

Rispetto alla 0.1.1 (31 ottobre 2026, la versione di prova dell'aggiornamento automatico). Prima versione anche per **macOS** (Apple Silicon).

### Video
- **Video sui due deck**, a tempo con la musica: si caricano, si fermano, si spostano e si scratchano come un brano; griglia, BPM, tonalità e stem funzionano sul loro audio.
- **Uscita su finestra o su schermo esterno / proiettore**, con collegamento e scollegamento dello schermo a caldo.
- **13 transizioni** (dissolvenza, taglio, tendina, zoom, glitch, cubo, vortice…) guidate dal crossfader oppure dai fader dei canali.
- Per i brani senza video: **scheda con copertina, titolo e artista** oppure **10 visualizzazioni animate** a tempo; titolo e artista anche sopra i video.
- **Modalità VIDEO automatica**: si accende da sola quando carichi un video.
- **Qualità adattata al computer**: misura di grafica, decodifica e codifica all'avvio, profili Bassa / Media / Alta / Massima, adattamento durante il set.
- **Registrazione del set in MP4** (H.264 + AAC) con immagine e audio del master.
- **Deck con un video**: lo schermo con l'anteprima prende il posto di forme d'onda e piatto, con la panoramica scrubbabile e lo scratch trascinando sullo schermo.
- **FFmpeg incluso** nel programma (solo build LGPL), con licenze e crediti.

### Griglia dei battiti e analisi
- Griglia più precisa: corretto l'errore di tempo 3:2, interpolazione del tempo limitata, nuova logica di tracciamento (Pulse Engine) con tempo che accelera e rallenta.
- **Battiti con la rete neurale Beat This!** e modalità per brano **AUTO / FISSA / FLESS.** (quest'ultima segue un batterista vero battuta per battuta).
- Pulsante **⚓ ANCHOR** sul deck per correggere la griglia in un punto; finestra GRID semplificata (½, ×2, ripristino).
- BPM e tonalità ripristinati quando un brano torna in libreria.

### Suonare
- **PLAY** parte subito anche con SYNC.
- **→0**: rientro graduale del pitch a zero (0,5 punti al secondo).
- **8 hot cue in una pagina**, spenti finché vuoti e accesi del loro colore quando memorizzati.
- Loop: OUT con il loop attivo ne esce invece di accorciarlo; nuove lunghezze del loop.
- Conto alla rovescia **rosso lampeggiante negli ultimi 20 secondi**.
- Cuffie: il ritardo del preascolto si azzera subito; l'anteprima si riapre da sola se il flusso si interrompe.
- Stem su Windows: si parte subito dalla CPU, la GPU si usa solo se è più veloce.

### Aspetto
- **Nuovo aspetto «Dark Club»**: grafite, alluminio e luci viola; piatto con bordo forato e luce che segue la rotazione; manopole metalliche con arco del valore; fader stile Pioneer; CUE e PLAY con ghiera cromata.
- **Skin Industrial e Retro Hardware** con materiali veri (lamiera forata e mandorlata, alluminio spazzolato, pelle) e **piatti motorizzati**.
- **Colore del tema a scelta** con selettore stile Photoshop e anteprima reale dell'interfaccia.
- **Layout per monitor ultrawide** (32:9): deck e mixer a sinistra, libreria a tutta altezza a destra, stem, sampler e FX affiancati.
- Stem con manopole al posto dei fader; manopole dell'EQ più grandi.
- Vista LIBRARY con il riquadro del brano selezionato; finestra Informazioni rifatta.
- Nuovi logo, icona dell'app e schermata di avvio.
- Forme d'onda verticali (vista per lo scratch) e forme d'onda più fluide.
- Diagnostica delle prestazioni (Ctrl+Maiusc+D).

### AUTOMIX
- Coda salvata fra una sessione e l'altra, riordino e aggiunta dei brani trascinandoli; il pulsante Avvia non perde più i clic.

### Documentazione
- Manuale aggiornato (video, griglia, skin, ultrawide), documento tecnico del video, registro delle decisioni e delle licenze, bozze di EULA e informativa sulla privacy, istruzioni per la prova su Windows.

## 0.1.1 — 31 ottobre 2026

Versione di prova dell'aggiornamento automatico (solo Windows).
