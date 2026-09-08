import { useEffect, useRef, useState } from "react";
import {
  listPlugins, installPluginFromDir, pickTextFile, runPluginAndImport,
  systemSpecs, ollamaStatus, pullModel, installOllama,
  onPluginProgress, onModelProgress, onOllamaInstallProgress,
  type PluginManifest, type SystemSpecs, type OllamaStatus,
} from "../utils/plugins";
import { BUILTIN_ENGINE, BUILTIN_ENGINE_ID, chooseModel, modelSizeB, runBuiltinEngine } from "../utils/builtinEngine";

// Keyframes (injectées une fois) pour la barre de progression indéterminée.
if (typeof document !== "undefined" && !document.getElementById("glucose-kf")) {
  const s = document.createElement("style");
  s.id = "glucose-kf";
  s.textContent = "@keyframes glucoseSlide{0%{left:-35%}100%{left:100%}}";
  document.head.appendChild(s);
}

interface Props {
  /** Fermeture programmatique (jamais un bouton : la poignée du dock ferme). */
  onClose?: () => void;
  /** Rendu en tiroir dans le dock : le dock fournit le positionnement. */
  docked?: boolean;
}

/** Nom de fichier seul (sans le chemin). */
function basename(p: string): string {
  return p.split(/[\\/]/).pop() || p;
}

/** Étapes du moteur → fraction de progression + libellé lisible. */
const PASSES: { re: RegExp; pct: number; label: string }[] = [
  { re: /passe 0/i, pct: 6, label: "Nettoyage du texte" },
  { re: /passe 1.*triage/i, pct: 18, label: "Triage par valeur (le plus long)" },
  { re: /passe 3 .*extra/i, pct: 42, label: "Extraction des unités" },
  { re: /polish|propre|passe 3\.3/i, pct: 52, label: "Mise au propre" },
  { re: /passe 3\.4|langue/i, pct: 62, label: "Unification de la langue" },
  { re: /passe 3\.5|emphase/i, pct: 70, label: "Mise en valeur" },
  { re: /passe 4 .*archi/i, pct: 80, label: "Architecture (thèmes + liens)" },
  { re: /passe 4 :|bulle/i, pct: 90, label: "Géométrie de la carte" },
  { re: /axe x|r[ée]sultat/i, pct: 97, label: "Finalisation" },
];
function matchPass(line: string): { pct: number; label: string } | null {
  let best: { pct: number; label: string } | null = null;
  for (const p of PASSES) if (p.re.test(line)) best = { pct: p.pct, label: p.label };
  return best;
}

