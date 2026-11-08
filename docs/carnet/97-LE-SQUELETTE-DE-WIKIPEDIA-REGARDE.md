# 97 — Le squelette de Wikipédia, regardé ; ce qu'un agent pose « dans » une membrane lui appartient

> 8/11/2026, tard, après la fiche 96. La fiche 94 avait fixé le cap (*« comprendre Wikipédia tout
> entier sur un canevas 2D, sans que ce soit trop brouillon »*) et la suite : un squelette, les cent
> articles vitaux du niveau 2, « Eau » et « Hydrogène » dedans. Le script était prêt depuis la fiche
> 95, éprouvé contre un faux Glucose. Cette fiche le pose dans le vrai, et **regarde**.

## 1. L'essai, en une commande

`nix develop -c bash outils/wikipedia/essai_squelette.sh <sortie>` : le vrai Glucose sans fenêtre, les
commandes `glucose` contre sa porte, puis le tableau entier, les neuf thèmes et les deux articles en
images (2 560 × 1 600, `GLUCOSE_VUES` nomme les membranes à rendre de près), le plan qu'un agent
lit, et le document. **9 thèmes, 103 articles, « Eau » (198 paragraphes, 53 sections) et
« Hydrogène » (52, 23) ; 542 nœuds rangés en colonnes ; 216 + 336 commandes, 6 s.** Les images et
le document sont gardés dans `~/.cache/glucose-bancs/squelette-wikipedia/` (`images-avant/` : avant
MEMB-8).

## 2. Ce que la première image a montré : la pose était fausse

Vue d'ensemble, les grands noms se lisaient — mais des titres de sections d'« Eau » s'empilaient
**dans** « Médecine » et « Énergie (physique) », en escalier. Vue de près : les sections d'« Eau »
s'emboîtaient en chaîne (*Géophysique › L'eau dans l'Univers › Découverte › Abondance ›…*), et
celles d'« Hydrogène » (*Atome d'hydrogène, Isotopes, Corps simple*) finissaient **au fond
d'« Eau »**. Le plan le confirmait : « Eau » ne contenait qu'**une** membrane, et sa section
« Généralités » était dans un autre article vital.

**La cause** : à sa naissance, un nœud appartient à la plus petite membrane qui contient son centre
(MEMB-1) — la règle de la main, qui dépose là où elle lâche. Un article qui grandit (MEMB-7, fiche
90) fait grandir ses parentes, **mais ne pousse pas ses voisines** — un lieu appris ne bouge pas en
silence (fiche 94, principe 4). « Eau », en grandissant dans son article vital, recouvrait ses
voisins ; un nœud posé « dans Eau », sous son contenu, tombait sur un voisin plus petit, et la
géométrie l'emportait sur ce que l'agent avait dit. Un article posé seul (fiches 91 et 92) ne le
montrait pas : il n'a pas de voisins.

## 3. MEMB-8 — ce qu'un agent pose « dans » une membrane lui appartient

VISION le disait déjà (22/10) : *le sens d'un tableau ne doit jamais tenir seulement dans la position ;
ce qui compte s'écrit dans une structure explicite.* Poser « dans » une membrane devient un seul
acte : `poser_pour_un_agent(…, dans)` et `poser_une_photo_pour_un_agent(…, dans)` — aucun appelant ne
peut l'oublier. Ce que l'agent nomme l'emporte sur la géométrie ; sans nom, la naissance décide,
comme pour la main. Deux refus neufs, dits en clair : **on ne pose rien dans ce qui n'est pas une
membrane** ; **une membrane ne se pose pas dans ce qu'elle contient** (un cycle). Un refus n'écrit
rien (le geste d'agent s'annule entier).

**Reposé** : aucune section hors de son article ; « Eau » contient ses 11 sections, « Hydrogène » ses
10 ; la profondeur la plus grande, 7 (*Wikipédia › Sciences et médecine › Chimie › Eau › Eau —
Wikipédia › Propriétés › Propriétés physiques*) ; le tableau, 20 830 × 10 633 au lieu de 13 683 de
haut.

