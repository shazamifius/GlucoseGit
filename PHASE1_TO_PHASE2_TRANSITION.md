# 🧬 Glucose — Transition Phase 1 → Phase 2

> Document de synthèse architecturale et feuille de route stratégique.
>
> **Date :** 2026-05-28 (créé) · **maj 2026-05-29** (Sprint 2 file manager + Sprint 1 closé)
> **Version courante :** 0.3.0-rc (Phase 1 = tout ce qui est livré jusqu'à Phase 7.5bis incluse, plus polish UI Sprint 1)
> **Auteur :** Bilan technique pré-Phase 2.

---

## 📑 Table des matières

1. [Étape 1 — Bilan & cartographie de la Phase 1](#étape-1--bilan--cartographie-de-la-phase-1)
2. [Étape 2 — Nettoyage & refactoring (prérequis Phase 2)](#étape-2--nettoyage--refactoring-prérequis-phase-2)
3. [Étape 3 — Feuille de route Phase 2 : RAG Universel + Test du Wiki](#étape-3--feuille-de-route-phase-2--rag-universel--test-du-wiki)

---

# Étape 1 — Bilan & cartographie de la Phase 1

> Note de cadrage : dans la roadmap historique, « Phase 1 » désignait à l'origine
> le sélecteur de zone. Dans ce document, **« Phase 1 » désigne l'ensemble de
> l'existant livré** (Phase 0 hygiène → Phase 7.5bis multijoueur LAN inclus),
> conformément à l'instruction utilisateur. C'est notre socle pour la « Phase 2 ».

## 1.1 Vision invariante

Glucose est une **surface cognitive infinie offline-first**. Trois lois de rendu
encadrent toute décision technique :

1. **Loi du Zoom Sémantique** — densité d'information visible constante.
2. **Loi de la Connexion Latente** — un lien existe dans la donnée, mais n'est
   rendu que s'il est pertinent au focus.
3. **Loi du Domaine Coloré** — un nœud appartient à 1..N domaines pondérés ;
   membrane = signature, badge = lève l'ambiguïté.

## 1.2 Stack technique consolidée

```
┌──────────────────────────────────────────────────────────────────────────┐
│  Frontend                                                                │
│  ─ React 19  + TypeScript 5.8                                            │
│  ─ Tailwind 4 (utilitaires)                                              │
│  ─ Zustand 5 (store UI + façade Automerge)                               │
│  ─ Zod 3 (validation .glucose v1 legacy)                                 │
│  ─ PixiJS 8 (raster : images, sprites, membranes)                        │
│  ─ SVG overlay React (vectoriel : flèches, dossiers, badges)             │
│  ─ HTML overlay (texte / sticky avec édition Markdown + LaTeX)           │
│                                                                          │
│  Persistance & CRDT                                                      │
│  ─ @automerge/automerge 3.2.6 (WASM via vite-plugin-wasm)                │
│  ─ Format .glucose v2 binaire (Automerge save())                         │
│  ─ Migration transparente v1 JSON → v2 binaire                           │
│  ─ Assets externalisés : asset:<sha256_16>.<ext> + dédup disque          │
│       ⚠️ Sprint 2 R-EMB-01 va passer en embed Automerge par défaut       │
│                                                                          │
│  Backend Rust (src-tauri)                                                │
│  ─ Tauri 2 (desktop) — capabilities strictes                             │
│  ─ reqwest + rustls (HTTP, anti-SSRF)                                    │
│  ─ tokio (async runtime)                                                 │
│  ─ open crate (App Bridge — Blender, Photoshop, Krita…)                  │
│  ─ yt-dlp PINNÉ avec vérification SHA-256                                │
│  ─ mdns-sd + tokio-tungstenite (multijoueur LAN)                         │
│  ─ sha2 + hex (hashing assets / yt-dlp)                                  │
└──────────────────────────────────────────────────────────────────────────┘
```

## 1.3 Cartographie du code (≈ 16 000 LOC TypeScript + ≈ 800 LOC Rust)

### Arborescence et rôle de chaque fichier

```
src/
├── App.tsx                    ── Bootstrap React, raccourcis globaux,
│                                  ErrorBoundary, lazy-loading des panels
├── main.tsx                   ── Entry point (StrictMode désactivé)
├── constants.ts               ── VIEWPORT, TIMING, CLUSTERING, SYMBIOSIS,
│                                  PATHFINDING, MIRROR, PANEL, LIMITS
│
├── types/index.ts             ── SCHÉMA CENTRAL : Domain, DomainAssignment,
│                                  TemporalAnchor, BoardImage, Annotation,
│                                  StoryboardPanel/Settings, Preset(Slot),
│                                  BoardZone, CanvasFolder, Board, Viewport,
│                                  Project, Tool
│
├── store/
│   ├── index.ts (1346 LOC)    ── Store Zustand CRDT-first. _doc Automerge =
│   │                              source de vérité, project = vue lecture.
│   │                              mutate() central + ~30 actions migrées.
│   │                              Undo/redo via stacks de Doc.
│   │                              Cascades miroirs + folders (BFS).
│   │                              Pomodoro module-level.
│   ├── automerge.ts           ── Wrapper minimal : create/change/save/load/
│   │                              merge/clone/applyChanges/getChanges/
│   │                              viewAt/getHeads/history/asPlain
│   ├── automerge.test.ts      ── 12 tests : roundtrip, merge commutatif,
│   │                              time machine viewAt, taille binaire
│   ├── mirrorGraph.ts         ── wouldCreateMirrorCycle() — BFS pur
│   ├── mirrorGraph.test.ts    ── Tests anti-cycle Inception
│   └── projectSchema.ts       ── Zod schemas (validation v1 JSON legacy)
│
├── canvas/
│   ├── GlucoseCanvas.tsx (2428 LOC) ── ⚠️ MONOLITHE. Init PixiJS, sync
│   │                                    images/annotations/membranes/zones,
│   │                                    pan/zoom, drag-create, capture
│   │                                    folders, navigation par zoom,
│   │                                    transition enter/exit board,
│   │                                    overlay édition in-place,
│   │                                    minimap, ghost preview, ZoneSelector
│   ├── HtmlAnnotationLayer.tsx (887 LOC) ── Rendu HTML des text/sticky
│   │                                       avec symbiose chromatique,
│   │                                       Markdown + KaTeX, badges Phase 3-6
│   ├── ArrowSvgLayer.tsx (659 LOC) ── Rendu SVG des flèches : pathfinding,
│   │                                  prédicats colorés, portails, longText
│   ├── FolderSvgLayer.tsx (487 LOC) ── Cadre SVG des dossiers + preview live
│   ├── AnnotationLayer.ts (509 LOC) ── Vestige PixiJS (annulé Phase 7.5)
│   ├── MembraneRenderer.ts    ── Clustering Union-Find + Convex Hull (Gift
│   │                              Wrapping) + somme vectorielle teintes
│   ├── Quadtree.ts (54 LOC)   ── SpatialHash actif (rectangle culling)
│   ├── SvgAnnotationLayer.tsx ── Mesure de texte
│   ├── ZoneRenderer.ts        ── Zones de presets (drag-drop slots)
│   ├── ZoneSelectorOverlay.tsx── Sélecteur de zone (Phase 1 historique)
│   ├── StoryboardLayer.ts     ── Panneaux storyboard
│   ├── AnnotationBadges.tsx   ── Badges 📅 ↻ + domaine
│   ├── dropHandler.ts         ── Détection drop web (URL/HTML/clipboard),
│   │                              URL CDN sans extension, og:image, lazy-src
│   └── fileImport.ts          ── Import disque (drag local + Ctrl+I)
│
├── components/
│   ├── Toolbar.tsx            ── Barre d'outils principale
│   ├── BoardTabs.tsx          ── Onglets de boards, réorder DnD
│   ├── PanelDock.tsx          ── Dock bottom-right (Organize/Storyboard/Pomodoro)
│   ├── Minimap.tsx            ── Radar 180×120, clic-to-navigate
│   ├── SearchPanel.tsx        ── Recherche globale Ctrl+F (textuelle)
│   ├── DomainsPanel.tsx       ── CRUD domaines + assignation pondérée
│   ├── PresetPanel.tsx        ── Presets créatifs
│   ├── OrganizePanel.tsx      ── 5 modes layout + tri taille/ratio/lum
│   ├── StoryboardControls.tsx ── Aspect ratio, cols, gap
│   ├── TemporalRuler.tsx      ── Réglette temporelle zoomable (Shift+R)
│   ├── TemporalAnchorPrompt.tsx ── Modal d'ancrage (Shift+T)
│   ├── TimelinePanel.tsx      ── Time Machine UI (Ctrl+H)
│   ├── FolderBreadcrumb.tsx   ── Breadcrumb VSCode-like
│   ├── FolderViewportIndicator.tsx ── Bordure colorée immersive
│   ├── ColorPicker.tsx        ── HSV style Blender
│   ├── ArrowOptions.tsx       ── Choix prédicat + style
│   ├── ArrowTextEditor.tsx    ── Édition texte de flèche
│   ├── ArrowDescriptionPanel.tsx ── Panneau Markdown/LaTeX longText
│   ├── SyntaxEditor.tsx       ── Textarea + overlay syntaxe
│   ├── AppBridgeIcon.tsx      ── 30+ icônes par extension
│   ├── PomodoroTimer.tsx + PomodoroOverlay.tsx
│   └── Toast.tsx              ── Notifications
│
├── multiplayer/
│   ├── MultiplayerPanel.tsx   ── UI Ctrl+Shift+L (peers, manual connect)
│   └── useMultiplayerSync.ts  ── Diffuse getChanges() à chaque mutation,
│                                  reçoit mp:patch et applyRemoteChanges
│
├── data/defaultPresets.ts     ── Presets builtin (CharaDesign, MoodBoard…)
│
└── utils/
    ├── project.ts             ── saveProject (v2 binaire) + loadProject
    │                              (détection v2/v1, base64 chunk-safe)
    ├── assets.ts              ── resolveAssetSrc, migrateLegacyAssets,
    │                              getAssetsDir (cache mémoire)
    ├── timeline.ts            ── Parsing dates souples + DEFAULT_ERAS (30)
    │                              formatAdaptive (1789 / 10 ka / 1,5 Ma)
    ├── timeline.test.ts       ── 29 tests
    ├── layout.ts              ── Algorithmes d'organisation auto
    ├── glucoseBus.ts          ── Bus d'événements custom
    ├── exportPng.ts           ── WebGL → binaire Tauri
    ├── imageUpgrade.ts        ── Upgrade auto résolution CDN
    ├── cursorWrap.ts          ── Wrap curseur pendant drag pan
    ├── loadKatexCss.ts        ── Chargement KaTeX différé
    └── nanoid.ts              ── ID 21 chars URL-safe

src-tauri/src/
├── lib.rs (768 LOC)           ── validate_scope (canonicalize + 7 roots),
│                                  fetch_image (anti-SSRF + Referer racine),
│                                  read_image_file, open_in_app (whitelist
│                                  ALLOWED_OPEN_EXTS, refus FORBIDDEN_*),
│                                  read/write_project_file,
│                                  read/write_glucose_binary (v2),
│                                  write_binary_file (export PNG),
│                                  download_video (yt-dlp timeout 5 min),
│                                  ensure_yt_dlp (PIN + SHA256 hardcoded),
│                                  save_asset / load_asset / get_assets_dir
└── multiplayer.rs (~330 LOC)  ── mDNS announce + browse service
                                  `_glucose._tcp.local`, serveur WS port 7777,
                                  broadcast mpsc, 5 commandes + 6 events
```

## 1.4 Modèle de données (single source of truth)

Le **`Project`** est l'entité racine. Une instance d'application = un seul
`Project` chargé en mémoire dans un `Automerge.Doc`. Sa hiérarchie :

```
Project
├── version, name, createdAt, updatedAt
├── activeBoardId      → id d'un Board présent
├── domains[]          → Domain { id, name, color, icon, createdAt }
├── presets[]          → Preset { id, name, slots[], isBuiltin, createdAt }
└── boards[]
    └── Board
        ├── id, name, viewport, createdAt, updatedAt
        ├── presetId?, zones[], folders[], bookmarks?
        ├── images[]   → BoardImage (src, geom, rotation, locked, tags,
        │                slotId?, sourceUrl?, isVideo?, mirrorOf?,
        │                domains[]?, temporalAnchor?)
        ├── annotations[] → Annotation (type: text|sticky|arrow|membrane)
        │                   ├── text/sticky : text?, bgColor?, w, h
        │                   ├── arrow : x2, y2, predicate?, sourceId?,
        │                   │   targetId?, sourceBlockId?, sourceTextSel?,
        │                   │   waypoints[]?, longText?, targetBoardId?
        │                   ├── sticky-operator : operator: AND|OR|BUT|BECAUSE
        │                   └── communs : domains?, temporalAnchor?,
        │                       mirrorOf?, pinned?
        ├── panels[]   → StoryboardPanel (frame + description)
        └── storyboard? → StoryboardSettings
```

**Invariants :**
- Un `Board` ne contient JAMAIS d'images d'un autre board (folders sont des
  sous-boards référencés par `folder.childBoardId`).
- Un **dossier-miroir partage le même `childBoardId`** que l'original
  → mutation propagée automatiquement (clé de l'architecture Miroirs).
- `activeBoardId` est garanti existant en cascade au remove (cf. `removeBoard`).
- `targetBoardId` orphelin sur flèche portail → patché à `undefined` au remove.
- Cascade miroirs : point-fixe BFS sur `mirrorOf` (max 16 sauts).
- Cascade folders : BFS pour identifier les childBoards orphelins.

## 1.5 Architecture du Store (CRDT-first)

```
┌──────────────────────────────────────────────────────────────────────────┐
│  useGlucoseStore (Zustand)                                               │
│                                                                          │
│  ─ État doc :                                                            │
│      _doc: Doc<Project>          ← source de vérité Automerge            │
│      project: Project            ← proxy lecture (doc casté)             │
│      _undoStack, _redoStack: Doc[] (max 50, structural sharing)          │
│      _previewHeads: Heads | null ← mode Time Machine (mutations bloquées)│
│                                                                          │
│  ─ Helper central :                                                      │
│      mutate(message, mutator) ─┬─→ A.change(doc, message, draft)         │
│                                ├─→ push old doc dans _undoStack          │
│                                ├─→ vide _redoStack                       │
│                                └─→ bloqué si _previewHeads !== null      │
│                                                                          │
│  ─ ~50 actions exposées, toutes via mutate() :                           │
│    setViewport, addImage, updateImage, removeImages (cascade miroirs),   │
│    updateMultipleImages, addAnnotation, updateAnnotation,                │
│    removeAnnotations (cascade flèches orphelines), deleteSelected,       │
│    duplicateSelected, moveSelected, setStoryboardSettings,               │
│    clearStoryboard, addPanel, updatePanel, removePanel, reorderPanels,   │
│    addBoard, removeBoard (patch targetBoardId orphelin + reset active),  │
│    renameBoard, reorderBoards, setActiveBoardId (reconstruit             │
│    folderStack), duplicateBoard, applyPresetToBoard, setBoardZones,      │
│    addPreset, removePreset, updatePreset, addDomain, updateDomain,       │
│    removeDomain (cascade des assignments), assignDomainToNode,           │
│    mirrorAnnotation, mirrorImage, mirrorFolder (anti-cycle),             │
│    findOriginal* (suit la chaîne mirrorOf), createFolder (capture        │
│    des items inside), updateFolder, removeFolders (BFS childBoards       │
│    orphelins), enterFolder/exitFolder/exitToRoot, setProjectName,        │
│    loadProject (doc neuf) / loadDoc (préserve historique), applyRemote   │
│    Changes (LAN, hors _undoStack), commitNamed, restoreToPreview, …      │
│                                                                          │
│  ─ État UI local (jamais persisté dans doc) :                            │
│      activeTool, selectedImageIds, selectedAnnotationIds,                │
│      folderStack, hoveredNodeId, transDomainVisible,                     │
│      temporalFilter, rightPanelOpen, smartGuidesEnabled, pomodoro*       │
│                                                                          │
│  ─ Bornes (clampViewport, clampSpatial) sur tout objet écrit :           │
│      COORD ±1 000 000 · SIZE 1..200 000 · SCALE 0.005..50                │
└──────────────────────────────────────────────────────────────────────────┘
```

### Subtilités à connaître pour ne pas casser le store

- À l'intérieur d'un mutator Automerge, `arr.filter()` / `arr.map()` ne
  mutent PAS le doc — utiliser `splice()` ou helper `removeWhere()`.
- Automerge refuse `undefined` à l'insertion → `clampSpatial()` strippe
  les clés undefined avant `push`.
- Un proxy Automerge importé dans un nouveau draft échoue → systématiquement
  `JSON.parse(JSON.stringify(...))` pour cloner un proxy avant `push`.
- `applyRemoteChanges` ne touche PAS `_undoStack` — sinon Ctrl+Z annulerait
  les actions des pairs.
- `loadDoc` préserve l'historique Automerge, `loadProject` crée un doc neuf.

## 1.6 Architecture du rendu (3 couches synchronisées)

```
┌──────────────────────────────────────────────────────────────────────────┐
│  GlucoseCanvas (orchestrateur, monolithe 2428 LOC)                       │
│                                                                          │
│  ┌─ PixiJS World (Container Pixi)──────────────────────────────────────┐ │
│  │  z-index 0 : MembraneRenderer (Graphics)                            │ │
│  │  z-index 1 : ZoneRenderer (Graphics presets)                        │ │
│  │  z-index 2 : StoryboardLayer (panneaux + cadres)                    │ │
│  │  z-index N : Sprite par image (asset:..., data:, http://)           │ │
│  │                                                                     │ │
│  │  SpatialHash global → culling (50% marge) au pan/zoom               │ │
│  └────────────────────────────────────────────────────────────────────┘ │
│                              ↕  synchro via vpRef + transform           │
│  ┌─ SVG Overlay (React) ───────────────────────────────────────────────┐ │
│  │  ArrowSvgLayer  : flèches + pathfinding anti-obstacles + badges     │ │
│  │  FolderSvgLayer : cadre folder + preview live + breadcrumb          │ │
│  │  SvgAnnotationLayer : mesure texte                                  │ │
│  │  AnnotationBadges : ↻ miroir, 📅 temporel, badges domaine (>40%)   │ │
│  └────────────────────────────────────────────────────────────────────┘ │
│                                                                          │
│  ┌─ HTML Overlay (React) ──────────────────────────────────────────────┐ │
│  │  HtmlAnnotationLayer : texte Markdown + sticky + symbiose chromatic │ │
│  │  EditOverlay : textarea fixed transparent (édition in-place)        │ │
│  │  Minimap, FolderBreadcrumb, TemporalRuler, Modals, Panels droits    │ │
│  └────────────────────────────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────────────────────────────┘

   Système d'événements transverse (CustomEvent sur window) :
   ─ glucose:teleport-to-mirror-original
   ─ glucose:portal-jump
   ─ glucose:fit-view
   ─ glucose:trigger-import
   ─ glucose:hover-arrow
   ─ glucose:delete-selected-folder
   ─ glucose:zone-selected
```

**Pourquoi 3 couches ?**
- PixiJS gère mille sprites/membranes en WebGL → performance.
- SVG bénéficie du `vectorEffect="non-scaling-stroke"` → flèches nettes à
  tout zoom.
- HTML permet l'édition Markdown + LaTeX en place (caret natif).

## 1.7 Sécurité (Sprint 1 bouclé)

| Vecteur | Mitigation |
|---|---|
| **RCE** via `open_in_app` | `FORBIDDEN_OPEN_EXTS` (exe/bat/ps1/lnk/...) + `ALLOWED_OPEN_EXTS` whitelist |
| **SSRF** dans `fetch_image` | Parse URL → résolution DNS → bloque IP privées/loopback/CGN/cloud-meta |
| **XSS** dans Markdown | react-markdown sans `rehype-raw` (HTML brut interdit) |
| **Path traversal** | `validate_scope()` canonicalise et exige `starts_with(allowed_root)` |
| **UNC paths** (`\\server\share`) | Refus explicite |
| **Supply-chain yt-dlp** | Version PINNED + vérif SHA-256 hardcoded |
| **Validation entrées** | Zod sur tous les `.glucose` v1, clamp coords/scale partout |
| **Capabilities Tauri** | Minimales (`tauri.conf.json`, `dragDropEnabled: false`) |

## 1.8 Tests automatisés (55 verts)

- `store/automerge.test.ts` (12) — roundtrip, merge, branches divergentes,
  viewAt, taille binaire compacte.
- `store/mirrorGraph.test.ts` — anti-cycle BFS.
- `utils/timeline.test.ts` (29) — parsing souple, format adaptatif,
  matching fenêtre, gestion époques nommées.
- Reste réparti en `*.test.ts` modulaires.

## 1.9 Synthèse — ce qui est solide vs. ce qui demande du soin

### ✅ Forces structurelles
- **Type system exceptionnel** : `Annotation` riche (prédicats, sub-blocks,
  waypoints, miroirs, domaines, temporel) — base saine pour Phase 2.
- **CRDT-first** : le doc Automerge est *vraiment* la source de vérité, pas
  un cache. Time Machine, multijoueur LAN, undo infini en découlent gratis.
- **Cascades correctes** : suppression d'un original purge miroirs, flèches
  orphelines, child boards orphelins (BFS).
- **Sécurité non-naïve** : SSRF, RCE, supply-chain pris au sérieux.
- **Frontière étanche** WikiGit ↔ CRDT prévue dès la roadmap (Phase 9).

### ⚠️ Zones de tension
- `GlucoseCanvas.tsx` à **2428 LOC** : couplage fort, useEffect imbriqués,
  refs partout. Premier candidat à la découpe.
- `store/index.ts` à **1346 LOC** : un seul fichier pour ~50 actions.
- Pas de **registre de pages / sub-blocs** côté types — `sourceBlockId`
  existe mais pas le concept de Block consultable.
- Pas de **système de plugin / IA hook** — tout est ad-hoc.
- Aucune **abstraction RAG** : ni embeddings, ni vecteurs, ni adapter LLM.

---

# Étape 2 — Nettoyage & refactoring (prérequis Phase 2)

> Avant d'attaquer la Phase 2, on doit assainir trois plans : **modularité**
> (le code ne tient plus dans la tête à 2428 LOC), **typage strict** (préparer
> les nouveaux concepts), **frontière** (séparer dur ce qui va devenir
> moteur, plugin, adaptateur).
>
> Chaque tâche cible un livrable concret et borné. La règle d'or : **aucune
> tâche ne doit changer le comportement utilisateur**. C'est de la chirurgie
> propre, pas une refonte.

## 2.1 Modularité — découper les monolithes

### R-MOD-01 — Découper `GlucoseCanvas.tsx` (2428 → ~600 + sous-hooks)

Le fichier mélange aujourd'hui : init Pixi, sync images, sync annotations,
sync zones, sync membranes, pan/zoom, drag-create, capture folders,
navigation par zoom, transitions enter/exit, overlay édition, ghost preview,
minimap, événements globaux (10+ `useEffect`).

**Plan de découpe** :

```
src/canvas/
├── GlucoseCanvas.tsx          ── ≤ 600 LOC : composition + JSX racine
├── hooks/
│   ├── usePixiApp.ts          ── init Application Pixi + cleanup safe
│   ├── useImageSync.ts        ── sync sprites ↔ board.images + culling
│   ├── useMembraneSync.ts     ── MembraneRenderer.update() debounced
│   ├── useZoneSync.ts         ── ZoneRenderer.update()
│   ├── useStoryboardSync.ts   ── StoryboardLayer.sync()
│   ├── usePanZoom.ts          ── pan/zoom + clampViewport + emitViewport
│   ├── useFolderTransition.ts ── animation enter/exit board (RAF safe)
│   ├── useAutoNavigate.ts     ── zoom-in/out auto sur folders (cooldown)
│   ├── useZoneSelector.ts     ── hook drag-create dossier/membrane
│   ├── useEditOverlay.ts      ── textarea fixed + suivi monde→écran
│   ├── useGhostPreview.ts     ── preview preset/storyboard suivant curseur
│   ├── useDropTarget.ts       ── intégration dropHandler + drop disque
│   └── useArrowDrawing.ts     ── drag flèche + snap aux cibles
└── (couches existantes inchangées : SvgAnnotationLayer, ArrowSvgLayer,
    HtmlAnnotationLayer, FolderSvgLayer, MembraneRenderer, ZoneRenderer,
    StoryboardLayer, AnnotationLayer, Quadtree, dropHandler, fileImport)
```

**Critères d'acceptation** :
- Aucun test ne casse.
- Aucun raccourci clavier ni interaction utilisateur ne diffère.
- Le diff visuel (cf. `bun run tauri dev`) est nul sur la démo Wiki.

### R-MOD-02 — Découper `store/index.ts` (1346 → ~250 + 6 modules)

```
src/store/
├── index.ts                   ── ~250 LOC : assemblage Zustand, mutate(),
│                                  undo/redo, time machine, applyRemoteChanges
├── slices/
│   ├── imagesSlice.ts         ── addImage, updateImage, removeImages,
│   │                              updateMultipleImages + cascade flèches
│   ├── annotationsSlice.ts    ── add/update/remove + cascade miroirs
│   ├── boardsSlice.ts         ── add/remove/rename/reorder/setActive/
│   │                              duplicate + reconstruction folderStack
│   ├── foldersSlice.ts        ── createFolder (capture), updateFolder,
│   │                              removeFolders (BFS childBoards), enter/exit
│   ├── domainsSlice.ts        ── CRUD domaines + cascade assignments
│   ├── mirrorsSlice.ts        ── mirrorAnnotation/Image/Folder + findOriginal*
│   └── selectionSlice.ts      ── selectAll, deleteSelected, duplicateSelected,
│                                  moveSelected (état UI local + actions)
├── helpers/
│   ├── mutators.ts            ── removeWhere, indexById, clampSpatial,
│   │                              clampViewport
│   └── factories.ts           ── newBoard, DEFAULT_PROJECT, DEFAULT_BOARD_ID
├── automerge.ts (inchangé)
├── mirrorGraph.ts (inchangé)
└── projectSchema.ts           ── à enrichir avec types Phase 2 (cf. R-TYP-02)
```

**Pattern de slice** : chaque slice exporte une fonction
`createXxxSlice(set, get, mutate)` qui renvoie le sous-objet du store.

### R-MOD-03 — Sortir l'`AnnotationLayer.ts` mort (509 LOC)

Le code roadmap dit que les flèches PixiJS sont retirées au profit du SVG.
Le fichier est gardé mais ne reçoit plus que `[]`. Soit on le supprime
complètement, soit on le réduit à ce dont on a *vraiment* besoin (rendu
texte/sticky Pixi pour le LOD MACRO). Décision recommandée : **supprimer**
puisque le LOD a aussi été retiré en Phase 7.5.

## 2.2 Typage strict — préparer les nouveaux concepts

### R-TYP-01 — Renforcer les types existants

Aujourd'hui `Annotation` mutualise 4 sous-types via le discriminant `type`.
Conséquence : `text?`, `x2?`, `predicate?`, `operator?` cohabitent et le
typage rend tout optionnel — invariant runtime à respecter à la main.

**Refactor** : passer en union discriminée stricte.

```ts
// types/index.ts

interface AnnotationBase {
  id: string;
  x: number;
  y: number;
  domains?: DomainAssignment[];
  mirrorOf?: string;
  temporalAnchor?: TemporalAnchor;
}

interface TextAnnotation extends AnnotationBase {
  type: "text";
  text: string;
  fontSize?: number;
  color?: string;
  width?: number;
  height?: number;
  cursorPos?: number;
}

interface StickyAnnotation extends AnnotationBase {
  type: "sticky";
  text: string;
  bgColor: string;
  width?: number;
  height?: number;
  operator?: "AND" | "OR" | "BUT" | "BECAUSE";
  sourceFile?: string;     // App Bridge
}

interface ArrowAnnotation extends AnnotationBase {
  type: "arrow";
  x2: number;
  y2: number;
  arrowType?: "straight" | "curved";
  arrowBidirectional?: boolean;
  predicate?: ArrowPredicate;
  strokeWidth?: number;
  waypoints?: { x: number; y: number }[];
  sourceId?: string;
  targetId?: string;
  sourceBlockId?: string;
  targetBlockId?: string;
  sourceTextSel?: string;
  targetTextSel?: string;
  longText?: string;
  targetBoardId?: string;
  pinned?: boolean;
}

interface MembraneAnnotation extends AnnotationBase {
  type: "membrane";
  width: number;
  height: number;
  bgColor: string;
}

export type Annotation =
  | TextAnnotation
  | StickyAnnotation
  | ArrowAnnotation
  | MembraneAnnotation;
```

Effet : tout consommateur fait un `switch(ann.type)` et bénéficie du
narrowing TypeScript — fin des `ann.x2!` et `ann.predicate ?? ...`.

### R-TYP-02 — Introduire `Block` (atome RAG)

Aujourd'hui un nœud texte stocke une seule string `text`. Un sticky aussi.
Pour la Phase 2 RAG, on veut adresser un **paragraphe précis** (déjà prévu
par `sourceBlockId` côté flèche, mais pas représenté côté données).

**Nouveau type** :

```ts
// types/index.ts (Phase 2)

/** Atome indexable côté RAG : un paragraphe, une légende, un caption
 *  d'image, une cellule de tableau, une transcription de section vidéo… */
export interface Block {
  id: string;                 // nanoid stable même si parent change
  parentId: string;           // id de l'image, annotation, folder, board
  parentKind: "image" | "annotation" | "folder" | "board";
  order: number;              // position relative dans le parent
  kind: "text" | "caption" | "transcript" | "table" | "code" | "section";
  content: string;            // texte brut indexable
  meta?: Record<string, string>; // ex: {lang, codecLang, transcriptStart}
  /** Vecteur RAG calculé asynchronement et stocké en cache disque,
   *  jamais dans le doc Automerge (cf. R-FRO-02). */
}
```

Le **TextAnnotation** et le **StickyAnnotation** gardent leur `text` brut
pour l'édition ; les blocks sont *dérivés* (parsing Markdown → liste de
paragraphes) à l'enregistrement. Source de vérité = doc Automerge ; cache
de blocks = matérialisation idempotente.

### R-TYP-03 — Schéma Zod aligné

`projectSchema.ts` doit refléter les unions discriminées (Zod
`z.discriminatedUnion`) et la nouvelle structure Block. La compatibilité
ascendante v1 reste assurée par le branchement legacy de `loadProject`.

## 2.3 Frontière — séparer moteur, plugin, adaptateur

### R-FRO-01 — Extraire un dossier `engine/`

Aujourd'hui, le store mélange CRDT, undo/redo, et logique métier. Pour
la Phase 2, on veut un **moteur réutilisable** capable de servir un futur
serveur (CLI headless, batch processing du wiki).

```
src/engine/
├── doc.ts          ── re-export typé d'Automerge (notre frontière)
├── mutate.ts       ── helper pur : mutate(doc, msg, fn) → newDoc
├── cascade.ts      ── purgeOrphanArrows, purgeOrphanMirrors,
│                      purgeOrphanChildBoards (extraits du store)
├── selection.ts    ── opérations pures sur sélections (move, duplicate)
├── geometry.ts     ── clamp, bbox, hit-test, snap
└── temporal.ts     ── (déjà ~ src/utils/timeline.ts → renommer)
```

Le store Zustand devient alors une *coque* React-friendly qui appelle
`engine/` ; le moteur reste utilisable côté tests Node.js et côté CLI
d'indexation Wiki (Étape 3 — `glucose-rag index`).

### R-FRO-02 — Séparer doc Automerge ↔ artefacts dérivés

Règle d'or : **le doc Automerge contient uniquement la donnée
authoritative produite par l'utilisateur**. Tout ce qui est *dérivable* —
embeddings, cluster cache, layout cache, thumbnails — vit hors du doc,
soit en mémoire, soit dans `app_data_dir/cache/`.

Pourquoi : Automerge stocke l'historique. Mettre des embeddings (768 floats
× milliers de nœuds) dans l'historique pourrirait la taille du `.glucose`
en quelques heures.

Cible :

```
app_data_dir/
├── assets/                    ── déjà existant (Phase 7.0)
└── cache/
    ├── embeddings/<project>/<docId>.bin   ← index vectoriel par projet
    ├── thumbs/<assetHash>.webp            ← thumbnails 256×256
    └── llm/<project>/conversations.jsonl  ← historique commandes RAG
```

### R-FRO-03 — Bus d'événements typé

`glucoseBus.ts` aujourd'hui dispatche des CustomEvent string. Pour
Phase 2 (plugins IA), un bus typé évite les `event.detail as any`.

```ts
// src/utils/bus.ts
export interface BusEvents {
  "teleport-to-mirror-original": { id: string };
  "portal-jump": { boardId: string; targetId: string };
  "fit-view": void;
  "delete-selected-folder": void;
  "zone-selected": { x0: number; y0: number; x1: number; y1: number };
  // Phase 2 :
  "rag-query": { query: string; scope: "board" | "project" | "wiki" };
  "rag-result": { results: BlockHit[] };
  "ai-command": { intent: string; payload: unknown };
}

export const bus = createTypedBus<BusEvents>();
bus.on("rag-query", ({ query, scope }) => { /* typed */ });
```

## 2.4 Hygiène mineure

### R-HYG-01 — Supprimer `pinned` legacy
Le champ `pinned?: boolean` sur `Annotation` est marqué « legacy Phase 2
LOD — sans effet en 7.5+ ». À enlever proprement (migration : strip à load).

### R-HYG-02 — Nettoyer les logs `[drop]`
Les logs ajoutés au bugfix drag-drop (Phase 7.5bis) sont en `console.log`
non gatés. Passer en `debug` ou retirer.

### R-HYG-03 — Migrer les `JSON.parse(JSON.stringify(...))`
~5 sites dans le store font ce clone pour échapper aux proxies Automerge.
Encapsuler dans un helper `clonePlain<T>(x): T` dans `engine/mutate.ts`.

### R-HYG-04 — Constante centrale `LIMITS.UNDO_DEPTH`
Le store utilise `UNDO_DEPTH = 50` locale, alors que `constants.ts`
expose `LIMITS.UNDO_DEPTH = 50`. Doublon — référencer la constante.

### R-HYG-05 — Build logs
`build.log`, `build2.log`, `build3.log`, `build_final.log`, `dev.log`
trainent à la racine. Les ajouter au `.gitignore` et supprimer.

### R-HYG-06 — Tauri capabilities review
Confirmer que les capabilities listent strictement le set utilisé. Les
nouvelles commandes RAG (Phase 2) devront être ajoutées explicitement.

## 2.5 Tests à ajouter avant Phase 2

| Test | Couvre | Fichier cible |
|---|---|---|
| Slice imagesSlice | mutations isolées | `slices/imagesSlice.test.ts` |
| Slice foldersSlice | capture + cascade | `slices/foldersSlice.test.ts` |
| Cascade arrows orphelines | suppression d'un node lié | existant — étendre |
| Block derivation | parsing Markdown → Block[] | `engine/blocks.test.ts` |
| Bus typé | dispatch + listener typés | `utils/bus.test.ts` |

**Cible : 75+ tests verts** avant d'ouvrir la Phase 2.

## 2.6 Dette résiduelle Phase 1 (items `[ ]` de la roadmap)

> Tâches explicitement non cochées ou « reportées » dans `ROADMAP.md` à
> l'intérieur de phases déjà déclarées **complétées**. Elles sont à
> traiter dans la même fenêtre que les refactors R-MOD / R-TYP / R-FRO,
> avant l'ouverture de la Phase 2.0.

### R-RES-01 — BUG-5 : Notifications partielles (Phase 0)
**Manque** : toasts feedback pour `création dossier`, `ajout image`,
`import vidéo`, `application preset`. Aujourd'hui ces actions sont
silencieuses alors que copy/cut/duplicate notifient.

**Action** : ajouter `showToast(...)` aux call-sites concernés (store
actions `createFolder`, `addImage`, `applyPresetToBoard`, branche
`download_video`). Effort : **½ j**.

### R-RES-02 — Hook `useZoneSelector(callback)` réutilisable (Phase 1)
**Manque** : le sélecteur de zone est aujourd'hui câblé en dur dans
`GlucoseCanvas.tsx` (refs `zoneStartRef`, `zoneLabelRef`,
`zonePendingActionRef`). La roadmap l'avait reporté car « pas de nouveaux
appelants ». La Phase 2 RAG en aura : sélection d'une zone pour requête
spatiale (« range-moi tout ce qui est dans ce rectangle »).

**Action** : extraire dans `src/canvas/hooks/useZoneSelector.ts` (s'aligne
avec R-MOD-01). Effort : **1 j**.

### R-RES-03 — Poignées de redimensionnement post-création (Phase 1 + 4)
**Manque** : un folder ou une membrane manuelle créée par drag ne peut
plus être redimensionnée (il faut la supprimer et la recréer). Reporté
deux fois.

**Action** : composant `ResizeHandles.tsx` (8 poignées N/S/E/W/NE/NW/SE/SW)
en SVG overlay, appelle `updateFolder` / `updateAnnotation`. Effort : **2 j**.

### R-RES-04 — Brancher `Quadtree` sur le rendu (Phase 2 — OBSOLÈTE)
**Manque** : ticket initial pour gagner en perf de culling.
**Décision** : **fermer comme obsolète** depuis la Phase 7.5 (LOD retiré,
le `SpatialHash` actif sur les images suffit, pas de bénéfice mesurable
pour les annotations / flèches).

**Action** : supprimer la ligne du `ROADMAP.md` ou la déplacer en
« Idées futures ». Effort : **5 min**.

### R-RES-05 — Bouton Time Machine dans la Toolbar (Phase 7.4)
**Manque** : Ctrl+H ouvre la Time Machine, mais aucun affordance visible.
Un utilisateur découvrant l'app ignore que la feature existe.

**Action** : icône 🕒 dans `Toolbar.tsx`, signal d'état (bordure colorée
si historique > 0). Effort : **½ j**.

### R-RES-06 — Animation capture des blocs au drag-create folder (Phase 7.5.1)
**Manque** : aujourd'hui, les blocs capturés disparaissent instantanément
du parent et apparaissent dans le child board sans transition. Effet
« téléport invisible » → désorientant.

**Action** : animation 250 ms (cubic ease-in) qui fait converger
les blocs vers le centre du folder créé avant la mutation. Utilise un
`requestAnimationFrame` + sprites temporaires PixiJS détachés du sync.
Effort : **1 j**.

### R-RES-07 — Refonte membranes auto + manuelles (Phase 7.5.2)
**Manque** : remarque utilisateur que la Phase 7.5 a explicitement
notée mais pas traitée. Les membranes implicites peuvent paraître
« sales » à grande échelle (hull trop agressif, couleur mal contrastée).

**Action** : audit visuel sur projet > 200 images. Pistes :
- adoucir le hull (chaikin smoothing ou Catmull-Rom)
- ajouter une option « membrane manuelle » avec sommets éditables
- réduire `MEMBRANE_PAD` quand la cluster est dense.

Effort : **2 j**. Volontairement à traiter **après** R-TYP-02 (Block)
car les membranes Phase 2 devront aussi savoir contenir des blocks.

### R-RES-08 — Curseurs flottants temps réel multijoueur (Phase 7.5bis polish)
**Manque** : un peer connecté ne voit pas où l'autre clique / drag. Frein
fort à la collaboration vraie.

**Action** : nouveau type d'event WebSocket `mp:cursor { peerId, x, y,
boardId }` à throttle 20 Hz côté émetteur. Rendu HTML overlay (curseur
SVG coloré par peer + nom). N'est PAS dans le doc Automerge (éphémère).
Effort : **1,5 j**.

### R-RES-09 — Reconnexion automatique WebSocket (Phase 7.5bis polish)
**Manque** : si un peer perd la connexion (Wi-Fi qui flotte, sleep), la
session est perdue.

**Action** : backoff exponentiel côté Rust dans `multiplayer.rs`
(`retry 1s, 2s, 4s, 8s, plafonné à 30s`). Tag visuel « reconnexion… »
dans `MultiplayerPanel`. Effort : **1 j**.

### R-RES-10 — Chiffrement TLS du LAN (Phase 7.5bis polish)
**Manque** : aujourd'hui WebSocket en clair (`ws://`). Acceptable LAN
privé, mais bloquant pour usage dans des contextes mixtes (espace
co-working, conférence Wi-Fi).

**Décision recommandée** : **différer Phase 11** (Cloud Sync). Pas un
prérequis Phase 2. À documenter comme limitation connue.

### R-RES-11 — Bugs App Bridge `.blend` / `.psd` / `.kra` (priorité HAUTE)
**Manque (deux bugs liés)** :
1. Les fichiers source créatifs s'affichent comme sticky vide, nom invisible.
2. Double-clic ne lance pas l'app native (`open_in_app` Rust ne trouve
   pas l'association ou chemin incorrect).

**Action** :
- côté Rust : log structuré du chemin reçu, vérifier la résolution
  d'association via le registre Windows (`assoc .blend` / `ftype`).
- côté front : `AppBridgeIcon` + `HtmlAnnotationLayer` pour les sticky
  `sourceFile` — sticky doit afficher le nom du fichier en gras + icône
  logiciel + ligne « double-clic pour ouvrir ».
- test E2E manuel : Blender installé sur Windows, `.blend` posé sur
  canvas, double-clic → Blender s'ouvre sur le fichier.

Effort : **2 j**. **Bloquant pour la crédibilité de l'App Bridge** —
à traiter en priorité.

### Récap dette résiduelle

| Ticket | Type | Effort | Priorité |
|---|---|---|---|
| R-RES-01 | Polish UX | ½ j | Basse |
| R-RES-02 | Refactor (s'aligne R-MOD-01) | 1 j | Moyenne |
| R-RES-03 | Feature manquante | 2 j | Moyenne |
| R-RES-04 | Cleanup roadmap | 5 min | Basse |
| R-RES-05 | Polish UX | ½ j | Moyenne |
| R-RES-06 | Polish UX | 1 j | Moyenne |
| R-RES-07 | Refonte visuelle | 2 j | Moyenne (post R-TYP-02) |
| R-RES-08 | Feature multijoueur | 1,5 j | Basse (peut différer post-Phase 2) |
| R-RES-09 | Robustesse réseau | 1 j | Moyenne |
| R-RES-10 | Sécurité réseau | — | Différé Phase 11 |
| R-RES-11 | Bug App Bridge | 2 j | **Haute** |

**Total dette résiduelle : ~11,5 jours** à insérer dans la fenêtre de
refactoring (Étape 2), ce qui porte le total préparatoire à
**~25-30 jours** avant ouverture Phase 2.0.

---

## 2.6bis — MISE À JOUR mai 2026 : Glucose = file manager visuel 2D

> Décisions utilisateur du 2026-05-29 qui transforment la portée de
> Glucose : ce n'est plus juste un canvas cognitif, c'est aussi un
> **explorateur de fichiers spatial** auto-portant. Ces tickets sont
> insérés dans la fenêtre Sprint 2 — à traiter **avant** Phase 2.0 RAG
> car ils touchent au format `.glucose` lui-même.

### R-EMB-01 — Embedding réel des assets dans le `.glucose` 🔴 PRIORITÉ ABSOLUE

**Problème (verbatim utilisateur)** : « je detest le fait que les fichier
image soit enregistrer en relatif et pas en dure dans le glucose lui meme ».

**État actuel (Phase 7.0)** : assets externalisés vers
`app_data_dir/assets/<sha256>.<ext>`, le `.glucose` ne contient qu'une
référence `asset:<sha>.png`. Copier le `.glucose` sur une autre machine
SANS copier le dossier `assets/` casse toutes les images.

**Décision** : passer à un modèle **dual** avec embed par défaut :

```ts
// types/index.ts — Phase 2
type AssetRef =
  | { kind: "inline"; data: Uint8Array; mime: string; sha256: string }
  | { kind: "linked"; path: string; sha256?: string; sizeBytes?: number };

interface BoardImage {
  // ... champs existants
  asset: AssetRef;       // remplace `src: string`
  // legacy `src` strippé à load + migré
}
```

| Cas | Mode par défaut | Override utilisateur |
|---|---|---|
| Image collée / drag depuis web | `inline` | Right-click → "Lier au fichier" |
| Image drag depuis disque < 5 MB | `inline` | Right-click → "Lier au fichier" |
| Image drag depuis disque ≥ 5 MB | `linked` (chemin absolu) | Right-click → "Intégrer dans le projet" |
| Folder-mirror (cf. R-FIL-02) | `linked` (chemin relatif au manifest) | Pas d'override |

**Implications techniques** :
- Automerge supporte `Uint8Array` natif via `A.Bytes`. La dédup par
  contenu se fait au niveau Automerge (un même blob référencé N fois =
  stocké 1×) → pas de duplication même avec inline.
- L'historique Automerge garde les blobs ; pour un projet créatif typique
  (50 images × 500 KB = 25 MB), c'est OK. Pour Wiki Test (10M images),
  on reste en `linked` + cache sidecar.
- Migration : `loadProject` détecte les anciens `src: "asset:..."` et
  les rapatrie automatiquement en `inline` au premier save (idempotent,
  silencieux).

**Effort** : **3-4 j** (changement de schéma + migration + UI right-click +
tests roundtrip).

**Priorité : HAUTE** — bloque la confiance de l'utilisateur dans le format.

---

### R-FIL-01 — Drag-and-drop universel de fichiers

**Problème (verbatim)** : « il y a un truck qui existe toujours pas dans
le glucose ces le fait de glisser deposer nimporte quel fichier et que on
puisse le lire. tout les .json les .txt et les .md si on glisse depo sur
glucose et bien on le vois sous forme de texte. et si il y a autre type
de fichier que on ne peut pas lire et editer directement et bien il
faudrai que ce soit un launcher genre les .blend tu clique depuis
glucose sa te louvre sur blender ».

**Aujourd'hui** : seuls les fichiers image / vidéo / `.glucose` sont
reconnus. Tout autre fichier est ignoré au drop.

**Nouveau type unifié** :

```ts
// types/index.ts — Phase 2
interface FileNode extends AnnotationBase {
  type: "file";
  asset: AssetRef;
  filename: string;
  mime: string;
  sizeBytes: number;
  /** Détermine le rendu sur le canvas. */
  view: FileView;
}

type FileView =
  | { kind: "text-inline"; content: string; lang?: string }       // .txt/.md/.json/.log/.csv
  | { kind: "code-inline"; content: string; lang: string }        // .ts/.js/.py/.rs/.go/...
  | { kind: "markdown-inline"; content: string }                  // .md (rendu = TextAnnotation)
  | { kind: "image-inline" }                                      // displaye via BoardImage
  | { kind: "launcher"; icon: string; tooltip: string }           // .blend/.kra/.psd/.nuke/...
  | { kind: "unsupported"; reason: string };                      // fallback safe
```

**Matrice de détection** (extension → vue par défaut) :

| Extensions | Vue | Rendu |
|---|---|---|
| `.txt .log .csv .json .yaml .toml .ini .env .gitignore .md` | text-inline / markdown-inline | Bloc texte read-only avec scroll vertical interne + bouton "Éditer" |
| `.ts .tsx .js .jsx .py .rs .go .c .cpp .java .rb .php .sh .ps1 .sql` | code-inline | Bloc code coloré (Shiki ou Prism — décision Sprint 3) |
| `.png .jpg .jpeg .gif .webp .avif .svg .bmp` | image-inline | Existant — BoardImage |
| `.mp4 .webm .mov .mkv` | image-inline (vidéo) | Existant — BoardImage + `isVideo` |
| `.blend .kra .psd .nuke .ai .indd .clip .ma .mb .max .c4d` | launcher | Icône grand format + nom + "double-clic → ouvre dans l'app" |
| `.pdf` | launcher (Phase 3 future : preview pages) | Icône PDF |
| `.zip .tar .gz .7z .rar` | launcher | Icône archive — pas d'extraction inline (volume) |
| tout autre (`.exe .dll .so .dylib` etc.) | unsupported | Refus avec toast explicatif (sécurité : déjà bloqué côté Rust) |

**Comportements** :
- Drop d'un fichier sur le canvas → crée un `FileNode` à la position du
  curseur. Le contenu est embedé si < 5 MB (réutilise R-EMB-01).
- Double-clic sur launcher → `open_in_app` (déjà existant, vérifié par
  whitelist Rust).
- Double-clic sur text-inline / code-inline → ouvre un éditeur modal
  (lecture seule par défaut, bouton "Éditer" passe en write si le fichier
  est `linked` ; un `inline` peut être édité et resté inline).

**Effort** : **4-5 j** (lecture sécurisée côté Rust, détection MIME via
`infer` crate, intégration dropHandler, 3 viewers, tests par type).

---

### R-FIL-02 — Drag d'un dossier → folder-mirror visuel

**Problème (verbatim)** : « quand je dit un fichier ces que je parle d'un
dossier sa crée un dossier sur le glucose, et sa crée des lien pour tout
les fichier .blend, .krt .nuke bref tout. et comme sa permeterai avoir un
univers visuel 2D ultra organiser ou tu litteralement un visuelateur de
fichier avec les icon des fichier pour que ce soit visuel ».

**Concept** : Glucose devient un **explorateur Finder/Explorer immersif**.
Drag d'un dossier OS sur le canvas → un `CanvasFolder` est créé, son
contenu est scanné, chaque fichier devient un `FileNode` (R-FIL-01)
positionné en grille.

**Schéma du `CanvasFolder` étendu** :

```ts
interface CanvasFolder {
  // ... champs existants
  /** Si le folder est un miroir d'un dossier OS, on stocke la racine. */
  mirrorSource?: {
    rootPath: string;        // chemin absolu du dossier OS
    mode: "snapshot" | "live"; // live = re-sync sur changement disque
    lastScannedAt: number;
    pattern?: string;        // glob optionnel (ex: "*.blend")
    recursive: boolean;
  };
  sortMode?: SortMode;       // cf. R-FIL-03
}
```

**Modes de mirror** :

| Mode | Comportement | Coût |
|---|---|---|
| `snapshot` (défaut) | Scan une fois au drop, FileNodes statiques | Faible |
| `live` | Watch via Tauri `notify` + delta sync | Moyen — nouveaux fichiers ajoutés / supprimés en temps réel |

**Layout par défaut au scan** :
- Grille : `cols = ceil(sqrt(N))`, espacement = 200 px
- Chaque FileNode est positionné en (col × 220, row × 220) relatif au
  centre du folder.
- Surcharge possible via R-FIL-03 (sort + filter).

**Implications sécurité** :
- Pas de scan transitive hors du `mirrorSource.rootPath` (validate_scope
  Rust déjà strict).
- Refus des fichiers `.exe .dll .lnk .bat .ps1` etc. au scan
  (`FORBIDDEN_OPEN_EXTS` déjà existant).
- Limite : pas de mirror sur un dossier > 10 000 fichiers sans confirmation
  utilisateur (toast "10k+ fichiers, continuer ?").

**Effort** : **5-6 j** (scan Rust, watcher optionnel, rendu grille, intégration
R-FIL-01 par fichier, garde-fous quantité).

---

### R-FIL-03 — Tri et filtres multi-critères sur les folders

**Problème (verbatim)** : « je pense qu'il serai interressant de crée des
filtre de trie genre 'dernier fois que on a ouvert un fichier, derniere
modifier, trier de A à Z ou encors des double truck du style trier par
tipe de fichier et en me temps par ordre choronologique ou meme par
taille ».

**Schéma** :

```ts
type SortKey = "name" | "createdAt" | "modifiedAt" | "openedAt"
             | "size" | "type" | "color";
type SortDir = "asc" | "desc";

interface SortMode {
  primary: { key: SortKey; dir: SortDir };
  /** Tri secondaire si égalité sur primary (ex: type asc + date desc). */
  secondary?: { key: SortKey; dir: SortDir };
  /** Filtre optionnel (ne supprime pas les nœuds, les atténue). */
  filter?: {
    extensions?: string[];      // ["blend", "nuke"]
    dateRange?: { start: number; end: number };
    sizeRange?: { minBytes: number; maxBytes: number };
    query?: string;              // recherche textuelle dans filename
  };
}
```

**UI** :
- Bouton « ⇅ Tri » dans le breadcrumb (en haut du folder) → ouvre un
  petit popover compact :
  - Tri primaire (dropdown 8 options) + sens (toggle ↑↓)
  - Tri secondaire (optionnel)
  - Filtres (sous-section repliée par défaut)
- Le sort déplace les FileNodes via une animation 300 ms ease-out.
- Le filter atténue (opacity 0.18) les nœuds qui ne matchent pas — comme
  le filtre temporel existant (Phase 6).

**Présélections rapides** (boutons d'accès direct dans le popover) :
- « Récents d'abord » : `modifiedAt desc`
- « Par type puis chronologique » : `type asc + modifiedAt desc`
- « Du plus gros au plus petit » : `size desc`
- « A → Z » : `name asc`

**Persistance** : le `sortMode` est stocké dans le `CanvasFolder` (donc
dans le doc Automerge) → préservé après reload, sync via multijoueur LAN.

**Effort** : **3 j** (schéma, popover, application au layout, presets,
tests, animation).

---

### Récap nouveaux tickets R-EMB / R-FIL

| Ticket | Type | Effort | Priorité | Bloque |
|---|---|---|---|---|
| R-EMB-01 | Format `.glucose` | 3-4 j | **🔴 ABSOLUE** | toute confiance utilisateur dans le format |
| R-FIL-01 | Feature drop universel | 4-5 j | **HAUTE** | dépend de R-EMB-01 |
| R-FIL-02 | Feature folder-mirror | 5-6 j | **HAUTE** | dépend de R-FIL-01 |
| R-FIL-03 | Feature tri/filtre | 3 j | **MOYENNE** | dépend de R-FIL-02 |

**Total Sprint 2 (file manager) : 15-18 jours.**

---

### ✅ État d'implémentation (2026-05-31)

> Branche `fix/folder-310-and-ui-polish`. Le drag-drop a dû basculer en
> **natif Tauri** (`dragDropEnabled: true`) car WebView2 ne donne le chemin
> absolu d'un fichier déposé QUE dans ce mode (cf. R-DND-FORK ci-dessous).

| Ticket | Statut | Détail |
|---|---|---|
| R-EMB-01 | ✅ Fait | embed Automerge `blobs[sha256]`, migration auto |
| R-FIL-01 | ✅ Fait | drop texte/code → annotation inline (natif: `read_text_file_inline`) |
| R-FIL-02 v1 | ✅ Fait | drop dossier → folder mirror plat |
| R-FIL-02 v2 | ✅ Fait | `scan_tree` récursif borné (5000 entrées / depth 8), sous-dossiers **navigables** (`createFolderTree`), fichiers = launchers icônés |
| Double-clic launch | ✅ Fait | `open_in_app` passé en **deny-list** (ouvre tout sauf exécutables) ; scan masque le bruit binaire mais garde les sources visibles |
| Icônes | ✅ Partiel | `AppBridgeIcon` badges par extension (Blender, Nuke, InDesign, Houdini…). Logos OS réels → R-FIL-ICON-OS |
| R-FIL-03 | 🟡 Logique faite | tri (`name/type/size/modified`, dossiers d'abord) appliqué au scan. **Reste : UI dropdown + re-scan** |

### R-DND-FORK — Smart drop target wry (drag web + drag fichier simultanés)

**Problème** : sur Windows/WebView2, `dragDropEnabled` est binaire — soit
les chemins de fichiers OS (natif), soit le drag HTML5 (contenu web), jamais
les deux (confirmé mainteneurs Tauri : issues #9830, #8581, discussion #9696).
On a choisi **natif** (fichiers/dossiers = cœur du file manager) ; le drag
d'image depuis un navigateur bascule sur **Ctrl+V** (déjà fonctionnel).

**Solution future** : patcher `wry` via `[patch.crates-io]` (ne touche PAS
l'app) avec un `IDropTarget` Windows « intelligent » qui inspecte le format
OLE du drag : `CF_HDROP` → chemins ; `text/html`/`uri-list`/bitmap → URL/bytes.
Tout passerait par un seul canal natif, drag web inclus.

**Effort** : 3-5 j (COM/OLE bas niveau en Rust) + dette de maintenance du fork.
**Priorité** : BASSE (Ctrl+V couvre le besoin). À déclencher en Phase 3 si le
workflow Ctrl+V devient gênant.

### R-FIL-ICON-OS — Logos d'application réels (icônes système)

**Problème (verbatim)** : « récuperer leur logo pour pouvoir les afficher ».
Actuellement `AppBridgeIcon` rend des badges typographiques par extension.
L'utilisateur veut idéalement le **vrai logo** de l'app associée (icône Blender,
Photoshop… extraite de l'OS).

**Solution** : commande Rust `get_file_icon(path) -> base64 PNG`. Windows :
`SHGetFileInfo` + `SHGFI_ICON` (crate `windows`). macOS : `NSWorkspace
iconForFile`. Linux : thème d'icônes XDG. Cache par extension (`app_data/
icon-cache/<ext>.png`).

**Effort** : 3-4 j (API système par OS, encodage HICON→PNG, cache).
**Priorité** : MOYENNE (les badges actuels sont déjà lisibles et cross-OS).

### R-IMG-SRC — Import web intelligent : upscale + attribution auteur

**Problème (verbatim 2026-05-31)** : « si on fait un glisser deposer [d'image
depuis le web] : 1. rechercher sur tout le web si il existerait la même image
en meilleure qualité → télécharger automatiquement la meilleure qualité ;
2. pouvoir citer l'auteur, le nom qu'il a donné à l'image, et clic droit →
accéder à la page du créateur + ses réseaux. »

**Contexte** : depuis le passage en drag-drop natif, l'import web passe par
Ctrl+V (URL ou bytes). C'est le bon moment pour enrichir CE chemin plutôt que
le drag. L'utilisateur dit explicitement « clairement pas pour tout de suite ».

**Pistes techniques** :
- **Upscale/meilleure source** : on a DÉJÀ `getCDNCandidates` (imageUpgrade.ts)
  qui tente les variantes haute-réso d'un même CDN. Étendre avec une vraie
  **recherche d'image inversée** (TinEye API, Google Lens non-officiel, ou
  Yandex) pour trouver la même image ailleurs en plus grand. Choix de la plus
  grande résolution réelle (pas juste upscalée).
- **Attribution** : extraire depuis la page d'origine les meta `og:`,
  `author`, microdata `schema.org/ImageObject` (creator, license, name) +
  liens réseaux (`rel=me`, profils détectés). Stocker dans `BoardImage` :
  `{ author?, authorUrl?, authorSocials?: string[], license?, title? }`.
- **UI clic droit** : menu contextuel sur l'image → « Voir le créateur »,
  « Réseaux », « Licence », « Source originale ».

**Effort** : 5-7 j (API recherche inversée + scraping attribution + schéma +
menu contextuel). Dépend d'une clé API tierce pour la recherche inversée.
**Priorité** : BASSE (idée long terme, post-Phase 2).

**Ordre d'exécution recommandé** :
1. R-EMB-01 (fondation format) →
2. R-FIL-01 (atome FileNode + rendu) →
3. R-FIL-02 (folder-mirror utilise FileNode) →
4. R-FIL-03 (tri/filtre par-dessus).

Le Sprint 2 se fait **après** R-TYP-02 (Block) car `FileNode` et `Block`
partagent des concepts (atome indexable, contenu typé, sourceFile). Il
est même judicieux de fusionner les deux : un `FileNode` est un `Block`
de `kind: "file"`. À trancher au début Sprint 2.

---

## 2.7 Récap consolidé des tickets

| Ticket | Type | Effort | Priorité |
|---|---|---|---|
| R-MOD-01 | Refactor canvas | L (3-4 j) | Haute |
| R-MOD-02 | Refactor store | L (3-4 j) | Haute |
| R-MOD-03 | Suppression AnnotationLayer.ts | S (½ j) | Moyenne |
| R-TYP-01 | Union discriminée Annotation | M (1-2 j) | Haute |
| R-TYP-02 | Type `Block` + dérivation | M (2 j) | Haute (prérequis RAG) |
| R-TYP-03 | Zod aligné | S (½ j) | Moyenne |
| R-FRO-01 | Dossier `engine/` | M (2 j) | Haute (prérequis CLI Wiki) |
| R-FRO-02 | Cache hors doc | S (1 j) | Haute (prérequis embeddings) |
| R-FRO-03 | Bus typé | S (1 j) | Moyenne |
| R-HYG-01..06 | Hygiène | S (½ j cumulé) | Basse |

**Total refactoring (R-MOD / R-TYP / R-FRO / R-HYG) : 14-18 jours**.
**Total file manager (R-EMB / R-FIL, cf. §2.6bis) : 15-18 jours**.
**Cumulé avec la dette résiduelle Phase 1 (cf. §2.6) : ~40-46 jours** avant
l'ouverture Phase 2.0.

### Ordre d'exécution global (Sprints 2 à 5)

| Sprint | Contenu | Durée |
|---|---|---|
| **Sprint 1** ✅ | R-TYP-01 (union discriminée), 9 bugs Automerge, FolderBreadcrumb #310, polish UI Toolbar/Pomodoro | terminé |
| **Sprint 2 — Format & file manager** | R-EMB-01 → R-FIL-01 → R-FIL-02 → R-FIL-03 | 15-18 j |
| **Sprint 3 — Type Block + dérivation** | R-TYP-02 (peut fusionner avec FileNode), R-TYP-03 (Zod) | 2-3 j |
| **Sprint 4 — Modularité** | R-MOD-01 (canvas), R-MOD-02 (store), R-MOD-03 (suppr AnnotationLayer) | 7-9 j |
| **Sprint 5 — Frontière & dette résiduelle** | R-FRO-01/02/03 + dette R-RES-01..11 (sauf R-RES-10 différé) | 14-16 j |
| **Phase 2.0** | RAG fondations | 3 sem |

**Phase 2.0 atteignable autour de fin août 2026** dans ce calendrier.

---

# Étape 3 — Feuille de route Phase 2 : RAG Universel + Test du Wiki

> **Objectif Phase 2 :** repousser toutes les limitations actuelles.
>
> Deux piliers :
> 1. **RAG Universel** — un système RAG modulaire et **agnostique du modèle**
>    (local OU cloud, OpenAI / Anthropic / Mistral / Ollama / Llamafile).
> 2. **Test du Wiki** — l'intégralité du wiki anglais (≈ 6,9 M articles)
>    dans un seul `.glucose`, avec organisation hiérarchique et spatiale
>    évidente.
>
> Le second pilier sert de **banc d'essai** au premier : si Glucose tient
> le wiki, il tient tout.

## 3.1 Cahier des charges

### 3.1.1 Le RAG doit être Universel

| Dimension | Exigence |
|---|---|
| **Agnostique modèle** | Adapter pour Ollama (local), llama.cpp (local), OpenAI API, Anthropic API, Mistral API, n'importe quel endpoint OpenAI-compatible (LM Studio, vLLM, llamafile, Groq). Un seul fichier `.glucose` peut être interrogé par n'importe quel modèle sans réindexation. |
| **Agnostique embeddings** | Pluggable : sentence-transformers via candle Rust (local, défaut), `nomic-embed-text` via Ollama, OpenAI `text-embedding-3-*`, BGE, Cohere. Le format vectoriel est neutre (`f32[d]`). |
| **Hybride dense + sparse** | Score = α × cosine_dense + β × BM25 + γ × bonus_spatial. La proximité dans le canvas est un signal de pertinence. |
| **Multimodal** | Texte + images. Embeddings d'images via CLIP (local) ou via API multimodale. Recherche cross-modale (texte → images, image → textes). |
| **Streaming** | Réponses LLM streamées. UI affiche le chunk au fur et à mesure. Annulation par Échap propre. |
| **Privacy-first** | Mode 100% local par défaut. Toute sortie réseau exige confirmation explicite + URL whitelisté. Aucune donnée n'est envoyée à un endpoint cloud sans `--allow-cloud`. |
| **Évolutif** | Index reconstructible incrémentalement (un nouveau bloc → un nouveau vecteur). Pas de full-rebuild bloquant. |

### 3.1.2 Le RAG doit être Modulaire

```
┌──────────────────────────────────────────────────────────────────────────┐
│                       RAG Universel — pile en couches                    │
│                                                                          │
│  Layer 5 — UI         ┌─────────────────────────────────────────┐        │
│                       │ Command palette · Ask anything · Stream │        │
│                       └────────────────┬────────────────────────┘        │
│                                        │                                 │
│  Layer 4 — Orchestrator                ▼                                 │
│                       ┌─────────────────────────────────────────┐        │
│                       │  RAGOrchestrator                        │        │
│                       │   plan(query) → tools[] → llm → answer  │        │
│                       └─────┬───────────┬────────────┬──────────┘        │
│                             │           │            │                   │
│  Layer 3 — Tools           ▼           ▼            ▼                    │
│           ┌──────────────────┐  ┌──────────────┐  ┌──────────────┐       │
│           │ HybridRetriever  │  │ SpatialQuery │  │  LLMAdapter  │       │
│           │  dense+sparse+   │  │  in folder,  │  │  Ollama·OAI· │       │
│           │  cluster         │  │  near id     │  │  Anthropic   │       │
│           └────────┬─────────┘  └──────┬───────┘  └──────┬───────┘       │
│                    │                   │                 │               │
│  Layer 2 — Stores  ▼                   ▼                 ▼               │
│           ┌──────────────────┐  ┌──────────────┐  ┌──────────────┐       │
│           │  VectorStore     │  │ SpatialIndex │  │ ModelRegistry│       │
│           │  HNSW · IVF      │  │  Quadtree    │  │ providers[]  │       │
│           └────────┬─────────┘  └──────┬───────┘  └──────┬───────┘       │
│                    │                   │                 │               │
│  Layer 1 — Engine  ▼                   ▼                 ▼               │
│           ┌──────────────────────────────────────────────────────┐       │
│           │  Block Indexer (Rust)                                │       │
│           │    parse Markdown → Block[] → embed → store          │       │
│           │  CLIP Image Indexer                                  │       │
│           │  Watchdog (Automerge change → re-index delta)        │       │
│           └──────────────────────────────────────────────────────┘       │
└──────────────────────────────────────────────────────────────────────────┘
```

### 3.1.3 Le Test du Wiki

**Énoncé :**
*« L'intégralité du wiki anglais doit pouvoir être contenue dans un seul
glucose, avec une organisation hiérarchique et spatiale si évidente que
n'importe quel utilisateur peut s'y retrouver instantanément. »*

**Ce que ça implique en chiffres** (Wikipedia EN, snapshot 2026) :
- ≈ 6,9 M articles, ≈ 24 GB de texte brut compressé.
- ≈ 1 milliard de liens internes (mediawiki wikilinks).
- Catégories : ≈ 2 M catégories hiérarchisées (DAG, pas un arbre pur).
- Images (Commons subset référencé) : ≈ 10 M.

**Implications techniques** :
- Un seul doc Automerge à cette échelle dépasse les limites raisonnables
  d'un binaire (multi-GB). **Décision architecturale** :
  → **adopter un sharding** transparent par `super-folder thématique`
  (une racine = une dizaine de méga-folders par grand champ : Sciences,
  Histoire, Arts, Géographie, …) ; chaque super-folder est un sous-doc
  Automerge chargé à la demande.
  → **`.glucose` devient un dossier** (extension préservée) : un manifest
  racine + N sous-docs lazy-loadés. Compat ascendante : un mono-fichier
  reste valide.
- L'organisation hiérarchique vient des **catégories Wikipedia** + des
  **domaines Glucose** déjà existants (Phase 3). On précompute la
  hiérarchie au moment de l'import.
- L'organisation spatiale vient d'un **layout 2D dérivé des embeddings**
  (UMAP / t-SNE batchés au moment de l'index). Chaque article reçoit un
  `(x, y)` cohérent au sein de son super-folder.

## 3.2 Découpage en sous-phases

### Phase 2.0 — Fondations RAG (3 semaines)

**Objectif :** infrastructure technique sans laquelle rien d'autre n'est
possible. Aucune UI utilisateur à ce stade.

| Item | Détail |
|---|---|
| **Block indexer Rust** | Crate `glucose-rag` : parse Markdown (`pulldown-cmark`) → liste de `Block` (paragraphes, sections, code blocks, captions). Idempotent par contenu hashé. |
| **VectorStore embarqué** | HNSW via crate `hora` ou `instant-distance`. Persistance disque dans `app_data_dir/cache/embeddings/<docId>/`. Format : un fichier par super-folder. |
| **Embeddings local par défaut** | `candle-core` + modèle `sentence-transformers/all-MiniLM-L6-v2` (90 MB, 384 dims). Téléchargement vérifié SHA-256 comme yt-dlp (Sprint 1 pattern). Fallback `sentence-transformers/all-mpnet-base-v2` (420 MB, 768 dims) si l'utilisateur veut plus de qualité. |
| **Sparse index BM25** | Crate `tantivy` (full-text Lucene-like). Sert pour les requêtes lexicales pures (noms propres, codes). |
| **ModelRegistry** | `src/rag/providers/*.ts` : un fichier par provider (`local-ollama.ts`, `openai.ts`, `anthropic.ts`, `mistral.ts`, `llamacpp.ts`, `lm-studio.ts`). Interface commune : `embed(text) → Float32Array` et `chat({system,messages,stream}) → AsyncIterable<string>`. |
| **Settings panel** | Nouveau panel `RAGSettingsPanel` (raccourci `Ctrl+,`) — choix du provider + modèle + clé API stockée OS-keychain (Tauri 2 `tauri-plugin-store` ou `keyring-rs`). |
| **Watchdog** | Worker Rust qui s'abonne aux changes Automerge (`getChanges` après applyChanges). À chaque commit : parser les blocs touchés, ré-embedder, mettre à jour le VectorStore. Idempotent + résilient au crash (write-ahead log léger). |

**Livrables Phase 2.0** :
- `cargo run -p glucose-rag-cli index --doc <file.glucose>` produit
  un index sur disque.
- 5 providers connectables.
- Aucun changement UI visible — on prépare le terrain.

### Phase 2.1 — Recherche hybride & spatiale (2 semaines)

| Item | Détail |
|---|---|
| **HybridRetriever** | Combine dense (HNSW) + sparse (tantivy) + bonus spatial (proximité dans le board courant ou même folder = +score). Pondérations par défaut α=0.6, β=0.3, γ=0.1, configurables. |
| **SpatialQuery** | API : `findIn({folder, board, radius, fromNodeId})`. Utilise `Quadtree.ts` existant côté front + index spatial côté Rust si requête distante. |
| **Cluster retrieval** | « Donne-moi tout ce qui forme un cluster autour de cette idée » → renvoie une membrane virtuelle (groupe d'IDs). Réutilise `MembraneRenderer` pour highlight. |
| **API frontend** | `src/rag/client.ts` expose `search(query, opts) → Promise<Hit[]>` qui dispatche vers le backend Rust via Tauri commands. |

**Critère de réussite** : sur un projet de 1 000 nœuds, latence p95 < 50 ms
pour une recherche hybride end-to-end (incluant l'aller-retour Tauri).

### Phase 2.2 — Orchestrator + Streaming LLM (2 semaines)

| Item | Détail |
|---|---|
| **RAGOrchestrator** | Boucle de raisonnement minimale : (1) reformulation query, (2) retrieval, (3) rerank (cross-encoder ou LLM-as-judge léger), (4) génération avec citations. |
| **Streaming UI** | `CommandPalette` (raccourci `Ctrl+K`) — input + résultats streamés. Chaque chunk LLM est appliqué dans un `<div>` dédié, sans re-render React. Échap = abort propre (le provider expose un AbortController). |
| **Citations cliquables** | La réponse contient des `[B-1]`, `[B-2]` qui sont des liens vers les blocks. Click → teleport viewport + flash highlight 600 ms. |
| **Garde-fou** | Pas d'envoi cloud sans confirmation explicite par projet. Un toggle visible « Local · Cloud » dans la palette. Le mode actif est stocké côté projet (méta-objet hors doc Automerge) + override par session. |
| **Historique** | `cache/llm/<project>/conversations.jsonl` — historique searchable et rejouable. Pas dans le doc Automerge (taille). |

### Phase 2.3 — Multimodal (2 semaines)

| Item | Détail |
|---|---|
| **CLIP local** | `candle-core` + CLIP ViT-B/32 (150 MB, embeddings 512 dims). Embedde les images au moment où elles sont ajoutées (`addImage` → enqueue indexer). |
| **Recherche par image** | Drag d'une image sur la palette → top-K voisins visuels (cosine sur embeddings CLIP). |
| **Cross-modale** | `« photos de cathédrales gothiques »` → texte → embed → recherche dans index CLIP des images. |
| **Captions auto (option)** | Moondream2 quantizé Q4 (≈ 1,8 GB) téléchargeable on-demand. Génère un caption qui devient un Block lié à l'image. **Optionnel** car gros download. |

### Phase 2.4 — Le Test du Wiki (4 semaines)

> *Ici on valide que tout ce qu'on vient de bâtir tient à l'échelle réelle.*

#### Phase 2.4.A — Importeur Wikipedia (1 semaine)

```
src-tauri/src/import/
└── wiki.rs
   ─ parse_dump(path: &Path) → Iterator<Article>
   ─ pulldown-cmark / parse-wiki-text-2 sur le wikitext
   ─ extrait : title, body Markdown-like, catégories, infobox, refs
```

CLI : `cargo run -p glucose-import-wiki -- --dump enwiki-latest.xml.bz2
--out wiki.glucose --parallel 8 --shard-by category-root`.

Sortie : **un dossier `wiki.glucose/`** contenant :
- `manifest.json` (versioning + index des super-folders)
- `core.automerge` (la racine + 10 super-folders thématiques)
- `shards/<super-folder-id>.automerge` (un par grand champ)
- `cache/embeddings/` (HNSW + BM25 par shard)

#### Phase 2.4.B — Sharding transparent (1,5 semaine)

| Item | Détail |
|---|---|
| **Manifest** | `manifest.json` : `{ version: 3, shards: [{id, name, articleCount, sizeBytes, lazy: true}] }`. Tout shard a un `id` et une miniature 256×256 visible en macro-zoom. |
| **Lazy load** | Au boot, on charge `core.automerge` (ultra-léger, ~few MB). Les shards ne sont chargés qu'à l'entrée dans leur super-folder (ou si une recherche les cite). |
| **Mémoire bornée** | LRU sur les shards : max 3 shards résidents en mémoire en parallèle (≈ 1-3 GB chacun). Un 4ème éjecte le moins récemment utilisé. |
| **Pré-fetch heuristique** | En zoomant vers un super-folder, on déclenche le load 800 ms avant d'y entrer effectivement (`scale ≥ 1.5` sur le folder). |
| **Recherche cross-shard** | L'index BM25 + HNSW est partitionné par shard ; le HybridRetriever interroge chaque index actif en parallèle (`rayon`) puis fusionne par score. |

#### Phase 2.4.C — Layout spatial évident (1 semaine)

| Item | Détail |
|---|---|
| **Niveau MACRO (galaxie)** | Les 10 super-folders sont positionnés autour de l'origine selon un layout circulaire stable (sci/math/tech à gauche, arts/lettres au sud, sciences sociales à droite, géo en haut, histoire au centre). Bordures colorées dérivées des domaines. |
| **Niveau MÉSO (constellation)** | À l'intérieur d'un super-folder, les sous-catégories sont placées en force-directed UMAP des embeddings de leurs sommaires. Couleur = sous-domaine. |
| **Niveau MICRO (étoiles)** | Les articles sont des nœuds-texte titrés, placés selon UMAP local sur l'embedding du résumé. Cluster naturel par sujet. |
| **Wikilinks** | Les `[[...]]` deviennent des flèches `inspire` (prédicat par défaut) — invisibles par défaut grâce à la règle anti-spaghetti (Phase 2 historique). |
| **Recherche centrale** | Ctrl+K avec « Renaissance italienne » → top-K articles surlignés + une mini-carte du chemin spatial. |

#### Phase 2.4.D — Validation UX (½ semaine)

| Critère | Cible |
|---|---|
| Boot p95 (manifest + core) | < 800 ms |
| Entrée dans super-folder p95 | < 1,5 s (shard load + layout) |
| Recherche en langage naturel p95 | < 2 s end-to-end |
| Mémoire RAM stable à 60 min usage | ≤ 4 GB |
| Taille disque totale `wiki.glucose/` | < 80 GB (texte + embeddings + thumbs) |
| Découverte autonome d'un sujet par un nouvel utilisateur | < 30 s pour rejoindre « Mathématiques → Topologie » depuis le canvas vierge |

### Phase 2.5 — Ligne de commande visuelle (1 semaine, polish)

Réutilise tout le RAG existant pour des commandes structurées :
- « Glucose, range tout ce qui est bleu à gauche »
- « Glucose, crée un fork de cette idée en version cyberpunk »
- « Glucose, retrouve toutes mes références de forêts »

Implémentation : LLM → JSON DSL d'actions store
(`{action: "move", filter: {color: "blue"}, target: {x: -2000}}`) → exécuté
côté store. Sandboxé : seuls les verbes whitelistés sont permis.

## 3.3 Architecture cible Phase 2

```
src/
├── rag/                       ── NOUVEAU
│   ├── client.ts              ── façade frontend (Tauri invoke)
│   ├── orchestrator.ts        ── boucle plan→retrieve→rerank→generate
│   ├── retriever.ts           ── HybridRetriever (dense+sparse+spatial)
│   ├── providers/
│   │   ├── index.ts           ── ModelRegistry + base Provider
│   │   ├── localOllama.ts
│   │   ├── openai.ts
│   │   ├── anthropic.ts
│   │   ├── mistral.ts
│   │   ├── llamacpp.ts
│   │   └── lmStudio.ts
│   ├── CommandPalette.tsx     ── UI Ctrl+K
│   ├── ResultStream.tsx       ── streaming render + citations
│   └── RAGSettingsPanel.tsx   ── config (provider, modèle, clé OS keychain)
│
├── engine/                    ── NOUVEAU (cf. R-FRO-01)
│   ├── doc.ts · mutate.ts · cascade.ts · selection.ts · geometry.ts
│   └── blocks.ts              ── parsing Markdown → Block[]
│
├── shards/                    ── NOUVEAU
│   ├── manifest.ts            ── load/save manifest.json
│   ├── lazyLoader.ts          ── LRU + pré-fetch heuristique
│   └── crossShardQuery.ts     ── fusion de scores HNSW multi-index
│
└── (reste de Phase 1 raffiné cf. Étape 2)

src-tauri/src/
├── lib.rs                     ── commandes RAG ajoutées (whitelist capabilities)
├── multiplayer.rs             ── inchangé
├── rag/
│   ├── mod.rs
│   ├── indexer.rs             ── parse → blocks → embed → write VectorStore
│   ├── retriever.rs           ── HNSW + tantivy + spatial bonus
│   ├── providers.rs           ── HTTP clients pour OpenAI/Anthropic/Ollama
│   └── watchdog.rs            ── écoute changes Automerge → re-index delta
└── import/
    └── wiki.rs                ── importeur Wikipedia
```

## 3.4 Choix de modèles par défaut

| Tâche | Modèle local (défaut) | Modèle cloud (optionnel) |
|---|---|---|
| **Embeddings texte** | `all-MiniLM-L6-v2` (90 MB, 384d) | `text-embedding-3-small` (1536d) |
| **Embeddings image** | CLIP ViT-B/32 (150 MB, 512d) | CLIP via Replicate / OpenAI |
| **Re-rank** | bge-reranker-base (110 MB) | Cohere Rerank |
| **Chat** | Llama 3.2 3B Q4 via Ollama | Claude Haiku 4.5 / GPT-5-mini |
| **Captioning** | Moondream2 Q4 (1,8 GB, opt-in) | Claude Sonnet 4.6 vision |

Tous les téléchargements suivent le pattern Sprint 1 : version PINNED +
SHA-256 vérifié.

## 3.5 Décisions cadres

- **Le doc Automerge reste la source de vérité ;** les embeddings, BM25,
  caches LLM vivent en dehors. Un `.glucose` peut être ouvert sans son
  cache et reste fonctionnel — juste sans RAG instantané (l'index se
  reconstruira au premier boot).
- **Privacy par défaut :** mode local. Tout opt-in cloud est explicite,
  par projet, et logged.
- **Pas de plugin tiers en Phase 2.** Le système de providers est interne
  (~7 providers maintenus en interne). Les plugins externes attendront
  Phase 12 (idées futures).
- **Compat ascendante stricte :** un mono-fichier `.glucose` v2 reste
  lisible. Le format v3 (dossier sharding) est opt-in, déclenché par
  un import volumineux ou par menu « Convertir en projet sharded ».
- **Le Test du Wiki = critère de validation, pas feature livrable.** On
  ne distribue pas `wiki.glucose` (60+ GB). Le pipeline import est une
  CLI séparée pour les utilisateurs qui veulent. Mais on prouve que
  l'archi tient.

## 3.6 Risques techniques identifiés

| Risque | Probabilité | Mitigation |
|---|---|---|
| **HNSW lent à insérer** à 6M nœuds | Moyenne | Sharder par super-folder + insertion batchée hors-thread. |
| **Embeddings hors RAM** à 6M × 384 × 4 = 9 GB | Élevée | mmap'd file + LRU sur shards (max 3 résidents). |
| **Automerge sur shards de 1 GB** | Moyenne | Tester tôt sur shard prototype « Mathématiques » (200k articles). Si trop lent → tronquer l'historique des shards (un shard est lu mais quasi jamais édité). |
| **Latence cloud LLM** | Faible mais visible | Streaming dès le premier token ; barre de progression honnête. |
| **Clé API qui fuite** | Élevée si négligé | OS keychain obligatoire (Windows Credential Manager / macOS Keychain / libsecret). Jamais en clair sur disque. |
| **Coût cloud incontrôlé** | Moyenne | Budget mensuel paramétrable côté projet, soft-warn à 80 %, hard-stop à 100 %. |
| **Provider qui change son API** | Élevée long terme | Provider modules isolés, contrats interface stricts, tests d'intégration mockés. |

## 3.7 Critères de fin de Phase 2

La Phase 2 est **complète** quand :

1. **RAG Universel**
   - [ ] 5+ providers (local Ollama, OpenAI, Anthropic, Mistral, llama.cpp,
         LM Studio) interchangeables au runtime sans réindexation.
   - [ ] Recherche hybride < 100 ms p95 sur un projet de 10 000 blocks.
   - [ ] Streaming LLM avec citations cliquables qui téléportent.
   - [ ] Mode local 100 % fonctionnel sans réseau.
   - [ ] Multimodal texte ↔ images opérationnel.

2. **Test du Wiki**
   - [ ] Pipeline d'import Wikipedia EN complet et reproductible
         (`glucose-import-wiki`).
   - [ ] Boot du `wiki.glucose` (manifest + core) < 800 ms.
   - [ ] Navigation libre d'un super-folder à l'autre fluide à 60 FPS.
   - [ ] Une personne extérieure qui découvre le canvas trouve
         « Topologie algébrique » depuis la racine en moins de 30 s.
   - [ ] Mémoire RAM stable sous 4 GB après 1 h d'usage exploratoire.

3. **Bases architecturales saines**
   - [ ] Étape 2 (refactoring) bouclée à 100 % avant ouverture Phase 2.0.
   - [ ] Couverture tests ≥ 75 unit + 10 intégration.
   - [ ] Aucune régression UX vs. Phase 1 (manuel + automatisé).

---

## ✍️ Mot de la fin

Le projet est dans un état **bien meilleur que la moyenne** pour ouvrir
une phase ambitieuse : le CRDT comme source de vérité, la sécurité
prise au sérieux, le type system précurseur, les invariants de rendu
explicites. Les seules vraies dettes sont la taille de deux fichiers
clés (`GlucoseCanvas.tsx`, `store/index.ts`) — pas de la dette
conceptuelle, juste de la dette de modularité.

La Phase 2 ne demande **aucune refonte** des fondations. Elle ajoute :

- Une **couche RAG** indépendante (`src/rag`, `src-tauri/src/rag`).
- Un **système de sharding** transparent (`src/shards`).
- Un **engine extrait** réutilisable côté CLI (`src/engine`).
- Des **providers** modulaires pour rester agnostique.

C'est un escalier, pas un mur — comme l'auteur original l'a écrit.
