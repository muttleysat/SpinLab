#!/usr/bin/env python3
"""Prepara FFmpeg da includere nell'installer di SpinLab (sidecar di Tauri).

FFmpeg viene distribuito **solo in versione LGPL** (senza x264, x265, fdk-aac e altri componenti GPL o non liberi): cosi' gli obblighi
si limitano a quelli della LGPL (testo della licenza, avviso, sorgenti, possibilita' di sostituirlo). Vedi docs/FFMPEG.md.

Uso, dalla cartella del progetto:
    python3 tools/fetch_ffmpeg.py windows        # scarica la build LGPL per Windows (non serve compilare)
    python3 tools/fetch_ffmpeg.py macos-arm64    # compila dai sorgenti ufficiali per Mac Apple Silicon
    python3 tools/fetch_ffmpeg.py macos-x64      # idem per Mac Intel (serve nasm)

Risultato (cartella src-tauri/binaries, ignorata da git perche' i file sono grandi):
    ffmpeg-<triple>[.exe]                  il programma, col nome che Tauri si aspetta per i sidecar
    licenses/FFMPEG-LICENSE.txt            testo della licenza di QUESTA build (LGPL 2.1 oppure LGPL 3), preso dai sorgenti ufficiali
    licenses/ffmpeg/COPYING.*              tutti i testi: LGPL 2.1, LGPL 3 e GPL 3 (la LGPL 3 si appoggia alla GPL 3)
    licenses/FFMPEG-BUILD-<piattaforma>.txt  versione, origine, sorgenti e configurazione di questa build (per l'offerta dei sorgenti)

Poi si compila l'installer con la configurazione di rilascio (src-tauri/tauri.release.conf.json), che include il sidecar.
"""
import hashlib
import os
import re
import shutil
import subprocess
import sys
import tarfile
import tempfile
import urllib.request
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
BIN = ROOT / "src-tauri" / "binaries"
LIC = BIN / "licenses"

# versione dei sorgenti ufficiali (macOS) e impronta SHA-256 del pacchetto: si aggiorna insieme quando si cambia versione
SRC_VERSION = "9.0.2"
SRC_URL = f"https://ffmpeg.org/releases/ffmpeg-{SRC_VERSION}.tar.xz"
SRC_SHA256 = "8c3850283eb25fa026482078a04051e0be17347b09ef81a0849bec15a96e002e"  # calcolata al primo scarico da ffmpeg.org; ricontrollare con la firma .asc quando si cambia versione

# build LGPL per Windows di BtbN (https://github.com/BtbN/FFmpeg-Builds): ramo di rilascio 9.0, eseguibile unico
WIN_URL = "https://github.com/BtbN/FFmpeg-Builds/releases/download/latest/ffmpeg-n9.0-latest-win64-lgpl-9.0.zip"
WIN_SUMS = "https://github.com/BtbN/FFmpeg-Builds/releases/download/latest/checksums.sha256"

# opzioni di compilazione LGPL: nessun componente GPL o non libero e nessuna libreria presa per caso dal sistema
MAC_CONFIGURE = [
    "--disable-gpl", "--disable-nonfree", "--disable-autodetect", "--disable-doc",
    "--disable-ffplay", "--disable-ffprobe", "--enable-static", "--disable-shared",
    "--enable-videotoolbox", "--enable-audiotoolbox", "--enable-zlib", "--enable-bzlib", "--enable-iconv",
    "--extra-libs=-liconv",   # su macOS la libreria di conversione dei caratteri va collegata a mano con --disable-autodetect
]


def sha256(path: Path) -> str:
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


def download(url: str, dest: Path):
    """Scarica un file: con curl se c'e' (piu' robusto con i server che chiudono le richieste di Python), con tre tentativi."""
    print(f"scarico {url}")
    for attempt in range(3):
        try:
            if shutil.which("curl"):
                subprocess.check_call(["curl", "-L", "--fail", "--silent", "--show-error", "--retry", "2", "-o", str(dest), url])
            else:
                req = urllib.request.Request(url, headers={"User-Agent": "Mozilla/5.0 SpinLab-fetch"})
                with urllib.request.urlopen(req) as r, open(dest, "wb") as f:
                    shutil.copyfileobj(r, f)
            return
        except Exception as e:  # noqa: BLE001
            print(f"tentativo {attempt + 1} non riuscito: {e}")
    sys.exit(f"download non riuscito: {url}")


