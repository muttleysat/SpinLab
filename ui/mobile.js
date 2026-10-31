"use strict";
/* Modelli verticali per telefono (docs/design/ux-mobile.md): A Stack (body.m-stack), C Focus + Platter (body.m-focus),
   D Mappa degli spazi (body.m-spaces). La disposizione e' in style.css; qui schede, cassetto del mixer, anello degli hot cue,
   tela degli spazi e gesti. Sul desktop e nel Cockpit questo file non fa nulla. Usa le funzioni globali di app.js e window.CK (cockpit.js). */
(function () {
  const B = document.body, app = $("#app");
  const is = (c) => B.classList.contains(c);
  const mob = () => is("mob");
  const node = (html) => { const t = document.createElement("template"); t.innerHTML = html.trim(); return t.content.firstChild; };
  const store = {
    get(k, d) { try { const v = localStorage.getItem("spinlab." + k); return v == null ? d : JSON.parse(v); } catch (e) { return d; } },
    set(k, v) { try { localStorage.setItem("spinlab." + k, JSON.stringify(v)); } catch (e) {} },
  };
  const later = () => { setTimeout(resizeCanvases, 30); setTimeout(resizeCanvases, 260); };
  // conferma breve (2,5 s): i messaggi di conferma di app.js restano 12 s, troppo per un gesto
  const note = (m) => { toast2(m); const t = $("#toast2"); clearTimeout(t._h); t._h = setTimeout(() => (t.style.display = "none"), 2500); };
  // doppio tocco fatto a mano: sui telefoni il dblclick non arriva sempre
  const dbl = (test, fn) => {
    let last = null;
    document.addEventListener("pointerup", (e) => {
      const t = mob() && test(e); if (!t) return;
      const now = performance.now();
      if (last && last.t === t && now - last.at < 320 && Math.hypot(e.clientX - last.x, e.clientY - last.y) < 30) { last = null; fn(t, e); }
      else last = { t, at: now, x: e.clientX, y: e.clientY };
    });
  };

  /* ---- schede in basso (A e C), velo e cassetto del mixer */
  const tabs = node(`<div id="mtabs"><i class="grip"></i><nav aria-label="Pannelli">
    <button type="button" data-t="lib">LIBRERIA</button><button type="button" data-t="mix">MIXER</button><button type="button" data-t="fx">FX</button><button type="button" data-t="stems">STEM</button><button type="button" data-t="sampler">SAMPLER</button></nav></div>`);
  const scrim = node(`<div id="mscrim"></div>`);
  app.append(tabs, scrim);
  const pnTab = (t) => $(`.ptabs [data-pn="${t}"]`);
  const setMix = (o) => { B.classList.toggle("mix-open", o); if (o) { CK.setLib(false); CK.setPn(false); } later(); };
  const openPanel = (t) => { B.classList.toggle("pn-fx", t === "fx"); if (t !== "fx") pnTab(t).click(); setMix(false); CK.setPn(true); };
  const panelOn = (t) => is("pn-open") && (t === "fx" ? is("pn-fx") : !is("pn-fx") && pnTab(t)?.classList.contains("on"));
  tabs.addEventListener("click", (e) => {
    const t = e.target.closest("button")?.dataset.t; if (!t) return;
    if (t === "lib") { const o = !is("lib-open"); setMix(false); CK.setPn(false); CK.setLib(o); }
    else if (t === "mix") setMix(!is("mix-open"));
    else if (panelOn(t)) CK.setPn(false); else openPanel(t);
  });
  scrim.addEventListener("click", () => { setMix(false); CK.setPn(false); });
  // maniglia: swipe su apre la libreria; sulla barra di ricerca swipe giu' la richiude
  const vswipe = (el, dir, fn) => {
    let y0 = null;
    el.addEventListener("pointerdown", (e) => { y0 = mob() && !e.target.closest("input,select") ? e.clientY : null; });
    el.addEventListener("pointermove", (e) => { if (y0 != null && (e.clientY - y0) * dir > 28) { y0 = null; fn(); } });
    ["pointerup", "pointercancel"].forEach((ev) => el.addEventListener(ev, () => (y0 = null)));
  };
  vswipe(tabs, -1, () => { setMix(false); CK.setPn(false); CK.setLib(true); });
  vswipe($(".searchbar"), 1, () => CK.setLib(false));
  // bordo sinistro → cassetto del mixer; swipe a sinistra sul cassetto lo chiude (il bordo destro apre il pannello: cockpit.js)
  let ex = null;
  document.addEventListener("pointerdown", (e) => {
    ex = mob() && !is("m-spaces") && (e.clientX < 22 && !is("mix-open") ? { open: true, x: e.clientX } : is("mix-open") && e.target.closest("#mixer") && !e.target.closest(".knob,.fader,select,button") ? { open: false, x: e.clientX } : null) || null;
  }, true);
  document.addEventListener("pointermove", (e) => {
    if (!ex) return;
    if (ex.open && e.clientX - ex.x > 40) { ex = null; setMix(true); } else if (!ex.open && ex.x - e.clientX > 60) { ex = null; setMix(false); }
  });
  ["pointerup", "pointercancel"].forEach((ev) => document.addEventListener(ev, () => (ex = null)));

  /* ---- waveform: due dita su/giu' = zoom, due dita di lato = beat-jump di 4 battiti (A, C, D) */
  const wf = new Map();
  let wg = null, synth = false;
  const mid = () => { const p = [...wf.values()]; return { x: (p[0].cx + p[1].cx) / 2, y: (p[0].cy + p[1].cy) / 2 }; };
  document.addEventListener("pointerdown", (e) => {
    const cv = mob() && e.target.closest?.("canvas.zoom"); if (!cv || synth) return;
    wf.set(e.pointerId, { cx: e.clientX, cy: e.clientY });
    if (wf.size !== 2) return;
    e.stopPropagation(); // non arriva allo scrub del canvas; gli altri gesti (pizzico della mappa) lo vedono ancora
    // il primo dito aveva iniziato uno scrub: lo si chiude come se il dito si fosse alzato
    synth = true; cv.dispatchEvent(new PointerEvent("pointercancel", { pointerId: [...wf.keys()][0], bubbles: true })); synth = false;
    const i = +cv.closest(".deck").dataset.i, m = mid();
    wg = { i, x0: m.x, y0: m.y, z0: decks[i].zoom, mode: null };
  }, true);
  document.addEventListener("pointermove", (e) => {
    const p = wf.get(e.pointerId); if (!p) return;
    p.cx = e.clientX; p.cy = e.clientY;
    if (!wg || wf.size !== 2) return;
    const m = mid(), dx = m.x - wg.x0, dy = m.y - wg.y0;
    if (!wg.mode) wg.mode = Math.abs(dy) > 16 && Math.abs(dy) > Math.abs(dx) ? "zoom" : Math.abs(dx) > 40 ? "jump" : null;
    if (wg.mode === "zoom") decks[wg.i].zoom = Math.min(40, Math.max(1.5, wg.z0 * Math.pow(1.012, dy / uiK)));
    if (wg.mode === "jump") { dc(wg.i, "beatjump", dx < 0 ? 4 : -4); wg.mode = "done"; }
  });
  ["pointerup", "pointercancel"].forEach((ev) => document.addEventListener(ev, (e) => { if (synth) return; wf.delete(e.pointerId); if (wf.size < 2) wg = null; }));

  /* ---- A (e deck della mappa): PLAY tenuto premuto = CUE temporaneo; doppio tocco su un pad = cancella l'hot cue;
     doppio tocco su un lato del crossfader = taglio istantaneo */
  const stackLike = () => is("m-stack") || is("m-spaces");
  let hold = null, noPlay = 0;
  document.addEventListener("pointerdown", (e) => {
    const b = stackLike() && e.target.closest?.(".rbtn.play"); if (!b) return;
    const i = +b.closest(".deck").dataset.i;
    hold = { i, held: false, t: setTimeout(() => { hold.held = true; dc(i, "cue_press"); }, 400) };
  }, true);
  const holdEnd = () => { if (!hold) return; clearTimeout(hold.t); if (hold.held) { dc(hold.i, "cue_release"); noPlay = performance.now(); } hold = null; };
  ["pointerup", "pointercancel"].forEach((ev) => document.addEventListener(ev, holdEnd));
  document.addEventListener("click", (e) => { if (performance.now() - noPlay < 400 && e.target.closest(".rbtn.play")) { e.stopImmediatePropagation(); e.preventDefault(); } }, true);
  dbl((e) => stackLike() && e.target.closest?.("[data-hot]"), (b) => {
    const i = +b.closest(".deck").dataset.i, d = decks[i];
    if (d.mode === "hc0" || d.mode === "hc1") dc(i, "hotcue_clear", +b.dataset.hot + (d.mode === "hc1" ? 4 : 0));
  });
  const xf = $("#crossfader");
  xf.addEventListener("dblclick", (e) => { if (mob()) e.stopImmediatePropagation(); }, true);
  dbl((e) => e.target.closest?.("#crossfader"), (el, e) => {
    const r = el.getBoundingClientRect(), x = (e.clientX - r.left) / r.width;
    el._apply(x < 0.35 ? 0 : x > 0.65 ? 1 : 0.5);
  });

  /* ---- C · Focus + Platter */
  let focus = store.get("focus", 0);
  const applyFocus = () => {
    const on = is("m-focus");
    DK.forEach((D, i) => { D.el.classList.toggle("foc", on && i === focus); D.el.classList.toggle("peek", on && i !== focus); });
    later();
  };
  const swapFocus = () => { focus = 1 - focus; store.set("focus", focus); applyFocus(); };
  // striscia dell'altro deck: tocco o swipe verticale = cambia deck; tenuta premuta = SYNC del deck in primo piano
  let pk = null;
  document.addEventListener("pointerdown", (e) => {
    const s = is("m-focus") && e.target.closest?.(".deck.peek"); if (!s || pk) return;
    pk = { y: e.clientY, long: false, t: setTimeout(() => { pk.long = true; const d = decks[focus]; dc(focus, d.snap?.sync ? "sync_off" : "sync"); note(`SYNC deck ${focus + 1}: ${d.snap?.sync ? "spento" : "acceso"}`); }, 550) };
  });
  document.addEventListener("pointerup", (e) => { if (!pk) return; clearTimeout(pk.t); if (!pk.long) swapFocus(); pk = null; });
  document.addEventListener("pointercancel", () => { if (pk) { clearTimeout(pk.t); pk = null; } });
  // anello degli 8 hot cue intorno al piatto, con l'arco del loop
  const RING = ["#ff3b4e", "#ffc21a", "#35e08a", "#b06cff", "#18b6ff", "#ff7a1a", "#ff6bd0", "#7ee0ff"];
  DK.forEach((D, i) => {
    const plat = $(".platter", D.el);
    const ring = node(`<div class="cring"><i></i>${RING.map((c, k) => `<button type="button" data-cue="${k}" style="--hc:${c}" aria-label="Hot cue ${k + 1}">${k + 1}</button>`).join("")}</div>`);
    plat.appendChild(ring);
    D.ring = ring;
    const place = () => {
      const w = plat.clientWidth, h = plat.clientHeight, m = Math.min(w, h), R = m * 0.44;
      plat.style.setProperty("--pm", m + "px");
      requestAnimationFrame(resizeCanvases);
      $$("button", ring).forEach((b, k) => { const a = ((k * 45 - 90 + 22.5) * Math.PI) / 180; b.style.left = w / 2 + Math.cos(a) * R + "px"; b.style.top = h / 2 + Math.sin(a) * R + "px"; });
      const arc = $("i", ring); Object.assign(arc.style, { left: (w - m) / 2 + "px", top: (h - m) / 2 + "px", width: m + "px", height: m + "px" });
    };
    new ResizeObserver(place).observe(plat);
    ring.addEventListener("pointerdown", (e) => { const b = e.target.closest("[data-cue]"); if (b && e.button === 0) dc(i, "hotcue", +b.dataset.cue); });
    ring.addEventListener("contextmenu", (e) => { const b = e.target.closest("[data-cue]"); if (!b) return; e.preventDefault(); const k = +b.dataset.cue; menu(e, [[`Elimina hot cue ${k + 1}`, () => dc(i, "hotcue_clear", k)]]); });
  });
  setInterval(() => {
    if (!is("m-focus")) return;
    DK.forEach((D, i) => { const hc = decks[i].snap?.hot_cues || []; $$("[data-cue]", D.ring).forEach((b, k) => b.classList.toggle("set", hc[k] != null)); });
  }, 200);
  const polar = (el, e) => { const r = el.getBoundingClientRect(), dx = e.clientX - r.left - r.width / 2, dy = e.clientY - r.top - r.height / 2; return { a: Math.atan2(dy, dx), rr: Math.hypot(dx, dy) / (Math.min(r.width, r.height) / 2) }; };
  // arco disegnato sulla corona fuori dal piatto = loop (piu' ampio l'arco, piu' lungo il loop); swipe sull'etichetta = beat-jump ±4
  let arc = null, flick = null;
  document.addEventListener("pointerdown", (e) => {
    if (!is("m-focus") || e.target.closest?.("[data-cue]")) return;
    const deck = e.target.closest?.(".deck.foc"); if (!deck) return;
    const i = +deck.dataset.i, plat = $(".platter", deck);
    if (!e.target.closest?.(".platter")) return;
    const p = polar(plat, e); // la corona si misura dal centro del piatto, anche se il dito cade su un angolo del canvas
    if (p.rr >= 0.78 && p.rr <= 1.05) {
      e.stopImmediatePropagation();
      arc = { i, plat, last: p.a, a0: p.a, sum: 0, fx: $(".cring i", deck) };
      plat.setPointerCapture(e.pointerId);
    } else if (e.target.closest?.("canvas.plat")) {
      const p = polar(e.target, e); if (p.rr > 0.42) return; // fuori dall'etichetta: scratch e pitch bend come sempre
      e.stopImmediatePropagation();
      flick = { i, x: e.clientX, y: e.clientY, t: performance.now() };
    }
  }, true);
  document.addEventListener("pointermove", (e) => {
    if (!arc) return;
    const a = polar(arc.plat, e).a; let da = a - arc.last; if (da > Math.PI) da -= 2 * Math.PI; if (da < -Math.PI) da += 2 * Math.PI;
    arc.last = a; arc.sum += da;
    const sw = Math.min(360, Math.abs(arc.sum) * 180 / Math.PI), start = (arc.sum < 0 ? arc.last : arc.a0) * 180 / Math.PI + 90;
    arc.fx.style.setProperty("--a0", start + "deg"); arc.fx.style.setProperty("--sw", sw); arc.fx.classList.add("on");
  });
  document.addEventListener("pointerup", (e) => {
    if (arc) {
      const deg = Math.abs(arc.sum) * 180 / Math.PI, i = arc.i;
      setTimeout(((fx) => () => fx.classList.remove("on"))(arc.fx), 250);
      arc = null;
      if (deg >= 25) {
        const beats = deg < 60 ? 1 : deg < 120 ? 2 : deg < 200 ? 4 : deg < 300 ? 8 : 16;
        dc(i, "loop_size", beats); if (!decks[i].snap?.loop_on) dc(i, "loop_toggle");
        note(`Loop di ${beats} ${beats === 1 ? "battuta" : "battute"} sul deck ${i + 1}`);
      }
    }
    if (flick) {
      const dx = e.clientX - flick.x, dy = e.clientY - flick.y;
      if (Math.abs(dx) > 50 && Math.abs(dx) > 2 * Math.abs(dy) && performance.now() - flick.t < 500) dc(flick.i, "beatjump", dx < 0 ? 4 : -4);
      flick = null;
    }
  });
  document.addEventListener("pointercancel", () => { if (arc) arc.fx.classList.remove("on"); arc = flick = null; });

  /* ---- D · Mappa degli spazi */
  const SP = [["deck", "DECK"], ["mixer", "MIXER"], ["library", "LIBRERIA"], ["fx", "FX"], ["stem", "STEM"], ["sampler", "SAMPLER"]];
  const NAME = Object.fromEntries(SP);
  const ICON = {
    deck: '<path d="M8 30h144M8 74h144" stroke-opacity=".25"/><path d="M12 30v-6m8 6v-14m8 14v-9m8 9v-18m8 18v-7m8 7v-12m8 12v-5m8 5v-15m8 15v-8m8 8v-11m8 11v-6m8 6v-13m8 13v-9m8 9v-4m8 4v-10m8 10v-7m8 7v-12m8 12v-6M12 74v-9m8 9v-5m8 5v-14m8 14v-8m8 8v-11m8 11v-6m8 6v-16m8 16v-7m8 7v-10m8 10v-5m8 5v-12m8 12v-8m8 8v-14m8 14v-6m8 6v-9m8 9v-4m8 4v-11m8 11v-7"/><path d="M80 6v74" stroke="#fff"/>',
    mixer: '<path d="M30 12v68M52 12v68" stroke-opacity=".3" stroke-width="8" stroke-linecap="round"/><path d="M30 80V40M52 80V52" stroke-width="8" stroke-linecap="round"/><circle cx="96" cy="22" r="11"/><circle cx="96" cy="50" r="11"/><circle cx="96" cy="78" r="11"/><circle cx="128" cy="22" r="11"/><circle cx="128" cy="50" r="11"/><circle cx="128" cy="78" r="11"/>',
    library: '<path d="M10 14h100M10 34h120M10 54h90M10 74h110" stroke-width="6" stroke-linecap="round" stroke-opacity=".7"/><path d="M134 14h16M134 54h16M134 74h16" stroke-width="6" stroke-linecap="round" stroke-opacity=".35"/>',
    fx: '<circle cx="46" cy="26" r="16"/><circle cx="110" cy="26" r="16"/><circle cx="46" cy="70" r="16"/><circle cx="110" cy="70" r="16"/><path d="M46 26l8-10M110 26l-10-8M46 70l12 2M110 70l6 12"/>',
    stem: '<path d="M10 14h130M10 38h100M10 62h116M10 86h76" stroke-width="12" stroke-linecap="round"/>',
    sampler: '<rect x="10" y="8" width="40" height="36" rx="8"/><rect x="60" y="8" width="40" height="36" rx="8"/><rect x="110" y="8" width="40" height="36" rx="8"/><rect x="10" y="54" width="40" height="36" rx="8"/><rect x="60" y="54" width="40" height="36" rx="8"/><rect x="110" y="54" width="40" height="36" rx="8"/>',
  };
  const TINT = { deck: "#18b6ff", mixer: "#ff7a1a", library: "#9aa6c2", fx: "#b06cff", stem: "#35e08a", sampler: "#ffc21a" };
  let st = store.get("spaces", { ov: true, a: "deck", b: null });
  let scenes = store.get("scenes", [{ name: "Warm-up", a: "library", b: "deck" }, { name: "Peak", a: "deck", b: "mixer" }, { name: "Chill", a: "deck", b: "fx" }]);
  const RP = new Set(["stem", "sampler"]); // pannelli dello stesso cassetto: non stanno insieme in split
  const fits = (a, b) => b && a !== b && !(RP.has(a) && RP.has(b));
  const view = node(`<section id="spaces" aria-label="Mappa degli spazi">
    <div class="scenes"><span>SCENE</span><div class="chips" style="display:flex;gap:8px"></div><button type="button" class="add" aria-label="Nuova scena dalla disposizione attuale">+</button></div>
    <div class="tiles">${SP.map(([k, n]) => `<button type="button" class="tile" data-s="${k}" style="--tc:${TINT[k]}"><span class="th">${n}<i data-st="${k}"></i></span><svg viewBox="0 0 160 92" fill="none" stroke="currentColor" stroke-width="3" aria-hidden="true">${ICON[k]}</svg></button>`).join("")}</div>
    <div class="drop">TRASCINA QUI UN RIQUADRO · SPLIT 50/50</div>
    <div class="mini"><div><span>SPAZI</span><span>3 DITA: ↑ PANORAMICA · ←/→ CAMBIA</span></div><nav>${SP.map(([k, n]) => `<button type="button" data-m="${k}">${n}</button>`).join("")}</nav></div>
  </section>`);
  const title = node(`<button type="button" id="spTitle" title="Panoramica degli spazi"><svg viewBox="0 0 24 24" width="14" height="14" aria-hidden="true"><path d="M4 4h7v7H4zM13 4h7v7h-7zM4 13h7v7H4zM13 13h7v7h-7z" fill="none" stroke="currentColor" stroke-width="2"/></svg><b></b></button>`);
  const bubble = node(`<div id="bubble" aria-label="Crossfader fluttuante"><span>XFADE</span><div class="fader h"></div></div>`);
  app.append(view, bubble);
  $(".topr").insertBefore(title, $("#uiBtn"));
  const save = () => store.set("spaces", st);
  const applySpaces = () => {
    SP.forEach(([k]) => B.classList.remove("sp-" + k));
    B.classList.remove("sp-ov", "sp-split");
    if (is("m-spaces")) {
      if (st.ov) B.classList.add("sp-ov");
      else { B.classList.add("sp-" + st.a); if (st.b) B.classList.add("sp-" + st.b, "sp-split"); }
      const p = [st.a, st.b].find((k) => RP.has(k)); if (!st.ov && p) pnTab(p === "stem" ? "stems" : "sampler")?.click();
    }
    $("b", title).textContent = st.ov ? "PANORAMICA" : [st.a, st.b].filter(Boolean).map((k) => NAME[k]).join(" + ");
    $$(".tile", view).forEach((t) => t.classList.toggle("on", t.dataset.s === st.a || t.dataset.s === st.b));
    $$(".mini button", view).forEach((t) => t.classList.toggle("on", t.dataset.m === st.a || t.dataset.m === st.b));
    bubble.classList.toggle("on", is("m-spaces") && !!st.bubble);
    Object.assign(bubble.style, { left: (st.bx ?? 240) + "px", top: (st.by ?? 440) + "px" });
    renderScenes(); later();
  };
  const go = (a, b = null) => { st = { ...st, ov: false, a, b: fits(a, b) ? b : null }; save(); applySpaces(); };
  const overview = () => { st.ov = true; save(); applySpaces(); };
  title.onclick = () => (st.ov ? go(st.a, st.b) : overview());
  $$(".mini button", view).forEach((b) => (b.onclick = () => go(b.dataset.m)));
  // scene: tocco = richiama; tenuta = aggiorna o elimina; + = nuova dalla disposizione attuale
  function renderScenes() {
    const box = $(".chips", view);
    box.innerHTML = scenes.map((s, k) => `<button type="button" data-k="${k}" class="${!st.ov && s.a === st.a && (s.b || null) === (st.b || null) ? "on" : ""}">${esc(s.name)}</button>`).join("");
  }
  const addScene = (a, b) => { scenes.push({ name: `Scena ${scenes.length + 1}`, a, b: b || null }); store.set("scenes", scenes); renderScenes(); note(`Scena salvata: ${NAME[a]}${b ? " + " + NAME[b] : ""}`); };
  $(".add", view).onclick = () => addScene(st.a, st.b);
  let chipT = null, chipLong = false;
  $(".chips", view).addEventListener("pointerdown", (e) => {
    const c = e.target.closest("[data-k]"); if (!c) return;
    chipLong = false;
    chipT = setTimeout(() => {
      chipLong = true; const k = +c.dataset.k;
      menu(e, [[`Aggiorna «${scenes[k].name}» con ${NAME[st.a]}${st.b ? " + " + NAME[st.b] : ""}`, () => { scenes[k] = { ...scenes[k], a: st.a, b: st.b }; store.set("scenes", scenes); renderScenes(); }],
        [`Elimina «${scenes[k].name}»`, () => { scenes.splice(k, 1); store.set("scenes", scenes); renderScenes(); }]]);
    }, 550);
  });
  $(".chips", view).addEventListener("pointerup", (e) => { clearTimeout(chipT); const c = e.target.closest("[data-k]"); if (c && !chipLong) { const s = scenes[+c.dataset.k]; go(s.a, s.b); } });
  // riquadri: tocco = entra; trascinato sulla zona in basso = split con lo spazio attuale; tenuto premuto (MIXER) = crossfader in bolla;
  // cerchio disegnato col dito su un riquadro = salva come scena
  const drop = $(".drop", view);
  let td = null;
  view.addEventListener("pointerdown", (e) => {
    const t = e.target.closest(".tile"); if (!t || td || e.pointerType === "mouse" && e.button !== 0) return; // solo il primo dito
    t.setPointerCapture(e.pointerId);
    td = { t, s: t.dataset.s, x: e.clientX, y: e.clientY, pts: [[e.clientX, e.clientY]], drag: false, long: false,
      timer: setTimeout(() => { td.long = true; if (td.s === "mixer") { st.bubble = !st.bubble; save(); applySpaces(); note(st.bubble ? "Crossfader staccato in una bolla" : "Bolla chiusa"); } }, 550) };
  });
  view.addEventListener("pointermove", (e) => {
    if (!td) return;
    const dx = e.clientX - td.x, dy = e.clientY - td.y;
    td.pts.push([e.clientX, e.clientY]);
    if (!td.drag && Math.hypot(dx, dy) > 12) { td.drag = true; clearTimeout(td.timer); td.t.classList.add("drag"); }
    if (!td.drag) return;
    td.t.style.transform = `translate(${dx / uiK}px, ${dy / uiK}px) rotate(-3deg)`;
    const r = drop.getBoundingClientRect(); drop.classList.toggle("hot", e.clientY > r.top - 20 && e.clientY < r.bottom + 20);
  });
  const circle = (pts) => {
    if (pts.length < 12) return false;
    const cx = pts.reduce((s, p) => s + p[0], 0) / pts.length, cy = pts.reduce((s, p) => s + p[1], 0) / pts.length;
    let sum = 0, len = 0;
    for (let k = 1; k < pts.length; k++) {
      let da = Math.atan2(pts[k][1] - cy, pts[k][0] - cx) - Math.atan2(pts[k - 1][1] - cy, pts[k - 1][0] - cx);
      if (da > Math.PI) da -= 2 * Math.PI; if (da < -Math.PI) da += 2 * Math.PI;
      sum += da; len += Math.hypot(pts[k][0] - pts[k - 1][0], pts[k][1] - pts[k - 1][1]);
    }
    const [x0, y0] = pts[0], [x1, y1] = pts[pts.length - 1];
    return Math.abs(sum) > 5.2 && len > 160 * uiK && Math.hypot(x1 - x0, y1 - y0) < len / 3;
  };
  const tileEnd = (e, cancel) => {
    if (!td) return;
    clearTimeout(td.timer);
    const { t, s, drag, long, pts } = td; td = null;
    t.classList.remove("drag"); t.style.transform = ""; const hot = drop.classList.contains("hot"); drop.classList.remove("hot");
    if (cancel || long) return;
    if (drag && circle(pts)) addScene(s, null);
    else if (drag && hot) { if (s !== st.a) go(st.a, s); }
    else if (!drag) go(s);
  };
  view.addEventListener("pointerup", (e) => tileEnd(e, false));
  view.addEventListener("pointercancel", (e) => tileEnd(e, true));
  // stato dei riquadri: brani, effetti accesi, campioni caricati
  setInterval(() => {
    if (!is("m-spaces") || !st.ov) return;
    const S = (k, v) => { const el = $(`[data-st="${k}"]`, view); if (el.textContent !== v) el.textContent = v; };
    S("deck", decks.slice(0, 2).some((d) => d.snap?.playing) ? "● in riproduzione" : "fermo");
    S("mixer", st.bubble ? "bolla attiva" : "2 canali");
    S("library", typeof lib !== "undefined" ? `${lib.rows.length} brani` : "");
    S("fx", `${decks.slice(0, 2).filter((d) => d.fx?.on).length} attivi`);
    S("stem", $("#stemStatus")?.textContent.trim() || "4 tracce");
    S("sampler", typeof padState !== "undefined" ? `${Object.values(padState).filter((p) => p?.name).length} di 8 slot` : "8 slot");
  }, 500);
  // pizzico: dita che si chiudono in uno spazio = panoramica; dita che si aprono su un riquadro = entra.
  // tre dita ←/→ = spazio precedente o successivo; tre dita ↑ = panoramica
  const tp = new Map();
  let g3 = null, g2 = null;
  const free = (e) => !e.target.closest?.("canvas.plat,canvas.over,.knob,.fader,input,select,#bubble");
  document.addEventListener("pointerdown", (e) => {
    if (!is("m-spaces") || e.pointerType === "mouse") return;
    tp.set(e.pointerId, { x: e.clientX, y: e.clientY, cx: e.clientX, cy: e.clientY, free: free(e) });
    const p = [...tp.values()];
    if (tp.size === 3) { g3 = { x: p.reduce((s, q) => s + q.cx, 0) / 3, y: p.reduce((s, q) => s + q.cy, 0) / 3, done: false }; g2 = null; if (td) tileEnd(e, true); }
    else if (tp.size === 2 && p.every((q) => q.free)) g2 = { d0: Math.hypot(p[0].cx - p[1].cx, p[0].cy - p[1].cy), done: false };
  }, true);
  document.addEventListener("pointermove", (e) => {
    const q = tp.get(e.pointerId); if (!q) return;
    q.cx = e.clientX; q.cy = e.clientY;
    const p = [...tp.values()];
    if (g3 && !g3.done && tp.size === 3) {
      const dx = p.reduce((s, r) => s + r.cx, 0) / 3 - g3.x, dy = p.reduce((s, r) => s + r.cy, 0) / 3 - g3.y;
      if (dy < -60 && Math.abs(dy) > Math.abs(dx)) { g3.done = true; overview(); }
      else if (Math.abs(dx) > 60) { g3.done = true; const k = SP.findIndex(([s]) => s === st.a), n = (k + (dx < 0 ? 1 : -1) + SP.length) % SP.length; go(SP[n][0]); }
    } else if (g2 && !g2.done && tp.size === 2) {
      const d = Math.hypot(p[0].cx - p[1].cx, p[0].cy - p[1].cy);
      if (!st.ov && d < g2.d0 * 0.65) { g2.done = true; overview(); }
      else if (st.ov && d > g2.d0 * 1.4) {
        g2.done = true; if (td) tileEnd(e, true);
        const t = document.elementFromPoint((p[0].cx + p[1].cx) / 2, (p[0].cy + p[1].cy) / 2)?.closest(".tile"); if (t) go(t.dataset.s);
      }
    }
  });
  ["pointerup", "pointercancel"].forEach((ev) => document.addEventListener(ev, (e) => { if (synth) return; tp.delete(e.pointerId); if (tp.size < 3) g3 = null; if (tp.size < 2) g2 = null; }));
  // bolla: si trascina dal bordo, il cursore dentro comanda il crossfader; doppio tocco sul bordo la chiude
  const bf = $(".fader", bubble);
  makeFader(bf, { value: 0.5, def: 0.5, onChange: (v) => { inv("mixer_cmd", { cmd: "crossfader", a: v }); xf._set(v); } });
  setInterval(() => { if (bubble.classList.contains("on") && !bf._dragging()) bf._set(+getComputedStyle(xf).getPropertyValue("--p") || 0.5); }, 200);
  let bd = null;
  bubble.addEventListener("pointerdown", (e) => { if (e.target.closest(".fader")) return; bubble.setPointerCapture(e.pointerId); bd = { x: e.clientX, y: e.clientY, l: st.bx ?? 240, t: st.by ?? 440 }; });
  bubble.addEventListener("pointermove", (e) => {
    if (!bd) return;
    st.bx = Math.max(0, bd.l + (e.clientX - bd.x) / uiK); st.by = Math.max(48, bd.t + (e.clientY - bd.y) / uiK);
    Object.assign(bubble.style, { left: st.bx + "px", top: st.by + "px" });
  });
  bubble.addEventListener("pointerup", () => { if (bd) { bd = null; save(); } });
  dbl((e) => !e.target.closest?.("#bubble .fader") && e.target.closest?.("#bubble"), () => { st.bubble = false; save(); applySpaces(); });

  /* ---- Indietro (cockpit.js): chiude il cassetto del mixer, poi dallo spazio torna alla panoramica */
  CK.back = { open: () => is("mix-open") || (is("m-spaces") && !st.ov), close: () => (is("mix-open") ? setMix(false) : overview()) };

  /* ---- cambio di modello: rimette a posto deck, spazi e schede */
  let lastMode = "";
  const syncTabs = () => $$("button", tabs).forEach((b) => { const t = b.dataset.t; b.classList.toggle("on", t === "lib" ? is("lib-open") : t === "mix" ? is("mix-open") : panelOn(t)); });
  new MutationObserver(() => {
    const mode = ["m-stack", "m-focus", "m-spaces"].find(is) || "";
    if (mode !== lastMode) {
      lastMode = mode;
      if (!mode) B.classList.remove("mix-open", "pn-fx");
      applyFocus(); applySpaces();
    }
    if (!is("pn-open") && is("pn-fx")) B.classList.remove("pn-fx");
    syncTabs();
  }).observe(B, { attributes: true, attributeFilter: ["class"] });
  applyFocus(); applySpaces();
})();
