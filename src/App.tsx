import { Component, lazy, Suspense, useEffect, useRef, useState, type ReactNode } from "react";
import "./App.css";
import GlucoseCanvas from "./canvas/GlucoseCanvas";
import Toolbar from "./components/Toolbar";
import BoardTabs from "./components/BoardTabs";
import PanelDock, { type TabId } from "./components/PanelDock";
import PomodoroOverlay from "./components/PomodoroOverlay";
import FolderViewportIndicator from "./components/FolderViewportIndicator";
import AppLaunchOverlay from "./components/AppLaunchOverlay";
import { useGlucoseStore, getActiveBoard } from "./store";
import { saveProject, loadProject } from "./utils/project";
import Toast, { showToast } from "./components/Toast";

// CLEANUP B-02 — Lazy-loading des panels lourds (split JS)
// Ils ne sont chargés que quand l'utilisateur les ouvre.
const PresetPanel = lazy(() => import("./components/PresetPanel"));
const DomainsPanel = lazy(() => import("./components/DomainsPanel"));
const SearchPanel = lazy(() => import("./components/SearchPanel"));
// Phase 6 — réglette temporelle (lazy : visible seulement quand activée)
const TemporalRuler = lazy(() => import("./components/TemporalRuler"));
const TemporalAnchorPrompt = lazy(() => import("./components/TemporalAnchorPrompt"));
// Phase 7.4 — Time Machine UI
const TimelinePanel = lazy(() => import("./components/TimelinePanel"));
// Phase 7.5bis — Multi-utilisateur LAN
const MultiplayerPanel = lazy(() => import("./multiplayer/MultiplayerPanel"));
import { useMultiplayerSync } from "./multiplayer/useMultiplayerSync";

class ErrorBoundary extends Component<{ children: ReactNode }, { error: Error | null }> {
  state = { error: null };
  static getDerivedStateFromError(e: Error) { return { error: e }; }
  render() {
    if (this.state.error) {
      const err = this.state.error as Error;
      return (
        <div style={{ padding: 32, color: "#f87171", background: "#0d0d0d", height: "100%", fontFamily: "monospace" }}>
          <b>Erreur :</b> {err.message}
          <pre style={{ fontSize: 11, marginTop: 12, color: "#666", whiteSpace: "pre-wrap" }}>{err.stack}</pre>
        </div>
      );
    }
    return this.props.children;
  }
}

