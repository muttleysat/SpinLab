"use strict";
// barra errori: mostra a schermo gli errori JS e dei comandi invece di perderli nella console
function toast(msg) {
  let t = document.getElementById("toast");
  if (!t) { t = document.createElement("div"); t.id = "toast"; document.body.appendChild(t); }
  t.textContent = msg; t.style.display = "block"; clearTimeout(t._h); t._h = setTimeout(() => (t.style.display = "none"), 9000);
}
window.addEventListener("error", (e) => toast("Errore: " + e.message + " (" + (e.filename || "").split("/").pop() + ":" + e.lineno + ")"));
window.addEventListener("unhandledrejection", (e) => toast("Errore: " + (e.reason?.message || e.reason)));
const T = window.__TAURI__;
const inv = (c, a = {}) => T.core.invoke(c, a);
const $ = (s, r = document) => r.querySelector(s);
const $$ = (s, r = document) => [...r.querySelectorAll(s)];
const COLORS = ["#00d6ff", "#ff3d9a", "#ffb020", "#44e07a"];
const RANGES = [4, 8, 16, 50, 100];
const fmtc = (s) => { s = Math.max(0, s || 0); const c = Math.floor(s * 100); return `${Math.floor(c / 6000)}:${String(Math.floor(c / 100) % 60).padStart(2, "0")}.${String(c % 100).padStart(2, "0")}`; };
const fmt = (s) => { s = Math.max(0, s || 0); return `${Math.floor(s / 60)}:${String(Math.floor(s % 60)).padStart(2, "0")}`; };
const esc = (s) => String(s ?? "").replace(/[&<>"]/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;" }[c]));
const dc = (i, cmd, a = 0, b = 0) => inv("deck_cmd", { deck: i, cmd, a, b });

/* =================================================================== COMPONENTI */
const clamp01 = (x) => Math.min(1, Math.max(0, x));

/* adattamento alla finestra: la grafica e' disegnata per BASE_W x BASE_H e si scala in blocco per riempire la finestra,
   rimpicciolendosi se e' piu' piccola (o lo zoom di Windows e' alto) e ingrandendosi se e' piu' grande (schermo pieno, 1440p, 4K) */
const BASE_W = 1240, BASE_H = 940, MAX_K = 3;
let uiK = 1;
function fitUI() {
  uiK = Math.min(MAX_K, innerHeight / BASE_H, innerWidth / BASE_W);
  document.documentElement.style.setProperty("--k", uiK.toFixed(4));
  requestAnimationFrame(resizeCanvases);
}
window.addEventListener("resize", fitUI);
fitUI();

/* manopola rotante: trascina in verticale (o con la rotella), doppio clic = valore iniziale */
function makeKnob(el, { value, def, onChange }) {
  let v = value;
  const draw = () => { el.style.setProperty("--a", (-135 + v * 270) + "deg"); el.style.setProperty("--p", v); };
  const set = (x, emit = true) => { v = clamp01(x); draw(); if (emit) onChange(v); };
  let st = null;
  el.addEventListener("pointerdown", (e) => { if (e.button !== 0) return; el.setPointerCapture(e.pointerId); st = { y: e.clientY, v }; });
  el.addEventListener("pointermove", (e) => { if (st) set(st.v + (st.y - e.clientY) / (e.shiftKey ? 600 : 150)); });
  ["pointerup", "pointercancel"].forEach((ev) => el.addEventListener(ev, () => (st = null)));
  el.addEventListener("dblclick", () => set(def));
  el.addEventListener("wheel", (e) => { e.preventDefault(); set(v + (e.deltaY < 0 ? 0.03 : -0.03)); }, { passive: false });
  el._set = (x) => { if (!st) { v = clamp01(x); draw(); } };
  el._apply = (x) => set(x); // da un controller MIDI: muove la manopola e invia il valore
  set(value, false);
  return el;
}

/* fader a cursore: verticale (0 in basso, 1 in alto) o orizzontale (0 a sinistra, 1 a destra) */
function makeFader(el, { value = 0, def = null, onChange }) {
  const vertical = el.classList.contains("v");
  el.innerHTML = '<div class="ftrack"></div><div class="fticks"></div><div class="ffill"></div><div class="fcap"></div>';
  let v = value, drag = false;
  const draw = () => el.style.setProperty("--p", v);
  const from = (e) => {
    const r = el.getBoundingClientRect(), pad = 18 * uiK;
    return vertical ? 1 - clamp01((e.clientY - r.top - pad) / Math.max(1, r.height - 2 * pad)) : clamp01((e.clientX - r.left - pad) / Math.max(1, r.width - 2 * pad));
  };
  const set = (x) => { v = clamp01(x); draw(); onChange?.(v); };
  el.addEventListener("pointerdown", (e) => { if (e.button !== 0) return; el.setPointerCapture(e.pointerId); drag = true; set(from(e)); });
  el.addEventListener("pointermove", (e) => { if (drag) set(from(e)); });
  ["pointerup", "pointercancel"].forEach((ev) => el.addEventListener(ev, () => (drag = false)));
  el.addEventListener("dblclick", () => { if (def != null) set(def); });
  el.addEventListener("wheel", (e) => { e.preventDefault(); set(v + (e.deltaY < 0 ? 0.02 : -0.02)); }, { passive: false });
  el._set = (x) => { if (!drag) { v = clamp01(x); draw(); } };
  el._apply = (x) => set(x); // da un controller MIDI: muove il cursore e invia il valore
  el._dragging = () => drag;
  draw();
  return el;
}

const IC = {
  play: '<svg viewBox="0 0 24 24" width="26" height="26"><path d="M7 4.5v15l13-7.5z" fill="currentColor"/></svg>',
  pause: '<svg viewBox="0 0 24 24" width="24" height="24"><path d="M6.5 4h4v16h-4zM13.5 4h4v16h-4z" fill="currentColor"/></svg>',
  head: '<svg viewBox="0 0 24 24" width="20" height="20"><path d="M4 15v-3a8 8 0 0 1 16 0v3M4 15h3v5H5a1 1 0 0 1-1-1zM20 15h-3v5h2a1 1 0 0 0 1-1z" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linejoin="round"/></svg>',
  note: '<svg viewBox="0 0 24 24" width="22" height="22"><path d="M9 18V6l10-2v12" fill="none" stroke="currentColor" stroke-width="2"/><circle cx="7" cy="18" r="2.5" fill="currentColor"/><circle cx="17" cy="16" r="2.5" fill="currentColor"/></svg>',
  mic: '<svg viewBox="0 0 24 24" width="18" height="18"><rect x="9" y="3" width="6" height="11" rx="3" fill="none" stroke="currentColor" stroke-width="1.8"/><path d="M5 11a7 7 0 0 0 14 0M12 18v3" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round"/></svg>',
  drum: '<svg viewBox="0 0 24 24" width="18" height="18"><ellipse cx="12" cy="13" rx="8" ry="4" fill="none" stroke="currentColor" stroke-width="1.8"/><path d="M4 13v4c0 2 3.6 3.5 8 3.5s8-1.5 8-3.5v-4M7 4l5 6M17 4l-5 6" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round"/></svg>',
  bass: '<svg viewBox="0 0 24 24" width="18" height="18"><path d="M3 12h3l2-6 4 12 3-9 2 3h4" fill="none" stroke="currentColor" stroke-width="1.9" stroke-linecap="round" stroke-linejoin="round"/></svg>',
  inst: '<svg viewBox="0 0 24 24" width="18" height="18"><rect x="3" y="6" width="18" height="12" rx="2" fill="none" stroke="currentColor" stroke-width="1.8"/><path d="M8 6v7M12 6v7M16 6v7" stroke="currentColor" stroke-width="1.8"/></svg>',
};
const PAL = [
  { c: "#18b6ff", lo: "#0a4dff", mid: "#18b6ff", hi: "#d2f4ff" },
  { c: "#ff7a1a", lo: "#ff3b1f", mid: "#ff8a1f", hi: "#ffe9d0" },
  { c: "#b06cff", lo: "#6a2cff", mid: "#a76bff", hi: "#eee2ff" },
  { c: "#35e08a", lo: "#12a85a", mid: "#35e08a", hi: "#d6ffe9" },
];
const HC = ["#ff3b4e", "#ffc21a", "#35e08a", "#18b6ff"];
const REV_SEC = 1.8; // un giro del piatto (33 giri e 1/3 al minuto)
const camelotColor = (k) => { const n = parseInt(k, 10); return n ? `hsl(${((n - 1) * 30 + 190) % 360} 75% 64%)` : "inherit"; };
const mmss = (s) => { s = Math.max(0, Math.round(s || 0)); return `${String(Math.floor(s / 60)).padStart(2, "0")}:${String(s % 60).padStart(2, "0")}`; };

/* =================================================================== DECKS */
const decks = [0, 1].map(() => ({
  wave: null, beats: [], downbeat: 0, dur: 0, snap: null, t: 0, over: null, zoom: 7, scrub: null, art: null, armT: 0, armR: 0.9,
  fx: { kind: "echo", on: false, wet: 0.5, param: 0.5, param2: 0.5, div: 3, feel: 0 }, hcPage: 0, pk: [0, 0], pkT: [0, 0], mode: "hc0", jumpSize: 4, rollBase: 0.125, brakeT: 1.5, meta: null, stTouch: 0, stGains: [1, 1, 1, 1], stMem: [1, 1, 1, 1], stText: "",
}));
let audioLatency = 0; // secondi di buffer del dispositivo: la grafica si ritarda di tanto per coincidere con il suono
const activeDecks = 2;
const DK = []; // riferimenti agli elementi di ogni deck e della sua striscia del mixer

function deckHTML(i) {
  return `<div class="deck${i % 2 ? " rev" : ""}" data-i="${i}">
  <div class="dhead">
    <div class="art" data-r="art">${i + 1}</div>
    <div class="meta"><div class="title" data-r="title">Trascina qui un brano o usa ${IC.note.replace(/width="22" height="22"/, 'width="13" height="13" style="display:inline;vertical-align:-2px"')}</div><div class="artist"><span data-r="artist"></span><span class="stm" data-r="stm"></span></div></div>
    <div class="dload"><button data-r="load" title="Carica sul deck il brano selezionato in libreria (senza selezione apre la scelta file)">LOAD</button><button data-r="eject" title="Svuota il deck">&#9167;</button></div>
    <div class="led" data-r="beat"></div>
    <div class="kv key"><small>KEY</small><b data-r="key">--</b></div>
    <div class="kv"><small>BPM</small><b data-r="bpm">--</b></div>
    <div class="clock2"><b data-r="remain">-00:00</b><span data-r="time">00:00 / 00:00</span></div>
  </div>
  <div class="wv"><canvas class="zoom"></canvas></div>
  <div class="wv"><canvas class="over"></canvas></div>
  <div class="dmain">
    <div class="lcol">
      <div class="card"><div class="ch"><button class="modebtn" data-r="mode" title="Cambia la funzione dei 4 pad: hot cue, beatjump, loop roll, freno/reverse">HOT CUE 1–4 ⌄</button></div>
        <div class="hcgrid">${[0, 1, 2, 3].map((k) => `<button data-hot="${k}">${k + 1}</button>`).join("")}</div>
        <div class="prow"><button data-r="pdn" title="Valore precedente">‹</button><span class="pv" data-r="pval"></span><button data-r="pup" title="Valore successivo">›</button><button class="chip2" data-r="qbtn" title="QUANTIZE: cue, loop e roll partono sulla griglia dei battiti">Q</button><button class="chip2" data-r="slipbtn" title="SLIP: durante roll, freno e reverse il brano continua in silenzio e al rilascio ti ritrovi dove sarebbe arrivato">SLIP</button></div></div>
      <div class="card loopc"><div class="ch"><span>LOOP</span></div>
        <div class="lr"><button data-loop="halve" title="Dimezza il loop">&lsaquo;</button><button class="lsz" data-r="lsz" title="Attiva o disattiva il loop">4</button><button data-loop="double" title="Raddoppia il loop">&rsaquo;</button></div>
        <div class="io"><button data-loop="in" title="Inizio del loop">IN</button><button data-loop="out" title="Fine del loop (lo attiva)">OUT</button></div></div>
      <div class="transport"><button class="rbtn cue" data-r="cue">CUE</button><button class="rbtn play" data-r="play">${IC.play}</button></div>
    </div>
    <div class="platter"><canvas class="plat"></canvas>
      <button class="pbtn bl" data-r="grid" title="Griglia: correzioni, anchor, FLUIDA/FISSA">GRID</button>
      <button class="pbtn qb" data-r="qpill" title="QUANTIZE: cue, loop, roll e hot cue si agganciano ai battiti della griglia">QUANT</button>
      <button class="pbtn br" data-r="pfl" title="Preascolto in cuffia (PFL)">${IC.head}</button></div>
    <div class="rcol">
      <button class="rbtn2" data-r="range" title="Intervallo del pitch: clic per cambiarlo (4, 8, 16, 50, 100%)">±8</button>
      <div class="pf"><div class="fader v noFill" data-r="tempo" title="Pitch · doppio clic per azzerare"></div></div>
      <div class="pval" data-r="tempoval">0.0%</div>
      <button class="rbtn2" data-r="sync" title="SYNC: tempo e fase seguono il deck guida. Resta acceso anche cambiando brano, finché non lo spegni (o muovi a mano il pitch di questo deck)">SYNC</button>
      <button class="rbtn2 rg" data-r="keybtn" title="Tonalità: key lock, Camelot, semitoni">KEY</button>
    </div>
  </div>
</div>`;
}

function stripHTML(i) {
  return `<div class="strip" data-i="${i}">
  <div class="kcol">
    <div class="fxsel"><select data-r="fxkind" title="Effetto del deck ${i + 1}"><option value="echo">ECHO</option><option value="pingpong">P-PONG</option><option value="reverb">REVERB</option><option value="flanger">FLANG</option><option value="phaser">PHASE</option><option value="crush">CRUSH</option><option value="gater">GATER</option><option value="scatter">SCATTER</option><option value="noise">NOISE</option><option value="bjump">B.JUMP</option><option value="brake">BRAKE</option><option value="spin">SPINBK</option><option value="rev">REVERSE</option></select><button class="pwr" data-r="fxon" title="Accende l'effetto"><i></i></button></div>
    <div class="knobwrap"><div class="knob" data-k="filter" title="Filtro: a sinistra passa-basso, a destra passa-alto · doppio clic per azzerare"></div><span>FILTER</span></div>
    <div class="knobwrap"><div class="knob" data-k="2"></div><span>HI</span></div>
    <div class="knobwrap"><div class="knob" data-k="1"></div><span>MID</span></div>
    <div class="knobwrap"><div class="knob" data-k="0"></div><span>LOW</span></div>
  </div>
  <div class="mcol">
    <div class="mmeters"><div class="fwrap"><div class="meter vert" data-r="meterL" title="Livello sinistro del canale (dopo il fader)"><u></u></div><div class="meter vert" data-r="meterR" title="Livello destro del canale (dopo il fader)"><u></u></div><div class="fader v noFill big" data-r="vol" title="Volume del canale: i meter dietro il cursore mostrano il livello"></div></div></div>
    <button class="hpbtn" data-r="pfl2" title="Preascolto in cuffia (PFL)">${IC.head}</button>
  </div>
</div>`;
}

function buildDecks() {
  $("#sideL").innerHTML = deckHTML(0);
  $("#sideR").innerHTML = deckHTML(1);
  $("#strips").innerHTML = [0, 1].map(stripHTML).join("");
  [0, 1].forEach((i) => bindDeck(i));
  // crossfader
  const xf = $("#crossfader");
  makeFader(xf, { value: 0.5, def: 0.5, onChange: (v) => inv("mixer_cmd", { cmd: "crossfader", a: v }) });
  initDecks();
}

function initDecks() {
  buildTargets();
  requestAnimationFrame(resizeCanvases);
}

