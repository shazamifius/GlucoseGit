# 152 — La courbe centripète : une flèche courbe ne boucle plus dans sa source (25/11 vers 1 h, le relais, compte a)

## 1. Sa note, et ce que la fiche 151 § 6 croyait

Du nœud « URGENT IMPORTANT » (`text-f5fn-345`), deux flèches partent vers le bas à gauche
(`arrow-g420-2318` vers `text-f5fn-369`, `arrow-w89o-2376` vers `relais-bugs-liste`) et semblaient
**commencer dans le texte**, à la hauteur de « mais surtout ». La fiche 151 § 6 supposait un passage :
il n'y en a pas. Lues dans une copie de son document (un exemple jetable de `glucose-core` sur
`persist::decode`, retiré) : ni passage, ni coude ; **la première est courbe** (`arrowType: "curved"`),
la seconde droite.

## 2. La cause : la spline uniforme sur des tronçons inégaux

Le contournement (FLECHE-5, avec ses obstacles) rend les mêmes points aux deux : la sortie sous la carte
(−1 555, −1 252), le coin du nœud du dessous **38 unités** plus loin, puis **1 950** d'un coup. La droite
les suit : juste. La courbe passait par eux en Catmull-Rom **uniforme** (les nombres de Tauri) : au coin,
la tangente vaut la moitié de la corde qui l'enjambe — près de 1 000 unités pour un tronçon de 38. Le
second point de contrôle tombait 260 unités **au-dessus** du coin, au-delà du haut de la carte : la courbe
remontait à travers le texte jusqu'à sa troisième ligne, puis redescendait. Vu au pilote sur une copie
(`voler 5100 4500 3`) : les « deux » flèches dans le texte étaient les deux bras de cette boucle.

`resserrer` (qui tend une courbe là où elle mord un bloc) ne pouvait pas la voir : la source et la cible ne
sont pas des obstacles (FLECHE-7).

## 3. COURBE-1 — chaque tronçon dure la racine de sa longueur

Cherché (sources) : Yuksel, Schaefer et Keyser, *Parameterization and applications of Catmull–Rom
curves*, Computer-Aided Design 43 (2011) — <https://people.engr.tamu.edu/schaefer/research/cr_cad.pdf> ;
<https://cemyuksel.com/research/catmullrom_param/>. La paramétrisation **centripète** (α = ½) est la seule
de la famille sans rebroussement ni boucle dans un tronçon, et elle suit ses points de plus près.

`arrow/trace.rs` : la tangente en chaque point est celle de la pyramide de Barry et Goldman (les vitesses
des deux tronçons, moins celle qui les enjambe), chaque tronçon durant `√longueur` ; la cubique d'Hermite
devient Bézier par le tiers de la durée. **Aucune constante** : un tronçon nul vaut sa limite (la vitesse
nulle), pas un seuil. **À pas égaux, ce sont exactement les nombres de Tauri** (épreuve) : seules les
courbes qui bouclaient changent. Sur sa flèche : le premier morceau reste sous la carte (son point le plus
haut à −1 250, le bas de la carte à −1 258).

**RESSERRER-1**, trouvé en chemin : quand le morceau fautif était **le premier**, `resserrer` cherchait son
voisin d'avant au rang `i.wrapping_sub(1)` = `usize::MAX` — un plantage (débordement, ou index hors
bornes en version optimisée). L'uniforme ne mordait jamais par son premier morceau dans l'épreuve ; la
centripète, si : le plantage est sorti. Les voisins se prennent maintenant de `i − 1` (saturé) à `i + 1`.

## 4. Preuves

* Épreuves : `trace/tests.rs` — la boucle de sa flèche (ses points, 1 000 échantillons par morceau hors de
  sa carte), les nombres de Tauri à pas égaux, un point doublé reste fini ; `contour/tests.rs` — la courbe
  qui mord se resserre, scène refaite (la centripète ne débordait plus au-delà de l'épingle : 2 unités
  contre 125 ; elle s'écarte de 13 avant elle, sur un bloc posé 8 unités dessous — c'est son premier
  morceau qui mord, le cas qui plantait).
* `outils/sabotages/courbe-152.json` : **5 sabotages sur 5 tombent** (l'uniforme, la division par zéro, le
  tiers, la corde qui enjambe, `wrapping_sub`).
* Les empreintes témoins (`bench/tests.rs`) : 59 pixels, tous sur la flèche courbe ; regardée avant/après,
  agrandie quatre fois — la même arche, à peine plus régulière à droite.
* Suite entière : 3 681 épreuves vertes ; clippy propre.
* Au pilote, sur une copie de son document, version optimisée : les deux flèches sortent par le bas de la
  carte, la courbe s'infléchit au coin du nœud du dessous ; plus rien ne traverse le texte.

## 5. Reste

* Son écran : la flèche courbe de « URGENT IMPORTANT » (après la prochaine relance de son Glucose).
* La centripète n'interdit pas qu'une courbe **frôle** sa source quand ses premiers points y collent ; si
  cela se voit encore, la règle serait que `resserrer` regarde aussi la source et la cible **hors** de
  leur premier et dernier tronçon — pas avant d'en avoir un cas.
