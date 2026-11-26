"use strict";
/* ============================================================ CONTROLLER MIDI
   Mappatura generica: ogni funzione dell'app (azione) si lega a un messaggio MIDI del controller (nota, CC o pitch bend)
   con il MIDI learn. Vale per qualsiasi controller MIDI; la mappatura si salva e si puo' esportare/importare. */

const MM = {
  bindings: [],          // { a: azione, k: "n|c|p:canale:numero", m: modo, sh: con SHIFT, port, ledOn, ledOff }
  out: undefined,        // porta di uscita per i LED ("" = nessuna, undefined = da scegliere)
  jogTicks: 800,         // messaggi del jog per un giro completo del piatto
  shift: false, learn: null, pend: null, last: null, seq: 0, lt: 0, ign: new Set(), ignWait: false, fxDecks: new Set([0]), fxSel: 0, fxHeld: new Map(), fxT: 0, rec: null, cal: null, ls: new Map(), sent: new Map(),
  layer: ["hc", "hc", "hc", "hc"], learnLy: null, // livello dei pad di ogni deck (i tasti Hot cue / Loop / Roll / Bank del controller)
  idx: new Map(), down: new Map(), msb: new Map(), lsb: new Map(), raw: new Map(),
};

/* ---------------------------------------------------------------- catalogo delle azioni */
const ACT = {};          // id -> { id, g (gruppo), l (nome), kind: btn|abs|rel, fn, lit }
const ACT_LIST = [];
function act(id, g, l, kind, fn, lit) { const a = { id, g, l, kind, fn, lit }; ACT[id] = a; ACT_LIST.push(a); }

const LAYERS = [["hc", "Hot cue"], ["loop", "Loop"], ["roll", "Roll"], ["bank", "Bank"]];
const isMod = (id) => id === "shift" || /\.ly\./.test(id || "");
const deckOfAct = (id) => { const m = /^d(\d)\./.exec(id); return m ? +m[1] : 0; };

const MP_JOG = [{}, {}];
function jogBegin(i) {
  const d = decks[i], j = MP_JOG[i];
  if (!d.dur || j.active) return;
  j.active = true; j.pos = currentPos(i); d.scrub = j.pos; dc(i, "hold", 1);
}
function jogEnd(i) {
  const d = decks[i], j = MP_JOG[i];
  if (!j.active) return;
  j.active = false; dc(i, "seek", j.pos); dc(i, "hold", 0);
  setTimeout(() => { if (!MP_JOG[i].active) d.scrub = null; }, 120);
}
function jogMove(i, n) {
  if (MM.cal) MM.cal.n += Math.abs(n);
  const d = decks[i], j = MP_JOG[i];
  if (!d.dur) return;
  if (j.touch || !d.snap?.playing) {
    // scratch: il piatto segue il jog (un giro del controller = un giro del piatto)
    if (!j.active) jogBegin(i);
    j.pos = Math.min(d.dur, Math.max(0, j.pos + (n / MM.jogTicks) * REV_SEC));
    d.scrub = j.pos;
    if (!j.raf) j.raf = requestAnimationFrame(() => { j.raf = 0; dc(i, "scrub", j.pos); });
    clearTimeout(j.idle);
    if (!j.touch) j.idle = setTimeout(() => jogEnd(i), 180);
  } else {
    // brano in riproduzione, jog non toccato: pitch bend momentaneo
    dc(i, "nudge", Math.max(-0.3, Math.min(0.3, n * 0.02)));
    clearTimeout(j.nt); j.nt = setTimeout(() => dc(i, "nudge", 0), 110);
  }
}

function libSteps(n, per) { for (let k = 0; k < Math.min(5, Math.abs(n)); k++) libMove(n > 0 ? per : -per); }

