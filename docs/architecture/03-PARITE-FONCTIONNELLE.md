# 03 — Parité fonctionnelle : l'inventaire complet

> Tu dis : *« j'ai l'impression d'être arrivé à 3 % de Glucose complet »*.
> Ce document mesure. La réponse honnête est **≈ 17 %**, et il est important de comprendre
> pourquoi ce n'est ni 3 % ni 50 %.

**Légende**

| Symbole | Sens |
|---|---|
| ✅ | Implémenté en Rust et **branché** dans l'application |
| 🟡 | Partiel — ça existe mais c'est incomplet ou faux |
| 💀 | **Code mort** : écrit et testé dans `glucose-core`, jamais appelé par l'app |
| ❌ | Absent |
| 🎨 | Dessiné à l'écran mais sans aucun comportement (« maquette ») |

---

## Le score

| Domaine | Fonctions | ✅ | 🟡 | 💀 | 🎨 | ❌ | Parité |
|---|---:|---:|---:|---:|---:|---:|---:|
| 1. Canvas & caméra | 14 | 5 | 2 | 0 | 1 | 6 | 43 % |
| 2. Sélection & manipulation | 18 | 5 | 3 | 2 | 1 | 7 | 36 % |
| 3. Images | 21 | 5 | 2 | 0 | 0 | 14 | 29 % |
| 4. Texte & Markdown | 19 | 2 | 4 | 1 | 0 | 12 | 21 % |
| 5. Stickies | 8 | 2 | 2 | 0 | 0 | 4 | 31 % |
| 6. Flèches | 16 | 1 | 1 | 1 | 0 | 13 | 12 % |
| 7. Membranes | 17 | 1 | 1 | 5 | 0 | 10 | 12 % |
| 8. Rideaux (curtains) | 11 | 0 | 0 | 3 | 0 | 8 | 0 % |
| 9. Dossiers & sous-canvas | 15 | 0 | 0 | 3 | 1 | 11 | 3 % |
| 10. Miroirs | 7 | 0 | 0 | 4 | 0 | 3 | 0 % |
| 11. Domaines sémantiques | 9 | 0 | 0 | 0 | 1 | 8 | 3 % |
| 12. Temporalité | 8 | 0 | 0 | 2 | 0 | 6 | 0 % |
| 13. Storyboard | 8 | 0 | 0 | 0 | 1 | 7 | 3 % |
| 14. Presets & zones | 7 | 0 | 0 | 0 | 1 | 6 | 4 % |
| 15. Undo / historique | 9 | 3 | 2 | 0 | 0 | 4 | 44 % |
| 16. Persistance | 12 | 0 | 0 | 2 | 0 | 10 | 0 % |
| 17. Export | 8 | 0 | 0 | 4 | 1 | 3 | 0 % |
| 18. Interface & panneaux | 22 | 5 | 3 | 0 | 8 | 6 | 30 % |
| 19. Entrées & plateforme | 16 | 5 | 3 | 0 | 0 | 8 | 41 % |
| 20. Collaboration | 12 | 0 | 0 | 0 | 1 | 11 | 0 % |
| 21. Plugins & App Bridge | 9 | 0 | 0 | 0 | 1 | 8 | 0 % |
| 22. Recherche & navigation | 7 | 1 | 1 | 0 | 0 | 5 | 21 % |
| 23. Divers (Pomodoro, télémétrie…) | 6 | 0 | 0 | 0 | 1 | 5 | 0 % |
| **TOTAL** | **279** | **35** | **24** | **27** | **18** | **175** | **≈ 17 %** |

*(Parité = (✅ × 1 + 🟡 × 0,5) / total. Le code mort compte pour 0 : une fonctionnalité non
branchée n'existe pas pour l'utilisateur.)*

> **Re-vérifié au commit `81aea31`.** Deux commits (`2da029f`, `81aea31`) sont arrivés pendant
> la rédaction : la grille adaptative (1.5), la sélection élastique (2.5) et le positionnement
> des images (3.19) sont désormais corrects, et la barre d'outils a reçu un vrai moteur de mise
> en page avec ses tests. Le détail est dans
> [`01-AUDIT-CODE-RUST.md`](01-AUDIT-CODE-RUST.md) § Re-vérification.

### Ce que le score dit vraiment

**Tu n'es pas à 3 %, tu es à 17 % — avec 27 fonctionnalités déjà écrites, testées et à un
branchement près.** Si on ne branchait *que* le code mort existant, sans écrire une ligne
d'algorithme nouveau, on passerait **de 17 % à ≈ 26 %**.

C'est ça, le vrai enseignement de cet inventaire : **ton problème n'est pas la quantité de code
manquant, c'est la quantité de code non relié.** Voir R-18.

Deuxième enseignement : les **18 fonctionnalités « maquette »** (🎨) sont ce qui fausse ta
perception. La barre d'outils montre 19 boutons ; **9 agissent**. Tu regardes l'écran et tu vois
Glucose ; tu cliques et il n'y a rien derrière. D'où l'impression de 3 % là où la mesure dit 17 %.

### Ce que le score ne dit PAS, et qu'il faut ajouter

Ce tableau compte des **fonctionnalités**, pas du **travail**. C'est une mesure utile mais
trompeuse, et il faut la corriger sur deux points.

#### 1. Une ligne du tableau peut valoir 2 000 lignes de code

« Collaboration : 12 fonctions, 0 % » occupe une ligne, comme « Storyboard ». Ce n'est pas le
même chantier. Volume réel à porter, mesuré sur le TypeScript hors tests :

