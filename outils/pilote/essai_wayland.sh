#!/usr/bin/env bash
# **Le presse-papiers et le glisser-déposer de Wayland, éprouvés dans le vrai Glucose** (fiche 100).
#
#   bash outils/pilote/essai_wayland.sh <dossier>
#
# Le vrai Glucose dans un sway sans écran (le pilote), et d'autres applications à côté :
# `wl-copy` / `wl-paste` (ce que fait Discord ou un terminal) et `dragon` (une application GTK
# qui glisse et reçoit des fichiers). Cinq gestes, chacun PASSE ou TOMBE, jugés sur ce que
# l'autre application reçoit ou sur les pixels de l'écran :
#
#   1. un texte écrit dans Glucose, copié, se relit ailleurs ;
#   2. une image copiée ailleurs se colle dans Glucose ;
#   3. cette photo, copiée dans Glucose, arrive ailleurs en PNG, au pixel près ;
#   4. une photo glissée depuis dragon se pose dans Glucose ;
#   5. une photo glissée hors de Glucose arrive dans dragon, au pixel près.
#
# Sort en erreur si un geste tombe. Rien ne se voit sur son écran.
#
# `PILOTE_COMPOSITEUR=niri bash outils/pilote/essai_wayland.sh <dossier>` (fiche 100 § 10) : les mêmes cinq
# gestes dans un vrai niri, le compositeur de son VivoBook, imbriqué dans le sway sans écran ;
# `dragon` y est une colonne à droite de Glucose, comme sway le pose en mosaïque (`niri.kdl`). `PILOTE_ECHELLE=2` : l'échelle de
# un écran à haute densité (le sien est à 1) ; ce qui se vise dans dragon suit l'échelle.
#
# Pièges payés (fiche 100) : `glisser` rend la main tout de suite — attendre sa durée avant de
# relâcher ; la fenêtre de dragon est petite (son bouton, en haut à gauche de sa case) : le
# reste de la case est le fond de sway, où rien ne se dépose.
set -u
racine=$(cd "$(dirname "$0")/../.." && pwd)
cd "$racine"
D=$1
P="bash outils/pilote/piloter.sh"
outil() { nix build --inputs-from "$racine" "nixpkgs#$1" --print-out-paths --no-link 2>/dev/null | head -1; }
PY=$(nix build --impure --print-out-paths --no-link --expr \
  '(import (builtins.getFlake "nixpkgs") {}).python3.withPackages (p: [p.pillow])' 2>/dev/null)/bin/python3
DRAGON=$(outil dragon-drop)/bin/dragon-drop
SWAYMSG=$(outil sway)/bin/swaymsg
WLC=$(outil wl-clipboard)/bin
rm -rf "$D"; mkdir -p "$D"
tombes=0
# L'échelle : l'intérieur de dragon grandit avec elle.
k=${PILOTE_ECHELLE:-1}
juger() {
  if [ "$2" = 0 ]; then echo "PASSE  $1"; else echo "TOMBE  $1"; tombes=$((tombes + 1)); fi
}

# Deux photos aux couleurs franches, pour les reconnaître à l'écran : orange et magenta.
"$PY" - "$D" <<'EOF'
import sys
from PIL import Image
for nom, (r, g, b) in (("orange", (255, 128, 0)), ("magenta", (230, 0, 230))):
    im = Image.new("RGB", (400, 250))
    im.putdata([(r, g, (b + (x * 7 + y * 3) % 20) % 256) for y in range(250) for x in range(400)])
    im.save(f"{sys.argv[1]}/{nom}.png")