/* i 4 pad di ogni deck: etichette e parametro della modalita' scelta */
function renderPads(i) {
  const d = decks[i], D = DK[i], f = d.frac || ((x) => String(x));
  const names = { hc0: "HOT CUE 1–4", hc1: "HOT CUE 5–8", jump: "BEAT JUMP", roll: "LOOP ROLL", brake: "FRENO / REV" };
  D.modebtn.textContent = names[d.mode] + " ⌄";
  const hc = d.mode === "hc0" || d.mode === "hc1";
  const labels = d.mode === "jump" ? ["◀◀", "◀", "▶", "▶▶"].map((x, k) => x) : d.mode === "roll" ? [1, 2, 4, 8].map((m) => f(d.rollBase * m)) : d.mode === "brake" ? ["BRAKE", "SPIN", "REV", "SLIP"] : [1, 2, 3, 4].map((k) => String(k + (d.mode === "hc1" ? 4 : 0)));
  D.hot.forEach((b, k) => {
    b.textContent = labels[k]; b.classList.toggle("sm", !hc && d.mode !== "roll" && d.mode !== "jump" || d.mode === "brake");
    b.title = d.mode === "jump" ? `Salta di ${[-2, -1, 1, 2][k] * d.jumpSize} battiti` : d.mode === "roll" ? `Loop roll di ${labels[k]} battito: tienilo premuto` : d.mode === "brake" ? ["Freno: tieni premuto, il disco si ferma", "Spinback: il disco gira all'indietro e il brano riparte", "Reverse: tieni premuto per andare al contrario", "SLIP acceso/spento"][k] : "Clic: imposta o salta al hot cue · Maiusc+clic o tasto destro: cancella";
  });
  D.qbtn.textContent = hc ? "QUANT" : "Q";
  D.pdn.hidden = D.pup.hidden = hc; D.pval.hidden = hc; D.slipbtn.hidden = hc || d.mode === "jump";
  D.pval.textContent = d.mode === "jump" ? `±${f(d.jumpSize)}` : d.mode === "roll" ? `${f(d.rollBase)}` : d.mode === "brake" ? `${d.brakeT.toFixed(1)}s` : "";
  D.pval.title = d.mode === "jump" ? "Lunghezza del salto in battiti (i pad fanno ±1× e ±2×)" : d.mode === "roll" ? "Lunghezza del primo pad (gli altri sono 2×, 4×, 8×)" : "Durata di freno e spinback";
}
function closeRolls(i) { dc(i, "roll_off"); dc(i, "motion_off"); }

function bindDeck(i) {
  const el = $(`.deck[data-i="${i}"]`), st = $(`.strip[data-i="${i}"]`), d = decks[i];
  const r = (n) => $(`[data-r="${n}"]`, el), sr = (n) => $(`[data-r="${n}"]`, st);
  DK[i] = {
    el, strip: st, r, sr,
    art: r("art"), title: r("title"), artist: r("artist"), stm: r("stm"), key: r("key"), bpm: r("bpm"), remain: r("remain"), time: r("time"), beat: r("beat"),
    play: r("play"), cue: r("cue"), sync: r("sync"), pfl: r("pfl"), pfl2: sr("pfl2"), range: r("range"), tempo: r("tempo"), tempoval: r("tempoval"),
    lsz: r("lsz"), hot: $$("[data-hot]", el), modebtn: r("mode"), pval: r("pval"), pdn: r("pdn"), pup: r("pup"), qbtn: r("qbtn"), qpill: r("qpill"), slipbtn: r("slipbtn"), prow: $(".prow", el), grid: r("grid"), meterL: sr("meterL"), meterR: sr("meterR"), eject: r("eject"), vol: sr("vol"), fxkind: sr("fxkind"), fxon: sr("fxon"),
    knobs: $$("[data-k]", st), plat: $("canvas.plat", el), zoomCv: $("canvas.zoom", el), over: $("canvas.over", el),
  };
  const D = DK[i];
  D.play.onclick = () => dc(i, "play");
  D.cue.addEventListener("pointerdown", () => dc(i, "cue_press"));
  ["pointerup", "pointerleave"].forEach((ev) => D.cue.addEventListener(ev, () => dc(i, "cue_release")));
  // SYNC e' un interruttore per deck: puo' essere acceso su tutti i deck e resta acceso (anche cambiando brano) finche' non lo spegni
  D.sync.onclick = () => dc(i, d.snap?.sync ? "sync_off" : "sync");
  const pfl = () => dc(i, "pfl", d.snap?.pfl ? 0 : 1);
  D.pfl.onclick = pfl; D.pfl2.onclick = pfl;
  D.eject.onclick = () => ejectDeck(i);
  D.r("load").onclick = async () => {
    const sel = lib.rows.find((t) => lib.sel.has(t.id));
    if (sel) loadTrack(i, sel.path); else { const p = await pickFile(); if (p) loadTrack(i, p); }
  };
  // pitch: in alto meno, in basso piu' veloce (come sui lettori CDJ); doppio clic = zero
  makeFader(D.tempo, { value: 0.5, def: 0.5, onChange: (v) => dc(i, "tempo", (1 - 2 * v) * (d.snap?.range_pct ?? 8)) });
  D.range.onclick = () => { const cur = Math.round(d.snap?.range_pct ?? 8), k = RANGES.indexOf(cur); dc(i, "range", RANGES[(k + 1) % RANGES.length]); };
  // mixer: volume, EQ, filtro, preascolto, FX
  makeFader(D.vol, { value: 0.85, def: 0.85, onChange: (v) => dc(i, "volume", v) });
  D.knobs.forEach((k) => {
    const id = k.dataset.k;
    makeKnob(k, { value: 0.5, def: 0.5, onChange: (v) => (id === "filter" ? dc(i, "filter", v) : dc(i, "eq", +id, v)) });
  });
  const sendFx = () => {
    const f = d.fx, mot = MOTFX.has(f.kind);
    if (d.fxMot && (!mot || d.fxMot !== f.kind)) { dc(i, "motion_off"); d.fxMot = null; if (mot) f.on = false; } // cambio effetto: rilascia il movimento in corso
    if (mot) {
      if (f.on && !d.fxMot) {
        const T = 0.3 + f.param * 3.7;
        if (f.kind === "brake") { dc(i, "brake_on", T); d.fxMot = "brake"; }
        else if (f.kind === "rev") { dc(i, "reverse_on"); d.fxMot = "rev"; }
        else if (f.kind === "spin") { dc(i, "spin_on", T); f.on = false; d.fxFlash = performance.now() + 400; }
        else { dc(i, "beatjump", (f.param2 < 0.5 ? -1 : 1) * JUMPSZ[Math.min(JUMPSZ.length - 1, Math.floor(f.param * JUMPSZ.length))]); f.on = false; d.fxFlash = performance.now() + 250; }
      } else if (!f.on && d.fxMot) { dc(i, "motion_off"); d.fxMot = null; }
    }
    // gli effetti sul brano non passano dal motore FX: li si lascia spenti
    inv("fx_set", { deck: i, kind: mot ? "echo" : f.kind, on: mot ? false : f.on, wet: f.wet, param: f.param, param2: f.param2, div: f.div, feel: f.feel });
    syncFxUi(i);
    if (d.fxFlash && mot) setTimeout(() => { d.fxFlash = 0; syncFxUi(i); }, 400);
  };
  d.sendFx = sendFx;
  D.fxkind.onchange = () => { d.fx.kind = D.fxkind.value; sendFx(); };
  bindFxBtn(D.fxon, () => d);
  // i 4 pad cambiano funzione con il menu del riquadro: hot cue, beatjump, loop roll, freno / reverse
  D.modebtn.onclick = (e) => {
    e.stopPropagation();
    const set = (m) => () => { d.mode = m; closeRolls(i); renderPads(i); };
    menu(e, [["Hot cue 1–4", set("hc0")], ["Hot cue 5–8", set("hc1")], "-", ["Beat jump", set("jump")], ["Loop roll", set("roll")], ["Freno · spinback · reverse", set("brake")]]);
  };
  const JUMPS = [0.25, 0.5, 1, 2, 4, 8, 16, 32], ROLLS = [1 / 16, 1 / 8, 1 / 4, 1 / 2, 1], BRAKES = [0.3, 0.5, 1, 1.5, 2, 3, 4];
  const step = (arr, cur, dir) => arr[Math.min(arr.length - 1, Math.max(0, arr.findIndex((x) => Math.abs(x - cur) < 1e-9) + dir))];
  D.pdn.onclick = () => { if (d.mode === "jump") d.jumpSize = step(JUMPS, d.jumpSize, -1); else if (d.mode === "roll") d.rollBase = step(ROLLS, d.rollBase, -1); else if (d.mode === "brake") d.brakeT = step(BRAKES, d.brakeT, -1); renderPads(i); };
  D.pup.onclick = () => { if (d.mode === "jump") d.jumpSize = step(JUMPS, d.jumpSize, 1); else if (d.mode === "roll") d.rollBase = step(ROLLS, d.rollBase, 1); else if (d.mode === "brake") d.brakeT = step(BRAKES, d.brakeT, 1); renderPads(i); };
  D.qbtn.onclick = () => dc(i, "quantize", d.snap?.quantize ? 0 : 1);
  D.qpill.onclick = () => dc(i, "quantize", d.snap?.quantize ? 0 : 1);
  D.slipbtn.onclick = () => dc(i, "slip", d.snap?.slip ? 0 : 1);
  const frac = (x) => (x >= 1 ? String(x) : "1/" + Math.round(1 / x));
  d.frac = frac;
  d.padAction = (k, down, e) => {
    const m = d.mode;
    if (m === "hc0" || m === "hc1") {
      const slot = k + (m === "hc1" ? 4 : 0);
      if (!down) return;
      if (e?.shiftKey) dc(i, "hotcue_clear", slot); else dc(i, "hotcue", slot);
    } else if (m === "jump") {
      if (down) dc(i, "beatjump", [-2, -1, 1, 2][k] * d.jumpSize);
    } else if (m === "roll") {
      const size = d.rollBase * [1, 2, 4, 8][k];
      if (down) dc(i, "roll_on", size); else dc(i, "roll_off");
    } else if (m === "brake") {
      if (k === 0) { if (down) dc(i, "brake_on", d.brakeT); else dc(i, "motion_off"); }
      else if (k === 1) { if (down) dc(i, "spin_on", d.brakeT); }
      else if (k === 2) { if (down) dc(i, "reverse_on"); else dc(i, "motion_off"); }
      else if (down) dc(i, "slip", d.snap?.slip ? 0 : 1);
    }
  };
  D.hot.forEach((b, k) => {
    let held = false;
    b.addEventListener("pointerdown", (e) => { if (e.button !== 0) return; held = true; b.setPointerCapture(e.pointerId); d.padAction(k, true, e); });
    const up = () => { if (held) { held = false; d.padAction(k, false); } };
    ["pointerup", "pointercancel"].forEach((ev) => b.addEventListener(ev, up));
    b.addEventListener("contextmenu", (e) => { e.preventDefault(); if (d.mode === "hc0" || d.mode === "hc1") dc(i, "hotcue_clear", k + (d.mode === "hc1" ? 4 : 0)); });
  });
  renderPads(i);
  // loop
  D.lsz.onclick = () => dc(i, "loop_toggle");
  $$("[data-loop]", el).forEach((b) => (b.onclick = () => dc(i, "loop_" + b.dataset.loop)));
  // griglia e tonalita': finestrelle
  D.grid.onclick = (e) => { e.stopPropagation(); openPop(i, "grid", D.grid); };
  D.r("keybtn").onclick = (e) => { e.stopPropagation(); openPop(i, "key", D.r("keybtn")); };

  // scrubbing: tieni premuto il mouse sulla forma d'onda o sul piatto e trascina. Il deck viene "tenuto" (muto e fermo)
  // finche' non rilasci, poi riprende da dove l'hai portato.
  const scrubber = (cv, begin, move) => {
    let st = null, raf = 0, pending = null;
    const flush = () => { raf = 0; if (pending != null) { dc(i, "scrub", pending); pending = null; } };
    const go = (p) => { d.scrub = p; pending = p; if (!raf) raf = requestAnimationFrame(flush); };
    cv.addEventListener("pointerdown", (e) => {
      if (e.button !== 0 || !d.dur) return;
      const s0 = begin(e); if (!s0) return;
      cv.setPointerCapture(e.pointerId); st = s0;
      dc(i, "hold", 1); go(move(e, st));
    });
    cv.addEventListener("pointermove", (e) => { if (st) go(move(e, st)); });
    const end = () => {
      if (!st) return; st = null;
      if (pending != null) { dc(i, "seek", pending); pending = null; }
      dc(i, "hold", 0);
      setTimeout(() => (d.scrub = null), 120);
    };
    cv.addEventListener("pointerup", end); cv.addEventListener("pointercancel", end);
  };
  // forma d'onda: trascinando verso sinistra si avanza, come girare un piatto
  scrubber(D.zoomCv, (e) => ({ x: e.clientX, pos: currentPos(i) }), (e, s) => {
    const secsPerPx = d.zoom / D.zoomCv.getBoundingClientRect().width;
    return Math.min(d.dur, Math.max(0, s.pos - (e.clientX - s.x) * secsPerPx));
  });
  // panoramica: la posizione segue il puntatore
  scrubber(D.over, () => ({}), (e) => {
    const rc = D.over.getBoundingClientRect();
    return Math.min(d.dur, Math.max(0, ((e.clientX - rc.left) / rc.width) * d.dur));
  });
  // piatto: la posizione segue l'angolo del puntatore (un giro = 1,8 s, come un 33 giri), anche all'indietro: scratch
  scrubber(D.plat, (e) => {
    const rc = D.plat.getBoundingClientRect(), cx = rc.left + rc.width / 2, cy = rc.top + rc.height / 2;
    const R = Math.min(rc.width, rc.height) * 0.47, dx = e.clientX - cx, dy = e.clientY - cy;
    if (Math.hypot(dx, dy) > R) return null;
    return { cx, cy, last: Math.atan2(dy, dx), pos: currentPos(i) };
  }, (e, s) => {
    const a = Math.atan2(e.clientY - s.cy, e.clientX - s.cx);
    let da = a - s.last; if (da > Math.PI) da -= 2 * Math.PI; if (da < -Math.PI) da += 2 * Math.PI;
    s.last = a;
    s.pos = Math.min(d.dur, Math.max(0, s.pos + (da / (2 * Math.PI)) * REV_SEC));
    return s.pos;
  });
  D.zoomCv.addEventListener("wheel", (e) => { e.preventDefault(); d.zoom = Math.min(40, Math.max(1.5, d.zoom * (e.deltaY > 0 ? 1.15 : 1 / 1.15))); }, { passive: false });
  D.zoomCv.addEventListener("dblclick", () => (d.zoom = 7));
}

/* finestrelle del deck: griglia/quantize e tonalita' */
function openPop(i, kind, anchor) {
  closePops();
  const el = DK[i].el, d = decks[i];
  const pop = document.createElement("div");
  pop.className = "keypop"; pop.dataset.kind = kind;
  if (kind === "grid") {
    pop.innerHTML = `<small>GRIGLIA (salvata per questo brano)</small>
      <div class="row"><button data-grid="shift" data-a="-5" title="Sposta la griglia indietro di 5 ms (Maiusc = 1 ms)">&#9664;</button><button data-grid="shift" data-a="5" title="Sposta la griglia avanti di 5 ms (Maiusc = 1 ms)">&#9654;</button>
      <button data-grid="halve" title="Dimezza il BPM (un marker ogni due)">&frac12;</button><button data-grid="double" title="Raddoppia il BPM (aggiunge i marker intermedi)">&times;2</button>
      <button data-grid="downbeat_here" title="Il battito sotto la testina diventa il &laquo;1&raquo; della battuta">1&#9656;</button>
      <button data-grid="anchor_here" title="Porta il marker più vicino esattamente sulla testina">&#9875;</button>
      <button data-grid="bar_shift" data-a="1" title="Sposta il &laquo;1&raquo; di un battito">+1</button>
      <button data-grid="reset" title="Ripristina la griglia calcolata automaticamente">&#8634;</button></div>
      <div class="row"><button data-r="gridfixed" title="FLUIDA: segue le variazioni di tempo del brano. FISSA: BPM costante">FLUIDA</button>
      <button data-r="quant" title="Quantizza cue e loop sui battiti">QUANTIZE</button></div>`;
    const gridCmd = async (cmd, a) => { const g = await inv("grid_cmd", { deck: i, cmd, a }); if (!g) return; d.beats = g.beats; d.downbeat = g.downbeat; };
    $$("[data-grid]", pop).forEach((b) => (b.onclick = (e) => gridCmd(b.dataset.grid, (+b.dataset.a || 0) * (e.shiftKey ? 0.2 : 1))));
    $('[data-r="gridfixed"]', pop).onclick = () => gridCmd("fixed", d.snap?.grid_fixed ? 0 : 1);
    $('[data-r="quant"]', pop).onclick = () => dc(i, "quantize", d.snap?.quantize ? 0 : 1);
  } else {
    pop.innerHTML = `<small>TONALITÀ</small>
      <div class="row"><button data-r="keylock">KEY LOCK</button><button data-r="keyreset" title="Ripristina la tonalità originale">&#8634;</button></div>
      <div class="row"><button data-wheel="-1" title="Camelot −1">&#9664;</button><span class="k" data-r="keynow">--</span><button data-wheel="1" title="Camelot +1">&#9654;</button></div>
      <div class="row"><button data-semi="-1">−½</button><span class="k" data-r="shift" style="color:var(--dim)">0 st</span><button data-semi="1">+½</button></div>`;
    $('[data-r="keylock"]', pop).onclick = () => dc(i, "keylock", d.snap?.keylock ? 0 : 1);
    $('[data-r="keyreset"]', pop).onclick = () => dc(i, "key_reset");
    $$("[data-wheel]", pop).forEach((b) => (b.onclick = () => dc(i, "key_wheel", +b.dataset.wheel)));
    $$("[data-semi]", pop).forEach((b) => (b.onclick = () => dc(i, "key_semitone", +b.dataset.semi)));
  }
  el.appendChild(pop);
  const a = anchor.getBoundingClientRect(), e = el.getBoundingClientRect();
  const x = Math.min(Math.max(4, (a.left - e.left) / uiK), e.width / uiK - pop.offsetWidth - 4), y = Math.max(4, (a.top - e.top) / uiK - pop.offsetHeight - 6);
  pop.style.left = x + "px"; pop.style.top = y + "px";
  d.pop = pop;
}
function closePops() { $$(".keypop").forEach((p) => p.remove()); }
document.addEventListener("pointerdown", (e) => { if (!e.target.closest(".keypop")) closePops(); }, true);