| Sous-système | TS à porter | Noyau Rust | Branché à l'UI ? |
|---|---:|---|---|
| Membranes (espace, focus, tween, étirement) | **2 661 l.** | 1 442 l. écrites | ❌ **3 modules morts** |
| Persistance (projet, schéma, bundle, assets, versions) | **2 572 l.** | 354 l. (`bundle`) | ❌ **mort** |
| Texte riche : Markdown + KaTeX + éditeur | **2 229 l.** | — | ❌ **rien** |
| Collaboration (CRDT, curseurs, canal d'assets) | **1 920 l.** | — | ❌ **rien** (1 bouton) |
| Export (SVG, HTML, PNG, Markdown, scène) | **1 565 l.** | 430 l. écrites | ❌ **mort** |
| Plugins & App Bridge | **1 471 l.** | — | ❌ **rien** (1 bouton) |
| Flèches (tracé, ancrage, texte, options) | **1 386 l.** | 166 l. (`arrow_anchor`) | ❌ **mort** |
| Dossiers & miroirs | **1 307 l.** | 110 l. (`mirror_graph`) | ❌ **mort** |
| Temporalité (timeline, règle, ancres) | **1 296 l.** | 256 l. (`timeline`) | ❌ **mort** |
| Rideaux (curtains) | **1 178 l.** | 417 l. écrites | ❌ **2 modules morts** |
| Télémétrie & diagnostics | 788 l. | — | ❌ rien |
| Presets & zones | 622 l. | — | 🎨 maquette |
| Storyboard | 464 l. | — | 🎨 maquette |
| Domaines sémantiques | 211 l. | complet et testé | ❌ **liste fantôme (R-47)** |
| **Total de ces 14 sous-systèmes** | **19 670 l.** | | |

Sur **32 423 lignes** de TypeScript hors tests, ces quatorze sous-systèmes en représentent **61 %**
— et **aucun** n'est utilisable aujourd'hui. Le reste (~12 750 l.) est le cœur du canvas
(`GlucoseCanvas.tsx` 4 181 l., `store/index.ts` 2 113 l., sélection, glisser-déposer, alignement),
qui est la partie réellement portée.

#### 2. Douze modules du noyau sur vingt ne sont appelés par personne

Mesure directe des références depuis `glucose-desktop`, au commit `e2cd410` :

| Module mort | l. | | Module mort | l. |
|---|---:|---|---|---:|
| `membrane_space` | 773 | | `curtain_model` | 216 |
| `membrane_focus` | 438 | | `geometry` | 215 |
| `export` | 430 | | `text_anchors` | 211 |
| `bundle` | 354 | | `curtain_panel` | 201 |
| `timeline` | 256 | | `arrow_anchor` | 166 |
| `membrane_stretch` | 231 | | `mirror_graph` | 110 |

**3 601 lignes écrites, testées, et inatteignables depuis l'interface.** Les huit modules vivants
sont `types`, `store`, `layout`, `smart_align`, `hit_priority`, `quadtree`, `symbiotic_hue`,
`error`.

C'est la mesure exacte de R-18, et elle est pire que l'estimation initiale : ce n'est pas « du
code mort », c'est **la moitié du noyau**.

#### 3. Même les 17 % acquis ne rendent pas ce qu'ils devraient

Le score compte une fonctionnalité comme ✅ dès qu'elle marche. Il ne dit rien de sa **fidélité**.
Deux défauts, documentés en R-45 et R-46, dégradent **tout** ce qui s'affiche :

- **Le texte ne suit pas le zoom.** Onze `clamp` bornent la police, les marges et les rayons de
  coin, alors que la boîte de la carte, elle, se met à l'échelle librement. La carte n'est fidèle
  qu'à `scale ≈ 1` ; partout ailleurs elle se déforme par paliers. C'est le « LOD » involontaire.
- **Les glyphes sont posés à des coordonnées entières tronquées.** Ni sous-pixel, ni arrondi. D'où
  un espacement irrégulier, un tremblement au déplacement, et une impression de pixelisation.

Autrement dit : **17 % des fonctionnalités existent, et elles s'affichent mal.** Le ressenti de
« 3 % » ne vient pas seulement de ce qui manque — il vient aussi de ce qui est là et paraît faux.

#### 4. Ce qui n'est dans aucune ligne du tableau

Trois manques n'apparaissent nulle part parce qu'ils ne sont pas des fonctionnalités :

- **Aucune animation.** `dock.rs` n'a ni `tween`, ni interpolation, ni easing — zéro occurrence.
  Côté TS, **30 fichiers** portent des transitions, et `PanelDock.tsx` ramène un panneau à sa
  place avec `transform 0.25s cubic-bezier(0.22, 1, 0.36, 1)`. D'où l'impression de fenêtres
  « complètement libres » : elles le sont littéralement, rien ne les rappelle.
- **`membraneTween.ts` (216 l.)** est un moteur d'animation dédié aux membranes, entièrement
  absent du portage.
- **Le redimensionnement** (`imageResize.ts`, poignées, contraintes de ratio) n'a pas d'équivalent
  branché.

---

## 1. Canvas & caméra

| # | Fonction | Source TS | Rust | Phase |
|---|---|---|:--:|:--:|
| 1.1 | Canvas infini, pan à la souris | `GlucoseCanvas` | ✅ | — |
| 1.2 | Zoom molette centré sur le curseur | `GlucoseCanvas` | ✅ | — |
| 1.3 | Bornes de zoom (0,01 – 50) | `navigation.ts` | ✅ | — |
| 1.4 | Viewport indépendant par board | `store.getViewport` | ✅ | — |
| 1.5 | Grille de points | `GlucoseCanvas` | ✅ pas adaptatif depuis `81aea31` (R-02 corrigé) | — |
| 1.6 | Fit-to-content (`F`) | `navigation.ts` | 🟡 remet à (0,0,1), ne cadre rien | 1 |
| 1.7 | Signets de vue 1–9 | `Board.bookmarks` | ❌ | 4 |
| 1.8 | Navigation au pavé numérique (4/6/8/2/+/−) | `GlucoseCanvas:2510` | ❌ | 4 |
| 1.9 | Verrouillage du zoom navigateur | `useZoomLock` | ❌ *(sans objet en natif)* | — |
| 1.10 | Minimap | `Minimap.tsx` | 🎨 dessinée, clic inatteignable (R-08) | 3 |
| 1.11 | Minimap : annotations + membranes | `Minimap.tsx` | ❌ images seules | 3 |
| 1.12 | Défilement horizontal (trackpad) | `GlucoseCanvas` | ❌ | 4 |
| 1.13 | Zoom au clavier (`+` / `−` / `0`) | `useZoomLock` | ❌ | 4 |
| 1.14 | Indicateur de viewport de dossier | `FolderViewportIndicator` | ❌ | 7 |

---

## 2. Sélection & manipulation

| # | Fonction | Source TS | Rust | Phase |
|---|---|---|:--:|:--:|
| 2.1 | Sélection au clic | `hitPriority` | ✅ | — |
| 2.2 | Priorité de sélection multi-couches | `hitPriority` + `pickArbiter` | ✅ (partiel : `collect_candidates` seul) | — |
| 2.3 | Sélection additive `Maj+clic` | `GlucoseCanvas` | ✅ | — |
| 2.4 | Tout sélectionner `Ctrl+A` | `store.selectAll` | ✅ | — |
| 2.5 | **Sélection élastique (marquee)** | `GlucoseCanvas` | ✅ corrigée par `2da029f` (R-10) | — |
| 2.6 | Déplacer la sélection au drag | `store.moveSelected` | ✅ | — |
| 2.7 | Alignement intelligent / magnétisme | `smartAlign` + runtime | 💀 **écrit, testé, jamais appelé** (R-09) | 1 |
| 2.8 | Guides d'alignement visuels | `GlucoseCanvas` | 🟡 dessine une liste toujours vide | 1 |
| 2.9 | Poignées de redimensionnement | `imageResize` | 🎨 dessinées, non interactives | 3 |
| 2.10 | Curseurs contextuels sur poignées | `hitPriority.handleCursor` | 💀 fonction écrite, jamais appelée | 3 |
| 2.11 | Rotation | `BoardImage.rotation` | ❌ champ ignoré (R-24) | 3 |
| 2.12 | Verrouillage d'élément | `BoardImage.locked` | 🟡 respecté au déplacement, invisible | 3 |
| 2.13 | Dupliquer `Ctrl+D` | `store.duplicateSelected` | 🟡 collision d'ids (R-13) | 1 |
| 2.14 | Supprimer `Suppr` / `Retour` | `store.deleteSelected` | ✅ | — |
| 2.15 | Copier / Couper / Coller entre éléments | `App.tsx:254-262` | ❌ | 4 |
| 2.16 | Menu contextuel (clic droit) | `ContextMenu.tsx` | ❌ le clic droit sert au pan | 3 |
| 2.17 | Ordre d'empilement (z-order) | — | ❌ ordre du `Vec` uniquement | 3 |
| 2.18 | Déplacement au clavier (flèches) | `GlucoseCanvas` | ❌ | 4 |

---

## 3. Images

| # | Fonction | Source TS | Rust | Phase |
|---|---|---|:--:|:--:|
| 3.1 | Import par dialogue de fichiers | `fileImport` | ✅ | — |
| 3.2 | Glisser-déposer de fichiers OS | `dropHandler` | 🟡 **un fichier à la fois** | 5 |
| 3.3 | **Glisser-déposer depuis un navigateur** | `dropHandler` | ❌ *(ton besoin n°1 — voir 02 § 0.1)* | 5 |
| 3.4 | Coller une image du presse-papiers | `dropHandler` | ✅ | — |
| 3.5 | Coller une URL d'image | `dropHandler` | ❌ | 5 |
| 3.6 | Retour visuel au survol du dépôt | `dropHandler` | ❌ | 5 |
| 3.7 | Formats PNG / JPEG / WebP / GIF / BMP | — | ✅ (via `image`) | — |
| 3.8 | Vidéos | `BoardImage.isVideo` | ❌ | 10 |
| 3.9 | Redimensionnement interactif | `imageResize` | ❌ (R-24) | 3 |
| 3.10 | Préservation du ratio | `imageResize` | ❌ | 3 |
| 3.11 | Mode `fit: contain` | `BoardImage.fit` | ❌ | 3 |
| 3.12 | Mipmaps / échantillonnage correct | — | ❌ bilinéaire pleine résolution (R-29) | 3 |
| 3.13 | Cache borné avec éviction | — | ❌ **cache infini** (R-29) | 3 |
| 3.14 | Décodage asynchrone | `imageUpgrade` | ❌ synchrone dans la frame (R-29) | 3 |
| 3.15 | Chargement progressif (vignette → pleine) | `imageUpgrade` | ❌ | 3 |
| 3.16 | Lecture d'en-tête pour les dimensions | — | ❌ **décodage complet** (R-30) | 3 |
| 3.17 | Assets embarqués, dédup sha256 | `assetRef` + `Project.blobs` | 🟡 `sha256()` existe, jamais appelé | 2 |
| 3.18 | Assets liés (chemin/URL externe) | `assetRef` | 🟡 champ `src` seul | 2 |
| 3.19 | Positionnement correct au rendu | — | ✅ corrigé par `81aea31` (R-06) | — |
| 3.20 | URL d'origine conservée | `sourceUrl` | ❌ champ jamais rempli | 5 |
| 3.21 | Étiquettes (tags) | `BoardImage.tags` | ❌ | 9 |

---

## 4. Texte & Markdown

| # | Fonction | Source TS | Rust | Phase |
|---|---|---|:--:|:--:|
| 4.1 | Créer une carte texte | `HtmlAnnotationLayer` | ✅ | — |
| 4.2 | Édition en place au double-clic | `HtmlAnnotationLayer` | ✅ | — |
| 4.3 | Curseur clignotant | — | 🟡 **ne clignote jamais** (R-15) | 1 |
| 4.4 | Navigation ← → Début Fin | — | 🟡 par octets, pas par graphèmes (R-17) | 4 |
| 4.5 | Navigation ↑ ↓ | — | ❌ | 4 |
| 4.6 | Sélection de texte (`Maj`+flèches, glisser) | — | ❌ | 4 |
| 4.7 | Copier / coller dans le texte | — | ❌ | 4 |
| 4.8 | **Retour à la ligne automatique** | `markdownText.wrapRuns` | ❌ **débordement infini** (R-28) | 4 |
| 4.9 | Titres `#` `##` `###` | `react-markdown` | 🟡 `#` seul, un niveau | 4 |
| 4.10 | Gras / italique / barré | `react-markdown` | ❌ | 4 |
| 4.11 | Code en ligne et blocs | `react-markdown` | ❌ | 4 |
| 4.12 | Listes à puces et numérotées | `react-markdown` | 🟡 `-` seul, pas de numérotation | 4 |
| 4.13 | Citations `>` | `remark-gfm` | ❌ | 4 |
| 4.14 | Tableaux | `remark-gfm` | ❌ | 9 |
| 4.15 | Liens cliquables | `react-markdown` | ❌ | 9 |
| 4.16 | **LaTeX / maths** | `katex` + `remark-math` | ❌ | 9 |
| 4.17 | Crénage & ligne de base correcte | — | ❌ (R-27, R-28) | 4 |
| 4.18 | Ancres de texte robustes (citation/préfixe/suffixe) | `textAnchors` | 💀 211 l. + 135 l. de tests, non branché | 8 |
| 4.19 | Éditeur de syntaxe | `SyntaxEditor` | ❌ | 12 |

---

## 5. Stickies

| # | Fonction | Source TS | Rust | Phase |
|---|---|---|:--:|:--:|
| 5.1 | Créer une sticky | `HtmlAnnotationLayer` | ✅ | — |
| 5.2 | Éditer en place | — | ✅ | — |
| 5.3 | **Couleur de fond personnalisée** | `bgColor` | ❌ **codée en dur** (R-24) | 1 |
| 5.4 | Couleur de texte | `color` | ❌ codée en dur | 1 |
| 5.5 | Opérateurs logiques (AND/OR/BUT/BECAUSE) | `operator` | 🟡 affiché via `{:?}` → « And » | 9 |
| 5.6 | Sélecteur de couleur | `ColorPicker` | ❌ | 3 |
| 5.7 | Curseur d'édition correct | — | 🟡 mesure tout le texte, pas le préfixe (R-28) | 4 |
| 5.8 | Redimensionnement | — | ❌ | 3 |

---

## 6. Flèches

| # | Fonction | Source TS | Rust | Phase |
|---|---|---|:--:|:--:|
| 6.1 | Créer une flèche | `ArrowSvgLayer` | 🟡 **taille fixe 120×80, non dessinable** | 8 |
| 6.2 | Dessiner par glisser | `GlucoseCanvas` | ❌ | 8 |
| 6.3 | Déplacer les extrémités | — | ❌ | 8 |
| 6.4 | Rendu de la flèche | `ArrowSvgLayer` | ✅ (droite, pointe simple) | — |
| 6.5 | Flèches courbes | `arrowType: "curved"` | ❌ (R-24) | 8 |
| 6.6 | Points de passage (waypoints) | `waypoints` | ❌ (R-24) | 8 |
| 6.7 | Bidirectionnelle | `arrowBidirectional` | ❌ (R-24) | 8 |
| 6.8 | Épaisseur & couleur | `strokeWidth`, `color` | ❌ (R-24) | 8 |
| 6.9 | Étiquette sur la flèche | `text` | ❌ jamais affichée | 8 |
| 6.10 | Éditeur d'étiquette | `ArrowTextEditor` (380 l.) | ❌ | 8 |
| 6.11 | Attache à un nœud source/cible | `sourceId`, `targetId` | 🟡 suit la source seule (R-12) | 8 |
| 6.12 | Ancrage géométrique au bord | `arrow_anchor.rs` | 💀 166 l. + 87 l. de tests | 8 |
| 6.13 | Attache à un sous-bloc de texte | `sourceBlockId` | ❌ | 8 |
| 6.14 | Attache à une sélection de texte | `sourceTextSel` | ❌ | 8 |
| 6.15 | **Prédicats sémantiques** (6 types) | `predicate` | ❌ modèle présent, rien de plus | 8 |
| 6.16 | Flèche-portail vers un autre board | `targetBoardId` | ❌ | 8 |

---

## 7. Membranes

| # | Fonction | Source TS | Rust | Phase |
|---|---|---|:--:|:--:|
| 7.1 | Créer une membrane | `SvgAnnotationLayer` | 🟡 **taille fixe 320×240, non dessinable** | 6 |
| 7.2 | Rendu (halo, pointillés, titre) | `SvgAnnotationLayer` | ✅ *(le rendu est fidèle et joli)* | — |
| 7.3 | Dessiner par glisser | — | ❌ | 6 |
| 7.4 | Redimensionner | — | 🎨 poignées dessinées, inertes | 6 |
| 7.5 | Appartenance stockée (`membraneId`) | invariant MEMB-1 | ❌ champ jamais écrit | 6 |
| 7.6 | Dépôt d'un élément → adoption | `membraneSpace` | 💀 772 l. + 422 l. de tests | 6 |
| 7.7 | Mode `classic` | `MembraneMode` | 🟡 seul mode existant, implicite | 6 |
| 7.8 | Mode `minimized` (contenu < 1) | `membraneSpace` | 💀 | 6 |
| 7.9 | Mode `stretched` (auto-agrandissement) | `membraneStretch` | 💀 231 l. + 199 l. de tests | 6 |
| 7.10 | Alerte d'étirement | `MembraneStretchAlert` | ❌ | 6 |
| 7.11 | **Mode Focus** | `membraneFocus` | 💀 438 l. + 337 l. de tests | 6 |
| 7.12 | Animation de transition | `membraneTween` | 💀 216 l. + 271 l. de tests | 6 |
| 7.13 | Options de membrane (panneau) | `MembraneOptions` | ❌ | 6 |
| 7.14 | Couleur dérivée des domaines | `symbioticHue` + domaines | ❌ | 11 |
| 7.15 | Titre éditable | — | 🟡 éditable au double-clic | — |
| 7.16 | Membrane imbriquée | `membraneId` sur membrane | ❌ | 6 |
| 7.17 | Suppression en cascade du contenu | — | ❌ | 6 |

---

## 8. Rideaux (curtains)

*La fonctionnalité la plus originale de Glucose. **0 % en Rust**, mais 3 modules déjà écrits.*

| # | Fonction | Source TS | Rust | Phase |
|---|---|---|:--:|:--:|
| 8.1 | Créer un rideau sur une membrane | `MembraneCurtainLayer` | ❌ | 11 |
| 8.2 | Modèle géométrique du rideau | `curtainModel` | 💀 216 l. + 287 l. de tests | 11 |
| 8.3 | Panneau de rideau | `curtainPanel` | 💀 201 l. | 11 |
| 8.4 | Rideau = board Glucose complet | `MembraneCurtain.boardId` | ❌ | 11 |
| 8.5 | Canvas du rideau | `CurtainCanvas` (721 l.) | ❌ | 11 |
| 8.6 | Visibilité privé / partagé | `CurtainVisibility` | ❌ | 11 |
| 8.7 | Droits owner / everyone | `CurtainEditable` | ❌ | 11 |
| 8.8 | Languette nommée et colorée | `ownerName`, `ownerColor` | ❌ | 11 |
| 8.9 | Ratios replié / déplié | `collapsedRatio` | ❌ | 11 |
| 8.10 | Notes legacy migrées | `CurtainNote` | ❌ | 11 |
| 8.11 | Visible uniquement en mode Focus | — | ❌ | 11 |

---

## 9. Dossiers & sous-canvas

| # | Fonction | Source TS | Rust | Phase |
|---|---|---|:--:|:--:|
| 9.1 | Outil Dossier | `Toolbar` | 🎨 **affiche un toast, rien d'autre** | 7 |
| 9.2 | Créer un dossier (capture le contenu) | `store.createFolder` | 💀 écrit dans le store, non appelé | 7 |
| 9.3 | Rendu d'un dossier sur le canvas | `FolderSvgLayer` (534 l.) | ❌ **jamais dessiné** (R-24) | 7 |
| 9.4 | Entrer / sortir d'un dossier | `store.enterFolder` | 💀 écrit, non appelé | 7 |
| 9.5 | Fil d'Ariane | `FolderBreadcrumb` | ❌ | 7 |
| 9.6 | Sous-board indépendant | `childBoardId` | 💀 | 7 |
| 9.7 | **Miroir d'un dossier OS** | `folderMirror` (381 l.) | ❌ | 7 |
| 9.8 | Scan `snapshot` / `live` | `FolderMirrorSource.mode` | ❌ | 7 |
| 9.9 | Scan récursif | `recursive` | ❌ | 7 |
| 9.10 | Scan paresseux (49 k+ fichiers) | `pendingScan` | ❌ | 7 |
| 9.11 | Filtre glob | `pattern` | ❌ | 7 |
| 9.12 | 7 modes de tri façon explorateur | `FolderSortMode` | ❌ | 7 |
| 9.13 | Vignettes des médias du dossier | `FolderTreeNode.images` | ❌ | 7 |
| 9.14 | Détection de cycle (anti-inception) | `mirror_graph.rs` | 💀 110 l. + 93 l. de tests | 7 |
| 9.15 | Ouvrir un fichier avec son application | App Bridge | ❌ | 12 |

---

## 10. Miroirs (alias vivants)

| # | Fonction | Source TS | Rust | Phase |
|---|---|---|:--:|:--:|
| 10.1 | Miroir d'annotation | `store.mirrorAnnotation` | 💀 écrit (collision d'id, R-13) | 7 |
| 10.2 | Miroir d'image | `store.mirrorImage` | ❌ | 7 |
| 10.3 | Miroir de dossier | `store.mirrorFolder` | 💀 écrit | 7 |
| 10.4 | Propagation des modifications | `mirrorOf` | ❌ **aucune propagation** | 7 |
| 10.5 | Suppression en cascade | `store.removeImages` | 💀 écrit (O(n²), R-22) | 7 |
| 10.6 | Remonter à l'original | `findOriginalAnnotation` | ❌ | 7 |
| 10.7 | Marquage visuel d'un miroir | — | ❌ | 7 |

