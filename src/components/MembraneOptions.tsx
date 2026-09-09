// ────────────────────────────────────────────────────────────────────────────
// MEMB-3 — Barre de modes d'une membrane sélectionnée.
//
// Trois modes, et un aller SANS RETOUR vers les deux modes spéciaux :
//
//   classique  — la membrane laisse déborder son contenu (comportement d'origine)
//   minimisée  — elle garde sa taille et réduit son contenu pour le faire tenir
//   étirée     — elle grandit pour contenir son contenu à sa taille réelle
//
// Une membrane spéciale ne redevient JAMAIS classique (cf. `canSwitchMode`) : son
// contenu a été dimensionné sous une règle que « classique » ne sait pas rendre,
// et le retour donnerait aux éléments une taille que personne n'a choisie. Le
// bouton reste donc visible mais désactivé — le cacher laisserait croire à un
// oubli, alors que c'est une décision.
//
// LA CONVERSION PREND UN INSTANTANÉ. Au moment précis où l'on quitte le mode
// classique, l'échelle vaut encore 1 : naturel et effectif coïncident, donc
// l'inclusion géométrique est sans ambiguïté. C'est le seul moment où l'on peut
// déduire l'appartenance de la géométrie sans se tromper — après, la membrane
// est plus petite que son contenu et le test s'inverserait.
//
// LE RIDEAU SE CRÉE ICI, et c'est délibéré. Il vivait sur la languette, au bord
// droit — donc atteignable uniquement en mode focus, alors que créer un rideau
// n'a rien à voir avec le fait d'en regarder un. Les trois choses qu'on décide
// d'une membrane (minimisée, étirée, un rideau) sont désormais au même endroit,
// et cette barre s'affiche aussi bien en focus qu'en dehors : elle ne dépend que
// de la sélection.
//
// Un seul rideau par personne et par membrane : le bouton se désactive une fois
// le sien créé plutôt que de disparaître — comme les modes, une impossibilité se
// montre, elle ne s'escamote pas.
//
// Chrome monochrome (style.md) : la couleur affichée est celle de la membrane,
// qui appartient à l'utilisateur.
// ────────────────────────────────────────────────────────────────────────────

import { useEffect, useState } from "react";
import { getActiveBoard, useGlucoseStore } from "../store";
import type { Annotation, MembraneAnnotation, MembraneMode } from "../types";
import { canSwitchMode, containedIn, itemsOfBoard } from "../canvas/membraneSpace";
import { applyBoardStretch } from "../canvas/membraneStretchRuntime";
import { createCurtain, detachCurtains } from "../canvas/curtainModel";
import { USER_CHANGED_EVENT, getLocalUser } from "../multiplayer/localUser";

interface Props {
  membrane: MembraneAnnotation;
}

const MODES: { value: MembraneMode; label: string; hint: string }[] = [
  { value: "classic", label: "Classique", hint: "Le contenu déborde librement" },
  { value: "minimized", label: "Minimisée", hint: "Taille fixe : le contenu se réduit pour tenir" },
  { value: "stretched", label: "Étirée", hint: "La membrane grandit pour contenir son contenu" },
];

