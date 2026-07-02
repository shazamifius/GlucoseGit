import { save as saveDialog, open as openDialog } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";
// SAVE-C — écriture/lecture en octets BRUTS via plugin-fs (plus de base64 : fini
// le +33 % de taille et la double passe CPU sur le thread principal). `rename`
// permet l'écriture ATOMIQUE (tmp + rename) du save complet.
import { writeFile, readFile, rename } from "@tauri-apps/plugin-fs";
import { homeDir } from "@tauri-apps/api/path";
import { Project } from "../types";
import { parseProjectFile } from "../store/projectSchema";
// Phase 7.0 — migration des images base64 legacy vers asset:<hash>.<ext>
// B-STORE — externalisation des blobs embarqués vers le store disque
import { migrateLegacyAssets, externalizeEmbeddedBlobs } from "./assets";
// R-EMB-01 (Sprint 2) — migration asset:<file> / data: / http → AssetRef embed/link
import { migrateProjectAssets, type AssetBytesFetcher } from "./projectMigration";
import { dataUrlToBytes } from "./assetRef";
// Phase 7.2 — format binaire `.glucose` v2 via Automerge
import * as A from "../store/automerge";
// SAVE-A — enregistrement incrémental (anti-freeze) : full au 1er save, delta ensuite.
import { planSave, commitSave, markLoaded, resetSaveState, type SavePlan } from "./saveState";
import { toRelative, getParentDir } from "./pathResolver";
import { getCollabHandle } from "../multiplayer/collabHandle";
// Git #1 Phase 3 — jalons AUTO à l'ampleur.
import { noteSavedDelta, maybeCreateAutoVersion, resetAutoVersionAccumulator } from "./autoVersion";

/**
 * Fetcher Tauri pour la migration R-EMB-01 : lit un asset:<filename> du
 * dossier `assets/` géré et renvoie ses bytes + mime. Renvoie null si
 * introuvable / interdit.
 */
const tauriAssetBytesFetcher: AssetBytesFetcher = async (filename) => {
  try {
    // Le backend renvoie une data URL `data:<mime>;base64,<payload>`.
    const dataUrl = await invoke<string>("load_asset", { filename });
    return dataUrlToBytes(dataUrl);
  } catch (e) {
    console.debug(`[asset fetcher] échec ${filename}:`, e);
    return null;
  }
};

// ────────────────────────────────────────────────────────────────────────────
// Save — toujours en v2 binaire (Automerge)
// ────────────────────────────────────────────────────────────────────────────
//
// `saveProject` accepte indifféremment un Project plain (cas legacy / tests)
// OU un Doc<Project> Automerge (cas standard depuis le store CRDT). Si on
// reçoit un Doc on save TEL QUEL — l'historique CRDT est préservé. Si on reçoit
// un Project, on crée un doc neuf (l'historique commence ici).

