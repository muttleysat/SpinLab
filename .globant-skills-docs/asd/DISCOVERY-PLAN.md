# Discovery Plan — SpinLab DJ

**Started:** 2026-11-05
**Last updated:** 2026-11-05
**Status:** In Progress

> Piano di esecuzione e registro di avanzamento per la documentazione dell'architettura (ASD).
> Ogni sessione comincia leggendo questo file e finisce aggiornandolo.

---

## Repositories

| # | Name / Service | Path or URL | Tech Stack | Status | Notes |
|---|----------------|-------------|-----------|--------|-------|
| 1 | Applicazione desktop (motore audio, libreria, stem, video) | `src-tauri/` | Rust 2021, Tauri 2, cpal (ASIO su Windows), Symphonia, ONNX Runtime (`ort`: DirectML su Windows, CoreML su macOS), rusqlite (SQLite), sysinfo | In Progress | ~10.600 righe in `src-tauri/src` (12+ moduli). Modulo video: `video.rs` |
| 2 | Interfaccia | `ui/` | HTML/CSS/JavaScript senza framework, WebGL 1 (GLSL ES 1.00) | In Progress | `app.js`, `video*.js`, `video-out.html` (finestra di uscita), `midi.js`, `automix.js`, `cockpit.js`, `mobile.js` |
| 3 | Server licenze e aggiornamenti | `server/` | PHP (libsodium), hosting condiviso | Pending | `api/`, `admin/`, `webhook.php`, `updates/`; descritto in `docs/SERVER-LICENZE.md` |
| 4 | Strumenti di sviluppo | `tools/` | Python | Pending | Modello stem, manifest aggiornamenti, server licenze di prova, banchi di misura |
| 5 | Riferimento (non in produzione) | `PulseEngine/` | C++ | Out of scope | Prototipo di riferimento per la griglia dei battiti, non compilato nell'app |

**Repository structure:** Single repo (monorepo: app desktop + server + strumenti + documentazione). Remoto: `muttleysat/SpinLab`.

---

## Supplementary Inputs

| Type | Source | Provided By | Status | Notes |
|------|--------|-------------|--------|-------|
| Documentazione utente | `docs/MANUALE.md` | Team SpinLab | Provided | Descrive tutte le funzioni (compresa la sezione Video) |
| Note per chi sviluppa | `docs/SVILUPPO.md`, `README.md` | Team SpinLab | Provided | Compilazione, installer, aggiornamenti, stem |
| Documentazione tecnica del video | `docs/VIDEO.md` | Team SpinLab | Provided | Scritta a mano prima di questo ASD; va riconciliata |
| PRD del video | `docs/PRD-video.md` | Team SpinLab | Provided | Rifatto con `g-p-generate-prd` nella sessione 1 |
| Licenze | `docs/LICENZE.md`, `docs/ASIO-licenza.md`, `docs/SERVER-LICENZE.md`, `THIRD-PARTY-LICENSES.md` | Team SpinLab | Provided | |
| Bozze legali | `docs/legale/EULA.md`, `PRIVACY.md` | Team SpinLab | Provided | Da far rivedere prima della vendita |
| Diagrammi | — | — | None | Generati dal codice in questo ASD (Mermaid) |

---

## Section Discovery Status

Ambito della sessione 1 (scelto da noi in assenza di indicazioni diverse): **sistema complessivo ad alto livello** (§1, §2) e **§6 con focus sul modulo video** (integrazioni incluse). Le altre sezioni restano da fare nelle sessioni successive.

