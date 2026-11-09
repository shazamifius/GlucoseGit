#!/usr/bin/env bash
# **Le squelette de Wikipédia dans le vrai Glucose, sans fenêtre** (fiches 95 et 96) : les cent
# articles vitaux du niveau 2, « Eau » et « Hydrogène » entiers, tout rangé en colonnes ; puis le
# tableau entier et chacune de ses membranes en images (2 560 × 1 600). Rien ne s'ouvre à l'écran.
#
#   nix develop -c bash outils/wikipedia/essai_squelette.sh <dossier de sortie>
#
# Le dossier de sortie reçoit : images/ (les PNG), wikipedia/ (les réponses de Wikipédia, gardées :
# une seconde pose ne redemande rien), squelette.txt (le compte rendu), plan.json (ce qu'un agent
# lit), glucose/ (le dossier où Glucose a écrit le document).
set -euo pipefail
sortie=$(realpath -m "$1")
mkdir -p "$sortie/images" "$sortie/wikipedia"
racine=$(cd "$(dirname "$0")/../.." && pwd)
cd "$racine"
cargo build -p glucose-desktop --bin glucose-desktop --example porte_sans_fenetre

# Un Glucose sans fenêtre, dans un dossier jetable ; une prise Unix tient en 108 caractères.
W=$(mktemp -d)
mkdir -p "$W/glucose" "$W/images"
: > "$W/images/main.jsonl"
prise=$(mktemp -u /tmp/glucose-wiki.XXXXXX)
# Les vues de près : les neuf thèmes, et les deux articles entiers (`GLUCOSE_VUES` les remplace).
vues=${GLUCOSE_VUES:-theme0,theme1,theme2,theme3,theme4,theme5,theme6,theme7,theme8,eau,hydrogene}
GLUCOSE_VUES="$vues" GLUCOSE_DOSSIER_ESSAI="$W/glucose" target/debug/examples/porte_sans_fenetre \
    "$prise" "$W/arret" "$W/images" > "$sortie/porte.txt" 2>&1 &
porte=$!
for _ in $(seq 200); do
    [ -S "$prise" ] && break
    sleep 0.1
done
[ -S "$prise" ] || { echo "la porte ne s'est pas ouverte : $sortie/porte.txt"; kill $porte; exit 1; }
export GLUCOSE_PORTE="$prise"

python3 -I outils/wikipedia/squelette.py target/debug/glucose-desktop "$sortie/wikipedia" \
    | tee "$sortie/squelette.txt"
# La proposition D, en prototype (fiche 98) : l'arbre des membranes en couleurs, posé par l'agent.
if [ -n "${GLUCOSE_ARBRE_EN_COULEURS:-}" ]; then
    python3 -I outils/wikipedia/teintes_arbre.py target/debug/glucose-desktop wiki \
        | tee -a "$sortie/squelette.txt"
fi
# Le sens des couleurs (fiche 99) : teinte = proximité des domaines, clarté = accessibilité.
if [ -n "${GLUCOSE_SENS_EN_COULEURS:-}" ]; then
    python3 -I outils/wikipedia/teintes_sens.py target/debug/glucose-desktop wiki \
        | tee -a "$sortie/squelette.txt"
fi
target/debug/glucose-desktop lire niveau=plan > "$sortie/plan.json"

# Glucose finit, et rend le tableau en images.
: > "$W/arret"
wait $porte
cp "$W"/images/*.png "$sortie/images/"
# Le document, tel que Glucose l'a écrit en finissant : de quoi l'ouvrir dans un essai à l'écran
# (outils/essai_isole.sh, son dossier `state/glucose`).
rm -rf "$sortie/glucose"
cp -r "$W/glucose" "$sortie/glucose"
echo "$(ls "$sortie/images" | wc -l) images dans $sortie/images ; le document dans $sortie/glucose"
rm -rf "$W"
