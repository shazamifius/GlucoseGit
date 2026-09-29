#!/bin/bash
# Construit les trois paquets Linux de Glucose Rust (fiche 48) : le .deb, le .rpm et l'AppImage.
#
# Ils reprennent la disposition exacte des paquets de Glucose Tauri — le paquet `glucose`, le
# programme /usr/bin/glucose, le fichier de bureau Glucose.desktop, les icônes `glucose` — pour
# que l'updater de Tauri pose chacun par-dessus le sien : `dpkg -i`, `rpm -U`, ou l'AppImage
# réécrit.
#
# **Leurs dépendances** : `dpkg -i` et `rpm -U` n'installent RIEN de ce qui manque — et un
# `dpkg -i` aux dépendances manquantes remplace quand même les fichiers, puis laisse le paquet
# à moitié configuré. Seul GTK 3 est donc exigé : Glucose Tauri l'exigeait déjà. Le reste est
# recommandé : Vulkan, dont Glucose se passe (la présentation par le processeur, fiche 36) ; et
# le clavier sous X11 (libxkbcommon-x11), dont il ne se passe PAS sur une session X11 — GNOME,
# KDE et Cinnamon l'installent déjà, et une session Wayland, le défaut d'aujourd'hui, n'en a
# pas besoin (fiche 48).
#
# Usage : bash outils/paquets/construire.sh <glucose-desktop> <dossier de sortie>
# Il faut dpkg-deb, rpmbuild ; et, pour l'AppImage, APPIMAGETOOL et RUNTIME (le runtime type 2).
set -euo pipefail

ICI="$(cd "$(dirname "$0")" && pwd)"
BINAIRE="$(realpath "$1")"
SORTIE="$(realpath -m "$2")"
mkdir -p "$SORTIE"
VERSION="$("$BINAIRE" --version | cut -d' ' -f2)"
# Debian et RPM classent une préversion par le tilde : 2.0.1~beta.1 passe avant 2.0.1.
VERSION_PAQUET="${VERSION/-/\~}"
TRAVAIL="$(mktemp -d)"
trap 'rm -rf "$TRAVAIL"' EXIT

# Ce que les paquets de Tauri posaient, aux mêmes endroits.
poser() {
  install -Dm755 "$BINAIRE" "$1/usr/bin/glucose"
  install -Dm644 "$ICI/Glucose.desktop" "$1/usr/share/applications/Glucose.desktop"
  for taille in 32x32 128x128 256x256; do
    install -Dm644 "$ICI/icones/$taille.png" "$1/usr/share/icons/hicolor/$taille/apps/glucose.png"
  done
}

# --- Le .deb ---
RACINE="$TRAVAIL/deb"
poser "$RACINE"
mkdir -p "$RACINE/DEBIAN"
cat > "$RACINE/DEBIAN/control" <<FIN
Package: glucose
Version: $VERSION_PAQUET
Architecture: amd64
Maintainer: shazamifius
Installed-Size: $(du -sk "$RACINE/usr" | cut -f1)
Depends: libgtk-3-0
Recommends: libxkbcommon-x11-0, libvulkan1, mesa-vulkan-drivers
Section: graphics
Priority: optional
Homepage: https://github.com/shazamifius/GlucoseGit
Description: Glucose, la toile infinie des idées et des images
FIN
# xz : lu par tous les dpkg ; zstd ne l'est pas avant Debian 12.
dpkg-deb -Zxz --root-owner-group --build "$RACINE" "$SORTIE/Glucose_${VERSION}_amd64.deb"

# --- Le .rpm ---
RPM="$TRAVAIL/rpm"
mkdir -p "$RPM/SPECS"
poser "$TRAVAIL/racine"
cat > "$RPM/SPECS/glucose.spec" <<FIN
Name: glucose
Version: $VERSION_PAQUET
Release: 1
Summary: Glucose, la toile infinie des idées et des images
License: MIT
URL: https://github.com/shazamifius/GlucoseGit
Recommends: libxkbcommon-x11.so.0()(64bit), libvulkan.so.1()(64bit), mesa-vulkan-drivers

%description
Glucose, la toile infinie des idées et des images.

%install
cp -a "$TRAVAIL/racine/." %{buildroot}/

%files
/usr/bin/glucose
/usr/share/applications/Glucose.desktop
/usr/share/icons/hicolor/32x32/apps/glucose.png
/usr/share/icons/hicolor/128x128/apps/glucose.png
/usr/share/icons/hicolor/256x256/apps/glucose.png
FIN
# Les bibliothèques liées (GTK 3) sont exigées par rpmbuild lui-même, lues dans le binaire, sous
# leur nom de bibliothèque : le même sur Fedora et openSUSE. Le binaire reste tel quel (ni
# dépouillé, ni débogage à part) ; gzip, lu par tous les rpm.
rpmbuild -bb \
  --define "_topdir $RPM" \
  --define "_binary_payload w9.gzdio" \
  --define "__strip /bin/true" \
  --define "debug_package %{nil}" \
  "$RPM/SPECS/glucose.spec"
cp "$RPM"/RPMS/x86_64/glucose-*.rpm "$SORTIE/Glucose-${VERSION}-1.x86_64.rpm"

# --- L'AppImage ---
APPDIR="$TRAVAIL/AppDir"
poser "$APPDIR"
cp "$ICI/Glucose.desktop" "$APPDIR/glucose.desktop"
cp "$ICI/icones/256x256.png" "$APPDIR/glucose.png"
ln -s usr/bin/glucose "$APPDIR/AppRun"
ARCH=x86_64 "$APPIMAGETOOL" --runtime-file "$RUNTIME" --no-appstream \
  "$APPDIR" "$SORTIE/Glucose_${VERSION}_amd64.AppImage"

ls -l "$SORTIE"
