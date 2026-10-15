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
* **« Une perte de chiffres dans la formule des racines »** : réfuté par la mesure (la forme
  stable n'a rien changé). La cause : WGSL tolère deux ulps et demi sur une division, et Slug
  décide par le **signe** d'un point posé pile sur un trait horizontal — un ulp y faisait sauter
  la couverture de cinquante-deux niveaux. **RACINES-1** : la carte reçoit l'inverse de la
  matrice, calculé par le processeur, et ne divise plus ; des deux côtés le point de la forme se
  calcule ensuite par les mêmes multiplications, correctement arrondies.
* **La loi de Slug n'est pas exacte aux coins** : sur un pixel de coin, la vérité vaut 26 et la
  loi 52. Les deux voies sont d'accord entre elles ; c'est une limite de la loi (deux rayons ne
  voient pas un coin comme une aire), pas une faute de l'une d'elles.

## 4 bis. L'épreuve du passage, qui tombait (la suite de la session, le même soir)

`test_slug_1_de_pres_un_passage_et_une_selection_se_dessinent_pareil` tombait : 17 547 canaux
« hors du bord des lettres » pour le passage qui brille. Cause inconnue en fin de session ;
trois hypothèses sur la table (le fond et le liseré par deux lois, une géométrie qui diffère, la
mise en page ouverte). **Aucune n'a été supposée** : les deux images enregistrées, comparées à
l'œil (identiques), puis par carte des écarts.

* **La géométrie était identique** : le liseré commence et finit sur les mêmes pixels des deux
  côtés, l'intérieur égal à un niveau ; la mise en page ouverte aussi (les lettres aux mêmes
  places). Deux hypothèses sur trois écartées par la mesure.
* **Les pixels comptés étaient tous des bords**, mais des bords que la règle ne reconnaissait
  pas : SLUG-1 appelait « couvert en partie » un mélange **du fond de la carte et de l'encre
  blanche**, lus sur toute l'image. Or sous un passage, les lettres sont rouges sur un fond
  teinté et la lueur rosit tout autour ; le liseré n'est pas une lettre ; sous une sélection,
  les lettres sont sur le bleu du surlignage. L'épreuve ne mesurait pas ce qu'elle disait.
* **Et il restait une vraie différence**, petite : le bord du fond, dans le creux de l'anneau,
  valait 66 au processeur là où le fond vaut 63 et le dehors 59 — **plus clair que les deux**. Le
  processeur remplissait le cadre par `tiny-skia` (anticrénelage par sous-échantillonnage, le
  liseré en anneau pair-impair), la lueur se découpe par la loi exacte : leurs deux couvertures
  du même bord se chevauchaient. Sur la carte, tout passait par la loi des membranes. **Deux lois
  pour un même bord** : c'était la troisième hypothèse, à un niveau de détail près.

Deux corrections, chacune à sa cause — aucun seuil n'a bougé :

* **CADRE-1** (`renderer/passages.rs`) : le cadre d'un passage **est une membrane** — un fond à
  20 % et un bord à 45 %, d'une seule teinte —, et `membrane_forme::peindre` la peint au
  processeur comme il peint déjà la brume. Le dessin vectoriel se tire de **la même** membrane
  (`membrane_du_cadre`) : la géométrie du cadre ne se calcule plus qu'à un endroit, et
  `Vectoriel::anneau` reçoit désormais le bord tel que la membrane le décrit (le rectangle du
  milieu du trait, sa demi-largeur). L'anneau pair-impair a disparu. Mesuré : le liseré à un
  niveau près sur les deux voies (76/76, 117/116 ; avant 75/76, 118/116), le bord du fond égal.
* **SLUG-1, lu localement** (`tests/voies_suite.rs`, `couvert_en_partie`) : un pixel est
  couvert en partie s'il est **un mélange de deux de ses voisins opposés** (horizontaux,
  verticaux ou diagonaux), à trois niveaux près, sans être l'un ni l'autre — quels que soient le
  fond et l'encre. Après correction : trois canaux hors des bords sur 1 920 000, un pixel du bord
  droit de l'image (une lettre coupée, sans voisin à droite : on ne peut pas juger, et c'est
  honnête qu'il compte). La garde « segment trop court » qu'un premier jet portait a été retirée
  avant de saboter : la marge l'impose déjà, et une longueur nulle donne `NaN`, que les
  comparaisons rejettent — une garde redondante est aveugle (fiche 57).

