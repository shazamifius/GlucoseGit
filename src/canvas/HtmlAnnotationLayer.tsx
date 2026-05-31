import React, { useEffect, useRef, useState, useMemo, memo } from "react";
import ReactMarkdown, { type Components } from "react-markdown";
import remarkGfm from "remark-gfm";
import remarkMath from "remark-math";
import rehypeKatex from "rehype-katex";
// CLEANUP SEC-09 : `rehypeRaw` retiré pour bloquer XSS via Markdown user-input.
// Toute injection HTML brute (ex. <img onerror=...>) est désormais ignorée.
// CLEANUP B-03 : `katex.min.css` chargé à la demande (gain ~200 KB au boot).
import { ensureKatexCssIfMath } from "../utils/loadKatexCss";
import { invoke } from "@tauri-apps/api/core";
import { Annotation } from "../types";
import { useGlucoseStore } from "../store";
import AppBridgeIcon, { getAppDef } from "../components/AppBridgeIcon";

/** R-FIL — ouvre un fichier source dans son app native (double-clic tuile).
 *  Feedback immédiat (le lancement d'une grosse app comme Blender peut prendre
 *  10-30 s) pour éviter l'impression que "rien ne se passe". */
function openSourceFile(path: string) {
  const name = path.split(/[\\/]/).pop() || path;
  // Animation de lancement (logo + couleur dominante de l'app) — signal clair
  // que ça démarre, même si l'app met 10-40 s à apparaître (Blender).
  window.dispatchEvent(new CustomEvent("glucose:app-launching", { detail: { path } }));
  import("../components/Toast").then(({ showToast }) => showToast(`Ouverture de ${name}…`, "🚀"));
  invoke("open_in_app", { path }).catch(async (err) => {
    const { showToast } = await import("../components/Toast");
    showToast(`Impossible d'ouvrir : ${String(err)}`, "⚠️");
  });
}
import { MirrorBadge, DomainBadges, TemporalBadge, resolveDomainBadges } from "./AnnotationBadges";
import { nodeMatchesTemporalFilter } from "../utils/timeline";

interface Props {
  annotations: Annotation[];
  selectedIds: string[];
  editingId: string | null;
  vpRef: React.MutableRefObject<{ x: number; y: number; scale: number }>;
  onSelect: (id: string, multi: boolean) => void;
  onEdit: (id: string) => void;
  onResize: (id: string, x: number, y: number, w: number, h: number) => void;
}

// État interne du drag d'une annotation (déplacement OU resize via une corner).
interface DragState {
  id: string;
  startX: number;
  startY: number;
  pStartX: number;
  pStartY: number;
  didMove: boolean;
  t0: number;
  corner?: string;
  startW?: number;
  startH?: number;
}

// Survol d'une flèche : informations sur les nœuds source/cible et les
// sous-blocs (paragraphes) éventuellement ciblés. Utilisé pour surligner
// le texte exact pointé par la flèche.
interface HoveredBlocks {
  sourceId?: string;
  targetId?: string;
  sourceBlockId?: string;
  targetBlockId?: string;
  sourceTextSel?: string;
  targetTextSel?: string;
}

// Cible de prévisualisation (édition de flèche) — quel nœud / quel sous-bloc
// est en train d'être ciblé par la flèche en cours d'édition.
interface PreviewTarget {
  annId: string;
  blockId?: string;
}

// ⚠️ Les arrays de plugins DOIVENT être des constantes module-level (identité stable).
// Si on les inline dans le JSX, ReactMarkdown reçoit des refs neuves à chaque render,
// son useEffect interne se rejoue infiniment, et un unmount/remount pendant un re-render
// déclenche React error #310 ("rendered more hooks than during the previous render").
const REMARK_PLUGINS = [remarkGfm, remarkMath];
const REHYPE_PLUGINS = [rehypeKatex];

// Composants fixes pour react-markdown pour éviter le unmount/remount
const StableMarkdownComponents = {
  text: memo(function StableText({ processedText, components }: { processedText: string, components: Components }) {
    return (
      <ReactMarkdown
        remarkPlugins={REMARK_PLUGINS}
        rehypePlugins={REHYPE_PLUGINS}
        components={components}
      >
        {processedText}
      </ReactMarkdown>
    );
  })
};