export default function PluginPanel({ docked }: Props) {
  const [plugins, setPlugins] = useState<PluginManifest[] | null>(null);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [textPath, setTextPath] = useState<string | null>(null);
  const [running, setRunning] = useState(false);
  const [pct, setPct] = useState(0);
  const [label, setLabel] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [doneMsg, setDoneMsg] = useState<string | null>(null);
  const [optionValues, setOptionValues] = useState<Record<string, string>>({});
  // Un traitement peut durer des dizaines de minutes : on doit pouvoir l'arrêter
  // sans perdre les sections déjà produites (elles sont importées quand même).
  const stopRef = useRef(false);

  // Environnement (Ollama / matériel)
  const [specs, setSpecs] = useState<SystemSpecs | null>(null);
  const [ollama, setOllama] = useState<OllamaStatus | null>(null);
  const [pulling, setPulling] = useState(false);
  const [pullPct, setPullPct] = useState(0);
  const [pullLine, setPullLine] = useState("");
  const [pullErr, setPullErr] = useState<string | null>(null);
  const [installing, setInstalling] = useState(false);
  const [installLine, setInstallLine] = useState("");
  const [installErr, setInstallErr] = useState<string | null>(null);

  // Le moteur INTÉGRÉ est toujours en tête de liste : il ne dépend d'aucun
  // binaire à installer, donc « Lancer » est utilisable dès la première ouverture.
  // Les plugins externes viennent s'ajouter à côté, jamais à sa place.
  async function refreshPlugins() {
    let installed: PluginManifest[] = [];
    try {
      installed = await listPlugins();
    } catch (e) {
      setError(String((e as Error)?.message ?? e)); // dossier illisible : on garde le moteur intégré
    }
    const list = [BUILTIN_ENGINE, ...installed.filter((p) => p.id !== BUILTIN_ENGINE_ID)];
    setPlugins(list);
    setSelectedId((cur) => (cur && list.some((p) => p.id === cur) ? cur : list[0].id));
  }

  useEffect(() => {
    refreshPlugins().catch((e) => setError(String(e)));
    systemSpecs().then(setSpecs).catch(() => {});
  }, []);

  // L'environnement change SOUS l'application : l'utilisateur installe Ollama à
  // côté, le démon met dix secondes à répondre, un modèle finit de se télécharger
  // ailleurs… Une sonde unique au montage obligeait à relancer Glucose pour voir
  // « Ollama actif ». On resonde donc tant que le panneau est ouvert.
  useEffect(() => {
    let alive = true;
    const probe = () => {
      // Pendant une install / un pull, ces flux pilotent déjà l'affichage : une
      // sonde concurrente ferait clignoter la section.
      if (pulling || installing) return;
      ollamaStatus().then((s) => { if (alive) setOllama(s); }).catch(() => {});
    };
    probe();
    const id = setInterval(probe, 4000);
    // Retour d'un installeur externe → on veut la mise à jour immédiate.
    window.addEventListener("focus", probe);
    return () => {
      alive = false;
      clearInterval(id);
      window.removeEventListener("focus", probe);
    };
  }, [pulling, installing]);

  // Quand le plugin sélectionné change : initialise les options à leurs défauts.
  useEffect(() => {
    const sel = plugins?.find((p) => p.id === selectedId);
    if (!sel?.options?.length) {
      setOptionValues({});
      return;
    }
    const init: Record<string, string> = {};
    for (const o of sel.options) init[o.id] = o.default ?? o.choices?.[0]?.value ?? "";
    setOptionValues(init);
  }, [selectedId, plugins]);

  async function chooseText() {
    setError(null);
    try {
      const p = await pickTextFile();
      if (p) setTextPath(p);
    } catch (e) {
      setError(String(e));
    }
  }

  async function installPlugin() {
    setError(null);
    try {
      const m = await installPluginFromDir();
      if (m) {
        await refreshPlugins();
        setSelectedId(m.id);
      }
    } catch (e) {
      setError(String((e as Error)?.message ?? e));
    }
  }

  async function launch() {
    if (!selectedId || !textPath || running) return;
    stopRef.current = false;
    setRunning(true);
    setError(null);
    setDoneMsg(null);
    setPct(2);
    setLabel("Démarrage du moteur…");
    let un: (() => void) | null = null;
    try {
      if (selectedId === BUILTIN_ENGINE_ID) {
        // Moteur intégré : il parle directement à l'IA locale et rapporte sa
        // progression par callback (pas d'event Tauri, pas de process externe).
        const model = chooseModel(recommended, ollama?.models ?? []);
        if (!model) {
          throw new Error(
            "Aucun modèle d'IA locale n'est installé. Télécharge le modèle conseillé dans la section IA locale ci-dessus.",
          );
        }
        await runBuiltinEngine({
          textPath,
          model,
          options: optionValues,
          onProgress: (p, l) => {
            setPct((prev) => Math.max(prev, p));
            setLabel(l);
          },
          shouldStop: () => stopRef.current,
        });
      } else {
        un = await onPluginProgress((line) => {
          const m = matchPass(line);
          if (m) {
            setPct((p) => Math.max(p, m.pct));
            setLabel(m.label);
          }
        });
        await runPluginAndImport(selectedId, textPath, optionValues);
      }
      setPct(100);
      setLabel("Terminé");
      setDoneMsg("Cours ajouté comme nouveau board ✓");
    } catch (e) {
      setError(formatErr(e));
    } finally {
      setRunning(false);
      if (un) un();
    }
  }

  async function doInstallOllama() {
    if (installing) return;
    setInstalling(true);
    setInstallErr(null);
    setInstallLine("Préparation…");
    let un: (() => void) | null = null;
    try {
      un = await onOllamaInstallProgress((line) => setInstallLine(line));
      const msg = await installOllama();
      setInstallLine(msg);
      setOllama(await ollamaStatus());
    } catch (e) {
      setInstallErr(String((e as Error)?.message ?? e));
      // winget absent → la page a été ouverte ; l'user a pu installer entre-temps.
      try { setOllama(await ollamaStatus()); } catch { /* ignore */ }
    } finally {
      setInstalling(false);
      if (un) un();
    }
  }

  async function downloadModel() {
    if (!specs || pulling) return;
    setPulling(true);
    setPullErr(null);
    setPullPct(0);
    setPullLine("Démarrage du téléchargement…");
    let un: (() => void) | null = null;
    try {
      un = await onModelProgress((line) => {
        setPullLine(line);
        const m = line.match(/(\d+)\s*%/);
        if (m) setPullPct(Math.min(100, Math.max(0, parseInt(m[1], 10))));
      });
      await pullModel(specs.recommended_model);
      setPullPct(100);
      setPullLine("Modèle prêt ✓");
      setOllama(await ollamaStatus());
    } catch (e) {
      setPullErr(String((e as Error)?.message ?? e));
    } finally {
      setPulling(false);
      if (un) un();
    }
  }

  const canLaunch = !!selectedId && !!textPath && !running;
  const selected = plugins?.find((p) => p.id === selectedId) ?? null;
  const recommended = specs?.recommended_model ?? null;
  const modelInstalled = !!(recommended && ollama?.models.includes(recommended));

  // Un bouton grisé sans explication est un cul-de-sac : on nomme toujours la
  // pièce qui manque. Depuis le moteur intégré, il n'en reste qu'une seule —
  // le texte source.
  const launchBlocker = running || plugins === null
    ? null
    : !selectedId
      ? "Choisis un moteur dans la liste ci-dessus."
      : !textPath
        ? "Choisis un texte source ci-dessus : c'est la seule chose qui manque."
        : null;
  // L'IA locale peut manquer sans que ce soit bloquant (un plugin externe peut
  // s'en passer) : on avertit, on ne grise pas.
  const modelToUse = chooseModel(recommended, ollama?.models ?? []);
  // Le modèle installé est-il nettement plus gros que celui qui tient dans la
  // VRAM ? C'est le cas vécu (32b conseillé à tort sur un GPU de 6 Go) : ça
  // marche, mais à ~1 mot/s. Mieux vaut le dire que laisser croire à un blocage.
  const usedSize = modelToUse ? modelSizeB(modelToUse) : null;
  const fitSize = recommended ? modelSizeB(recommended) : null;
  const slowModel = !!(usedSize && fitSize && usedSize > fitSize * 1.5);
  const launchWarning = launchBlocker
    ? null
    : ollama?.reachable === false
      ? "Ollama ne répond pas : démarre l'IA locale ci-dessus, sinon le moteur échouera."
      : ollama?.reachable && !modelToUse
        ? "Aucun modèle installé : télécharge le modèle conseillé ci-dessus."
        : slowModel
          ? `« ${modelToUse} » dépasse ce que ta carte peut tenir : le traitement tournera sur le processeur, très lentement (plusieurs minutes par section). Télécharger ${recommended} ci-dessus le rendra bien plus rapide.`
          : recommended && ollama?.reachable && !modelInstalled && modelToUse
            ? `Modèle conseillé (${recommended}) absent — le moteur utilisera « ${modelToUse} ».`
            : null;

  return (
    <div style={docked ? { ...panel, ...panelDocked } : panel}>
      {/* Header */}
      <div style={header}>
        <span style={{ color: "#eee", fontWeight: 600, fontSize: 13, letterSpacing: 1 }}>Plugins</span>
      </div>

      <div style={{ padding: 16, overflowY: "auto", display: "flex", flexDirection: "column", gap: 18 }}>

        {/* ── IA locale (Ollama) ── */}
        <Section title="IA locale">
          <div style={{ display: "flex", alignItems: "center", gap: 8, marginBottom: 8 }}>
            <Dot ok={ollama?.reachable === true} />
            <span style={{ color: ollama?.reachable ? "#cfd6d2" : "#c9a", fontSize: 12 }}>
              {ollama === null ? "Vérification…" : ollama.reachable ? "Ollama actif" : "Ollama introuvable"}
            </span>
          </div>

          {specs && (
            <div style={{ color: "#666", fontSize: 11, marginBottom: 8 }}>
              Ce PC : {specs.ram_gb} Go RAM · {specs.cores} cœurs
              {specs.vram_gb ? ` · GPU ${specs.vram_gb} Go` : ""}
            </div>
          )}

          {/* Ollama injoignable → installation automatique (winget) */}
          {ollama && !ollama.reachable && (
            <>
              <button onClick={doInstallOllama} disabled={installing} style={{ ...softBtn, opacity: installing ? 0.6 : 1 }}>
                {installing ? "Installation d'Ollama…" : "Installer Ollama automatiquement"}
              </button>
              {installing && <ProgressBar indeterminate line={installLine} />}
              {installErr && <ErrBox>{installErr}</ErrBox>}
              <div style={{ color: "#666", fontSize: 11, marginTop: 6 }}>
                Sinon, installe-le depuis <span style={{ color: "#89a" }}>ollama.com</span> et rouvre ce panneau.
              </div>
            </>
          )}

          {/* Ollama OK → modèle recommandé / téléchargement */}
          {ollama?.reachable && recommended && (
            modelInstalled ? (
              <div style={{ color: "#7bb89f", fontSize: 12 }}>✓ Modèle <b style={{ color: "#cfe" }}>{recommended}</b> installé</div>
            ) : (
              <>
                <div style={{ color: "#8a9", fontSize: 12, marginBottom: 8 }}>
                  Modèle conseillé pour ce PC : <b style={{ color: "#cfe" }}>{recommended}</b>
                </div>
                <button onClick={downloadModel} disabled={pulling} style={{ ...softBtn, opacity: pulling ? 0.6 : 1 }}>
                  {pulling ? "Téléchargement…" : `Télécharger ${recommended}`}
                </button>
                {pulling && <ProgressBar pct={pullPct} line={pullLine} />}
                {pullErr && <ErrBox>{pullErr}</ErrBox>}
              </>
            )
          )}
        </Section>

        <Divider />

        {/* ── Moteur ── (le moteur intégré est toujours présent : plus de liste vide) */}
        <Section title="Moteur">
          {plugins === null ? (
            <Muted>Chargement…</Muted>
          ) : (
            <div style={{ display: "flex", flexDirection: "column", gap: 6, marginBottom: 8 }}>
              {plugins.map((p) => {
                const sel = selectedId === p.id;
                return (
                  <button
                    key={p.id}
                    onClick={() => setSelectedId(p.id)}
                    style={{
                      ...pluginRow,
                      borderColor: sel ? "#3a3a3a" : "#1f1f1f",
                      background: sel ? "#181818" : "#141414",
                      boxShadow: sel ? "inset 2px 0 0 #34d399" : "none",
                    }}
                  >
                    <div style={{ color: "#e6e6e6", fontSize: 13, fontWeight: 600 }}>{p.name}</div>
                    {p.description && <div style={{ color: "#7c7c7c", fontSize: 11, marginTop: 2 }}>{p.description}</div>}
                    {p.version && <div style={{ color: "#4d4d4d", fontSize: 10, marginTop: 2 }}>v{p.version}</div>}
                  </button>
                );
              })}
            </div>
          )}
        </Section>

        {/* ── Réglages (recette) — générés AUTOMATIQUEMENT depuis le manifeste ── */}
        {selected?.options?.length ? (
          <>
            <Divider />
            <Section title="Réglages">
              {selected.options.map((opt) => (
                <div key={opt.id} style={{ marginBottom: 12 }}>
                  <div style={{ color: "#bdbdbd", fontSize: 12, marginBottom: 6 }}>{opt.label}</div>
                  {(opt.type ?? "enum") === "enum" && opt.choices && (
                    <div style={{ display: "flex", flexDirection: "column", gap: 4 }}>
                      {opt.choices.map((c) => (
                        <label key={c.value} style={{ display: "flex", alignItems: "center", gap: 8, cursor: "pointer", color: "#9a9a9a", fontSize: 12 }}>
                          <input
                            type="radio"
                            name={opt.id}
                            checked={optionValues[opt.id] === c.value}
                            onChange={() => setOptionValues((v) => ({ ...v, [opt.id]: c.value }))}
                            style={{ accentColor: "#34d399" }}
                          />
                          {c.label}
                        </label>
                      ))}
                    </div>
                  )}
                  {opt.description && (
                    <div style={{ color: "#5e5e5e", fontSize: 10, marginTop: 4 }}>{opt.description}</div>
                  )}
                </div>
              ))}
            </Section>
          </>
        ) : null}

        <Divider />

        {/* ── Texte source ── */}
        <Section title="Texte source">
          <button onClick={chooseText} style={softBtn} disabled={running}>
            {textPath ? "Changer de texte…" : "Choisir un texte…"}
          </button>
          {textPath && (
            <div style={{ color: "#9aa", fontSize: 11, marginTop: 6, wordBreak: "break-all" }}>{basename(textPath)}</div>
          )}
        </Section>

        {/* ── Lancer ── */}
        <button
          onClick={launch}
          disabled={!canLaunch}
          style={{ ...primaryBtn, opacity: canLaunch ? 1 : 0.4, cursor: canLaunch ? "pointer" : "not-allowed" }}
        >
          {running ? "Le moteur travaille…" : "Lancer"}
        </button>
        {launchBlocker && <Muted>{launchBlocker}</Muted>}
        {launchWarning && <div style={{ color: "#b9975b", fontSize: 11, lineHeight: 1.6 }}>{launchWarning}</div>}

        {running && (
          <>
            <ProgressBar pct={pct} line={label} />
            <Muted>L'IA locale traite ton texte — laisse la fenêtre ouverte.</Muted>
            {selectedId === BUILTIN_ENGINE_ID && (
              <button
                onClick={() => {
                  stopRef.current = true;
                  setLabel("Arrêt demandé — la partie en cours se termine…");
                }}
                style={ghostBtn}
              >
                Arrêter et garder ce qui est déjà fait
              </button>
            )}
          </>
        )}
        {doneMsg && <div style={{ color: "#34d399", fontSize: 12 }}>{doneMsg}</div>}
        {error && <ErrBox>{error}</ErrBox>}

        <Divider />

        {/* ── Avancé ── : ajouter un moteur tiers. Volontairement en dernier —
            le parcours normal (moteur intégré → texte → lancer) n'en a pas besoin. */}
        <Section title="Avancé">
          <button onClick={installPlugin} style={ghostBtn} disabled={running}>
            Installer un plugin externe…
          </button>
          <div style={{ color: "#5e5e5e", fontSize: 10, marginTop: 6, lineHeight: 1.6 }}>
            Un dossier contenant <code>manifest.json</code> et son binaire.
          </div>
        </Section>
      </div>
    </div>
  );
}