function buildActions() {
  for (let i = 0; i < activeDecks; i++) {
    const G = `Deck ${i + 1}`, id = (s) => `d${i}.${s}`, S = () => decks[i].snap || {};
    const D = () => DK[i], d = () => decks[i];
    act(id("play"), G, "Play / Pausa", "btn", (dn) => dn && dc(i, "play"), () => S().playing);
    act(id("cue"), G, "Cue (tieni premuto)", "btn", (dn) => dc(i, dn ? "cue_press" : "cue_release"));
    act(id("sync"), G, "Sync", "btn", (dn) => dn && D().sync.onclick(), () => S().sync);
    act(id("pfl"), G, "Cuffie (PFL)", "btn", (dn) => dn && D().pfl.onclick(), () => S().pfl);
    act(id("load"), G, "Carica il brano selezionato", "btn", (dn) => dn && D().r("load").onclick());
    act(id("eject"), G, "Svuota il deck", "btn", (dn) => dn && D().eject.onclick());
    act(id("target"), G, "Usa questo deck per FX e stem", "btn", (dn) => dn && setTarget(i));
    for (let k = 0; k < 8; k++) act(id("hot" + (k + 1)), G, `Hot cue ${k + 1}`, "btn", (dn) => dn && dc(i, "hotcue", k), () => S().hot_cues?.[k] != null);
    for (let k = 0; k < 8; k++) act(id("hotclr" + (k + 1)), G, `Cancella hot cue ${k + 1}`, "btn", (dn) => dn && dc(i, "hotcue_clear", k));
    act(id("loop"), G, "Loop acceso / spento", "btn", (dn) => dn && dc(i, "loop_toggle"), () => S().loop_on);
    // un solo tasto (per esempio il clic dell'encoder): 1° clic = IN, 2° = OUT e il loop parte, 3° = esce dal loop
    act(id("loopinout"), G, "Loop IN / OUT con un solo tasto", "btn", (dn) => {
      if (!dn) return;
      const dd = d(), st = dd._lio && dd._lio.m === dd.meta ? dd._lio.s : 0;
      if (S().loop_on) { dc(i, "loop_toggle"); dd._lio = { m: dd.meta, s: 0 }; }
      else if (st === 1) { dc(i, "loop_out"); dd._lio = { m: dd.meta, s: 2 }; }
      else { dc(i, "loop_in"); dd._lio = { m: dd.meta, s: 1 }; }
    }, () => S().loop_on);
    act(id("loopin"), G, "Loop IN", "btn", (dn) => dn && dc(i, "loop_in"));
    act(id("loopout"), G, "Loop OUT", "btn", (dn) => dn && dc(i, "loop_out"));
    act(id("loophalf"), G, "Loop ÷ 2", "btn", (dn) => dn && dc(i, "loop_halve"));
    act(id("loopdbl"), G, "Loop × 2", "btn", (dn) => dn && dc(i, "loop_double"));
    act(id("loopsize"), G, "Lunghezza loop (encoder)", "rel", (n) => dc(i, n > 0 ? "loop_double" : "loop_halve"));
    act(id("jumpb"), G, "Beat jump indietro", "btn", (dn) => dn && dc(i, "beatjump", -d().jumpSize));
    act(id("jumpf"), G, "Beat jump avanti", "btn", (dn) => dn && dc(i, "beatjump", d().jumpSize));
    for (let k = 0; k < 4; k++) act(id("roll" + (k + 1)), G, `Loop roll ${k + 1}`, "btn", (dn) => (dn ? dc(i, "roll_on", d().rollBase * [1, 2, 4, 8][k]) : dc(i, "roll_off")), () => S().rolling);
    // lunghezza base dei roll (i 4 pad fanno 1×, 2×, 4× e 8× questo valore): stessi tasti ‹ › della sezione pad
    const rollStep = (dir) => { const dd = d(), R = [1 / 16, 1 / 8, 1 / 4, 1 / 2, 1], k = Math.max(0, R.findIndex((x) => Math.abs(x - dd.rollBase) < 1e-9)); dd.rollBase = R[Math.min(R.length - 1, Math.max(0, k + dir))]; renderPads(i); toast(`Roll: ${dd.rollBase} battiti (pad ${[1, 2, 4, 8].map((m) => dd.rollBase * m).join(" · ")})`); };
    act(id("rollup"), G, "Roll: lunghezza più lunga", "btn", (dn) => dn && rollStep(1));
    act(id("rolldn"), G, "Roll: lunghezza più corta", "btn", (dn) => dn && rollStep(-1));
    act(id("rollsize"), G, "Roll: lunghezza (encoder)", "rel", (n) => rollStep(n > 0 ? 1 : -1));
    act(id("brake"), G, "Freno (tieni premuto)", "btn", (dn) => (dn ? dc(i, "brake_on", d().brakeT) : dc(i, "motion_off")), () => S().motion === 1);
    act(id("spin"), G, "Spinback", "btn", (dn) => dn && dc(i, "spin_on", d().brakeT), () => S().motion === 2);
    act(id("rev"), G, "Reverse (tieni premuto)", "btn", (dn) => (dn ? dc(i, "reverse_on") : dc(i, "motion_off")), () => S().motion === 3);
    act(id("slip"), G, "Slip", "btn", (dn) => dn && dc(i, "slip", S().slip ? 0 : 1), () => S().slip);
    act(id("quant"), G, "Quantize", "btn", (dn) => dn && dc(i, "quantize", S().quantize ? 0 : 1), () => S().quantize);
    act(id("keylock"), G, "Key lock", "btn", (dn) => dn && dc(i, "keylock", S().keylock ? 0 : 1), () => S().keylock);
    act(id("range"), G, "Escursione del pitch", "btn", (dn) => dn && D().range.onclick());
    for (let k = 0; k < 4; k++) act(id("pad" + (k + 1)), G, `Pad ${k + 1} (nella modalità scelta)`, "btn", (dn) => d().padAction(k, dn));
    act(id("padmode"), G, "Cambia modalità dei pad", "btn", (dn) => {
      if (!dn) return;
      const modes = ["hc0", "jump", "roll", "brake"], dd = d();
      dd.mode = modes[(modes.indexOf(dd.mode) + 1) % modes.length]; closeRolls(i); renderPads(i);
    });
    act(id("fxon"), G, "Effetto acceso / spento", "btn", (dn) => {
      const dd = d(), t0 = performance.now();
      if (dn) { dd.fx.on = !dd.fx.on; dd.sendFx(); dd._fxT = t0; }
      else if ((dd.fx.kind === "brake" || dd.fx.kind === "rev") && dd.fx.on && t0 - (dd._fxT || 0) > 350) { dd.fx.on = false; dd.sendFx(); }
    }, () => d().fx.on);
    [["Batteria", 0], ["Basso", 1], ["Altro", 2], ["Voce", 3]].forEach(([n, k]) => act(id("stem" + k), G, `Stem ${n} (muto)`, "btn", (dn) => {
      if (!dn) return;
      const dd = d();
      if (dd.stGains[k] > 0.001) { dd.stMem[k] = dd.stGains[k]; dd.stGains[k] = 0; } else dd.stGains[k] = dd.stMem[k] || 1;
      touchStems(i);
    }, () => d().stGains[k] <= 0.001));
    for (const [k, n] of LAYERS) act(id("ly." + k), G, `Livello dei pad: ${n} (tasto di modalità)`, "btn", (dn) => dn && (MM.layer[i] = k), () => MM.layer[i] === k);
    act(id("jogtouch"), G, "Jog: tocco del piatto", "btn", (dn) => {
      const j = MP_JOG[i]; j.touch = dn;
      if (dn) { clearTimeout(j.idle); jogBegin(i); } else jogEnd(i);
    });
    act(id("jog"), G, "Jog (scratch e pitch bend)", "rel", (n) => jogMove(i, n));
    act(id("browse"), G, "Libreria: scorri i brani (encoder)", "rel", (n) => libSteps(n, 1));
    act(id("browsefast"), G, "Libreria: scorri veloce, 10 brani per scatto (encoder)", "rel", (n) => libSteps(n, 10));
    act(id("browseload"), G, "Libreria: carica il brano scelto (premi l'encoder)", "btn", (dn) => dn && D().r("load").onclick());
    // fader e manopole
    act(id("vol"), G, "Volume del canale", "abs", (v) => D().vol._apply(v));
    act(id("hi"), G, "EQ alti", "abs", (v) => D().knobs.find((k) => k.dataset.k === "2")._apply(v));
    act(id("mid"), G, "EQ medi", "abs", (v) => D().knobs.find((k) => k.dataset.k === "1")._apply(v));
    act(id("low"), G, "EQ bassi", "abs", (v) => D().knobs.find((k) => k.dataset.k === "0")._apply(v));
    act(id("filter"), G, "Filtro", "abs", (v) => D().knobs.find((k) => k.dataset.k === "filter")._apply(v));
    act(id("tempo"), G, "Pitch (fader)", "abs", (v) => D().tempo._apply(v));
    act(id("fxwet"), G, "Effetto: MIX", "abs", (v) => { d().fx.wet = v; d().sendFx(); });
    act(id("fxp1"), G, "Effetto: parametro 1", "abs", (v) => { d().fx.param = v; d().sendFx(); });
    act(id("fxp2"), G, "Effetto: parametro 2", "abs", (v) => { d().fx.param2 = v; d().sendFx(); });
  }
  // sezione effetti di un controller con un solo set di comandi (pulsante on/off, encoder dei parametri, manopola MIX, tasti 1 / 2 per scegliere il deck)
  const FG = "Effetti del controller (deck assegnati con i tasti FX 1 / 2)";
  const fxMark = () => { $("#fxL1")?.classList.toggle("sel", MM.fxSel === 0); $("#fxL2")?.classList.toggle("sel", MM.fxSel === 1); };
  const fxTargets = () => [...MM.fxDecks].filter((i) => i < activeDecks).map((i) => decks[i]);
  fxMark();
  const fxAssign = (set) => { MM.fxDecks = new Set(set); setTarget(Math.min(...set)); };
  for (let i = 0; i < activeDecks; i++) {
    act(`fx.deck${i}`, FG, `Assegna l'effetto al deck ${i + 1} (con due tasti premuti insieme: entrambi)`, "btn", (dn) => {
      if (!dn) return;
      // due tasti premuti a meno di 0,4 s uno dall'altro = entrambi i deck; altrimenti solo quello premuto
      const now = performance.now(), recent = [...MM.fxHeld].filter(([k, t]) => k !== i && now - t < 400).map(([k]) => k);
      MM.fxHeld.set(i, now);
      fxAssign(recent.length ? [...recent, i] : [i]);
      toast(MM.fxDecks.size > 1 ? "Effetti: entrambi i deck" : `Effetti: deck ${i + 1}`);
    }, () => MM.fxDecks.has(i));
  }
  act("fx.both", FG, "Assegna l'effetto a tutti i deck", "btn", (dn) => { if (dn) { fxAssign([...Array(activeDecks).keys()]); toast("Effetti: entrambi i deck"); } }, () => MM.fxDecks.size >= activeDecks);
  act("fx.on", FG, "Effetto acceso / spento (sui deck assegnati)", "btn", (dn) => {
    const ds = fxTargets(), now = performance.now();
    if (dn) {
      const on = !ds.every((d) => d.fx.on);
      ds.forEach((d) => { d.fx.on = on; d.sendFx(); });
      MM.fxT = now;
    } else if (now - MM.fxT > 350) {
      // freno e reverse durano quanto la pressione
      ds.forEach((d) => { if ((d.fx.kind === "brake" || d.fx.kind === "rev") && d.fx.on) { d.fx.on = false; d.sendFx(); } });
    }
  }, () => fxTargets().some((d) => d.fx.on));
  // Encoder dei parametri: gli scatti hanno un'accelerazione (piano = fine, veloce = largo) e il valore ci arriva con una piccola rampa,
  // cosi' non si sente a gradini. Premere l'encoder lo sposta un po': gli scatti di quel momento si scartano (altrimenti toccano
  // il parametro vecchio e poi quello nuovo).
  const R = MM.fxRot = { pend: 0, timer: 0, burst: 0, last: 0, lastApplied: 0, ignoreUntil: 0, tgt: new Map(), raf: 0 };
  const fxEase = () => {
    R.raf = 0; let more = false;
    for (const [key, t] of [...R.tgt]) {
      const [di, k] = key.split("|"), d = decks[+di], diff = t - d.fx[k];
      if (Math.abs(diff) < 0.0005) { d.fx[k] = t; R.tgt.delete(key); } else { d.fx[k] += diff * 0.35; more = true; }
      d.sendFx();
    }
    if (more) R.raf = requestAnimationFrame(fxEase);
  };
  const fxMove = (n) => {
    const now = performance.now(), dt = now - R.lastApplied; R.lastApplied = now;
    const f = dt < 25 ? 4 : dt < 60 ? 2.5 : dt < 120 ? 1.5 : 1;
    [...MM.fxDecks].filter((i) => i < activeDecks).forEach((i) => {
      const d = decks[i], k = MM.fxSel && (FXMAP[d.fx.kind] || FXMAP.echo)[1] ? "param2" : "param", key = i + "|" + k;
      R.tgt.set(key, Math.min(1, Math.max(0, (R.tgt.get(key) ?? d.fx[k]) + n * 0.025 * f)));
    });
    if (!R.raf) R.raf = requestAnimationFrame(fxEase);
  };
  act("fx.enc", FG, "Effetto: parametro (encoder)", "rel", (n) => {
    const now = performance.now();
    if (now < R.ignoreUntil) return;
    R.burst = now - R.last < 150 ? R.burst + 1 : 1; R.last = now;
    if (R.burst > 3) { R.pend += n; clearTimeout(R.timer); const p = R.pend; R.pend = 0; fxMove(p); return; }
    // i primi scatti aspettano un attimo: se subito dopo arriva il clic dell'encoder erano solo il tocco
    R.pend += n; clearTimeout(R.timer);
    R.timer = setTimeout(() => { const p = R.pend; R.pend = 0; if (p) fxMove(p); }, 70);
  });
  act("fx.encclick", FG, "Effetto: cambia il parametro dell'encoder (clic)", "btn", (dn) => {
    const now = performance.now();
    R.pend = 0; clearTimeout(R.timer); R.burst = 0;
    R.ignoreUntil = now + (dn ? 400 : 200);
    if (!dn) return;
    const d = fxTargets()[0]; if (!d) return;
    const m = FXMAP[d.fx.kind] || FXMAP.echo;
    // l'encoder spesso manda piu' messaggi per un solo clic (rimbalzo): conta uno solo ogni 0,25 s
    if (now - (R.clickT || 0) < 250) return;
    R.clickT = now;
    MM.fxSel = m[1] ? 1 - MM.fxSel : 0;
    fxMark();
    toast(`Encoder FX: ${m[MM.fxSel] || m[0]}`);
  });
  act("fx.wet", FG, "Effetto: MIX dry / wet (manopola)", "abs", (v) => fxTargets().forEach((d) => { d.fx.wet = v; d.sendFx(); }));
  const fxKindStep = (dir) => {
    const sel = $("#fxKind"), kinds = [...sel.options].map((o) => o.value);
    fxTargets().forEach((d) => { const k = Math.max(0, kinds.indexOf(d.fx.kind)); d.fx.kind = kinds[(k + dir + kinds.length) % kinds.length]; d.sendFx(); });
    const d = fxTargets()[0]; if (d) toast(`Effetto: ${sel.options[Math.max(0, kinds.indexOf(d.fx.kind))]?.textContent || d.fx.kind}`);
  };
  act("fx.next", FG, "Effetto successivo", "btn", (dn) => dn && fxKindStep(1));
  act("fx.prev", FG, "Effetto precedente", "btn", (dn) => dn && fxKindStep(-1));
  act("fx.kindenc", FG, "Scelta dell'effetto (encoder)", "rel", (n) => fxKindStep(n > 0 ? 1 : -1));

  const G = "Mixer e libreria";
  act("automix", G, "AUTOMIX avvia / ferma", "btn", (dn) => dn && (AM.on ? amStop() : amStart()), () => AM.on);
  act("automixskip", G, "AUTOMIX: salta al prossimo brano", "btn", (dn) => dn && amSkip());
  act("shift", G, "SHIFT", "btn", (dn) => { MM.shift = dn; }, () => MM.shift);
  act("xfader", G, "Crossfader", "abs", (v) => $("#crossfader")._apply(v));
  act("master", G, "Volume master", "abs", (v) => $("#masterKnob")._apply(v));
  act("cuemix", G, "Cue mix", "abs", (v) => $("#cueMix")._apply(v));
  act("phones", G, "Volume cuffie", "abs", (v) => $("#hpVol")._apply(v));
  act("sampvol", G, "Volume campionatore", "abs", (v) => { const e = $("#samplerVol"); e.value = v * 1.5; e.dispatchEvent(new Event("input")); });
  act("drumvol", G, "Volume batteria", "abs", (v) => { const e = $("#drumVol"); e.value = v * 1.5; e.dispatchEvent(new Event("input")); });
  act("browse", G, "Sfoglia la libreria (encoder)", "rel", (n) => libSteps(n, 1));
  act("browseup", G, "Libreria: brano precedente", "btn", (dn) => dn && libMove(-1));
  act("browsedn", G, "Libreria: brano successivo", "btn", (dn) => dn && libMove(1));
}

