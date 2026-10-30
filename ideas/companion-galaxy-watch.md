# SpinLab per Galaxy Watch 8: idea e proposte

Aggiornato il 30 ottobre 2026.

Prototipi interattivi delle 12 proposte: <https://claude.ai/artifact/WyzUAcdfyHw1NTHkcPvvqP> (link privato, visibile solo a chi lo ha creato finché non viene condiviso).

## L'idea

La companion app trasforma il Galaxy Watch 8 in un telecomando e in un display per SpinLab: audio, stem e libreria restano sul tablet, l'orologio mostra lo stato dei deck e manda i comandi.

Serve al DJ quando ha le mani occupate o lo sguardo altrove: lanciare un cue, saltare 8 battute, silenziare la voce di un brano, vedere quanto manca al prossimo mix, senza toccare il tablet. Telecomandi simili esistono già per Apple Watch (djay) e per Traktor; nelle ricerche fatte non ne ho trovati per Wear OS né per altri programmi DJ.

Punto di partenza: SpinLab gira già su desktop e sul Galaxy Tab S10+ (port Android, PR #2 già unita a main). L'orologio non fa calcoli pesanti: invia comandi e mostra stato.

## Come l'orologio parla con SpinLab

L'orologio non parla con SpinLab direttamente: serve un piccolo server di controllo nel backend Rust, raggiunto via Wi-Fi locale (WebSocket) o Bluetooth LE.

- **App dell'orologio:** Tauri non gira su Wear OS, quindi è un progetto Android separato, in Kotlin con Compose for Wear OS; Tile e complication danno l'accesso rapido dal quadrante.
- **Accoppiamento:** il Galaxy Watch si accoppia a un telefono, non a un tablet, quindi l'API ufficiale orologio-telefono (Wearable Data Layer) probabilmente non lo collega al Tab S10. Da verificare con l'orologio reale.
- **Percorso consigliato:** WebSocket sulla Wi-Fi locale, più semplice da sviluppare e provare; Bluetooth LE come alternativa senza rete.
- **Server di controllo:** riceve JSON che corrispondono ai comandi esistenti (`deck_cmd`, `mixer_cmd`, `deck_stems`, `fx_set`) e invia periodicamente lo stato di `get_state`. Si prova da un browser e servirebbe anche a telefoni e telecomandi web.
- **Sicurezza:** il tablet mostra un QR con un codice di accoppiamento; senza, chiunque sulla stessa rete potrebbe controllarlo.

Il ritardo di rete e di vibrazione non è stato misurato.

## Le 12 proposte

Sono dodici schermate da 432 px (un quadrante da circa 44 mm), ognuna con un prototipo interattivo. I dati nei prototipi (brani, BPM, chiavi, tempi) sono di esempio.

| # | Proposta | Cosa fa | Origine | Dal backend serve |
| --- | --- | --- | --- | --- |
| 1 | Telecomando | Un deck alla volta: play, cue, sync, tempo rimanente | Telecomandi djay e Kontrol for Traktor | Comandi esistenti (`deck_cmd`) |
| 2 | Battito | Anello che pulsa a tempo, vibrazione sul battito, tap tempo, ±0,1 BPM | Metronomi a vibrazione per orologio | Fase e BPM in tempo reale: da esporre |
| 3 | Ghiera | Un solo controllo grande per crossfader, volumi e filtro | Idea nostra (ghiera fisica del Watch8 Classic) | Comandi esistenti (`mixer_cmd`) |
| 4 | Scaletta | Brano in corso, conto alla rovescia del mix, coda, Automix | Idea nostra, dall'Automix di SpinLab | Coda Automix: da esporre |
| 5 | Pad | Hot cue, loop e roll; tocco prolungato = elimina con conferma | Kontrol for Traktor (hot cue, loop) | Comandi esistenti (`hotcue`, `loop_*`) |
| 6 | Stem | Mute e solo per voce, batteria, basso, altro | Idea nostra, dagli stem di SpinLab | `deck_stems` esiste; stato degli stem: da esporre |
| 7 | Tre schermate | Deck A, Mixer, Deck B con swipe, crossfader trascinabile | djay per Apple Watch | Comandi esistenti |
| 8 | Salto battute | Salti di 1, 2, 4, 8, 16 battute avanti e indietro | djay (8 battute), Kontrol for Traktor (1–8) | Comando `beatjump` esistente |
| 9 | Effetti | Eco momentaneo, tempo 1/4–2, blocca, mix | djay (effetti da polso) | `fx_set` esistente |
| 10 | Mix a mano | Crossfader a trascinamento, dissolvenza automatica 4 o 8 battute, taglio | Critica al crossfader a frecce di djay | Crossfader esistente; dissolvenza automatica: da verificare |
| 11 | Compatibili | Solo i brani compatibili per chiave e BPM, caricabili con un tocco | Critica allo sfogliare migliaia di brani da polso | BPM e chiave già analizzati; ricerca di compatibilità: da aggiungere |
| 12 | Quadrante | Ora, set e BPM, anelli dei due deck, REC, mix tra | Idea nostra, per quadrante e complication Wear OS | Stato esistente (`get_state`); REC: da verificare |

## Ricerca e idee nostre

Le proposte 7–11 si appoggiano a una sola recensione letta per intero; le altre sono idee nostre. Fonti raccolte il 30 ottobre 2026, ma datate: l'app djay per Apple Watch è del 2015.

**Pagina aperta e letta**

- [Digital DJ Tips: djay su Apple Watch](https://www.digitaldjtips.com/video-djing-on-an-apple-watch/). Tre schermate scorrevoli (deck sinistro, deck destro, mixer), play con anello di avanzamento, salti di 8 battute, loop da 1/16 di battito a 8 battute, effetti, sync e crossfader.
- Stessa recensione, critiche: il crossfader è "fiddly" perché si tocca con le frecce, sfogliare migliaia di brani è "a non-starter", non si correggono i mix a mano, serve l'iPhone vicino. Giudizio finale: bello da mostrare, non per un uso "serio".

**Solo da risultati di ricerca, pagine non aperte (da considerare approssimative)**

- [Kontrol for Traktor](https://apps.apple.com/us/app/kontrol-for-traktor/id1045782401): controlla da orologio effetti, loop, hot cue, crossfader e salti di 1–8 battute.
- [PocketBeat](https://mwm.ai/apps/pocketbeat-vibrating-metronome/6775906865) e altri metronomi a vibrazione per orologio: tap tempo e battito silenzioso, indipendenti dal programma DJ.

**Idee nostre, senza riscontro nelle ricerche**

- Ghiera (3), Scaletta (4), Stem (6), Quadrante (12), timer del set e indicatore REC.
- Non ho trovato un telecomando da polso per Serato o rekordbox, né un'app per Wear OS di djay, né alcun orologio con vibrazione sincronizzata col software DJ: non so se esistano.

## Raccomandazione

Il primo prototipo dovrebbe essere un'unica app a pagine scorrevoli: Telecomando (1), Mixer con crossfader a trascinamento (10), Pad (5) e Stem (6), con il Quadrante (12) come colpo d'occhio sul quadrante.

1. Server di controllo nel tablet con QR di accoppiamento, provato da un browser.
2. App minima per l'orologio: connessione, play, cue, sync, brano e tempo rimanente.
3. Pagine Mixer, Pad e Stem.
4. Tile e complication (Quadrante).
5. Estensioni: Compatibili (11), Salto battute (8) ed Effetti (9).
6. Battito (2) per ultimo, dopo aver misurato il ritardo di rete.

Ghiera (3) e Scaletta (4) restano in attesa: la prima dipende dal modello di orologio, la seconda dalla coda dell'Automix da esporre.

## Rischi e domande aperte

Nessuno dei prototipi è stato provato su un orologio reale e l'accoppiamento orologio-tablet non è verificato: sono i due punti da chiarire prima di scrivere codice.

- Il ritardo di rete e della vibrazione non è misurato: Battito (2) potrebbe risultare fuori tempo.
- Dimensioni e leggibilità vanno controllate sul Galaxy Watch 8 vero (40 o 44 mm).
- Le proposte 4, 10, 11 e 12 richiedono funzioni da aggiungere o verificare nel backend: coda dell'Automix, dissolvenza automatica, ricerca dei brani compatibili, REC.
- Le fonti sono datate (2015) e in parte non aperte: vanno rilette prima di prendere decisioni.
- Domande aperte: a cosa deve servire soprattutto (telecomando, display, vibrazione)? Hai un telefono Android per i test? Quale modello di Galaxy Watch 8?