/** Ollama est la panne la plus probable → message orienté. */
function formatErr(e: unknown): string {
  const s = String((e as Error)?.message ?? e);
  if (/injoignable|connection|refused|11434|ollama/i.test(s)) {
    return `${s}\n\nVérifie qu'Ollama tourne (le moteur a besoin d'une IA locale).`;
  }
  return s;
}

// ── Sous-composants / styles (thème sombre Glucose, accent vert = prêt/progrès) ──
function Section({ title, children }: { title: string; children: React.ReactNode }) {
  return (
    <div>
      <div style={{ color: "#5e5e5e", fontSize: 10, textTransform: "uppercase", letterSpacing: 1.2, marginBottom: 10 }}>{title}</div>
      {children}
    </div>
  );
}
function Muted({ children }: { children: React.ReactNode }) {
  return <div style={{ color: "#6f6f6f", fontSize: 11, lineHeight: 1.6 }}>{children}</div>;
}
function Divider() {
  return <div style={{ height: 1, background: "#1c1c1c" }} />;
}
function Dot({ ok }: { ok: boolean }) {
  return (
    <span style={{
      width: 7, height: 7, borderRadius: "50%",
      background: ok ? "#34d399" : "#5a5a5a",
      boxShadow: ok ? "0 0 6px #34d39988" : "none", flexShrink: 0,
    }} />
  );
}
function ProgressBar({ pct = 0, line, indeterminate }: { pct?: number; line: string; indeterminate?: boolean }) {
  return (
    <div style={{ marginTop: 8 }}>
      <div style={{ position: "relative", height: 4, background: "#171717", borderRadius: 2, overflow: "hidden" }}>
        {indeterminate ? (
          <div style={{
            position: "absolute", top: 0, bottom: 0, width: "35%",
            background: "#34d399", borderRadius: 2, animation: "glucoseSlide 1.2s ease-in-out infinite",
          }} />
        ) : (
          <div style={{
            height: "100%", width: `${Math.max(2, Math.min(100, pct))}%`,
            background: "#34d399", borderRadius: 2, transition: "width .4s ease",
          }} />
        )}
      </div>
      {line && (
        <div style={{ color: "#7d7d7d", fontSize: 11, marginTop: 6, whiteSpace: "nowrap", overflow: "hidden", textOverflow: "ellipsis" }}>
          {line}
        </div>
      )}
    </div>
  );
}
function ErrBox({ children }: { children: React.ReactNode }) {
  return (
    <div style={{
      color: "#e89", fontSize: 11, lineHeight: 1.5, marginTop: 8,
      background: "#161010", border: "1px solid #2e1b1b", borderRadius: 6, padding: "8px 10px",
      whiteSpace: "pre-wrap",
    }}>
      {children}
    </div>
  );
}

