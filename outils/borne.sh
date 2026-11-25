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
# dépasse meurt seul, en une seconde (code 137, ou 143 pour la commande entière). Prouvé le 12/11 :
# 400 Mo demandés dans une borne de 200 Mo, tués en 0,6 s. **Un `rustc` tué peut laisser son crate
# à moitié écrit** : la compilation suivante le réutilise et l'édition de liens tombe sur des
# « undefined reference » (le 19/11) — `cargo clean -p <crate>`, puis recompiler.
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
# Cargo n'y lance que ce que la borne peut tenir : **une compilation par 2,5 Go de borne**. Le
# `rustc` qui compile les épreuves du bureau monte à 2,4 Go (tué par sa borne le 19/11 à 10 h 46,
# deux comme lui dans 4 Go : `anon-rss:2417076kB` au journal du noyau) ; le 12/11, un `rustc` du
# bureau en tenait 1,08. Moins d'une : la commande ne part pas, et le dit — mieux vaut attendre
# que l'étouffer. Les épreuves gardent la moitié des cœurs. L'appelant qui les a déjà réglés garde
# ses réglages ; `nice` laisse la main à ce qu'il fait pendant ce temps.
#
# **Le disque aussi** (fiche 171). Cargo ne retire jamais un ancien exécutable : chaque suite en
# ajoute un exemplaire de chaque exemple et de chaque épreuve. Le 25/11, `target/debug/examples`
# pesait 101 Go (une trentaine d'exemplaires de chaque exemple), `target/` 160 Go, et le disque s'est
# rempli pendant une suite (« No space left on device » à l'édition de liens) — or c'est aussi le
# disque où son Glucose écrit. Avant chaque commande, **seul le plus récent de chaque exécutable
# reste**, et un plus ancien redevenu utile se recompile. « Le même », c'est la même unité de Cargo,
# lue dans son dossier d'empreinte (`.fingerprint/<caisse>-<hachage>/<unité>`, le hachage de
# l'exécutable) — jamais le nom du fichier : les épreuves d'une bibliothèque, son programme et les
# épreuves de son programme portent tous `glucose_desktop-<hachage>`, et n'en garder qu'un en
# recompilerait deux à chaque suite. Ce qu'aucune empreinte ne nomme ne se touche pas, ni les
# bibliothèques : deux exemplaires d'une même caisse y vivent légitimement.
set -u
[ $# -ge 1 ] || { echo "usage : borne.sh <commande> [arguments…]" >&2; exit 2; }
garder_le_plus_recent() {
    {
        find "$1/.fingerprint" -mindepth 2 -maxdepth 2 -type f -regextype posix-extended \
            -regex '.*/[^/]+-[0-9a-f]{16}/(bin|test|example|bench)-[^./]+' -printf 'U %h/%f\n'
        find "$1/deps" "$1/examples" -maxdepth 1 -type f -executable -regextype posix-extended \
            -regex '.*-[0-9a-f]{16}' -printf 'X %T@ %p\n' 2>/dev/null | sort -k2 -rn
    } | awk '$1 == "U" { n = split($2, a, "/"); d = a[n - 1]
                         unite[substr(d, length(d) - 15)] = substr(d, 1, length(d) - 17) "/" a[n]; next }
             { h = substr($3, length($3) - 15); if ((h in unite) && vu[unite[h]]++) print $3 }' \
        | xargs -r -d '\n' rm -f --
}
for profil in "$(cd "$(dirname "$0")/.." && pwd)"/target/*/; do
    [ -d "$profil.fingerprint" ] && garder_le_plus_recent "${profil%/}"
done
total_ko=$(awk '/MemTotal/ {print $2}' /proc/meminfo)
disponible_ko=$(awk '/MemAvailable/ {print $2}' /proc/meminfo)
une_compilation_ko=$(( 2560 * 1024 ))
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
