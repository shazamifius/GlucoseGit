#!/bin/bash
# La bascule et la mise à jour sous Linux, éprouvées sur une machine jetable de GitHub (fiche 48),
# sous les trois formes que ses utilisateurs de Glucose Tauri peuvent avoir : le .deb (Ubuntu
# 22.04, dans un conteneur), le .rpm (Fedora, dans un conteneur), l'AppImage (sur la machine).
# Chaque forme : une_forme.sh.
#
# NE SE LANCE QUE SUR UNE MACHINE JETABLE DE GITHUB : elle installe Glucose dans le système.
#
# Usage : eprouver.sh <dossier Tauri> <dossier N> <dossier servi> <N> <N+1>
#   <dossier Tauri> : tauri.deb, tauri.rpm, tauri.AppImage — la release publique 1.0.2-beta.1
#   <dossier N>     : les trois paquets de la version N
#   <dossier servi> : latest.json et les trois paquets N+1, signés
set -euo pipefail
if [ "${GITHUB_ACTIONS:-}" != true ] || [ "${RUNNER_ENVIRONMENT:-}" != github-hosted ]; then
  echo "refus : cette epreuve installe Glucose ; elle ne se lance que sur une machine jetable de GitHub"
  exit 2
fi
ICI="$(cd "$(dirname "$0")" && pwd)"
TAURI="$(realpath "$1")"; N="$(realpath "$2")"; SERVI="$(realpath "$3")"; VN="$4"; VN1="$5"

python3 -m http.server 8765 --bind 127.0.0.1 --directory "$SERVI" > /dev/null 2>&1 &
SERVEUR=$!
trap 'kill $SERVEUR' EXIT
for _ in $(seq 1 100); do
  curl -sf -o /dev/null http://127.0.0.1:8765/latest.json && break
  sleep 0.2
done

conteneur() {
  local image="$1"; shift
  docker run --rm --network host -e GITHUB_ACTIONS -e RUNNER_ENVIRONMENT \
    -v "$ICI:/outils:ro" -v "$TAURI:/tauri:ro" -v "$N:/n:ro" -v "$SERVI:/servi" \
    "$image" bash /outils/une_forme.sh "$@"
}

echo "== Debian et Ubuntu : le .deb"
conteneur ubuntu:22.04 deb /tauri/tauri.deb "/n/Glucose_${VN}_amd64.deb" \
  "/servi/Glucose_${VN1}_amd64.deb" "$VN" "$VN1"

echo "== Fedora : le .rpm"
conteneur fedora:40 rpm /tauri/tauri.rpm "/n/Glucose-${VN}-1.x86_64.rpm" \
  "/servi/Glucose-${VN1}-1.x86_64.rpm" "$VN" "$VN1"

echo "== L'AppImage"
bash "$ICI/une_forme.sh" appimage "$TAURI/tauri.AppImage" "$N/Glucose_${VN}_amd64.AppImage" \
  "$SERVI/Glucose_${VN1}_amd64.AppImage" "$VN" "$VN1"

echo "== La bascule et la mise à jour sont éprouvées sous les trois formes."
