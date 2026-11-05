/* Configurazione automatica del video: rileva il computer, misura quanto regge (grafica, decodifica, codifica) e sceglie un profilo.
   La misura parte da sola poche volte (primo avvio o se l'hardware cambia) quando i deck sono fermi; il profilo si puo' forzare a mano.
   Le soglie sono provvisorie: vanno tarate con prove su PC Windows di fascia bassa e media (vedi PRD, fase 0). */
(() => {
  const T = window.__TAURI__;
  const LS = (k, v) => { try { if (v === undefined) return localStorage.getItem(k); if (v === null) localStorage.removeItem(k); else localStorage.setItem(k, v); } catch (e) {} return null; };
  const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

  // h = altezza massima di disegno, fps = fotogrammi al secondo massimi, visH = altezza delle visualizzazioni, simple = solo transizioni leggere
  const PROFILES = {
    bassa:   { id: "bassa",   name: "Bassa",   h: 480,  fps: 24, visH: 360, simple: true,  rec: "480p 24 fps",  level: 3 },
    media:   { id: "media",   name: "Media",   h: 720,  fps: 30, visH: 540, simple: false, rec: "720p 30 fps",  level: 2 },
    alta:    { id: "alta",    name: "Alta",    h: 1080, fps: 30, visH: 720, simple: false, rec: "1080p 30 fps", level: 1 },
    massima: { id: "massima", name: "Massima", h: 1080, fps: 60, visH: 720, simple: false, rec: "1080p 60 fps", level: 0 },
  };
  // transizioni leggere (indici in VIDEO_TRANS): dissolvenza, taglio, dissolvenza al nero, flash bianco, wipe
  const SIMPLE_TRANS = [0, 1, 2, 3, 4];

  /** Sceglie il profilo dalle misure. Prudente: senza la misura della decodifica non si supera "media". */
  function decide(r) {
    if (!r || r.error) return { id: "media", why: "misura non disponibile: profilo prudente" };
    const why = [];
    let id = "bassa";
    if (r.shader720 >= 40) { id = "media"; why.push(`grafica 720p ${r.shader720.toFixed(0)} fps`); }
    else why.push(`grafica 720p solo ${r.shader720.toFixed(0)} fps (servono 40)`);
    if (id !== "bassa" && r.shader1080 >= 40 && r.decodeX >= 2 && r.ram >= 8) { id = "alta"; why.push(`grafica 1080p ${r.shader1080.toFixed(0)} fps, decodifica ${r.decodeX.toFixed(1)}×`); }
    if (id === "alta" && r.shader1080 >= 90 && r.decodeX >= 3.5 && r.ram >= 16) { id = "massima"; why.push(`1080p fino a ${r.shader1080.toFixed(0)} fps, ${r.ram} GB di memoria`); }
    if (id !== "bassa" && !r.decodeX) why.push("decodifica non misurata: non si supera 'media'");
    return { id, why: why.join(" · ") };
  }

  async function gpuAndShaders() {
    const out = { gpu: "", shader1080: 0, shader720: 0, visFps: 0 };
    if (!window.VideoGL) return { ...out, error: "modulo grafico assente" };
    const cv = document.createElement("canvas"); cv.width = 64; cv.height = 64;
    const comp = VideoGL.create(cv);
    if (!comp) return { ...out, error: VideoGL.error || "WebGL non disponibile" };
    out.gpu = comp.info();
    const tex = document.createElement("canvas"); tex.width = 1920; tex.height = 1080;
    const c2 = tex.getContext("2d");
    const g = c2.createLinearGradient(0, 0, 1920, 1080); g.addColorStop(0, "#2a6"); g.addColorStop(0.5, "#c3a"); g.addColorStop(1, "#38f");
    c2.fillStyle = g; c2.fillRect(0, 0, 1920, 1080);
    for (let i = 0; i < 200; i++) { c2.fillStyle = `hsl(${i * 37 % 360},70%,${30 + i % 40}%)`; c2.fillRect((i * 97) % 1900, (i * 61) % 1050, 120, 70); }
    comp.upload(0, tex, 1920, 1080); comp.upload(1, tex, 1920, 1080);
    // transizioni pesanti (glitch, vortice) a piena velocita': i disegni al secondo che la scheda regge
    const run = async (w, h, ms) => {
      comp.resize(w, h); let n = 0; const t0 = performance.now();
      while (performance.now() - t0 < ms) {
        for (const ty of [6, 11]) comp.draw({ t: 0.5, type: ty, time: performance.now() / 1000, beat: 0.3, has: [true, true] });
        comp.sync(); n += 2;
        if (n % 8 === 0) await sleep(0);
      }
      return n * 1000 / (performance.now() - t0);
    };
    await run(640, 360, 150);                       // riscaldamento (compilazione, cache)
    out.shader1080 = await run(1920, 1080, 800);
    out.shader720 = await run(1280, 720, 800);
    if (window.VideoVis) {
      const v = VideoVis.create(960, 540);
      if (v) { let n = 0; const t0 = performance.now(); while (performance.now() - t0 < 600) { for (const sc of [8, 5]) v.render({ scene: sc, time: performance.now() / 1000, beat: 0.3, bar: 0.2, level: 0.6, hue: 0.4 }); v.sync(); n += 2; if (n % 8 === 0) await sleep(0); } out.visFps = n * 1000 / (performance.now() - t0); }
    }
    return out;
  }

  const fingerprint = (hw, gpu) => [hw.cpu, hw.cores, hw.ram_gb, gpu, hw.os].join("|");
  let busy = false, last = null;
  try { last = JSON.parse(LS("spinlab.video_perf") || "null"); } catch (e) {}

  async function measure(onStep) {
    if (busy) return last;
    busy = true;
    const step = (t) => { try { onStep && onStep(t); } catch (e) {} };
    try {
      step("Rilevamento del computer…");
      const hw = await T.core.invoke("video_hw");
      step("Misura della grafica…");
      const gs = await gpuAndShaders();
      let decodeX = 0, encoder = "", encodeFps = 0, decodeErr = "";
      step("Misura di decodifica e codifica…");
      try {
        const clip = await T.core.invoke("video_bench_clip");
        encoder = clip.encoder; encodeFps = clip.encode_fps;
        try { decodeX = Math.min(10, await T.core.invoke("video_bench_decode", { path: clip.path })); } catch (e) { decodeErr = String(e.message || e); }
      } catch (e) { decodeErr = String(e); }
      try { await T.core.invoke("video_bench_clean"); } catch (e) {}
      let hwDecode = null;
      try { if (navigator.mediaCapabilities) { const m = await navigator.mediaCapabilities.decodingInfo({ type: "file", video: { contentType: 'video/mp4; codecs="avc1.640028"', width: 1920, height: 1080, bitrate: 8e6, framerate: 30 } }); hwDecode = !!m.powerEfficient; } } catch (e) {}
      let stems = null; try { stems = !!(await T.core.invoke("stems_cfg")).realtime; } catch (e) {}
      const res = { shader1080: gs.shader1080, shader720: gs.shader720, visFps: gs.visFps, decodeX, hwDecode, encoder, encodeFps, ram: hw.ram_gb, cores: hw.cores, stemsRealtime: stems, error: gs.error, decodeErr };
      const pick = decide(res);
      last = { v: 2, fp: fingerprint(hw, gs.gpu), hw: { ...hw, gpu: gs.gpu }, res, auto: pick.id, why: pick.why, at: Date.now() };
      LS("spinlab.video_perf", JSON.stringify(last));
      return last;
    } finally { busy = false; }
  }

  /** Profilo in uso: quello scelto a mano o quello automatico (prudente "media" finche' non c'e' una misura). */
  const manualId = () => { const m = LS("spinlab.video_profile"); return m && PROFILES[m] ? m : "auto"; };
  const active = () => { const m = manualId(); return PROFILES[m !== "auto" ? m : last?.auto || "media"]; };

  // ---------------------------------------------------------------- scheda VIDEO delle Impostazioni
  const VEXT = /\.(mp4|m4v|mov|mkv|webm|avi|ts|mpg|mpeg)$/i;
  const esc = (s) => String(s ?? "").replace(/[&<>"]/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;" }[c]));
  const ID_BY_LEVEL = ["massima", "alta", "media", "bassa"];
  let mountEl = null, status = "";
  function render() {
    if (!mountEl) return;
    const r = last?.res, hw = last?.hw, cur = active(), m = manualId();
    const row = (k, v) => `<tr><td>${k}</td><td>${v}</td></tr>`;
    mountEl.innerHTML = `<div class="vp-body">
      <h4>Miscela</h4>
      <label class="vp-sel">La miscela video segue <select id="vpMix"><option value="xfader">il crossfader</option><option value="channels">i fader dei canali</option></select></label>
      <p class="hint">Con <b>il crossfader</b> la transizione dipende solo da quello (come negli altri programmi). Con <b>i fader dei canali</b> il crossfader viene ignorato e il video passa da un deck all'altro in base al volume dei due canali: serve a chi mixa con i canali e lascia il crossfader fermo.</p>
      <h4>Profilo</h4>
      <label class="vp-sel">Qualità dell'uscita video <select id="vpSel"><option value="auto">Automatico</option>${Object.values(PROFILES).map((p) => `<option value="${p.id}">${p.name} (fino a ${p.h}p, ${p.fps} fps)</option>`).join("")}</select></label>
      <p>In uso: <b>${cur.name}</b> — fino a ${cur.h}p a ${cur.fps} fps${cur.simple ? ", solo transizioni leggere" : ", tutte le transizioni"}; registrazione ${cur.rec}.<br><span class="dim">${m === "auto" ? (last ? "Scelto in automatico: " + esc(last.why) : "In attesa della prima analisi del computer.") : "Forzato da te."}</span></p>
      <p class="vp-live">Uscita in questo momento: <b id="vpLive">${esc(liveText(window.__vq))}</b></p>
      <div class="btns" style="justify-content:flex-start"><button id="vpCal">Ottimizza con un video in riproduzione</button><span class="vp-st">${esc(status)}</span></div>
      <p class="hint">All'avvio il programma analizza il computer e sceglie un profilo di partenza. «Ottimizza» lo affina sul campo: avvia un video su un deck, premi il pulsante e per circa 10 secondi il programma prova la qualità più alta e scende finché l'uscita è fluida, poi ricorda il risultato. Durante il set l'uscita scende da sola se il computer fatica e risale quando il carico cala.</p>
      <p class="hint">Il profilo limita risoluzione e fotogrammi al secondo dell'<b>uscita a schermo intero</b>: la differenza si nota su un proiettore o un monitor grande. In una piccola finestra di anteprima, o con un video a 25 fps, profili diversi sembrano uguali.</p>
      ${last ? `<h4>Analisi del computer</h4><table>${row("Processore", `${esc(hw.cpu)} · ${hw.cores} core`)}${row("Memoria", `${hw.ram_gb} GB`)}${row("Scheda grafica", esc(hw.gpu || "non rilevata"))}${row("Sistema", esc(hw.os))}${row("Encoder video", hw.encoders?.length ? esc(hw.encoders.join(", ")) : "nessuno (FFmpeg mancante?)")}${row("Grafica 1080p / 720p", r.error ? `non misurata (${esc(r.error)})` : `${r.shader1080.toFixed(0)} / ${r.shader720.toFixed(0)} disegni/s`)}${row("Visualizzazioni", r.error ? "—" : `${r.visFps.toFixed(0)} disegni/s`)}${row("Decodifica video", r.decodeX ? `${r.decodeX.toFixed(1)}× il tempo reale${r.hwDecode === true ? " (accelerata)" : r.hwDecode === false ? " (software)" : ""}` : `non misurata${r.decodeErr ? " — " + esc(r.decodeErr) : ""}`)}${row("Codifica MP4", r.encoder ? `${esc(r.encoder)}: ${r.encodeFps.toFixed(0)} fotogrammi/s` : "non misurata")}${row("Stem", r.stemsRealtime === null ? "—" : r.stemsRealtime ? "in tempo reale" : "solo preparati")}</table>` : ""}
    </div>`;
    const sel = mountEl.querySelector("#vpSel"); sel.value = m;
    sel.onchange = () => { LS("spinlab.video_profile", sel.value === "auto" ? null : sel.value); render(); };
    mountEl.querySelector("#vpCal").onclick = calibrate;
    const mx = mountEl.querySelector("#vpMix"); mx.value = window.VideoMix ? VideoMix.mode : "xfader"; mx.onchange = () => { if (window.VideoMix) VideoMix.set(mx.value); };
  }
  /** Testo della qualita' reale dell'uscita (quella che l'adattamento sta usando adesso, non solo il profilo scelto). */
  function liveText(q) {
    if (!q) return "uscita video non attiva (si accende quando carichi un video o premi VIDEO)";
    const lv = q.off === 1 ? "1 livello" : `${q.off} livelli`;
    return `${q.h}p a ${q.fps} fps (disegno reale ${q.real} fps)${q.off > 0 ? `, ridotta di ${lv} perché il computer fatica` : ""}${q.calib ? " — calibrazione in corso" : ""}`;
  }
  const live = () => { const e = mountEl && mountEl.querySelector("#vpLive"); if (e) e.textContent = liveText(window.__vq); };
  function mount(el) { mountEl = el; render(); }
  const refresh = () => render();
  const setStatus = (t) => { status = t; const e = mountEl && mountEl.querySelector(".vp-st"); if (e) e.textContent = t; };

  // Calibrazione sul campo: l'uscita video parte dal livello piu' alto e scende finche' regge i fotogrammi al secondo (vedi video-out.html)
  let calTimer = 0;
  function calibrate() {
    const playing = (typeof decks !== "undefined") && decks.slice(0, 2).some((d) => d.snap?.playing && d.snap?.loaded && VEXT.test(d.snap.path || ""));
    if (!playing) { setStatus("Avvia un video su un deck (play) e riprova."); return; }
    setStatus("Calibrazione in corso (circa 10 secondi)…");
    const b = mountEl && mountEl.querySelector("#vpCal"); if (b) b.disabled = true;
    try { T.event.emit("video-calibrate", {}); } catch (e) {}
    clearTimeout(calTimer);
    calTimer = setTimeout(() => { setStatus("L'uscita video non ha risposto: attiva VIDEO e riprova."); const bb = mountEl && mountEl.querySelector("#vpCal"); if (bb) bb.disabled = false; }, 40000);
  }
  T.event.listen("video-calibrated", (e) => {
    clearTimeout(calTimer);
    const level = Math.max(0, Math.min(3, e.payload.level | 0)), id = ID_BY_LEVEL[level];
    last = { ...(last || { v: 2, fp: "", hw: {}, res: {} }), auto: id, why: `calibrato con un video in riproduzione: l'uscita regge ${PROFILES[id].h}p ${PROFILES[id].fps} fps`, at: Date.now() };
    LS("spinlab.video_perf", JSON.stringify(last));
    status = `Fatto: profilo ${PROFILES[id].name}.`;
    try { (typeof toast2 === "function" ? toast2 : () => {})(`Video ottimizzato: profilo ${PROFILES[id].name}`); } catch (er) {}
    render();
  });

  // ---------------------------------------------------------------- misura automatica
  // Parte da sola quando manca (primo avvio) o l'hardware e' cambiato, ma solo a deck fermi: la misura usa la scheda grafica.
  let tries = 0;
  async function auto() {
    if (busy || ++tries > 60) return;
    const playing = (typeof decks !== "undefined") && decks.slice(0, 2).some((d) => d.snap?.playing);
    if (playing) return;
    try {
      const hw = await T.core.invoke("video_hw");
      const cv = document.createElement("canvas"); const comp = window.VideoGL && VideoGL.create(cv); const gpu = comp ? comp.info() : "";
      const bad = last && (last.v !== 2 || last.res?.error || !(last.res?.shader720 > 0));   // misura fallita o di una versione vecchia
      if (last && !bad && last.fp === fingerprint(hw, gpu)) { tries = 999; return; }
    } catch (e) { return; }
    tries = 999;
    const say = (t) => { try { (typeof toast2 === "function" ? toast2 : () => {})(t); } catch (e) {} };
    say("Analisi del computer per il video (pochi secondi)…");
    try { const r = await measure(); say(`Video: profilo ${PROFILES[r.auto].name} scelto in automatico (dettagli nel menu VIDEO → Prestazioni)`); } catch (e) { console.warn("misura video", e); }
  }
  setTimeout(() => { auto(); setInterval(auto, 15000); }, 10000);

  window.VideoPerf = { PROFILES, SIMPLE_TRANS, decide, measure, active, mount, live, liveText, refresh, manualId, last: () => last };
})();