---

## 11. Domaines sémantiques

| # | Fonction | Source TS | Rust | Phase |
|---|---|---|:--:|:--:|
| 11.1 | Créer / éditer / supprimer un domaine | `DomainsPanel` | ❌ (store écrit, non branché) | 9 |
| 11.2 | Panneau Domaines | `DomainsPanel` (211 l.) | 🎨 bouton → toast | 9 |
| 11.3 | Affecter un domaine avec un poids | `assignDomainToNode` | ❌ | 9 |
| 11.4 | Couleur de membrane dérivée des poids | — | ❌ | 9 |
| 11.5 | Badges de domaine sur les nœuds | `AnnotationBadges` | ❌ | 9 |
| 11.6 | Liens trans-domaines en pointillés | `Toolbar` toggle | 🎨 bascule un booléen inutilisé | 9 |
| 11.7 | Filtrer par domaine | — | ❌ | 9 |
| 11.8 | Icône / emoji de domaine | `Domain.icon` | ❌ | 9 |
| 11.9 | Domaines partagés entre boards | `Project.domains` | ❌ | 9 |

---

## 12. Temporalité

| # | Fonction | Source TS | Rust | Phase |
|---|---|---|:--:|:--:|
| 12.1 | Ancrage temporel d'un nœud | `TemporalAnchor` | ❌ modèle présent seul | 9 |
| 12.2 | Invite de saisie de date | `TemporalAnchorPrompt` | ❌ | 9 |
| 12.3 | Règle temporelle | `TemporalRuler` (270 l.) | ❌ | 9 |
| 12.4 | Filtre temporel | `store.setTemporalFilter` | 💀 champ écrit, jamais lu | 9 |
| 12.5 | Panneau Timeline | `TimelinePanel` (635 l.) | ❌ | 9 |
| 12.6 | Calculs de timeline | `timeline.rs` | 💀 256 l. + 174 l. de tests | 9 |
| 12.7 | Plages (ex. Renaissance 1400-1600) | `start`/`end` | ❌ | 9 |
| 12.8 | Années négatives (av. J.-C.) | — | ❌ | 9 |

