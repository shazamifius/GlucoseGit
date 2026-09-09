// ────────────────────────────────────────────────────────────────────────────
// MEMB-3 — Le rideau à l'écran.
//
// Panneau collé au bord droit, qui RECOUVRE le canvas de la membrane focalisée
// (celui-ci ne bouge jamais). Il n'existe qu'en mode Focus, et nulle part
// ailleurs. Toute la mécanique de déploiement vit dans `curtainPanel` — ce
// fichier ne fait que l'appliquer et dessiner.
//
// Disposition retenue (variante A de la maquette) : une colonne de languettes
// collée au bord droit, une par rideau, à la couleur de son propriétaire. Elle
// reste visible déployée ou non — on voit donc toujours qui travaille, sans
// avoir à ouvrir quoi que ce soit. C'est ce que les deux autres variantes
// perdaient.
//
// Chrome MONOCHROME (cf. style.md) : la seule couleur ici appartient aux
// personnes, jamais à l'interface.
// ────────────────────────────────────────────────────────────────────────────

import { useEffect, useMemo, useRef, useState } from "react";
import type { MembraneAnnotation, MembraneCurtain } from "../types";
import { useGlucoseStore } from "../store";
import { getLocalUser, USER_CHANGED_EVENT } from "../multiplayer/localUser";
import {
  COLLAPSE_STEP, EXPAND_STEP, canEdit, configOf, createCurtain, curtainKind,
  detachCurtains, stepCollapsed, stepExpanded, visibleCurtains,
} from "./curtainModel";
import {
  initialState, panelRect, step, type CurtainState,
} from "./curtainPanel";
// MEMB-7 — le rideau n'affiche plus des notes : il affiche SON BOARD.
import CurtainCanvas from "./CurtainCanvas";

interface Props {
  /** Membrane focalisée, ou `null` — hors focus, la couche ne rend rien. */
  membrane: MembraneAnnotation | null;
  boardId: string;
}

/** Largeur de la colonne de languettes, en pixels écran (constante au zoom). */
const TAB_COL = 30;

