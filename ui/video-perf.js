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

  /** Riproduce a velocita' 4× il video di prova nel motore web: quanti fotogrammi al secondo decodifica (in multipli del tempo reale). */
  async function decodeBench(path) {
    const v = document.createElement("video");
    v.muted = true; v.playsInline = true; v.loop = true; v.preload = "auto";
    v.style.cssText = "position:fixed;left:0;top:0;width:2px;height:2px;opacity:.01;pointer-events:none";
    document.body.appendChild(v);
    try {
      v.src = T.core.convertFileSrc(path);
      await new Promise((res, rej) => { v.onloadeddata = res; v.onerror = () => rej(new Error("video di prova non leggibile")); setTimeout(() => rej(new Error("timeout")), 5000); });
      v.playbackRate = 4;
      await v.play();
      await sleep(400);
      const q0 = v.getVideoPlaybackQuality(), t0 = performance.now();
      await sleep(1500);
      const q1 = v.getVideoPlaybackQuality(), dt = (performance.now() - t0) / 1000;
      const frames = (q1.totalVideoFrames - q1.droppedVideoFrames) - (q0.totalVideoFrames - q0.droppedVideoFrames);
      return Math.min(4, frames / dt / 30);
    } finally { try { v.pause(); v.removeAttribute("src"); v.load(); } catch (e) {} v.remove(); }
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
        try { decodeX = await decodeBench(clip.path); } catch (e) { decodeErr = String(e.message || e); }
      } catch (e) { decodeErr = String(e); }
      try { await T.core.invoke("video_bench_clean"); } catch (e) {}
      let hwDecode = null;
      try { if (navigator.mediaCapabilities) { const m = await navigator.mediaCapabilities.decodingInfo({ type: "file", video: { contentType: 'video/mp4; codecs="avc1.640028"', width: 1920, height: 1080, bitrate: 8e6, framerate: 30 } }); hwDecode = !!m.powerEfficient; } } catch (e) {}
      let stems = null; try { stems = !!(await T.core.invoke("stems_cfg")).realtime; } catch (e) {}
      const res = { shader1080: gs.shader1080, shader720: gs.shader720, visFps: gs.visFps, decodeX, hwDecode, encoder, encodeFps, ram: hw.ram_gb, cores: hw.cores, stemsRealtime: stems, error: gs.error, decodeErr };
      const pick = decide(res);
      last = { v: 1, fp: fingerprint(hw, gs.gpu), hw: { ...hw, gpu: gs.gpu }, res, auto: pick.id, why: pick.why, at: Date.now() };
      LS("spinlab.video_perf", JSON.stringify(last));
      return last;
    } finally { busy = false; }
  }

  /** Profilo in uso: quello scelto a mano o quello automatico (prudente "media" finche' non c'e' una misura). */
  const manualId = () => { const m = LS("spinlab.video_profile"); return m && PROFILES[m] ? m : "auto"; };
  const active = () => { const m = manualId(); return PROFILES[m !== "auto" ? m : last?.auto || "media"]; };

  // ---------------------------------------------------------------- pannello
  let panel = null;
  const esc = (s) => String(s ?? "").replace(/[&<>"]/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;" }[c]));
  function render() {
    if (!panel) return;
    const r = last?.res, hw = last?.hw, cur = active(), m = manualId();
    const row = (k, v) => `<tr><td>${k}</td><td>${v}</td></tr>`;
    panel.querySelector(".vp-body").innerHTML = last ? `
      <h4>Computer</h4><table>${row("Processore", `${esc(hw.cpu)} · ${hw.cores} core`)}${row("Memoria", `${hw.ram_gb} GB`)}${row("Scheda grafica", esc(hw.gpu || "non rilevata"))}${row("Sistema", esc(hw.os))}${row("Encoder video", hw.encoders?.length ? esc(hw.encoders.join(", ")) : "nessuno (FFmpeg mancante?)")}</table>
      <h4>Misure</h4><table>${row("Grafica 1080p", `${r.shader1080.toFixed(0)} disegni/s`)}${row("Grafica 720p", `${r.shader720.toFixed(0)} disegni/s`)}${row("Visualizzazioni", `${r.visFps.toFixed(0)} disegni/s`)}${row("Decodifica video", r.decodeX ? `${r.decodeX.toFixed(1)}× il tempo reale${r.hwDecode === true ? " (accelerata)" : r.hwDecode === false ? " (software)" : ""}` : `non misurata${r.decodeErr ? " — " + esc(r.decodeErr) : ""}`)}${row("Codifica MP4", r.encoder ? `${esc(r.encoder)}: ${r.encodeFps.toFixed(0)} fotogrammi/s` : "non misurata")}${row("Stem", r.stemsRealtime === null ? "—" : r.stemsRealtime ? "in tempo reale" : "solo preparati")}</table>
      <h4>Profilo</h4><p><b>${cur.name}</b> — fino a ${cur.h}p a ${cur.fps} fps${cur.simple ? ", transizioni leggere" : ", tutte le transizioni"}; registrazione ${cur.rec}.<br><span class="dim">${m === "auto" ? "Scelto in automatico: " + esc(last.why) : "Forzato dall'utente."}</span></p>`
      : `<p>Nessuna misura ancora. Premi «Misura ora».</p>`;
    const sel = panel.querySelector("select"); sel.value = m;
  }
  function openPanel() {
    if (panel) { panel.remove(); panel = null; return; }
    panel = document.createElement("div"); panel.className = "vperf";
    panel.innerHTML = `<div class="vp-box"><div class="vp-head"><b>Prestazioni video</b><button class="vp-x" title="Chiudi">×</button></div><div class="vp-body"></div>
      <div class="vp-foot"><label>Profilo <select><option value="auto">Automatico</option>${Object.values(PROFILES).map((p) => `<option value="${p.id}">${p.name} (${p.h}p ${p.fps} fps)</option>`).join("")}</select></label><span class="vp-st"></span><button class="vp-go">Misura ora</button></div></div>`;
    document.body.appendChild(panel);
    panel.querySelector(".vp-x").onclick = () => { panel.remove(); panel = null; };
    panel.onclick = (e) => { if (e.target === panel) { panel.remove(); panel = null; } };
    panel.querySelector("select").onchange = (e) => { LS("spinlab.video_profile", e.target.value === "auto" ? null : e.target.value); render(); };
    panel.querySelector(".vp-go").onclick = async () => {
      const st = panel.querySelector(".vp-st"), b = panel.querySelector(".vp-go"); b.disabled = true;
      try { await measure((t) => { st.textContent = t; }); } catch (e) { st.textContent = "Errore: " + e; }
      b.disabled = false; if (panel) { st.textContent = ""; render(); }
    };
    render();
  }

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
      if (last && last.fp === fingerprint(hw, gpu)) { tries = 999; return; }
    } catch (e) { return; }
    tries = 999;
    const say = (t) => { try { (typeof toast2 === "function" ? toast2 : () => {})(t); } catch (e) {} };
    say("Analisi del computer per il video (pochi secondi)…");
    try { const r = await measure(); say(`Video: profilo ${PROFILES[r.auto].name} scelto in automatico (dettagli nel menu VIDEO → Prestazioni)`); } catch (e) { console.warn("misura video", e); }
  }
  setTimeout(() => { auto(); setInterval(auto, 15000); }, 10000);

  window.VideoPerf = { PROFILES, SIMPLE_TRANS, decide, measure, active, openPanel, manualId, last: () => last };
})();