export default function HtmlAnnotationLayer({
  annotations, selectedIds, editingId, vpRef, onSelect, onEdit, onResize
}: Props) {
  const containerRef = useRef<HTMLDivElement>(null);
  const dragRef = useRef<DragState | null>(null);
  const lastClickRef = useRef<{ id: string; time: number } | null>(null);
  const { activeTool } = useGlucoseStore();
  const setHoveredNodeId = useGlucoseStore(s => s.setHoveredNodeId);
  // ⚠️ Sélecteur Zustand : on récupère la ref directement, sans `?? []`,
  // pour ne pas créer une nouvelle ref à chaque render (qui ferait boucler React).
  // DEFAULT_PROJECT garantit toujours `domains: []`, et `loadProject` normalise.
  const domains = useGlucoseStore(s => s.project.domains) ?? [];
  // Phase 6 — réglette temporelle : atténue les nœuds hors fenêtre.
  const temporalFilter = useGlucoseStore(s => s.temporalFilter);

  // CLEANUP P-02 — Calcule les teintes symbiotiques en une passe au niveau parent
  // (memoïsé). Évite N×N à chaque render (1 par AnnotationItem qui re-itérait
  // sur toutes les annotations). Recalculé uniquement quand la liste change.
  const huesByAnnId = useMemo(() => {
    const map = new Map<string, number>();
    for (const a of annotations) {
      map.set(a.id, getSymbioticHue(a, annotations));
    }
    return map;
  }, [annotations]);

  const [hoveredBlocks, setHoveredBlocks] = useState<{
    sourceId?: string, sourceBlockId?: string, 
    targetId?: string, targetBlockId?: string,
    sourceTextSel?: string, targetTextSel?: string,
  } | null>(null);
  const [previewTarget, setPreviewTarget] = useState<{ annId: string, blockId?: string } | null>(null);
  const [guides, setGuides] = useState<{ x?: number[]; y?: number[] } | null>(null);

  useEffect(() => {
    const onHover = (e: Event) => setHoveredBlocks((e as CustomEvent).detail);
    const onPreview = (e: Event) => setPreviewTarget((e as CustomEvent).detail);
    window.addEventListener("glucose:hover-arrow", onHover);
    window.addEventListener("glucose:arrow-target-preview", onPreview);
    return () => {
      window.removeEventListener("glucose:hover-arrow", onHover);
      window.removeEventListener("glucose:arrow-target-preview", onPreview);
    };
  }, []);

  // Ref pour ResizeObserver local
  const resizeObserver = useRef<ResizeObserver | null>(null);

  useEffect(() => {
    // Synchronisation de la transformation CSS avec le viewport PixiJS
    let rafId: number;
    function updateTransform() {
      if (containerRef.current && vpRef.current) {
        const { x, y, scale } = vpRef.current;
        containerRef.current.style.transform = `translate(${x}px, ${y}px) scale(${scale})`;
      }
      rafId = requestAnimationFrame(updateTransform);
    }
    updateTransform();
    return () => cancelAnimationFrame(rafId);
  }, [vpRef]);

  useEffect(() => {
    // Observer pour mettre à jour la taille des annotations dans le store quand elles changent de taille
    resizeObserver.current = new ResizeObserver((entries) => {
      const boardId = useGlucoseStore.getState().project.activeBoardId;
      for (const entry of entries) {
        const id = (entry.target as HTMLElement).dataset.id;
        if (!id || id === editingId) continue;

        // On récupère la taille du layout original (non scalé)
        // entry.contentRect ou offsetWidth/offsetHeight (qui ne sont pas affectés par transform)
        const w = (entry.target as HTMLElement).offsetWidth;
        const h = (entry.target as HTMLElement).offsetHeight;

        // Comparaison avec les données actuelles pour éviter les boucles infinies
        const ann = useGlucoseStore.getState().project.boards
          .find(b => b.id === boardId)?.annotations.find(a => a.id === id);

        // On met à jour seulement s'il y a une différence notable.
        // Les flèches n'ont pas de width/height — elles utilisent (x,y) → (x2,y2).
        if (ann && ann.type !== "arrow"
            && (Math.abs((ann.width || 0) - w) > 2 || Math.abs((ann.height || 0) - h) > 2)) {
          useGlucoseStore.getState().updateAnnotation(boardId, id, { width: w, height: h });
        }
      }
    });

    return () => {
      resizeObserver.current?.disconnect();
    };
  }, [editingId]);

  function handleDown(ann: Annotation, e: React.PointerEvent, corner?: string) {
    if (e.button !== 0) return;
    if (activeTool !== "select") return;
    e.stopPropagation();

    const now = Date.now();
    if (lastClickRef.current?.id === ann.id && now - lastClickRef.current.time < 350) {
      lastClickRef.current = null;
      // R-FIL — Détection de double-clic FIABLE (basée sur pointerdown, pas sur
      // l'event dblclick natif qui ne se déclenche que si les 2 clics tombent au
      // même pixel → lancement « 1 fois sur 10 »). Pour une tuile fichier
      // (sourceFile : launcher OU bloc texte de dossier), double-clic = OUVRIR
      // dans l'app native, JAMAIS éditer le nom. C'est l'unique point de décision
      // du double-clic (handleDblClick ne relance pas → pas de double lancement).
      const sf = (ann as { sourceFile?: string }).sourceFile;
      if (sf) { openSourceFile(sf); return; }
      onEdit(ann.id);
      return;
    }
    lastClickRef.current = { id: ann.id, time: now };

    const isSelected = selectedIds.includes(ann.id);
    const multi = e.ctrlKey || e.metaKey || e.shiftKey;

    if (multi) {
      onSelect(ann.id, true);
    } else if (!isSelected) {
      onSelect(ann.id, false);
    }

    startDrag(ann, e, corner);
  }

  function handleDblClick(ann: Annotation, e: React.MouseEvent) {
    if (activeTool !== "select") return;
    e.stopPropagation();
    // Le double-clic est géré de façon FIABLE dans handleDown (pointerdown).
    // Ici on ne fait QUE consommer l'event natif dblclick pour qu'il ne remonte
    // pas au canvas. Surtout : pour une tuile fichier (sourceFile) on ne relance
    // PAS (sinon double lancement) et on n'édite JAMAIS le nom.
    const sf = (ann as { sourceFile?: string }).sourceFile;
    if (sf) return;
    // Annotation normale : handleDown a déjà ouvert l'édition au 2ᵉ pointerdown ;
    // on ne refait rien ici (idempotent) pour éviter tout effet de bord.
  }

  function screenToWorld(cx: number, cy: number) {
    const canvas = document.querySelector("canvas");
    if (!canvas) return { x: 0, y: 0 };
    const rect = canvas.getBoundingClientRect();
    const vx = vpRef.current.x, vy = vpRef.current.y, vs = vpRef.current.scale;
    return {
      x: (cx - rect.left - vx) / vs,
      y: (cy - rect.top - vy) / vs,
    };
  }

  function startDrag(ann: Annotation, ev: React.PointerEvent, corner?: string) {
    const { x: wx, y: wy } = screenToWorld(ev.clientX, ev.clientY);
    useGlucoseStore.getState().pushHistory();

    // width/height n'existent pas sur les flèches (elles utilisent x2/y2).
    const annW = ann.type === "arrow" ? 160 : (ann.width ?? 160);
    const annH = ann.type === "arrow" ? 120 : (ann.height ?? 120);

    dragRef.current = {
      id: ann.id,
      startX: ann.x, startY: ann.y,
      pStartX: wx, pStartY: wy,
      didMove: false, t0: Date.now(),
      corner, startW: annW, startH: annH,
    };

    function onGlobalMove(ev: PointerEvent) {
      const ds = dragRef.current;
      if (!ds) return;
      const { x: wx2, y: wy2 } = screenToWorld(ev.clientX, ev.clientY);
      const dx = wx2 - ds.pStartX;
      const dy = wy2 - ds.pStartY;
      if (Math.abs(dx) + Math.abs(dy) > 2) ds.didMove = true;
      if (!ds.didMove) return;

      if (ds.corner) {
        const sw = ds.startW ?? 160;
        const sh = ds.startH ?? 120;
        let nx = ds.startX, ny = ds.startY, nw = sw, nh = sh;
        if (ds.corner === "br") { nw = Math.max(60, sw + dx); nh = Math.max(40, sh + dy); }
        else if (ds.corner === "bl") { nw = Math.max(60, sw - dx); nh = Math.max(40, sh + dy); nx = ds.startX + (sw - nw); }
        else if (ds.corner === "tr") { nw = Math.max(60, sw + dx); nh = Math.max(40, sh - dy); ny = ds.startY + (sh - nh); }
        else if (ds.corner === "tl") { nw = Math.max(60, sw - dx); nh = Math.max(40, sh - dy); nx = ds.startX + (sw - nw); ny = ds.startY + (sh - nh); }
        onResize(ds.id, nx, ny, nw, nh);
      } else {
        const boardId = useGlucoseStore.getState().project.activeBoardId;
        const board = useGlucoseStore.getState().project.boards.find(b => b.id === boardId);
        const smartEnabled = useGlucoseStore.getState().smartGuidesEnabled;
        
        let finalDX = wx2 - ds.pStartX;
        let finalDY = wy2 - ds.pStartY;

        // Snapping intelligent
        if (smartEnabled && selectedIds.length === 1 && board) {
          const ann = board.annotations.find(a => a.id === ds.id);
          if (ann && ann.type !== "arrow") {
            const SNAP_DIST = 8;
            const currentX = ann.x + finalDX;
            const currentY = ann.y + finalDY;
            const w = ann.width || 200;
            const h = ann.height || 100;

            const targetsX: { val: number; type: string }[] = [];
            const targetsY: { val: number; type: string }[] = [];

            board.annotations.forEach(other => {
              if (other.id === ann.id || (other.type !== "text" && other.type !== "sticky")) return;
              const ow = other.width || 200;
              const oh = other.height || 100;
              targetsX.push({ val: other.x, type: "left" });
              targetsX.push({ val: other.x + ow, type: "right" });
              targetsX.push({ val: other.x + ow / 2, type: "center" });
              targetsY.push({ val: other.y, type: "top" });
              targetsY.push({ val: other.y + oh, type: "bottom" });
              targetsY.push({ val: other.y + oh / 2, type: "center" });
            });

            let snapX: number | undefined;
            let snapY: number | undefined;

            const myXPoints = [
              { val: currentX, type: "left" },
              { val: currentX + w, type: "right" },
              { val: currentX + w / 2, type: "center" }
            ];
            for (const myP of myXPoints) {
              for (const target of targetsX) {
                if (Math.abs(myP.val - target.val) < SNAP_DIST) {
                  snapX = target.val;
                  finalDX = target.val - (myP.type === "left" ? ann.x : myP.type === "right" ? ann.x + w : ann.x + w / 2);
                  break;
                }
              }
              if (snapX !== undefined) break;
            }

            const myYPoints = [
              { val: currentY, type: "top" },
              { val: currentY + h, type: "bottom" },
              { val: currentY + h / 2, type: "center" }
            ];
            for (const myP of myYPoints) {
              for (const target of targetsY) {
                if (Math.abs(myP.val - target.val) < SNAP_DIST) {
                  snapY = target.val;
                  finalDY = target.val - (myP.type === "top" ? ann.y : myP.type === "bottom" ? ann.y + h : ann.y + h / 2);
                  break;
                }
              }
              if (snapY !== undefined) break;
            }

            setGuides({ 
              x: snapX !== undefined ? [snapX] : undefined, 
              y: snapY !== undefined ? [snapY] : undefined 
            });
          }
        } else {
          setGuides(null);
        }

        if (Math.abs(finalDX) > 0.01 || Math.abs(finalDY) > 0.01) {
          ds.pStartX += finalDX;
          ds.pStartY += finalDY;
          useGlucoseStore.getState().moveSelected(boardId, finalDX, finalDY);
        }
      }
    }

    function onGlobalUp(ev: PointerEvent) {
      const ds = dragRef.current;
      if (ds && !ds.didMove && Date.now() - ds.t0 < 500 && !ds.corner) {
        const multi = ev.ctrlKey || ev.metaKey || ev.shiftKey;
        if (!multi) {
          onSelect(ds.id, false);
        }
      }
      dragRef.current = null;
      setGuides(null);
      window.removeEventListener("pointermove", onGlobalMove);
      window.removeEventListener("pointerup", onGlobalUp);
    }

    window.addEventListener("pointermove", onGlobalMove);
    window.addEventListener("pointerup", onGlobalUp);
  }

  return (
    <div
      style={{
        position: "absolute", inset: 0,
        width: "100%", height: "100%",
        pointerEvents: "none",
        zIndex: 2, // au dessus du canvas PixiJS
        overflow: "visible",
      }}
    >
      <div
        ref={containerRef}
        style={{ transformOrigin: "0 0", position: "absolute", left: 0, top: 0, right: 0, bottom: 0, pointerEvents: "none" }}
      >
        {annotations.map((ann) => {
          if (ann.id === editingId) return null;
          // Phase 6 — atténuation temporelle (n'affecte pas les nœuds atemporels)
          const dimmed = !nodeMatchesTemporalFilter(ann.temporalAnchor, temporalFilter);
          const item = (
            <AnnotationItem
              key={ann.id}
              ann={ann}
              allAnnotations={annotations}
              auraHue={huesByAnnId.get(ann.id) ?? 0}
              selected={selectedIds.includes(ann.id)}
              activeTool={activeTool}
              domains={domains}
              setHoveredNodeId={setHoveredNodeId}
              onEdit={onEdit}
              handleDown={handleDown}
              handleDblClick={handleDblClick}
              resizeObserver={resizeObserver}
              hoveredBlocks={hoveredBlocks}
              previewTarget={previewTarget}
            />
          );
          if (!dimmed) return item;
          return (
            <div key={ann.id} style={{ opacity: 0.12, pointerEvents: "none", transition: "opacity 200ms" }}>
              {item}
            </div>
          );
        })}

        {/* Guides intelligents d'alignement — Style léger et professionnel */}
        {guides?.x?.map(gx => (
          <div key={`gx-${gx}`} style={{
            position: "absolute", left: gx, top: -10000, bottom: -10000,
            width: 1 / vpRef.current.scale, 
            borderLeft: `${1 / vpRef.current.scale}px dashed rgba(255, 255, 255, 0.3)`,
            zIndex: 1000, pointerEvents: "none"
          }} />
        ))}
        {guides?.y?.map(gy => (
          <div key={`gy-${gy}`} style={{
            position: "absolute", top: gy, left: -10000, right: -10000,
            height: 1 / vpRef.current.scale, 
            borderTop: `${1 / vpRef.current.scale}px dashed rgba(255, 255, 255, 0.3)`,
            zIndex: 1000, pointerEvents: "none"
          }} />
        ))}


      </div>
    </div>
  );
}