export default function MembraneCurtainLayer({ membrane, boardId }: Props) {
  const updateAnnotation = useGlucoseStore((s) => s.updateAnnotation);
  // MEMB-7 — le contenu d'un rideau est un board : sa creation vit dans le store.
  const ensureCurtainBoard = useGlucoseStore((s) => s.ensureCurtainBoard);
  const removeCurtain = useGlucoseStore((s) => s.removeCurtain);
  const beginLiveEdit = useGlucoseStore((s) => s.beginLiveEdit);
  const endLiveEdit = useGlucoseStore((s) => s.endLiveEdit);

  // L'identité peut changer pendant la session (panneau Collaboration) : on se
  // réabonne plutôt que de figer le nom au montage.
  const [me, setMe] = useState(() => getLocalUser());
  useEffect(() => {
    const onChange = () => setMe(getLocalUser());
    window.addEventListener(USER_CHANGED_EVENT, onChange);
    return () => window.removeEventListener(USER_CHANGED_EVENT, onChange);
  }, []);

  const curtains = useMemo(
    () => visibleCurtains(membrane?.curtains ?? [], me.id),
    [membrane?.curtains, me.id],
  );

  const [activeId, setActiveId] = useState<string | null>(null);
  const active = curtains.find((c) => c.id === activeId) ?? curtains[0] ?? null;
  const cfg = useMemo(() => configOf(active), [active]);

  // ── Déploiement au survol ────────────────────────────────────────────────
  const rootRef = useRef<HTMLDivElement>(null);
  const stateRef = useRef<CurtainState>(initialState(cfg));
  const cursorRef = useRef<number | null>(null);
  const [ratio, setRatio] = useState(cfg.collapsed);

  useEffect(() => {
    if (!membrane || curtains.length === 0) return;
    let raf = 0;
    let last = performance.now();
    const tick = (now: number) => {
      const dt = now - last;
      last = now;
      const el = rootRef.current;
      if (el) {
        const w = el.clientWidth;
        const next = step(stateRef.current, cursorRef.current, { width: w }, cfg, now, dt);
        if (next !== stateRef.current || next.ratio !== stateRef.current.ratio) {
          stateRef.current = next;
          setRatio(next.ratio);
        }
      }
      raf = requestAnimationFrame(tick);
    };
    raf = requestAnimationFrame(tick);
    return () => cancelAnimationFrame(raf);
  }, [membrane, curtains.length, cfg]);

  if (!membrane) return null;

  // ── Écritures ────────────────────────────────────────────────────────────
  /** Toute écriture passe par ici — et DÉTACHE. Réinsérer dans le document un
   *  objet qui en vient déjà fait lever Automerge (cf. `detachCurtains`). */
  function writeCurtains(next: MembraneCurtain[]) {
    updateAnnotation(
      boardId, membrane!.id,
      { curtains: detachCurtains(next) } as Partial<MembraneAnnotation>,
    );
  }

  function patchCurtain(id: string, patch: Partial<MembraneCurtain>) {
    writeCurtains((membrane!.curtains ?? []).map((c) => (c.id === id ? { ...c, ...patch } : c)));
  }

  function addCurtain() {
    const fresh = createCurtain(me);
    // Créer le rideau et lui donner son board sont UN seul geste : séparés, un
    // Ctrl+Z laisserait une languette dont le contenu n'existe pas.
    beginLiveEdit();
    try {
      writeCurtains([...(membrane!.curtains ?? []), fresh]);
      ensureCurtainBoard(boardId, membrane!.id, fresh.id);
    } finally {
      endLiveEdit();
    }
    setActiveId(fresh.id);
    stateRef.current = { ...stateRef.current, phase: "expanded" };
  }

  const mine = active?.ownerId === me.id;
  const writable = active ? canEdit(active, me.id) : false;

  // MEMB-7 — Le board du rideau est fabriqué À L'OUVERTURE, pas au chargement
  // du projet : un rideau qu'on ne regarde jamais ne coûte rien. C'est aussi
  // ce qui donne son board à un rideau d'AVANT cette version, par le même
  // chemin que pour un rideau neuf — il n'y a pas de code de migration à part.
  const activeBoardId = active?.boardId ?? null;
  useEffect(() => {
    if (!membrane || !active || active.boardId) return;
    ensureCurtainBoard(boardId, membrane.id, active.id);
  }, [membrane, active, boardId, ensureCurtainBoard]);

  // Le canvas garde toujours une bande à gauche : c'est l'invariant du module.
  const panel = panelRect(ratio, { width: rootRef.current?.clientWidth ?? 1000, height: 0 });

  return (
    <div
      ref={rootRef}
      // L'arbitre de priorité au clic doit laisser ce panneau tranquille : ce
      // n'est pas du contenu de canvas (cf. hitPriority).
      data-arbiter-skip=""
      onPointerMove={(e) => {
        const r = e.currentTarget.getBoundingClientRect();
        cursorRef.current = e.clientX - r.left;
      }}
      onPointerLeave={() => { cursorRef.current = null; }}
      style={{
        position: "absolute", inset: 0, zIndex: 45,
        pointerEvents: "none",
        fontFamily: "system-ui, -apple-system, sans-serif",
      }}
    >
      {curtains.length === 0 ? (
        // Aucun rideau : une seule languette, pour en créer un. Il n'en existe
        // jamais par défaut.
        <button
          type="button"
          onClick={addCurtain}
          title="Créer mon rideau personnel"
          aria-label="Créer mon rideau personnel"
          style={{
            position: "absolute", top: 0, right: 0, bottom: 0, width: TAB_COL,
            pointerEvents: "auto", cursor: "pointer",
            background: "rgba(10,10,10,0.72)",
            backdropFilter: "blur(9px)", WebkitBackdropFilter: "blur(9px)",
            border: 0, borderLeft: "1px solid #262626",
            color: "#7c7c7c", fontSize: 15, padding: 0,
          }}
        >
          +
        </button>
      ) : (
        <div
          style={{
            position: "absolute", top: 0, right: 0, bottom: 0,
            width: Math.max(TAB_COL, panel.width),
            pointerEvents: "auto",
            display: "flex",
            background: "rgba(12,12,12,0.86)",
            // Le flou demandé : la languette laisse deviner sans donner à lire.
            backdropFilter: "blur(9px) saturate(0.7)",
            WebkitBackdropFilter: "blur(9px) saturate(0.7)",
            borderLeft: "1px solid #333",
            overflow: "hidden",
          }}
        >
          {/* ── Corps du rideau ── */}
          <div style={{ flex: 1, minWidth: 0, position: "relative", overflow: "hidden" }}>
            {active && (
              <div style={{
                position: "absolute", inset: 0, padding: "14px 16px",
                display: "flex", flexDirection: "column", gap: 8, overflow: "auto",
              }}>
                <div style={{
                  display: "flex", alignItems: "center", gap: 8,
                  fontSize: 10, letterSpacing: "0.14em", textTransform: "uppercase",
                  color: "#6f6f6f", whiteSpace: "nowrap",
                }}>
                  <span style={{ width: 7, height: 7, flex: "0 0 7px", background: active.ownerColor }} />
                  {active.ownerName}
                  <span style={{ color: "#4a4a4a" }}>· {curtainKind(active)}</span>
                </div>

                {mine && (
                  <div style={{ display: "flex", gap: 6, flexWrap: "wrap" }}>
                    <button type="button" style={chip()} onClick={() => patchCurtain(active.id, {
                      visibility: active.visibility === "private" ? "shared" : "private",
                    })}>
                      {active.visibility === "private" ? "Privé" : "Partagé"}
                    </button>
                    <button
                      type="button"
                      style={chip(active.visibility === "private")}
                      disabled={active.visibility === "private"}
                      title={active.visibility === "private"
                        ? "Un rideau privé est réservé : personne d'autre n'y accède"
                        : "Qui peut y déposer"}
                      onClick={() => patchCurtain(active.id, {
                        editable: active.editable === "owner" ? "everyone" : "owner",
                      })}
                    >
                      {active.editable === "owner" ? "Moi seul" : "Tout le monde"}
                    </button>
                    <button type="button" style={chip()} title="Rétrécir le rideau déployé"
                      onClick={() => patchCurtain(active.id, { expandedRatio: stepExpanded(active.expandedRatio, -EXPAND_STEP) })}>⇥</button>
                    <button type="button" style={chip()} title="Élargir le rideau déployé"
                      onClick={() => patchCurtain(active.id, { expandedRatio: stepExpanded(active.expandedRatio, EXPAND_STEP) })}>⇤</button>
                    {/* La LANGUETTE : ce qui reste visible une fois replié. Elle
                        se règle séparément du déployé — l'une est une cible de
                        survol, l'autre une surface de travail. */}
                    <button type="button" style={chip()} title="Languette plus fine"
                      onClick={() => patchCurtain(active.id, { collapsedRatio: stepCollapsed(active.collapsedRatio, -COLLAPSE_STEP) })}>▖</button>
                    <button type="button" style={chip()} title="Languette plus large"
                      onClick={() => patchCurtain(active.id, { collapsedRatio: stepCollapsed(active.collapsedRatio, COLLAPSE_STEP) })}>▄</button>
                    <button type="button" style={chip()} title="Supprimer ce rideau"
                      onClick={() => {
                        // Le board du rideau part avec lui — sinon il resterait
                        // dans le projet, invisible et inatteignable.
                        removeCurtain(boardId, membrane.id, active.id);
                        setActiveId(null);
                      }}>✕</button>
                  </div>
                )}

                {/* Le contenu du rideau EST un board : on monte le canvas de
                    Glucose dessus, pas une liste de notes. */}
                <div style={{ flex: 1, minHeight: 0, position: "relative" }}>
                  {activeBoardId
                    ? <CurtainCanvas boardId={activeBoardId} editable={writable} />
                    : (
                      <div style={{ fontSize: 11, color: "#4a4a4a", padding: "8px 0" }}>
                        Préparation du canvas…
                      </div>
                    )}
                </div>
              </div>
            )}
          </div>

          {/* ── Colonne de languettes (variante A) ── */}
          <div style={{
            width: TAB_COL, flex: `0 0 ${TAB_COL}px`,
            borderLeft: "1px solid #262626",
            background: "rgba(10,10,10,0.75)",
            display: "flex", flexDirection: "column",
          }}>
            {curtains.map((c) => (
              <button
                key={c.id}
                type="button"
                title={`${c.ownerName} · ${curtainKind(c)}`}
                aria-label={`Rideau de ${c.ownerName}`}
                aria-selected={c.id === active?.id}
                onPointerEnter={() => setActiveId(c.id)}
                onClick={() => setActiveId(c.id)}
                style={{
                  flex: "0 0 auto", height: 62, border: 0,
                  borderBottom: "1px solid #262626",
                  background: c.id === active?.id ? "rgba(255,255,255,0.05)" : "transparent",
                  cursor: "pointer", padding: 0,
                  display: "flex", alignItems: "center", justifyContent: "center",
                }}
              >
                <span style={{
                  width: 3, height: c.id === active?.id ? 52 : 34,
                  background: c.ownerColor,
                }} />
              </button>
            ))}
            {!curtains.some((c) => c.ownerId === me.id) && (
              <button
                type="button"
                onClick={addCurtain}
                title="Créer mon rideau personnel"
                aria-label="Créer mon rideau personnel"
                style={{
                  flex: "0 0 auto", height: 40, border: 0,
                  borderBottom: "1px solid #262626", background: "transparent",
                  color: "#6f6f6f", cursor: "pointer", fontSize: 14, padding: 0,
                }}
              >
                +
              </button>
            )}
          </div>
        </div>
      )}
    </div>
  );
}

function chip(disabled = false): React.CSSProperties {
  return {
    background: "transparent", border: "1px solid #333",
    color: disabled ? "#3a3a3a" : "#9a9a9a",
    fontSize: 10, letterSpacing: "0.06em",
    padding: "3px 7px", cursor: disabled ? "default" : "pointer",
    fontFamily: "inherit",
  };
}
