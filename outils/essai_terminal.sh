#!/usr/bin/env bash
# **L'essai réel des commandes du terminal** (fiche 90) : le vrai Glucose sans fenêtre
# (`porte_sans_fenetre`, qui répond « Autoriser » à la question d'accord), et le vrai programme
# `glucose` qui lui parle par sa porte, comme l'humain dans son terminal. Rien ne s'ouvre sur son
# écran ; son dossier Glucose n'est jamais touché.
#
#   nix develop -c cargo build -p glucose-desktop --bin glucose-desktop --example porte_sans_fenetre
#   nix develop -c bash outils/essai_terminal.sh <dossier jetable>
#
# Le dossier est vidé puis rempli. Code 0 : tout passe ; chaque vérification se lit sur la sortie.
set -u
racine=$(cd "$(dirname "$0")/.." && pwd)
T=${1:?usage : essai_terminal.sh <dossier jetable>}
rm -rf "$T"
mkdir -p "$T/glucose" "$T/images" "$T/tmp"
T=$(cd "$T" && pwd)
: > "$T/images/main.jsonl"
glucose="$racine/target/debug/glucose-desktop"
banc="$racine/target/debug/examples/porte_sans_fenetre"
# Une prise Unix tient en 108 caractères : la sienne reste courte.
prise=$(mktemp -u /tmp/glucose-terminal.XXXXXX)
GLUCOSE_DOSSIER_ESSAI="$T/glucose" TMPDIR="$T/tmp" "$banc" "$prise" "$T/arret" "$T/images" \
  > "$T/banc.txt" 2>&1 &
banc_pid=$!
for _ in $(seq 600); do [ -S "$prise" ] && break; sleep 0.05; done
export GLUCOSE_PORTE="$prise"

fautes=0
ok() {
  if [ "$1" = 0 ]; then echo "  OK   $2"; else echo "  FAUX $2"; fautes=$((fautes + 1)); fi
}
# La commande, sa sortie et sa sortie d'erreur dans des fichiers ; son code dans $code.
lancer() {
  "$glucose" "$@" > "$T/out.txt" 2> "$T/err.txt"
  code=$?
}
json() { python3 -I -c "import json,sys; d=json.load(open('$T/out.txt')); print($1)"; }

echo "=== 1. L'aide, tirée de la table des outils"
lancer aide
ok $(( code != 0 )) "glucose aide : code $code"
grep -q '^  chercher$' "$T/out.txt" && grep -q '^  poser-texte$' "$T/out.txt"
ok $? "chaque commande y est, chercher et poser-texte compris"
lancer aide poser-texte
grep -q "texte : un texte, exigé" "$T/out.txt"
ok $? "glucose aide poser-texte dit l'argument exigé"

echo "=== 2. Poser, sous ses yeux"
lancer poser-texte id=eau "L'eau, ça mouille."
ok $(( code != 0 )) "glucose poser-texte : code $code, $(cat "$T/out.txt")"
[ "$(json "d['id']")" = eau ]
ok $? "il rend l'identifiant du nœud posé"
printf '# Vapeur\nLa vapeur monte.' | "$glucose" poser-texte id=vapeur - > "$T/out.txt" 2> "$T/err.txt"
ok $? "un texte lu de l'entrée standard (« - »)"

echo "=== 3. Chercher, comme Ctrl+F"
lancer chercher VAPEUR
[ "$(json "d['total']")" = 1 ] && [ "$(json "d['resultats'][0]['id']")" = vapeur ]
ok $? "glucose chercher VAPEUR : $(cat "$T/out.txt")"
lancer chercher eau
[ "$(wc -l < "$T/out.txt")" = 1 ]
ok $? "dans un tuyau, une seule ligne (pour jq)"
script -qec "$glucose chercher eau" /dev/null > "$T/tty.txt" 2>&1
[ "$(wc -l < "$T/tty.txt")" -gt 5 ]
ok $? "dans un terminal, en retrait ($(wc -l < "$T/tty.txt") lignes)"
lancer lire niveau=complet
grep -q 'La vapeur monte.' "$T/out.txt"
ok $? "glucose lire niveau=complet rend le texte entier"

