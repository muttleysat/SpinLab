#!/usr/bin/env python3
"""Crea latest.json, il file che i programmi installati leggono per sapere se esiste una versione piu' nuova.

Prima si crea l'installer firmato (vedi docs/SVILUPPO.md); poi:

    python tools/make_update_manifest.py <versione> <indirizzo base dei file> ["note della versione"]

Esempio:
    python tools/make_update_manifest.py 0.2.0 https://download.example.com/spinlab "Nuovo AUTOMIX e correzioni"

Legge src-tauri/target/release/bundle/nsis/SpinLab_<versione>_x64-setup.exe e il suo .sig e scrive latest.json accanto.
Poi si pubblicano sullo stesso indirizzo: latest.json, l'installer .exe (il .sig serve solo qui, il contenuto e' dentro latest.json).
"""
import json
import sys
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
BUNDLE = ROOT / "src-tauri" / "target" / "release" / "bundle" / "nsis"


def main():
    if len(sys.argv) < 3:
        sys.exit(__doc__)
    version, base = sys.argv[1], sys.argv[2].rstrip("/")
    notes = sys.argv[3] if len(sys.argv) > 3 else f"SpinLab {version}"
    exe = BUNDLE / f"SpinLab_{version}_x64-setup.exe"
    sig = exe.with_name(exe.name + ".sig")
    if not exe.exists() or not sig.exists():
        sys.exit(f"manca {exe.name} o la sua firma (.sig) in {BUNDLE}: l'installer va creato con TAURI_SIGNING_PRIVATE_KEY impostata")
    manifest = {
        "version": version,
        "notes": notes,
        "pub_date": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "platforms": {"windows-x86_64": {"signature": sig.read_text(encoding="utf-8").strip(), "url": f"{base}/{exe.name}"}},
    }
    out = BUNDLE / "latest.json"
    out.write_text(json.dumps(manifest, indent=2, ensure_ascii=False), encoding="utf-8")
    print("scritto", out)
    print("da pubblicare in", base + "/:", "latest.json,", exe.name)


if __name__ == "__main__":
    main()
