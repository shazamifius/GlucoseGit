#!/bin/bash
# Une forme de Glucose sous Linux, éprouvée de bout en bout (fiche 48) — appelée par
# eprouver.sh : dans un conteneur pour le .deb et le .rpm, sur la machine pour l'AppImage.
#
# 1. Glucose Tauri s'installe ; on plante des données : celles de Glucose Rust, une image du
#    magasin de Tauri, un document.
# 2. La bascule : le geste exact de l'updater de Tauri — `dpkg -i`, `rpm -U`, ou l'AppImage
#    réécrit par-dessus lui-même.
# 3. La mise à jour de Glucose Rust par un latest.json signé : un paquet altéré est refusé, pour
#    la bonne raison — sa signature ; l'intact est posé.
# Les données plantées intactes à chaque pas.
#
# Usage : une_forme.sh <deb|rpm|appimage> <Tauri> <paquet N> <paquet N+1 servi> <N> <N+1>
set -euo pipefail
if [ "${GITHUB_ACTIONS:-}" != true ] || [ "${RUNNER_ENVIRONMENT:-}" != github-hosted ]; then
  echo "refus : cette epreuve installe Glucose ; elle ne se lance que sur une machine jetable de GitHub"
  exit 2
fi
FORME="$1"; TAURI="$2"; PAQUET_N="$3"; SERVI_N1="$4"; VN="$5"; VN1="$6"

constater() {
  local message="$1"; shift
  if "$@"; then echo "  ok : $message"; else echo "::error::$FORME : $message"; exit 1; fi
}
egal() {
  [ "$1" = "$2" ] || { echo "    attendu « $2 », lu « $1 »"; return 1; }
}
contient() {
  grep -q "$2" <<< "$1" || { echo "    « $2 » absent de « $1 »"; return 1; }
}

case "$FORME" in
  deb)
    export DEBIAN_FRONTEND=noninteractive
    apt-get update -qq
    apt-get install -y -qq "$TAURI" > /dev/null
    GLUCOSE=/usr/bin/glucose
    version_du_paquet() { dpkg-query -W -f='${Version}' glucose; }
    basculer() { dpkg -i "$PAQUET_N" > /dev/null; }
    ;;
  rpm)
    dnf install -y -q "$TAURI" > /dev/null
    GLUCOSE=/usr/bin/glucose
    version_du_paquet() { rpm -q --qf '%{VERSION}' glucose; }
    basculer() { rpm -U "$PAQUET_N"; }
    ;;
  appimage)
    mkdir -p "$HOME/Applications"
    GLUCOSE="$HOME/Applications/Glucose.AppImage"
    install -m755 "$TAURI" "$GLUCOSE"
    version_du_paquet() { stat -c '%s octets' "$GLUCOSE"; }
    # L'updater de Tauri réécrit le fichier par-dessus lui-même, ses droits gardés.
    basculer() { cat "$PAQUET_N" > "$GLUCOSE"; }
    ;;
  *) echo "forme inconnue : $FORME"; exit 2 ;;
esac
echo "  (Glucose Tauri installé : $(version_du_paquet))"

DONNEES="${XDG_STATE_HOME:-$HOME/.local/state}/glucose"
MAGASIN="${XDG_DATA_HOME:-$HOME/.local/share}/com.glucose.app"
PLANTES=(
  "$DONNEES/brouillons/brouillon-essai.glucose"
  "$DONNEES/brouillons/recuperation/essai-0-1.fin"
  "$DONNEES/boite-noire/session-essai.jsonl"
  "$DONNEES/apercus/v2/essai.apercu"
  "$DONNEES/dernier-document.txt"
  "$MAGASIN/assets/essai.png"
  "$HOME/Documents/essai-tauri.glucose"
)
for f in "${PLANTES[@]}"; do
  mkdir -p "$(dirname "$f")"
  head -c 65536 /dev/urandom > "$f"
done
EMPREINTES="$(mktemp)"
sha256sum "${PLANTES[@]}" > "$EMPREINTES"
intact() {
  constater "$1 : les ${#PLANTES[@]} fichiers plantés sont intacts" sha256sum --quiet -c "$EMPREINTES"
}

echo "  -- la bascule, par le geste de l'updater de Tauri"
constater "Glucose Rust se pose par-dessus Glucose Tauri" basculer
constater "il dit sa version" egal "$("$GLUCOSE" --version)" "Glucose $VN"
if [ "$FORME" != appimage ]; then
  constater "le système le connaît" egal "$(version_du_paquet)" "${VN/-/\~}"
  constater "son fichier de bureau est le nôtre" \
    grep -q '^StartupWMClass=glucose$' /usr/share/applications/Glucose.desktop
fi
intact "après la bascule"

echo "  -- la mise à jour, par un latest.json signé"
BON="$(mktemp)"
cp "$SERVI_N1" "$BON"
printf 'x' >> "$SERVI_N1"
SORTIE="$("$GLUCOSE" --mettre-a-jour)" && CODE=0 || CODE=$?
constater "un paquet altéré est refusé : $SORTIE" [ "$CODE" -ne 0 ]
constater "pour sa signature" contient "$SORTIE" "signature"
constater "rien ne change" egal "$("$GLUCOSE" --version)" "Glucose $VN"
cp "$BON" "$SERVI_N1"
SORTIE="$("$GLUCOSE" --mettre-a-jour)" && CODE=0 || CODE=$?
constater "le paquet signé est accepté : $SORTIE" [ "$CODE" -eq 0 ]
constater "Glucose est passé à $VN1" egal "$("$GLUCOSE" --version)" "Glucose $VN1"
if [ "$FORME" = appimage ]; then
  constater "l'AppImage reste exécutable" egal "$(stat -c %a "$GLUCOSE")" 755
else
  constater "le système le connaît" egal "$(version_du_paquet)" "${VN1/-/\~}"
fi
intact "après la mise à jour"
