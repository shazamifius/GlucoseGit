// ────────────────────────────────────────────────────────────────────────────
// Phase 7.5bis — Hook de synchronisation Automerge ↔ pairs LAN
// ────────────────────────────────────────────────────────────────────────────
//
// Quand le multijoueur est ACTIF :
//   1. À chaque mutation locale du store (`_doc` change), on calcule
//      `getChanges(oldDoc, newDoc)` → bytes Automerge → on envoie à tous les
//      peers via `mp_send_patch`.
//   2. Quand un patch arrive d'un peer (event `mp:patch`), on l'applique au
//      store via `applyRemoteChanges` SANS le re-broadcaster (Automerge
//      `getChanges` après applyChanges retourne vide pour les changes déjà connus
//      → boucle naturellement coupée).
//
// IMPORTANT : on doit ignorer les *premiers* deltas après un applyRemoteChanges
// pour éviter d'inonder le réseau. Heureusement, le wrapper du store garantit
// que `applyRemoteChanges` ne push pas dans `_undoStack` ni ne crée de change
// supplémentaire — donc le diff suivant est vide et rien n'est ré-envoyé.

import { useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { useGlucoseStore } from "../store";
import * as A from "../store/automerge";

function bytesToBase64(bytes: Uint8Array): string {
  const CHUNK = 32_768;
  let bin = "";
  for (let i = 0; i < bytes.length; i += CHUNK) {
    bin += String.fromCharCode.apply(null, Array.from(bytes.subarray(i, i + CHUNK)));
  }
  return btoa(bin);
}

function base64ToBytes(b64: string): Uint8Array {
  const bin = atob(b64);
  const out = new Uint8Array(bin.length);
  for (let i = 0; i < bin.length; i++) out[i] = bin.charCodeAt(i);
  return out;
}

/**
 * Active la synchronisation multijoueur tant que `enabled === true`.
 * Le hook se débranche proprement quand `enabled` repasse à false.
 */
export function useMultiplayerSync(enabled: boolean) {
  useEffect(() => {
    if (!enabled) return;

    let prevDoc = useGlucoseStore.getState()._doc;
    let unlisten: UnlistenFn | null = null;

    // 1) Diffusion locale : à chaque mutation locale, envoie le delta
    const unsubStore = useGlucoseStore.subscribe((state) => {
      const newDoc = state._doc;
      if (newDoc === prevDoc) return;
      try {
        const changes = A.getChanges(prevDoc, newDoc);
        prevDoc = newDoc;
        for (const c of changes) {
          const b64 = bytesToBase64(c);
          invoke("mp_send_patch", { bytesB64: b64 }).catch((err) => {
            console.warn("[mp] send_patch failed:", err);
          });
        }
      } catch (e) {
        console.error("[mp] getChanges failed:", e);
      }
    });

    // 2) Réception : applique les patches venant des peers
    listen<{ from: string; bytes_b64: string }>("mp:patch", (event) => {
      try {
        const bytes = base64ToBytes(event.payload.bytes_b64);
        // Important : applyRemoteChanges est silencieux côté undo et ne
        // déclenche pas un re-broadcast (le diff suivant sera vide).
        useGlucoseStore.getState().applyRemoteChanges([bytes]);
        // Met à jour notre référence prevDoc pour ne pas recalculer un faux delta
        prevDoc = useGlucoseStore.getState()._doc;
      } catch (e) {
        console.error("[mp] applyChanges from peer failed:", e);
      }
    }).then((u) => { unlisten = u; });

    return () => {
      unsubStore();
      if (unlisten) unlisten();
    };
  }, [enabled]);
}
