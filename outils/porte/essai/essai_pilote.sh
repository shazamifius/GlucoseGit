#!/usr/bin/env bash
# **L'essai réel de la porte publique** (fiche 116) : le vrai Glucose hors écran (le pilote), le relais
# (local : `npx wrangler dev` dans outils/porte ; ou en ligne), un faux claude.ai qui fait tout le chemin
# — la découverte, la page, le code, la clé, les outils —, chaque étape capturée.
#   bash outils/porte/essai/essai_pilote.sh <dossier> <racine du relais>
# Il faut `target/release/glucose-desktop` (cargo build --release -p glucose-desktop).
set -u
D=$1; RELAIS=$2
P=$(cd "$(dirname "$0")/../../pilote" && pwd)/piloter.sh
ICI=$(cd "$(dirname "$0")" && pwd)
rm -rf "$D"; mkdir -p "$D/maison" "$D/sortie"
pil() { bash "$P" "$@"; }
pil demarrer "$D/maison" "$D/sortie" GLUCOSE_RELAIS="$RELAIS"
sleep 15
pil capture "$D/0-lance.png"
# Collaborer, par la palette : la question de la porte fermée, puis « Ouvrir la porte ».
pil ctrlmaj p; sleep 1
pil taper collaborer; sleep 1
pil capture "$D/1-palette.png"
pil touche Return; sleep 1.5
pil capture "$D/2-porte-fermee.png"
pil touche Return; sleep 3
pil capture "$D/3-porte-ouverte.png"
pil touche Escape; sleep 1
F="$D/maison/state/glucose/porte-publique.json"
ID=$(python3 -c "import json,base64,hashlib,sys; s=json.load(open('$F'))['secret']; s+='='*(-len(s)%4); print(hashlib.sha256(base64.urlsafe_b64decode(s)).hexdigest()[:32])")
echo "identifiant : $ID"
node "$ICI/faux_claude.mjs" "$RELAIS/g/$ID/mcp" "$D/code.txt" > "$D/claude.txt" 2>&1 &
CLAUDE=$!
for i in $(seq 1 40); do [ -s "$D/code.txt" ] && break; sleep 0.5; done
echo "code de la page : $(cat "$D/code.txt")"
sleep 2
pil capture "$D/4-question-d-accord.png"
pil touche Return; sleep 4
wait $CLAUDE
pil capture "$D/5-claude-a-ecrit.png"
pil alt f; sleep 2
pil capture "$D/6-tout-cadre.png"
pil ctrlmaj p; sleep 1; pil taper collaborer; sleep 1; pil touche Return; sleep 1.5
pil capture "$D/7-porte-avec-claude.png"
pil touche Escape; sleep 1
pil ctrl h; sleep 2
pil capture "$D/8-time-machine.png"
pil arreter
echo "fini"
