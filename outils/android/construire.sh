#!/usr/bin/env bash
# **Construit l'APK de Glucose** (fiche 54) : la bibliothèque Rust pour les deux processeurs des
# téléphones (ARM 64 et 32 bits), puis l'enveloppe Java par Gradle.
#
#   bash outils/android/construire.sh            l'APK de travail (signé de la clé de débogage)
#
# GLUCOSE_PROCESSEURS choisit les processeurs (par défaut « arm64-v8a armeabi-v7a », ceux des
# téléphones) : « x86_64 » donne l'APK du téléphone virtuel de ce PC (l'émulateur, fiche 82).
#
# Avec GLUCOSE_KEYSTORE (la clé) et GLUCOSE_KEYSTORE_PASSWORD, l'APK de publication, signé de la
# vraie clé — celle dont toute mise à jour doit être signée ; GLUCOSE_VERSION en donne la version.
#
# Ce qu'il faut sur la machine : le SDK et le NDK d'Android (ANDROID_HOME, ou
# ~/Android/Sdk), Java 17 ou plus, Gradle 9.6 ou plus, `cargo-ndk`, et les cibles Rust
# d'Android. La CI fait la même chose (tâche « apk ») et dépose l'APK.
set -euo pipefail
RACINE="$(cd "$(dirname "$0")/../.." && pwd)"
export ANDROID_HOME="${ANDROID_HOME:-$HOME/Android/Sdk}"
export ANDROID_NDK_HOME="${ANDROID_NDK_HOME:-$ANDROID_HOME/ndk/29.0.14206865}"
# Les objets d'Android à part de ceux du bureau : rien ne se recompile de l'un pour l'autre.
export CARGO_TARGET_DIR="$RACINE/target/android"

cd "$RACINE"
processeurs="${GLUCOSE_PROCESSEURS:-arm64-v8a armeabi-v7a}"
cibles=()
for processeur in $processeurs; do
  cibles+=(-t "$processeur")
done
# Seuls les processeurs demandés restent : la bibliothèque d'une construction précédente (celle
# du téléphone virtuel) partirait sinon dans l'APK.
for dossier in android/app/src/main/jniLibs/*/; do
  [ -d "$dossier" ] || continue
  case " $processeurs " in
    *" $(basename "$dossier") "*) ;;
    *) rm -rf "$dossier" ;;
  esac
done
cargo ndk "${cibles[@]}" -P 21 \
  -o android/app/src/main/jniLibs \
  build --release -p glucose-android

# Gradle (9.6 ou plus) : celui du système, ou celui que GRADLE désigne.
cd "$RACINE/android"
if [ -n "${GLUCOSE_KEYSTORE:-}" ]; then
  "${GRADLE:-gradle}" --no-daemon assembleRelease
  echo "APK : $RACINE/android/app/build/outputs/apk/release/app-release.apk"
else
  "${GRADLE:-gradle}" --no-daemon assembleDebug
  echo "APK : $RACINE/android/app/build/outputs/apk/debug/app-debug.apk"
fi
