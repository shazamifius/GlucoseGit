# 96 — Le gris des lignes, le Focus par un geste choisi, et une colonne de trop

> Session du 8/11/2026 au soir, après la fiche 95. Sa parole du soir (VISION, « Wikipédia sur le
> canevas ») : *« le mode Focus, je préférais que ce soit un geste choisi »* ; *« je suis carrément
> d'accord pour le gris des lignes »*. La fiche 95 § 11 les avait écrits sans les compiler, dans un
> correctif ; cette session les reprend, les compile, les éprouve, les sabote — et la remise en
> question y a trouvé trois défauts.

## 1. Le correctif repris

`95-focus-et-gris-a-reprendre.patch` s'appliquait tel quel sur `21e14d1`. Clippy n'a refusé qu'un type
trop complexe dans une épreuve du focus (nommé) ; le cliquet des 80 lignes, `layout_paragraph` (81) :
le gris d'une ligne est devenu `Typography::gris`, une mesure de typographie (largeur sans les blancs
de fin, encre des lettres), que les post-it partagent au lieu de la recopier. L'ancienne épreuve
SCALE-2 exigeait qu'un nœud vu de loin ne montre **aucune encre** — le cadre vide qu'il a refusé ;
réécrite : de loin, **aucune lettre ne se rastérise** (la part juste de l'ancienne garde), et le
texte montre le gris de ses lignes.

## 2. COLONNES-2 — une colonne de trop, née d'un arrondi (un défaut de la fiche 92)

L'épreuve des coupes tombait sur le code juste (fiche 95 § 11) : graine 0, une colonne de 2 196 contre
1 211. **La disposition avait tort.** Pour une colonne, la bissection prenait comme hauteur haute la
somme de la suite (`sum() + écarts`) ; le remplissage, lui, additionne bloc après bloc, écart compris,
dans un autre ordre. Deux ordres de somme diffèrent au dernier bit : le remplissage dépassait la
hauteur de toute la suite d'un cheveu, et **le dernier bloc partait seul dans une seconde colonne** —
huit blocs d'un côté, un de l'autre. Les épreuves ne le voyaient pas : leurs hauteurs étaient des
entiers, qui s'additionnent exactement dans n'importe quel ordre. Les titres qui grandissent (TITRE-1)
donnent des hauteurs qui ne le sont pas.

**La réparation n'est pas une tolérance** : une colonne ne s'additionne plus que d'une façon,
`empiler`, que le remplissage et la hauteur d'une colonne partagent. L'addition arrondie est monotone :
la preuve d'optimalité du remplissage glouton (une colonne qui commence plus tard finit au moins aussi
loin) tient alors au bit près, sans epsilon. Les hauteurs des épreuves sont désormais des septièmes,
qui ne s'écrivent pas en binaire.

**L'épreuve aveugle, comprise.** Réparée, l'épreuve des coupes passait — et le sabotage « la partition
ignore la place des titres » restait aveugle : ses données (des boîtes de 300 à 12 000 de large)
rangeaient **tout en une seule colonne**, où il n'y a aucune coupe à faire. Une simulation a mesuré
ce que les titres changent : un dixième des hauteurs, qui ne déplace une coupe que dans quelques cas
sur cent. L'épreuve balaie donc 120 cas (12 et 24 membranes, trois proportions, vingt graines), **exige
plusieurs colonnes dans chacun**, et compare à la plus basse colonne possible (`coupes_pour`, prouvée
contre la force brute). Sabotages `outils/sabotages/colonnes-96.json` : **3 sur 3**.

## 3. GRIS-1, fini — rien ne se cache de loin, pas même un ornement