export async function saveProject(
  projectOrDoc: Project | A.Doc<Project>,
  existingPath?: string,
): Promise<string | null> {
  let path = existingPath;
  // Détecte si c'est déjà un doc Automerge ou un project plain
  const isDoc = (val: unknown): val is A.Doc<Project> => {
    try { A.getHeads(val as A.Doc<Project>); return true; } catch { return false; }
  };

  // Pour le nom par défaut, on lit `name` (fonctionne pour les deux types via Proxy)
  const projectName = (projectOrDoc as Project).name || "projet";

  if (!path) {
    let defaultPath: string;
    try {
      const home = await homeDir();
      defaultPath = `${home}/${projectName}.glucose`;
    } catch (_) {
      defaultPath = `${projectName}.glucose`;
    }
    const result = await saveDialog({
      defaultPath,
      filters: [{ name: "Projet Glucose Git", extensions: ["glucose"] }],
    });
    path = result ?? undefined;
  }
  if (!path) return null;

  if (!path.endsWith(".glucose")) path += ".glucose";

  // ── SAVE-A — full vs incrémental ──────────────────────────────────────────
  let doc: A.Doc<Project>;
  let forceFull = false;
  if (isDoc(projectOrDoc)) {
    // Cas standard : le doc est la source de vérité.
    doc = projectOrDoc;
  } else {
    // Cas legacy : project plain → doc neuf, aucune filiation → toujours full.
    const plain = projectOrDoc as Project;
    doc = A.create<Project>({ ...plain, version: "2.0.0", updatedAt: Date.now() });
    forceFull = true;
  }
  // Pas de path fourni = dialog (« Enregistrer sous » / 1er save) → full (recompacte
  // sur le nouveau fichier ; aucune filiation fiable avec un baseline existant).
  // Normalise les chemins vers des chemins relatifs par rapport au fichier .glucose sauvegardé.
  // ⚠️ SOLO UNIQUEMENT : en collab, `doc` est le doc d'un handle automerge-repo ;
  // un `A.change` brut dessus déclenche le panic WASM « unsafe aliasing » (cf. mémoire
  // collab-automerge-repo). Et sémantiquement, un chemin relatif au .glucose LOCAL n'a
  // aucun sens à synchroniser vers un pair (chemins disque différents). On saute donc
  // la réécriture quand un handle collab est actif.
  const projectDir = getParentDir(path);
  if (projectDir && !getCollabHandle()) {
    const nextDoc = A.change(doc, "Portabilité des chemins", (proj) => {
      for (const board of proj.boards) {
        for (const img of board.images) {
          if (img.sourceUrl) {
            img.sourceUrl = toRelative(img.sourceUrl, path);
          }
        }
        for (const ann of board.annotations) {
          if ((ann as any).sourceFile) {
            (ann as any).sourceFile = toRelative((ann as any).sourceFile, path);
          }
        }
      }
    });
    doc = nextDoc;

    // Met à jour le store Zustand de manière asynchrone pour synchroniser le document en mémoire
    import("../store").then(({ useGlucoseStore }) => {
      useGlucoseStore.setState({
        _doc: nextDoc,
        project: nextDoc as unknown as Project,
      });
    }).catch((e) => console.warn("[saveProject] échec de la mise à jour du store :", e));
  }

  const plan: SavePlan = forceFull ? { mode: "full", bytes: A.save(doc), deltaBytes: 0 } : planSave(doc, path);

  if (plan.mode === "full") {
    // Save COMPLET → écriture ATOMIQUE (octets bruts) : on écrit dans un fichier
    // temporaire puis on le renomme par-dessus la cible. `rename` remplace
    // atomiquement → un crash en pleine écriture ne laisse JAMAIS le .glucose
    // existant à moitié écrasé (ferme le dernier trou de sécurité du save).
    const tmp = `${path}.tmp`;
    await writeFile(tmp, plan.bytes);
    await rename(tmp, path);
    commitSave(path, doc, plan);
    // deltaBytes 0 = nouvelle ligne de base (fichier neuf / « Enregistrer sous »)
    // → le compteur d'ampleur repart de zéro pour ce fichier.
    if (plan.deltaBytes === 0) resetAutoVersionAccumulator();
  } else if (plan.bytes.length > 0) {
    // Save INCRÉMENTAL → ajoute le delta à la fin du fichier (octets bruts, O(édits)).
    // Un crash en plein append est rattrapé par `loadResilient` au chargement.
    await writeFile(path, plan.bytes, { append: true });
    commitSave(path, doc, plan);
  }
  // Git #1 Phase 3 — comptabilise l'ampleur réellement écrite ; au-delà du seuil,
  // un jalon durable AUTO est posé (non bloquant, sûr en collab car A.save est en
  // lecture seule). plan.bytes vide = rien n'a changé → no-op disque ET no-op ici.
  if (plan.deltaBytes > 0) {
    noteSavedDelta(plan.deltaBytes);
    void maybeCreateAutoVersion(path, doc);
  }
  return path;
}

// ────────────────────────────────────────────────────────────────────────────
// Load — détecte automatiquement v1 (JSON) ou v2 (binaire Automerge)
// ────────────────────────────────────────────────────────────────────────────
//
// Renvoie soit un `doc` (v2 → préserve l'historique Automerge complet à l'ouverture),
// soit un `project` plain (v1 → l'historique commencera au prochain change). Le
// store appelle `loadDoc(doc)` ou `loadProject(project)` selon.

export interface LoadProjectResult {
  /** Présent si v2 binaire — la source de vérité Automerge originale. */
  doc?: A.Doc<Project>;
  /** Toujours présent — vue plain pour les checks et migrations legacy. */
  project: Project;
  path: string;
  /** SAVE-A — true si le fichier avait une fin corrompue récupérée (delta tronqué
   *  ignoré) : l'UI doit prévenir l'utilisateur et l'inviter à réenregistrer. */
  recovered?: boolean;
}

