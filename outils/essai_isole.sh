#!/usr/bin/env bash
# **Essayer le vrai Glucose sans rien toucher** : ni ses données, ni sa télémétrie.
#
#   bash outils/essai_isole.sh <secondes> [capture.png]
#
# Le dossier personnel est un dossier neuf, et la question du journal technique y a déjà reçu
# « non » : un clic distrait a envoyé une session d'essai vers sa télémétrie le 6/11 (fiche 84).
# La fenêtre s'ouvre sur son écran et prend le focus : garder l'essai court, et le dire.
# Variables utiles : GLUCOSE_EXE (le programme, par défaut target/release/glucose-desktop),
# GLUCOSE_TOMBER=vulkan (tomber à l'ouverture de ce moteur), GLUCOSE_PERF=1.
set -u
racine=$(cd "$(dirname "$0")/.." && pwd)
secondes=${1:?usage : essai_isole.sh <secondes> [capture.png]}
capture=${2:-}
exe=${GLUCOSE_EXE:-$racine/target/release/glucose-desktop}
ici=${GLUCOSE_ESSAI_DOSSIER:-$(mktemp -d)}
mkdir -p "$ici/state/glucose"
accord="$ici/state/glucose/telemetrie.txt"
[ -e "$accord" ] || printf 'envoyer=non\nvue=oui\ninstallation=%032x\n' 1 > "$accord"
sortie="$ici/sortie.txt"
nix develop "$racine" --no-write-lock-file -c env HOME="$ici" XDG_DATA_HOME="$ici/data" \
  XDG_STATE_HOME="$ici/state" XDG_CONFIG_HOME="$ici/config" XDG_CACHE_HOME="$ici/cache" \
  GLUCOSE_MISE_A_JOUR=non "$exe" > "$sortie" 2>&1 &
pere=$!
sleep "$secondes"
[ -n "$capture" ] && grim "$capture"
pkill -TERM -P "$pere" 2>/dev/null
pkill -TERM -f "^$exe" 2>/dev/null
wait "$pere"
echo "code=$? dossier=$ici sortie=$sortie"
