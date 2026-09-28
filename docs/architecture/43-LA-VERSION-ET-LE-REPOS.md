# 43 — La version, et le repos

> **Rôle de ce document.** La session du 28/09 au soir a repris exactement où la fiche 42
> s'arrêtait : ses points ouverts (§ 8.1), puis ce que sa dernière chronique dit du repos.
> **Écrite au fil de l'eau.**
>
> **Date** : 2026-09-28 · commits `0c0d813` et suivants.
> **État au départ** : `e124ac4`, 1 876 épreuves vertes, clippy strict à zéro, arbre propre —
> vérifié.

---

## 1. Ce qui était sur la table

* **Son compte rendu** des essais de la fiche 42 (le contournement, l'interligne des formules) :
  pas encore arrivé.
* **Sa chronique du 28/09 à 18 h 20** (100 s, sa session des formules), que personne n'avait lue :
  **51 images par seconde**, 41 % des images au-dessus du plancher de 10 ms. Les plus lentes
  (38 à 46 ms) n'ont **qu'un seul nœud à l'écran** : `docks` 12-13 ms, `blit` ~10 ms,
  `soumettre` ~9 ms. C'est la troisième chronique qui le montre (25/09, 26/09, 28/09). § 5.
* Les points ouverts de la fiche 42 § 8.1 : l'écart d'un amas lointain par sa boîte englobante
  (§ 3), la colonne de deux cents cartes (§ 4), l'oubli de tous les itinéraires à chaque version
  (§ 2 et § 4).

---

## 2. Un clic publiait une version (VERSION-1, `0c0d813`)

Trouvé en lisant ce qui fait avancer la version, pour l'oubli des itinéraires : `end_live_edit`
la faisait avancer **même quand le geste n'avait rien écrit**. Un simple clic sur un nœud ouvre
un geste à l'appui (pour un glisser éventuel) et le ferme au relâchement : il publiait une
version. La documentation de `begin_live_edit` disait l'inverse — *« un geste resté immobile ne
la touche pas »*.

Ce que la version pilote, et qui se refaisait donc **à chaque clic** : l'état « modifié » du
document (SAVE-2), l'index spatial, le cache des panneaux, la minimap, les teintes, les domaines,
le focus, les cibles de l'aimant, et les itinéraires de **toutes** les flèches.

**La règle retenue : la version avance exactement quand le journal publie.** `Journal::record` et
`Journal::end` disent s'ils publient une transaction ; le magasin ne publie une version que
dans ce cas. Un geste vide, une édition seule qui ne change rien (renommer un onglet de son
propre nom), un geste en bloc qui ne déplace rien : aucune version.

**Ce qui le tient** : un clic par le vrai chemin de la souris (`test_live_5`) ; les trois cas
dans le noyau. Cinq sabotages tombent — le cinquième passait d'abord : le geste en bloc sans
effet n'avait pas de cas ; `move_selected` sans rien de sélectionné lui en donne un.

---

## 3. L'écart d'un amas lointain, mesuré

La fiche 42 § 8.1 : dans `se_voient`, un amas que le segment n'approche pas s'écarte par sa
boîte englobante ; le sabotage ne le voyait pas, et le chronomètre ne tranchait pas.

**Le chronomètre ne tranchera jamais sur cette machine.** Six scènes par le vrai index (un mur
de 20 × 20 et de 100 × 100 photos, deux mosaïques de murs séparés par des couloirs, deux
colonnes de cartes), vingt et une passes chacune, trois fois en alternance : la **même** scène
varie du simple au triple d'une passe à l'autre (le mur de 20 × 20 : 90 à 287 µs).

**Compter le travail tranche.** Les tests de boîtes faits par la visibilité, pour un même
itinéraire :

| scène | avec l'englobante | sans |
|---|---:|---:|
| un mur (un seul amas) | identique | identique |
| mosaïque de 3 × 3 murs de 6 × 6 photos | 1 176 | 1 764 |
| mosaïque de 5 × 5 murs | 2 814 | 5 334 |
| colonne de 50 cartes séparées | 10 550 | 28 504 |

Dès qu'il y a plusieurs amas — ses tableaux : des groupes de photos, des cartes éparses —, l'écart
divise ce travail par 1,5 à 2,7. **Gardé.** Son épreuve compte le travail d'un même test de
visibilité à côté d'un mur de dix blocs de côté puis de vingt : il est le même. Sans l'écart, il
suivrait le pourtour du mur. Deux sabotages tombent.

Les scènes restent dans le dépôt (`arrow_edit/mesures_tests.rs`, ignorées par défaut) :
`cargo test --release -p glucose-desktop --lib mesure_des_scenes -- --ignored --nocapture`.

**La colonne de deux cents cartes, mieux comprise.** Compter le travail la décrit mieux que la
fiche 42 : 794 sommets (deux cents cartes séparées, quatre coins chacune), et la source et la
cible au milieu de la colonne rendent ses **deux côtés exactement aussi courts** — l'A* les
explore tous deux, soit environ huit cents développements de huit cents candidats chacun. Le
coût est la boucle des candidats, O(n²), plus que la visibilité. Aucune des pistes regardées
(ne tester la visibilité qu'au dépilement, filtrer les arêtes bitangentes) ne descend sous
O(n²) : il faudrait un balayage angulaire (Lee) ou un maillage de l'espace libre (Polyanya).
Pas fait ; la colonne de cinquante cartes, elle, se contourne en 0,4 ms.

---

## 4. Le repos : ce qui partait vers la carte (ENVOI-1, `2382f79`)

### 4.1 Sa chronique, relue

Ses deux dernières sessions finissent de la même façon : des images de 30 à 46 ms **avec un seul
nœud à l'écran**, `blit` ~10 ms, `soumettre` ~9 ms, `docks` ~12 ms quand un panneau se refait.
Le 26/09, c'était la fin de son essai n° 8 — les jalons datés, **la Time Machine ouverte**.

### 4.2 Une hypothèse démentie

Le panneau de la Time Machine prend toute la hauteur ; la couche du dessus part par lignes
entières (BANDE-1) ; recomposé à l'identique à chaque image, il en fait partir **toutes** les
lignes, 11 Mo. J'ai d'abord cru que le volume faisait le coût. `bench_envoi` (hors écran) l'a
démenti : envoyer ces 11 Mo coûte **0,5 à 1,2 ms** sur ses deux cartes, avec le réglage de
mémoire par défaut.

### 4.3 La cause : l'allocateur au plus juste

L'application ouvre sa carte avec `MemoryHints::MemoryUsage` (ETAGES-1 : il ne réserve rien
d'avance, et rend la mémoire graphique aux applications d'à côté). wgpu règle alors les blocs de
mémoire visible du processeur à **quatre mébioctets**. `Queue::write_texture` range chaque envoi
dans un tampon de transfert neuf : tout envoi plus gros qu'un bloc reçoit une **allocation
dédiée**, créée puis rendue au pilote à chaque fois. Sous ce réglage, sur sa RTX, les mêmes 11 Mo
coûtent **5,9 ms**, et un niveau de photo neuf de 2,3 Mpx, 3,4 ms — ce qui pourrait être une part
des pics de `textures` en zoomant (16 à 23 ms, fiche 38 § 1), à confirmer par sa chronique.

### 4.4 Le remède : des tampons qui restent

`present/envoi.rs` : tout ce qui part vers la carte — les deux couches, les photos et les
composants, l'image entière de la voie processeur — passe par des tampons de transfert qui
**restent** d'une image à l'autre (le `StagingBelt` de wgpu, sans taille de tranche choisie :
chaque tampon a la taille de ce qu'il a porté, et resert dès que la carte l'a lu). Les copies
partent d'un bloc, avant l'image qui s'en sert. Ces tampons vivent dans la mémoire du système :
la raison d'ETAGES-1 tient.

| `bench_envoi`, réglage de l'application | `write_texture` | tampon qui reste |
|---|---:|---:|
| RTX 5070 — la couche entière, 11 Mo | 5,92 ms | **1,64 ms** |
| RTX 5070 — un niveau de photo neuf, 9 Mo | 3,43 ms | **0,97 ms** |
| RTX 5070 — quatre bandes (la chrome sans panneau) | 0,72 ms | 0,95 ms |
| Arc 140T — la couche entière | 0,76 ms | **0,48 ms** |
| Arc 140T — un niveau de photo neuf | 0,88 ms | **0,36 ms** |

Sur la vraie chaîne (RTX, 2 160 × 1 350, la vue qui glisse, la Time Machine ouverte), une image
passait de **13,4-13,9 ms à 5,2-5,6 ms** ; `soumettre`, de 6-6,7 à 0,35-0,41 ms.

**Ce qui empire, et je le dis** : sur la RTX, écrire dans ces tampons est environ 30 % plus lent
que dans ceux de wgpu (une question de mémoire, pas d'allocation — un seul tampon par couche n'y
change rien). Une image ordinaire sans panneau paie **+0,2 ms** (≈ 2,5 → 2,7 ms). Je l'ai gardé :
le tempo suit le p99, et ce sont les pires images qui tombent. Le vrai remède aux envois répétés
est ailleurs — ne pas renvoyer une chrome qui n'a pas changé (§ 5).

**Ce qui le tient** : un rectangle envoyé au milieu d'une texture aux rangées non alignées, lu
dans une source plus large, arrive exactement à sa place, et rien d'autre ne bouge. Six
sabotages tombent (le pas de la source, l'origine, la scène ou les couches qui n'envoient rien,
le harnais des deux voies qui oublie de soumettre). **Pas éprouvable hors fenêtre** : l'oubli de
la soumission dans la présentation elle-même.

### 4.5 Une faute de méthode, la mienne

Pour mesurer la vraie chaîne, j'ai écrit un banc qui ouvrait une fenêtre de Glucose, et je l'ai
lancé **neuf fois** sur son écran pendant qu'il travaillait, sans le prévenir. Il l'a arrêté :
*« c'est pas comme ça qu'on fait les tests d'habitude »*. Le banc est retiré du dépôt ; les
mesures se font hors écran, et ce qui exige la vraie fenêtre, c'est lui qui le lance. Ses
réglages n'ont pas été touchés : `carte.txt` a été réécrit à 20 h 49 par sa propre session,
ouverte depuis 18 h 18 — mes lancements ont eu lieu à 19 h 44 et entre 22 h 23 et 22 h 27.


## 5. Les panneaux, posés par la carte (PANNEAUX-1, `e78ba98`)

Même après ENVOI-1, un panneau **immobile** coûtait encore à chaque image : recomposé dans la
couche du dessus depuis son cache (`bench_chrome` : 1,10 ms pour la Time Machine à l'échelle 1,
environ 2,25 fois plus à 150 %), puis ses lignes effacées, relevées et renvoyées — toute la
hauteur de l'écran pour la Time Machine. Rien de cela ne changeait d'une image à l'autre.

**Ce qui est fait** : sur la voie graphique, les panneaux deviennent des **textures que la carte
pose après la couche du dessus** — l'ordre même de la voie processeur, qui les composait en
dernier. Le cache des panneaux tient chacun à jour sans le composer ; la clé de sa texture
change à chaque redessin, et à lui seul. La scène de la carte, qui posait déjà photos et cartes
de texte avec leurs clés, leur cache et leur budget (CASCADE-2), pose une troisième tranche.
Un panneau immobile ne repart donc jamais ; un panneau qui change ne renvoie que son rectangle
(2,3 Mo pour la Time Machine, contre 11). Un tampon refusé retombe sur le dessin dans la couche,
comme avant : rien ne disparaît.

**Ce qui le tient** : la couche du dessus avec la Time Machine ouverte est, **au bit près**,
celle d'une image sans panneau, et ses bandes aussi ; un panneau immobile garde sa clé, la vue
qui glisse n'y change rien, un réglage du panneau en donne une neuve et l'ancienne ne désigne
plus rien ; la carte pose la Time Machine sur la scène témoin et rend l'image de la voie
processeur, dans les bornes de la scène seule (pire écart 4 niveaux, celui de la scène). Six
sabotages tombent, dont les panneaux posés avant la couche du dessus.

**Pas éprouvable hors fenêtre** : le branchement de la source de leurs pixels dans la
présentation elle-même. **Pas mesuré sur la vraie chaîne** : je ne lancerai plus de fenêtre sur
son écran ; sa prochaine chronique le dira (`docks`, `effacer`, `relever` et `blit` avec un
panneau ouvert).

**Ce qui reste** : un panneau qui se **redessine** (la souris dedans, la réglette qu'on tient)
coûte toujours son dessin au processeur — 2,5 ms pour la Time Machine à l'échelle 1 au banc,
12-13 ms dans sa chronique. Et sans panneau, la couche du dessus renvoie encore la barre, les
onglets et la minimap à chaque image (~784 lignes à 150 %), qui ne changent pas davantage.


---

## 6. L'histoire qui écrivait chaque mouvement (FONDRE-1 `dbedcd5`, GLISSER-1 `6e8ed1f`)

Son registre, entrée 12 : *« elle enregistre beaucoup de choses inutiles »*. La fiche 41 § 11
l'avait vu : un geste continu écrivait dans le document **à chaque mouvement de la main**, et
chaque écriture entrait dans la pile d'annulation et dans l'histoire du fichier. Deux formes,
deux remèdes, tous deux **exacts**.

### 6.1 Les éditions qui portent des valeurs se fondent (FONDRE-1)

Redimensionner une carte cinq secondes, c'était plus d'un millier d'éditions de la **même**
case, chacune portant le nœud entier avant et après — texte compris. À la fermeture d'un geste,
deux éditions **adjacentes** de la même liste, à la même case, se fondent quand la seconde part
exactement de là où la première arrive : l'avant de l'une, l'après de l'autre. Ces éditions
portent des valeurs, donc la composée est exacte au bit près, quelle que soit la paire
(changer puis changer, insérer puis changer, changer puis retirer, retirer puis réinsérer à la
même place). Ce qui ne change plus rien disparaît. Adjacentes seulement : une insertion
ailleurs dans la liste, entre deux, décalerait les rangs (JRN-2).

**Une conséquence, dite** : restaurer dans la Time Machine un point du passé **identique** au
présent ne publie plus de geste — les gestes retournés se fondent en rien. `Ctrl+Z` défait
alors le vrai geste précédent, comme après un clic. L'épreuve de la Time Machine supposait
l'inverse ; elle dit désormais cette règle.

### 6.2 Un glisser s'écrit en une translation (GLISSER-1)

Une translation porte un **pas**, pas une valeur : additionner des pas en virgule flottante ne
redonne pas la position qu'ils ont laissée — `(x + a) + b` n'est pas toujours `x + (a + b)`.
Fondre les pas d'un glisser en leur somme aurait écrit un fichier qui, relu, pose les nœuds un
ulp à côté de l'écran. L'épreuve le montre : soixante mouvements fractionnaires rejoués pas à
pas posaient la carte et la flèche ailleurs que l'écran.

Le glisser relève donc **une fois**, au premier mouvement, ce que la sélection emporte et d'où
chaque chose part (le parcours du tableau entier se faisait à **chaque** mouvement — c'est une
part du chantier n° 5, faite au passage). À chaque mouvement, chaque position devient son
départ plus le déplacement **total** — l'addition même que la translation du journal fera en
rejouant. À la fermeture du geste, ses pas se remplacent par une seule translation, du départ à
l'arrivée. Un glisser revenu à son point de départ ne laisse rien ; `Échap` rend chaque départ
exact. La constante `1e-7` du glisser disparaît : un pas nul se reconnaît exactement.

Pendant le geste, les pas restent écrits : ce qui suit le geste en cours — l'écran, les
itinéraires des flèches (GESTE-1, FLECHE-5) — les lit pour savoir ce qui a bougé.

### 6.3 Ce qui les tient

FONDRE-1 : cent changements d'une case en font un ; ce qui ne change rien disparaît ; seules des
voisines qui s'enchaînent se fondent (une autre case, un autre tableau, un avant qui ne suit
pas) ; une translation jamais ; le geste publié rejoue exactement le geste ; un
redimensionnement du magasin s'écrit en une édition. Six sabotages tombent — dont un d'abord
aveugle (une valeur entière fondue sans s'enchaîner), complété.

GLISSER-1 : la translation qui rejoue l'écran au bit près, avec la preuve que l'ancienne façon
s'en écartait ; le retour au départ sans trace ; `Échap` exact ; deux glissers dans un même
geste ; un glisser à la vraie souris — quarante mouvements, une édition. Neuf sabotages tombent,
après deux angles complétés (les coudes d'une flèche emportée, la cible d'une flèche qui suit).

**Ce qui reste** : défaire un glisser (`Ctrl+Z`) applique la translation opposée, `(x + T) − T`,
qui peut laisser un ulp — comme avant ; seul `Échap` est exact. La rotation et le recadrage,
eux, portent des valeurs : FONDRE-1 les couvre déjà.

---

**Retour** : [`00-INDEX.md`](00-INDEX.md)
