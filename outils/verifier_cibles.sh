#!/usr/bin/env bash
# **Vérifier ici le code de Windows et du Mac** (fiche 156). Jusqu'au 25/11, une faute dans ce qui ne
# se compile que pour eux (`#[cfg(windows)]`, `#[cfg(target_os = "macos")]`) n'apparaissait qu'à la CI
# d'une publication. Ici, le `clippy` de la CI (`--workspace --all-targets -D warnings`), pour leurs
# cibles, en une minute environ chacune (66 s pour Windows, 35 s pour le Mac, le 25/11, à froid).
#
#   bash outils/verifier_cibles.sh             # Windows, puis le Mac
#   bash outils/verifier_cibles.sh windows     # une seule : windows, mac
#
# **Ce qu'il prouve** : que le code de ces cibles se compile et passe `clippy`, épreuves comprises.
# **Ce qu'il ne prouve pas** : qu'il marche. Rien ne s'y lie ni ne s'y lance ; leurs épreuves ne
# tournent que sur la CI, et une correction propre à l'un reste « sans essai réel » (MÉTHODE § 5).
#
# **Comment, sans un second Rust.** Le `rustc` de nixpkgs n'apporte que la bibliothèque standard de
# Linux. `-Zbuild-std` la compile pour la cible, depuis ses sources (`rustPlatform.rustLibSrc`, du même
# nixpkgs verrouillé que `nix develop`, donc de la même version) ; `RUSTC_BOOTSTRAP=1` ouvre les `-Z`
# à un compilateur stable. Cargo cherche ces sources dans le dossier de `rustc` (`--print sysroot`),
# où nixpkgs ne les met pas : `__CARGO_TESTS_ONLY_SRC_ROOT` les lui montre. **C'est un nom interne de
# Cargo** : s'il disparaît, Cargo réclamera « rust-src » et l'outil tombera, jamais vert en silence —
# alors prendre une chaîne qui porte la bibliothèque des cibles (fenix ou rust-overlay), au prix d'un
# second Rust et d'une entrée de plus au flake.
#
# * **Windows : `-msvc`**, la cible de la CI ; `-gnu` demande le `dlltool` de MinGW.
# * **Le Mac : `aarch64-apple-darwin`.** `ring` (sous `rustls`) compile du C : le `clang` nu de
#   nixpkgs (son enveloppe ajoute les options de Linux, et `gcc` ne connaît pas `-arch`), et le SDK
#   d'Apple, sa source seule (424 Mo ; le paquet ne se construit que sur un Mac), en `SDKROOT`.
# * **Android, pas encore** : `ring` y demande les en-têtes du NDK, absent ici.
# * **Sa propre cible, `target/autres`** : `RUSTC_BOOTSTRAP` change les empreintes de Cargo ; dans
#   `target/`, chaque passage recompilerait tout le dépôt, et la compilation suivante encore.
set -u
cd "$(dirname "$0")/.." || exit 2
cibles=("$@")
[ ${#cibles[@]} -eq 0 ] && cibles=(windows mac)

nixpkgs() { nix build --inputs-from . "nixpkgs#$1" --print-out-paths --no-link; }
sources=$(nixpkgs rustPlatform.rustLibSrc) || exit 2

tombees=0
for nom in "${cibles[@]}"; do
  extra=()
  case "$nom" in
    windows) triple=x86_64-pc-windows-msvc ;;
    mac)
      triple=aarch64-apple-darwin
      clang=$(nixpkgs llvmPackages.clang-unwrapped) || exit 2
      sdk=$(nixpkgs apple-sdk.src) || exit 2
      extra=("CC_aarch64_apple_darwin=$clang/bin/clang" "SDKROOT=$sdk")
      ;;
    *) echo "cible inconnue : $nom (windows, mac)" >&2; exit 2 ;;
  esac
  debut=$(date +%s)
  bash outils/borne.sh nix develop -c env __CARGO_TESTS_ONLY_SRC_ROOT="$sources" RUSTC_BOOTSTRAP=1 \
    "${extra[@]}" cargo clippy -Zbuild-std=std,panic_abort --target "$triple" --target-dir target/autres \
    --workspace --all-targets -- -D warnings
  code=$?
  duree=$(( $(date +%s) - debut ))
  if [ $code -eq 0 ]; then
    echo "== $nom ($triple) : vert en $duree s"
  else
    echo "== $nom ($triple) : TOMBÉ (code $code) en $duree s"
    tombees=$(( tombees + 1 ))
  fi
done
[ $tombees -eq 0 ]
