# SpinLab Android: layout «Cockpit» (specifiche)

Stato: fasi 1 e 2 implementate (non provate su dispositivo), fase 3 da fare. Prototipo di riferimento: <https://claude.ai/artifact/DHgXnnMvibpFCNSvXmg6un> (artboard «B · Cockpit orizzontale», link privato).

Questo file NON fa parte del manuale per gli utenti (non finisce nella guida dentro il programma).

## Obiettivo

Far usare SpinLab su un telefono Galaxy S26 Ultra tenuto in orizzontale con due pollici: un pollice per deck, mixer al centro. Il layout desktop (1240×940 scalato in blocco) su un telefono diventerebbe illeggibile (scala ~0,44): serve un layout dedicato, non una versione rimpicciolita.

Fuori ambito: tablet (Galaxy Tab S10+ resta sul layout attuale), layout verticali (prototipi A, C, D), companion per orologio.

## Dispositivo di riferimento

- Galaxy S26 Ultra, 6,9″. Con le impostazioni di serie la WebView in orizzontale misura **697×330 dp** (dpr 3, barre di sistema escluse); con lo zoom dello schermo al minimo circa 915×412.
- Altezza minima da supportare: 320 dp. Larghezza minima: 640 dp.
- **Variante compatta** (altezza ≤ 380 o larghezza ≤ 780, quindi il telefono di serie): il piatto non c'è (sarebbe largo 70 dp) e la waveform fa da piatto per lo scrub; 4 pad su una riga; mixer da 176 dp con manopole ridotte; niente PFL nella barra (restano sui canali) né QUANT accanto ai pad (resta nel menu del deck). Il pitch bend sul bordo del piatto (G9) vale solo nella variante larga.
- Il motore audio e il backend Rust non cambiano: la modifica è solo di interfaccia (`ui/`) più il blocco dell'orientamento in `AndroidManifest.xml`.

## Quando si attiva

La modalità Cockpit si attiva (classe `cockpit` sul `body`) quando **tutte** queste condizioni sono vere:

1. User agent Android;
2. finestra in orizzontale (larghezza > altezza);
3. altezza finestra < 600 dp.

Per provarla da browser desktop: parametro `?cockpit` nell'indirizzo. Su tablet (altezza ≥ 600 dp) e in finestra mobile Samsung ampia resta il layout attuale. La classe si ricalcola a ogni cambio di misura. In modalità Cockpit la scala `--k` vale 1 (nessun ridimensionamento in blocco), quindi i canvas si disegnano alla loro misura reale.

L'app è bloccata in orizzontale (`sensorLandscape`) finché non esistono i layout verticali.

## Scelta dell'interfaccia

In **Impostazioni › INTERFACCIA** (ingranaggio nella barra) si sceglie fra:

| Scelta | Cosa mostra |
| --- | --- |
| Automatica (predefinita) | Cockpit sui telefoni Android in orizzontale, desktop altrove |
| Desktop | Il layout di sempre, scalato in blocco |
| Cockpit | Cockpit alla misura reale dello schermo: sul telefono di serie la variante compatta, senza piatto |
| Cockpit prototipo B | L'artboard «B · Cockpit orizzontale» del canvas di Claude Design: piatti, pad 2×2, riga SYNC + LOOP, mixer con HI/MID/LOW. Disegnato a 892×412 e scalato in blocco: sul telefono di serie testi e pulsanti sono circa il 78% |

La stessa scelta si fa dal pulsante INTERFACCIA nella barra, accanto all'ingranaggio, sempre visibile (anche nel layout desktop, per poter tornare al Cockpit dal telefono): mostra l'interfaccia attiva e al tocco apre il menu con le quattro voci.

La scelta si salva nel browser della WebView (`localStorage`, chiave `spinlab.ui`) e vale subito. Per provarle da browser: `?cockpit` o `?proto` nell'indirizzo.

## Struttura della schermata

Dall'alto in basso, tutto a tutta finestra:

| Zona | Altezza | Contenuto |
| --- | --- | --- |
| Barra | 36 dp | Marchio, pulsante RECORD, maniglia LIBRERIA, indicatore REC, CPU, MASTER, ingranaggio impostazioni |
| Palco | resto | Tre colonne: deck 1 · mixer (212 dp) · deck 2 |