// posizione corrente interpolata fra due poll (secondi)
function currentPos(i) {
  const d = decks[i], s = d.snap; if (!s) return 0;
  if (d.scrub != null) return d.scrub;
  let pos = s.pos;
  if (s.playing && !s.held) pos += ((performance.now() - d.t) / 1000) * (1 + s.tempo_pct / 100);
  return pos;
}

/* copertine: incorporate nei file, ridotte e tenute in memoria */
const artCache = new Map();
function getArt(path, size) {
  const k = size + "|" + path;
  if (!artCache.has(k)) artCache.set(k, inv("track_art", { path, size }).catch(() => null));
  return artCache.get(k);
}
async function setDeckArt(i, path) {
  const d = decks[i], D = DK[i];
  d.art = null; D.art.style.backgroundImage = ""; D.art.textContent = i + 1;
  const url = await getArt(path, 256);
  if (!url || decks[i].artPath !== path) return;
  const img = new Image(); img.src = url;
  await img.decode().catch(() => {});
  d.art = img; D.art.style.backgroundImage = `url(${url})`; D.art.textContent = "";
}

/* svuota un deck (eject) */
async function ejectDeck(i) {
  const d = decks[i];
  await dc(i, "unload");
  d.wave = null; d.beats = []; d.downbeat = 0; d.dur = 0; d.over = null; d.scrub = null; d.art = null; d.meta = null; d.title_ = ""; d.loading = null; d.stText = "";
  DK[i].art.style.backgroundImage = ""; DK[i].art.textContent = i + 1;
}

/* sfoglia la libreria con i tasti (e, piu' avanti, con i controller): seleziona il brano precedente o successivo */
function libMove(dir) {
  if (!lib.rows.length) return;
  const ids = lib.rows.map((t) => t.id);
  let k = lib.last != null ? ids.indexOf(lib.last) : -1;
  k = Math.min(ids.length - 1, Math.max(0, k < 0 ? 0 : k + dir));
  lib.sel = new Set([ids[k]]); lib.last = ids[k];
  $$("#tracks tbody tr").forEach((r) => r.classList.toggle("sel", lib.sel.has(+r.dataset.id)));
  $(`#tracks tbody tr[data-id="${ids[k]}"]`)?.scrollIntoView({ block: "nearest" });
  showInspector();
}

async function loadTrack(i, path) {
  const d = decks[i], D = DK[i];
  // il deck si svuota subito: niente brano vecchio (ne' audio) mentre il nuovo si carica
  await dc(i, "unload");
  d.wave = null; d.beats = []; d.downbeat = 0; d.dur = 0; d.over = null; d.scrub = null; d.art = null; d.stTouch = 0; d.stGains = [1, 1, 1, 1]; d.stMem = [1, 1, 1, 1];
  d.meta = lib.rows.find((t) => t.path === path) || null;
  d.artPath = path;
  D.title.textContent = "Caricamento…"; D.artist.textContent = "";
  D.art.style.backgroundImage = ""; D.art.textContent = i + 1;
  d.loading = path;
  try {
    const t0 = performance.now();
    const info = await inv("load_track", { deck: i, path });
    d.wave = info.wave; d.beats = info.beats; d.downbeat = info.downbeat; d.dur = info.duration; d.over = null; d.scrub = null;
    D.tempo._set(0.5);
    D.title.title = `Caricato in ${info.ms} ms (decodifica ${info.decode_ms} ms, analisi ${info.cached ? "dal database" : "calcolata ora"}, interfaccia ${Math.round(performance.now() - t0 - info.ms)} ms)`;
    if (info.ms + (performance.now() - t0 - info.ms) > 1500) toast(`Caricamento lento: ${D.title.title}`);
    d.title_ = d.meta?.title || info.name;
    D.title.textContent = d.title_; D.artist.textContent = d.meta?.artist || "";
    setDeckArt(i, path);
  } catch (e) {
    D.title.textContent = "Errore: " + e;
  }
  d.loading = null;
}

async function pickFile(opts = {}) {
  const r = await T.dialog.open({
    multiple: false,
    filters: [{ name: "Audio", extensions: ["mp3", "wav", "flac", "ogg", "opus", "m4a", "aac", "aif", "aiff", "wma", "alac", "ape", "wv", "mp4", "mka", "webm", "caf"] }, { name: "Tutti i file", extensions: ["*"] }],
    ...opts,
  });
  return r;
}

/* ---- canvas */
function resizeCanvases() {
  // con #app scalato da --k i canvas appaiono uiK volte piu' grandi: la risoluzione interna segue, cosi' restano nitidi
  const dpr = (window.devicePixelRatio || 1) * uiK;
  $$("canvas").forEach((c) => { const w = Math.round(c.clientWidth * dpr), h = Math.round(c.clientHeight * dpr); if (w && (c.width !== w || c.height !== h)) { c.width = w; c.height = h; } });
  decks.forEach((d) => (d.over = null));
}
window.addEventListener("resize", resizeCanvases);

// una colonna a 3 bande: bassi sotto, medi sopra, alti in cima (colori del deck)
function drawCol(g, x, mid, hl, hm, hh, dim, pal) {
  g.globalAlpha = dim;
  g.fillStyle = pal.lo; g.fillRect(x, mid - hl, 1, hl * 2);
  g.fillStyle = pal.mid; g.fillRect(x, mid - hm * 0.62, 1, hm * 1.24);
  g.fillStyle = pal.hi; g.fillRect(x, mid - hh * 0.36, 1, hh * 0.72);
}
function bandMax(arr, k0, k1) { let m = 0; for (let q = Math.max(0, k0); q <= k1 && q < arr.length; q++) if (arr[q] > m) m = arr[q]; return m; }

function drawZoom(i, pos) {
  const c = DK[i].zoomCv; if (!c.width) return;
  const g = c.getContext("2d"), W = c.width, H = c.height, d = decks[i], pal = PAL[i];
  g.globalAlpha = 1; g.clearRect(0, 0, W, H);
  if (!d.wave) return;
  const win = d.zoom, t0 = pos - win / 2, pps = W / win, mid = H / 2, rate = d.wave.rate, amp = mid * 0.96 / 255;
  for (let x = 0; x < W; x++) {
    const t = t0 + x / pps, k0 = Math.floor(t * rate), k1 = Math.max(k0, Math.floor((t + 1 / pps) * rate));
    if (k1 < 0) continue;
    const hl = bandMax(d.wave.lo, k0, k1), hm = bandMax(d.wave.mid, k0, k1), hh = bandMax(d.wave.hi, k0, k1);
    if (hl + hm + hh === 0) continue;
    drawCol(g, x, mid, hl * amp, hm * amp, hh * amp, t < pos ? 0.55 : 1, pal);
  }
  g.globalAlpha = 1;
  // griglia: ogni linea e' un battito rilevato. Tempo forte (inizio battuta) piu' marcato.
  let lo = 0, hi = d.beats.length;
  while (lo < hi) { const m = (lo + hi) >> 1; if (d.beats[m] < t0) lo = m + 1; else hi = m; }
  g.font = `${Math.round(H * 0.16)}px sans-serif`;
  for (let b = lo; b < d.beats.length && d.beats[b] <= t0 + win; b++) {
    const x = Math.round((d.beats[b] - t0) * pps), bar = (((b - d.downbeat) % 4) + 4) % 4 === 0;
    if (bar) { g.fillStyle = "rgba(255,255,255,.9)"; g.fillRect(x, 0, 2, H); }
    else { g.fillStyle = "rgba(255,255,255,.22)"; g.fillRect(x, 0, 1, H); g.fillStyle = "rgba(255,255,255,.7)"; g.fillRect(x, H * 0.76, 1, H * 0.24); }
    if (bar && win < 20) { g.fillStyle = "#fff"; g.fillText(String(Math.floor((b - d.downbeat) / 4) + 1), x + 4, H * 0.17); }
  }
  const s = d.snap;
  const mark = (t, color) => { const x = (t - t0) * pps; if (x >= 0 && x < W) { g.fillStyle = color; g.fillRect(x, 0, 2, H); g.beginPath(); g.moveTo(x - 6, 0); g.lineTo(x + 8, 0); g.lineTo(x + 1, 10); g.fill(); } };
  if (s) {
    mark(s.cue, "#ffb020"); s.hot_cues.forEach((t, k) => t != null && mark(t, HC[k % 4]));
    if (s.loop_on || s.loop_out > s.loop_in) {
      const x0 = (s.loop_in - t0) * pps, x1 = (s.loop_out - t0) * pps;
      g.fillStyle = s.loop_on ? "rgba(53,224,138,.22)" : "rgba(255,255,255,.07)"; g.fillRect(x0, 0, x1 - x0, H);
      g.fillStyle = s.loop_on ? "#35e08a" : "#8a96b4"; g.fillRect(x0, 0, 2, H); g.fillRect(x1 - 2, 0, 2, H);
    }
  }
  g.fillStyle = "#fff"; g.shadowColor = "#fff"; g.shadowBlur = 8; g.fillRect(Math.round(W / 2) - 1, 0, 2, H); g.shadowBlur = 0;
}

function buildOver(i, W, H) {
  const d = decks[i], o = document.createElement("canvas"); o.width = W; o.height = H; const og = o.getContext("2d");
  if (!d.wave) return o;
  const n = d.wave.lo.length, mid = H / 2, amp = mid * 0.95 / 255;
  for (let x = 0; x < W; x++) {
    const k0 = Math.floor((x / W) * n), k1 = Math.max(k0, Math.floor(((x + 1) / W) * n) - 1);
    drawCol(og, x, mid, bandMax(d.wave.lo, k0, k1) * amp, bandMax(d.wave.mid, k0, k1) * amp, bandMax(d.wave.hi, k0, k1) * amp, 1, PAL[i]);
  }
  return o;
}

function drawOver(i, pos) {
  const c = DK[i].over; if (!c.width || c.offsetParent === null) return;
  const d = decks[i], W = c.width, H = c.height, g = c.getContext("2d");
  if (!d.over || d.over.width !== W || d.over.height !== H) d.over = buildOver(i, W, H);
  g.globalAlpha = 1; g.clearRect(0, 0, W, H); g.globalAlpha = 0.8; g.drawImage(d.over, 0, 0); g.globalAlpha = 1;
  if (!d.dur) return;
  const px = (pos / d.dur) * W;
  g.fillStyle = "rgba(0,0,0,.45)"; g.fillRect(0, 0, px, H);
  d.snap?.hot_cues.forEach((t, k) => { if (t != null) { g.fillStyle = HC[k % 4]; g.fillRect((t / d.dur) * W, 0, 2, H); } });
  // finestra della forma d'onda ingrandita
  const wx = ((pos - d.zoom / 2) / d.dur) * W, ww = (d.zoom / d.dur) * W;
  g.strokeStyle = "rgba(255,255,255,.9)"; g.lineWidth = Math.max(1, W / 700); g.strokeRect(wx, 1, Math.max(4, ww), H - 2);
  g.fillStyle = "#fff"; g.fillRect(px - 1, 0, 2, H);
}

/* piatto con vinile: gira con la posizione del brano (quindi segue pitch e scratch). Il disco e' disegnato una volta sola
   (solchi, spazi fra i brani, micrograffi, polvere) e poi ruotato; i riflessi di luce restano fermi, come nella realta'.
   Braccio in stile Technics SL-1200: perno cardanico, tubo a S, testina con cartuccia, contrappeso. */
function vinylTexture(rv, dpr) {
  const S = Math.ceil(rv * 2) + 2, c = document.createElement("canvas");
  c.width = c.height = S;
  const g = c.getContext("2d"), cx = S / 2, cy = S / 2, TAU = Math.PI * 2;
  let seed = 987654321; const rnd = () => ((seed = (seed * 1664525 + 1013904223) >>> 0) / 4294967296);
  const gr = g.createRadialGradient(cx, cy, rv * 0.3, cx, cy, rv);
  gr.addColorStop(0, "#171c26"); gr.addColorStop(0.55, "#0d1118"); gr.addColorStop(1, "#07090d");
  g.fillStyle = gr; g.beginPath(); g.arc(cx, cy, rv, 0, TAU); g.fill();
  // solchi fitti con intensita' irregolare; fra un brano e l'altro la superficie e' liscia
  const gaps = [[0.50, 0.515], [0.64, 0.655], [0.78, 0.795], [0.88, 0.895]];
  g.lineWidth = Math.max(0.6, dpr * 0.6);
  for (let r = rv * 0.405; r < rv * 0.955; r += 1.45 * dpr) {
    const f = r / rv; if (gaps.some(([a, b]) => f > a && f < b)) continue;
    g.strokeStyle = `rgba(255,255,255,${(0.015 + rnd() * 0.055).toFixed(3)})`;
    g.beginPath(); g.arc(cx, cy, r, 0, TAU); g.stroke();
  }
  g.lineWidth = 1.3 * dpr;
  for (const [a, b] of gaps) { g.strokeStyle = "rgba(255,255,255,.075)"; g.beginPath(); g.arc(cx, cy, rv * (a + b) / 2, 0, TAU); g.stroke(); }
  // anello di uscita vicino all'etichetta e bordo con bisello
  g.strokeStyle = "rgba(255,255,255,.06)"; g.lineWidth = 2.5 * dpr; g.beginPath(); g.arc(cx, cy, rv * 0.385, 0, TAU); g.stroke();
  g.strokeStyle = "rgba(255,255,255,.12)"; g.lineWidth = 1.6 * dpr; g.beginPath(); g.arc(cx, cy, rv - 0.9 * dpr, 0, TAU); g.stroke();
  g.strokeStyle = "rgba(0,0,0,.5)"; g.lineWidth = 1.2 * dpr; g.beginPath(); g.arc(cx, cy, rv - 3 * dpr, 0, TAU); g.stroke();
  // micrograffi e polvere: sono quello che fa vedere il disco girare
  for (let k = 0; k < 50; k++) {
    const r = rv * (0.42 + rnd() * 0.52), a0 = rnd() * TAU, len = 0.03 + rnd() * 0.3;
    g.strokeStyle = `rgba(255,255,255,${(0.04 + rnd() * 0.1).toFixed(3)})`; g.lineWidth = dpr * (0.5 + rnd() * 0.6);
    g.beginPath(); g.arc(cx, cy, r, a0, a0 + len); g.stroke();
  }
  for (let k = 0; k < 34; k++) {
    const r = rv * (0.4 + rnd() * 0.56), a = rnd() * TAU;
    g.fillStyle = `rgba(225,230,240,${(0.15 + rnd() * 0.3).toFixed(2)})`;
    g.beginPath(); g.arc(cx + Math.cos(a) * r, cy + Math.sin(a) * r, dpr * (0.5 + rnd() * 0.9), 0, TAU); g.fill();
  }
  return c;
}

