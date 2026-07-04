// ────────────────────────────────────────────────────────────────────────────
// Collab — CANAL D'ASSETS (transfert des octets d'image entre pairs)
// ────────────────────────────────────────────────────────────────────────────
//
// PROBLÈME. Les images sont stockées « du dur » : le fichier vit dans le magasin
// content-addressed LOCAL (`assets/<hash16>.<ext>`) et le doc principal ne porte
// qu'une référence `asset:<nom>`. En collab, seul le doc principal transite sur
// le réseau → le pair reçoit la référence mais PAS les octets → image blanche.
//
// POURQUOI PAS « remettre les octets dans le doc principal ». Déjà tenté (juin) :
// ça regonfle le doc, `A.save` (déclenché à chaque navigation/sauvegarde) refreeze
// 5 s, et un doc géant sature la synchro. Rejeté.
//
// SOLUTION. Un DEUXIÈME document Automerge, dédié aux octets, synchronisé par le
// même `Repo` (donc le même serveur, la même reconnexion). Le doc principal reste
// minuscule (aucun octet dedans → aucune régression de freeze). Le canal, lui, ne
// se synchronise QUE quand une image est ajoutée, pas à chaque nav.
//
//   Forme du canal :  { blobs: { [nom: string]: { mime, bytes } } }
//   `nom` = le nom de fichier `asset:<nom>` (ex. `a1b2c3d4e5f60718.png`).
//
// Le nom de fichier est DÉTERMINISTE (`sha256(bytes)[..16].ext`, cf. `save_asset`
// côté Rust) : les MÊMES octets produisent le MÊME nom sur toutes les machines.
// Donc le pair qui reçoit les octets les réécrit via `save_asset` → le fichier
// réapparaît sous le même `asset:<nom>` → la résolution locale marche → l'image
// s'affiche. Aucune coordination de noms nécessaire.
//
// Symétrie : les DEUX pairs écoutent le doc principal et publient dans le canal
// les assets qu'ils ont sur leur disque et que le canal ne connaît pas encore.
// Celui qui a les octets pousse ; l'autre matérialise. Fonctionne peu importe qui
// a importé l'image.
//
// ⚠️ Ce module réutilise `load_asset`/`save_asset` (Rust) EXISTANTS — zéro
// nouvelle commande Rust. Le cœur (collect/publish/materialize) est GÉNÉRIQUE
// (I/O injectée) → testable à deux `Repo` sans Tauri (cf. assetChannel.*.test.ts).

import type { Repo, DocHandle, AutomergeUrl } from "@automerge/automerge-repo";
import { isValidAutomergeUrl } from "@automerge/automerge-repo";
import { invoke } from "@tauri-apps/api/core";
import { dataUrlToBytes, extFromMime } from "../utils/assetRef";
import { saveAssetFromBytes } from "../utils/assets";
import { useGlucoseStore } from "../store";
import type { Project } from "../types";

/** Un octet-paquet d'asset : les bytes bruts + leur type MIME. */
export interface AssetBlob {
  mime: string;
  bytes: Uint8Array;
}

/** Forme du document « canal d'assets ». */
export interface AssetChannelDoc {
  blobs: Record<string, AssetBlob>;
}

// ── Cœur GÉNÉRIQUE (I/O injectée → testable sans Tauri) ─────────────────────

/**
 * PUR — collecte les noms de fichiers `asset:<nom>` référencés par un projet.
 * Regarde `img.asset` (mode link) ET `img.src` (legacy), sur tous les boards.
 */
export function collectAssetNames(project: {
  boards: { images: { asset?: unknown; src?: string }[] }[];
}): string[] {
  const out = new Set<string>();
  const add = (v: string | undefined) => {
    if (v && v.startsWith("asset:")) out.add(v.slice("asset:".length));
  };
  for (const b of project.boards ?? []) {
    for (const img of b.images ?? []) {
      const asset = img.asset as { mode?: string; href?: string } | undefined;
      if (asset?.mode === "link") add(asset.href);
      add(img.src);
    }
  }
  return [...out];
}

/**
 * Matérialise les octets présents dans le canal mais pas encore vus localement.
 * `already` = ensemble des noms déjà écrits (mémoïsation, évite de réécrire).
 * `writeBytes` = adaptateur d'écriture disque (injecté). Renvoie le nombre écrit.
 */
