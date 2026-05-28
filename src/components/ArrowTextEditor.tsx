import React, { useState, useEffect, useRef } from "react";
import { useGlucoseStore, getActiveBoard } from "../store";
import { getSymbioticHue } from "../canvas/HtmlAnnotationLayer";

/**
 * ArrowTextEditor — Mode d'édition interactif pour sélectionner le texte
 * précis que la flèche connecte de chaque côté.
 * 
 * Flux :
 * 1. Zoom sur le bloc source → sélection de texte à la souris
 * 2. Bouton "Valider" → zoom sur le bloc target → sélection
 * 3. Bouton "Terminer" → sauvegarde et sortie
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
  const arrow = board?.annotations.find(a => a.id === arrowId);
  const allAnnotations = board?.annotations ?? [];

  const [step, setStep] = useState<EditStep>("source");
  const [sourceSelection, setSourceSelection] = useState(arrow?.sourceTextSel ?? "");
  const [targetSelection, setTargetSelection] = useState(arrow?.targetTextSel ?? "");
  const selectionRef = useRef<HTMLDivElement>(null);

  const srcAnn = arrow?.sourceId ? board?.annotations.find(a => a.id === arrow.sourceId) : null;
  const tgtAnn = arrow?.targetId ? board?.annotations.find(a => a.id === arrow.targetId) : null;

  const currentAnn = step === "source" ? srcAnn : tgtAnn;
  const currentSel = step === "source" ? sourceSelection : targetSelection;
  const setCurrentSel = step === "source" ? setSourceSelection : setTargetSelection;

  // Couleur symbiotique de l'annotation courante
  const currentHue = currentAnn ? getSymbioticHue(currentAnn, allAnnotations) : 200;
  const stepColor = `hsl(${currentHue}, 75%, 65%)`;

  // Zoom sur le bloc courant au changement d'étape
  useEffect(() => {
    if (!currentAnn || step === "done") return;
    window.dispatchEvent(new CustomEvent("glucose:zoom-to-annotation", {
      detail: { annId: currentAnn.id, padding: 100 }
    }));
  }, [step, currentAnn?.id]);

  // Écouter les sélections de texte via un listener document-level
  useEffect(() => {
    function onMouseUp(e: MouseEvent) {
      const selection = window.getSelection();
      if (!selection || selection.isCollapsed || !selectionRef.current) return;

      const range = selection.getRangeAt(0);
      if (!selectionRef.current.contains(range.commonAncestorContainer)) return;

      const selectedText = selection.toString().trim();
      if (!selectedText) return;

      // Ctrl/Meta = multi-sélection (ajouter au lieu de remplacer)
      const isMulti = e.ctrlKey || e.metaKey;

      if (isMulti) {
        // Lire la sélection courante directement (éviter les closures stale)
        const curSel = step === "source" ? sourceSelection : targetSelection;
        const setter = step === "source" ? setSourceSelection : setTargetSelection;
        if (curSel) {
          const parts = curSel.split(" ‖ ");
          if (!parts.includes(selectedText)) {
            setter(parts.concat(selectedText).join(" ‖ "));
          }
        } else {
          setter(selectedText);
        }
      } else {
        const setter = step === "source" ? setSourceSelection : setTargetSelection;
        setter(selectedText);
      }
    }

    document.addEventListener("mouseup", onMouseUp);
    return () => document.removeEventListener("mouseup", onMouseUp);
  }, [step, sourceSelection, targetSelection]);

  function handleValidate() {
    if (step === "source") {
      setStep("target");
    } else if (step === "target") {
      updateAnnotation(boardId, arrowId, {
        sourceTextSel: sourceSelection || undefined,
        targetTextSel: targetSelection || undefined,
      });
      setStep("done");
      onClose();
    }
  }

  function handleSkip() {
    if (step === "source") {
      setStep("target");
    } else {
      updateAnnotation(boardId, arrowId, {
        sourceTextSel: sourceSelection || undefined,
        targetTextSel: targetSelection || undefined,
      });
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
          ref={selectionRef}
          data-allow-select
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
          }}
        >
          <TextWithHighlights text={currentAnn.text || ""} highlights={currentSel} color={stepColor} />
        </div>

        {/* Sélection actuelle */}
        {currentSel && (
          <div style={{
            marginTop: 8, padding: "8px 12px",
            background: `color-mix(in srgb, ${stepColor} 7%, transparent)`, border: `1px solid color-mix(in srgb, ${stepColor} 20%, transparent)`,
            borderRadius: 4, fontSize: 11, color: stepColor,
          }}>
            <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
              <span style={{ fontWeight: 600 }}>Sélectionné :</span>
              <button
                onClick={() => setCurrentSel("")}
                style={{ background: "none", border: "none", color: "#666", cursor: "pointer", fontSize: 11 }}
              >
                Effacer ✗
              </button>
            </div>
            <div style={{ marginTop: 4, color: "#aaa", fontStyle: "italic" }}>
              "{currentSel}"
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

/* ── Texte avec rendu Markdown et surlignage des sélections ── */
function TextWithHighlights({ text, highlights, color }: { text: string, highlights: string, color: string }) {
  const lines = text.split("\n");

  function renderLine(line: string, idx: number) {
    const h3 = line.match(/^### (.+)/);
    const h2 = !h3 && line.match(/^## (.+)/);
    const h1 = !h3 && !h2 && line.match(/^# (.+)/);
    const li = line.match(/^[-*] (.+)/);
    
    const content = h3 ? h3[1] : h2 ? h2[1] : h1 ? h1[1] : li ? li[1] : line;
    
    const rendered = highlights ? highlightText(content, highlights, color) : content;
    
    if (h1) return <h1 key={idx} style={{ fontSize: 18, fontWeight: 700, margin: "8px 0 4px", color: "#eee" }}>{rendered}</h1>;
    if (h2) return <h2 key={idx} style={{ fontSize: 15, fontWeight: 600, margin: "6px 0 3px", color: "#ddd" }}>{rendered}</h2>;
    if (h3) return <h3 key={idx} style={{ fontSize: 13, fontWeight: 600, margin: "4px 0 2px", color: "#bbb" }}>{rendered}</h3>;
    if (li) return <div key={idx} style={{ display: "flex", gap: 6, margin: "2px 0" }}><span style={{ color: "#555" }}>•</span><span>{rendered}</span></div>;
    if (!line.trim()) return <div key={idx} style={{ height: 8 }} />;
    return <p key={idx} style={{ margin: "3px 0" }}>{rendered}</p>;
  }

  return <>{lines.map((line, i) => renderLine(line, i))}</>;
}

function highlightText(text: string, highlights: string, color: string): React.ReactNode {
  if (!highlights || !text) return text;
  
  const parts = highlights.split(" ‖ ");
  let result: (string | React.ReactNode)[] = [text];

  for (const part of parts) {
    const newResult: (string | React.ReactNode)[] = [];
    for (const segment of result) {
      if (typeof segment !== "string") {
        newResult.push(segment);
        continue;
      }
      const idx = segment.toLowerCase().indexOf(part.toLowerCase());
      if (idx === -1) {
        newResult.push(segment);
        continue;
      }
      if (idx > 0) newResult.push(segment.slice(0, idx));
      newResult.push(
        <mark key={`hl-${part.slice(0,10)}-${idx}`} style={{
          background: `color-mix(in srgb, ${color} 25%, transparent)`,
          color: color,
          borderRadius: 3,
          padding: "1px 4px",
          outline: `1px solid color-mix(in srgb, ${color} 40%, transparent)`,
          outlineOffset: 1,
          boxShadow: `0 0 8px color-mix(in srgb, ${color} 15%, transparent)`,
        }}>
          {segment.slice(idx, idx + part.length)}
        </mark>
      );
      if (idx + part.length < segment.length) newResult.push(segment.slice(idx + part.length));
    }
    result = newResult;
  }

  return <>{result}</>;
}

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
