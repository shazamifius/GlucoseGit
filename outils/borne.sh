#!/usr/bin/env bash
# **Borner une commande lourde** (fiche 110). Le 12/11/2026, `cargo test --workspace` a lancé huit
# compilations à la fois ; la mémoire (7,6 Go) a débordé dans le swap compressé (zram), où Linux ne
# tue rien tant que le swap répond : sa machine est restée figée de 14 h 36 à 17 h 05, jusqu'à ce
# que le noyau tue enfin un `rustc` d'un gigaoctet. « Une seule grosse compilation à la fois » était
# respecté en apparence — une seule commande `cargo` en lance une par cœur.
#
#   bash outils/borne.sh <commande> [arguments…]
#   bash outils/borne.sh nix develop -c cargo test --workspace --no-fail-fast
#
# La commande vit dans sa propre part de mémoire (un « scope » de systemd), sans swap. Ce qui la
# dépasse meurt seul, en une seconde (code 137). Prouvé le 12/11 : 400 Mo demandés dans une borne de
# 200 Mo, tués en 0,6 s.
#
# **La part se mesure à ce que la machine laisse au lancement** (fiche 122), pas à sa taille : la
# plus petite de 60 % de sa mémoire et de `MemAvailable` — « an estimate of how much memory is
# available for starting new applications, without swapping » (la documentation du noyau,
# `proc.rst`) — moins un gigaoctet laissé à son bureau. Le 19/11 à 9 h 36, la borne fixe de 60 %
# (4,8 Go) a été prise alors que Brave, Glucose et VS Code tenaient déjà 3 Go : 1,7 Go de son
# bureau sont partis dans le swap, la mémoire était en pression 11 % du temps (`/proc/pressure`),
# et son écran est tombé à une image par seconde — la borne protégeait la machine d'une compilation
# seule, pas de lui. Le gigaoctet est un **nombre de ressenti** : de quoi ouvrir un onglet ou poser
# une image pendant que ça compile (valeurs essayées : 1 Go, le 19/11).
#
# Cargo n'y lance que ce que la borne peut tenir : un `rustc` du bureau monte à un peu plus d'un
# gigaoctet (le tué du 12/11 : 1,08 Go), donc une compilation par 1,5 Go de borne. Moins d'une : la
# commande ne part pas, et le dit — mieux vaut attendre que l'étouffer. Les épreuves gardent la
# moitié des cœurs. L'appelant qui les a déjà réglés garde ses réglages ; `nice` laisse la main à ce
# qu'il fait pendant ce temps.
set -u
[ $# -ge 1 ] || { echo "usage : borne.sh <commande> [arguments…]" >&2; exit 2; }
total_ko=$(awk '/MemTotal/ {print $2}' /proc/meminfo)
disponible_ko=$(awk '/MemAvailable/ {print $2}' /proc/meminfo)
une_compilation_ko=$(( 1536 * 1024 ))
borne_ko=$(( total_ko * 6 / 10 ))
laisse_ko=$(( disponible_ko - 1024 * 1024 ))
[ "$laisse_ko" -lt "$borne_ko" ] && borne_ko=$laisse_ko
if [ "$borne_ko" -lt "$une_compilation_ko" ]; then
    echo "borne.sh : la machine ne laisse que $(( disponible_ko / 1024 )) Mo sans faire swapper ce qui" \
         "tourne déjà ; il en faut $(( (une_compilation_ko + 1024 * 1024) / 1024 )). Rien n'est lancé." >&2
    exit 75
fi
compilations=$(( borne_ko / une_compilation_ko ))
export CARGO_BUILD_JOBS=${CARGO_BUILD_JOBS:-$compilations}
export RUST_TEST_THREADS=${RUST_TEST_THREADS:-$(( ($(nproc) + 1) / 2 ))}
echo "borne.sh : $(( borne_ko / 1024 )) Mo, $CARGO_BUILD_JOBS compilation(s) à la fois" >&2
exec systemd-run --user --scope --quiet -p MemoryMax="${borne_ko}K" -p MemorySwapMax=0 \
    nice -n 10 "$@"
