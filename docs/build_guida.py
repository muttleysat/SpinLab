#!/usr/bin/env python3
"""Genera ui/guida.html (la guida che si apre dentro il programma) a partire da docs/MANUALE.md.

Uso, dalla cartella del progetto:   python docs/build_guida.py

Il manuale si scrive una volta sola in MANUALE.md: questo script lo converte in una pagina HTML autonoma (senza internet,
con indice laterale e ricerca) che il programma mostra dal pulsante «?» in alto. Va rilanciato dopo ogni modifica del manuale.
"""
import html
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SRC = ROOT / "docs" / "MANUALE.md"
OUT = ROOT / "ui" / "guida.html"


def slug(text: str) -> str:
    t = re.sub(r"[^\w\s-]", "", text.lower(), flags=re.UNICODE)
    return re.sub(r"[\s_]+", "-", t).strip("-")


def inline(t: str) -> str:
    t = html.escape(t, quote=False)
    t = re.sub(r"`([^`]+)`", r"<code>\1</code>", t)
    t = re.sub(r"\*\*([^*]+)\*\*", r"<b>\1</b>", t)
    t = re.sub(r"(?<![*\w])\*([^*\n]+)\*(?![*\w])", r"<i>\1</i>", t)
    t = re.sub(r"!\[([^\]]*)\]\(([^)]+)\)", lambda m: f'<img src="{m.group(2).replace("../ui/", "")}" alt="{m.group(1)}" class="logo">', t)
    t = re.sub(r"(?<!!)\[([^\]]+)\]\(([^)]+)\)", lambda m: f'<a href="{m.group(2)}">{m.group(1)}</a>', t)
    return t


def convert(md: str):
    lines = md.splitlines()
    out, toc = [], []
    i, para, lst, open_sec = 0, [], None, False  # lst: (tipo, [voci])

    def flush_para():
        nonlocal para
        if para:
            out.append("<p>" + inline(" ".join(para)) + "</p>")
            para = []

    def flush_list():
        nonlocal lst
        if lst:
            tag = lst[0]
            out.append(f"<{tag}>" + "".join(f"<li>{inline(x)}</li>" for x in lst[1]) + f"</{tag}>")
            lst = None

    while i < len(lines):
        ln = lines[i]
        if ln.startswith("```"):
            flush_para(); flush_list()
            j = i + 1
            code = []
            while j < len(lines) and not lines[j].startswith("```"):
                code.append(lines[j]); j += 1
            out.append("<pre><code>" + html.escape("\n".join(code)) + "</code></pre>")
            i = j + 1
            continue
        m = re.match(r"^(#{1,3})\s+(.*)$", ln)
        if m:
            flush_para(); flush_list()
            lvl, text = len(m.group(1)), m.group(2).strip()
            sid = slug(text)
            if lvl == 2:
                if open_sec:
                    out.append("</section>")
                out.append(f'<section data-title="{html.escape(text)}">')
                open_sec = True
                toc.append((sid, text))
            out.append(f'<h{lvl} id="{sid}">{inline(text)}</h{lvl}>')
            i += 1
            continue
        if ln.strip() == "---":
            flush_para(); flush_list(); i += 1
            continue
        if ln.startswith(">"):
            flush_para(); flush_list()
            q = []
            while i < len(lines) and lines[i].startswith(">"):
                q.append(lines[i].lstrip("> ").strip()); i += 1
            out.append("<blockquote>" + inline(" ".join(q)) + "</blockquote>")
            continue
        if ln.startswith("|") and i + 1 < len(lines) and re.match(r"^\|[\s:|-]+\|$", lines[i + 1].strip()):
            flush_para(); flush_list()
            head = [c.strip() for c in ln.strip().strip("|").split("|")]
            rows = []
            i += 2
            while i < len(lines) and lines[i].startswith("|"):
                rows.append([c.strip() for c in lines[i].strip().strip("|").split("|")]); i += 1
            t = "<div class='tw'><table><thead><tr>" + "".join(f"<th>{inline(c)}</th>" for c in head) + "</tr></thead><tbody>"
            t += "".join("<tr>" + "".join(f"<td>{inline(c)}</td>" for c in r) + "</tr>" for r in rows) + "</tbody></table></div>"
            out.append(t)
            continue
        m = re.match(r"^\s*[-*]\s+(.*)$", ln)
        if m:
            flush_para()
            if not lst or lst[0] != "ul":
                flush_list(); lst = ("ul", [])
            lst[1].append(m.group(1)); i += 1
            continue
        m = re.match(r"^\s*\d+\.\s+(.*)$", ln)
        if m:
            flush_para()
            if not lst or lst[0] != "ol":
                flush_list(); lst = ("ol", [])
            lst[1].append(m.group(1)); i += 1
            continue
        if not ln.strip():
            flush_para(); flush_list(); i += 1
            continue
        # prosecuzione di una voce di elenco (riga rientrata) oppure testo normale
        if lst and ln.startswith("  "):
            lst[1][-1] += " " + ln.strip()
        else:
            flush_list(); para.append(ln.strip())
        i += 1
    flush_para(); flush_list()
    if open_sec:
        out.append("</section>")
    return "\n".join(out), toc


