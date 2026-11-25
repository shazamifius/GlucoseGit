# 161 — Les photos HEIC : l'étude (25/11, compte b)

> INVENTAIRE § 3 : « le presse-papiers d'Android, les photos HEIC — à faire ». Une étude, pas de
> code : le choix engage une dépendance (`decisions/`), et rien ne s'essaie ici sans photo HEIC.

## 1. Où on en est

`image` 0.25 ne lit pas le HEIC (ni HEIF) : une photo d'iPhone glissée dans Glucose est refusée
comme illisible (le cache négatif, R-29). Le HEIC, c'est une image HEVC (H.265) dans un conteneur
HEIF ; l'iPhone l'écrit par défaut depuis 2017, beaucoup d'Android aussi.

## 2. Les chemins (recherche du 25/11)

* **Un décodeur en Rust pur** :
  * [`heifer`](https://docs.rs/crate/heifer/0.3.26) — HEIF et HEVC écrits de zéro en Rust sûr,
    **validé sur les fichiers de conformité officiels** ; il écrit aussi le HEIC, et documente le
    passage depuis `libheif-rs`. **crates.io, lu le 25/11** : 0.5.1 du 14/11/2026, MIT ou
    Apache-2.0, Rust 1.85, quatre caisses à lui (`heifer-hevc-dec`, `heifer-isobmff`, l'encodeur
    et `image`, facultatifs) et `thiserror` — vivant, et peu de dépendances.
  * [`heic`](https://docs.rs/crate/heic/0.2.0/source/README.md) (imazen) — sans une ligne
    `unsafe`, mais partiel : 118 fichiers de test sur 162, les seules tranches I du HEVC ; **sous
    AGPL-3.0 ou licence commerciale** (crates.io, 0.1.6) : Glucose est sous MIT, l'AGPL imposerait la sienne au tout — écarté pour cela seul.
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

## 5. HEIC-1, la mesure (25/11, compte b)

**Le banc** : `outils/heic/` (hors de l'espace de travail : il ne compile rien dans Glucose),
`heifer` 0.5.1 sans encodeur, `decode_srgb_rgba8` — ce que Glucose montrerait : huit bits, le
profil Display P3 de l'iPhone converti en sRGB. Un décodage par processus, `time -f %M`, sur ce
poste (Ryzen 5 7600X, six cœurs). **Les fichiers** : ceux des épreuves de
[`pillow_heif`](https://github.com/bigcat88/pillow_heif) (`benchmarks/`, `tests/images/`), dont
deux photos d'iPhone de 12 Mpx — jamais commités.

| fichier | taille | un fil | tous les cœurs | pic (un fil) |
|---|---|---|---|---|
| `pug.heic`, iPhone, 4032×3024 | 1,3 Mo | 423 ms | 235 ms | 100 Mo |
| `arrow.heic`, iPhone, 3024×4032 (tourné) | 0,8 Mo | 430-440 ms | 260 ms | 100 Mo |
| `spatial_photo.heic`, 2560×2560 | 1,3 Mo | 229 ms | 131 ms | 56 Mo |
| `image_large.heic`, 8000×6000 (48 Mpx) | 8,7 Mo | 1 269 ms | 248 ms | 387 Mo |

* **Le poids** : 627 Ko de plus au binaire (dépouillé : `banc-heic` 1 026 Ko, un programme vide
  399 Ko).
* **La vignette** de l'iPhone (l'élément `thmb`, 240×320) : **2 ms** — `decode_thumbnail`. Le
  reste des fichiers n'en a pas (0,03 ms pour le savoir).
* **La signature** (`check_file_type`) : quelques microsecondes.
* **La mémoire** : le pic, moins l'image rendue (4 octets par pixel), le fichier et le processus
  (~3 Mo), laisse **3,8 octets par pixel** de travail au décodeur (3,9 sur tous les cœurs). La
  borne de l'admission (`admission::pic` : la pyramide, plus **4 octets par pixel** de travail)
  le couvre — de justesse : 5 % de marge, à garder sous les yeux si `heifer` change.
* **Les couleurs, regardées** : `pug.heic` réduit huit fois (`apercu`) — des teintes naturelles,
  aucune jointure entre les tuiles de 512 px de l'iPhone ; `arrow.heic` sort debout (la rotation
  du fichier appliquée).
* **Les fichiers d'épreuve** : les 31 de `pillow_heif` (8, 10 et 12 bits, alpha, gris, ICC vide,
  identifiant invalide, paire stéréo) se décodent tous.
* **Une photo abîmée** : 60 copies de `pug.heic` (10 tronquées, 50 avec 1 à 32 octets changés) —
  58 refusées par une erreur, 2 décodées, **aucune panique, aucune au-delà de 20 s**. `heifer`
  refuse une image plutôt que d'en montrer une moitié : Glucose la mettrait au cache négatif
  (R-29), comme un JPEG illisible.

**Ce que cela dit** : la proposition tient. 235 à 440 ms pour une photo d'iPhone, c'est sur un
ouvrier (la charte : rien ne bloque l'image) — mais c'est trois à cinq fois un JPEG de même taille,
et sur une petite machine, plutôt une seconde. **D'où la vignette d'abord** : l'aperçu de l'atelier
prendrait le `thmb` (2 ms), l'image entière suivrait. **Les fils** : l'atelier décode déjà plusieurs
photos à la fois ; `max_threads = 1` évite d'en lancer six de plus par photo (le gain de tous les
cœurs ne vaut que pour une photo seule). Choix pour HEIC-2 : un fil, et la vignette.

**Le brevet** : ce qu'en dit le § 3 reste vrai et se dit dans `decisions/20`.

## 6. HEIC-2, la conception (25/11, compte b)

**Une seule porte : les crochets de `image`.** `image` 0.25.8 et plus (`image::hooks`) accepte des
décodeurs ajoutés : `register_format_detection_hook` (la signature `ftyp` + marque à l'octet 4,
masque sur la taille de la boîte) et `register_decoding_hook`. Enregistrés une fois au
démarrage, ils font lire le HEIC à **toutes** les portes qui passent par `with_guessed_format` —
l'ouvrier de l'atelier (`atelier/ouvrier.rs`), le dépôt d'une image venue d'une page
(`drop.rs`, `sceller_une_image`), le presse-papiers, la sélection Wayland — sans une ligne de
plus dans chacune. Un chemin propre au HEIC dans l'atelier seul laisserait le dépôt le refuser.

**Mais pas le décodeur de `heifer` (`heifer::image_crate::HeifDecoder`), pour trois raisons lues
dans son code :**
1. **Il décode dans son constructeur.** Or `sceller_une_image` ne demande que les dimensions
   (`into_dimensions`), et sur le fil de l'interface : 430 ms d'image bloquée par photo
   d'iPhone, contre la charte. Et dans l'ouvrier, le décodage aurait lieu dans `into_decoder`,
   **avant** l'admission de la mémoire (`porte.pendant`, MEMOIRE-3) : elle serait contournée.
2. **Il garde l'image sur seize bits** (`decode_with_options` : 8 octets par pixel), le double de
   `decode_rgba8` — au-delà de la borne `admission::pic`.
3. **Il rend les couleurs telles que codées** (Display P3 sur l'iPhone) et donne le profil à
   part ; Glucose ne gère pas les profils : les photos sortiraient ternes. `decode_srgb_rgba8`
   convertit.

**Le nôtre, paresseux** (~60 lignes, `renderer/photo/heic.rs`) : le constructeur lit les octets et
le conteneur seulement (`heifer_isobmff::HeifFile::parse`, quelques microsecondes) ; `dimensions`
se calcule de l'`ispe` de l'image première, puis de ses propriétés dans l'ordre (`clap` recadre,
`irot` impair échange largeur et hauteur ; `imir` ne change rien) — la formule du `clap` est celle
de `heifer::clean_aperture` (non publique : à recopier, six lignes, et une épreuve qui compare
avec le décodage). `color_type` : `Rgba8` ; `read_image` décode alors (`decode_srgb_rgba8`, un
fil) et **refuse par une erreur** si l'image ne fait pas les dimensions annoncées (jamais une
panique, jamais un tampon faux). La pointe : 7,8 octets par pixel pendant le décodage, puis
l'image de `heifer` (4) et la pyramide (5,3) ensemble à la copie — la borne `pic` (encodés + 9,3
octets par pixel) tient, sans marge ; à mesurer au pilote.

