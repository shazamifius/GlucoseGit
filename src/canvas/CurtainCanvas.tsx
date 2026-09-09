// ────────────────────────────────────────────────────────────────────────────
// MEMB-7 / MEMB-8 — Le canvas d'un rideau.
//
// Le contenu d'un rideau est un BOARD (cf. `ensureCurtainBoard`). Ce composant
// le rend dans le panneau — et son travail consiste surtout à NE RIEN
// RÉÉCRIRE : il monte les couches existantes de Glucose, celles-là mêmes qui
// dessinent la scène principale. Un texte, une note, une membrane, une flèche
// se comportent donc ici exactement comme dehors, y compris les membranes
// minimisées, parce que c'est le même code qui les dessine.
//
// UN RIDEAU EST UN MONDE DE CLIC AUTONOME. C'est la clé de tout le reste. Il a
// sa caméra, son board, sa sélection — et par conséquent son propre arbitre de
// priorité. Ce que les couches devaient apprendre pour ça, et rien de plus :
//
// ① UN AUTRE VIEWPORT (`viewportEvent`). Les couches se synchronisent sur
//    `glucose:viewport-changed`, l'évènement de la caméra principale. Ici la
//    caméra est celle du rideau : elles écoutent `glucose:curtain-viewport`.
//    Sans ça, faire glisser la scène derrière ferait glisser le rideau avec.
//
// ② UNE AUTRE PORTÉE D'ARBITRE (`pickScope`). Le registre de `pickArbiter` est
//    indexé par (portée, type de couche) : les couches du rideau s'inscrivent
//    sous `curtainScope(boardId)`, celles de la scène sous `main`. Elles ne se
//    volent donc plus le routage — et le rideau récupère du même coup le CYCLE
//    « re-clic = cible suivante », qui passe par ce registre.
//
// ③ UN AUTRE DÉPLACEUR (`onMove`). Les couches appellent `moveSelected` sur le
//    board ACTIF et la sélection GLOBALE. Dans un rideau, les deux sont faux :
//    tirer un texte y déplaçait ce qui était sélectionné DEHORS. Elles
//    reçoivent ici le déplaceur du rideau.
//
// LES IMAGES SONT EN DOM, pas en PixiJS. Une seconde instance WebGL pour une
// surface qui fait un dixième d'écran la plupart du temps coûterait un second
// contexte, un second cycle de vie et un second jeu de textures en VRAM. Comme
// il n'y a donc pas de sprite à saisir, c'est ce composant qui porte la
// préhension des images — le seul endroit où le rideau écrit du code que la
// scène n'a pas.
// ────────────────────────────────────────────────────────────────────────────

import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { useGlucoseStore } from "../store";
import type { Annotation, Board, BoardImage } from "../types";
import { resolveImageSrc } from "../utils/assets";
import {
  hasScaling, itemsOfBoard, projectBoard, reconcileMembership, resolveItems, scaleOf,
} from "./membraneSpace";
import {
  PICK, advanceOnRelease, collectCandidates, pickAtDown,
  type CycleState, type PickCandidate,
} from "./hitPriority";
import {
  beginPick, curtainScope, markHijack, registerPickHandler, wasHijacked,
} from "./pickArbiter";
import { computeResize } from "./imageResize";
import { applyBoardStretch } from "./membraneStretchRuntime";
import type { StretchAlertData } from "./MembraneStretchAlert";
import MembraneStretchAlert from "./MembraneStretchAlert";
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

/** Ce que le rideau retient d'un geste sur une image. */
interface ImageDrag {
  id: string;
  /** Position NATURELLE au grab — celle du document, pas celle affichée. */
  startX: number;
  startY: number;
  /** Point de départ du curseur, en coordonnées monde du rideau. */
  pStartX: number;
  pStartY: number;
  /** Échelle effective de l'image (membrane minimisée). Le curseur avance de
   *  `d` à l'écran, la coordonnée naturelle doit avancer de `d / k`. */
  k: number;
  didMove: boolean;
}

