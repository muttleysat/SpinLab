"use strict";
/* ============================================================ AUTOMIX
   Suona da solo i brani della libreria visibile (una playlist, una cartella, i risultati di una ricerca) uno dopo l'altro, mixandoli come un DJ:
   - quando il brano che suona sta per finire, sul deck libero e' gia' pronto il successivo, con il tempo allineato (SYNC);
   - la transizione dura un certo numero di battute: il crossfader scorre, e con lo stile «scambio dei bassi» i bassi passano da un brano all'altro
     a meta' (cosi' non si sommano due bassi);
   - a fine transizione il deck vecchio si svuota e il giro ricomincia con un nuovo brano in preparazione.
   Tutto passa dai comandi dei deck che si usano anche a mano: decks[i].snap e' lo stato letto dal motore ogni 33 ms. */

const AM = {
  on: false, queue: [], idx: 0,          // coda dei brani e posizione del prossimo da suonare
  cur: 0, nxt: 1,                        // deck che suona e deck in preparazione
  busy: false, mix: null, ramp: null, msg: "",
  opt: { bars: 16, style: "bass", order: "list", loop: false },
};
try { Object.assign(AM.opt, JSON.parse(localStorage.getItem("spinlab.automix") || "{}")); } catch (e) { /* senza memoria locale restano i valori di partenza */ }
const amSave = () => { try { localStorage.setItem("spinlab.automix", JSON.stringify(AM.opt)); } catch (e) { /* non e' grave */ } };

const amSnap = (i) => decks[i]?.snap || {};
const amReady = (i, path) => { const s = amSnap(i); return !!s.loaded && s.path === path && !decks[i].loading; };
const amSleep = (ms) => new Promise((r) => setTimeout(r, ms));
const amKnob = (deck, k) => DK[deck]?.knobs.find((x) => x.dataset.k === String(k));
const amEq = (deck, band, v) => amKnob(deck, band)?._apply(v);          // 0 = bassi, 1 = medi, 2 = alti; 0,5 = neutro
const amXf = (v) => $("#crossfader")?._apply(Math.min(1, Math.max(0, v)));
const amSide = (deck) => (deck % 2 === 0 ? 0 : 1);                       // lato del crossfader del deck (A = deck 1 e 3)

/* ---------------------------------------------------------------- coda */
function camelotDistance(a, b) {
  const pa = /^(\d{1,2})([AB])$/i.exec(a || ""), pb = /^(\d{1,2})([AB])$/i.exec(b || "");
  if (!pa || !pb) return 3;                                               // tonalita' sconosciuta: costo medio
  const na = +pa[1], nb = +pb[1], dn = Math.min((na - nb + 12) % 12, (nb - na + 12) % 12);
  const same = pa[2].toUpperCase() === pb[2].toUpperCase();
  return same ? dn : dn === 0 ? 1 : dn + 2;                               // stessa chiave 0, vicina 1, relativa maggiore/minore 1, altre piu' lontane
}
function bpmDistance(a, b) {
  if (!a || !b) return 0.08;
  let r = b / a; while (r > 1.5) r /= 2; while (r < 0.67) r *= 2;
  return Math.abs(r - 1);
}
function amOrderTracks(rows, first) {
  let list = rows.filter((t) => t.path);
  const o = AM.opt.order, byBpm = o === "bpmup" || o === "bpmdown";
  if (byBpm) {
    // per tempo: i brani senza BPM vanno in fondo, a parita' di tempo si resta nell'ordine della tabella
    const key = (t) => (t.bpm > 0 ? (o === "bpmup" ? t.bpm : -t.bpm) : Infinity);
    list = list.map((t, k) => [t, k]).sort((a, b) => (key(a[0]) === key(b[0]) ? a[1] - b[1] : key(a[0]) < key(b[0]) ? -1 : 1)).map((x) => x[0]);
  }
  if (first != null) {
    const k = list.findIndex((t) => t.id === first);
    // «come in elenco» e per BPM: si va avanti dal brano di partenza fino in fondo (i brani sopra suonano solo con «Ripeti»); gli altri ordini li usano tutti
    if (k > 0) list = (o === "list" || byBpm) && !AM.opt.loop ? list.slice(k) : [...list.slice(k), ...list.slice(0, k)];
  }
  if (AM.opt.order === "shuffle") {
    const head = first != null ? list.splice(0, 1) : [];
    for (let k = list.length - 1; k > 0; k--) { const j = Math.floor(Math.random() * (k + 1)); [list[k], list[j]] = [list[j], list[k]]; }
    return [...head, ...list];
  }
  if (AM.opt.order === "smart" && list.length > 2) {
    // dal primo brano si sceglie sempre il successivo piu' compatibile per tonalita' e tempo (non e' ottimo ma e' stabile e veloce)
    const out = [list.shift()];
    while (list.length) {
      const p = out[out.length - 1]; let best = 0, bc = Infinity;
      list.forEach((t, k) => { const c = camelotDistance(p.key, t.key) * 1.0 + bpmDistance(p.bpm, t.bpm) * 25; if (c < bc) { bc = c; best = k; } });
      out.push(list.splice(best, 1)[0]);
    }
    return out;
  }
  return list;
}