La barra nasconde: LIBRARY, MIXER e SETTINGS della navigazione (le impostazioni si aprono dall'ingranaggio), orologio, CUE MIX e volume cuffie (tornano in fase 2; il preascolto PFL resta su ogni deck e sul canale), pulsanti informazioni e guida.

### Deck (340 dp circa l'uno)

Dall'alto: testata compatta (numero, titolo, artista, BPM, tempo rimanente), waveform ingrandita a tutta larghezza (la panoramica si nasconde), poi tre colonne:

- **Deck 1:** pitch (44 dp) · piatto · controlli (112 dp).
- **Deck 2:** specchiato: controlli · piatto · pitch.
- **Piatto:** cerchio che occupa l'altezza disponibile; scrub col dito come oggi.
- **Controlli:** riga CUE + PLAY (48 dp), griglia 2×2 di pad (min. 44 dp), riga loop ‹ N › (36 dp), chip modalità pad sopra i pad. SYNC sta in fondo alla colonna del pitch.

Il pitch ha il solo fader e il pulsante portata nascosto (si cambia dal tocco prolungato sul fader, fase 2). KEY, GRID e QUANT escono dal deck: si raggiungono dal menu del piatto (fase 2).

### Mixer (212 dp)

Per canale: filtro, HI, MID, LOW (manopole 38 dp), fader volume con meter. Sotto, crossfader orizzontale alto 46 dp. Nascosti in fase 1: AUTO GAIN, AUTO STEM, selettore FX (tornano nel cassetto FX di fase 2).

### Libreria a tendina

Pannello a tutta larghezza che scende dal bordo alto sopra i deck; i deck restano sotto e continuano a suonare. Contiene esplora cartelle, ricerca e tabella brani come oggi. Il pannello laterale (AUTOMIX, SAMPLER, DRUMS, STEMS) non è raggiungibile in fase 1.

## Gesture

| # | Gesture | Dove | Effetto | Fase |
| --- | --- | --- | --- | --- |
| G1 | Swipe giù dal bordo alto, o tap sulla maniglia | Barra | Apre la libreria | 1 |
| G2 | Swipe su sulla tendina, o tap sulla maniglia | Libreria | La chiude | 1 |
| G3 | Tieni premuto (300 ms) o trascina il chip modalità | Pad | Menu radiale: su Hot cue, destra Beat jump, giù Loop roll, sinistra Freno/Reverse; si sceglie rilasciando nella direzione | 1 |
| G4 | Tap breve sul chip modalità | Pad | Menu a tendina esistente | 1 |
| G5 | Pizzica due dita | Mixer | Si chiude a soli fader + crossfader (pizzico verso dentro) o si riapre con EQ (verso fuori) | 1 |
| G6 | Tap con due dita | Manopola | Riporta la manopola al valore iniziale (equivale al doppio clic) | 1 |
| G7 | Trascina verticale, poi di lato | Manopola | Regola il valore; spostandosi di lato oltre 24 dp la regolazione diventa fine (1/4 della velocità) | 2 |
| G8 | Scrub | Piatto, waveform | Come oggi | 1 (già presente) |
| G9 | Trascina lungo il bordo esterno del piatto (oltre l'82% del raggio) | Piatto | Senso orario accelera, antiorario frena (comando `nudge`, ±50% al massimo); al rilascio torna normale | 2 |
| G10 | Tap sul numero del deck | Testata | Menu: griglia, tonalità, quantize, portata pitch, carica, espelli (al posto del tocco prolungato sul fader, che farebbe saltare il pitch) | 2 |
| G11 | Tap sul pulsante PANNELLO, o swipe a sinistra dal bordo destro | Barra, bordo | Cassetto con AUTOMIX, SAMPLER, DRUMS, STEMS e FX; si chiude con swipe a destra sulle schede | 2 |

Regole comuni: bersagli di tocco ≥ 44 dp (eccezione: manopole 38 dp, ma con area di tocco estesa a 44 dp con padding); nessun gesto di sistema in conflitto: si usa `touch-action: none` sugli elementi che trascinano; i gesti a due dita non si attivano se un dito è già su un pad o sul piatto.

## Fasi

**Fase 1 (questo branch).** Rilevamento Cockpit, layout, libreria a tendina, G1–G6, blocco orizzontale. Verifica con `?cockpit` da browser (layout) e su dispositivo (gesti).

**Fase 2 (fatta).** Cassetto del pannello (stem, FX, sampler, drums, automix), selettore FX nel mixer, CUE MIX e volume cuffie nella barra, menu del deck, regolazione fine, pitch bend.

**Fase 3.** Messa a punto su dispositivo: misura della latenza del tocco su piatto e scrub, dimensione dei bersagli, consumo di batteria, prova dal vivo.

## Rischi e punti aperti

- **Latenza dello scratch col dito.** Dipende dal ritardo di tocco della WebView di Tauri; da misurare su dispositivo prima di promettere lo scratch (G8/G9).
- **Altezza.** Con 412 dp (meno la barra di stato) la testata e la waveform sono strette: se non basta si può togliere la testata durante la riproduzione.
- **Swipe dal bordo destro.** Può scontrarsi con il gesto Indietro di Android: per questo c'è anche il pulsante PANNELLO.
- **Pitch bend.** Il valore resta finché il dito è sul bordo (non torna da solo se il dito si ferma); da tarare su dispositivo.
- **Pad e hot cue.** Il pad scatta subito alla pressione; per questo il menu radiale è sul chip modalità e non sul pad.
- **Mano sinistra.** Il layout assume il deck 1 a sinistra come su desktop; nessuna inversione per mancini in fase 1.
- Layout verificato sul telefono (697×330) e in browser a 915×412; i gesti non sono ancora stati provati a mano.
- **Pizzico sul mixer.** Se le dita partono sopra le manopole queste possono muoversi di poco prima che il pizzico venga riconosciuto.
