# Prova minima dell'orologio (Galaxy Watch 8)

Stato (4 novembre 2026): provata su Galaxy Watch8 (Wear OS 16) con S26 Ultra: collegamento, vibrazione a tempo e taglio degli stem funzionano. **Il rientro del drop sulla battuta e il servizio con schermo spento non sono ancora verificati a fondo.** Sviluppo documentato in [ideas/companion-galaxy-watch.md](../ideas/companion-galaxy-watch.md).

Questo file NON fa parte del manuale per gli utenti.

## Cosa fa

- Logo animato all'avvio (come lo splash del programma); un tocco lo salta.
- L'orologio vibra a tempo con il deck selezionato del telefono (impulso più forte sul primo battito della battuta). Anello a 16 tacche e disco centrale che pulsano a tempo.
- Tenendo premuto il tasto centrale escono batteria e basso (il disco conta le battute); al rilascio il telefono li fa rientrare sulla **prossima battuta «1»** della griglia, con un lampo e un impulso di conferma. Se il rientro non arriva entro 4,5 s l'orologio torna alla schermata normale.
- Se il telefono smette di mandare lo stato per più di 1,5 s (app in secondo piano, Bluetooth caduto) l'orologio si ferma e mostra «IN ATTESA DEL BRANO».
- Toccando in alto si apre la **taratura**: ritardo andata-ritorno (RTT), tempo del comando dall'orologio alla pagina e due tasti, `-` e `+`, per l'anticipo della vibrazione (5 ms alla volta, il valore si ricorda).
- Servizio in primo piano (`PulseService`) con attività in corso: tiene vivi collegamento e vibrazione a schermo spento, con la CPU sveglia solo mentre c'è musica, e si ferma dopo 90 s senza musica. Se arriva lo stato con la musica e l'app è chiusa, `WatchListener` prova ad avviarlo; se Wear OS non lo permette mostra una notifica da toccare.

## Come funziona

Orologio e telefono parlano con il Data Layer di Google (Bluetooth). Funziona solo se le due app hanno **stesso nome di pacchetto** (`com.spinlab.dj`) e **stessa firma**; in debug la firma è la chiave `~/.android/debug.keystore` di questo Mac.

- Orologio: progetto [watch/](../watch/), una sola schermata disegnata a mano (niente Compose).
- Orologio, in breve: `Engine` (stato e metronomo), `PulseService` (tiene vivo tutto), `WatchListener` (si sveglia ai messaggi), `RingView` (grafica), `Logo` (marchio generato dall'SVG).
- Telefono: [WatchBridge.kt](../src-tauri/gen/android/app/src/main/java/com/spinlab/dj/WatchBridge.kt) riceve i messaggi e li passa alla pagina; [ui/watch.js](../ui/watch.js) manda ogni 250 ms tempo, fase e stem e applica i comandi come un tocco sullo schermo.
- Sull'orologio un metronomo proprio fa girare la posizione fra un messaggio e l'altro, così un messaggio in ritardo non sposta la vibrazione.

## Prerequisiti

1. Il Watch 8 accoppiato al S26 Ultra (con l'app Galaxy Wearable/Wear OS), entrambi con il Bluetooth acceso.
2. Sull'orologio: Impostazioni › Opzioni sviluppatore › Debug ADB e Debug tramite Wi-Fi.
3. SpinLab **aperto in primo piano** sul telefono, con un brano caricato nel deck selezionato e in riproduzione. Se l'app va in secondo piano la pagina può fermarsi e i comandi si perdono.

## Installazione

```
# orologio (la prima volta: abbina con il codice mostrato sull'orologio)
adb pair IP:PORTA_DI_ABBINAMENTO
adb connect IP:PORTA
source env-android.sh
cd watch && ./gradlew :app:assembleDebug
adb -s IP:PORTA install -r app/build/outputs/apk/debug/app-debug.apk
```

Il telefono va ricostruito e reinstallato come al solito (la build Android di Tauri), perché sono cambiati il manifest, `MainActivity.kt` e `ui/`.

Non usare `./gradlew installDebug` nella cartella `watch`: installerebbe l'app dell'orologio su ogni dispositivo collegato, telefono compreso.

## Cosa misurare

| Misura | Dove si legge | Cosa ci dice |
| --- | --- | --- |
| RTT | riga verde sull'orologio | ritardo del collegamento; l'andata è circa metà |
| Comando | riga sotto, "comando N ms" | dal tocco sull'orologio alla pagina che applica gli stem (non include il cambio dell'audio) |
| Vibrazione a tempo | a mano, col telefono che suona | regolare l'anticipo con `-` / `+` finché vibrazione e cassa coincidono |

Con il telefono su cassa Bluetooth la vibrazione risulterà sfasata: il ritardo dell'audio Bluetooth non è compensato. Provare prima con le cuffie o la cassa a cavo.

## Limiti noti

- Non implementato: pagine Mixer/Pad, Tile e complication, sezione «Orologio» nell'app del telefono (stato e interruttore).
- Il rientro sulla battuta parte da JavaScript con un anticipo stimato di 12 ms (`IPC_LEAD` in `ui/watch.js`): non misurato. Se l'audio rientra un po' presto o tardi, va tarato lì.
- Il deck è sempre quello selezionato sul telefono per FX e stem.
- Tenendo premuto con stem non ancora separati il telefono li separa partendo dal punto di ascolto (circa 1,7 volte il tempo reale, un solo deck): l'audio può interrompersi finché non sono pronti.
- Non verificato: l'apertura automatica dell'app dall'avvio della musica (Wear OS limita l'avvio dallo sfondo), la presenza dell'attività in corso sul quadrante, la vibrazione con lo schermo spento.
- Distribuzione: l'app Android è un APK indipendente, quindi l'orologio si installa solo con `adb` (`tools/watch-adb.sh` abbina e collega il Watch via Wi-Fi). Per utenti normali servirà il Play Store: la documentazione di Google non prevede l'installazione automatica sull'orologio, ma una notifica che invita a installarla.
- In `Engine.startDemo()` c'è una modalità demo (`am start ... --ez demo true`, 14 s) usata solo per fotografare la grafica.
- Se il Bluetooth cade, il Data Layer può instradare i messaggi dalla nuvola di Google: ritardi alti e irregolari. Non è uno scenario di riserva per un uso dal vivo.
