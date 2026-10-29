#!/usr/bin/env python3
"""Mette il modello degli stem in src-tauri/models/htdemucs.onnx, da dove l'installer lo include.

Il file (circa 166 MB) NON sta nel repository (.gitignore): prima di creare l'installer si lancia questo script.
Se il modello e' gia' stato scaricato dal programma (cartella dei dati dell'utente) lo copia da li' invece di riscaricarlo.

Uso:   python tools/fetch_model.py
"""
import hashlib
import os
import shutil
import sys
import urllib.request
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
DEST = ROOT / "src-tauri" / "models" / "htdemucs.onnx"
URL = "https://huggingface.co/adowu/htdemucs-onnx/resolve/main/htdemucs_fp16weights.onnx"
SHA256 = "d05c269d0178d2a72ad484b10b11dd370193fc923201c3b27a99f848745db70a"


def sha(p: Path) -> str:
    h = hashlib.sha256()
    with open(p, "rb") as f:
        for b in iter(lambda: f.read(1 << 20), b""):
            h.update(b)
    return h.hexdigest()


def main():
    DEST.parent.mkdir(parents=True, exist_ok=True)
    if DEST.exists() and sha(DEST) == SHA256:
        print("modello gia' presente e verificato:", DEST)
        return
    local = Path(os.environ.get("APPDATA", "")) / "com.spinlab.dj" / "models" / "htdemucs.onnx"
    if local.exists() and sha(local) == SHA256:
        shutil.copy(local, DEST)
        print("copiato da", local)
        return
    print("scarico", URL)
    tmp = DEST.with_suffix(".part")
    urllib.request.urlretrieve(URL, tmp)
    if sha(tmp) != SHA256:
        tmp.unlink(missing_ok=True)
        sys.exit("il file scaricato non corrisponde alla firma attesa: modello cambiato? controllare la licenza prima di aggiornare SHA256")
    tmp.replace(DEST)
    print("scritto", DEST)


if __name__ == "__main__":
    main()
