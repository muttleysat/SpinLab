"""Confronta la beatgrid di SpinLab con quella calcolata da Engine DJ (libreria di riferimento) su una selezione di brani.

  python tools/engine_compare.py [quanti=60] [seme=1] [filtro del percorso, es. "Dance 90"]

Legge E:/Engine Library/Database2/m.db (copia di sola lettura), ricava per ogni brano la griglia di Engine (tempo costante) e la confronta
con quella che il programma calcola ora (test analyze_dump). Per ogni brano:
  - rapporto fra i tempi (giusto = 1, 2/3 = errore da 3 battiti, 2 o 0,5 = ottava);
  - scarto di fase mediano in ms (positivo = battiti di SpinLab dopo quelli di Engine) e dispersione (quanto i battiti si allontanano dal valore mediano);
  - deriva fra primo e ultimo quarto del brano.
"""
import json, os, random, shutil, sqlite3, struct, subprocess, sys, tempfile, zlib
import numpy as np

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
DB = r"E:\Engine Library\Database2\m.db"
MUSIC_ROOT = r"E:\\"


def engine_tracks(filtro):
    dst = os.path.join(tempfile.gettempdir(), "engine_m_copy.db")
    shutil.copy(DB, dst)
    db = sqlite3.connect(dst)
    out = {}
    for tid, path in db.execute("select id, path from Track where isAnalyzed=1 or bpmAnalyzed is not null"):
        if filtro.lower() not in path.lower():
            continue
        fs = os.path.normpath(os.path.join(os.path.dirname(DB), "..", path.replace("/", os.sep)))
        fs = os.path.normpath(os.path.join(MUSIC_ROOT, path.replace("../", "").replace("/", os.sep)))
        if not os.path.exists(fs):
            continue
        r = db.execute("select beatData from PerformanceData where trackId=?", (tid,)).fetchone()
        if not r or not r[0]:
            continue
        try:
            raw = zlib.decompress(r[0][4:])
            sr, total = struct.unpack(">dd", raw[:16])
            if raw[16] != 1:
                continue
            off = 17
            cnt = struct.unpack(">Q", raw[off:off + 8])[0]
            off += 8
            mk = [struct.unpack("<dqii", raw[off + 24 * k: off + 24 * k + 24]) for k in range(cnt)]
            if cnt < 2:
                continue
            (a, ia, _, _), (b, ib, _, _) = mk[0], mk[-1]
            step = (b - a) / (ib - ia)          # campioni per battito
            out[fs] = {"sr": sr, "t0": a / sr - ia * step / sr, "period": step / sr, "bpm": 60.0 * sr / step, "total": total / sr}
        except Exception:
            continue
    return out


def classify(r):
    for name, k in (("giusto", 1.0), ("ottava", 2.0), ("ottava", 0.5), ("3:2", 1.5), ("3:2", 2 / 3)):
        if abs(r / k - 1) < 0.03:
            return name
    return "altro"


def main():
    n = int(sys.argv[1]) if len(sys.argv) > 1 else 60
    seed = int(sys.argv[2]) if len(sys.argv) > 2 else 1
    filtro = sys.argv[3] if len(sys.argv) > 3 else ""
    eng = engine_tracks(filtro)
    files = sorted(eng)
    random.Random(seed).shuffle(files)
    files = files[:n]
    print(f"{len(eng)} brani con griglia Engine; ne provo {len(files)}")
    out = os.path.join(tempfile.gettempdir(), "spinlab_cmp.json")
    if os.path.exists(out):
        os.remove(out)
    lst = os.path.join(tempfile.gettempdir(), "spinlab_cmp_list.txt")
    with open(lst, "w", encoding="utf8") as fh:
        fh.write(chr(10).join(files))
    env = dict(os.environ, SPIN_LIST=lst, SPIN_OUT=out)
    subprocess.run([sys.executable, os.path.join(ROOT, "tools", "test.py"), "analyze_dump", "--ignored"], env=env, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
    res = json.load(open(out, encoding="utf8"))
    rows, cnt = [], {}
    for f in files:
        if f not in res:
            continue
        e, a = eng[f], res[f]
        beats = np.array(a["beats"])
        c = classify(a["bpm"] / e["bpm"])
        cnt[c] = cnt.get(c, 0) + 1
        P = e["period"]
        d = (beats - e["t0"]) % P
        d = np.where(d > P / 2, d - P, d) * 1000
        mid = d[(beats > e["total"] * 0.1) & (beats < e["total"] * 0.9)]
        if c != "giusto" or len(mid) < 30:
            rows.append((c, os.path.basename(f)[:46], e["bpm"], a["bpm"], None, None, None))
            continue
        med = float(np.median(mid))
        q = np.array_split(mid, 4)
        drift = abs(float(np.median(q[-1])) - float(np.median(q[0])))
        rows.append((c, os.path.basename(f)[:46], e["bpm"], a["bpm"], med, float(np.percentile(np.abs(mid - med), 90)), drift))
    print("\nrapporto dei tempi:", ", ".join(f"{k} {v}" for k, v in sorted(cnt.items())))
    print(f"{'brano':46s} Engine  SpinLab  scarto-ms  disp90-ms  deriva-ms")
    for r in sorted(rows, key=lambda r: (r[0] == "giusto", -(r[5] or 0))):
        if r[4] is None:
            print(f"{r[1]:46s} {r[2]:6.1f}  {r[3]:6.1f}   ({r[0]})")
        else:
            print(f"{r[1]:46s} {r[2]:6.1f}  {r[3]:6.1f}  {r[4]:+8.1f}  {r[5]:8.1f}  {r[6]:8.1f}")
    ok = [r for r in rows if r[4] is not None]
    if ok:
        med = np.array([r[4] for r in ok]); disp = np.array([r[5] for r in ok]); dr = np.array([r[6] for r in ok])
        print(f"\n{len(ok)} brani confrontabili: scarto mediano tipico {np.median(med):+.1f} ms (dev. {np.std(med):.1f})")
        print(f"  scarto entro 10 ms: {np.mean(np.abs(med) < 10) * 100:.0f}%   entro 25 ms: {np.mean(np.abs(med) < 25) * 100:.0f}%   entro 50 ms: {np.mean(np.abs(med) < 50) * 100:.0f}%")
        print(f"  dispersione (90%) tipica {np.median(disp):.1f} ms; sotto 25 ms: {np.mean(disp < 25) * 100:.0f}%   deriva sotto 25 ms: {np.mean(dr < 25) * 100:.0f}%")


if __name__ == "__main__":
    main()
