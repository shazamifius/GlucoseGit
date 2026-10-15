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

## Ce qui est encore à définir — sa parole

* **Trans-domaines** : de la recherche ; *« j'ai une intention, mais je n'ai pas fini mon
  raisonnement sur ce que j'entends par domaine »*.
* Le design des dossiers, à retravailler ; le dessin de la sélection (E, pour l'instant).
