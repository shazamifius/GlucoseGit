# 101 — Le bas de l'écran : rien ne sort, rien ne couvre la minimap

> Session du 10/11/2026, fin d'après-midi, juste après la fiche 100. Ce que l'essai sous niri à
> l'échelle 2 a montré en passant (fiche 100 § 10) : dans une fenêtre étroite, la barre de couleur
> chevauche la minimap. Regardé à la taille de son téléphone, c'était pire.

## 1. Ce qu'on voyait, à la taille de son téléphone

Le pilote sait maintenant prendre la taille d'un écran (`PILOTE_ECRAN=1080x2400 PILOTE_ECHELLE=2.75`,
son moto g54 ; § 5). Un texte choisi, à 393 points de large
(`~/.cache/glucose-bancs/bas-de-l-ecran/avant-moto-g54.png`) :

* **la barre de couleur sortait de l'écran des deux côtés** : 421 points pour 393 — « Couleur »
  devenait « ouleur », la case de la roue était coupée à droite ;
* **la barre de couleur et la barre d'action couvraient la minimap**, qu'on ne pouvait plus toucher.

Et l'épreuve écrite ensuite (§ 4) a trouvé plus grave : **la barre des flèches mesurait 699 points
même sans les mots des relations** — sur un téléphone, presque deux fois l'écran. Et une sélection
de photos et de textes donnait une barre d'action de 510 points (« Origines individuelles »).

## 2. La recherche, et le parti pris

Deux questions, deux réponses des guides d'interface :

* **Un élément permanent et une barre passagère se rencontrent en bas** : Material 1 faisait
  *monter* le bouton flottant devant la barre passagère ([mdui, Material 1][fab]) ; ses règles
  d'aujourd'hui déconseillent d'animer les autres éléments avec elle ([material.io][snack]).
* **Une barre manque de largeur** : les boîtes à outils proposent un menu de débordement ou une
  rangée qui défile ([Telerik][kendo], [Firefox Acorn][acorn]). Glucose refuse les deux : *aucun
  bouton derrière un défilement* (la compétence du téléphone), *tous les boutons obligatoires* (sa
  parole) — un bouton caché n'existe pas pour qui ne le cherche pas.

D'où deux lois, dans un seul module (`ui/bas.rs`) :

* **La minimap ne bouge jamais** : elle est là tout le temps, la main sait où elle est. Une barre,
  qui naît et meurt avec la sélection, **monte au-dessus d'elle** quand elle la rencontrerait
  (`bas::poser`), et celles qu'elle porte montent avec elle.
* **Une barre ne sort jamais de l'écran : ce qui ne tient pas passe à la ligne, comme un texte**
  (`bas::ranger`). La barre d'action et la barre de couleur élément par élément ; la barre des
  flèches **groupe par groupe** — « Épaisseur 1 2 3 5 » ne se coupe pas, et un filet de séparation
  n'ouvre jamais une rangée. Elle garde son ordre de préférence (BADGE-2) : les mots des relations
  sur une rangée si elle tient, sinon les sigles sur une rangée, et seulement alors à la ligne.

**Aucune constante neuve.** Le bord est celui de la minimap (`minimap::BORD`, fiche 06 § 9) ; l'écart
entre deux barres, `bas::AU_DESSUS` — il existait **trois fois**, en copies (la couleur, les flèches,
la roue), et n'existe plus qu'une.

## 3. Deux défauts trouvés en chemin

* **Un bouton à deux états bougeait sous la main.** Basculer « Origine commune » / « Origines
  individuelles » (ou « Verrouiller » / « Verrouillé ») changeait la largeur du bouton, et la barre,
  centrée, se recentrait : le bouton glissait sous le curseur. Et avec le passage à la ligne, la
  hauteur de la barre aurait dépendu de l'état — or la barre de couleur et celle des flèches se
  posent sur une barre d'action calculée « Origine commune ». **Un bouton à deux états a désormais
  la largeur du plus long de ses deux mots** : rien ne bouge quand on bascule, et la hauteur de la
  barre ne dépend plus de ce qu'il montre. Seul changement visible sur un bureau : « Origine
  commune » est un peu plus large.
