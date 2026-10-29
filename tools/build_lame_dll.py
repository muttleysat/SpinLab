#!/usr/bin/env python3
"""Compila libmp3lame.dll dai sorgenti ufficiali di LAME 3.100 (licenza LGPL).

Perche' una DLL a parte: LAME e' sotto LGPL. Legato dentro l'eseguibile (collegamento statico) obbligherebbe a
fornire a chiunque la possibilita' di ricollegare il programma con una LAME diversa; caricato come DLL separata
l'utente puo' sostituire il file e la condizione e' soddisfatta. SpinLab lo carica a richiesta (src-tauri/src/lame.rs).

Uso:   python tools/build_lame_dll.py <cartella dei sorgenti di LAME 3.100> [cartella di uscita]
Sorgenti: https://lame.sourceforge.io  (lame-3.100.tar.gz). Servono LLVM (clang-cl) e gli strumenti C++ di Visual Studio.
Risultato: <uscita>/libmp3lame.dll  (default: src-tauri/libs)
"""
import os
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
CLANG_CL = r"C:\Program Files\LLVM\bin\clang-cl.exe"
VCVARS = [
    r"C:\Program Files\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat",
    r"C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvars64.bat",
    r"C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat",
    r"C:\Program Files\Microsoft Visual Studio\2019\BuildTools\VC\Auxiliary\Build\vcvars64.bat",
]
SOURCES = ["bitstream", "encoder", "fft", "gain_analysis", "id3tag", "lame", "newmdct", "presets", "psymodel", "quantize_pvt",
           "vector/xmm_quantize_sub", "quantize", "reservoir", "set_get", "tables", "takehiro", "util", "vbrquantize", "VbrTag", "version"]
# solo cio' che SpinLab usa
EXPORTS = ["get_lame_version", "lame_init", "lame_close", "lame_set_in_samplerate", "lame_set_num_channels", "lame_set_brate",
           "lame_set_quality", "lame_set_mode", "lame_init_params", "lame_encode_buffer_interleaved_ieee_float",
           "lame_encode_buffer_interleaved", "lame_encode_flush", "lame_encode_flush_nogap"]


def main():
    if len(sys.argv) < 2:
        sys.exit(__doc__)
    src = Path(sys.argv[1]).resolve()
    out = Path(sys.argv[2]).resolve() if len(sys.argv) > 2 else ROOT / "src-tauri" / "libs"
    if not (src / "libmp3lame" / "lame.c").exists():
        sys.exit(f"{src} non sembra la cartella dei sorgenti di LAME (manca libmp3lame/lame.c)")
    vcvars = next((v for v in VCVARS if os.path.exists(v)), None)
    if not vcvars:
        sys.exit("Visual Studio Build Tools non trovati")
    out.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory() as td:
        td = Path(td)
        (td / "msvc").mkdir()
        shutil.copy(src / "configMS.h", td / "msvc" / "config.h")
        (td / "lame.def").write_text("LIBRARY libmp3lame\nEXPORTS\n" + "\n".join("    " + e for e in EXPORTS) + "\n")
        files = " ".join(f'"{src / "libmp3lame" / (s + ".c")}"' for s in SOURCES)
        flags = ('/O2 /W0 /LD /DHAVE_CONFIG_H /DTAKEHIRO_IEEE754_HACK /DFLOAT8=float /DREAL_IS_FLOAT=1 /DBS_FORMAT=BINARY '
                 f'/I"{src / "include"}" /I"{td / "msvc"}" /I"{src / "libmp3lame"}"')
        cmd = f'call "{vcvars}" >nul && "{CLANG_CL}" {flags} {files} /Fe"{td / "libmp3lame.dll"}" /link /DEF:"{td / "lame.def"}"'
        r = subprocess.run(cmd, shell=True, cwd=td)
        if r.returncode != 0 or not (td / "libmp3lame.dll").exists():
            sys.exit("compilazione non riuscita")
        shutil.copy(td / "libmp3lame.dll", out / "libmp3lame.dll")
    print("scritto", out / "libmp3lame.dll")


if __name__ == "__main__":
    main()
