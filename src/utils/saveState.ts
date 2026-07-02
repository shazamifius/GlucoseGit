// ────────────────────────────────────────────────────────────────────────────
// SAVE-A — Enregistrement incrémental (anti-freeze Ctrl+S).
// ────────────────────────────────────────────────────────────────────────────
//
// Problème : `A.save(doc)` sérialise TOUT l'historique Automerge à chaque
// enregistrement → coût O(historique) sur le thread principal → freeze sur une
// grosse session. (cf. memory image-storage-git-bundle)
//
// Solution : un fichier `.glucose` peut être un « save complet » suivi de
// « changements » ajoutés à la fin — `A.load()` relit la concaténation et
// reconstruit le document à l'identique (vérifié empiriquement). Donc :
//   • 1er save / « Enregistrer sous » / fichier différent → save COMPLET (truncate).
//   • saves suivants sur le même fichier → on n'écrit que le DELTA depuis le
//     dernier save (`A.getChanges`), AJOUTÉ à la fin du fichier. Coût O(édits).
//
// Compaction : pour que le fichier ne gonfle pas indéfiniment, quand le total des
// deltas ajoutés depuis le dernier save complet dépasse `COMPACT_RATIO ×` la
// taille du dernier save complet, on refait un save complet (qui recompacte).
//
// Ce module est PUR (aucune I/O Tauri) → entièrement testable côté Node.
// L'écriture disque réelle (write/append) est faite par l'appelant (project.ts).

import * as A from "../store/automerge";
import type { Project } from "../types";

interface SaveBaseline {
  /** Fichier auquel ce baseline correspond. */
  path: string;
  /** Doc tel qu'il était au dernier écrit disque (base du prochain getChanges). */
  doc: A.Doc<Project>;
  /** Taille (octets) du dernier save COMPLET écrit pour ce fichier. */
  fullSize: number;
  /** Total des deltas ajoutés depuis le dernier save complet. */
  appendedSize: number;
}

let _baseline: SaveBaseline | null = null;

/** Au-delà de ce ratio (deltas accumulés / taille du dernier full), on recompacte
 *  via un save complet. 1 = le fichier ne dépasse jamais ~2× sa forme compacte. */
const COMPACT_RATIO = 1;

export type SaveMode = "full" | "incremental";

export interface SavePlan {
  mode: SaveMode;
  /** Octets à écrire. `full` → truncate ; `incremental` → append. Vide si rien à faire. */
  bytes: Uint8Array;
  /** Git #1 Phase 3 — taille (octets) des SEULS changements Automerge depuis le
   *  dernier écrit disque (toujours le « petit » nombre, même sur une compaction
   *  full). 0 = rien de nouveau, ou nouvelle ligne de base (fichier neuf/save-as).
   *  Sert à mesurer « l'ampleur » cumulée pour les jalons auto. */
  deltaBytes: number;
}

/** Réinitialise tout (tests, nouveau projet, ou chargement nécessitant un full). */
export function resetSaveState(): void {
  _baseline = null;
}

/**
 * Pose le baseline après un CHARGEMENT depuis disque dont le fichier correspond
 * EXACTEMENT au doc en mémoire (v2 sans migration). Permet au 1er Ctrl+S d'être
 * incrémental. Si le fichier ne correspond pas au doc (migration), l'appelant
 * doit appeler `resetSaveState()` à la place.
 */
export function markLoaded(path: string, doc: A.Doc<Project>, fileSize: number): void {
  _baseline = { path, doc, fullSize: fileSize, appendedSize: 0 };
}

function concat(chunks: Uint8Array[]): Uint8Array {
  let total = 0;
  for (const c of chunks) total += c.length;
  const out = new Uint8Array(total);
  let off = 0;
  for (const c of chunks) { out.set(c, off); off += c.length; }
  return out;
}

/**
 * Décide quoi écrire pour enregistrer `doc` dans `path`, SANS faire d'I/O.
 *
 * - Pas de baseline, ou fichier différent → save COMPLET.
 * - Sinon, delta depuis le dernier écrit. Si rien n'a changé → `incremental`
 *   avec `bytes` vide (l'appelant n'écrit rien). Si les deltas accumulés
 *   deviennent trop gros → save COMPLET (compaction).
 */
export function planSave(doc: A.Doc<Project>, path: string): SavePlan {
  const b = _baseline;
  if (!b || b.path !== path) {
    return { mode: "full", bytes: A.save(doc), deltaBytes: 0 };
  }
  const changes = A.getChanges(b.doc, doc);
  if (changes.length === 0) {
    return { mode: "incremental", bytes: new Uint8Array(0), deltaBytes: 0 };
  }
  const delta = concat(changes);
  if (b.appendedSize + delta.length > b.fullSize * COMPACT_RATIO) {
    // Compaction : on réécrit tout le doc, mais l'AMPLEUR réelle de ce save reste
    // le petit delta (pas la taille du full) → évite un faux jalon auto.
    return { mode: "full", bytes: A.save(doc), deltaBytes: delta.length };
  }
  return { mode: "incremental", bytes: delta, deltaBytes: delta.length };
}

/**
 * Enregistre le nouveau baseline APRÈS une écriture disque réussie. À appeler
 * avec le `doc` et le `plan` effectivement écrits.
 */
export function commitSave(path: string, doc: A.Doc<Project>, plan: SavePlan): void {
  if (plan.mode === "full") {
    _baseline = { path, doc, fullSize: plan.bytes.length, appendedSize: 0 };
    return;
  }
  // incremental
  if (_baseline && _baseline.path === path) {
    _baseline.doc = doc;
    _baseline.appendedSize += plan.bytes.length;
  } else {
    // Garde-fou : pas censé arriver (un incremental implique un baseline existant).
    _baseline = { path, doc, fullSize: plan.bytes.length, appendedSize: 0 };
  }
}

/** Lecture du baseline courant (tests / debug). */
export function _peekBaseline(): { path: string; fullSize: number; appendedSize: number } | null {
  return _baseline
    ? { path: _baseline.path, fullSize: _baseline.fullSize, appendedSize: _baseline.appendedSize }
    : null;
}