/* ---------------------------------------------------------------- avvio e arresto */
// la coda parte dal brano che suona (o da quello selezionato) e segue l'elenco com'e' ordinato in tabella
function amBuildQueue() {
  const rows = (typeof lib !== "undefined" && lib.rows) || [];
  const playing = [0, 1].find((i) => amSnap(i).playing && amSnap(i).loaded);
  const pth = playing != null ? amSnap(playing).path : null;
  const here = pth ? rows.find((t) => t.path === pth) : null;
  let q = amOrderTracks(rows, here ? here.id : playing == null ? lib.last : null);
  if (pth) q = q.filter((t) => t.path !== pth);
  return { q, playing };
}
// da fermo la coda e' un'anteprima che si rifa' quando cambia qualcosa (cartella, ordine, brano che suona o selezionato),
// a meno che tu non l'abbia modificata a mano: in quel caso resta com'e' finche' non premi «Ripristina»
function amSyncPreview() {
  if (AM.on) return;
  const rows = (typeof lib !== "undefined" && lib.rows) || [];
  const pl = [0, 1].map((i) => (amSnap(i).playing && amSnap(i).loaded ? amSnap(i).path : "")).join("|");
  const sig = [rows.length, rows[0]?.id, rows[rows.length - 1]?.id, AM.opt.order, AM.opt.loop, lib.last, pl].join("/");
  if (sig === AM.sig || (AM.edited && AM.queue.length)) { if (!AM.edited) AM.sig = sig; return; }
  AM.sig = sig; AM.queue = amBuildQueue().q; AM.idx = 0;
}
function amStart() {
  if (!((typeof lib !== "undefined" && lib.rows) || []).length) { toast("AUTOMIX: scegli prima una cartella o una playlist con dei brani"); return; }
  amSyncPreview();
  const playing = [0, 1].find((i) => amSnap(i).playing && amSnap(i).loaded);
  if (playing != null) { const pth = amSnap(playing).path; AM.queue = AM.queue.filter((t) => t.path !== pth); }
  if (!AM.queue.length) { toast("AUTOMIX: la coda e' vuota"); return; }
  AM.idx = 0; AM.mix = null; AM.ramp = null; AM.on = true; AM.busy = false;
  AM.cur = playing ?? [0, 1].find((i) => !amSnap(i).loaded) ?? 0; AM.nxt = 1 - AM.cur;
  amMsg(playing != null ? "In corso: il prossimo brano parte alla fine di quello che suona" : "Avvio…");
  amRender();
}
function amStop(why) {
  AM.on = false; AM.busy = false; AM.mix = null; AM.ramp = null; AM.edited = false; AM.sig = "";
  amMsg(why || "Fermato. I deck restano come sono.");
  amRender();
}
function amMsg(t) { AM.msg = t; }
const amNext = () => {
  if (AM.idx >= AM.queue.length && AM.opt.loop && AM.queue.length) AM.idx = 0;
  return AM.queue[AM.idx] || null;
};

// un brano che hai caricato tu sull'altro deck non va sostituito: diventa il prossimo della coda
function amAdoptManual() {
  const s = amSnap(AM.nxt), d = decks[AM.nxt];
  if (!s.loaded || s.playing || d.loading || !s.path) return;
  const next = AM.queue[AM.idx];
  if ((next && next.path === s.path) || s.path === AM.lastPre) return;   // quello preparato da AUTOMIX stesso non e' una scelta tua
  const known = (typeof lib !== "undefined" && lib.rows.find((t) => t.path === s.path)) || { id: -1, path: s.path, title: s.name, artist: "", bpm: s.bpm_orig };
  const at = AM.queue.findIndex((t, k) => k >= AM.idx && t.path === s.path);
  if (at >= 0) AM.queue.splice(at, 1);
  AM.queue.splice(AM.idx, 0, known);
}

