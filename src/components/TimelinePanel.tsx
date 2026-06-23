// ────────────────────────────────────────────────────────────────────────────
// Phase 7.4 — Time Machine UI  (tiroir vertical droit, style historique GitHub)
// ────────────────────────────────────────────────────────────────────────────
//
// Tiroir docké à droite du canvas, AU-DESSUS de la minimap. Le user peut :
//   • Glisser la réglette fine (les N « gestes ») → aperçu live d'un état passé
//     (PixiJS redessine car `project` dérive du doc preview)
//   • « Restaurer cet état » → applique l'aperçu comme nouveau commit
//   • « + Marquer un jalon » → écrit une VERSION DURABLE (save complet sur disque)
//     qui apparaît dans la liste, restaurable même si le doc vivant se corrompt
//   • Cliquer « ↩ Restaurer » sur une version → revient à ce point exact
//
// Deux niveaux d'historique, distincts à dessein :
//   - GESTES = chaque change Automerge (fin, borné par UNDO_DEPTH) → la réglette
//   - VERSIONS = jalons durables sur disque (les « commits » que l'user pose) → la liste
//
// Mode interaction : tant que `_previewHeads !== null`, toutes les mutations du
// store sont bloquées (cf. store mutate). Un liseré ambre signale ce mode.

import { useEffect, useMemo, useRef, useState } from "react";
import { useGlucoseStore } from "../store";
import * as A from "../store/automerge";
import type { Project } from "../types";
import { getCurrentPath } from "../utils/currentPath";
import { saveVersion, listVersions, loadVersionDoc, type VersionMeta } from "../utils/versions";
import { showToast } from "./Toast";

interface Props {
  onClose: () => void;
}

interface CommitEntry {
  index: number;
  message: string;
  time: number;            // ms unix
  heads: A.Heads;
}

const DRAWER_WIDTH = 324;

// Messages de mutation « vue/navigation » (cf. store mutateView) : déplacer la
// caméra, changer de board, ouvrir un dossier, re-mesurer une taille. Ce sont
// des gestes DÉRIVÉS du rendu, pas des éditions du contenu → on les EXCLUT de la
// Time Machine (sinon 300+ « gestes » qui ne sont que des pans de caméra).
const NAV_NOISE = new Set([
  "setViewport", "syncAnnotationSize", "setActiveBoardId",
  "expandFolder", "enterFolder", "exitFolder", "exitToRoot",
]);

