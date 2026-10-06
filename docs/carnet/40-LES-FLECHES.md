# 40 — Les flèches

> **Rôle de ce document.** Son essai de la fiche 38 : *« absolument TOUT fonctionne enfin je
> crois »*. Sur les membranes, il n'a pas tranché ; il a donné à la place le relevé de tout ce
> qui ne va pas dans Tauri (fiche [`39`](39-LE-REGISTRE-DE-TAURI.md)), et proposé : *« au lieu
> d'attaquer les membranes, revoir COMPLÈTEMENT les flèches actuelles, et revoir même leur
> design, car sur Tauri elles sont trop belles »*. Cette fiche dit ce que les flèches sont
> devenues : l'aspect de Tauri, peint par une loi au lieu d'un traceur (§ 2), une barre
> d'options (§ 3), et les **ancres de texte** — l'entrée 1 de son registre, le défaut du double
> « bonjours » (§ 4).
>
> **Date** : 2026-09-25 · commits `2f9cd0d` à `1b6b547`, et celui de cette fiche.
> **État vérifié** : `cargo test --workspace` exit 0, **1 740 tests verts**, clippy strict à
> zéro, `cargo fmt --check` à zéro ; après chaque suite, `%LOCALAPPDATA%\Glucose` n'a rien
> reçu.
>
> **Vu à l'écran** : rien de cette fiche encore — seulement des images de rendu, regardées
> agrandies. Le § 8 dit quoi regarder.

---

## 0. En une page

| | ce qui change pour lui | commit |
|---|---|---|
| **Le registre** | ses dix-huit remarques sur Tauri, gardées une par une, avec leur état dans Rust | `2f9cd0d` |
| **FLECHE-1** | une flèche est un fil en **dégradé** de la couleur de sa source à celle de sa cible, avec un **halo**, et une **pastille** au bout ; sélectionnée, blanche | `994c5a9`, `43f85ab` |
| **FLECHE-2** | ce design coûtait 34 ms par image pour 80 flèches ; peint par la loi de son champ, 9,3 ms au processeur — et sur sa machine, c'est la **carte graphique** qui les peint | `994c5a9`, `43f85ab` |
| **FLECHE-3** | une **barre d'options** : droite ou courbe, double sens, épaisseur, relation — pour toutes les flèches sélectionnées | `457d3cb` |
| **FLECHE-4** | une flèche désigne un **passage exact** d'une carte : elle en part, à sa hauteur ; au survol, **lui seul** brille ; il suit quand on écrit ; on le choisit dans la carte même (« Ancrer… ») | `71606ea`, `f8e6a2b`, `1b6b547` |

---

## 1. Son essai de la fiche 38, lu

Sa chronique (`sortie-chronique-2026-09-25-onglets.txt`, 1 402 s, 11 056 images, écran 240 Hz,
fenêtre 2 160 × 1 350 à 150 %) :

* **73 images par seconde** entre deux images consécutives — contre 26 à l'essai précédent. 8 %
  des images dépassent le plancher de 10 ms, contre 74 %.
