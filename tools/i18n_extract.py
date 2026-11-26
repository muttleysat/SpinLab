#!/usr/bin/env python3
"""Elenca le frasi italiane dell'interfaccia che mancano nel dizionario inglese (ui/i18n/en.js).

Uso:  python tools/i18n_extract.py            (stampa le frasi mancanti, una per riga, con il file d'origine)
      python tools/i18n_extract.py --json     (le stampa come JSON, pronte da tradurre)
Serve quando si aggiunge testo all'interfaccia: ogni frase nuova va aggiunta al dizionario, o resta in italiano anche in inglese.
"""
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
UI = ROOT / "ui"
SRC = ROOT / "src-tauri" / "src"

IT = re.compile(
    r"\b(il|lo|la|le|gli|di|del|dello|della|dei|degli|delle|per|con|non|un|una|che|si|da|dal|al|alla|nel|nella|sul|sulla|sono|questo|questa|"
    r"brano|brani|tasto|clic|premi|scegli|apri|carica|ferma|avvia|nessun|nessuna|errore|impossibile|cartella|scheda|uscita|cuffie|griglia|"
    r"impostazioni|registrazione|libreria|tonalità|battiti|battute|video|stem|dispositivo|frequenza|campionamento|predefinit[ao]|"
    r"automatic[ao]|attiv[ao]|spent[ao]|acces[ao]|volume|velocità|qualità|pulsante|finestra|schermo|elimina|rinomina|annulla|chiudi|salva|"
    r"applica|analizza|crea|aggiungi|rimuovi|togli|sposta|ripristina|cerca|esporta|importa|cancella|ignora|calibra|misura|modo|livello|"
    r"ogni|anche|solo|più|meno|sopra|sotto|dopo|prima|ancora|già|mai|sempre|niente|tutti|tutte|nuova|nuovo)\b|[àèéìòù]",
    re.I,
)


def italian(s):
    s = re.sub(r"\s+", " ", s).strip()
    if len(s) < 2 or not re.search(r"[A-Za-zÀ-ú]{2}", s):
        return None
    if re.search(r"^(https?:|\.|#|\$|--|var\(|[a-z]+[A-Z]\w*$|[\w-]+\.(js|css|html|png|svg|json)$)", s):
        return None
    if "{" in s and "}" in s and re.search(r"[;=]", s):
        return None  # codice dentro una stringa
    return s if IT.search(s) else None


def from_html(path):
    s = path.read_text(encoding="utf-8")
    s = re.sub(r"<script.*?</script>|<style.*?</style>|<!--.*?-->", "", s, flags=re.S)
    out = [t for t in re.findall(r">([^<>]+)<", s)]
    out += re.findall(r'(?:title|placeholder|alt|aria-label)="([^"]+)"', s)
    return out


LIT = re.compile(r'"((?:[^"\\\n]|\\.)*)"' + r"|'((?:[^'\\\n]|\\.)*)'" + r"|`([^`]*)`")


def from_js(path):
    s = path.read_text(encoding="utf-8")
    s = re.sub(r"^\s*//.*$", "", s, flags=re.M)
    s = re.sub(r"/\*.*?\*/", "", s, flags=re.S)
    out = []
    for a, b, c in LIT.findall(s):
        t = a or b or c
        if c:  # modello HTML: testi fra i tag e attributi; le ${...} diventano {a}, {b}...
            n = iter("abcdefghij")
            t2 = re.sub(r"\$\{[^{}]*(\{[^{}]*\}[^{}]*)*\}", lambda m: "{" + next(n, "z") + "}", t)
            parts = re.findall(r">([^<>]+)<", "<" + t2 + ">") if "<" in t2 else [t2]
            parts += re.findall(r'(?:title|placeholder|alt|aria-label)="([^"]+)"', t2)
            out += parts
        else:
            out.append(t)
    return out


def from_rs(path):
    s = path.read_text(encoding="utf-8")
    s = re.sub(r"^\s*//.*$", "", s, flags=re.M)
    s = re.sub(r"eprintln!\(.*?\);", "", s, flags=re.S)
    s = re.split(r"#\[cfg\(test\)\]\s*mod tests", s)[0]
    out = []
    for t in re.findall(r'"((?:[^"\\]|\\.)*)"', s):
        n = iter("abcdefghij")
        out.append(re.sub(r"\{[^{}]*\}", lambda m: "{" + next(n, "z") + "}", t))
    return out


def known():
    p = UI / "i18n" / "en.js"
    if not p.exists():
        return set(), []
    s = p.read_text(encoding="utf-8")
    keys = set(json.loads(m) for m in re.findall(r'^\s*("(?:[^"\\]|\\.)*")\s*:', s, flags=re.M))
    pats = [json.loads(m) for m in re.findall(r'^\s*\[\s*("(?:[^"\\]|\\.)*")\s*,', s, flags=re.M)]
    rx = [re.compile("^" + re.sub(r"\\\{\w+\\\}", r"[\\s\\S]*?", re.escape(re.sub(r"\s+", " ", p).strip())) + "$") for p in pats]
    return keys, rx


def main():
    sys.stdout.reconfigure(encoding="utf-8")
    keys, rx = known()
    seen, rows = set(), []
    files = [(p, from_html) for p in UI.glob("*.html") if p.name not in ("guida.html",)]
    files += [(p, from_js) for p in UI.glob("*.js") if p.name not in ("i18n.js",)]
    files += [(p, from_rs) for p in SRC.glob("*.rs")]
    for p, fn in files:
        for raw in fn(p):
            s = italian(raw.replace("\\n", " "))
            if not s or s in seen:
                continue
            seen.add(s)
            if s in keys or any(r.match(s) for r in rx):
                continue
            rows.append((s, p.relative_to(ROOT).as_posix()))
    if "--json" in sys.argv:
        print(json.dumps({s: "" for s, _ in rows}, ensure_ascii=False, indent=1))
    else:
        for s, f in rows:
            print(f"{f}\t{s}")
        print(f"\n{len(rows)} frasi senza traduzione", file=sys.stderr)


if __name__ == "__main__":
    main()