export async function materializeFromChannel(
  channel: DocHandle<AssetChannelDoc>,
  already: Set<string>,
  writeBytes: (name: string, blob: AssetBlob) => Promise<void>,
): Promise<number> {
  const blobs = channel.doc()?.blobs ?? {};
  let wrote = 0;
  for (const [name, blob] of Object.entries(blobs)) {
    if (already.has(name)) continue;
    try {
      await writeBytes(name, blob);
      already.add(name);
      wrote++;
    } catch (e) {
      console.warn("[assetChannel] matérialisation échouée :", name, e);
    }
  }
  return wrote;
}

/**
 * Publie dans le canal les assets `names` que le canal ne connaît pas encore et
 * dont on possède les octets localement. `loadBytes` renvoie `null` si l'asset
 * n'est pas sur NOTRE disque (dans ce cas c'est à l'autre pair de le publier).
 * `inflight` évite les chargements concurrents en double. Renvoie le nb publié.
 */
export async function publishToChannel(
  channel: DocHandle<AssetChannelDoc>,
  names: string[],
  loadBytes: (name: string) => Promise<AssetBlob | null>,
  inflight: Set<string> = new Set(),
): Promise<number> {
  let published = 0;
  for (const name of names) {
    if (channel.doc()?.blobs?.[name] || inflight.has(name)) continue;
    inflight.add(name);
    try {
      const blob = await loadBytes(name);
      if (!blob) continue; // pas sur mon disque → l'autre s'en chargera
      // Re-vérifie DANS le change : un pair a pu publier entre-temps.
      channel.change((d) => {
        if (!d.blobs) d.blobs = {};
        if (!d.blobs[name]) d.blobs[name] = { mime: blob.mime, bytes: blob.bytes };
      });
      published++;
    } catch (e) {
      console.warn("[assetChannel] publication échouée :", name, e);
    } finally {
      inflight.delete(name);
    }
  }
  return published;
}

// ── Adaptateurs Tauri (I/O réelle) ──────────────────────────────────────────

/** Normalise des octets lus depuis un proxy Automerge en vrai Uint8Array. */
function toU8(raw: unknown): Uint8Array {
  if (raw instanceof Uint8Array) return raw;
  if (ArrayBuffer.isView(raw)) {
    const v = raw as ArrayBufferView;
    return new Uint8Array(v.buffer, v.byteOffset, v.byteLength);
  }
  const obj = raw as Record<number, number> & { length?: number };
  const len = obj?.length ?? Object.keys(obj ?? {}).length;
  const u8 = new Uint8Array(len);
  for (let i = 0; i < len; i++) u8[i] = obj[i] ?? 0;
  return u8;
}

/** Extension (sans point) d'un nom `<hash>.<ext>`. */
function extFromName(name: string): string {
  const dot = name.lastIndexOf(".");
  return dot >= 0 ? name.slice(dot + 1) : "png";
}

/** Lit les octets d'un asset LOCAL. `null` si absent (pas sur ce disque). */
async function loadLocalAsset(name: string): Promise<AssetBlob | null> {
  try {
    const dataUrl = await invoke<string>("load_asset", { filename: name });
    const { bytes, mime } = dataUrlToBytes(dataUrl);
    return { mime, bytes };
  } catch {
    return null;
  }
}

/** Écrit des octets reçus dans le magasin LOCAL (dédup par hash côté Rust). */
async function writeLocalAsset(name: string, blob: AssetBlob): Promise<void> {
  const bytes = toU8(blob.bytes);
  const ext = extFromName(name) || extFromMime(blob.mime);
  const assetId = await saveAssetFromBytes(bytes, ext);
  // Garde-fou : le magasin content-addressed doit reproduire le même nom. Si non,
  // la référence `asset:<name>` du doc ne pointera pas sur le fichier écrit.
  if (assetId !== `asset:${name}`) {
    console.warn(`[assetChannel] nom matérialisé inattendu : ${assetId} ≠ asset:${name}`);
  }
}

// ── Cycle de vie (branché par collabBridge) ─────────────────────────────────

let _channel: DocHandle<AssetChannelDoc> | null = null;
let _channelOff: (() => void) | null = null;
let _mainOff: (() => void) | null = null;
let _publishTimer: ReturnType<typeof setTimeout> | null = null;
let _role: "hôte" | "pair" | null = null;
const _materialized = new Set<string>();
const _inflight = new Set<string>();

/** Le handle du canal actif (ou null). Exposé pour debug/tests. */
export function getAssetChannel(): DocHandle<AssetChannelDoc> | null {
  return _channel;
}

