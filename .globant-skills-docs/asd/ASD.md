# Architecture Specification Document (ASD)

**System:** SpinLab DJ
**Version:** 0.1.0 (draft)
**Date:** 2026-11-05
**Author:** Vincenzo Trombetta (con assistenza di Claude)
**Reviewer / Approver:** [TO BE DEFINED]
**Template Version:** 1.9.9

**Discovery Plan:** [DISCOVERY-PLAN.md](./DISCOVERY-PLAN.md)

---

## Source Repositories & Inputs

| Type | Name / Path | Tech Stack | Notes |
|------|-------------|-----------|-------|
| Repository | `src-tauri/` | Rust 2021, Tauri 2, cpal, Symphonia, ONNX Runtime, SQLite | Motore audio, libreria, stem, registrazione, modulo video (`video.rs`) |
| Repository | `ui/` | HTML/CSS/JavaScript, WebGL 1 | Interfaccia, finestra di uscita video (`video-out.html`) |
| Repository | `server/` | PHP, libsodium | Licenze e aggiornamenti; non ancora documentato |
| Repository | `tools/` | Python | Strumenti di sviluppo; non ancora documentato |

**Supplementary inputs:**

| Type | Source | Status |
|------|--------|--------|
| Manuale utente | `docs/MANUALE.md` | Provided |
| Documentazione tecnica del video | `docs/VIDEO.md` | Provided |
| PRD del video | `docs/PRD-video.md` | Provided (rifatto) |
| Note di sviluppo e licenze | `docs/SVILUPPO.md`, `docs/LICENZE.md`, `docs/SERVER-LICENZE.md` | Provided |

---

## About This Document

Questo ASD formalizza l'architettura di un sistema esistente, ricostruita dal codice e dai materiali disponibili. Nella prima sessione l'ambito è il **sistema a livello di quadro** (§1, §2) e il **modulo video in profondità** (§6). Il resto è pianificato in `DISCOVERY-PLAN.md`.

Il documento è una cartella di file Markdown collegati: si caricano solo quelli che servono al compito.

---

## Section Status

| # | Section | Status | Summary |
|---|---------|--------|---------|
| 1 | [Context](./01-context.md) | In Progress | Contesto tecnico e componenti collegati; contesto di business da definire |
| 2 | [Functional Overview](./02-functional-overview.md) | Done (draft) | Aree funzionali e riferimento ai requisiti (PRD video, manuale) |
| 3 | [Non-Functional Requirements](./03-nfr.md) | Pending | — |
| 4 | [Constraints](./04-constraints.md) | Pending | — |
| 5 | [Principles](./05-principles.md) | Pending | Richiede input umano |
| 6 | [Software Architecture](./06-software-architecture.md) | In Progress | Quadro dei componenti, stack, integrazioni e viste del modulo video |
| 7 | [Infrastructure Architecture](./07-infrastructure.md) | Pending | — |
| 8 | [Data Architecture](./08-data-architecture.md) | Pending | — |
| 9 | [DevOps (CI/CD)](./09-devops.md) | Pending | — |
| 10 | [Testing Principles](./10-testing.md) | Pending | — |
| 11 | [Operation and Support](./11-operations-support.md) | Skipped (N/A) | — |
| 12 | [Digital Transformation Capabilities](./12-digital-transformation.md) | Skipped (N/A) | — |

**Legend:** Done · Pending · In Progress · Blocked · Skipped (N/A) · Has open items

---

## Flagged Concerns

| ID | Concern | Type | Affects | Status |
|----|---------|------|---------|--------|
| FC-01 | Il protocollo `asset://` ha ambito `**`: la pagina può leggere qualsiasi file del disco | Security gap | §6.3.5 (WebView) | Open |
| FC-02 | Il modulo video non è provato su Windows (WebView2, encoder hardware, D3D) | Environment mismatch | §6.3.5, §3 | Open |
| FC-03 | Gli elenchi delle transizioni leggere sono duplicati in due file (`video-perf.js`, `video-out.html`) | Undocumented dependency | §6.3.4 | Open |
| FC-04 | Non risulta una pipeline CI nel repository | Tech stack gap | §9 | Open |
| FC-05 | `recorder::tests::mp3_is_written_and_valid` fallisce già prima del modulo video | Stale docs conflict | §10 | Open |
| FC-06 | La registrazione MP4 è verificata con un test automatico della pipeline, ma non con il dialogo né con prove lunghe, e non su Windows | Environment mismatch | §6.3.5 (FFmpeg), §10 | Open |

**Concern types:** NFR-Constraint conflict · Tech stack gap · Integration risk · Undocumented dependency · Zombie service (no active callers) · Stale docs conflict · Environment mismatch · Security gap

---

## Open Items

| Section | Item | Priority |
|---------|------|----------|
| §1.1 | Contesto di business (problema, utenti, piani di vendita) | High |
| §5 | Principi architetturali del team | High |
| §6.3.5 | Contatti e responsabilità di FFmpeg come dipendenza (build, licenza) | Medium |
| §6.3.5 | Prove su Windows delle integrazioni WebView e FFmpeg | High |

---

## How to Use This ASD

| Task | Load these files |
|------|-----------------|
| Implementation planning | `06-software-architecture.md` + relevant section |
| Understanding requirements | `02-functional-overview.md` |
| Checking discovery progress | `DISCOVERY-PLAN.md` |
