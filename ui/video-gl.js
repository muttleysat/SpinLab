/* Compositore WebGL dell'uscita video: prende le immagini dei due deck (video o scheda del brano) e le mescola con la transizione
   scelta in base alla posizione del crossfader. E' un modulo a se': non sa nulla di deck o di Tauri, quindi il motore di disegno si
   puo' sostituire (per esempio con uno nativo) senza toccare le transizioni. Shader in GLSL ES 1.00 (WebGL 1) per la massima compatibilita'. */
(() => {
  const VS = `attribute vec2 aPos; varying vec2 vUv; void main(){ vUv = aPos*0.5+0.5; gl_Position = vec4(aPos,0.0,1.0); }`;
  const FS = `
#ifdef GL_FRAGMENT_PRECISION_HIGH
precision highp float;
#else
precision mediump float;
#endif
varying vec2 vUv;
uniform sampler2D uA;
uniform sampler2D uB;
uniform vec2 uRes;
uniform vec2 uSizeA;
uniform vec2 uSizeB;
uniform float uT;      // 0 = solo deck A, 1 = solo deck B
uniform float uTime;
uniform float uBeat;   // fase del battito (0..1) del deck dominante
uniform float uHasA;
uniform float uHasB;
uniform int uType;
const float PI = 3.14159265;
const vec4 BLACK = vec4(0.0, 0.0, 0.0, 1.0);

float hash(float n) { return fract(sin(n * 12.9898) * 43758.5453); }

// immagine adattata allo schermo senza deformarla (bande nere ai lati se serve)
vec2 fitUv(vec2 size, vec2 uv) {
  float ca = uRes.x / uRes.y, va = size.x / max(size.y, 1.0);
  vec2 p = uv - 0.5;
  if (va > ca) p.y *= va / ca; else p.x *= ca / va;
  return p + 0.5;
}
vec4 fa(vec2 uv) {
  if (uSizeA.x < 1.0) return BLACK;
  vec2 p = fitUv(uSizeA, uv);
  if (p.x < 0.0 || p.x > 1.0 || p.y < 0.0 || p.y > 1.0) return BLACK;
  return texture2D(uA, p);
}
vec4 fb(vec2 uv) {
  if (uSizeB.x < 1.0) return BLACK;
  vec2 p = fitUv(uSizeB, uv);
  if (p.x < 0.0 || p.x > 1.0 || p.y < 0.0 || p.y > 1.0) return BLACK;
  return texture2D(uB, p);
}

vec4 trans(vec2 uv) {
  float t = uT;
  float k = sin(PI * t);            // 0 agli estremi, 1 a meta': intensita' degli effetti che deformano
  float asp = uRes.x / uRes.y;

  if (uType == 1) { return t < 0.5 ? fa(uv) : fb(uv); }                                   // taglio

  if (uType == 2) { return t < 0.5 ? fa(uv) * (1.0 - 2.0 * t) : fb(uv) * (2.0 * t - 1.0); }  // dissolvenza al nero

  if (uType == 3) {                                                                        // flash bianco
    vec4 w = vec4(1.0);
    return t < 0.5 ? mix(fa(uv), w, 2.0 * t) : mix(w, fb(uv), 2.0 * t - 1.0);
  }

  if (uType == 4) {                                                                        // wipe con bordo sfumato
    float e = t * 1.1 - 0.05;
    float m = 1.0 - smoothstep(e - 0.04, e + 0.04, uv.x);
    return mix(fa(uv), fb(uv), m);
  }

  if (uType == 5) {                                                                        // zoom a spinta
    float sa = 1.0 + t * 1.2, sb = 0.6 + 0.4 * t;
    vec2 ua = (uv - 0.5) / sa + 0.5, ub = (uv - 0.5) / sb + 0.5;
    return mix(fa(ua), fb(ub), smoothstep(0.25, 0.75, t));
  }

  if (uType == 6) {                                                                        // glitch: righe spostate, RGB sdoppiato, blocchi
    float tt = floor(uTime * 20.0);
    float row = floor(uv.y * 28.0);
    float jr = hash(row + tt * 3.1);
    float off = step(0.7, jr) * (jr - 0.5) * 0.25 * k;
    vec2 u = vec2(uv.x + off, uv.y);
    float blk = hash(floor(uv.x * 10.0) * 7.0 + row * 1.3 + tt);
    float sel = step(blk, t);
    float sp = 0.012 * k;
    vec4 ca = vec4(fa(u + vec2(sp, 0.0)).r, fa(u).g, fa(u - vec2(sp, 0.0)).b, 1.0);
    vec4 cb = vec4(fb(u + vec2(sp, 0.0)).r, fb(u).g, fb(u - vec2(sp, 0.0)).b, 1.0);
    return mix(ca, cb, sel);
  }

  if (uType == 7) {                                                                        // pixel: si scompone in blocchi e si ricompone
    float n = mix(1920.0, 18.0, k);
    vec2 g = vec2(n, n / asp);
    vec2 u = (floor(uv * g) + 0.5) / g;
    return mix(fa(u), fb(u), smoothstep(0.45, 0.55, t));
  }

  if (uType == 8) {                                                                        // caleidoscopio
    vec2 p = (uv - 0.5) * vec2(asp, 1.0);
    float r = length(p);
    float a = atan(p.y, p.x) + uTime * 0.4 * k;
    float seg = 6.2831853 / 6.0;
    a = mod(a, seg);
    a = abs(a - seg * 0.5);
    vec2 q = vec2(cos(a), sin(a)) * r;
    q.x /= asp;
    vec2 u = mix(uv, q + 0.5, k);
    return mix(fa(u), fb(u), smoothstep(0.4, 0.6, t));
  }

  if (uType == 9) {                                                                        // ondulazione come acqua
    vec2 d = uv - 0.5;
    float r = length(d * vec2(asp, 1.0));
    float w = sin(r * 40.0 - uTime * 8.0) * 0.02 * k;
    vec2 u = uv + normalize(d + vec2(0.00001)) * w;
    return mix(fa(u), fb(u), smoothstep(0.35, 0.65, t));
  }

  if (uType == 10) {                                                                       // cubo 3D: due facce che ruotano
    float xb = max(1.0 - t, 0.001);
    if (uv.x < xb) {
      float s = uv.x / xb;
      float h = 1.0 - 0.3 * k * s;
      float vy = (uv.y - 0.5) / h + 0.5;
      if (abs(vy - 0.5) > 0.5) return BLACK;
      return fa(vec2(s, vy)) * (1.0 - 0.5 * k);
    }
    float s = (uv.x - xb) / max(1.0 - xb, 0.001);
    float h = (1.0 - 0.3 * k) + 0.3 * k * s;
    float vy = (uv.y - 0.5) / h + 0.5;
    if (abs(vy - 0.5) > 0.5) return BLACK;
    return fb(vec2(s, vy)) * (1.0 - 0.5 * k);
  }

  if (uType == 11) {                                                                       // vortice
    vec2 p = (uv - 0.5) * vec2(asp, 1.0);
    float r = length(p);
    float ang = k * 5.0 * (1.0 - smoothstep(0.0, 0.8, r));
    float c = cos(ang), s = sin(ang);
    p = vec2(c * p.x - s * p.y, s * p.x + c * p.y);
    vec2 u = vec2(p.x / asp, p.y) + 0.5;
    return mix(fa(u), fb(u), smoothstep(0.4, 0.6, t));
  }

  if (uType == 12) {                                                                       // stroboscopio sul ritmo
    float ph = fract(uBeat);
    float useB = t >= 0.999 ? 1.0 : (t <= 0.001 ? 0.0 : step(ph, t));
    float flash = exp(-ph * 9.0) * 0.4 * k;
    return mix(fa(uv), fb(uv), useB) + vec4(flash);
  }

  return mix(fa(uv), fb(uv), t);                                                           // 0: dissolvenza
}

void main() {
  vec2 uv = vUv;
  vec4 c;
  if (uHasA < 0.5 && uHasB < 0.5) c = BLACK;
  else if (uHasB < 0.5) c = fa(uv);
  else if (uHasA < 0.5) c = fb(uv);
  else c = trans(uv);
  gl_FragColor = vec4(c.rgb, 1.0);
}`;

  function create(canvas) {
    let gl = null;
    try { gl = canvas.getContext("webgl", { alpha: false, antialias: false, powerPreference: "high-performance" }); } catch (e) {}
    if (!gl) { VideoGL.error = "WebGL non disponibile"; return null; }
    const shader = (type, src) => {
      const s = gl.createShader(type); gl.shaderSource(s, src); gl.compileShader(s);
      if (!gl.getShaderParameter(s, gl.COMPILE_STATUS)) throw new Error(gl.getShaderInfoLog(s) || "errore shader");
      return s;
    };
    let prog;
    try {
      prog = gl.createProgram();
      gl.attachShader(prog, shader(gl.VERTEX_SHADER, VS));
      gl.attachShader(prog, shader(gl.FRAGMENT_SHADER, FS));
      gl.linkProgram(prog);
      if (!gl.getProgramParameter(prog, gl.LINK_STATUS)) throw new Error(gl.getProgramInfoLog(prog) || "errore di collegamento");
    } catch (e) { VideoGL.error = String(e.message || e); return null; }
    gl.useProgram(prog);
    const buf = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, buf);
    gl.bufferData(gl.ARRAY_BUFFER, new Float32Array([-1, -1, 1, -1, -1, 1, 1, 1]), gl.STATIC_DRAW);
    const loc = gl.getAttribLocation(prog, "aPos");
    gl.enableVertexAttribArray(loc);
    gl.vertexAttribPointer(loc, 2, gl.FLOAT, false, 0, 0);
    gl.pixelStorei(gl.UNPACK_FLIP_Y_WEBGL, true);
    const U = {};
    ["uA", "uB", "uRes", "uSizeA", "uSizeB", "uT", "uTime", "uBeat", "uHasA", "uHasB", "uType"].forEach((n) => (U[n] = gl.getUniformLocation(prog, n)));
    const tex = [0, 1].map((i) => {
      const t = gl.createTexture();
      gl.activeTexture(gl.TEXTURE0 + i);
      gl.bindTexture(gl.TEXTURE_2D, t);
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR);
      gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, 1, 1, 0, gl.RGBA, gl.UNSIGNED_BYTE, new Uint8Array([0, 0, 0, 255]));
      return t;
    });
    gl.uniform1i(U.uA, 0);
    gl.uniform1i(U.uB, 1);
    const size = [[0, 0], [0, 0]];
    return {
      /** Dimensioni di disegno (pixel). */
      resize(w, h) { canvas.width = w; canvas.height = h; gl.viewport(0, 0, w, h); },
      /** Carica l'immagine del deck i (elemento video, canvas, immagine). Ritorna false se la scheda grafica la rifiuta. */
      upload(i, src, w, h) {
        try {
          gl.activeTexture(gl.TEXTURE0 + i);
          gl.bindTexture(gl.TEXTURE_2D, tex[i]);
          gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, gl.RGBA, gl.UNSIGNED_BYTE, src);
          size[i] = [w, h];
          return true;
        } catch (e) { VideoGL.error = String(e.message || e); return false; }
      },
      clear(i) { size[i] = [0, 0]; },
      /** p: { t: 0..1, type, time, beat, has: [bool, bool] } */
      draw(p) {
        gl.uniform2f(U.uRes, canvas.width, canvas.height);
        gl.uniform2f(U.uSizeA, size[0][0], size[0][1]);
        gl.uniform2f(U.uSizeB, size[1][0], size[1][1]);
        gl.uniform1f(U.uT, p.t);
        gl.uniform1f(U.uTime, p.time);
        gl.uniform1f(U.uBeat, p.beat || 0);
        gl.uniform1f(U.uHasA, p.has[0] ? 1 : 0);
        gl.uniform1f(U.uHasB, p.has[1] ? 1 : 0);
        gl.uniform1i(U.uType, p.type | 0);
        gl.activeTexture(gl.TEXTURE0); gl.bindTexture(gl.TEXTURE_2D, tex[0]);
        gl.activeTexture(gl.TEXTURE1); gl.bindTexture(gl.TEXTURE_2D, tex[1]);
        gl.drawArrays(gl.TRIANGLE_STRIP, 0, 4);
      },
    };
  }
  const VideoGL = { create, error: "" };
  window.VideoGL = VideoGL;
})();
