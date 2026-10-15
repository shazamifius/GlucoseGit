# 13 — Les glyphes du SVG par `ttf-parser`, et `resvg` pour l'éprouver

> L'export SVG (P5) écrivait son texte en `<text>`, qui demande sa police au lecteur : Inter
> absente, le lecteur la remplaçait, et chaque passage en gras laissait un trou (fiche 70 § 1).
> Cette note dit les deux caisses que la fiche 70 a prises, et qu'elle n'avait pas défendues :
> **`ttf-parser`** dans l'application, **déjà dans l'arbre** par `fontdue` (même version, rien de
> nouveau à compiler) ; **`resvg`** dans les épreuves seulement, comme oracle.
>
> **Date** : 2026-10-15 · **Portée** : `renderer/svg/glyphes.rs`, `renderer/math/svg.rs` ;
> `persist/export/svg/tests.rs` · **Fiches** : 70, 71.

---

## `ttf-parser` — les contours des glyphes

### Ce qu'il faut

Un glyphe en tracé est le contour même que la police décrit : ses courbes, dans les unités de
la police. `fontdue` ne les rend pas — il rastérise, et ses contours restent privés. Il faut
lire les tables `glyf` (et `CFF` pour une police qui en porte) d'Inter, des fontes de KaTeX et de
Noto Emoji.

### Pourquoi `ttf-parser`

* **Déjà là** : `fontdue` 0.9 lit ses polices par `ttf-parser` 0.25.1 (`Cargo.lock`). La prendre en
  direct, à la même version et **sans ses fonctions par défaut**, ne compile rien de plus.
* **Elle donne le contour par un rappel** (`OutlineBuilder` : `move_to`, `line_to`, `quad_to`,
  `curve_to`, `close`) : le chemin SVG s'écrit au fil de la lecture, sans structure
  intermédiaire.
* **En Rust pur, sans allocation ni `unsafe`** dans la lecture.

### Pourquoi pas autre chose

| | ce que ça donne | pourquoi non |
|---|---|---|
| `<text>` et la police nommée | le lecteur choisit la police | Inter n'est installée presque nulle part : des lignes coupées à la mesure d'Inter débordent |
| une `@font-face` embarquée (tldraw, Excalidraw) | la police dans le fichier | `resvg`, Inkscape et Illustrator l'ignorent ; seuls les navigateurs la lisent |
| lire `glyf` à la main | les contours | `fontdue` dépend déjà de `ttf-parser` : la réécrire ferait deux lecteurs de la même table |

Le mode retenu est celui de matplotlib (`svg.fonttype = 'path'`, son défaut) et l'« Outline text »
de Figma : chaque glyphe défini une fois dans `<defs>`, posé par un `<use>`, et chaque ligne garde
ses mots dans un `aria-label`.

### Ce que ça engage

* Une ligne dans `Cargo.toml`, **la version de `fontdue`** : si `fontdue` change de
  `ttf-parser`, celle-ci suit (sinon deux versions se compilent — `cargo tree -d` le dit).
* Le jour où la charte réécrit la lecture des polices (le zéro dépendance, à terme : `VISION.md`),
  c'est un seul lecteur qui remplacera les deux usages.

## `resvg` — l'oracle de l'export SVG

### Ce qu'il faut

La fiche 69 n'avait jamais regardé son SVG : aucune épreuve ne pouvait dire qu'il montrait autre
chose que le canevas. Il faut un lecteur **indépendant** qui rende le SVG en pixels, pour le
superposer à l'export PNG du même tableau — sans fenêtre (Brave sans tête ne rend pas la main).

### Pourquoi `resvg`

* **Un lecteur complet et indépendant** : il ne partage rien de notre code ; ce qu'il lit d'un
  SVG, d'autres lecteurs le lisent.
* **Sur notre `tiny-skia`** (0.11) : la 0.46 est la seule version qui n'en tire pas une seconde
  (la 0.45 tirait deux versions de `gif` et `zune-jpeg`, la 0.48 un second `tiny-skia`).
* **Sans ses fonctions par défaut** : ni texte, ni polices du système — nos glyphes sont des
  tracés ; ses décodeurs de photos (`raster-images`) sont ceux d'`image`.

### Ce que ça coûte

**Treize caisses**, toutes **dans les épreuves seulement** (`[dev-dependencies]`, jamais dans
l'application) : `resvg`, `usvg`, `roxmltree`, `simplecss`, `svgtypes`, `kurbo`, `euclid`,
`polycool`, `data-url`, `imagesize`, `rgb`, `xmlwriter`, `pico-args` (`Cargo.lock`, relevé le
15/10). C'est la règle de l'oracle déjà suivie pour `automerge` (`decisions/03`) : une
bibliothèque de référence qui juge notre code, et que l'utilisateur ne télécharge jamais.

### Ce qu'il a déjà trouvé

* À la fiche 70 : les polices remplacées, les formules en source LaTeX, le dossier absent, l'ordre
  des lueurs.
* À la fiche 71 : **la région des flous**. Chaque lueur déclarait une région de ±100 000 unités,
  et un lecteur floutait un carré de 200 000 de côté par lueur — **28,3 s** pour `resvg` sur la
  vitrine (l'épreuve prenait 80 s dans la suite). La région est désormais la boîte élargie de la
  portée du flou (`portee_du_flou`, HALO-2) : **62 ms**, et la superposition est la même au
  centième près (forme 26,62, teinte 6,31).
