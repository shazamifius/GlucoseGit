#!/usr/bin/env bash
# **L'essai réel des photos d'un participant à l'autre** (B1, fiche 118) : deux vrais Glucose hors
# écran, Alice et Bob, contre la salle d'un relais (local : `npx wrangler dev --ip 127.0.0.1 --port
# 8787` dans `outils/porte` ; ou en ligne). Regarder CHAQUE image.
#
#   bash outils/porte/essai/essai_photos_pilote.sh <dossier> [racine du relais]
#
# 1. Alice colle une photo, PUIS partage : la photo est dans la base. Bob rejoint : son bonjour la
#    demande, Alice la lui renvoie — il la voit, et non un cadre gris.
# 2. Alice colle une seconde photo, Bob déjà là : son geste part avant que la photo soit encodée ;
#    le facteur d'Alice attend les octets et les envoie — Bob la voit sans rien demander.
set -u
D=$1
RELAIS=${2:-http://127.0.0.1:8787}
ICI=$(cd "$(dirname "$0")" && pwd)
P=$ICI/../../pilote/piloter.sh
rm -rf "$D"; mkdir -p "$D"
A() { PILOTE_ETAT=/tmp/glucose-pilote-alice bash "$P" "$@"; }
B() { PILOTE_ETAT=/tmp/glucose-pilote-bob bash "$P" "$@"; }
py() { nix shell --impure --expr "(import (builtins.getFlake \"nixpkgs\") {}).python3.withPackages (p: [p.$1])" -c python3 -I "${@:2}"; }
# Deux photos qu'on reconnaît : un damier rouge et bleu, des bandes vertes et jaunes.
py pillow -c "
from PIL import Image
a = Image.new('RGB', (400, 300))
a.putdata([(220, 30, 30) if (x // 50 + y // 50) % 2 else (30, 60, 220) for y in range(300) for x in range(400)])
a.save('$D/damier.png')
b = Image.new('RGB', (400, 300))
b.putdata([(30, 200, 60) if (y // 30) % 2 else (240, 220, 30) for y in range(300) for x in range(400)])
b.save('$D/bandes.png')
"
A demarrer "$D/alice" "$D/alice-sortie" GLUCOSE_RELAIS="$RELAIS"
B demarrer "$D/bob" "$D/bob-sortie" GLUCOSE_RELAIS="$RELAIS"
sleep 18
palette() { local qui=$1; shift; $qui ctrlmaj p; sleep 1; $qui taper "$1"; sleep 0.8; $qui touche Return; }

# 1. Une photo dans la base.
A presse-image "$D/damier.png"; A aller 700 600; A ctrl v; sleep 4
A capture "$D/1-alice-avant-de-partager.png"
palette A collaborer; sleep 1.5; A touche Return; sleep 3
A presse-lire > "$D/code.txt"; A touche Escape
echo "code : $(cut -c1-22 "$D/code.txt")…"
B presse-ecrire "$(cat "$D/code.txt")"
palette B collaborer; sleep 1.5; B touche Tab; sleep 0.3; B touche Return; sleep 12
B touche Escape; B alt f; sleep 4
B capture "$D/1-bob-voit-la-base.png"
# 2. Une photo posée pendant le partage.
A presse-image "$D/bandes.png"; A aller 1300 600; A ctrl v; sleep 8
A alt f; B alt f; sleep 4
A capture "$D/2-alice.png"; B capture "$D/2-bob.png"

A arreter; B arreter
echo "fini : regarder chaque image de $D"