export default function MembraneOptions({ membrane }: Props) {
  const project = useGlucoseStore((s) => s.project);
  const updateAnnotation = useGlucoseStore((s) => s.updateAnnotation);
  const beginLiveEdit = useGlucoseStore((s) => s.beginLiveEdit);
  const endLiveEdit = useGlucoseStore((s) => s.endLiveEdit);

  const ensureCurtainBoard = useGlucoseStore((s) => s.ensureCurtainBoard);

  // L'identité peut changer pendant la session (panneau Collaboration) : on se
  // réabonne plutôt que de figer le nom au montage.
  const [me, setMe] = useState(() => getLocalUser());
  useEffect(() => {
    const onChange = () => setMe(getLocalUser());
    window.addEventListener(USER_CHANGED_EVENT, onChange);
    return () => window.removeEventListener(USER_CHANGED_EVENT, onChange);
  }, []);

  const board = getActiveBoard(project);
  const jenAiUn = (membrane.curtains ?? []).some((c) => c.ownerId === me.id);
  const current: MembraneMode = membrane.mode ?? "classic";
  const members = (board.annotations.filter(
    (a) => a.type !== "arrow" && a.membraneId === membrane.id,
  ).length)
    + board.images.filter((i) => i.membraneId === membrane.id).length;

  function switchTo(next: MembraneMode) {
    if (next === current || !canSwitchMode(current, next)) return;

    // Une seule entrée d'annulation pour la conversion ET l'instantané.
    beginLiveEdit();
    try {
      if (current === "classic") {
        // On quitte le mode classique : l'échelle vaut encore 1, donc ce qui est
        // géométriquement dedans EST dedans. C'est la seule occasion de le
        // déduire sans ambiguïté — après, la membrane sera plus petite que son
        // contenu et l'inclusion s'inverserait.
        const items = itemsOfBoard(board);
        const self = items.find((i) => i.id === membrane.id);
        if (self) {
          const imgIds = new Set(board.images.map((i) => i.id));
          for (const it of containedIn(items, self)) {
            if (it.membraneId) continue; // déjà rattaché ailleurs : on ne vole rien
            if (imgIds.has(it.id)) {
              useGlucoseStore.getState().updateImage(board.id, it.id, { membraneId: membrane.id });
            } else {
              updateAnnotation(board.id, it.id, { membraneId: membrane.id } as Partial<Annotation>);
            }
          }
        }
      }
      updateAnnotation(board.id, membrane.id, { mode: next } as Partial<Annotation>);

      // MEMB-4 — La conversion est le second instant où l'étirement se joue
      // (l'autre est la fin d'un glisser). Il est DANS la transaction : passer
      // en « étirée » et grandir sont un seul geste, donc une seule annulation.
      // On appelle aussi en QUITTANT le mode étiré — pour effacer un
      // avertissement qui n'a plus d'objet, la membrane ne poussant plus.
      const blocked = applyBoardStretch(board.id).filter((o) => o.blocked);
      window.dispatchEvent(new CustomEvent("glucose:stretch-blocked", {
        detail: blocked.length === 0 ? null : {
          membraneIds: blocked.map((o) => o.membraneId),
          blockerIds: [...new Set(blocked.flatMap((o) => o.blockerIds))],
        },
      }));
    } finally {
      endLiveEdit();
    }
  }

  /** Crée mon rideau sur cette membrane, avec son board. */
  function addCurtain() {
    if (jenAiUn) return;
    const fresh = createCurtain(me);
    // Créer le rideau et lui donner son board sont UN seul geste : séparés, un
    // Ctrl+Z laisserait une languette dont le contenu n'existe pas.
    beginLiveEdit();
    try {
      // Toute écriture DÉTACHE : réinsérer dans le document un objet qui en
      // vient déjà fait lever Automerge (cf. `detachCurtains`).
      updateAnnotation(board.id, membrane.id, {
        curtains: detachCurtains([...(membrane.curtains ?? []), fresh]),
      } as Partial<Annotation>);
      ensureCurtainBoard(board.id, membrane.id, fresh.id);
    } finally {
      endLiveEdit();
    }
  }

  const btn: React.CSSProperties = {
    padding: "3px 8px", fontSize: 11, borderRadius: 3,
    border: "1px solid #333", cursor: "pointer", background: "#1a1a1a", color: "#888",
  };
  const btnActive: React.CSSProperties = { ...btn, background: "#2d2d2d", color: "#ccc", borderColor: "#555" };
  const btnOff: React.CSSProperties = { ...btn, color: "#3a3a3a", cursor: "not-allowed", borderColor: "#242424" };

  return (
    <div style={{
      position: "absolute", bottom: 48, left: "50%", transform: "translateX(-50%)",
      background: "#111", border: "1px solid #2a2a2a", borderRadius: 6,
      padding: "8px 12px", display: "flex", alignItems: "center", gap: 10,
      zIndex: 300, boxShadow: "0 4px 20px rgba(0,0,0,0.7)",
      pointerEvents: "all", flexWrap: "wrap", maxWidth: 700,
      fontFamily: "system-ui, sans-serif",
    }}>
      <span style={{ fontSize: 10, color: "#444", textTransform: "uppercase", letterSpacing: 1 }}>
        Membrane
      </span>
      <span style={{
        width: 8, height: 8, borderRadius: "50%",
        background: membrane.color ?? "#60a5fa", flex: "0 0 8px",
      }} />
      <div style={{ width: 1, height: 16, background: "#2a2a2a" }} />

      {MODES.map((m) => {
        const active = m.value === current;
        const allowed = active || canSwitchMode(current, m.value);
        return (
          <button
            key={m.value}
            type="button"
            aria-pressed={active}
            disabled={!allowed}
            title={allowed ? m.hint : "Une membrane spéciale ne redevient jamais classique"}
            onClick={() => switchTo(m.value)}
            style={active ? btnActive : allowed ? btn : btnOff}
          >
            {m.label}
          </button>
        );
      })}

      <div style={{ width: 1, height: 16, background: "#2a2a2a" }} />

      <button
        type="button"
        disabled={jenAiUn}
        title={jenAiUn
          ? "Vous avez déjà un rideau sur cette membrane"
          : "Créer mon rideau personnel — il ne se voit qu'en mode focus"}
        onClick={addCurtain}
        style={jenAiUn ? btnOff : btn}
      >
        Rideau
      </button>

      <div style={{ width: 1, height: 16, background: "#2a2a2a" }} />
      <span style={{ fontSize: 10, color: "#444" }}>
        {members === 0 ? "vide" : members === 1 ? "1 élément" : `${members} éléments`}
      </span>
    </div>
  );
}
