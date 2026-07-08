// ────────────────────────────────────────────────────────────────────────────
// Télémétrie — OPT-IN strict, anonyme, best-effort
// ────────────────────────────────────────────────────────────────────────────
//
// But : nous donner enfin une vue réelle du terrain (lag, gels, erreurs, matériel,
// renderer GPU) pour faire évoluer Glucose sans deviner. RIEN n'est envoyé tant que
// l'utilisateur n'a pas explicitement consenti (popup au 1er lancement). Toutes les
// fonctions ci-dessous sont des NO-OP si le consentement n'est pas « granted ».
//
// Confidentialité :
//   • identifiant client = UUID aléatoire local (aucun nom, mail, chemin de fichier) ;
//   • aucun contenu de document n'est jamais collecté — seulement des métriques ;
//   • l'envoi passe par Rust (`telemetry_send`), qui ajoute la clé d'ingestion
//     ÉCRITURE-SEULE + l'OS/arch/version faisant autorité, puis POST vers le serveur.

import { invoke } from "@tauri-apps/api/core";
import { getPerfSnapshot, getGpuInfo } from "./perfMonitor";

const CONSENT_KEY = "glucose.telemetry.consent";
const CID_KEY = "glucose.telemetry.cid";

export type Consent = "granted" | "denied" | "unset";

/** Un événement télémétrie (hétérogène : session/perf/error/event). */
type Event = { kind: string; ts: number; [k: string]: unknown };
/** Événement fourni par un collecteur (sans `ts`, ajouté à l'enfilage). */
type EventInput = { kind: string; [k: string]: unknown };

let queue: Event[] = [];
let session = "";
let started = false;
let flushTimer: ReturnType<typeof setInterval> | null = null;
let perfTimer: ReturnType<typeof setInterval> | null = null;

// ── Consentement ────────────────────────────────────────────────────────────

function readConsent(): Consent {
  try {
    const v = localStorage.getItem(CONSENT_KEY);
    return v === "granted" || v === "denied" ? v : "unset";
  } catch {
    return "unset";
  }
}

/** État courant : « unset » = pas encore demandé (→ afficher la popup de consentement). */
export function getConsentState(): Consent {
  return readConsent();
}

export function isTelemetryEnabled(): boolean {
  return readConsent() === "granted";
}

/** Enregistre le choix de l'utilisateur. Si accordé, démarre l'envoi + session_start. */
export function setTelemetryConsent(granted: boolean): void {
  try {
    localStorage.setItem(CONSENT_KEY, granted ? "granted" : "denied");
  } catch {
    /* localStorage indispo → on ne pourra pas envoyer, tant pis */
  }
  if (granted) {
    ensureStarted();
    void reportSessionStart();
  } else {
    stopTelemetry();
  }
}

// ── Identité anonyme ──────────────────────────────────────────────────────────

function clientId(): string {
  try {
    let id = localStorage.getItem(CID_KEY);
    if (!id) {
      id = crypto.randomUUID();
      localStorage.setItem(CID_KEY, id);
    }
    return id;
  } catch {
    return "anon";
  }
}

function sessionId(): string {
  if (!session) {
    session = (crypto.randomUUID?.() ?? String(Math.random()).slice(2)).slice(0, 18);
  }
  return session;
}

// ── File d'envoi ──────────────────────────────────────────────────────────────

function enqueue(ev: EventInput): void {
  if (!isTelemetryEnabled()) return;
  queue.push({ ...ev, ts: Date.now() });
  // Plafond dur : jamais plus de 200 événements en mémoire (best-effort — on
  // abandonne les plus vieux plutôt que de fuir de la RAM si le serveur est down).
  if (queue.length > 200) queue = queue.slice(-200);
  if (queue.length >= 50) void flush();
}

async function flush(): Promise<void> {
  if (!isTelemetryEnabled() || queue.length === 0) return;
  const batch = queue;
  queue = [];
  const body = JSON.stringify({ cid: clientId(), session: sessionId(), events: batch });
  try {
    await invoke("telemetry_send", { body });
  } catch {
    // Réseau coupé / serveur down / endpoint non configuré → on jette ce lot.
    // La télémétrie ne doit JAMAIS gêner l'app ni s'accumuler indéfiniment.
  }
}

function ensureStarted(): void {
  if (started) return;
  started = true;
  // Flush régulier (30 s) + à chaque passage en arrière-plan (l'utilisateur ferme).
  flushTimer = setInterval(() => void flush(), 30_000);
  // Rapport de perf périodique (2 min) — courbe FPS/gels dans le temps.
  perfTimer = setInterval(() => reportPerf("periodic"), 120_000);
  document.addEventListener("visibilitychange", () => {
    if (document.visibilityState === "hidden") void flush();
  });
}

function stopTelemetry(): void {
  started = false;
  queue = [];
  if (flushTimer) clearInterval(flushTimer);
  if (perfTimer) clearInterval(perfTimer);
  flushTimer = perfTimer = null;
}

// ── Collecteurs publics ───────────────────────────────────────────────────────

/**
 * À appeler une fois au boot. Attache les capteurs d'erreurs globaux (toujours,
 * mais ils sont no-op sans consentement) et, si déjà consenti, démarre l'envoi.
 */
export function initTelemetry(): void {
  // Capteurs d'erreurs globaux — gate interne sur le consentement.
  window.addEventListener("error", (e) => {
    reportError("window.onerror", e.message, e.error?.stack);
  });
  window.addEventListener("unhandledrejection", (e) => {
    const r = e.reason;
    reportError("unhandledrejection", r?.message ?? String(r), r?.stack);
  });
  if (isTelemetryEnabled()) {
    ensureStarted();
    void reportSessionStart();
  }
}

/** Envoie l'empreinte machine (matériel + GPU + environnement) en début de session. */
export async function reportSessionStart(): Promise<void> {
  if (!isTelemetryEnabled()) return;
  ensureStarted();
  const gpu = getGpuInfo();
  let specs: unknown = null;
  let wayland: unknown = null;
  try {
    specs = await invoke("system_specs");
  } catch {
    /* non-Tauri / web */
  }
  try {
    wayland = await invoke("is_wayland");
  } catch {
    /* non-Tauri / web */
  }
  enqueue({
    kind: "session_start",
    ua: navigator.userAgent,
    lang: navigator.language,
    screen: { w: window.innerWidth, h: window.innerHeight, dpr: window.devicePixelRatio || 1 },
    gpu,
    wayland,
    specs,
  });
  void flush();
}

/** Rapporte une erreur (exception JS, boundary React, échec critique). */
export function reportError(source: string, message: unknown, stack?: unknown): void {
  enqueue({
    kind: "error",
    source,
    message: String(message ?? "").slice(0, 500),
    stack: stack != null ? String(stack).slice(0, 2000) : undefined,
  });
}

/** Rapporte un instantané de perf (FPS/gels). `context` : « periodic », « pan », … */
export function reportPerf(context: string): void {
  const p = getPerfSnapshot();
  // Rien d'exploitable si l'app vient de démarrer (aucune frame mesurée).
  if (p.totalFrames < 30) return;
  enqueue({ kind: "perf", context, ...p });
}

/** Événement d'usage générique (ouverture d'un panneau, action clé…). */
export function track(name: string, data?: Record<string, unknown>): void {
  enqueue({ kind: "event", name, ...(data ?? {}) });
}
