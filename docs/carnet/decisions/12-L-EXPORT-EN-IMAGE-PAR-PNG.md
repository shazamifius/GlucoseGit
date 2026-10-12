# 12 — L'export en image, écrit par `png`

> Le bouton « Exporter » de la barre ne savait écrire que du Markdown et du SVG ; Glucose Tauri
> exportait aussi en PNG (fiche 03, 17.6), et c'est l'idée qu'il a donnée le 09/10 : les images
> d'une sélection composées en une, dans la disposition de l'auteur (fiche 59). Cette note dit
> la caisse prise **en direct**, `png`, qui était **déjà dans l'arbre** par `image` — même
> version, rien de nouveau à compiler.
>
> **Date** : 2026-10-12 · **Portée** : `persist/export/png.rs` · **Fiche** : 62.

---

## Ce qu'il faut

Un export ne connaît pas de taille à l'avance : un tableau de quelques images fait un million
de pixels, celui de son document principal treize, et un tableau large en ferait des
centaines. Tenir l'image entière en mémoire pour l'écrire d'un bloc mettrait la borne de
l'export dans la mémoire de la machine — et une borne choisie (Glucose Tauri : 8 000 pixels de
côté, 40 millions de pixels) est la constante arbitraire que la charte refuse.

L'image se dessine donc **par bandes**, chacune de la surface d'un écran, et chaque bande
s'écrit dans le fichier dès qu'elle est faite : la mémoire tient une bande, quelle que soit
l'image. Il faut pour cela un encodeur PNG **qui écrit au fil de l'eau**.

## Pourquoi `png`

* **Déjà là** : `image` l'emploie pour lire et écrire le PNG (`png` 0.18.1, `cargo tree -i`) ; la
  prendre en direct ne compile rien de plus.
* **Il écrit au fil de l'eau** : `Writer::stream_writer` reçoit les rangées à mesure, compresse
  et écrit — ce que l'encodeur d'`image` (`PngEncoder::write_image`) ne fait pas : il veut
  l'image entière.
* **En Rust pur**, sans rien à installer.

## Pourquoi pas autre chose

| | ce que ça donne | pourquoi non |
|---|---|---|
| `image::codecs::png::PngEncoder` | l'image entière d'un bloc | la mémoire de la machine devient la borne de l'export |
| `tiny_skia::Pixmap::encode_png` | pareil (il emploie `png` 0.17, lui) | pareil |
| écrire le PNG à la main | des blocs et un CRC, plus `zlib` | il faudrait réécrire la compression : un coût démesuré pour ce que `png` fait déjà |

## Ce que ça engage

* Une ligne dans `Cargo.toml`, la même version qu'`image` : si `image` change de `png`, celle-ci
  suit (sinon deux versions se compilent — `cargo tree -d` le dit).
* L'export s'écrit par le seul endroit qui pose un fichier à la place d'un autre
  (`persist::atomic`, SAVE-1) : un export interrompu ne laisse jamais une image à moitié écrite.