/* ---------------------------------------------------------------- decodifica dei messaggi */
const KINDS = { 0x80: "n", 0x90: "n", 0xB0: "c", 0xE0: "p" };
const keyOf = (st, d1) => `${KINDS[st & 0xF0]}:${st & 15}:${(st & 0xF0) === 0xE0 ? 0 : d1}`;
const MODES = {
  btn: [["btn", "Pulsante"], ["cw", "Encoder: solo rotazione oraria"], ["ccw", "Encoder: solo rotazione antioraria"], ["cwo", "Encoder (offset 64): oraria"], ["ccwo", "Encoder (offset 64): antioraria"]],
  abs: [["abs", "Assoluto 7 bit"], ["abs14", "Assoluto 14 bit (MSB + LSB)"]],
  rel: [["rel", "Relativo (complemento a 2)"], ["reloff", "Relativo (offset 64)"], ["relsb", "Relativo (bit di segno)"], ["absd", "Assoluto, usato come differenza"]],
};

function reindex() {
  MM.idx.clear();
  for (const b of MM.bindings) {
    const [kind, ch, num] = b.k.split(":");
    if (!b.id) b.id = ++MM.seq;
    b.kind = kind; b.ch = +ch; b.num = +num;
    const add = (key) => { if (!MM.idx.has(key)) MM.idx.set(key, []); MM.idx.get(key).push(b); };
    add(b.k);
    if (b.m === "abs14" && kind === "c" && b.num < 32) add(`c:${b.ch}:${b.num + 32}`);
  }
}

