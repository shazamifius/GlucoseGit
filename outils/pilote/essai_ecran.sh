#!/usr/bin/env bash
# **Le vrai Glucose sur SON écran, ses vrais gestes** (fiche 112) : une fenêtre, sa souris pilotée
# par le pointeur virtuel de niri, son pavé par un pavé virtuel (uinput, `pave_virtuel.py`) — le
# pincement et les deux doigts qu'il fait vraiment.
#
#   bash outils/pilote/essai_ecran.sh <dossier> [VAR=valeur…]
#
# ANNONCER AVANT : une fenêtre s'ouvre sur son écran, sa souris bouge seule environ 75 s ; dire le
# nombre EXACT de fenêtres d'une série. Rien de lui n'est touché : un dossier personnel neuf (la
# télémétrie coupée d'avance), le document copié.
#
#   GLUCOSE_EXE  le programme (défaut : target/release/glucose-desktop)
#   DOCUMENT     le document copié comme brouillon (défaut : sa carte de Wikipédia avec ses liens)
#   LOURD        des ordres de pave_virtuel.py joués avant les gestes (un zoom), séparés par \n
#
# Sortie dans <dossier> : sortie.txt (GLUCOSE_PERF=1), segments.txt (les lignes de chaque geste),
# chronique.txt (sa section « Ce que l'ecran a montre », PARU-1), vue.png.
#
# Les garde-fous, payés le 12/11 à 20 h 58 (fiche 112 § 3) : sous niri 26.04, une fenêtre neuve
# ne prend pas le focus et s'ouvre en colonne hors de la vue — et niri ne dit pas où est une
# colonne. Des gestes sont tombés sur VS Code. Donc Glucose est mis au premier plan, flottant, à
# une place connue ; chaque geste vérifie d'abord qu'il a le focus, sinon tout s'arrête ; et le
# scénario le ferme de lui-même à 75 s.
set -u
B=${1:?usage : essai_ecran.sh <dossier> [VAR=valeur…]}; shift
ici=$(cd "$(dirname "$0")" && pwd)
racine=$(cd "$ici/../.." && pwd)
exe=${GLUCOSE_EXE:-$racine/target/release/glucose-desktop}
document=${DOCUMENT:-$HOME/Documents/Glucose-essai-wikipedia-liens.glucose}
PY=$(nix build --impure --print-out-paths --no-link --expr \
  '(import (builtins.getFlake "nixpkgs") {}).python3.withPackages (p: [p.pywayland p.evdev])' 2>/dev/null)/bin/python3
rm -rf "$B"; mkdir -p "$B/maison/state/glucose/brouillons"
cp "$document" "$B/maison/state/glucose/brouillons/document.glucose"
printf 'envoyer=non\nvue=oui\ninstallation=%032x\n' 1 > "$B/maison/state/glucose/telemetrie.txt"
printf '3 tout-cadrer\n75 quitter\n' > "$B/scenario.txt"
S=$B/sortie.txt
(cd "$racine" && nix develop "$racine" --no-write-lock-file -c env -u DISPLAY \
  HOME="$B/maison" XDG_DATA_HOME="$B/maison/data" XDG_STATE_HOME="$B/maison/state" \
  XDG_CONFIG_HOME="$B/maison/config" XDG_CACHE_HOME="$B/maison/cache" GLUCOSE_MISE_A_JOUR=non \
  GLUCOSE_SCENARIO="$B/scenario.txt" GLUCOSE_PERF=1 "$@" "$exe" > "$S" 2>&1
  echo "CODE $?" >> "$S") &