export default function App() {
  const { project, loadProject: loadStore, loadDoc, setActiveTool, undo, redo } = useGlucoseStore();
  const pathRef = useRef<string | null>(null);
  const [presetOpen, setPresetOpen] = useState(false);
  const [domainsOpen, setDomainsOpen] = useState(false);
  // Sync l'ouverture des panels droits avec le store → la minimap se décale automatiquement
  useEffect(() => {
    useGlucoseStore.getState().setRightPanelOpen(presetOpen || domainsOpen);
  }, [presetOpen, domainsOpen]);
  const [zenMode, setZenMode] = useState(false);
  const [searchOpen, setSearchOpen] = useState(false);
  // Phase 6 — état UI réglette temporelle
  const [temporalRulerOpen, setTemporalRulerOpen] = useState(false);
  const [anchorPromptOpen, setAnchorPromptOpen] = useState(false);
  // Phase 7.4 — Time Machine UI
  const [timelineOpen, setTimelineOpen] = useState(false);
  // Phase 7.5bis — Multi-utilisateur LAN
  const [multiplayerOpen, setMultiplayerOpen] = useState(false);
  const [multiplayerEnabled, setMultiplayerEnabled] = useState(false);
  // Active la synchro Automerge ↔ peers tant que `multiplayerEnabled`
  useMultiplayerSync(multiplayerEnabled);
  const [dockTabs, setDockTabs]           = useState<TabId[]>([]);
  const [dismissingTabs, setDismissingTabs] = useState<TabId[]>([]);

  const DISMISS_DURATION = 200;

  function toggleDockTab(id: TabId) {
    if (dismissingTabs.includes(id)) {
      // Cancel pending close — snap back
      setDismissingTabs((d) => d.filter((x) => x !== id));
      return;
    }
    if (dockTabs.includes(id)) {
      // Close with exit animation, remove from DOM after it completes
      setDismissingTabs((d) => [...d, id]);
      setTimeout(() => {
        setDockTabs((t) => t.filter((x) => x !== id));
        setDismissingTabs((d) => d.filter((x) => x !== id));
      }, DISMISS_DURATION);
    } else {
      setDockTabs((t) => [...t, id]);
    }
  }

  function onDockDismiss(id: TabId) {
    // Called by PanelDock after its own drag-dismiss animation — just remove
    setDockTabs((t) => t.filter((x) => x !== id));
  }

  useEffect(() => {
    function onKey(e: KeyboardEvent) {
      if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement) return;

      // ── Sélection d'outils canvas (touche unique sans modifier) ──
      // Convention : on n'agit que si AUCUN modifier n'est appuyé pour éviter
      // de surcharger des combos comme Ctrl+F (recherche) ou Ctrl+Shift+F (fit).
      const noMod = !e.ctrlKey && !e.metaKey && !e.shiftKey && !e.altKey;
      if (noMod) {
        if (e.key === "v" || e.key === "V") setActiveTool("select");
        else if (e.key === "t" || e.key === "T") setActiveTool("text");
        else if (e.key === "n" || e.key === "N") setActiveTool("sticky");
        else if (e.key === "a" || e.key === "A") setActiveTool("arrow");
        else if (e.key === "f" || e.key === "F") setActiveTool("folder");
        else if (e.key === "m" || e.key === "M") setActiveTool("membrane");
      }

      if (e.key === " ") {
        e.preventDefault();
        setActiveTool("pan");
        useGlucoseStore.getState().setSelectedImageIds([]);
        useGlucoseStore.getState().setSelectedAnnotationIds([]);
      }
      if (e.key === "Escape") {
        setPresetOpen(false);
        setZenMode(false);
        setSearchOpen(false);
        setAnchorPromptOpen(false);
        setActiveTool("select");
        // Read current dockTabs via functional update to avoid stale closure
        setDockTabs((current) => {
          if (current.length > 0) {
            setDismissingTabs(() => [...current]);
            setTimeout(() => {
              setDockTabs([]);
              setDismissingTabs([]);
            }, DISMISS_DURATION);
          }
          return current;
        });
      }
      // Mode Zen (full-screen, panels cachés). F11 = convention OS standard.
      if (e.key === "F11") {
        e.preventDefault();
        setZenMode((v) => !v);
      }
      if ((e.ctrlKey || e.metaKey) && e.key === "a") {
        e.preventDefault();
        const { project: p } = useGlucoseStore.getState();
        useGlucoseStore.getState().selectAll(getActiveBoard(p).id);
      }
      if ((e.key === "g" || e.key === "G") && noMod) {
        useGlucoseStore.getState().toggleSmartGuides();
        const enabled = useGlucoseStore.getState().smartGuidesEnabled;
        showToast(enabled ? "Alignement intelligent activé" : "Alignement intelligent désactivé", "🧲");
      }
      // Phase 7.4 — Time Machine (toggle)
      if ((e.ctrlKey || e.metaKey) && (e.key === "h" || e.key === "H") && !e.shiftKey && !e.altKey) {
        e.preventDefault();
        setTimelineOpen((v) => !v);
      }
      // Phase 7.5bis — Multijoueur LAN (toggle panel)
      if ((e.ctrlKey || e.metaKey) && e.shiftKey && (e.key === "l" || e.key === "L")) {
        e.preventDefault();
        setMultiplayerOpen((v) => !v);
      }
      // Phase 6 — réglette temporelle
      if (e.shiftKey && (e.key === "T" || e.key === "t")) {
        e.preventDefault();
        const { selectedAnnotationIds, selectedImageIds } = useGlucoseStore.getState();
        const total = selectedAnnotationIds.length + selectedImageIds.length;
        if (total === 0) {
          showToast("Sélectionne d'abord un nœud à ancrer dans le temps", "📅");
        } else {
          setAnchorPromptOpen(true);
        }
      }
      if (e.shiftKey && (e.key === "R" || e.key === "r")) {
        e.preventDefault();
        setTemporalRulerOpen((v) => !v);
      }
      if (e.key === "Delete" || e.key === "Backspace") {
        const { project: p, selectedImageIds, selectedAnnotationIds } = useGlucoseStore.getState();
        const boardId = getActiveBoard(p).id;
        const total = selectedImageIds.length + selectedAnnotationIds.length;
        if (total > 0) {
          useGlucoseStore.getState().deleteSelected(boardId);
          window.dispatchEvent(new CustomEvent("glucose:hover-arrow", { detail: null }));
          showToast(`${total} élément${total > 1 ? "s" : ""} supprimé${total > 1 ? "s" : ""}`, "🗑");
        } else {
          // Demande de suppression dossier sélectionné via event
          window.dispatchEvent(new CustomEvent("glucose:delete-selected-folder"));
        }
      }
      if ((e.key === "l" || e.key === "L") && noMod) {
        const { project: p, selectedImageIds } = useGlucoseStore.getState();
        if (selectedImageIds.length > 0) {
          const board = getActiveBoard(p);
          const selImgs = board.images.filter((img) => selectedImageIds.includes(img.id));
          const allLocked = selImgs.every((img) => img.locked);
          const boardId = board.id;
          selectedImageIds.forEach((id) => useGlucoseStore.getState().updateImage(boardId, id, { locked: !allLocked }));
          showToast(allLocked ? "Images déverrouillées" : "Images verrouillées", allLocked ? "🔓" : "🔒");
        }
      }
      // Ctrl+I — import images (déclenche le bouton "Images" de la toolbar)
      if ((e.ctrlKey || e.metaKey) && (e.key === "i" || e.key === "I") && !e.shiftKey && !e.altKey) {
        e.preventDefault();
        window.dispatchEvent(new Event("glucose:trigger-import"));
      }
      if ((e.ctrlKey || e.metaKey) && e.key === "z" && !e.shiftKey) {
        e.preventDefault(); undo();
        showToast("Annulé", "↩");
      }
      if ((e.ctrlKey || e.metaKey) && (e.key === "y" || (e.key === "z" && e.shiftKey))) {
        e.preventDefault(); redo();
        showToast("Rétabli", "↪");
      }
      if ((e.ctrlKey || e.metaKey) && e.key === "c") {
        const { selectedImageIds, selectedAnnotationIds } = useGlucoseStore.getState();
        const total = selectedImageIds.length + selectedAnnotationIds.length;
        if (total > 0) showToast(`${total} élément${total > 1 ? "s" : ""} copié${total > 1 ? "s" : ""}`, "📋");
      }
      if ((e.ctrlKey || e.metaKey) && e.key === "x") {
        const { selectedImageIds, selectedAnnotationIds } = useGlucoseStore.getState();
        const total = selectedImageIds.length + selectedAnnotationIds.length;
        if (total > 0) showToast(`${total} élément${total > 1 ? "s" : ""} coupé${total > 1 ? "s" : ""}`, "✂");
      }
      if ((e.ctrlKey || e.metaKey) && e.key === "d") {
        e.preventDefault();
        const { project: p } = useGlucoseStore.getState();
        useGlucoseStore.getState().duplicateSelected(getActiveBoard(p).id);
      }
      // Phase 5 — Alt+1..4 : transforme un sticky sélectionné en opérateur logique
      // (AND, OR, BUT, BECAUSE). Alt+0 retire l'opérateur (redevient sticky normal).
      if (e.altKey && !e.ctrlKey && !e.metaKey && !e.shiftKey
          && ["0", "1", "2", "3", "4"].includes(e.key)) {
        const state = useGlucoseStore.getState();
        const board = getActiveBoard(state.project);
        const selectedStickies = board.annotations.filter(
          a => a.type === "sticky" && state.selectedAnnotationIds.includes(a.id)
        );
        if (selectedStickies.length === 0) return;
        e.preventDefault();
        const map: Record<string, "AND" | "OR" | "BUT" | "BECAUSE" | undefined> = {
          "0": undefined, "1": "AND", "2": "OR", "3": "BUT", "4": "BECAUSE",
        };
        const op = map[e.key];
        for (const s of selectedStickies) {
          state.updateAnnotation(board.id, s.id, { operator: op });
        }
        const labels = { AND: "ET", OR: "OU", BUT: "MAIS", BECAUSE: "PARCE QUE" } as const;
        showToast(op ? `Opérateur ${labels[op]}` : "Sticky standard restauré", "⊕");
      }
      // Phase 4 — Ctrl+Shift+M : créer miroir(s) de la sélection (offset 40px)
      if ((e.ctrlKey || e.metaKey) && e.shiftKey && (e.key === "m" || e.key === "M")) {
        e.preventDefault();
        const state = useGlucoseStore.getState();
        const board = getActiveBoard(state.project);
        const OFFSET = 40;
        let count = 0;
        const cycleRefused = 0;
        for (const annId of state.selectedAnnotationIds) {
          const ann = board.annotations.find((a) => a.id === annId);
          if (!ann) continue;
          const newId = state.mirrorAnnotation(board.id, annId, ann.x + OFFSET, ann.y + OFFSET);
          if (newId) count++;
        }
        for (const imgId of state.selectedImageIds) {
          const img = board.images.find((i) => i.id === imgId);
          if (!img) continue;
          const newId = state.mirrorImage(board.id, imgId, img.x + OFFSET, img.y + OFFSET);
          if (newId) count++;
        }
        // (Pour les dossiers : on n'a pas encore d'UI de sélection multi-folder ; le mirroring
        // de dossiers passe via un menu contextuel à venir, mais l'API store est déjà prête.)
        if (count > 0) showToast(`${count} miroir${count > 1 ? "s" : ""} créé${count > 1 ? "s" : ""}`, "↻");
        else if (cycleRefused > 0) showToast("Cycle Inception refusé", "⚠");
      }
      if ((e.ctrlKey || e.metaKey) && e.shiftKey && (e.key === "f" || e.key === "F")) {
        e.preventDefault();
        window.dispatchEvent(new Event("glucose:fit-view"));
      }
      if ((e.ctrlKey || e.metaKey) && e.key === "f") {
        e.preventDefault();
        setSearchOpen((v) => !v);
      }
      if ((e.ctrlKey || e.metaKey) && (e.key === "s" || e.key === "S")) {
        e.preventDefault();
        // Ctrl+Maj+S = "Enregistrer sous" → ignore le path courant pour
        // forcer un dialog. Ctrl+S = enregistrement rapide sur le path courant.
        const forceDialog = e.shiftKey;
        const targetPath = forceDialog ? undefined : (pathRef.current ?? undefined);
        saveProject(project, targetPath)
          .then((p) => {
            if (p) {
              pathRef.current = p;
              showToast(forceDialog ? "Projet enregistré sous…" : "Projet enregistré", "💾");
            }
          })
          .catch((err) => alert(`Erreur de sauvegarde:\n${err?.message || String(err)}`));
      }
      if ((e.ctrlKey || e.metaKey) && e.key === "o") {
        e.preventDefault();
        loadProject()
          .then((r) => {
            if (!r) return;
            if (r.doc) loadDoc(r.doc); // v2 — historique Automerge préservé
            else loadStore(r.project);  // v1 ou migration legacy — doc neuf
            pathRef.current = r.path;
          })
          .catch((err) => alert(`Erreur de chargement:\n${err?.message || String(err)}`));
      }
    }
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [project]);

  return (
    <div style={{ display: "flex", flexDirection: "column", width: "100vw", height: "100vh", background: "#0d0d0d", overflow: "hidden" }}>
      {!zenMode && (
        <>
          <ErrorBoundary>
            <Toolbar
              onTogglePreset={() => setPresetOpen((v) => !v)}
              presetPanelOpen={presetOpen}
              onToggleDomains={() => setDomainsOpen((v) => !v)}
              domainsPanelOpen={domainsOpen}
              onToggleOrganize={() => toggleDockTab("organize")}
              organizePanelOpen={dockTabs.includes("organize")}
              onToggleStoryboard={() => toggleDockTab("storyboard")}
              storyboardPanelOpen={dockTabs.includes("storyboard")}
              onTogglePomodoro={() => toggleDockTab("pomodoro")}
              pomodoroOpen={dockTabs.includes("pomodoro")}
            />
          </ErrorBoundary>
          <ErrorBoundary>
            <BoardTabs />
          </ErrorBoundary>
        </>
      )}

      {/* Indicateur zen mode */}
      {zenMode && (
        <div
          onClick={() => setZenMode(false)}
          title="Quitter le mode zen (F ou Escape)"
          style={{
            position: "fixed", top: 8, right: 12,
            fontSize: 10, color: "#2a2a2a", cursor: "pointer",
            zIndex: 100, letterSpacing: 1,
          }}
          onMouseOver={(e) => { e.currentTarget.style.color = "#555"; }}
          onMouseOut={(e) => { e.currentTarget.style.color = "#2a2a2a"; }}
        >
          ZEN · F
        </div>
      )}

      {/* Main area: canvas + floating panels */}
      <div style={{ flex: 1, position: "relative", overflow: "hidden", display: "flex" }}>
        <ErrorBoundary>
          <GlucoseCanvas />
        </ErrorBoundary>

        {/* Phase 7.5 — indicateur visuel "vous êtes dans un dossier" */}
        <FolderViewportIndicator />

        {/* R-FIL — animation de lancement d'app native (logo + couleur dominante) */}
        <AppLaunchOverlay />

        {/* Floating right panels (lazy-loaded — Suspense montre rien le temps que ça charge) */}
        {presetOpen && (
          <ErrorBoundary>
            <Suspense fallback={null}>
              <PresetPanel onClose={() => setPresetOpen(false)} />
            </Suspense>
          </ErrorBoundary>
        )}
        {domainsOpen && (
          <ErrorBoundary>
            <Suspense fallback={null}>
              <DomainsPanel onClose={() => setDomainsOpen(false)} />
            </Suspense>
          </ErrorBoundary>
        )}

        {/* Pomodoro always-visible overlay when timer running */}
        <PomodoroOverlay onOpen={() => {
          if (!dockTabs.includes("pomodoro")) toggleDockTab("pomodoro");
        }} />

        {/* Unified bottom-right dock */}
        <ErrorBoundary>
          <PanelDock openTabs={dockTabs} dismissingTabs={dismissingTabs} onDismiss={onDockDismiss} />
        </ErrorBoundary>

        {searchOpen && (
          <ErrorBoundary>
            <Suspense fallback={null}>
              <SearchPanel onClose={() => setSearchOpen(false)} />
            </Suspense>
          </ErrorBoundary>
        )}

        {/* Phase 6 — réglette temporelle (Shift+R) */}
        {temporalRulerOpen && (
          <ErrorBoundary>
            <Suspense fallback={null}>
              <TemporalRuler onClose={() => setTemporalRulerOpen(false)} />
            </Suspense>
          </ErrorBoundary>
        )}
        {anchorPromptOpen && (
          <ErrorBoundary>
            <Suspense fallback={null}>
              <TemporalAnchorPrompt onClose={() => setAnchorPromptOpen(false)} />
            </Suspense>
          </ErrorBoundary>
        )}

        {/* Phase 7.4 — Time Machine (Ctrl+H) */}
        {timelineOpen && (
          <ErrorBoundary>
            <Suspense fallback={null}>
              <TimelinePanel onClose={() => setTimelineOpen(false)} />
            </Suspense>
          </ErrorBoundary>
        )}

        {/* Phase 7.5bis — Multijoueur LAN (Ctrl+Shift+L) */}
        {multiplayerOpen && (
          <ErrorBoundary>
            <Suspense fallback={null}>
              <MultiplayerPanel
                enabled={multiplayerEnabled}
                onToggle={setMultiplayerEnabled}
                onClose={() => setMultiplayerOpen(false)}
              />
            </Suspense>
          </ErrorBoundary>
        )}
      </div>
      <Toast />
    </div>
  );
}