* **Les membranes ne coûtent plus rien** : 0 ms en médiane, 0,26 au p90.
* Restent : les panneaux, **11,6 ms au p99 au repos** vers la fin de la session (une vingtaine
  d'images, dont on ne sait pas encore ce qu'elles faisaient) ; et les textures, **29 ms au p99
  en éditant du texte** (2,4 Mpx téléversés d'un coup). Ni l'un ni l'autre n'est traité ici.

---

## 2. Le design de Tauri, peint par une loi (FLECHE-1, FLECHE-2)

### 2.1 Ce que Tauri dessinait

`ArrowSvgLayer.tsx`, relu nombre par nombre : un **dégradé** `hsl(h, 80 %, 65 %)` de la teinte de
la source à celle de la cible, par une médiane `hsl(h̄, 85 %, 70 %)` prise par le plus court arc
du cercle chromatique ; un **halo** de `sw + 4` pixels à 18 % (`sw + 10` à 45 % sélectionnée) ; un
**trait** de `sw` (2) à 92 %, blanc sélectionné ; une **pastille** sombre au bout, cerclée de la
teinte d'arrivée (au départ aussi si la flèche va dans les deux sens) ; sélectionnée, deux
disques pleins cerclés de blanc. Le trait garde son épaisseur **à l'écran** quel que soit le zoom
(`non-scaling-stroke`). Tout est dans `glucose_core::arrow::aspect`.

La forme vient de `glucose_core::arrow::trace` : segments, ou courbe de Catmull-Rom (les nombres
de Tauri). **Le clic vise la courbe**, là où Tauri visait la ligne brisée qui la porte ; pour
viser, une courbe s'aplatit par la formule de Wang, dont le nombre de tronçons se **déduit** de
la tolérance voulue (un quart de pixel).

Le témoin gagne une flèche courbe à double sens entre deux cartes de couleurs différentes.

### 2.2 Ce que ce design coûtait, et la loi du champ

`bench_fleches`, 80 flèches, 2 160 × 1 350, voie processeur :

| | les flèches seules |
|---|---|
| l'ancien trait gris | 6,4 ms |
| le design de Tauri par le traceur de `tiny-skia` | **34 ms** |
| le même, par la loi du champ | **9,3 ms** |

Rogner le design aurait été une rustine. Une flèche est un **champ** : en chaque pixel, sa
distance au tracé et sa place le long du dégradé, et la composition du halo, du trait et des
disques (`glucose_core::arrow::champ`). Elle s'évalue seulement là où elle peut laisser de
l'encre, et un coude n'y est jamais couvert deux fois. Le reste des 9,3 ms est la surface : le
halo couvre quatre fois plus de pixels que l'ancien trait. Par pixel, c'est plus rapide qu'avant.

**Sur sa machine, c'est la carte graphique qui peint** (`present::fleches_gpu`) : un quad par
segment et par disque, et chaque pixel n'est écrit que par **un** d'eux — le segment le plus
proche, sinon le premier disque qui le contient. Les flèches ne touchent plus la couche du dessus,
qui partait par bandes de lignes (une diagonale en touche beaucoup pour peu de pixels).

### 2.3 Ce que les épreuves ont trouvé

* Les bornes du peintre étaient des marges de prudence **empilées** : quatre sabotages sur six
  passaient. Toutes se déduisent désormais d'une seule, le demi-pixel du filtre ; huit sur huit
  tombent.
* Sur la carte, un premier test d'appartenance « large » laissait sans écriture un pixel posé sur
  le bord d'un carré : **242 niveaux** d'écart, trouvés par l'épreuve d'accord. La propriété se
  décide par des distances **strictes**.
* L'accord entre les deux voies est d'**un niveau par flèche superposée** — déduit (chaque couche
  s'arrondit une fois de chaque côté) et mesuré (825 pixels à 1 sous une flèche, 2 à 2 sous
  trois).

---

## 3. La barre d'options (FLECHE-3)

Rien, dans Rust, ne permettait de rendre une flèche courbe, à double sens, ni de changer son
épaisseur. Au-dessus de la barre d'action : **Flèche | Droite · Courbe | Double sens |
Épaisseur 1 2 3 5 | les six relations**, par leurs sigles. Elle s'applique à **toutes** les
flèches sélectionnées (Tauri : une seule) ; un réglage s'allume quand toutes le partagent ;
recliquer une relation portée par toutes la retire. Chaque réglage est un geste (`Ctrl+Z`).

---

## 4. Les ancres de texte (FLECHE-4) — l'entrée 1 de son registre

*« Le problème vient du système de sélection, qui prend les mots à part entière et pas leur…
token identitaire. »*

### 4.1 Le module était cassé

Le portage de `textAnchors.ts` mêlait deux unités — la création comptait des caractères, la
résolution découpait des octets — et coupait son contexte « 32 octets avant » **au milieu d'une
lettre accentuée** : un plantage sur le premier texte français. Personne ne l'appelait encore.
Réécrit : tout en octets de la source, rien ne se découpe hors d'une frontière de caractère ; une
position venue de Tauri (qui compte en UTF-16 d'un texte rendu) est simplement fausse, et l'ancre
se retrouve par sa citation.

### 4.2 Une position n'est pas une identité

L'épreuve a retrouvé **son défaut exact**, sous une autre forme : un texte ajouté devant peut
amener le **premier** « bonjours » à la position qu'occupait le second, et le raccourci « la
citation est là » s'en contentait. Chaque occurrence est désormais notée sur son contexte ; la
position ne départage que des ex æquo.

### 4.3 L'ancre suit l'écriture

Écrire une carte fait **suivre** les ancres des flèches qui y désignent un passage, dans le même
geste (`Store::ecrire_le_texte`) : taper avant ou après un mot ancré ne l'allonge pas ; le
réécrire garde l'ancre sur ce qu'on a écrit à sa place ; l'effacer retire l'ancre, la flèche
garde sa carte.

### 4.4 La flèche part du passage, et s'y vise

Comme chez Tauri, une flèche ancrée part de la hauteur du passage. Le noyau **demande** cette
hauteur par un trait (`arrow::Noeuds`) ; le bureau la mesure par la vraie mise en page. Le dessin,
les poignées et le clic passent par le même interlocuteur : on vise la flèche là où elle part
(loi L4).

### 4.5 Au survol, lui seul brille

La souris suit la flèche qu'elle survole (l'arbitre de clic en juge) ; le passage qu'elle désigne
s'éclaire à la couleur de sa carte — fond 20 %, liseré 45 %, lueur floutée de 16 : les nombres de
Tauri. **Un mot répété n'entraîne pas ses homonymes** : l'épreuve regarde l'image, et le fond du
premier « bonjours » ne bouge pas quand le second brille.

### 4.6 L'éditeur : on choisit dans la carte même

Tauri recopiait le texte dans une fenêtre et y faisait sélectionner : deux rendus d'un même texte,
deux espaces de positions — c'est là que ses ancres divergeaient. Ici : **« Ancrer… »** dans la
barre, la caméra vole jusqu'à la carte source, un panneau guide (« Sélectionnez le texte exact ·
Ctrl pour en ajouter ») ; un clic prend un mot, un glisser prend exactement ce qu'il couvre, `Ctrl`
ajoute ; `Entrée` passe à la cible puis termine — un seul geste pour les deux côtés ; `Échap`
laisse tout comme avant.

