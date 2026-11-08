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

## 5. Prouver

* `memb_8_ce_qu_un_agent_pose_dans_une_membrane_lui_appartient` : le scénario du squelette réduit à
  deux articles voisins, l'un qui grandit par-dessus l'autre — il exige le recouvrement, puis que
  chaque nœud soit dans la membrane nommée ; `memb_8_les_refus_de_dans`.
* **Sabotages** (`outils/sabotages/memb-8-97.json`) : **3 sur 3** (ignorer la membrane nommée, poser dans ce qui n'est pas une membrane, poser une membrane dans ce qu'elle contient). **Suite entière : 2 931 épreuves, aucune tombée.**
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
