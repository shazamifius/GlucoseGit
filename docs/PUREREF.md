# PureRef, fonction par fonction — la cible du preset PureRef

> **Sa liste du 15/10/2026** (fiche 71), confrontée à ce que Glucose Rust fait **aujourd'hui**. C'est
> la cible du **preset PureRef** ([VISION](VISION.md) : un preset est l'interface entière — la
> disposition, le comportement, les couleurs, les raccourcis, les fenêtres) : *« retrouver 100 %,
> ou presque, des fonctionnalités »*. Ce document se tient à jour comme [INVENTAIRE](INVENTAIRE.md),
> dont il est le pendant pour PureRef.
>
> **La règle de lecture** : PureRef dit *quoi*, jamais *comment* (R2) — une fonction se recrée
> dans la loi de Glucose, avec un bouton au PC et au téléphone. Ce qui est relevé ici vient du
> code (au 15/10) ; une ligne fausse se corrige.

✅ Glucose le fait · 🟡 en partie (dit comment) · ❌ pas encore · — sans objet

---

## 1. Ajouter et importer des images

| PureRef | Glucose | |
|---|---|---|
| Glisser-déposer depuis l'ordinateur ou un navigateur | ✅ | depuis un navigateur sous Windows (Pinterest en pleine résolution, DEPOT-WEB-6) ; ailleurs, un lien qui dit pourquoi |
| Coller (`Ctrl+V`) | ✅ | |
| Importer par l'explorateur (`Ctrl+I`) | ✅ | bouton Images, menu du vide ; au téléphone, le sélecteur de photos et « Partager » |
| Coller une adresse de page : **toutes** ses images | 🟡 | une épingle se rapatrie en une image ; extraire toutes les images d'une page : à faire |
| Remplacer une image existante | 🟡 | « Remplacer par l'image » pour un lien seulement |
| Relier les images manquantes | — | les images sont **scellées** dans le `.glucose` : aucune ne manque |
| Les formats courants | ✅ | PNG, JPEG, WebP, GIF, BMP — HEIC au téléphone à faire |
| GIF **animés** | ❌ | un GIF se montre figé |
| Profils de couleur ICC embarqués | ❌ | |

## 2. Organiser, arranger

