# Le cerveau d'image — un projet d'avant Glucose, gardé pour la R&D

> Déposé le 15/10/2026 à sa demande (fiche 72) : *« un ancien projet perso qui peut énormément
> nous servir pour cet améliorateur d'image, qui a pour but d'être extrêmement précis et
> optimisé »*. **Pas pour maintenant** : la piste vit dans [SUITE](../../docs/SUITE.md) § 4.
> Ce dossier n'est compilé par rien dans Glucose.

## Ce que c'est

`main.cpp` (2 852 lignes, C++17, aucune dépendance hors `stb_image.h` pour lire une image au
PC) : une image lue **en flux**, pixel après pixel, sans jamais la stocker entière, analysée
en **arithmétique entière pure** — il a été écrit pour un Raspberry Pi Pico (RP2040, un
Cortex-M0+ sans unité de calcul flottant, 264 Ko de mémoire, aucune allocation) — puis
résumée en une **signature de texture** d'environ 80 octets, dont il tire un nuanceur HLSL
procédural qui imite la matière (bois, pierre, béton, rotin).

Les analyseurs, chacun en une passe et en mémoire bornée :

| analyseur | ce qu'il mesure |
|---|---|
| Welford (sommes exactes, König-Huygens) | moyenne et variance de la luminance, couleur dominante |
| GLCM 16 × 16 et descripteurs d'Haralick | contraste, homogénéité, entropie |
| **Pyramide de filtres de Gabor**, trois niveaux (λ = 4, 16, 64 px, décimation ×4) et quatre orientations | l'anisotropie : grain, fibres, strates, et leur direction |
| LBP (motifs binaires locaux) | la micro-structure : bords, coins, taches |
| Autocorrélation horizontale | la périodicité (tissus, grilles) |
| Tuilage 4 × 4 | la non-stationnarité (veines, salissures, dégradés) |

## Ce qu'il peut donner à Glucose

L'**améliorateur d'image** à l'import (sa parole du 15/10) : comprendre une image avant d'y
toucher — où est le grain, où sont les bords, quelle direction porte la matière — pour
débruiter, déflouter ou agrandir **sans inventer**. Les filtres de Gabor sont exactement le
banc d'orientations que demande un débruitage qui respecte les fibres et les contours.

Deux qualités du cerveau tombent juste avec la charte de Glucose : il travaille **en flux, en
mémoire bornée** (la mémoire du téléphone, fiche 66), et en **arithmétique entière
déterministe** (deux voies qui rendent les mêmes pixels au bit près).

## Ce qu'il faudra remettre en question avant de s'en servir

* Le noyau de Gabor 5 × 5 est petit pour λ = 4 px ; une pyramide plus fine, ou une
  décomposition orientable (*steerable pyramid*), serait à comparer.
* Il analyse ; il ne modifie pas l'image. L'améliorateur est à concevoir, après une recherche
  sur l'état de l'art (débruitage guidé, super-résolution sans réseau de neurones).
* Recréer, ne pas traduire (R2) : en Rust, sur les deux voies (processeur et carte graphique).
