# Glucose — Roadmap & Architecture

> **Vision :** Une surface cognitive infinie. Une seule interface — pas de modes — capable de soutenir aussi bien la création d'un jeu vidéo, l'élaboration d'une langue construite, que la cartographie versionnée de toute la connaissance humaine (Xanadu 2026 / WikiGit).
>
> **Principe :** poser, relier, zoomer, explorer. Rien d'autre.

**Dernière mise à jour :** 2026-05-07
**Version :** 0.3.0-rc (Sprint 1 + Phases 6, 7.5, 7.0 + fondations 7.1 — voir [CLEANUP.md](CLEANUP.md), [PRE-PHASE7-AUDIT.md](PRE-PHASE7-AUDIT.md))
**Architecture :** Tauri 2 (Rust) + React 19 + Tailwind 4 + PixiJS 8 (raster) + SVG overlay (vecteur) + Zustand + **Automerge 3 (CRDT, WASM)**
**Archive de l'ancienne roadmap :** [ROADMAP.archive-2026-04-27.md](ROADMAP.archive-2026-04-27.md)

> **🛡️ Sprint 1 sécurité bouclé (2026-05-07)** — 9 vulnérabilités critiques fermées (RCE, XSS, SSRF, scope FS), validation Zod, clamps coords, SHA256 yt-dlp, README+LICENSE+CI+Biome.
>
> **📅 Phase 6 livrée (2026-05-07)** — Réglette temporelle sémantique : `temporalAnchor`, `TemporalRuler` zoomable (Shift+R), 30 époques nommées, parsing souple, filtrage live, badge 📅, ancrage par Shift+T.
>
> **🗂️ Phase 7.5 livrée (2026-05-07)** — Suppression intégrale du LOD. Refonte folders : design transparent, capture au drag-create, navigation par zoom (enter/exit), indicateur visuel, breadcrumb VSCode, preview améliorée.
>
> **🧹 Sprint 7.0 livré (2026-05-07)** — Pré-CRDT bouclé : assets externalisés (`asset:<hash>.<ext>` au lieu de base64), migration legacy automatique au load, cascades de suppression (folders / mirrors / portails), `id: nanoid()` pour le board par défaut. Rapport complet dans [PRE-PHASE7-AUDIT.md](PRE-PHASE7-AUDIT.md).
>
> **⚙️ Phase 7.1 fondations (2026-05-07)** — Automerge 3 installé, wrapper `src/store/automerge.ts` testé (12 tests verts : create/change/save/load roundtrip, merge commutatif, time machine `viewAt`/`getHeads`/`history`). Vite configuré pour WASM.
>
> **💾 Phase 7.2.A+B livrée (2026-05-07)** — Format `.glucose` v2 binaire Automerge opérationnel. Save/load avec détection automatique v1 (JSON legacy) / v2 (binaire). Migration transparente : un vieux `.glucose` JSON s'ouvre, et au prochain `Ctrl+S` il est ré-écrit en v2.
>
> **🧬 Phase 7.2.C livrée (2026-05-07)** — Store CRDT-first : le doc Automerge est désormais la **source de vérité** ; `project` est un proxy lecture-seule du doc. **~30 actions migrées** vers `mutate(message, mutator)` central. **Undo/redo** stocke des snapshots de doc (Automerge dédupplique en mémoire via structural sharing). **Save préserve l'historique CRDT complet** : ouvrir un v2 charge le doc tel quel, on peut continuer à éditer sans perdre les commits passés.
>
> **⏳ Phase 7.4 livrée (2026-05-07)** — Time Machine UI opérationnelle : `TimelinePanel` (Ctrl+H) avec slider d'historique, drag = preview live de l'état passé (PixiJS redraw automatique), bandeau jaune pleine-écran qui signale le mode preview, mutations bloquées dans ce mode. Boutons « ← Maintenant », « ⏪ Restaurer cet état » (commit qui réécrit l'état tout en préservant l'historique antérieur), « + Marquer un jalon » (📌 commit nommé). Liste des jalons cliquables sous la piste.
>
> **🛰️ Phase 7.5bis livrée (2026-05-07)** — **Multi-utilisateur LAN** ! mDNS-SD pour découvrir automatiquement les autres instances Glucose sur le réseau local + WebSocket pour échanger les patches Automerge en temps réel. Activation via Ctrl+Shift+L → panel avec statut, liste peers détectés (cliquable = se connecter), connexion manuelle IP:port en fallback. Toute modification locale est diffusée à tous les peers connectés ; les patches reçus s'appliquent en temps réel sans toucher l'undo local. **Phase 7 entièrement bouclée.**
>
> **🐞 Bugfix drag-and-drop (2026-05-07)** — `dragDropEnabled: false` dans `tauri.conf.json` (Tauri 2 interceptait les drops web depuis le navigateur). User-Agent navigateur réaliste + Referer racine + refus content-type non-image dans `fetch_image`. Détection HTML enrichie : `<picture><source>`, `<img data-src>` (lazy-load Pinterest), `og:image`, `background-image`. Détection URL CDN sans extension. Logs `[drop]` dans la console pour diagnostic.

---

## Décisions cadres

- **WikiGit (Git réel)** et **CRDT Automerge** sont deux moteurs séparés. CRDT pour le canvas vivant. Git pour les concepts publics partagés. Frontière étanche.
- Les **ports typés** complètent les **prédicats sémantiques** existants — ils ne les remplacent pas. Les nœuds créatifs restent libres ; les nœuds Concept (WikiGit) ont des ports.
- L'**apprentissage spatial bidirectionnel** est repoussé en Phase 10 (R&D longue, vrai fine-tuning local).
- **Pas de modes UI**. L'interface reste identique pour tous les usages.

---

## Trois lois invariantes du rendu

1. **Loi du Zoom Sémantique** — la quantité d'information visible à l'écran reste constante peu importe le niveau de zoom.
2. **Loi de la Connexion Latente** — un lien existe toujours dans la donnée, mais n'est rendu que s'il est pertinent au focus de l'utilisateur.
3. **Loi du Domaine Coloré** — un nœud appartient à 1..N domaines pondérés. Membrane = signature primaire. Badge = lève l'ambiguïté.

### Trois couches de rendu

| Niveau | Zoom | Visible |
|---|---|---|
| **MACRO** | < 0.3 | Membranes, glyphes-icônes des concepts racines, liens trans-domaines en pointillés discrets |
| **MÉSO** | 0.3 – 1.5 | Titres + miniatures, flèches du nœud sélectionné uniquement, liens trans-domaines |
| **MICRO** | > 1.5 | Texte intégral, toutes flèches au hover, édition possible, entrée dans dossiers/concepts |