def fetch_text(url: str) -> str:
    with tempfile.TemporaryDirectory() as tmp:
        f = Path(tmp) / "t.txt"
        download(url, f)
        return f.read_text(encoding="utf-8", errors="replace")


def copy_licenses_from_source(version3: bool):
    """Copia i testi delle licenze dai sorgenti ufficiali di FFmpeg (scaricati e controllati con l'impronta) e sceglie quello della build."""
    with tempfile.TemporaryDirectory() as tmp:
        tarball = Path(tmp) / f"ffmpeg-{SRC_VERSION}.tar.xz"
        download(SRC_URL, tarball)
        if SRC_SHA256 and sha256(tarball) != SRC_SHA256:
            sys.exit("impronta SHA-256 dei sorgenti diversa da quella attesa: file non attendibile")
        (LIC / "ffmpeg").mkdir(parents=True, exist_ok=True)
        with tarfile.open(tarball) as t:
            for n in ("COPYING.LGPLv2.1", "COPYING.LGPLv3", "COPYING.GPLv3"):
                m = t.getmember(f"ffmpeg-{SRC_VERSION}/{n}")
                with t.extractfile(m) as src, open(LIC / "ffmpeg" / n, "wb") as dst:
                    shutil.copyfileobj(src, dst)
    shutil.copy2(LIC / "ffmpeg" / ("COPYING.LGPLv3" if version3 else "COPYING.LGPLv2.1"), LIC / "FFMPEG-LICENSE.txt")


def write_build_info(name: str, lines: list):
    LIC.mkdir(parents=True, exist_ok=True)
    f = LIC / f"FFMPEG-BUILD-{name}.txt"
    f.write_text("\n".join(lines) + "\n", encoding="utf-8")
    (LIC / "FFMPEG-BUILD.txt").write_text("\n".join(lines) + "\n", encoding="utf-8")   # nome fisso, quello che entra nell'installer
    print(f"scritto {f}")


def windows():
    BIN.mkdir(parents=True, exist_ok=True)
    LIC.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory() as tmp:
        z = Path(tmp) / "ffmpeg.zip"
        download(WIN_URL, z)
        sums = fetch_text(WIN_SUMS)
        want = next((l.split()[0] for l in sums.splitlines() if l.strip().endswith(WIN_URL.rsplit("/", 1)[1])), None)
        got = sha256(z)
        if want and want != got:
            sys.exit(f"impronta SHA-256 diversa da quella pubblicata ({got} invece di {want}): file non attendibile")
        print("SHA-256 verificata" if want else "attenzione: impronta non trovata nell'elenco, non verificata")
        with zipfile.ZipFile(z) as zf:
            names = zf.namelist()
            exe = next(n for n in names if n.endswith("/bin/ffmpeg.exe"))
            out = BIN / "ffmpeg-x86_64-pc-windows-msvc.exe"
            with zf.open(exe) as s, open(out, "wb") as d:
                shutil.copyfileobj(s, d)
    data = out.read_bytes()
    m = re.search(rb"n\d+\.\d+(?:\.\d+)?-\d+-g[0-9a-f]{7,}-\d{8}", data) or re.search(rb"ffmpeg version ([^\s\x00]+)", data)
    ver = (m.group(0) if m.lastindex is None else m.group(1)).decode() if m else "sconosciuta"
    # --enable-version3 rende la build LGPL 3 (non 2.1): cambiano il testo da includere e alcune condizioni (brevetti)
    v3 = b"--enable-version3" in data
    copy_licenses_from_source(v3)
    lic_name = "GNU LGPL 3 o successiva (build con --enable-version3)" if v3 else "GNU LGPL 2.1 o successiva"
    write_build_info("windows", [
        "FFmpeg incluso in SpinLab DJ (Windows, 64 bit)",
        f"Versione: {ver}",
        f"Licenza: {lic_name}; build senza componenti GPL ne' non liberi",
        f"Origine del programma: {WIN_URL}",
        f"SHA-256 dell'archivio: {got}",
        f"SHA-256 di ffmpeg.exe: {sha256(out)}",
        "Build a cura di BtbN: https://github.com/BtbN/FFmpeg-Builds (script e configurazione pubblici)",
        "Sorgenti di FFmpeg: https://ffmpeg.org/releases/ e https://git.ffmpeg.org/ffmpeg.git (commit indicato nella versione)",
        "Testi delle licenze: licenses/FFMPEG-LICENSE.txt e licenses/ffmpeg/",
        "Per ricevere i sorgenti su altro supporto, scrivere all'indirizzo indicato in Informazioni.",
        "Si puo' sostituire ffmpeg.exe, che e' un file separato accanto a SpinLab, con una versione compatibile.",
    ])
    print(f"fatto: {out} ({out.stat().st_size // 1048576} MB), versione {ver}")


