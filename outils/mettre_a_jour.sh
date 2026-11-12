#!/usr/bin/env bash
# **Mettre à jour son Glucose installé, sous un chien de garde** (fiche 110). `nix profile upgrade`
# compile dans le service de Nix, hors de toute borne (`cores = 0` : une compilation par cœur ; on
# n'est pas « utilisateur de confiance » pour le changer), et le 12/11/2026 une compilation sans borne
# a figé sa machine deux heures et demie. Toutes les deux secondes, la mémoire libre et la pression
# sont lues : sous 0,8 Go libres, ou au-delà de 40 % de pression « full » sur dix secondes, la mise à
# jour est coupée — Nix abandonne la compilation, son Glucose installé reste tel quel (code 3).
#
#   bash outils/mettre_a_jour.sh [journal.txt]
set -u
journal=${1:-$HOME/.cache/glucose-mise-a-jour.txt}
nice -n 10 nix profile upgrade --refresh GlucoseGit > "$journal" 2>&1 &
maj=$!
while kill -0 $maj 2>/dev/null; do
  libre=$(awk '/MemAvailable/ {print int($2/1024)}' /proc/meminfo)
  pression=$(awk '/^full/ {split($2,a,"="); print int(a[2])}' /proc/pressure/memory)
  if [ "$libre" -lt 800 ] || [ "$pression" -gt 40 ]; then
    echo "CHIEN DE GARDE : libre ${libre} Mo, pression ${pression} % — mise à jour coupée" >> "$journal"
    kill $maj; sleep 2; kill -9 $maj 2>/dev/null
    exit 3
  fi
  sleep 2
done
wait $maj; code=$?
echo "code $code" >> "$journal"
exit $code
