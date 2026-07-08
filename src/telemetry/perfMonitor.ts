// ────────────────────────────────────────────────────────────────────────────
// Moniteur de performance — la « vérité terrain » du lag
// ────────────────────────────────────────────────────────────────────────────
//
// On corrige le lag Linux/Niri à l'aveugle depuis plusieurs betas. Ce module est
// l'instrument : une boucle `requestAnimationFrame` autonome qui mesure la cadence
// RÉELLE de rendu de la webview + une sonde WebGL qui révèle le renderer GPU.
//
// La chaîne renderer (`UNMASKED_RENDERER_WEBGL`) est LE signal décisif : si elle
// contient « llvmpipe » / « swrast » / « software », c'est du rendu LOGICIEL (CPU)
// → tout est lent, quel que soit le fix de pan. C'est ce qu'on cherche à savoir.
//
// Le moniteur ne dépend PAS de Pixi : il observe la boucle du navigateur, donc il
// capture aussi les gels causés par le compositing (DMABUF), le GC, la webview…
// C'est volontaire — on veut la latence perçue, pas juste le temps de `app.render()`.

/** Instantané de perf lisible par l'UI (HUD) et par la télémétrie. */
export interface PerfSnapshot {
  /** FPS moyen lissé sur ~1 s (nombre de frames observées la dernière seconde). */
  fps: number;
  /** Pire durée de frame (ms) observée sur la dernière seconde. Le lag se voit ici. */
  worstMs: number;
  /** Durée médiane de frame (ms) sur la dernière seconde. */
  medianMs: number;
  /** Micro-saccades cumulées (28–50 ms) : petits à-coups que le seuil « jank »
   *  rate mais que l'œil perçoit (le FPS moyen reste ~60 alors que ça saccade). */
  microStutters: number;
  /** Frames « lentes » cumulées (> 50 ms ≈ sous 20 fps) depuis le démarrage. */
  jankFrames: number;
  /** Gels cumulés (> 200 ms — l'app a visiblement figé) depuis le démarrage. */
  stalls: number;
  /** Nombre total de frames observées depuis le démarrage. */
  totalFrames: number;
}

/** Infos GPU/WebGL (collectées une seule fois — ça ne change pas). */
export interface GpuInfo {
  /** Chaîne renderer démasquée (ex. « NVIDIA GeForce RTX 3060 » ou « llvmpipe »). */
  renderer: string;
  /** Chaîne vendor démasquée (ex. « NVIDIA Corporation », « Mesa »). */
  vendor: string;
  /** True si le rendu est LOGICIEL (llvmpipe/swrast/software) — cause quasi certaine de lag. */
  software: boolean;
  /** Version WebGL négociée (« webgl2 », « webgl », ou « none »). */
  api: string;
}

// Seuils (ms). 28 ms = une frame ratée à 60 Hz (micro-saccade perçue mais que le
// FPS moyen masque) ; 50 ms ≈ 20 fps (saccade nette) ; 200 ms = gel franc.
const MICRO_MS = 28;
const JANK_MS = 50;
const STALL_MS = 200;
// Fenêtre glissante d'1 s de durées de frame (à 60 fps → ~60 échantillons).
const WINDOW = 90;

let running = false;
let rafId = 0;
let last = 0;
const recent: number[] = []; // durées de frame (ms) sur la dernière ~seconde
let microStutters = 0;
let jankFrames = 0;
let stalls = 0;
let totalFrames = 0;
let gpu: GpuInfo | null = null;

function tick(now: number) {
  if (!running) return;
  if (last !== 0) {
    const dt = now - last;
    // Ignore les deltas absurdes (onglet en arrière-plan → rAF suspendu puis reprise
    // avec un dt de plusieurs secondes ; ce n'est pas du lag, c'est une pause).
    if (dt > 0 && dt < 5000) {
      recent.push(dt);
      if (recent.length > WINDOW) recent.shift();
      totalFrames++;
      if (dt > STALL_MS) stalls++;
      else if (dt > JANK_MS) jankFrames++;
      else if (dt > MICRO_MS) microStutters++;
    }
  }
  last = now;
  rafId = requestAnimationFrame(tick);
}

/** Démarre la boucle de mesure (idempotent). À appeler tôt au boot. */
export function startPerfMonitor(): void {
  if (running) return;
  running = true;
  last = 0;
  rafId = requestAnimationFrame(tick);
}

/** Arrête la boucle (rarement utile — on mesure en continu). */
export function stopPerfMonitor(): void {
  running = false;
  if (rafId) cancelAnimationFrame(rafId);
  rafId = 0;
}

/** Instantané courant. Sûr à appeler à tout moment (lecture pure). */
export function getPerfSnapshot(): PerfSnapshot {
  if (recent.length === 0) {
    return { fps: 0, worstMs: 0, medianMs: 0, microStutters, jankFrames, stalls, totalFrames };
  }
  const sorted = [...recent].sort((a, b) => a - b);
  const worst = sorted[sorted.length - 1];
  const median = sorted[Math.floor(sorted.length / 2)];
  const sum = recent.reduce((a, b) => a + b, 0);
  // fps = frames observées / durée réelle de la fenêtre (robuste aux gels).
  const fps = sum > 0 ? Math.round((recent.length / sum) * 1000) : 0;
  return {
    fps,
    worstMs: Math.round(worst),
    medianMs: Math.round(median),
    microStutters,
    jankFrames,
    stalls,
    totalFrames,
  };
}

/**
 * Interroge le GPU via un canvas WebGL jetable (mémoïsé). L'extension
 * `WEBGL_debug_renderer_info` démasque le vrai nom du GPU ; sans elle on tombe
 * sur le renderer générique (« WebKit WebGL »), moins utile mais on le renvoie.
 */
export function getGpuInfo(): GpuInfo {
  if (gpu) return gpu;
  let renderer = "inconnu";
  let vendor = "inconnu";
  let api = "none";
  try {
    const canvas = document.createElement("canvas");
    const gl =
      (canvas.getContext("webgl2") as WebGL2RenderingContext | null) ??
      (canvas.getContext("webgl") as WebGLRenderingContext | null) ??
      (canvas.getContext("experimental-webgl") as WebGLRenderingContext | null);
    if (gl) {
      api = "webgl2" in window && gl instanceof WebGL2RenderingContext ? "webgl2" : "webgl";
      const dbg = gl.getExtension("WEBGL_debug_renderer_info");
      if (dbg) {
        renderer = String(gl.getParameter(dbg.UNMASKED_RENDERER_WEBGL) ?? renderer);
        vendor = String(gl.getParameter(dbg.UNMASKED_VENDOR_WEBGL) ?? vendor);
      } else {
        renderer = String(gl.getParameter(gl.RENDERER) ?? renderer);
        vendor = String(gl.getParameter(gl.VENDOR) ?? vendor);
      }
    }
  } catch {
    /* pas de WebGL → on garde « inconnu » */
  }
  const soft = /llvmpipe|swrast|softpipe|software|microsoft basic render/i.test(
    `${renderer} ${vendor}`,
  );
  gpu = { renderer, vendor, software: soft, api };
  return gpu;
}
