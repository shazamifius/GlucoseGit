<div align="center">

# 🧬 Glucose

### Une surface cognitive infinie pour penser sans limites

*Pose. Relie. Zoome. Explore.*

[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg?style=flat-square)](LICENSE)
[![Version](https://img.shields.io/badge/version-0.3.0-green.svg?style=flat-square)](../../releases/latest)
[![Tauri](https://img.shields.io/badge/Tauri-2-orange.svg?style=flat-square&logo=tauri)](https://tauri.app)
[![React](https://img.shields.io/badge/React-19-61dafb.svg?style=flat-square&logo=react)](https://react.dev)
[![Rust](https://img.shields.io/badge/Rust-stable-CE422B.svg?style=flat-square&logo=rust)](https://www.rust-lang.org/)
[![Tests](https://img.shields.io/badge/tests-304%20passing-brightgreen.svg?style=flat-square)](#)

[**📥 Télécharger**](../../releases/latest) · [**📖 Guide**](GUIDE.md) · [**🗺️ Roadmap**](ROADMAP.md) · [**🐛 Issues**](../../issues)

</div>

---

## ✨ Qu'est-ce que c'est ?

**Glucose** est un canvas infini desktop pour penser à long terme : recherche, écriture, design de monde, prototypage de gameplay, conlanging, cartographie de connaissance.

Une seule interface — pas de modes. Pose ce que tu veux, relie comme tu veux, zoome jusqu'à l'infini.

> Pensez Miro / FigJam, mais **offline, sans modes**, conçu pour l'exploration profonde plutôt que les sessions de brainstorm courtes.

## 🌟 Fonctionnalités

<table>
<tr>
<td width="50%">

### 🎨 Canvas infini
- Pan / zoom illimité, fluide à 60 FPS
- Multi-boards imbriqués (**dossiers zoomables** — entre dans un dossier en zoomant dessus)
- Membranes auto autour des clusters d'images
- Drag-create dossier qui **capture** ce qui est dessous

</td>
<td width="50%">

### 🔗 Relations sémantiques
- Flèches avec **prédicats typés** (inspire, contredit, hérite_de…)
- Sub-block targeting — pointer un paragraphe précis
- Pathfinding anti-obstacles automatique
- Liens **trans-domaines** en pointillés

</td>
</tr>
<tr>
<td width="50%">

### 🪞 Miroirs (alias vivants)
- Copie d'un nœud — modifier l'original propage partout
- **Garde-fou anti-Inception** : interdit les cycles infinis
- Téléportation cliquable vers l'original

</td>
<td width="50%">

### 🌈 Domaines & Temporel
- Catégoriser tes nœuds (Science, Art, Histoire…)
- Couleurs des membranes dérivées des domaines
- **Réglette temporelle** : ancrer un nœud à 1789, à la Renaissance, ou -3000 av. J.-C.

</td>
</tr>
<tr>
<td width="50%">

### ⏳ Time Machine (CRDT Automerge)
- **Undo / Redo infini** natif
- Slider d'historique (Ctrl+H) — drag pour voyager
- **Jalons nommés** 📌 cliquables
- Restauration sans perdre l'historique antérieur

</td>
<td width="50%">

### 🛰️ Multi-utilisateur LAN
- Découverte mDNS automatique sur le réseau local
- Synchronisation en temps réel via WebSocket
- Merge CRDT transparent — pas de conflits
- Activation Ctrl+Shift+L

</td>
</tr>
<tr>
<td width="50%">

### 📥 Multimédia & App Bridge
- Drag-drop images / vidéos depuis le navigateur
- Import URL YouTube / TikTok / Instagram (yt-dlp embarqué)
- Ouverture native `.blend`, `.psd`, `.kra` (Blender, Photoshop, Krita…)

</td>
<td width="50%">

### 🛡️ Privé & sécurisé
- 100% offline, sauvegarde locale `.glucose`
- Validation Zod des fichiers chargés
- Scope checks stricts sur toutes les commandes natives
- Anti-XSS, anti-SSRF, capabilities Tauri minimales

</td>
</tr>
</table>

## 📥 Installation

Téléchargements sur la **[page Releases](../../releases/latest)** — un seul fichier à choisir selon ton OS.

### 🪟 Windows 10 / 11

1. Télécharger `Glucose_0.3.0_x64-setup.exe`
2. Double-cliquer · *pas de droits admin requis*
3. WebView2 est embarqué (silencieux si manquant)

### 🍎 macOS

Télécharger `Glucose_0.3.0_aarch64.dmg` — fonctionne **nativement sur Apple Silicon (M1/M2/M3)** et via **Rosetta 2 sur Intel**.

Ouvrir le `.dmg`, glisser Glucose dans `Applications`. **Premier lancement** : clic-droit → Ouvrir (Gatekeeper non signé pour l'instant).

> Sur Mac Intel sans Rosetta 2 installé, le système le proposera automatiquement au premier lancement.

### 🐧 Linux

**AppImage (universel, marche partout)** :
```bash
wget https://github.com/shazamifius/GlucoseGit/releases/latest/download/glucose_0.3.0_amd64.AppImage
chmod +x glucose_0.3.0_amd64.AppImage
./glucose_0.3.0_amd64.AppImage
```

**Debian / Ubuntu / dérivés** :
```bash
wget https://github.com/shazamifius/GlucoseGit/releases/latest/download/glucose_0.3.0_amd64.deb
sudo dpkg -i glucose_0.3.0_amd64.deb
sudo apt-get install -f  # résout les deps manquantes si besoin
```

**Fedora / RHEL / openSUSE** :
```bash
sudo dnf install ./glucose-0.3.0-1.x86_64.rpm
```

**❄️ NixOS** (via flake) :
```bash
# Lancer une fois sans installer :
nix run github:shazamifius/GlucoseGit

# Installer dans le profil utilisateur :
nix profile install github:shazamifius/GlucoseGit
```

> Pour AppImage sur NixOS pur, active `programs.nix-ld.enable = true;` ou utilise `nix-shell -p appimage-run`.

### Désinstallation

| OS | Commande |
|---|---|
| Windows | Panneau Config → Programmes → Désinstaller "Glucose" |
| macOS | Glisser `Applications/Glucose.app` à la corbeille |
| Linux `.deb` | `sudo apt-get remove glucose` |
| Linux `.rpm` | `sudo dnf remove glucose` |
| Linux AppImage | Supprimer le fichier |
| NixOS | `nix profile remove glucose` |

> Tes données (`%APPDATA%/Glucose` sur Win, `~/Library/Application Support/Glucose` sur Mac, `~/.config/Glucose` sur Linux) sont **conservées**. Effacer manuellement si désiré.

## ⚡ Démarrage rapide

À l'ouverture, l'app crée un projet vierge. **Touches principales** :

| Touche | Action |
|---|---|
| `V` | Outil sélection |
| `T` | Texte |
| `N` | Note sticky |
| `A` | Flèche (entre deux blocs) |
| `F` | Dossier (drag pour dessiner) |
| `M` | Membrane (zone colorée) |
| `Espace` | Pan (maintenu) |
| `Ctrl+Z` / `Ctrl+Y` | Undo / Redo infini |
| `Ctrl+H` | Time Machine |
| `Ctrl+Shift+L` | Multijoueur LAN |
| `Shift+R` | Réglette temporelle |
| `Shift+T` | Ancrer une date à la sélection |
| `F11` | Mode Zen (cache toute l'UI) |

📚 **[GUIDE.md](GUIDE.md)** contient le manuel complet (concepts, tous les raccourcis, workflows types).

## 🏗️ Stack technique

```
┌──────────────────────────────────────────────────┐
│  React 19 + TypeScript + Tailwind 4              │
│            ↓                                     │
│  Zustand + Automerge 3 (CRDT)  ·  Zod            │
│            ↓                                     │
│  PixiJS 8 (raster)  +  SVG overlay (vectoriel)   │
│            ↓                                     │
│  Tauri 2 (Rust)                                  │
│  · reqwest (rustls)  · yt-dlp embarqué           │
│  · mDNS-SD + tokio-tungstenite (multijoueur LAN) │
└──────────────────────────────────────────────────┘
```

## 🛠️ Build depuis les sources

**Prérequis** :
- [Rust toolchain](https://rustup.rs/) stable
- [Bun](https://bun.sh/) ou Node 20+
- Linux : voir [tauri prereqs](https://tauri.app/start/prerequisites/) (`libwebkit2gtk-4.1-dev`, etc.)

```bash
git clone https://github.com/shazamifius/GlucoseGit
cd GlucoseGit
bun install
bun run tauri dev      # mode développement
bun run tauri build    # build release
```

Tests + lint :
```bash
bun run typecheck      # tsc --noEmit
bun run test           # vitest (55 tests)
bun run lint           # Biome
```

L'artefact release est dans `src-tauri/target/release/bundle/`.

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

</div>
