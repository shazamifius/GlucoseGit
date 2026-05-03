# Installation de Glucose

## Windows

1. [Télécharger `Glucose_*_x64-setup.exe`](https://github.com/shaza/glucose/releases/latest)
2. Double-cliquer
3. L'app s'installe dans `%LOCALAPPDATA%\Programs\Glucose` (pas de droits admin)

WebView2 est embarqué : si manquant, le bootstrapper l'installe silencieusement.

## macOS

1. [Télécharger `Glucose_*.dmg`](https://github.com/shaza/glucose/releases/latest)
   - **Apple Silicon (M1+)** : `Glucose_*_aarch64.dmg`
   - **Intel** : `Glucose_*_x64.dmg`
2. Ouvrir, glisser Glucose dans Applications
3. Premier lancement : clic-droit → Ouvrir (contournement Gatekeeper non signé)

> Pour une signature officielle (sans warning Gatekeeper), il faut un certificat
> Apple Developer (~99 $/an). Pas encore activé.

## Linux

### Le plus simple : AppImage (universel, **incluant NixOS**)

```bash
# Télécharger
wget https://github.com/shaza/glucose/releases/latest/download/glucose_0.2.0_amd64.AppImage

# Rendre exécutable
chmod +x glucose_0.2.0_amd64.AppImage

# Lancer
./glucose_0.2.0_amd64.AppImage
```

#### NixOS spécifique

L'AppImage utilise des libs système. Sur NixOS pur (pas FHS), il faut soit :

**Option A — `nix-ld`** (recommandé, déjà actif sur la plupart des configs récentes) :
```nix
# Dans configuration.nix
programs.nix-ld.enable = true;
programs.nix-ld.libraries = with pkgs; [
  webkitgtk_4_1
  glib
  gtk3
  libsoup_3
];
```

**Option B — `appimage-run`** (plus simple, pas de config) :
```bash
nix-shell -p appimage-run --run "appimage-run glucose_0.2.0_amd64.AppImage"
```

### Debian / Ubuntu : .deb

```bash
wget https://github.com/shaza/glucose/releases/latest/download/glucose_0.2.0_amd64.deb
sudo dpkg -i glucose_0.2.0_amd64.deb
sudo apt-get install -f  # résout les dépendances manquantes si nécessaire
```

### NixOS via Flake (puristes)

```bash
# Lancer une seule fois sans installer
nix run github:shaza/glucose

# Installer dans le profil utilisateur
nix profile install github:shaza/glucose
```

> Le flake build Glucose depuis les sources. Première compilation ~10 min.
> Voir [`flake.nix`](flake.nix) pour les détails.

### Arch Linux

Pas encore packagé. Possible via AUR à l'avenir.

## Build depuis les sources

```bash
git clone https://github.com/shaza/glucose
cd glucose
bun install
bun run tauri build
```

Prérequis :
- [Rust toolchain](https://rustup.rs/) stable
- [Bun](https://bun.sh) ou Node 20+
- Linux : `libwebkit2gtk-4.1-dev`, `libgtk-3-dev`, etc. ([liste complète Tauri](https://tauri.app/start/prerequisites/))
- macOS : Xcode Command Line Tools
- Windows : Visual Studio Build Tools + WebView2 Runtime

L'artefact est dans `src-tauri/target/release/bundle/`.

## Désinstallation

| OS | Commande |
|---|---|
| Windows | Panneau Config → Programmes → Désinstaller "Glucose" |
| macOS | Glisser Glucose vers la Corbeille |
| Linux .deb | `sudo apt-get remove glucose` |
| Linux AppImage | Supprimer le fichier |
| NixOS profile | `nix profile remove glucose` |

Les données utilisateur (`%APPDATA%/Glucose` sur Win, `~/Library/Application Support/Glucose` sur Mac, `~/.config/Glucose` sur Linux) sont **conservées**. Effacer manuellement si besoin.