/* ---------------------------------------------------------------- calcoli sul brano che suona */
function amMixLen(a, b) {
  // durata della transizione in secondi di brano (la linea del tempo del file) e se si puo' sincronizzare
  const bpmA = a.bpm_orig, bpmB = b?.bpm;
  const tempoOk = bpmA > 0 && bpmB > 0 && bpmDistance(a.bpm_now || bpmA, bpmB) <= 0.12;
  const bars = AM.opt.style === "cut" ? 2 : AM.opt.bars;
  const sec = bpmA > 0 ? (bars * 4 * 60) / bpmA : 8;
  return { sec: Math.min(sec, Math.max(4, a.duration / 3)), sync: tempoOk && AM.opt.style !== "cut" };
}

/* ---------------------------------------------------------------- il ciclo (ogni 250 ms) */
async function amTick() {
  if (!AM.on || AM.busy) return;
  const A = amSnap(AM.cur);
  // brano che suona: si prepara il prossimo e al momento giusto si parte con la transizione
  if (AM.mix) return amMixStep();
  if (!A.loaded) return amPlayFresh();
  if (!A.playing) {
    // brano finito da solo (nessun mix riuscito): si passa al prossimo; se e' solo in pausa non si fa niente
    if (A.duration > 0 && A.pos >= A.duration - 0.6) return amPlayFresh();
    return;
  }
  amAdoptManual();
  const next = amNext();
  if (!next) { if (A.duration - A.pos < 1.5 || !A.playing) return amStop("Automix terminato: la coda e' finita."); amMsg("Ultimo brano: la coda e' finita"); return; }
  const remaining = A.duration - A.pos, plan = amMixLen(A, next);
  const lead = Math.max(75, plan.sec + 45);
  if (remaining <= lead && !amReady(AM.nxt, next.path) && !decks[AM.nxt].loading) return amPreload(next);
  const barSec = A.bpm_now > 0 ? 240 / A.bpm_now : 3;    // il mix aspetta al massimo un «uno» di battuta: si parte una battuta prima
  if (remaining <= plan.sec + barSec + 0.3 && amReady(AM.nxt, next.path)) return amStartMix(next, plan);
  amMsg(`Sta suonando «${A.name}» · mix tra ${fmt(Math.max(0, remaining - plan.sec))}`);
}

// nessun brano sul deck corrente: carica e suona il prossimo
async function amPlayFresh() {
  const next = amNext();
  if (!next) return amStop("Automix terminato: la coda e' finita.");
  AM.busy = true;
  try {
    amMsg(`Carico «${next.title || next.path}»…`);
    await loadTrack(AM.cur, next.path);
    for (let k = 0; k < 60 && !amReady(AM.cur, next.path); k++) await amSleep(100);
    if (!amReady(AM.cur, next.path)) { AM.idx++; amMsg("Brano non caricato: salto al successivo"); return; }
    amXf(amSide(AM.cur)); amEq(AM.cur, 0, 0.5);
    AM.idx++;
    await dc(AM.cur, "play");
  } finally { AM.busy = false; amRender(); }
}

// prepara il prossimo sul deck libero: carica, allinea il tempo, abbassa i bassi
async function amPreload(next) {
  AM.busy = true;
  try {
    AM.lastPre = next.path;
    amMsg(`Preparo «${next.title || next.path}»…`);
    await loadTrack(AM.nxt, next.path);
    for (let k = 0; k < 80 && !amReady(AM.nxt, next.path); k++) await amSleep(100);
    if (!amReady(AM.nxt, next.path)) { AM.idx++; amMsg("Brano non caricato: salto al successivo"); return; }
    amXf(amSide(AM.cur));                                 // si sente ancora solo il brano che suona
    if (AM.opt.style === "bass") amEq(AM.nxt, 0, 0);       // il nuovo brano entra senza bassi
  } finally { AM.busy = false; amRender(); }
}

async function amStartMix(next, plan) {
  AM.busy = true;
  try {
    // il nuovo brano parte dal suo primo «uno» di battuta, nel momento in cui il deck che suona arriva a un «uno»:
    // cosi' cassa con cassa e rullante con rullante (il sync da solo allinea i battiti, non le battute)
    const barOk = plan.sync && amBarSeek(AM.nxt);
    if (plan.sync) await dc(AM.nxt, "sync");               // tempo e battiti del nuovo brano sul deck che suona
    if (barOk) await amWaitBarStart(AM.cur);
    if (!AM.on) return;
    await dc(AM.nxt, "play");
    AM.mix = { t0: amSnap(AM.cur).pos, sec: plan.sec, from: AM.cur, to: AM.nxt, path: next.path, started: performance.now() };
    AM.idx++;
    amMsg(`Mix in corso verso «${next.title || next.path}»`);
  } finally { AM.busy = false; amRender(); }
}