function drawTonearm(g, dpr, rv, cx, cy, tx, ty) {
  const TAU = Math.PI * 2, u = rv / 200; // unita': 1/200 del raggio del disco
  const px = cx + rv * 1.03, py = cy - rv * 0.9;
  // direzione perno -> puntina e testina con il suo angolo di offset
  const dPT = Math.hypot(tx - px, ty - py) || 1, ux = (tx - px) / dPT, uy = (ty - py) / dPT;
  const hs = 40 * u, off = 0.32, co = Math.cos(off), so = Math.sin(off);
  const hdx = ux * co - uy * so, hdy = ux * so + uy * co; // asse della testina
  const hx = tx - hdx * hs, hy = ty - hdy * hs;           // attacco testina / tubo
  const dx = hx - px, dy = hy - py, L = Math.hypot(dx, dy) || 1, nx = -dy / L, ny = dx / L;
  const c1 = [px + dx * 0.3 + nx * L * 0.09, py + dy * 0.3 + ny * L * 0.09], c2 = [px + dx * 0.72 - nx * L * 0.07, py + dy * 0.72 - ny * L * 0.07];
  const tube = () => { g.beginPath(); g.moveTo(px, py); g.bezierCurveTo(c1[0], c1[1], c2[0], c2[1], hx, hy); };
  const wt = Math.max(3 * dpr, 6.5 * u);
  g.save(); g.lineCap = "round"; g.lineJoin = "round";
  // ombra sul disco
  g.save(); g.translate(7 * dpr, 10 * dpr); g.strokeStyle = "rgba(0,0,0,.38)"; g.lineWidth = wt + 2 * dpr; tube(); g.stroke();
  g.lineWidth = 15 * u; g.beginPath(); g.moveTo(hx, hy); g.lineTo(tx, ty); g.stroke(); g.restore();
  // contrappeso: asta posteriore e cilindro con le tacche
  const bx = -dx / L, by = -dy / L, ba = Math.atan2(by, bx);
  g.strokeStyle = "#3c4351"; g.lineWidth = wt * 0.7; g.beginPath(); g.moveTo(px, py); g.lineTo(px + bx * 30 * u, py + by * 30 * u); g.stroke();
  g.save(); g.translate(px + bx * 47 * u, py + by * 47 * u); g.rotate(ba);
  const cw = 34 * u, ch = 27 * u;
  const cg = g.createLinearGradient(0, -ch / 2, 0, ch / 2); cg.addColorStop(0, "#2a303b"); cg.addColorStop(0.35, "#8993a7"); cg.addColorStop(0.6, "#4a5263"); cg.addColorStop(1, "#1f242d");
  g.fillStyle = cg; g.beginPath(); g.roundRect(-cw / 2, -ch / 2, cw, ch, 5 * u); g.fill();
  g.strokeStyle = "rgba(0,0,0,.55)"; g.lineWidth = Math.max(1, dpr);
  for (let k = -2; k <= 2; k++) { g.beginPath(); g.moveTo(k * cw * 0.17, -ch / 2); g.lineTo(k * cw * 0.17, ch / 2); g.stroke(); }
  g.restore();
  // tubo a S: contorno, alluminio, riflesso
  g.strokeStyle = "#232833"; g.lineWidth = wt + 2.4 * dpr; tube(); g.stroke();
  g.strokeStyle = "#c3cad8"; g.lineWidth = wt; tube(); g.stroke();
  g.strokeStyle = "rgba(255,255,255,.75)"; g.lineWidth = Math.max(1, wt * 0.28); g.save(); g.translate(-wt * 0.2, -wt * 0.22); tube(); g.stroke(); g.restore();
  g.strokeStyle = "rgba(60,68,85,.55)"; g.lineWidth = Math.max(1, wt * 0.22); g.save(); g.translate(wt * 0.28, wt * 0.3); tube(); g.stroke(); g.restore();
  // attacco con ghiera zigrinata
  g.save(); g.translate(hx, hy); g.rotate(Math.atan2(hdy, hdx));
  g.fillStyle = "#2c313c"; g.beginPath(); g.roundRect(-4 * u, -wt * 0.95, 11 * u, wt * 1.9, 2 * u); g.fill();
  g.strokeStyle = "rgba(255,255,255,.18)"; g.lineWidth = Math.max(1, dpr * 0.8);
  for (let k = 0; k < 3; k++) { g.beginPath(); g.moveTo(-1 * u + k * 3 * u, -wt * 0.9); g.lineTo(-1 * u + k * 3 * u, wt * 0.9); g.stroke(); }
  // testina nera con la linguetta (finger lift) e la cartuccia
  const hw = 15 * u;
  const hg = g.createLinearGradient(0, -hw / 2, 0, hw / 2); hg.addColorStop(0, "#3b414d"); hg.addColorStop(0.5, "#1a1d24"); hg.addColorStop(1, "#0d0f14");
  g.fillStyle = hg; g.beginPath(); g.roundRect(6 * u, -hw / 2, hs - 6 * u, hw, 3 * u); g.fill();
  g.strokeStyle = "rgba(200,210,230,.38)"; g.lineWidth = Math.max(1, dpr); g.stroke();
  g.strokeStyle = "#8d96a8"; g.lineWidth = Math.max(1.4, 2.2 * u); g.beginPath(); g.moveTo(9 * u, -hw / 2); g.quadraticCurveTo(2 * u, -hw * 1.15, -3 * u, -hw * 1.45); g.stroke();
  g.fillStyle = "#e9e6df"; g.beginPath(); g.roundRect(hs * 0.42, -hw * 0.42, hs * 0.5, hw * 0.84, 2 * u); g.fill();
  g.fillStyle = "#d8283a"; g.fillRect(hs * 0.42, -hw * 0.42, hs * 0.07, hw * 0.84);
  g.fillStyle = "#1b1e26"; g.beginPath(); g.moveTo(hs * 0.9, -hw * 0.22); g.lineTo(hs + 2 * u, 0); g.lineTo(hs * 0.9, hw * 0.22); g.fill();
  g.fillStyle = "#e9eef9"; g.beginPath(); g.arc(hs + 1.5 * u, 0, Math.max(1, 1.4 * u), 0, TAU); g.fill();
  g.restore();
  // perno cardanico: base scura, anello e cupola metallica
  const rb = 24 * u;
  const bg = g.createRadialGradient(px - rb * 0.3, py - rb * 0.3, 1, px, py, rb); bg.addColorStop(0, "#4a5262"); bg.addColorStop(1, "#171b23");
  g.fillStyle = bg; g.beginPath(); g.arc(px, py, rb, 0, TAU); g.fill();
  g.strokeStyle = "#0b0d12"; g.lineWidth = 2 * dpr; g.beginPath(); g.arc(px, py, rb, 0, TAU); g.stroke();
  const ig = g.createRadialGradient(px - 5 * u, py - 5 * u, 1, px, py, 15 * u); ig.addColorStop(0, "#f3f6fb"); ig.addColorStop(0.55, "#9aa4b8"); ig.addColorStop(1, "#4d5668");
  g.fillStyle = ig; g.beginPath(); g.arc(px, py, 15 * u, 0, TAU); g.fill();
  g.strokeStyle = "rgba(0,0,0,.45)"; g.lineWidth = dpr; g.beginPath(); g.arc(px, py, 15 * u, 0, TAU); g.stroke();
  g.fillStyle = "#2a303c"; g.beginPath(); g.arc(px, py, 4.5 * u, 0, TAU); g.fill();
  g.restore();
}

function drawPlatter(i, pos) {
  const cv = DK[i].plat; if (!cv.width || cv.offsetParent === null) return;
  const g = cv.getContext("2d"), W = cv.width, H = cv.height, d = decks[i], s = d.snap, pal = PAL[i];
  const dpr = W / Math.max(1, cv.clientWidth);
  g.clearRect(0, 0, W, H);
  const R = Math.min(W, H) * 0.47, cx = W / 2, cy = H / 2, TAU = Math.PI * 2;
  const ang = (pos / REV_SEC) * TAU, loaded = !!s?.loaded && d.dur > 0;
  const rim = R - 4 * dpr, rv = R - 14 * dpr;
  // anello al neon: tenue, con la parte gia' suonata piu' accesa
  g.save();
  g.lineWidth = 5 * dpr; g.strokeStyle = pal.c; g.shadowColor = pal.c;
  g.globalAlpha = 0.3; g.shadowBlur = 16 * dpr; g.beginPath(); g.arc(cx, cy, R, 0, TAU); g.stroke();
  if (loaded) { g.globalAlpha = 1; g.shadowBlur = 24 * dpr; g.beginPath(); g.arc(cx, cy, R, -Math.PI / 2, -Math.PI / 2 + TAU * Math.min(1, pos / d.dur)); g.stroke(); }
  g.restore();
  // bordo del piatto in alluminio con i puntini dello stroboscopio (girano col piatto)
  g.save(); g.translate(cx, cy);
  const rg = g.createRadialGradient(0, 0, rv, 0, 0, rim);
  rg.addColorStop(0, "#69728a"); rg.addColorStop(0.45, "#c7cdda"); rg.addColorStop(1, "#7b8499");
  g.fillStyle = rg; g.beginPath(); g.arc(0, 0, rim, 0, TAU); g.arc(0, 0, rv, 0, TAU, true); g.fill();
  g.rotate(ang);
  const mid = (rim + rv) / 2, nd = 90;
  g.fillStyle = "rgba(25,29,38,.85)";
  for (let k = 0; k < nd; k += 2) { g.save(); g.rotate((k / nd) * TAU); g.fillRect(-0.8 * dpr, -mid - 1.7 * dpr, 1.6 * dpr, 3.4 * dpr); g.restore(); }
  g.restore();
  // disco (texture ruotata) e luci fisse
  if (!d.tex || d.tex.key !== Math.round(rv)) d.tex = { key: Math.round(rv), cv: vinylTexture(rv, dpr) };
  g.save(); g.translate(cx, cy); g.rotate(ang); g.drawImage(d.tex.cv, -d.tex.cv.width / 2, -d.tex.cv.height / 2); g.restore();
  const rl = rv * 0.34;
  if (g.createConicGradient) {
    const cg = g.createConicGradient(-0.9, cx, cy);
    [[0, 0], [0.035, 0], [0.085, 0.2], [0.135, 0], [0.5, 0], [0.535, 0], [0.585, 0.17], [0.635, 0], [1, 0]].forEach(([p, a]) => cg.addColorStop(p, `rgba(255,255,255,${a})`));
    g.save(); g.beginPath(); g.arc(cx, cy, rv * 0.985, 0, TAU); g.arc(cx, cy, rl, 0, TAU, true); g.clip();
    g.fillStyle = cg; g.fillRect(cx - rv, cy - rv, rv * 2, rv * 2); g.restore();
  }
  const sh = g.createRadialGradient(cx - rv * 0.35, cy - rv * 0.4, rv * 0.1, cx, cy, rv);
  sh.addColorStop(0, "rgba(255,255,255,.05)"); sh.addColorStop(1, "rgba(0,0,0,.28)");
  g.fillStyle = sh; g.beginPath(); g.arc(cx, cy, rv, 0, TAU); g.arc(cx, cy, rl, 0, TAU, true); g.fill();
  // etichetta con la copertina (ruota col disco), bordo di carta e foro con il perno cromato
  g.save(); g.translate(cx, cy); g.rotate(ang);
  g.beginPath(); g.arc(0, 0, rl, 0, TAU); g.clip();
  if (d.art && d.art.naturalWidth) g.drawImage(d.art, -rl, -rl, rl * 2, rl * 2);
  else {
    const lg = g.createRadialGradient(0, 0, 0, 0, 0, rl); lg.addColorStop(0, pal.c); lg.addColorStop(1, "#0a0f18");
    g.fillStyle = lg; g.fillRect(-rl, -rl, rl * 2, rl * 2);
    g.fillStyle = "rgba(255,255,255,.85)"; g.font = `600 ${rl * 0.7}px sans-serif`; g.textAlign = "center"; g.textBaseline = "middle"; g.fillText(String(i + 1), 0, 0);
  }
  const lv = g.createRadialGradient(0, 0, rl * 0.55, 0, 0, rl); lv.addColorStop(0, "rgba(0,0,0,0)"); lv.addColorStop(1, "rgba(0,0,0,.45)");
  g.fillStyle = lv; g.fillRect(-rl, -rl, rl * 2, rl * 2);
  g.fillStyle = "rgba(255,255,255,.9)"; g.beginPath(); g.arc(0, -rl * 0.84, 2.2 * dpr, 0, TAU); g.fill();
  g.restore();
  g.strokeStyle = "rgba(0,0,0,.6)"; g.lineWidth = 2 * dpr; g.beginPath(); g.arc(cx, cy, rl, 0, TAU); g.stroke();
  g.strokeStyle = pal.c; g.globalAlpha = 0.85; g.lineWidth = 1.6 * dpr; g.beginPath(); g.arc(cx, cy, rl + 1.6 * dpr, 0, TAU); g.stroke(); g.globalAlpha = 1;
  const sp = g.createRadialGradient(cx - 1.2 * dpr, cy - 1.2 * dpr, 0.5, cx, cy, 5 * dpr); sp.addColorStop(0, "#ffffff"); sp.addColorStop(0.6, "#aeb7c8"); sp.addColorStop(1, "#4d5668");
  g.fillStyle = sp; g.beginPath(); g.arc(cx, cy, 4.6 * dpr, 0, TAU); g.fill();
  // braccio: appoggia sul disco quando c'e' un brano e scende verso il centro mentre il brano avanza
  d.armT += ((loaded ? 1 : 0) - d.armT) * 0.1;
  const prog = loaded ? Math.min(1, pos / d.dur) : 0, phi = 0.8;
  d.armR = 0.92 - 0.5 * prog;
  const onx = cx + Math.cos(phi) * rv * d.armR, ony = cy + Math.sin(phi) * rv * d.armR;
  const parkx = cx + rv * 1.1, parky = cy - rv * 0.12;
  // appoggio del braccio (quando e' a riposo)
  g.fillStyle = "#1e232d"; g.beginPath(); g.roundRect(parkx - 8 * dpr, parky + 7 * dpr, 20 * dpr, 12 * dpr, 4 * dpr); g.fill();
  const tx = parkx + (onx - parkx) * d.armT, ty = parky + (ony - parky) * d.armT;
  drawTonearm(g, dpr, rv, cx, cy, tx, ty);
}

/* barra spaziatrice: play/stop del deck sotto il mouse, altrimenti dell'ultimo deck usato */
let focusDeck = 0;
document.addEventListener("pointerdown", (e) => { const dk = e.target.closest?.(".deck,.strip"); if (dk) { focusDeck = +dk.dataset.i; if (focusDeck < activeDecks) setTarget(focusDeck); } }, true);
document.addEventListener("keydown", (e) => {
  if (e.code !== "Space" || e.repeat || e.ctrlKey || e.altKey || e.metaKey) return;
  if (e.target.matches?.("input:not([type=range]),textarea,select") || $("#audioDlg").open) return;
  e.preventDefault();
  if (document.activeElement?.tagName === "BUTTON") document.activeElement.blur();
  const hov = document.querySelector(".deck:hover");
  const i = hov ? +hov.dataset.i : focusDeck;
  if (i < activeDecks && decks[i].snap?.loaded) dc(i, "play");
});

/* ---- aggiornamento stato */
async function poll() {
  try {
    const s = await inv("get_state");
    const now = performance.now();
    s.decks.slice(0, 2).forEach((ds, i) => { decks[i].snap = ds; decks[i].t = now; updateDeckUI(i, ds); });
    setMeter($("#masterMeter"), s.master_level);
    $("#autoGain").classList.toggle("on", !!s.auto_gain);
    $("#cpuBar").style.setProperty("--v", Math.min(100, (s.cpu ?? 0) * 100) + "%");
    updateStemsUI();
    updatePads(s.pads, s.pad_sync); updateDrums(s); updateRec(s);
  } catch (e) { /* motore non pronto */ }
  setTimeout(poll, 33);
}