---

## 13. Storyboard

| # | Fonction | Source TS | Rust | Phase |
|---|---|---|:--:|:--:|
| 13.1 | Panneaux de storyboard | `StoryboardPanel` | ❌ modèle présent, jamais dessiné | 9 |
| 13.2 | Contrôles | `StoryboardControls` (317 l.) | 🎨 bouton → toast | 9 |
| 13.3 | Ratios (16:9, 4:3, 2.35:1, 1:1, 9:16) | `AspectRatio` | ❌ | 9 |
| 13.4 | Grille de panneaux (colonnes, gouttière) | `StoryboardSettings` | ❌ | 9 |
| 13.5 | Assigner une image à un panneau | `imageId` | ❌ | 9 |
| 13.6 | Description sous le panneau | `description` | ❌ | 9 |
| 13.7 | Réordonner | `store.reorderPanels` | ❌ | 9 |
| 13.8 | Couche de rendu | `StoryboardLayer` | ❌ | 9 |

---

## 14. Presets & zones

| # | Fonction | Source TS | Rust | Phase |
|---|---|---|:--:|:--:|
| 14.1 | Presets artistiques intégrés | `defaultPresets` | ❌ | 9 |
| 14.2 | Panneau Presets | `PresetPanel` (189 l.) | 🎨 bouton → toast | 9 |
| 14.3 | Appliquer un preset à un board | `applyPresetToBoard` | ❌ (store écrit, non branché) | 9 |
| 14.4 | Zones colorées avec libellé | `BoardZone` | ❌ jamais dessinées | 9 |
| 14.5 | Outil de sélection de zone | `ZoneSelectorOverlay` | ❌ | 9 |
| 14.6 | Rendu des zones | `ZoneRenderer` (283 l.) | ❌ | 9 |
| 14.7 | Affecter une image à un emplacement | `slotId` | ❌ | 9 |

