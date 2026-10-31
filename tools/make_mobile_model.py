"""Crea htdemucs_mobile.onnx, il modello degli stem per i telefoni Android, a partire da htdemucs.onnx.

    python tools/make_mobile_model.py percorso/htdemucs.onnx percorso/htdemucs_mobile.onnx

Serve: pip install onnx onnxslim (onnxruntime solo per --verify).

Cosa cambia rispetto al modello normale (il risultato e' lo stesso, differenze sotto 1e-4):
1. i pesi fp16 diventano fp32 e spariscono i Cast che li convertivano a ogni apertura;
2. il grafo si semplifica con l'ingresso a misura fissa (circa 25 000 nodi -> 1 500): si apre in 0,6 s invece di 5,5;
3. l'iSTFT, che il modello fa con due ConvTranspose da 2049x4096 (45 miliardi di moltiplicazioni per segmento),
   diventa una IDFT + overlap-add.

Misure (CPU, 4 thread): su Mac un segmento da 7,8 s passa da 1,72 a 1,54 s. Sul Galaxy S26 Ultra il modello si apre
in 0,65 s invece di 6,4, ma un segmento costa uguale (2,9-3,3 s): la DFT di ORT su ARM (0,37 s) costa quasi quanto
le ConvTranspose che sostituisce (0,47 s). Per guadagnare davvero l'iSTFT andrebbe fatta in Rust con rustfft.
"""
import argparse
import numpy as np
import onnx
from onnx import TensorProto, helper, numpy_helper

N, HOP, BINS, FRAMES, OUT_LEN = 4096, 1024, 2049, 340, 347136


def fp16_weights_to_fp32(m):
    g = m.graph
    inits = {i.name: i for i in g.initializer}
    rename, drop = {}, []
    for n in g.node:
        if n.op_type == "Cast" and n.input[0] in inits and inits[n.input[0]].data_type == TensorProto.FLOAT16:
            if [a.i for a in n.attribute if a.name == "to"][0] == TensorProto.FLOAT:
                rename[n.output[0]] = n.input[0]
                drop.append(n)
    for n in drop:
        g.node.remove(n)
    targets = set(rename.values())
    for i in g.initializer:
        if i.data_type == TensorProto.FLOAT16 and i.name in targets:
            i.CopyFrom(numpy_helper.from_array(numpy_helper.to_array(i).astype(np.float32), i.name))
    for n in g.node:
        for k, x in enumerate(n.input):
            if x in rename:
                n.input[k] = rename[x]
    print(f"cast tolti: {len(drop)}")


def check_istft_weights(init, re_w, im_w):
    """L'iSTFT del modello deve essere quella attesa: finestra di Hann, pesi 1/64 (2/64 per i bin intermedi)."""
    wr = numpy_helper.to_array(init[re_w])[:, 0, :].astype(np.float64)
    wi = numpy_helper.to_array(init[im_w])[:, 0, :].astype(np.float64)
    n, k = np.arange(N), np.arange(BINS)[:, None]
    win = 0.5 - 0.5 * np.cos(2 * np.pi * n / N)
    a = np.full((BINS, 1), 2 / 64)
    a[0] = a[-1] = 1 / 64
    er = np.abs(wr - a * np.cos(2 * np.pi * k * n / N) * win).max()
    ei = np.abs(wi + a * np.sin(2 * np.pi * k * n / N) * win).max()
    if max(er, ei) > 1e-4:
        raise SystemExit(f"iSTFT diversa da quella attesa (errore {er:.2e}, {ei:.2e}): il modello non e' quello previsto")


