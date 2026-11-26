#!/usr/bin/env bash
# **Lancer sous Wine les épreuves de Glucose compilées pour Windows** (fiche 175). `verifier_cibles.sh`
# prouve que le code de Windows se compile ; celui-ci le fait **tourner** : les épreuves de
# `glucose-desktop` (un filtre, comme pour `cargo test`), compilées pour Windows et lancées par
# Wine.
#
#   bash outils/essai_wine.sh video          # les épreuves dont le nom contient « video »
#
# **Ce qu'il prouve** : que le chemin de Windows marche sous Wine — ses portes (Media Foundation,
# COM, WIC…) sont celles que Wine imite, sur GStreamer. **Ce qu'il ne prouve pas** : qu'il marche
# sous un vrai Windows, dont le comportement peut s'écarter de l'imitation. Un vert ici est « vu
# sous Wine », jamais « réglé ».
#
# **Le Wine de nixpkgs, 64 bits** (`wineWow64Packages.full`, avec GStreamer ; 280 Mo tirés du cache
# une fois), pas celui de la machine : il n'a que sa moitié 32 bits, et en 32 bits
# `GetWindowLongPtrW` rend un `i32` (onze fautes de compilation dans le code des fenêtres). La
# cible est `x86_64-pc-windows-gnu` (MinGW de nixpkgs, `pkgsCross.mingwW64`, 120 Mo) : la même
# largeur que la cible de la CI (`-msvc`), un autre éditeur de liens. La bibliothèque standard se
# compile depuis ses sources, comme pour `verifier_cibles.sh` (son en-tête dit comment).
#
# La bibliothèque standard de Rust lie `libpthread.a` (winpthreads), que le MinGW de nixpkgs ne
# porte pas (ses fils sont ceux de `mcfgthread`) : l'enveloppe de son `gcc` la reçoit par
# `NIX_CFLAGS_LINK_<cible>`, qui ne change pas les empreintes de Cargo (`RUSTFLAGS` recompilerait tout).
#
# MinGW passe devant le `PATH` **dans** `nix develop` (`bash -c`) : écrit dehors, le `PATH` serait
# celui d'avant `nix develop`, et un autre `rustc` compilerait.
#
# **Son préfixe Wine** vit sous `target/autres/wine` : jamais celui de l'utilisateur. Il se crée
# avant la première épreuve, **sans Mono ni Gecko** (`WINEDLLOVERRIDES`) : sinon `wineboot` ouvre la
# boîte qui propose de les installer, l'attend, et le préfixe reste à moitié fait — Media
# Foundation n'y avait aucun décodeur vidéo inscrit (8 transformations au lieu de 48).
#
# `GLUCOSE_EXIGER_LES_VIDEOS` : sous Wine, les vidéos doivent se lire ; une épreuve qui se
# sauterait faute de lecteur mentirait en vert (le premier essai : « 36 passées » en 0,22 s).
set -u
cd "$(dirname "$0")/.." || exit 2
filtre=${1:-}
# `^out` : la sortie principale seule (la page de manuel sortait la première).
nixpkgs() { nix build --inputs-from . "nixpkgs#$1^out" --print-out-paths --no-link; }
sources=$(nixpkgs rustPlatform.rustLibSrc) || exit 2
mingw=$(nixpkgs pkgsCross.mingwW64.buildPackages.gcc) || exit 2
binutils=$(nixpkgs pkgsCross.mingwW64.buildPackages.binutils) || exit 2
pthreads=$(nixpkgs pkgsCross.mingwW64.windows.pthreads) || exit 2
wine=$(nixpkgs wineWow64Packages.full) || exit 2
prefixe=$PWD/target/autres/wine
export WINEPREFIX=$prefixe WINEDEBUG=-all WINEDLLOVERRIDES="mscoree,mshtml="
if [ ! -f "$prefixe/system.reg" ]; then
  timeout 600 "$wine/bin/wine" wineboot --init >/dev/null 2>&1
  timeout 300 "$wine/bin/wineserver" -w
fi
triple=x86_64-pc-windows-gnu
TRIPLE=X86_64_PC_WINDOWS_GNU
bash outils/borne.sh nix develop -c env __CARGO_TESTS_ONLY_SRC_ROOT="$sources" RUSTC_BOOTSTRAP=1 \
  MINGW="$mingw/bin:$binutils/bin" \
  "NIX_CFLAGS_LINK_x86_64_w64_mingw32=-L$pthreads/lib" \
  "CC_x86_64_pc_windows_gnu=x86_64-w64-mingw32-gcc" \
  "CARGO_TARGET_${TRIPLE}_LINKER=x86_64-w64-mingw32-gcc" \
  "CARGO_TARGET_${TRIPLE}_RUNNER=$wine/bin/wine" \
  GLUCOSE_EXIGER_LES_VIDEOS=1 \
  bash -c 'PATH="$MINGW:$PATH" exec "$@"' -- \
  cargo test -Zbuild-std=std,panic_unwind --target "$triple" --target-dir target/autres \
  -p glucose-desktop --lib -- "$filtre" "${@:2}"
