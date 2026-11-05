/* Registrazione del set in MP4 (finestra principale). Coordina tre parti:
   - l'audio del master, registrato come sempre (WAV 24 bit temporaneo) dal motore;
   - il video composto, catturato dalla finestra di uscita e codificato da FFmpeg (vedi video-out.html e video.rs);
   - l'unione finale in un MP4 (H.264 + AAC) con la correzione dei ritardi.
   L'uscita video si accende da sola per tutta la registrazione e si riporta allo stato di prima alla fine. */
(() => {
  const T = window.__TAURI__;
  let st = null;     // registrazione in corso: { out, audio }

  /** Aspetta un evento (o null allo scadere del tempo). Va chiamata PRIMA di mandare la richiesta, per non perdere la risposta. */
  const once = (name, ms) => new Promise((res) => {
    let done = false, un = null;
    const to = setTimeout(() => { if (!done) { done = true; if (un) un(); res(null); } }, ms);
    T.event.once(name, (e) => { if (done) return; done = true; clearTimeout(to); res(e.payload); }).then((f) => { un = f; if (done) f(); });
  });
  const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

  const SIZES = { "480": { h: 480, fps: 24 }, "720": { h: 720, fps: 30 }, "1080": { h: 1080, fps: 30 }, "1080p60": { h: 1080, fps: 60 } };
  /** Altezza e fotogrammi al secondo del video registrato: quelli scelti, o quelli del profilo di qualita' in uso. La larghezza la decide la
      finestra di uscita (stessa forma dell'immagine sull'uscita, senza barre nere aggiunte); `w` qui e' solo un valore di ripiego 16:9. */
  function dims(q) {
    const c = q === "profile" || !SIZES[q] ? (({ h, fps }) => ({ h, fps }))(window.VideoPerf ? VideoPerf.active() : { h: 720, fps: 30 }) : SIZES[q];
    return { w: Math.round(c.h * 16 / 9 / 2) * 2, h: c.h, fps: c.fps };
  }

  const log = (m) => { try { if (window.vlog) window.vlog("registrazione: " + m); } catch (e) {} };
  async function start(path, quality) {
    log(`avvio -> ${path} (${quality})`);
    if (st) throw new Error("c'e' gia' una registrazione in corso");
    const vm = window.VideoMode;
    if (!vm) throw new Error("il modulo video non e' disponibile");
    await vm.force(true);                                  // accende l'uscita
    log("uscita accesa, attendo che disegni");
    const undo = () => { vm.force(false); };
    // aspetta che la finestra di uscita sia viva: manda la qualita' ogni secondo quando disegna
    let t = performance.now();
    while (!(window.__vq && performance.now() - window.__vq.at < 3000)) {
      if (performance.now() - t > 10000) {
        let why = "l'uscita video non risponde: controlla che lo schermo di uscita sia disponibile";
        try {
          if (!(await T.core.invoke("video_is_open"))) why = "l'uscita video e' chiusa o non si e' aperta: premi il pulsante VIDEO e riprova";
          else {
            const pong = once("video-pong", 2000); T.event.emit("video-ping", {});
            const r = await pong;
            why = r ? (r.vis && r.vis !== "visible" ? "l'uscita video e' nascosta dietro altre finestre e non disegna: portala in primo piano o spostala su un altro schermo, poi riprova" : r.gl ? "l'uscita video risponde ma non disegna ancora: riprova fra qualche secondo" : `l'uscita video e' aperta ma non puo' disegnare (${r.reason || "WebGL non disponibile"})`) : "l'uscita video e' aperta ma non risponde (finestra bloccata o nascosta?)";
          }
        } catch (e) {}
        log("errore: " + why + " | stato: " + JSON.stringify(window.__vq || null)); undo(); throw new Error(why);
      }
      await sleep(100);
    }
    log("l'uscita disegna");
    let p;
    try { p = await T.core.invoke("video_rec_prepare"); } catch (e) { undo(); throw new Error(String(e)); }
    const d = dims(quality);
    const ready = once("video-rec-ready", 15000);
    T.event.emit("video-rec-start", d);
    const r = await ready;
    log("risposta dell'uscita: " + JSON.stringify(r));
    if (!r || !r.ok) { undo(); throw new Error(r ? r.error : "l'uscita video non ha iniziato la registrazione"); }
    try { await T.core.invoke("rec_start", { path: p.audio, format: "wav24" }); }
    catch (e) {
      const done = once("video-rec-done", 10000); T.event.emit("video-rec-stop", {}); await done; undo();
      throw new Error(String(e));
    }
    st = { out: path, audio: p.audio };
    log("in corso");
  }

  async function stop() {
    const s = st;
    if (!s) return null;
    st = null;
    let a;
    try { a = await T.core.invoke("rec_stop"); } catch (e) { window.VideoMode.force(false); throw e; }
    const done = once("video-rec-done", 25000);
    T.event.emit("video-rec-stop", {});
    let v = await done;
    if (!v) { try { v = { ok: true, ...(await T.core.invoke("video_rec_stop")) }; } catch (e) { v = { ok: false, error: String(e) }; } }   // l'uscita non risponde: si chiude dal programma
    window.VideoMode.force(false);
    if (!v.ok) throw new Error(`video non salvato: ${v.error}. L'audio e' in ${s.audio}`);
    // l'immagine e' stata mostrata in ritardo rispetto al suono registrato: latenza del dispositivo piu' l'allineamento scelto
    const latency = ((typeof audioLatency !== "undefined" ? audioLatency : 0) + (typeof visOffset !== "undefined" ? visOffset : 0)) * 1000;
    const m = await T.core.invoke("video_rec_mux", { audio: s.audio, out: s.out, latencyMs: latency });
    return { ...m, secs: a.secs, frames: v.frames, dropped: v.dropped, skipped: v.skipped };
  }

  window.VideoRec = { start, stop, active: () => !!st };
})();