## 4. Ce que les images disent, jugé

* **Ça se lit comme une carte** : *Wikipédia — les cent articles vitaux*, puis *Sciences et médecine*,
  *Chimie*, *Eau*, *Eau — Wikipédia*, et dans l'article ses huit sections et leurs sous-sections,
  chacune à la taille de sa région (TITRE-1) ; de loin, chaque texte montre le gris de ses lignes
  (GRIS-1), jamais un cadre vide.
* **Le déséquilibre** : « Sciences et médecine » prend presque tout l'écran — deux articles entiers y
  sont, contre cent résumés de deux phrases ailleurs. Les huit autres thèmes ont des noms qu'on ne lit
  pas à cette distance. C'est la loi qui joue honnêtement sur des données déséquilibrées : la surface
  dit la quantité, pas l'importance. Avec tout Wikipédia, l'équilibre revient ; avec un squelette, une
  question pour lui (§ 6).
* **De grands vides** : sous « Chimie », dans la première colonne des sciences, à côté de la longue
  introduction d'« Eau ». La limite dite de la fiche 92 : un bloc géant impose sa hauteur, et l'ordre
  de lecture interdit de combler en déplaçant un bloc plus loin.
* **Tout est du même bleu** : aucune région ne sert de repère par la couleur — la proposition D (une
  teinte par grande région, Tree Colors, fiche 96 § 7) y répondrait.
* **Un niveau redit** : l'article vital « Eau » (ses deux phrases) contient l'article entier « Eau —
  Wikipédia » ; de même *Élément chimique › Hydrogène — Wikipédia*. Un choix du script, pas de Glucose.

## 4 bis. Le squelette dans le vrai programme : ce que la mesure a montré

