# Ce que Glucose veut devenir

> **Sa parole du 15/10/2026**, recueillie après l'inventaire de la fiche 71 — la première fois
> qu'elle est écrite. Elle dit **pourquoi** Glucose existe ; [ÉTAT](ETAT.md) dit où il en est, et
> [SUITE](SUITE.md) ce qui vient. Toute décision se juge aussi contre elle — et elle se remet en
> question comme le reste, avec des arguments.

---

## Le but : remplacer entièrement la feuille de papier

*« En 2026, une feuille de papier est et reste indispensable. Mon défi : créer un système qui
s'adapte à la technologie, pour les étudiants, les développeurs, les artistes, les joueurs, ou
qui que ce soit. »*

Le canevas infini n'est pas la fin : c'est la feuille, enfin sans bord. Ce qui compte est qu'on
puisse y faire **tout** ce qu'on fait sur du papier, et ce que le papier ne sait pas faire.

## Quatre objectifs, dans l'ordre où il les a dits

1. **Le premier vecteur d'échange d'humain à humain.** Concentrer **tout Wikipédia** dans Glucose,
   et beaucoup de bases de données propres — un immense canevas. **Mais** : par la disposition
   (un travail presque d'illustration), les domaines, Trans-domaines, les flèches, les membranes et
   la couleur, un humain ordinaire doit pouvoir **consulter ce wiki de façon tout à fait
   naturelle**. *« Autant te dire que c'est un défi super compliqué. »* (C'est le canevas de
   `glucose-brain` ; la cible des 10⁷ nœuds vient de là.)
2. **Le premier vecteur d'IA à IA.** L'idée de **Proteo** (un projet parallèle, sur son ordinateur) :
   donner à des IA, pour se parler, un socle **immensément meilleur que de simples jetons dans le
   vide**. Une grande étape de recherche, et de suivi de l'actualité.
3. **Le premier vecteur d'humain à IA et d'IA à humain**, pour créer des projets de tout genre :
   communiquer avec une IA par un canevas 2D plutôt que par un fil de discussion — **des graphes de
   questions et de réponses**, qui ne se pensent plus comme un texte linéaire. Et des systèmes d'IA
   qui remettent un projet en forme : exemple, on invente une langue pour un univers ; un plugin
   (sur son GitHub) comprend la langue à partir des mots et des schémas déjà posés, aide à nettoyer
   les idées, regroupe les mots, propose des mots neufs cohérents avec elle, capte les intentions et
   la complexité cachée de la langue.
4. **Réinventer le document linéaire de physique et de mathématiques** pour la recherche. *« Que
   Glucose soit extrêmement performant pour la recherche et le développement, d'IA à IA comme
   d'humain à humain ; qu'il soit utilisé par les plus grands ingénieurs du monde. »*

## Une application mère, ouverte à tous

* **Des centaines de plugins**, et des modifications du code lui-même : Glucose est une
  application mère (voir [PLUGINS.md](../PLUGINS.md)).
* **Open source, en Rust**, pour que **n'importe qui puisse forker** et créer sa version de
  Glucose. C'est pour cela que l'arborescence des fichiers est extrêmement minutieuse : elle doit
  se lire par un inconnu.
* **Glucose devient lui-même un ensemble de dépendances** (sa parole du 15/10) : des briques en
  Rust, propres, que d'autres prennent pour bâtir leur version — *« qu'il y ait des milliers de
  forks, adaptés à tout besoin »*. Le noyau sans dépendance (`glucose-core`) est la première ; le
  rendu, la typographie, les formules, le format en sont d'autres à découper, chacune lisible
  seule.
* **Les dépendances : à terme, zéro absolu.** Sa parole du 15/10 précise la charte (« 0 n'est pas
  l'objectif » *aujourd'hui*) : on prend une dépendance quand on ne peut pas faire sans, et on la
  réécrit quand on sait faire mieux nous-mêmes — le chemin va vers zéro, jamais au prix d'une
  machine exclue.

## Les presets : l'interface entière, au choix

**Un preset n'est pas un gabarit de zones** (ce que le panneau « Presets » de Tauri appelait ainsi).
C'est **l'interface et sa manière de réagir** : la disposition, les couleurs, les raccourcis
clavier, toutes les fenêtres en règle générale. Le but : **faire mieux que PureRef, Figma et Miro**.

