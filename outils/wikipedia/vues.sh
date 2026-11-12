#!/usr/bin/env bash
# **Les quatre vues de la carte de Wikipédia** (fiches 104 à 110), dans le vrai Glucose hors écran,
# par la carte graphique — la voie qu'il voit : l'ensemble, « Eau », le fond d'« Eau », puis
# « Philosophie » après trois `Alt+Z`. Rien ne s'ouvre sur son écran.
#
#   bash outils/wikipedia/vues.sh <document.glucose> <dossier> [VAR=valeur…]
#
# Le document est **copié** dans une maison jetable (`<dossier>/maison`) : l'original ne s'ouvre
# jamais, ni ne reçoit un geste. `GLUCOSE_EXE` nomme le programme (par défaut, la version de release
# du dépôt) ; les `VAR=valeur` vont au Glucose piloté. Les images : `<dossier>/sortie/*.png`, et
# `sortie.txt` ce que Glucose a dit. Les places de la souris sont celles de l'écran du pilote
# (1920 × 1080) : la carte entière tient la vue au lancement, « Eau » est sous (1200, 600).
set -u
[ $# -ge 2 ] || { echo "usage : vues.sh <document.glucose> <dossier> [VAR=valeur…]" >&2; exit 2; }
document=$(realpath "$1"); B=$(realpath -m "$2"); shift 2
racine=$(cd "$(dirname "$0")/../.." && pwd)
export GLUCOSE_EXE=${GLUCOSE_EXE:-$racine/target/release/glucose-desktop}
P="bash $racine/outils/pilote/piloter.sh"
rm -rf "$B"; mkdir -p "$B/maison/state/glucose/brouillons" "$B/sortie"
cp "$document" "$B/maison/state/glucose/brouillons/carte.glucose"
$P demarrer "$B/maison" "$B/sortie" "$@" > "$B/demarrer.txt" 2>&1
# Glucose est prêt quand il a parlé ; puis la parenté des textes et les textures se font.
for _ in $(seq 60); do sleep 1; grep -q "Glucose\|image\|carte" "$B/sortie/sortie.txt" 2>/dev/null && break; done
sleep 14
$P aller 3 540; sleep 1
$P capture "$B/sortie/1-ensemble.png"
$P aller 1200 600; sleep 0.3; $P molette -4 80; sleep 3; $P aller 3 540; sleep 4
$P capture "$B/sortie/2-eau.png"
$P aller 1200 600; sleep 0.3; $P molette -3 80; sleep 3; $P aller 3 540; sleep 4
$P capture "$B/sortie/3-eau-profond.png"
$P alt z; sleep 1; $P alt z; sleep 1; $P alt z; sleep 2
$P aller 1200 600; sleep 0.3; $P molette -3 80; sleep 3; $P aller 3 540; sleep 4
$P capture "$B/sortie/4-philosophie.png"
$P arreter
ls "$B/sortie"/*.png