function btnDown(hi, d1, d2) {
  if (hi === 0x90) return d2 > 0;
  if (hi === 0x80) return false;
  if (hi === 0xB0) return d2 >= 64;
  if (hi === 0xE0) return (d1 | (d2 << 7)) > 8192;
  return false;
}

function applyMsg(b, hi, d1, d2) {
  const a = ACT[b.a];
  if (!a) return;
  if (a.kind === "btn" && b.m !== "btn") {
    // un encoder usato come pulsante: la funzione scatta a ogni scatto nella direzione scelta (es. orario = Loop × 2, antiorario = Loop ÷ 2)
    if (hi !== 0xB0) return;
    const n = b.m.endsWith("o") ? d2 - 64 : d2 < 64 ? d2 : d2 - 128;
    if (n && (n > 0) === b.m.startsWith("cw")) { a.fn(true); a.fn(false); }
  } else if (a.kind === "btn") {
    const dn = btnDown(hi, d1, d2);
    if (hi === 0x80 && !MM.down.has(b)) return;
    if (dn) MM.down.set(b, b); else MM.down.delete(b);
    a.fn(dn);
  } else if (a.kind === "abs") {
    let v;
    if (hi === 0xE0) v = (d1 | (d2 << 7)) / 16383;
    else if (b.m === "abs14") {
      const key = `${b.ch}:${b.num}`;
      if (d1 === b.num) MM.msb.set(key, d2); else MM.lsb.set(key, d2);
      v = (((MM.msb.get(key) || 0) << 7) | (MM.lsb.get(key) || 0)) / 16383;
    } else v = d2 / 127;
    a.fn(Math.min(1, Math.max(0, v)));
  } else {
    let n;
    if (hi === 0xE0 || b.m === "absd") {
      const raw = hi === 0xE0 ? (d1 | (d2 << 7)) : d2, span = hi === 0xE0 ? 16384 : 128, prev = MM.raw.get(b);
      MM.raw.set(b, raw);
      if (prev === undefined) return;
      n = raw - prev; if (n > span / 2) n -= span; else if (n < -span / 2) n += span;
    } else if (b.m === "reloff") n = d2 - 64;
    else if (b.m === "relsb") n = d2 > 64 ? -(d2 - 64) : d2;
    else n = d2 < 64 ? d2 : d2 - 128;
    if (n) a.fn(n);
  }
}

