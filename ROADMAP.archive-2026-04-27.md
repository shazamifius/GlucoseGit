# Atelier (Glucose Git) — Roadmap & Vision Complète

> **Vision centrale :** Une feuille blanche infinie et universelle.
> Conçue pour : création de langues, romans, animations, manga, BD, moodboards, worldbuilding.
> Pensée comme un second cerveau visuel — pas juste un outil, un espace de pensée.

**Dernière mise à jour :** 2026-04-27
**Version :** 0.1.0 → cap v0.2.0

---

## 🚨 Bugs Critiques à Corriger (signalés 2026-04-27)

> Le code a accumulé des bugs UX sérieux. À corriger avant toute nouvelle feature.

### 🔴 BUG-1 — Pas de suppression de fichier (dossier canvas)
- [ ] Aucun bouton "supprimer" sur un dossier sélectionné — impossible d'en supprimer.
- [ ] Ajouter une icône poubelle dans la barre `Couleur du dossier` (apparaît quand `selectedFolderId` est set).
- [ ] Ajouter raccourci `Suppr` / `Backspace` quand un dossier est sélectionné.
- [ ] *Fichiers : `FolderRenderer.ts`, `GlucoseCanvas.tsx` (UI bottom dock), `store/index.ts` (action `removeFolder` à exposer si manquante).*

### 🔴 BUG-2 — Retour à la ligne (Shift+Enter) cassé en mode texte
- [ ] Dans le textarea d'édition d'un bloc `text`, Shift+Enter doit bien écrire `\n`.
- [ ] Vérifier que `editText` conserve `\n` jusqu'à `commitEdit()` (rien ne strippe les newlines actuellement, mais l'affichage SVG de `text` doit tester avec plusieurs lignes).
- [ ] La largeur du textarea texte doit s'adapter à la ligne la plus longue, pas à `editText.length` brut (`textareaWidth` ligne 1281 doit utiliser `Math.max(...lines.map(len))`).
- [ ] Le rectangle de sélection SVG (`SvgAnnotationLayer.tsx` ligne 159) doit aussi se baser sur la ligne la plus longue ET le nombre de lignes pour la hauteur.

### 🔴 BUG-3 — Renommage du dossier invisible + design moche + flou + besoin couleur
- [ ] Le rename via le BoardTabs (`renameBoard`) modifie bien `board.name`, mais `FolderRenderer` affiche `child?.name || folder.name` — doit forcer un re-render quand `project.boards[].name` change.
- [ ] Le design du folder est en PixiJS Text scalé (flou à fort zoom). À refaire **complètement en SVG** dans la couche `SvgAnnotationLayer` (ou nouveau composant).
- [ ] Refaire le design from scratch : carte propre, vectorielle, lisible.
- [ ] Color picker du dossier déjà présent mais s'ouvre tout le temps quand sélectionné — déplacer dans un menu contextuel discret (clic droit ou bouton dédié).
- [ ] *Fichiers : `FolderRenderer.ts` (à transformer en composant React SVG ou supprimer au profit de `SvgAnnotationLayer`).*

### 🔴 BUG-4 — Postit en édition limité à 2 lignes + sélection invisible
- [ ] En mode édition, la `<textarea>` du sticky a `height = (editOverlay.height ?? 120) - 16` mais le contenu peut dépasser → l'utilisateur ne voit que ~2 lignes.
- [ ] **Refonte** : la zone de texte en édition doit avoir EXACTEMENT le même rendu visuel que l'affichage (mêmes dimensions, même police, même padding, même `whiteSpace: pre-wrap`). Pas de dissociation édition/affichage.
- [ ] Si le texte dépasse, le sticky doit s'étendre automatiquement vers le bas (ou afficher une barre de scroll explicite).
- [ ] Indicateur visuel clair de la sélection (bordure colorée + caret bien visible).
- [ ] *Fichiers : `GlucoseCanvas.tsx` ligne 1426 (textarea) + `SvgAnnotationLayer.tsx` `renderSticky`.*

