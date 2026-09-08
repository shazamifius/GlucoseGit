import React, { useState, useEffect, useMemo, useRef } from "react";
import ReactMarkdown, { type Components } from "react-markdown";
import { useGlucoseStore, getActiveBoard } from "../store";
import {
  getSymbioticHue,
  preprocessText,
  MdPre,
  MdCode,
  REMARK_PLUGINS,
  REHYPE_PLUGINS,
} from "../canvas/HtmlAnnotationLayer";
import { isArrowAnnotation } from "../types";
import {
  type TextAnchor,
  addAnchor,
  createAnchor,
  domPointToOffset,
  highlightDomRanges,
  indexDomText,
  normalizeTextSel,
  resolveAnchors,
} from "../utils/textAnchors";

/**
 * ArrowTextEditor — Mode d'édition interactif pour sélectionner le texte
 * précis que la flèche connecte de chaque côté.
 *
 * Flux :
 * 1. Zoom sur le bloc source → sélection de texte à la souris
 * 2. Bouton "Valider" → zoom sur le bloc target → sélection
 * 3. Bouton "Terminer" → sauvegarde et sortie
 *
 * La sélection est stockée en ANCRES (`utils/textAnchors`), pas en chaîne : deux
 * occurrences d'un même mot sont deux ancres distinctes. Le bloc est rendu par
 * le MÊME pipeline markdown que le canvas — même texte rendu de part et d'autre,
 * donc mêmes offsets, donc même surlignage ici et là-bas.
 */

interface Props {
  arrowId: string;
  onClose: () => void;
}

type EditStep = "source" | "target" | "done";

