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
# La commande vit dans sa propre part de mémoire (un « scope » de systemd) : **60 % de la mémoire de
# la machine, sans swap**. Ce qui la dépasse meurt seul, en une seconde (code 137) ; le reste de la
# machine — son bureau, ses applications — ne s'en aperçoit pas. Prouvé le 12/11 : 400 Mo demandés
# dans une borne de 200 Mo, tués en 0,6 s.
#
# Cargo n'y lance que ce que la borne peut tenir : un `rustc` du bureau monte à un peu plus d'un
# gigaoctet (le tué du 12/11 : 1,08 Go), donc une compilation par 1,5 Go de borne — trois sur son
# VivoBook. Les épreuves gardent la moitié des cœurs. L'appelant qui les a déjà réglés garde ses
# réglages ; `nice` laisse la main à ce qu'il fait pendant ce temps.
set -u
[ $# -ge 1 ] || { echo "usage : borne.sh <commande> [arguments…]" >&2; exit 2; }
total_ko=$(awk '/MemTotal/ {print $2}' /proc/meminfo)
borne_ko=$(( total_ko * 6 / 10 ))
compilations=$(( borne_ko / (1536 * 1024) ))
[ "$compilations" -ge 1 ] || compilations=1
export CARGO_BUILD_JOBS=${CARGO_BUILD_JOBS:-$compilations}
export RUST_TEST_THREADS=${RUST_TEST_THREADS:-$(( ($(nproc) + 1) / 2 ))}
exec systemd-run --user --scope --quiet -p MemoryMax="${borne_ko}K" -p MemorySwapMax=0 \
    nice -n 10 "$@"
