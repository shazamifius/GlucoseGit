#!/usr/bin/env bash
# **L'essai réel de REVENIR-2** (fiche 148) : deux vrais Glucose hors écran, Alice et Bob, contre la
# salle d'un relais (local : `npx wrangler dev --ip 127.0.0.1 --port 8787` dans `outils/porte` ; ou en
# ligne). Chaque étape est capturée ; regarder CHAQUE image, et les sorties (`revient`, `revenu`).
#
#   bash outils/porte/essai/essai_revenir_pilote.sh <dossier> [racine du relais]
#
# 1. Alice pose « Alice », partage ; Bob rejoint par le code : il voit « Alice ».
# 2. Alice quitte le partage. Bob pose « Bob » ; Alice, dehors, pose « ici ».
# 3. Alice partage de nouveau : elle **revient dans la même salle** (aucun code neuf) — « Bob » chez
#    elle, « ici » chez lui.
# 4. Alice s'arrête net (tuée) ; Bob pose « B3 » ; Alice relancée **revient seule** : « B3 » chez elle.
#
# Les coordonnées sont celles d'un écran de 1920 × 1080 et du document d'accueil.
set -u
D=$1
RELAIS=${2:-http://127.0.0.1:8787}
ICI=$(cd "$(dirname "$0")" && pwd)
P=$ICI/../../pilote/piloter.sh
rm -rf "$D"; mkdir -p "$D"
A() { PILOTE_ETAT=/tmp/glucose-pilote-alice bash "$P" "$@"; }
B() { PILOTE_ETAT=/tmp/glucose-pilote-bob bash "$P" "$@"; }
palette() { local qui=$1; shift; $qui ctrlmaj p; sleep 1; $qui taper "$1"; sleep 0.8; $qui touche Return; }
texte() { local qui=$1; $qui touche t; sleep 0.4; $qui clic "$2" "$3"; sleep 1; $qui taper "$4"; $qui touche Escape; sleep 0.4; $qui touche Escape; }
# Collaborer, puis le choix au rang donné (0 : le premier).
collaborer() { local qui=$1; palette "$qui" collaborer; sleep 1.5; for _ in $(seq 1 "$2"); do $qui touche Tab; sleep 0.3; done; $qui touche Return; }

A demarrer "$D/alice" "$D/alice-sortie" GLUCOSE_RELAIS="$RELAIS"
B demarrer "$D/bob" "$D/bob-sortie" GLUCOSE_RELAIS="$RELAIS"
sleep 18

# 1. Alice pose, partage ; Bob rejoint.
texte A 1790 250 Alice; sleep 2
# La première fois, son nom et sa couleur d'abord (IDENTITE-2) : « Garder ».
collaborer A 0; sleep 1.5; A touche Return; sleep 3
A capture "$D/1-alice-partage.png"
A presse-lire > "$D/code.txt"; A touche Escape
echo "code : $(cut -c1-22 "$D/code.txt")…"
B presse-ecrire "$(cat "$D/code.txt")"
collaborer B 1; sleep 1.5; B touche Return; sleep 2
B capture "$D/1-bob-code.png"; B touche Return; sleep 10
B touche Escape
B capture "$D/1-bob.png"

# 2. Alice quitte (Copier écrire, Copier lire, Quitter) ; chacun pose un texte.
collaborer A 2; sleep 2; A touche Escape
texte B 130 700 Bob; sleep 3
texte A 900 500 ici; sleep 3
A capture "$D/2-alice-dehors.png"; B capture "$D/2-bob.png"

# 3. Alice partage de nouveau : elle revient.
collaborer A 0; sleep 10
A capture "$D/3-alice-revenue.png"; A touche Escape
A presse-lire > "$D/code-2.txt"
sleep 3; B capture "$D/3-bob.png"

# 4. L'arrêt net : Alice tuée, Bob pose, Alice relancée revient.
A arreter
texte B 500 960 B3; sleep 3
A demarrer "$D/alice" "$D/alice-sortie-2" GLUCOSE_RELAIS="$RELAIS"
sleep 25
A capture "$D/4-alice-relancee.png"; B capture "$D/4-bob.png"

A arreter; B arreter
grep -h "canevas partagé" "$D"/alice-sortie* "$D/bob-sortie" 2>/dev/null
echo "fini : regarder chaque image de $D"
