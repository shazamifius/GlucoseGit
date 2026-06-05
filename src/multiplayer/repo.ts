// ────────────────────────────────────────────────────────────────────────────
// Collab — singleton Repo automerge-repo (synchro + persistance)
// ────────────────────────────────────────────────────────────────────────────
//
// Le Repo gère pour nous : le protocole de synchro Automerge (catch-up complet à
// la connexion), la reconnexion automatique, et la persistance.
//
//   • Réseau   : WebSocket vers le serveur public `wss://sync.automerge.org`.
//                Ce serveur est TOUJOURS allumé et stocke le document → c'est lui
//                qui permet « le PC d'un pair est fermé mais l'autre a tout ».
//   • Stockage : IndexedDB (persistance navigateur, filet local côté collab).
//
// Construit PARESSEUSEMENT : le mode solo n'instancie jamais de Repo. Pour passer
// sur un serveur auto-hébergé plus tard, il suffit de changer `SYNC_SERVER_URL`.

import { Repo } from "@automerge/automerge-repo";
import { BrowserWebSocketClientAdapter } from "@automerge/automerge-repo-network-websocket";
import { IndexedDBStorageAdapter } from "@automerge/automerge-repo-storage-indexeddb";

/** Serveur de synchro always-on. Public pour le moment (test immédiat). */
export const SYNC_SERVER_URL = "wss://sync.automerge.org";

let _repo: Repo | null = null;

/** Renvoie le Repo singleton, en le construisant à la première demande. */
export function getRepo(): Repo {
  if (_repo) return _repo;
  _repo = new Repo({
    storage: new IndexedDBStorageAdapter("glucose"),
    network: [new BrowserWebSocketClientAdapter(SYNC_SERVER_URL)],
  });
  // Anti perte de données : si on ferme/recharge vite, on force l'écriture des
  // derniers changements (la sauvegarde locale est sinon débouncée ~100 ms).
  // Les changements sont déjà ENVOYÉS au serveur dès qu'ils sont faits ; ce flush
  // garantit en plus qu'ils sont persistés localement → resync au prochain
  // démarrage même si le réseau était lent.
  if (typeof window !== "undefined") {
    const flush = () => { _repo?.flush().catch(() => { /* best-effort */ }); };
    window.addEventListener("pagehide", flush);
    window.addEventListener("beforeunload", flush);
  }
  return _repo;
}

/** True dès qu'au moins un pair (le serveur de synchro) est connecté. */
export function isServerConnected(): boolean {
  return _repo !== null && _repo.peers.length > 0;
}

/**
 * Attend que le Repo soit CONNECTÉ au serveur de synchro avant toute opération
 * réseau (find/partage). Indispensable : sans ça, un `repo.find()` lancé pendant
 * que le WebSocket s'ouvre encore conclut « Document unavailable » (course).
 */
export async function ensureConnected(timeoutMs = 12000): Promise<void> {
  const repo = getRepo();
  if (repo.peers.length > 0) return;
  // 1) Adaptateur réseau prêt (WebSocket ouvert).
  await Promise.race([
    repo.networkSubsystem.whenReady(),
    new Promise<void>((res) => setTimeout(res, timeoutMs)),
  ]);
  if (repo.peers.length > 0) return;
  // 2) Annonce d'au moins un pair (le serveur).
  await new Promise<void>((resolve, reject) => {
    const timer = setTimeout(() => {
      cleanup();
      reject(new Error("Serveur de synchro injoignable — vérifie ta connexion internet."));
    }, timeoutMs);
    const onPeer = () => { cleanup(); resolve(); };
    const cleanup = () => {
      clearTimeout(timer);
      repo.networkSubsystem.off("peer", onPeer);
    };
    repo.networkSubsystem.on("peer", onPeer);
  });
}

/**
 * S'abonne aux changements de connexion réseau (connexion/déconnexion d'un pair,
 * typiquement le serveur de synchro). Renvoie une fonction de désabonnement.
 */
export function onConnectivityChange(cb: () => void): () => void {
  const repo = getRepo();
  repo.networkSubsystem.on("peer", cb);
  repo.networkSubsystem.on("peer-disconnected", cb);
  return () => {
    repo.networkSubsystem.off("peer", cb);
    repo.networkSubsystem.off("peer-disconnected", cb);
  };
}
