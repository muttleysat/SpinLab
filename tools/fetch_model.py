#!/usr/bin/env python3
"""Mette i modelli in src-tauri/models, da dove l'installer li include: quello degli stem (htdemucs.onnx) e i due dei battiti
(Beat This!, models/beat-this). Cosi' il programma installato non scarica niente.

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


BEAT_URL = "https://raw.githubusercontent.com/danigb/beat-this-rs/main/models/"
BEAT = {"mel_spectrogram.onnx": "fdd59e65c515331308e4c8841edf99972deca646bdf6197744c2a5b7755e3de9",
        "beat_this_small.onnx": "a5f8d39d989f31859454ba27afe61c5317ca95e4d9373e6853e5361b8937172f"}


def data_dirs():
    """cartelle dei dati del programma (Windows e macOS), dove i modelli possono essere gia' stati scaricati"""
    return [Path(os.environ.get("APPDATA", "")) / "com.spinlab.dj", Path.home() / "Library" / "Application Support" / "com.spinlab.dj"]


def fetch_beat_this():
    dest_dir = DEST.parent / "beat-this"
    dest_dir.mkdir(parents=True, exist_ok=True)
    for name, h in BEAT.items():
        dest = dest_dir / name
        if dest.exists() and sha(dest) == h:
            print("gia' presente e verificato:", dest); continue
        local = next((d / "models" / "beat-this" / name for d in data_dirs() if (d / "models" / "beat-this" / name).exists()), None)
        if local and sha(local) == h:
            shutil.copy(local, dest); print("copiato da", local); continue
        print("scarico", BEAT_URL + name)
        urllib.request.urlretrieve(BEAT_URL + name, dest)
        if sha(dest) != h:
            dest.unlink(missing_ok=True)
            sys.exit(f"{name}: il file scaricato non corrisponde alla firma attesa")


def main():
    fetch_beat_this()
    DEST.parent.mkdir(parents=True, exist_ok=True)
    if DEST.exists() and sha(DEST) == SHA256:
        print("modello gia' presente e verificato:", DEST)
        return
    local = next((d / "models" / "htdemucs.onnx" for d in data_dirs() if (d / "models" / "htdemucs.onnx").exists()), None)
    if local and sha(local) == SHA256:
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