* **Un preset de Glucose**, *« extrêmement travaillé, qui n'a jamais été créé »* : l'interface
  principale, qui regroupe **absolument toutes** les fonctionnalités ; extrêmement ergonomique,
  efficiente, rapide pour les professionnels, et en même temps très, très propre à l'œil — *« pousser
  l'outil au maximum dans son ergonomie »*.
* **Un preset PureRef**, **un preset Figma**, **un preset Miro** : retrouver 100 %, ou presque, des
  fonctionnalités de chacun, avec sa façon de faire.
* Sa liste des capacités de PureRef à refaire viendra après.

## L'utilisateur choisit toujours

*« Je suis d'accord avec le système qui permettrait à l'utilisateur de TOUJOURS CHOISIR ! »* Là où
deux réponses sont légitimes — la planche claire ou sombre, l'export à 100 % ou aux originaux, la
numérotation, la place d'un numéro —, Glucose ne tranche pas à la place de l'utilisateur : il
propose un défaut raisonnable, et laisse choisir. Les presets porteront ces défauts.

## Ce que cela demande au code, dès maintenant

Écrit par la session du 15/10, à remettre en question :

* **L'interface devient une donnée.** Un preset ne peut exister que si les raccourcis, le thème, la
  disposition des panneaux et les comportements se lisent dans **une seule table** chacun, et non
  dans le code de chaque geste. Chaque fonctionnalité écrite avec un raccourci en dur ajoute au
  travail de demain.
* **Les plugins** veulent une frontière nette entre le noyau, l'application et ce qui s'y greffe —
  le noyau sans dépendance l'est déjà.
* **L'échelle** : 10⁷ nœuds navigables (l'arène, écrite, attend son branchement) — et une
  navigation sémantique (domaines, couleur), jamais la dégradation du sens.
* **Le graphe avant le texte** : les flèches, les ancres de texte et les membranes sont déjà la
  grammaire des objectifs 3 et 4.
* **L'ergonomie se fait à la fin** : une grosse session, quand toutes les fonctionnalités seront
  là. Avant, chaque fonction a son bouton, au PC et au téléphone (INVENTAIRE).

## Le pont entre l'humain et l'IA — sa parole du 22/10/2026

*« Depuis le début, Glucose visait à faire le pont entre l'humain et l'IA. »* Les quatre objectifs
plus haut en sont les faces. Trois précisions du même jour :

* **Un Glucose doit pouvoir être lu par un humain ET par une IA, de quasi la même façon.**
  *« ATTENTION : impérativement. »* Ce qu'un humain voit, une IA doit pouvoir le lire, et l'inverse.
* **Pas seulement Wikipédia.** *« Pourquoi se limiter à Wikipédia ? Certains datasets d'IA sont
  super utiles, MÊME pour un humain : les avoir sur Glucose, ça doit pouvoir être incroyable. »*
* **Un Glucose que tous les Protéo remplissent ensemble**, et où un Protéo **apprend** : *« pour
  que Protéo lui-même démonte Maya, le comprenne — en fait il va apprendre plein de trucs avant de
  le remonter pour finaliser l'addon ; tout ça peut être écrit sur Glucose. »*

Et trois exigences pour les agents, dites le même jour :

* **Une IA doit savoir où se situe quoi dans la carte** — pas seulement la liste des nœuds : leur
  géographie (ce qui est à côté de quoi, dans quelle région, dans quelle membrane).
* **Un système de déplacement et de recherche ULTRA avancé pour les IA**, *« pour qu'elles
  retrouvent ce dont elles ont besoin directement, sans trop galérer — attention : Glucose est là
  pour faire des recherches, mais des fois on ne sait pas ce qu'on recherche »*. Il faut réfléchir à
  des systèmes pour **coller Protéo à Glucose**.
* **Une IA peut être elle-même le sous-agent d'une autre** — avec peu de mémoire de carte graphique
  et peu de temps pour répondre (côté Protéo : sa conception, § 4.20).

**Un usage qui attend** : un projet Rust de plusieurs millions de lignes (web3), à poser **sur**
Glucose — non pas son code, mais **l'arbre complet de comment il est fait** (*« une arborescence
mathématique super complexe »*), que Protéo construit ; puis travailler sur ce projet avec Protéo,
depuis Glucose.

## Le domaine — sa définition (22/10/2026)

*« Toutes les sessions de code me demandent ce qu'est un domaine : c'est le moment de répondre. »*

