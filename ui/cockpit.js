"use strict";
/* Gesture del layout Cockpit (telefono Android in orizzontale). Specifiche: docs/specifiche-android-cockpit.md.
   Tutto agisce solo con body.cockpit; sul desktop questo file non fa nulla. Usa le funzioni globali di app.js. */
(function () {
  const B = document.body;
  const on = () => B.classList.contains("cockpit");

  /* ---- libreria a tendina (G1, G2): maniglia, swipe giu' dalla barra, swipe su dalla barra di ricerca */
  const top = $("#top"), handle = document.createElement("button");
  handle.className = "lib-handle"; handle.type = "button"; handle.setAttribute("aria-label", "Libreria");
  top.insertBefore(handle, top.querySelector(".topr"));
  const setLib = (open) => {
    B.classList.toggle("lib-open", open);
    handle.textContent = open ? "LIBRERIA ▴" : "LIBRERIA ▾";
    setTimeout(resizeCanvases, 260);
  };
  setLib(false);
  setTimeout(() => B.classList.add("anim"), 600);
  let swiped = 0;
  handle.addEventListener("click", () => { if (performance.now() - swiped > 400) setLib(!B.classList.contains("lib-open")); });
  const swipe = (el, dir, skip) => {
    let y0 = null;
    el.addEventListener("pointerdown", (e) => { y0 = on() && !e.target.closest(skip || "x-none") ? e.clientY : null; });
    el.addEventListener("pointermove", (e) => {
      if (y0 == null) return;
      if ((e.clientY - y0) * dir > 28) { y0 = null; swiped = performance.now(); setLib(dir > 0); }
    });
    ["pointerup", "pointercancel"].forEach((ev) => el.addEventListener(ev, () => (y0 = null)));
  };
  swipe(top, 1, ".knob,.fader");
  swipe($(".searchbar"), -1, "input,button");
  handle.style.touchAction = "none";

  /* ---- cassetto del pannello laterale (AUTOMIX, SAMPLER, DRUMS, STEMS): pulsante nella barra, swipe verso sinistra dal bordo destro, swipe verso destra per chiuderlo */
  const ph = document.createElement("button");
  ph.className = "pn-handle"; ph.type = "button"; ph.textContent = "PANNELLO";
  top.insertBefore(ph, top.querySelector(".topr"));
  const setPn = (open) => { B.classList.toggle("pn-open", open); if (open) setLib(false); };
  ph.addEventListener("click", () => setPn(!B.classList.contains("pn-open")));
  let x0 = null;
  const hswipe = (el, edge) => {
    el.addEventListener("pointerdown", (e) => { x0 = on() && (edge ? e.clientX > innerWidth - 22 : !e.target.closest("input,select,.fader,.knob")) ? e.clientX : null; });
    el.addEventListener("pointermove", (e) => {
      if (x0 == null) return;
      if (edge ? x0 - e.clientX > 40 : e.clientX - x0 > 60) { x0 = null; setPn(edge); }
    });
    ["pointerup", "pointercancel"].forEach((ev) => el.addEventListener(ev, () => (x0 = null)));
  };
  hswipe(document, true);
  hswipe($(".ptabs"), false);

  /* ---- menu del deck (tocco sul numero): griglia, tonalita', quantize, portata del pitch, carica, espelli (i pulsanti relativi sono nascosti nel Cockpit) */
  document.addEventListener("click", (e) => {
    const art = on() && e.target.closest(".art");
    if (!art) return;
    const i = +art.closest(".deck").dataset.i, D = DK[i], d = decks[i];
    menu(e, [
      ["Griglia…", () => openPop(i, "grid", art)], ["Tonalità…", () => openPop(i, "key", art)],
      [d.snap?.quantize ? "Quantize: spegni" : "Quantize: accendi", () => dc(i, "quantize", d.snap?.quantize ? 0 : 1)],
      [`Portata pitch (±${Math.round(d.snap?.range_pct ?? 8)}%): cambia`, () => D.range.click()],
      "-", ["Carica il brano selezionato", () => D.r("load").click()], ["Espelli", () => D.eject.click()],
    ]);
  });

  /* ---- pitch bend (G9): trascinando lungo il bordo esterno del piatto in senso orario si accelera, in senso antiorario si frena; al rilascio torna normale */
  let bend = null;
  document.addEventListener("pointerdown", (e) => {
    const cv = on() && e.target.closest?.("canvas.plat");
    if (!cv) return;
    const b = cv.getBoundingClientRect(), cx = b.left + b.width / 2, cy = b.top + b.height / 2, R = Math.min(b.width, b.height) / 2;
    if (Math.hypot(e.clientX - cx, e.clientY - cy) < R * 0.82) return; // dentro: scrub come sempre
    e.stopImmediatePropagation();
    cv.setPointerCapture(e.pointerId);
    bend = { i: +cv.closest(".deck").dataset.i, cx, cy, a: Math.atan2(e.clientY - cy, e.clientX - cx), tot: 0 };
  }, true);
  document.addEventListener("pointermove", (e) => {
    if (!bend) return;
    const a = Math.atan2(e.clientY - bend.cy, e.clientX - bend.cx);
    let da = a - bend.a; if (da > Math.PI) da -= 2 * Math.PI; if (da < -Math.PI) da += 2 * Math.PI;
    bend.a = a; bend.tot = Math.max(-0.6, Math.min(0.6, bend.tot + da));
    dc(bend.i, "nudge", Math.max(-0.5, Math.min(0.5, bend.tot * 0.8)));
  });
  const bendEnd = () => { if (bend) { dc(bend.i, "nudge", 0); bend = null; } };
  document.addEventListener("pointerup", bendEnd); document.addEventListener("pointercancel", bendEnd);

  /* ---- prototipo B: riga SYNC + LOOP sotto i pad. Inoltra ai pulsanti di sempre (nascosti) e ne copia lo stato acceso */
  DK.forEach((D) => {
    const row = document.createElement("div");
    row.className = "ck-row";
    row.innerHTML = `<button type="button">SYNC</button><button type="button">LOOP</button>`;
    const [bs, bl] = row.children;
    bs.onclick = () => D.sync.click(); bl.onclick = () => D.lsz.click();
    D.el.querySelector(".lcol").appendChild(row);
    const mirror = () => { bs.classList.toggle("on", D.sync.classList.contains("on")); bl.classList.toggle("on", D.lsz.classList.contains("on")); };
    [D.sync, D.lsz].forEach((b) => new MutationObserver(mirror).observe(b, { attributes: true, attributeFilter: ["class"] }));
  });

  /* ---- menu radiale dei pad (G3): tieni premuto il chip della modalita' o trascinalo; il tocco breve apre il menu di sempre (G4) */
  const rad = document.createElement("div");
  rad.id = "radial";
  const CH = [["hc0", "HOT CUE"], ["jump", "BEAT JUMP"], ["roll", "LOOP ROLL"], ["brake", "FRENO / REV"]]; // su, destra, giu', sinistra
  rad.innerHTML = CH.map((c) => `<b>${c[1]}</b>`).join("");
  document.body.appendChild(rad);
  let r = null, noClick = 0;
  const pick = (dx, dy) => (Math.hypot(dx, dy) < 26 ? -1 : Math.abs(dy) >= Math.abs(dx) ? (dy < 0 ? 0 : 2) : dx > 0 ? 1 : 3);
  const radOpen = () => {
    if (!r || r.open) return;
    r.open = true;
    rad.style.left = Math.min(innerWidth - 100, Math.max(100, r.x0)) + "px";
    rad.style.top = Math.min(innerHeight - 100, Math.max(100, r.y0)) + "px";
    rad.classList.add("show");
  };
  const radEnd = (commit) => {
    if (!r) return;
    clearTimeout(r.timer);
    if (r.open) {
      noClick = performance.now();
      if (commit && r.sel >= 0) { const d = decks[r.i]; d.mode = CH[r.sel][0]; closeRolls(r.i); renderPads(r.i); }
      rad.classList.remove("show");
    }
    r = null;
  };
  document.addEventListener("pointerdown", (e) => {
    const chip = on() && e.target.closest(".modebtn");
    if (!chip) return;
    const i = +chip.closest(".deck").dataset.i;
    chip.setPointerCapture(e.pointerId);
    r = { i, x0: e.clientX, y0: e.clientY, open: false, sel: -1, timer: setTimeout(radOpen, 300) };
  }, true);
  document.addEventListener("pointermove", (e) => {
    if (!r) return;
    const dx = e.clientX - r.x0, dy = e.clientY - r.y0;
    if (!r.open && Math.hypot(dx, dy) > 14) radOpen();
    if (!r.open) return;
    r.sel = pick(dx, dy);
    [...rad.children].forEach((b, k) => b.classList.toggle("on", k === r.sel));
  });
  document.addEventListener("pointerup", () => radEnd(true));
  document.addEventListener("pointercancel", () => radEnd(false));
  document.addEventListener("click", (e) => {
    if (performance.now() - noClick < 350 && e.target.closest(".modebtn")) { e.stopImmediatePropagation(); e.preventDefault(); }
  }, true);

  /* ---- mixer: pizzico (G5) e tap con due dita su una manopola (G6) */
  const mx = $("#mixer"), P = new Map();
  let g = null;
  mx.addEventListener("pointerdown", (e) => {
    if (!on() || e.pointerType === "mouse") return;
    P.set(e.pointerId, { x: e.clientX, y: e.clientY, t: performance.now(), knob: e.target.closest(".knob") });
    if (P.size === 2) {
      const [a, b] = [...P.values()];
      g = { d0: Math.hypot(a.x - b.x, a.y - b.y), done: false, moved: false, t0: performance.now(), knob: a.knob || b.knob, twin: Math.abs(a.t - b.t) < 160 };
    }
  }, true);
  mx.addEventListener("pointermove", (e) => {
    const p = P.get(e.pointerId); if (!p) return;
    if (Math.hypot(e.clientX - p.x, e.clientY - p.y) > 10 && g) g.moved = true;
    p.cx = e.clientX; p.cy = e.clientY;
    if (!g || g.done || P.size !== 2) return;
    const [a, b] = [...P.values()], d = Math.hypot((a.cx ?? a.x) - (b.cx ?? b.x), (a.cy ?? a.y) - (b.cy ?? b.y));
    if (d < g.d0 * 0.7 || d > g.d0 * 1.3) { g.done = true; B.classList.toggle("mix-min", d < g.d0); requestAnimationFrame(resizeCanvases); }
  });
  const rel = (e) => {
    if (!P.delete(e.pointerId) || !g || P.size) return;
    if (!g.done && !g.moved && g.twin && g.knob && performance.now() - g.t0 < 400) g.knob.dispatchEvent(new MouseEvent("dblclick", { bubbles: true }));
    g = null;
  };
  mx.addEventListener("pointerup", rel); mx.addEventListener("pointercancel", rel);
})();
