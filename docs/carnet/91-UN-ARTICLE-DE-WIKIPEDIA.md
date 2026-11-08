# 91 — Un article de Wikipédia, posé par les commandes : ce qui manque

> Session du 8/11/2026, après la fiche 90. SUITE le voulait : *« un morceau de Wikipédia posé par les
> commandes, pour voir ce qui manque »*. La charte : *« le blocage actuel n'est pas la performance,
> c'est le manque de fonctionnalités : Glucose n'en a pas encore assez pour rendre une telle carte
> navigable »*. Mieux vaut que le manque se montre sur un vrai article que de le deviner.

## 1. L'essai

L'article **« Eau »** de Wikipédia en français (8 680 mots, 10 sections, 31 sous-sections, 7 catégories,
plus de 500 liens) — l'exemple même de sa définition du domaine. Posé **par les seules commandes du
terminal** (`outils/wikipedia/`, fiche 90) dans le vrai Glucose sans fenêtre : une membrane pour
l'article, une par section et sous-section, emboîtées par `dans` ; un texte par paragraphe ; les
catégories comme domaines. **259 commandes, 3 secondes, 12 ms chacune** — les positions laissées à
Glucose (« sous le contenu de sa membrane »). Puis lu comme une IA, et regardé comme un humain.

## 2. Ce qui s'est montré

### MEMB-7 — un vrai défaut, réparé

L'article a trois niveaux ; Glucose en rendait **huit**, et rangeait « Problématique de l'eau en
montagne » dans « Polarité ». Une membrane qui recevait un nœud grandissait, **pas celle qui la
contenait** : son centre sortait de sa parente, et l'appartenance, qui se lit par la géométrie (MEMB-1),
la rangeait ailleurs ; les nœuds posés ensuite tombaient dans la boîte d'une autre. **MEMB-7** : la
croissance remonte jusqu'à la première membrane qui n'a pas à bouger
(`Store::agrandir_la_membrane_pour`). Éprouvé (trois membranes, un texte de 5 000 de haut), saboté ;
**rejoué sur l'article : trois niveaux, chaque section à sa place.**

### Pour une IA : un plan (niveau « plan ») et des coordonnées courtes (COORD-1)

* **Le sommaire d'un seul article pesait 140 721 caractères** (environ 35 000 jetons). Sa vision : *« une
  IA doit savoir où se situe quoi dans la carte »*. `glucose_lire` a désormais un niveau **`plan`** : les
  seules membranes, emboîtées, leur titre, leur boîte, et **ce que chacune contient** — **12 007
  caractères** pour l'article, onze fois moins ; l'agent descend ensuite par `dans`. Les instructions le
  lui disent : sur un grand tableau, le plan d'abord.
* **COORD-1** : une hauteur mesurée en `f32` s'écrivait « 51.599998474121094 ». Le document place tout à
  1/2⁸ d'unité (`glucose_core::fixed`) ; trois décimales (10³ ≥ 2⁸) font retomber chaque point de la
  grille sur lui-même — **déduit, pas choisi** ; éprouvé sur 4 000 points de la grille.
* **Reste lourd : les domaines répétés**, 46 % du sommaire. Une catégorie vaut pour l'article entier ;
  posée comme domaine, elle se répète sur les 198 paragraphes, et elle ne distingue plus rien. **Sa
  question** (§ 4).

### Pour un humain : illisible

