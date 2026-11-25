# 151 — Ce qu'on glisse avance de pixels entiers (25/11 vers 1 h, le relais, compte a)

## 1. Sa note : la lenteur quand on déplace une membrane

*« beaucoup de fois j'ai de gros gros problème de perf même sur des très bons PC juste parce que je
[…] modifie de place une membrane »* (`text-w89o-2380`, 24/11 ; la bulle 🔍 `revue-code-1`). Son écran
ici : LG 2560 × 1440 à 165 Hz, échelle 1, sous niri — 6 ms par image.

## 2. Mesuré au pilote, sur une copie de son document

Une copie de `GLUCOSE PROJET.glucose` (`~/.cache/glucose-bancs/membrane-lente`), le vrai Glucose
optimisé, la membrane « 4 · Les codes de partage » glissée par son titre, de près, 3 s aller, 3 s retour :

| écran | la main | glisser : médiane / p90 / p99 / pire | dont textures (médiane) |
|---|---|---|---|
| 1920 × 1080, échelle 1 | positions entières | 2,44 / 5,79 / 8,19 / 10,0 ms | 0,01 ms |
| 3840 × 2160, échelle 1,5 | positions fractionnaires | 6,89 / 9,74 / 11,59 / 19,8 ms | **3,44 ms** |
| 3840 × 2160, échelle 1,5 | pas entiers en points (3 pixels) | — | 0,03 ms |

**La cause** : une position fractionnaire de la main pose ce qu'on glisse à une autre phase
sous-pixel à chaque pas ; la phase entre dans l'empreinte d'un composant (`renderer/composants.rs`,
au bit près), donc chaque texte emporté par la membrane se rendait de nouveau, à chaque image. Sous
Wayland, la main est fractionnaire même à 100 % (l'accélération de libinput, `wl_fixed`) : c'est son
cas ici. Le pilote, lui, pose sa main sur des pixels entiers — c'est pourquoi rien ne se voyait au
banc à l'échelle 1. Sous Windows, le curseur donne des pixels entiers : probablement pas cette cause
là-bas (à confirmer par la télémétrie, qui attend Cloudflare).

## 3. GLISSER-2 — le geste avance de pixels entiers

Le déplacement du geste est arrondi au pixel entier de l'écran où il se fait, avant l'aimant
(`interactions/drag.rs`, `au_pixel_entier`) : la phase reste celle du départ, la texture celle qu'on
avait, et la main n'a jamais plus d'un demi-pixel d'avance. **Écarté** : traiter ce qu'on glisse
« en mouvement » (phase nulle, posé en bilinéaire, comme la caméra) — un demi-pixel de flou sur le
texte pendant tout le geste, et l'ensemble de ce qui bouge à porter dans le `Regime` ; l'arrondi à la
source ne refait rien et reste net.

**Après**, même geste fractionnaire en 4K à 150 % : **4,10 / 6,89 / 11,59 / 14,5 ms, textures 0,02 ms**
(au lieu de 6,89 / 9,74 / 11,59 / 19,8 et 3,44). Épreuve
`test_glisser_2_une_main_fractionnaire_avance_de_pixels_entiers` (zoom 1,37, cinq pas fractionnaires :
écart entier, phase gardée) ; 2 sabotages sur 2 (`outils/sabotages/glisser-2.json`).

## 4. Reste

* **Le p99 à 11,6 ms en 4K** : quelques images refont encore des textures (p99 5,75 ms) — l'aimant qui
  pose la sélection sur une nouvelle phase, ou le passage de 1 024 pixels (fiche 109) ; et `relever`,
  1,7 ms de médiane, à lire.
* **Pas vu à son écran**, avec sa vraie souris : la ligne à essayer est dans ÉTAT.
* Sous Windows, la cause de sa lenteur reste inconnue si elle y est.

## 5. TRONQUER-1 — un long résultat de Ctrl+F tient dans sa case

Vu au pilote le 24/11 (`revue-code-1`) : dans Ctrl+F, la ligne d'un long résultat sortait de sa case
des deux côtés. Une réponse de la question (`ui/question.rs`) se centre dans sa rangée ; plus large
qu'elle, son bord gauche passait sous celui de la carte. **Une seule loi** : `renderer/wrap.rs`,
`tronquer` — la ligne s'arrête au dernier caractère qui laisse place à « … », linéaire comme le
découpage (chaque caractère mesuré une fois). Le Storyboard en avait une copie à lui, qui mesurait la
chaîne entière à chaque caractère retiré (quadratique) : elle appelle désormais la même. Épreuves
`test_tronquer_1_…` (la loi ; une réponse de 400 caractères au téléphone tient dans sa case, « Annuler »
reste entier) ; 3 sabotages sur 3 (`outils/sabotages/tronquer-1.json`). Pas regardé au pilote.

## 6. Les flèches qui traversent leur propre texte — reproduites, pas comprises

Sur une copie de son document du 25/11 (pilote, `voler 2176 1700 0.9` dans un scénario passé par
`GLUCOSE_SCENARIO=` à `piloter.sh demarrer`) : du nœud « URGENT IMPORTANT » (`text-f5fn-345`), les deux
flèches qui partent vers le bas à gauche (`arrow-g420-2318`, `arrow-w89o-2376`) **commencent dans le
texte**, à la hauteur de la deuxième ligne (probablement un passage : `glucose lire` ne montre pas les
passages d'une flèche), et en sortent par le coin bas-gauche avec un coude ; celles qui partent à gauche,
ancrées à un passage, sortent bien par le côté gauche à sa hauteur (FLECHE-6 : le côté qui fait face, à
la hauteur du passage, **seulement si la cible est à côté** — vers le bas à gauche, la règle ne joue pas
et l'ancre, au milieu de la carte à la hauteur du passage, est dedans). **Indice** : `glucose lire` (le noyau, sans obstacles ni polices) donne
leur enveloppe à partir du bas du nœud (y = −1 246, son bas est à −1 258) ; l'application, elle, part de
l'intérieur — c'est donc le contournement de l'application (`Noeuds::obstacles`, `itineraire`) qui pose
une première étape dans la boîte de la source, ou `perimeter_exit` qui rend l'ancre elle-même (t < 0).
**La suite** : une épreuve qui rejoue cette géométrie avec les `Noeuds` du rendu (`renderer/arrow`), lire
les étapes rendues, puis corriger à la source.

**Essayé ensuite (0 h 58)** : la même géométrie en épreuve (`renderer/arrow/ancres_tests.rs`, sa carte
de 451 × 188,8, un nœud 30 unités dessous, une cible loin en bas à gauche, le passage « surtout »), par
`path_with` et les `NoeudsDuRendu` **sans mémoire d'itinéraires** (`contournement: None`) : le départ est
**au bord gauche**, à la hauteur du passage, puis le coin du nœud du dessous — juste. Le défaut vu au
pilote vient donc de ce qui diffère dans l'application : la mémoire des itinéraires
(`contournement: Some`, une route retenue d'avant ?), la flèche **courbe** (`est_courbe`, `resserrer`),
ou la boîte vivante (`NoeudsDuRendu::vivante`). L'épreuve, qui ne reproduisait rien, n'est pas gardée.

**Compris (25/11, 1 h)** : aucun passage ; une seule des deux est fausse, **la courbe** — la spline uniforme bouclait à travers le texte. Réparé : fiche 152 (COURBE-1).