export default function ArrowTextEditor({ arrowId, onClose }: Props) {
  const { project, updateAnnotation } = useGlucoseStore();
  const boardId = getActiveBoard(project).id;
  const board = project.boards.find(b => b.id === boardId);
  const arrowRaw = board?.annotations.find(a => a.id === arrowId);
  const arrow = arrowRaw && isArrowAnnotation(arrowRaw) ? arrowRaw : undefined;
  const allAnnotations = board?.annotations ?? [];

  const [step, setStep] = useState<EditStep>("source");
  const [sourceAnchors, setSourceAnchors] = useState<TextAnchor[]>(() => normalizeTextSel(arrow?.sourceTextSel));
  const [targetAnchors, setTargetAnchors] = useState<TextAnchor[]>(() => normalizeTextSel(arrow?.targetTextSel));
  const textRef = useRef<HTMLDivElement>(null);

  const srcAnn = arrow?.sourceId ? board?.annotations.find(a => a.id === arrow.sourceId) : null;
  const tgtAnn = arrow?.targetId ? board?.annotations.find(a => a.id === arrow.targetId) : null;

  const currentAnn = step === "source" ? srcAnn : tgtAnn;
  const currentAnchors = step === "source" ? sourceAnchors : targetAnchors;
  const setCurrentAnchors = step === "source" ? setSourceAnchors : setTargetAnchors;

  // Couleur symbiotique de l'annotation courante
  const currentHue = currentAnn ? getSymbioticHue(currentAnn, allAnnotations) : 200;
  const stepColor = `hsl(${currentHue}, 75%, 65%)`;

  // Même prétraitement que le canvas — sinon les offsets ne correspondraient pas.
  const processedText = preprocessText(currentAnn?.text || "");
  // Mémoïsé : sans ça, React re-réconcilierait le sous-arbre markdown à chaque
  // changement de sélection et arracherait les <mark> injectés dans le DOM.
  const markdown = useMemo(() => (
    <ReactMarkdown
      remarkPlugins={REMARK_PLUGINS}
      rehypePlugins={REHYPE_PLUGINS}
      components={EDITOR_MD_COMPONENTS}
    >
      {processedText}
    </ReactMarkdown>
  ), [processedText]);

  // Zoom sur le bloc courant au changement d'étape
  useEffect(() => {
    if (!currentAnn || step === "done") return;
    window.dispatchEvent(new CustomEvent("glucose:zoom-to-annotation", {
      detail: { annId: currentAnn.id, padding: 100 }
    }));
  }, [step, currentAnn?.id]);

  // ── Migration ascendante ──
  // Les projets d'avant la refonte n'ont que la citation (`start: -1`). Dès que
  // le bloc est rendu, on les rebase sur de vraies positions : rouvrir l'éditeur
  // suffit à convertir l'ancienne donnée, sans passe de migration au chargement.
  useEffect(() => {
    const root = textRef.current;
    if (!root) return;
    if (!currentAnchors.some(a => a.start < 0)) return;
    const plain = indexDomText(root).plain;
    const rebased = resolveAnchors(plain, currentAnchors)
      .map(r => createAnchor(plain, r.start, r.end))
      .filter((a): a is TextAnchor => a !== null);
    if (rebased.length > 0) setCurrentAnchors(rebased);
  }, [step, processedText]);

  // ── Surlignage des ancres dans l'aperçu ──
  // Exactement le même résolveur que le glow du canvas.
  useEffect(() => {
    const root = textRef.current;
    if (!root) return;
    const ranges = resolveAnchors(indexDomText(root).plain, currentAnchors);
    return highlightDomRanges(root, ranges, (mark) => {
      mark.style.cssText = `
        background: color-mix(in srgb, ${stepColor} 25%, transparent);
        color: ${stepColor};
        border-radius: 3px;
        padding: 1px 4px;
        outline: 1px solid color-mix(in srgb, ${stepColor} 40%, transparent);
        outline-offset: 1px;
        box-shadow: 0 0 8px color-mix(in srgb, ${stepColor} 15%, transparent);
      `;
    });
  }, [currentAnchors, stepColor, processedText]);

  // ── Capture d'une sélection souris ──
  useEffect(() => {
    function onMouseUp(e: MouseEvent) {
      const root = textRef.current;
      const selection = window.getSelection();
      if (!selection || selection.isCollapsed || !root) return;

      const range = selection.getRangeAt(0);
      if (!root.contains(range.commonAncestorContainer)) return;

      // Offsets dans le texte rendu : c'est ce qui distingue la 1ʳᵉ occurrence
      // d'un mot de la 2ᵉ, là où l'ancienne chaîne les confondait.
      const plain = indexDomText(root).plain;
      const start = domPointToOffset(root, range.startContainer, range.startOffset);
      const end = domPointToOffset(root, range.endContainer, range.endOffset);
      const anchor = createAnchor(plain, start, end);
      if (!anchor) return;

      // Ctrl/Meta = multi-sélection (ajouter au lieu de remplacer)
      const isMulti = e.ctrlKey || e.metaKey;
      const setter = step === "source" ? setSourceAnchors : setTargetAnchors;
      setter(prev => (isMulti ? addAnchor(prev, anchor) : [anchor]));
      selection.removeAllRanges();
    }

    document.addEventListener("mouseup", onMouseUp);
    return () => document.removeEventListener("mouseup", onMouseUp);
  }, [step]);

  function commit() {
    updateAnnotation(boardId, arrowId, {
      sourceTextSel: sourceAnchors.length > 0 ? sourceAnchors : undefined,
      targetTextSel: targetAnchors.length > 0 ? targetAnchors : undefined,
    });
  }

  function handleValidate() {
    if (step === "source") {
      setStep("target");
    } else if (step === "target") {
      commit();
      setStep("done");
      onClose();
    }
  }

  function handleSkip() {
    if (step === "source") {
      setStep("target");
    } else {
      commit();
      onClose();
    }
  }

  if (!arrow || !currentAnn) {
    return (
      <div style={overlayStyle}>
        <div style={panelStyle}>
          <p style={{ color: "#888", fontSize: 13 }}>Flèche ou bloc introuvable</p>
          <button onClick={onClose} style={btnStyle}>Fermer</button>
        </div>
      </div>
    );
  }

  const stepLabel = step === "source" ? "SOURCE" : "CIBLE";

  return (
    <div style={overlayStyle}>
      <div style={{
        ...panelStyle,
        borderColor: `color-mix(in srgb, ${stepColor} 27%, transparent)`,
        boxShadow: `0 8px 40px rgba(0,0,0,0.9), 0 0 30px color-mix(in srgb, ${stepColor} 13%, transparent)`,
      }}>
        {/* Header */}
        <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", marginBottom: 12 }}>
          <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
            <div style={{
              width: 8, height: 8, borderRadius: "50%",
              background: stepColor, boxShadow: `0 0 8px ${stepColor}`,
            }} />
            <span style={{ fontSize: 11, color: stepColor, textTransform: "uppercase", letterSpacing: 1.5, fontWeight: 600 }}>
              {stepLabel}
            </span>
            <span style={{ fontSize: 11, color: "#555" }}>
              — Sélectionnez le texte exact
            </span>
          </div>
          <div style={{ display: "flex", gap: 4 }}>
            <span style={{
              fontSize: 9, color: step === "source" ? "#fff" : "#555",
              background: step === "source" ? stepColor : "#333",
              padding: "2px 6px", borderRadius: 3,
            }}>1</span>
            <span style={{
              fontSize: 9, color: step === "target" ? "#fff" : "#555",
              background: step === "target" ? stepColor : "#333",
              padding: "2px 6px", borderRadius: 3,
            }}>2</span>
          </div>
        </div>

        {/* Instructions */}
        <div style={{ fontSize: 11, color: "#666", marginBottom: 10, lineHeight: 1.5 }}>
          Sélectionnez le texte avec la souris.
          Maintenez <kbd style={kbdStyle}>Ctrl</kbd> pour ajouter plusieurs sélections.
        </div>

        {/* Zone de texte sélectionnable */}
        <div
          ref={textRef}
          data-allow-select
          data-glucose-text
          className="prose prose-invert prose-sm max-w-none break-words"
          style={{
            background: "#0d0d0d",
            border: `1px solid color-mix(in srgb, ${stepColor} 13%, transparent)`,
            borderRadius: 6,
            padding: "14px 18px",
            maxHeight: 280,
            overflowY: "auto",
            fontSize: 13,
            lineHeight: 1.7,
            color: "#ccc",
            cursor: "text",
            position: "relative",
            whiteSpace: "pre-wrap",
          }}
        >
          {markdown}
        </div>

        {/* Sélection actuelle */}
        {currentAnchors.length > 0 && (
          <div style={{
            marginTop: 8, padding: "8px 12px",
            background: `color-mix(in srgb, ${stepColor} 7%, transparent)`, border: `1px solid color-mix(in srgb, ${stepColor} 20%, transparent)`,
            borderRadius: 4, fontSize: 11, color: stepColor,
          }}>
            <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
              <span style={{ fontWeight: 600 }}>
                Sélectionné{currentAnchors.length > 1 ? ` (${currentAnchors.length})` : ""} :
              </span>
              <button
                onClick={() => setCurrentAnchors([])}
                style={{ background: "none", border: "none", color: "#666", cursor: "pointer", fontSize: 11 }}
              >
                Effacer ✗
              </button>
            </div>
            {/* Une puce par ancre : deux occurrences du même mot restent deux
                entrées distinctes, retirables séparément. */}
            <div style={{ marginTop: 6, display: "flex", flexWrap: "wrap", gap: 6 }}>
              {currentAnchors.map((a, i) => (
                <span
                  key={`${a.start}-${a.end}-${i}`}
                  style={{
                    display: "inline-flex", alignItems: "center", gap: 6,
                    padding: "2px 6px", borderRadius: 3,
                    background: `color-mix(in srgb, ${stepColor} 12%, transparent)`,
                    border: `1px solid color-mix(in srgb, ${stepColor} 25%, transparent)`,
                    color: "#aaa", fontStyle: "italic",
                  }}
                >
                  "{a.quote.length > 32 ? `${a.quote.slice(0, 30)}…` : a.quote}"
                  <button
                    onClick={() => setCurrentAnchors(currentAnchors.filter((_, j) => j !== i))}
                    title="Retirer cette sélection"
                    style={{ background: "none", border: "none", color: "#666", cursor: "pointer", fontSize: 11, padding: 0, lineHeight: 1 }}
                  >
                    ✗
                  </button>
                </span>
              ))}
            </div>
          </div>
        )}

        {/* Boutons */}
        <div style={{ display: "flex", gap: 8, marginTop: 12, justifyContent: "flex-end" }}>
          <button onClick={handleSkip} style={{ ...btnStyle, color: "#666" }}>
            {step === "source" ? "Passer →" : "Annuler"}
          </button>
          <button onClick={handleValidate} style={{
            ...btnStyle,
            background: `color-mix(in srgb, ${stepColor} 13%, transparent)`,
            borderColor: `color-mix(in srgb, ${stepColor} 33%, transparent)`,
            color: stepColor,
          }}>
            {step === "source" ? "Valider → Cible" : "Terminer ✓"}
          </button>
        </div>
      </div>
    </div>
  );
}

