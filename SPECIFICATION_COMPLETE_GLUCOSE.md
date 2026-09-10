# 🧬 Spécification Technique & Fonctionnelle Exhaustive de Glucose (100% Code Source)

> **Document de référence absolue issu de l'analyse intégrale de chaque script `.ts`, `.tsx` et `.rs` du projet Glucose.**  
> *Ce document ne se base pas sur les anciens fichiers `.md` mais exclusivement sur le code source réel.*  
> *Il détaille avec précision mathématique et algorithmique le rôle, les mécanismes, les règles métier et les invariants de chaque script afin de guider la réécriture architecturale en Rust pur.*

---

## 📑 Sommaire
1. [Les 4 Piliers Fondamentaux de Glucose](#1-les-4-piliers-fondamentaux-de-glucose)
2. [Modèle de Données & Schémas (`src/types`, `src/store`)](#2-modèle-de-données--schémas)
3. [Moteur Canvas & Couches de Rendu (`src/canvas`)](#3-moteur-canvas--couches-de-rendu)
4. [Algorithmes Géométriques, Spatial & Snapping (`src/canvas`)](#4-algorithmes-géométriques-spatial--snapping)
5. [Composants d'Interface & Modales (`src/components`)](#5-composants-dinterface--modales)
6. [Utilitaires, Moteurs & Exportateurs (`src/utils`)](#6-utilitaires-moteurs--exportateurs)
7. [Collaboration Multi-utilisateur & Télémétrie (`src/multiplayer`, `src/telemetry`)](#7-collaboration-multi-utilisateur--télémétrie)
8. [Services Système Natifs Rust (`src-tauri/src`)](#8-services-système-natifs-rust)
9. [Matrice de Traduction & Roadmap Rust Pur](#9-matrice-de-traduction--roadmap-rust-pur)

---

## 1. Les 4 Piliers Fondamentaux de Glucose

L'analyse du code source original met en lumière 4 mécaniques centrales qui différencient Glucose d'un simple visualiseur d'images :

### 1.1. Le Biome Chromatique Symbiotique (`symbioticHue.ts`)
Contrairement à un thème fixe sombre ou clair, Glucose est divisé en **biomes de couleur organiques** à l'échelle de l'espace infini :
- **Bruit 2D continu** : la position $(x, y)$ sur le canvas génère une teinte de base ($0^\circ$ à $360^\circ$) par interpolation cubique `smooth(t) = t*t*(3-2*t)` sur une grille de **2000 pixels**.
- **Signature unique** : chaque carte dérive son angle de base avec un hash de son identifiant `((idHash(id) % 80) - 40)`.
- **Attraction trigonométrique des voisines (rayon 1200px)** : chaque carte dans un rayon de 1200px attire la teinte par une moyenne vectorielle circulaire :
  $$\sum X = \cos(\theta) \cdot (1 - d/1200)^2, \quad \sum Y = \sin(\theta) \cdot (1 - d/1200)^2$$
- **Résultat visuel** : des colonnes et régions entières aux reflets harmonieux (ambre, émeraude, cyan, violet) où chaque carte projette un halo doux `color-mix(in srgb, ${auraColor} 15%, transparent)` avec un flou de 60px à 80px.

### 1.2. Les Membranes Adaptatives (`SvgAnnotationLayer.tsx`, `membraneSpace.ts`, `membraneStretch.ts`)
Les membranes ne sont pas de simples boîtes :
- **Dessin vectoriel** : rectangle à coins très arrondis ($rx = 60\text{px}$), bordure pointillée fine (ou continue quand sélectionnée), remplissage quasi transparent ($2\%$), et **étiquette de titre posée en haut à gauche à $x=16, y=-12$** avec un contour sombre ($4\text{px}$) par-dessus la bordure.
- **Facteur d'échelle interne $k = \min(1, \dots)$** : en mode `Minimized`, tout le contenu interne est automatiquement redimensionné sans modifier ses coordonnées stockées.
- **Étirement anti-collision (`membraneStretch.ts`)** : si du contenu est ajouté ou déplacé, la membrane grandit dynamiquement mais **bute sur les éléments extérieurs** en émettant une alerte visuelle (`MembraneStretchAlert`).
- **Mode Focus asymétrique (`membraneFocus.ts`)** : zoomer à $\ge 92\%$ de la membrane active le mode focus plein écran avec estompage du reste du canvas ; dézoomer à $\le 80\%$ en ressort.

### 1.3. L'Édition de Texte en Place & Markdown Vivant (`HtmlAnnotationLayer.tsx`, `GlucoseCanvas.tsx`)
- **Double-clic d'édition** : double-cliquer sur une carte ouvre instantanément un textarea en superposition exacte (`editOverlay`), avec alignement de police, respect de la taille et capture du focus clavier.
- **Rendu Markdown enrichi** : prise en charge des titres `#`, listes, blocs de code avec coloration syntaxique (`MdPre`, `MdCode`), et formules mathématiques LaTeX via KaTeX.
- **Ancrage textuel W3C (`textAnchors.ts`)** : les flèches relationnelles peuvent pointer vers un paragraphe précis ou une citation textuelle d'un document. Si le texte est modifié plus haut, l'ancre se recalcule sans se casser.

### 1.4. Rendu Vectoriel Crisp & Élimination de la Pixelisation
- Les icônes de la barre d'outils et du canvas sont définies par des coordonnées vectorielles précises (SVG `viewBox="0 0 14 14"` ou `16 16`, `strokeWidth=1.2` à `1.5`, `strokeLinecap="round"`, `strokeLinejoin="round"`).
- L'interface ne doit sous aucun prétexte utiliser de polices matricielles 8x8 ou d'icônes bitmap étirées : chaque trait, cercle, flèche et texte est vectorisé et anti-aliasé à n'importe quel facteur d'échelle.

---

## 2. Modèle de Données & Schémas

### `src/types/index.ts`
Définit l'intégralité du modèle de données de Glucose :
- **`Project`** : contient `version` (format v1/v2/v3), `activeBoardId`, tableau `boards: Board[]`, `domains: Domain[]`, `presets: Preset[]`, `blobs: Record<string, string>`, `createdAt`, `updatedAt`.
- **`Board`** : espace de travail indépendant avec `id`, `name`, `viewport: { x, y, scale }`, `images: BoardImage[]`, `annotations: Annotation[]`, `folders: Folder[]`, `panels: StoryboardPanel[]`, `storyboard: boolean`, `presetId?: string`.
- **`BoardImage`** : `id`, `x`, `y`, `width`, `height`, `rotation`, `locked`, `tags: string[]`, `slotId?: string`, `sourceUrl?: string`, `originalWidth`, `originalHeight`, `fit?: "contain" | "cover"`, `asset?: AssetRef`, `src?: string`, `mirrorOf?: string`, `temporalAnchor?: TemporalAnchor`.
- **`Annotation`** : union discriminée stricte :
  - `Text` : texte Markdown, dimensions optionnelles, police, couleur, `sourceFile?: string` (fichier lié sur disque), domaines.
  - `Sticky` : pense-bête coloré, `operator?: "AND" | "OR" | "BUT" | "BECAUSE"`, `bgColor`, `author`.
  - `Arrow` : flèche orientée de $(x, y)$ à $(x2, y2)$, waypoints intermédiaires, `predicate?: string`, `sourceId`, `targetId`, `sourceBlockId`, `targetBlockId`, ancres textuelles, `arrowBidirectional`.
  - `Membrane` : conteneur de groupe, dimensions, `mode: "Classic" | "Minimized" | "Stretched"`, rideaux `curtains: MembraneCurtain[]`.
- **`Folder`** : sous-canvas imbriqué avec `id`, `name`, `x`, `y`, `width`, `height`, `childBoardId`.
- **`Domain`** : classification thématique avec `id`, `name`, `color`.
- **`TemporalAnchor`** : datation historique d'un nœud (année astronomique de -100 millions d'années à +3000).

### `src/store/index.ts`
Store réactif central (Zustand dans l'original) :
- **Gestion de l'historique (Undo / Redo)** : pile d'annulation limitée à 200 entrées (`maxUndo: 200`). Préservation absolue de la caméra et du board actif (`preserveView`) lors d'un `undo` ou `redo` : annuler un déplacement d'objet ne téléporte JAMAIS la caméra de l'utilisateur.
- **Transaction Live-Edit (`beginLiveEdit`, `endLiveEdit`)** : pendant un glissement à la souris, un seul snapshot est enregistré au premier mouvement réel, évitant de polluer l'historique avec 60 micro-étapes par seconde.
- **Mutations complètes** : ajout, mise à jour, suppression, duplication décalée (+20px) de tous les éléments.
- **Navigation dans les dossiers** : pile de navigation `folderStack` pour descendre dans les sous-dossiers (`enterFolder`) et remonter (`exitFolder`, `exitToRoot`).

### `src/store/projectSchema.ts`
Schéma de validation Zod et migrations de formats de fichiers :
- Valide la conformité structurelle des fichiers `.glucose` lors du chargement.
- Gère la migration transparente depuis les anciennes versions v1 et v2 vers la version v3 moderne (normalisation des champs `domains`, `assetRef`, `fit`, etc.).

### `src/store/mirrorGraph.ts`
Moteur de graphe pour miroirs et dossiers imbriqués :
- Algorithme de parcours en largeur (BFS) avec ensemble de visite (`visited`) pour **détecter et interdire les cycles récursifs** (anti-effet Inception).
- Empêche qu'un dossier devienne son propre ancêtre ou qu'un miroir référence une boucle infinie.

---

## 3. Moteur Canvas & Couches de Rendu

### `src/canvas/GlucoseCanvas.tsx`
Orchestrateur principal du canvas infini (3500+ lignes) :
- **Gestion du Viewport** : conversion coordonnées écran $\leftrightarrow$ monde, zoom logarithmique centré sous le curseur (`MIN_SCALE = 0.02`, `MAX_SCALE = 20`), pan par clic milieu, clic droit ou barre Espace.
- **Grille de points infinie** : motif discret espacé de 60px (`DOT_GRID_SIZE = 60`).
- **Niveaux de détail (LOD)** : pour les images, calcul de la résolution requise selon l'échelle d'affichage afin d'économiser la mémoire vive et ré-affinage dynamique lors des zooms avant.
- **Sélection élastique (Marquee)** : tracé rectangulaire translucide pour sélectionner des groupes d'éléments au relâchement.
- **Superposition d'édition en place (`editOverlay`)** : positionnement précis d'un champ d'édition textuelle lors du double-clic sur un texte, post-it ou membrane.
- **Curseur infini (`cursorWrap.ts`)** : téléportation invisible du curseur souris d'un bord à l'autre de l'écran lors des déplacements continus.

### `src/canvas/HtmlAnnotationLayer.tsx`
Couche de rendu des cartes textuelles, notes post-it et tuiles de fichiers sources :
- **Intégration du Biome Symbiotique** : calcule l'aura chromatique via `getSymbioticHue(ann, annotations)` et applique les dégradés et ombres portées douces.
- **Rendu Markdown & LaTeX** : mise en forme dynamique des titres, puces, citations, blocs de code avec syntaxe VSCode et formules KaTeX.
- **Surlignage au survol des flèches** : lorsqu'une flèche relationnelle est survolée, les blocs ou fragments de texte précis reliés par cette flèche s'illuminent en surbrillance instantanément.
- **Tuiles de fichiers de code et 3D (`SourceFileTile`)** : cartes compactes avec badge de statut du fichier (présent ou lien rompu), icône de l'application associée (Blender, Photoshop, VSCode), et lancement externe au double-clic.

### `src/canvas/SvgAnnotationLayer.tsx`
Couche vectorielle SVG dédiée aux membranes :
- **Tracé de la frontière** : rectangle arrondi à $rx=60$, bordure pointillée `10 10` ou continue si sélectionnée, couleur thématique du domaine ou couleur libre.
- **Halo diffus** : double filtre de lueur externe (`feGaussianBlur stdDeviation="25"`) pour matérialiser la zone sans masquer les cartes intérieures.
- **Titre flottant** : affichage textuel en $x=16, y=-12$ avec contour noir protecteur de 4px (`paintOrder="stroke"`) assurant une lisibilité parfaite sur tout arrière-plan.
- **Poignées de redimensionnement** : 4 poignées aux coins déclenchant le redimensionnement magnétique (SNAP-1).

### `src/canvas/ArrowSvgLayer.tsx`
Couche vectorielle SVG des flèches et liens sémantiques :
- **Courbes de Bézier & Droites** : calcul de la trajectoire optimale entre les deux points d'ancrage avec contournement élégant des cartes.
- **Interpolation de couleur symbiotique** : le dégradé le long de la flèche part de la teinte symbiotique de la source pour rejoindre celle de la cible.
- **Prédicats sémantiques** : badge textuel centré sur la flèche affichant la relation (`inspire`, `contredit`, `dépend_de`, etc.) avec menu de sélection au clic.
- **Pointes de flèches orientées** : têtes triangulaires ou bidirectionnelles orientées précisément selon la tangente d'arrivée.

### `src/canvas/FolderSvgLayer.tsx`
Couche vectorielle des dossiers (sous-canvas) :
- **Aperçu miniature du sous-canvas** : dessine les vignettes miniatures des images et notes contenues à l'intérieur du dossier.
- **Double-clic d'immersion** : plonge la caméra dans le sous-board avec animation fluide de zoom.
- **Glisser-déposer vers dossier** : déposer un élément sur la tuile du dossier l'intègre directement dans le sous-board.

### `src/canvas/CurtainCanvas.tsx` & `MembraneCurtainLayer.tsx`
Système de rideaux de membranes :
- Panneaux latéraux coulissants attachés aux membranes pour prendre des notes privées ou collaboratives sans encombrer l'espace visuel principal.
- Niveaux de visibilité (Privé, Partagé) et de permissions (Auteur seul ou Tout le monde).

---

## 4. Algorithmes Géométriques, Spatial & Snapping

### `src/canvas/hitPriority.ts` & `pickArbiter.ts`
Arbitre de sélection déterministe en 7 rangs stricts (PICK-1) :
- **Rang 1** : Poignées de redimensionnement (zone de tolérance généreuse de 36px).
- **Rang 2** : Bordure active du conteneur (périphérie de membrane ou de dossier).
- **Rang 3** : Flèches vectorielles et leurs badges de prédicats.
- **Rang 4** : Images.
- **Rang 5** : Post-its / Stickies.
- **Rang 6** : Cartes de texte Markdown.
- **Rang 7** : Fond de membrane / conteneur parent.
- **Cyclage de sélection** : cliquer plusieurs fois au même endroit cycle séquentiellement entre les éléments empilés.

### `src/canvas/smartAlign.ts` & `smartAlignRuntime.ts`
Moteur d'alignement intelligent et magnétisme (SNAP-1) :
- **Guides horizontaux et verticaux** : détection des alignements sur les bords gauche, centre et droit des boîtes environnantes.
- **Seuil d'attraction constant à l'écran** : seuil de capture fixé à 8 pixels écran, indépendamment du niveau de zoom de la caméra.
- **Affichage dynamique des guides** : lignes vectorielles roses traversant le canvas lors des accrochages.

### `src/canvas/membraneSpace.ts`
Algorithme d'espace local et d'échelle déduite pour membranes :
- Calcule le facteur d'échelle $k = \min(1, \dots)$ en fonction de la taille de la membrane par rapport à son contenu.
- Met à jour l'appartenance automatique des nœuds (`reconcileMembership`) selon leur inclusion géométrique.

### `src/canvas/membraneStretch.ts` & `membraneStretchRuntime.ts`
Planificateur d'étirement élastique sans collision :
- Calcule l'agrandissement d'une membrane pour englober son contenu qui s'étend.
- Détecte les obstacles extérieurs (autres images, cartes, membranes) et bloque l'expansion pour éviter tout chevauchement non désiré.

### `src/canvas/membraneFocus.ts`
Mode focus asymétrique :
- Transition visuelle continue lorsque l'utilisateur cadre une membrane spécifique.
- Calcul de cadrage optimal avec marges esthétiques et assombrissement ciblé du monde extérieur.

### `src/canvas/membraneTween.ts`
Interpolation et animations de transition douces :
- Calcul des trajectoires avec amortissement cubique pour les entrées/sorties de focus et les bascules de mode.

### `src/canvas/dropHandler.ts` & `fileImport.ts`
Classification intelligente des dépôts de fichiers :
- Détecte le type MIME ou l'extension et route automatiquement vers le bon type d'objet (Image pour PNG/WebP/JPG, TextAnnotation pour MD/Code/TXT, SourceFileTile pour 3D/Audio/Vidéo/Projets, FolderTree pour dossiers).

### `src/canvas/imageResize.ts`
Calcul du redimensionnement d'images avec préservation stricte du ratio d'aspect initial ou liberté selon les touches modificatrices (Shift).

### `src/canvas/navigation.ts`
Fonctions de caméra pour centrer la vue sur la sélection, recentrer sur l'origine ($x=0, y=0, scale=1.0$) ou zoomer vers un rectangle d'intérêt (`fitBounds`).

### `src/canvas/Quadtree.ts`
Table de hachage spatiale (`SpatialHash`) découpant l'espace infini en cellules pour le test d'intersection et le culling ultra-rapide des éléments hors écran.

---

## 5. Composants d'Interface & Modales

### `src/components/Toolbar.tsx`
Barre d'outils supérieure fixe (hauteur 44px, fond `#1A1A1A`, bordure `#2A2A2A`) :
- Logo textuel **GLUCOSE** blanc en majuscules grasses.
- Groupe d'outils : `Select` (`V`), `Pan` (`Espace`), `Text` (`T`), `Sticky` (`N`), `Arrow` (`A`), `Folder` (`F`), `Membrane` (`M`).
- Bouton `+ Images` avec déclenchement du dialogue natif (`rfd`).
- Boutons d'action : `Ordonner`, `Timer`, `Storyboard`, `Aimant` (toggle SNAP-1), `Trans-domaines`.
- Menu latéral droit : `Collaborer` (indicateur LED verte de connexion LAN), `Exporter`, `Plugins`, `Preset`, `Domaines`.

### `src/components/BoardTabs.tsx`
Barre d'onglets inférieure (hauteur 34px, fond `#111111`) :
- Liste des onglets de boards avec titre éditable.
- Soulignement blanc net de 2px sous l'onglet actuellement actif.
- Bouton `+` pour ajouter un nouveau board indépendant avec sa propre caméra et son propre canvas.

### `src/components/BoardMinimap.tsx` & `FloatingMinimap.tsx`
Minimap interactive (180x120px) :
- Affichage en réduction de toutes les images, cartes et membranes du board.
- Rectangle blanc translucide représentant le champ de vision exact de la caméra.
- Clic ou glissement direct sur la minimap pour téléporter ou déplacer la caméra instantanément.

### `src/components/Toast.tsx`
Système de notifications flottantes au centre en bas d'écran avec animations d'apparition et de disparition douces pour confirmer les opérations (`Image collée`, `Canvas ordonné`, etc.).

### `src/components/ContextMenu.tsx`
Menu contextuel natif au clic droit :
- Couper, Copier, Coller, Dupliquer, Supprimer.
- Grouper dans une nouvelle membrane, Créer un sous-dossier, Mettre au premier plan, Mettre à l'arrière-plan.

### `src/components/DomainsPanel.tsx`
Gestionnaire de domaines thématiques :
- Création de domaines personnalisés avec palette de couleurs associée.
- Attribution d'un domaine aux cartes et filtrage de visibilité.

### `src/components/ExportMenu.tsx` & `ExportModal.tsx`
Interface d'exportation multi-formats :
- Export HTML interactif autonome (visualisable dans tout navigateur sans installation).
- Export PNG Haute Définition avec cadrage automatique.
- Export SVG vectoriel pur et Markdown structuré par zones.

### `src/components/TimelinePanel.tsx` & `TemporalRuler.tsx`
Réglette temporelle astronomique :
- Permet d'ancrer des cartes et images sur une échelle de temps de -100 millions d'années à l'an 3000.
- Filtrage interactif masquant ou atténuant les éléments hors de l'époque sélectionnée.

### `src/components/StoryboardControls.tsx`
Contrôleur de séquences et de storyboard :
- Définition de panneaux de plans avec caméra mémorisée, description et durée.
- Lecture pas à pas ou continue pour présenter un projet ou animer un pitch.

### `src/components/PresetPanel.tsx`
Sélecteur de dispositions prédéfinies :
- Presets de pipelines artistiques (CharaDesign, Environment, Props, Storyboard).
- Assignation rapide de tags et de couleurs selon le rôle de l'image (Référence, Croquis, Lineart, Final).

### `src/components/SearchPanel.tsx`
Recherche globale instantanée dans tous les boards par mots-clés, tags, types ou dates.

### `src/components/PomodoroTimer.tsx` & `PomodoroOverlay.tsx`
Chronomètre de concentration intégré (session de 25 minutes) avec alerte visuelle discrète.

---

## 6. Utilitaires, Moteurs & Exportateurs

### `src/utils/symbioticHue.ts`
Algorithme de calcul pur de la teinte organique :
- Générateur de bruit pseudo-aléatoire 2D deterministe basé sur le sinus et produit scalaire.
- Lissage cubique pour supprimer les discontinuités entre les cellules de 2000px.
- Calcul trigonométrique des moyennes d'angles évitant les anomalies au passage de $360^\circ \rightarrow 0^\circ$.

### `src/utils/textAnchors.ts`
Moteur d'ancrage sub-block W3C :
- Analyse le texte en plain-text et génère des descripteurs avec préfixe, citation exacte et suffixe.
- Recalcule la position exacte de l'ancre même si l'utilisateur insère ou supprime des lignes avant ou après la citation.

### `src/utils/layout.ts`
Moteur d'auto-organisation automatique :
- `gridLayout` : grille carrée proportionnelle sans étirement.
- `compactRowsLayout` : disposition compacte en rangées type maçonnerie respectant les ratios d'aspect.
- `timelineLayout` & `spiralLayout` : arrangements chronologiques et concentriques.

### `src/utils/bundle.ts` & `bundleActions.ts`
Système de packaging de projet :
- Calcul de condensats SHA-256 natifs pour dédupliquer les images et fichiers identiques.
- Export et import d'archives autonomes `.glucose` contenant à la fois la structure JSON et les blobs médias.

### `src/utils/export/*`
- `toHtml.ts` : génère une page HTML autonome contenant le moteur de rendu et l'intégralité du projet embarqué en base64.
- `toSvg.ts` : exporte une vue vectorielle SVG complète avec flèches, textes et conteneurs.
- `toMarkdown.ts` : extrait toute la hiérarchie textuelle ordonnée par zones et liens.
- `toPng.ts` : rastérisation haute définition du canvas.

---

## 7. Collaboration Multi-utilisateur & Télémétrie

### `src/multiplayer/*`
- **`localUser.ts`** : identifiant unique de l'utilisateur local, couleur de curseur et pseudonyme.
- **`PeerCursorsLayer.tsx`** : affichage en temps réel des curseurs et sélections des autres utilisateurs connectés sur le même board.
- **`collabBridge.ts` & `collabHandle.ts`** : protocole de synchronisation par messages de patchs et de curseurs via WebSocket.
- **`assetChannel.ts`** : canal de transmission point-à-point pour transférer les images ajoutées localement vers les pairs du réseau.

### `src/telemetry/*`
- **`telemetry.ts`** : journalisation anonymisée et opt-in des métriques d'utilisation (durée de session, nombre d'images, etc.) vers un serveur auto-hébergé.
- **`perfMonitor.ts`** : mesure continue du taux de rafraîchissement (FPS), des temps de boucle de rendu et de la consommation mémoire.

---

## 8. Services Système Natifs Rust (`src-tauri/src`)

### `src-tauri/src/lib.rs`
Le backend système complet de Glucose (2520 lignes de Rust) :
- **Sécurité & Sandboxing (`validate_scope`)** : restriction stricte des lectures et écritures aux dossiers utilisateurs autorisés (Bureau, Documents, Téléchargements, Images, Vidéos) et **rejet formel des chemins réseau UNC non sécurisés** (protection contre les failles SSRF et vol de hash NTLM).
- **Lancement d'applications sécurisé (`open_in_app`)** : lancement de fichiers dans leurs logiciels respectifs (Blender, Photoshop, Krita) avec **liste noire stricte d'exécutables (`FORBIDDEN_OPEN_EXTS`)** pour empêcher toute exécution de code arbitraire au double-clic (protection RCE).
- **Miroir de dossiers OS (`scan_directory`, `scan_tree`)** : exploration récursive du système de fichiers pour générer des hiérarchies de sous-dossiers navigables en miroir vivant.
- **Intégration d'IA Locale (Ollama)** :
  - `system_specs` : sonde matérielle détectant le nombre de cœurs CPU, la mémoire RAM totale et la VRAM GPU pour recommander le modèle d'IA adapté (`qwen2.5:3b`, `7b`, `14b`, `32b`).
  - `ollama_status`, `ollama_generate`, `pull_model` : interrogation et exécution locale de modèles d'IA pour transformer des textes bruts en cartes de cours spatialisées sur le canvas.

### `src-tauri/src/multiplayer.rs`
Moteur réseau multi-utilisateur :
- Découverte automatique des pairs sur le réseau local (LAN) par broadcast UDP.
- Serveur et client WebSocket intégrés pour la synchronisation en temps réel sans nécessiter de serveur cloud externe.

---

## 9. Matrice de Traduction & Roadmap Rust Pur

Pour porter 100% des capacités de Glucose dans une architecture desktop native en Rust pur avec zéro boîte noire et zéro dépendance dans le core :

| Composant Original | Rôle Fonctionnel | Équivalent Architecture Rust Pur |
|---|---|---|
| `symbioticHue.ts` | Teinte organique selon $(x,y)$ + moyenne circulaire trigonométrique | Module mathématique pur `glucose_core::symbiotic_hue` (bruit 2D + vecteurs $\sum \cos, \sum \sin$). |
| `SvgAnnotationLayer.tsx` | Dessin des membranes ($rx=60$, pointillés, halos, titre à $x=16, y=-12$) | Rendu vectoriel anti-aliasé `Renderer::draw_membranes` avec étiquettes typographiques nettes. |
| `HtmlAnnotationLayer.tsx` | Cartes Markdown avec aura, texte centré, bordures douces et surlignage flèches | `Renderer::draw_cards` avec calcul d'aura lumineuse et parseur Markdown vectoriel anti-aliasé. |
| `GlucoseCanvas.tsx` (`editOverlay`) | Édition de texte en direct sous le curseur avec saisie clavier native | Composant `TextInputState` gérant le curseur clignotant, sélection, frappe et validation (`Enter`/`Esc`). |
| `Toolbar.tsx` | Barre d'outils vectorielle haute fidélité (icônes fines non pixelisées, boutons 44px) | Tracé vectoriel ultra-précis via `tiny-skia` basé sur les vrais paths SVG d'origine (14x14 et 16x16). |
| `dropHandler.ts` | Détection intelligente des formats (images, code source, dossiers, 3D) | Handler d'événement de drop OS décodant et créant les cartes correspondantes instantanément. |
| `layout.ts` | Auto-organisation en grille, rangées compactes, spirale ou chronologie | Algorithmes d'agencement automatique intégrés au Store pour réorganiser le board en 1 clic. |
| `src-tauri/src/lib.rs` | Sécurité scope, lancement sécurisé, scan de dossiers | Implémentation en Rust `std::fs` et `std::process::Command` avec les deny-lists de sécurité SEC-01..09. |
