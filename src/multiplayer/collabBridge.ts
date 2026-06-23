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

import { getRepo, ensureConnected } from "./repo";
import {
  setCollabHandle, getCollabHandle, suppressAutoReconnect, isAutoReconnectSuppressed,
} from "./collabHandle";
import { useGlucoseStore } from "../store";
import * as A from "../store/automerge";
import { resetSaveState } from "../utils/saveState";
import type { Project } from "../types";
import type { AutomergeUrl, DocHandle, DocHandleChangePayload } from "@automerge/automerge-repo";
import { isValidAutomergeUrl } from "@automerge/automerge-repo";

const LS_KEY = "glucose:collabShareUrl";

let _changeOff: (() => void) | null = null;

/** Branche un handle comme source de vérité et adopte son état dans le store. */
function wireHandle(handle: DocHandle<Project>): void {
  // Réseau → local
  const onChange = (payload: DocHandleChangePayload<Project>) => {
    // IMPORTANT — RE-ENTRANCE WASM : automerge-repo émet « change » AU MILIEU de
    // son propre `handle.change` (le doc Rust est encore emprunté en mutation).
    // Toucher le doc/le store de façon SYNCHRONE ici re-rentre dans l'objet WASM
    // → « recursive use of an object … unsafe aliasing in rust », qui CORROMPT le
    // handle et fige toute mutation suivante (c'était le gel du Ctrl+Z). On diffère
    // d'un microtask : l'emprunt WASM est alors relâché. `payload.doc` est un
    // snapshot Automerge immuable, donc sûr à lire plus tard.
    const doc = payload.doc as unknown as A.Doc<Project>;
    queueMicrotask(() => {
      const st = useGlucoseStore.getState();
      if (doc === st._doc) return; // notre propre mutation locale, déjà adoptée
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
    });
  };
  handle.on("change", onChange);
  _changeOff = () => handle.off("change", onChange);

  setCollabHandle(handle);

  // CORRUPTION-GUARD — entrer en collab change la LIGNÉE du doc (`_doc` devient le
  // doc du handle, souvent fusionné avec le distant). Le baseline d'enregistrement
  // incrémental pointe encore sur l'ancien doc/fichier ; sans reset, le prochain
  // Ctrl+S/autosave calculerait un delta `getChanges(ancien, nouveau)` qui référence
  // des ops absentes du fichier → fichier abîmé (« MissingOps » au rechargement).
  // On force donc un SAVE COMPLET propre au prochain enregistrement.
  resetSaveState();

  // Adopte immédiatement l'état courant du handle comme état du store. On repart
  // d'une pile undo/redo vierge : la session collaborative est un nouveau départ.
  const doc = handle.doc() as unknown as A.Doc<Project>;
  useGlucoseStore.setState({
    _doc: doc,
    project: doc as unknown as Project,
    _undoStack: [],
    _redoStack: [],
    _liveEdit: false,
    _previewHeads: null,
    selectedImageIds: [],
    selectedAnnotationIds: [],
    // Caméra : on repart de celle du doc partagé (override local vidé). Ensuite
    // chaque utilisateur a sa propre caméra, jamais synchronisée.
    localViewports: {},
  });
}

/** URL de partage mémorisée (lien stable), ou null. */
export function getSavedShareUrl(): AutomergeUrl | null {
  const v = localStorage.getItem(LS_KEY);
  return v && isValidAutomergeUrl(v) ? v : null;
}

/** `repo.find` avec attente de la connexion serveur + quelques essais (le doc
 *  peut mettre un instant à être répliqué depuis l'hôte vers le serveur). */
async function findShared(url: AutomergeUrl): Promise<DocHandle<Project>> {
  const repo = getRepo();
  await ensureConnected();
  for (let i = 0; i < 4; i++) {
    try {
      const handle = await repo.find<Project>(url);
      await handle.whenReady();
      return handle;
    } catch (e) {
      console.warn("[collab] find échec, nouvel essai…", e);
      await new Promise((r) => setTimeout(r, 1200));
    }
  }
  throw new Error("Chaîne introuvable. Vérifie le code, et que l'autre a bien créé la chaîne (en restant connecté au moins une fois).");
}

/**
 * Crée un NOUVEAU partage à partir du projet local courant (avec tout son
 * historique Time Machine) et renvoie le code de partage. Écrase le lien
 * mémorisé. Attend la connexion au serveur pour que le doc y soit bien poussé
 * avant qu'on diffuse le code.
 */
export async function createShare(): Promise<string> {
  const repo = getRepo();
  const bytes = A.save(useGlucoseStore.getState()._doc);
  const handle = repo.import<Project>(bytes);
  // Embarque le lien DANS le document : il sera sauvegardé par Ctrl+S et
  // permettra la reconnexion auto en rouvrant le fichier.
  handle.change((d) => { (d as Project).collabUrl = handle.url; });
  localStorage.setItem(LS_KEY, handle.url);
  wireHandle(handle);
  await ensureConnected(); // garantit que le serveur reçoit le doc
  return handle.url;
}

/**
 * Reconnexion AUTOMATIQUE depuis le `collabUrl` embarqué dans le document
 * courant (ex : on vient d'ouvrir un fichier .glucose qui appartenait à une
 * chaîne). Fusionne les éventuelles modifications faites hors-ligne dans le
 * document partagé (aucune perte), puis branche le handle.
 */
export async function reconnectFromDoc(): Promise<boolean> {
  if (getCollabHandle()) return false; // déjà en collab
  const st = useGlucoseStore.getState();
  const url = (st.project as Project).collabUrl;
  if (!url || !isValidAutomergeUrl(url)) return false;
  if (isAutoReconnectSuppressed(url)) return false; // l'utilisateur a quitté exprès
  const localDoc = st._doc; // état local (peut contenir des édits hors-ligne)
  const handle = await findShared(url as AutomergeUrl);
  // Fusionne le local dans la chaîne — clone d'abord (interdit de muter le doc
  // du handle directement). merge() combine les deux historiques (ils partagent
  // une racine commune puisque le fichier vient de cette chaîne).
  try {
    handle.update((d) => A.merge(A.clone(d as unknown as A.Doc<Project>), localDoc) as never);
  } catch (e) {
    console.warn("[collab] fusion locale au reconnect impossible :", e);
  }
  localStorage.setItem(LS_KEY, handle.url);
  wireHandle(handle);
  return true;
}

/**
 * Reprend le partage mémorisé (le serveur en détient l'état le plus à jour).
 * À utiliser quand l'hôte rouvre l'app : le lien partagé reste valide.
 */
export async function resumeShare(): Promise<string> {
  const saved = getSavedShareUrl();
  if (!saved) return createShare();
  const handle = await findShared(saved);
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
  const handle = await findShared(url as AutomergeUrl);
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
  // Empêche la reconnexion auto de re-joindre aussitôt cette chaîne (le
  // collabUrl reste dans le doc, mais on respecte le choix de l'utilisateur
  // pour la session courante).
  const activeUrl = getCollabHandle()?.url;
  if (activeUrl) suppressAutoReconnect(activeUrl);
  if (_changeOff) {
    _changeOff();
    _changeOff = null;
  }
  setCollabHandle(null);
  useGlucoseStore.setState({ _undoStack: [], _redoStack: [] });
  // Même raison qu'à l'entrée : on quitte vers un `_doc` dont la lignée diffère du
  // fichier baseline → prochain enregistrement = full propre (jamais d'append gappy).
  resetSaveState();
}