---

## 15. Undo / historique

| # | Fonction | Source TS | Rust | Phase |
|---|---|---|:--:|:--:|
| 15.1 | Annuler `Ctrl+Z` | `store.undo` | ✅ | — |
| 15.2 | Rétablir `Ctrl+Y` / `Ctrl+Maj+Z` | `store.redo` | ✅ | — |
| 15.3 | La navigation ne pollue pas l'undo | `mutateView` | ✅ *(bon invariant, à garder)* | — |
| 15.4 | Caméra préservée à travers l'undo | `preserve_view` | 🟡 écrit, à revalider | 1 |
| 15.5 | Transaction live atomique (drag) | `beginLiveEdit` | 🟡 écrit, à revalider | 1 |
| 15.6 | **Coût mémoire acceptable** | Automerge (incrémental) | ❌ **clone complet ×200** (R-04) | 1 |
| 15.7 | Coalescence de la frappe | — | ❌ | 1 |
| 15.8 | Libellé de l'action (« Annuler : … ») | `commitNamed` | ❌ | 1 |
| 15.9 | Time machine / historique nommé | `setPreviewHeads` | ❌ | 11 |

---

## 16. Persistance

*Le domaine à **0 %**, et le plus urgent.*

| # | Fonction | Source TS | Rust | Phase |
|---|---|---|:--:|:--:|
| 16.1 | **Enregistrer un projet** | `project.ts` | ❌ (R-01) | 2 |
| 16.2 | **Ouvrir un projet** | `project.ts` | ❌ (R-01) | 2 |
| 16.3 | Enregistrer sous | `saveState` | ❌ | 2 |
| 16.4 | Format de fichier `.glucose` | `bundle.ts` | ❌ | 2 |
| 16.5 | Assets embarqués dans le fichier | `Project.blobs` | ❌ | 2 |
| 16.6 | Dédup sha256 | `bundle.sha256` | 💀 fonction écrite (353 l. + 246 l. de tests) | 2 |
| 16.7 | Sauvegarde automatique | `useAutosave` | ❌ | 2 |
| 16.8 | Écriture atomique (anti-corruption) | `saveOverwrite` | ❌ | 2 |
| 16.9 | Versions automatiques | `autoVersion`, `versions` | ❌ | 2 |
| 16.10 | Compaction du document | `compaction` | ❌ | 2 |
| 16.11 | Migration des anciens formats | `projectMigration` | 💀 concept présent dans `bundle.rs` | 2 |
| 16.12 | Récupération après crash | — | ❌ | 2 |

