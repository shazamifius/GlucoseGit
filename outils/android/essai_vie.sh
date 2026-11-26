#!/usr/bin/env bash
# **La vie de Glucose au téléphone virtuel** (VIE-3, fiche 173) — l'essai qui remplace les
# épreuves qu'Android n'a nulle part : un réglage du système ne recrée pas l'activité, le
# retour ferme Glucose sans plantage, et il se rouvre.
#
#   bash outils/android/essai_vie.sh      # Glucose installé, le téléphone démarré
#
# Avant VIE-3 : après le retour, Glucose restait figé sur son logo (le fil de Glucose attendait
# dans la boucle de winit, Android attendait qu'il sorte) ; un réglage de la taille du texte
# le fermait ; et `exit` le faisait tomber en SIGABRT (« APP CRASH(NATIVE) »).
set -uo pipefail
T="bash $(dirname "$0")/telephone_virtuel.sh adb"
APP=com.glucose.app
echec=0
dire() { echo "$1"; }
tombe() { echo "TOMBE : $1"; echec=1; }
pid() { $T shell pidof "$APP" 2>/dev/null | tr -d '\r'; }
attendre() { # attendre <condition> <secondes>
    for _ in $(seq "$2"); do eval "$1" && return 0; sleep 1; done
    return 1
}
lancer() { # un processus neuf, jusqu'à sa fenêtre
    $T shell am start -n "$APP/.MainActivity" >/dev/null
    attendre '[ -n "$(pid)" ] && $T logcat -d -s Glucose 2>/dev/null | grep -q "fenetre :"' 40
}

# Un Glucose neuf, quel que soit celui d'avant.
$T shell am force-stop "$APP"
$T logcat -c
lancer || tombe "Glucose ne s'ouvre pas"
avant=$(pid)

# 1. Un réglage du système : l'activité reste — jamais détruite —, le processus aussi.
$T logcat -c
$T shell settings put system font_scale 1.15
sleep 4
$T shell settings put system font_scale 1.0
sleep 2
if $T logcat -d 2>/dev/null | grep -q "threaded_app: Destroy"; then
    tombe "1. la taille du texte a détruit l'activité"
elif [ "$(pid)" = "$avant" ]; then
    dire "1. taille du texte changée : Glucose reste ($avant)"
else
    tombe "1. la taille du texte a fermé Glucose (avant $avant, après '$(pid)')"
fi

# 2. Le retour : Glucose finit, le processus part de lui-même, sans plantage.
$T logcat -c
$T shell input keyevent 4
if attendre '[ -z "$(pid)" ]' 15; then
    raison=$($T shell dumpsys activity exit-info "$APP" | grep -oE " reason=[0-9]+ \([A-Z_ ]+\)" | head -1)
    case "$raison" in
        *EXIT_SELF*) dire "2. retour : le processus part de lui-même ($raison)" ;;
        *) tombe "2. retour : le processus est mort autrement ($raison)" ;;
    esac
else
    tombe "2. retour : le processus reste (figé ?)"
fi

# 3. Il se rouvre.
$T logcat -c
if lancer; then
    dire "3. rouvert ($(pid))"
else
    tombe "3. ne se rouvre pas : son logo reste"
fi

[ "$echec" = 0 ] && echo "VIE-3 : tout tient" || echo "VIE-3 : un essai tombe"
exit "$echec"
