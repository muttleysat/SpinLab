"""Misura quanto l'analisi del tempo sbaglia su una cartella di brani che hanno il BPM nei tag (TBPM).

  python tools/tempo_bench.py "E:/cartella" [quanti=60] [seme=1]

Per ogni brano confronta il BPM trovato dal programma con quello scritto nel file e lo classifica:
  giusto (entro 2%), ottava (x2 o /2), 3:2 (x1,5 o 2/3), altro. Elenca gli sbagliati.
Serve ffprobe nel PATH.
"""
import json, os, random, subprocess, sys, tempfile

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))


def tag_bpm(path):
    r = subprocess.run(["ffprobe", "-v", "error", "-show_entries", "format_tags=TBPM,BPM", "-of", "default=nw=1:nk=1", path], stdout=subprocess.PIPE, text=True)
    for line in r.stdout.split():
        try:
            v = float(line.replace(",", "."))
            if 50 < v < 220:
                return v
        except ValueError:
            pass
    return None


def classify(found, ref):
    r = found / ref
    for name, k in (("giusto", 1.0), ("ottava", 2.0), ("ottava", 0.5), ("3:2", 1.5), ("3:2", 2 / 3), ("4:3", 4 / 3), ("4:3", 0.75)):
        if abs(r / k - 1) < 0.025:
            return name
    return "altro"


def main():
    if len(sys.argv) < 2:
        sys.exit(__doc__)
    folder, n, seed = sys.argv[1], int(sys.argv[2]) if len(sys.argv) > 2 else 60, int(sys.argv[3]) if len(sys.argv) > 3 else 1
    files = sorted(os.path.join(folder, f) for f in os.listdir(folder) if f.lower().endswith((".mp3", ".flac", ".m4a", ".wav")))
    random.Random(seed).shuffle(files)
    ref = {}
    for f in files:
        b = tag_bpm(f)
        if b:
            ref[f] = b
        if len(ref) >= n:
            break
    print(f"{len(ref)} brani con BPM nei tag (su {len(files)} file)")
    out = os.path.join(tempfile.gettempdir(), "spinlab_bench.json")
    if os.path.exists(out):
        os.remove(out)
    lst = os.path.join(tempfile.gettempdir(), "spinlab_bench_list.txt")
    with open(lst, "w", encoding="utf8") as fh:
        fh.write(chr(10).join(ref))
    env = dict(os.environ, SPIN_LIST=lst, SPIN_OUT=out)
    r = subprocess.run([sys.executable, os.path.join(ROOT, "tools", "test.py"), "analyze_dump", "--ignored"], env=env, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
    if not os.path.exists(out):
        sys.exit("analisi fallita:\n" + r.stdout[-1500:])
    res = json.load(open(out, encoding="utf8"))
    ref = {f: b for f, b in ref.items() if f in res}
    counts, bad = {}, []
    for f, rb in ref.items():
        fb = res[f]["bpm"]
        c = classify(fb, rb)
        counts[c] = counts.get(c, 0) + 1
        if c != "giusto":
            bad.append((c, os.path.basename(f), rb, fb))
    print("\nrisultato:", ", ".join(f"{k} {v}" for k, v in sorted(counts.items())), f"  -> giusti {100 * counts.get('giusto', 0) / len(ref):.0f}%")
    for c, name, rb, fb in sorted(bad):
        print(f"  {c:7s} {name[:58]:58s} tag {rb:6.1f}  trovato {fb:6.1f}  (x{fb / rb:.3f})")
    json.dump({os.path.basename(f): {"tag": ref[f], "bpm": res[f]["bpm"]} for f in ref}, open(os.path.join(tempfile.gettempdir(), "spinlab_bench_summary.json"), "w"))


if __name__ == "__main__":
    main()