---

## 17. Export

| # | Fonction | Source TS | Rust | Phase |
|---|---|---|:--:|:--:|
| 17.1 | Menu d'export | `ExportMenu` (226 l.) | 🎨 bouton → toast (R-33) | 10 |
| 17.2 | Construction de la scène d'export | `export/scene.ts` (441 l.) | 💀 `build_scene` écrit | 10 |
| 17.3 | Export SVG | `toSvg.ts` | 💀 `scene_to_svg` écrit | 10 |
| 17.4 | Export Markdown | `toMarkdown.ts` | 💀 `scene_to_markdown` écrit | 10 |
| 17.5 | Export HTML | `toHtml.ts` | ❌ | 10 |
| 17.6 | Export PNG | `toPng.ts` | ❌ | 10 |
| 17.7 | Mesure de texte pour l'export | `makeMeasure` | 💀 | 10 |
| 17.8 | **Écrire le résultat sur le disque** | — | ❌ | 10 |

---

## 18. Interface & panneaux

| # | Fonction | Source TS | Rust | Phase |
|---|---|---|:--:|:--:|
| 18.1 | Barre d'outils | `Toolbar` (350 l.) | ✅ *(le look est fidèle)* | — |
| 18.2 | Sélection d'outil au clic | `Toolbar` | ✅ | — |
| 18.3 | Survol des boutons | `Toolbar` | ✅ | — |
| 18.4 | Onglets de boards | `BoardTabs` | 🟡 **hit-test décalé** (R-07) | 1 |
| 18.5 | Créer un board (`+`) | `store.addBoard` | 🟡 collision d'ids (R-14) | 1 |
| 18.6 | Renommer un board | `store.renameBoard` | ❌ | 3 |
| 18.7 | Fermer / réordonner les boards | `reorderBoards` | ❌ | 3 |
| 18.8 | Notifications (toasts) | `Toast` (244 l.) | 🟡 une seule, n'expire pas (R-15) | 1 |
| 18.9 | Dock de panneaux | `PanelDock` (344 l.) | ❌ | 3 |
| 18.10 | Sélecteur de couleur | `ColorPicker` (331 l.) | ❌ | 3 |
| 18.11 | Panneau Organiser | `OrganizePanel` (349 l.) | 🎨 bouton → grille naïve (R-11) | 3 |
| 18.12 | Panneau Plugins | `PluginPanel` (569 l.) | 🎨 toast | 12 |
| 18.13 | Panneau Presets | `PresetPanel` | 🎨 toast | 9 |
| 18.14 | Panneau Domaines | `DomainsPanel` | 🎨 toast | 9 |
| 18.15 | Panneau Multijoueur | `MultiplayerPanel` (283 l.) | 🎨 toast | 11 |
| 18.16 | Menu Export | `ExportMenu` | 🎨 toast | 10 |
| 18.17 | Timer Pomodoro | `PomodoroTimer` + overlay | 🎨 toast | 12 |
| 18.18 | Infobulles | `title=` HTML | ❌ `_title` collecté puis ignoré | 3 |
| 18.19 | Thème centralisé | Tailwind | ❌ ~130 littéraux (R-31) | 1 |
| 18.20 | Barre de statut / erreurs visibles | `ErrorBoundary` | ❌ (R-21) | 1 |
| 18.21 | Navigation clavier dans l'UI | — | ❌ | 3 |
| 18.22 | HUD de diagnostic | `DiagnosticsHUD` | ❌ | 3 |