function midiIn(port, st, d1, d2) {
  const hi = st & 0xF0;
  if (!KINDS[hi]) return;
  const key = keyOf(st, d1);
  MM.last = { port, st, d1, d2, t: performance.now() };
  // sensori di tocco (l'encoder che si sfiora prima di cliccare...): si possono escludere del tutto, cosi' non finiscono nel learn ne' comandano nulla
  if (MM.ignWait) {
    if ((hi === 0x90 && d2 > 0) || (hi === 0xB0 && d2 > 0)) { MM.ign.add(key); MM.ignWait = false; $("#mmIgnore").textContent = "Ignora un tasto…"; renderIgn(); saveMap(); toast(`Ignorato: ${describeKey(key)}`); }
    return;
  }
  if (MM.ign.has(key)) return;
  if (MM.rec) { recPush(port, st, d1, d2); return; }
  if (MM.learn) {
    // durante il learn l'eco dei LED che abbiamo appena acceso (alcune porte rimandano indietro quello che ricevono) non e' un comando
    const se = MM.sent.get(key);
    if (se !== undefined && performance.now() - se < 300) return;
    // durante il learn il tasto SHIFT gia' assegnato continua a funzionare: cosi' si puo' assegnare SHIFT + tasto
    const sb = !isMod(MM.learn) && MM.bindings.find((x) => isMod(x.a) && x.k === key);
    if (sb) {
      applyMsg(sb, hi, d1, d2);
      // un tasto di livello premuto durante il learn: la funzione che si sta assegnando vale solo in quel livello
      const lm = /\.ly\.(\w+)/.exec(sb.a);
      if (lm && btnDown(hi, d1, d2)) MM.learnLy = lm[1];
    } else learnCapture(port, hi, key, d1, d2);
    return;
  }
  const c = MM.idx.get(key);
  if (!c) return;
  // un messaggio puo' comandare piu' funzioni: valgono quelle con lo stesso stato di SHIFT (o, se non ce ne sono, quelle senza SHIFT)
  let list = c.filter((x) => !!x.sh === MM.shift);
  if (!list.length) list = c.filter((x) => !x.sh);
  // livelli dei pad: se ci sono assegnazioni per il livello attuale del deck valgono solo quelle, altrimenti quelle valide sempre
  const inLy = list.filter((x) => x.ly && MM.layer[deckOfAct(x.a)] === x.ly);
  list = inLy.length ? inLy : list.filter((x) => !x.ly);
  // il rilascio va alle stesse funzioni che il tasto ha acceso, anche se SHIFT e' cambiato nel frattempo
  if (!btnDown(hi, d1, d2)) list = [...new Set([...c.filter((x) => MM.down.has(x)), ...list])];
  for (const b of list) applyMsg(b, hi, d1, d2);
}

/* ---------------------------------------------------------------- learn */
function learnCapture(port, hi, key, d1, d2) {
  const a = ACT[MM.learn];
  if (!a) { MM.learn = null; MM.pend = null; return; }
  const down = btnDown(hi, d1, d2), cont = hi === 0xB0 || hi === 0xE0;
  const P = MM.pend;
  // rilascio del primo tasto premuto senza che ne sia arrivato un altro: e' lui il tasto da assegnare
  if (P && key === P.key) {
    if (down) return;
    MM.pend = null;
    if (a.kind === "btn") bindLearned(a, P.hi, P.key, P.port, false);
    return;
  }
  // un valore piccolo su un CC e' un encoder che gira in senso orario: la funzione si assegna alla rotazione
  const enc = a.kind === "btn" && hi === 0xB0 && d2 > 0 && d2 < 64;
  if (a.kind === "btn" ? !(down || enc) : !cont) return;
  if (P) {
    // un secondo controllo mentre il primo e' tenuto premuto: il primo e' SHIFT, il secondo la funzione
    const sb = MM.bindings.find((x) => x.a === "shift");
    if (a.id !== "shift" && !sb) {
      const nb = { a: "shift", k: P.key, m: "btn", sh: false, port: P.port, ledOn: 127, ledOff: 0 };
      MM.bindings.push(nb); reindex();
      MM.shift = true; MM.down.set(nb, nb);
      toast("Il tasto tenuto premuto è stato assegnato a SHIFT");
    }
    MM.pend = null;
    finishLearn(a, hi, key, d1, d2, port, a.id !== "shift" && MM.shift, enc);
    return;
  }
  // un pulsante potrebbe essere il tasto SHIFT di una combinazione: si aspetta di vedere se viene rilasciato da solo
  if (a.kind === "btn" && !enc && a.id !== "shift" && !MM.shift && !MM.bindings.some((x) => x.a === "shift")) { MM.pend = { key, hi, port }; return; }
  finishLearn(a, hi, key, d1, d2, port, a.id !== "shift" && MM.shift, enc);
}

// I controlli continui si assegnano solo dopo un movimento vero: un sensore che manda sempre lo stesso valore o un po' di rumore non conta.
// Con il movimento si riconosce anche il tipo di encoder (jog compreso).
function finishLearn(a, hi, key, d1, d2, port, sh, enc) {
  if (a.kind === "btn" && !enc) return bindLearned(a, hi, key, port, sh, "");
  const now = performance.now(), pb = hi === 0xE0, v = pb ? (d1 | (d2 << 7)) : d2;
  if (!MM.lt || now - MM.lt > 1500) { MM.ls.clear(); MM.lt = now; }
  let st = MM.ls.get(key);
  if (!st || now - st.t > 1000) { st = { x: [], t: now, hi, port }; MM.ls.set(key, st); }
  st.t = now; st.x.push(v);
  const x = st.x, span = Math.max(...x) - Math.min(...x);
  let mode = null;
  if (a.kind === "btn") { if (x.length >= 2) mode = "cw"; }
  else if (a.kind === "abs") { if (span >= (pb ? 1500 : 12)) mode = "abs"; }
  else if (pb) { if (x.length >= 3 && span > 0) mode = "absd"; }
  else {
    // un jog manda spesso piu' messaggi insieme (posizione, velocita'...): si aspetta un attimo e si sceglie quello che si comporta da encoder
    if (now - MM.lt < 250) return;
    let best = null;
    for (const [k, c] of MM.ls) {
      if (c.hi === 0xE0 || c.x.length < 4) continue;
      const m = detectRel(c.x); if (!m) continue;
      const sc = (m === "absd" ? 3 : new Set(c.x).size > 1 ? 2 : 1) * 1000 + c.x.length;
      if (!best || sc > best.sc) best = { sc, k, m, c };
    }
    if (!best) return;
    const [kind, ch, num] = best.k.split(":");
    return bindLearned(a, best.c.hi, best.k, best.c.port, sh, best.m);
  }
  if (mode) bindLearned(a, hi, key, port, sh, mode);
}

