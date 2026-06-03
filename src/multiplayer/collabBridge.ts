// ────────────────────────────────────────────────────────────────────────────
// Collab — pont store ↔ DocHandle automerge-repo
// ────────────────────────────────────────────────────────────────────────────
//
// Active/désactive la collaboration et fait le lien bidirectionnel entre le
// store Zustand (`_doc`) et le DocHandle partagé :
//
//   • Local → réseau : géré côté store. Quand un handle est attaché, `mutate`
//     applique les changements via `handle.change(...)`, ce qui synchronise et
//     persiste automatiquement (cf. src/store/index.ts).
//   • Réseau → local : le listener `handle.on("change")` ci-dessous pousse le
//     doc reçu (d'un pair ou du serveur) dans le store, sans le re-diffuser
//     (Automerge coupe la boucle : un change déjà connu ne régénère rien).
//
// Identité du document = code de partage `automerge:…` (handle.url). On le
// mémorise en localStorage pour offrir un lien STABLE entre deux sessions : si
// l'hôte ferme puis rouvre, il « reprend » le même document (le serveur en garde
// l'état) au lieu d'en créer un nouveau qui casserait le lien du pair.

import { getRepo } from "./repo";
import { setCollabHandle } from "./collabHandle";
import { useGlucoseStore } from "../store";
import * as A from "../store/automerge";
import type { Project } from "../types";
import type { AutomergeUrl, DocHandle, DocHandleChangePayload } from "@automerge/automerge-repo";
import { isValidAutomergeUrl } from "@automerge/automerge-repo";

const LS_KEY = "glucose:collabShareUrl";

let _changeOff: (() => void) | null = null;

/** Branche un handle comme source de vérité et adopte son état dans le store. */
function wireHandle(handle: DocHandle<Project>): void {
  // Réseau → local
  const onChange = (payload: DocHandleChangePayload<Project>) => {
    const doc = payload.doc as unknown as A.Doc<Project>;
    const st = useGlucoseStore.getState();
    if (doc === st._doc) return; // notre propre mutation locale : rien à faire
    if (st._previewHeads !== null) {
      // L'utilisateur explore la Time Machine : on met à jour le doc sous-jacent
      // mais on garde la vue figée sur l'instant prévisualisé.
      try {
        const viewed = A.viewAt<Project>(doc, st._previewHeads);
        useGlucoseStore.setState({ _doc: doc, project: viewed as unknown as Project });
      } catch {
        useGlucoseStore.setState({ _doc: doc });
      }
      return;
    }
    useGlucoseStore.setState({ _doc: doc, project: doc as unknown as Project });
  };
  handle.on("change", onChange);
  _changeOff = () => handle.off("change", onChange);

  setCollabHandle(handle);

  // Adopte immédiatement l'état courant du handle comme état du store. On repart
  // d'une pile undo/redo vierge : la session collaborative est un nouveau départ.
  const doc = handle.docSync() as unknown as A.Doc<Project>;
  useGlucoseStore.setState({
    _doc: doc,
    project: doc as unknown as Project,
    _undoStack: [],
    _redoStack: [],
    _liveEdit: false,
    _previewHeads: null,
    selectedImageIds: [],
    selectedAnnotationIds: [],
  });
}

/** URL de partage mémorisée (lien stable), ou null. */
export function getSavedShareUrl(): AutomergeUrl | null {
  const v = localStorage.getItem(LS_KEY);
  return v && isValidAutomergeUrl(v) ? v : null;
}

/**
 * Crée un NOUVEAU partage à partir du projet local courant (avec tout son
 * historique Time Machine) et renvoie le code de partage. Écrase le lien
 * mémorisé.
 */
export function createShare(): string {
  const repo = getRepo();
  const bytes = A.save(useGlucoseStore.getState()._doc);
  const handle = repo.import<Project>(bytes);
  localStorage.setItem(LS_KEY, handle.url);
  wireHandle(handle);
  return handle.url;
}

/**
 * Reprend le partage mémorisé (le serveur en détient l'état le plus à jour).
 * À utiliser quand l'hôte rouvre l'app : le lien partagé reste valide.
 */
export async function resumeShare(): Promise<string> {
  const saved = getSavedShareUrl();
  if (!saved) return createShare();
  const repo = getRepo();
  const handle = await repo.find<Project>(saved);
  await handle.whenReady();
  wireHandle(handle);
  return handle.url;
}

/**
 * Rejoint le partage d'un pair via son code. ⚠️ Remplace le projet local courant
 * par le document partagé (on adopte l'état de l'hôte).
 */
export async function joinByCode(code: string): Promise<string> {
  const url = code.trim();
  if (!isValidAutomergeUrl(url)) throw new Error("Code de partage invalide");
  const repo = getRepo();
  const handle = await repo.find<Project>(url);
  await handle.whenReady();
  localStorage.setItem(LS_KEY, handle.url);
  wireHandle(handle);
  return handle.url;
}

/**
 * Quitte la collaboration. Le `_doc` reste à l'état synchronisé courant → on
 * repart en mode solo sans perdre le travail. Ne supprime pas le lien mémorisé
 * (on pourra reprendre le partage plus tard).
 */
export function leaveCollab(): void {
  if (_changeOff) {
    _changeOff();
    _changeOff = null;
  }
  setCollabHandle(null);
  useGlucoseStore.setState({ _undoStack: [], _redoStack: [] });
}
