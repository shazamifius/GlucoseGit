#!/usr/bin/env bash
# **Un second Glucose sur cette machine, pour essayer le canevas partagé seul** (fiche 117) : « Bob »,
# à côté du sien, tout entier dans un dossier jetable — son dossier Glucose (`~/.local/state/glucose`)
# n'est jamais touché, ni sa chronique (`TMPDIR` à part). Sans télémétrie, sans mise à jour. La porte
# des agents reste à son Glucose à lui : le second s'en passe.
#
#   bash outils/porte/essai/un_second_glucose.sh      (sa sortie : ~/sortie-glucose-bob.txt)
#
# Dans son Glucose : Collaborer → « Partager ce canevas » (le code est copié). Dans le second :
# Collaborer → « Rejoindre (le code copié) ». `GLUCOSE_EXE=<programme>` pour un autre que `glucose`.
set -u
M=/tmp/glucose-second
mkdir -p "$M/state/glucose" "$M/data" "$M/config" "$M/cache" "$M/tmp"
accord="$M/state/glucose/telemetrie.txt"
[ -e "$accord" ] || printf 'envoyer=non\nvue=oui\ninstallation=%032x\n' 4 > "$accord"
exec env XDG_STATE_HOME="$M/state" XDG_DATA_HOME="$M/data" XDG_CONFIG_HOME="$M/config" \
  XDG_CACHE_HOME="$M/cache" TMPDIR="$M/tmp" GLUCOSE_MISE_A_JOUR=non \
  "${GLUCOSE_EXE:-glucose}" > "$HOME/sortie-glucose-bob.txt" 2>&1
