#!/usr/bin/env bash
# **Le téléphone virtuel de ce poste** (fiche 171) : un Android 16 x86_64 sans fenêtre, pour
# éprouver Glucose au téléphone sans téléphone — rien ne s'affiche sur son écran.
#
#   bash outils/android/telephone_virtuel.sh demarrer          # le crée s'il n'existe pas, le lance
#   bash outils/android/telephone_virtuel.sh construire        # l'APK x86_64 (construire.sh)
#   bash outils/android/telephone_virtuel.sh installer [apk]   # installe et lance Glucose
#   bash outils/android/telephone_virtuel.sh capture <f.png>
#   bash outils/android/telephone_virtuel.sh adb <arguments…>  # tout le reste : input, logcat…
#   bash outils/android/telephone_virtuel.sh arreter
#
# Le SDK vient de `sdk.nix` (`~/.cache/glucose-android/sdk`, construit au premier appel) ; le
# téléphone vit dans `~/.cache/glucose-android/avd`. Ses gestes : `adb shell input tap x y`,
# `input text`, `input keycombination 113 50` (Ctrl+V : Glucose colle comme au bureau).
set -euo pipefail
RACINE="$(cd "$(dirname "$0")/../.." && pwd)"
CACHE="$HOME/.cache/glucose-android"
LIEN="$CACHE/sdk"
if [ ! -e "$LIEN" ]; then
  mkdir -p "$CACHE"
  (cd "$RACINE" && nix build --impure --expr 'import ./outils/android/sdk.nix {}' -o "$LIEN")
fi
export ANDROID_HOME="$LIEN/libexec/android-sdk"
export ANDROID_SDK_ROOT="$ANDROID_HOME"
export ANDROID_AVD_HOME="$CACHE/avd"
export ANDROID_EMULATOR_HOME="$CACHE"
ADB="$ANDROID_HOME/platform-tools/adb"
IMAGE="system-images;android-36;google_apis;x86_64"
NOM=glucose

case "${1:-}" in
demarrer)
  mkdir -p "$ANDROID_AVD_HOME"
  if [ ! -d "$ANDROID_AVD_HOME/$NOM.avd" ]; then
    echo no | "$LIEN/bin/avdmanager" create avd -n "$NOM" -k "$IMAGE" -d pixel_6
  fi
  # Sans fenêtre, sans son, sans instantané : chaque démarrage part propre. La carte graphique
  # passe par SwiftShader (le processeur) : le rendu du téléphone ne dépend pas de celle du PC.
  nohup "$ANDROID_HOME/emulator/emulator" -avd "$NOM" -no-window -no-audio -no-boot-anim \
    -no-snapshot -gpu swangle_indirect -memory 4096 > "$CACHE/emulateur.txt" 2>&1 &
  echo $! > "$CACHE/emulateur.pid"
  "$ADB" wait-for-device
  until [ "$("$ADB" shell getprop sys.boot_completed 2>/dev/null | tr -d '\r')" = 1 ]; do sleep 2; done
  echo "téléphone virtuel prêt"
  ;;
construire)
  # La bibliothèque standard d'Android se compile depuis ses sources, comme `verifier_cibles.sh`
  # le fait (sans second Rust) ; `cargo-ndk` et Gradle 9 viennent de nixpkgs.
  export ANDROID_NDK_HOME="$ANDROID_HOME/ndk/29.0.14206865"
  cd "$RACINE"
  sources=$(nix build --inputs-from . nixpkgs#rustPlatform.rustLibSrc --print-out-paths --no-link)
  outils=$(nix build --inputs-from . nixpkgs#cargo-ndk nixpkgs#gradle_9 --print-out-paths --no-link \
    | sed 's|$|/bin|' | paste -sd:)
  # Une bibliothèque standard compilée ici n'a pas les sources C de `compiler-builtins` : le TLS
  # émulé d'Android (`__emutls_get_address`) n'y est pas, et `-nodefaultlibs` écarte les builtins
  # du NDK qui le portent — Glucose ne se chargeait pas (25/11). On les nomme à l'édition de liens.
  builtins=$(echo "$ANDROID_NDK_HOME"/toolchains/llvm/prebuilt/linux-x86_64/lib/clang/*/lib/linux/libclang_rt.builtins-x86_64-android.a)
  # Le PATH de `nix develop` (son Rust), puis ces outils : il se compose à l'intérieur.
  CARGO_TARGET_X86_64_LINUX_ANDROID_RUSTFLAGS="-Clink-arg=$builtins" \
  OUTILS="$outils" RUSTC_BOOTSTRAP=1 __CARGO_TESTS_ONLY_SRC_ROOT="$sources" \
    CARGO_UNSTABLE_BUILD_STD=std,panic_abort GLUCOSE_PROCESSEURS=x86_64 \
    bash outils/borne.sh nix develop -c bash -c 'PATH="$OUTILS:$PATH" exec bash outils/android/construire.sh'
  ;;
installer)
  apk="${2:-$RACINE/android/app/build/outputs/apk/debug/app-debug.apk}"
  "$ADB" install -r "$apk"
  "$ADB" shell am start -n com.glucose.app/.MainActivity
  ;;
capture) "$ADB" exec-out screencap -p > "$2" ;;
adb) shift; "$ADB" "$@" ;;
arreter)
  "$ADB" emu kill 2>/dev/null || true
  [ -e "$CACHE/emulateur.pid" ] && kill "$(cat "$CACHE/emulateur.pid")" 2>/dev/null || true
  rm -f "$CACHE/emulateur.pid"
  ;;
*) sed -n 2,14p "$0"; exit 2 ;;
esac
