"use strict";
/* ============================================================ LINGUA DELL'INTERFACCIA
   L'interfaccia e' scritta in italiano. Per le altre lingue un dizionario (ui/i18n/<lingua>.js) associa a ogni frase italiana la sua
   traduzione: la frase italiana fa da chiave (come gettext), quindi il codice resta scritto come prima.
   - Le scritte fisse di index.html e quelle create dal JavaScript (finestre, menu, avvisi, messaggi del motore) si traducono mentre
     compaiono, con un MutationObserver; in italiano il modulo resta spento (nessun costo).
   - Le frasi con parti variabili (numeri, nomi) si traducono con modelli: "Caricato in {a} ms" -> "Loaded in {a} ms".
   - Titoli, artisti, cartelle e playlist non si traducono mai: i loro contenitori hanno l'attributo data-noi18n.
   Lingua: localStorage "spinlab.lang" = auto | it | en; auto = la lingua del sistema (italiano se e' italiano, altrimenti inglese). */

const I18N = (() => {
  const DICTS = { en: () => window.SL_I18N_EN || { exact: {}, patterns: [] } };
  const ATTRS = ["title", "placeholder", "alt", "aria-label"];
  // testo: niente dentro campi modificabili; attributi (placeholder, title...): anche dei campi, ma mai nelle zone data-noi18n
  const SKIP = "script,style,textarea,input,[data-noi18n],[contenteditable]";
  const SKIP_EL = "script,style,[data-noi18n]";
  let lang = "it", exact = new Map(), pats = [], memo = new Map(), obs = null;
  const orig = new WeakMap(); // nodo -> testo italiano originale (per tornare all'italiano senza riavviare)

  const chosen = () => { try { return localStorage.getItem("spinlab.lang") || "auto"; } catch (e) { return "auto"; } };
  const resolve = (c) => (c === "it" || c === "en" ? c : /^it\b/i.test(navigator.language || "") ? "it" : "en");

  function load(l) {
    exact = new Map(); pats = []; memo = new Map();
    if (l === "it" || !DICTS[l]) return;
    const d = DICTS[l]();
    for (const [k, v] of Object.entries(d.exact || {})) exact.set(norm(k), v);
    // modello: le parti {nome} diventano gruppi; il resto si confronta alla lettera
    for (const [k, v] of d.patterns || []) {
      const names = [];
      const rx = new RegExp("^" + norm(k).replace(/[.*+?^$()|[\]\\]/g, "\\$&").replace(/\\?\{(\w+)\\?\}/g, (_, n) => { names.push(n); return "([\\s\\S]*?)"; }) + "$");
      pats.push({ rx, names, out: v });
    }
  }
  const norm = (s) => s.replace(/\s+/g, " ").trim();

  /** traduce una frase (o la restituisce com'e' se non c'e' nel dizionario) */
  function t(s) {
    if (lang === "it" || s == null) return s;
    const str = String(s), key = norm(str);
    if (!key || !/[A-Za-zÀ-ú]/.test(key)) return str;
    if (memo.has(key)) { const r = memo.get(key); return r == null ? str : keepSpaces(str, r); }
    let r = exact.get(key);
    if (r == null && key.includes(" · ")) {
      // frase composta da pezzi uniti con « · » (per esempio «Nota 10 · canale 3»): si traduce pezzo per pezzo
      const parts = key.split(" · "), tr = parts.map((x) => t(x));
      if (tr.some((x, i) => x !== parts[i])) r = tr.join(" · ");
    }
    if (r == null) {
      for (const p of pats) {
        const m = p.rx.exec(key);
        if (m) { r = p.out.replace(/\{(\w+)\}/g, (_, n) => { const i = p.names.indexOf(n); return i >= 0 ? t(m[i + 1]) : ""; }); break; }
      }
    }
    if (memo.size > 4000) memo.clear();
    memo.set(key, r ?? null);
    return r == null ? str : keepSpaces(str, r);
  }
  // mantiene gli spazi iniziali e finali del testo originale (contano nell'impaginazione dei nodi di testo)
  const keepSpaces = (src, r) => (src.match(/^\s*/)[0]) + r + (src.match(/\s*$/)[0]);

  function skip(el) { return !el || (el.closest && el.closest(SKIP)); }
  function skipEl(el) { return !el || (el.closest && el.closest(SKIP_EL)); }

  function doText(n) {
    if (skip(n.parentElement)) return;
    const cur = n.nodeValue;
    const base = orig.has(n) && orig.get(n).tr === cur ? orig.get(n).it : cur; // gia' tradotto da noi: si riparte dall'italiano
    const tr = t(base);
    if (tr !== cur) { orig.set(n, { it: base, tr }); n.nodeValue = tr; }
  }
  function doAttrs(el) {
    if (skipEl(el)) return;
    let o = orig.get(el);
    for (const a of ATTRS) {
      if (!el.hasAttribute(a)) continue;
      const cur = el.getAttribute(a), prev = o && o[a];
      const base = prev && prev.tr === cur ? prev.it : cur;
      const tr = t(base);
      if (tr !== cur) { if (!o) { o = {}; orig.set(el, o); } o[a] = { it: base, tr }; el.setAttribute(a, tr); }
    }
  }
  function walk(root) {
    if (root.nodeType === 3) return doText(root);
    if (root.nodeType !== 1 || skipEl(root)) return;
    doAttrs(root);
    const tw = document.createTreeWalker(root, NodeFilter.SHOW_ELEMENT | NodeFilter.SHOW_TEXT, {
      acceptNode: (n) => (n.nodeType === 1 && skipEl(n) ? NodeFilter.FILTER_REJECT : NodeFilter.FILTER_ACCEPT),
    });
    for (let n = tw.nextNode(); n; n = tw.nextNode()) n.nodeType === 3 ? doText(n) : doAttrs(n);
  }
  // torna all'italiano: rimette i testi originali salvati
  function revert(root) {
    const tw = document.createTreeWalker(root, NodeFilter.SHOW_ELEMENT | NodeFilter.SHOW_TEXT);
    for (let n = tw.currentNode; n; n = tw.nextNode()) {
      const o = orig.get(n);
      if (!o) continue;
      if (n.nodeType === 3) { if (n.nodeValue === o.tr) n.nodeValue = o.it; }
      else for (const a of ATTRS) if (o[a] && n.getAttribute(a) === o[a].tr) n.setAttribute(a, o[a].it);
      orig.delete(n);
    }
  }

  function start() {
    walk(document.body);
    if (obs) return;
    obs = new MutationObserver((ms) => {
      for (const m of ms) {
        if (m.type === "childList") m.addedNodes.forEach(walk);
        else if (m.type === "characterData") doText(m.target);
        else if (m.type === "attributes" && m.target.nodeType === 1) doAttrs(m.target);
      }
    });
    obs.observe(document.body, { childList: true, subtree: true, characterData: true, attributes: true, attributeFilter: ATTRS });
  }
  function stop() { if (obs) { obs.disconnect(); obs = null; } }

  function apply(c) {
    const l = resolve(c);
    document.documentElement.lang = l;
    if (l === lang && (l === "it" || obs)) return;
    stop();
    if (lang !== "it") revert(document.body);
    lang = l;
    load(l);
    if (l !== "it") start();
    window.dispatchEvent(new CustomEvent("sl-lang", { detail: l }));
  }

  /** cambia la lingua (auto | it | en), la ricorda e la applica subito */
  function set(c) {
    try { localStorage.setItem("spinlab.lang", c); } catch (e) {}
    apply(c);
  }

  return { t, set, apply: () => apply(chosen()), chosen, get lang() { return lang; } };
})();
// nel codice: I18N.t("frase italiana") (T e' gia' usato per Tauri)