function setMeter(el, lv) { [...el.children].forEach((c, k) => c.style.setProperty("--v", Math.min(100, (lv[k] ?? lv[0]) * 100) + "%")); }

const setText = (el, t) => { if (el.textContent !== t) el.textContent = t; };
const loopLabel = (b) => (b >= 1 ? String(Math.round(b * 100) / 100).replace(/\.0+$/, "") : "1/" + Math.round(1 / b));

function updateDeckUI(i, s) {
  const D = DK[i], d = decks[i];
  if (s.loaded && !d.loading) { setText(D.title, d.title_ || s.name); setText(D.artist, d.meta?.artist || ""); }
  else if (!s.loaded && !d.loading) { setText(D.title, "Trascina qui un brano o usa ♫"); setText(D.artist, ""); }
  setText(D.bpm, s.bpm_now ? s.bpm_now.toFixed(1) : "--");
  setText(D.key, s.key_now || "--");
  D.key.title = s.key_orig ? `Originale ${s.key_orig} (${s.key_orig_name}) → ora ${s.key_now} (${s.key_now_name})` : "";
  setText(D.remain, "-" + mmss(s.duration - s.pos));
  setText(D.time, `${mmss(s.pos)} / ${mmss(s.duration)}`);
  if (D._playing !== s.playing) { D._playing = s.playing; D.play.innerHTML = s.playing ? IC.pause : IC.play; }
  D.play.classList.toggle("on", s.playing);
  D.sync.classList.toggle("on", !!s.sync);
  D.pfl.classList.toggle("on", !!s.pfl); D.pfl2.classList.toggle("on", !!s.pfl);
  $(`#hpBtns button[data-pfl="${i}"]`).classList.toggle("on", !!s.pfl);
  setText(D.range, "±" + Math.round(s.range_pct));
  setText(D.tempoval, (s.tempo_pct >= 0 ? "+" : "") + s.tempo_pct.toFixed(1) + "%");
  D.tempo._set(0.5 - (s.tempo_pct / s.range_pct) / 2);
  setText(D.lsz, loopLabel(s.loop_beats)); D.lsz.classList.toggle("on", !!s.loop_on);
  D.grid.classList.toggle("on", s.grid_fixed || !s.quantize);
  const hcm = d.mode === "hc0" || d.mode === "hc1";
  D.hot.forEach((b, k) => {
    if (hcm) { const t = s.hot_cues[k + (d.mode === "hc1" ? 4 : 0)]; b.classList.toggle("set", t != null); b.classList.remove("act"); b.title = t != null ? `Hot cue ${k + 1 + (d.mode === "hc1" ? 4 : 0)} a ${fmtc(t)} · Maiusc+clic: cancella` : "Clic: imposta qui un hot cue"; }
    else { b.classList.remove("set"); b.classList.toggle("act", (d.mode === "roll" && s.rolling) || (d.mode === "brake" && ((k === 0 && s.motion === 1) || (k === 1 && s.motion === 2) || (k === 2 && s.motion === 3) || (k === 3 && s.slip)))); }
  });
  D.qbtn.classList.toggle("on", !!s.quantize); D.qpill.classList.toggle("on", !!s.quantize); D.slipbtn.classList.toggle("on", !!s.slip);
  D.play.classList.toggle("armed", !!s.armed);
  // meter stereo del canale: scala in dB (da -54 a 0) con indicatore del picco che resta un attimo
  [[D.meterL, s.level_l, 0], [D.meterR, s.level_r, 1]].forEach(([el, lv, k]) => {
    const v = Math.max(0, Math.min(1, (20 * Math.log10(Math.max(lv || 0, 1e-4)) + 54) / 54));
    const now = performance.now();
    if (v >= d.pk[k]) { d.pk[k] = v; d.pkT[k] = now + 900; } else if (now > d.pkT[k]) d.pk[k] = Math.max(v, d.pk[k] - 0.02);
    el.style.setProperty("--v", (v * 100).toFixed(1) + "%"); el.style.setProperty("--pk", (d.pk[k] * 100).toFixed(1) + "%");
  });
  D.beat.classList.toggle("on", s.playing && s.beat_phase < 0.18);
  // pop-up aperti: aggiorna lo stato
  if (d.pop && d.pop.isConnected) {
    const p = d.pop, q = (n) => $(`[data-r="${n}"]`, p);
    if (q("gridfixed")) { q("gridfixed").textContent = s.grid_fixed ? "FISSA" : "FLUIDA"; q("gridfixed").classList.toggle("on", s.grid_fixed); q("quant").classList.toggle("on", s.quantize); }
    if (q("keylock")) { q("keylock").classList.toggle("on", s.keylock); q("keynow").textContent = s.key_orig ? `${s.key_orig} → ${s.key_now}` : "--"; q("shift").textContent = (s.key_shift > 0 ? "+" : "") + s.key_shift + " st"; }
  }
  // stem (stato dell'ultimo brano caricato)
  if (performance.now() - d.stTouch > 700 && s.stem_gains) { d.stGains = [...s.stem_gains]; s.stem_gains.forEach((g, k) => { if (g > 0.001) d.stMem[k] = g; }); }
  setText(D.stm, d.stText);
}

function frame() {
  for (let i = 0; i < activeDecks; i++) {
    const s = decks[i].snap;
    const pos = currentPos(i) - (decks[i].scrub == null && s?.playing ? audioLatency * (1 + s.tempo_pct / 100) : 0);
    drawZoom(i, pos); drawOver(i, pos); drawPlatter(i, pos);
  }
  requestAnimationFrame(frame);
}

/* ==================================================== BARRA SUPERIORE E MIXER */
{
  let on = true; try { on = localStorage.getItem("spinlab.auto_stem") !== "0"; } catch (e) {}
  const b = $("#autoStem"); b.classList.toggle("on", on); inv("stems_auto", { on });
  b.onclick = () => { on = !on; b.classList.toggle("on", on); inv("stems_auto", { on }); try { localStorage.setItem("spinlab.auto_stem", on ? "1" : "0"); } catch (e) {} };
}
$("#autoGain").onclick = () => inv("mixer_cmd", { cmd: "auto_gain", a: $("#autoGain").classList.contains("on") ? 0 : 1 });
// cuffie: pulsanti PFL per deck, CUE MIX fra i due lati e volume (ricordati fra una sessione e l'altra)
$$("#hpBtns button").forEach((b) => (b.onclick = () => { const i = +b.dataset.pfl; dc(i, "pfl", decks[i].snap?.pfl ? 0 : 1); }));
const knobInit = (id, cmd, def) => {
  let v = def; try { v = +(localStorage.getItem("spinlab." + cmd) ?? def); } catch (e) {}
  makeKnob($(id), { value: v, def, onChange: (x) => { inv("mixer_cmd", { cmd, a: x }); try { localStorage.setItem("spinlab." + cmd, x); } catch (e) {} } });
  inv("mixer_cmd", { cmd, a: v });
};
knobInit("#cueMix", "cue_mix", 0.5); knobInit("#hpVol", "hp_vol", 0.8); knobInit("#masterKnob", "master", 0.8);
$("#samplerVol").oninput = (e) => inv("mixer_cmd", { cmd: "sampler_vol", a: +e.target.value });
setInterval(() => { const t = new Date(); $("#clock").textContent = `${String(t.getHours()).padStart(2, "0")}:${String(t.getMinutes()).padStart(2, "0")}`; }, 1000);
$("#clock").textContent = (() => { const t = new Date(); return `${String(t.getHours()).padStart(2, "0")}:${String(t.getMinutes()).padStart(2, "0")}`; })();

/* navigazione */
$$("#nav button").forEach((b) => (b.onclick = () => {
  const n = b.dataset.nav;
  if (n === "lib" || n === "mixer") {
    document.body.classList.toggle("view-lib", n === "lib");
    $$("#nav button").forEach((x) => x.classList.toggle("on", x === b || (x.dataset.nav !== "lib" && x.dataset.nav !== "mixer" && false)));
    requestAnimationFrame(resizeCanvases);
  } else if (n === "rec") { if (recOn) stopRecording(); else openRecDialog(); }
  else if (n === "settings") openAudioSettings();
}));

/* ---- registrazione dell'uscita master (WAV o MP3), con il file scelto prima di partire */
let recOn = false;
const recExt = () => ($("#recFmt").value.startsWith("mp3") ? "mp3" : "wav");
function openRecDialog() {
  let fmt = "wav16", dir = ""; try { fmt = localStorage.getItem("spinlab.rec_fmt") || fmt; dir = localStorage.getItem("spinlab.rec_dir") || ""; } catch (e) {}
  $("#recFmt").value = fmt; $("#recPath").value = ""; $("#recErr").textContent = ""; $("#recGo").disabled = true;
  $("#recDlg").dataset.dir = dir;
  $("#recDlg").showModal();
}
$("#recFmt").onchange = () => {
  const p = $("#recPath"); if (p.value) p.value = p.value.replace(/\.(wav|mp3)$/i, "." + recExt());
  try { localStorage.setItem("spinlab.rec_fmt", $("#recFmt").value); } catch (e) {}
};
$("#recPick").onclick = async () => {
  const d = new Date(), z = (n) => String(n).padStart(2, "0"), ext = recExt();
  const name = `SpinLab ${d.getFullYear()}-${z(d.getMonth() + 1)}-${z(d.getDate())} ${z(d.getHours())}.${z(d.getMinutes())}.${ext}`;
  const dir = $("#recDlg").dataset.dir;
  const f = await T.dialog.save({ defaultPath: dir ? dir.replace(/[\\/]$/, "") + "\\" + name : name, filters: [{ name: ext === "mp3" ? "MP3" : "WAV", extensions: [ext] }] });
  if (!f) return;
  const p = /\.(wav|mp3)$/i.test(f) ? f.replace(/\.(wav|mp3)$/i, "." + ext) : f + "." + ext;
  $("#recPath").value = p; $("#recGo").disabled = false;
  try { localStorage.setItem("spinlab.rec_dir", p.replace(/[\\/][^\\/]*$/, "")); } catch (e) {}
};
$("#recCancel").onclick = () => $("#recDlg").close();
$("#recGo").onclick = async () => {
  try { await inv("rec_start", { path: $("#recPath").value, format: $("#recFmt").value }); $("#recDlg").close(); }
  catch (e) { $("#recErr").textContent = String(e); }
};
async function stopRecording() {
  try {
    const r = await inv("rec_stop");
    toast2(`Registrazione salvata: ${r.path} (${mmss(r.secs)}, ${(r.bytes / 1048576).toFixed(1)} MB)` + (r.dropped ? ` — attenzione: ${r.dropped} blocchi persi, il disco era troppo lento` : ""));
  } catch (e) { toast(String(e)); }
}
$("#recInd").onclick = stopRecording;
function toast2(msg) { // messaggio di conferma (non e' un errore)
  let t = document.getElementById("toast2");
  if (!t) { t = document.createElement("div"); t.id = "toast2"; document.body.appendChild(t); }
  t.textContent = msg; t.style.display = "block"; clearTimeout(t._h); t._h = setTimeout(() => (t.style.display = "none"), 12000);
}
function updateRec(s) {
  recOn = !!s.rec_on;
  const ind = $("#recInd"); ind.hidden = !recOn;
  if (recOn) { const t = Math.floor(s.rec_secs), h = Math.floor(t / 3600); setText($("#recTime"), (h ? h + ":" : "") + String(Math.floor(t / 60) % 60).padStart(2, "0") + ":" + String(t % 60).padStart(2, "0")); }
  const nb = $('#nav button[data-nav="rec"]'); nb.classList.toggle("recording", recOn); setText(nb, recOn ? "■ STOP REC" : "RECORD");
}

/* ---- pannello destro: stem, sampler, FX del deck selezionato */
let target = 0;
function buildTargets() {
  $("#tgt").innerHTML = [...Array(activeDecks).keys()].map((k) => `<button data-t="${k}" style="--c:${PAL[k].c}">${k + 1}</button>`).join("");
  $$("#tgt button").forEach((b) => (b.onclick = () => setTarget(+b.dataset.t)));
  if (target >= activeDecks) target = 0;
  setTarget(target);
}
function setTarget(i) {
  target = i;
  $$("#tgt button").forEach((b) => b.classList.toggle("on", +b.dataset.t === i));
  $("#rpanel").style.setProperty("--c", PAL[i].c);
  syncFxUi(i); updateStemsUI(true);
  $("#stemStatus").textContent = decks[i].stText || "";
}
function setPanel(name) {
  $$(".ptabs button").forEach((b) => b.classList.toggle("on", b.dataset.pn === name));
  $$(".pn").forEach((p) => p.classList.toggle("on", p.id === "pn-" + name));
  $("#rpanel").classList.toggle("full", name === "drums"); // i pad della batteria usano tutto lo spazio: via FX e selettore deck
  requestAnimationFrame(resizeCanvases);
}
$$(".ptabs button").forEach((b) => (b.onclick = () => setPanel(b.dataset.pn)));

/* FX: i comandi del pannello agiscono sul deck selezionato e restano allineati alla striscia del mixer */
// nome dei tre comandi e se TIME/FEEL contano, per ogni effetto
const FXMAP = {
  echo: ["TONE", "FEEDBACK", "MIX", true], pingpong: ["TONE", "FEEDBACK", "MIX", true], reverb: ["SIZE", "", "MIX", false], flanger: ["RATE", "FEEDBACK", "MIX", false],
  bjump: ["SIZE", "DIR ◀ ▶", "", false], brake: ["TIME", "", "", false], spin: ["TIME", "", "", false], rev: ["", "", "", false],
  phaser: ["DEPTH", "FEEDBACK", "MIX", true], crush: ["BITS", "RATE", "MIX", false], gater: ["DEPTH", "LENGTH", "MIX", true], scatter: ["AMOUNT", "PATTERN", "MIX", true], noise: ["CUTOFF", "RESO", "LEVEL", false],
};
// effetti sul brano (agiscono sul disco, non sul segnale): beat jump, freno, spinback, reverse
const MOTFX = new Set(["bjump", "brake", "spin", "rev"]), JUMPSZ = [1, 2, 4, 8, 16, 32];
// clic = acceso/spento; per freno e reverse, tenendo premuto l'effetto dura quanto la pressione
function bindFxBtn(btn, getD) {
  let t0 = 0;
  btn.onpointerdown = (e) => {
    e.preventDefault();
    const d = getD(); t0 = performance.now();
    d.fx.on = !d.fx.on; d.sendFx();
  };
  btn.onpointerup = btn.onpointerleave = () => {
    const d = getD();
    if (t0 && performance.now() - t0 > 350 && d.fxMot && d.fx.on) { d.fx.on = false; d.sendFx(); }
    t0 = 0;
  };
}
function syncFxUi(i) {
  const f = decks[i].fx, D = DK[i], lit = f.on || performance.now() < (decks[i].fxFlash || 0);
  if (D) { D.fxkind.value = f.kind; D.fxon.classList.toggle("on", lit); }
  if (i === target) {
    const m = FXMAP[f.kind] || FXMAP.echo;
    $("#fxKind").value = f.kind; $("#fxOn").classList.toggle("on", lit);
    $("#fxDepth")._set(f.param); $("#fxFb")._set(f.param2); $("#fxMix")._set(f.wet);
    $("#fxL1").textContent = m[0]; $("#fxL2").textContent = m[1] || "—"; $("#fxL3").textContent = m[2];
    $("#fxFb").parentElement.classList.toggle("dim", !m[1]);
    $("#fxDepth").parentElement.classList.toggle("dim", !m[0]);
    $("#fxMix").parentElement.classList.toggle("dim", !m[2]);
    $("#fxTime").classList.toggle("off", !m[3]);
    $("#fxDiv").value = String(f.div); $("#fxFeel").value = String(f.feel);
  }
}
$("#fxDiv").onchange = (e) => { decks[target].fx.div = +e.target.value; decks[target].sendFx(); };
$("#fxFeel").onchange = (e) => { decks[target].fx.feel = +e.target.value; decks[target].sendFx(); };
$("#fxKind").onchange = (e) => { const f = decks[target].fx; f.kind = e.target.value; decks[target].sendFx(); };
bindFxBtn($("#fxOn"), () => decks[target]);
makeKnob($("#fxDepth"), { value: 0.5, def: 0.5, onChange: (v) => { decks[target].fx.param = v; decks[target].sendFx(); } });
makeKnob($("#fxFb"), { value: 0.5, def: 0.5, onChange: (v) => { decks[target].fx.param2 = v; decks[target].sendFx(); } });
makeKnob($("#fxMix"), { value: 0.5, def: 0.5, onChange: (v) => { decks[target].fx.wet = v; decks[target].sendFx(); } });

