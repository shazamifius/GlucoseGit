// ────────────────────────────────────────────────────────────────────────────
// Phase 7 — Wrapper Automerge minimal pour la migration progressive du store.
// ────────────────────────────────────────────────────────────────────────────
//
// On expose une API petite et explicite (pas un mega-namespace) pour que la
// migration des actions Zustand (~30 sites) soit mécanique :
//
//   import * as A from "../store/automerge";
//   const doc = A.create<Project>(initial);
//   const next = A.change(doc, draft => { draft.boards.push(newBoard); });
//   const bytes = A.save(next);     // → Uint8Array (binaire `.glucose` v2)
//   const back = A.load<Project>(bytes);
//   const merged = A.merge(localDoc, remoteDoc);
//
// Les helpers "isAutomergeDoc / asPlain" servent à interfacer avec l'UI React
// qui s'attend à un objet plain JS (pas un Proxy Automerge).

import { next as Automerge } from "@automerge/automerge";

export type Doc<T> = Automerge.Doc<T>;
export type Patch = Automerge.Patch;
export type Heads = Automerge.Heads;

/**
 * Crée un nouveau document Automerge à partir d'un objet plain JS.
 * Utilisé une seule fois au démarrage si pas de fichier à charger, ou pour
 * convertir un projet legacy JSON → CRDT.
 */
export function create<T extends object>(initial: T): Doc<T> {
  let doc = Automerge.init<T>();
  doc = Automerge.change(doc, "init", (d) => {
    Object.assign(d as object, initial);
  });
  return doc;
}

/**
 * Applique une mutation sur le document. Le `mutator` reçoit un draft mutable
 * (style Immer). Renvoie un NOUVEAU document (immutable côté API).
 *
 * @param message - libellé court qui sera enregistré dans l'historique
 *                  Automerge (visible dans la Time Machine).
 */
export function change<T>(doc: Doc<T>, message: string, mutator: (draft: T) => void): Doc<T> {
  return Automerge.change(doc, message, (d) => mutator(d as T));
}

/**
 * Applique plusieurs changements externes (reçus par réseau / merge LAN)
 * en bloc. Si on n'a aucun changement, renvoie `doc` inchangé.
 */
export function applyChanges<T>(doc: Doc<T>, changes: Uint8Array[]): Doc<T> {
  if (changes.length === 0) return doc;
  const [next] = Automerge.applyChanges(doc, changes);
  return next;
}

/**
 * Récupère les changes du doc `newDoc` qui ne sont pas dans `oldDoc`.
 * Utilisé pour calculer le delta à envoyer aux peers après une mutation locale.
 */
export function getChanges<T>(oldDoc: Doc<T>, newDoc: Doc<T>): Uint8Array[] {
  return Automerge.getChanges(oldDoc, newDoc);
}

/** Sérialise le document en binaire (format `.glucose` v2). */
export function save<T>(doc: Doc<T>): Uint8Array {
  return Automerge.save(doc);
}

/** Charge un document à partir du binaire. */
export function load<T>(bytes: Uint8Array): Doc<T> {
  return Automerge.load<T>(bytes);
}

/** Fusionne un document distant dans le local — résolution CRDT automatique. */
export function merge<T>(local: Doc<T>, remote: Doc<T>): Doc<T> {
  return Automerge.merge(local, remote);
}

/**
 * Clone le document avec un nouvel acteur (id distinct). Indispensable pour
 * simuler deux utilisateurs qui éditent en parallèle, ou pour brancher la
 * Time Machine "preview" sans toucher au document source.
 */
export function clone<T>(doc: Doc<T>): Doc<T> {
  return Automerge.clone(doc);
}

/**
 * Renvoie une copie « plain JS » du document, utilisable par React/PixiJS
 * sans risquer de muter le doc Automerge par mégarde.
 *
 * Utile pour les composants UI qui s'attendent à un Object/Array natif. Coût
 * O(n) en deep copy — à n'utiliser qu'à la frontière React.
 *
 * R-EMB-01 — Gère les `Uint8Array` (utilisés pour Project.blobs) via un
 * marqueur sentinelle dans le sérialiseur. Un JSON.parse(JSON.stringify)
 * naïf transforme `Uint8Array` en objet `{0:1, 1:2, ...}` — corruption
 * silencieuse des blobs. Le marqueur `__u8: number[]` est rebuilt en
 * Uint8Array à la lecture.
 */
export function asPlain<T>(doc: Doc<T>): T {
  return JSON.parse(
    JSON.stringify(doc, (_key, value) => {
      if (value instanceof Uint8Array) {
        return { __u8: Array.from(value) };
      }
      return value;
    }),
    (_key, value) => {
      if (
        value
        && typeof value === "object"
        && Array.isArray((value as { __u8?: unknown }).__u8)
      ) {
        return new Uint8Array((value as { __u8: number[] }).__u8);
      }
      return value;
    }
  );
}

/** Récupère l'historique sous forme de liste de commits Automerge. */
export function history<T>(doc: Doc<T>) {
  return Automerge.getHistory(doc);
}

/**
 * Récupère l'état du document à un index d'historique donné. Utilisé par la
 * Time Machine pour afficher un état passé sans détruire l'état courant.
 */
export function viewAt<T>(doc: Doc<T>, heads: Automerge.Heads): Doc<T> {
  return Automerge.view(doc, heads);
}

/**
 * Récupère les `heads` (identifiants des commits courants) du document.
 * Utile pour comparer deux documents ou positionner la Time Machine.
 */
export function getHeads<T>(doc: Doc<T>): Automerge.Heads {
  return Automerge.getHeads(doc);
}
