# 08 — Spécification Complète des Nœuds & Fonctionnalités Canvas

> **Rôle de ce document** : fournir la **spécification unitaire et exhaustive** de chaque type d'élément pouvant vivre sur le canvas de Glucose, de ses propriétés, de ses contraintes d'affichage et de ses algorithmes de manipulation.
> Ce document sert de contrat technique pour le portage de toute la couche logique métier en Rust natif.
>
> **Méthode (12/09/2026)** : chaque point est allé voir le code. Ce qui est implémenté **et tenu par un test** est sorti de cette fiche ; ce qui reste est la liste de travail. Le constat d'ensemble : **le noyau est écrit et testé bien au-delà de ce que l'interface branche** — dossiers, miroirs, modes de membrane, opérateurs, prédicats existent dans `glucose-core` et n'ont pas de geste dans `glucose-desktop`.

---

## 1. Les Images (`BoardImage`)

### 1.1 Modèle de Données & Représentation
* **À FAIRE** — `fit` est une `Option<String>` là où la fiche demandait un `Option<ImageFit>` typé (`Contain` pour les vignettes de dossiers, qui n'existent pas encore). À typer avec les vignettes.

### 1.2 Pipeline de Culling Spatial & Virtualisation VRAM
> L'index spatial existe et est mesuré (`SpatialHash`, 40 µs pour un viewport sur 10⁶ nœuds). Tout le reste est le chantier « actifs et rendu » (fiche 09 § 4, plan de marche RQ-2) : le cache d'images est indexé par chemin, sans limite, sans niveaux de détail, et décode **dans la boucle de rendu**.

* **À FAIRE** — **Hystérésis de visibilité** : marge de chargement viewport $+ 50\,\%$ (`loadMargin = base × 0.5`), marge de conservation viewport $+ 250\,\%$ (`keepMargin = base × 2.5`). Une image n'est déchargée que passé la seconde, ce qui élimine les rafales au bord de l'écran.
* **À FAIRE** — **Chargement progressif par priorité radiale** : lors d'un dézoom massif ou d'un *Fit View* (Ctrl+Shift+F), file de décodage triée par distance au centre de l'écran, jamais un décodage en bloc (gel de 500 ms).
* **À FAIRE** — **Décodage multi-résolution (LOD de rendu — la charte le veut)** : $\text{pixels}_{\text{texture}} = \text{clamp}(\text{taille}_{\text{écran}} \times 1.25,\, 256,\, \text{résolution}_{\text{native}})$, ré-affinage automatique à $\times 1.8$ du besoin initial.

### 1.3 Manipulation Directe
> Déplacement, ancrage au côté opposé, taille minimale, rapport d'aspect conservé sur les coins : tenus par `resize_suite` et `interactions/resize/tests.rs`, par le chemin de la souris. **Extension assumée** par rapport à la cible : huit poignées (les côtés changent une seule dimension) et `Shift` libère le rapport — le comportement par défaut reste celui de la cible. Duplication : décalage de **20 px**, comme Glucose Tauri (`OFFSET = 20`) — la fiche disait 24 ; tenu par `test_duplicate_offsets_the_clone_by_twenty_pixels_and_selects_it`.

* **À FAIRE** — **Ancrage centré (touche Ctrl)** pendant le redimensionnement : le centre $(x, y)$ reste immobile, la boîte grandit symétriquement. `ResizeRule` n'a pas de notion d'ancre au centre.
* **À FAIRE** — **Verrouillage (touche `L`)** : bascule `locked` ; les poignées disparaissent, la carte devient inerte au glisser. Toast `"Images verrouillées 🔒"` / `"déverrouillées 🔓"`. Le modèle porte `locked` et `move_selected` le respecte ; aucun raccourci ne le bascule.

---

## 2. Les Cartes de Texte & Markdown (`TextAnnotation`)

### 2.1 Moteur de Rendu Texte
> Le moteur comprend `#`, `##` et les puces `-`/`*` ; tout le reste est lu comme du corps (tenu par `test_the_markdown_the_card_understands_and_the_markdown_it_does_not`, qui tombera quand la suite sera écrite). L'édition se fait **dans la carte elle-même** — pas d'éditeur superposé à aligner « au millimètre » : la carte est l'éditeur. La saisie entière est une entrée d'annulation (fiche 09 § 2, `test_live_3`).

* **À FAIRE** — **Markdown GFM** : titres H3 à H6, citations `>`, tableaux, blocs de code, gras `**concept**` et italique `*terme*`.
* **À DÉCIDER** — **Formules mathématiques** (`$E = mc^2$` en ligne, `$$…$$` en bloc). KaTeX est une bibliothèque JavaScript ; en Rust, c'est soit un moteur de composition mathématique à écrire, soit une dépendance lourde. À peser contre l'exigence 1 de la charte quand le rendu texte GPU sera posé.

### 2.2 Ancrage Fin de Flèches sur le Texte (`TextSelection`)
> Tenu : `TextAnchor { start, end, quote, prefix, suffix }` persiste, et le réancrage par préfixe et suffixe après modification du texte est testé (`test_resolve_anchors_reanchor_after_edit`).

---

## 3. Les Notes Adhésives & Opérateurs Logiques (`StickyAnnotation`)

### 3.1 Note Adhésive Classique
* **À FAIRE** — **Palette pastel** : Jaune `#f5c542`, Rose `#f472b6`, Bleu `#60a5fa`, Vert `#4ade80`, Orange `#fb923c`, Violet `#c084fc`. Le modèle porte `bg_color` ; aucun choix de couleur dans l'interface.
* **À VÉRIFIER (fiche 06)** — Coins francs ou très subtilement arrondis, ombre marquée simulant le papier posé.

### 3.2 Opérateurs Logiques (Phase 5)
> Le modèle porte `StickyOperator { And, Or, But, Because }`, persisté (`test_every_sticky_operator_round_trips`), et le rendu affiche ET / OU / MAIS / PARCE QUE.

* **À FAIRE** — **Raccourcis** `Alt+1` à `Alt+4` (et `Alt+0` pour réinitialiser) sur une note sélectionnée. Aucun raccourci n'existe.
* **À FAIRE** — **Couleur par opérateur** : ET vert menthe `#34d399`, OU bleu ciel `#60a5fa`, MAIS ambre `#f59e0b`, PARCE QUE lilas `#a78bfa`. Le rendu emploie une seule couleur pour les quatre.
* **À FAIRE** — **Pilule** de $44\text{ px}$ de haut, texte gras centré, glow coloré. Le rendu écrit le libellé au-dessus du texte, dans le papier ordinaire.

---

## 4. Les Flèches & Graphe de Connaissance (`ArrowAnnotation`)

### 4.1 Évitement Dynamique d'Obstacles (Pathfinding)
* **À FAIRE** — **`getDynamicRoute`** : test d'intersection du segment $[A, B]$ avec les boîtes du board ; obstacle heurté étendu de $32\text{ px}$ (`OBSTACLE_PAD = 32`) ; points de contournement aux quatre coins étendus ; plus court chemin valide, récursif, profondeur maximale 10. `arrow_anchor.rs` ancre les extrémités aux bords des nœuds ; rien ne contourne.

### 4.2 Coudes & Points de Passage Utilisateur (Waypoints)
> `waypoints: Vec<{ x, y }>` est dans le modèle, persisté, et `arrow_anchor` en tient compte pour l'ancrage.

* **À FAIRE** — **Poignée centrale (`MidHandle`)** au survol de chaque segment, à glisser pour créer un coude.

### 4.3 Prédicats Sémantiques Prédéfinis
> Tenu : `ArrowPredicate` porte les six — `est_precurseur`, `contredit`, `herite_de`, `inspire`, `depend_de`, `illustre` — sous ces noms exacts, persistés (`test_every_arrow_predicate_round_trips`).

### 4.4 Flèches Portails Inter-Boards
> `target_board_id` est dans le modèle et persisté.

* **À FAIRE** — **Visuel** : extrémité ornée d'un double anneau bleu tournant avec la flèche `↗`.
* **À FAIRE** — **Interaction** : cliquer sur l'anneau téléporte sur le tableau de destination et centre la caméra sur l'élément cible.

---

## 5. Les Dossiers & Sous-Canvases (`CanvasFolder`)

### 5.1 Architecture des Sous-Canvases
> Tenu : `child_board_id` pointe un tableau complet, l'imbrication est sans limite (`test_workflow_nested_folders`), et le store sait entrer, sortir, renommer et supprimer sans casser la navigation (`e2e_workflows_suite`).

### 5.2 Navigation & Fil d'Ariane
* **À FAIRE** — **Entrée** : double-clic sur le bandeau ou le corps d'un dossier ; la caméra plonge en zoom avant ($400\text{ ms}$), puis le canvas charge le board enfant. Le store a `try_enter_folder`, testé ; **aucun geste du desktop ne l'appelle**.
* **À FAIRE** — **Fil d'Ariane (`FolderBreadcrumb`)** en haut à gauche sous la barre d'outils (`Projet > Recherches > Dossier 1`) ; cliquer un niveau remonte. Le store tient `folder_stack` ; rien ne l'affiche.
* **À FAIRE** — **Zoom de sortie adaptatif** : dézoomer fortement dans un dossier fait surface vers le parent sans bouton.

### 5.3 Dossiers Miroirs du Disque OS (Folder Mirror)
> Le mode de tri est dans le modèle et persisté (`test_every_folder_sort_mode_round_trips`). Un dépôt de fichier sur la fenêtre n'importe que des images (`import_image_files`), jamais un dossier.

* **À FAIRE** — **Scan sécurisé** de la hiérarchie locale ; exécutables et scripts masqués (`SCAN_HIDE_EXTS`).
* **À FAIRE** — **Scan paresseux** : seul le premier niveau immédiatement (`pendingScan = true`), les sous-dossiers à l'entrée.
* **À FAIRE** — **Tri Explorateur** : par nom (A→Z, Z→A), par extension, par taille (`size-desc`, `size-asc`), par date de modification (`modified-desc`).

---

## 6. Les Miroirs & Alias Vivants (Phase 4)

> Le noyau sait créer un miroir (`try_mirror_annotation`, `try_mirror_folder`) et refuse les cycles ; **aucun geste du desktop ne crée de miroir**.

### 6.1 Synchronisation Bidirectionnelle Instantanée
* **À FAIRE** — Modifier le texte, la taille, les domaines ou les étiquettes d'un miroir modifie l'original et tous les autres miroirs ; seule la position $(x, y)$ est propre à chaque copie. `update_annotation` ne propage rien à travers `mirror_of`.

### 6.2 Moteur Anti-Cycle Inception (`mirrorGraph.ts`)
> Tenu — et **sans la limite de profondeur de 16** que la fiche prévoyait : le parcours en largeur avec ensemble des visités détecte un cycle à toute profondeur et accepte un graphe sain de toute profondeur (`test_a_cycle_is_detected_at_any_depth_without_a_limit`, à 40 niveaux). La constante a disparu au lieu d'être réglée.

* **À FAIRE** — Le refus doit se voir : toast `"Cycle Inception refusé ⚠"`. Le noyau rend `CoreError::CycleDetected` ; personne ne l'affiche, faute de geste.

### 6.3 Téléportation Caméra
* **À FAIRE** — Pastille ronde bleue `↻` au coin supérieur gauche de chaque miroir ; cliquer glisse la caméra ($400\text{ ms}$) jusqu'à l'original, dossiers traversés compris.

---

## 7. Les Membranes Organisationnelles (`MembraneAnnotation`)

### 7.1 Les Trois Modes de Membrane
> Tenu dans le noyau : `MembraneMode { Classic, Minimized, Stretched }`, l'échelle du contenu en mode minimisé (`test_content_scale_single_and_both_axes`), le plan d'étirement (`test_stretched_mode_plan`, `test_plan_board_stretch_*`), et la marge d'air de $32\text{ px}$ (`STRETCH_PADDING`, `test_the_stretch_padding_is_thirty_two_pixels`).

* **À FAIRE** — **Basculer le mode depuis l'interface.** Le desktop crée toute membrane en `Classic` et n'offre aucun moyen d'en changer ; le rendu n'applique ni la réduction du minimisé ni l'englobement de l'étiré.

### 7.2 Alerte de Collision Élastique (`MembraneStretchAlert`)
> Tenu dans le noyau : l'expansion est stoppée au bord de l'obstacle et le plan dit qu'elle est bloquée (`test_plan_board_stretch_free_element_blocks`, `…_sibling_blocks`).

* **À FAIRE** — **La bannière** non intrusive au-dessus de la membrane, qui explique la contrainte et propose de repousser les voisins ou de repasser en mode classique.

---

## 8. Tuiles de Lancement d'Applications (App Bridge)

> Rien n'existe.

* **À FAIRE** — **Icônes natives** : `.blend` → logo Blender, halo orange `#ea7600` ; `.psd` → Photoshop bleu `#31a8ff` ; `.rs` → crabe Rust `#dea584`.
* **À FAIRE** — **Lancement au double-clic (`open_in_app`)** via l'ouverture système.
* **À FAIRE** — **Liste de blocage (`FORBIDDEN_OPEN_EXTS`)** : `.exe`, `.bat`, `.cmd`, `.ps1`, `.vbs`, `.js`, `.sh`, `.scr`, `.com`, `.jar` affichables, jamais exécutés.
* **À FAIRE** — **Liens rompus** : bordure pointillée rouge `#ef4444` ; cliquer le badge ouvre un sélecteur pour réassocier le chemin.