/* ---- stem: un fader per stem (con muto e solo) sul deck selezionato. La separazione parte in anticipo; il cambio e' immediato */
const STEMS = [{ n: "VOCALS", k: 3, c: "#b06cff", ic: IC.mic }, { n: "DRUMS", k: 0, c: "#18b6ff", ic: IC.drum }, { n: "BASS", k: 1, c: "#ff3b4e", ic: IC.bass }, { n: "INSTR", k: 2, c: "#ff9a1a", ic: IC.inst }];
let stemTimer = 0;
function sendStems(i) {
  clearTimeout(stemTimer);
  stemTimer = setTimeout(async () => {
    const d = decks[i]; if (!d.snap?.path) return;
    try { await inv("deck_stems", { deck: i, gains: d.stGains }); } catch (e) { toast(String(e)); }
  }, 50);
}
function touchStems(i) { decks[i].stTouch = performance.now(); updateStemsUI(true); sendStems(i); }
$("#stemCols").innerHTML = STEMS.map((s) => `<div class="stcol" data-k="${s.k}" style="--c:${s.c}"><div class="sic">${s.ic}</div><div class="sn">${s.n}</div><div class="fader v" data-f="${s.k}"></div><div class="ms"><button class="m" data-m="${s.k}" title="Muto">M</button><button class="s" data-s="${s.k}" title="Solo">S</button></div></div>`).join("");
$$("#stemCols .fader").forEach((f) => makeFader(f, { value: 1, def: 1, onChange: (v) => { const d = decks[target], k = +f.dataset.f; d.stGains[k] = v; if (v > 0.001) d.stMem[k] = v; touchStems(target); } }));
const toggleStem = (k) => { const d = decks[target]; if (d.stGains[k] > 0.001) { d.stMem[k] = d.stGains[k]; d.stGains[k] = 0; } else d.stGains[k] = d.stMem[k] || 1; touchStems(target); };
$$("#stemCols [data-m]").forEach((b) => (b.onclick = () => toggleStem(+b.dataset.m)));
// anche un clic sull'icona (o sul nome) dello stem lo esclude o lo riammette
$$("#stemCols .stcol").forEach((col) => $$(".sic,.sn", col).forEach((x) => { x.onclick = () => toggleStem(+col.dataset.k); x.title = "Clic: escludi o riammetti questo stem"; }));
$$("#stemCols [data-s]").forEach((b) => (b.onclick = () => {
  const d = decks[target], k = +b.dataset.s, solo = d.stGains.every((g, j) => (j === k ? g > 0.001 : g <= 0.001));
  for (let j = 0; j < 4; j++) { if (d.stGains[j] > 0.001) d.stMem[j] = d.stGains[j]; }
  if (solo) d.stGains = d.stMem.map((g) => g || 1); else d.stGains = d.stGains.map((_, j) => (j === k ? d.stMem[j] || 1 : 0));
  touchStems(target);
}));
const PRESETS = { acap: [0, 0, 0, 1], inst: [1, 1, 1, 0], all: [1, 1, 1, 1] };
$$("[data-preset]").forEach((b) => (b.onclick = () => { const d = decks[target]; d.stGains = [...PRESETS[b.dataset.preset]]; d.stGains.forEach((g, k) => { if (g > 0) d.stMem[k] = 1; }); touchStems(target); }));
function updateStemsUI(force) {
  const d = decks[target];
  $$("#stemCols .stcol").forEach((col) => {
    const k = +col.dataset.k, g = d.stGains[k];
    $(".fader", col)._set(g);
    col.classList.toggle("off", g <= 0.001);
    $(".m", col).classList.toggle("on", g <= 0.001);
    $(".s", col).classList.toggle("on", g > 0.001 && d.stGains.every((x, j) => j === k || x <= 0.001));
  });
  $$("[data-preset]").forEach((b) => b.classList.toggle("on", PRESETS[b.dataset.preset].every((x, k) => Math.abs(x - d.stGains[k]) < 0.01)));
}
T.event.listen("stems-progress", (e) => {
  const [path, pct, gpu] = e.payload;
  decks.forEach((d, i) => {
    if (d.snap?.path === path) { d.stText = pct >= 100 ? "" : `stem ${pct}% ${gpu ? "GPU" : "CPU"}`; if (i === target) $("#stemStatus").textContent = d.stText; }
  });
  showStemBar(path, pct);
});
/* barra di avanzamento della separazione, sotto le playlist */
function showStemBar(path, pct) {
  const bar = $("#stemBar");
  if (pct >= 100) { if (bar.dataset.path === path) bar.hidden = true; return; }
  const row = lib.rows.find((t) => t.path === path);
  bar.dataset.path = path;
  $(".sbn", bar).textContent = `Stem: ${row?.title || path.split("\\").pop()}`;
  $(".sbp", bar).textContent = `${pct}%`;
  bar.style.setProperty("--v", `${pct}%`);
  bar.hidden = false;
}
T.event.listen("stems-ready", (e) => {
  const [path, ok] = e.payload;
  showStemBar(path, 100);
  decks.forEach((d, i) => { if (d.snap?.path === path) { d.stText = ok ? "" : "errore stem"; if (i === target) $("#stemStatus").textContent = d.stText; } });
});
T.event.listen("stems-error", (e) => toast("Stem: " + e.payload[1]));


/* ================================================================ LIBRERIA */
const lib = { rows: [], sel: new Set(), last: null, playlist: null, folder: null, sort: "", desc: false, pls: [] };
let libToken = 0;
const anaPct = new Map(); // percorso -> avanzamento analisi (0..99) dei brani in analisi
let searchTimer;

const sortKey = { title: "title", artist: "artist", bpm: "bpm", key: "key", genre: "genre", duration: "duration", rating: "rating", tags: "tags" };
async function refreshLib() {
  const q = $("#search").value.trim(), tok = ++libToken;
  let rows = [];
  if (lib.playlist !== null) rows = await inv("lib_search", { query: q, playlist: lib.playlist, sort: lib.sort, desc: lib.desc });
  else if (lib.folder) {
    rows = await inv("fs_tracks", { path: lib.folder });
    const words = q.toLowerCase().split(/\s+/).filter(Boolean);
    if (words.length) rows = rows.filter((t) => { const h = [t.title, t.artist, t.album, t.genre, t.tags, t.comment, t.key, Math.trunc(t.bpm)].join(" ").toLowerCase(); return words.every((w) => h.includes(w)); });
    const k = sortKey[lib.sort];
    if (k) rows.sort((a, b) => { const x = a[k], y = b[k], c = typeof x === "number" ? x - y : String(x).localeCompare(String(y), undefined, { sensitivity: "base" }); return lib.desc ? -c : c; });
  }
  if (tok !== libToken) return; // nel frattempo e' stata scelta un'altra cartella o playlist
  lib.rows = rows;
  lib.sel = new Set([...lib.sel].filter((id) => rows.some((t) => t.id === id)));
  renderRows();
  $("#resCount").textContent = `${rows.length} brani`;
  $("#empty").style.display = rows.length ? "none" : "block";
  $("#empty").textContent = q ? "Nessun risultato."
    : lib.playlist !== null ? "Playlist vuota: trascina qui i brani (o intere cartelle) dall'esplora risorse."
    : lib.folder ? "Nessun brano audio in questa cartella." : "Scegli una cartella dall'esplora risorse a sinistra per vedere i suoi brani.";
}

function stars(n) { return `<span class="stars">${[1, 2, 3, 4, 5].map((k) => `<i data-star="${k}" class="${k <= n ? "on" : ""}">★</i>`).join("")}</span>`; }

function renderRows() {
  const tb = $("#tracks tbody");
  tb.innerHTML = lib.rows.map((t) => `<tr data-id="${t.id}" class="${lib.sel.has(t.id) ? "sel" : ""}${anaPct.has(t.path) ? " ana" : ""}" style="--p:${anaPct.get(t.path) ?? 0}%">
    <td class="num"><button class="pv" data-pv title="Anteprima in cuffia (Maiusc+clic: parte da metà brano). Con l'anteprima attiva, clic o trascinamento sulla riga per spostarsi nel brano">▶</button></td>
    <td class="cv"><span class="cover" data-art="${esc(t.path)}"></span></td>
    <td title="${esc(t.title)}">${esc(t.title)}</td><td>${esc(t.artist)}</td>
    <td class="${t.analyzed && !anaPct.has(t.path) ? "" : "na"}" title="${t.analyzed ? "Analizzato" : "Non ancora analizzato: viene analizzato quando lo metti in una playlist"}">${anaPct.has(t.path) ? anaPct.get(t.path) + "%" : t.bpm ? Math.round(t.bpm) : ""}</td>
    <td class="kc" style="color:${camelotColor(t.key)}">${esc(t.key)}</td><td>${esc(t.genre)}</td>
    <td>${fmt(t.duration)}</td>
    <td class="ld">${[0, 1].map((d) => `<button data-d="${d}">${d + 1}</button>`).join("")}</td>
    <td class="rt">${stars(t.rating)}</td></tr>`).join("");
  markPreview();
  $$(".cover[data-art]", tb).forEach((el) => coverObs.observe(el));
}
// copertine delle righe: si caricano solo quando la riga e' visibile
const coverObs = new IntersectionObserver((ents) => ents.forEach((en) => {
  if (!en.isIntersecting) return;
  const el = en.target; coverObs.unobserve(el);
  getArt(el.dataset.art, 64).then((u) => { if (u) el.style.backgroundImage = `url(${u})`; });
}), { root: $(".tablewrap") });

async function refreshPlaylists() {
  lib.pls = await inv("playlists");
  $("#plList").innerHTML = lib.pls.map((p) => `<div class="plitem ${lib.playlist === p.id ? "on" : ""}" data-pl="${p.id}"><span>${esc(p.name)}</span><em>${p.count}</em></div>`).join("");
}

function setPlaceholder() {
  $("#search").placeholder = lib.playlist !== null ? `Cerca in «${lib.pls.find((p) => p.id === lib.playlist)?.name ?? ""}»…` : "Cerca nella cartella: titolo, artista, tag, chiave, BPM…";
}
function selectPlaylist(id) {
  lib.playlist = id; lib.sel.clear(); lib.sort = ""; lib.desc = false;
  $$("#tree .tn.on").forEach((n) => n.classList.remove("on"));
  setPlaceholder();
  if (id !== null) inv("analyze_playlist", { id });
  refreshPlaylists(); refreshLib(); showInspector();
}
function selectFolder(path) {
  lib.playlist = null; lib.folder = path; lib.sel.clear(); lib.sort = ""; lib.desc = false;
  try { localStorage.setItem("spinlab.folder", path); } catch (e) {}
  setPlaceholder();
  $$("#tree .tn").forEach((n) => n.classList.toggle("on", n.dataset.path === path));
  refreshPlaylists(); refreshLib(); showInspector();
}

/* ---- esplora risorse ad albero: unita' e cartelle del computer */
const tree = { roots: [], kids: new Map(), open: new Set() };
async function loadKids(path) { if (!tree.kids.has(path)) tree.kids.set(path, await inv("fs_list", { path })); return tree.kids.get(path); }
function treeHTML(nodes, depth) {
  return nodes.map((n) => {
    const kids = tree.kids.get(n.path), open = tree.open.has(n.path);
    const arrow = kids && kids.length === 0 ? "" : open ? "▼" : "▶";
    return `<div class="tn ${lib.playlist === null && lib.folder === n.path ? "on" : ""}" data-path="${esc(n.path)}" style="padding-left:${4 + depth * 14}px"><span class="ar">${arrow}</span><span class="ic">${depth === 0 && /^Unit/.test(n.name) ? "💽" : "📁"}</span><span class="nm" title="${esc(n.path)}">${esc(n.name)}</span></div>` +
      (open && kids ? treeHTML(kids, depth + 1) : "");
  }).join("");
}
function renderTree() { $("#tree").innerHTML = treeHTML(tree.roots, 0); }
async function toggleNode(path, force) {
  if (tree.open.has(path) && !force) tree.open.delete(path);
  else { tree.open.add(path); await loadKids(path); }
  renderTree();
}
$("#tree").addEventListener("click", async (e) => {
  const n = e.target.closest(".tn"); if (!n) return;
  const path = n.dataset.path;
  if (e.target.classList.contains("ar")) return toggleNode(path);
  selectFolder(path);
  if (!tree.open.has(path)) toggleNode(path, true);
});
async function initTree() {
  tree.roots = await inv("fs_roots");
  renderTree();
  let last = null; try { last = localStorage.getItem("spinlab.folder"); } catch (e) {}
  if (last) {
    // riapre il percorso fino all'ultima cartella usata
    const root = tree.roots.filter((r) => last.toLowerCase().startsWith(r.path.toLowerCase())).sort((a, b) => b.path.length - a.path.length)[0];
    if (root) {
      const parts = last.slice(root.path.length).split(/[\\/]/).filter(Boolean);
      let cur = root.path; tree.open.add(cur); await loadKids(cur);
      for (const p of parts) { cur = (/[\\/]$/.test(cur) ? cur : cur + "\\") + p; tree.open.add(cur); await loadKids(cur); }
      renderTree(); selectFolder(last);
    }
  }
}

$("#search").addEventListener("input", () => { clearTimeout(searchTimer); searchTimer = setTimeout(refreshLib, 120); });
$("#side").addEventListener("click", (e) => {
  const it = e.target.closest(".plitem"); if (it) selectPlaylist(+it.dataset.pl);
});
$("#side").addEventListener("contextmenu", (e) => {
  const it = e.target.closest(".plitem"); if (!it) return;
  e.preventDefault(); const id = +it.dataset.pl;
  menu(e, [["Analizza tutti i brani", () => inv("analyze_playlist", { id })], ["Crea stem di tutta la playlist", () => inv("stems_queue_playlist", { id })], ["Rinomina…", async () => { const n = prompt("Nuovo nome", lib.pls.find((p) => p.id === id).name); if (n) { await inv("playlist_rename", { id, name: n }); refreshPlaylists(); } }],
    ["Elimina playlist", async () => { if (confirm("Eliminare la playlist? (i file restano sul disco)")) { await inv("playlist_delete", { id }); if (lib.playlist === id) { selectPlaylist(null); if (lib.folder) selectFolder(lib.folder); } else refreshPlaylists(); } }]]);
});
$("#newPl").onclick = async () => { const n = prompt("Nome della playlist"); if (n?.trim()) { const id = await inv("playlist_create", { name: n.trim() }); await refreshPlaylists(); selectPlaylist(id); } };

/* avanzamento dell'analisi del singolo brano: barra trasparente sulla riga e percentuale nella colonna BPM */
let anaRefresh = 0;
T.event.listen("analysis-track", (e) => {
  const [path, pct] = e.payload;
  const row = lib.rows.find((t) => t.path === path);
  if (pct >= 100) anaPct.delete(path); else anaPct.set(path, pct);
  if (row) {
    const tr = $(`#tracks tbody tr[data-id="${row.id}"]`);
    if (tr) {
      tr.classList.toggle("ana", pct < 100); tr.style.setProperty("--p", pct + "%");
      const td = tr.children[4]; if (pct < 100) { td.textContent = pct + "%"; td.className = "na"; }
    }
  }
  decks.forEach((d, i) => { if (d.loading === path && pct < 100) $(`.deck[data-i="${i}"] .title`).textContent = `Analisi in corso… ${pct}%`; });
  if (pct >= 100) { clearTimeout(anaRefresh); anaRefresh = setTimeout(refreshLib, 250); }
});

/* stato dell'analisi in background dei brani delle playlist */
T.event.listen("analysis-progress", (e) => {
  const [done, total] = e.payload;
  $("#scanStatus").textContent = total ? `Analisi brani… ${done}/${total}` : "";
});

$$("#tracks th[data-s]").forEach((th) => (th.onclick = () => {
  const s = th.dataset.s; lib.desc = lib.sort === s ? !lib.desc : false; lib.sort = s; refreshLib();
}));

