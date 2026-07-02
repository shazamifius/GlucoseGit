// ────────────────────────────────────────────────────────────────────────────
// Git #1 Phase 4 partie 2 — COMPACTION de l'historique fin
// ────────────────────────────────────────────────────────────────────────────
//
// Problème : un `.glucose` garde TOUT l'historique d'ops Automerge depuis la
// création du projet (chaque geste ajoute des ops, `A.save` resérialise tout). La
// pile d'undo est bornée à 200 (mémoire), mais le FICHIER, lui, gonfle sans fin.
//
// Compacter = repartir d'un doc de LIGNÉE NEUVE à partir de l'état courant :
// `A.create(A.asPlain(doc))` donne un doc dont l'historique COMMENCE maintenant
// (un unique change « init » contenant l'état complet) → `A.save` minuscule, zéro
// trou. C'est exactement le chemin déjà éprouvé par `store.loadProject` (legacy).
//
// C'est le morceau DÉLICAT (réécriture de l'op-set, même classe que le panic
// Ctrl+Z). Garde-fous BÉTON, dans cet ordre :
//   1. SOLO UNIQUEMENT — jamais sur le doc d'un handle collab (lignée divergente
//      + un pair ne pourrait plus merger). cf. mémoire collab-automerge-repo.
//   2. VÉRIFIER le roundtrip AVANT de toucher au fichier : on sérialise, on
//      recharge exactement comme le fera l'ouverture (`loadResilient`), et on
//      EXIGE un état rechargé identique. Sinon → on jette, fichier intact.
//   3. Poser un jalon durable « avant compaction » : l'historique complet
//      pré-compaction survit sur disque comme point de reprise (filet Phase 4 p1).
//   4. Écriture ATOMIQUE (tmp + rename) : un crash ne laisse jamais un fichier à
//      moitié écrit.
//
// L'undo n'est PAS cassé par la lignée neuve : `undo()` fait du forward-revert (il
// lit `A.asPlain(snapshot)` et le ré-applique EN AVANT sur le doc vivant) → c'est
// lignée-agnostique (cf. store/index.ts, mémoire undo-forward-revert-wasm-panic).
//
// I/O Tauri via plugin-fs uniquement (pas de node:fs) → testable en fs mémoire.

import { writeFile, rename } from "@tauri-apps/plugin-fs";
import * as A from "../store/automerge";
import type { Project } from "../types";
import { getCollabHandle } from "../multiplayer/collabHandle";
import { saveVersion } from "./versions";
import { markLoaded } from "./saveState";
import { resetAutoVersionAccumulator } from "./autoVersion";

/** Échec de compaction. Quand elle est levée, le fichier n'a PAS été touché. */
export class CompactionError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "CompactionError";
  }
}

/**
 * Égalité structurelle profonde, INDÉPENDANTE de l'ordre des clés (l'ordre
 * d'insertion des clés d'une map Automerge peut différer entre le doc source et
 * le doc reconstruit). Gère les `Uint8Array` (blobs d'images) octet par octet.
 * Sert à prouver que l'état rechargé du compacté est bien identique à l'original.
 */
function deepEqual(a: unknown, b: unknown): boolean {
  if (a === b) return true;
  if (a instanceof Uint8Array || b instanceof Uint8Array) {
    if (!(a instanceof Uint8Array) || !(b instanceof Uint8Array)) return false;
    if (a.length !== b.length) return false;
    for (let i = 0; i < a.length; i++) if (a[i] !== b[i]) return false;
    return true;
  }
  if (Array.isArray(a) || Array.isArray(b)) {
    if (!Array.isArray(a) || !Array.isArray(b) || a.length !== b.length) return false;
    for (let i = 0; i < a.length; i++) if (!deepEqual(a[i], b[i])) return false;
    return true;
  }
  if (a && b && typeof a === "object" && typeof b === "object") {
    const ao = a as Record<string, unknown>;
    const bo = b as Record<string, unknown>;
    const ak = Object.keys(ao);
    const bk = Object.keys(bo);
    if (ak.length !== bk.length) return false;
    for (const k of ak) {
      if (!Object.prototype.hasOwnProperty.call(bo, k)) return false;
      if (!deepEqual(ao[k], bo[k])) return false;
    }
    return true;
  }
  return false;
}

