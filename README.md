<div align="center">

# 🧬 Glucose

### Le canvas de référence infini natif en Rust — style PureRef, 0 boîte noire, 0 dépendance dans le core

*Pose. Relie. Zoome. Explore.*

[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg?style=flat-square)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-100%25%20pure%20std%20core-CE422B.svg?style=flat-square&logo=rust)](https://www.rust-lang.org/)
[![Dependencies](https://img.shields.io/badge/core%20dependencies-0%20(zero)-brightgreen.svg?style=flat-square)](#)
[![Tests](https://img.shields.io/badge/tests-204%20passing%20(100%25)-brightgreen.svg?style=flat-square)](#)
[![Compatibility](https://img.shields.io/badge/OS%20compatibility-100%25%20kernels-blueviolet.svg?style=flat-square)](#)

[**📖 Guide**](GUIDE.md) · [**🗺️ Architecture & Handoff**](HANDOFF.md) · [**🐛 Issues**](../../issues)

</div>

---

## ✨ Qu'est-ce que c'est ?

**Glucose** est une plateforme visuelle desktop native ultra-rapide, inspirée de **PureRef**, réécrite **intégralement en Rust from scratch** :

- 🎨 **PureRef & Moodboards** — référence visuelle infinie, drag & drop d'images instantané (WebP, PNG, JPEG, GIF, BMP), collage depuis le presse-papiers (`Ctrl+V` depuis Pinterest ou le web), fenêtre always-on-top (`Alt+T`), zoom au curseur, pan fluide.
- 📐 **Interface Glucose complète** — TopBar (44px) avec tous les outils vectoriels (`Select V`, `Pan Espace`, `Text T`, `Sticky N`, `Arrow A`, `Folder F`, `Membrane M`), bouton `+ Images`, `Ordonner`, `Timer`, `Storyboard`, `Aimant` (SNAP-1), `Trans-domaines`, `Collaborer`, `Exporter`, `Plugins`, `Preset`, `Domaines`.
- 📑 **Onglets de Boards (34px)** — gestion multi-boards dynamique, création instantanée (`+`), bascule fluide et minimap interactive (180x120px) en temps réel.
- 🍞 **Toasts d'action animés** — feedback visuel immédiat en fondu alpha (`📥 Image collée`, `📐 Canvas ordonné`, etc.).
- 🧬 **Membranes adaptatives** — regroupements intelligents d'images et de notes avec facteur d'échelle dynamique $k = \min(1, \dots)$, mode focus plein écran et étirement anti-collision.
- 🗂️ **Dossiers zoomables & Miroirs** — sous-canvas imbriqués avec détection de cycles acycliques (anti-Inception) pour dupliquer des vues vivantes.
- 🔗 **Relations sémantiques** — flèches orientées avec prédicats typés (`inspire`, `contredit`, `dépend_de`), sub-block text anchoring et calcul de contour géométrique.
- 🛡️ **0 boîte noire, 0 dépendance dans le moteur** — `glucose-core` fonctionne à 100% avec la bibliothèque standard Rust (`std`), sans aucune crate externe (`[dependencies]` strictement vide).
- 🖥️ **Compatibilité 100% OS & Kernels** — Rendu logiciel universel vectoriel (`tiny-skia` + `softbuffer` + `winit`) sans aucune exigence de pilote GPU propriétaire. Tourne partout : Windows, Linux (Wayland/X11), macOS, BSD.

---

## 🏗️ Architecture du Workspace

Le projet est structuré en un workspace Rust propre et modulaire :

```
GlucoseGit/
├── crates/
│   ├── glucose-core/       # 100% PURE RUST STD (0 DÉPENDANCE EXTERNE)
│   │   ├── src/
│   │   │   ├── types.rs           # Modèles de données purs (Board, Image, Annotation, etc.)
│   │   │   ├── geometry.rs        # Primitives géométriques, boîtes orientées, bandes de bordure
│   │   │   ├── quadtree.rs        # SpatialHash déterministe pour culling de viewport ultra-rapide
│   │   │   ├── hit_priority.rs    # Arbitre PICK-1 (7 rangs de priorité, cycle de clic, terminus texte)
│   │   │   ├── smart_align.rs     # Magnétisme intelligent SNAP-1 (guides d'alignement, seuil écran)
│   │   │   ├── membrane_space.rs  # Repère local, échelle déduite k = min(1, ...), appartenance stockée
│   │   │   ├── membrane_stretch.rs# Planification d'étirement avec contournement d'obstacles
│   │   │   ├── membrane_focus.rs  # Focus asymétrique (entrée >= 92%, sortie <= 0.8)
│   │   │   ├── curtain_model.rs   # Permissions (carnet/vitrine/atelier), sanitisation de notes
│   │   │   ├── curtain_panel.rs   # Timers de dwell, détection de hover sans battement
│   │   │   ├── arrow_anchor.rs    # Mathématiques de sortie de périmètre, invariant anti-inversion
│   │   │   ├── text_anchors.rs    # Ancrage sub-block W3C robuste aux éditions textuelles
│   │   │   ├── timeline.rs        # Calendrier astronomique BC/AD (-100Ma à 3000)
│   │   │   ├── mirror_graph.rs    # Détecteur BFS de cycles pour miroirs et dossiers
│   │   │   ├── bundle.rs          # SHA-256 natif en pur Rust std, déduplication et vérification
│   │   │   ├── store.rs           # Store central avec annulation/rétablissement et préservation de caméra
│   │   │   └── export.rs          # Exportateurs SVG vectoriel et Markdown en pur Rust std
│   │   └── tests/                 # 14 suites de tests d'intégration (165 tests + 37 unitaires = 202)
│   │
│   └── glucose-desktop/    # CLIENT DESKTOP NATIF PUREREF
│       ├── assets/                # Polices vectorielles KaTeX intégrées au binaire
│       ├── src/
│       │   ├── canvas.rs          # Mappings coordonnées Écran <-> Monde et zoom centré curseur
│       │   ├── typography.rs      # Typographie vectorielle anti-aliasée TrueType (fontdue)
│       │   ├── icons.rs           # Tracé vectoriel de toutes les icônes de Glucose (tiny-skia)
│       │   ├── ui.rs              # TopBar (44px), BoardTabs (34px), Minimap (180x120), Toasts animés
│       │   ├── renderer.rs        # Rendu 2D haute fidélité (fond #0D0E12, grille, halos, pilules #18181B)
│       │   ├── app.rs             # Contrôleur Winit 0.30 + Softbuffer 0.4 + Presse-papiers (Ctrl+V)
│       │   └── main.rs            # Point d'entrée de l'application
```

---

## 🌟 Fonctionnalités

<table>
<tr>
<td width="50%">

### 🎨 Expérience PureRef native
- Pan fluide (clic du milieu, clic droit ou Espace)
- Zoom continu centré précisément sous le curseur de la souris
- Drag-and-drop instantané d'images depuis l'explorateur de fichiers OS
- Mode Always-on-Top commutable (`T`) pour survoler Blender/Photoshop/Krita
- Cadrage automatique (`Espace` ou `F`)

</td>
<td width="50%">

### 🧬 Membranes & Repères locaux
- 3 modes : `Classic`, `Minimized`, `Stretched`
- Échelle déduite automatique $k = \min(1, \dots)$
- Mode Focus plein écran avec fond teinté dynamique
- Planification d'étirement qui s'arrête strictement sur les obstacles
- Détection d'appartenance événementielle (`membrane_id`)

</td>
</tr>
<tr>
<td width="50%">

### 🎯 Sélection & Magnétisme (PICK-1 & SNAP-1)
- Arbitre de hit à 7 rangs (poignée > bord > flèche > image > sticky > texte > fond)
- Cyclage intelligent de sélection par clics consécutifs
- Guides d'alignement automatiques à l'écran
- Seuil de capture constant en pixels écran quel que soit le niveau de zoom

</td>
<td width="50%">

### 🪞 Miroirs & Dossiers imbriqués
- Copies vivantes et synchronisées d'éléments et de dossiers
- Détection de cycle BFS anti-Inception
- Téléportation instantanée vers la source originale
- Capture spatiale automatique à la création de dossier

</td>
</tr>
<tr>
<td width="50%">

### ⏳ Undo / Redo & Transparence caméra
- Annulation / rétablissement infini
- **Navigation transparente** : naviguer (pan, zoom) n'est jamais enregistré dans la pile undo
- Préservation rigoureuse de la caméra lors d'un `Ctrl+Z` ou `Ctrl+Y`
- Sessions de drag atomiques (`begin_live_edit` / `end_live_edit`)

</td>
<td width="50%">

### 📦 Bundles & Export natif
- Hachage cryptographique **SHA-256 implémenté en pur Rust `std`**
- Déduplication de contenu sans perte
- Export SVG autonome sans dépendance
- Export Markdown structuré hiérarchique

</td>
</tr>
</table>

---

## ⚡ Raccourcis clavier & Contrôles

| Action | Raccourci / Geste |
|---|---|
| **Pan (déplacer la vue)** | Clic milieu glissé, ou Clic droit glissé, ou `Espace` + Clic gauche |
| **Zoom au curseur** | Molette de la souris |
| **Sélectionner** | Clic gauche (cyclage multi-niveaux si empilé) |
| **Multi-sélection** | `Ctrl` + Clic gauche |
| **Déplacer la sélection** | Clic gauche glissé sur un élément |
| **Ajouter des images** | Glisser-déposer des fichiers image directement dans la fenêtre |
| **Recentrer / Cadrer tout** | `Espace` (sans drag) ou `F` |
| **Always on Top** | `T` (épingle la fenêtre au-dessus des autres applications) |
| **Supprimer** | `Suppr` ou `Backspace` |
| **Dupliquer** | `Ctrl+D` |
| **Annuler / Rétablir** | `Ctrl+Z` / `Ctrl+Y` |
| **Quitter** | `Échap` ou fermer la fenêtre |

---

## 🛠️ Compilation & Tests

### Prérequis
- [Rust toolchain](https://rustup.rs/) (édition 2021 stable ou plus récente).
- Aucun outil tiers, aucun Node.js, aucun GPU requis.

### Lancer tous les tests (202 tests verts)
```bash
cargo test --workspace
```

### Lancer l'application PureRef native
```bash
cargo run -p glucose-desktop --release
```

### Compiler les binaires de production
```bash
cargo build --release -p glucose-desktop
```
L'exécutable portable et autonome se trouve dans `target/release/glucose-desktop.exe` (ou `glucose-desktop` sous Linux/macOS).

---

## 📄 Licence

[MIT](LICENSE) — utilisation, modification et redistribution libres.

---

<div align="center">

**Glucose, c'est juste poser, relier, zoomer, explorer.**

</div>