/* ---- anteprima in cuffia: un solo brano alla volta, sull'uscita scelta in Impostazioni audio */
const pv = { path: null, playing: false, pos: 0, dur: 0 };
function markPreview() {
  $$("#tracks tbody tr").forEach((tr) => {
    const t = trackById(+tr.dataset.id), on = !!t && pv.playing && t.path === pv.path;
    tr.classList.toggle("pvon", on);
    tr.style.setProperty("--pp", on && pv.dur ? (pv.pos / pv.dur) * 100 + "%" : "0%");
    const b = $(".pv", tr); if (b) b.textContent = on ? "■" : "▶";
  });
}
async function togglePreview(t, half) {
  if (!t) return;
  if (pv.playing && pv.path === t.path) { await inv("preview_stop"); pv.playing = false; markPreview(); return; }
  pv.path = t.path; pv.playing = true; pv.pos = 0; pv.dur = t.duration; markPreview();
  try { await inv("preview_play", { path: t.path, start: half ? 0.5 : 0 }); }
  catch (e) { pv.playing = false; markPreview(); toast(String(e)); }
}
/* scrub dell'anteprima: clic o trascinamento sulla riga in anteprima porta l'ascolto in quel punto del brano */
let pvSeek = null, pvSeekRaf = 0;
const pvFrac = (e) => { const r = pvSeek.getBoundingClientRect(); return Math.min(1, Math.max(0, (e.clientX - r.left) / r.width)); };
const pvApply = (e) => {
  const f = pvFrac(e); pv.pos = f * pv.dur; markPreview();
  if (!pvSeekRaf) pvSeekRaf = requestAnimationFrame(() => { pvSeekRaf = 0; inv("preview_seek", { frac: pv.pos / (pv.dur || 1) }); });
};
$("#tracks tbody").addEventListener("pointerdown", (e) => {
  const tr = e.target.closest("tr");
  if (!tr || e.button !== 0 || !tr.classList.contains("pvon") || e.target.closest("button,.stars,.chip")) return;
  pvSeek = tr; pvApply(e);
});
document.addEventListener("pointermove", (e) => { if (pvSeek) pvApply(e); });
document.addEventListener("pointerup", () => { pvSeek = null; });

setInterval(async () => {
  if (!pv.playing || pvSeek) return;
  try { const s = await inv("preview_state"); Object.assign(pv, { path: s.path || pv.path, playing: s.playing, pos: s.pos, dur: s.duration || pv.dur }); markPreview(); } catch (e) {}
}, 250);

/* selezione, caricamento, tag */
const trackById = (id) => lib.rows.find((t) => t.id === id);
$("#tracks tbody").addEventListener("click", (e) => {
  const tr = e.target.closest("tr"); if (!tr) return;
  const id = +tr.dataset.id;
  if (e.target.closest("[data-pv]")) { togglePreview(trackById(id), e.shiftKey); return; }
  if (e.target.dataset.chip) { $("#search").value = e.target.dataset.chip; refreshLib(); return; }
  if (e.target.dataset.d !== undefined) { const t = trackById(id); loadTrack(+e.target.dataset.d, t.path); return; }
  if (e.target.dataset.star) { const t = trackById(id); const n = +e.target.dataset.star === t.rating ? 0 : +e.target.dataset.star; inv("lib_update_tags", { id, update: { rating: n } }).then(refreshLib); return; }
  if (e.ctrlKey) lib.sel.has(id) ? lib.sel.delete(id) : lib.sel.add(id);
  else if (e.shiftKey && lib.last != null) {
    const ids = lib.rows.map((t) => t.id), a = ids.indexOf(lib.last), b = ids.indexOf(id);
    lib.sel = new Set(ids.slice(Math.min(a, b), Math.max(a, b) + 1));
  } else lib.sel = new Set([id]);
  lib.last = id; $$("#tracks tbody tr").forEach((r) => r.classList.toggle("sel", lib.sel.has(+r.dataset.id))); showInspector();
});
$("#tracks tbody").addEventListener("dblclick", (e) => {
  const tr = e.target.closest("tr"); if (!tr || e.target.closest(".ld,.stars,.chip,.pv")) return;
  const t = trackById(+tr.dataset.id); const free = [...Array(activeDecks).keys()].find((k) => !decks[k].snap?.loaded && !decks[k].snap?.playing) ?? 0;
  loadTrack(free, t.path);
});
$("#tracks tbody").addEventListener("contextmenu", async (e) => {
  const tr = e.target.closest("tr"); if (!tr) return; e.preventDefault();
  const id = +tr.dataset.id; if (!lib.sel.has(id)) { lib.sel = new Set([id]); renderRows(); showInspector(); }
  const ids = [...lib.sel];
  const items = [...Array(activeDecks).keys()].map((d) => [`Carica sul deck ${d + 1}`, () => loadTrack(d, trackById(id).path)]);
  items.push("-");
  lib.pls.forEach((p) => items.push([`Aggiungi a «${p.name}»`, async () => { await inv("playlist_add", { id: p.id, tracks: ids }); refreshPlaylists(); }]));
  items.push(["Nuova playlist con selezione…", async () => { const n = prompt("Nome della playlist"); if (n?.trim()) { const pid = await inv("playlist_create", { name: n.trim() }); await inv("playlist_add", { id: pid, tracks: ids }); refreshPlaylists(); } }]);
  items.push("-");
  items.push(["Analizza (bpm, chiave, beatgrid)", () => inv("analyze_tracks", { ids })]);
  items.push(["Crea stem (salvati nella cartella «stems»)", () => inv("stems_queue", { ids, front: false })]);
  // l'eliminazione compare solo se almeno un brano selezionato ha gli stem pronti
  const have = await inv("stems_have", { paths: ids.map((k) => trackById(k)?.path).filter(Boolean) }).catch(() => []);
  if (have.some(Boolean)) items.push(["Elimina gli stem preparati", () => inv("stems_delete", { ids })]);
  if (lib.playlist !== null) items.push(["Rimuovi dalla playlist", () => removeFromPlaylist(ids)]);
  menu(e, items);
});

async function removeFromPlaylist(ids) {
  if (lib.playlist === null || !ids.length) return;
  await inv("playlist_remove", { id: lib.playlist, tracks: ids }); lib.sel.clear(); refreshPlaylists(); refreshLib(); showInspector();
}
document.addEventListener("keydown", (e) => {
  if (e.key !== "Delete" || e.target.matches("input,textarea,select")) return;
  removeFromPlaylist([...lib.sel]);
});

function showInspector() {
  const form = $("#insp"), t = lib.sel.size === 1 ? trackById([...lib.sel][0]) : null;
  form.hidden = !t; $("#insp-empty").hidden = !!t || false;
  $("#insp-empty").textContent = lib.sel.size > 1 ? `${lib.sel.size} brani selezionati` : "Seleziona un brano";
  if (!t) return;
  form.dataset.id = t.id;
  ["title", "artist", "album", "genre", "tags", "comment"].forEach((k) => (form.elements[k].value = t[k]));
  $("#stars").innerHTML = stars(t.rating);
}
$("#insp").addEventListener("change", async (e) => {
  const id = +$("#insp").dataset.id, k = e.target.name; if (!k) return;
  await inv("lib_update_tags", { id, update: { [k]: e.target.value } }); refreshLib();
});
$("#brUp").onclick = () => libMove(-1);
$("#brDown").onclick = () => libMove(1);
// frecce su/giu' sfogliano la libreria; Invio carica il brano selezionato sul deck in uso
document.addEventListener("keydown", (e) => {
  if (e.ctrlKey || e.altKey || e.metaKey || e.target.matches("input,textarea,select") || $("#audioDlg").open || $("#recDlg").open) return;
  if (e.key === "ArrowDown") { e.preventDefault(); libMove(1); }
  else if (e.key === "ArrowUp") { e.preventDefault(); libMove(-1); }
  else if (e.key === "Enter") {
    const sel = lib.rows.find((t) => lib.sel.has(t.id));
    if (sel && focusDeck < activeDecks) { e.preventDefault(); loadTrack(focusDeck, sel.path); }
  }
});
$("#inspBtn").onclick = () => { $("#inspector").hidden = !$("#inspector").hidden; };
$("#inspClose").onclick = () => ($("#inspector").hidden = true);
$("#stars").addEventListener("click", async (e) => {
  if (!e.target.dataset.star) return; const id = +$("#insp").dataset.id, t = trackById(id);
  const n = +e.target.dataset.star === t.rating ? 0 : +e.target.dataset.star;
  await inv("lib_update_tags", { id, update: { rating: n } }); await refreshLib(); showInspector();
});

/* menu contestuale */
function menu(e, items) {
  const m = $("#ctx");
  m.innerHTML = items.map((it, k) => (it === "-" ? "<hr>" : `<div data-k="${k}">${esc(it[0])}</div>`)).join("");
  m.hidden = false; m.style.left = Math.min(e.clientX, innerWidth - 220) + "px"; m.style.top = Math.min(e.clientY, innerHeight - m.offsetHeight - 6) + "px";
  m.onclick = (ev) => { const k = ev.target.dataset.k; if (k !== undefined) items[k][1](); m.hidden = true; };
}
document.addEventListener("pointerdown", (e) => { if (!e.target.closest("#ctx")) $("#ctx").hidden = true; });

/* ---- drag & drop con il puntatore (funziona anche con il drop dei file di sistema attivo) */
let drag = null;
/* niente drag nativo di WebKit (immagini/canvas/testo): il trascinamento e' gestito qui con i pointer event */
document.addEventListener("dragstart", (e) => e.preventDefault());
/* durante il trascinamento WebKit selezionerebbe il testo come una pagina web: bloccato alla fonte */
document.addEventListener("selectstart", (e) => { if (drag?.on) e.preventDefault(); });
$("#tracks tbody").addEventListener("pointerdown", (e) => {
  const tr = e.target.closest("tr"); if (!tr || e.button !== 0 || e.target.closest("button,.stars,.chip")) return;
  if (tr.classList.contains("pvon")) return; // sulla riga in anteprima il trascinamento fa lo scrub, non sposta il brano
  drag = { id: +tr.dataset.id, x: e.clientX, y: e.clientY, on: false };
});
$("#tree").addEventListener("pointerdown", (e) => {
  const n = e.target.closest(".tn"); if (!n || e.button !== 0 || e.target.classList.contains("ar")) return;
  drag = { folder: n.dataset.path, label: n.querySelector(".nm").textContent, x: e.clientX, y: e.clientY, on: false };
});
document.addEventListener("pointermove", (e) => {
  if (!drag) return;
  if (!drag.on && Math.hypot(e.clientX - drag.x, e.clientY - drag.y) > 6) {
    drag.on = true;
    getSelection()?.removeAllRanges(); // azzera l'eventuale selezione partita col trascinamento
    const g = $("#ghost"); g.hidden = false;
    if (drag.folder) g.textContent = "📁 " + drag.label;
    else {
      if (!lib.sel.has(drag.id)) { lib.sel = new Set([drag.id]); renderRows(); }
      drag.ids = lib.rows.filter((t) => lib.sel.has(t.id)).map((t) => t.id);
      g.textContent = drag.ids.length > 1 ? `${drag.ids.length} brani` : trackById(drag.id).title;
    }
  }
  if (!drag.on) return;
  const g = $("#ghost"); g.style.left = e.clientX + 12 + "px"; g.style.top = e.clientY + 8 + "px";
  $$(".drop,.dropbefore").forEach((x) => x.classList.remove("drop", "dropbefore"));
  const el = document.elementFromPoint(e.clientX, e.clientY);
  if (!drag.folder) el?.closest(".deck")?.classList.add("drop");
  if (!drag.folder) el?.closest(".pad")?.classList.add("drop");
  if (!drag.folder) el?.closest(".dpad")?.classList.add("drop");
  const pl = el?.closest(".plitem"); pl?.classList.add("drop");
  if (!drag.folder && lib.playlist !== null && !lib.sort) el?.closest("#tracks tbody tr")?.classList.add("dropbefore");
});
document.addEventListener("pointerup", async (e) => {
  if (!drag) return; const d = drag; drag = null;
  $("#ghost").hidden = true; $$(".drop,.dropbefore").forEach((x) => x.classList.remove("drop", "dropbefore"));
  if (!d.on) return;
  const el = document.elementFromPoint(e.clientX, e.clientY);
  const deck = el?.closest(".deck"), pl = el?.closest(".plitem"), row = el?.closest("#tracks tbody tr"), padEl = el?.closest(".pad"), dpEl = el?.closest(".dpad");
  if (d.folder) {
    if (!pl) return;
    $("#scanStatus").textContent = "Ricerca dei brani nella cartella…";
    try { const n = await inv("playlist_add_folder", { id: +pl.dataset.pl, folder: d.folder }); $("#scanStatus").textContent = `Aggiunti ${n} brani: analisi in corso`; } catch (err) { toast(String(err)); }
    refreshPlaylists(); if (lib.playlist === +pl.dataset.pl) refreshLib();
    return;
  }
  if (dpEl) loadDrum(+dpEl.dataset.k, trackById(d.id).path);
  else if (padEl) loadPad(+padEl.dataset.p, trackById(d.id).path);
  else if (deck) loadTrack(+deck.dataset.i, trackById(d.id).path);
  else if (pl) { await inv("playlist_add", { id: +pl.dataset.pl, tracks: d.ids }); refreshPlaylists(); }
  else if (row && lib.playlist !== null && !lib.sort) {
    const to = lib.rows.findIndex((t) => t.id === +row.dataset.id);
    for (const id of d.ids.slice().reverse()) await inv("playlist_move", { id: lib.playlist, track: id, to });
    refreshLib();
  } else if (el?.closest("#browser") && lib.playlist !== null && !row) { /* niente */ }
});

/* drop di file dal sistema operativo sui deck / sulla libreria */
T.event.listen("tauri://drag-drop", async (e) => {
  const { paths, position } = e.payload, dpr = window.devicePixelRatio || 1;
  const el = document.elementFromPoint(position.x / dpr, position.y / dpr);
  const dpEl = el?.closest(".dpad");
  if (dpEl && paths.length) { loadDrum(+dpEl.dataset.k, paths[0]); return; }
  const padEl = el?.closest(".pad");
  if (padEl && paths.length) { loadPad(+padEl.dataset.p, paths[0]); return; }
  const deck = el?.closest(".deck");
  if (deck && paths.length) { loadTrack(+deck.dataset.i, paths[0]); return; }
  // sulle playlist: aggiunge file e cartelle (sottocartelle comprese); altrimenti alla playlist aperta
  const plEl = el?.closest(".plitem"), pid = plEl ? +plEl.dataset.pl : lib.playlist;
  if (pid === null || pid === undefined) { toast("Trascina i file su una playlist, oppure aprine una."); return; }
  const ids = await inv("lib_add_files", { paths });
  await inv("playlist_add", { id: pid, tracks: ids });
  refreshPlaylists(); if (lib.playlist === pid) refreshLib();
});