const panel: React.CSSProperties = {
  position: "absolute", top: 0, right: 0, bottom: 0,
  width: 340, background: "#111", borderLeft: "1px solid #222",
  display: "flex", flexDirection: "column", zIndex: 100,
};
/** En tiroir : le dock place le panneau, on annule donc l'ancrage plein écran.
 *  Les 14 px du bas laissent respirer la poignée. */
const panelDocked: React.CSSProperties = {
  position: "static", top: "auto", right: "auto", bottom: "auto", zIndex: "auto",
  maxHeight: "70vh", border: "1px solid #222", borderLeft: "1px solid #222",
  borderRadius: 6, paddingBottom: 14, overflow: "hidden",
};
const header: React.CSSProperties = {
  display: "flex", alignItems: "center",
  padding: "12px 16px", borderBottom: "1px solid #1e1e1e",
};
const pluginRow: React.CSSProperties = {
  textAlign: "left", border: "1px solid #1f1f1f", borderRadius: 6,
  padding: "8px 10px", cursor: "pointer",
};
const softBtn: React.CSSProperties = {
  width: "100%", padding: "8px 10px", borderRadius: 6,
  border: "1px solid #2a2a2a", background: "#161616", color: "#bdbdbd",
  fontSize: 12, cursor: "pointer",
};
/** Action secondaire : lisible mais visiblement moins importante que `softBtn`. */
const ghostBtn: React.CSSProperties = {
  width: "100%", padding: "7px 10px", borderRadius: 6,
  border: "1px solid #212121", background: "transparent", color: "#8a8a8a",
  fontSize: 11, cursor: "pointer",
};
const primaryBtn: React.CSSProperties = {
  width: "100%", padding: "10px 12px", borderRadius: 6,
  border: "1px solid #2f4f43", background: "#16221d", color: "#dceee7",
  fontSize: 13, fontWeight: 600,
};