export default function TimelinePanel({ onClose }: Props) {
  const _doc = useGlucoseStore((s) => s._doc);
  const _previewHeads = useGlucoseStore((s) => s._previewHeads);
  const setPreviewHeads = useGlucoseStore((s) => s.setPreviewHeads);
  const restoreToPreview = useGlucoseStore((s) => s.restoreToPreview);
  const commitNamed = useGlucoseStore((s) => s.commitNamed);
  const restoreFromPlain = useGlucoseStore((s) => s.restoreFromPlain);

  const trackRef = useRef<HTMLDivElement>(null);
  const [draggingIdx, setDraggingIdx] = useState<number | null>(null);
  const [namedDialog, setNamedDialog] = useState(false);
  const [jalonName, setJalonName] = useState("");

  // ── Versions DURABLES (jalons écrits sur disque, incorruptibles) ──────────
  const path = getCurrentPath();
  const [versions, setVersions] = useState<VersionMeta[]>([]);
  const [busy, setBusy] = useState(false);

  // (Re)charge la liste à l'ouverture du panneau. PAS sur chaque changement de
  // `_doc` : en solo, bouger la caméra réécrit le doc à chaque frame → relire le
  // disque (readDir) 60×/s gelait l'app. Les nouveaux jalons rafraîchissent la
  // liste manuellement (cf. doMark).
  useEffect(() => {
    if (!path) { setVersions([]); return; }
    let alive = true;
    listVersions(path).then((v) => { if (alive) setVersions(v); }).catch(() => {});
    return () => { alive = false; };
  }, [path]);

  // Débounce du doc pour la lecture (coûteuse) de l'historique : pendant un
  // pan/zoom, `_doc` change à chaque frame, mais on ne veut recalculer la
  // réglette qu'au repos (~150 ms) — sinon `A.history` tourne 60×/s.
  const [stableDoc, setStableDoc] = useState(_doc);
  useEffect(() => {
    const t = setTimeout(() => setStableDoc(_doc), 150);
    return () => clearTimeout(t);
  }, [_doc]);

  // Marque un jalon : repère in-doc (slider) + version durable sur disque.
  async function doMark() {
    const name = jalonName.trim() || "Jalon";
    commitNamed(name);            // repère dans l'historique Automerge (slider)
    setJalonName("");
    setNamedDialog(false);
    if (!path) {
      showToast("Jalon posé. Enregistre le projet (Ctrl+S) pour des versions durables.", "📌");
      return;
    }
    try {
      setBusy(true);
      await Promise.resolve();    // laisse une éventuelle mutation collab se poser
      await saveVersion(path, useGlucoseStore.getState()._doc, name, "manuel");
      const v = await listVersions(path);
      setVersions(v);
      showToast(`Version durable « ${name} » enregistrée`, "💾");
    } catch (e) {
      console.error("[TimelinePanel] saveVersion échec:", e);
      showToast("Échec de l'écriture de la version durable", "⚠️");
    } finally {
      setBusy(false);
    }
  }

  // Restaure une version durable (remplace le contenu courant, annulable Ctrl+Z).
  async function doRestore(meta: VersionMeta) {
    if (busy) return;
    if (!window.confirm(
      `Restaurer la version « ${meta.label} » ?\nL'état actuel sera remplacé (annulable par Ctrl+Z).`
    )) return;
    try {
      setBusy(true);
      const doc = await loadVersionDoc(meta);
      const plain = A.asPlain<Project>(doc);
      setPreviewHeads(null);      // sort d'un éventuel aperçu
      restoreFromPlain(plain);
      showToast(`Version « ${meta.label} » restaurée`, "⏮");
    } catch (e) {
      console.error("[TimelinePanel] restore version échec:", e);
      showToast("Échec de la restauration de la version", "⚠️");
    } finally {
      setBusy(false);
    }
  }

  // ── Reconstruit la liste des commits depuis l'historique Automerge ────
  const commits = useMemo<CommitEntry[]>(() => {
    try {
      // Phase 7.4 : on prend chaque change Automerge comme un point de timeline.
      // Le `message` passé à `mutate(message, ...)` est conservé ici.
      const history = A.history(stableDoc);
      const out: CommitEntry[] = [];
      // Pour récupérer les heads à un point N de l'historique, on doit refaire
      // un viewAt sur les changes.hash de chaque commit. On saute les gestes de
      // navigation/caméra (NAV_NOISE) : ce ne sont pas de vraies éditions.
      for (let i = 0; i < history.length; i++) {
        const entry = history[i];
        const message = entry.change.message || "(sans message)";
        if (NAV_NOISE.has(message)) continue;
        out.push({
          index: i,
          message,
          time: entry.change.time * 1000,
          heads: [entry.change.hash],
        });
      }
      return out;
    } catch (e) {
      console.error("[TimelinePanel] history error:", e);
      return [];
    }
  }, [stableDoc]);

  // Index du commit actuellement sélectionné (preview ou présent)
  const currentIdx = useMemo(() => {
    if (!_previewHeads) return commits.length - 1;
    // Match par hash : le head preview correspond à un seul commit
    const target = _previewHeads[0];
    const idx = commits.findIndex((c) => c.heads[0] === target);
    return idx === -1 ? commits.length - 1 : idx;
  }, [_previewHeads, commits]);

  // ── Drag de la réglette ───────────────────────────────────────────────
  function pickAtX(clientX: number) {
    if (!trackRef.current || commits.length === 0) return;
    const rect = trackRef.current.getBoundingClientRect();
    const ratio = Math.max(0, Math.min(1, (clientX - rect.left) / rect.width));
    const idx = Math.round(ratio * (commits.length - 1));
    setDraggingIdx(idx);
    if (idx === commits.length - 1) {
      // Position « maintenant » → sortir du preview
      setPreviewHeads(null);
    } else {
      setPreviewHeads(commits[idx].heads);
    }
  }

  function startDrag(e: React.PointerEvent) {
    e.preventDefault();
    pickAtX(e.clientX);
    const onMove = (m: PointerEvent) => pickAtX(m.clientX);
    const onUp = () => {
      window.removeEventListener("pointermove", onMove);
      window.removeEventListener("pointerup", onUp);
      setDraggingIdx(null);
    };
    window.addEventListener("pointermove", onMove);
    window.addEventListener("pointerup", onUp);
  }

  // ── Échap = retour au présent + fermeture ─────────────────────────────
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        if (_previewHeads) setPreviewHeads(null);
        else onClose();
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [_previewHeads, onClose, setPreviewHeads]);

  // Position du curseur (en %) sur la réglette
  const cursorPct = commits.length <= 1
    ? 100
    : (currentIdx / (commits.length - 1)) * 100;

  const inPreview = _previewHeads !== null;
  const currentCommit = commits[currentIdx];

  return (
    <>
      {/* Liseré ambre quand on explore le passé */}
      {inPreview && (
        <div
          style={{
            position: "fixed", inset: 0, pointerEvents: "none",
            border: "3px solid #fbbf24",
            boxShadow: "inset 0 0 60px rgba(251, 191, 36, 0.18)",
            zIndex: 1090,
            transition: "opacity 200ms",
          }}
        />
      )}

      {/* ── Tiroir vertical droit (style historique GitHub) ─────────────── */}
      <div
        style={{
          position: "absolute", top: 12, right: 12, bottom: 12,
          width: DRAWER_WIDTH,
          background: "linear-gradient(180deg, rgba(20,20,24,0.97), rgba(13,13,16,0.97))",
          border: `1px solid ${inPreview ? "#fbbf2466" : "#2a2a2a"}`,
          borderRadius: 10,
          boxShadow: "0 8px 32px rgba(0,0,0,0.55)",
          backdropFilter: "blur(8px)",
          WebkitBackdropFilter: "blur(8px)",
          userSelect: "none",
          zIndex: 1100,                       // au-dessus de la minimap (1000)
          display: "flex", flexDirection: "column",
          fontFamily: "system-ui, sans-serif",
          overflow: "hidden",
        }}
      >
        {/* Header */}
        <div style={{
          display: "flex", alignItems: "center", justifyContent: "space-between",
          padding: "12px 14px 10px", borderBottom: "1px solid #1f1f23",
        }}>
          <span style={{ display: "flex", alignItems: "center", gap: 8 }}>
            <span style={{ fontSize: 13, fontWeight: 600, color: "#e5e7eb", letterSpacing: 0.3 }}>
              ⏳ Time Machine
            </span>
            <span style={{
              fontSize: 9, fontWeight: 700, letterSpacing: 0.5,
              padding: "2px 7px", borderRadius: 999,
              background: inPreview ? "rgba(251,191,36,0.15)" : "rgba(16,185,129,0.15)",
              color: inPreview ? "#fbbf24" : "#34d399",
              border: `1px solid ${inPreview ? "rgba(251,191,36,0.4)" : "rgba(16,185,129,0.4)"}`,
            }}>
              {inPreview ? "APERÇU" : "EN DIRECT"}
            </span>
          </span>
          <button onClick={onClose} title="Fermer (Échap)" style={btnIcon()}>✕</button>
        </div>

        {/* Réglette fine : les N gestes (historique d'undo) */}
        <div style={{ padding: "10px 14px 12px", borderBottom: "1px solid #1f1f23" }}>
          <div style={{
            display: "flex", justifyContent: "space-between", alignItems: "baseline",
            fontSize: 9, color: "#6b7280", letterSpacing: 0.4, marginBottom: 6,
          }}>
            <span>{commits.length} GESTES</span>
            <span style={{
              color: "#9ca3af", fontSize: 10, maxWidth: 170,
              overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap",
            }}>
              {currentCommit
                ? `${currentIdx + 1}/${commits.length} · ${formatTimeAgo(currentCommit.time)}`
                : "—"}
            </span>
          </div>
          <div
            ref={trackRef}
            onPointerDown={startDrag}
            style={{
              position: "relative", height: 22,
              background: "linear-gradient(180deg, rgba(255,255,255,0.03), rgba(255,255,255,0.012))",
              borderRadius: 4, cursor: "pointer",
              border: "1px solid #1f1f23",
            }}
          >
            {commits.map((c, i) => {
              const pct = commits.length <= 1 ? 0 : (i / (commits.length - 1)) * 100;
              const isJalon = c.message.startsWith("📌");
              // Au-delà de 80 gestes, on n'affiche qu'un tick fin sur deux (les
              // jalons restent toujours visibles) → réglette lisible, pas un mur.
              if (!isJalon && commits.length > 80 && i % 2 === 1) return null;
              return (
                <div
                  key={i}
                  style={{
                    position: "absolute",
                    left: `${pct}%`, top: 3, bottom: 3,
                    width: isJalon ? 2 : 1,
                    marginLeft: isJalon ? -1 : 0,
                    background: isJalon ? "#fbbf24" : "#3a3a3a",
                    pointerEvents: "none",
                  }}
                />
              );
            })}
            {/* Curseur courant */}
            <div
              style={{
                position: "absolute",
                left: `${cursorPct}%`, top: -3, bottom: -3,
                width: 3, marginLeft: -1.5, borderRadius: 2,
                background: inPreview ? "#fbbf24" : "#10b981",
                boxShadow: `0 0 8px ${inPreview ? "rgba(251,191,36,0.6)" : "rgba(16,185,129,0.6)"}`,
                pointerEvents: "none",
                transition: draggingIdx === null ? "left 100ms ease-out" : "none",
              }}
            />
          </div>
          {inPreview && (
            <div style={{ display: "flex", gap: 6, marginTop: 8 }}>
              <button onClick={() => setPreviewHeads(null)} style={btnSecondary()}>
                ← Maintenant
              </button>
              <button onClick={restoreToPreview} style={{ ...btnPrimary(), flex: 1 }}>
                ⏪ Restaurer cet état
              </button>
            </div>
          )}
        </div>

        {/* Liste des VERSIONS durables (commits façon GitHub) */}
        <div style={{ flex: 1, overflowY: "auto", padding: "12px 14px" }}>
          <div style={{ fontSize: 9, color: "#6b7280", letterSpacing: 0.5, marginBottom: 10 }}>
            💾 VERSIONS DURABLES
          </div>

          {!path ? (
            <div style={EMPTY_HINT}>
              Enregistre le projet (<kbd style={KBD}>Ctrl</kbd>+<kbd style={KBD}>S</kbd>) pour
              créer des versions durables, restaurables même si le document se corrompt.
            </div>
          ) : versions.length === 0 ? (
            <div style={EMPTY_HINT}>
              Aucune version pour l'instant. Clique{" "}
              <b style={{ color: "#cbd5e1" }}>+ Marquer un jalon</b>{" "}
              en bas pour créer ton premier point de restauration.
            </div>
          ) : (
            <div>
              {versions.map((v, i) => {
                const last = i === versions.length - 1;
                const isAuto = v.kind === "auto";
                const accent = isAuto ? "#60a5fa" : "#2dd4bf";
                const halo = isAuto ? "rgba(96,165,250,0.15)" : "rgba(45,212,191,0.15)";
                return (
                  <div key={v.file} style={{ display: "flex", gap: 10 }}>
                    {/* Rail du graphe : pastille + ligne de connexion */}
                    <div style={{
                      display: "flex", flexDirection: "column", alignItems: "center", width: 12,
                    }}>
                      <div style={{
                        width: 10, height: 10, borderRadius: 999, marginTop: 4,
                        background: accent, boxShadow: `0 0 0 3px ${halo}`,
                      }} />
                      {!last && <div style={{ flex: 1, width: 2, background: "#26262b", marginTop: 2 }} />}
                    </div>
                    {/* Contenu du « commit » */}
                    <div style={{ flex: 1, paddingBottom: 16, minWidth: 0 }}>
                      <div style={{
                        display: "flex", alignItems: "flex-start", justifyContent: "space-between", gap: 6,
                      }}>
                        <div style={{
                          fontSize: 12.5, fontWeight: 600, color: "#f3f4f6",
                          lineHeight: 1.3, wordBreak: "break-word",
                        }}>
                          {v.label}
                        </div>
                        <button
                          onClick={() => doRestore(v)}
                          disabled={busy}
                          title="Restaurer cette version"
                          style={{
                            flexShrink: 0, background: "transparent",
                            border: `1px solid ${accent}55`, color: accent,
                            borderRadius: 6, padding: "3px 9px",
                            fontSize: 10.5, fontWeight: 600,
                            cursor: busy ? "default" : "pointer",
                            opacity: busy ? 0.5 : 1, whiteSpace: "nowrap",
                            fontFamily: "system-ui, sans-serif",
                          }}
                        >
                          ↩ Restaurer
                        </button>
                      </div>
                      <div style={{
                        display: "flex", alignItems: "center", gap: 6,
                        marginTop: 4, fontSize: 10, color: "#6b7280",
                      }}>
                        <span style={{
                          padding: "1px 6px", borderRadius: 999,
                          fontSize: 9, fontWeight: 700, letterSpacing: 0.3,
                          background: isAuto ? "rgba(96,165,250,0.12)" : "rgba(45,212,191,0.12)",
                          color: accent,
                        }}>
                          {isAuto ? "AUTO" : "MANUEL"}
                        </span>
                        <span>{formatTimeAgo(v.time)}</span>
                      </div>
                    </div>
                  </div>
                );
              })}
            </div>
          )}
        </div>

        {/* Footer : créer un jalon */}
        <div style={{ padding: "10px 14px", borderTop: "1px solid #1f1f23" }}>
          <button
            onClick={() => setNamedDialog(true)}
            disabled={inPreview || busy}
            title={inPreview ? "Sors de l'aperçu pour marquer un jalon" : "Créer un point de restauration nommé"}
            style={{
              ...btnPrimary(), width: "100%", padding: "9px 12px", fontSize: 12,
              opacity: (inPreview || busy) ? 0.5 : 1,
              cursor: (inPreview || busy) ? "default" : "pointer",
            }}
          >
            + Marquer un jalon
          </button>
        </div>
      </div>

      {/* Dialog jalon nommé */}
      {namedDialog && (
        <div
          onClick={() => setNamedDialog(false)}
          style={{
            position: "fixed", inset: 0, background: "rgba(0,0,0,0.5)",
            display: "flex", alignItems: "center", justifyContent: "center", zIndex: 1200,
          }}
        >
          <div
            onClick={(e) => e.stopPropagation()}
            style={{
              background: "#1a1a1a", border: "1px solid #333", borderRadius: 8,
              padding: 18, width: 380, fontFamily: "system-ui, sans-serif",
              boxShadow: "0 24px 48px rgba(0,0,0,0.6)",
            }}
          >
            <div style={{ color: "#fde68a", fontSize: 11, letterSpacing: 0.5, marginBottom: 6 }}>
              📌 NOUVEAU JALON
            </div>
            <input
              autoFocus
              value={jalonName}
              onChange={(e) => setJalonName(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === "Enter") doMark();
                if (e.key === "Escape") setNamedDialog(false);
              }}
              placeholder="ex: « Première version du concept », « avant refonte »…"
              style={{
                width: "100%", padding: "10px 12px",
                background: "#0d0d0d", color: "#f3f4f6",
                border: "1px solid #444", borderRadius: 6,
                fontSize: 13, outline: "none", boxSizing: "border-box",
              }}
            />
            <div style={{ display: "flex", justifyContent: "flex-end", gap: 8, marginTop: 12 }}>
              <button onClick={() => setNamedDialog(false)} style={btnSecondary()}>Annuler</button>
              <button onClick={doMark} style={btnPrimary()}>Marquer</button>
            </div>
          </div>
        </div>
      )}
    </>
  );
}