export async function loadProject(): Promise<LoadProjectResult | null> {
  let defaultPath: string | undefined;
  try {
    defaultPath = await homeDir();
  } catch (_) {}

  const result = await openDialog({
    defaultPath,
    filters: [{ name: "Projet Glucose Git", extensions: ["glucose", "atelier"] }],
    multiple: false,
  });
  const path = result as string | null;
  if (!path) return null;

  let project: Project | null = null;
  let doc: A.Doc<Project> | undefined;
  let v2ByteLen = 0;       // SAVE-A — taille du fichier v2 chargé (baseline incrémental)
  let recovered = false;   // SAVE-A — true si fin de fichier corrompue récupérée

  // SAVE-C — lecture des octets BRUTS une seule fois (plus de base64).
  let fileBytes: Uint8Array;
  try {
    fileBytes = await readFile(path);
  } catch (readErr) {
    throw new Error(`Impossible de lire le fichier .glucose.\n${(readErr as Error).message}`);
  }

  // 1) Tentative v2 binaire
  try {
    v2ByteLen = fileBytes.length;
    // Chargement TOLÉRANT : une fin tronquée (crash pendant un append incrémental)
    // ne doit pas rendre tout le fichier illisible — on récupère le plus grand
    // préfixe sain.
    const res = A.loadResilient<Project>(fileBytes);
    recovered = res.recovered;
    if (res.recovered) {
      console.warn(
        `[loadProject] fin de fichier corrompue récupérée (${res.droppedBytes} octet(s) ignorés) — ` +
        `le projet sera réécrit proprement au prochain enregistrement`
      );
    }
    const plain = A.asPlain(res.doc);
    if (plain && Array.isArray((plain as Project).boards)) {
      project = plain as Project;
      doc = res.doc;
      console.info("[loadProject] format v2 (binaire Automerge) détecté");
    }
  } catch (binErr) {
    console.debug("[loadProject] v2 binaire KO, tentative v1 JSON :", binErr);
  }

  // 2) Tentative v1 JSON (legacy) — on décode les mêmes octets en texte UTF-8.
  if (!project) {
    try {
      const data = new TextDecoder().decode(fileBytes);
      const raw = JSON.parse(data);
      const parsed = parseProjectFile(raw);
      if (!parsed.ok) {
        throw new Error(`Le fichier .glucose est invalide ou corrompu (${parsed.error}).`);
      }
      project = parsed.project;
      console.info("[loadProject] format v1 (JSON legacy) détecté — sera migré au prochain save");
    } catch (jsonErr) {
      throw new Error(
        `Impossible de lire le fichier .glucose. Ni JSON valide ni binaire Automerge.\n${(jsonErr as Error).message}`
      );
    }
  }

  // Phase 7.0 — Migration des assets base64 legacy vers asset:<hash>.<ext>.
  const migration = await migrateLegacyAssets(project);
  if (migration.migrated > 0) {
    console.info(`[loadProject] ${migration.migrated} image(s) legacy migrée(s) vers asset:`);
    // Si on avait un doc v2 mais qu'on a migré des assets, le doc est désynchronisé
    // → on force la création d'un nouveau doc à partir du project migré
    doc = undefined;
  }
  if (migration.failed > 0) {
    console.warn(`[loadProject] ${migration.failed} image(s) legacy n'ont pas pu être migrées`);
  }

  // R-EMB-01 (Sprint 2) — Migration des images vers le modèle dual AssetRef.
  // Idempotente : un projet déjà migré est inchangé. Cette migration peut
  // embedder de gros volumes (toutes les images du dossier assets/ sont
  // rapatriées dans le .glucose) — c'est précisément l'objectif : un
  // .glucose self-contained.
  const embMigration = await migrateProjectAssets(migration.project, tauriAssetBytesFetcher);
  if (embMigration.migrated > 0) {
    console.info(
      `[loadProject] R-EMB-01 : ${embMigration.migrated} image(s) migrée(s) en AssetRef ` +
      `(${embMigration.blobsAdded} blob(s) ajoutés, ${embMigration.failed} échec(s))`
    );
    // Comme pour la migration précédente : le doc Automerge originel est
    // désynchronisé du project muté → on force la re-création.
    doc = undefined;
  }

  // B-STORE — Externalisation « du dur » : sort toutes les images embarquées
  // (project.blobs) vers le store disque et passe les refs en `link`. C'est ce
  // qui vide le doc de ses ~112 Mo d'images et supprime le freeze de `A.save`.
  // Idempotent (no-op si aucun blob). Force un doc neuf (sans l'historique gonflé).
  const extMigration = await externalizeEmbeddedBlobs(embMigration.project);
  if (extMigration.externalized > 0) {
    console.info(
      `[loadProject] B-STORE : ${extMigration.externalized} image(s) sortie(s) du doc vers le disque ` +
      `(${extMigration.failed} échec(s)) — le projet sera allégé au prochain enregistrement`
    );
    doc = undefined;
  }

  // SAVE-A — baseline pour l'enregistrement incrémental. Si `doc` est encore défini
  // ici, AUCUNE migration n'a touché le projet → le fichier sur disque correspond
  // exactement à `save(doc)`, donc le 1er Ctrl+S pourra être incrémental. Sinon
  // (migration legacy / v1 / blobs externalisés / fin récupérée), le doc en mémoire
  // diffère du fichier → on force un save COMPLET au prochain enregistrement (ce
  // qui réécrit/nettoie le fichier, notamment après une récupération).
  if (doc && !recovered) markLoaded(path, doc, v2ByteLen);
  else resetSaveState();

  return { project: extMigration.project, doc, path, recovered };
}