function detectRel(x) {
  if (x.length < 4) return null;
  const d = [];
  for (let i = 1; i < x.length; i++) { let t = x[i] - x[i - 1]; if (t > 64) t -= 128; if (t < -64) t += 128; d.push(t); }
  if (new Set(x).size >= 4 && (d.every((t) => t >= 1 && t <= 8) || d.every((t) => t <= -1 && t >= -8))) return "absd"; // valore assoluto che gira
  if (x.every((v) => v >= 44 && v <= 84)) return "reloff";
  if (x.every((v) => v <= 20 || v >= 108)) return "rel";
  if (x.every((v) => v <= 20 || (v >= 65 && v <= 84))) return "relsb";
  return x.length >= 16 ? "rel" : null;
}

function bindLearned(a, hi, key, port, sh, mode) {
  const ly = isMod(a.id) ? "" : MM.learnLy || "";
  // una funzione puo' avere piu' assegnazioni e un messaggio puo' comandare piu' funzioni; si sostituisce solo la stessa coppia funzione + messaggio
  MM.bindings = MM.bindings.filter((b) => !(b.a === a.id && b.k === key && !!b.sh === sh && (b.ly || "") === ly));
  MM.bindings.push({ a: a.id, ly, k: key, m: mode || (a.kind === "btn" ? "btn" : a.kind === "abs" ? "abs" : hi === 0xE0 ? "absd" : "rel"), sh, port, ledOn: 127, ledOff: 0 });
  MM.learn = null; MM.pend = null; MM.learnLy = null; MM.raw.clear(); MM.ls.clear();
  reindex(); saveMap(); renderMap();
}

const describeKey = (k) => {
  const [kind, ch, num] = k.split(":");
  return `${kind === "n" ? "Nota " + num : kind === "c" ? "CC " + num : "Pitch bend"} · canale ${+ch + 1}`;
};
const NOTES = ["Do", "Do#", "Re", "Re#", "Mi", "Fa", "Fa#", "Sol", "Sol#", "La", "La#", "Si"];
function describeMsg(m) {
  if (!m) return "— muovi un controllo —";
  const hi = m.st & 0xF0, ch = (m.st & 15) + 1;
  const t = hi === 0x90 ? (m.d2 ? `Nota ${m.d1} (${NOTES[m.d1 % 12]}${Math.floor(m.d1 / 12) - 1}) premuta, velocità ${m.d2}` : `Nota ${m.d1} rilasciata`)
    : hi === 0x80 ? `Nota ${m.d1} rilasciata`
    : hi === 0xB0 ? `CC ${m.d1} = ${m.d2}`
    : `Pitch bend = ${m.d1 | (m.d2 << 7)}`;
  return `${t} · canale ${ch} · ${m.port}`;
}

/* ---------------------------------------------------------------- salvataggio */
let saveT = 0;
function mapJson() {
  return JSON.stringify({ v: 1, out: MM.out ?? null, jogTicks: MM.jogTicks, ignore: [...MM.ign], bindings: MM.bindings.map(({ a, k, m, sh, ly, port, ledOn, ledOff }) => ({ a, k, m, sh, ly: ly || "", port, ledOn, ledOff })) }, null, 1);
}
function saveMap() { clearTimeout(saveT); saveT = setTimeout(() => inv("midi_map_set", { json: mapJson() }).catch(() => {}), 300); }
function loadMapText(t) {
  const j = JSON.parse(t);
  MM.bindings = (j.bindings || []).filter((b) => ACT[b.a] && /^[ncp]:\d+:\d+$/.test(b.k)).map((b) => ({ a: b.a, k: b.k, m: b.m || "btn", sh: !!b.sh, ly: LAYERS.some((l) => l[0] === b.ly) ? b.ly : "", port: b.port || "", ledOn: b.ledOn ?? 127, ledOff: b.ledOff ?? 0 }));
  if (j.out !== undefined) MM.out = j.out === null ? undefined : j.out;
  if (j.jogTicks) MM.jogTicks = j.jogTicks;
  MM.ign = new Set((j.ignore || []).filter((k) => /^[ncp]:\d+:\d+$/.test(k)));
  reindex();
}

/* ---------------------------------------------------------------- LED del controller */
function ledMsg(b, on) {
  const v = on ? b.ledOn : b.ledOff;
  if (b.kind === "n") return [0x90 | b.ch, b.num, v];
  if (b.kind === "c") return [0xB0 | b.ch, b.num, v];
  return null;
}
function ledTick() {
  if (!MM.out) return;
  for (const b of MM.bindings) {
    const a = ACT[b.a];
    if (!a?.lit || a.kind !== "btn" || b.m !== "btn" || b.led === false) continue;
    let on = false; try { on = !!a.lit(); } catch (e) {}
    if (b._led === on) continue;
    b._led = on;
    const m = ledMsg(b, on); if (m) { MM.sent.set(keyOf(m[0], m[1]), performance.now()); inv("midi_send", { bytes: m }).catch(() => {}); }
  }
}
setInterval(ledTick, 90);

