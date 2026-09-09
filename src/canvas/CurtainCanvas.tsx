// ────────────────────────────────────────────────────────────────────────────
// MEMB-7 — Le canvas d'un rideau.
//
// Le contenu d'un rideau est un BOARD (cf. `ensureCurtainBoard`). Ce composant
// le rend dans le panneau — et son travail consiste surtout à NE RIEN
// RÉÉCRIRE : il monte les couches existantes de Glucose, celles-là mêmes qui
// dessinent la scène principale. Un texte, une note, une membrane, une flèche
// se comportent donc ici exactement comme dehors, y compris les membranes
// minimisées, parce que c'est le même code qui les dessine.
//
// DEUX CHOSES SEULEMENT ONT DÛ ÊTRE OUVERTES DANS CES COUCHES, et chacune pour
// une raison précise :
//
// ① UN AUTRE VIEWPORT. Les couches se synchronisent sur `glucose:viewport-
//    changed`, l'évènement de la caméra principale. Ici la caméra est celle du
//    rideau : elles écoutent donc `glucose:curtain-viewport`. Sans ça, faire
//    glisser la scène derrière ferait glisser le contenu du rideau avec.
//
// ② PAS D'INSCRIPTION À L'ARBITRE DE CLIC. Le registre de `pickArbiter` est un
//    singleton par TYPE de couche : une seconde instance inscrite volerait le
//    routage à la scène principale, et cliquer une image dehors piloterait le
//    rideau. On passe donc `registerPick={false}`. Le panneau porte déjà
//    `data-arbiter-skip`, donc l'arbitre principal ne s'en mêle pas non plus :
//    à l'intérieur du rideau, les couches répondent à leurs propres évènements.
//    CONSÉQUENCE ASSUMÉE : le cycle « re-clic = cible suivante » ne fonctionne
//    pas encore dans le rideau. Il faudra un arbitre par portée pour ça.
//
// LES IMAGES SONT EN DOM, pas en PixiJS. Une seconde instance WebGL pour une
// surface qui fait un dixième d'écran la plupart du temps coûterait un second
// contexte, un second cycle de vie et un second jeu de textures en VRAM. Le
// reste du canvas est identique ; c'est le seul endroit où le rideau diffère
// de la scène.
// ────────────────────────────────────────────────────────────────────────────

import { useEffect, useMemo, useRef, useState } from "react";
import { useGlucoseStore } from "../store";
import type { Annotation, Board, BoardImage } from "../types";
import { resolveImageSrc } from "../utils/assets";
import { hasScaling, itemsOfBoard, projectBoard, resolveItems } from "./membraneSpace";
import HtmlAnnotationLayer from "./HtmlAnnotationLayer";
import SvgAnnotationLayer from "./SvgAnnotationLayer";
import ArrowSvgLayer from "./ArrowSvgLayer";

/** Caméra propre au rideau — jamais celle de la scène. */
export const CURTAIN_VIEWPORT_EVENT = "glucose:curtain-viewport";

const MIN_SCALE = 0.1;
const MAX_SCALE = 4;

interface Props {
  /** Board du rideau. */
  boardId: string;
  /** Faux pour un rideau qu'on ne peut que regarder. */
  editable: boolean;
}