PAGE = """<!doctype html>
<html lang="it">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>SpinLab DJ - Guida</title>
<style>
:root { --bg: #0b0f18; --panel: #101726; --line: #25324d; --text: #dbe4f5; --dim: #8393b3; --accent: #18b6ff; }
* { box-sizing: border-box; }
html { scroll-behavior: smooth; }
body { margin: 0; background: var(--bg); color: var(--text); font: 14px/1.6 system-ui, "Segoe UI", sans-serif; }
#wrap { display: grid; grid-template-columns: 250px minmax(0, 1fr); min-height: 100vh; }
nav { position: sticky; top: 0; align-self: start; height: 100vh; overflow: auto; padding: 16px 12px; border-right: 1px solid var(--line); background: var(--panel); }
nav input { width: 100%; padding: 7px 9px; border-radius: 7px; border: 1px solid var(--line); background: var(--bg); color: var(--text); margin-bottom: 10px; }
nav a { display: block; padding: 5px 8px; border-radius: 6px; color: var(--dim); text-decoration: none; font-size: 13px; }
nav a:hover, nav a.on { background: color-mix(in srgb, var(--accent) 16%, transparent); color: var(--text); }
main { padding: 22px 38px 80px; max-width: 920px; }
h1 { font-size: 26px; margin: 0 0 8px; } h2 { font-size: 20px; margin: 34px 0 8px; padding-top: 6px; border-top: 1px solid var(--line); }
h3 { font-size: 15px; margin: 22px 0 6px; color: var(--accent); letter-spacing: .3px; }
p, li { max-width: 78ch; } li { margin: 3px 0; }
code { background: color-mix(in srgb, var(--accent) 14%, transparent); padding: 1px 5px; border-radius: 4px; font: 12.5px ui-monospace, Consolas, monospace; }
pre { background: var(--panel); border: 1px solid var(--line); border-radius: 8px; padding: 12px 14px; overflow: auto; } pre code { background: none; padding: 0; }
blockquote { margin: 12px 0; padding: 8px 14px; border-left: 3px solid var(--accent); background: var(--panel); color: var(--dim); border-radius: 0 8px 8px 0; }
.tw { overflow-x: auto; margin: 10px 0; } table { border-collapse: collapse; font-size: 13px; min-width: 60%; }
th, td { text-align: left; padding: 6px 10px; border: 1px solid var(--line); vertical-align: top; } th { background: var(--panel); }
a { color: var(--accent); } mark { background: #ffd34d66; color: inherit; border-radius: 3px; }
section.hide { display: none; } img.logo { height: 120px; width: auto; display: block; margin: 8px 0; border-radius: 4px; }
@media (max-width: 820px) { #wrap { grid-template-columns: 1fr; } nav { position: static; height: auto; border-right: 0; border-bottom: 1px solid var(--line); } main { padding: 16px; } }
</style>
</head>
<body>
<div id="wrap">
<nav><input id="q" type="search" placeholder="Cerca nella guida…" autocomplete="off">
@@TOC@@
</nav>
<main>
@@BODY@@
</main>
</div>
<script>
(function () {
  const secs = [...document.querySelectorAll("main section")], links = [...document.querySelectorAll("nav a")];
  const q = document.getElementById("q"), strip = () => document.querySelectorAll("mark").forEach((m) => m.replaceWith(m.textContent));
  function mark(node, re) {
    for (const c of [...node.childNodes]) {
      if (c.nodeType === 3) {
        const t = c.nodeValue; if (!re.test(t)) continue; re.lastIndex = 0;
        const f = document.createDocumentFragment(); let last = 0, m;
        while ((m = re.exec(t))) { f.append(t.slice(last, m.index)); const k = document.createElement("mark"); k.textContent = m[0]; f.append(k); last = m.index + m[0].length; }
        f.append(t.slice(last)); c.replaceWith(f);
      } else if (c.nodeType === 1 && c.tagName !== "MARK") mark(c, re);
    }
  }
  q.addEventListener("input", () => {
    strip(); const v = q.value.trim().toLowerCase();
    secs.forEach((s, i) => { const hit = !v || s.textContent.toLowerCase().includes(v); s.classList.toggle("hide", !hit); links[i].style.display = hit ? "" : "none"; });
    if (v.length > 1) { const re = new RegExp(v.replace(/[.*+?^${}()|[\\]\\\\]/g, "\\\\$&"), "gi"); secs.forEach((s) => !s.classList.contains("hide") && mark(s, re)); }
  });
  const io = new IntersectionObserver((es) => es.forEach((e) => { if (e.isIntersecting) { links.forEach((l) => l.classList.toggle("on", l.getAttribute("href") === "#" + e.target.querySelector("h2").id)); } }), { rootMargin: "-10% 0px -80% 0px" });
  secs.forEach((s) => io.observe(s));
})();
</script>
</body>
</html>
"""


