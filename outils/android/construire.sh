#!/usr/bin/env bash
# **Construit l'APK de Glucose** (fiche 54) : la bibliothèque Rust pour les deux processeurs des
# téléphones (ARM 64 et 32 bits), puis l'enveloppe Java par Gradle.
#
#   bash outils/android/construire.sh            l'APK de travail (signé de la clé de débogage)
#
# Ce qu'il faut sur la machine : le SDK et le NDK d'Android (ANDROID_HOME, ou
# ~/Android/Sdk), Java 17 ou plus, `cargo-ndk`, et les cibles Rust d'Android.
set -euo pipefail
RACINE="$(cd "$(dirname "$0")/../.." && pwd)"
export ANDROID_HOME="${ANDROID_HOME:-$HOME/Android/Sdk}"
export ANDROID_NDK_HOME="${ANDROID_NDK_HOME:-$ANDROID_HOME/ndk/29.0.14206865}"
# Les objets d'Android à part de ceux du bureau : rien ne se recompile de l'un pour l'autre.
export CARGO_TARGET_DIR="$RACINE/target/android"

cd "$RACINE"
cargo ndk -t arm64-v8a -t armeabi-v7a -P 21 \
  -o android/app/src/main/jniLibs \
  build --release -p glucose-android

cd "$RACINE/android"
./gradlew --no-daemon assembleDebug
echo "APK : $RACINE/android/app/build/outputs/apk/debug/app-debug.apk"
