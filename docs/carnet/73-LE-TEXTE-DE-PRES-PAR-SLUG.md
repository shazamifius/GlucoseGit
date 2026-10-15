# 73 — Le texte de près, dessiné par la carte depuis ses formes (Slug)

> Session du 15/10/2026, après la fiche 72. Son message d'ouverture : la beta 4 publiée par lui ;
> **plus de 24 000 nœuds au téléphone, « ça n'a pas trop lagé, c'était plutôt cool »** ; au PC,
> *« tout fonctionne merveilleusement bien »*. Il a demandé un tableau de ce qui reste, puis de
> continuer. Sa télémétrie n'avait encore rien reçu de la beta 4 (les sessions partent au
> lancement suivant) : P0 attend ses lignes. Le chantier pris : **P8, le texte net de près**, le
> seul défaut de rendu qu'il ait dit « extrêmement important ».

## 1. Ce qu'il voyait, et pourquoi

De près, un nœud de texte devenait flou, en gros pixels, et ne redevenait net qu'au bout d'un
moment (fiche 72 § 2). La carte graphique le posait comme une **image** que le processeur avait
peinte (COMPOSANT-1). Plus grand que l'écran, il se découpait en carrés de 256 pixels repeints un à
un dans le temps libre de chaque image (DE-PRES-1), l'image entière agrandie dix ou vingt fois
posée en attendant. GRAND-1 avait rendu chaque carré moins cher (66 → 19 ms à ×32) ; il restait
**un repeint par zoom**, donc un flou par zoom.

## 2. La recherche

* **Slug** (Eric Lengyel) : *GPU-Centered Font Rendering Directly from Glyph Outlines* (JCGT
  2017), *A Decade of Slug* (blog Terathon, mars 2026). Pour chaque pixel, un rayon horizontal et
  un rayon vertical ; chaque courbe du second degré croisée ajoute ou retire sa part du pixel ;
  les deux rayons se mêlent selon leur proximité au croisement. Des **bandes** limitent les
  courbes lues, **triées** pour s'arrêter tôt. L'éligibilité des racines se lit sur les signes des
  trois points (une table de seize bits, `0x2E74`) : aucune tolérance, aucun cas particulier.
  Depuis dix ans : la division des bandes et le suréchantillonnage adaptatif retirés, la
  **dilatation dynamique** ajoutée (le quad grossi d'un demi-pixel à l'écran).
* **Le brevet** (US 10 373 352) est versé au **domaine public depuis le 17/03/2026** ; les
  nuanceurs de référence (`EricLengyel/Slug` : `SlugPixelShader.hlsl`, `SlugVertexShader.hlsl`)
  sont sous licence **MIT** (le nuanceur de sommets MIT ou Apache-2.0) — un crédit est demandé :
  `present/formes_gpu/LICENSE-Slug.txt`, et le README.
* Ce que Slug **ne fait pas** : les cubiques. Les polices TrueType n'en ont pas ; les CFF si (Inter
  Italic, Inter SemiBold Italic). `cu2qu`, l'outil de Google Fonts, convertit à une unité près.
* Aucune implémentation de Slug pour `wgpu` ou en Rust n'a été trouvée : celle-ci est écrite ici.

Sources : Hackaday, « Slug Algorithm For On-GPU Rendering Of Fonts With Bézier Curves Now In
Public Domain » ; terathon.com/blog/decade-slug.html ; github.com/EricLengyel/Slug (README,
`SlugPixelShader.hlsl`, `SlugVertexShader.hlsl`) ; sluglibrary.com ; metalbyexample.com/slug ;
terathon.com/i3d2018_lengyel.pdf.

## 3. La conception — et ce qu'elle a retiré

**Quand** : exactement là où le découpage commençait — un nœud **plus grand que l'écran**. En deçà,
sa texture reste : rendue une fois, posée pour quatre nombres, ce qui vaut mieux que mille glyphes
quand on voit tout un tableau de loin. **Aucune frontière n'est ajoutée** : c'est la même.