/* ============================================================ CAMPIONATORE */
function buildPads() {
  $("#pads").innerHTML = [0, 1, 2, 3, 4, 5, 6, 7].map((k) => `<div class="pad" data-p="${k}"><span class="lp"></span><span class="pb"></span><div>Pad ${k + 1}<small>vuoto</small></div></div>`).join("");
  $$(".pad").forEach((p) => {
    const k = +p.dataset.p;
    p.addEventListener("pointerdown", async (e) => {
      if (e.button !== 0) return;
      if (e.target.closest(".lp")) { inv("pad_cmd", { pad: k, cmd: "loop", a: padState[k]?.looped ? 0 : 1 }); return; }
      if (!p.classList.contains("loaded")) return loadPad(k);
      // clic: parte; un altro clic lo ferma (in modo sync parte sul prossimo battito)
      inv("pad_cmd", { pad: k, cmd: "trigger", a: 0 });
    });
    p.addEventListener("contextmenu", (e) => {
      e.preventDefault();
      const s = padState[k];
      menu(e, [["Carica un altro suono…", () => loadPad(k)], [s?.looped ? "Loop: spegni (suona una volta)" : "Loop: accendi (ripete)", () => inv("pad_cmd", { pad: k, cmd: "loop", a: s?.looped ? 0 : 1 })], ["Stop", () => inv("pad_cmd", { pad: k, cmd: "stop", a: 0 })], "-",
        [`BPM del campione${s?.bpm ? ` (${s.bpm.toFixed(1)})` : ""}…`, () => { const v = prompt("BPM del campione (0 = nessuno: partirà sul battito ma senza adeguare il tempo)", s?.bpm ? s.bpm.toFixed(1) : "0"); if (v !== null) inv("pad_cmd", { pad: k, cmd: "bpm", a: parseFloat(v.replace(",", ".")) || 0 }); }],
        ["BPM ÷2", () => inv("pad_cmd", { pad: k, cmd: "bpm_half", a: 0 })], ["BPM ×2", () => inv("pad_cmd", { pad: k, cmd: "bpm_double", a: 0 })]]);
    });
  });
  $("#padSync").onclick = () => inv("mixer_cmd", { cmd: "pad_sync", a: $("#padSync").classList.contains("on") ? 0 : 1 });
}
const padState = [];
async function loadPad(k, path) {
  const f = path || (await pickFile());
  if (!f) return;
  const el = $(`.pad[data-p="${k}"]`); el?.classList.add("busy");
  try { await inv("pad_load", { pad: k, path: f }); } catch (e) { toast(String(e)); }
  el?.classList.remove("busy");
}
function updatePads(ps, sync) {
  $("#padSync").classList.toggle("on", !!sync);
  $$(".pad").forEach((p, k) => {
    const s = ps[k]; padState[k] = s;
    p.classList.toggle("loaded", s.loaded); p.classList.toggle("playing", s.playing); p.classList.toggle("armed", s.armed);
    const label = s.loaded ? `${esc(s.name)}<small>${s.armed ? "in attesa del battito…" : "pad " + (k + 1)}</small>` : `Pad ${k + 1}<small>vuoto — clic o trascina un file</small>`;
    const box = $("div", p); if (box._h !== label) { box._h = label; box.innerHTML = label; }
    $(".pb", p).textContent = s.loaded && s.bpm ? Math.round(s.bpm) + " BPM" : "";
    const lp = $(".lp", p); lp.textContent = s.loaded ? (s.looped ? "∞" : "1×") : ""; lp.title = s.looped ? "Loop: ripete all'infinito finché non lo fermi con un altro clic (clic qui per suonarlo una volta sola)" : "Suona una volta sola (clic qui per metterlo in loop)";
  });
}
/* ============================================================ BATTERIA */
const DKEYS = ["z", "x", "c", "v", "a", "s", "d", "f", "q", "w", "e", "r", "1", "2", "3", "4"]; // pad 0..15, in basso a sinistra il primo (come su un controller)
const ROWC = ["#ff3b4e", "#ffb020", "#35e08a", "#18b6ff"];
const drumState = []; let prevLearn = null;
function buildDrums() {
  const order = [12, 13, 14, 15, 8, 9, 10, 11, 4, 5, 6, 7, 0, 1, 2, 3];
  $("#drumPads").innerHTML = order.map((k) => `<div class="dpad" data-k="${k}" style="--c:${ROWC[Math.floor(k / 4)]}"><span class="dk">${DKEYS[k].toUpperCase()}</span><span class="dn"></span><div class="dl">vuoto</div></div>`).join("");
  $$(".dpad").forEach((p) => {
    const k = +p.dataset.k; let down = false;
    p.addEventListener("pointerdown", (e) => {
      if (e.button !== 0) return;
      if (!p.classList.contains("loaded")) return loadDrum(k);
      down = true; p.setPointerCapture(e.pointerId); p.classList.add("hit");
      inv("drum_cmd", { pad: k, cmd: "trigger", a: e.shiftKey ? 0.5 : 0.95 });
    });
    const up = () => { if (!down) return; down = false; p.classList.remove("hit"); inv("drum_cmd", { pad: k, cmd: "release", a: 0 }); };
    ["pointerup", "pointercancel"].forEach((ev) => p.addEventListener(ev, up));
    p.addEventListener("contextmenu", (e) => {
      e.preventDefault(); const s = drumState[k];
      const ask = (txt, cur, cmd, scale = 1) => { const v = prompt(txt, cur); if (v !== null && v.trim() !== "") inv("drum_cmd", { pad: k, cmd, a: (parseFloat(v.replace(",", ".")) || 0) * scale }); };
      menu(e, [["Carica un campione…", () => loadDrum(k)], ["MIDI learn: poi colpisci un pad del controller", () => inv("drum_cmd", { pad: k, cmd: "learn", a: 1 })], "-",
        [s?.gate ? "Modo: suona per intero (spegni gate)" : "Modo: gate (suona finché tieni premuto)", () => inv("drum_cmd", { pad: k, cmd: "gate", a: s?.gate ? 0 : 1 })],
        [`Aftertouch: ${["spento", "volume", "intonazione", "volume + intonazione"][s?.at_mode ?? 1]} (clic per cambiare)`, () => inv("drum_cmd", { pad: k, cmd: "aftertouch", a: ((s?.at_mode ?? 1) + 1) % 4 })],
        [`Intonazione (${(s?.tune ?? 0).toFixed(1)} semitoni)…`, () => ask("Intonazione in semitoni (da -12 a +12)", String(s?.tune ?? 0), "tune")],
        [`Gruppo choke (${s?.group || "nessuno"})…`, () => ask("Gruppo choke 1-8: i pad dello stesso gruppo si zittiscono a vicenda (hi-hat chiuso/aperto). 0 = nessuno", String(s?.group ?? 0), "group")],
        [`Volume (${Math.round((s?.gain ?? 1) * 100)}%)…`, () => ask("Volume del pad in % (0-150)", String(Math.round((s?.gain ?? 1) * 100)), "gain", 0.01)],
        [`Nota MIDI (${s?.note === 255 ? "nessuna" : s?.note})…`, () => ask("Numero della nota MIDI (0-127)", String(s?.note ?? 36), "note")], "-",
        ["Svuota il pad", () => inv("drum_cmd", { pad: k, cmd: "clear", a: 0 })]]);
    });
  });
  $("#drumVol").oninput = (e) => inv("mixer_cmd", { cmd: "drum_vol", a: +e.target.value });
}
async function loadDrum(k, path) {
  const f = path || (await pickFile());
  if (!f) return;
  try { await inv("drum_load", { pad: k, path: f }); } catch (e) { toast(String(e)); }
}
function updateDrums(s) {
  if (!s.drums) return;
  $$(".dpad").forEach((p) => {
    const k = +p.dataset.k, d = s.drums[k]; drumState[k] = d;
    p.classList.toggle("loaded", d.loaded); p.classList.toggle("playing", d.playing); p.classList.toggle("learn", s.learn === k);
    const label = s.learn === k ? "colpisci un pad del controller…" : d.loaded ? esc(d.name) : "vuoto";
    const box = $(".dl", p); if (box._h !== label) { box._h = label; box.innerHTML = label; }
    setText($(".dn", p), d.note === 255 ? "—" : String(d.note));
    p.style.setProperty("--at", d.at.toFixed(2));
  });
  if (s.learn === null && prevLearn !== null) inv("drum_save");
  prevLearn = s.learn;
  if (s.last_note >= 0 && s.note_seq !== updateDrums.seq) { updateDrums.seq = s.note_seq; $("#midiStat").title = `Ultima nota MIDI ricevuta: ${s.last_note}`; setText($("#midiStat"), midiNames + " · nota " + s.last_note); }
}
let midiNames = "MIDI: —";
async function pollMidi() {
  try { const p = await inv("midi_ports"); midiNames = p.length ? "MIDI: " + p.join(", ") : "nessun controller MIDI"; const m = $("#midiStat"); m.classList.toggle("ok", p.length > 0); if (updateDrums.seq === undefined) m.textContent = midiNames; } catch (e) {}
  setTimeout(pollMidi, 2000);
}
document.addEventListener("keydown", (e) => {
  if (e.repeat || e.ctrlKey || e.altKey || e.metaKey || e.target.matches("input,textarea,select") || !$("#pn-drums").classList.contains("on")) return;
  const k = DKEYS.indexOf(e.key.toLowerCase()); if (k < 0) return;
  e.preventDefault(); $(`.dpad[data-k="${k}"]`)?.classList.add("hit");
  inv("drum_cmd", { pad: k, cmd: "trigger", a: e.shiftKey ? 0.5 : 0.95 });
});
document.addEventListener("keyup", (e) => {
  const k = DKEYS.indexOf(e.key.toLowerCase()); if (k < 0 || !$("#pn-drums").classList.contains("on")) return;
  $(`.dpad[data-k="${k}"]`)?.classList.remove("hit"); inv("drum_cmd", { pad: k, cmd: "release", a: 0 });
});

const PAD_KEYS = ["1", "2", "3", "4", "5", "6", "7", "8"];
document.addEventListener("keydown", (e) => {
  if (e.target.matches("input,textarea,select") || e.repeat || !$("#pn-sampler").classList.contains("on")) return;
  const k = PAD_KEYS.indexOf(e.key); if (k >= 0) inv("pad_cmd", { pad: k, cmd: "trigger", a: 0 });
});

/* ======================================================= IMPOSTAZIONI AUDIO */
const BUFFERS = [64, 128, 256, 512, 1024, 2048];
async function openAudioSettings(tab) {
  const dlg = $("#audioDlg"), list = await inv("audio_info", { driver: null });
  fillAudio(list, true);
  showAudioStatus(list.current);
  setSettingsTab(typeof tab === "string" ? tab : "audio");
  dlg.showModal();
}
function setSettingsTab(t) {
  $$(".stabs button").forEach((b) => b.classList.toggle("on", b.dataset.sp === t));
  $$(".spane").forEach((p) => (p.hidden = p.id !== "sp-" + t));
  $("#aApply").hidden = t !== "audio";
  if (t === "stem") refreshStemPane();
  if (t === "midi") refreshMidiPane();
}
$$(".stabs button").forEach((b) => (b.onclick = () => setSettingsTab(b.dataset.sp)));

/* scheda STEM: modalita' (automatica / tempo reale / solo preparati) e misura della velocita' del PC */
function showStemCfg(c) {
  $("#stMode").value = c.mode;
  const sp = c.speed > 0 ? `${c.speed.toFixed(1)}×` : "non ancora misurata";
  const how = c.realtime ? `<b>Tempo reale attivo</b>: gli stem si separano mentre il brano suona.` : `<span class="warn"><b>Solo stem preparati</b></span>: su questo PC il tempo reale non regge, prepara gli stem prima (clic destro sul brano → Crea stem). I brani già preparati si caricano subito.`;
  $("#stInfo").innerHTML = `Velocità di separazione: <b>${sp}</b> (secondi di musica al secondo; per il tempo reale servono almeno ${c.min_speed.toFixed(1)}×, perché due deck separano insieme).<br>${how}`;
}
async function refreshStemPane() { try { showStemCfg(await inv("stems_cfg")); } catch (e) {} }
$("#stMode").onchange = async () => showStemCfg(await inv("stems_cfg_set", { mode: $("#stMode").value }));
$("#stBench").onclick = async () => {
  const b = $("#stBench"); b.disabled = true; b.textContent = "Misuro…";
  try { showStemCfg(await inv("stems_bench_cmd")); } catch (e) { toast("Misura non riuscita: " + e); }
  b.disabled = false; b.textContent = "Misura la velocità di questo PC";
};
// riempie driver, dispositivi, frequenze e buffer; `own`: usa le impostazioni in uso, altrimenti quelle di un altro driver appena scelto
function fillAudio(list, own) {
  const cur = list.current;
  $("#aDrv").innerHTML = list.hosts.map((h) => `<option value="${h.toLowerCase()}" ${h === list.driver ? "selected" : ""}>${h === "WASAPI" ? "Windows (WASAPI)" : h === "ASIO" ? "ASIO (driver della scheda)" : h}</option>`).join("");
  const dev = own ? cur.device || list.default_device : list.default_device;
  $("#aDev").innerHTML = list.devices.map((n) => `<option ${n === dev ? "selected" : ""}>${esc(n)}</option>`).join("");
  $("#aRate").innerHTML = `<option value="0">Automatica (${cur.sample_rate} Hz)</option>` + list.rates.map((r) => `<option value="${r}" ${own && r === cur.sample_rate && savedAudio.sample_rate ? "selected" : ""}>${r} Hz</option>`).join("");
  $("#pDev").innerHTML = `<option value="">Predefinita di Windows</option><option value="@main" ${list.preview_device === "@main" ? "selected" : ""}>Stesso dispositivo dell'uscita principale · canali 3-4 (cuffie del controller)</option>` + (list.preview_devices || list.devices).map((n) => `<option ${n === list.preview_device ? "selected" : ""}>${esc(n)}</option>`).join("");
  $("#aBuf").innerHTML = `<option value="0">Automatico</option>` + BUFFERS.map((b) => `<option value="${b}" ${own && b === cur.buffer ? "selected" : ""}>${b} frame (${(b / cur.sample_rate * 1000).toFixed(1)} ms)</option>`).join("");
}
$("#aDrv").onchange = async () => { const l = await inv("audio_info", { driver: $("#aDrv").value }); fillAudio(l, false); };
let savedAudio = {};
function showAudioStatus(i, msg) {
  $("#aStatus").innerHTML = `Uscita attiva: <b>${esc(i.driver || "")} · ${esc(i.device)}</b> a <b>${i.sample_rate} Hz</b>, buffer ${i.buffer || "auto"}${i.actual_buffer ? ` (${i.actual_buffer} frame reali)` : ""}, latenza stimata <b>${i.latency_ms.toFixed(1)} ms</b>. Blocchi audio elaborati: ${i.rendered}${i.missed ? ` (saltati ${i.missed})` : ""}.` + (msg ? `<br>${esc(msg)}` : "") + (i.error ? `<br><span class="warn">${esc(i.error)}</span>` : "");
}
$("#audioBtn").onclick = openAudioSettings;
// guida: il manuale (guida.html, generato da docs/MANUALE.md) in una finestra; i link «data-help» aprono la sezione giusta
function openHelp(section) {
  const f = $("#helpFrame"), url = "guida.html" + (section ? "#" + section : "");
  if (!f.getAttribute("src")) f.src = url; else if (section) f.contentWindow.location.hash = section;
  if (!$("#helpDlg").open) $("#helpDlg").showModal();
}
$("#helpBtn").onclick = () => openHelp();
$("#helpClose").onclick = () => $("#helpDlg").close();
document.addEventListener("click", (e) => { const a = e.target.closest?.("[data-help]"); if (a) { e.preventDefault(); openHelp(a.dataset.help); } });
$("#helpDlg").addEventListener("click", (e) => { if (e.target === $("#helpDlg")) $("#helpDlg").close(); });
$("#aClose").onclick = () => $("#audioDlg").close();
$("#aApply").onclick = async () => {
  const btn = $("#aApply"); btn.disabled = true; btn.textContent = "Applico…";
  const settings = { driver: $("#aDrv").value || null, device: $("#aDev").value || null, sample_rate: +$("#aRate").value || null, buffer: +$("#aBuf").value || null, preview_device: $("#pDev").value || null };
  try {
    const info = await inv("audio_set", { settings });
    const l = await inv("audio_info", { driver: null });
    savedAudio = settings; audioLatency = l.current.latency_ms / 1000;
    fillAudio(l, true); // mostra cio' che e' davvero attivo (se l'ASIO non e' partito si e' tornati a WASAPI)
    const wrong = settings.device && !info.error && l.current.device !== settings.device ? `Attenzione: hai scelto «${settings.device}» ma l'uscita attiva è «${l.current.device}».` : "";
    showAudioStatus(l.current, info.error ? "" : wrong || "Impostazioni applicate e salvate.");
    if (info.error) toast(info.error);
  } catch (e) { $("#aStatus").innerHTML = `<span class="warn">Errore: ${esc(e)}</span>`; }
  btn.disabled = false; btn.textContent = "Applica";
};
async function refreshLatency() { try { const c = await inv("audio_current"); audioLatency = c.latency_ms / 1000; } catch (e) {} }

/* ================================================================== START */
buildDecks(); buildPads(); buildDrums(); pollMidi(); decks.forEach((d) => d.sendFx()); resizeCanvases();
refreshPlaylists().then(refreshLib);
poll(); requestAnimationFrame(frame);
refreshLatency(); setTimeout(refreshLatency, 1500);
inv("startup_files").then((fs) => fs.slice(0, 2).forEach((f, i) => loadTrack(i, f)));
initTree();