La recherche le confirme : le comparateur de Skia Gold (Chromium) masque les pixels de bord,
repérés par un filtre de Sobel sur l'image attendue, avant de comparer le reste à une tolérance,
et a un réglage pour **ignorer les pixels du bord de l'image** — notre cas du pixel 799. La règle
d'ici est plus étroite : un bord n'excuse un écart que si le pixel est un mélange de ses voisins.
Les reftests du web (`<meta name=fuzzy>`, WPT) bornent seulement l'écart maximal et le nombre de
pixels : c'est la règle du reste de cette suite.

Sources : chromium.googlesource.com, `ui/base/test/skia_gold_matching_algorithm.h` ;
skia.googlesource.com, `buildbot/gold-client/go/imgmatching/sobel/sobel.go` ;
web-platform-tests.org, *Reftests* (« fuzzy matching »).

**Une épreuve écrite pour ce qui est partagé** : depuis CADRE-1, les deux voies tirent le cadre
de la même membrane — leur comparaison ne peut plus voir une faute dans sa géométrie.
`test_cadre_1_le_cadre_est_celui_de_tauri` le peint seul, à ×4 (des bords sur des pixels
entiers), et lit les valeurs exactes contre les nombres de Tauri : un fond à 51/255 débordant de
sa marge (3 et 1), un liseré à 115/255 épais de 1,5, décalé de 2. Même raisonnement pour
RACINES-1 : l'épreuve de la passe compare la carte à la loi, et **toutes deux** appellent
`inverse()` ; ce que RACINES-1 garde, c'est que la carte ne divise pas elle-même — le sabotage
porte donc sur le nuanceur, et une faute dans `inverse()` se voit contre le processeur
(SLUG-1).

## 4 ter. Les sabotages — cinq aveugles sur trente-six

`outils/sabotages/p8.json`, lancé pour la première fois (chaque commande d'abord sur le code
intact, pour vérifier qu'elle exécute l'épreuve visée) : **31 tombent, 5 passent**. Aucun des
cinq n'était un défaut du rendu ; tous disaient qu'une épreuve ne regardait pas ce qu'elle
croyait garder.

