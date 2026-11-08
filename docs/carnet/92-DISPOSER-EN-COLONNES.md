# 92 — Disposer en colonnes : un article qui tient dans un écran

> Session du 8/11/2026, après la fiche 91. L'article « Eau » posé par les commandes s'empilait en une
> colonne de 60 000 unités de haut : illisible. VISION : *« par la disposition (un travail presque
> d'illustration) […] un humain ordinaire doit pouvoir consulter ce wiki de façon tout à fait
> naturelle »*. Le prototype de la fiche 91, fait de l'extérieur par les commandes, a montré la voie ;
> cette fiche en fait une fonction de Glucose.

## 1. Ce que c'est

**« Disposer en colonnes »** range ce que porte une membrane — textes, post-it, photos, sous-membranes,
elles-mêmes rangées d'abord — en **colonnes de journal**, **dans l'ordre de lecture** (de haut en bas,
puis de gauche à droite : un article posé par un agent, empilé, se relit dans l'ordre où il a été
écrit), et donne à chaque membrane **la proportion de l'écran**. Son coin haut-gauche ne bouge pas ;
chaque nœud reste dans sa membrane ; **un seul `Ctrl+Z`** rend le document exactement tel qu'il était.

Trois portes, une seule loi (`Store::disposer_en_colonnes`) :

* **au menu du clic droit**, et à la palette (« Disposer en colonnes ») — au PC comme au doigt : les
  membranes choisies, sinon celles qui portent ce qu'on a choisi (`Store::membranes_de_la_selection`),
  comme `F` cadre ce qu'on désigne ; hors de toute membrane, rien. **Aucun message** : le résultat se
  voit, et un `Ctrl+Z` le défait ;
* **pour un agent** : `glucose_disposer` (`dans`, `proportion` ; par défaut celle de l'écran) ;
* **au terminal**, sans rien écrire de plus (TERMINAL-1) : `glucose disposer eau`.

## 2. Le calcul — COLONNES-1, deux bissections et aucune constante

Le prototype essayait toutes les découpes : O(k·n²) pour chaque k, trop lent pour une membrane de mille
nœuds. La forme juste est le **partage des peintres** :

* **pour k colonnes, la plus basse hauteur possible** : à une hauteur donnée, remplir les colonnes dans
  l'ordre dit combien il en faut, et ce nombre ne croît jamais quand la hauteur croît ; une bissection
  sur la hauteur, jusqu'à deux flottants voisins, rend l'optimum exact de la partition linéaire, en O(n)
  par pas ;
* **le nombre de colonnes** : plus il y en a, plus la membrane s'élargit et s'abaisse ; sa proportion ne
  décroît pas avec k. Une seconde bissection trouve la frontière, et le meilleur de ses deux voisins
  l'emporte.

Les marges sont celles d'une membrane de Glucose (l'écart entre deux nœuds, le bandeau du titre), celles
qu'un agent emploie déjà : le module n'en choisit aucune. La disposition « de bas en haut » des graphes
composés vient de Kiel (Kasperowski et von Hanxleden, [*Top-Down Drawings of Compound Graphs*](https://arxiv.org/pdf/2312.07319)) ;
l'ordre de lecture, qu'un article exige, est ce qu'ils n'ont pas.

## 3. Prouver

* **L'optimum exact**, comparé à la **force brute** (toutes les découpes) sur 40 suites, toutes les
  tailles jusqu'à dix, tous les k ; **la proportion** : aucun autre k ne fait mieux ; **l'ordre et
  l'emboîtement** : chaque enfant dans sa membrane, sous son bandeau, deux frères jamais l'un sur
  l'autre, l'ordre de lecture gardé.
* **Dans le document** : un article posé comme par un agent — l'arbre ne change pas, le coin ne bouge
  pas, chacun dans sa membrane, l'ordre de lecture, et **un `Ctrl+Z` le rend égal au bit près** ; une
  membrane vide, un texte, un nom inconnu ne rangent rien et n'écrivent rien.
* **L'application** : la commande range et se défait ; un nœud choisi range sa membrane, un nœud hors de
  toute membrane ne range rien ; l'entrée du menu n'existe que là ; l'agent range en un geste, et refuse
  ce qui n'est pas une membrane.
* **Les cliquets ont corrigé la première version** : le menu lisait lui-même les annotations du tableau
  (le couplage au modèle montait de 38 à 39) et deux messages s'ajoutaient (48 toasts pour 46). Le noyau
  dit désormais quelles membranes ranger, et le geste se tait.
* **L'essai réel** : « Eau » posé par les commandes, puis **`glucose disposer eau`** : 251 nœuds rangés en
  une commande de 8 ms, l'article en 8 360 × 5 413 (1,54 pour 1,6 visé), regardé en image : il tient
  dans un écran, en blocs de sections, dans l'ordre.
* **Sabotages** (`outils/sabotages/colonnes-92.json`) : **9 tombés sur 9**.

## 4. Les limites, dites

* **Les titres des membranes** restent de la même petite taille : de loin, on voit la structure, pas
  les noms. L'écrire plus gros selon la taille de la membrane, comme une carte écrit un pays plus gros
  qu'une ville, est de l'ergonomie : **sa décision** (fiche 91 § 4).
* **La proportion** n'est qu'approchée : le nombre de colonnes est entier.
* **Des vides** restent sous une colonne courte quand un bloc très haut impose sa hauteur ; l'ordre de
  lecture interdit de les remplir en déplaçant un bloc plus loin.
* **Les flèches ne se rangent pas** : un bout accroché suit son nœud, une flèche libre dans la membrane
  reste où elle était.
* **L'ordre de lecture est celui de la géométrie** : sur un tableau fait à la main, ce que l'œil lit
  d'abord ; un agent qui veut un autre ordre pose dans cet ordre.
