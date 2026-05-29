import React, { useRef, useEffect } from "react";
import katex from "katex";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import remarkMath from "remark-math";
import rehypeKatex from "rehype-katex";

// Constantes module-level — identité stable des deps ReactMarkdown internes
// (cf. HtmlAnnotationLayer pour le rationale).
const REMARK_PLUGINS = [remarkGfm, remarkMath];
const REHYPE_PLUGINS = [rehypeKatex];
// CLEANUP SEC-09 : rehypeRaw retiré (anti-XSS).
// CLEANUP B-03 : CSS KaTeX chargé à la demande quand l'éditeur s'ouvre.
import { ensureKatexCss } from "../utils/loadKatexCss";

interface SyntaxEditorProps {
  value: string;
  onChange: (val: string) => void;
  onKeyDown: (e: React.KeyboardEvent<HTMLTextAreaElement>) => void;
  style?: React.CSSProperties;
  scale: number;
  type: "text" | "sticky";
  onHeightChange?: (height: number) => void;
  initialCursorPos?: number;
  onCursorChange?: (pos: number) => void;
}

export default function SyntaxEditor({ value, onChange, onKeyDown, style, scale, type, onHeightChange, initialCursorPos, onCursorChange }: SyntaxEditorProps) {
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  // L'éditeur peut afficher du LaTeX rendu : on précharge le CSS dès l'ouverture.
  useEffect(() => { void ensureKatexCss(); }, []);
  
  // Fonction de surlignage syntaxique
  const renderHighlighted = () => {
    // Séparation des blocs LaTeX
    const parts = value.split(/(\$\$[\s\S]*?\$\$|\$[\s\S]*?\$)/g);
    
    return parts.map((part, i) => {
      if (part.startsWith("$$") || part.startsWith("$")) {
        const isBlock = part.startsWith("$$");
        const content = part.substring(isBlock ? 2 : 1, part.length - (isBlock ? 2 : 1));
        let valid = true;
        try {
          katex.renderToString(content, { throwOnError: true, displayMode: isBlock });
        } catch {
          valid = false;
        }
        const color = valid ? "#4ade80" : "#f87171"; // Vert si stable, Rouge si erreur
        return <span key={i} style={{ color, backgroundColor: valid ? "rgba(74, 222, 128, 0.1)" : "rgba(248, 113, 113, 0.1)", borderRadius: 3 }}>{part}</span>;
      }
      
      // Surlignage Markdown
      // On split par symboles markdown (titres, listes, bold, subtext)
      const mdParts = part.split(/(^[ \t]*#{1,6}[ \t]+|^-# |\*\*|\*|__|_|~~|`)/gm);
      return mdParts.map((mp, j) => {
        if (!mp) return null;
        if (mp.match(/^[ \t]*#{1,6}[ \t]+$/) || mp === "-# " || mp.match(/^[*_~`]+$/)) {
          return <span key={j} style={{ color: "#888" }}>{mp}</span>;
        }
        return <span key={j} style={{ color: type === "sticky" ? "#222" : "#fff" }}>{mp}</span>;
      });
    });
  };

  function preprocessText(text: string) {
    let t = text.replace(/^-# (.*)$/gm, '<span class="text-[0.65em] opacity-75">$1</span>');
    // Préserver les lignes vides multiples
    t = t.replace(/\n(?=\n)/g, '\n&nbsp;');
    return t;
  }

  // Auto-grow
  useEffect(() => {
    if (textareaRef.current) {
      textareaRef.current.style.height = "auto";
      const need = textareaRef.current.scrollHeight;
      textareaRef.current.style.height = `${need}px`;
      if (onHeightChange) onHeightChange(need);
    }
  }, [value, onHeightChange]);

  useEffect(() => {
    if (textareaRef.current) {
      const el = textareaRef.current;
      const applyFocus = () => {
        el.focus();
        if (initialCursorPos !== undefined) {
          el.setSelectionRange(initialCursorPos, initialCursorPos);
        } else {
          const len = el.value.length;
          el.setSelectionRange(len, len);
        }
      };
      
      applyFocus();
      setTimeout(applyFocus, 10);
      setTimeout(applyFocus, 50);
    }
  }, []); // Run only on mount

  return (
    <div style={{ position: "relative", ...style }}>
      {/* Couche de rendu syntaxique visuelle */}
      <div 
        aria-hidden="true"
        style={{
          position: "absolute",
          inset: 0,
          padding: type === "sticky" ? 0 : `${4 * scale}px`,
          whiteSpace: "pre-wrap",
          wordBreak: "break-word",
          fontFamily: "system-ui, sans-serif",
          fontSize: "inherit",
          lineHeight: "1.4",
          pointerEvents: "none",
          color: "transparent", // Texte de base transparent (géré par les spans)
        }}
      >
        {renderHighlighted()}
        {/* Hack pour que la div fasse la même taille que le textarea s'il finit par un \n */}
        {value.endsWith('\n') ? <br /> : null}
      </div>

      {/* Textarea invisible qui gère la saisie */}
      <textarea
        ref={textareaRef}
        autoFocus
        value={value}
        onChange={(e) => onChange(e.target.value)}
        onKeyDown={onKeyDown}
        onPointerDown={(e) => e.stopPropagation()}
        onSelect={(e) => {
          if (onCursorChange) {
            onCursorChange(e.currentTarget.selectionStart);
          }
        }}
        style={{
          width: "100%",
          height: "100%",
          padding: type === "sticky" ? 0 : `${4 * scale}px`,
          boxSizing: "border-box",
          fontFamily: "system-ui, sans-serif",
          fontSize: "inherit",
          lineHeight: "1.4",
          whiteSpace: "pre-wrap",
          wordBreak: "break-word",
          resize: "none",
          background: "transparent",
          border: "none",
          outline: "none",
          color: "transparent", // Le texte natif est transparent !
          caretColor: type === "sticky" ? "#222" : "#60a5fa", // Le curseur reste visible
          overflow: "hidden",
        }}
        placeholder={type === "sticky" ? "Écrire..." : ""}
      />

      {/* Live Preview Flottante */}
      <div
        style={{
          position: "absolute",
          left: "calc(100% + 20px)", // Décalé à droite pour ne pas superposer
          top: 0,
          width: "max-content",
          maxWidth: 500,
          background: "rgba(15, 15, 20, 0.95)",
          border: "1px solid rgba(255, 255, 255, 0.1)",
          borderRadius: 8,
          padding: "16px 20px",
          boxShadow: "0 8px 32px rgba(0, 0, 0, 0.5)",
          backdropFilter: "blur(8px)",
          color: "#fff",
          zIndex: 10005,
          pointerEvents: "none", // Laisse passer les clics
        }}
      >
        <div style={{ fontSize: 10, textTransform: "uppercase", letterSpacing: 1, color: "#888", marginBottom: 8 }}>
          Prévisualisation en direct
        </div>
        <div className="prose prose-invert prose-sm max-w-none break-words" style={{ lineHeight: 1.4, whiteSpace: "pre-wrap" }}>
          <ReactMarkdown 
            remarkPlugins={REMARK_PLUGINS}
            rehypePlugins={REHYPE_PLUGINS}
          >
            {preprocessText(value) || " "}
          </ReactMarkdown>
        </div>
      </div>
    </div>
  );
}
