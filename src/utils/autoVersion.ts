// ────────────────────────────────────────────────────────────────────────────
// Git #1 Phase 3 — Jalons AUTO « à l'ampleur »
// ────────────────────────────────────────────────────────────────────────────
//
// Idée (tranchée avec l'user) : Glucose pose TOUT SEUL un jalon durable quand une
// GROSSE modification a été faite depuis le dernier jalon — déclencheur = AMPLEUR
// du changement (volume d'octets de delta Automerge), PAS le temps qui passe.
//
// On accumule ici le volume réellement écrit à chaque sauvegarde (le « petit »
// delta, cf. SavePlan.deltaBytes). Quand le cumul dépasse le seuil, on écrit un
// jalon durable `kind="auto"` (un `.glucose` complet indépendant, cf. versions.ts)
// puis on élague les vieux jalons auto. Le compteur repart de zéro à chaque jalon
// (manuel OU auto) et à chaque chargement de projet.
//
// Registre module-level (même motif que currentPath.ts) : l'état vit hors-React,
// lu/écrit par le pipeline de save (project.ts) sans cycle d'import.

import { LIMITS } from "../constants";
import * as A from "../store/automerge";
import type { Project } from "../types";
import { saveVersion, pruneAutoVersions } from "./versions";

let _accumBytes = 0;
let _inFlight = false;

/** Comptabilise le volume de modifications d'une sauvegarde réussie. */
export function noteSavedDelta(bytes: number): void {
  if (bytes > 0) _accumBytes += bytes;
}

/** Remet le compteur « depuis le dernier jalon » à zéro (jalon posé, ou nouveau
 *  projet chargé / enregistré sous). */
export function resetAutoVersionAccumulator(): void {
  _accumBytes = 0;
}

/** Octets accumulés depuis le dernier jalon (tests / debug). */
export function _peekAutoAccum(): number {
  return _accumBytes;
}

/**
 * Si le volume accumulé depuis le dernier jalon dépasse le seuil d'ampleur, écrit
 * un jalon durable AUTO et élague les anciens. NON bloquante : à appeler en
 * fire-and-forget après une sauvegarde réussie. `A.save` (dans saveVersion) est en
 * LECTURE SEULE → sûr même sur le doc d'un handle collab (contrairement à
 * `A.change`, cf. memory collab-automerge-repo).
 */
export async function maybeCreateAutoVersion(path: string, doc: A.Doc<Project>): Promise<void> {
  if (_inFlight) return;
  if (_accumBytes < LIMITS.AUTO_VERSION_DELTA_BYTES) return;

  _inFlight = true;
  // Remise à zéro AVANT l'await : une sauvegarde concurrente peut ré-accumuler
  // pendant l'écriture du jalon sans provoquer un second déclenchement.
  _accumBytes = 0;
  try {
    const stamp = new Date().toLocaleString("fr-FR", { dateStyle: "short", timeStyle: "short" });
    await saveVersion(path, doc, `auto ${stamp}`, "auto");
    await pruneAutoVersions(path, LIMITS.AUTO_VERSION_KEEP);
  } catch (e) {
    console.warn("[autoVersion] échec du jalon auto:", e);
  } finally {
    _inFlight = false;
  }
}