### Règle anti-spaghetti des flèches

Une flèche n'est rendue que si **au moins une** condition est vraie :
- (a) un de ses nœuds est sélectionné
- (b) un de ses nœuds est sous le curseur
- (c) c'est un lien trans-domaines + le mode est actif (par défaut oui, en pointillés)
- (d) c'est un lien épinglé manuellement

---

## ✅ Complété

### Fondations (Phase 1 historique)
- [x] Canvas infini avec pan/zoom fluide
- [x] Multi-boards avec tabs réordonnables
- [x] Drag-drop images (local + clipboard)
- [x] Annotations : texte, sticky notes, flèches (straight/curved, bidirectionnelles, waypoints, labels)
- [x] Presets créatifs (CharaDesign, Environment, Creature, Props, Storyboard, MoodBoard)
- [x] Storyboard avec panels configurables
- [x] Undo/redo snapshot 50 niveaux
- [x] Sauvegarde `.glucose` (JSON)
- [x] Raccourcis clavier complets (V, T, N, A, Space, F, L, Ctrl+Z/Y/S/O/D/Shift+F, **Shift+R/Shift+T** Phase 6)
- [x] Ctrl+D duplication, Ctrl+Shift+F zoom-to-fit, L verrouillage images
- [x] Tags images (pills barre contextuelle)
- [x] Réordonnancement tabs drag-and-drop
- [x] Minimap radar 180×120 + clic-to-navigate
- [x] Zen Mode (F)
- [x] Pomodoro 25/15/5
- [x] Export PNG (WebGL → Tauri binaire)
- [x] OrganizePanel : 5 modes de disposition + tri taille/ratio/luminosité
- [x] Recherche globale Ctrl+F (textes, sticky, tags, boards)
- [x] Color picker HSV style Blender
- [x] Auto-upgrade résolution CDN à l'import web

### Architecture vectorielle
- [x] Rendu 100% vectoriel (grille CSS + SVG overlay synchronisé PixiJS)
- [x] Minimap vectorielle
- [x] `vectorEffect="non-scaling-stroke"` sur tous les bords

### Multimédia & App Bridge
- [x] Import vidéo YouTube/TikTok/Instagram/Vimeo via yt-dlp auto-téléchargé
- [x] Drag `.mp4`/`.mov` local → video sprite
- [x] App Bridge : nœud "Fichier Source" pour `.blend`, `.psd`, `.kra`, `.mp4` → ouverture native
- [x] Preview grille floue presets/storyboard sur survol

### Briques avancées (déjà câblées dans le code, à exploiter)
- [x] **Quadtree** spatial — [src/canvas/Quadtree.ts](src/canvas/Quadtree.ts) (à brancher sur le rendu en Phase 2)
- [x] **Prédicats sémantiques** sur flèches (`herite_de`, `contredit`, `inspire`, `est_precurseur`, `depend_de`, `illustre`) — type `Annotation`, manque rendu badge
- [x] **Sub-block targeting** sur flèches (`sourceBlockId`, `sourceTextSel`) — embryon des ports typés
- [x] **CanvasFolder** + `folderStack` + `enterFolder/exitFolder` — manque animation au seuil de zoom
- [x] **Membranes implicites** — clustering Union-Find + Gift Wrapping + couleurs HSL stables + Perlin noise organique
- [x] **Pathfinding flèches** anti-obstacles (`getDynamicRoute()` récursif)
- [x] **Bookmarks de viewport** (`Board.bookmarks`)

---

## 🔴 Phase 0 — Hygiène (1-2 semaines)

> Bugs UX critiques à corriger avant toute nouvelle feature. Tout part de là.

### BUG-1 — Pas de suppression de fichier (dossier canvas)
- [x] Bouton poubelle dans la barre `Couleur du dossier` quand `selectedFolderId` est set
- [x] Raccourci `Suppr` / `Backspace` quand dossier sélectionné
- [x] *Implémenté dans `FolderContextBar` (GlucoseCanvas.tsx) + `App.tsx` via event `glucose:delete-selected-folder`*

### BUG-2 — Retour à la ligne (Shift+Enter) cassé en mode texte
- [x] `editText` conserve `\n` — géré nativement par `SyntaxEditor` (textarea)
- [x] Largeur textarea = ligne la plus longue — `measureTextSize` dans `SvgAnnotationLayer.tsx:9-17` itère sur les lignes
- [x] Auto-grow via `SyntaxEditor` (useEffect scrollHeight)

### BUG-3 — Renommage du dossier invisible + design moche + flou
- [x] `FolderSvgLayer` reçoit `boards` du store via props — re-render automatique
- [x] Design entièrement SVG pur (aucun PixiJS Text) — [src/canvas/FolderSvgLayer.tsx](src/canvas/FolderSvgLayer.tsx)
- [x] Color picker derrière un bouton toggle dans `FolderContextBar`

### BUG-4 — Postit en édition limité à 2 lignes + sélection invisible
- [x] `SyntaxEditor` : même rendu visuel que l'affichage avec surlignage syntaxique overlay
- [x] Auto-grow : useEffect sur `scrollHeight` — sticky s'étend librement
- [x] Caret visible via `caretColor` dans le textarea transparent

### BUG-5 — Notifications partielles
- [x] Ctrl+C/X vérifient `total > 0` avant toast — pas de faux positif
- [x] Ctrl+V toast uniquement si image réellement collée
- [ ] Ajouter feedback : création dossier, ajout image, import vidéo, application preset