Trouvé en sabotant : un clic n'importe où sur le canevas choisissait le mot le plus proche de la
carte (la mise en page ramène tout point au caractère le plus proche, ce qui est voulu pour
éditer). Un choix ne commence plus que sur la carte.

---

## 5. Ce qui n'est pas fait, ou pas prouvé

* **Vu à l'écran** : rien de cette fiche.
* **Le contournement des obstacles** : Tauri faisait passer une flèche sans coude **autour** des
  cartes qu'elle traverse (`getDynamicRoute`, un algorithme glouton récursif sur les coins des
  obstacles). Pas encore fait ; les références (Excalidraw, A* sur une grille non uniforme) sont
  en § 9.
* **La description longue** (le badge « i » et son panneau Markdown) et le **portail** vers un
  autre tableau : pas encore faits. Un bouton qui ne ferait rien serait un mensonge ; ils ne sont
  donc pas dessinés.
* **L'étiquette** d'une flèche s'écrit toujours au double-clic ; la barre ne la propose pas.
* Une ancre venue de **Tauri** se retrouve par sa citation et son contexte, qui sont ceux d'un
  texte rendu : sur un texte très formaté, elle peut ne pas se retrouver (elle disparaît alors
  sans briller à tort).

---

## 6. Vu en chemin

* **Les tailles d'écran du canevas sont en pixels physiques.** Les barres suivent les 150 % de
  son écran ; les poignées, les étiquettes, les bandes de clic, le trait des flèches, non. Sur son
  écran, une flèche fait 2 pixels là où celle de Tauri en fait 3, et une poignée est d'un tiers
  plus petite. C'est une part de sa plainte n° 8 (*« il faut viser PILE »*). Les flèches passent
  déjà par la fonction commune ; la corriger là les corrigera toutes.