Glucose empilait tout : **une colonne d'environ 60 000 de haut**, quelques pixels de large en vue
d'ensemble. VISION : c'est *« par la disposition (un travail presque d'illustration) »* qu'un humain
ordinaire doit pouvoir consulter Wikipédia naturellement. **Le premier manque, c'est la disposition des
membranes emboîtées.**

## 3. La recherche, et un prototype (8/11/2026)

* Presque aucun canevas ne dispose de lui-même un contenu emboîté : [Syncfusion](https://www.syncfusion.com/forums/198092/is-auto-layout-inside-group-nodes-supported-or-is-manual-layout-required)
  ne le fait pas dans un groupe ; [Aha!](https://www.aha.io/blog/automatically-organize-shapes-on-your-whiteboard)
  dispose un cadre en rangée ; [Mindvas](https://simbench.gitbook.io/mindvas/features/forest-layout) en
  grille. Le travail le plus juste vient de Kiel, autour d'ELK : [« Top-Down Drawings of Compound
  Graphs »](https://arxiv.org/pdf/2312.07319) — chaque conteneur se dispose de bas en haut, en visant une
  proportion ([ELK](https://eclipse.dev/elk/blog/posts/2023/23-04-11-topdown-layout.html)).
* **Ce qu'ils n'ont pas, et qu'un article exige : l'ordre de lecture.** D'où le prototype (`outils/wikipedia/en_colonnes.py`,
  par les seules commandes) : chaque membrane range ses enfants, **dans l'ordre**, en colonnes de
  journal ; la **partition linéaire** (programmation dynamique, exacte) rend la plus haute colonne la
  plus courte possible ; le nombre de colonnes se choisit pour que la membrane ait la proportion de
  l'écran. Aucune constante à régler hors des marges.
* **Le résultat, regardé** : l'article tient dans un écran (8 928 × 6 342, au lieu de 60 000 de haut), ses
  sections en blocs, ses paragraphes en colonnes, dans l'ordre. **Ce qui ne va pas encore** : les titres
  des sections, tous de la même petite taille, sont illisibles de loin (sur une carte, un pays s'écrit
  plus gros qu'une ville) ; des vides sous les colonnes courtes quand un bloc très haut impose sa
  hauteur ; beaucoup de cadres emboîtés.

## 4. Ce qui attend sa parole

* **Un domaine posé sur une membrane vaut-il pour tout ce qu'elle contient ?** Défaut proposé : oui, par
  héritage — un nœud dans la membrane « Eau » est de l'eau, sauf s'il dit autre chose par son propre
  poids. Sa définition dit que le domaine **traverse** les membranes ; l'héritage dirait que la membrane
  **transmet** : deux mécanismes, qui ne se contredisent pas. C'est sa vision du domaine : **sa
  décision.**
* **La taille des titres des membranes selon la taille de la membrane**, pour qu'une vue d'ensemble se
  lise comme une carte : de l'ergonomie, donc l'étape 5 — à moins qu'il ne veuille l'avancer pour
  Wikipédia.

## 5. La suite proposée

1. **La disposition en colonnes dans Glucose** : « Disposer en colonnes » au menu d'une membrane et dans
   Ordonner (récursif, annulable d'un `Ctrl+Z`), et la même règle pour un agent qui pose dans une membrane
   — avec sa parole sur l'image.
2. **Les liens entre articles** : les 500 liens de « Eau » vers d'autres articles, des flèches vers
   rien tant que l'autre article n'est pas posé. Un second article (« Hydrogène ») posé à côté, et les
   flèches entre eux : la vraie carte commence là.
3. **Selon sa réponse**, l'héritage des domaines par les membranes.

## 6. Les leçons

* **Un essai d'usage réel trouve ce qu'aucune épreuve ne cherchait** : 2 874 épreuves vertes, et un
  article à trois niveaux rendu sur huit. Le défaut n'apparaît qu'à la troisième profondeur.
* **Les commandes rendent le prototype possible** : une disposition essayée de l'extérieur, en Python,
  regardée en image, avant d'écrire une ligne de Rust. Ce qui engage sa vision s'essaie ainsi d'abord.
* **Chercher toutes les formes d'un nom avant de le changer** : `complet:` trouvait l'un des deux
  appelants, l'autre écrivait `complet,`.
* **Lire l'heure, ne pas l'estimer** : les points de cette session annonçaient trois heures de travail
  quand il en était passé une.
