# 94 — Comprendre Wikipédia d'un regard : ce que dit la recherche, et ce qui en découle

> 8/11/2026, après la fiche 93. Sa parole, ce soir-là, devant les deux questions de la fiche 91 :
> *« j'en ai aucune idée ; l'idée, c'est de travailler sur comment le cerveau humain serait capable de
> comprendre Wikipédia tout entier sur un canevas 2D sans que ce soit trop brouillon, avec le maximum de
> facilité »*. Il ne tranche pas : il donne **le critère**. Cette fiche l'applique.

## 1. Ce que la recherche dit (consultée le 8/11/2026)

* **La mémoire de travail tient environ quatre morceaux**, et ils s'emboîtent : on ne garde que ceux du
  niveau le plus haut, et on déplie les autres au besoin (Cowan, [*The magical number 4 in short-term
  memory*](https://memory.psych.missouri.edu/assets/doc/articles/2001/cowan-bbs-2001.pdf), 2001 — le « 7 »
  de Miller était une estimation large). **À chaque distance, la carte doit donc montrer peu de grandes
  régions nommées, chacune se dépliant en quelques autres.**
* **Wikipédia a déjà cette ossature** : les [articles vitaux](https://en.wikipedia.org/wiki/Wikipedia:Vital_articles),
  cinq niveaux de 10, 100, 1 000, 10 000 et 50 000 articles, chacun contenant le précédent, rangés par
  thèmes (personnes, histoire, géographie, arts, sciences…). Dix fois plus à chaque niveau : un niveau par
  palier de zoom.
* **Les cartes de géographie le font depuis toujours** : la hiérarchie des noms par la taille et la
  graisse, et moins de noms à petite échelle ([un cours de cartographie](https://atlas.co/courses/gis-basics/typography-in-maps/)) —
  un continent s'écrit plus gros qu'un pays, un pays qu'une ville ; à toute distance, à peu près autant de
  noms lisibles à l'écran.
* **Des groupes dessinés comme des pays** se lisent mieux, pour le grand public, que des nœuds reliés par
  des traits (Gansner, Hu, Kobourov, [*GMap*](https://arxiv.org/pdf/0907.2585), 2009-2010 — leur
  argument, sans étude de lecture trouvée).
* **On retrouve par l'endroit** : la mémoire spatiale porte le classement ([*Data Mountain*](https://dblp.uni-trier.de/rec/conf/uist/RobertsonCLRTD98.html),
  Microsoft, 1998). Un lieu appris ne doit plus bouger en silence.
* **Le « brouillard du désert »** : zoomé dans le vide, on ne sait plus où l'on est ; il faut des repères
  de ce qui est trop loin ou trop petit pour se voir (Jul et Furnas, UIST 1998 — cité de mémoire, non
  vérifié ici).
* **Des cartes de Wikipédia existent** : [Encartopedia](https://blog.fastforwardlabs.com/2017/08/08/encartopedia.html)
  (100 000 articles placés par proximité de sens, à côté de l'article lu), Knowledge Mapper (le laboratoire ContextLab)
  (250 000 articles, la densité égalisée). Elles placent par le sens ; elles ne disent pas comment s'y
  retrouver d'un regard.

## 2. Ce qui en découle pour Glucose — quatre principes

1. **Peu de régions nommées à chaque distance** : l'ossature des articles vitaux en membranes emboîtées
   (dix « continents », cent « pays », mille « régions »…), le reste dedans.
2. **Le nom d'une région à la taille de la région** : le titre d'une membrane grandit avec elle, pour
   qu'à toute distance on lise les noms des régions qui tiennent à l'écran — tous les titres restent dessinés, aucun ne se cache — sans
   rien changer au texte, ni remplacer un nœud par autre chose (la charte refuse la LOD sémantique ; une
   hiérarchie typographique n'en est pas une : le titre est le même, à sa taille).
3. **Le nom d'une région vaut pour ce qu'elle contient** : un domaine posé sur une membrane vaut pour
   tout ce qu'elle porte — comme le nom d'un pays vaut pour ses villes, qu'on n'étiquette pas une à une.
4. **Un lieu appris ne bouge plus** : aucune disposition ne se refait en silence ; « Disposer en
   colonnes » est un geste, annulable.

## 3. Les deux questions de la fiche 91, tranchées par ce critère

* **Un domaine posé sur une membrane vaut-il pour ce qu'elle contient ?** **Oui, par héritage** (principe
  3) : un nœud appartient aux domaines de ses membranes, sauf s'il dit autre chose par son propre poids.
  Rien ne s'écrit sur les nœuds : l'héritage se lit. Pour une IA, le sommaire ne répète plus sur chaque
  paragraphe ce que porte l'article (46 % du sommaire de « Eau ») ; le plan, lui, le dit une fois.
* **Les titres des grandes membranes plus gros ?** **Oui** (principe 2). Les titres des membranes sont
  encore peints au processeur (SUITE, étape 3 : « les titres des membranes en formes ») : agrandis, ils
  doivent passer par le dessin des lettres depuis leur forme (P8), net à toute taille.

## 4. La suite, dans l'ordre

1. **DOM-6 — l'héritage des domaines par les membranes** : à la lecture (`glucose_lire`, `glucose_domaines`,
   le panneau « où »), sans rien écrire sur les nœuds.
2. **Les titres des membranes à la taille de leur région**, dessinés depuis leur forme.
3. **Un squelette de Wikipédia** : les niveaux 1 et 2 des articles vitaux (dix continents, cent pays),
   posés et rangés par les commandes ; puis « Eau » et « Hydrogène » dedans, à leur place, reliés.
4. **Les liens** : des millions de flèches ne se lisent pas. La charte interdit d'en cacher pour
   « simplifier » ; c'est à l'agent qui pose la carte de choisir, comme un cartographe, les liens qu'il
   trace. À concevoir avec lui, sur un vrai morceau.

## 5. DOM-6, fait le même soir

* **La loi** (`store/agents/decrit.rs`) : chaque nœud décrit garde ses domaines **propres**, et porte à
  côté ceux qu'il **hérite** de ses membranes, de la plus proche à la plus lointaine, chacun avec la
  membrane qui le donne ; son propre poids l'emporte ; une membrane plus proche l'emporte sur une plus
  lointaine. **Rien ne s'écrit sur les nœuds.** Une seule description sert au tableau ouvert et aux
  fichiers fermés : l'héritage traverse les fichiers sans rien de plus.
* **Ce qui la lit** : le filtre `domaine=` de `glucose_lire` ; la liste d'un domaine pour un agent
  (`glucose_domaines`, chaque nœud hérité disant `herite_de`) ; **pas** le compte des endroits (une
  membrane, un endroit), ni la liste « où » du panneau, où l'humain vole vers la membrane plutôt que vers
  ses 198 paragraphes. Les instructions des agents disent la règle : poser le domaine sur la membrane
  d'un article, pas sur chaque paragraphe.
* **L'essai réel**, « Eau » posé par les commandes, ses sept catégories sur la seule membrane de
  l'article : **le sommaire passe de 135 376 à 70 287 caractères** ; `glucose lire domaine=Hydraulique`
  rend les 252 nœuds de l'article ; `glucose domaines domaine=Eau`, 252 nœuds dont 251 hérités, pour un
  seul endroit.
* **Prouvé** : trois épreuves (la loi dans le noyau, à travers les fichiers, pour un agent de bout en
  bout) ; **8 sabotages sur 8** (`outils/sabotages/heritage-94.json`).

## 6. Sa parole du 8/11 au soir, qui corrige cette fiche

*« Pas de système de LOD, et pas de simplification possible »* ; *« j'accepte de revoir le
fonctionnement des membranes afin qu'elles soient plus intéressantes »* (VISION).

* **Ce qui tombe** : ce que la cartographie appelle « moins de noms à petite échelle » ne s'applique
  pas — **aucun titre ne se cache** ; le principe 2 ne garde que **la taille** des titres selon leur
  région. Le § 4.4 (« l'agent choisit les liens qu'il trace ») est **à remettre en question** : choisir à
  la pose n'est pas cacher à la vue, mais « pas de simplification » peut vouloir dire que tous les liens
  doivent pouvoir être là — lui demander, en prose, avec un vrai morceau sous les yeux.
* **Ce qui s'ouvre** : la membrane peut être revue (sa forme, ce qu'elle porte, comment elle se lit et
  s'explore) pour rendre la carte plus lisible **sans rien simplifier**. À concevoir avec lui : la
  recherche d'abord, puis des propositions chiffrées.
