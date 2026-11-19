#!/usr/bin/env bash
# **Construire les polices d'émojis de Glucose** (EMOJI-2, fiche 125) : Twemoji 17 en COLRv0, et le
# reste de Noto Emoji. Reproductible : nanoemoji, fontTools et picosvg sont ceux de nixpkgs ; la
# source est épinglée par son empreinte (construire.py).
#
#   bash outils/emojis/construire.sh <dossier de travail> <dossier de sortie>
#
# Le dossier de travail garde l'archive téléchargée et la construction (environ 150 Mo) ; la sortie
# reçoit GlucoseEmoji-Couleur.ttf, NotoEmoji-Reste.ttf et la licence des dessins.
set -eu
[ $# -eq 2 ] || { echo "usage : construire.sh <travail> <sortie>" >&2; exit 2; }
ici=$(cd "$(dirname "$0")" && pwd)
mkdir -p "$1" "$2"
travail=$(cd "$1" && pwd); sortie=$(cd "$2" && pwd)
# nanoemoji apporte ninja avec lui.
outils='(import (builtins.getFlake "nixpkgs") {}).python3.withPackages (p: [ p.nanoemoji p.fonttools p.picosvg ])'
exec nix shell --impure --expr "$outils" -c \
    python3 -I "$ici/construire.py" "$travail" "$ici/NotoEmoji-Regular.ttf" \
    "$sortie/GlucoseEmoji-Couleur.ttf" "$sortie/NotoEmoji-Reste.ttf"
