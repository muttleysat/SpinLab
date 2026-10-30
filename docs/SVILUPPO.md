# SpinLab DJ - note per chi sviluppa

Questo file NON fa parte del manuale per gli utenti (non finisce nella guida dentro il programma). Vedi anche `docs/ASIO-licenza.md` per gli obblighi legati ad ASIO.

## Compilare il programma

Serve: Git, Rust (rustup.rs) con gli strumenti C++ di Visual Studio, LLVM/clang e l'**SDK ASIO** di Steinberg (non incluso nel progetto per la sua licenza). L'SDK va estratto in `src-tauri/asio-sdk` (la cartella è ignorata da git, non va mai caricata nel repository) in modo che dentro ci siano direttamente `common`, `host`, `driver`…: il percorso in `src-tauri/.cargo/config.toml` è relativo, uguale su tutti i PC. Usare l'SDK 2.3.4 (o più recente) dal sito Steinberg. LLVM si cerca in `C:/Program Files/LLVM/bin`: se è altrove, cambiare `LIBCLANG_PATH` e `CLANG_PATH` nello stesso file. Serve anche FFmpeg nel PATH (formati audio meno comuni).

```
cargo install tauri-cli --version "^2"
git clone https://github.com/muttleysat/SpinLab.git
cd SpinLab\src-tauri
cargo tauri dev        (per provare)
cargo tauri build      (per creare l'installer)
```

I dati dell'utente (impostazioni, libreria, mappatura MIDI, modello degli stem) stanno in `%APPDATA%\com.spinlab.dj` e non sono nel repository.

**«Accesso negato (os error 5)» compilando.** Il programma è ancora aperto e Windows non lo lascia sostituire. Chiudi SpinLab (e le copie rimaste nel Gestore attività).

## Creare l'installer e pubblicare un aggiornamento

Prima volta sul PC: `python tools/fetch_model.py` (mette il modello degli stem in `src-tauri/models`, da dove l'installer lo include).
La DLL dell'encoder MP3 e' gia' in `src-tauri/libs` (si ricostruisce con `tools/build_lame_dll.py`).

Per ogni versione:

1. Cambia il numero di versione in `src-tauri/tauri.conf.json` e in `src-tauri/Cargo.toml`.
2. Se sono cambiate le dipendenze: `python tools/gen_third_party.py` (aggiorna `THIRD-PARTY-LICENSES.md`, che finisce nell'installer).
3. Imposta la chiave di firma (vedi sotto) e crea l'installer: dalla cartella `src-tauri`
   ```
   set TAURI_SIGNING_PRIVATE_KEY=C:\percorso\della\chiave.key
   cargo tauri build --config tauri.release.conf.json
   ```
   Risultato in `src-tauri/target/release/bundle/` (`nsis` e `msi`). Con `tauri.release.conf.json` l'installer include modello, DLL e licenze e crea i file firmati degli aggiornamenti.
4. Crea il file che descrive la versione: `python tools/make_update_manifest.py 0.2.0 https://tuo.sito/spinlab "Cosa c'e' di nuovo"`.
5. Pubblica `latest.json` e `SpinLab_0.2.0_x64-setup.exe` all'indirizzo scelto. Il programma controlla l'indirizzo scritto in `src-tauri/tauri.conf.json` (`plugins.updater.endpoints`).

### Chiave di firma degli aggiornamenti

Gli aggiornamenti sono firmati: il programma accetta solo file firmati con la chiave giusta. La chiave attuale (`%USERPROFILE%\.tauri\spinlab-dev-updater.key`) e' **solo per sviluppo, senza password**. **Prima della prima versione pubblica**:

```
cargo tauri signer generate -w %USERPROFILE%\.tauri\spinlab.key
```

(scegli una password), copia la chiave pubblica stampata in `plugins.updater.pubkey` di `tauri.conf.json` e **fai una copia di sicurezza della chiave privata e della password**: se le perdi, i programmi gia' installati non potranno piu' ricevere aggiornamenti. La chiave privata non va mai nel repository.

L'indirizzo degli aggiornamenti e' ora quello di GitHub Releases del repository: funziona solo se il repository (o la release) e' pubblico. Per un prodotto commerciale conviene un server o uno spazio tuo (per esempio un bucket) e cambiare `endpoints`.

## Icone

Le icone dell'app si generano da un'unica immagine quadrata 1024 x 1024 (con trasparenza):

```
cd src-tauri
cargo tauri icon percorso\icona.png
```

## Aggiornare la guida dentro il programma

Il manuale si scrive in `docs/MANUALE.md`. Dopo ogni modifica:

```
python docs/build_guida.py
```

Genera `ui/guida.html`, che il programma mostra dal pulsante **?**.
