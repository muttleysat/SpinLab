/* PROTOTIPO — modalita' VIDEO. Si accende da sola quando su un deck c'e' un video e si spegne quando restano solo brani audio;
   il pulsante VIDEO la forza (anche per i soli brani). Tutto resta spento finche' non serve: nessuna finestra, nessun invio di dati. */
(() => {
  const VEXT = /\.(mp4|m4v|mov|mkv|webm|avi|ts|mpg|mpeg)$/i;
  const btn = document.getElementById("videoBtn");
  if (!btn) return;
  let manual = null;       // null = automatica, true/false = scelta dell'utente
  let open = false, busy = false, timer = 0;
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
  const proxies = {};      // percorso del video -> percorso della copia di lavoro (quando e' pronta) o "" se in preparazione / non riuscita
  const askProxy = (path) => {
    if (path in proxies) return;
    proxies[path] = "";
    inv("video_proxy", { path }).then((p) => { proxies[path] = p; }).catch((e) => { console.warn("video_proxy", e); });
  };
  const cleanup = () => { inv("video_cleanup", { keep: Object.values(proxies).filter(Boolean).filter((p, i, a) => decks.slice(0, 2).some((d) => proxies[d.snap?.path] === p)) }).catch(() => {}); };
  inv("video_cleanup", { keep: [] }).catch(() => {});   // all'avvio: copie rimaste da una chiusura anomala

  async function sync() {
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
      try {
        let monitor = null;
        try { const m = JSON.parse(localStorage.getItem("spinlab.video_monitor")); if (Number.isInteger(m)) monitor = m; } catch (e) {}
        await inv("video_open", { monitor }); open = true;
      } catch (e) { console.warn("video_open", e); }
      busy = false;
    } else if (!w && open && !busy) { try { await inv("video_close"); } catch (e) {} open = false; }
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
      decks: decks.slice(0, 2).map((d, i) => ({
        proxy: proxies[d.snap?.path] || "", beat: d.snap?.beat_phase ?? 0, held: d.scrub != null || !!d.snap?.held, loaded: !!d.snap?.loaded, title: d.title_ || d.snap?.name || "", artist: d.meta?.artist || "", path: d.snap?.path || "", name: d.snap?.name || "", playing: !!d.snap?.playing,
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
  btn.oncontextmenu = async (e) => {
    e.preventDefault();
    const mons = await inv("video_monitors");
    const names = mons.map((m, i) => `${i}: ${m.name} ${m.width}x${m.height}${m.primary ? " (principale)" : ""}`).join("\n");
    const a = prompt(`Schermo per l'uscita video (numero; vuoto = finestra normale):\n${names}`);
    if (a === null) return;
    try { localStorage.setItem("spinlab.video_monitor", a === "" ? "null" : String(+a)); } catch (e) {}
    if (open) { await inv("video_close"); open = false; sync(); }
  };
  btn.dataset.mode = manual === null ? "AUTO" : manual ? "ON" : "OFF";
})();
