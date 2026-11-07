#!/usr/bin/env bash
# **Glucose survit-il à sa console ?** (CONSOLE-1, fiche 88). Le vrai Glucose, dans un compositeur
# sans écran (rien ne s'ouvre chez lui), sur un dossier jetable ; sa sortie part dans un tuyau dont le
# lecteur s'en va après la première ligne — un terminal qu'on ferme. Au bout de `<secondes>`, il doit
# être encore vivant. Le 5/11/2026, une vraie session est tombée ainsi.
#
#   bash outils/console_disparue.sh <secondes> [programme]
#   GLUCOSE_CONSOLE=garde bash outils/console_disparue.sh …   le témoin : une console qui reste
#
# Rend `VIVANT` (code 0) ou `MORT` (code 1), et ce que sa boîte noire dit de sa fin.
set -u
racine=$(cd "$(dirname "$0")/.." && pwd)
secondes=${1:?usage : console_disparue.sh <secondes> [programme]}
exe=${2:-$racine/target/release/glucose-desktop}
maison=$(mktemp -d)
mkdir -p "$maison/state/glucose"
printf 'envoyer=non\nvue=oui\ninstallation=%032x\n' 4 > "$maison/state/glucose/telemetrie.txt"
compo=$(mktemp -d /tmp/gw.XXXXXX)
printf 'output HEADLESS-1 resolution 1280x800@60Hz\ndefault_border none\n' > "$compo/sway.conf"
env -u WAYLAND_DISPLAY -u DISPLAY -u NIRI_SOCKET -u SWAYSOCK XDG_RUNTIME_DIR="$compo" \
  WLR_BACKENDS=headless WLR_LIBINPUT_NO_DEVICES=1 \
  nix shell --inputs-from "$racine" nixpkgs#sway -c sway -c "$compo/sway.conf" > /dev/null 2>&1 &
pere_compo=$!
while kill -0 "$pere_compo" 2>/dev/null && ! [ -S "$compo/wayland-1" ]; do sleep 0.2; done
# La console : un tuyau dont le lecteur part après une ligne.
( nix develop "$racine" --no-write-lock-file -c env -u DISPLAY -u NIRI_SOCKET HOME="$maison" \
    XDG_RUNTIME_DIR="$compo" WAYLAND_DISPLAY=wayland-1 XDG_DATA_HOME="$maison/data" \
    XDG_STATE_HOME="$maison/state" XDG_CONFIG_HOME="$maison/config" XDG_CACHE_HOME="$maison/cache" \
    TMPDIR="$maison" "$exe" 2>&1 | if [ "${GLUCOSE_CONSOLE:-}" = garde ]; then cat > /dev/null; \
    else head -n 1 > /dev/null; fi ) &
# Compter depuis le début de sa session, pas depuis la commande : `nix develop` peut prendre son
# temps, et un Glucose pas encore lancé n'est pas un Glucose mort.
journal=""
for _ in $(seq 1 300); do
  journal=$(ls -t "$maison/state/glucose/journal-technique/"20*.txt 2>/dev/null | head -1)
  [ -n "$journal" ] && break
  sleep 0.2
done
if [ -z "$journal" ]; then
  verdict="JAMAIS LANCÉ"
else
  sleep "$secondes"
  pid=$(pgrep -f -x "$exe" | head -1)
  if [ -n "$pid" ]; then verdict=VIVANT; kill -TERM "$pid"; sleep 2; else verdict=MORT; fi
fi
echo "$verdict"
[ -n "$journal" ] && grep -E '"type":"(panique|fin)' "$journal" | cut -c1-200
for p in $(pgrep -P "$pere_compo") "$pere_compo"; do kill "$p" 2>/dev/null; done
wait 2>/dev/null
rm -rf "$compo" "$maison"
[ "$verdict" = VIVANT ]