def macos(arch: str):
    triple = "aarch64-apple-darwin" if arch == "arm64" else "x86_64-apple-darwin"
    BIN.mkdir(parents=True, exist_ok=True)
    LIC.mkdir(parents=True, exist_ok=True)
    work = Path(tempfile.mkdtemp(prefix="ffmpeg-build-"))
    try:
        tarball = work / f"ffmpeg-{SRC_VERSION}.tar.xz"
        download(SRC_URL, tarball)
        got = sha256(tarball)
        if SRC_SHA256 and got != SRC_SHA256:
            sys.exit(f"impronta SHA-256 dei sorgenti diversa da quella attesa ({got}): file non attendibile")
        print(f"SHA-256 dei sorgenti: {got}" + ("" if SRC_SHA256 else "  <- da scrivere in SRC_SHA256 di questo script"))
        with tarfile.open(tarball) as t:
            t.extractall(work)
        src = work / f"ffmpeg-{SRC_VERSION}"
        flags = list(MAC_CONFIGURE) + [f"--arch={'arm64' if arch == 'arm64' else 'x86_64'}", "--extra-cflags=-mmacosx-version-min=11.0", "--extra-ldflags=-mmacosx-version-min=11.0"]
        print("configure:", " ".join(flags))
        subprocess.check_call(["./configure"] + flags, cwd=src)
        subprocess.check_call(["make", f"-j{os.cpu_count() or 4}"], cwd=src)
        exe = src / "ffmpeg"
        conf = subprocess.run([str(exe), "-hide_banner", "-buildconf"], capture_output=True, text=True).stdout
        if re.search(r"--enable-(gpl|nonfree|version3)|libx26[45]|fdk", conf):
            sys.exit("la build contiene componenti GPL o non liberi: non e' distribuibile come LGPL")
        enc = subprocess.run([str(exe), "-hide_banner", "-encoders"], capture_output=True, text=True).stdout
        for need in ("h264_videotoolbox", " aac ", "mjpeg"):
            if need not in enc:
                sys.exit(f"manca un encoder necessario: {need.strip()}")
        out = BIN / f"ffmpeg-{triple}"
        shutil.copy2(exe, out)
        subprocess.run(["strip", str(out)], check=False)
        (LIC / "ffmpeg").mkdir(parents=True, exist_ok=True)
        for n in ("COPYING.LGPLv2.1", "COPYING.LGPLv3", "COPYING.GPLv3"):
            shutil.copy2(src / n, LIC / "ffmpeg" / n)
        shutil.copy2(src / "COPYING.LGPLv2.1", LIC / "FFMPEG-LICENSE.txt")      # senza --enable-version3 la build e' LGPL 2.1 o successiva
        write_build_info("macos-" + arch, [
            f"FFmpeg incluso in SpinLab DJ (macOS {arch})",
            f"Versione: {SRC_VERSION}",
            "Licenza: GNU LGPL 2.1 o successiva (build senza componenti GPL ne' non liberi)",
            f"Sorgenti ufficiali: {SRC_URL}",
            f"SHA-256 dei sorgenti: {got}",
            f"SHA-256 di ffmpeg: {sha256(out)}",
            "Configurazione: ./configure " + " ".join(flags),
            "Si puo' sostituire il file ffmpeg (separato, accanto al programma) con una versione compatibile.",
        ])
        print(f"fatto: {out} ({out.stat().st_size // 1048576} MB)")
    finally:
        shutil.rmtree(work, ignore_errors=True)


if __name__ == "__main__":
    t = sys.argv[1] if len(sys.argv) > 1 else ""
    if t == "windows":
        windows()
    elif t in ("macos-arm64", "macos-x64"):
        macos("arm64" if t.endswith("arm64") else "x64")
    else:
        sys.exit(__doc__)