| PureRef | Glucose | |
|---|---|---|
| Sélection simple, multiple, rectangle, tout, désélection | ✅ | et aux doigts (GESTES-1) |
| Arranger : disposition optimale | ✅ | Ordonner : rangées compactes, maçonnerie, grille alignée, même hauteur, par mood board |
| Arranger par taille, ordre actuel | ✅ | huit tris : l'ordre actuel, la taille (↑↓), le rapport (portrait, paysage), et **trois par la couleur** — la teinte, du sombre au clair, du clair au sombre (Oklab) — que PureRef n'a pas |
| Arranger par nom, ordre d'ajout | 🟡 | le noyau sait trier par nom et par date (`OrganizeSort`), **aucun bouton ne le montre** : à brancher |
| … par hiérarchie, chemin de fichier, aléatoire (relancer inverse) | ❌ | |
| Aligner (gauche, droite, haut, bas) avec espacement réglable | 🟡 | l'aimant aligne aux voisines pendant un glisser (SNAP-4) ; une commande « Aligner » : à faire |
| Distribuer en lignes horizontales ou verticales | 🟡 | le noyau sait poser en ligne et en colonne (`OrganizeMode::Row`, `Column`), **aucun bouton ne le montre** : à brancher |
| Empiler (pile selon l'ordre des calques) | ❌ | |
| Normaliser hauteur, largeur, taille, aire, échelle | 🟡 | même hauteur (Ordonner) ; même largeur dans le noyau sans bouton ; taille, aire, échelle à faire |
| Premier plan, arrière-plan | ✅ | `Ctrl+]`, `Ctrl+[` |
| Déplacement contraint sur un axe | ❌ | |
| Aimanter aux bords des voisins | ✅ | l'aimant, et le placement aimanté dès le premier clic |
| Grille d'arrière-plan (lignes ou points), aimanter à la grille | 🟡 | la grille de points se dessine ; ni lignes, ni aimant à la grille |
| Groupes : fond coloré et opacité, verrou, ajout par glisser, taille automatique | 🟡 | **les membranes** possèdent ce qu'on y dépose, ont leur couleur et leur titre ; les dossiers ; le verrou d'un groupe et la taille automatique à voir |
| Hiérarchie (outliner) : l'arbre, parents et enfants, renommer (`F2`), statut | ❌ | les membranes et les dossiers en sont la matière ; la vue en arbre est à faire |
| Verrouiller un nœud, dupliquer | ✅ | `L`, `Ctrl+D` |

## 3. Transformer une image

| PureRef | Glucose | |
|---|---|---|
| Déplacer, zoomer, se déplacer (souris, pavé, doigts) | ✅ | le pavé par *Direct Manipulation* ; les doigts au téléphone |
| Rotation libre, **magnétisée à 45°** | 🟡 | libre (`Alt` + coin) ; le magnétisme à 45° à faire |
| Mise à l'échelle non destructive, depuis le centre (`Alt`) | ✅ | origine commune ou individuelle, comme Blender |
| Réinitialiser la transformation | ❌ | |
| Gizmo de transformation | ✅ | les poignées du groupe |
| Recadrage non destructif, pan et zoom dans le recadrage, ratio verrouillé, réinitialiser | 🟡 | double-clic, non destructif (RECADRAGE-1) ; ratio verrouillé et réinitialiser à voir |
| Miroir horizontal et vertical | ❌ | |
| Opacité d'une image, du tableau | ❌ | (le mode référence : opacité de la fenêtre, pas pour l'instant à sa parole) |
| Niveaux de gris, par image ou partout | ❌ | |
| Interpolation bilinéaire ou plus proche voisin (pixel art) | ❌ | |

## 4. Annoter, créer

| PureRef | Glucose | |
|---|---|---|
| Notes en texte riche (gras, italique, listes, cases à cocher, liens, alignement, police, taille) | 🟡 | Markdown, tableaux, liens, **LaTeX fidèle à KaTeX** — que PureRef n'a pas ; souligné, alignement, police et taille par note : à voir |
| Dessin à main levée, gomme, couleur, pointillés, 45° avec `Maj` | ❌ | |
| Formes (rectangles, ellipses, lignes) | 🟡 | des membranes rectangulaires ; ellipses et lignes libres à faire |
| Pointes de flèche | ✅ | **les flèches de Glucose** : courbes, double sens, six relations, coudes, contournement, ancres de texte |
| Commentaires attachés aux images | ❌ | (les post-it et les ancres en sont proches) |
| Les dessins comme éléments de la hiérarchie | ❌ | |

## 5. La fenêtre

