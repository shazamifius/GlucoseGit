# 161 — Les photos HEIC : l'étude (25/11, compte b)

> INVENTAIRE § 3 : « le presse-papiers d'Android, les photos HEIC — à faire ». Une étude, pas de
> code : le choix engage une dépendance (`decisions/`), et rien ne s'essaie ici sans photo HEIC.

## 1. Où on en est

`image` 0.25 ne lit pas le HEIC (ni HEIF) : une photo d'iPhone glissée dans Glucose est refusée
comme illisible (le cache négatif, R-29). Le HEIC, c'est une image HEVC (H.265) dans un conteneur
HEIF ; l'iPhone l'écrit par défaut depuis 2017, beaucoup d'Android aussi.

## 2. Les chemins (recherche du 25/11)

* **Un décodeur en Rust pur** :
  * [`heifer`](https://docs.rs/crate/heifer/0.3.26) (0.3.x) — HEIF et HEVC écrits de zéro en Rust
    sûr, **validé sur les fichiers de conformité officiels** ; il écrit aussi le HEIC, et documente
    le passage depuis `libheif-rs`.
  * [`heic`](https://docs.rs/crate/heic/0.2.0/source/README.md) (imazen) — sans une ligne
    `unsafe`, mais partiel : 118 fichiers de test sur 162, les seules tranches I du HEVC.
  * [`heic-decoder`](https://github.com/ente-io/heic-decoder) (ente) — HEIF, HEIC, AVIF, branché
    sur `image`.
* **`libheif` en C** ([`heif-rs`](https://docs.rs/crate/heif-rs/26.6.0), lié statiquement avec
  libde265) : le plus complet, mais du C à compiler pour quatre systèmes et Android, et sa
  dernière version ne se construisait pas sur docs.rs.
* **Les décodeurs des systèmes** : le Mac (ImageIO, depuis 10.13) et Android (`ImageDecoder`,
  Android 9 et plus) le lisent d'office ; Windows seulement avec les extensions « HEIF » et
  « HEVC » du Store (la seconde est payante) ; Linux, rien d'office. **Une machine exclue**, contre
  la charte.

## 3. La proposition

**`heifer`, derrière la même porte que les autres formats** (le décodeur de l'atelier, l'aperçu,
la signature `ftypheic`/`ftypmif1` à l'octet 4) : une seule voie pour toutes les machines, sans C,
qui se compile pour le téléphone comme le reste. **À mesurer avant de choisir** (aucun chiffre
ici n'est vérifié) : le poids qu'il ajoute au binaire, le temps d'une photo d'iPhone de 12
mégapixels (la charte : rien ne bloque l'image ; le décodage est sur un ouvrier, mais un HEVC
logiciel peut prendre des centaines de millisecondes), sa mémoire de pointe (l'admission doit la
connaître : `admission::pic`), et les fichiers de conformité qu'il refuse.

**Le brevet** : le HEVC est couvert par des brevets ; un décodeur logiciel dans une application
libre et gratuite est l'usage courant (VLC, libde265), mais c'est à dire, pas à taire.

## 4. Les étapes

1. **HEIC-1** : la mesure — `heifer` dans un banc hors de l'application (`examples/`), sur des
   HEIC d'iPhone (à trouver : des fichiers d'exemple libres, ou les siens), le temps, la mémoire,
   le poids ; puis la note `decisions/`.
2. **HEIC-2** : la signature, le décodeur de l'atelier, l'aperçu — le format du document ne change
   pas (les octets d'origine sont gardés, comme pour un GIF).
