#!/usr/bin/env python3
"""Genera THIRD-PARTY-LICENSES.md: i testi delle licenze di tutte le librerie usate da SpinLab, piu' i componenti che non sono librerie Rust.

Le licenze MIT, Apache, BSD ecc. chiedono di riportare l'avviso di copyright e il testo della licenza quando si distribuisce il programma:
questo file serve a quello, e finisce nell'installer (cartella «licenses»).

Uso (dalla cartella del progetto):   python tools/gen_third_party.py
Rilanciarlo ogni volta che cambiano le dipendenze (Cargo.toml / Cargo.lock).
"""
import hashlib
import json
import subprocess
import sys
from collections import defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
OUT = ROOT / "THIRD-PARTY-LICENSES.md"

# componenti che non sono crate Rust (o che hanno bisogno di una nota)
EXTRA = """## Componenti che non sono librerie Rust

### LAME (encoder MP3) - LGPL 2.1
SpinLab registra in MP3 con la libreria LAME 3.100 (https://lame.sourceforge.io), fornita come file separato `libmp3lame.dll` e
caricata a richiesta. La libreria e' sotto licenza GNU LGPL: potete sostituire `libmp3lame.dll` con una vostra versione compatibile.
I sorgenti ufficiali sono su https://lame.sourceforge.io e lo script con cui la DLL e' stata costruita e' nel repository del progetto
(`tools/build_lame_dll.py`). Il testo della licenza e' in `licenses/LICENSE-LAME.txt`.

### Modello di separazione degli stem - HT-Demucs
Gli stem si separano con il modello HT-Demucs (Rouard, Massa, Defossez, "Hybrid Transformers for Music Source Separation", 2023),
sviluppato da Meta (facebookresearch/demucs) nella versione ONNX di StemSplit
(https://huggingface.co/adowu/htdemucs-onnx), distribuita con licenza MIT. Il testo della licenza di Demucs, con il copyright di Meta:

```
MIT License

Copyright (c) Meta, Inc. and its affiliates.

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

### ONNX Runtime - MIT
Copyright (c) Microsoft Corporation. Esegue il modello di separazione. https://github.com/microsoft/onnxruntime

### DirectML - Microsoft
`DirectML.dll` (accelerazione su GPU) e' redistribuibile secondo i termini Microsoft: https://learn.microsoft.com/windows/ai/directml

### ASIO - Steinberg
ASIO is a trademark of Steinberg Media Technologies GmbH, registered in Europe and other countries.
ASIO SDK usato su licenza Steinberg.

### Microsoft Edge WebView2
L'interfaccia usa il runtime WebView2 di Microsoft, installato da Windows o dal programma di installazione.

"""


def main():
    meta = json.loads(subprocess.check_output(
        ["cargo", "metadata", "--format-version", "1", "--filter-platform", "x86_64-pc-windows-msvc"],
        cwd=ROOT / "src-tauri", text=True, encoding="utf-8"))
    pk = {p["id"]: p for p in meta["packages"]}
    nodes = {n["id"]: n for n in meta["resolve"]["nodes"]}
    seen, stack = set(), [meta["resolve"]["root"]]
    while stack:
        i = stack.pop()
        if i in seen:
            continue
        seen.add(i)
        for d in nodes[i]["deps"]:
            if any(k.get("kind") in (None, "build") for k in d["dep_kinds"]):
                stack.append(d["pkg"])
    crates = sorted((pk[i] for i in seen if pk[i]["name"] != "spinlab"), key=lambda p: (p["name"].lower(), p["version"]))

    # per ogni crate: i file di licenza veri e propri; i testi uguali si scrivono una volta sola
    texts = {}                       # hash -> testo
    users = defaultdict(list)        # hash -> [crate]
    nolic = []
    for p in crates:
        d = Path(p["manifest_path"]).parent
        files = sorted(f for f in d.iterdir() if f.is_file() and f.name.upper().split(".")[0] in
                       ("LICENSE", "LICENCE", "COPYING", "NOTICE", "LICENSE-MIT", "LICENSE-APACHE", "LICENSE-BSD", "LICENSE-ZLIB",
                        "LICENSE-MPL", "UNLICENSE", "LICENSE-2.0", "LICENSE-THIRD-PARTY", "COPYRIGHT")) \
            or sorted(f for f in d.iterdir() if f.is_file() and f.name.upper().startswith(("LICENSE", "LICENCE", "COPYING", "UNLICENSE")))
        if not files:
            nolic.append(p)
            continue
        for f in files:
            try:
                t = f.read_text(encoding="utf-8", errors="replace").strip()
            except OSError:
                continue
            h = hashlib.sha1(t.encode("utf-8")).hexdigest()
            texts[h] = t
            users[h].append(f"{p['name']} {p['version']}")

    out = ["# Licenze di terze parti", "",
           "SpinLab DJ usa le librerie e i componenti elencati qui sotto. Ognuno e' distribuito con la propria licenza, di cui si riporta il testo.", "",
           EXTRA, "## Librerie Rust", "", f"{len(crates)} librerie. Riepilogo (nome, versione, licenza dichiarata):", ""]
    for p in crates:
        out.append(f"- {p['name']} {p['version']} - {p.get('license') or 'vedi il testo'}")
    out += ["", "## Testi delle licenze", "", "I testi identici sono riportati una volta sola, con l'elenco delle librerie a cui si applicano.", ""]
    for n, (h, t) in enumerate(sorted(texts.items(), key=lambda kv: -len(users[kv[0]])), 1):
        names = sorted(set(users[h]))
        out += [f"### Testo {n} - usato da: " + ", ".join(names), "", "```text", t, "```", ""]
    if nolic:
        out += ["## Librerie senza file di licenza nel pacchetto", "",
                "Per queste vale la licenza dichiarata nel riepilogo qui sopra:", ""]
        out += [f"- {p['name']} {p['version']} - {p.get('license')}" for p in nolic]
    OUT.write_text("\n".join(out) + "\n", encoding="utf-8")
    print(f"scritto {OUT.name}: {len(crates)} librerie, {len(texts)} testi diversi, {len(nolic)} senza file di licenza, {OUT.stat().st_size // 1024} KB")


if __name__ == "__main__":
    sys.exit(main())