| PureRef | Glucose | |
|---|---|---|
| Toujours au premier plan | ✅ | **le mode référence** (`Ctrl+Maj+A`) |
| Au premier plan d'**une** application, qui la suit | ❌ | |
| Toujours derrière (fond d'écran interactif) | ❌ | |
| Transparente à la souris (décalquer) | ❌ | pas pour l'instant, à sa parole (15/10) |
| Overlay : chaque image en fenêtre flottante | ❌ | |
| Verrouiller la fenêtre, plein écran, redimensionner à la sélection | 🟡 | sans cadre, déplacée et redimensionnée au pavé (`Alt`) ; redimensionner à la sélection à faire |
| Barre de titre configurable | 🟡 | sans cadre en mode référence ; les quatre réglages à faire |
| Fenêtres multiples | ✅ | et le copier-glisser de l'une à l'autre |

## 6. Regarder, présenter

| PureRef | Glucose | |
|---|---|---|
| Focus sur une image (`Espace`, double-clic), suivante, précédente, tout | 🟡 | `F` cadre la sélection, `Alt+F` tout ; suivante et précédente à faire |
| Diaporama : durée, ordre, pause | 🟡 | **l'animatique du storyboard** (une durée par case, lecture, pause) ; un diaporama de n'importe quelles images à faire |
| Lire un GIF (pause, frise, vitesse, image par image, extraire) | ❌ | |
| Pipette (RGB, HSV, HEX), hors de la fenêtre aussi | ❌ | |
| Coordonnées dans l'image, nom, source, taille | ❌ | |

## 7. Enregistrer, exporter, les fichiers

| PureRef | Glucose | |
|---|---|---|
| Images embarquées | ✅ | scellées par leur empreinte, chaque octet une fois |
| … ou liées à l'extérieur | ❌ | |
| Enregistrer, enregistrer sous | ✅ | `Ctrl+S` pose un jalon |
| Sauvegarde périodique, récupération après plantage | ✅ | **mieux** : chaque geste s'écrit aussitôt, le texte en cours de frappe survit à un plantage, la Time Machine |
| Format ancien lisible | ✅ | les documents de Glucose Tauri |
| Exporter le tableau, la sélection en une image | ✅ | PNG par bandes (EXPORT-PNG-1), et **SVG** complet |
| Exporter chaque image en fichier séparé, gabarits de noms | 🟡 | « Copier » plusieurs images les donne en fichiers ; un export par lot à faire |
| Copier comme fichier | ✅ | COPIER-1 |
| Fichiers récents avec vignettes, épingler, rouvrir la dernière scène | 🟡 | la dernière est rouverte au lancement ; la liste avec vignettes à faire |
| Ouvrir l'emplacement de la scène | ❌ | |
| Vignettes dans l'explorateur de Windows | ❌ | |
| Gérer les images par lot (réduire, optimiser, embarquer ou lier) | ❌ | |
| Cache et lecture en continu depuis le disque | ✅ | **la mémoire par étages** : carte graphique, mémoire, disque, bornés par ce que la machine laisse |

## 8. Personnaliser, le reste

| PureRef | Glucose | |
|---|---|---|
| Interface minimaliste : raccourcis et clic droit | ✅ | chaque commande est au clic droit (ACCES-1) |
| **Raccourcis entièrement personnalisables** (souris, contextes, global) | 🟡 | **la fondation est posée** (fiche 72) : chaque raccourci est une ligne d'une couche de texte, les couches s'empilent et se retirent ; la couche de l'utilisateur, lue sur le disque, reste à faire ; la souris et le raccourci global aussi |
| **Palette de commandes** (`Ctrl+Maj+P`) | ✅ | les 75 commandes de la table, cherchées sans accents, chacune avec son raccourci (fiche 72) |
| Chercher dans les réglages, importer et exporter les réglages | ❌ | un preset exporté *est* un fichier de réglages |
| Interface traduite (sept langues) | ❌ | Glucose parle français |
| Annuler, rétablir, taille de l'historique réglable | 🟡 | annuler et rétablir, mot par mot dans un texte ; l'historique n'a pas de taille à régler (il coûte la taille de chaque changement) |
| Tactile, gestes du pavé, SpaceMouse | 🟡 | doigts et pavé ✅ ; SpaceMouse ❌ |
| Profil de couleur de l'écran | ❌ | |
| Versions beta à la demande | 🟡 | les betas se publient ; le choix « recevoir les betas » est à faire |
| Installation silencieuse, version portable | 🟡 | un installeur par utilisateur ; une version portable à voir |

---

## Ce qui en sort

* **Ce que Glucose fait déjà mieux** : les flèches et leurs relations, le LaTeX, les tris par la
  couleur, l'enregistrement geste après geste et la Time Machine, la mémoire par étages, le
  storyboard, le téléphone.
* **Ce qui manque le plus pour un preset PureRef** : les raccourcis personnalisables et la palette
  de commandes (une seule fondation), la hiérarchie, les GIF animés, le miroir, l'opacité et les
  niveaux de gris, la pipette, le dessin, les modes de fenêtre (derrière tout, transparente,
  overlay).
