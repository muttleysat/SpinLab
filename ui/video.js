/* PROTOTIPO — modalita' VIDEO. Si accende da sola quando su un deck c'e' un video e si spegne quando restano solo brani audio;
   il pulsante VIDEO la forza (anche per i soli brani). Tutto resta spento finche' non serve: nessuna finestra, nessun invio di dati. */
(() => {
  const VEXT = /\.(mp4|m4v|mov|mkv|webm|avi|ts|mpg|mpeg)$/i;
  const btn = document.getElementById("videoBtn");
  if (!btn) return;
  let manual = null;       // null = automatica, true/false = scelta dell'utente
  let open = false, busy = false, timer = 0, tick = 0, openedKey = "";
  // schermo di uscita: si ricorda per nome e risoluzione (l'indice cambia quando si collega o scollega qualcosa)
  const keyOf = (m) => `${m.name}|${m.width}x${m.height}`;
  const savedKey = () => { try { return localStorage.getItem("spinlab.video_monitor_key") || ""; } catch (e) { return ""; } };
  const pick = (mons) => { const k = savedKey(); const m = k ? mons.find((x) => keyOf(x) === k) : null; return m ? { index: m.index, key: k } : { index: null, key: "" }; };
  const say = (t) => { try { if (typeof toast2 === "function") toast2(t); else if (typeof toast === "function") toast(t); } catch (e) {} };
  async function openOut() {
    let mons = []; try { mons = await inv("video_monitors"); } catch (e) {}
    const t = pick(mons);
    await inv("video_open", { monitor: t.index });
    open = true; openedKey = t.key;
  }
  try { const m = localStorage.getItem("spinlab.video_manual"); manual = m === "1" ? true : m === "0" ? false : null; } catch (e) {}

  const hasVideo = () => decks.slice(0, 2).some((d) => d.snap?.loaded && VEXT.test(d.snap.path || ""));
  const wanted = () => (manual === null ? hasVideo() : manual);

  // menu delle transizioni (la scelta si ricorda)
  const sel = document.getElementById("vtrans");
  if (sel && window.VIDEO_TRANS) {
    sel.innerHTML = VIDEO_TRANS.map((n, i) => `<option value="${i}">${n}</option>`).join("");
    try { const v = localStorage.getItem("spinlab.video_trans"); if (v !== null && +v < VIDEO_TRANS.length) sel.value = v; } catch (e) {}
    sel.onchange = () => { try { localStorage.setItem("spinlab.video_trans", sel.value); } catch (e) {} };
  }
  const vsel = document.getElementById("vvis");
  if (vsel && window.VIDEO_VIS) {
    vsel.innerHTML = VIDEO_VIS.map((n, i) => `<option value="${i}">${n}</option>`).join("");
    try { const v = localStorage.getItem("spinlab.video_vis"); if (v !== null && +v < VIDEO_VIS.length) vsel.value = v; } catch (e) {}
    vsel.onchange = () => { try { localStorage.setItem("spinlab.video_vis", vsel.value); } catch (e) {} };
  }
  // tonalita' Camelot (1A..12B) -> colore base delle visualizzazioni
  const hueOf = (k) => { const n = parseInt(k, 10); return n >= 1 && n <= 12 ? (n - 1) / 12 : 0.6; };
  const proxies = {};      // percorso del video -> percorso della copia di lavoro (quando e' pronta) o "" se in preparazione / non riuscita
  const askProxy = (path) => {
    if (path in proxies) return;
    proxies[path] = "";
    inv("video_proxy", { path }).then((p) => { proxies[path] = p; }).catch((e) => { console.warn("video_proxy", e); });
  };
  const cleanup = () => { inv("video_cleanup", { keep: Object.values(proxies).filter(Boolean).filter((p, i, a) => decks.slice(0, 2).some((d) => proxies[d.snap?.path] === p)) }).catch(() => {}); };
  inv("video_cleanup", { keep: [] }).catch(() => {});   // all'avvio: copie rimaste da una chiusura anomala

  // con un profilo «bassa» restano solo le transizioni leggere (le altre si vedono ma sono disattivate)
  function applyProfileToMenu() {
    if (!sel || !window.VideoPerf) return;
    const simple = VideoPerf.active().simple;
    [...sel.options].forEach((o) => { const off = simple && !VideoPerf.SIMPLE_TRANS.includes(+o.value); o.disabled = off; o.title = off ? "Non disponibile con il profilo Bassa" : ""; });
    if (simple && !VideoPerf.SIMPLE_TRANS.includes(+sel.value)) sel.value = "0";
  }
  const badge = document.getElementById("vqBadge");
  window.__TAURI__.event.listen("video-quality", (e) => {
    const q = window.__vq = { ...e.payload, at: performance.now() };
    if (badge) { badge.hidden = false; badge.textContent = `${q.h}p ${q.fps}`; badge.classList.toggle("low", q.off > 0); badge.title = window.VideoPerf ? window.VideoPerf.liveText(q) : ""; }
    if (window.VideoPerf) VideoPerf.live();
  });
  setInterval(() => { if (badge && window.__vq && performance.now() - window.__vq.at > 3000) { badge.hidden = true; window.__vq = null; if (window.VideoPerf) VideoPerf.live(); } }, 1500);
  window.__TAURI__.event.listen("video-adapt", (e) => say(String(e.payload || "")));

  async function sync() {
    applyProfileToMenu();
    const w = wanted();
    if (w) decks.slice(0, 2).forEach((d) => { if (d.snap?.loaded && VEXT.test(d.snap.path || "")) askProxy(d.snap.path); });
    // copie dei video non piu' sui deck: si cancellano
    const used = new Set(decks.slice(0, 2).map((d) => d.snap?.path).filter(Boolean));
    let changed = false;
    for (const k of Object.keys(proxies)) if (!used.has(k)) { delete proxies[k]; changed = true; }
    if (changed) cleanup();
    btn.classList.toggle("on", w);
    document.body.classList.toggle("vmode", w);
    if (w && !open && !busy) {
      busy = true;
      try { await openOut(); } catch (e) { console.warn("video_open", e); }
      busy = false;
    } else if (!w && open && !busy) { try { await inv("video_close"); } catch (e) {} open = false; }
    if (open && !busy && ++tick % 4 === 0) {
      try {
        const t = pick(await inv("video_monitors"));
        if (t.key !== openedKey) {          // lo schermo scelto e' comparso o scomparso: l'uscita lo segue senza fermare audio e video
          busy = true; await inv("video_open", { monitor: t.index }); busy = false;
          say(t.key ? "Schermo esterno collegato: uscita video a schermo intero" : "Schermo esterno scollegato: l'uscita video passa in una finestra");
          openedKey = t.key;
        }
      } catch (e) { busy = false; }
    }
    if (w && open && !timer) timer = setInterval(send, 33);
    if ((!w || !open) && timer) { clearInterval(timer); timer = 0; }
  }
  const artSent = [null, null];
  function send() {
    // copertina: si manda una volta sola al cambio di brano (e' pesante per il flusso a 30 messaggi al secondo)
    decks.slice(0, 2).forEach((d, i) => {
      const p = d.snap?.loaded ? d.snap.path : "";
      if (p !== artSent[i]) { artSent[i] = p; (p ? getArt(p, 512) : Promise.resolve(null)).then((art) => window.__TAURI__.event.emit("video-art", { i, path: p, art })); }
    });
    const s = {
      xf: window.__lastXf ?? 0.5,
      trans: sel ? +sel.value : 0,
      quality: window.VideoPerf ? (({ h, fps, visH, simple, level }) => ({ h, fps, visH, simple, level }))(VideoPerf.active()) : { h: 720, fps: 30, visH: 540, simple: false, level: 2 },
      vis: vsel ? +vsel.value : 0,
      decks: decks.slice(0, 2).map((d, i) => ({
        proxy: proxies[d.snap?.path] || "", beat: d.snap?.beat_phase ?? 0, bpm: d.snap?.bpm_now || 0, level: Math.max(d.snap?.level_l || 0, d.snap?.level_r || 0), hue: hueOf(d.snap?.key_now),  held: d.scrub != null || !!d.snap?.held, loaded: !!d.snap?.loaded, title: d.title_ || d.snap?.name || "", artist: d.meta?.artist || "", path: d.snap?.path || "", name: d.snap?.name || "", playing: !!d.snap?.playing,
        // come per le onde: il suono esce dal dispositivo con un ritardo (buffer), quindi anche il video va ritardato di tanto
        pos: smoothPos(i) - (d.scrub == null && d.snap?.playing ? (audioLatency + visOffset) * (1 + (d.snap?.tempo_pct || 0) / 100) : 0), rate: 1 + (d.snap?.tempo_pct || 0) / 100, assign: d.snap?.xf_assign ?? i,
      })),
    };
    window.__TAURI__.event.emit("video-state", s);
  }
  // il crossfader arriva dallo stato del motore (anche se mosso da un controller MIDI)
  setInterval(async () => { try { const s = await inv("get_state"); window.__lastXf = s.crossfader; } catch (e) {} }, 100);
  setInterval(sync, 500);

  // clic: automatica -> forzata accesa/spenta; clic destro = scelta dello schermo
  btn.onclick = () => {
    manual = manual === null ? !wanted() : manual === true ? false : null;
    try { localStorage.setItem("spinlab.video_manual", manual === null ? "" : manual ? "1" : "0"); } catch (e) {}
    btn.dataset.mode = manual === null ? "AUTO" : manual ? "ON" : "OFF";
    sync();
  };
  let menu = null;
  const closeMenu = () => { if (menu) { menu.remove(); menu = null; } };
  btn.oncontextmenu = async (e) => {
    e.preventDefault();
    if (menu) { closeMenu(); return; }
    let mons = []; try { mons = await inv("video_monitors"); } catch (err) {}
    const cur = savedKey();
    menu = document.createElement("div"); menu.className = "vmenu";
    const item = (label, key) => {
      const b = document.createElement("button"); b.textContent = label; if (key === cur) b.classList.add("sel");
      b.onclick = async () => {
        try { localStorage.setItem("spinlab.video_monitor_key", key); } catch (er) {}
        closeMenu();
        if (open && !busy) { busy = true; try { const t = pick(mons); await inv("video_open", { monitor: t.index }); openedKey = t.key; } catch (er) {} busy = false; }
      };
      menu.appendChild(b);
    };
    const h = document.createElement("div"); h.className = "vh"; h.textContent = "Uscita video su…"; menu.appendChild(h);
    item("Finestra normale", "");
    mons.forEach((m) => item(`${m.name} · ${m.width}×${m.height}${m.primary ? " (principale)" : ""}`, keyOf(m)));
    document.body.appendChild(menu);
    const r = btn.getBoundingClientRect();
    menu.style.top = `${r.bottom + 6}px`; menu.style.right = `${Math.max(8, innerWidth - r.right)}px`;
    setTimeout(() => document.addEventListener("click", closeMenu, { once: true }), 0);
  };
  document.addEventListener("keydown", (e) => { if (e.key === "Escape") closeMenu(); });
  btn.dataset.mode = manual === null ? "AUTO" : manual ? "ON" : "OFF";
})();