* **Une carte non mesurée se dessine plus haute que sa boîte** : le dessin s'étire pour montrer
  ses lignes, la boîte garde la hauteur enregistrée, et le bas de la carte ne se clique pas. La
  saisie et l'import mesurent, donc l'usage normal n'y tombe pas.
* Chaque flèche visible cherchait ses deux nœuds **dans tout le tableau**, à chaque image :
  corrigé (l'index spatial rend un nœud en temps constant). Le calcul du clic, lui, le fait
  encore pour chaque flèche — seulement au clic.
* La barre d'options, comme la barre d'action, parcourt le tableau pour trouver la sélection, à
  chaque image où quelque chose est sélectionné : le magasin n'a pas d'index par identifiant.
* Un message (le « Bienvenue » d'une application neuve) passe entre la barre d'options et la barre
  d'action : les trois se partagent le bas de l'écran.

---

## 7. Ce qui attend sa parole

1. **Les membranes** (fiche 38 § 8) : la discussion reste ouverte.
2. **Les rideaux** (fiche 39 § 14) : il demande des références de logiciels — à préparer avec la
   discussion des membranes.

---

## 8. Ce qu'il faut regarder à l'écran

```text
cargo run --release > sortie-fleches.txt 2>&1
```

1. **Le design** : relier deux cartes de couleurs différentes par une flèche. Le fil passe de la
   couleur de l'une à celle de l'autre, avec un halo doux et une petite pastille au bout. Dire si
   c'est aussi beau que dans Tauri.
2. **La barre** : sélectionner une flèche — une barre apparaît au-dessus de « 1 sélectionné ».
   Essayer **Courbe** (ajouter un coude par le losange du milieu pour voir la courbe), **Double
   sens**, les **épaisseurs**, une **relation**, puis `Ctrl+Z`.
3. **Son défaut** : une carte avec deux fois le même mot, une flèche qui en part. Sélectionner la
   flèche, **« Ancrer… »** : la caméra va sur la carte. Cliquer le **second** mot, `Entrée`,
   cliquer un mot de la carte cible, `Entrée`. Puis passer la souris sur la flèche : **seul** le
   second mot doit briller, et la flèche doit partir de sa ligne.
4. **Écrire dans la carte** au-dessus du mot ancré, valider, repasser sur la flèche : c'est
   toujours le même mot qui brille.
5. Avant de relancer, copier `%TEMP%\glucose-chronique\derniere-session.txt` en
   `sortie-chronique-2026-09-25-fleches.txt`.

---

## 9. Les sources

* tldraw, [*Arrow binding options*](https://tldraw.dev/examples/arrow-binding-options) et
  [*TLArrowBindingProps*](https://tldraw.dev/reference/tlschema/TLArrowBindingProps).
* Steve Ruiz, [*Arrows (At Length)*](https://gitnation.com/contents/arrows-at-length), et
  [perfect-arrows](https://github.com/steveruizok/perfect-arrows).
* Excalidraw, [*Building Elbow Arrows, part 2*](https://plus.excalidraw.com/blog/building-elbow-arrows-part-two)
  — pour le contournement des obstacles, pas encore fait.
* Hypothesis, [*Fuzzy Anchoring*](https://web.hypothes.is/blog/fuzzy-anchoring/) — le modèle
  position + citation + contexte des ancres.
* Glucose Tauri : `ArrowSvgLayer.tsx`, `ArrowOptions.tsx`, `ArrowTextEditor.tsx`,
  `utils/textAnchors.ts`, `HtmlAnnotationLayer.tsx`.

---

**Retour** : [`00-INDEX.md`](00-INDEX.md)
