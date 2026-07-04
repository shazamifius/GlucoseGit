// ────────────────────────────────────────────────────────────────────────────
// Diagnostic collab — canal d'assets (TEMPORAIRE, débogage du partage d'images)
// ────────────────────────────────────────────────────────────────────────────
//
// Affiché UNIQUEMENT quand la collab est active. Donne la vérité terrain sur le
// transfert d'images entre pairs, pour distinguer d'un coup d'œil :
//   • un problème de TRANSPORT (les octets n'arrivent pas : « Blobs dans canal »
//     reste < « Images→asset ») ;
//   • un problème d'AFFICHAGE (les octets sont là et matérialisés, mais les
//     images ne s'affichent pas → c'est le canvas, pas le réseau).
//
// À retirer une fois le partage d'images validé.

import { useEffect, useState } from "react";
import { getCollabHandle } from "../multiplayer/collabHandle";
import { getAssetChannelStats, collectAssetNames } from "../multiplayer/assetChannel";
import { useGlucoseStore } from "../store";
import type { Project } from "../types";

function Row({ k, v, bad }: { k: string; v: string; bad?: boolean }) {
  return (
    <div style={{ display: "flex", justifyContent: "space-between", gap: 14 }}>
      <span>{k}</span>
      <span style={{ color: bad ? "#f87171" : "#d4d4dd" }}>{v}</span>
    </div>
  );
}

export default function CollabDebug() {
  // Re-render 1×/s pour rafraîchir les compteurs (lecture directe, pas d'abo store).
  const [, setTick] = useState(0);
  useEffect(() => {
    const id = setInterval(() => setTick((t) => t + 1), 1000);
    return () => clearInterval(id);
  }, []);

  if (!getCollabHandle()) return null; // pas en collab → rien

  const stats = getAssetChannelStats();
  const project = useGlucoseStore.getState().project as Project;
  const refs = collectAssetNames(project).length;
  const urlInDoc = !!project.assetChannelUrl;
  const isPeer = stats.role === "pair";
  // Vert = tout est cohérent (le canal a tous les octets ; côté pair, tous matérialisés).
  const ok =
    stats.active &&
    urlInDoc &&
    stats.channelBlobs >= refs &&
    (!isPeer || stats.materialized >= refs);

  return (
    <div
      style={{
        position: "absolute", top: 64, left: 12, zIndex: 2000,
        background: "#0d0d0dee", border: "1px solid #26262e", borderRadius: 6,
        padding: "8px 10px", minWidth: 200,
        font: "11px/1.55 ui-monospace, SFMono-Regular, Menlo, monospace",
        color: "#9a9aa0", pointerEvents: "none", userSelect: "none",
      }}
    >
      <div style={{ color: "#d4d4dd", marginBottom: 5, letterSpacing: "0.05em" }}>
        <span style={{
          display: "inline-block", width: 8, height: 8, borderRadius: 8,
          background: ok ? "#4ade80" : "#f87171", marginRight: 6, verticalAlign: "middle",
        }} />
        CANAL D&apos;ASSETS · {stats.role ?? "—"}
      </div>
      <Row k="Canal actif" v={stats.active ? "oui" : "NON"} bad={!stats.active} />
      <Row k="URL dans le doc" v={urlInDoc ? "oui" : "NON"} bad={!urlInDoc} />
      <Row k="Images→asset" v={String(refs)} />
      <Row k="Blobs dans canal" v={String(stats.channelBlobs)} bad={stats.channelBlobs < refs} />
      <Row k="Matérialisés" v={String(stats.materialized)} bad={isPeer && stats.materialized < refs} />
      {stats.inflight > 0 && <Row k="En cours…" v={String(stats.inflight)} />}
    </div>
  );
}
