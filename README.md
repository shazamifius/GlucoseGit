<div align="center">

# 🧬 Glucose

### Une surface cognitive infinie pour penser sans limites

*Pose. Relie. Zoome. Explore.*

[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg?style=flat-square)](LICENSE)
[![Version](https://img.shields.io/badge/version-0.2.0-green.svg?style=flat-square)](../../releases/latest)
[![Tauri](https://img.shields.io/badge/Tauri-2-orange.svg?style=flat-square&logo=tauri)](https://tauri.app)
[![React](https://img.shields.io/badge/React-19-61dafb.svg?style=flat-square&logo=react)](https://react.dev)
[![TypeScript](https://img.shields.io/badge/TypeScript-5.8-3178c6.svg?style=flat-square&logo=typescript)](https://www.typescriptlang.org/)
[![Rust](https://img.shields.io/badge/Rust-stable-CE422B.svg?style=flat-square&logo=rust)](https://www.rust-lang.org/)
[![Tests](https://img.shields.io/badge/tests-20%20passing-brightgreen.svg?style=flat-square)](src/canvas/lod.test.ts)

[**📥 Télécharger**](../../releases/latest) · [**📖 Guide**](GUIDE.md) · [**🗺️ Roadmap**](ROADMAP.md) · [**🐛 Issues**](../../issues)

</div>

---

## ✨ Qu'est-ce que c'est ?

**Glucose** est un **canvas infini** desktop pour les projets cognitifs profonds : recherche, écriture, design de monde, prototypage de gameplay, conlanging, cartographie de connaissance.

Une seule interface — pas de modes. Pose ce que tu veux, relie comme tu veux, zoome jusqu'à l'infini.

> Pensez Miro/FigJam, mais offline, sans modes, conçu pour le long terme et l'exploration profonde plutôt que les sessions de brainstorm courtes.

## 🌟 Fonctionnalités clés

<table>
<tr>
<td width="50%">

### 🎨 Canvas infini
- Pan/zoom illimité, fluide à 60 FPS
- Multi-boards imbriqués (**dossiers zoomables**)
- Niveaux de détail (LOD) automatiques selon le zoom
- Membranes colorées auto-générées autour des clusters

</td>
<td width="50%">

### 🔗 Relations sémantiques
- Flèches avec **prédicats typés** (inspire, contredit, hérite_de…)
- Sub-block targeting — pointer un paragraphe précis
- Pathfinding anti-obstacles automatique
- Détection des liens **trans-domaines** en pointillés

</td>
</tr>
<tr>
<td width="50%">

### 🪞 Miroirs (alias)
- Copie vivante d'un nœud — modifier l'original propage partout
- **Garde-fou anti-Inception** : interdit les cycles infinis
- Téléportation cliquable vers l'original

</td>
<td width="50%">

### 🌈 Domaines sémantiques
- Catégoriser tes nœuds (Science, Art, Histoire…)
- Couleurs des membranes dérivées des domaines pondérés
- Badges visuels d'appartenance

</td>
</tr>
<tr>
<td width="50%">

### 📥 Multimédia & App Bridge
- Drag-drop images / vidéos locales
- Import URL YouTube / TikTok / Instagram via yt-dlp
- Ouverture native `.blend`, `.psd`, `.kra` (Blender, Photoshop, Krita…)

</td>
<td width="50%">

### 🛡️ Privé & sécurisé
- 100% offline, sauvegarde locale `.glucose`
- Validation Zod des fichiers chargés
- Scope checks stricts sur toutes les commandes natives
- Anti-XSS Markdown, anti-SSRF, capabilities Tauri minimales

</td>
</tr>
</table>

## 📥 Installation

Téléchargements directs : **[releases](../../releases/latest)**

| OS | Format | Notes |
|---|---|---|
| 🪟 **Windows** | `Glucose_x.y.z_x64-setup.exe` | NSIS installer · Pas de droits admin · WebView2 embarqué |
| 🍎 **macOS** | `Glucose_x.y.z_x64.dmg` / `_aarch64.dmg` | Intel ou Apple Silicon |
| 🐧 **Linux** | `glucose_x.y.z_amd64.AppImage` | Universel · Marche aussi sur NixOS |
| 📦 **Debian/Ubuntu** | `glucose_x.y.z_amd64.deb` | `sudo dpkg -i` |
| ❄️ **NixOS** | flake | `nix run github:shazamifius/GlucoseGit` |

> Voir [INSTALL.md](INSTALL.md) pour les instructions détaillées par OS.

## ⚡ Démarrage rapide

```bash
bun install
bun run tauri dev
```

L'app s'ouvre. **Touches principales** :

| Touche | Outil |
|---|---|
| `V` | Sélection |
| `T` | Texte |
| `N` | Note sticky |
| `A` | Flèche |
| `F` | Dossier |
| `M` | Membrane |
| `Espace` | Pan (maintenu) |
| `F11` | Mode Zen |

📚 **[GUIDE.md](GUIDE.md)** contient le manuel utilisateur complet (vocabulaire, tous les raccourcis, workflows types).

## 🏗️ Stack technique

```
┌─────────────────────────────────────────────────┐
│  React 19 + TypeScript + Tailwind 4             │
│           ↓                                     │
│  Zustand (état) · Zod (validation)              │
│           ↓                                     │
│  PixiJS 8 (raster: images, vidéos)              │
│  SVG overlay (vectoriel: texte, flèches)        │
│           ↓                                     │
│  Tauri 2 (Rust backend)                         │
│  reqwest (rustls) · yt-dlp embarqué             │
└─────────────────────────────────────────────────┘
```

## 🗺️ Architecture

```
src/
├── canvas/              Renderers (PixiJS + SVG) + hooks rendu
│   ├── lod.ts           Level-of-detail anti-spaghetti (Phase 2)
│   ├── MembraneRenderer Union-Find + Convex Hull + couleurs domaines
│   └── AnnotationBadges Composants miroir/domaines extraits
├── components/          UI panels et composants
│   ├── ArrowDescriptionPanel  Markdown long sur les flèches (Phase 5)
│   ├── DomainsPanel           Création/assignation domaines (Phase 3)
│   └── …
├── store/               Zustand + cycle detector miroirs
│   ├── mirrorGraph.ts   ⚠️ Anti-Inception (BFS strict)
│   └── projectSchema.ts Validation Zod des .glucose
├── utils/               Helpers (project save/load, glucoseBus, etc.)
└── types/               Types TypeScript partagés

src-tauri/
├── src/lib.rs           Commandes Tauri sécurisées (scope checks)
├── capabilities/        Permissions Tauri minimales
└── tauri.conf.json      Config app + bundle multi-OS
```

## 🛠️ Développement

```bash
bun install              # installe les deps JS
bun run dev              # vite seul (sans Tauri)
bun run tauri dev        # app desktop en dev
bun run typecheck        # tsc --noEmit
bun run lint             # Biome lint
bun run test             # vitest (20 tests sur mirrorGraph + lod)
bun run tauri build      # build release
```

### Prérequis
- [Rust toolchain](https://rustup.rs/) stable
- [Bun](https://bun.sh/) ou Node 20+
- Linux : voir [tauri prereqs](https://tauri.app/start/prerequisites/)

## 🔒 Sécurité

| Mesure | État |
|---|---|
| Scope check sur toutes les commandes Tauri (path traversal) | ✅ |
| Whitelist d'extensions à l'ouverture native (anti-RCE) | ✅ |
| Anti-SSRF dans `fetch_image` (IPs privées bloquées) | ✅ |
| `rehypeRaw` retiré (anti-XSS Markdown) | ✅ |
| Validation Zod des fichiers `.glucose` | ✅ |
| Capabilities Tauri minimales (pas de `home-recursive`) | ✅ |
| Coordonnées extrêmes clampées (anti-NaN PixiJS) | ✅ |
| Anti-cycle miroir Inception | ✅ |

Voir [CLEANUP.md](CLEANUP.md) pour l'audit complet.

## 📖 Documentation

- 📘 **[GUIDE.md](GUIDE.md)** — Manuel utilisateur A → Z
- 🗺️ **[ROADMAP.md](ROADMAP.md)** — Plan de développement (Phases 0-11)
- 🧹 **[CLEANUP.md](CLEANUP.md)** — Audit qualité / sécurité
- 📦 **[INSTALL.md](INSTALL.md)** — Instructions install par OS

## 🤝 Contribuer

Contributions bienvenues ! Avant de soumettre une PR :
- `bun run typecheck` passe
- `bun run test` passe
- `bun run lint` n'a pas d'erreurs (warnings tolérés)

Pour les changements importants, ouvre d'abord une [issue](../../issues).

## 📄 License

[MIT](LICENSE) — utilisation, modification et redistribution libres.

---

<div align="center">

**Glucose, c'est juste poser, relier, zoomer, explorer.**

*Made with ❤️ — un projet de [@shazamifius](https://github.com/shazamifius)*

</div>
