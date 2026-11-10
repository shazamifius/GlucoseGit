#!/usr/bin/env bash
# **Piloter le vrai Glucose dans un compositeur sans écran** (fiche 99) : de vrais clics (un pointeur
# virtuel persistant, `pointeur.py` et `protocole.py`), de vraies frappes (`wtype`, un clavier virtuel Wayland), des
# captures (`grim`). Rien ne se voit sur son écran, rien ne prend son focus. Le banc hors écran
# (`outils/banc_hors_ecran.sh`) ne joue que des commandes ; celui-ci joue la main.
#
#   bash outils/pilote/piloter.sh demarrer <maison> <sortie> [VAR=valeur…]
#        (le document : le dernier brouillon de <maison>/state/glucose/brouillons ; jamais le sien)
#   bash outils/pilote/piloter.sh clic|double|droit|aller <x> <y>
#   bash outils/pilote/piloter.sh presser|relacher|milieu-presser|milieu-relacher
#   bash outils/pilote/piloter.sh glisser <x0> <y0> <x1> <y1> <ms> <hz>   (le déplacement au clic molette)
#   bash outils/pilote/piloter.sh molette <crans> [<ms>]   (négatif : on zoome ; positif : on dézoome)
#   « glisser » et « molette » rendent la main tout de suite : le pointeur joue le geste pendant
#   que le script continue — attendre sa durée avant l'ordre suivant (un « relacher » trop tôt
#   lâche au milieu du geste).
#   bash outils/pilote/piloter.sh taper <texte> | touche <nom> | ctrl <t> | alt <t> | altmaj <t>
#   bash outils/pilote/piloter.sh capture <f.png> | arreter
#
# `PILOTE_COMPOSITEUR=niri … demarrer` (fiche 100 § 10) : Glucose dans un vrai niri — le compositeur de son
# VivoBook —, imbriqué dans ce sway sans écran (`niri.kdl`, mêmes coordonnées que sway seul).
# `PILOTE_ECHELLE=2` : l'échelle de son écran (1 par défaut) ; les coordonnées restent celles de
# la capture. Le pointeur, le clavier, les captures et `prise` vont alors à niri.
#
# Pièges payés (fiche 99 § 1) : sway sans écran n'a aucun périphérique — ses commandes de curseur
# bougent un curseur que nul client ne reçoit ; le pointeur virtuel doit vivre AVANT Glucose et
# entre deux ordres. `wtype` perd la première touche : une pause d'amorce (`-s 300`). Glucose
# s'ouvre en 1440 × 900 puis sway le pose à 1920 × 1080 : les coordonnées sont celles de la capture.
set -u
racine=$(cd "$(dirname "$0")/../.." && pwd)
ici=$(cd "$(dirname "$0")" && pwd)
etat=/tmp/glucose-pilote

outil() { nix build --inputs-from "$racine" "nixpkgs#$1" --print-out-paths --no-link 2>/dev/null | head -1; }
python_wl() {
  nix build --impure --print-out-paths --no-link --expr \
    '(import (builtins.getFlake "nixpkgs") {}).python3.withPackages (p: [p.pywayland])' 2>/dev/null
}
compo() { cat "$etat/compo"; }
# La prise Wayland où vit Glucose : celle de sway, ou celle de niri imbriqué.
prise() { cat "$etat/prise" 2>/dev/null || echo wayland-1; }
wl() { local c; c=$(compo); env -u SWAYSOCK -u NIRI_SOCKET XDG_RUNTIME_DIR="$c" WAYLAND_DISPLAY="$(prise)" "$@"; }
# Un ordre au pointeur, par son tuyau. Borné : un tuyau sans lecteur (le pointeur mort) figerait
# l'écriture pour toujours — et le banc entier avec elle (fiche 100 § 10).
ordre() {
  local texte; texte=$(printf "$@")
  timeout 5 bash -c 'printf "%s\n" "$1" > "$2"' _ "$texte" "$etat/pointeur" ||
    echo "le pointeur ne répond plus ($texte)" >&2
}
wtype_() { wl "$(cat "$etat/wtype")" "$@"; }

