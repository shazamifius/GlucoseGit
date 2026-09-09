// ────────────────────────────────────────────────────────────────────────────
// MEMB-4 — « Ça n'a pas pu grandir, et voilà pourquoi. »
//
// Une membrane étirée qui bute est un NON-ÉVÉNEMENT visuel : elle reste
// simplement à sa taille, exactement comme si tout allait bien. C'est le pire
// cas d'une règle silencieuse — l'utilisateur conclut que la fonctionnalité est
// cassée. Cette couche est là pour que le refus se voie et se comprenne.
//
// ELLE MONTRE L'OBSTACLE, PAS LA MEMBRANE. Le problème n'est pas « la membrane
// n'a pas grandi », c'est « CE bloc-là est sur son chemin ». On entoure donc
// les bloqueurs, et on offre d'aller les voir : la seule action utile est de
// pousser l'intrus, et pour ça il faut d'abord le trouver — il est souvent hors
// écran, puisque la membrane a poussé vers un endroit qu'on ne regardait pas.
//
// ELLE NE SE FERME PAS TOUTE SEULE. Une disparition au bout de N secondes ferait
// rater l'information à qui regardait ailleurs, et couperait le geste de celui
// qui allait cliquer « y aller ». Elle part au clic, ou quand un geste suivant
// réussit — c'est-à-dire quand le problème est résolu.
//
// GÉOMÉTRIE EFFECTIVE, pas naturelle : on entoure ce que l'utilisateur VOIT.
// Un bloqueur posé dans une membrane minimisée est à l'écran là où elle le rend.
//
// Chrome monochrome (style.md) : contour blanc et pointillés, aucune couleur
// d'alerte — la couleur appartient au contenu.
// ────────────────────────────────────────────────────────────────────────────

import { useEffect, useMemo, useRef } from "react";
import { useGlucoseStore } from "../store";
import { itemsOfBoard, resolveItems } from "./membraneSpace";

export interface StretchAlertData {
  /** Membranes qui ont buté. */
  membraneIds: string[];
  /** Éléments à entourer, sans doublon. */
  blockerIds: string[];
}

interface Props {
  alert: StretchAlertData | null;
  boardId: string;
  vpRef: React.MutableRefObject<{ x: number; y: number; scale: number }>;
  onDismiss: () => void;
}

/** Marge du contour autour du bloqueur, en unités MONDE (elle suit le zoom). */
const OUTLINE_PAD = 6;

export default function MembraneStretchAlert({ alert, boardId, vpRef, onDismiss }: Props) {
  const project = useGlucoseStore((s) => s.project);
  const groupRef = useRef<SVGGElement>(null);

  // Boîtes EFFECTIVES des bloqueurs encore présents. Un bloqueur supprimé ou
  // déplacé entre-temps sort de la liste tout seul — on ne garde jamais une
  // géométrie figée au moment de l'avertissement, elle mentirait dès le premier
  // déplacement.
  const boxes = useMemo(() => {
    if (!alert || alert.blockerIds.length === 0) return [];
    const board = project.boards.find((b) => b.id === boardId);
    if (!board) return [];
    const resolved = resolveItems(itemsOfBoard(board));
    const wanted = new Set(alert.blockerIds);
    const out: { id: string; x: number; y: number; width: number; height: number }[] = [];
    for (const id of wanted) {
      const r = resolved.get(id);
      if (r) out.push({ id, x: r.x, y: r.y, width: r.width, height: r.height });
    }
    return out;
  }, [alert, project, boardId]);

  // Même couture que les autres couches SVG : le groupe suit le viewport Pixi
  // sans repasser par React (sinon chaque frame de zoom re-rendrait la couche).
  useEffect(() => {
    const apply = (x: number, y: number, scale: number) => {
      groupRef.current?.setAttribute("transform", `translate(${x},${y}) scale(${scale})`);
    };
    const onVp = (e: Event) => {
      const { x, y, scale } = (e as CustomEvent<{ x: number; y: number; scale: number }>).detail;
      apply(x, y, scale);
    };
    window.addEventListener("glucose:viewport-changed", onVp);
    const { x, y, scale } = vpRef.current;
    apply(x, y, scale);
    return () => window.removeEventListener("glucose:viewport-changed", onVp);
  }, [vpRef, boxes.length]);

  if (!alert || boxes.length === 0) return null;

  /** Amène le premier bloqueur au centre — il est souvent hors écran. */
  function goToBlocker() {
    const b = boxes[0];
    if (!b) return;
    window.dispatchEvent(new CustomEvent("glucose:jump-viewport", {
      detail: { wx: b.x + b.width / 2, wy: b.y + b.height / 2 },
    }));
  }

  const n = boxes.length;

  return (
    <>
      <svg
        style={{
          position: "absolute", inset: 0, width: "100%", height: "100%",
          pointerEvents: "none", zIndex: 260,
        }}
      >
        <g ref={groupRef}>
          {boxes.map((b) => (
            <rect
              key={b.id}
              x={b.x - OUTLINE_PAD}
              y={b.y - OUTLINE_PAD}
              width={b.width + OUTLINE_PAD * 2}
              height={b.height + OUTLINE_PAD * 2}
              fill="none"
              stroke="#e8e8e8"
              strokeWidth={2}
              strokeDasharray="8 6"
              vectorEffect="non-scaling-stroke"
              rx={3}
            />
          ))}
        </g>
      </svg>

      <div
        role="status"
        style={{
          position: "absolute", bottom: 96, left: "50%", transform: "translateX(-50%)",
          background: "#111", border: "1px solid #2a2a2a", borderRadius: 6,
          padding: "8px 12px", display: "flex", alignItems: "center", gap: 10,
          zIndex: 301, boxShadow: "0 4px 20px rgba(0,0,0,0.7)",
          pointerEvents: "all", fontFamily: "system-ui, sans-serif",
        }}
      >
        <span style={{ fontSize: 10, color: "#444", textTransform: "uppercase", letterSpacing: 1 }}>
          Étirement bloqué
        </span>
        <span style={{ fontSize: 11, color: "#888" }}>
          {n === 1
            ? "Un élément qui n'appartient pas à la membrane est sur son chemin."
            : `${n} éléments qui n'appartiennent pas à la membrane sont sur son chemin.`}
        </span>
        <button
          type="button"
          onClick={goToBlocker}
          style={{
            padding: "3px 8px", fontSize: 11, borderRadius: 3,
            border: "1px solid #555", cursor: "pointer", background: "#2d2d2d", color: "#ccc",
          }}
        >
          {n === 1 ? "Y aller" : "Voir le premier"}
        </button>
        <button
          type="button"
          aria-label="Fermer l'avertissement"
          onClick={onDismiss}
          style={{
            padding: "3px 8px", fontSize: 11, borderRadius: 3,
            border: "1px solid #333", cursor: "pointer", background: "#1a1a1a", color: "#666",
          }}
        >
          ✕
        </button>
      </div>
    </>
  );
}
