/* Visualizzazioni generative per i deck senza video: animazioni astratte originali (shader) che pulsano sul battito della griglia
   e reagiscono al livello del deck. Disegnano su un canvas a parte, che la finestra di uscita carica come texture del deck.
   Scene: 0 anelli, 1 tunnel, 2 onde, 3 griglia al neon, 4 stelle, 5 plasma, 6 barre, 7 esagoni, 8 fluido, 9 raggi. */
(() => {
  const VS = `attribute vec2 aPos; varying vec2 vUv; void main(){ vUv = aPos*0.5+0.5; gl_Position = vec4(aPos,0.0,1.0); }`;
  const FS = `
#ifdef GL_FRAGMENT_PRECISION_HIGH
precision highp float;
#else
precision mediump float;
#endif
varying vec2 vUv;
uniform vec2 uRes;
uniform float uTime;
uniform float uBeat;    // fase del battito 0..1
uniform float uBar;     // fase della battuta 0..1
uniform float uLevel;   // livello del deck 0..1
uniform float uHue;     // colore base (dalla tonalita') 0..1
uniform int uScene;
const float PI = 3.14159265;

vec3 hsv(float h, float s, float v) {
  vec3 c = clamp(abs(mod(h * 6.0 + vec3(0.0, 4.0, 2.0), 6.0) - 3.0) - 1.0, 0.0, 1.0);
  return v * mix(vec3(1.0), c, s);
}
float hash21(vec2 p) { return fract(sin(dot(p, vec2(127.1, 311.7))) * 43758.5453); }
float vnoise(vec2 p) {
  vec2 i = floor(p), f = fract(p);
  f = f * f * (3.0 - 2.0 * f);
  return mix(mix(hash21(i), hash21(i + vec2(1.0, 0.0)), f.x), mix(hash21(i + vec2(0.0, 1.0)), hash21(i + vec2(1.0, 1.0)), f.x), f.y);
}

vec3 scene(vec2 uv, vec2 p, float pulse, float lv) {
  float t = uTime;
  float r = length(p);
  if (uScene == 0) {                                   // anelli che pulsano
    float rings = sin(r * 18.0 - t * 2.0 - pulse * 3.0) * 0.5 + 0.5;
    return hsv(uHue + r * 0.15, 0.8, smoothstep(0.2, 1.0, rings) * (0.3 + 0.7 * lv + 0.5 * pulse)) * (1.1 - r * 0.4);
  }
  if (uScene == 1) {                                   // tunnel
    float a = atan(p.y, p.x), z = 1.0 / (r + 0.05);
    float u = a / 6.2832 * 8.0, v = z + t * (1.0 + pulse);
    float ch = mod(floor(u) + floor(v), 2.0);
    return hsv(uHue + v * 0.02, 0.7, mix(0.08, 0.9, ch)) * clamp(r * 2.2, 0.0, 1.0) * (0.7 + 0.5 * pulse);
  }
  if (uScene == 2) {                                   // onde luminose
    vec3 col = vec3(0.0);
    for (int i = 0; i < 6; i++) {
      float fi = float(i);
      float y = sin(p.x * 3.0 + t * (1.0 + 0.3 * fi) + fi) * 0.14 * (0.4 + lv + pulse * 0.6) + (fi - 2.5) * 0.2;
      float d = abs(p.y - y);
      col += hsv(uHue + 0.06 * fi, 0.8, 1.0) * (0.012 / (d + 0.004));
    }
    return clamp(col, 0.0, 1.5);
  }
  if (uScene == 3) {                                   // griglia al neon con sole
    vec3 col = hsv(uHue, 0.6, 0.12 + 0.1 * (p.y + 1.0));
    if (p.y < 0.0) {
      float z = 1.0 / (-p.y + 0.02);
      float gx = abs(fract(p.x * z * 0.35) - 0.5), gz = abs(fract(z * 0.25 - t * (0.6 + pulse * 0.5)) - 0.5);
      float line = smoothstep(0.46, 0.5, max(gx, gz));
      col = hsv(uHue + 0.5, 0.8, 0.06) + hsv(uHue + 0.5, 0.7, 1.0) * line * (0.4 + 0.6 * pulse) * clamp(1.5 / z + 0.2, 0.0, 1.0);
    } else {
      float sun = smoothstep(0.42 + 0.03 * pulse, 0.40, length(p - vec2(0.0, 0.35)));
      col += hsv(uHue + 0.08, 0.7, 1.0) * sun * (0.7 + 0.5 * lv);
    }
    return col;
  }
  if (uScene == 4) {                                   // stelle in avvicinamento
    vec3 col = vec3(0.0);
    for (int i = 0; i < 4; i++) {
      float fi = float(i);
      float depth = fract(fi * 0.25 + t * 0.15 * (1.0 + pulse * 0.5));
      float sc = mix(20.0, 0.5, depth);
      vec2 q = p * sc + fi * 17.3, id = floor(q), f = fract(q) - 0.5;
      float h = hash21(id);
      vec2 o = (vec2(hash21(id + 1.3), hash21(id + 2.7)) - 0.5) * 0.6;
      float s = smoothstep(0.1 * (0.5 + depth), 0.0, length(f - o)) * step(0.6, h);
      col += hsv(uHue + h * 0.2, 0.5, 1.0) * s * (0.3 + depth);
    }
    return col * (0.7 + 0.6 * lv);
  }
  if (uScene == 5) {                                   // plasma
    float v = sin(p.x * 3.0 + t) + sin(p.y * 4.0 - t * 1.3) + sin((p.x + p.y) * 3.0 + t * 0.7) + sin(r * 5.0 - t * 2.0 + pulse * 2.0);
    return hsv(uHue + v * 0.08, 0.85, 0.5 + 0.3 * sin(v * 1.5 + t) + 0.3 * pulse * (0.4 + lv));
  }
  if (uScene == 6) {                                   // barre
    float x = uv.x * 32.0, id = floor(x), f = fract(x);
    float h = 0.12 + 0.72 * vnoise(vec2(id * 3.1, t * 2.0)) * (0.4 + lv) + pulse * 0.18 * hash21(vec2(id, 1.0));
    float bar = step(uv.y, h) * step(0.12, f) * step(f, 0.88);
    return hsv(uHue + id / 64.0, 0.8, bar * (0.45 + 0.55 * (uv.y / max(h, 0.01))));
  }
  if (uScene == 7) {                                   // esagoni che lampeggiano
    vec2 q = p * 4.0, rr = vec2(1.0, 1.732), hh = rr * 0.5;
    vec2 a = mod(q, rr) - hh, b = mod(q - hh, rr) - hh;
    vec2 g = dot(a, a) < dot(b, b) ? a : b;
    vec2 id = q - g;
    float d = max(dot(abs(g), vec2(0.5, 0.866)), abs(g.x));
    float rnd = hash21(id);
    float on = step(0.65, fract(rnd + floor(t * 2.0) * 0.37)) * (0.3 + 0.9 * pulse) + 0.08;
    float edge = smoothstep(0.40, 0.46, d);
    return hsv(uHue + rnd * 0.3, 0.8, mix(on, 0.9 * (0.3 + lv), edge));
  }
  if (uScene == 8) {                                   // fluido
    vec2 q = p * 2.0;
    for (int i = 0; i < 3; i++) { q += 0.6 * vec2(vnoise(q + t * 0.3), vnoise(q.yx - t * 0.25)); }
    float v = vnoise(q);
    return hsv(uHue + v * 0.3, 0.8, 0.3 + 0.7 * v) * (0.7 + 0.5 * pulse);
  }
  // 9: raggi
  float a = atan(p.y, p.x);
  float rays = smoothstep(0.4, 0.6, abs(sin(a * 10.0 + t * 0.5 + uBar * 6.2832)));
  float g = rays * (0.3 + 0.7 * exp(-r * 1.2));
  return hsv(uHue + r * 0.1, 0.8, g * (0.6 + pulse + lv * 0.6)) + hsv(uHue, 0.5, 0.25) * exp(-r * 3.0) * (0.5 + pulse);
}

void main() {
  float asp = uRes.x / uRes.y;
  vec2 uv = vUv;
  vec2 p = (uv - 0.5) * vec2(asp, 1.0) * 2.0;
  float pulse = exp(-uBeat * 5.0);
  vec3 c = scene(uv, p, pulse, uLevel);
  c *= 1.0 - 0.25 * dot(uv - 0.5, uv - 0.5) * 4.0;     // vignettatura leggera
  gl_FragColor = vec4(clamp(c, 0.0, 1.0), 1.0);
}`;

  function create(w, h) {
    const canvas = document.createElement("canvas");
    canvas.width = w; canvas.height = h;
    let gl = null;
    try { gl = canvas.getContext("webgl", { alpha: false, antialias: false, preserveDrawingBuffer: true }); } catch (e) {}
    if (!gl) { VideoVis.error = "WebGL non disponibile"; return null; }
    const sh = (type, src) => { const s = gl.createShader(type); gl.shaderSource(s, src); gl.compileShader(s); if (!gl.getShaderParameter(s, gl.COMPILE_STATUS)) throw new Error(gl.getShaderInfoLog(s) || "errore shader"); return s; };
    let prog;
    try {
      prog = gl.createProgram();
      gl.attachShader(prog, sh(gl.VERTEX_SHADER, VS)); gl.attachShader(prog, sh(gl.FRAGMENT_SHADER, FS)); gl.linkProgram(prog);
      if (!gl.getProgramParameter(prog, gl.LINK_STATUS)) throw new Error(gl.getProgramInfoLog(prog) || "errore di collegamento");
    } catch (e) { VideoVis.error = String(e.message || e); return null; }
    gl.useProgram(prog);
    const buf = gl.createBuffer(); gl.bindBuffer(gl.ARRAY_BUFFER, buf);
    gl.bufferData(gl.ARRAY_BUFFER, new Float32Array([-1, -1, 1, -1, -1, 1, 1, 1]), gl.STATIC_DRAW);
    const loc = gl.getAttribLocation(prog, "aPos"); gl.enableVertexAttribArray(loc); gl.vertexAttribPointer(loc, 2, gl.FLOAT, false, 0, 0);
    const U = {}; ["uRes", "uTime", "uBeat", "uBar", "uLevel", "uHue", "uScene"].forEach((n) => (U[n] = gl.getUniformLocation(prog, n)));
    gl.viewport(0, 0, w, h);
    return {
      canvas, w, h,
      sync() { gl.readPixels(0, 0, 1, 1, gl.RGBA, gl.UNSIGNED_BYTE, new Uint8Array(4)); },
      /** p: { scene 0..9, time, beat, bar, level, hue } */
      render(p) {
        gl.uniform2f(U.uRes, w, h); gl.uniform1f(U.uTime, p.time); gl.uniform1f(U.uBeat, p.beat || 0); gl.uniform1f(U.uBar, p.bar || 0);
        gl.uniform1f(U.uLevel, Math.max(0, Math.min(1, p.level || 0))); gl.uniform1f(U.uHue, p.hue ?? 0.6); gl.uniform1i(U.uScene, p.scene | 0);
        gl.drawArrays(gl.TRIANGLE_STRIP, 0, 4);
      },
    };
  }
  const VideoVis = { create, error: "" };
  window.VideoVis = VideoVis;
})();
