#!/usr/bin/env bash
# **Lire ce que le téléphone a gardé** (P0, ANDROID-MEMOIRE, fiche 66) — sans rien y changer,
# et sans ouvrir Glucose : la raison du dernier redémarrage, les morts passées de Glucose telles
# qu'Android les a notées, la boîte des incidents du système, les journaux, la mémoire, le
# pilote graphique, et un rapport de bogue complet (il porte le journal du noyau d'avant le
# redémarrage quand le téléphone le garde).
#
#   bash outils/android/lire_le_telephone.sh <dossier de sortie>
#
# Le téléphone branché par le câble, le débogage USB accepté. Rien n'est écrit sur le téléphone.
set -u
ADB="${ADB:-C:/Users/Administrator/Android/Sdk/platform-tools/adb.exe}"
SORTIE="${1:?un dossier de sortie}"
PAQUET="com.glucose.app"
mkdir -p "$SORTIE"

lire() {
  local nom="$1"
  shift
  "$ADB" "$@" > "$SORTIE/$nom" 2>&1
  echo "$nom : $(wc -c < "$SORTIE/$nom") octets"
}

lire appareils.txt devices -l
lire proprietes.txt shell getprop
lire redemarrage.txt shell 'getprop sys.boot.reason; getprop sys.boot.reason.last; getprop ro.boot.bootreason; getprop persist.sys.boot.reason.history; cat /proc/uptime'
lire morts-de-glucose.txt shell dumpsys activity exit-info "$PAQUET"
lire paquet.txt shell dumpsys package "$PAQUET"
lire dropbox.txt shell dumpsys dropbox --print
lire plantages.txt logcat -b crash -d
lire journal.txt logcat -b all -d -v threadtime
lire meminfo-systeme.txt shell cat /proc/meminfo
lire meminfo-glucose.txt shell dumpsys meminfo "$PAQUET"
lire memoire-du-systeme.txt shell dumpsys meminfo
lire carte-graphique.txt shell 'dumpsys gpu; getprop ro.hardware.vulkan; getprop ro.hardware.egl; getprop ro.gfx.driver.0'
lire lmkd.txt shell 'getprop | grep -i -e lmk -e zram -e swap; cat /proc/pressure/memory'
echo "rapport de bogue complet (deux à cinq minutes)..."
"$ADB" bugreport "$SORTIE/rapport.zip" > "$SORTIE/rapport.txt" 2>&1
echo "rapport : $(wc -c < "$SORTIE/rapport.zip" 2>/dev/null || echo 0) octets"
