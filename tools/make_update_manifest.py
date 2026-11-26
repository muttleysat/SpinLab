#!/usr/bin/env python3
"""Crea latest.json, il file che i programmi installati leggono per sapere se esiste una versione piu' nuova.

Prima si crea l'installer firmato (vedi docs/SVILUPPO.md); poi:

    python tools/make_update_manifest.py <versione> ["note della versione"] [indirizzo base dei file]

Esempio:
    python tools/make_update_manifest.py 0.2.0 "Nuovo AUTOMIX e correzioni"

L'indirizzo base di partenza e' la cartella updates del server delle licenze (BASE sotto). latest.json e l'installer vengono
copiati anche in server/updates/, pronti da caricare via FTP nella cartella updates dell'hosting.

Legge gli installer firmati presenti su questo computer (Windows: bundle/nsis/SpinLab_<versione>_x64-setup.exe + .sig; macOS:
bundle/macos/SpinLab.app.tar.gz + .sig) e scrive server/updates/latest.json, unendo le piattaforme gia' presenti per la stessa versione.
Poi si pubblicano sullo stesso indirizzo: latest.json, l'installer .exe (il .sig serve solo qui, il contenuto e' dentro latest.json).
"""
import json
import shutil
import sys
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
BUNDLE = ROOT / "src-tauri" / "target" / "release" / "bundle" / "nsis"
OUT = ROOT / "server" / "updates"
BASE = "https://www.djtrombetta.it/server/updates"


MAC = ROOT / "src-tauri" / "target" / "release" / "bundle" / "macos"


def platforms_found(version, base):
    """installer firmati presenti su questo computer: Windows (nsis) e/o macOS (app.tar.gz). Ognuno si crea sul suo sistema."""
    found, files = {}, []
    exe = BUNDLE / f"SpinLab_{version}_x64-setup.exe"
    if exe.exists() and exe.with_name(exe.name + ".sig").exists():
        found["windows-x86_64"] = {"signature": exe.with_name(exe.name + ".sig").read_text(encoding="utf-8").strip(), "url": f"{base}/{exe.name}"}
        files.append((exe, exe.name))
    tgz = MAC / "SpinLab.app.tar.gz"
    if tgz.exists() and tgz.with_name(tgz.name + ".sig").exists():
        name = f"SpinLab_{version}_aarch64.app.tar.gz"      # nome con la versione sul server
        found["darwin-aarch64"] = {"signature": tgz.with_name(tgz.name + ".sig").read_text(encoding="utf-8").strip(), "url": f"{base}/{name}"}
        files.append((tgz, name))
    return found, files


def main():
    if len(sys.argv) < 2:
        sys.exit(__doc__)
    version = sys.argv[1]
    notes = sys.argv[2] if len(sys.argv) > 2 else f"SpinLab {version}"
    base = (sys.argv[3] if len(sys.argv) > 3 else BASE).rstrip("/")
    found, files = platforms_found(version, base)
    if not found:
        sys.exit("nessun installer firmato trovato: va creato con TAURI_SIGNING_PRIVATE_KEY impostata (Windows: bundle/nsis/*.exe + .sig; "
                 "macOS: bundle/macos/SpinLab.app.tar.gz + .sig)")
    OUT.mkdir(parents=True, exist_ok=True)
    # Windows e Mac si compilano su computer diversi: se in server/updates c'e' gia' il manifest della stessa versione, si aggiungono le
    # piattaforme a quelle che c'erano (cosi' il secondo computer non cancella il primo)
    platforms = {}
    prev = OUT / "latest.json"
    if prev.exists():
        try:
            old = json.loads(prev.read_text(encoding="utf-8"))
            if old.get("version") == version:
                platforms.update(old.get("platforms", {}))
        except Exception:
            pass
    platforms.update(found)
    manifest = {
        "version": version,
        "notes": notes,
        "pub_date": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "platforms": platforms,
    }
    prev.write_text(json.dumps(manifest, indent=2, ensure_ascii=False), encoding="utf-8")
    for src, name in files:
        shutil.copy2(src, OUT / name)
    print("scritto", prev, "con le piattaforme:", ", ".join(platforms))
    print("pronti in", OUT, ": da caricare via FTP nella cartella updates dell'hosting ->", base + "/")
    for _, name in files:
        print(" ", name)
    print("  latest.json (per ultimo, cosi' i programmi vedono la versione solo a installer gia' disponibile)")


if __name__ == "__main__":
    main()
