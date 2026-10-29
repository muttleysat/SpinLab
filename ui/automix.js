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
  if (first != null) { const k = list.findIndex((t) => t.id === first); if (k > 0) list = [...list.slice(k), ...list.slice(0, k)]; }
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
function amStart() {
  const rows = (typeof lib !== "undefined" && lib.rows) || [];
  if (!rows.length) { toast("AUTOMIX: scegli prima una cartella o una playlist con dei brani"); return; }
  const playing = [0, 1].find((i) => amSnap(i).playing && amSnap(i).loaded);
  AM.queue = amOrderTracks(rows, playing == null ? lib.last : null);
  if (playing != null) { const pth = amSnap(playing).path; AM.queue = AM.queue.filter((t) => t.path !== pth); }
  AM.idx = 0; AM.mix = null; AM.ramp = null; AM.on = true; AM.busy = false;
  AM.cur = playing ?? [0, 1].find((i) => !amSnap(i).loaded) ?? 0; AM.nxt = 1 - AM.cur;
  amMsg(playing != null ? "In corso: il prossimo brano parte alla fine di quello che suona" : "Avvio…");
  amRender();
}
function amStop(why) {
  AM.on = false; AM.busy = false; AM.mix = null; AM.ramp = null;
  amMsg(why || "Fermato. I deck restano come sono.");
  amRender();
}
function amMsg(t) { AM.msg = t; }
const amNext = () => {
  if (AM.idx >= AM.queue.length && AM.opt.loop && AM.queue.length) AM.idx = 0;
  return AM.queue[AM.idx] || null;
};

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
  const next = amNext();
  if (!next) { if (A.duration - A.pos < 1.5 || !A.playing) return amStop("Automix terminato: la coda e' finita."); amMsg("Ultimo brano: la coda e' finita"); return; }
  const remaining = A.duration - A.pos, plan = amMixLen(A, next);
  const lead = Math.max(75, plan.sec + 45);
  if (remaining <= lead && !amReady(AM.nxt, next.path) && !decks[AM.nxt].loading) return amPreload(next);
  if (remaining <= plan.sec + 0.05 && amReady(AM.nxt, next.path)) return amStartMix(next, plan);
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
    const A = amSnap(AM.cur);
    AM.mix = { t0: A.pos, sec: plan.sec, from: AM.cur, to: AM.nxt, path: next.path, started: performance.now() };
    if (plan.sync) await dc(AM.nxt, "sync");               // tempo e battiti del nuovo brano sul deck che suona
    await dc(AM.nxt, "play");
    AM.idx++;
    amMsg(`Mix in corso verso «${next.title || next.path}»`);
  } finally { AM.busy = false; amRender(); }
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
  $("#amStatus").textContent = AM.on ? AM.msg : AM.msg || "Suona da solo i brani della libreria visibile, uno dopo l'altro, con transizioni a tempo.";
  const cur = amSnap(AM.cur), list = [];
  if (AM.on && cur.loaded) list.push(`<div class="amq now">▶ ${esc(cur.name || "")}</div>`);
  const upcoming = AM.queue.slice(AM.idx, AM.idx + 1);                    // il pannello e' basso: solo il brano che suona e il prossimo; la coda intera e' nel suggerimento
  if (AM.on) upcoming.forEach((t, k) => list.push(`<div class="amq${k === 0 ? " next" : ""}">${esc(t.title || t.path)}${t.artist ? " — " + esc(t.artist) : ""}<span>${t.bpm ? Math.round(t.bpm) : ""} ${esc(t.key || "")}</span></div>`));
  const html = list.join("") || (AM.on ? "" : "");
  const q = $("#amQueue"); if (q._h !== html) { q._h = html; q.innerHTML = html; }
  q.title = AM.on ? AM.queue.slice(AM.idx, AM.idx + 12).map((t, k) => `${k + 1}. ${t.title || t.path}`).join("\n") : "";
}

function bindAutomix() {
  if (!$("#amGo")) return;
  $("#amGo").onclick = () => (AM.on ? amStop() : amStart());
  $("#amSkip").onclick = amSkip;
  const bind = (id, key, num) => { const e = $(id); e.value = String(AM.opt[key]); e.onchange = () => { AM.opt[key] = num ? +e.value : e.value; amSave(); }; };
  bind("#amBars", "bars", true); bind("#amStyle", "style"); bind("#amOrder", "order");
  $("#amLoop").checked = !!AM.opt.loop; $("#amLoop").onchange = () => { AM.opt.loop = $("#amLoop").checked; amSave(); };
  setInterval(() => { amTick().catch((e) => { AM.busy = false; amMsg("Errore: " + (e?.message || e)); }); amRampStep(); amRender(); }, 250);
}
bindAutomix();
