# Server delle licenze (hosting Aruba)

Documento per lo sviluppatore, non per gli utenti. Il codice è in `server/`; la parte nel programma è `src-tauri/src/license.rs`.

## Come funziona

1. Il cliente compra (piattaforma di pagamento) → la piattaforma chiama `webhook.php` → il server crea una **chiave** `SPIN-XXXXX-XXXXX-XXXXX-XXXXX` e la manda per email.
2. Nel programma il cliente incolla la chiave: il programma manda al server chiave + **impronta del computer** (SHA-256 dell'identificativo di sistema, il server non vede l'identificativo vero).
3. Il server controlla (chiave valida, non revocata, posti liberi: 2 computer) e risponde con un **certificato firmato Ed25519** legato a quel computer.
4. Il programma verifica la firma da solo con la **chiave pubblica incorporata** (`src-tauri/license_pubkey.txt`): funziona anche offline. Il certificato dura 30 giorni; ogni avvio (dopo 15 s) prova a rinnovarlo in silenzio. Se manca la rete c'è una tolleranza di altri 14 giorni, poi chiede di collegarsi.
5. Se la licenza viene **revocata** (rimborso o dal pannello), al prossimo rinnovo il programma si de-registra.
6. Il cliente può **disattivare** il computer dalle Informazioni (libera un posto) e registrarsi su un altro.

Senza registrazione il programma non carica brani. Le build di sviluppo (`cargo tauri dev`) e quelle mobili non hanno obblighi; per provare il blocco in sviluppo: `SPINLAB_FORCE_LICENSE=1`.

## Una volta sola, sul tuo PC

```
pip install cryptography bcrypt
python tools/license_setup.py keys    # coppia di chiavi di firma
python tools/license_setup.py hash    # hash della password del pannello admin
```

- `keys` mette la chiave **privata** in `~/.spinlab/license-private.key` (non viene mai mostrata né messa nel progetto) e la **pubblica** in `src-tauri/license_pubkey.txt` (va nel repository, è compilata nel programma). Non rifarlo mai dopo la prima vendita: invaliderebbe tutte le licenze emesse. **Fai una copia di sicurezza della chiave privata fuori dal PC** (chiavetta, gestore di password): se la perdi non puoi più emettere licenze compatibili con i programmi già venduti.
- `hash` stampa il valore per `admin_hash`.

## Installazione su Aruba (FTP e pannello)

1. **Dominio e HTTPS**: serve un indirizzo con certificato valido (Aruba offre Let's Encrypt gratuito dal pannello). Scegli un sottodominio, per esempio `licenze.tuodominio.it`.
2. **Database**: dal pannello crea un database MySQL (nome, utente, password) e da phpMyAdmin importa `server/inc/schema.sql`.
3. **Configurazione**: copia `server/inc/config.example.php` in `server/inc/config.php` e compila database, `sign_key_hex` (contenuto di `license-private.key`), `admin_hash`, `webhook_secret` (una stringa casuale lunga, la stessa che metterai nella piattaforma di vendita), `mail_from` (un indirizzo del tuo dominio, perché le email arrivino e non finiscano in spam).
4. **Carica via FTP** il contenuto di `server/` nella cartella web del sottodominio: `api/`, `admin/`, `inc/`, `webhook.php`. `inc/config.php` si carica solo da qui, non è nel repository. PHP 7.2 o più recente (libsodium è di serie).
5. **Verifica**: aprendo `https://licenze.tuodominio.it/inc/config.php` nel browser deve dare **errore 403** (se mostra una pagina bianca o altro, `inc/.htaccess` non è attivo: sposta `inc` fuori dalla cartella web e cambia i `require` nei file). `https://…/admin/` deve chiedere la password.
6. **Programma**: compila con l'indirizzo vero: `SPINLAB_LICENSE_URL=https://licenze.tuodominio.it/api` (oppure cambia la costante `API` in `license.rs`). Stessa cosa per l'installer: la variabile va impostata prima di `cargo tauri build`.
7. **Prova**: dal pannello admin crea una licenza di prova, registra il programma, controlla che compaia il computer nell'elenco, disattiva e riprova.

## Pannello admin (`/admin/`)

Elenco delle licenze con cerca per chiave, email, nome, ordine. Per ogni licenza: computer attivati (nome, versione, ultimo collegamento), **Revoca / Riattiva**, **Azzera computer** (libera tutti i posti, utile se il cliente ha cambiato PC senza disattivare), **Rimanda email**, **Imposta max** computer, **Elimina**. In alto si crea una licenza a mano (regali, assistenza, recensori) con invio email opzionale.

## Vendite automatiche (webhook)

`webhook.php` accetta le vendite con firma HMAC-SHA256 nell'intestazione `X-Signature`:

- **Lemon Squeezy**: pronto (eventi `order_created` e `order_refunded`; il rimborso revoca la licenza).
- **Paddle / Stripe / PayPal**: usano altre firme; si adatta solo la funzione `verify_signature()` e la lettura dei campi.
- **Generico**: `{"event":"order_created","order_id":"…","email":"…","name":"…"}`.

Una stessa vendita ricevuta due volte non crea due chiavi. Dimmi quale piattaforma scegli e lo rifinisco su quella.

## Sicurezza e limiti onesti

- La chiave privata sta solo sul server (e nella tua copia di sicurezza). Chi la ottiene può fabbricare licenze: non va mai in git, in email o in chat.
- Il blocco è **dissuasivo, non inviolabile**: il programma gira sul PC del cliente e un esperto può modificarlo. Serve a far comprare chi altrimenti passerebbe la chiave, non a fermare un cracker.
- Tentativi: più di 15 chiavi sbagliate in un'ora dallo stesso IP vengono rallentati.
- **Privacy (GDPR)**: il server conserva email, nome, nome del computer, impronta del computer, indirizzo IP e data dell'ultimo collegamento. Va scritto nell'informativa privacy del prodotto, con lo scopo (gestione della licenza) e come chiedere la cancellazione (basta «Elimina» dal pannello).
- Backup: esporta ogni tanto il database da phpMyAdmin.
- Gli aggiornamenti automatici possono stare sullo stesso hosting (`latest.json` e gli installer in una cartella `updates/`), cambiando l'indirizzo in `tauri.conf.json`: da fare quando decidi dove pubblicare.

## Prove senza hosting

`python tools/mock_license_server.py` avvia un server finto con la stessa interfaccia, che firma con la tua chiave privata. Per il test completo: compila con `SPINLAB_LICENSE_URL=http://127.0.0.1:8765/api` e lancia `python tools/test.py license_live --ignored` (con `SPINLAB_FORCE_LICENSE=1`).