**Mesuré** : vu de loin, un nœud a la couleur moyenne de son dessin de près à **0,07 niveau** près
(sur 255 ; l'écart du texte à son fond est de 18). La borne de l'épreuve n'est plus « 25 % de
l'écart », provisoire : elle se **déduit** de ce que garantissent deux images de 8 bits — chacune à
moins d'un niveau de sa couleur exacte par pixel, donc deux niveaux par canal entre deux moyennes —,
avec la preuve à l'envers (le texte doit s'écarter de son fond de deux fois la borne, sur un canal au
moins : un post-it jaune à l'encre sombre ne bouge presque pas dans le bleu).

**Ce que le correctif oubliait** : de loin, seules les lettres devenaient grises ; **la plaque d'un
bloc de code, le trait d'un `---`, la barre d'une citation, les puces, les numéros et les passages qui
brillent disparaissaient** — cacher selon la distance, contre sa parole. L'épreuve, enrichie de
chaque ornement, l'a dit d'un coup : **4,7 niveaux de trop**, de loin. Deux causes :

* les ornements n'étaient pas posés (réparé : `ornement_de_loin`, aux places de `draw_ornament`) ;
* une fois posés, **la grille des pixels** (SCALE-3, `fill_crisp`) ramène à un pixel entier tout
  rectangle plus fin — net de près, dix fois trop d'encre de loin pour un filet d'un dixième de pixel.
  De loin, chaque ornement se peint donc par **la loi des membranes**, à la couverture exacte de chaque
  pixel ; la puce, comme un rectangle arrondi en disque. Écart après : **0,1 niveau**.

**Une géométrie, deux peintres** : ce qu'un nœud montre de loin se décrit une fois, en formes
(`formes_de_loin`) ; le canevas les peint à la couverture exacte, le dessin vectoriel les pose tels
quels — **l'export SVG et la carte graphique** (un nœud plus grand que l'écran, DE-PRES-1), qui ne
posaient encore que la brume. Les post-it suivent la même loi (`ligne_grise`, `peintre`).

Épreuves : le nœud de texte et le post-it gardent de loin la couleur de leurs lettres ; le dessin
vectoriel pose le gris des lignes et aucune lettre ; sous le seuil, aucune lettre ne se rastérise.

**Le seuil, remis en question** : un cinquième de zoom est-il une constante arbitraire ? Non : le corps
d'un nœud vaut toujours 14 (aucun nœud n'a de police à lui), et le seuil dit **une lettre de 2,8
pixels**. C'est la forme que la recherche donne au « greeking » des interfaces zoomables : un seuil en
taille de lettre affichée, pas en zoom ([Piccolo](https://www.cs.umd.edu/projects/hcil/jazz/learn/piccolo.net/doc-1.2/api/UMD.HCIL.Piccolo.Util.PUtil.GreekThreshold.html),
[GEF](https://www.eclipse.org/lists/gef-dev/msg00037.html), InDesign). Le jour où un nœud aura sa
propre taille, le seuil devra se lire par nœud.

## 4. FOCUS-1, éprouvé

Ce que la fiche 95 § 11 décrit, compilé. Deux épreuves manquaient : « Entrer dans la membrane » n'existe
au menu que sur une membrane choisie, et conduit à sa commande ; « Sortir de la membrane », à la
palette, sort comme `Échap` (et ne fait rien hors du focus). Sabotages : voir § 6.

## 5. GRIS-2 — de loin, rien ne se tait

Sous un cinquième de zoom, **neuf dessins se taisaient encore** (SCALE-2) : le nom des zones, le nom
d'un dossier (son bandeau), les étiquettes des flèches, le mot d'un opérateur (ET, OU…), le sigle
d'un domaine sur sa réglette, les textes des cases d'un storyboard, la pastille d'un prédicat, et deux
lueurs (celle d'une membrane, celle d'un dossier choisi). Sa règle — *rien ne se cache selon la
distance* — et la loi qu'il a acceptée pour les nœuds s'y appliquent telles quelles.

**La loi vit à un seul endroit, la typographie du canevas** (`typography/encre.rs`) : une lettre plus
petite que `LETTRE_LISIBLE` (2,8 pixels) se peint du **gris de son encre** — le rectangle de sa
ligne, à l'opacité de l'encre de ses lettres sur cette place, à la couverture exacte de chaque pixel
(`forme_grise`, `peindre_exact`, que les nœuds de texte et les post-it suivent aussi). Les neuf
conditions **disparaissent** : chaque dessin écrit toujours, et la typographie décide comment. Le seuil
de zoom des nœuds (`SIMPLIFIED_BELOW`) n'est plus un second nombre : il **se déduit** de la lettre
lisible et du corps d'un nœud (2,8 ÷ 14). Au passage disparaît un autre excès : `clamp_font_size`
ramenait toute lettre de moins d'un pixel à un pixel — quatre fois l'encre d'un texte d'un
demi-pixel.

**Le support décide** (`Vectoriel::a_des_pixels`) : la carte graphique finit en pixels, et suit la
même loi ; **l'export SVG, qui s'agrandit sans perte, garde ses lettres à toute distance** — à leur
vraie taille, sans le pixel minimal. (Jusqu'à GRIS-1, un SVG exporté de loin n'avait aucun texte ;
GRIS-1 y mettait du gris ; le voici avec ses lettres.)

**Mesuré** : sous le seuil, l'encre peinte vaut celle des lettres (21,14 pour 21,19) ; **au passage du
seuil, l'encre ne saute pas** (25,1 en lettres au corps lisible, 26,1 en gris juste dessous, sous la
borne de 1,4 que deux images de 8 bits garantissent) — un nom ne change pas de teinte en dézoomant.

**Ce qui ne suit pas encore la loi, dit** :

* **Les titres des membranes** (détourés, TITRE-1) restent des lettres à toute taille : leur gris
  devrait composer celui du halo (`(h − c)/(1 − c)` sous celui des lettres, une forme exacte) ; c'est
  aussi le coût d'un zoom (fiche 95 § 8), qui attend sa parole.
* **Tout rectangle plus fin qu'un pixel** hors des nœuds de texte (les traits des zones, des poignées,
  des cases) passe encore par la grille des pixels (SCALE-3) : ramené à un pixel entier, trop lourd
  de loin. À reprendre au même principe : de loin, la couverture exacte.
* **Un bloc de code vu de loin** s'écarte d'1,5 niveau de son dessin de près (le texte courant, de
  0,1) : sous la borne, mais plus que le reste — à comprendre.
* **Le curseur et l'aperçu d'une formule** pendant qu'on écrit restent de près seulement (`dessus.rs`) :
  des aides d'édition, pas du contenu.

## 6. Prouver

* **Le noyau** : 1 001 épreuves ; COLONNES-2 (la bissection exacte sur des hauteurs non entières, les
  coupes sur 120 cas à plusieurs colonnes).
* **L'application** : le gris (texte, post-it, bloc de code, vecteurs selon le support, aucune lettre
  sous le seuil), le Focus (entrer, Échap, la palette, le menu), GRIS-2 (l'encre peinte, le passage du
  seuil, la carte graphique, et chacun des neuf silences levés : `rien_ne_se_tait_tests.rs`, les
  dossiers, les sigles), les cliquets.