// ── Helpers de style ─────────────────────────────────────────────────────────
const EMPTY_HINT: React.CSSProperties = {
  fontSize: 11, color: "#9ca3af", lineHeight: 1.5,
  background: "rgba(255,255,255,0.02)", border: "1px dashed #2a2a2a",
  borderRadius: 8, padding: "12px 14px",
};
const KBD: React.CSSProperties = {
  background: "#0d0d0d", border: "1px solid #333", borderRadius: 4,
  padding: "0 5px", fontSize: 10, color: "#cbd5e1", fontFamily: "monospace",
};

function btnIcon(): React.CSSProperties {
  return {
    background: "transparent",
    border: "1px solid #444",
    color: "#aaa",
    borderRadius: 4,
    fontSize: 11,
    padding: "2px 8px",
    cursor: "pointer",
    fontFamily: "system-ui, sans-serif",
  };
}
function btnSecondary(): React.CSSProperties {
  return {
    background: "transparent",
    border: "1px solid #444",
    color: "#cbd5e1",
    borderRadius: 4,
    fontSize: 11,
    padding: "5px 12px",
    cursor: "pointer",
    fontFamily: "system-ui, sans-serif",
  };
}
function btnPrimary(): React.CSSProperties {
  return {
    background: "#fbbf24",
    color: "#0d0d0d",
    border: "none",
    borderRadius: 4,
    fontSize: 11,
    padding: "5px 12px",
    cursor: "pointer",
    fontWeight: 600,
    fontFamily: "system-ui, sans-serif",
  };
}

function formatTimeAgo(ts: number): string {
  const sec = Math.max(0, (Date.now() - ts) / 1000);
  if (sec < 5) return "à l'instant";
  if (sec < 60) return `il y a ${Math.round(sec)} s`;
  const min = sec / 60;
  if (min < 60) return `il y a ${Math.round(min)} min`;
  const h = min / 60;
  if (h < 24) return `il y a ${Math.round(h)} h`;
  const d = h / 24;
  return `il y a ${Math.round(d)} j`;
}