def build(src: Path, out: Path, lang: str, ids=None):
    """Una guida: `ids` (dalla guida italiana) da' alle sezioni gli stessi identificatori in ogni lingua, cosi' i link «data-help»
    del programma aprono la sezione giusta anche in inglese (le due versioni del manuale hanno le stesse sezioni, nello stesso ordine)."""
    md = src.read_text(encoding="utf-8")
    # il collegamento all'altra lingua serve solo su GitHub: nella guida dentro il programma (senza internet) si toglie
    md = "\n".join(l for l in md.splitlines() if not re.match(r"^(\*\*English\*\* · |\[English\]\(MANUALE\.en\.md\))", l))
    body, toc = convert(md)
    if ids is not None:
        own = re.findall(r'<h[23] id="([^"]+)"', body)
        if len(own) != len(ids):
            print(f"attenzione: {src.name} ha {len(own)} titoli, la guida italiana {len(ids)}: i link alle sezioni possono non funzionare")
        else:
            it = iter(ids)
            body = re.sub(r'(<h[23] id=")[^"]+(")', lambda m: m.group(1) + next(it) + m.group(2), body)
            mp = dict(zip(own, ids))
            body = re.sub(r'href="#([^"]+)"', lambda m: f'href="#{mp.get(m.group(1), m.group(1))}"', body)
            toc = [(mp.get(sid, sid), t) for sid, t in toc]
    nav = "\n".join(f'<a href="#{sid}">{html.escape(t)}</a>' for sid, t in toc)
    page = PAGE
    if lang == "en":
        page = page.replace('<html lang="it">', '<html lang="en">').replace("SpinLab DJ - Guida", "SpinLab DJ - Guide").replace("Cerca nella guida…", "Search the guide…")
    out.write_text(page.replace("@@TOC@@", nav).replace("@@BODY@@", body), encoding="utf-8", newline="\n")
    print(f"scritto {out.relative_to(ROOT)}: {len(toc)} sezioni, {out.stat().st_size // 1024} KB")
    return re.findall(r'<h[23] id="([^"]+)"', body)


def main():
    ids = build(SRC, OUT, "it")
    en = ROOT / "docs" / "MANUALE.en.md"
    if en.exists():
        build(en, ROOT / "ui" / "guida.en.html", "en", ids)


if __name__ == "__main__":
    sys.exit(main())