---

## 19. Entrées & plateforme

| # | Fonction | Source TS | Rust | Phase |
|---|---|---|:--:|:--:|
| 19.1 | Raccourcis d'outils V/T/N/A/F/M | `App.tsx:145-150` | ✅ | — |
| 19.2 | `Espace` = pan | `App.tsx:153` | ✅ | — |
| 19.3 | `Ctrl+Z` / `Ctrl+Y` / `Ctrl+D` / `Ctrl+A` | `App.tsx` | ✅ | — |
| 19.4 | `Ctrl+V` coller | `App.tsx` | ✅ | — |
| 19.5 | `Alt+T` toujours au premier plan | — | ✅ *(bonus PureRef, absent en TS)* | — |
| 19.6 | `Échap` annule le geste courant | `App.tsx:159` | 🟡 sort de l'édition seulement | 4 |
| 19.7 | `Ctrl+S` / `Ctrl+O` projet | `App.tsx:323,348` | ❌ (R-01) | 2 |
| 19.8 | `Ctrl+F` recherche | `App.tsx:319` | ❌ | 12 |
| 19.9 | `F11` plein écran | `App.tsx:177` | ❌ | 4 |
| 19.10 | `Ctrl+Maj+M` / `Ctrl+Maj+F` / `Ctrl+H` … | `App.tsx` | ❌ | 4 |
| 19.11 | **Gestion du DPI** | navigateur | ❌ (R-16) | 1 |
| 19.12 | **IME (accents, CJK)** | navigateur | ❌ (R-17) | 4 |
| 19.13 | Molette horizontale | navigateur | ❌ | 4 |
| 19.14 | Bouton du milieu distinct du droit | `GlucoseCanvas` | 🟡 confondus | 3 |
| 19.15 | Presse-papiers multi-formats en écriture | navigateur | 🟡 lecture seule | 4 |
| 19.16 | Tablette / pression stylet | — | ❌ | — |

---

## 20. Collaboration