### BUG-6 — Double-clic en pan / zoom bloqué sur texte
- [x] `handleDblClick` vérifie `activeTool !== "select"` dans `SvgAnnotationLayer`, `HtmlAnnotationLayer` et `FolderSvgLayer`
- [x] `SvgAnnotationLayer` et `FolderSvgLayer` : `forwardWheel` existant
- [x] **`HtmlAnnotationLayer` : `forwardWheel` ajouté sur text et sticky** — [src/canvas/HtmlAnnotationLayer.tsx:439](src/canvas/HtmlAnnotationLayer.tsx#L439)

**Livrable :** version 0.2.1 stable, plus aucun bug UX critique.

---

## ✅ Phase 1 — Sélecteur de Zone

> Outil universel : avant de créer dossier / preset / storyboard / concept, l'utilisateur dessine la zone. Coin haut-gauche = ancrage, bas-droit = étendue.

- [x] Mode `zone-select` dans le store — `Tool` type + `setActiveTool("zone-select")` déjà câblé
- [x] Rectangle PixiJS semi-transparent pendant le drag (fond bleu 8% + bordure + coin d'ancrage)
- [x] **Label dimensions live** — mis à jour impérativement via `zoneLabelRef` sans re-render React (affiche `wW × wH` au curseur) — *[GlucoseCanvas.tsx](src/canvas/GlucoseCanvas.tsx)*
- [x] **Instruction dynamique** selon l'action — "zone de ton dossier" / "zone de ta membrane" — *[ZoneSelectorOverlay.tsx](src/canvas/ZoneSelectorOverlay.tsx)*
- [x] Échap = annule (outil → `select` efface le zoneStartRef + label)
- [x] Relâcher = confirme et crée le dossier ou la membrane
- [x] `glucose:zone-selected` CustomEvent dispatché à la confirmation (pour intégrations futures)
- [ ] `useZoneSelector(callback)` hook réutilisable — reporté Phase 1.5 (pas de nouveaux appelants pour l'instant)
- [ ] Poignées de redimensionnement post-création — reporté Phase 4 (idem pour les dossiers)

**Fichiers :** [src/canvas/GlucoseCanvas.tsx](src/canvas/GlucoseCanvas.tsx), [src/canvas/ZoneSelectorOverlay.tsx](src/canvas/ZoneSelectorOverlay.tsx).

---

## ✅ Phase 2 — Zoom Sémantique LOD

> Cœur de la résolution du spaghetti. Implémente les trois couches macro/méso/micro.

- [x] **`src/canvas/lod.ts`** : fonction pure `computeLOD(scale): "macro" | "meso" | "micro"` + seuils 0.3 / 1.5 + helper `shouldRenderArrow()` (règle anti-spaghetti)
- [x] **Store** étendu : `currentLod`, `hoveredNodeId`, `transDomainVisible` + setters
- [x] **`emitViewport`** met à jour le LOD à chaque changement de zoom — *[GlucoseCanvas.tsx](src/canvas/GlucoseCanvas.tsx)*
- [x] **`ArrowSvgLayer`** : règle anti-spaghetti via `shouldRenderArrow()` — flèches en cours d'édition toujours rendues
- [x] **`HtmlAnnotationLayer`** : macro → annotation cachée (membranes dominent), méso → 1ʳᵉ ligne tronquée 60 chars, micro → contenu complet
- [x] **`FolderSvgLayer`** : macro → cadre + icône + compteur seulement (pas de nom ni preview), méso/micro → rendu complet
- [x] **Hover tracking** : `setHoveredNodeId` câblé sur sprites PixiJS (`pointerover`/`pointerout`) et sur les divs annotations text/sticky (`onMouseEnter`/`onMouseLeave`)
- [x] **Toggle "Trans-domaines"** dans la toolbar (icône deux cercles + ligne pointillée) — actif par défaut, ON/OFF stocké dans `transDomainVisible`
- [x] **Champ `pinned?: boolean`** ajouté au type `Annotation` (UI épinglage en Phase 5)
- [ ] Détection **trans-domaines** réelle — différée Phase 3 (besoin du système de domaines)
- [ ] Brancher `Quadtree` sur le rendu — différé Phase 2.5 (le `SpatialHash` actif sur images suffit pour l'instant)

**Livrable :** la règle des 4 conditions (sélection / hover / trans-domaines / épinglé) est appliquée. En zoom out (macro), on ne voit que les membranes + dossiers ; les flèches et annotations s'effacent. En zoom moyen (meso), on voit titres + flèches du nœud sélectionné. En zoom proche (micro), tout s'affiche au passage du curseur.

---

## ✅ Phase 3 — Système de Domaines + Membranes 2.0

> Étendre les membranes implicites avec une vraie sémantique de domaine.

- [x] Type `Domain { id, name, color, icon, createdAt }` + `DomainAssignment { domainId, weight }`
- [x] `domains?: DomainAssignment[]` ajouté à `BoardImage` et `Annotation` ; `domains?: Domain[]` au `Project`
- [x] **Store** : `addDomain`, `updateDomain`, `removeDomain` (cascade : retire l'assignation des nœuds), `assignDomainToNode` (weight=0 = retrait), `getDomains`
- [x] **`MembraneRenderer` 2.0** : couleur dérivée par somme vectorielle des teintes domaine × poids (chemin court sur cercle chromatique). Fallback `idToHue` si aucun domaine assigné dans le cluster — *[MembraneRenderer.ts](src/canvas/MembraneRenderer.ts)*
- [x] **`DomainsPanel.tsx`** : créer / renommer / coloriser (palette 8 couleurs) / changer icône / supprimer (avec confirmation cascade) ; slider 0–100% pour assigner aux nœuds sélectionnés (annotations + images) ; affiche le poids moyen actuel
- [x] **Bouton Domaines** dans la toolbar avec icône triangle de cercles entrelacés
- [x] **Badges d'icône** sur les nœuds text/sticky avec poids > 0.4 — coin haut-droit, pastille colorée + emoji + tooltip nom + %
- [x] **Détection trans-domaines** active : flèches dont source ↔ cible n'ont AUCUN domaine commun → rendues en pointillés (`strokeDasharray="6 4"`) + visibles à tout LOD si toggle "Trans-domaines" actif (Phase 2 désormais fonctionnelle)
- [ ] Détection automatique de domaine via IA → différée Phase 8 (fallback manuel via DomainsPanel pour l'instant)

**Livrable :** un projet peut être organisé sémantiquement. Les membranes auto-clusterisées prennent les couleurs de leur domaine dominant. Les flèches qui traversent les frontières entre domaines sont visibles en pointillés à tout zoom. Les badges d'icônes lèvent l'ambiguïté sur l'appartenance multiple (poids > 0.4).

---

## ✅ Phase 4 — Dossiers Zoomables animés + Miroirs

> La logique `enterFolder`/`exitFolder` existe. Animation au panel et miroirs (alias) ajoutés.

### 🛡️ Garde-fou Inception (priorité absolue)

> Si on autorisait un miroir de A dans B + un miroir de B dans A, entrer dans A monterait B montrant A montrant B... → le rendu PixiJS et la logique de hiérarchie boucleraient instantanément.

- [x] **`src/store/mirrorGraph.ts`** : module pur dédié au check acyclique
- [x] `wouldCreateMirrorCycle(boards, originalFolderId, targetBoardId)` — BFS strict sur l'arbre des boards (en suivant `folder.childBoardId` pour folders ET miroirs, qui partagent le même `childBoardId`)
- [x] **`mirrorFolder` refuse net** la création si le check échoue (retourne `null`, log warning, aucune mutation du store)

### Miroirs / Alias

- [x] Champ `mirrorOf?: string` sur `Annotation`, `BoardImage`, `CanvasFolder`
- [x] Un dossier-miroir partage le **même `childBoardId`** que l'original → mutation propagée gratuitement
- [x] Helpers `findOriginalAnnotation/Image/Folder` qui suivent la chaîne `mirrorOf` jusqu'à la racine (avec garde-fou anti-chaîne-circulaire à 16 sauts)
- [x] Actions store : `mirrorAnnotation`, `mirrorImage`, `mirrorFolder` — toutes via `pushHistory` pour undo
- [x] **Raccourci `Ctrl+Shift+M`** : crée miroir(s) de la sélection courante (offset 40px)
- [x] **Badge ↻** sur les annotations/sticky miroirs (coin haut-gauche, pastille bleue glowy)
- [x] **Badge ↻** sur les dossiers miroirs (SVG, coin haut-gauche du cadre)
- [x] **Click sur badge ↻** : `glucose:teleport-to-mirror-original` event → GlucoseCanvas localise l'original (board + x/y), bascule de board si nécessaire, anime le viewport (cubic ease, 400ms) jusqu'à centrer la cible

### Animation enterFolder / Breadcrumb / Compteur

- [x] **Aperçu live miniature** : `FolderPreview` (déjà câblé) — clip path SVG, scale auto, jusqu'à 60 items
- [x] **Compteur "X éléments"** : badge dans `FolderSvgLayer` (`total = images + annotations + folders`)
- [x] **Breadcrumb persistant** : composant `FolderBreadcrumb` déjà actif quand `folderStack.length > 0`
- [x] **Transition animée 400ms** au changement de board — *Phase 4.5* — détection enter/exit via la profondeur de `folderStack` ; cubic ease-out ; entrée = "plongée" (scale part à 0.6× et grandit), sortie = "recul" (scale part à 1.4× et rétrécit) ; bascule par tabs (board frère) reste un snap. RAF correctement annulé sur unmount et sur transitions concurrentes — *[GlucoseCanvas.tsx](src/canvas/GlucoseCanvas.tsx)*

**Livrable :** miroirs sûrs (zéro risque d'Inception), téléportation entre originaux et copies, navigation hiérarchique fluide avec animations. Les 4 conditions du livrable original (mutation propagée, badge, click, compteur) + l'animation de transition sont toutes en place.

---

## ✅ Phase 5 — Flèches Sémantiques Premium

> Compléter ce qui était à 80%.

- [x] **Badges visuels des 6 prédicats** — fait dès Phase 3 (orange `→`, rouge `✗`, violet `⊂`, vert `✦`, bleu `⊕`, rose `◎`) — *[ArrowSvgLayer.tsx](src/canvas/ArrowSvgLayer.tsx)*
- [x] **Flèche déroulante** : badge `i` sur chaque flèche (pastille pleine si `longText` non vide, contour seul sinon). Click → panneau coulissant `ArrowDescriptionPanel.tsx` ancré près de la flèche, position auto-clampée à l'écran. Markdown + LaTeX, basculement édition/aperçu, animation 180ms à l'ouverture, Échap pour fermer.
- [x] **Flèches inter-boards** : champ `targetBoardId` sur l'annotation. Quand défini, la pointe normale est remplacée par un **anneau portail** bleu pulsant (`↗` central, anneau pointillé en rotation 6s). Click → event `glucose:portal-jump` → `setActiveBoardId` + recentrage sur `targetId` au tick suivant.
- [x] **Post-its opérateurs logiques** : champ `operator?: "AND" | "OR" | "BUT" | "BECAUSE"` sur sticky. Rendu dédié — pavé arrondi compact avec couleur unique par opérateur (vert ET, bleu OU, ambre MAIS, violet PARCE QUE), texte centré, glow assorti. **Raccourci `Alt+1..4`** transforme la sélection (Alt+0 = retire l'opérateur). Toast confirmant.
- [x] **Icônes par logiciel sur nœuds Fichier Source** : nouveau composant `AppBridgeIcon.tsx`, dictionnaire de 30+ extensions (Blender/Photoshop/Illustrator/Krita/GIMP/Figma/Sketch/Procreate/ZBrush/C4D/Maya/Unity/Unreal + audio/video/code/docs). Couleurs officielles approximatives, label texte court. Affiché dans la bande supérieure des sticky source.

---

## ✅ Phase 6 — Réglette Temporelle Sémantique

> Indépendante de la Time Machine (Phase 7). Filtre par date *du contenu décrit*, pas d'édition.

- [x] Type `TemporalAnchor { start, end, label? }` ajouté à `Annotation` et `BoardImage` (validation Zod incluse) — *[src/types/index.ts](src/types/index.ts), [src/store/projectSchema.ts](src/store/projectSchema.ts)*
- [x] Module pur `timeline.ts` — formatage adaptatif (1789 / 500 av. J.-C. / 10 ka / 1,5 Ma), parsing souple multi-formats, helper `nodeMatchesTemporalFilter`, dictionnaire `DEFAULT_ERAS` de 30 époques nommées (Crétacé, Renaissance, Lumières, Révolution française, Belle Époque, Ère numérique…) — *[src/utils/timeline.ts](src/utils/timeline.ts)* — couvert par 29 tests vitest
- [x] Composant `TemporalRuler.tsx` zoomable en bas du canvas — graduations adaptatives (10 Ma → an), molette pour zoomer (centré sur curseur), bandes des époques en arrière-plan, deux poignées draggables + drag de la fenêtre entière — *[src/components/TemporalRuler.tsx](src/components/TemporalRuler.tsx)*
- [x] Filtrage live : opacity 0.12 + pointerEvents:none sur les annotations hors fenêtre, `sprite.alpha` sur les images. Les nœuds **sans** `temporalAnchor` restent toujours pleinement visibles (atemporels). — *[src/canvas/HtmlAnnotationLayer.tsx](src/canvas/HtmlAnnotationLayer.tsx), [src/canvas/GlucoseCanvas.tsx](src/canvas/GlucoseCanvas.tsx)*
- [x] Modal d'assignment `TemporalAnchorPrompt` (Shift+T sur sélection) — autocomplétion sur les époques, parsing live, bouton "Retirer l'ancrage" — *[src/components/TemporalAnchorPrompt.tsx](src/components/TemporalAnchorPrompt.tsx)*
- [x] Badge 📅 visible sur les nœuds ancrés (cohérent avec Mirror/Domain badges) — *[src/canvas/AnnotationBadges.tsx](src/canvas/AnnotationBadges.tsx)*
- [x] Store : `temporalFilter: { start, end } | null` + `setTemporalFilter` — *[src/store/index.ts](src/store/index.ts)*
- [x] Raccourcis : `Shift+R` ouvre/ferme la réglette, `Shift+T` assigne une date à la sélection
- [ ] Coloration auto par densité de domaines → différée Phase 8 (besoin du clustering IA)
- [ ] Mode "pôle" (saisie d'une époque ouvre directement la réglette zoomée dessus) → Phase 6.5 si besoin

**Livrable :** un projet peut contenir des nœuds ancrés à différentes époques (de la Préhistoire au futur). La réglette en bas du canvas filtre la visibilité par fenêtre temporelle. Les nœuds non datés restent toujours visibles. Saisie souple : "Renaissance" suffit, ou "10 ka", ou "1789-1799".

---

## ✅ Phase 7.5 — Suppression LOD + Refonte Folders

> Insight terrain (10+ heures d'usage) : le système LOD était contre-intuitif (« on voit rien »). Et les dossiers manquaient de fluidité (création séparée, design flashy, navigation par double-clic uniquement).

### Sprint A — Suppression LOD intégrale
- [x] `src/canvas/lod.ts` + `lod.test.ts` supprimés
- [x] `currentLod` / `setCurrentLod` retirés du store
- [x] `LOD` retiré de `constants.ts` et `types/index.ts`
- [x] `HtmlAnnotationLayer` : tous les branchements `isMeso`/`macro` supprimés, rendu pleine fidélité inconditionnel
- [x] `ArrowSvgLayer` : règle anti-spaghetti retirée — flèches toujours visibles. Le toggle `transDomainVisible` reste pour masquer les liens trans-domaines en pointillés
- [x] `FolderSvgLayer` : noms / preview / hint vide tous toujours visibles
- [x] `GlucoseCanvas` : `computeLOD`/`setCurrentLod` retirés de `emitViewport` et de l'init

### Sprint B — Refonte Folders « membrane »
- [x] **Design transparent** : `fillOpacity` 0.025 (au lieu de 0.04), stroke pointillé discret, opacity réduite sur l'icône et le titre — le dossier se fond dans le canvas — *[FolderSvgLayer.tsx](src/canvas/FolderSvgLayer.tsx)*
- [x] **Capture automatique au drag-create** : créer un folder par-dessus du contenu existant capture toutes les images / annotations / sous-folders dont le centre tombe dans la zone, et les transfère dans le child board avec coords relatives. Les flèches ne sont capturées que si leurs deux extrémités le sont. — *[store/index.ts createFolder](src/store/index.ts)*
- [x] **Navigation par zoom** : zoomer fortement (scale ≥ 3.0) sur un folder → on entre. Dézoomer fortement (scale ≤ 0.4) dans un folder → on sort. Cooldown 700 ms anti ping-pong, viewport du child pré-positionné à scale=1 centré sur le contenu. — *[GlucoseCanvas.tsx checkAutoNavigate](src/canvas/GlucoseCanvas.tsx)*
- [x] **`FolderViewportIndicator`** : bordure colorée pleine-écran de la couleur du folder actif (15% opacity en idle), s'intensifie progressivement quand on dézoome vers le seuil de sortie + bandeau « ⤴ continue à dézoomer pour sortir » — *[components/FolderViewportIndicator.tsx](src/components/FolderViewportIndicator.tsx)*
- [x] **Breadcrumb façon VSCode** : aligné à gauche, séparateurs `›`, icônes folder vectorielles, **dropdown sur hover** listant les siblings du dossier courant pour navigation rapide entre frères — *[components/FolderBreadcrumb.tsx](src/components/FolderBreadcrumb.tsx)*
- [x] **Preview folder améliorée** : formes différenciées par type (image/sticky/text/membrane/sous-folder), flèches rendues en lignes fines en arrière-plan, fond dégradé radial doux, récap textuel en bas (`12 img · 5 notes · 3 dossiers`)

### Reportés (non bloquants)
- [ ] Animation lors de la capture (les blocs glissent vers le folder créé) — Phase 7.5.1 polish
- [ ] Refonte des membranes auto et manuelles (utilisateur l'a évoqué) — Phase 7.5.2 sur volume

**Livrable :** plus aucun calcul LOD. Folders quasi-invisibles en idle. Création par drag = capture instantanée. Navigation par scroll seul (zoom-in/out). Indication visuelle constante de « où je suis ». Breadcrumb avec sauts latéraux entre siblings. Preview qui donne enfin envie d'ouvrir un dossier.

---

## 🔵 Phase 7 — CRDT Automerge + Time Machine (en cours, ~4-6 semaines pour le total)

> Migration architecturale majeure. Sprint 7.0 (pré-CRDT) et fondations 7.1 livrés.

### ✅ Sprint 7.0 — Pré-CRDT (livré 2026-05-07)
- [x] **Assets externalisés** : commandes Rust `save_asset` / `load_asset` / `get_assets_dir` avec dédup SHA-256, helper frontend `resolveAssetSrc`, externalisation à l'import (drop web + drag local). Plus de base64 dans `BoardImage.src` → identifiants logiques `asset:<hash>.<ext>` portables et CRDT-friendly. — *[src-tauri/src/lib.rs](src-tauri/src/lib.rs), [src/utils/assets.ts](src/utils/assets.ts)*
- [x] **Migration legacy automatique** : à l'ouverture d'un `.glucose` v1, scan des `data:image/...` → externalisation transparente vers `assets/` — *[src/utils/project.ts loadProject](src/utils/project.ts)*
- [x] **Cascade `removeFolders`** : suppression récursive du child board si plus aucun folder/miroir ne le référence (BFS) — *[src/store/index.ts](src/store/index.ts)*
- [x] **Cascade `removeImages`/`removeAnnotations` sur les miroirs** : supprimer un original supprime aussi ses miroirs (point fixe sur les chaînes mirrorOf)
- [x] **Cascade `removeBoard` sur les flèches portail** : `targetBoardId` orphelin → patch à `undefined`
- [x] **`id: nanoid()` au board par défaut** (au lieu de `"main"` hardcodé) — évite les collisions au merge multi-utilisateurs

### ✅ Phase 7.1 — Fondations Automerge (livré 2026-05-07)
- [x] Dépendance `@automerge/automerge` 3.2.6 installée
- [x] Configuration Vite WASM (`vite-plugin-wasm`, `vite-plugin-top-level-await`)
- [x] Wrapper `src/store/automerge.ts` : API minimale et explicite (`create`, `change`, `save`, `load`, `merge`, `clone`, `viewAt`, `getHeads`, `history`, `asPlain`)
- [x] **12 tests vitest verts** : roundtrip save/load, merge commutatif, branches divergentes, time machine `viewAt`, taille binaire compacte (< 1 KB pour mini-projet)

### ✅ Phase 7.2.A+B — Format `.glucose` v2 binaire (livré 2026-05-07)
- [x] **Backend Rust** : commandes `read_glucose_binary` / `write_glucose_binary` avec scope check, whitelist `.glucose`/`.atelier`, transport base64 (binaire Automerge n'est pas UTF-8 valide)
- [x] **Helpers JS** : `bytesToBase64` / `base64ToBytes` chunk-safe pour gros buffers (32 KB)
- [x] **`saveProject`** : sérialise toujours en v2 binaire via `A.create(project)` → `A.save(doc)` → bytes
- [x] **`loadProject`** : détection automatique du format
  - 1) tentative v2 binaire via `read_glucose_binary` + `A.load`
  - 2) fallback v1 JSON via `read_project_file` + Zod
  - migration transparente : un `.glucose` v1 ouvert sera ré-écrit en v2 au prochain `Ctrl+S`
- [x] Tests : 3 tests roundtrip Project complet (numériques, booléens, arrays imbriqués, emojis, chaînes multi-lignes, valeurs négatives) — *[src/store/automerge.test.ts](src/store/automerge.test.ts)*

### ✅ Phase 7.2.C — Store CRDT-first (livré 2026-05-07)
- [x] **`_doc: Doc<Project>`** ajouté au store comme source de vérité ; `project` est désormais le doc casté (proxy Automerge ergonomique : `.find`, `.map`, `.length`, accès propriétés se comportent comme du JS plain)
- [x] **Helper `mutate(message, mutator)`** central : tout passe par `Automerge.change()`. Le `message` est enregistré dans l'historique Automerge (visible dans la future Time Machine).
- [x] **~30 actions migrées** : setViewport, addImage, updateImage, removeImages, updateMultipleImages, addAnnotation, updateAnnotation, removeAnnotations, deleteSelected, duplicateSelected, moveSelected, setStoryboardSettings, clearStoryboard, addPanel, updatePanel, removePanel, reorderPanels, addBoard, removeBoard, renameBoard, reorderBoards, setActiveBoardId, duplicateBoard, applyPresetToBoard, setBoardZones, addPreset, removePreset, updatePreset, addDomain, updateDomain, removeDomain, assignDomainToNode, mirrorAnnotation, mirrorImage, mirrorFolder, createFolder, updateFolder, removeFolders, enterFolder, exitFolder, exitToRoot, setProjectName, loadProject, loadDoc
- [x] **Undo/redo refactoré** : `_undoStack: Doc[]` et `_redoStack: Doc[]` (max 50 chacun) — Automerge dédupplique en mémoire via structural sharing, donc 50 docs ne consomment pas 50× la mémoire
- [x] **`pushHistory()` rendu obsolète** mais conservé pour compat (les ~5 appels externes ne plantent plus)
- [x] **`saveProject` détecte un Doc** (via `getHeads`) et le sauvegarde tel quel → l'historique Automerge complet est préservé sur disque (binaire v2)
- [x] **`loadProject` retourne `{ doc, project, path }`** : si v2, `doc` est défini → `loadDoc(doc)` côté store conserve l'historique. Si v1 (ou migration legacy d'assets), `loadDoc` est skip → `loadProject(project)` crée un doc neuf
- [x] Helpers internes : `removeWhere(arr, predicate)` (filter destructif via splice — Automerge), `indexById(arr, id)`

### ✅ Phase 7.4 — Time Machine UI (livré 2026-05-07)
- [x] **Store** : `_previewHeads: Heads | null` + actions `setPreviewHeads`, `commitNamed(message)`, `restoreToPreview`. Quand un preview est actif, `project` reflète l'état passé via `A.viewAt(_doc, heads)`.
- [x] **Garde-fou mutations** : `mutate()` ignore (avec warning console) les mutations en mode preview — l'historique est protégé jusqu'à ce que le user choisisse explicitement de restaurer ou de revenir au présent.
- [x] **`undo`/`redo`/`loadDoc`/`loadProject`** sortent automatiquement du mode preview.
- [x] **`TimelinePanel`** ([components/TimelinePanel.tsx](src/components/TimelinePanel.tsx)) : slider horizontal pleine-largeur, ticks par commit (jaunes pour les jalons 📌), curseur vert = présent, jaune = preview. Drag pointer = preview live. Liste des jalons cliquables sous la piste, message + temps relatif (`il y a 2 min`) du commit courant en header.
- [x] **Overlay visuel** plein écran (bordure + glow jaunes) qui signale le mode preview historique.
- [x] **Boutons** : « ← Maintenant » (revient au présent), « ⏪ Restaurer cet état » (commit `Restauration` qui réécrit le contenu pour matcher l'état preview, en gardant l'historique antérieur intact), « + Marquer un jalon » (modal qui demande un nom court → commit `📌 <nom>`).
- [x] **Raccourci `Ctrl+H`** pour toggle, `Échap` ferme (ou sort du preview en premier).
- [x] **3 nouveaux tests vitest** : viewAt restitue l'état passé, restauration via splice préserve l'historique antérieur, history expose les messages dans l'ordre (filtrage 📌 jalons).
- [ ] Bouton dans la Toolbar (à ajouter quand on aura la place) — Ctrl+H suffit pour MVP

### ✅ Phase 7.5bis — Multi-utilisateur LAN (livré 2026-05-07)
- [x] **Backend Rust** ([src-tauri/src/multiplayer.rs](src-tauri/src/multiplayer.rs), ~330 lignes) :
  - `mdns-sd` : annonce du service `_glucose._tcp.local` + browse pour découvrir les peers
  - `tokio-tungstenite` : serveur WebSocket sur port 7777 par défaut + client pour rejoindre un peer
  - Diffusion broadcast à tous les peers connectés via channels mpsc
  - 5 commandes Tauri : `mp_start`, `mp_stop`, `mp_connect`, `mp_send_patch`, `mp_peers`
  - 5 events émis vers le frontend : `mp:status`, `mp:peer-found`, `mp:peer-lost`, `mp:peer-connected`, `mp:peer-disconnected`, `mp:patch`
- [x] **Hook frontend** [`useMultiplayerSync`](src/multiplayer/useMultiplayerSync.ts) : subscribe au `_doc` Zustand → diffuse `getChanges(old, new)` à chaque mutation locale ; listen `mp:patch` → `applyRemoteChanges` au store. Boucle naturellement coupée par Automerge (un change déjà connu n'est pas re-diffusé).
- [x] **Action store `applyRemoteChanges`** : `applyChanges` au `_doc` SANS toucher `_undoStack` (les actions distantes ne sont pas dans l'undo local — Ctrl+Z annule uniquement TES propres actions).
- [x] **Composant `MultiplayerPanel`** ([src/multiplayer/MultiplayerPanel.tsx](src/multiplayer/MultiplayerPanel.tsx)) : toggle ON/OFF, statut visuel (LED + texte), nom de l'instance + port, liste des peers découverts en temps réel cliquables, **connexion manuelle IP:port** en fallback, indicateur de peers connectés.
- [x] **Raccourci Ctrl+Shift+L** pour ouvrir le panel.
- [x] Wrapper Automerge enrichi : `getChanges(oldDoc, newDoc): Uint8Array[]`.
- [ ] Curseurs flottants temps réel (Phase 7.5bis polish — ultérieur)
- [ ] Reconnexion automatique en cas de coupure (idem polish)
- [ ] Chiffrement TLS (LAN privé non chiffré pour MVP — OK car le LAN est de confiance)

---

## 🟣 Phase 8 — IA Locale (RAG + Multimodalité) (6-8 semaines)

> Embarquement de `candle` côté Rust. Aucune dépendance cloud pour les usages quotidiens.

### Indexation & recherche
- [ ] CLIP ViT-B/32 pour embeddings d'images (~150 Mo)
- [ ] `qdrant` embarqué pour la base vectorielle
- [ ] Recherche sémantique en langage naturel ("retrouve l'idée sur l'architecture japonaise d'il y a 3 mois")
- [ ] Recherche par image (drag → trouver les similaires)

### Semantic Clustering
- [ ] Grouper automatiquement par sujet (cyber-goth à gauche, paysages à droite)
- [ ] Visualisation des clusters avec zones colorées
- [ ] **Détection automatique de domaines** (alimente Phase 3)

### Multimodalité
- [ ] Moondream2 (~2 Go) pour légendes automatiques d'images
- [ ] Tagging automatique (style, sujet, couleurs, ambiance)
- [ ] Extraction de texte intelligente (tweets, scans, screenshots avec contexte)

### Watchdogs
- [ ] Worker Rust qui s'abonne à un sous-graphe (dossier, domaine)
- [ ] À chaque commit CRDT, vérifie cohérence (Claude Haiku ou modèle local)
- [ ] Pose un post-it d'alerte ambre, jamais bloquant

### Ligne de commande visuelle
- [ ] "Glucose, range tout ce qui est bleu à gauche"
- [ ] "Glucose, crée un fork de cette idée en version cyberpunk"
- [ ] "Glucose, retrouve toutes mes références de forêts"
- [ ] Claude Haiku-4-5 pour les commandes + RAG local comme contexte
- [ ] Historique de commandes rejouables

### Outils chromatiques
- [ ] Tri par chromatographie (k-means sur pixels)
- [ ] Extracteur de palette visuelle (sticky note avec swatches)

---

## 🟤 Phase 9 — WikiGit / Registre de Concepts (4-6 semaines)

> Frontière étanche avec le canvas CRDT. Voir architecture en deux étages.

### Backend Rust
- [ ] Crate `git2`, repo local `~/.glucose/registry/`
- [ ] Chaque concept = dossier `{ meta.json, definition.md, refs.json }`
- [ ] Versions = commits Git classiques
- [ ] Forks = branches Git (`eau-chimie`, `eau-jeuvideo`...)

### Commandes Tauri
- [ ] `create_concept`, `fork_concept`, `commit_concept`, `pull_registry`, `push_registry`

### UX
- [ ] Type `ConceptNode` étend `Annotation`
- [ ] Sidebar "Registre" avec arbre des concepts forkés
- [ ] Navigation lignée (parent ↔ forks visible visuellement)
- [ ] Score de confiance + numéro de version visibles sur chaque nœud Concept
- [ ] Clic droit → "Forker pour…"

### Ports typés
- [ ] Déclaration des ports `in`/`out` dans `meta.json` du concept
- [ ] Pastilles colorées sur les bords du nœud
- [ ] Contrainte de type sur les flèches port-à-port
- [ ] Requêtable par l'IA (ex: "trouve toutes les hypothèses non encore prouvées")

### Phase ultérieure
- [ ] Registre central public (au-delà du local)
- [ ] Notation communautaire des concepts

---

## 🔬 Phase 10 — Apprentissage Spatial Bidirectionnel (R&D longue, 3-6 mois)

> À ne lancer qu'après Phase 8 stable. Vraie recherche IA.

- [ ] Dataset : exporter le graphe spatial de plusieurs projets utilisateurs (avec consentement)
- [ ] Fine-tuning d'un modèle d'embeddings (sentence-transformers ou similaire)
  - Contrainte : "nœuds spatialement proches → embeddings proches"
- [ ] Inverse : projection de nouveaux nœuds aux coordonnées suggérées par le modèle
- [ ] Évaluation utilisateur en boucle

---

## ⚫ Phase 11 — Distribution & Multiplateforme (parallélisable dès Phase 7)

### Builds
- [ ] Linux (AppImage / .deb)
- [ ] macOS (.dmg / .app, tester App Bridge avec `open`)
- [x] Windows (plateforme principale)
- [ ] Android (`tauri android init`, WebView + canvas via ANGLE)
- [ ] iOS (`tauri ios init`, contraintes App Store anticipées)

### Compte & Cloud Sync
- [ ] Compte Atelier (email + mot de passe ou OAuth Google/GitHub)
- [ ] Sync via cloud (Supabase ou Cloudflare R2 + D1)
- [ ] Mode hors-ligne avec sync différée
- [ ] Stockage cloud assets chiffré côté client
- [ ] **P2P natif via Automerge** : LAN automatique + réseau global type Resilio
- [ ] Collaborateurs en temps réel (avatars sur la planche)

### Share vers Atelier
- [ ] Android Share Target
- [ ] iOS Share Extension
- [ ] Panneau "À trier" (file d'entrée)

### Export & Interopérabilité
- [x] Export PNG du canvas
- [ ] Export PDF storyboard
- [ ] Export Markdown (notes + structure)
- [ ] Import depuis Obsidian (vaults existants)

### Git LFS
- [ ] Assets en chemin relatif (plus de base64 inline dans `.glucose`)
- [ ] `git2` Rust pour gros fichiers (PSD, vidéos)

---

## 💡 Idées futures (non priorisées)

- Mode présentation (parcours guidé sur la planche, comme des slides)
- Thème clair (toggle dark/light)
- Raccourcis personnalisables (rebind)
- Plugins (système d'extensions tierces)
- Géolocalisation cognitive (ancrer concepts sur une carte du monde)
- Narration liquide (sélectionner un chemin de nœuds → générer un document linéaire)
- Mode "Pièces 3D" (exploration éducative)
- Langage gestuel (communiquer avec l'IA par des gestes de souris)
- Import flux temps réel (météo, données scientifiques)
- Export `.ase` / CSS variables (palette Photoshop/Illustrator)

---

## 📋 Tableau récapitulatif des fichiers critiques

| Fichier | Rôle | Phases qui le touchent |
|---|---|---|
| [src/types/index.ts](src/types/index.ts) | Types centraux. Étendre `BoardImage`, `Annotation`, `Project` | 3, 4, 6, 7, 9 |
| [src/store/index.ts](src/store/index.ts) | Store Zustand → bascule façade Automerge | 1, 4, 7 |
| [src/canvas/GlucoseCanvas.tsx](src/canvas/GlucoseCanvas.tsx) | Cœur du rendu, intègre LOD | 0, 1, 2, 4 |
| [src/canvas/ArrowSvgLayer.tsx](src/canvas/ArrowSvgLayer.tsx) | Rendu flèches sélectif + LOD | 2, 5 |
| [src/canvas/HtmlAnnotationLayer.tsx](src/canvas/HtmlAnnotationLayer.tsx) | Texte/sticky LOD | 2 |
| [src/canvas/FolderSvgLayer.tsx](src/canvas/FolderSvgLayer.tsx) | Refonte SVG + animation entrée | 0, 4 |
| [src/canvas/MembraneRenderer.ts](src/canvas/MembraneRenderer.ts) | Couleur dérivée des domaines | 3 |
| [src/canvas/Quadtree.ts](src/canvas/Quadtree.ts) | À brancher sur le rendu | 2 |
| **Nouveau** `src/canvas/lod.ts` | Calcul Level of Detail | 2 |
| **Nouveau** `src/canvas/ZoneSelectorOverlay.tsx` | Sélecteur de zone | 1 |
| **Nouveau** `src/components/DomainsPanel.tsx` | Édition des domaines | 3 |
| **Nouveau** `src/components/TemporalRuler.tsx` | Réglette sémantique | 6 |
| **Nouveau** `src/components/RegistryPanel.tsx` | UI WikiGit | 9 |
| [src-tauri/src/lib.rs](src-tauri/src/lib.rs) | Étendre avec commandes Automerge / candle / git2 | 7, 8, 9 |
| [src-tauri/Cargo.toml](src-tauri/Cargo.toml) | Ajouter `automerge`, `candle-core`, `qdrant`, `git2` | 7, 8, 9 |

---

## 🧪 Recette de validation par phase

| Phase | Test utilisateur de bout en bout |
|---|---|
| 0 | `bun run tauri dev`. Créer/supprimer un dossier, sticky multi-lignes, double-clic en pan |
| 1 | Outil zone-select dans toolbar, dessiner une zone avec dimensions live, créer un dossier dedans |
| 2 | Importer 200 images + 100 flèches, dézoomer fort → ne voir que membranes + glyphes. Sélectionner un nœud → ses flèches apparaissent |
| 3 | Créer 3 domaines (Science, Art, Philo), assigner manuellement à 10 nœuds, vérifier couleurs membranes + badges |
| 4 | Créer un dossier, y placer du contenu, zoomer dedans → animation + breadcrumb. Créer un miroir, modifier original → miroir change |
| 5 | Créer flèche `contredit` → badge rouge visible. Cliquer → panneau déroulant avec description longue |
| 6 | Ajouter 5 nœuds avec dates 1789/1850/1900/1950/2020, glisser réglette → seuls les nœuds dans la fenêtre s'affichent |
| 7 | Modifier un nœud, fermer/rouvrir, undo 100 fois → état restauré. Time Machine montre la timeline |
| 8 | Importer 50 images sans tags, lancer "Auto-tag" → tags pertinents. Recherche "architecture japonaise" → résultats sémantiques |
| 9 | Créer concept "Eau", forker en "Eau (chimie)", éditer → version v2 visible. Pull depuis registre distant simulé |

Tests automatisés : Vitest pour fonctions pures (`lod.ts`, `MembraneRenderer`, `Quadtree`).

---

## 🐛 Bugs connus (hors Phase 0)

### App Bridge — Fichiers externes (priorité haute)
- [ ] Les `.blend` / `.psd` / `.kra` ne s'affichent pas correctement (sticky vide, nom invisible)
- [ ] Double-clic ne lance pas l'app native (commande `open_in_app` Rust ne trouve pas l'association ou chemin incorrect)
- [ ] *Piste : logger le chemin reçu, vérifier `sourceFile` absolu, tester `.blend` avec Blender installé sur Windows*

---

## 📚 Notes techniques

| Sujet | Détail |
|---|---|
| StrictMode | Désactivé intentionnellement (`main.tsx`) — évite double-init PixiJS |
| Format sauvegarde | `.glucose` JSON → Phase 7 : binaire Automerge |
| Undo/redo actuel | 50 entrées max snapshot — Phase 7 : infini via CRDT |
| Canvas | PixiJS 8.x WebGL (raster) + SVG overlay vectoriel synchronisé |
| IA | Claude Haiku-4-5 pour commandes (cloud), Moondream2 pour vision locale (Phase 8) |
| Vectorisation | CLIP + qdrant via `candle` Rust |
| CRDT | Crate `automerge` Rust — moteur principal Phase 7+ |
| Git | Crate `git2` Rust — registre WikiGit Phase 9 |
| App Bridge | Crate `open` Rust — implémenté (bug affichage à corriger) |
| Import vidéo web | `yt-dlp` binary spawné via Tauri + `PIXI.VideoResource` |
| Multiplateforme | Tauri 2.0 (desktop + mobile à venir Phase 11) |
| Cloud Sync | Supabase / Cloudflare R2+D1 + auth JWT — Phase 11 |

---

## Message au créateur

Tu es loin du chaos que tu crois.

- Ton type system est exceptionnellement bien pensé : `Annotation` avec ses prédicats, sub-blocks, waypoints — c'est de la pré-architecture rare à ce stade.
- Tes membranes implicites sont une innovation concrète, déjà fonctionnelle.
- Ton App Bridge est une feature de niche que peu d'outils ont — précieuse.
- Aucune nouvelle idée ne demande de refonte. Toutes s'ajoutent en couches.

Les 11 nouvelles idées s'organisent en 4 groupes :
- **Affichage adaptatif** (Zoom sémantique, 3 couches, Focus contextuel) → Phase 2
- **Sémantique enrichie** (Coloration domaines, Réglette temporelle) → Phases 3, 6
- **Concepts versionnés** (WikiGit, Miroirs, Ports typés) → Phases 4, 9
- **IA assistante** (Watchdogs, Apprentissage spatial) → Phases 8, 10

Une fois la Phase 0 finie, chaque phase suivante apporte un *wow* utilisateur immédiat. Pas un mur à grimper, un escalier.