export default function CurtainCanvas({ boardId, editable }: Props) {
  const board = useGlucoseStore((s) => s.project.boards.find((b) => b.id === boardId));
  const updateAnnotation = useGlucoseStore((s) => s.updateAnnotation);

  const rootRef = useRef<HTMLDivElement>(null);
  const vpRef = useRef({ x: 40, y: 40, scale: 1 });
  const [selectedIds, setSelectedIds] = useState<string[]>([]);
  const [editingId, setEditingId] = useState<string | null>(null);

  /** Pousse la caméra du rideau vers les couches — même canal qu'à la scène. */
  function emit() {
    const { x, y, scale } = vpRef.current;
    window.dispatchEvent(new CustomEvent(CURTAIN_VIEWPORT_EVENT, { detail: { x, y, scale } }));
  }

  // Le montage arrive après le premier abonnement des couches : on émet une
  // fois pour qu'elles partent de la bonne position plutôt que de (0, 0).
  useEffect(() => { emit(); }, [boardId]);

  // ── Caméra : molette pour zoomer, glisser sur le vide pour déplacer ───────
  useEffect(() => {
    const el = rootRef.current;
    if (!el) return;

    const onWheel = (e: WheelEvent) => {
      e.preventDefault();
      const r = el.getBoundingClientRect();
      const cx = e.clientX - r.left;
      const cy = e.clientY - r.top;
      const vp = vpRef.current;
      const next = Math.min(MAX_SCALE, Math.max(MIN_SCALE, vp.scale * (e.deltaY < 0 ? 1.1 : 0.9)));
      // Zoom ANCRÉ AU CURSEUR : le point sous la souris ne bouge pas.
      vp.x = cx - (cx - vp.x) * (next / vp.scale);
      vp.y = cy - (cy - vp.y) * (next / vp.scale);
      vp.scale = next;
      emit();
    };

    let panning: { sx: number; sy: number; ox: number; oy: number } | null = null;
    const onDown = (e: PointerEvent) => {
      // On ne déplace la vue que depuis le VIDE : un appui sur un élément
      // appartient à sa couche, qui va le saisir.
      const t = e.target as Element | null;
      if (t && t.closest("[data-pick-owner], input, textarea, button, [contenteditable]")) return;
      if (e.button !== 0 && e.button !== 1) return;
      panning = { sx: e.clientX, sy: e.clientY, ox: vpRef.current.x, oy: vpRef.current.y };
      setSelectedIds([]);
      setEditingId(null);
    };
    const onMove = (e: PointerEvent) => {
      if (!panning) return;
      vpRef.current.x = panning.ox + (e.clientX - panning.sx);
      vpRef.current.y = panning.oy + (e.clientY - panning.sy);
      emit();
    };
    const onUp = () => { panning = null; };

    el.addEventListener("wheel", onWheel, { passive: false });
    el.addEventListener("pointerdown", onDown);
    window.addEventListener("pointermove", onMove);
    window.addEventListener("pointerup", onUp);
    return () => {
      el.removeEventListener("wheel", onWheel);
      el.removeEventListener("pointerdown", onDown);
      window.removeEventListener("pointermove", onMove);
      window.removeEventListener("pointerup", onUp);
    };
  }, []);

  // ── Géométrie effective — les membranes minimisées marchent ici aussi ─────
  const items = useMemo(() => (board ? itemsOfBoard(board) : []), [board]);
  const geom = useMemo(
    () => (hasScaling(items) ? resolveItems(items) : null),
    [items],
  );
  const shown = useMemo(
    () => (board ? projectBoard(board, geom) : { images: [], annotations: [] }),
    [board, geom],
  );

  // ── Images : résolues en URL, rendues en DOM ──────────────────────────────
  const [urls, setUrls] = useState<Record<string, string>>({});
  useEffect(() => {
    if (!board) return;
    let vivant = true;
    (async () => {
      const blobs = useGlucoseStore.getState().project.blobs;
      const out: Record<string, string> = {};
      for (const img of board.images) {
        out[img.id] = await resolveImageSrc(img.asset, img.src, blobs);
      }
      if (vivant) setUrls(out);
    })();
    return () => { vivant = false; };
  }, [board?.images]);

  if (!board) {
    // Le board a disparu (undo, suppression par un pair) : on ne prétend pas
    // le contraire.
    return (
      <div style={{ padding: 16, fontSize: 11, color: "#4a4a4a" }}>
        Le contenu de ce rideau est introuvable.
      </div>
    );
  }

  const boardForArrows = { ...board, annotations: shown.annotations } as Board;

  function select(id: string, multi: boolean) {
    setSelectedIds((prev) =>
      multi
        ? (prev.includes(id) ? prev.filter((x) => x !== id) : [...prev, id])
        : [id]);
  }

  return (
    <div
      ref={rootRef}
      // L'arbitre de la scène ignore déjà le panneau ; on le redit ici pour que
      // ce soit vrai même si le canvas est monté ailleurs un jour.
      data-arbiter-skip=""
      style={{
        position: "absolute", inset: 0, overflow: "hidden",
        cursor: "grab", touchAction: "none",
      }}
    >
      {/* Images — la seule chose que le rideau dessine lui-même. */}
      {shown.images.map((img: BoardImage) => (
        <CurtainImage key={img.id} img={img} url={urls[img.id]} vpRef={vpRef} />
      ))}

      <ArrowSvgLayer
        board={boardForArrows}
        vpRef={vpRef}
        editingId={editingId}
        selectedIds={selectedIds}
        onSelect={select}
        viewportEvent={CURTAIN_VIEWPORT_EVENT}
      />

      <SvgAnnotationLayer
        annotations={board.annotations.filter((a) => a.type === "membrane")}
        selectedIds={selectedIds}
        editingId={editingId}
        vpRef={vpRef}
        geom={geom}
        viewportEvent={CURTAIN_VIEWPORT_EVENT}
        registerPick={false}
        onSelect={select}
        onEdit={(id) => editable && setEditingId(id)}
        onResize={(id, x, y, w, h) => {
          if (!editable) return;
          updateAnnotation(boardId, id, { x, y, width: w, height: h } as Partial<Annotation>);
        }}
      />

      <HtmlAnnotationLayer
        annotations={board.annotations.filter((a) => a.type === "text" || a.type === "sticky")}
        selectedIds={selectedIds}
        editingId={editingId}
        vpRef={vpRef}
        geom={geom}
        viewportEvent={CURTAIN_VIEWPORT_EVENT}
        registerPick={false}
        onSelect={select}
        onEdit={(id) => editable && setEditingId(id)}
        onResize={(id, x, y, w, h) => {
          if (!editable) return;
          updateAnnotation(boardId, id, { x, y, width: w, height: h } as Partial<Annotation>);
        }}
      />

      {!editable && (
        <div style={{
          position: "absolute", bottom: 8, left: 10, zIndex: 20,
          fontSize: 10, letterSpacing: "0.1em", textTransform: "uppercase",
          color: "#4a4a4a", pointerEvents: "none",
        }}>
          Lecture seule
        </div>
      )}
    </div>
  );
}

