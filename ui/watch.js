"use strict";
/* ============================================================ OROLOGIO (Wear OS)
   Esiste solo nell'app Android: MainActivity aggiunge SpinlabWatch (vedi WatchBridge.kt). Altrove questo file non fa nulla.
   - ogni 250 ms manda all'orologio tempo, posizione nella battuta e stem del deck selezionato (quello di FX e stem);
   - dall'orologio riceve i volumi degli stem da applicare e risponde con un'ack, per misurare il ritardo del comando. */
(() => {
  if (!window.SpinlabWatch) return;

  const beatIdx = (d, t) => {
    const b = d.beats; if (!b.length || t < b[0]) return -1;
    let lo = 0, hi = b.length - 1;
    while (lo < hi) { const m = (lo + hi + 1) >> 1; if (b[m] <= t) lo = m; else hi = m - 1; }
    return lo;
  };

  /* millisecondi alla prossima battuta "1" della griglia (0 se il deck non suona o non ha griglia) */
  const msToNextBar = (i) => {
    const d = decks[i], s = d?.snap;
    if (!s || !s.playing || !d.beats.length) return 0;
    const pos = currentPos(i), b = beatIdx(d, pos);
    if (b < 0) return 0;
    let n = b + 1;
    while (n < d.beats.length && ((((n - d.downbeat) % 4) + 4) % 4) !== 0) n++;
    if (n >= d.beats.length) return 0;
    return Math.max(0, ((d.beats[n] - pos) / (1 + s.tempo_pct / 100)) * 1000);
  };
  const IPC_LEAD = 12;   // ms: tempo del comando verso il motore (stima: la sua misura reale e' nella diagnostica, Ctrl+Maiusc+D)

  window.spinlabWatch = (txt) => {
    try {
      const m = JSON.parse(txt);
      if (m.t === "stems" && Array.isArray(m.gains) && m.gains.length === 4) {
        const i = target, d = decks[i];
        const gains = m.gains.map((g) => Math.min(1, Math.max(0, +g || 0)));
        if (m.q === "bar") {
          // rientro del drop: il comando parte un attimo prima della battuta, cosi' gli stem entrano proprio sul "1"
          const ms = msToNextBar(i);
          setTimeout(() => {
            d.stGains = gains;
            d.stGains.forEach((g, k) => { if (g > 0) d.stMem[k] = g; });
            updateStemsUI(true);
            inv("deck_stems", { deck: i, gains: d.stGains }).catch((e) => toast(String(e)));
          }, Math.max(0, ms - IPC_LEAD));
        } else {
          d.stGains = gains;
          d.stGains.forEach((g, k) => { if (g > 0) d.stMem[k] = g; });
          touchStems(i);
        }
      }
      if (m.id != null) SpinlabWatch.ack(String(m.id));
    } catch (e) { /* messaggio non valido */ }
  };

  setInterval(() => {
    const i = target, d = decks[i], s = d?.snap;
    if (!s) return;
    let bpos = 0;
    if (s.loaded && s.bpm_now > 0 && d.beats.length > 1) {
      // la vibrazione deve cadere sul battito che si SENTE: la posizione del motore e' un po' avanti all'audio in uscita
      const rate = 1 + s.tempo_pct / 100;
      const pos = currentPos(i) - (typeof audioLatency === "number" ? audioLatency : 0) * rate;
      const b = beatIdx(d, pos);
      if (b >= 0 && b + 1 < d.beats.length) {
        const phase = Math.min(1, Math.max(0, (pos - d.beats[b]) / (d.beats[b + 1] - d.beats[b])));
        bpos = ((((b - d.downbeat) % 4) + 4) % 4) + phase;
      }
    }
    SpinlabWatch.state(JSON.stringify({
      deck: i, bpm: s.loaded ? s.bpm_now : 0, bpos, playing: !!(s.loaded && s.playing),
      stems: s.stem_gains || [1, 1, 1, 1], title: s.name || "",
    }));
  }, 250);
})();
