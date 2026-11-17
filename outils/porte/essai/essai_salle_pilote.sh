#!/usr/bin/env bash
# **L'essai réel du canevas partagé** (fiche 117) : deux vrais Glucose hors écran — Alice, et Bob
# derrière un réseau lent (`reseau_lent.py` : ce qui descend vers lui arrive 3 s plus tard) —,
# chacun dans son compositeur, contre la salle d'un relais (local : `npx wrangler dev --ip
# 127.0.0.1 --port 8787` dans `outils/porte` ; ou en ligne). Chaque étape est capturée ; regarder
# CHAQUE image.
#
#   bash outils/porte/essai/essai_salle_pilote.sh <dossier> [racine du relais]
#
# 1. Alice partage (Collaborer → Partager ce canevas) : le code est copié.
# 2. Bob colle le code et rejoint (Collaborer → Rejoindre) : il rattrape la base, par le réseau lent.
# 3. Alice pose un texte, Bob en pose un : chacun le voit chez l'autre.
# 4. Les curseurs : la souris d'Alice chez Bob, celle de Bob chez Alice, de leur couleur et nommées.
# 5. Le conflit : Alice colore « Alice » en rouge ; une seconde plus tard Bob, qui ne voit pas encore
#    le rouge, le colore en émeraude — le plus récent l'emporte chez les deux, et le message le dit.
# 6. Ctrl+Z : Alice pose « A2 », Bob « B2 » (le dernier geste) ; Alice défait deux fois — « A2 »
#    part chez les deux, « B2 » reste.
#
# Les coordonnées sont celles d'un écran de 1920 × 1080 et du document d'accueil.
set -u
D=$1
RELAIS=${2:-http://127.0.0.1:8787}
ICI=$(cd "$(dirname "$0")" && pwd)
P=$ICI/../../pilote/piloter.sh
LENT=8791
rm -rf "$D"; mkdir -p "$D"
A() { PILOTE_ETAT=/tmp/glucose-pilote-alice bash "$P" "$@"; }
B() { PILOTE_ETAT=/tmp/glucose-pilote-bob bash "$P" "$@"; }
py() { nix shell --impure --expr "(import (builtins.getFlake \"nixpkgs\") {}).python3.withPackages (p: [p.$1])" -c python3 -I "${@:2}"; }

py websockets "$ICI/reseau_lent.py" "$(echo "$RELAIS" | sed 's/^http/ws/')" $LENT 3 > "$D/reseau-lent.txt" 2>&1 &
RESEAU=$!
A demarrer "$D/alice" "$D/alice-sortie" GLUCOSE_RELAIS="$RELAIS"
B demarrer "$D/bob" "$D/bob-sortie" GLUCOSE_RELAIS="http://127.0.0.1:$LENT"
sleep 18
palette() { local qui=$1; shift; $qui ctrlmaj p; sleep 1; $qui taper "$1"; sleep 0.8; $qui touche Return; }
texte() { local qui=$1; $qui touche t; sleep 0.4; $qui clic "$2" "$3"; sleep 1; $qui taper "$4"; $qui touche Escape; sleep 0.4; $qui touche Escape; }

# 1. Alice partage.
palette A collaborer; sleep 1.5; A touche Return; sleep 3
A capture "$D/1-alice-partage.png"
A presse-lire > "$D/code.txt"; A touche Escape
echo "code : $(cut -c1-22 "$D/code.txt")…"
# 2. Bob rejoint.
B presse-ecrire "$(cat "$D/code.txt")"
palette B collaborer; sleep 1.5; B touche Tab; sleep 0.3; B touche Return; sleep 10
B touche Escape
# 3. Un geste chacun.
texte A 1790 250 Alice; sleep 5
texte B 130 700 Bob; sleep 5
A alt f; B alt f; sleep 5
A capture "$D/3-alice.png"; B capture "$D/3-bob.png"
# 4. Les curseurs.
A aller 900 760; sleep 5; B capture "$D/4-bob-voit-alice.png"
B aller 640 900; sleep 2; A capture "$D/4-alice-voit-bob.png"
# 5. Le conflit.
A clic 1414 480; B clic 1414 480; sleep 1
# Les deux palettes s'ouvrent ensemble ; Bob valide une seconde après Alice — bien avant que le
# rouge lui arrive par le réseau lent (3 s).
A ctrlmaj p; B ctrlmaj p; sleep 0.8; A taper rouge; B taper emeraude; sleep 0.5
A touche Return; sleep 1; B touche Return; sleep 0.6
A capture "$D/5-alice-juste-apres.png"
for i in 1 2 3 4; do sleep 0.9; B capture "$D/5-bob-$i.png"; done
sleep 4; A capture "$D/5-alice.png"; B capture "$D/5-bob.png"
# 6. Ctrl+Z.
A touche Escape; B touche Escape
texte A 1300 800 A2; sleep 5
texte B 500 960 B2; sleep 5
A capture "$D/6-alice-avant.png"; B capture "$D/6-bob-avant.png"
A ctrl z; sleep 0.5; A ctrl z; sleep 5
A capture "$D/6-alice.png"; B capture "$D/6-bob.png"

A arreter; B arreter
kill $RESEAU 2>/dev/null
echo "fini : regarder chaque image de $D"