function preprocessText(text: string) {
  let t = text.replace(/^-# (.*)$/gm, '<span class="text-[0.65em] opacity-75">$1</span>');
  // Préserver les lignes vides multiples en injectant un espace insécable
  t = t.replace(/\n(?=\n)/g, '\n&nbsp;');
  return t;
}

/** TXT-1 — Borne le contenu rendu dans une tuile texte de dossier : on retire
 *  le header markdown « ### 📄 nom » (redondant avec l'en-tête de la tuile) et
 *  le footer « tronqué », puis on clippe à `maxLines` (en refermant un fence de
 *  code resté ouvert) → coût du pipeline markdown BORNÉ par tuile. */
export function clipTextForTile(raw: string, maxLines = 60): string {
  const t = (raw || "")
    .replace(/^###\s*📄[^\n]*\n+/, "")
    .replace(/_\(tronqué[^)]*\)_/g, "");
  const lines = t.split("\n");
  if (lines.length <= maxLines) return t.trim();
  let clipped = lines.slice(0, maxLines).join("\n");
  // Referme un éventuel bloc de code (```) laissé ouvert par la coupe.
  if (((clipped.match(/```/g) || []).length) % 2 === 1) clipped += "\n```";
  return `${clipped.trim()}\n…`;
}

// ── TXT-2 — Coloration syntaxique LÉGÈRE (sans dépendance) ───────────────────
// Tokeniseur générique multi-langage (mots-clés / chaînes / nombres / commentaires)
// avec couleurs façon VSCode Dark. Volontairement léger (cf. plafond perf documenté
// `annotation-layer-no-culling`) : le contenu est déjà clippé, donc le coût est borné.
const CODE_KEYWORDS = new Set([
  "if","else","elif","for","while","do","switch","case","default","break","continue",
  "return","yield","def","function","fn","func","class","struct","enum","interface","trait",
  "impl","import","from","export","include","using","namespace","package","module","require",
  "const","let","var","val","mut","static","public","private","protected","final","abstract",
  "new","delete","this","self","super","null","none","nil","true","false","void","async","await",
  "try","catch","except","finally","throw","raise","with","as","in","is","not","and","or","del",
  "lambda","pass","global","nonlocal","match","where","then","begin","end","use","pub","extern",
  "unsafe","move","ref","dyn","typedef","typename","template","operator","goto","sizeof","auto",
  "int","float","str","bool","string","number","boolean","char","double","long","short","let",
]);

/** Préfixe de commentaire de ligne selon le langage du fence. */
function lineCommentToken(lang: string): string {
  if (/^(py|python|rb|ruby|sh|bash|zsh|fish|ya?ml|toml|ini|r|pl|perl|makefile|dockerfile|conf|cfg|env|nim|jl|julia|ex|exs|elixir|cr|crystal|tcl)/.test(lang)) return "#";
  if (/^(sql|lua|hs|haskell|elm|ada|vhdl)/.test(lang)) return "--";
  return "//";
}

/** Découpe `code` en spans colorés (commentaires, chaînes, nombres, mots-clés). */
export function highlightCode(code: string, lang: string): React.ReactNode[] {
  const lc = lineCommentToken(lang);
  const re = new RegExp(
    [
      `(${lc}[^\\n]*)`,                                                                  // 1 commentaire ligne
      `(/\\*[\\s\\S]*?\\*/)`,                                                            // 2 commentaire bloc
      `("""[\\s\\S]*?"""|'''[\\s\\S]*?'''|"(?:\\\\.|[^"\\\\])*"|'(?:\\\\.|[^'\\\\])*'|\`(?:\\\\.|[^\`\\\\])*\`)`, // 3 chaîne
      `(\\b\\d[\\d_]*\\.?\\d*(?:[eE][+-]?\\d+)?\\b)`,                                    // 4 nombre
      `([A-Za-z_$][\\w$]*)`,                                                             // 5 identifiant
    ].join("|"),
    "g",
  );
  const out: React.ReactNode[] = [];
  let last = 0;
  let key = 0;
  let m: RegExpExecArray | null = re.exec(code);
  while (m !== null) {
    if (m.index > last) out.push(code.slice(last, m.index));
    let color = "";
    if (m[1] || m[2]) color = "#6a9955";            // commentaire (vert)
    else if (m[3]) color = "#ce9178";               // chaîne (orange)
    else if (m[4]) color = "#b5cea8";               // nombre (vert clair)
    else if (m[5] && CODE_KEYWORDS.has(m[5])) color = "#569cd6"; // mot-clé (bleu)
    out.push(color ? <span key={key++} style={{ color }}>{m[0]}</span> : m[0]);
    last = re.lastIndex;
    m = re.exec(code);
  }
  if (last < code.length) out.push(code.slice(last));
  return out;
}

// biome-ignore lint/suspicious/noExplicitAny: props react-markdown hétérogènes (node + HTML)
function MdPre({ children }: any) {
  return (
    <pre style={{
      margin: "4px 0", padding: "7px 9px", background: "#1e1e2e", borderRadius: 6,
      overflow: "hidden", fontFamily: "ui-monospace, monospace", fontSize: 11,
      lineHeight: 1.45, whiteSpace: "pre-wrap", wordBreak: "break-word",
    }}>
      {children}
    </pre>
  );
}

// biome-ignore lint/suspicious/noExplicitAny: props react-markdown hétérogènes (node + HTML)
function MdCode({ className, children }: any) {
  const text = String(children ?? "").replace(/\n$/, "");
  const isBlock = /language-/.test(className || "");
  if (!isBlock) {
    return (
      <code style={{
        background: "rgba(255,255,255,0.08)", padding: "1px 4px", borderRadius: 4,
        fontFamily: "ui-monospace, monospace", fontSize: "0.92em",
      }}>
        {text}
      </code>
    );
  }
  const lang = (/language-(\w+)/.exec(className || "")?.[1] ?? "").toLowerCase();
  return (
    <code style={{ fontFamily: "ui-monospace, monospace", color: "#d4d4d4" }}>
      {highlightCode(text, lang)}
    </code>
  );
}

export function getSymbioticHue(ann: Annotation, allAnnotations: Annotation[]): number {
  const idHash = (id: string) => {
    let h = 5381;
    for (let i = 0; i < id.length; i++) h = ((h << 5) + h) + id.charCodeAt(i);
    return Math.abs(h);
  };

  // 1. TOUTES LES COULEURS DU MONDE (Bruit Organique)
  const getZoneHue = (x: number, y: number) => {
    const scale = 2000; // Les zones de couleur changent tous les ~2000 pixels
    const cx = x / scale;
    const cy = y / scale;

    const random2D = (ix: number, iy: number) => {
      const dot = ix * 12.9898 + iy * 78.233;
      const sin = Math.sin(dot) * 43758.5453;
      return sin - Math.floor(sin);
    };

    const smooth = (t: number) => t * t * (3 - 2 * t);

    const x0 = Math.floor(cx);
    const x1 = x0 + 1;
    const y0 = Math.floor(cy);
    const y1 = y0 + 1;

    const sx = smooth(cx - x0);
    const sy = smooth(cy - y0);

    const nx0 = random2D(x0, y0) * (1 - sx) + random2D(x1, y0) * sx;
    const nx1 = random2D(x0, y1) * (1 - sx) + random2D(x1, y1) * sx;
    const value = nx0 * (1 - sy) + nx1 * sy;

    // ON DÉBLOQUE TOUT : 0 à 360 degrés !
    return value * 360; 
  };

  // Identité de base (Biome + variation unique du bloc)
  let myBaseHue = getZoneHue(ann.x, ann.y) + ((idHash(ann.id) % 80) - 40);
  myBaseHue = ((myBaseHue % 360) + 360) % 360;

  // 2. L'ALGORITHME DE DÉGRADÉ POUSSÉ (Moyenne Vectorielle Circulaire)
  const RAYON = 1200; 
  
  // Pour ne pas créer de couleurs moches ou de sauts étranges, on ne fait pas 
  // une bête moyenne mathématique. On utilise des vecteurs (Trigonométrie).
  let sumX = 0;
  let sumY = 0;
  let envWeightSum = 0;

  for (const other of allAnnotations) {
    if (other.id === ann.id || other.type !== "text") continue;
    
    const dx = ann.x - other.x;
    if (Math.abs(dx) > RAYON) continue;
    const dy = ann.y - other.y;
    if (Math.abs(dy) > RAYON) continue;
    
    const dist = Math.sqrt(dx * dx + dy * dy);
    if (dist < RAYON) {
      const weight = Math.pow(1 - (dist / RAYON), 2);
      
      const otherHue = getZoneHue(other.x, other.y) + ((idHash(other.id) % 80) - 40);
      
      // Conversion de la couleur en vecteur (angle -> x, y)
      const rad = otherHue * (Math.PI / 180);
      sumX += Math.cos(rad) * weight;
      sumY += Math.sin(rad) * weight;
      
      envWeightSum += weight;
    }
  }

  // 3. SYMBIOSE PARFAITE
  if (envWeightSum > 0) {
    // On retrouve l'angle moyen exact de l'environnement (sans aucun saut au passage par 0)
    let envHue = Math.atan2(sumY, sumX) * (180 / Math.PI);
    if (envHue < 0) envHue += 360;
    
    // Calcul de l'écart via le chemin le plus court sur la roue des couleurs
    let diff = envHue - myBaseHue;
    if (diff > 180) diff -= 360;
    if (diff < -180) diff += 360;
    
    // L'environnement tire doucement le bloc vers lui (dégradé très harmonieux)
    const influence = 0.5 * (1 - 1 / (1 + envWeightSum));
    myBaseHue += (diff * influence);
  }

  return ((myBaseHue % 360) + 360) % 360;
}

function forwardWheel(e: React.WheelEvent) {
  const canvas = document.querySelector("canvas");
  if (!canvas) return;
  canvas.dispatchEvent(new WheelEvent("wheel", {
    deltaX: e.deltaX, deltaY: e.deltaY, deltaZ: e.deltaZ, deltaMode: e.deltaMode,
    clientX: e.clientX, clientY: e.clientY,
    ctrlKey: e.ctrlKey, shiftKey: e.shiftKey, altKey: e.altKey, metaKey: e.metaKey,
    bubbles: false, cancelable: true,
  }));
  e.preventDefault();
}

function AnnotationItem({
  ann, allAnnotations: _allAnnotations, auraHue, selected: sel, activeTool, domains, setHoveredNodeId,
  onEdit, handleDown, handleDblClick, resizeObserver, hoveredBlocks, previewTarget
}: {
  ann: Annotation, allAnnotations: Annotation[],
  /** Phase P-02 : pré-calculé par le parent en une passe (Map) — évite N×N. */
  auraHue: number,
  selected: boolean, activeTool: string,
  domains: import("../types").Domain[],
  setHoveredNodeId: (id: string | null) => void,
  onEdit: (id: string) => void,
  handleDown: (ann: Annotation, e: React.PointerEvent, corner?: string) => void,
  handleDblClick: (ann: Annotation, e: React.MouseEvent) => void,
  resizeObserver: React.MutableRefObject<ResizeObserver | null>,
  hoveredBlocks: HoveredBlocks | null,
  previewTarget: PreviewTarget | null,
}) {
  // CLEANUP B-03 — déclenche le chargement à la demande du CSS KaTeX
  // si l'annotation contient du LaTeX.
  ensureKatexCssIfMath(ann.text);
  const domainBadges = resolveDomainBadges(ann.domains, domains);
  // CLEANUP P-02 : auraHue vient désormais du parent (Map mémoïsée) — plus de calcul N×N par render
  const isBaseWhite = !ann.color || ann.color === "#ffffff" || ann.color === "#fff";
  const auraColor = isBaseWhite ? `hsl(${auraHue}, 75%, 65%)` : ann.color;
  const annRef = useRef<HTMLDivElement>(null);

  // ── Surlignage du texte exact quand une flèche est survolée ──
  useEffect(() => {
    if (!annRef.current || !hoveredBlocks) return;
    
    let textToHighlight: string | undefined;
    const hlColor = auraColor; // Utiliser la couleur symbiotique du bloc
    
    if (hoveredBlocks.sourceId === ann.id && hoveredBlocks.sourceTextSel) {
      textToHighlight = hoveredBlocks.sourceTextSel;
    }
    if (hoveredBlocks.targetId === ann.id && hoveredBlocks.targetTextSel) {
      textToHighlight = hoveredBlocks.targetTextSel;
    }

    if (!textToHighlight) return;

    const container = annRef.current;
    const marks: HTMLElement[] = [];
    
    const selections = textToHighlight.split(" ‖ ").map(s => s.trim()).filter(Boolean);
    
    for (const sel of selections) {
      const walker = document.createTreeWalker(container, NodeFilter.SHOW_TEXT, null);
      const textNodes: Text[] = [];
      let nd: Node | null;
      while ((nd = walker.nextNode())) {
        if ((nd.parentNode as HTMLElement)?.getAttribute?.("data-glucose-hl")) continue;
        textNodes.push(nd as Text);
      }
      
      for (const textNode of textNodes) {
        const nodeText = textNode.textContent || "";
        const idx = nodeText.toLowerCase().indexOf(sel.toLowerCase());
        if (idx === -1) continue;
        
        try {
          const before = nodeText.slice(0, idx);
          const match = nodeText.slice(idx, idx + sel.length);
          const after = nodeText.slice(idx + sel.length);
          
          const mark = document.createElement("mark");
          mark.setAttribute("data-glucose-hl", "true");
          mark.style.cssText = `
            background: color-mix(in srgb, ${hlColor} 20%, transparent);
            color: ${hlColor};
            border-radius: 3px;
            padding: 1px 3px;
            box-shadow: 0 0 16px 6px color-mix(in srgb, ${hlColor} 25%, transparent);
            outline: 1.5px solid color-mix(in srgb, ${hlColor} 45%, transparent);
            outline-offset: 2px;
            transition: all .15s ease-out;
          `;
          mark.textContent = match;
          
          const parent = textNode.parentNode;
          if (!parent) continue;
          
          if (after) parent.insertBefore(document.createTextNode(after), textNode.nextSibling);
          parent.insertBefore(mark, textNode.nextSibling);
          textNode.textContent = before;
          
          marks.push(mark);
        } catch {}
        break;
      }
    }

    return () => {
      marks.forEach(mark => {
        const parent = mark.parentNode;
        if (parent) {
          const text = mark.textContent || "";
          parent.insertBefore(document.createTextNode(text), mark);
          parent.removeChild(mark);
          parent.normalize();
        }
      });
    };
  }, [hoveredBlocks, ann.id]);

  // On mémoise les composants pour éviter le remount à chaque render
  const markdownComponents = useMemo(() => {
            const createBlockRenderer = (Tag: React.ElementType) => {
              // react-markdown injecte un mix de props HTML intrinsèques (variant selon Tag) +
              // un `node` mdast/hast — typer strictement obligerait à décliner BlockComponent
              // par Tag (h1/h2/p/li) avec ComponentProps<Tag>, ce qui multiplie le code par 5
              // sans gain de sûreté côté usage.
              // biome-ignore lint/suspicious/noExplicitAny: voir commentaire ci-dessus
              return function BlockComponent({ node, children, ...props }: any) {
                const line = node?.position?.start?.line;
                if (!line) return <Tag {...props}>{children}</Tag>;
                const blockId = `L${line}`;
                const isHeader = Tag === 'h1' || Tag === 'h2' || Tag === 'h3';
                const isHoveredBlock = hoveredBlocks && (hoveredBlocks.sourceBlockId === blockId || hoveredBlocks.targetBlockId === blockId) && (hoveredBlocks.sourceId === ann.id || hoveredBlocks.targetId === ann.id);
                const isPreviewedBlock = previewTarget && previewTarget.annId === ann.id && previewTarget.blockId === blockId;
                const isHighlight = isHoveredBlock || isPreviewedBlock;

                return (
                  <Tag
                    {...props}
                    className={`group relative glucose-sub-block ${props.className || ''}`}
                    data-ann-id={ann.id}
                    data-block-id={blockId}
                    style={{
                      ...(isHeader && ann.color ? { textShadow: `0 0 12px ${ann.color}80, 0 0 24px ${ann.color}40` } : {}),
                      ...(isHighlight ? { 
                         background: `color-mix(in srgb, ${auraColor} 60%, transparent)`, 
                         outline: `3px solid ${auraColor}`, 
                         outlineOffset: "4px", 
                         borderRadius: "4px", 
                         zIndex: 10,
                         boxShadow: `0 0 40px 10px color-mix(in srgb, ${auraColor} 80%, transparent)` // x4 lueur
                      } : {}),
                      transition: "all 0.2s ease-out"
                    }}
                    onDoubleClick={(e: React.MouseEvent) => {
                      if (activeTool !== "select") return;
                      e.stopPropagation();
                      onEdit(ann.id);
                    }}
                  >
                    {children}
                  </Tag>
                );
              };
            };
            return {
              p: createBlockRenderer('p'),
              h1: createBlockRenderer('h1'),
              h2: createBlockRenderer('h2'),
              h3: createBlockRenderer('h3'),
              li: createBlockRenderer('li'),
              // TXT-2 — code coloré (façon VSCode) dans les blocs fenced.
              pre: MdPre,
              code: MdCode,
            };
          }, [ann.id, ann.color, activeTool, onEdit]);

          // ── LOD ──
          // macro : marqueur de région (aura/bgColor visible, pas d'interaction)
          // meso  : titre seul (1ʳᵉ ligne tronquée, taille compacte)
          // Phase 7.5 — LOD supprimé : rendu complet inconditionnel.
          const lodSourceText = ann.text || "Texte";

          if (ann.type === "text") {
            const processedText = preprocessText(lodSourceText);

            // ── R-FIL — Tuile texte d'un FOLDER MIRROR (a un sourceFile) ──────
            // Rendue en CARTE CLAMPÉE (taille fixe, contenu clippé) au lieu du
            // `fit-content` qui faisait grandir la boîte à la hauteur réelle du
            // fichier (des milliers de px) → les tuiles texte se chevauchaient
            // en « pagaille ». Aperçu des 1ères lignes + double-clic = ouvrir.
            const txtSource = (ann as { sourceFile?: string }).sourceFile;
            if (txtSource) {
              const tw = ann.width ?? 210;
              const th = ann.height ?? 180;
              const fname = txtSource.split(/[\\/]/).pop() || "Fichier";
              // TXT-1 — VRAI texte rendu (markdown + LaTeX via react-markdown),
              // clippé à la tuile (taille fixe calculée au scan → le ResizeObserver
              // relit la même taille, pas de croissance/chevauchement). Le clip
              // borne le coût du pipeline markdown par tuile.
              const tileBody = clipTextForTile(ann.text || "");
              return (
                <div
                  key={ann.id}
                  data-id={ann.id}
                  ref={(el) => { if (el && resizeObserver.current) resizeObserver.current.observe(el); }}
                  title={`${fname} — double-clic pour ouvrir`}
                  style={{
                    position: "absolute",
                    left: ann.x, top: ann.y,
                    width: tw, height: th,
                    boxSizing: "border-box",
                    background: "#14141c",
                    border: sel ? "1px solid #fff" : "1px solid #2c2c3a",
                    borderRadius: 10,
                    overflow: "hidden",
                    pointerEvents: "all",
                    cursor: "pointer",
                    boxShadow: sel ? "0 0 0 2px #fff, 0 6px 14px rgba(0,0,0,0.5)" : "0 4px 10px rgba(0,0,0,0.4)",
                    display: "flex", flexDirection: "column",
                  }}
                  onPointerDown={(e) => handleDown(ann, e)}
                  onDoubleClick={(e) => handleDblClick(ann, e)}
                  onWheel={forwardWheel}
                  onMouseEnter={() => setHoveredNodeId(ann.id)}
                  onMouseLeave={() => setHoveredNodeId(null)}
                >
                  {/* En-tête : icône + nom */}
                  <div style={{
                    display: "flex", alignItems: "center", gap: 6,
                    padding: "5px 8px", flexShrink: 0,
                    background: "rgba(255,255,255,0.05)",
                    borderBottom: "1px solid #2c2c3a",
                  }}>
                    <AppBridgeIcon filePath={txtSource} size={16} />
                    <span style={{
                      fontSize: 11, fontWeight: 600, color: "#e8e8f0",
                      overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap",
                    }}>
                      {fname}
                    </span>
                  </div>
                  {/* TXT-1 — contenu rendu (markdown + LaTeX), clippé + fondu bas */}
                  <div style={{ position: "relative", flex: 1, overflow: "hidden" }}>
                    <div
                      className="glucose-tile-md"
                      style={{
                        padding: "6px 9px", fontSize: 11, lineHeight: 1.4,
                        color: "#c8cce0", wordBreak: "break-word",
                      }}
                    >
                      <StableMarkdownComponents.text processedText={tileBody} components={markdownComponents} />
                    </div>
                    <div style={{
                      position: "absolute", left: 0, right: 0, bottom: 0, height: 28,
                      background: "linear-gradient(to bottom, rgba(20,20,28,0), #14141c)",
                      pointerEvents: "none",
                    }} />
                  </div>

                  <MirrorBadge mirrorOf={ann.mirrorOf} />
                  <TemporalBadge anchor={ann.temporalAnchor} />

                  {sel && [
                    { id: "br", cx: tw, cy: th }, { id: "bl", cx: 0, cy: th },
                    { id: "tr", cx: tw, cy: 0 }, { id: "tl", cx: 0, cy: 0 },
                  ].map((c) => (
                    <div
                      key={c.id}
                      onPointerDown={(e) => { e.stopPropagation(); handleDown(ann, e, c.id); }}
                      style={{
                        position: "absolute",
                        left: c.cx - 12, top: c.cy - 12,
                        width: 24, height: 24,
                        cursor: c.id === "br" || c.id === "tl" ? "nwse-resize" : "nesw-resize",
                        zIndex: 10,
                        display: "flex", alignItems: "center", justifyContent: "center",
                      }}
                    >
                      <div style={{ width: 8, height: 8, background: "#fff", border: "1px solid #333", borderRadius: "50%" }} />
                    </div>
                  ))}
                </div>
              );
            }

            const isHoveredBox = hoveredBlocks && (hoveredBlocks.sourceId === ann.id || hoveredBlocks.targetId === ann.id) && !hoveredBlocks.sourceBlockId && !hoveredBlocks.targetBlockId;
            const isPreviewedBox = previewTarget && previewTarget.annId === ann.id && !previewTarget.blockId;
            const isHighlightBox = isHoveredBox || isPreviewedBox;

            return (
              <div
                key={ann.id}
                data-id={ann.id}
                ref={(el) => { 
                  annRef.current = el;
                  if (el && resizeObserver.current) resizeObserver.current.observe(el); 
                }}
                style={{
                  '--aura-color': auraColor,
                  position: "absolute",
                  left: ann.x, top: ann.y,
                  width: ann.width || "max-content",
                  minWidth: "min-content",
                  maxWidth: ann.width ? undefined : 600,
                  height: "fit-content",
                  minHeight: ann.height,
                  pointerEvents: activeTool === "select" ? "all" : "none",
                  cursor: activeTool === "select" ? "move" : "default",
                  color: ann.color || "#ffffff",
                  fontSize: ann.fontSize || 14,
                  // La douce brume en fond
                  background: `color-mix(in srgb, ${auraColor} 3%, transparent)`,
                  boxShadow: isHighlightBox 
                    ? `0 0 80px 40px color-mix(in srgb, ${auraColor} 40%, transparent)`
                    : `0 0 60px 30px color-mix(in srgb, ${auraColor} 15%, transparent)`,
                  padding: "16px 24px",
                  borderRadius: "32px", // Coins très arrondis pour accentuer l'effet nuage/brume
                  transition: "box-shadow 0.2s, outline 0.2s",
                  outline: (sel && activeTool === "select") ? "1px dashed rgba(255,255,255,0.5)" : "none",
                  outlineOffset: "4px",
                  // `--aura-color` est une CSS custom property exposée par cette annotation pour
                  // symbiose chromatique de ses enfants. React.CSSProperties ne valide pas les
                  // propriétés `--*` ; le cast contourne ça.
                  // biome-ignore lint/suspicious/noExplicitAny: voir commentaire ci-dessus
                } as any}
                onPointerDown={(e) => handleDown(ann, e)}
                onDoubleClick={(e) => handleDblClick(ann, e)}
                onWheel={forwardWheel}
                onMouseEnter={() => setHoveredNodeId(ann.id)}
                onMouseLeave={() => setHoveredNodeId(null)}
              >
                <div className="prose prose-invert prose-sm max-w-none break-words glucose-text-block" style={{ color: "inherit", fontSize: "inherit", lineHeight: 1.4, whiteSpace: "pre-wrap" }}>
                  <StableMarkdownComponents.text processedText={processedText} components={markdownComponents} />
                </div>

                <DomainBadges badges={domainBadges} />
                <MirrorBadge mirrorOf={ann.mirrorOf} />
                <TemporalBadge anchor={ann.temporalAnchor} />

                {/* Poignées de redimensionnement pour le texte (micro uniquement — en méso la boîte s'adapte au titre) */}
                {sel && activeTool === "select" && [
                  { id: "br", cx: ann.width || 0, cy: ann.height || 0 },
                  { id: "bl", cx: 0, cy: ann.height || 0 },
                  { id: "tr", cx: ann.width || 0, cy: 0 },
                  { id: "tl", cx: 0, cy: 0 },
                ].map((c) => (
                  <div
                    key={c.id}
                    onPointerDown={(e) => { e.stopPropagation(); handleDown(ann, e, c.id); }}
                    style={{
                      position: "absolute",
                      left: c.id.includes('r') ? "calc(100% - 12px)" : "-12px",
                      top: c.id.includes('b') ? "calc(100% - 12px)" : "-12px",
                      width: 24, height: 24,
                      cursor: c.id === "br" || c.id === "tl" ? "nwse-resize" : "nesw-resize",
                      zIndex: 10,
                      display: "flex", alignItems: "center", justifyContent: "center",
                    }}
                  >
                    <div style={{
                      width: 8, height: 8,
                      background: "#fff", border: "1px solid #333",
                      borderRadius: "50%",
                    }} />
                  </div>
                ))}
              </div>
            );
          }

          // Sticky-opérateur logique (Phase 5) — rendu compact avec glyphe central, prioritaire sur le rendu sticky standard
          if (ann.type === "sticky" && ann.operator) {
            const OP_LABELS: Record<string, { label: string; color: string }> = {
              AND:     { label: "ET",       color: "#34d399" }, // vert : conjonction positive
              OR:      { label: "OU",       color: "#60a5fa" }, // bleu : alternative
              BUT:     { label: "MAIS",     color: "#f59e0b" }, // ambre : nuance
              BECAUSE: { label: "PARCE QUE", color: "#a78bfa" }, // violet : causalité
            };
            const op = OP_LABELS[ann.operator];
            const opW = ann.width ?? (ann.operator === "BECAUSE" ? 130 : 80);
            const opH = ann.height ?? 44;
            return (
              <div
                key={ann.id}
                data-id={ann.id}
                ref={(el) => { if (el && resizeObserver.current) resizeObserver.current.observe(el); }}
                style={{
                  position: "absolute",
                  left: ann.x, top: ann.y,
                  width: opW, height: opH,
                  background: `${op.color}20`,
                  border: `1.5px solid ${op.color}`,
                  borderRadius: 22,
                  display: "flex", alignItems: "center", justifyContent: "center",
                  pointerEvents: activeTool === "select" ? "all" : "none",
                  cursor: activeTool === "select" ? "move" : "default",
                  color: op.color, fontSize: 13, fontWeight: 700,
                  letterSpacing: 1, textTransform: "uppercase",
                  boxShadow: sel ? `0 0 0 2px #fff, 0 0 18px ${op.color}55` : `0 0 18px ${op.color}33`,
                  userSelect: "none",
                }}
                onPointerDown={(e) => handleDown(ann, e)}
                onDoubleClick={(e) => handleDblClick(ann, e)}
                onWheel={forwardWheel}
                onMouseEnter={() => setHoveredNodeId(ann.id)}
                onMouseLeave={() => setHoveredNodeId(null)}
              >
                {op.label}
                <MirrorBadge mirrorOf={ann.mirrorOf} />
                <TemporalBadge anchor={ann.temporalAnchor} />
              </div>
            );
          }

          if (ann.type === "sticky") {
            const w = ann.width ?? 160;
            const h = ann.height ?? 120;
            const bg = ann.bgColor ?? "#f5c542";
            const processedText = preprocessText(ann.text || "");
            // const HANDLE = 7;
            const corners = [
              { id: "br", cx: w, cy: h },
              { id: "bl", cx: 0, cy: h },
              { id: "tr", cx: w, cy: 0 },
              { id: "tl", cx: 0, cy: 0 },
            ];

            const isSource = !!ann.sourceFile;
            const title = ann.sourceFile?.split(/[\\/]/).pop() || "Code Source";

            // ── Fichier source → TUILE ICÔNE (style Mac/Android, pas postit) ──
            // Grande icône centrée + nom dessous + halo « fumée » dans la
            // couleur dominante de l'app (Blender = orange, etc.). Double-clic
            // → open_in_app (déjà câblé via handleDblClick).
            if (isSource) {
              const def = getAppDef(ann.sourceFile!);
              const glow = def.bg;
              const iconSize = Math.max(40, Math.min(w, h) * 0.46);
              return (
                <div
                  key={ann.id}
                  data-id={ann.id}
                  ref={(el) => { if (el && resizeObserver.current) resizeObserver.current.observe(el); }}
                  title={`${def.name} — double-clic pour ouvrir`}
                  style={{
                    position: "absolute",
                    left: ann.x, top: ann.y,
                    width: w, height: h,
                    display: "flex", flexDirection: "column",
                    alignItems: "center", justifyContent: "center",
                    gap: 8, padding: 8, boxSizing: "border-box",
                    borderRadius: 14,
                    background: `radial-gradient(circle at 50% 36%, ${glow}40 0%, ${glow}1f 38%, #161622 78%)`,
                    border: `1px solid ${glow}66`,
                    boxShadow: sel
                      ? `0 0 0 2px #fff, 0 0 26px ${glow}88`
                      : `0 0 20px ${glow}3a, 0 6px 14px rgba(0,0,0,0.45)`,
                    // Toujours cliquable : double-clic = ouvrir le fichier, quel
                    // que soit l'outil actif. Évite que le clic traverse vers le
                    // canvas (qui éditait/créait un postit) quand l'outil ≠ select.
                    pointerEvents: "all",
                    cursor: "pointer",
                    userSelect: "none",
                  }}
                  onPointerDown={(e) => handleDown(ann, e)}
                  onDoubleClick={(e) => handleDblClick(ann, e)}
                  onWheel={forwardWheel}
                  onMouseEnter={() => setHoveredNodeId(ann.id)}
                  onMouseLeave={() => setHoveredNodeId(null)}
                >
                  <AppBridgeIcon filePath={ann.sourceFile!} size={iconSize} />
                  <span style={{
                    maxWidth: "100%",
                    fontSize: 11, fontWeight: 600,
                    color: "#e8e8f0",
                    textAlign: "center",
                    lineHeight: 1.2,
                    overflow: "hidden", textOverflow: "ellipsis",
                    display: "-webkit-box", WebkitLineClamp: 2, WebkitBoxOrient: "vertical",
                    wordBreak: "break-word",
                    textShadow: "0 1px 3px rgba(0,0,0,0.6)",
                  }}>
                    {title}
                  </span>

                  <MirrorBadge mirrorOf={ann.mirrorOf} />
                  <TemporalBadge anchor={ann.temporalAnchor} />

                  {sel && corners.map((c) => (
                    <div
                      key={c.id}
                      onPointerDown={(e) => { e.stopPropagation(); handleDown(ann, e, c.id); }}
                      style={{
                        position: "absolute",
                        left: c.cx - 12, top: c.cy - 12,
                        width: 24, height: 24,
                        cursor: c.id === "br" || c.id === "tl" ? "nwse-resize" : "nesw-resize",
                        zIndex: 10,
                        display: "flex", alignItems: "center", justifyContent: "center",
                      }}
                    >
                      <div style={{ width: 8, height: 8, background: "#fff", border: "1px solid #333", borderRadius: "50%" }} />
                    </div>
                  ))}
                </div>
              );
            }

            return (
              <div
                key={ann.id}
                data-id={ann.id}
                ref={(el) => { if (el && resizeObserver.current) resizeObserver.current.observe(el); }}
                style={{
                  position: "absolute",
                  left: ann.x, top: ann.y,
                  width: w,
                  height: h,
                  backgroundColor: bg,
                  boxShadow: sel ? "0 0 0 2px #fff" : "0 4px 6px rgba(0,0,0,0.3)",
                  pointerEvents: activeTool === "select" ? "all" : "none",
                  cursor: activeTool === "select" ? "move" : "default",
                  color: isSource ? "#333" : "#222",
                  fontSize: ann.fontSize || (isSource ? 10 : 13),
                  fontFamily: isSource ? "monospace" : undefined,
                  borderRadius: 2,
                }}
                onPointerDown={(e) => handleDown(ann, e)}
                onDoubleClick={(e) => handleDblClick(ann, e)}
                onWheel={forwardWheel}
                onMouseEnter={() => setHoveredNodeId(ann.id)}
                onMouseLeave={() => setHoveredNodeId(null)}
              >
                {/* Bande collante en haut + icône App Bridge si fichier source (Phase 5) */}
                <div style={{
                  position: "absolute", left: 0, right: 0, top: 0, height: isSource ? 26 : 16,
                  background: isSource ? "rgba(0,0,0,0.12)" : "rgba(255,255,255,0.15)",
                  borderRadius: "2px 2px 0 0",
                  display: "flex", alignItems: "center", gap: 6, paddingLeft: 6, paddingRight: 6,
                }}>
                  {isSource && (
                    <>
                      <AppBridgeIcon filePath={ann.sourceFile!} size={18} />
                      <span style={{ fontSize: 11, fontWeight: "bold", color: "#000", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
                        {title.length > 20 ? title.substring(0, 18) + "..." : title}
                      </span>
                    </>
                  )}
                </div>

                {/* Contenu Markdown ou Code */}
                <div
                  className={isSource ? "" : "prose prose-sm max-w-none break-words p-2"}
                  style={{
                    color: "inherit", fontSize: "inherit", lineHeight: isSource ? 1.2 : 1.4,
                    width: "100%",
                    height: "100%",
                    overflow: "hidden",
                    paddingTop: isSource ? 28 : 16,
                    paddingLeft: 8, paddingRight: 8, paddingBottom: 8,
                    whiteSpace: "pre-wrap",
                    wordBreak: isSource ? "break-all" : "break-word"
                  }}
                >
                  {isSource ? (
                    processedText || " "
                  ) : (
                    <StableMarkdownComponents.text processedText={processedText || " "} components={markdownComponents} />
                  )}
                </div>

                <DomainBadges badges={domainBadges} />
                <MirrorBadge mirrorOf={ann.mirrorOf} />
                <TemporalBadge anchor={ann.temporalAnchor} />

                {/* Poignées de redimensionnement */}
                {sel && corners.map((c) => (
                  <div
                    key={c.id}
                    onPointerDown={(e) => { e.stopPropagation(); handleDown(ann, e, c.id); }}
                    style={{
                      position: "absolute",
                      left: c.cx - 12, top: c.cy - 12,
                      width: 24, height: 24,
                      cursor: c.id === "br" || c.id === "tl" ? "nwse-resize" : "nesw-resize",
                      zIndex: 10,
                      display: "flex", alignItems: "center", justifyContent: "center",
                    }}
                  >
                    <div style={{
                      width: 8, height: 8,
                      background: "#fff", border: "1px solid #333",
                      // On garde le style carré pour les stickies pour différencier? 
                      // Non, l'utilisateur a dit "les petits points", restons sur des ronds pour le design premium.
                      borderRadius: "2px", 
                    }} />
                  </div>
                ))}
              </div>
            );
          }

  return null;
}