/* ---------------------------------------------------------------- pannello nelle impostazioni */
const mmOpen = new Set();
function renderMap() {
  const box = $("#mmList"); if (!box) return;
  const q = ($("#mmFilter").value || "").toLowerCase(), keep = box.scrollTop;
  const groups = [...new Set(ACT_LIST.map((a) => a.g))];
  box.innerHTML = groups.map((g) => {
    const acts = ACT_LIST.filter((a) => a.g === g && (!q || (a.g + " " + a.l).toLowerCase().includes(q)));
    if (!acts.length) return "";
    const n = acts.filter((a) => MM.bindings.some((b) => b.a === a.id)).length;
    return `<details data-g="${esc(g)}" ${q || mmOpen.has(g) || MM.learn && ACT[MM.learn]?.g === g ? "open" : ""}><summary>${esc(g)} <i>${n}/${acts.length} assegnati</i></summary>` + acts.map((a) => {
      const bs = MM.bindings.filter((x) => x.a === a.id), learning = MM.learn === a.id, modes = MODES[a.kind];
      const learnBtn = `<button class="mmL">${learning ? "Annulla" : bs.length ? "+ Altro" : "Learn"}</button>`;
      const hint = learning ? `<span class="mmb">muovi o premi un controllo… (tieni SHIFT o premi un tasto di livello per una combinazione)</span>` : "";
      const rows = bs.length ? bs : [null];
      return rows.map((b, n) => `<div class="mmrow ${learning && n === 0 ? "learning" : ""} ${b ? "set" : ""}" data-a="${a.id}" ${b ? `data-b="${b.id}"` : ""}><span class="mml">${n === 0 ? esc(a.l) : ""}</span>`
        + (learning && n === 0 ? hint : `<span class="mmb">${b ? (b.sh ? "SHIFT + " : "") + esc(describeKey(b.k)) + sharedWith(b) : "—"}</span>`)
        + `<select class="mmm" ${b && modes.length > 1 ? "" : "disabled"}>${modes.map(([v, t]) => `<option value="${v}" ${b?.m === v ? "selected" : ""}>${t}</option>`).join("")}</select>`
        + `<select class="mmy" title="Livello dei pad: la funzione vale solo quando il deck è in questo livello (tasti Hot cue / Loop / Roll / Bank del controller)" ${b && !isMod(a.id) ? "" : "disabled"}><option value="">Ogni livello</option>${LAYERS.map(([v, t]) => `<option value="${v}" ${b?.ly === v ? "selected" : ""}>${t}</option>`).join("")}</select>`
        + `<label class="mms" title="La funzione vale solo con il tasto SHIFT premuto"><input type="checkbox" ${b?.sh ? "checked" : ""} ${b ? "" : "disabled"}>SHIFT</label>`
        + (a.lit ? `<span class="mmled" title="Valori inviati per accendere e spegnere il LED del tasto (serve l'uscita LED)"><input type="number" min="0" max="127" value="${b?.ledOn ?? 127}" data-led="on" ${b ? "" : "disabled"}><input type="number" min="0" max="127" value="${b?.ledOff ?? 0}" data-led="off" ${b ? "" : "disabled"}></span>` : `<span class="mmled"></span>`)
        + (n === 0 ? learnBtn : "<span></span>") + `<button class="mmX" ${b ? "" : "disabled"} title="Cancella questa assegnazione">✕</button></div>`).join("");
    }).join("") + `</details>`;
  }).join("");
  box.scrollTop = keep;
}

// avverte quando lo stesso messaggio (con lo stesso SHIFT e livello) comanda anche altre funzioni: succede se si riassegna un controllo senza cancellare la vecchia assegnazione
function sharedWith(b) {
  const o = MM.bindings.filter((x) => x !== b && x.k === b.k && !!x.sh === !!b.sh && (x.ly || "") === (b.ly || ""));
  if (!o.length) return "";
  const names = o.map((x) => `${ACT[x.a]?.g || ""}: ${ACT[x.a]?.l || x.a}`).join(" · ");
  return ` <span class="mmw" title="Questo stesso messaggio comanda anche: ${esc(names)}">⚠ anche ${esc(names)}</span>`;
}

function renderIgn() {
  const b = $("#mmIgnList"); if (!b) return;
  b.innerHTML = [...MM.ign].map((k) => `<span class="mmign" data-k="${k}">${esc(describeKey(k))} <button title="Smetti di ignorare">✕</button></span>`).join("") || "<i>nessuno</i>";
}

async function refreshMidiPane() {
  try {
    const ports = await inv("midi_ports");
    $("#mmPorts").textContent = ports.length ? ports.join(" · ") : "nessun controller MIDI collegato";
    const o = await inv("midi_out_info");
    // se non e' mai stata scelta, prova ad abbinare l'uscita al controller collegato (stesso nome senza «MIDIIN/MIDIOUT»)
    const norm = (n) => n.toLowerCase().replace(/midi(in|out)\d*\s*/g, "").replace(/[()]/g, "").trim();
    if (MM.out === undefined && ports.length) {
      const g = o.ports.find((p) => ports.some((q) => norm(q) === norm(p)));
      if (g) { MM.out = g; await inv("midi_out_select", { name: g }).catch(() => {}); saveMap(); }
    }
    $("#mmOut").innerHTML = `<option value="">Nessuna (niente LED)</option>` + o.ports.map((p) => `<option ${p === (MM.out || o.current) ? "selected" : ""}>${esc(p)}</option>`).join("");
  } catch (e) { /* porte non leggibili */ }
  $("#mmJog").value = MM.jogTicks;
  renderMap(); renderIgn();
}