### 🟠 BUG-5 — Pas de feedback (notifications) ✅ partiellement fait
- [x] `Toast.tsx` créé + raccourcis Save/Undo/Redo/Copy/Paste/Cut/Delete branchés dans `App.tsx`.
- [ ] Vérifier que toutes les actions le déclenchent réellement (Ctrl+C/V/X actuellement déclenchent un toast SANS vérifier qu'une vraie sélection existe → faux positifs).
- [ ] Ajouter feedback : création de dossier, ajout d'image, import vidéo, application de preset, etc.

### 🔴 BUG-6 — Double-clic en mode pan = passe en édition + zoom impossible sur texte
- [ ] En mode `pan` (Space) ou `select` quand on déplace, double-clic sur du texte/sticky ouvre l'édition. **DOIT** être désactivé pendant un déplacement pour éviter les éditions accidentelles.
- [ ] Le zoom molette ne doit JAMAIS être bloqué : actuellement `app.canvas.addEventListener("wheel")` fonctionne, mais les annotations SVG ont `pointerEvents: all` et peuvent intercepter le wheel sur les éléments. Vérifier que `pointer-events` ne bloque pas le scroll wheel sur le canvas en dessous (le SVG layer est en `zIndex: 2`).
- [ ] Désactiver `onDoubleClick` dans `SvgAnnotationLayer` quand `activeTool !== "select"` (déjà fait pour le sticky source, à étendre à text/sticky/membrane).
- [ ] Important : le zoom molette doit fonctionner par-dessus n'importe quel élément (texte, postit, dossier).

---

## ✅ Complété

- [x] Canvas infini avec pan/zoom
- [x] Multi-boards avec tabs
- [x] Drag-drop images (local + clipboard)
- [x] Annotations : texte, sticky notes, flèches (straight/curved)
- [x] Presets créatifs (CharaDesign, Environment, Creature, Props, Storyboard, MoodBoard)
- [x] Storyboard avec panels configurables
- [x] Undo/redo (50 niveaux)
- [x] Sauvegarde `.glucose` (JSON)
- [x] Raccourcis clavier (V, T, N, A, Space, F, L, Ctrl+Z/Y/S/O/D/Shift+F)
- [x] **Bug fix** — Texte/sticky pixelisé → rendu à 3× + `scale.set(1/3)`
- [x] **Bug fix** — Outil texte : focus immédiat via `requestAnimationFrame`
- [x] **Bug fix** — Storyboard : scrollbar + placement viewport
- [x] **Bug fix** — Zones preset : interception pan + sélection avant drag
- [x] **Bug fix** — Sauvegarde `.glucose` via commandes Rust directes
- [x] **Bug fix** — Clic molette ne sélectionne jamais rien
- [x] **Bug fix** — Clic molette : plus de saut au premier clic (coord canvas-locales)
- [x] **Performance** — Grille TilingSprite O(1) au lieu de Graphics O(n)
- [x] **Ctrl+D** — Duplication des éléments sélectionnés
- [x] **Ctrl+Shift+F** — Zoom to fit (animation sur tout le contenu)
- [x] **L** — Verrouiller/déverrouiller images (barre contextuelle + raccourci)
- [x] **Tags images** — pills dans barre contextuelle · Enter/virgule = ajouter
- [x] **Waypoints flèches** — handles `+` par segment · drag · Ctrl+clic = supprimer
- [x] **Réordonnancement tabs** — drag-and-drop souris (mousedown/mouseenter/mouseup)
- [x] **Minimap** — radar 180×120px (images, annotations, indicateur viewport)
- [x] **Zen Mode** — touche `F` masque toolbar + tabs
- [x] **Pomodoro** — timer flottant 25/15/5 min, anneau de progression
- [x] **Export PNG** — capture WebGL synchrone → dialogue Tauri → écriture binaire Rust
- [x] **Labels flèches** — texte au milieu, saisie dans ArrowOptions
- [x] **Flèches dynamiques** — suivent les images au déplacement (sourceId/targetId)
- [x] **Flèches bidirectionnelles** — bouton ⇄ dans ArrowOptions
- [x] **OrganizePanel** — 5 modes de disposition + tri par taille/ratio avant disposition
- [x] **Minimap click-to-navigate** — clic/drag sur la minimap pour téléporter le viewport
- [x] **Recherche globale (Ctrl+F)** — textes, sticky notes, tags, noms de boards → jump vers résultat
- [x] **Couleur de sticky/flèche personnalisée** — color picker HSV style Blender (roue + slider valeur + champs H/S/V/Hex)
- [x] **App Bridge — Nœud "Fichier Source"** — drag `.blend`, `.psd`, `.kra`, `.mp4`, etc. → sticky spécial. Double-clic → ouvre le logiciel natif
- [x] **Tri par luminosité** — Sombre→Clair / Clair→Sombre dans OrganizePanel (analyse pixel 32×32)
- [x] **Auto-upgrade résolution CDN** — à l'import d'une image web, tentative automatique de récupérer la version originale/haute-résolution via patterns CDN. Aucune clé API requise.
- [x] **Rendu 100% Vectoriel** — grille CSS radial-gradient, texte/sticky/flèches en SVG overlay synchronisé avec le viewport PixiJS. Zéro pixelisation à n'importe quel zoom.
- [x] **Minimap vectorielle** — état vide rendu HTML (non canvas), pas de `imageRendering: pixelated`
- [x] **Import vidéo YouTube/TikTok/Instagram/Vimeo** — coller ou drag-drop une URL → yt-dlp auto-téléchargé si absent → vidéo jouée en live sur le canvas (muted, loop). Drag de `.mp4/.mov` local également supporté.
- [x] **Preview grille flou presets/storyboard** — miniatures SVG visuelles dans les panneaux + ghost animé sur le canvas au survol (grille floue centrée sur le viewport, pulsation douce)

---

## ✅ PHASE 1 — Fondations *(complète)*

Toutes les fondations sont terminées — voir la section **✅ Complété** ci-dessus.

---

## ✅ PRIORITÉ ARCHITECTURALE — Rendu 100% Vectoriel *(complète)*

- [x] Overlay SVG synchronisé avec PixiJS (texte, sticky, flèches)
- [x] Grille CSS `radial-gradient` (zéro bitmap)
- [x] `vectorEffect="non-scaling-stroke"` sur tous les bordures

---

## ✅ PHASE 1.5 — Import Multimédia Web *(complète)*

- [x] Import vidéo YouTube/TikTok/Instagram/Vimeo (Ctrl+V URL ou drag depuis navigateur)
- [x] yt-dlp auto-téléchargé au premier usage (~12 Mo, mis en cache AppData)
- [x] Vidéo jouée en live sur le canvas PixiJS (muted, loop)
- [x] Drag fichier `.mp4/.mov` local → video sprite directement

---

## 🟠 PHASE 2 — Navigation & Structure

### A. Minimap Tactique
- [x] **Radar en bas à droite** — minimap 180×120px
- [x] **Clic sur la minimap** — clic/drag sur la minimap pour s'y téléporter

### B. Sélecteur de Zone *(fondamental — prérequis pour tout le reste)*

> **Principe :** Quand l'utilisateur crée quelque chose qui occupe de l'espace (preset, storyboard, dossier), au lieu d'apparaître à une position aléatoire, il dessine lui-même la zone sur le canvas.
> Le coin **haut-gauche** est le point le plus important — c'est l'ancrage principal.
> Le coin **bas-droit** définit l'étendue de la zone.
> La zone peut toujours être redimensionnée après création.

- [ ] **Mode "Sélection de zone"** — outil temporaire activé automatiquement avant certaines actions (créer preset, créer storyboard, créer dossier)
- [ ] **Overlay d'instruction clair** — message visible à l'écran : "Dessine la zone de ton [storyboard / dossier]" + coin haut-gauche mis en valeur visuellement
- [ ] **Rectangle de prévisualisation** — pendant le drag, afficher un rectangle SVG semi-transparent avec les dimensions en temps réel
- [ ] **Validation et annulation** — clic droit ou Échap = annuler ; relâcher le bouton = confirmer
- [ ] **Redimensionnement post-création** — poignées sur les bords de la zone pour l'ajuster après placement
- [ ] *Technique : mode canvas spécial → enregistre `{x1, y1, x2, y2}` en coordonnées monde → transmis à la création de l'élément*

### C. Dossiers Zoomables *(feature signature)*

> **Concept :** Un dossier n'est pas une liste — c'est un espace. Tu zoomes dessus et tu y entres.
> À l'intérieur se trouve un nouveau canvas infini où tu peux mettre des images, des notes, et d'autres dossiers.
> Tu peux créer une infinité de sous-dossiers. Pour sortir, tu dézoomez ou tu appuies sur Backspace.
> La navigation est entièrement spatiale et visuelle.

- [ ] **Création par Sélecteur de Zone** *(voir B ci-dessus)* — dessiner la zone = créer le dossier
- [ ] **Design du dossier** — carte visuellement distincte avec : nom éditable, aperçu miniature du contenu, couleur personnalisable, bord arrondi + ombre douce
- [ ] **Compteur de contenu** — affiche le nombre d'éléments à l'intérieur (ex: "12 éléments")
- [ ] **Zoom-in → Entrée** — quand le viewport atteint le dossier à un certain seuil de zoom, transition animée vers le canvas intérieur
- [ ] **Canvas intérieur infini** — identique au canvas principal, supporte images, sticky, flèches, et sous-dossiers
- [ ] **Navigation breadcrumb** — en haut de l'écran : `Projet > Personnages > Héros > Costumes`
- [ ] **Zoom-out → Sortie** — dézoomer en dessous du seuil de sortie = retour au canvas parent avec animation inverse
- [ ] **Aperçu live** — le dossier sur le canvas parent montre une miniature du contenu intérieur (rendu statique mis à jour à la sortie)
- [ ] *Technique : chaque dossier = une Board enfant liée à la board parente. Le viewport surveille le facteur de zoom et la position sur le dossier pour déclencher la transition.*

### D. Ghost Branches — Calques d'idées *(feature signature)*
- [ ] **Calque "idée folle"** — ajouter un calque par-dessus le board actuel (tout y est invisible depuis le board principal)
- [ ] **Toggle on/off** — activer/désactiver le calque en un clic
- [ ] **Merge avec le board** — si le calque plaît, le fusionner avec le board principal
- [ ] *Implémentation : boards "enfants" liés à un board parent, avec un indicateur de calque actif*

### E. Groupement d'éléments
- [ ] **Grouper/dégrouper** — sélectionner plusieurs éléments, les regrouper en bloc déplaçable
- [ ] **Dossiers visuels légers** — cadre conteneur simple sans zoom-in (différent des Dossiers Zoomables)

---

## 🟡 PHASE 3 — Automatisation & Intelligence Locale

### Smart Masonry 2.0
- [x] **5 modes de disposition** — compact, masonry, grid, sameHeight, bySlot
- [x] **Tri par taille et ratio** — avant disposition dans OrganizePanel
- [ ] **Tri par chromatographie** — regrouper images par couleur dominante (analyse pixel canvas 2D)
- [x] **Tri par luminosité** — du plus clair au plus sombre (analyse pixel 32×32)

### Semantic Clustering *(feature signature)*
- [ ] **Grouper par Sujet** — modèle CLIP local (via crate `candle` de HuggingFace en Rust) classe les images automatiquement : cyber-goth à gauche, paysages naturels à droite — sans tag manuel
- [ ] **Visualisation des clusters** — dessiner des zones colorées autour de chaque groupe détecté
- [ ] *Technique : `candle` + CLIP ViT-B/32, inférence locale en Rust, ~150MB*

### Extracteur de Palette Visuelle
- [ ] **Générer palette depuis sélection** — extraire les couleurs dominantes (k-means sur pixels)
- [ ] **Sticky note palette** — créer automatiquement un post-it avec les swatches de couleur
- [ ] ~~Export .ase / CSS variables~~ → *déplacé en Idées Futures (gadget de logiciel de design)*

---

## 🟢 PHASE 4 — Visualisation & Fléchage

### Liens Dynamiques
- [x] **Flèches qui suivent les images** — endpoints recalculés automatiquement
- [x] **Snap aux bords intelligents** — perimeter snap
- [x] **Flèches bidirectionnelles** — A ↔ B
- [x] **Labels sur les flèches** — texte au milieu
- [ ] **Types de liens** — "inspire", "précède", "contredit", "hérite de" (badge coloré sur la flèche)
- [ ] **Flèches qui relient boards** — une flèche peut pointer vers un élément dans un autre board (portail léger)

### App Bridge *(voir aussi Phase 1)*
- [x] **Nœud "Fichier Source"** — sticky spécial pour `.blend`, `.psd`, `.kra`, `.mp4`
- [x] **Ouverture native** — crate `open` Rust avec chemin absolu du fichier
- [x] **Badge du logiciel** — affichage de l'extension dans le sticky source
- [ ] **Icône reconnaissable par logiciel** — icône Blender/Photoshop/Krita réelle (SVG bundlé)

### Node-Logic *(post-its intelligents)*
- [ ] **Post-it "opérateur"** — type logique (ET, OU, MAIS, PARCE QUE)
- [ ] **Synthèse de connexions** — combien d'éléments liés à un nœud
- [ ] **Vue graphe** — basculer vers une vue de relations (comme Obsidian)

---

## 🔵 PHASE 5 — Versioning & Time Machine

> **Décision architecturale :** On utilise **Automerge (CRDT)** comme moteur interne de données dès le début.
> Cela donne nativement : Undo/Redo infini, Time Machine, et multi-utilisateurs.
> Git est utilisé **uniquement** pour versionner les gros assets externes (via Git LFS).
> Cette approche évite le conflit architectural Git vs CRDT — ne pas coder les deux.

### Fondations Automerge
- [ ] **Migrer le store Zustand vers Automerge** — chaque mutation devient un op CRDT
- [ ] **Undo/Redo infini** — natif avec Automerge (remplace le système actuel 50-entrées)
- [ ] **Sauvegarde binaire** — `.glucose` devient un fichier Automerge binaire (plus compact)

### Time Machine *(feature signature)*
- [ ] **Historique complet** — chaque session est un snapshot CRDT timestampé
- [ ] **Time-Scrubbing UI** — slider horizontal en bas du canvas : glisser = voir les éléments apparaître/disparaître sur PixiJS en temps réel
- [ ] **Revenir à un instant T** — restaurer le board à n'importe quel état passé

### Commit Visuel
- [ ] **Ctrl+Shift+S** — "Instantané de ma pensée" avec un message court
- [ ] **Timeline des commits** — panneau historique des snapshots nommés

### Git LFS pour les Assets
- [ ] **Assets en chemin relatif** — `.glucose` ne stocke plus les images en base64 inline (gain massif de performance save/load)
- [ ] **Git LFS transparent** — gérer les gros fichiers (PSD, vidéos) via `git2` crate Rust

---

## 🟣 PHASE 6 — Intelligence Artificielle (RAG Local)

### Base de Connaissance Locale
- [ ] **Indexation vectorielle** — embedder images + notes dans une BDD vectorielle locale
- [ ] **Recherche sémantique** — "retrouve l'idée sur l'architecture japonaise d'il y a 3 mois"
- [ ] **Recherche par image** — drag une image pour trouver les références similaires
- [ ] *Technique : qdrant + CLIP embeddings (candle Rust)*

### Multimodalité Locale *(remplace l'OCR Tesseract — supprimé)*
- [ ] **Légendes automatiques** — Moondream2 décrit chaque image (texte ET contexte, pas juste OCR brut)
- [ ] **Tagging automatique** — générer tags pertinents (style, sujet, couleurs, ambiance)
- [ ] **Extraction de texte intelligente** — Moondream lit les tweets, scans, screenshots — avec compréhension du contexte (remplace Tesseract qui échouait sur les typographies stylisées)
- [ ] *Technique : Moondream2 ~2GB, inférence locale via `candle`*

### Ligne de Commande Visuelle *(feature signature)*
- [ ] **Commande naturelle** — taper une commande ouvre un prompt :
  - "Glucose, range-moi tout ce qui est bleu à gauche"
  - "Glucose, crée un fork de cette idée en version cyberpunk"
  - "Glucose, retrouve-moi toutes mes références de forêts"
- [ ] **Historique de commandes** — revoir et rejouer
- [ ] *Technique : Claude Haiku-4-5 pour les commandes + RAG local comme contexte*

---

## ⚫ PHASE 7 — Collaboration, Distribution & Multiplateforme

### Multi-plateforme *(objectif universel)*

> **Cible :** Atelier doit tourner sur TOUS les systèmes — desktop et mobile.

- [ ] **Linux** — build Tauri AppImage / .deb (déjà supporté par Tauri, tester les permissions fs)
- [ ] **macOS** — build Tauri .dmg / .app (déjà supporté, tester App Bridge avec `open`)
- [ ] **Windows** — ✅ plateforme de dev principale
- [ ] **Android** — Tauri 2.0 mobile target (`tauri android init`), rendu WebView + canvas
- [ ] **iOS** — Tauri 2.0 mobile target (`tauri ios init`), contraintes App Store à anticiper
- [ ] *Technique : Tauri 2.0 unifie desktop + mobile. Canvas PixiJS/WebGL fonctionne dans WebView mobile via ANGLE.*

### Compte Utilisateur & Cloud Sync

> **Objectif :** L'utilisateur se connecte une fois et retrouve ses dossiers sur n'importe quel appareil.

- [ ] **Compte Atelier** — email + mot de passe ou OAuth (Google, GitHub)
- [ ] **Sync du projet actif** — les dossiers, images et notes se synchronisent via le cloud
- [ ] **Mode hors-ligne** — tout fonctionne sans connexion, sync différée au retour du réseau
- [ ] **Stockage cloud assets** — images et vidéos uploadées en arrière-plan (chiffrement côté client)
- [ ] *Technique : backend léger (Supabase ou Cloudflare R2 + D1) + auth JWT. CRDT Phase 5 gère les conflits de sync.*

### Share vers Atelier *(mobile → app)*
- [ ] **Android Share Target** — Atelier apparaît dans la liste "Partager avec" native Android
- [ ] **iOS Share Extension** — extension iOS qui reçoit les URLs/images depuis Safari, Instagram, TikTok
- [ ] **Réception automatique** — l'URL est envoyée à l'app qui lance l'extraction (vidéo) ou import (image)
- [ ] **Panneau "À trier"** — file d'entrée des éléments reçus, à ranger dans un dossier

### Synchronisation P2P via CRDT *(Phase 5 requis)*
- [ ] **P2P natif** — Automerge (Phase 5) envoie les ops en P2P chiffré
- [ ] **Réseau LAN automatique** — synchro locale sans internet
- [ ] **Réseau P2P global** — type Resilio, sans serveur central
- [ ] **Collaborateurs en temps réel** — avatars sur la planche

### Export & Interopérabilité
- [x] **Export PNG du canvas** — bouton toolbar, dialogue Tauri, écriture binaire Rust
- [ ] **Export PDF storyboard** — panels storyboard en PDF partageable
- [ ] **Export Markdown** — convertir notes et structure en fichiers .md
- [ ] **Import depuis Obsidian** — lire les vaults Obsidian existants

---

## 🎯 Prochaines Étapes Recommandées *(2026)*

Par ordre de priorité — les 3 premières débloquent toute la vision :

1. 🔴 **[Bug] App Bridge** — déboguer `open_in_app` + affichage du nom de fichier dans le sticky
2. 🔶 **Couche SVG** *(Architecture Vectorielle)* — refonte annotations/grille sur overlay SVG, prérequis pour zoom infini propre
3. 🟠 **Sélecteur de Zone** *(Phase 2-B)* — UX fondamentale, prérequis pour Dossiers et placement Preset/Storyboard
4. 🟠 **Dossiers Zoomables** *(Phase 2-C)* — feature signature, dépend du Sélecteur de Zone
5. 🟤 **Import vidéo web** *(Phase 1.5)* — bundler `yt-dlp`, lecteur PixiJS VideoResource
6. 🟡 **Groupement d'éléments** *(Phase 2-E)* — aucune dépendance architecturale, implémentable rapidement
7. 🟡 **Tri par chromatographie** *(Phase 3)* — k-means canvas 2D, complète OrganizePanel
8. 🟢 **Types de liens sur flèches** *(Phase 4)* — badge coloré, données déjà dans le type `Annotation`

---

## 🐛 Bugs Connus

### App Bridge — Fichiers Externes *(priorité haute)*
- [ ] **Les fichiers `.blend` / `.psd` / `.kra` ne s'affichent pas correctement** — le sticky créé est vide, le nom/extension du fichier n'est pas visible pour l'utilisateur
- [ ] **Double-clic ne lance pas l'application native** — la commande `open_in_app` Rust ne trouve pas l'association fichier/logiciel, ou le chemin absolu stocké est incorrect
- [ ] *Piste de débogage : logger le chemin reçu dans `open_in_app`, vérifier que l'annotation `sourceFile` contient bien le chemin absolu complet, tester avec un `.blend` et Blender installé sur Windows*

---

## 💡 Idées Futures (Non Priorisées)

- **Export .ase / CSS variables** — format palette Photoshop/Illustrator (déplacé depuis Phase 3)
- **OCR Tesseract classique** — supprimé de Phase 3 : remplacé par Moondream2 en Phase 6 (meilleure compréhension, typographies stylisées, contexte)
- **Mode Langue** — canvas spécialisé conlang : phonologie, morphologie, lexique visuel
- **Mode Roman** — vue chapitres, structure narrative, fiches personnages
- **Mode Manga/BD** — découpage de planches avec export prêt impression
- **Timeline narrative** — axe horizontal avec scènes, actes, arcs narratifs
- **Tableau de bord projet** — vue synthèse : progression, éléments actifs
- **Mode présentation** — parcours guidé sur la planche (comme des slides)
- **Thème clair** — toggle dark/light mode
- **Raccourcis personnalisables** — rebind des touches
- **Plugins** — système d'extensions tierces

---

## Notes Techniques

| Sujet | Détail |
|-------|--------|
| StrictMode | Désactivé intentionnellement (`main.tsx`) — évite double-init PixiJS |
| Format sauvegarde | `.glucose` (JSON) → Phase 5 : binaire Automerge |
| Undo/redo actuel | 50 entrées max — `store/index.ts` — Phase 5 : infini via CRDT |
| Canvas | PixiJS 8.x (WebGL, raster) + SVG overlay à venir (vectoriel) |
| IA recommandée | Claude Haiku-4-5 pour commandes, Moondream2 pour vision locale (Phase 6) |
| Vectorisation | CLIP + qdrant via `candle` (Rust) |
| CRDT | crate `automerge` (Rust) — moteur principal Phase 5+ |
| Git | crate `git2` (Rust) — uniquement pour LFS assets Phase 5 |
| Semantic Clustering | CLIP ViT-B/32 via `candle` — Phase 3 |
| App Bridge | crate `open` Rust — ✅ implémenté (bug affichage à corriger) |
| Import vidéo web | `yt-dlp` binary spawné via Tauri + `PIXI.VideoResource` — Phase 1.5 |
| Dossiers Zoomables | Board enfant + transition viewport au seuil de zoom — Phase 2-C |
| Multiplateforme | Tauri 2.0 (desktop ✅ + mobile Android/iOS à venir) |
| Cloud Sync | Supabase / Cloudflare R2+D1 + auth JWT — Phase 7 |
