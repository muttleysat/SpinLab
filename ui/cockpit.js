"use strict";
/* Gesture del layout Cockpit (telefono Android in orizzontale). Specifiche: docs/specifiche-android-cockpit.md.
   Agisce con body.cockpit; menu del deck, pitch bend, menu radiale e cassetto del pannello valgono anche per i modelli
   verticali (body.mob, mobile.js). Sul desktop questo file non fa nulla. Usa le funzioni globali di app.js. */
(function () {
  const B = document.body;
  const ck = () => B.classList.contains("cockpit");
  const on = () => ck() || B.classList.contains("mob");

  /* ---- libreria a tendina (G1, G2): maniglia, swipe giu' dalla barra, swipe su dalla barra di ricerca */
  const top = $("#top"), handle = document.createElement("button");
  handle.className = "lib-handle"; handle.type = "button"; handle.setAttribute("aria-label", "Libreria");
  top.insertBefore(handle, top.querySelector(".topr"));
  const setLib = (open) => {
    const was = B.classList.contains("lib-open");
    B.classList.toggle("lib-open", open);
    // chiudendo la libreria l'anteprima in cuffia si ferma: senza i brani davanti non c'e' piu' modo di capire cosa suona
    if (was && !open && typeof pv !== "undefined" && pv.playing) { pv.playing = false; markPreview(); inv("preview_stop").catch(() => {}); }
    handle.textContent = open ? "LIBRERIA ▴" : "LIBRERIA ▾";
    setTimeout(resizeCanvases, 260);
  };
  setLib(false);
  setTimeout(() => B.classList.add("anim"), 600);
  let swiped = 0;
  handle.addEventListener("click", () => { if (performance.now() - swiped > 400) setLib(!B.classList.contains("lib-open")); });
  const swipe = (el, dir, skip) => {
    let y0 = null;
    el.addEventListener("pointerdown", (e) => { y0 = ck() && !e.target.closest(skip || "x-none") ? e.clientY : null; });
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
  // ogni swipe ha il suo punto di partenza: con uno solo condiviso il listener del documento (bordo) azzerava quello delle schede
  const hswipe = (el, edge) => {
    let x0 = null;
    el.addEventListener("pointerdown", (e) => { x0 = on() && (edge ? e.clientX > innerWidth - 22 : !e.target.closest("input,select,.fader,.knob")) ? e.clientX : null; });
    el.addEventListener("pointermove", (e) => {
      if (x0 == null) return;
      if (edge ? x0 - e.clientX > 40 : e.clientX - x0 > 60) { x0 = null; setPn(edge); }
    });
    ["pointerup", "pointercancel"].forEach((ev) => el.addEventListener(ev, () => (x0 = null)));
  };
  hswipe(document, true);
  hswipe($(".ptabs"), false);
  // per mobile.js (modelli verticali): libreria e pannello si aprono anche dalle schede in basso
  window.CK = { setLib, setPn };

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
    const dist = Math.hypot(e.clientX - cx, e.clientY - cy);
    if (dist < R * 0.82 || dist > R) return; // dentro: scrub come sempre; negli angoli fuori dal disco: altri gesti (arco del loop, mobile.js)
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
    if (!ck() || e.pointerType === "mouse") return;
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

  /* ---- tira verso il basso per aggiornare l'elenco dei brani (dalla cima dell'elenco); una barretta in alto dice cosa succede */
  (() => {
    const wrap = $(".tablewrap"), browser = $("#browser");
    if (!wrap || !browser) return;
    const ind = document.createElement("div");
    ind.id = "ptr"; ind.setAttribute("role", "status"); browser.appendChild(ind);
    const TH = 72;                                   // quanto si tira (px, dopo la resistenza) per attivare l'aggiornamento
    let y0 = null, dy = 0, busy = false;
    const show = (d, txt) => {
      ind.style.setProperty("--d", d + "px");
      ind.textContent = txt ?? (d >= TH ? "Rilascia per aggiornare" : "Tira per aggiornare");
      ind.classList.toggle("on", d > 0 || busy);
      ind.classList.toggle("ready", d >= TH);
    };
    wrap.addEventListener("touchstart", (e) => { y0 = (busy || e.touches.length !== 1 || wrap.scrollTop > 0) ? null : e.touches[0].clientY; dy = 0; window.__ptrArm = y0 !== null; }, { passive: true });
    wrap.addEventListener("touchmove", (e) => {
      if (y0 === null) return;
      const d = e.touches[0].clientY - y0;
      if (wrap.scrollTop > 0) { y0 = null; show(dy = 0); return; }     // la lista e' scesa: e' uno scorrimento normale
      if (d <= 0) { if (dy) show(dy = 0); return; }
      e.preventDefault();                                              // niente scorrimento mentre si tira
      dy = Math.min(110, d * 0.5);                                     // un po' di resistenza
      show(dy);
    }, { passive: false });
    const end = async () => {
      window.__ptrArm = false;
      if (y0 === null) return;
      y0 = null;
      if (dy >= TH && !busy) {
        busy = true; show(52, "Aggiorno…");
        try { await refreshAll(); } catch (e) { toast(String(e)); }
        busy = false;
      }
      dy = 0; show(0);
    };
    wrap.addEventListener("touchend", end); wrap.addEventListener("touchcancel", end);
  })();

  /* ---- Indietro di Android: chiude menu, finestre, pannello e libreria (uno alla volta) invece di uscire dall'app.
     L'attivita' Android torna indietro nella cronologia della WebView finche' ce n'e', poi chiude: con qualcosa aperto
     si tiene una voce di cronologia in piu', e la si toglie quando tutto si chiude in altro modo. Vale con ogni interfaccia. */
  if (/Android/i.test(navigator.userAgent)) {
    // mobile.js aggiunge i suoi pannelli (cassetto del mixer, spazi della mappa) con CK.back = { open(), close() }
    const extra = () => window.CK.back;
    const anyOpen = () => !$("#ctx").hidden || !!$(".keypop") || !!$("dialog[open]") || !!extra()?.open() || B.classList.contains("pn-open") || B.classList.contains("lib-open");
    const closeTop = () => {
      if (!$("#ctx").hidden) $("#ctx").hidden = true;
      else if ($(".keypop")) closePops();
      else if ($("dialog[open]")) [...$$("dialog[open]")].pop().close();
      else if (B.classList.contains("pn-open")) setPn(false);
      else if (extra()?.open()) extra().close();
      else if (B.classList.contains("lib-open")) {
        // libreria aperta sulle cartelle: prima si torna ai brani, poi si chiude la libreria
        if (B.classList.contains("side-open") && (lib.folder || lib.playlist !== null)) $("#sideBar").click();
        else setLib(false);
      }
    };
    // Con l'ascoltatore ufficiale di Tauri il tasto Indietro lo decide la pagina. Prima si usava la cronologia del browser,
    // che Chrome puo' saltare: allora Indietro usciva dall'app. Ora: chiude l'ultimo pannello aperto; dalla schermata principale
    // chiede di premere ancora per uscire (entro 2 secondi), poi chiude.
    const onBack = window.__TAURI__?.app?.onBackButtonPress;
    if (typeof onBack === "function") {
      let exitAt = 0;
      onBack(() => {
        if (anyOpen()) { closeTop(); return; }
        const now = performance.now();
        if (now - exitAt < 2200 && window.SpinlabApp) { window.SpinlabApp.exit(); return; }
        exitAt = now;
        toast2("Premi ancora Indietro per uscire");
      });
      return;
    }
    // senza l'ascoltatore (versioni vecchie di Tauri) resta il vecchio metodo con la cronologia
    let guard = false, dropping = false;
    const sync = () => {
      if (dropping) return;
      const o = anyOpen();
      // solo dopo un tocco: Chrome segna come da saltare le voci aggiunte senza, e Indietro uscirebbe dall'app
      if (o && !guard && navigator.userActivation?.isActive !== false) { history.pushState({ spinlabBack: 1 }, ""); guard = true; }
      else if (!o && guard) { dropping = true; history.back(); }
    };
    addEventListener("popstate", () => {
      guard = false;
      if (dropping) dropping = false; else closeTop();
      sync();
    });
    const mo = new MutationObserver(sync);
    mo.observe(B, { attributes: true, attributeFilter: ["class"] });
    mo.observe($("#ctx"), { attributes: true, attributeFilter: ["hidden"] });
    $$("dialog").forEach((d) => mo.observe(d, { attributes: true, attributeFilter: ["open"] }));
    DK.forEach((D) => mo.observe(D.el, { childList: true }));
  }
})();