* **Sabotages** : `colonnes-96.json` **3 sur 3** ; `focus-gris-96.json` **15 sur 16** au premier tour —
  l'aveugle : « de loin, la plaque d'un bloc de code disparaît ». Une ligne de code parmi vingt-cinq,
  une plaque presque de la couleur du fond : son absence changeait la moyenne de moins d'un niveau.
  Une épreuve dont le texte n'est presque qu'un bloc de code la garde désormais. `gris-2-96.json` :
  *à compléter*.

## 7. La recherche (consultée le 8/11/2026)

* **Le seuil du gris** : les cadres zoomables le posent en taille de lettre affichée (Piccolo,
  GEF ; InDesign règle le sien en points affichés) — § 3.
* **Une teinte par grande région** (la proposition D de la fiche 95) : **Tree Colors** (Tennekes et de
  Jonge, [IEEE TVCG 2014](https://vis.cs.ucdavis.edu/vis2014papers/TVCG/papers/2072_20tvcg12-tennekes-2346277.pdf)) —
  la roue des teintes partagée récursivement entre les branches d'un arbre, chaque enfant proche de
  la teinte de son parent, la clarté et la saturation réglées par la profondeur ; les frères de même
  clarté. Déterministe, sans palette choisie à la main : la forme que la charte veut. Une palette à
  plat n'a qu'une dizaine de couleurs sûres ([Colorcet](https://colorcet.holoviz.org/user_guide/Categorical.html)) ;
  l'arbre des articles vitaux en a cent — c'est pourquoi la teinte doit suivre l'arbre.
* **Un fil « où je suis »** (la proposition C) : dans les canevas emboîtés, un fil d'Ariane cliquable
  en haut, la racine et les derniers niveaux gardés quand il est long ; au-delà de trois ou quatre
  niveaux, on se perd quel que soit le fil (un avis de praticiens, pas une étude).

## 8. Les leçons

* **Une épreuve qui passe sur des données qui n'exercent rien est aveugle**, même juste : celle des
  coupes ne faisait jamais de coupe. Une épreuve dit ce qu'elle exerce, et l'exige.
* **Des entiers cachent les arrondis** : une épreuve de flottants prend des valeurs qui ne s'écrivent
  pas en binaire.
* **« Le même nombre » calculé deux fois n'est pas le même nombre** : ce qui se compare se calcule par
  une seule fonction (la leçon de la fiche 88, « une même chose sous deux noms », en arithmétique).
* **Une épreuve dont le texte porte tout ce qu'un texte peut porter** a trouvé deux défauts d'un coup
  (les ornements absents, puis trop lourds) : le texte de la première version n'avait ni code, ni
  trait, ni citation.
