# 102 — Les liens : de loin une trace, `Ctrl`+clic les suit, une fenêtre les amène

> Session du 10/11/2026, à partir de 18 h 31. La passation datait d'avant les fiches 100 et 101 ;
> les quatre questions de la fiche 99 restaient sans réponse (il n'avait pas ouvert Glucose depuis le
> 9/11 à 13 h 56, d'après sa chronique). Redites en prose au début de la session ; en attendant, le
> premier chantier du prochain lot — **les liens** — a été bâti sur des défauts argumentés, que sa
> parole peut renverser (§ 7).

## 1. La recherche

* **Bring & Go** — Moscovich, Chevalier, Henry, Pietriga et Fekete, *Topology-Aware Navigation in
  Large Networks*, CHI 2009 ([l'article](https://www.microsoft.com/en-us/research/publication/topology-aware-navigation-large-networks/)),
  **lu dans le texte cette fois**. Choisir un nœud colore ses liens et fait pâlir le reste (le
  *highlighting*) ; Bring & Go amène les voisins *« sur des cercles concentriques centrés sur le nœud
  choisi […] triés par distance, le plus proche d'abord ; chacun cherche, depuis le cercle intérieur,
  le premier où il se pose sans chevaucher, en gardant son angle »* ; choisir un voisin y va (la vue
  dézoome puis rezoome pendant que tout reprend sa place) ; et cela s'enchaîne. Leur limite, dite par
  eux : *« il donne très peu de contexte géographique »*. Dans leur étude, Bring & Go était nettement
  plus rapide que le zoom, la vue d'ensemble et Link Sliding.
* **La proximité de deux articles par leurs liens** — Milne et Witten, *An effective, low-cost
  measure of semantic relatedness obtained from Wikipedia links* (AAAI 2008,
  [Waikato](https://researchcommons.waikato.ac.nz/handle/10289/1777)) : le vecteur des liens d'un
  article, chacun pondéré par `log(|W| / |T|)` (un lien vers ce que tous citent compte moins), puis le
  cosinus — 0,66 de corrélation avec le jugement humain, 0,72 pour la mesure par les liens entrants.
* **La sériation circulaire** : l'ordre spectral de Coifman et al. (deux vecteurs propres du
  laplacien, [Recanati et al.](https://arxiv.org/pdf/1807.07122)) et l'algorithme optimal d'Armstrong,
  Guzmán et Sing-Long pour les dissemblances de Robinson strictes
  ([arXiv 2106.05944](https://arxiv.org/pdf/2106.05944)). Sur neuf thèmes, l'énumération exacte de
  la fiche 99 (20 160 cercles) reste juste ; ces méthodes serviront au-delà d'une dizaine.

## 2. Le banc : le squelette porte ses vrais liens

`GLUCOSE_LIENS=1 … essai_squelette.sh` (`squelette.py`, étape 7) pose les liens de Wikipédia entre
les cent articles vitaux : **1 128 flèches**, du résumé de chaque article (là où un lecteur le
trouve) vers la membrane de l'article cité, en quatre requêtes (`pltitles`, cinquante contre
cinquante). C'est sur lui que chaque image de cette fiche a été regardée.

## 3. LIEN-1 — de loin, une trace ; ce qu'on regarde, plein

Sa parole : *« de base, les liens sont très transparents ; dès qu'on s'approche d'un bloc de texte,
ils apparaissent »* — sa seule exception de niveau de détail. **Trois versions, et c'est l'image qui
a tranché deux fois** :

1. *Le seuil du gris* (une lettre de 2,8 pixels, GRIS-1) : en deçà, le lien rétrécit avec le monde.
   La vue d'ensemble n'en est qu'à deux crans : **une pelote** (`squelette-liens/images/tableau.png`,
   la première pose).
2. *La taille réelle* (une unité du monde par pixel logique, où un bloc de texte a ses 14 pixels) :
   de près, l'épaisseur d'écran de Tauri ; plus loin, une chose du monde. La vue d'ensemble se lisait —
   mais au pilote, à un tiers de la taille réelle, **encore une pelote sur l'article « Eau »**. La
   mesure dit pourquoi : un lien qui rétrécit avec le monde garde à l'écran **la densité d'encre de
   la lecture** — cinq fois plus de liens, chacun cinq fois plus fin et cinq fois plus court.
3. **Rétrécir et pâlir d'autant** (`WorldScale::lecture`, un seul facteur appliqué à la géométrie et
   à la présence) : la densité baisse avec la distance — un neuvième de l'encre de lecture au tiers,
   un centième au dixième. Regardé (`pilote-fenetre2/sortie/`) : la carte entière montre des traces,
   et au tiers les liens du nœud choisi ressortent seuls. **Aucune constante** : le seuil est
   l'identité du monde et de l'écran.

* **Ceux qu'on regarde restent pleins**, à toute distance : la flèche choisie ou survolée, celles
  d'un nœud choisi ou survolé (le *highlighting* de l'article), et toutes celles de la fenêtre des
  liens (§ 5). Le survol suit le nœud sous la souris en **une seule visée** avec la flèche survolée,
  et ne redessine que si un lien change d'aspect (pas à la taille réelle, pas sur un nœud sans lien).
* Plus fin qu'un pixel, le champ peint le trait à sa **couverture exacte** (le filtre-boîte de
  FLECHE-2), sur les deux voies ; l'export SVG suit la même loi. Les pastilles du bout sont déjà de
  la géométrie du monde et le restent — un point d'un tiers de pixel en vue d'ensemble ; les faire
  pâlir aussi demanderait une opacité de plus aux deux voies (nuanceur compris).
* Deux épreuves DPI-1 posaient le zoom 1 sur un écran à 150 % ou 200 % : le monde y est aux deux
  tiers de sa taille réelle, et LIEN-1 y amincit le lien — voulu. Leur décor est à la taille réelle
  de chaque écran ; la loi qu'elles gardent (des pixels logiques) et leur attendu n'ont pas changé.

## 4. LIEN-2 — `Ctrl`+clic suit un lien

*« Un clic sur un lien : un focus sur ce à quoi il est rattaché. »* Le clic seul choisit déjà une
flèche (pour la plier, la colorer, la supprimer) : le voler ôterait l'édition. `Ctrl`+clic est le
geste qui, dans Glucose, suit déjà un lien (une adresse dans un texte). **Vers le bout où l'on n'est
pas** — le plus loin du clic, ou, depuis le menu et la palette, du centre de l'écran —, à sa taille
de lecture ; `Alt+Z` ramène ; une flèche libre se cadre elle-même. Au doigt, « Suivre le lien » au
menu d'une flèche choisie ; à la palette, `lien.suivre`. Une épreuve a montré qu'un `Ctrl`+clic sur
la poignée du milieu d'une flèche **choisie** y pose un coude : les poignées d'un objet choisi
passent avant, c'est voulu.

## 5. LIEN-3 — la fenêtre des liens

### La disposition (`glucose_core::amener`)

Les cercles concentriques de l'article demandent un pas entre deux cercles : une constante. Chaque
voisin glisse plutôt **sur la demi-droite de sa vraie place**, au plus près du centre où il ne
chevauche rien de ce qui est posé, l'air d'une membrane (`ECART`) entre deux boîtes — la limite
continue des cercles. Exacte : une boîte qui glisse le long d'une demi-droite chevauche une boîte
posée exactement quand son point d'accroche est dans la somme de Minkowski des deux, que la
demi-droite traverse en un intervalle (la méthode des tranches). **Le point d'accroche** est né
d'une épreuve tombée : une membrane de provenance n'est pas centrée sur son nœud (son titre est
au-dessus) ; c'est le nœud, pas le milieu de la boîte, qui doit garder la direction.

### Un tableau de copies, à côté du document (`store/fenetre.rs`)

Ce qu'on y voit doit se dessiner, se viser, se zoomer et se lire **comme le canevas** : un tableau le
fait déjà. La fenêtre est un tableau de copies que le magasin montre à la place du tableau actif
(`active_board`), **à côté du projet** : aucune écriture du document ne l'atteint (elles nomment un
tableau du projet), aucun enregistrement ne le lit (la vue s'écrit depuis le projet), ses copies ont
des identités à elles, et sa propre identité change à chaque ouverture (l'index des nœuds la voit
changer). Changer de tableau, charger un document la ferment. Une épreuve exige qu'ouvrir, zoomer,
déplacer, choisir des copies et les supprimer laissent le document identique.

* **La provenance** : chacun dans une membrane titrée du chemin de ses membranes, à la couleur de la
  plus proche ; **ce que tous partagent ne se répète pas** (« Wikipédia — les cent articles vitaux »
  ouvrait chaque titre au premier regard — la racine se tait), chacun garde au moins sa membrane la
  plus proche.
* **Une membrane liée** vient par son nom, son chemin, et son premier texte dans l'ordre de lecture
  — la membrane entière serait une région de kilomètres.
* **Les liens** sont copiés sans leurs coudes ; ils contournent ce qu'ils traverseraient (FLECHE-5)
  plutôt que de barrer un texte qu'on vient lire — l'article les aplatit, Glucose garde sa loi.

### La main (`interactions/fenetre.rs`)

`B` (comme *Bring* ; changeable), « Montrer ses liens » au menu d'un nœud qui a des liens, la
palette (`liens.montrer`) ; sur une flèche choisie, son bout le plus proche du centre de l'écran.
Dans la fenêtre : **un appui relâché sans bouger y mène** (au relâchement, comme un bouton : au doigt,
glisser et l'appui long restent possibles), `Alt+Z` ramène ; `Maj`+clic la refait autour du nœud ;
glisser déplace la vue, d'où qu'on parte ; le clic droit et l'appui long choisissent la copie, et le
menu offre « Aller à ce nœud », « Montrer ses liens », « Fermer la fenêtre des liens ». **Elle est
passagère** : `Échap` la ferme, et **tout autre geste la ferme d'abord** — une commande (la seule
porte, `executer`, sauf ce qui la regarde), un clic dans l'interface. Rien de ce qui change le
document ne s'exécute pendant qu'on la regarde.

**Regardé au pilote** (`~/.cache/glucose-bancs/pilote-fenetre*/sortie/`) : `Ctrl+F` « Hydrogène »,
Entrée — le résumé d'« Eau » choisi, ses liens allumés —, `B` : l'Univers, les plantes, le feu, la
matière, le transport… amenés autour de lui, chacun sa provenance ; dézoomer : seize articles, tous à
gauche et en dessous, comme dans la carte ; un clic sur « Le transport » vole jusqu'à la région
Technologie ; `Alt+Z` ramène où l'on regardait la fenêtre.

## 6. Le sens des couleurs des membranes, version 2

`teintes_sens.py` lit désormais **les liens que la carte porte** (`glucose lire`) : deux régions se
ressemblent comme les articles qu'elles citent et qui les citent (Milne et Witten, chaque région
comptée parmi ses voisins pour qu'un lien direct rapproche aussi). Sur le squelette : **des
ressemblances de 0,31 à 0,71 entre thèmes**, contre 0,05 à 0,14 par les mots des résumés. Ce
qu'elles disent est juste : Philosophie et Société les plus proches (0,71), puis Histoire et
Société (0,65) ; Géographie avec Sciences (0,51), Sciences avec Technologie (0,44) ; les
Mathématiques à part. Image : `~/.cache/glucose-bancs/squelette-sens2/images/` — Sciences orangé,
Géographie sa voisine, Société lilas, Histoire rose, Mathématiques sarcelle. Une région sans lien
prend la teinte de sa parente ; les mots restent la mesure d'une carte sans liens.

## 7. Ce qui attend sa parole, et ce qui n'est pas fait

* **Les liens ont été bâtis sur mes défauts**, à renverser s'il veut : la transparence suit le
  **zoom** et le **survol** (les deux) ; le « show » s'ouvre sur un **nœud** (comme l'article) — et,
  sur une flèche choisie, sur son bout le plus proche ; **le clic sur un lien le suit avec `Ctrl`**,
  pas seul (le clic seul édite).
* **La fenêtre remplace la carte** le temps qu'on la regarde ; l'article garde la carte dessous
  (les amenés sont des mandataires posés dessus) — une suite possible, si l'on veut voir les deux.
* Ses trois autres questions restent ouvertes (fiche 99) : le sens de la couleur des textes (A, B,
  C) ; l'image du sens des membranes et une opacité de membrane dans le modèle ; `Alt+Z` après un
  voyage à la molette.
* Le nœud d'accueil d'un premier lancement se pose sur la carte (fiche 99 § 9) : **vu** sur le thème
  « Histoire » ; pas réparé.
* Les pastilles des liens ne pâlissent pas (§ 3) ; GRIS-3, B, C, REPOS-3 inchangés.

## 8. Les leçons

* **Regarder l'image a corrigé la loi deux fois.** La première passait ses épreuves, la deuxième
  aussi ; chacune faisait une pelote sur la vraie carte. Le calcul qui l'explique (la densité d'encre
  d'une chose du monde ne change pas avec le zoom) n'est venu qu'après l'image.
* **Mesurer avant de supposer**, encore : l'encre qui ne baissait pas de huit fois n'était pas
  l'arrondi à huit bits (vérifié en flottants : pire), mais les pastilles, déjà du monde.
* **Les instruments de l'épreuve** : `prendre_la_touche` ne porte que ce qui passe avant tout — les
  raccourcis passent par `handle_shortcut_input` ; la main déplace la vue par l'élan, posé à l'image
  suivante ; viser demande un index synchronisé sur le tableau qu'on regarde ; un décor qui ajoute
  un nœud le choisit.
* **Lire l'heure avant de la dire** : des points ont annoncé « 19 h 30 », « 20 h 05 » quand il était
  18 h 50 — estimées, pas lues. Corrigé dans la session, et la mémoire le disait déjà.

## 9. Le compte

* **La suite complète** : 3 073 épreuves passées, une tombée — le cliquet des 80 lignes : la porte des
  commandes (`executer`) en faisait 85 avec la fenêtre ; extraite (`faire`, un seul `match` sans joker,
  et les signets en une aide) ; les seize cliquets et les 117 épreuves des commandes, des signets, du
  menu et des liens rejouées ensuite. 7 ignorées (les bancs). Aucune image témoin n'a changé.
  `cargo fmt`, clippy strict à zéro sur tout l'espace de travail.
* **Sabotages : 68 sur 68** — `lien-1-102.json` 23 (la loi, la géométrie et l'opacité, en lumière, le
  survol, les deux voies, le SVG), `lien-2-102.json` 9 (rejoués après l'extraction), `lien-3-102.json`
  36 (21 au noyau : la disposition, la provenance, la fenêtre à côté du document ; 15 à la main). Une
  garde sans effet retirée au lieu d'être sabotée (deux fois) ; sept comportements sans épreuve
  trouvés en cherchant le sabotage qui passerait, et éprouvés.
* **Regardé au pilote**, dans le vrai Glucose sans fenêtre, sur la carte de Wikipédia et ses liens :
  `~/.cache/glucose-bancs/pilote-fenetre/` et `pilote-fenetre2/`. **Rien à son écran.**