| # | Fonction | Source TS | Rust | Phase |
|---|---|---|:--:|:--:|
| 20.1 | Document CRDT | `automerge` | ❌ | 11 |
| 20.2 | Synchronisation WebSocket | `automerge-repo-network-websocket` | ❌ | 11 |
| 20.3 | Créer un partage | `collabBridge` | 🎨 toast « Collaboration connectée » | 11 |
| 20.4 | Rejoindre un partage | `collabHandle` | ❌ | 11 |
| 20.5 | Curseurs des pairs | `PeerCursorsLayer` (404 l.) | ❌ | 11 |
| 20.6 | Canal d'assets séparé | `assetChannel` (327 l.) | ❌ | 11 |
| 20.7 | Identité locale | `localUser` | ❌ | 11 |
| 20.8 | Lien de collab dans le document | `Project.collabUrl` | ❌ champ présent | 11 |
| 20.9 | Reconnexion automatique | `useMultiplayerSync` | ❌ | 11 |
| 20.10 | Panneau Multijoueur | `MultiplayerPanel` | 🎨 | 11 |
| 20.11 | Débogage collab | `CollabDebug` | ❌ | 11 |
| 20.12 | Rideaux partagés / privés | `CurtainVisibility` | ❌ | 11 |

---

## 21. Plugins & App Bridge

| # | Fonction | Source TS | Rust | Phase |
|---|---|---|:--:|:--:|
| 21.1 | Système de plugins | `utils/plugins.ts` | ❌ | 12 |
| 21.2 | Moteur intégré | `builtinEngine` (504 l.) | ❌ | 12 |
| 21.3 | Panneau Plugins | `PluginPanel` | 🎨 toast | 12 |
| 21.4 | Texte → carte par IA | `PLUGINS.md` | ❌ | 12 |
| 21.5 | Bus d'événements | `glucoseBus` | ❌ | 12 |
| 21.6 | App Bridge : ouvrir un fichier | `AppBridgeIcon` | ❌ | 12 |
| 21.7 | Overlay de lancement | `AppLaunchOverlay` | ❌ | 12 |
| 21.8 | Résolution de chemins | `pathResolver` | ❌ | 12 |
| 21.9 | Vérification d'existence de fichier | `fileExistence` | ❌ | 12 |

---

## 22. Recherche & navigation

| # | Fonction | Source TS | Rust | Phase |
|---|---|---|:--:|:--:|
| 22.1 | Panneau de recherche | `SearchPanel` | ❌ | 12 |
| 22.2 | Recherche plein texte | `SearchPanel` | ❌ | 12 |
| 22.3 | Naviguer vers un résultat | — | ❌ | 12 |
| 22.4 | Multi-boards | `store.boards` | ✅ | — |
| 22.5 | Changer de board actif | `setActiveBoardId` | 🟡 hit-test décalé (R-07) | 1 |
| 22.6 | Dupliquer un board | `duplicateBoard` | ❌ | 3 |
| 22.7 | Importer un board | `importBoard` | ❌ | 10 |

---

## 23. Divers

| # | Fonction | Source TS | Rust | Phase |
|---|---|---|:--:|:--:|
| 23.1 | Timer Pomodoro | `PomodoroTimer` | 🎨 toast | 12 |
| 23.2 | Overlay Pomodoro | `PomodoroOverlay` | ❌ | 12 |
| 23.3 | Télémétrie (avec consentement) | `telemetry` (319 l.) | ❌ | — |
| 23.4 | Moniteur de performance | `perfMonitor` | ❌ | 1 |
| 23.5 | Mise à jour de l'application | `UpdatePrompt` | ❌ | 12 |
| 23.6 | Barrière d'erreur | `ErrorBoundary` | ❌ (R-21) | 1 |

---

## Ce que cet inventaire change dans le plan

Trois conclusions opérationnelles :

### 1. Brancher avant d'écrire

**27 fonctionnalités sont à un branchement près.** Ce sont les 💀. Elles représentent
≈ 3 700 lignes déjà écrites et testées. Les brancher fait passer la parité de **17 % à 26 %**
pour un coût dérisoire comparé à leur réécriture.

C'est pour ça que la phase 1 de la roadmap s'appelle « Brancher le noyau » et pas
« Nouvelles fonctionnalités ».

### 2. Supprimer les maquettes

**10 boutons sur 19 n'ont aucun effet observable.** Tant qu'ils sont là, tu ne peux pas mesurer
ta progression, et quiconque essaie l'app croit qu'elle est cassée plutôt qu'inachevée.

**Décision** : un bouton dont la fonction n'existe pas est **retiré de la barre**, ou grisé avec
une infobulle « bientôt ». Jamais un toast qui simule une action. C'est la loi L10 appliquée à
l'UI.

### 3. Le vrai ordre des priorités

Le classement par « manque le plus criant » n'est pas le classement par « nombre de cases ❌ » :

| Rang | Domaine | Pourquoi en premier |
|---|---|---|
| 1 | **Persistance** (16) | 0 %. Sans elle, aucun autre travail n'est utilisable ni testable. |
| 2 | **Undo** (15) | 44 % mais avec un défaut bloquant : l'app meurt en mémoire avant tout. |
| 3 | **Performance** (1, 2, 3) | Deux fautes algorithmiques rendent l'app inutilisable sur un vrai canvas. |
| 4 | **Texte** (4) | 21 %, et c'est le cœur de Glucose. Sans retour à la ligne, les cartes sont inutilisables. |
| 5 | **Glisser-déposer** (3.2, 3.3) | Ton besoin explicite, et un vrai différenciateur technique. |
| 6 | **Membranes + Dossiers** (7, 9) | 12 % / 3 %, mais ce sont **les** fonctionnalités identitaires de Glucose. |
| 7 | Le reste | Dans l'ordre de la roadmap. |

---

**Suite** : [`04-ROADMAP.md`](04-ROADMAP.md) — le plan de bataille, phase par phase.