Ouvert dans le vrai Glucose (le banc hors écran, la carte graphique de cette machine), « tout
cadrer » sur le squelette : **chaque image coûtait 33 ms, et Glucose ne se posait jamais** — 99 % des
images redessinées « parce que la netteté revient », trente secondes durant. Trois choses à séparer,
par la mesure en paires entrelacées (`21e14d1`, d'avant cette session ; `6a358c9`, après GRIS-1 ;
l'actuel) et le profil (`perf`), pas par l'hypothèse :

* **Ce qui était à moi** : l'image médiane passait de 23 à 33 ms après GRIS-1. J'ai d'abord accusé
  le gris des lignes (la loi des membranes, `Membrane::alpha`, de 6 à 11 % du profil). **Faux** — ou
  presque : le profil avec les appelants a montré `Contenants::fond_en`, qui calcule, pour **chaque
  nœud de texte**, la couleur du fond sous lui en lisant la loi de **chaque membrane** du tableau,
  sur deux chemins : 542 nœuds × 190 membranes × 2, à chaque image. Vus de loin, les nœuds ont
  désormais un contenu (leur gris), et ce calcul s'est mis à courir pour tous. **FOND-1** : chaque
  membrane relève son enveloppe une fois par image ; hors d'elle sa loi vaut zéro au bit près, et
  `fond_en` la saute — l'épreuve compare au calcul complet en 80 000 points, au bit près. **L'image
  médiane revient à 23 ms**, celle d'avant la session (le p99, 33 à 39 ms contre 27).
* **Une affirmation fausse, corrigée** : la fiche 96 disait qu'un filet d'un dixième de pixel, peint
  par la loi des membranes, « garde un dixième de son encre ». La loi calcule la couverture d'un pixel
  par sa distance au bord (`½ − d`) : exacte pour un bord franc, **pas pour une forme plus fine qu'un
  pixel** — un tel filet y prenait en moyenne trois fois son encre (jusqu'à cinq dixièmes). **RECT-1** :
  un rectangle droit vu de loin (le gris d'une ligne, un filet, une plaque) se peint par
  `peindre_un_rectangle`, la couverture exacte — le produit des recouvrements en largeur et en
  hauteur —, plus rapide aussi. Le gris d'un nœud vu de loin s'accorde désormais à son dessin de
  près à 0,05 niveau. Un disque (la puce) reste à la loi des membranes, juste dès qu'il fait un pixel.
* **Ce qui est ancien, et le plus grave** : Glucose ne se pose jamais sur ce tableau — `21e14d1`
  non plus. Le réveil « la netteté revient » a trois raisons (un élan qui s'arrête ; une **finesse à
  rendre** après une résolution réduite ; une **scène inachevée**) ; sur un tableau où l'image entière
  coûte plus que le budget, la deuxième peut tourner en cercle (l'image nette est lente, la
  résolution baisse, à l'arrêt on la rend nette…), un cercle vicieux au sens de METHODE § 4 — **à
  confirmer par la mesure**, pas fait. Les lettres des titres s'y re-rastérisent à chaque image
  (`blend_glyph`, `halo`, 30 % du profil). « Le repos à zéro » (SUITE, étape 1) n'est pas vrai sur une
  carte de Wikipédia : un cœur tourne en permanence. **À lui de dire** si c'est le prochain chantier
  (sa parole du 7/11 : pas d'église de l'optimisation, sauf un usage réel — c'en est un).

## 5. Prouver

* `memb_8_ce_qu_un_agent_pose_dans_une_membrane_lui_appartient` : le scénario du squelette réduit à
  deux articles voisins, l'un qui grandit par-dessus l'autre — il exige le recouvrement, puis que
  chaque nœud soit dans la membrane nommée ; `memb_8_les_refus_de_dans`.
* `le_fond_saute_les_membranes_loin_du_point_sans_rien_changer` ; les rectangles du noyau (sur la grille, l'encre égale à l'aire à toute épaisseur, rien pour une boîte vide) ; `gris_2_un_filet_d_un_dixieme_de_pixel_garde_sa_juste_encre`.
* **Sabotages** (`outils/sabotages/memb-8-97.json`) : **3 sur 3** (ignorer la membrane nommée, poser dans ce qui n'est pas une membrane, poser une membrane dans ce qu'elle contient). `outils/sabotages/fond-et-filet-97.json` : **3 sur 3** (le fond qui saute une membrane qui touche le point, le rectangle qui couvre le pixel entier, le gris de loin rendu à la loi des membranes). **Suite entière : 2 936 épreuves, aucune tombée.**
* **L'essai réel** : le squelette reposé, le plan relu, les images regardées (§ 3, § 4).

## 6. Ce qui attend sa parole

* **Le squelette à son écran** : le document est dans `~/.cache/glucose-bancs/squelette-wikipedia/glucose`,
  à ouvrir dans un Glucose isolé (`outils/essai_isole.sh`, son dossier `state/glucose`).
* **Le déséquilibre** : laisser la surface dire la quantité (la loi d'aujourd'hui), ou donner à
  chaque thème une place à lui — une décision de vision, pas de technique.
* **B, C, D** (fiche 95 § 4) : une membrane dit ce qu'elle porte ; un fil « où je suis » (la recherche :
  Wu et Vakkari 2014, des lecteurs de Wikipédia se servaient de l'arbre de la hiérarchie plutôt que de
  la carte) ; une teinte par grande région (Tree Colors).

## 7. Les leçons

* **Regarder l'image avant de déclarer** : 2 900 épreuves vertes, un script éprouvé contre un faux
  Glucose — et la première image du vrai montrait un arbre faux.
* **Une réparation qui tient seule peut tomber à plusieurs** : MEMB-7 réglait un article posé seul ;
  à côté de ses voisins, la croissance devenait un chevauchement.
* **Ce qui est dit ne se devine pas** : quand l'appelant nomme la structure, la géométrie n'a pas à la
  reconstruire.