EOF
# Combien de pixels de cette couleur dans cette région d'une capture.
compter() {
  "$PY" - "$@" <<'EOF'
import sys
from PIL import Image
f, couleur, x0, x1 = sys.argv[1], sys.argv[2], int(sys.argv[3]), int(sys.argv[4])
im = Image.open(f).convert("RGB").crop((x0, 80, x1, 1080))
proche = {"orange": lambda p: p[0] > 200 and 90 < p[1] < 170 and p[2] < 60,
          "magenta": lambda p: p[0] > 180 and p[1] < 60 and p[2] > 180}[couleur]
print(sum(1 for p in im.get_flattened_data() if proche(p)))
EOF
}
# Le centre des pixels de cette couleur, pour y saisir la photo.
#   centre <capture> <x0> <x1> <y0> <y1>
centre() {
  "$PY" - "$@" <<'EOF'
import sys
from PIL import Image
im = Image.open(sys.argv[1]).convert("RGB")
x0, x1, y0, y1 = (int(a) for a in sys.argv[2:6])
pts = [(x, y) for y in range(y0, y1, 4) for x in range(x0, x1, 4)
       if (lambda p: p[0] > 180 and p[1] < 60 and p[2] > 180)(im.getpixel((x, y)))]
print(sum(p[0] for p in pts) // max(len(pts), 1), sum(p[1] for p in pts) // max(len(pts), 1))
EOF
}
# Deux images ont-elles les mêmes pixels ?
pareilles() {
  "$PY" - "$1" "$2" <<'EOF'
import sys
from PIL import Image
a, b = (Image.open(f).convert("RGB") for f in sys.argv[1:3])
sys.exit(0 if a.size == b.size and a.tobytes() == b.tobytes() else 1)
EOF
}
# La boîte des pixels magenta de la moitié gauche d'une capture : « x0 y0 x1 y1 ».
boite() {
  "$PY" - "$1" <<'EOF'
import sys
from PIL import Image
im = Image.open(sys.argv[1]).convert("RGB")
pts = [(x, y) for y in range(80, im.height, 2) for x in range(0, 960, 2)
       if (lambda p: p[0] > 180 and p[1] < 60 and p[2] > 180)(im.getpixel((x, y)))]
xs, ys = [p[0] for p in pts] or [0], [p[1] for p in pts] or [0]
print(min(xs), min(ys), max(xs), max(ys))
EOF
}

$P demarrer "$D/maison" "$D/sortie"
timeout 90 bash -c "until grep -q 'scénario' '$D/sortie/sortie.txt' 2>/dev/null; do sleep 0.5; done"
sleep 2
c=$(cat /tmp/glucose-pilote/compo)
W="env -u SWAYSOCK -u NIRI_SOCKET -u DISPLAY XDG_RUNTIME_DIR=$c WAYLAND_DISPLAY=$(cat /tmp/glucose-pilote/prise)"
# Range dragon dans la moitié droite : sway le fait flotter, on le remet en mosaïque ; niri l'y
# pose de lui-même (sa règle de fenêtre).
ranger_dragon() {
  if [ "${PILOTE_COMPOSITEUR:-sway}" = sway ]; then
    $SWAYMSG -s "$(ls "$c"/sway-ipc.*.sock | head -1)" '[app_id="dragon-drop"] floating disable' > /dev/null
  fi
  sleep 1.5
}
# Au-dessus du contenu de dragon, sway laisse une bande de 26 pixels ; niri, qui demande aux
# applications de ne pas dessiner de barre, aucune.
if [ "${PILOTE_COMPOSITEUR:-sway}" = sway ]; then haut=26; else haut=0; fi

# 1. Un texte écrit dans Glucose, copié, se relit ailleurs.
TEXTE="Bonjour Discord, ceci vient de Glucose"
# L'outil texte par sa touche : dans une fenêtre étroite, la barre se replie en rail.
$P touche t; sleep 0.8; $P clic 100 560; sleep 1
$P taper "$TEXTE"; sleep 0.8; $P ctrl a; sleep 0.5; $P ctrl c; sleep 1
recu=$($W timeout 5 "$WLC/wl-paste" --no-newline 2>/dev/null)
[ "$recu" = "$TEXTE" ]; juger "copier un texte vers une autre application (« $recu »)" $?
$P touche Escape; sleep 0.5; $P clic 900 900; sleep 0.5

# 2. Une image copiée ailleurs se colle dans Glucose.
$P capture "$D/2-avant.png"
$W "$WLC/wl-copy" --type image/png < "$D/orange.png"; sleep 0.5
$P aller 700 300; sleep 0.3; $P ctrl v; sleep 2
$P capture "$D/2-apres.png"
avant=$(compter "$D/2-avant.png" orange 0 1920); apres=$(compter "$D/2-apres.png" orange 0 1920)
[ "$apres" -gt $((avant + 5000)) ]; juger "coller une image copiée ailleurs ($avant -> $apres pixels orange)" $?

# 3. Cette photo, choisie après le collage, copiée dans Glucose, arrive ailleurs en PNG.
$P ctrl c; sleep 1.5
$W timeout 5 "$WLC/wl-paste" --type image/png > "$D/3-recu.png" 2>/dev/null
pareilles "$D/3-recu.png" "$D/orange.png"; juger "copier une photo vers une autre application, au pixel près" $?

# 4. Une photo glissée depuis dragon se pose dans Glucose (Glucose à gauche, dragon à droite).
# D'abord dézoomer : la photo déposée doit tenir dans l'écran pour qu'on voie où elle tombe.
$P aller 700 600; sleep 0.3; $P molette 10 500; sleep 1.5
# Un chemin court : dragon l'écrit en entier, et un long l'élargirait au-delà de sa demi-colonne.
cp "$D/magenta.png" "$c/m.png"
$W GDK_BACKEND=wayland "$DRAGON" "$c/m.png" > /dev/null 2> "$D/dragon-source.txt" < /dev/null &
source_pid=$!
sleep 3
ranger_dragon
$P capture "$D/4-avant.png"
# La vignette de dragon, cherchée sur la capture : le magenta en haut de la moitié droite.
read -r vx vy < <(centre "$D/4-avant.png" 960 1920 0 300)
$P aller $vx $vy; sleep 0.3; $P presser; sleep 0.2
$P glisser $vx $vy $((vx - 30)) $((vy + 40)) 300 125; $P glisser $((vx - 30)) $((vy + 40)) 400 500 600 125; sleep 1.4
$P relacher; sleep 2.5
$P capture "$D/4-apres.png"
avant=$(compter "$D/4-avant.png" magenta 0 960); apres=$(compter "$D/4-apres.png" magenta 0 960)
[ "$apres" -gt $((avant + 5000)) ]; juger "glisser une photo d'une autre application vers Glucose ($avant -> $apres pixels magenta)" $?
# Et posée là où on l'a lâchée : Glucose centre un dépôt sous le curseur (à deux pixels près, la
# demi-teinte du bord).
read -r x0 y0 x1 y1 < <(boite "$D/4-apres.png")
cx=$(((x0 + x1) / 2)); cy=$(((y0 + y1) / 2))
[ $((cx > 400 ? cx - 400 : 400 - cx)) -le 3 ] && [ $((cy > 500 ? cy - 500 : 500 - cy)) -le 3 ]
juger "la photo déposée est centrée sous le curseur (lâchée en 400 500, centrée en $cx $cy)" $?
kill $source_pid 2> /dev/null; sleep 1

# 5. Une photo glissée hors de Glucose arrive dans dragon, au pixel près.
$W GDK_BACKEND=wayland "$DRAGON" --target --print-path > "$D/5-cible.txt" 2> "$D/dragon-cible.txt" < /dev/null &
cible_pid=$!
sleep 3
ranger_dragon
$P capture "$D/5-avant.png"
read -r mx my < <(centre "$D/5-avant.png" 0 960 80 1080)
$P aller "$mx" "$my"; sleep 0.3; $P presser; sleep 0.2
# Le bouton de dragon (« Drag something here »), et non le fond de sa case.
$P glisser "$mx" "$my" 700 400 400 125; $P glisser 700 400 $((960 + 90 * k)) $((haut + 16 * k)) 500 125; sleep 1.6
$P relacher; sleep 1.5
recu=$(grep -m1 "\.png" "$D/5-cible.txt")
[ -n "$recu" ] && pareilles "$recu" "$D/magenta.png"
juger "glisser une photo de Glucose vers une autre application (${recu:-rien reçu})" $?
kill $cible_pid 2> /dev/null

$P arreter
echo "$tombes geste(s) tombé(s)"
[ "$tombes" = 0 ]
