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

  async function sync() {
    const w = wanted();
    btn.classList.toggle("on", w);
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
  function send() {
    const s = {
      xf: window.__lastXf ?? 0.5,
      decks: decks.slice(0, 2).map((d, i) => ({
        loaded: !!d.snap?.loaded, path: d.snap?.path || "", name: d.snap?.name || "", playing: !!d.snap?.playing,
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
