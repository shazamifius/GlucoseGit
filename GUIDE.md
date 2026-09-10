# 📖 Guide d'utilisation Glucose (Natif Rust)

> **Glucose** est un canvas de référence infini natif en Rust, inspiré de **PureRef**. 
> Une seule interface ultra-fluide, zéro boîte noire, zéro latence : pose, relie, zoome, explore.

---

## 1. Démarrage rapide & Contrôles PureRef

À l'ouverture, Glucose s'ouvre sur un canvas sombre infini quadrillé de points discrets.

### 🖱️ Gestes fondamentaux (PureRef style)
1. **Glisser-déposer des images** depuis l'explorateur de fichiers OS directement dans la fenêtre (WebP, PNG, JPEG, GIF, BMP) → elles s'insèrent immédiatement.
2. **Coller depuis le presse-papiers (`Ctrl + V`)** :
   - Copie n'importe quelle image depuis un navigateur (Pinterest, ArtStation, Google Images) ou capture d'écran, puis fais `Ctrl + V` dans Glucose : elle apparaît instantanément avec un toast de confirmation.
3. **Bouton `+ Images` (`Ctrl + O`)** :
   - Ouvre le sélecteur de fichiers natif de l'OS pour charger des lots complets d'images.
4. **Pan (déplacer la vue)** :
   - Maintiens le **Clic milieu** et glisse la souris, OU
   - Maintiens le **Clic droit** et glisse la souris, OU
   - Maintiens la barre **`Espace`** et glisse avec le clic gauche.
5. **Zoom au curseur** :
   - Fais tourner la **molette de la souris** : le zoom s'effectue exactement centré sur le point du monde sous ton curseur.
6. **Cadrer tout (Fit)** :
   - Appuie sur **`F`** pour recentrer la caméra sur l'origine du canvas.
7. **Always on Top (Épingler au-dessus)** :
   - Raccourci **`Alt + T`** : Glucose reste au premier plan au-dessus de Blender, Photoshop, ZBrush ou Krita pendant que tu crées.
8. **Création rapide d'annotations** :
   - **`T`** : Outil Carte Texte
   - **`N`** : Outil Sticky Note
   - **`A`** : Outil Flèche relationnelle
   - **`M`** : Outil Membrane de regroupement
   - **`V`** : Outil Sélection
9. **Historique complet & Duplication** :
   - **`Ctrl + Z`** : Annuler
   - **`Ctrl + Y`** ou **`Ctrl + Shift + Z`** : Rétablir
   - **`Ctrl + D`** : Dupliquer les éléments sélectionnés
   - **`Suppr`** ou **`Retour arrière`** : Supprimer la sélection
   - Bouton **`Ordonner`** : Réorganise instantanément les images et notes en grille propre.

---

## 2. Vocabulaire du Système

| Terme | Définition |
|---|---|
| **Board** | Un espace de travail infini indépendant doté de ses propres images, annotations, membranes, dossiers et caméra. |
| **Image** | Une référence visuelle native positionnée, redimensionnable, sélectionnable et déplaçable sur le canvas. |
| **Annotation** | Éléments textuels ou vectoriels : notes stickies, blocs de texte Markdown, flèches relationnelles, membranes. |
| **Sticky** | Note pense-bête colorée dotée d'opérateurs logiques optionnels (`AND`, `OR`, `BUT`, `BECAUSE`). |
| **Texte** | Bloc typographique pour la prose structurée, avec ancrage sub-block de précision W3C. |
| **Flèche** | Lien orienté entre deux éléments portant un **prédicat sémantique** (`inspire`, `contredit`, `dépend_de`…). |
| **Membrane** | Conteneur élastique regroupant un ensemble d'éléments. Dispose de 3 modes : `Classic`, `Minimized`, `Stretched`. |
| **Dossier** | Sous-canvas imbriqué capturant l'espace géométrique intérieur lors de sa création. |
| **Miroir ↻** | Alias vivant d'un nœud ou dossier : toute modification de l'original se répercute instantanément, protégé contre les cycles infinis. |
| **Domaine** | Catégorie sémantique (Science, Art, etc.) influençant la signature chromatique des membranes. |

---

## 3. Priorité de Hit & Sélection (Règles PICK-1)

Quand plusieurs éléments se superposent (par exemple du texte posé sur une image elle-même contenue dans une membrane), Glucose applique un arbitre déterministe en **7 rangs stricts** :

```
[Rang 1] Poignées de redimensionnement (priorité absolue, taille généreuse constante à l'écran)
   ↓
[Rang 2] Bordure active du conteneur (bande périphérique et poignée de membrane / dossier)
   ↓
[Rang 3] Flèches vectorielles (tracé fin)
   ↓
[Rang 4] Images
   ↓
[Rang 5] Stickies
   ↓
[Rang 6] Textes (Terminus pour permettre le double-clic d'édition)
   ↓
[Rang 7] Intérieur du conteneur (un conteneur ne vole jamais un clic à son contenu)
```

### 🔄 Cyclage de sélection au clic
- **Cliquer plusieurs fois sans bouger la souris** descend d'un cran dans la hiérarchie : 1er clic sur le bord d'une membrane → la membrane ; 2e clic → l'image sous-jacente ; 3e clic → la note.
- L'avancement dans le cycle s'effectue **au relâchement du clic** (mouse up), afin de ne jamais perturber un glisser-déplacer d'élément.
- Le cycle **s'arrête automatiquement sur les éléments éditables (texte, sticky)** pour préserver le geste naturel du double-clic d'édition.