**Une seule description** : la carte dessine ce que l'export SVG écrit (`renderer/vectoriel.rs`,
le trait `Vectoriel`) — l'ancienne « sœur SVG » d'un nœud de texte devient **le** dessin vectoriel
(`card/vecteurs.rs`), avec deux destinataires : `Svg`, et le **relevé** (`vectoriel/releve.rs`)
qui en tire des primitives pour la carte. Elle s'est complétée en passant : le fond des passages
qui brillent, la sélection pendant l'édition, les encres d'une formule qu'on écrit ; la barre, le
filet et la plaque **nets sur la grille des pixels** (le SVG les écrivait flous — la règle de
`fill_crisp`, comme la fiche 71 l'avait fait pour la `Surface`). L'export reste identique (ses
douze épreuves, superposition comprise).

**Deux lois, une passe** (`present/formes_gpu`) : les **arrondis** — brume, plaques, surlignages,
filets, puces, liserés — par la **loi des membranes** (la même que le processeur : une seule copie
WGSL, `present/arrondi.wgsl`, que les membranes et les formes lisent) ; les **courbes** — lettres,
formes de KaTeX — par **Slug**. Un seul tampon d'instances : la carte les compose dans leur ordre,
la plaque sous le texte, le texte sous son trait.

**Le noyau** (`glucose-core::courbes`, sans dépendance — une brique réutilisable seule) :

* `Contours` bâtit les contours fermés en quadratiques ; **CUBIQUE-1** : une cubique se coupe en
  `n` morceaux égaux, chacun remplacé par sa quadratique du milieu ; l'écart est borné par
  `√3/36 · |Δ³| / n³` — `n` **se calcule**. La tolérance n'est pas choisie : la moitié de l'unité
  de la grille où la forme a été dessinée (l'unité de la police ; un demi-niveau sur 255 pour une
  forme posée à l'écran ; une demi-unité du tracé pour une forme de KaTeX).
* `emballage` range courbes et bandes comme la carte les lit. **BANDES-1** : autant de bandes que
  de courbes — un pixel lit en moyenne au plus deux courbes de plus que celles que son rayon
  croise vraiment ; aucun nombre. **BANDES-2** : une courbe est dans **chaque** bande que la carte
  peut calculer pour elle — par la formule même du nuanceur, en `f32`, par les deux arithmétiques
  (multiplication et addition séparées ou fondues) : la marge de Slug (« environ 1/1024 d'em »)
  sans le nombre.
* `couverture` est **la loi**, en Rust, opération pour opération ; le nuanceur la recopie.

**Ce qui a disparu** : le découpage en carrés (`composants/decoupe.rs`, son repli posé, ses
épreuves). La texture au plus haut palier qui tient dans l'écran se **prépare** encore, dans le
temps qui reste, sans jamais se poser : c'est elle qui se pose à l'instant où, dézoomé, le nœud
retient dans l'écran — au lieu d'un trou, ou d'une image de 20 ms qui gèle. Le cadre d'une photo
en chemin passe aussi en formes : il n'y a plus qu'une façon de dessiner de près.

## 4. Ce que la vérification a démenti

* **Une lettre de 2 400 pixels s'écartait de la loi de dix-huit niveaux**, sur quatre pixels, là
  où le rayon frôle le haut d'un « O ». Le point de la forme sous chaque pixel était **interpolé**
  depuis les coins d'un quad géant ; la loi l'**inversait**. Calculé par la carte comme par la loi
  — l'inverse de la matrice appliqué au centre du pixel —, l'écart tombe à **un niveau**, et le
  « g » et le « @ » sortent identiques au bit près. Un zoom ×256 ne dépend plus de la précision de
  l'interpolation.
* **L'épreuve des deux voies demandait l'égalité au bit près, et ce n'est plus vrai — ni faux.**
  De près, le processeur pose une lettre au quart de pixel (GLYPH-1) et la filtre par son aire ;
  Slug la pose exactement et filtre le long de deux rayons. Mesuré : tous les écarts sur un bord de
  lettre, écart signé moyen **nul** (−0,05) — ni plus gras ni plus maigre —, indiscernable à l'œil
  (captures comparées). Relever le seuil aurait été une rustine ; la loi écrite est **SLUG-1** : un
  écart de plus de trois niveaux ne tombe que sur un pixel que l'une des voies couvre en partie.
  Une carte absente, une lettre manquante ou décalée d'un pixel tombent.
* **Git Bash refuse un heredoc qui porte des apostrophes** (« unexpected EOF ») : les scripts de
  la session ont été écrits dans le scratchpad, puis `python -I` — METHODE § 5 le disait.

## 5. Mesures

`bench_de_pres` (réécrit : il mesurait des carrés qui n'existent plus), son écran 2 560 × 1 600,
une carte qui le couvre, à l'arrêt :

| zoom | avant (fiche 72) | maintenant |
|---|---|---|
| ×16 | 21,3 ms de repeint, flou en attendant | relevé 1,0 ms, carte 1,1 ms par image, net |
| ×32 | 19,7 ms, flou | 1,0 ms, 1,0 ms, net |
| ×50 | 18,7 ms, flou | 1,1 ms, 1,0 ms, net |
| ×256 | — | 1,1 ms, 1,1 ms, net |

Le temps de la carte compte l'envoi et l'attente de sa fin : c'est un plafond. La première image
d'une échelle remplit l'atlas (1,8 à 2,6 ms). **Non mesuré** : son téléphone.

## 6. Ce qui reste

* **À son écran** : de près, un texte, un titre de membrane, une formule — nets dès la première
  image, à tout zoom jusqu'à ×256 ; zoomer et dézoomer sans trou ni à-coup. Au téléphone de même.
* Le **processeur** dessine toujours les lettres au quart de pixel : la voie processeur reste
  telle qu'elle était (et nette à l'arrêt). Les deux voies ne partagent pas encore la même loi des
  lettres — Slug au processeur coûterait trop par pixel ; SLUG-1 dit ce qui les sépare.
* Les **titres des membranes**, des zones et des cases restent peints au processeur, à l'échelle
  exacte, dans la couche du dessous : nets, mais repeints à chaque image où ils bougent.
* Un nœud **immense** (des milliers de mots) relève toute sa mise en page à chaque image de près ;
  les lettres hors de l'écran ne partent pas, mais leur mise en page se calcule.