**Les épreuves** : un HEIC fabriqué à l'épreuve par l'encodeur de `heifer` (en dépendance de
développement seulement, `features = ["encoder"]` : le résolveur 2 ne l'ajoute pas au binaire) ;
la signature reconnue, les dimensions sans décodage, les pixels, un fichier tronqué refusé.
**À faire ensuite** : l'aperçu par la vignette `thmb` (2 ms) — l'atelier n'en a pas encore pour
les autres formats ; le nom et l'extension dans `menu_image.rs` (`guess_format` ne connaît pas
les crochets) ; ARCHITECTURE, à la ligne des dépendances.

## 7. HEIC-2, fait (25/11, compte b)

`renderer/photo/heic.rs`, comme le § 6 le dit ; `demarrage::lancer` l'enregistre (le bureau et
Android y passent tous deux). **Vérifié** : les dimensions lues sans décoder sont celles que
`heifer` rend en décodant, sur les 36 fichiers réels du banc (les deux photos d'iPhone, dont
`arrow.heic` debout par son `irot`, le 48 Mpx, les 31 d'épreuve) — une épreuve d'un jour, les
fichiers n'étant pas commitables. 8 épreuves dans le module (une fabrique ses HEIC par l'encodeur
de `heifer`, en dépendance de développement), une dans le menu (l'extension `.heic` d'une photo
enregistrée, plus `.bin`) ; 9 sabotages sur 9 (`outils/sabotages/heic-161.json`) ; la suite entière verte (3 719).
**Non gardés par une épreuve** : la conversion en sRGB (il faudrait un HEIC portant un profil
P3) ; l'appel au démarrage (le pilote le prouvera) ; `max_threads = 1`.
**Reste** : le pilote — une photo d'iPhone glissée dans un Glucose piloté, regardée debout et
nette, sa mémoire mesurée contre la borne `pic` ; l'aperçu par la vignette `thmb`.