> **Le domaine, c'est l'idée de créer un fork.** Dans le domaine **eau**, on peut très bien avoir :
> « l'eau, ça mouille » ; « eau = H₂O » ; « l'eau dans les jeux vidéo : il faut des systèmes de
> noise map et de displacement pour créer un effet de 3D ». Ce sont **trois choses très séparées**,
> dans des **blocs de texte distincts**, qui peuvent eux-mêmes être dans des **membranes**, elles-mêmes
> peut-être dans des **fichiers** différents — **mais le domaine, lui, va traverser tout ça : tout
> fait partie du domaine de l'eau.** Aujourd'hui, sur Glucose, on ne peut tout simplement pas le
> faire : c'est pour ça que le domaine existe.

**Pour l'instant, on se concentre sur le domaine** — mais pas sur sa visualisation : *« ça va dans
la catégorie ergonomie de l'application, et l'ergonomie, ce n'est pas pour tout de suite »*.

**Trans-domaines** : *« une espèce de vue en perspective 3D de tous les domaines… mais c'est de la
R&D, je ne suis pas sûr. »* Il attend ; le bouton dit « bientôt ».

**Les dimensions de Glucose** (sa piste de R&D, telle qu'il la dit) : un espace latent de 4 096
dimensions, posé sur un canevas 2D, devient *« absolument PLUS exploitable »* ; des chercheurs
ont réussi à travailler un espace latent sur une centaine de dimensions pour le redéplier ensuite.
Glucose n'est pas *juste* un canevas 2D : les axes x et y ; la couleur des blocs (*« autant de
couleurs visibles que de dimensions »*) ; la membrane (une dimension, plus une couleur) ; la flèche
(*« une infinité de dimensions ? c'est un système qui a énormément d'options »*) ; le domaine (une
infinité de dimensions, plus une couleur).

### Ce que la session du 22/10 en comprend, et y précise (à remettre en question)

* **Membrane et domaine sont deux structures différentes, et c'est ce qui fait le domaine.** Une
  membrane **contient** : un nœud est dans une seule membrane à la fois, et les membranes
  s'emboîtent — un arbre. Un domaine **traverse** : un nœud peut appartenir à plusieurs domaines, à
  des degrés divers, quelle que soit sa membrane ou son fichier — un recouvrement, pas un arbre. Le
  noyau a déjà cette forme pour un document (`Project.domains`, chaque nœud porte ses domaines avec
  un poids de 0 à 1 : une appartenance graduée) ; il lui manque de **traverser les fichiers**, d'être
  **vu** (filtrer, colorer) et d'être **lu par une IA** (`glucose_lire` ne dit pas les domaines).
* **« Lu de la même façon » oblige à une règle** : le sens d'un tableau ne doit jamais tenir
  **seulement** dans la position. Un humain lit la proximité ; une IA reçoit des coordonnées qu'elle
  n'interprète pas. Ce qui compte s'écrit donc aussi dans une structure explicite — une membrane, une
  flèche, un domaine —, que les deux lisent pareil.
* **Les dimensions, comptées honnêtement** : x et y sont deux dimensions continues ; la couleur
  perçue en a trois (clarté, deux axes de teinte), et l'œil ne distingue de façon sûre qu'une
  dizaine de couleurs **catégorielles** (ordre de grandeur des guides de visualisation, à vérifier) —
  au-delà d'une dizaine de domaines, la couleur seule ne suffira plus à les reconnaître ; chaque
  domaine est une dimension à lui (appartient, et à quel degré) : leur nombre n'a pas de borne.
  Ce qui rend Glucose plus exploitable qu'un espace latent n'est pas de le **projeter** en 2D (une
  projection de 4 096 dimensions en perd presque tout), c'est que ses dimensions sont **explicites et
  nommées** — « eau », « contredit » — là où celles d'un modèle sont anonymes.

## Ce qui est encore à définir — sa parole

* ~~**Trans-domaines** : de la recherche ; *« j'ai une intention, mais je n'ai pas fini mon
  raisonnement sur ce que j'entends par domaine »*.~~ **Le domaine est défini depuis le 22/10**
  (plus haut) ; Trans-domaines reste de la R&D (une vue en perspective 3D de tous les domaines).
* Le design des dossiers, à retravailler ; le dessin de la sélection (E, pour l'instant).
