# HANDOFF — Glucose (Réécriture Native Rust)

> Réécrit le **2026-09-10**, après la réécriture intégrale de PureRef/Glucose en **Rust natif from scratch**.
> Branche **`main`**.

---

## 0. TL;DR

Glucose est désormais un **logiciel desktop natif PureRef réécrit intégralement en Rust from scratch** :
- **`crates/glucose-core`** : Moteur pur en **100% Rust `std` avec strictement ZÉRO dépendance externe** (aucune crate tierce). Logique pure, déterministe et indestructible.
- **`crates/glucose-desktop`** : Client desktop natif offrant l'expérience complète de PureRef avec rendu logiciel vectoriel (`tiny-skia` + `softbuffer` + `winit`), garantissant une **compatibilité universelle avec 100% des OS et des noyaux** (Windows, Linux X11/Wayland, macOS, BSD) sans besoin de carte graphique ni de pilote GPU propriétaire.
- **Moteur de police bitmap intégré** (`font.rs` + `font_data.rs`) en pur Rust sans aucune dépendance de fichier de police système.
- **202 tests Rust (37 unitaires + 165 d'intégration) verts à 100%** (`cargo test --workspace` passe en 0.00s sans aucun warning).
- **Zéro warning du compilateur** : code propre, types rigoureux, zéro unsafe inutile.

---

## 1. Structure du Répertoire Rust

```
crates/
├── glucose-core/
│   ├── Cargo.toml               # [dependencies] STRICTEMENT VIDE (0 externe)
│   ├── src/
│   │   ├── lib.rs               # Exports publics et 37 tests unitaires du store
│   │   ├── types.rs             # Modèles Board, BoardImage, Annotation, Viewport, Project, etc.
│   │   ├── geometry.rs          # Rect, Point, intersections, OBB, bandes de bordure
│   │   ├── quadtree.rs          # SpatialHash pour viewport culling ultra-rapide
│   │   ├── hit_priority.rs      # Arbitre PICK-1 : 7 rangs, handle slop 36px, cyclage, terminus texte
│   │   ├── smart_align.rs       # Guides SNAP-1 : snap_move, snap_resize, seuil écran constant
│   │   ├── membrane_space.rs    # Repères locaux, échelle déduite k = min(1, ...), appartenance stockée
│   │   ├── membrane_stretch.rs  # Planification d'étirement avec butée sur obstacles
│   │   ├── membrane_focus.rs    # Focus asymétrique (entrée >= 92%, sortie <= 0.8)
│   │   ├── curtain_model.rs     # Permissions rideaux (carnet/vitrine/atelier), sanitisation
│   │   ├── curtain_panel.rs     # Timers de dwell, survol sans battement
│   │   ├── arrow_anchor.rs      # Mathématiques de sortie de périmètre, invariant anti-inversion
│   │   ├── text_anchors.rs      # Ancrage sub-block W3C robuste aux éditions textuelles
│   │   ├── timeline.rs          # Calendrier astronomique BC/AD (-100Ma à 3000)
│   │   ├── mirror_graph.rs      # Détecteur BFS de cycles pour miroirs et dossiers
│   │   ├── bundle.rs            # SHA-256 natif pur Rust std, déduplication et vérification
│   │   ├── store.rs             # Store central avec undo/redo et préservation de caméra
│   │   └── export.rs            # Exportateurs SVG vectoriel et Markdown en pur Rust std
│   └── tests/                   # 14 suites d'intégration exhaustives
│
└── glucose-desktop/
    ├── Cargo.toml               # winit 0.30, softbuffer 0.4, tiny-skia 0.11, image 0.25
    └── src/
        ├── main.rs              # Point d'entrée de l'application
        ├── app.rs               # Winit event loop, drag-and-drop OS, raccourcis PureRef
        ├── canvas.rs            # Mappings Écran <-> Monde et zoom centré curseur
        ├── renderer.rs          # Rendu 2D logiciel (grille, membranes, images, sélecteurs, HUD)
        ├── font.rs              # Moteur de rendu de glyphes bitmap 8x8 pur Rust
        └── font_data.rs         # Table binaire de glyphes ASCII/Latin
```

---

## 2. Les 6 Invariants Architecturaux Fondamentaux

Ces 6 invariants sont scrupuleusement respectés et validés par les tests d'intégration :

### ① L'échelle du contenu ne se stocke pas, elle se déduit
$$k = \min\left(1, \frac{\text{largeur}}{\text{étendue}_X}, \frac{\text{hauteur}}{\text{étendue}_Y}\right)$$
Aucun champ `scale` mutable n'est stocké dans le modèle de données. L'échelle est une projection dynamique géométrique. Si la membrane s'agrandit, le contenu retrouve sa taille naturelle (plafonnée à 1.0).

### ② L'appartenance à une membrane est persistée (`membrane_id`)
Une membrane minimisée est par définition plus petite que son contenu naturel à l'échelle 1. Un calcul géométrique d'inclusion continue la viderait dès qu'elle se réduit. L'appartenance est donc un événement stocké (`membrane_id`).

### ③ La conversion classique vers minimisée/étirée est le seul moment géométrique
En mode `classic`, l'échelle vaut 1 : l'inclusion géométrique est sans ambiguïté. Dès que la membrane bascule, l'appartenance est scellée par événement.

### ④ Le chemin rapide `has_scaling()`
Tant qu'aucune membrane n'est minimisée, `project_board` renvoie les coordonnées du board de façon directe sans allocation ni projection inutile.

### ⑤ Isolation des états et robustesse documentaire
Toutes les mutations du document s'opèrent avec sanitisation et vérification des clés (`detach_curtains`, etc.), évitant toute corruption d'arbre.

### ⑥ Magnétisme SNAP-1 et conversion incrémentale
Une session d'aimantation convertit le déplacement cumulé depuis le début du drag en delta incrémental pour le store. Le seuil d'accroche (en pixels écran) est divisé par le zoom courant, garantissant une ergonomie rigoureusement constante.

---

## 3. Détail des Fonctionnalités Livrées

### Sélection & Cyclage (PICK-1)
- Hiérarchie stricte en 7 rangs : Poignées de redimensionnement > Bordures actives de conteneurs > Flèches > Images > Stickies > Textes > Intérieur de conteneurs.
- Cyclage vers la cible suivante au relâchement de clics successifs immobiles.
- Terminus sur texte et stickies pour autoriser le double-clic d'édition immédiat.

### Moteur de Membranes
- 3 modes supportés : `Classic`, `Minimized`, `Stretched`.
- Mode Stretched avec détection fine de collision d'obstacles : la membrane grandit avec son contenu mais s'arrête net sans recouvrir les éléments extérieurs.
- Mode Focus avec entrée asymétrique ($\ge 92\%$ de couverture écran) et sortie au dézoom ($\le 0.8$).

### Graphe de Miroirs & Anti-Inception
- Algorithme BFS vérifiant l'acyclicité avant toute création de miroir ou déplacement de conteneur.
- Prévention garantie de toute récursion infinie ou boucle miroir.

### Moteur Undo / Redo & Préservation de Caméra
- Undo/Redo illimité.
- **Transparence de navigation** : `set_viewport`, `pan` et `zoom` ne créent AUCUNE entrée dans la pile d'historique.
- `undo()` et `redo()` restaurent les données géométriques tout en préservant intacte la position de caméra de l'utilisateur.
- Sessions `begin_live_edit()` et `end_live_edit()` assurant qu'un drag de 50 frames ne génère qu'un seul commit.

### SHA-256 & Bundles en Pur Rust
- Implémentation complète de l'algorithme SHA-256 standard en pur Rust `std` sans aucune bibliothèque cryptographique externe.
- Déduplication d'images et de ressources par hachage de contenu.
- Exportations autonomes SVG et Markdown.

---

## 4. Compilation et Tests

```bash
# Lancer les 202 tests Rust
cargo test --workspace

# Vérifier la compilation de tous les crates
cargo check --workspace

# Lancer l'application PureRef native en mode release
cargo run -p glucose-desktop --release
```

---

<div align="center">

**Glucose, c'est juste poser, relier, zoomer, explorer.**

[← Retour au README](README.md) · [Guide d'utilisation](GUIDE.md)

</div>

