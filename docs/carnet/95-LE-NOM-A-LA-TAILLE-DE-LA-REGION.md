# 95 — Le nom d'une région à la taille de la région ; revoir les membranes

> Session du 8/11/2026 au soir, après la fiche 94. Sa parole du jour : *« pas de système de LOD, et
> pas de simplification possible »* ; *« j'accepte de revoir le fonctionnement des membranes afin
> qu'elles soient plus intéressantes »* (VISION, « Wikipédia sur le canevas »). La fiche 94 avait
> tranché, par son critère, que **le titre d'une membrane grandit avec elle** (principe 2) ; sa
> parole du soir y ajoute : **aucun ne se cache**.

## 1. La télémétrie

145 sessions, 8 installations. **Ses pincements du 8/11 y sont** : deux sessions de son VivoBook
(`7565fa5e`) portent des épisodes « zoomer » (12, puis 21), absents des sessions d'avant la fiche 93 ;
le pire d'un zoom, 12,5 ms. La panique du 7/11 est celle que CONSOLE-1 a réparée.

## 2. La recherche (consultée le 8/11/2026)

* **Des groupes dessinés en régions d'un seul tenant se lisent mieux** : Saket, Simonetto, Kobourov,
  Börner, [*Node, Node-Link, and Node-Link-Group Diagrams: An Evaluation*](https://arxiv.org/pdf/1404.1911)
  (IEEE TVCG 2014 ; 36 personnes, 2 916 essais) — les tâches de groupe nettement plus justes et plus
  rapides, rien d'autre de gêné. L'étude contraire (Jianu et al.) avait des régions **morcelées** et des
  **titres posés sur le contenu**, qui cachaient l'information : deux choses à ne pas faire.
* **La mémoire range les lieux par régions emboîtées** : Hirtle et Jonides,
  [*Evidence of hierarchies in cognitive maps*](https://link.springer.com/content/pdf/10.3758/BF03197683.pdf)
  (1985) ; deux lieux d'une même région se jugent plus proches, une frontière allonge un trajet.
* **Le « brouillard du désert »** (Jul et Furnas, UIST 1998, vérifié par [ces notes de cours](https://www.cs.ubc.ca/~tmm/courses/533-09/slides/nav.pdf)) :
  zoomé dans le vide, on ne sait plus où l'on est ; il faut des « résidus » de ce qui est loin.
* **Bubble Sets** (Collins, Penn, Carpendale, [2009](https://innovis.cpsc.ucalgary.ca/Research/BubbleSets)) :
  un contour qui joint les membres d'un ensemble **sans déplacer la disposition** — exactement la forme
  d'un domaine qui traverse les membranes.
* **La forme d'une région compte** : Blake, Stapleton, Rodgers et al., [*The impact of shape on the
  perception of Euler diagrams*](https://research.brighton.ac.uk/en/publications/the-impact-of-shape-on-the-perception-of-euler-diagrams)
  (2014) — cercles, puis carrés, puis ellipses et rectangles.
* **Le nom d'une région, au plus grand qui tient** : la cartographie ([*Computing Curved Area Labels*](https://arxiv.org/pdf/2001.02938)),
  Imhof — un nom tient dans sa région.
* **Les canevas** : les sections de [Heptabase](https://wiki.heptabase.com/section) gardent leur titre
  lisible dézoomé ; les [boîtes de Kinopio](https://help.kinopio.club/posts/boxes/) grandissent pour
  envelopper ; les tableaux emboîtés de [Muse](https://museapp.com/guide) s'entrent d'un geste et se
  quittent par un fil d'Ariane ; l'[hypertexte spatial](https://people.engr.tamu.edu/shipman/spatialhypertext.html)
  (VIKI) : les gens expriment la structure par la place et les attributs visuels plutôt que par des liens.

## 3. Ce que la recherche et l'image ont montré

**L'article « Eau » rangé en colonnes, regardé** : la structure se voit, **aucun nom ne se lit**, et
**toutes les membranes se ressemblent** (même forme, même teinte) — aucune région ne sert de repère.

**Trois endroits où Glucose cachait déjà selon la distance**, contre sa parole :

1. **Le titre d'une membrane**, sous 20 % de zoom (SCALE-2, `draws_detail`) — et d'un corps fixe de 16,
   qu'une membrane fasse 300 ou 8 000 de large. **Réparé ici** (TITRE-1).
2. **Le texte de chaque nœud**, sous 20 % : remplacé par son cadre vide (SCALE-2). Une lettre y fait
   moins de 3 pixels ; mais le cadre vide ment — un long article et un nœud vide se ressemblent.
   **Proposé** : le gris exact de chaque ligne. **Sa parole attendue.**
3. **Le mode Focus** (MEMB-2, de Tauri) : zoomer jusqu'à ce qu'une membrane couvre 92 % de l'écran cale
   la caméra et **fait disparaître tout le reste**. **Proposé** : un geste choisi (double-clic, « Entrer
   dans la membrane », Échap), plus un effet du zoom. **Sa parole attendue.**

## 4. Les propositions, en prose (le message du 8/11, 19 h 35)

A. **Le nom à la taille du pays** — décidé (fiche 94) : fait ici. B. **Une membrane dit ce qu'elle
porte** (« 12 sections · 198 paragraphes »), ce que l'IA lit déjà dans le plan. C. **Où je suis** : un
fil en haut du canevas (« Eau › Propriétés › Polarité »), chaque nom cliquable, la même phrase pour
l'IA. D. **Une teinte par grande région**, ses régions en nuances — à voir en image. E. **Plus tard** :
un bord qui épouse le contenu (GMap) — pas recommandé, le texte se range en rectangles ; le domaine
dessiné comme une bulle qui traverse les membranes (Bubble Sets) — sa visualisation, rangée dans
l'ergonomie. **Ses deux questions** : le Focus en geste ; le gris des lignes.

## 5. TITRE-1 — la loi

`glucose_core::membrane_titre`. **Chaque membrane consacre à son nom la même part de sa surface
qu'une membrane neuve** (320 × 240, un titre de 16) : chaque mesure du titre — corps, retrait,
élévation — est multipliée par `s = √(A / A₀)`, jamais moins que 1. Aucune constante neuve ; rien de
ce qui est plus petit qu'une membrane neuve ne change. Sur « Eau » : l'article, un titre de 387 (120
pixels quand il remplit l'écran) ; une section, 112 ; une sous-section, 45 ; un paragraphe, 14.

* **Aucun titre ne se cache** : la condition `draws_detail` disparaît des deux titres (le canevas et le
  SVG). Un titre trop petit pour se lire est dessiné tel quel.
* **Un nom tient dans sa région** : le corps de la loi, ramené s'il le faut au plus grand qui tient
  dans la largeur de la membrane, toutes ses mesures ensemble (`MembraneVue::titre`, une seule
  règle pour l'écran et le SVG). Il ne fait que rétrécir.
* **L'encre d'une membrane** (`Annotation::encre`) : sa boîte, et la bande de son titre au-dessus. Le
  culling (l'index, le dessin), tout cadrer et l'export, `F`, le vol vers un nœud, la sélection
  élastique et la zone qu'un glisser salit la lisent ; l'appartenance, l'aimant, les poignées et la
  boîte qu'un agent lit restent la boîte.
* **La bande de clic** au-dessus du bord (30) grandit avec le titre : on saisit une région par son nom.
* **Disposer en colonnes** réserve au-dessus de chaque sous-membrane ce que son titre prend en plus
  d'un titre ordinaire (`surcroit`, zéro pour une membrane neuve : toute disposition d'avant reste la
  même), dans la partition comme dans la pose.

**Regardé** : la vue d'ensemble d'« Eau » se lit comme une carte — « Eau — Wikipédia », ses huit
sections (Propriétés, Politique et économie, Symbolique…), leurs sous-sections ; aucun titre ne couvre
un bloc ; une première version laissait deux titres de colonnes étroites se chevaucher, d'où la règle
de la largeur.

## 6. CHANTIER-2 — Glucose redessinait à vide (un défaut de la fiche 86)

Le banc hors écran l'a montré : titres visibles, **2 309 images en 18 s, tableau immobile**, toutes « du
système » ; `main` : 36. Une sonde l'a nommé : **le cache des teintes** (TEINTES-1, fiche 86) restait
« en retard » ; son réveil demandait une image **sans rien salir** ; l'image n'avait rien à redessiner,
ne consultait pas le cache, et le retard ne retombait jamais. Mes grands titres mangeaient la tranche
des teintes et l'ont révélé ; un gros tableau le pouvait aussi — **dans les betas 8 et 9**. Une teinte
en retard salit désormais la scène (elle n'a pas les pixels de la vraie) ; une vignette seule demande
une image sans salir ; les deux au compte du chantier. Au repos : 2,9 images par seconde, comme `main`.

## 7. TITRE-2 — détourer en deux passes

Le profil (`perf`, un zoom sur « Eau ») : `blend_glyph` à 32 % du temps. Un texte détouré se composait
en **neuf passes** : huit copies du contour, puis la lettre. **Composer une teinte sur elle-même ne
change que son opacité** (la loi de MEMB-FORME-1) : les huit copies valent une seule image d'opacité
`1 − Π(1 − cₖ)`, **le halo**, calculé une fois et gardé dans le cache des lettres. Écart aux neuf
passes : 4 niveaux sur 255 au pire, mesuré (chaque passe tronquait à l'octet). Une lettre tracée
(GRAND-1) se détoure par un **trait** de 1,5 pixel puis son remplissage : le même corps au pixel près,
l'anneau à 1,3 % de surface. Au repos, le poste des titres passe de 11,6 à 3,4 ms.

## 8. Ce qui reste : le coût d'un zoom sur une très grande région

Mesuré sur cette machine (UHD 620, 1 920 × 1 080, le vrai Glucose hors écran, en paires entrelacées
avec `main`) : pendant un zoom sur « Eau », l'image médiane passe d'environ 9 ms (`main`) à environ
19 ms ; le pire reste celui de `main` (~50 ms). La cause, au profil : un titre change de taille à
chaque image, et chaque lettre se rastérise à nouveau. `main` ne le payait pas : ses titres étaient
cachés de loin et minuscules de près. Un tableau ordinaire (des membranes pas plus grandes qu'une
neuve) ne change pas.

Deux routes, chiffrées : **la carte graphique** (Slug, les lettres depuis leur forme) — mesuré ici,
6 ms pour un écran couvert de grandes lettres (`bench_de_pres`, 2 560 × 1 600), et le halo en neuf
copies le multiplierait : pas sans repenser le détourage ; **les titres en textures à paliers**, comme
les nœuds de texte (net à l'arrêt, une texture un peu plus grande réduite pendant le geste). À décider
avec lui, à son écran : sa parole du 7/11 ne veut pas d'un chantier de rendu sans usage qui le demande.

## 9. Prouver

* **Le noyau** : la loi (une neuve garde son titre, la même part de surface pour toutes — et l'ancienne
  loi ne la gardait pas —, la surface seule compte, rien ne rapetisse), l'encre, le cadrage et le vol,
  la bande de clic ; les colonnes (la loi pure, avec sa preuve à l'envers ; et dans le document).
* **L'application** : un titre se dessine à 1 % ; il grandit comme la racine de la surface ; il tient
  dans sa membrane ; une membrane sous l'écran montre déjà son nom ; glisser salit le titre ; un
  rectangle qui touche le titre prend la membrane ; les teintes en retard font redessiner jusqu'au bout ;
  le halo vaut les huit copies ; le trait détoure comme les neuf copies.
* **Sabotages** : `outils/sabotages/titre-95.json`.

## 10. Les leçons

* **Un banc lit ce qu'une épreuve ne voit pas** : la tempête de redessins ne tombait dans aucune
  épreuve ; le banc hors écran l'a comptée (« le système »), une sonde l'a nommée en une compilation.
* **Comparer à `main`, pas à une vieille version** : le programme optimisé du 7/11 n'avait pas les
  fiches 89 à 94 ; une construction de `main` sans mes changements a séparé ce qui était à moi.
* **Le profil contredit l'hypothèse** : je croyais les lettres géantes tracées coupables ; c'étaient
  les titres moyens, neuf fois composés. Mesurer avant d'écrire la réparation.
* **Une borne d'épreuve se déduit** : l'anneau du trait contre les huit copies — un dixième de pixel
  sur 1,5, pas un pourcentage choisi pour passer.

## 11. Sa parole, le soir même — et ce qui en est écrit, pas encore compilé

*« Effectivement, le mode Focus, je préférais que ce soit un geste choisi »* ; *« je suis carrément
d'accord pour le gris des lignes »* (VISION). **La session s'est arrêtée à sa demande avant la
compilation** : tout ce qui suit est écrit dans **`95-focus-et-gris-a-reprendre.patch`** (19
fichiers, il s'applique sur `f50499b` : `git apply docs/carnet/95-focus-et-gris-a-reprendre.patch`),
**jamais compilé dans l'application** (le noyau, lui, a passé ses 66 épreuves). À reprendre : compiler,
éprouver, saboter, regarder, puis enregistrer — et retirer le correctif du carnet.

### FOCUS-1 — on entre dans une membrane par un geste choisi

« Entrer dans la membrane », au menu d'une membrane choisie (ou qui porte la sélection) et à la
palette ; « Sortir de la membrane » à la palette, `Échap` (après un menu ouvert et un tracé en cours),
ou dézoomer et s'éloigner, comme avant. Le noyau ne décide plus que les sorties (`decider`) ; l'entrée
est `entrer(carte, membrane, écran, instant)`, au cadrage d'avant. **Disparaissent**, avec l'entrée
par le zoom : le seuil de 92 %, `coverage` et `to_screen`, la « vue » de chaque membrane et le calcul
des membranes minimisées qui ne nourrissait qu'elle ; `FocusInput` perd `resolved`. L'application
applique toute décision par une seule fonction (`appliquer_le_focus`). Épreuves réécrites : zoomer
n'entre plus ; le geste entre (même petite et emboîtée, au cadrage de six pour cent) ; Échap sort ;
dézoomer sort ; l'image décide des sorties, jamais des entrées. **À vérifier** : l'empreinte du menu
témoin (« Entrer dans la membrane » s'ajoute si la sélection du témoin porte une membrane) ; les
cliquets (le couplage du menu au modèle, les toasts) ; un double-clic sur une membrane édite toujours
son titre (gardé exprès).

### GRIS-1 — de loin, le gris de chaque ligne

Sous un cinquième de zoom, un nœud de texte posait son cadre vide (SCALE-2). Il montre le gris de
chaque ligne : un rectangle de la largeur de son texte, de la couleur de son encre, à l'opacité de
**son encre sur sa place** — la surface que ses lettres couvrent (`Typography::encre`, lue une fois
par lettre sur son image au demi-carré, puis × corps²), sur sa largeur fois la hauteur d'une ligne —,
peint par la loi des membranes (`Membrane::plein`, la couverture exacte de chaque pixel). Largeur et
encre se notent une fois, quand la ligne se coupe (`VisualLine::largeur`, `encre`), dans la mise en
page gardée en mémoire. Les tableaux : la largeur sur les taquets, l'en-tête en gras. Les formules :
le gris d'un texte de leur largeur, à la densité de la police (`Typography::densite`, l'alphabet) — une
approximation, dite. Les post-it : la même loi (`ligne_grise`, partagée), leurs lignes recalculées à
chaque image. **Pas encore** : la pilule des opérateurs (ET, OU…), dont le mot se cache encore de loin.
L'épreuve écrite (`card/gris/tests.rs`) compare la couleur moyenne d'un nœud vu à 0,1 à celle de son
dessin à 1 : sa borne (25 % de l'écart du texte à la brume) est **provisoire** — la mesurer, puis la
déduire.

### Les sabotages de la fiche 95 : 24 sur 25

`titre-95.json`, joués dans un arbre détaché : 24 tombent. **Aveugle** : « la partition ignore la
place des titres ». Une épreuve neuve est dans le correctif (`titre_1_les_coupes_comptent_la_place_des_titres`,
des membranes larges et basses dont les titres pèsent) — et **elle tombe sur le code juste** (graine 0 :
la plus haute colonne 2 196, l'optimum de la force brute 1 211 pour le nombre de colonnes retenu).
Deux possibilités, à trancher par la mesure : l'épreuve compte mal (le nombre de colonnes lu sur les
abscisses ?), ou la disposition a un vrai défaut quand les boîtes sont de largeurs très diverses.
**Ne pas relever la borne : comprendre.**
