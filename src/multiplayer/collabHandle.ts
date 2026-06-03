// ────────────────────────────────────────────────────────────────────────────
// Collab — accès au DocHandle actif (source de vérité quand la collab est ON)
// ────────────────────────────────────────────────────────────────────────────
//
// Module volontairement minimal et sans dépendance vers le store ou le bridge,
// pour éviter tout import circulaire : le store lit `getCollabHandle()` dans son
// chemin chaud (`mutate`/`undo`/`redo`), le bridge écrit via `setCollabHandle()`.
//
// - handle === null  → mode SOLO (comportement historique, `_doc` local).
// - handle !== null  → mode COLLAB (le handle automerge-repo possède le doc :
//                      synchronisation + persistance gérées par la lib).

import type { DocHandle } from "@automerge/automerge-repo";
import type { Project } from "../types";

let _handle: DocHandle<Project> | null = null;

export function getCollabHandle(): DocHandle<Project> | null {
  return _handle;
}

export function setCollabHandle(handle: DocHandle<Project> | null): void {
  _handle = handle;
}

export function isCollabActive(): boolean {
  return _handle !== null;
}

/** Code de partage du handle actif (`automerge:…`), ou null si pas en collab. */
export function getActiveShareUrl(): string | null {
  return _handle ? _handle.url : null;
}