echo "=== 4. Retirer, puis ne plus trouver"
lancer retirer vapeur
ok $(( code != 0 )) "glucose retirer vapeur : $(cat "$T/out.txt")"
lancer chercher vapeur
[ "$(json "d['total']")" = 0 ]
ok $? "plus rien à trouver"

echo "=== 5. Les échecs se disent, par leur code"
lancer lire ids_=a
[ "$code" = 2 ] && grep -q niveau "$T/err.txt"
ok $? "une clé inconnue : code $code, $(cat "$T/err.txt")"
lancer lir
[ "$code" = 2 ]
ok $? "une faute de frappe n'ouvre rien : code $code, $(cat "$T/err.txt")"
lancer deplacer id=personne x=0 y=0
[ "$code" = 1 ] && [ -s "$T/err.txt" ]
ok $? "un refus de Glucose : code $code, $(cat "$T/err.txt")"
lancer "$T/nulle-part.glucose"
[ "$code" = 2 ] && grep -q introuvable "$T/err.txt"
ok $? "un document introuvable : code $code, aucune fenêtre"
GLUCOSE_PORTE=$(mktemp -u /tmp/glucose-absente.XXXXXX) "$glucose" lire > "$T/out.txt" 2> "$T/err.txt"
code=$?
[ "$code" = 1 ] && grep -q 'ne répond pas' "$T/err.txt" && ! grep -q 'SUN_LEN' "$T/err.txt"
ok $? "une porte qui ne répond pas : code $code, $(cat "$T/err.txt")"

echo "=== 5 bis. La Time Machine : un geste signé, un écrit attaché (MAIN-1, NOTE-1, fiche 108)"
lancer poser-texte id=pluie "La pluie est de l'eau qui tombe."
lancer noter "texte=Ce texte relie la pluie à l'eau : c'est pourquoi il compte."
ok $(( code != 0 )) "glucose noter : code $code, $(cat "$T/out.txt") $(cat "$T/err.txt")"
lancer histoire nombre=3
dernier="d['gestes'][-1]"
[ "$(json "$dernier['auteur']")" = "agent le terminal" ] \
  && json "$dernier['fait']" | grep -q 'pose le texte « pluie »' \
  && json "$dernier['ecrits'][0]['texte']" | grep -q "relie la pluie"
ok $? "glucose histoire : le dernier geste, son auteur, ce qu'il a fait, son écrit — $(cat "$T/out.txt" | head -c 400)"

echo "=== 6. Ouvrir un document par son nom (OUVRIR-1), dans un écran invisible"
doc="$T/projets/demande.glucose"
mkdir -p "$T/projets"
lancer poser-texte id=demande "Le document demandé, ouvert par son nom."
printf '{"enregistrer":"%s"}\n' "$doc" >> "$T/images/main.jsonl"
sleep 1
# Le banc finit proprement : tout est écrit, et il ne tient plus le document.
: > "$T/arret"
wait "$banc_pid"
rm -f "$prise"
[ -s "$doc" ]
ok $? "le document est enregistré : $doc"
printf '4 quitter\n' > "$T/ouvrir.txt"
enveloppe="$T/glucose-sur-le-document"
printf '#!/bin/sh\nexec "%s" "%s"\n' "$glucose" "$doc" > "$enveloppe"
chmod +x "$enveloppe"
GLUCOSE_EXE="$enveloppe" GLUCOSE_BANC_MAISON="$T/maison" GLUCOSE_CAPTURE_A=3 \
  bash "$racine/outils/banc_hors_ecran.sh" "$T/ouvrir.txt" "$T/ouvrir"
grep -q 'CODE 0' "$T/ouvrir/sortie.txt"
ok $? "Glucose s'est ouvert sur le document, puis fermé : $(tail -1 "$T/ouvrir/sortie.txt")"
grep -rqs 'demande.glucose' "$T/maison/state/glucose" "$T/ouvrir"
ok $? "il retient le document demandé comme le dernier ouvert"
echo "  la capture, à regarder : $T/ouvrir/capture.png"
echo
if [ "$fautes" = 0 ]; then echo "TOUT PASSE"; else echo "$fautes FAUTE(S)"; fi
exit "$fautes"
