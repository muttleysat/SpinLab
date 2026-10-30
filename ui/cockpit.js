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