/** Redimensionnement d'une image : mêmes champs que la scène (cf. imageResize). */
interface ImageResize {
  id: string;
  cx: number;
  cy: number;
  aspect: number;
  ax: number;
  ay: number;
}

export default function CurtainCanvas({ boardId, editable }: Props) {
  const board = useGlucoseStore((s) => s.project.boards.find((b) => b.id === boardId));
  const updateAnnotation = useGlucoseStore((s) => s.updateAnnotation);
  const updateImage = useGlucoseStore((s) => s.updateImage);

  const rootRef = useRef<HTMLDivElement>(null);
  const vpRef = useRef({ x: 40, y: 40, scale: 1 });
  // Sélection LOCALE au rideau : la sélection du store décrit la scène, et les
  // deux ne doivent jamais se confondre (cf. ③ en tête de fichier).
  const [selectedIds, setSelectedIds] = useState<string[]>([]);
  const [selectedImageIds, setSelectedImageIds] = useState<string[]>([]);
  const [editingId, setEditingId] = useState<string | null>(null);
  const [stretchAlert, setStretchAlert] = useState<StretchAlertData | null>(null);

  const scope = useMemo(() => curtainScope(boardId), [boardId]);

  /** Pousse la caméra du rideau vers les couches — même canal qu'à la scène. */
  const emit = useCallback(() => {
    const { x, y, scale } = vpRef.current;
    window.dispatchEvent(new CustomEvent(CURTAIN_VIEWPORT_EVENT, { detail: { x, y, scale } }));
  }, []);

  // Le montage arrive après le premier abonnement des couches : on émet une
  // fois pour qu'elles partent de la bonne position plutôt que de (0, 0).
  useEffect(() => { emit(); }, [boardId, emit]);

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
      setSelectedImageIds([]);
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
  }, [emit]);

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

  // Refs de lecture pour les gestionnaires d'évènements natifs, qui vivent hors
  // du cycle de rendu de React et liraient sinon une fermeture périmée.
  const shownRef = useRef(shown);
  shownRef.current = shown;
  const geomRef = useRef(geom);
  geomRef.current = geom;
  const editableRef = useRef(editable);
  editableRef.current = editable;
  const selAnnRef = useRef(selectedIds);
  selAnnRef.current = selectedIds;
  const selImgRef = useRef(selectedImageIds);
  selImgRef.current = selectedImageIds;

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

  // ── Déplacement de la sélection du rideau ─────────────────────────────────
  //
  // L'équivalent de `moveSelected`, mais sur CE board et CETTE sélection. Comme
  // lui, il divise le déplacement par l'échelle effective de chaque élément :
  // dans une membrane à 0,4 une coordonnée naturelle doit avancer 2,5 fois plus
  // que le curseur pour que l'élément reste sous lui.
  const moveCurtainSelection = useCallback((dx: number, dy: number) => {
    if (!editableRef.current) return;
    const g = geomRef.current;
    const b = useGlucoseStore.getState().project.boards.find((x) => x.id === boardId);
    if (!b) return;
    for (const id of selImgRef.current) {
      const img = b.images.find((i) => i.id === id);
      if (!img || img.locked) continue;
      const k = scaleOf(g, id);
      updateImage(boardId, id, { x: img.x + dx / k, y: img.y + dy / k });
    }
    for (const id of selAnnRef.current) {
      const a = b.annotations.find((x) => x.id === id);
      if (!a) continue;
      const k = scaleOf(g, id);
      updateAnnotation(boardId, id, { x: a.x + dx / k, y: a.y + dy / k } as Partial<Annotation>);
    }
  }, [boardId, updateImage, updateAnnotation]);

  // ── Préhension d'une image (il n'y a pas de sprite ici pour le faire) ─────
  const dragRef = useRef<ImageDrag | null>(null);
  const resizeRef = useRef<ImageResize | null>(null);

  /** Point cliqué, en coordonnées monde du rideau. */
  const toWorld = useCallback((clientX: number, clientY: number) => {
    const r = rootRef.current?.getBoundingClientRect();
    const vp = vpRef.current;
    const left = r?.left ?? 0;
    const top = r?.top ?? 0;
    return { wx: (clientX - left - vp.x) / vp.scale, wy: (clientY - top - vp.y) / vp.scale };
  }, []);

  const beginImagePick = useCallback((id: string, e: PointerEvent, corner?: string) => {
    const st = useGlucoseStore.getState();
    if (st.activeTool !== "select") return;
    const b = st.project.boards.find((x) => x.id === boardId);
    const img = b?.images.find((i) => i.id === id);
    if (!img) return;

    const multi = e.ctrlKey || e.metaKey || e.shiftKey;
    setSelectedImageIds((prev) => {
      if (multi) return prev.includes(id) ? prev.filter((x) => x !== id) : [...prev, id];
      return prev.includes(id) ? prev : [id];
    });
    if (!multi) setSelectedIds([]);
    setEditingId(null);

    if (!editableRef.current || img.locked) return;
    const { wx, wy } = toWorld(e.clientX, e.clientY);

    if (corner) {
      // Même construction qu'à la scène : l'ancre est le coin diagonalement
      // OPPOSÉ à la poignée tirée, et elle se lit sur la géométrie NATURELLE.
      const sgnX = corner === "tl" || corner === "bl" ? -1 : 1;
      const sgnY = corner === "tl" || corner === "tr" ? -1 : 1;
      resizeRef.current = {
        id, cx: img.x, cy: img.y,
        aspect: img.width / Math.max(1, img.height),
        ax: img.x - sgnX * (img.width / 2),
        ay: img.y - sgnY * (img.height / 2),
      };
      return;
    }

    dragRef.current = {
      id, startX: img.x, startY: img.y,
      pStartX: wx, pStartY: wy,
      k: scaleOf(geomRef.current, id),
      didMove: false,
    };
  }, [boardId, toWorld]);

  const beginImagePickRef = useRef(beginImagePick);
  beginImagePickRef.current = beginImagePick;

  useEffect(
    () => registerPickHandler("image", (id, e, corner) => beginImagePickRef.current(id, e, corner), scope),
    [scope],
  );

  // Suite du geste image : déplacement ou redimensionnement, jusqu'au relâcher.
  useEffect(() => {
    function onMove(e: PointerEvent) {
      const rz = resizeRef.current;
      const dg = dragRef.current;
      if (!rz && !dg) return;
      const { wx, wy } = toWorld(e.clientX, e.clientY);
      const st = useGlucoseStore.getState();

      if (rz) {
        st.beginLiveEdit();
        const r = computeResize(rz, wx, wy, e.ctrlKey);
        updateImage(boardId, rz.id, r);
        return;
      }
      if (!dg) return;
      const dx = wx - dg.pStartX;
      const dy = wy - dg.pStartY;
      if (!dg.didMove) {
        if (Math.hypot(dx, dy) * vpRef.current.scale <= PICK.CYCLE_RADIUS_PX) return;
        dg.didMove = true;
        st.beginLiveEdit();
      }
      updateImage(boardId, dg.id, { x: dg.startX + dx / dg.k, y: dg.startY + dy / dg.k });
    }

    function onUp() {
      if (dragRef.current || resizeRef.current) useGlucoseStore.getState().endLiveEdit();
      dragRef.current = null;
      resizeRef.current = null;
    }

    window.addEventListener("pointermove", onMove);
    window.addEventListener("pointerup", onUp);
    window.addEventListener("pointercancel", onUp);
    return () => {
      window.removeEventListener("pointermove", onMove);
      window.removeEventListener("pointerup", onUp);
      window.removeEventListener("pointercancel", onUp);
    };
  }, [boardId, toWorld, updateImage]);

  // ── L'arbitre du rideau ───────────────────────────────────────────────────
  //
  // Copie conforme de celui de la scène, moins ce que le rideau n'a pas
  // (dossiers, mode focus, ghost de layout) : on intercepte en phase CAPTURE,
  // on demande à `hitPriority` qui gagne, on remet le clic à la couche
  // propriétaire de NOTRE portée. Le pas du cycle se joue au relâchement, pour
  // la même raison que dehors — à l'appui, on ne sait pas encore si le geste
  // sera un clic ou un glisser.
  const applyPickSelection = useCallback((picked: PickCandidate) => {
    if (picked.owner === "image") {
      setSelectedImageIds([picked.id]);
      setSelectedIds([]);
    } else {
      setSelectedImageIds([]);
      setSelectedIds([picked.id]);
    }
  }, []);

  useEffect(() => {
    const el = rootRef.current;
    if (!el) return;

    const cycleRef: { current: CycleState | null } = { current: null };
    let pending: { sx: number; sy: number; candidates: PickCandidate[]; multi: boolean } | null = null;

    function disarm() {
      pending = null;
      window.removeEventListener("pointerup", onDrop, true);
      window.removeEventListener("pointerup", onUp);
      window.removeEventListener("pointercancel", onCancel);
    }

    function arm(e: PointerEvent, candidates: PickCandidate[], multi: boolean) {
      disarm();
      pending = { sx: e.clientX, sy: e.clientY, candidates, multi };
      // MEMB-1 — l'appartenance se règle en phase CAPTURE, donc dans la MÊME
      // entrée d'undo que le déplacement (les couches referment leur
      // transaction dans leur propre `pointerup`).
      window.addEventListener("pointerup", onDrop, true);
      window.addEventListener("pointerup", onUp);
      window.addEventListener("pointercancel", onCancel);
    }

    /** Fin de glisser : ce qu'on vient de lâcher change-t-il de membrane ? */
    function onDrop(e: PointerEvent) {
      const ctx = pending;
      if (!ctx || !editableRef.current) return;
      if (Math.hypot(e.clientX - ctx.sx, e.clientY - ctx.sy) <= PICK.CYCLE_RADIUS_PX) return;

      const st = useGlucoseStore.getState();
      const b = st.project.boards.find((x) => x.id === boardId);
      if (!b) return;
      const moved = [...selImgRef.current, ...selAnnRef.current];
      if (moved.length === 0) return;

      const its = itemsOfBoard(b);
      const changes = reconcileMembership(its, resolveItems(its), moved);
      const imgIds = new Set(b.images.map((i) => i.id));
      for (const c of changes) {
        // `undefined` retire la clé du document ; `null` y écrirait une valeur.
        const patch = { membraneId: c.membraneId ?? undefined };
        if (imgIds.has(c.id)) st.updateImage(b.id, c.id, patch);
        else st.updateAnnotation(b.id, c.id, patch as Partial<Annotation>);
      }
      // MEMB-4 — l'étirement vient APRÈS l'appartenance : un élément qu'on vient
      // de déposer doit compter dans l'étendue du contenu.
      const blocked = applyBoardStretch(b.id).filter((o) => o.blocked);
      setStretchAlert(blocked.length === 0 ? null : {
        membraneIds: blocked.map((o) => o.membraneId),
        blockerIds: [...new Set(blocked.flatMap((o) => o.blockerIds))],
      });
    }

    function onUp(e: PointerEvent) {
      const ctx = pending;
      disarm();
      if (!ctx) return;
      // Le pointeur a bougé : c'était un GLISSER, pas un clic. Aucun cran, et le
      // cycle est oublié — la géométrie sous le curseur vient de changer.
      if (ctx.multi || Math.hypot(e.clientX - ctx.sx, e.clientY - ctx.sy) > PICK.CYCLE_RADIUS_PX) {
        cycleRef.current = null;
        return;
      }
      const { picked, cycle } = advanceOnRelease(ctx.candidates, cycleRef.current, Date.now());
      cycleRef.current = cycle;
      if (picked) applyPickSelection(picked);
    }

    function onCancel() {
      disarm();
      cycleRef.current = null;
    }

    function onDown(e: PointerEvent) {
      markHijack(false, scope);
      if (e.button !== 0) return;
      const st = useGlucoseStore.getState();
      if (st.activeTool !== "select") return;

      const target = e.target as Element | null;
      if (!target || typeof target.closest !== "function") return;
      if (target.closest("input, textarea, select, button, a, [contenteditable]")) return;
      const naturalRoot = target.closest("[data-pick-owner]") as HTMLElement | null;
      // Hors du rideau, ou sur un de ses ornements : clic inchangé.
      if (naturalRoot && !el?.contains(naturalRoot)) return;

      const { wx, wy } = toWorld(e.clientX, e.clientY);
      const hintOwner = naturalRoot?.dataset.pickOwner as PickCandidate["owner"] | undefined;
      const hintId = naturalRoot?.dataset.pickId ?? null;
      const arrowId = hintOwner === "arrow" ? hintId : null;
      const multi = e.ctrlKey || e.metaKey || e.shiftKey;

      const candidates = collectCandidates({
        wx, wy, scale: vpRef.current.scale,
        images: shownRef.current.images,
        annotations: shownRef.current.annotations,
        // Un rideau ne porte pas de dossiers : sa couche n'est pas montée.
        folders: [],
        selectedImageIds: selImgRef.current,
        selectedAnnotationIds: selAnnRef.current,
        selectedFolderId: null,
        arrowId,
        domHint: hintOwner && hintOwner !== "arrow" && hintId ? { owner: hintOwner, id: hintId } : null,
      });
      if (candidates.length === 0) {
        disarm();
        cycleRef.current = null;
        return;
      }

      const { picked, cycle } = pickAtDown(
        candidates, cycleRef.current, e.clientX, e.clientY, Date.now(),
        { alt: e.altKey, multi },
      );
      cycleRef.current = cycle;
      if (!picked) return;

      // Une flèche gagnante est déjà la cible naturelle du DOM : on laisse filer
      // l'event, ArrowSvgLayer fait le reste — le cycle continue de compter.
      if (picked.owner === "arrow") {
        arm(e, candidates, multi);
        return;
      }

      e.stopPropagation();
      markHijack(true, scope);
      beginPick(picked.owner, picked.id, e, picked.corner, scope);
      arm(e, candidates, multi);
    }

    // Un pointerdown détourné produit quand même un dblclick natif sur la cible
    // d'ORIGINE : sans ça, double-cliquer une image posée dans une membrane
    // ouvrirait l'éditeur de la MEMBRANE. Le drapeau est propre à la portée.
    function onDblClick(e: MouseEvent) {
      if (wasHijacked(scope)) e.stopPropagation();
    }

    el.addEventListener("pointerdown", onDown, { capture: true });
    el.addEventListener("dblclick", onDblClick, { capture: true });
    return () => {
      disarm();
      el.removeEventListener("pointerdown", onDown, { capture: true });
      el.removeEventListener("dblclick", onDblClick, { capture: true });
    };
  }, [boardId, scope, toWorld, applyPickSelection]);

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
    setSelectedImageIds([]);
    setSelectedIds((prev) =>
      multi
        ? (prev.includes(id) ? prev.filter((x) => x !== id) : [...prev, id])
        : [id]);
  }

  /** Amène un point du rideau au centre de son panneau. */
  function jumpTo(wx: number, wy: number) {
    const r = rootRef.current?.getBoundingClientRect();
    const vp = vpRef.current;
    vp.x = (r?.width ?? 0) / 2 - wx * vp.scale;
    vp.y = (r?.height ?? 0) / 2 - wy * vp.scale;
    emit();
  }

  return (
    <div
      ref={rootRef}
      // L'arbitre de la SCÈNE ignore ce sous-arbre : le rideau a le sien, et
      // deux arbitres sur le même clic se disputeraient la cible.
      data-arbiter-skip=""
      style={{
        position: "absolute", inset: 0, overflow: "hidden",
        cursor: "grab", touchAction: "none",
      }}
    >
      {/* Images — la seule chose que le rideau dessine lui-même. */}
      {shown.images.map((img: BoardImage) => (
        <CurtainImage
          key={img.id}
          img={img}
          url={urls[img.id]}
          vpRef={vpRef}
          selected={selectedImageIds.includes(img.id)}
        />
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
        pickScope={scope}
        onSelect={select}
        onMove={moveCurtainSelection}
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
        pickScope={scope}
        onSelect={select}
        onMove={moveCurtainSelection}
        onEdit={(id) => editable && setEditingId(id)}
        onResize={(id, x, y, w, h) => {
          if (!editable) return;
          updateAnnotation(boardId, id, { x, y, width: w, height: h } as Partial<Annotation>);
        }}
      />

      <MembraneStretchAlert
        alert={stretchAlert}
        boardId={boardId}
        vpRef={vpRef}
        viewportEvent={CURTAIN_VIEWPORT_EVENT}
        onJump={jumpTo}
        onDismiss={() => setStretchAlert(null)}
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
 *
 * Elle porte `data-pick-owner` : c'est ce que lit l'arbitre pour savoir ce que
 * le DOM aurait naturellement touché, et c'est aussi ce qui empêche le panneau
 * de partir en déplacement de caméra quand on appuie dessus.
 *
 * Le cadre de sélection est un CALQUE À PART, en pixels écran. Le mettre dans
 * l'image l'aurait fait grossir avec le zoom : à l'échelle 4 les poignées
 * auraient couvert l'image, à 0,2 elles auraient disparu.
 */
function CurtainImage({ img, url, vpRef, selected }: {
  img: BoardImage;
  url: string | undefined;
  vpRef: React.MutableRefObject<{ x: number; y: number; scale: number }>;
  selected: boolean;
}) {
  const ref = useRef<HTMLImageElement>(null);
  const boxRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const apply = (x: number, y: number, s: number) => {
      // Les sprites sont ancrés au CENTRE côté Pixi ; en DOM on pose le coin.
      const left = x + (img.x - img.width / 2) * s;
      const top = y + (img.y - img.height / 2) * s;
      const el = ref.current;
      if (el) {
        el.style.transform =
          `translate(${left}px, ${top}px) scale(${s}) rotate(${img.rotation || 0}rad)`;
      }
      const box = boxRef.current;
      if (box) {
        box.style.transform = `translate(${left}px, ${top}px)`;
        box.style.width = `${img.width * s}px`;
        box.style.height = `${img.height * s}px`;
      }
    };
    const onVp = (e: Event) => {
      const { x, y, scale } = (e as CustomEvent<{ x: number; y: number; scale: number }>).detail;
      apply(x, y, scale);
    };
    window.addEventListener(CURTAIN_VIEWPORT_EVENT, onVp);
    const { x, y, scale } = vpRef.current;
    apply(x, y, scale);
    return () => window.removeEventListener(CURTAIN_VIEWPORT_EVENT, onVp);
  }, [img.x, img.y, img.width, img.height, img.rotation, vpRef, selected]);

  if (!url) return null;
  return (
    <>
      <img
        ref={ref}
        src={url}
        alt=""
        draggable={false}
        data-pick-owner="image"
        data-pick-id={img.id}
        style={{
          position: "absolute", top: 0, left: 0,
          width: img.width, height: img.height,
          transformOrigin: "top left",
          userSelect: "none",
        }}
      />
      {selected && (
        <div
          ref={boxRef}
          style={{
            position: "absolute", top: 0, left: 0,
            transformOrigin: "top left",
            border: "1px solid #e8e8e8",
            boxSizing: "border-box",
            pointerEvents: "none",
            zIndex: 12,
          }}
        >
          {(["tl", "tr", "bl", "br"] as const).map((c) => (
            <span
              key={c}
              style={{
                position: "absolute",
                width: 9, height: 9,
                background: "#e8e8e8",
                left: c === "tl" || c === "bl" ? -5 : undefined,
                right: c === "tr" || c === "br" ? -5 : undefined,
                top: c === "tl" || c === "tr" ? -5 : undefined,
                bottom: c === "bl" || c === "br" ? -5 : undefined,
              }}
            />
          ))}
        </div>
      )}
    </>
  );
}