for _ in $(seq 90); do sleep 1; grep -q "frame #" "$S" 2>/dev/null && break; done
sleep 8
# Ce Glucose-ci, par son numéro : `pkill -x glucose-desktop` tuerait aussi le sien.
pid=$(pgrep -f "^$exe( |\$)" | head -1)
fermer() { [ -n "$pid" ] && kill -TERM "$pid" 2>/dev/null; }
arreter() { echo "ARRET : $1" | tee -a "$B/fenetre.txt"; [ -p "$B/pointeur" ] && printf 'fin\n' > "$B/pointeur"; fermer; exit 1; }
[ -n "$pid" ] || arreter "ce Glucose ne s'est pas lancé"
id=$(niri msg -j windows | "$PY" -c "
import json,sys
print(next((f['id'] for f in json.load(sys.stdin) if f.get('pid')==$pid), ''))")
[ -n "$id" ] || arreter "aucune fenetre a ce Glucose"
niri msg action focus-window --id "$id"; sleep 0.5
niri msg action toggle-window-floating --id "$id"; sleep 0.8
niri msg action move-floating-window --id "$id" -x 492 -y 42; sleep 0.8
lieu() { niri msg -j focused-window | "$PY" -c "
import json,sys
f=json.load(sys.stdin) or {}
l=f.get('layout') or {}; p=l.get('tile_pos_in_workspace_view'); o=l.get('window_offset_in_tile') or [0,0]; t=l.get('window_size') or [0,0]
print(f.get('pid'), int(p[0]+o[0]) if p else -1, int(p[1]+o[1]) if p else -1, t[0], t[1])"; }
read -r app x y w h < <(lieu)
echo "fenetre $app $x $y $w $h" > "$B/fenetre.txt"
garde() {
  read -r a gx gy _ _ < <(lieu)
  { [ "$a" = "$pid" ] && [ "$gx" = "$x" ] && [ "$gy" = "$y" ] && [ "$x" -ge 0 ]; } ||
    arreter "le focus n'est plus sur Glucose ($a $gx $gy)"
}
garde
cx=$((x + w / 2)); cy=$((y + h / 2))
tuyau=$B/pointeur; rm -f "$tuyau"
PILOTE_ECRAN=1920x1080 "$PY" "$ici/pointeur.py" "$tuyau" > "$B/pointeur.txt" 2>&1 &
for _ in $(seq 50); do [ -p "$tuyau" ] && break; sleep 0.1; done
o() { printf "$@" > "$tuyau"; }
n() { wc -l < "$S"; }
pave() { garde; printf "$1" > "$B/$2.txt"; "$PY" "$ici/pave_virtuel.py" "$B/$2.txt" > "$B/pave-$2.txt" 2>&1; }
G=$B/segments.txt
o 'aller %s %s\n' "$cx" "$cy"; sleep 1
[ -n "${LOURD:-}" ] && { pave "$LOURD" zoom; sleep 2; }
grim -g "$x,$y ${w}x${h}" "$B/vue.png" 2>/dev/null
garde; echo "souris $(n)" > "$G"; o 'glisser %s %s %s %s 3000 125\n' $((x + 60)) $((y + 80)) $((x + w - 60)) $((y + h - 80)); sleep 3.5; echo "fin $(n)" >> "$G"
garde; o 'aller %s %s\n' "$cx" "$cy"; o 'milieu-presser\n'; sleep 0.2
echo "deplacer $(n)" >> "$G"; o 'glisser %s %s %s %s 2500 125\n' "$cx" "$cy" $((cx - 300)) $((cy - 200)); sleep 3; echo "fin $(n)" >> "$G"
o 'milieu-relacher\n'; sleep 1; o 'aller %s %s\n' "$cx" "$cy"; sleep 0.3
garde; echo "molette $(n)" >> "$G"; o 'molette -3 600\n'; sleep 1.5; o 'molette 3 600\n'; sleep 1.5; echo "fin $(n)" >> "$G"
echo "pincer $(n)" >> "$G"
pave 'pincer 1.8 700\nattendre 300\npincer 0.55 700\nattendre 300\npincer 1.8 700\nattendre 300\npincer 0.55 700\nattendre 300\npincer 1.6 900\nattendre 300\npincer 0.6 900\n' pincer
sleep 1; echo "fin $(n)" >> "$G"
echo "deux-doigts $(n)" >> "$G"
pave 'glisser 30 0 500\nattendre 200\nglisser -30 0 500\nattendre 200\nglisser 0 20 500\nattendre 200\nglisser 0 -20 500\nattendre 200\nglisser 30 15 600\nattendre 200\nglisser -30 -15 600\n' deux-doigts
sleep 1; echo "fin $(n)" >> "$G"
o 'fin\n'; sleep 0.5
fermer
for _ in $(seq 20); do grep -q "^CODE" "$S" 2>/dev/null && break; sleep 0.5; done
cp /tmp/glucose-chronique/derniere-session.txt "$B/chronique.txt" 2>/dev/null
echo "ESSAI FINI $B"