case "${1:-}" in
demarrer)
  maison=$2; sortie=$3; shift 3
  mkdir -p "$etat" "$sortie" "$maison/state/glucose"
  sortie=$(cd "$sortie" && pwd)
  sway=$(outil sway)/bin; py=$(python_wl)/bin/python3
  echo "$(outil wtype)/bin/wtype" > "$etat/wtype"
  accord="$maison/state/glucose/telemetrie.txt"
  [ -e "$accord" ] || printf 'envoyer=non\nvue=oui\ninstallation=%032x\n' 3 > "$accord"
  c=$(mktemp -d /tmp/gp.XXXXXX); echo "$c" > "$etat/compo"
  printf 'output HEADLESS-1 resolution 1920x1080@60Hz\ndefault_border none\n' > "$c/sway.conf"
  # PILOTE_DETAIL=1 : le journal détaillé de sway (focus, grabs) et chaque message qu'il échange avec
  # ses clients (WAYLAND_DEBUG=server), dans compositeur.txt.
  env -u WAYLAND_DISPLAY -u DISPLAY -u NIRI_SOCKET -u SWAYSOCK XDG_RUNTIME_DIR="$c" \
    ${PILOTE_DETAIL:+WAYLAND_DEBUG=server} \
    WLR_BACKENDS=headless WLR_LIBINPUT_NO_DEVICES=1 "$sway/sway" ${PILOTE_DETAIL:+-d} -c "$c/sway.conf" \
    > "$sortie/compositeur.txt" 2>&1 &
  echo $! > "$etat/sway.pid"
  for _ in $(seq 100); do [ -S "$c/wayland-1" ] && break; sleep 0.1; done
  echo wayland-1 > "$etat/prise"
  if [ "${PILOTE_COMPOSITEUR:-sway}" = niri ]; then
    k=${PILOTE_ECHELLE:-1}
    sed "s/@ECHELLE@/$k/" "$ici/niri.kdl" > "$c/niri.kdl"
    env -u DISPLAY -u NIRI_SOCKET -u SWAYSOCK XDG_RUNTIME_DIR="$c" WAYLAND_DISPLAY=wayland-1 \
      "$(outil niri)/bin/niri" -c "$c/niri.kdl" > "$sortie/niri.txt" 2>&1 &
    echo $! > "$etat/niri.pid"
    for _ in $(seq 150); do [ -S "$c/wayland-2" ] && break; sleep 0.1; done
    echo wayland-2 > "$etat/prise"
  fi
  rm -f "$etat/pointeur"
  wl "$py" "$ici/pointeur.py" "$etat/pointeur" > "$sortie/pointeur.txt" 2>&1 &
  echo $! > "$etat/pointeur.pid"
  for _ in $(seq 100); do [ -p "$etat/pointeur" ] && break; sleep 0.1; done
  # Le cadrage attend que sway ait posé la fenêtre à sa taille : sinon la scène est décalée.
  printf '3 tout-cadrer\n1200 quitter\n' > "$etat/scenario.txt"
  exe=${GLUCOSE_EXE:-$racine/target/release/glucose-desktop}
  (cd "$racine" && nix develop "$racine" --no-write-lock-file -c env -u DISPLAY -u NIRI_SOCKET \
    HOME="$maison" XDG_RUNTIME_DIR="$c" WAYLAND_DISPLAY="$(prise)" XDG_DATA_HOME="$maison/data" \
    XDG_STATE_HOME="$maison/state" XDG_CONFIG_HOME="$maison/config" XDG_CACHE_HOME="$maison/cache" \
    GLUCOSE_MISE_A_JOUR=non GLUCOSE_SCENARIO="$etat/scenario.txt" "$@" "$exe" \
    > "$sortie/sortie.txt" 2>&1; echo "CODE $?" >> "$sortie/sortie.txt") &
  echo $! > "$etat/glucose.pid"
  ;;
clic) ordre 'aller %s %s\npresser\nrelacher\n' "$2" "$3" ;;
double) ordre 'aller %s %s\npresser\nrelacher\npresser\nrelacher\n' "$2" "$3" ;;
droit) ordre 'aller %s %s\ndroit\n' "$2" "$3" ;;
aller) ordre 'aller %s %s\n' "$2" "$3" ;;
presser) ordre 'presser\n' ;;
relacher) ordre 'relacher\n' ;;
milieu-presser) ordre 'milieu-presser\n' ;;
milieu-relacher) ordre 'milieu-relacher\n' ;;
glisser) ordre 'glisser %s %s %s %s %s %s\n' "$2" "$3" "$4" "$5" "$6" "$7" ;;
molette) ordre 'molette %s %s\n' "$2" "${3:-80}" ;;
taper) wtype_ -s 300 -d 40 -- "$2" ;;
touche) wtype_ -s 300 -k "$2" ;;
ctrl) wtype_ -s 300 -M ctrl -k "$2" -m ctrl ;;
alt) wtype_ -s 300 -M alt -k "$2" -m alt ;;
altmaj) wtype_ -s 300 -M alt -M shift -k "$2" -m shift -m alt ;;
capture) wl grim "$2" ;;
arreter)
  c=$(compo)
  ordre 'fin\n' 2>/dev/null
  kill "$(cat "$etat/pointeur.pid")" 2>/dev/null
  [ -e "$etat/niri.pid" ] && kill "$(cat "$etat/niri.pid")" 2>/dev/null
  kill "$(cat "$etat/sway.pid")" 2>/dev/null
  rm -f "$etat/niri.pid" "$etat/prise"
  sleep 1
  rm -rf "$c" ;;
*) echo "usage : voir l'en-tête"; exit 2 ;;
esac
