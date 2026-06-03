// CLEANUP A-01 — Bus d'événements typé.
//
// Avant : ~30 `window.addEventListener("glucose:...")` non typés répartis dans
// 12+ fichiers, sans inventaire ni autocomplétion. Difficile à débugger,
// fragile en StrictMode (double listeners).
//
// Après : un module unique avec Map<EventName, Set<handler>>, API typée
// `bus.emit(name, payload)` et `bus.on(name, handler) → unsubscribe`.
// Compatibilité conservée : tous les events sont aussi dispatchés sur `window`
// pour les anciens consommateurs qui n'ont pas encore migré.
//
// Migration progressive : appeler `bus.emit("teleport", payload)` côté émetteur
// suffit. L'inscription se fait via `bus.on()` ou via le hook `useBus()`.

// ════════════════════════════════════════════════════════════════════════════
// Inventaire complet des events (1 seul endroit pour les voir tous)
// ════════════════════════════════════════════════════════════════════════════

export type GlucoseEvents = {
  // Pan/zoom
  "viewport-changed": { x: number; y: number; scale: number };
  "fit-view": void;
  "jump-viewport": { boardId: string; viewport: { x: number; y: number; scale: number } };
  "pan-viewport-to": { x: number; y: number };
  "zoom-to-annotation": { annId: string; padding?: number };

  // Phase 4 — miroirs
  "teleport-to-mirror-original": {
    mirrorOf: string;
    type: "annotation" | "image" | "folder";
  };

  // Phase 5 — flèches premium
  "open-arrow-description": { arrowId: string; screenX: number; screenY: number };
  "portal-jump": { boardId: string; targetId?: string };

  // Flèches — interactions de hover et de preview
  "hover-arrow": null | {
    sourceId?: string;
    sourceBlockId?: string;
    targetId?: string;
    targetBlockId?: string;
    sourceTextSel?: string;
    targetTextSel?: string;
  };
  "arrow-target-preview": null | { annId: string; blockId?: string };

  // Layout / sélection / ghost
  "layout-preview": null | unknown;
  "zone-selected": { x: number; y: number; w: number; h: number };

  // UI
  "delete-selected-folder": void;
  "trigger-import": void;
};

type EventName = keyof GlucoseEvents;
type Handler<K extends EventName> = (payload: GlucoseEvents[K]) => void;

// ════════════════════════════════════════════════════════════════════════════
// Implémentation
// ════════════════════════════════════════════════════════════════════════════

class GlucoseBus {
  private handlers = new Map<EventName, Set<(payload: unknown) => void>>();

  on<K extends EventName>(name: K, handler: Handler<K>): () => void {
    let set = this.handlers.get(name);
    if (!set) {
      set = new Set();
      this.handlers.set(name, set);
    }
    set.add(handler as (payload: unknown) => void);
    return () => {
      set?.delete(handler as (payload: unknown) => void);
    };
  }

  emit<K extends EventName>(name: K, payload: GlucoseEvents[K]): void {
    // Notifie les abonnés du bus
    const set = this.handlers.get(name);
    if (set) {
      for (const h of set) {
        try { h(payload); } catch (err) { console.error(`[bus] handler error for ${name}`, err); }
      }
    }
    // Compat ascendante : on dispatch aussi sur window pour les anciens listeners
    if (typeof window !== "undefined") {
      window.dispatchEvent(new CustomEvent(`glucose:${name}`, { detail: payload }));
    }
  }
}

export const bus = new GlucoseBus();