* **L'écran visible oubliait la navigation de droite.** `UiState::ecran_visible` ne retirait que le
  bas, alors que sa propre documentation y posait la minimap, qui retire aussi la droite (un
  téléphone couché, sa navigation à droite). Les barres auraient cherché la minimap où elle
  n'était pas. Il retire maintenant les deux, et la minimap et les barres lisent la même emprise
  (`minimap::emprise`) : une seule géométrie.

## 4. Prouver

* **Une propriété** (`ui/bas/tests.rs`, BAS-1) : de **320 à 2 560 points**, aux échelles **1, 1,25,
  2 et 2,75**, pour trois sélections (un texte ; douze photos et trois textes ; une flèche et un
  texte), chaque barre tient entre les bords, sous le haut de l'écran, hors de la minimap et des
  autres barres ; chaque bouton dans sa barre, sans en chevaucher un autre. C'est elle qui a
  trouvé la barre des flèches de 699 points.
* BAS-2 : **sur un bureau, rien ne change** — une rangée, la barre d'action à 14 points du bas, la
  barre de couleur 8 points au-dessus. BAS-3 : sur son téléphone, la couleur sur deux rangées,
  « Couleur » sur la première, au-dessus de la minimap. BAS-4 : basculer ne déplace aucun bouton.
  BAS-5 : ranger, comme un texte. BAS-6 : couché, la navigation à droite, les barres évitent la
  minimap **réelle**. BAS-7 : aucun filet n'ouvre une rangée.
* **Sabotages** (`bas-101.json`) : dix sur dix (§ 6).
* **Regardé** à la taille de son moto g54 (1 080 × 2 400 à 2,75), du Redmi 9 (720 × 1 600 à 2,
  360 points) et d'un bureau : `~/.cache/glucose-bancs/bas-de-l-ecran/apres-*.png`. Au téléphone, les
  trois barres s'empilent au-dessus de la minimap — la couleur sur deux rangées, les flèches sur
  trois (quatre à 360 points), l'action sur deux ; rien ne sort, rien ne couvre. Au bureau, tout
  est comme avant.
* **Les témoins** : deux empreintes ont changé (la scène sélectionnée, le menu). Les images d'avant
  (le commit envoyé, dans un arbre temporaire) et d'après, côte à côte : **5 935 pixels, tous dans
  la barre d'action** (645-1471 × 810-887) — « Origine commune » à la largeur des deux états, la
  barre un peu plus large et recentrée ; les six autres témoins identiques au pixel près. Regardés,
  puis mis à jour. Et le cliquet des 80 lignes a demandé une extraction (`haut_de_ce_qui_porte`).

## 5. Le banc a appris

`PILOTE_ECRAN=<l>x<h>` et `PILOTE_ECHELLE` valent aussi sous sway : le pilote rend Glucose à la
taille d'un téléphone, et le pointeur vise les pixels de cet écran. **Regarder un téléphone sans
téléphone**, comme la compétence le demande — mais avec la main, pas seulement une image fixe.

## 6. Le compte, et ce qui reste

**Le compte** : la suite complète, **3 039 épreuves, aucune tombée** ; clippy strict et `cargo fmt` à
zéro ; **sabotages 10 sur 10** (`bas-101.json` : la montée au-dessus de la minimap, le passage à la
ligne de `ranger` et de chacune des trois barres, les deux bascules, le mot « Couleur », le filet en
tête de rangée, la navigation de droite), les deux de la couleur rejoués après l'extraction.

**Reste** :

* **au vrai téléphone** : la même scène sur l'APK — le chemin d'Android pose les barres par le même
  code, mais ses marges (la navigation, le clavier) ne se voient qu'à l'écran ;
* la pile des trois barres est haute sur un téléphone (presque un tiers de l'écran avec une flèche
  choisie) : c'est le prix de *tous les boutons visibles*. Si cela gêne, la réponse est à lui
  (replier la barre des flèches derrière un geste ? la garder entière ?) — pas à trancher ici.

[fab]: https://www.mdui.org/en/design/1/components/buttons-floating-action-button.html
[snack]: https://material.io/components/snackbars
[kendo]: https://www.telerik.com/kendo-angular-ui-develop/components/toolbar/floating-toolbar/responsive
[acorn]: https://acorn.firefox.com/latest/mobile/components/floating-toolbar/android-muQbnzSr