---

## 4. Magnétisme Intelligent (SNAP-1)

Lors du déplacement ou du redimensionnement d'un élément sélectionné :
- Glucose calcule dynamiquement les alignements sur les bords gauche, droit, haut, bas et les centres des autres éléments visibles.
- **Seuil de capture constant en pixels écran** : l'aimantation reste aussi précise et naturelle à fort dézoom qu'en très gros plan.
- Des **guides d'alignement cyan et magenta** s'affichent instantanément à l'écran pour visualiser les correspondances géométriques.

---

## 5. Membranes & Repères Locaux

Les membranes réinventent le regroupement visuel :

1. **Facteur d'échelle déduit $k$** :
   $$k = \min\left(1, \frac{\text{largeur}}{\text{étendue}_X}, \frac{\text{hauteur}}{\text{étendue}_Y}\right)$$
   L'échelle n'est jamais stockée sous forme de variable mutable : elle découle purement des dimensions de la membrane par rapport à l'étendue naturelle de son contenu.
2. **Appartenance événementielle (`membrane_id`)** :
   L'appartenance d'un élément à une membrane est persistée dès son dépôt géométrique. Une membrane minimisée ne « perd » jamais son contenu même si ses dimensions physiques deviennent inférieures aux éléments qu'elle abrite.
3. **Mode Stretched (Étiré avec arrêt sur obstacles)** :
   Une membrane configurée en mode étiré s'adapte automatiquement à l'ajout de nouveau contenu, mais stoppe sa course sans jamais écraser ou englober les éléments tiers extérieurs.
4. **Mode Focus Asymétrique** :
   - Entrée automatique lorsque la membrane couvre **$\ge 92\%$** de l'écran.
   - Sortie lorsque le dézoom franchit **$\le 80\%$** de l'échelle de cadrage initial.
   - Cette asymétrie garantit l'absence totale d'oscillations visuelles.

---

## 6. Miroirs Vivants & Graphe Acyclique

Les miroirs permettent de créer des alias interactifs d'images, de notes ou de dossiers entiers.
- **Protection Anti-Inception (BFS Acyclique)** : Glucose vérifie par parcours en largeur que l'insertion d'un miroir ne génère aucun cycle de dépendance directe ou indirecte. Toute tentative de boucle infinie est rejetée de manière sécurisée.
- **Téléportation source** : Cliquer sur le badge miroir ↻ recentre instantanément la caméra sur l'élément original, y compris s'il se trouve dans un autre board.

---

## 7. Moteur d'Annulation / Rétablissement (Undo/Redo)

Le store Glucose garantit un historique indestructible :
- **Transparence de navigation** : Les manipulations de caméra (panoramique, zoom) ne polluent jamais la pile undo. Revenir en arrière annule l'action géométrique sans téléporter la caméra de l'utilisateur.
- **Sessions atomiques (`begin_live_edit` / `end_live_edit`)** : Un glisser-déplacer d'un groupe d'éléments ou un redimensionnement continu ne génère qu'une seule et unique entrée dans l'historique lors du relâchement.
- **Cascade d'intégrité** : La suppression d'un élément entraîne la suppression propre et réversible des miroirs associés et des flèches orphelines.

---

## 8. Exportations Natives (0 Dépendance)

Glucose intègre directement dans son moteur `glucose-core` :
- **Export SVG vectoriel** : Génération d'un document SVG autonome complet représentant fidèlement la scène, les cadres, les textes échappés en toute sécurité, les flèches courbes et les têtes de flèches orientées.
- **Export Markdown structuré** : Conversion hiérarchique du canvas en document Markdown clair, organisant les cartes, zones, liens sémantiques et textes sous forme de fiches lisibles.

---

## 9. Tableau Récapitulatif des Raccourcis

| Raccourci | Fonction |
|---|---|
| **Clic milieu glissé** | Panoramique de la vue (PureRef style) |
| **Clic droit glissé** | Panoramique de la vue (PureRef alternatif) |
| **`Espace` + Clic gauche** | Panoramique de la vue classique |
| **Molette souris** | Zoom avant / arrière centré sur le curseur |
| **`Espace` (clic sec)** | Cadrer l'intégralité du contenu (Zoom to fit) |
| **`F`** | Cadrer l'intégralité du contenu |
| **`T`** | Basculer la fenêtre en **Always on Top** (toujours au premier plan) |
| **Glisser-déposer de fichiers** | Importation instantanée d'images dans le canvas |
| **Clic gauche** | Sélection / Cyclage de cible empilée |
| **`Ctrl` + Clic gauche** | Multi-sélection additive |
| **`Ctrl+D`** | Dupliquer les éléments sélectionnés |
| **`Suppr` / `Backspace`** | Supprimer la sélection |
| **`Ctrl+Z`** | Annuler la dernière action (sans perturber la vue) |
| **`Ctrl+Y`** | Rétablir la dernière action |
| **`Échap`** | Désélectionner / Quitter |

---

<div align="center">

**Glucose, c'est juste poser, relier, zoomer, explorer.**

[← Retour au README](README.md) · [Handoff Technique](HANDOFF.md)

</div>