| sabotage aveugle | pourquoi | ce qui a changé |
|---|---|---|
| le rayon vertical dans le mauvais sens (noyau) | son signe ne compte qu'où **les deux** rayons ont du poids et voient deux couvertures différentes : un coin. Ailleurs, le plancher `min(\|x\|, \|y\|)` masque la faute. Aucune épreuve ne regardait un coin | `test_pres_d_un_coin_les_deux_rayons_se_melent` : un pixel à 0,2 et 0,1 pixel d'un coin, la moyenne pondérée calculée à la main (0,643 ; un signe faux donne 0,6, la vérité serait 0,42 — la loi n'est pas exacte aux coins, l'épreuve dit ce qu'elle calcule) |
| la rogne sur un seul axe (carte) | les bords verticaux de sa rogne étaient entiers (10 et 60) : la part verticale d'un pixel de bord valait toujours 0 ou 1 | la rogne fractionnaire sur les deux axes (10,3 et 60,6), comme une formule rognée par sa boîte |
| l'anneau d'un seul côté (carte) | **une garde redondante** : la couverture d'un trait est paire en `d`, au bit près (`−max(a, b) = min(−a, −b)`, la négation exacte) — le `abs` ne pouvait rien changer | le `abs` retiré du nuanceur et de la loi de l'épreuve ; la parité écrite dans le noyau et éprouvée au bit près (`test_la_couverture_d_un_trait_est_paire_au_bit_pres`) ; deux sabotages qui la cassent vraiment |
| l'atlas resserré ne renvoie rien | l'épreuve affirmait `envoyees == (0, 0)` après le resserrement — mais cet atlas n'avait jamais rien envoyé : vrai d'avance. Et le resserrement lit l'image **d'avant** : deux images ne le déclenchent pas | l'atlas « envoie » avant de se resserrer ; et `test_un_atlas_resserre_renvoie_tout`, à la carte, sur **trois** images (huit lettres, puis le g seul, deux fois) : la carte doit lire le g à sa nouvelle place |
| SLUG-1 : une lettre décalée d'un pixel | toute loi qui excuse un écart « au bord d'une forme » excuse un bord **déplacé** : les pixels qu'il déplace sont des pixels de bord. L'ancienne règle comme la nouvelle | **une loi de place** : le déplacement d'ensemble entre les deux voies (moindres carrés de Lucas et Kanade, `δ = −Σ(b − a)·∇a / Σ∇a²`) borné par **un huitième de pixel** — la borne du quart de pixel de GLYPH-1 (`SUBPIXEL_PHASES`), pas un nombre choisi. Mesuré : au pire 0,047 pixel dans les quatre scènes ; une carte décalée d'un pixel en donne 1,1 |

La reprise rejoue ces cinq-là et les deux sabotages de la parité : **six sur six tombent**. Le
sabotage du `abs` disparu est retiré du fichier ; `p8.json` en porte trente-sept, qui tombent
tous, et chacun retrouve son texte dans le code (vérifié après `cargo fmt`). Puis : clippy
strict à zéro sous Windows (Rust 1.99.1, mis à jour avant) et pour Android ; la suite entière,
**2 590 épreuves**, toutes vertes.

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
  les lettres hors de l'écran ne partent pas, mais leur mise en page se calcule — mesuré au § 7.

## 7. Sa conception, remise en question (sa demande)

**Un nœud immense — mesuré, et c'est un défaut.** `bench_de_pres` (sa nouvelle partie), à ×16,
l'écran ne montrant que le début du nœud :

| mots | relevé, première image | puis, par image | dont une mise en page | la carte |
|---|---|---|---|---|
| 44 | 1,0 ms | 0,46 ms | 0,06 ms | 1,03 ms |
| 440 | 1,4 | 0,71 | 0,10 | 1,05 |
| 4 400 | 4,5 | 3,56 | 0,63 | 1,05 |
| 17 600 | 12,3 | **10,5** | 3,3 | 1,04 |

La carte ne bouge pas (elle ne reçoit que ce que l'écran montre) ; le **processeur** paie la
longueur du nœud à chaque image : la mise en page refaite, puis chaque ligne parcourue lettre
par lettre avant que chaque lettre hors de l'écran soit écartée une à une. À 17 600 mots, le
relevé seul dépasse les 10 ms d'une image ; à 4 400, il prend presque toute l'image d'un écran
à 240 Hz. Un article de Wikipédia vu de près, c'est l'objectif 1 de sa vision. Deux remèdes, à
concevoir proprement (c'est le chantier suivant) : **la mise en page ne dépend pas du zoom** —
elle se calcule en unités du monde (CARD-1) et peut se garder d'une image à l'autre ; et **une
ligne qui ne peut pas toucher l'écran ne se parcourt pas** — ce qui demande une borne exacte de
ce qu'une ligne dessine hors de sa boîte : une formule réserve plusieurs lignes et se dessine
depuis la première, par-dessus les suivantes.

**La frontière « plus grand que l'écran ».** Elle n'ajoute aucun nombre, et c'est sa force. Mais
elle ne retire pas tout le flou : **sous** elle, un nœud reste une texture, et pendant un zoom il
change de palier dyadique ; tant que le palier suivant n'est pas peint, c'est la texture du palier
d'avant, agrandie jusqu'à deux fois, qui se pose. Un nœud de la moitié de l'écran qu'on zoome est
donc encore flou un instant. La loi qui découle de la charte (« un modèle de coût qui prédit ») :
**les formes sont le repli**, à la place de la texture agrandie — posées tant que la texture du
bon palier n'est pas prête, la texture ensuite (elle coûte quatre nombres à l'arrêt). Il n'y
aurait plus de frontière du tout : la texture quand elle est juste, les formes sinon. Le prix
inconnu : un zoom sur un tableau plein de textes ferait dessiner beaucoup de formes à la fois ;
Lengyel conçoit Slug pour dessiner **tout** le texte à chaque image, mais sa carte n'est pas
la PowerVR de son téléphone. **Protocole** avant d'écrire une ligne : un banc qui zoome sur 10,
100 et 1 000 nœuds de texte, coût des formes par image contre coût des paliers, à son PC puis à
son téléphone (la télémétrie y dira le poste).

**Slug au processeur ?** Il rendrait les deux voies identiques au bit près, de près comme de
loin. Mais le processeur paierait deux rayons et leurs courbes **par pixel**, là où il pose
aujourd'hui un glyphe déjà rastérisé (GLYPH-1) ou trace les grands (GRAND-1). La charte dit
« deux moteurs vraiment distincts » : la carte pour le parallèle, le processeur pour le reste.
SLUG-1 écrit ce qui les sépare, et la loi de place borne la seule faute qui compterait. Pas
maintenant ; à rouvrir si un écart se voit à l'œil.

**Les titres des membranes** restent peints au processeur dans la couche du dessous : nets,
mais repeints à chaque image où ils bougent. Le chemin est tracé — ils passent par la
`Surface`, qui sait déjà écrire en vecteurs (fiche 70) ; un `Releve` qui serait une `Surface` les
enverrait à la carte. C'est la suite naturelle de P8.