function bindMidiPane() {
  const box = $("#mmList");
  box.addEventListener("click", (e) => {
    const row = e.target.closest(".mmrow"); if (!row) return;
    const id = row.dataset.a;
    if (e.target.closest(".mmL")) { MM.pend = null; MM.learnLy = null; MM.ls.clear(); MM.learn = MM.learn === id ? null : id; mmOpen.add(ACT[id].g); renderMap(); }
    else if (e.target.closest(".mmX")) { MM.bindings = MM.bindings.filter((b) => b.id !== +row.dataset.b); reindex(); saveMap(); renderMap(); }
  });
  box.addEventListener("toggle", (e) => { const g = e.target.dataset?.g; if (g) { if (e.target.open) mmOpen.add(g); else mmOpen.delete(g); } }, true);
  box.addEventListener("change", (e) => {
    const row = e.target.closest(".mmrow"); if (!row) return;
    const b = MM.bindings.find((x) => x.id === +row.dataset.b); if (!b) return;
    if (e.target.matches(".mmm")) { b.m = e.target.value; reindex(); }
    else if (e.target.matches(".mms input")) b.sh = e.target.checked;
    else if (e.target.matches(".mmy")) b.ly = e.target.value;
    else if (e.target.matches("[data-led]")) { b[e.target.dataset.led === "on" ? "ledOn" : "ledOff"] = Math.max(0, Math.min(127, +e.target.value || 0)); b._led = undefined; }
    saveMap();
  });
  $("#mmFilter").oninput = renderMap;
  $("#mmOut").onchange = async () => {
    MM.out = $("#mmOut").value;
    try { await inv("midi_out_select", { name: MM.out }); } catch (e) { toast(String(e)); }
    MM.bindings.forEach((b) => (b._led = undefined)); saveMap();
  };
  $("#mmJog").onchange = () => { MM.jogTicks = Math.max(50, Math.min(8000, +$("#mmJog").value || 800)); $("#mmJog").value = MM.jogTicks; saveMap(); };
  $("#mmExport").onclick = () => { $("#mmJson").value = mapJson(); $("#mmJsonBox").hidden = false; $("#mmJsonApply").hidden = true; $("#mmJson").select(); };
  $("#mmImport").onclick = () => { $("#mmJson").value = ""; $("#mmJsonBox").hidden = false; $("#mmJsonApply").hidden = false; $("#mmJson").focus(); };
  $("#mmJsonApply").onclick = () => {
    try { loadMapText($("#mmJson").value); saveMap(); renderMap(); $("#mmJsonBox").hidden = true; toast("Mappatura importata"); } catch (e) { toast("Mappatura non valida: " + e.message); }
  };
  $("#mmJsonClose").onclick = () => ($("#mmJsonBox").hidden = true);
  $("#mmAnalyze").onclick = () => {
    const b = $("#mmAnalyze");
    if (MM.rec) return;
    MM.rec = { map: new Map() }; MM.learn = null; let t = 8;
    const tick = () => {
      if (--t > 0) { b.textContent = `Muovi il controllo… ${t}`; MM.rec.timer = setTimeout(tick, 1000); return; }
      const rep = recReport(); MM.rec = null; b.textContent = "Analizza un controllo";
      $("#mmJson").value = rep; $("#mmJsonBox").hidden = false; $("#mmJsonApply").hidden = true;
    };
    b.textContent = `Muovi il controllo… ${t}`; MM.rec.timer = setTimeout(tick, 1000);
  };
  $("#mmCal").onclick = () => {
    const b = $("#mmCal");
    if (!MM.cal) { MM.cal = { n: 0 }; b.textContent = "Fatto un giro: premi qui"; toast("Ora gira il piatto di un giro intero (con il brano caricato), poi premi di nuovo il pulsante"); return; }
    const n = Math.round(MM.cal.n); MM.cal = null; b.textContent = "Calibra il jog";
    if (n >= 20) { MM.jogTicks = Math.min(8000, n); $("#mmJog").value = MM.jogTicks; saveMap(); toast(`Jog: ${MM.jogTicks} scatti per giro`); }
    else toast("Il jog non ha mandato abbastanza messaggi: assegnalo e riprova con un brano caricato");
  };
  $("#mmIgnore").onclick = () => {
    MM.ignWait = !MM.ignWait; MM.learn = null; MM.pend = null;
    $("#mmIgnore").textContent = MM.ignWait ? "Tocca il tasto da ignorare… (premi qui per annullare)" : "Ignora un tasto…";
    renderMap();
  };
  $("#mmIgnList").addEventListener("click", (e) => { const c = e.target.closest(".mmign"); if (c && e.target.closest("button")) { MM.ign.delete(c.dataset.k); renderIgn(); saveMap(); } });
  $("#mmClear").onclick = async () => { if (await askConfirm("Cancellare tutte le assegnazioni MIDI?", "", { ok: "Cancella", danger: true })) { MM.bindings = []; reindex(); saveMap(); renderMap(); } };
  // monitor: ultimo messaggio ricevuto, solo con il pannello aperto
  setInterval(() => {
    const m = $("#mmMon"); if (!m || !$("#audioDlg").open || $("#sp-midi").hidden) return;
    m.textContent = describeMsg(MM.last);
  }, 100);
  document.addEventListener("keydown", (e) => { if (e.key === "Escape" && MM.learn) { MM.learn = null; MM.pend = null; MM.learnLy = null; renderMap(); } }, true);
}

/* ---------------------------------------------------------------- analisi di un controllo (per capire cosa manda il jog) */
function recPush(port, st, d1, d2) {
  const hi = st & 0xF0, key = `${port}|${hi.toString(16)}|${st & 15}|${hi === 0xE0 ? 0 : d1}`;
  let r = MM.rec.map.get(key);
  if (!r) MM.rec.map.set(key, (r = { port, hi, ch: (st & 15) + 1, num: d1, vals: [] }));
  r.vals.push(hi === 0xE0 ? (d1 | (d2 << 7)) : d2);
}
function recReport() {
  const rows = [...MM.rec.map.values()].sort((a, b) => b.vals.length - a.vals.length);
  if (!rows.length) return "Nessun messaggio ricevuto: il controllo non manda MIDI (o la porta non è collegata).";
  return "Messaggi ricevuti (il più frequente per primo):\n\n" + rows.map((r) => {
    const name = r.hi === 0xB0 ? `CC ${r.num}` : r.hi === 0xE0 ? "Pitch bend" : r.hi === 0x90 ? `Nota ${r.num} (on)` : r.hi === 0x80 ? `Nota ${r.num} (off)` : `Stato ${r.hi.toString(16)} ${r.num}`;
    const h = new Map(); r.vals.forEach((v) => h.set(v, (h.get(v) || 0) + 1));
    const top = [...h.entries()].sort((a, b) => b[1] - a[1]).slice(0, 14).map(([v, n]) => `${v}×${n}`).join("  ");
    let hint = "";
    if ((r.hi === 0xB0) && r.vals.length >= 4) { const m = detectRel(r.vals.slice(0, 40)); if (m) hint = `\n   modo probabile: ${m}`; }
    return `${name} · canale ${r.ch} · ${r.vals.length} messaggi\n   valori: ${top}\n   primi 40: ${r.vals.slice(0, 40).join(" ")}${hint}`;
  }).join("\n\n");
}

/* ---------------------------------------------------------------- avvio */
buildActions();
bindMidiPane();
T.event.listen("midi-in", (e) => { for (const [port, st, d1, d2] of e.payload) midiIn(port, st, d1, d2); });
inv("midi_map_get").then((t) => {
  if (t) { try { loadMapText(t); } catch (e) { /* mappatura illeggibile: si riparte vuoti */ } }
  if (MM.out) inv("midi_out_select", { name: MM.out }).catch(() => {});
  renderMap();
});