/** Instantané pour le diagnostic à l'écran (débogage collab). */
export function getAssetChannelStats(): {
  active: boolean;
  role: string | null;
  channelBlobs: number;
  materialized: number;
  inflight: number;
} {
  return {
    active: _channel !== null,
    role: _role,
    channelBlobs: _channel ? Object.keys(_channel.doc()?.blobs ?? {}).length : 0,
    materialized: _materialized.size,
    inflight: _inflight.size,
  };
}

async function resolveChannel(
  repo: Repo,
  mainHandle: DocHandle<Project>,
  canCreate: boolean,
): Promise<DocHandle<AssetChannelDoc> | null> {
  const url = mainHandle.doc()?.assetChannelUrl;
  if (url && isValidAutomergeUrl(url)) {
    return await repo.find<AssetChannelDoc>(url as AutomergeUrl);
  }
  if (canCreate) {
    const ch = repo.create<AssetChannelDoc>({ blobs: {} });
    mainHandle.change((d) => {
      (d as Project).assetChannelUrl = ch.url;
    });
    return ch;
  }
  return null;
}

/** Matérialise le contenu courant du canal et prévient le canvas si du neuf. */
async function pullFromChannel(): Promise<void> {
  if (!_channel) return;
  const wrote = await materializeFromChannel(_channel, _materialized, writeLocalAsset);
  if (wrote > 0) useGlucoseStore.getState().bumpAssetEpoch();
}

/** Publie dans le canal les assets référencés que J'AI localement. */
async function pushLocalAssets(): Promise<void> {
  if (!_channel) return;
  const project = useGlucoseStore.getState().project as Project;
  const names = collectAssetNames(project);
  await publishToChannel(_channel, names, loadLocalAsset, _inflight);
}

/** Débounce la publication (le doc principal peut changer en rafale). */
function schedulePublish(): void {
  if (_publishTimer) return;
  _publishTimer = setTimeout(() => {
    _publishTimer = null;
    void pushLocalAssets();
  }, 150);
}

/**
 * Démarre le canal d'assets pour la session collab courante.
 * - `canCreate` : true côté hôte (createShare/resumeShare) → crée le canal s'il
 *   n'existe pas encore et inscrit son URL dans le doc principal.
 * - côté pair (join) : si l'URL n'est pas encore là (l'hôte n'a pas fini de la
 *   publier), on attend le prochain changement du doc principal puis on retente.
 */
export async function startAssetChannel(
  repo: Repo,
  mainHandle: DocHandle<Project>,
  opts: { canCreate: boolean },
): Promise<void> {
  stopAssetChannel();
  _role = opts.canCreate ? "hôte" : "pair";

  const channel = await resolveChannel(repo, mainHandle, opts.canCreate);
  if (!channel) {
    // Pas encore d'URL de canal et on ne peut pas le créer → on attend que
    // l'hôte l'inscrive dans le doc principal, puis on rebranche.
    const onMainChange = () => {
      const url = mainHandle.doc()?.assetChannelUrl;
      if (url && isValidAutomergeUrl(url)) {
        _mainOff?.();
        _mainOff = null;
        void startAssetChannel(repo, mainHandle, { canCreate: false });
      }
    };
    mainHandle.on("change", onMainChange);
    _mainOff = () => mainHandle.off("change", onMainChange);
    return;
  }

  _channel = channel;
  await channel.whenReady();

  // Canal → disque local : matérialise ce qui arrive.
  const onChannelChange = () => {
    void pullFromChannel();
  };
  channel.on("change", onChannelChange);
  _channelOff = () => channel.off("change", onChannelChange);

  // Doc principal → canal : publie les octets que je possède quand de nouvelles
  // références d'images apparaissent (import local OU reçu d'un pair).
  const onMainChange = () => {
    schedulePublish();
  };
  mainHandle.on("change", onMainChange);
  _mainOff = () => mainHandle.off("change", onMainChange);

  // Amorçage : tire ce qui est déjà dans le canal, pousse ce que j'ai déjà.
  await pullFromChannel();
  await pushLocalAssets();
}

/** Coupe le canal (quitter la collab). Idempotent. */
export function stopAssetChannel(): void {
  _channelOff?.();
  _channelOff = null;
  _mainOff?.();
  _mainOff = null;
  if (_publishTimer) {
    clearTimeout(_publishTimer);
    _publishTimer = null;
  }
  _channel = null;
  _role = null;
  _materialized.clear();
  _inflight.clear();
}