// indice del battito (nella griglia del deck) a cui si trova la posizione t, e posizione dentro la battuta (0..3) con la fase 0..1
function amBeatAt(deck, t) {
  const d = decks[deck], b = d.beats || [];
  if (b.length < 8) return null;
  let lo = 0, hi = b.length;
  while (lo < hi) { const m = (lo + hi) >> 1; if (b[m] <= t) lo = m + 1; else hi = m; }
  const i = lo - 1;
  if (i < 0 || i >= b.length - 1) return null;
  return { i, bar: (((i - d.downbeat) % 4) + 4) % 4, ph: (t - b[i]) / (b[i + 1] - b[i]) };
}
// porta il deck sul primo «uno» di battuta della griglia; false se la griglia non c'e'
function amBarSeek(deck) {
  const d = decks[deck], b = d.beats || [];
  if (b.length < 8) return false;
  const k = [0, 1, 2, 3].map((x) => x).find((x) => (((x - d.downbeat) % 4) + 4) % 4 === 0);
  if (k == null || b[k] == null) return false;
  dc(deck, "seek", b[k]);
  return true;
}
// aspetta che il deck che suona sia sull'ultimo battito prima dell'«uno»: il play dato ora parte sul battito successivo, cioe' all'«uno»
async function amWaitBarStart(deck) {
  for (let k = 0; k < 400 && AM.on; k++) {
    const s = amSnap(deck), p = amBeatAt(deck, s.pos);
    if (!p) return;
    if (p.bar === 3 && p.ph > 0.1 && p.ph < 0.65) return;
    await amSleep(15);
  }
}

function amMixStep() {
  const m = AM.mix, A = amSnap(m.from);
  const p = Math.min(1, Math.max(0, (A.pos - m.t0) / m.sec));
  const toB = amSide(m.to) === 1;
  amXf(toB ? p : 1 - p);
  if (AM.opt.style === "bass") {
    // scambio dei bassi attorno a meta' transizione
    const s = Math.min(1, Math.max(0, (p - 0.45) / 0.14)), e = s * s * (3 - 2 * s);
    amEq(m.from, 0, 0.5 * (1 - e)); amEq(m.to, 0, 0.5 * e);
  }
  if (p >= 1 || !A.playing || performance.now() - m.started > (m.sec + 20) * 1000) amFinishMix();
}

async function amFinishMix() {
  const m = AM.mix; AM.mix = null; AM.busy = true;
  try {
    const A = amSnap(m.from);
    if (A.playing) await dc(m.from, "play");               // ferma il vecchio
    await ejectDeck(m.from);
    amEq(m.from, 0, 0.5); amEq(m.to, 0, 0.5);
    amXf(amSide(m.to));
    AM.cur = m.to; AM.nxt = m.from;
    // il tempo del nuovo brano era stato adattato al precedente: torna piano al suo tempo originale
    const B = amSnap(AM.cur);
    if (Math.abs(B.tempo_pct || 0) > 0.3) { await dc(AM.cur, "sync_off"); AM.ramp = { deck: AM.cur, from: B.tempo_pct, t0: performance.now(), dur: 24000 }; }
  } finally { AM.busy = false; amRender(); }
}

function amRampStep() {
  const r = AM.ramp; if (!r) return;
  const t = Math.min(1, (performance.now() - r.t0) / r.dur), pct = r.from * (1 - t * t * (3 - 2 * t));
  if (amSnap(r.deck).sync || t >= 1) { if (t >= 1) dc(r.deck, "tempo", 0); AM.ramp = null; return; }
  dc(r.deck, "tempo", pct);
}

function amSkip() {
  if (!AM.on) return;
  const A = amSnap(AM.cur), next = amNext();
  if (AM.mix || !next) return;
  if (!A.loaded || !A.playing) { amPlayFresh(); return; }
  // salta: si porta il brano che suona a pochi secondi dalla fine, cosi' parte la transizione normale
  const plan = amMixLen(A, next);
  dc(AM.cur, "seek", Math.max(0, A.duration - plan.sec - 2));
}