| # | Section | Status | Repo / Pass | Assigned Session | Notes |
|---|---------|--------|-------------|-----------------|-------|
| 1 | Context | In Progress | Tutti | 1 | §1.1 richiede input umano: segnato `[TO BE DEFINED]` |
| 2 | Functional Overview | Done (bozza) | Tutti | 1 | Derivata dal manuale e dal PRD |
| 3 | NFRs | Pending | Tutti | 2 | Dedurre dal codice e confermare |
| 4 | Constraints | Pending | Tutti | 2 | |
| 5 | Principles | Pending | — | 2 | **Solo input umano** |
| 6 | Software Architecture | In Progress | Per repo | 1 (video) / 2 (resto) | §6.3.5 e §6.4 scritti per il video con `g-e-integration-contracts`; PBC e stack del resto dell'app solo a livello di quadro |
| 7 | Infrastructure | Pending | Tutti | 3 | App desktop + hosting PHP: da confermare cosa rientra |
| 8 | Data Architecture | Pending | Tutti | 3 | SQLite `library.db`, file stem, cache temporanee |
| 9 | DevOps (CI/CD) | Pending | Tutti | 3 | Non risultano pipeline CI nel repository (da confermare) |
| 10 | Testing | Pending | Tutti | 3 | Test Rust inline (`cargo test --lib`), controlli manuali |
| 11 | Operation & Support | Skipped (N/A) | — | — | Da confermare se in ambito |
| 12 | Digital Transformation | Skipped (N/A) | — | — | **Solo input umano**, da confermare se in ambito |

**Status values:** Pending · In Progress · Done · Blocked · Skipped (N/A)

---

## Human Input Required

| Section | What's needed | Question to ask | Scheduled session | Status |
|---------|--------------|-----------------|------------------|--------|
| §1.1 Business Context | Quale problema risolve SpinLab, per chi, come si vende | "Con parole tue: cos'è SpinLab e che valore dà? Chi sono gli utenti e gli altri interessati (per esempio chi lo vende o lo supporta)? Quali sono i piani di vendita?" | 1 → 2 | Pending |
| §5 Principles | Principi architetturali del team | "Quali principi segue il progetto (per esempio: l'audio non si interrompe mai, senza dipendenze fuori dal sistema, tutto offline)? Ce ne sono di aziendali?" | 2 | Pending |
| §12 Digital Transformation | Contesto organizzativo | "Il tracciamento delle capacità di trasformazione digitale è rilevante per questo progetto?" | TBD | Pending |

---

## Open Questions & Blockers

| ID | Question / Blocker | Affects | Raised | Status |
|----|-------------------|---------|--------|--------|
| Q-01 | Il server PHP (`server/`) e gli strumenti (`tools/`) rientrano nell'ASD o solo l'app desktop? | §6, §7, §8, §9 | 2026-11-05 | Open |
| Q-02 | Esiste una pipeline di build/rilascio automatica (CI) fuori dal repository? Non ne risultano nel codice | §9 | 2026-11-05 | Open |
| Q-03 | Quali piattaforme sono davvero di destinazione oltre a Windows e macOS (Android? Il codice ha rami per Android) | §1, §4, §7 | 2026-11-05 | Open |

---

## Session Log

| Session | Date | Sections worked | What was completed | What's next | Context notes |
|---------|------|-----------------|-------------------|-------------|---------------|
| 1 | 2026-11-05 | Phase 0; §1, §2, §6 (video) | Inventario; ASD.md e DISCOVERY-PLAN.md; §1 e §2 in bozza; §6 con PBC, stack, integrazioni del video (4 contratti) e viste (C4 + 3 sequenze). PRD del video rifatto con `g-p-generate-prd` | Confermare le sezioni con il team; §1.1 e §5 (input umano); poi §3, §4 e le sezioni 7–10 | I contenuti sono dedotti dal codice del branch `video-mixing`; il modulo video non è provato su Windows |

---

## Notes

- Il motore audio non deve mai interrompersi: è la regola da cui discendono molte scelte del modulo video (l'audio è il master, si degrada il video).
- Quando in questo ASD si dice «(verificato)» il dato è stato letto nel codice o misurato; «(da verificare)» indica ciò che non è stato provato (in particolare su Windows).