/* ── Rendu markdown de l'aperçu ──
 * Uniquement du style : aucun composant n'ajoute ni ne retire de caractère, le
 * texte rendu reste donc identique à celui du canvas (condition pour que les
 * offsets des ancres soient valables des deux côtés). `pre` / `code` sont
 * carrément ceux du canvas, qui normalisent le saut de ligne final des blocs. */
const EDITOR_MD_COMPONENTS: Components = {
  h1: ({ children }) => <h1 style={{ fontSize: 18, fontWeight: 700, margin: "8px 0 4px", color: "#eee" }}>{children}</h1>,
  h2: ({ children }) => <h2 style={{ fontSize: 15, fontWeight: 600, margin: "6px 0 3px", color: "#ddd" }}>{children}</h2>,
  h3: ({ children }) => <h3 style={{ fontSize: 13, fontWeight: 600, margin: "4px 0 2px", color: "#bbb" }}>{children}</h3>,
  p: ({ children }) => <p style={{ margin: "3px 0" }}>{children}</p>,
  ul: ({ children }) => <ul style={{ margin: "3px 0", paddingLeft: 18, listStyle: "disc" }}>{children}</ul>,
  ol: ({ children }) => <ol style={{ margin: "3px 0", paddingLeft: 18 }}>{children}</ol>,
  li: ({ children }) => <li style={{ margin: "2px 0" }}>{children}</li>,
  pre: MdPre,
  code: MdCode,
};

/* ── Styles ── */
const overlayStyle: React.CSSProperties = {
  position: "fixed",
  inset: 0,
  zIndex: 500,
  display: "flex",
  alignItems: "center",
  justifyContent: "center",
  background: "rgba(0, 0, 0, 0.7)",
  backdropFilter: "blur(8px)",
};

const panelStyle: React.CSSProperties = {
  background: "#141414",
  border: "1px solid #2a2a2a",
  borderRadius: 10,
  padding: "20px 24px",
  width: 480,
  maxHeight: "80vh",
  overflowY: "auto",
};

const btnStyle: React.CSSProperties = {
  padding: "6px 14px",
  fontSize: 12,
  borderRadius: 5,
  border: "1px solid #333",
  cursor: "pointer",
  background: "#1a1a1a",
  color: "#888",
};

const kbdStyle: React.CSSProperties = {
  background: "#222",
  border: "1px solid #444",
  borderRadius: 3,
  padding: "1px 5px",
  fontSize: 10,
  color: "#999",
};