def fft_istft(m):
    g = m.graph
    byname = {n.name: n for n in g.node}
    init = {i.name: i for i in g.initializer}
    check_istft_weights(init, byname["/real_istft/ConvTranspose"].input[1], byname["/real_istft/ConvTranspose_1"].input[1])
    re_t = byname["/real_istft/Gather_5"].output[0]  # parte reale [8, 2049, 340]
    im_t = byname["/real_istft/Gather_6"].output[0]  # parte immaginaria
    env = byname["/real_istft/Div"].input[1]  # inviluppo delle finestre [347136]
    out_name = byname["/real_istft/Div"].output[0]  # [8, 347136]
    for nm in ["/real_istft/ConvTranspose", "/real_istft/ConvTranspose_1", "/real_istft/Add", "/real_istft/Gather_7", "/real_istft/Slice", "/real_istft/Div"]:
        g.node.remove(byname[nm])
    used = set(x for n in g.node for x in n.input) | {env}
    for i in list(g.initializer):
        if i.name not in used:
            g.initializer.remove(i)
    P = "/fft_istft/"
    nodes = []

    def c(name, arr):
        g.initializer.append(numpy_helper.from_array(np.asarray(arr), P + name))
        return P + name

    def node(op, ins, outs, **kw):
        nodes.append(helper.make_node(op, ins, [P + o for o in outs], name=P + outs[0], **kw))
        return P + outs[0]

    # finestra di Hann x 64: la IDFT di ONNX divide per N, i pesi originali per 64
    win = ((0.5 - 0.5 * np.cos(2 * np.pi * np.arange(N) / N)) * (N / 64)).astype(np.float32)
    re = node("Unsqueeze", [node("Transpose", [re_t], ["re"], perm=[0, 2, 1]), c("ax_last", np.array([-1], np.int64))], ["re1"])
    im = node("Unsqueeze", [node("Transpose", [im_t], ["im"], perm=[0, 2, 1]), P + "ax_last"], ["im1"])
    half = node("Concat", [re, im], ["half"], axis=-1)  # [8, 340, 2049, 2]
    # spettro intero: i bin 2047..1 coniugati (Gather: lo Slice a passo -1 e' lento in ORT)
    mir = node("Gather", [half, c("mi", np.arange(BINS - 2, 0, -1, dtype=np.int64))], ["mir"], axis=2)
    mirc = node("Mul", [mir, c("conj", np.array([1.0, -1.0], np.float32))], ["mirc"])
    full = node("Concat", [half, mirc], ["full"], axis=2)  # [8, 340, 4096, 2]
    x = node("DFT", [full], ["idft"], axis=2, inverse=1, onesided=0)
    frames = node("Mul", [node("Gather", [x, c("i0", np.array(0, np.int64))], ["real"], axis=3), c("win", win)], ["frames"])
    # overlap-add con passo N/4: ogni quarto di finestra va sommato in uno di 4 blocchi sfalsati
    f4 = node("Reshape", [frames, c("s4", np.array([8, FRAMES, 4, HOP], np.int64))], ["f4"])
    node("Split", [f4, c("sp", np.array([1, 1, 1, 1], np.int64))], ["p0", "p1", "p2", "p3"], axis=2)
    acc = None
    for r in range(4):
        q = node("Squeeze", [P + f"p{r}", c(f"ax2_{r}", np.array([2], np.int64))], [f"q{r}"])
        pd = node("Pad", [q, c(f"pad{r}", np.array([0, r, 0, 0, 3 - r, 0], np.int64))], [f"pd{r}"])
        acc = pd if acc is None else node("Add", [acc, pd], [f"acc{r}"])
    y = node("Reshape", [acc, c("sy", np.array([8, (FRAMES + 3) * HOP], np.int64))], ["y"])
    cut = node("Slice", [y, c("y0", np.array([N // 2], np.int64)), c("y1", np.array([N // 2 + OUT_LEN], np.int64)), c("ya", np.array([1], np.int64))], ["ycut"])
    nodes.append(helper.make_node("Div", [cut, env], [out_name], name=P + "div"))
    idx = next(i for i, n in enumerate(g.node) if out_name in n.input)
    for k, n in enumerate(nodes):
        g.node.insert(idx + k, n)


def verify(src, dst):
    import onnxruntime as ort
    seg = 343_980
    rng = np.random.default_rng(1)
    t = np.arange(seg) / 44100
    x = (0.3 * np.sin(2 * np.pi * 110 * t) * (np.sin(2 * np.pi * 2 * t) > 0) + 0.05 * rng.standard_normal(seg)).astype(np.float32)
    x = np.stack([x, np.roll(x, 300)])[None]
    a = ort.InferenceSession(src, providers=["CPUExecutionProvider"]).run(None, {"mix": x})[0]
    b = ort.InferenceSession(dst, providers=["CPUExecutionProvider"]).run(None, {"mix": x})[0]
    d = np.abs(a - b).max()
    print(f"differenza massima dal modello originale: {d:.2e}")
    if d > 1e-3:
        raise SystemExit("il modello per telefoni non da' lo stesso risultato")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("src")
    ap.add_argument("dst")
    ap.add_argument("--verify", action="store_true", help="confronta l'uscita con il modello originale (serve onnxruntime)")
    a = ap.parse_args()
    import onnxslim
    m = onnx.load(a.src)
    fp16_weights_to_fp32(m)
    m = onnxslim.slim(m)
    print(f"nodi dopo la semplificazione: {len(m.graph.node)}")
    fft_istft(m)
    onnx.checker.check_model(m)
    onnx.save(m, a.dst)
    print(f"scritto {a.dst}")
    if a.verify:
        verify(a.src, a.dst)


if __name__ == "__main__":
    main()