/* ---------------------------------------------------------------- pannello */
function amRender() {
  const go = $("#amGo"); if (!go) return;
  go.textContent = AM.on ? "FERMA AUTOMIX" : "AVVIA AUTOMIX"; go.classList.toggle("on", AM.on);
  $("#amSkip").disabled = !AM.on;
  $("#amStatus").textContent = AM.on ? AM.msg : AM.msg || "";
  amSyncPreview();
  const cur = amSnap(AM.cur), list = [], SHOW = 40, base = AM.on ? AM.idx : 0;
  const row = (t, k) => `<div class="amq${k === 0 ? " next" : ""}" data-k="${base + k}"><b>${k + 1}.</b><em title="${esc(t.title || t.path)}${t.artist ? " — " + esc(t.artist) : ""}">${esc(t.title || t.path)}${t.artist ? " — " + esc(t.artist) : ""}</em><span>${t.bpm ? Math.round(t.bpm) : "–"} ${esc(t.key || "")}</span>`
    + `<i data-act="top" title="Mettilo per primo">⤒</i><i data-act="up" title="Sposta su">▲</i><i data-act="dn" title="Sposta giù">▼</i><i data-act="del" title="Toglilo dalla coda">✕</i></div>`;
  if (AM.on && cur.loaded) list.push(`<div class="amq now"><em>▶ ${esc(cur.name || "")}</em></div>`);
  const left = AM.queue.length - base;
  if (AM.on) list.push(`<div class="amh">PROSSIMI${AM.opt.loop ? " (poi ricomincia)" : ""} · ${left} in coda</div>`);
  else {
    const rows = (typeof lib !== "undefined" && lib.rows) || [];
    list.push(`<div class="amhelp">AUTOMIX suona i brani della tabella della libreria (playlist, cartella o ricerca) nell'ordine scelto, a partire dal brano che suona o da quello selezionato. Puoi cambiare la coda qui sotto: sposta, togli o metti un brano per primo.</div>`);
    if (!rows.length) list.push(`<div class="amh">Apri una playlist o una cartella per avere dei brani</div>`);
    else list.push(`<div class="amh">${AM.edited ? "CODA MODIFICATA" : "SUONERANNO"} · ${left} brani${AM.edited ? ` <u data-act="reset">ripristina</u>` : ""}</div>`);
  }
  AM.queue.slice(base, base + SHOW).forEach((t, k) => list.push(row(t, k)));
  if (left > SHOW) list.push(`<div class="amh">… altri ${left - SHOW}</div>`);
  const html = list.join("");
  const q = $("#amQueue"); if (q._h !== html) { q._h = html; q.innerHTML = html; }
}

// modifica della coda con i pulsanti sulle righe
function amEdit(act, k) {
  const Q = AM.queue, floor = AM.on ? AM.idx : 0;
  if (act === "reset") { AM.edited = false; AM.sig = ""; return; }
  if (k < floor || k >= Q.length) return;
  if (act === "del") Q.splice(k, 1);
  else if (act === "top") Q.splice(floor, 0, Q.splice(k, 1)[0]);
  else if (act === "up" && k > floor) [Q[k - 1], Q[k]] = [Q[k], Q[k - 1]];
  else if (act === "dn" && k < Q.length - 1) [Q[k + 1], Q[k]] = [Q[k], Q[k + 1]];
  AM.edited = true;
  amRender();
}

function bindAutomix() {
  if (!$("#amGo")) return;
  $("#amGo").onclick = () => (AM.on ? amStop() : amStart());
  $("#amSkip").onclick = amSkip;
  $("#amQueue").addEventListener("click", (e) => { const a = e.target.closest("[data-act]"); if (!a) return; const r = a.closest("[data-k]"); amEdit(a.dataset.act, r ? +r.dataset.k : -1); });
  const bind = (id, key, num) => { const e = $(id); e.value = String(AM.opt[key]); e.onchange = () => { AM.opt[key] = num ? +e.value : e.value; amSave(); }; };
  bind("#amBars", "bars", true); bind("#amStyle", "style"); bind("#amOrder", "order");
  $("#amOrder").addEventListener("change", () => { AM.edited = false; AM.sig = ""; });
  $("#amLoop").checked = !!AM.opt.loop; $("#amLoop").onchange = () => { AM.opt.loop = $("#amLoop").checked; amSave(); };
  setInterval(() => { amTick().catch((e) => { AM.busy = false; amMsg("Errore: " + (e?.message || e)); }); amRampStep(); amRender(); }, 250);
}
bindAutomix();