/**
 * Construit le doc compacté (lignée neuve à partir de l'état courant) et VÉRIFIE
 * le roundtrip. PUR (Automerge réel, aucune I/O) → testable directement.
 *
 * @throws {CompactionError} si le compacté ne se recharge pas à l'identique. Dans
 *         ce cas l'appelant NE DOIT PAS remplacer le fichier (le doc vivant reste
 *         la seule source de vérité).
 */
export function compactDoc(doc: A.Doc<Project>): { compacted: A.Doc<Project>; bytes: Uint8Array } {
  const before = A.asPlain<Project>(doc);
  const compacted = A.create<Project>(before);
  const bytes = A.save(compacted);

  // Roundtrip : on recharge EXACTEMENT comme le fera l'ouverture du fichier.
  let reloaded: A.Doc<Project>;
  try {
    const res = A.loadResilient<Project>(bytes);
    if (res.recovered) {
      throw new CompactionError("le compacté ne se recharge pas proprement (fin récupérée)");
    }
    reloaded = res.doc;
  } catch (e) {
    if (e instanceof CompactionError) throw e;
    throw new CompactionError(`le compacté est illisible : ${(e as Error).message}`);
  }

  const after = A.asPlain<Project>(reloaded);
  if (!deepEqual(before, after)) {
    throw new CompactionError("l'état rechargé du compacté diffère de l'état courant");
  }
  return { compacted, bytes };
}

/** Résultat d'une compaction réussie (tailles en octets, avant/après). */
export interface CompactionResult {
  compacted: A.Doc<Project>;
  bytes: Uint8Array;
  before: number;
  after: number;
}

/**
 * Compacte l'historique du projet `path` de bout en bout, avec tous les garde-fous.
 *
 * Renvoie le résultat à ADOPTER (le store doit remplacer son `_doc` par
 * `compacted`), ou `null` si le doc est déjà compact (rien à gagner → fichier non
 * touché).
 *
 * @throws {CompactionError} en collaboration (mode solo requis) ou si le roundtrip
 *         de vérification échoue — dans les deux cas le fichier reste INTACT.
 */
export async function runCompaction(path: string, doc: A.Doc<Project>): Promise<CompactionResult | null> {
  // 1) SOLO uniquement — jamais réécrire l'op-set d'un handle collab.
  if (getCollabHandle()) {
    throw new CompactionError("compaction indisponible en collaboration (mode solo requis)");
  }

  const currentSize = A.save(doc).length;

  // 2) Construit + VÉRIFIE le compacté AVANT toute I/O (jette si roundtrip KO).
  const { compacted, bytes } = compactDoc(doc);

  // Déjà compact (aucun gain) → on ne touche à rien : ni jalon, ni fichier.
  if (bytes.length >= currentSize) return null;

  // 3) Jalon de secours AVANT de remplacer le fichier vivant : l'historique
  //    complet pré-compaction survit sur disque comme point de reprise.
  await saveVersion(path, doc, "avant compaction", "auto");

  // 4) Écriture ATOMIQUE : le fichier n'est remplacé qu'après roundtrip vérifié.
  const tmp = `${path}.tmp`;
  await writeFile(tmp, bytes);
  await rename(tmp, path);

  // Le fichier sur disque == save(compacted) EXACTEMENT → le prochain Ctrl+S peut
  // être incrémental contre cette nouvelle base. Le compteur d'ampleur repart de
  // zéro (on vient d'écrire un jalon).
  markLoaded(path, compacted, bytes.length);
  resetAutoVersionAccumulator();

  return { compacted, bytes, before: currentSize, after: bytes.length };
}
