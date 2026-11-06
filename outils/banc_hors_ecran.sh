#!/usr/bin/env bash
# **Le vrai Glucose, sa vraie carte graphique, et aucune fenêtre sur son écran** (fiche 87).
#
#   bash outils/banc_hors_ecran.sh <scénario> <dossier de sortie> [VAR=valeur…]
#
# Un compositeur Wayland sans écran (`sway`, moteur `headless`) tourne dans un dossier à lui : la
# fenêtre existe, Vulkan présente, l'horloge de 60 Hz bat, mais rien ne prend le focus et rien
# ne se voit. Deux mesures comparables peuvent donc se répéter sans le déranger (fiche 86 : quatre
# fenêtres pour deux annoncées, et il a cru à un défaut).
#
# Le tableau : le dossier d'essai `GLUCOSE_BANC_MAISON` (par défaut ~/.cache/glucose-bancs/essai),
# jamais le sien. La télémétrie y est coupée d'avance. Le programme : `GLUCOSE_EXE`.
# Sortie : `sortie.txt`, `chronique.txt`, `journal.txt`, et `CODE <n>` à la fin de `sortie.txt`.
# Une variable `GLUCOSE_CAPTURE_A=<secondes>` prend aussi une capture (`capture.png`, par `grim`).
set -u
racine=$(cd "$(dirname "$0")/.." && pwd)
scenario=${1:?usage : banc_hors_ecran.sh <scénario> <dossier de sortie> [VAR=valeur…]}
sortie=${2:?usage : banc_hors_ecran.sh <scénario> <dossier de sortie> [VAR=valeur…]}
shift 2
maison=${GLUCOSE_BANC_MAISON:-$HOME/.cache/glucose-bancs/essai}
exe=${GLUCOSE_EXE:-$racine/target/release/glucose-desktop}
mkdir -p "$sortie" "$maison/state/glucose"
sortie=$(cd "$sortie" && pwd)
scenario=$(cd "$(dirname "$scenario")" && pwd)/$(basename "$scenario")
accord="$maison/state/glucose/telemetrie.txt"
[ -e "$accord" ] || printf 'envoyer=non\nvue=oui\ninstallation=%032x\n' 3 > "$accord"

# Une prise Unix tient en 108 caractères : le dossier du compositeur reste court.
compo=$(mktemp -d /tmp/gw.XXXXXX)
temp="$sortie/temp"
mkdir -p "$temp"
printf 'output HEADLESS-1 resolution 1920x1080@60Hz\ndefault_border none\n' > "$compo/sway.conf"
env -u WAYLAND_DISPLAY -u DISPLAY -u NIRI_SOCKET -u SWAYSOCK XDG_RUNTIME_DIR="$compo" \
  WLR_BACKENDS=headless WLR_LIBINPUT_NO_DEVICES=1 \
  nix shell nixpkgs#sway -c sway -c "$compo/sway.conf" > "$sortie/compositeur.txt" 2>&1 &
pere_compo=$!
for _ in $(seq 1 100); do
  [ -S "$compo/wayland-1" ] && break
  sleep 0.2
done
if ! [ -S "$compo/wayland-1" ]; then
  echo "CODE compositeur absent" >> "$sortie/sortie.txt"
  kill "$pere_compo" 2>/dev/null
  rm -rf "$compo"
  exit 1
fi

if [ -n "${GLUCOSE_CAPTURE_A:-}" ]; then
  (sleep "$GLUCOSE_CAPTURE_A" && env -u SWAYSOCK XDG_RUNTIME_DIR="$compo" WAYLAND_DISPLAY=wayland-1 \
    grim "$sortie/capture.png") &
fi

nix develop "$racine" --no-write-lock-file -c env -u DISPLAY -u NIRI_SOCKET HOME="$maison" \
  XDG_RUNTIME_DIR="$compo" WAYLAND_DISPLAY=wayland-1 XDG_DATA_HOME="$maison/data" \
  XDG_STATE_HOME="$maison/state" XDG_CONFIG_HOME="$maison/config" XDG_CACHE_HOME="$maison/cache" \
  TMPDIR="$temp" GLUCOSE_MISE_A_JOUR=non GLUCOSE_SCENARIO="$scenario" "$@" "$exe" \
  > "$sortie/sortie.txt" 2>&1
code=$?
echo "CODE $code" >> "$sortie/sortie.txt"
cp "$temp/glucose-chronique/derniere-session.txt" "$sortie/chronique.txt" 2>/dev/null
journal=$(ls -t "$maison/state/glucose/journal-technique/"20*.txt 2>/dev/null | head -1)
[ -n "$journal" ] && cp "$journal" "$sortie/journal.txt"

# Le compositeur et son enveloppe `nix shell` : par leur père, jamais par un motif (METHODE § 5).
for p in $(pgrep -P "$pere_compo") "$pere_compo"; do kill "$p" 2>/dev/null; done
wait "$pere_compo" 2>/dev/null
rm -rf "$compo"
exit "$code"