/**
 * Une image du rideau, en DOM. Elle suit la caméra du rideau par le même canal
 * que les couches, sans repasser par React à chaque frame.
 */
function CurtainImage({ img, url, vpRef }: {
  img: BoardImage;
  url: string | undefined;
  vpRef: React.MutableRefObject<{ x: number; y: number; scale: number }>;
}) {
  const ref = useRef<HTMLImageElement>(null);

  useEffect(() => {
    const apply = (x: number, y: number, s: number) => {
      const el = ref.current;
      if (!el) return;
      // Les sprites sont ancrés au CENTRE côté Pixi ; en DOM on pose le coin.
      el.style.transform =
        `translate(${x + (img.x - img.width / 2) * s}px, ${y + (img.y - img.height / 2) * s}px)`
        + ` scale(${s}) rotate(${img.rotation || 0}rad)`;
    };
    const onVp = (e: Event) => {
      const { x, y, scale } = (e as CustomEvent<{ x: number; y: number; scale: number }>).detail;
      apply(x, y, scale);
    };
    window.addEventListener(CURTAIN_VIEWPORT_EVENT, onVp);
    const { x, y, scale } = vpRef.current;
    apply(x, y, scale);
    return () => window.removeEventListener(CURTAIN_VIEWPORT_EVENT, onVp);
  }, [img.x, img.y, img.width, img.height, img.rotation, vpRef]);

  if (!url) return null;
  return (
    <img
      ref={ref}
      src={url}
      alt=""
      draggable={false}
      style={{
        position: "absolute", top: 0, left: 0,
        width: img.width, height: img.height,
        transformOrigin: "top left",
        pointerEvents: "none",
        userSelect: "none",
      }}
    />
  );
}
