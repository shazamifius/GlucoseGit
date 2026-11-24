# 140 — Son nom et sa couleur dans un canevas partagé (Q19)

> Le 24/11/2026, de 7 h 35 à 8 h, session du relais (compte b), après la fiche 139. Sa réponse à
> Q19, lue le 23/11 : *« carrément, je suis plutôt d'accord : on demande le pseudo ainsi que la
> couleur d'affichage »*. Notre proposition (`relais-rep-q19`) : demander les deux la première fois
> qu'il partage ou rejoint, une seule fois, et les changer ensuite dans Collaborer ; dire quand deux
> personnes ont la même couleur, sans choisir à sa place.

## 1. Ce qu'il y avait

Un nom tiré au sort la première fois (« Aigle Émeraude », comme Glucose Tauri,
`src/multiplayer/localUser.ts`), gardé dans `identite.txt`, et une couleur parmi les dix de Tauri —
**qu'on ne changeait que dans ce fichier**. Son autre appareil s'appelle « Aigle Émeraude » depuis
le 23/11.

## 2. Ce qui est fait — IDENTITE-2 et PASTILLES-1

* **L'identité retient qu'il l'a choisie** : une troisième ligne, `choisie`, que les versions
  d'avant ignorent (elles lisent les deux premières, vérifié par l'épreuve). Un fichier d'avant se
  relit « tirée ».
* **La première fois** qu'il clique « Partager ce canevas » ou « Rejoindre… » avec une identité
  tirée, Glucose demande d'abord « Ton nom et ta couleur » — le champ, son nom tout choisi (ce qu'il
  tape le remplace), et **dix pastilles** —, puis fait ce qu'il demandait. « Annuler » revient à
  Collaborer sans rien partager. **« Ton nom et ta couleur… »** est dans Collaborer, seul comme
  partagé.
* **Les pastilles** (PASTILLES-1, `ui/question/pastilles.rs`) : une rangée sous le champ, une case
  par couleur — la case entière est la cible d'un doigt, le carré au centre se voit ; la choisie est
  cernée du filet de la sélection. Au clavier, `Alt` + gauche ou droite (les flèches seules
  déplacent le curseur du champ, et doivent le garder) ; sans champ, les flèches seules.
* **Dans un canevas partagé**, « Garder » change le nom et la couleur du partage : chaque lettre les
  porte, et chez les autres chacune les relit (`un_pair`) — le changement part avec le geste ou le
  mouvement de souris suivant. Le facteur des photos reçoit les siens (`Course::Signer`) : ses
  lettres ne changent pas le nom affiché chez les autres, mais elles ne doivent pas mentir.
* **La couleur déjà prise** par quelqu'un qui est là : la question l'écrit (« « Loup Vert » a déjà
  cette couleur »), à chaque pastille touchée ; elle ne choisit pas à sa place.
* **Un nom vide** se redemande, la couleur choisie gardée.

**La couleur reste l'une des dix** — nous avions écrit « ta couleur sur la roue ». Deux raisons :
elles ont été choisies pour rester lisibles en pastille de quelques pixels sur le fond sombre, et la
lettre porte le **rang** de la couleur, qu'un Glucose déjà installé (la beta.5, ses autres
appareils) comprend ; une couleur libre demanderait un champ neuf dans la lettre, que les versions
publiées ne liraient pas. À contester s'il veut la roue : elle viendrait avec une publication.

## 3. Prouvé

* `identite_2_*` (5 épreuves, par les vrais chemins : le menu, la souris sur une pastille, `Alt` +
  flèche, le bouton « Garder ») et `test_pastilles_1_sous_le_champ_au_dessus_des_reponses` ; les
  trois épreuves de REJOINDRE-1 passent par la question du nom la première fois, comme lui.
* `identite-140.json` : **8 sabotages sur 8**.
* **Au pilote** (`~/.cache/glucose-bancs/q19-140/`) : Collaborer montre « Ton nom et ta couleur… » ;
  « Partager ce canevas » dans une maison neuve pose d'abord la question ; la pastille rose touchée
  se cerne, « Shaza » tapé remplace le nom tiré ; « Garder » écrit `Shaza / 4 / choisie`. **À la
  taille de son téléphone** (1080 × 2400 à 275 %) : les dix pastilles tiennent sur une ligne.

## 4. À contester

* Le cadre de la pastille choisie est le filet de la sélection, d'un pixel : discret sur un écran
  très dense.
* Deux personnes de la même couleur restent possibles : Glucose le dit et laisse faire, comme
  promis.

## 5. Le coût du flou sur la voie du processeur, mesuré (fiche 137 § 7)

`banc_flou_au_processeur` (ignoré par défaut ; `cargo test --release -p glucose-desktop --lib --
banc_flou --ignored --nocapture`) : le masque d'un nœud choisi au milieu de l'écran, la gaussienne
en pyramide (`glucose_core::flou::flouter`), le mélange de l'écran entier (`meler`) — les fonctions
mêmes de la voie du processeur, trente passages après trois d'échauffement. **Sur ce poste** (Ryzen 5
7600X, un seul fil) :

```text
                     tout (médiane, pire)   masque   gaussienne   mélange
1920 x 1080, ×1      65,6 ms   67,2 ms      0,02     15,0         50,6
2560 x 1440, ×1     118,0 ms  119,6 ms      0,03     26,8         91,1
2560 x 1440, ×2     114,9 ms  116,0 ms      0,02     24,0         90,9
```

**Six à douze fois le plancher de la charte** (10 ms), sur une machine rapide ; son VivoBook serait
plusieurs fois plus lent. Sans effet sur ses appareils (tous passent par la carte graphique), mais
une machine sans carte — une machine virtuelle, un bureau distant — attend un dixième de seconde
chaque image où le flou vit et que la scène change. **Le mélange pèse les trois quarts** : 24 ns par
pixel, là où une lecture bilinéaire incrémentale par rangée et des entiers devraient en coûter une
fraction (à prouver). Le chantier est ouvert dans SUITE, avec ces nombres pour point de départ.

**Un premier pas, au bit près** (MELER-1, le 24/11 vers 8 h 15) : `meler` calcule les voisins et les
poids de chaque colonne et de chaque rangée une fois (`Pas`), et lit le masque avant le flou — un
pixel que le masque garde net ne lit plus le flou. Les mêmes opérations, dans le même ordre :
`test_flou_meler_rend_au_bit_pres_la_lecture_pixel_par_pixel` rejoue l'ancienne loi pixel par pixel
(sans étage et avec deux) et exige des octets identiques ; `meler-140.json` **2 sur 2**,
`flou-137.json` toujours **16 sur 16**, l'accord des deux voies (`voies_suite`, 34) inchangé.

```text
                     tout (médiane)   mélange
1920 x 1080, ×1      65,6 → 41,9 ms   50,6 → 27,0
2560 x 1440, ×1     118,0 → 75,7 ms   91,1 → 49,0
```

**Reste** quatre fois le plancher : le mélange en virgule flottante (un `round` et deux `clamp` par
canal), et la gaussienne (`agrandir` lit encore pixel par pixel). La suite : la même chose pour
`agrandir`, puis des entiers — en gardant l'accord des deux voies à 4 niveaux.

**`agrandir`, la même lecture** (vers 8 h 25) : une seule fonction bilinéaire (`bilineaire`, ses pas
déjà faits) sert désormais au mélange et à l'agrandissement, au bit près
(`test_flou_agrandir_rend_au_bit_pres_la_lecture_pixel_par_pixel`) ; `meler-140.json` **3 sur 3**,
`flou-137.json` **16 sur 16**, `voies_suite` 34. Le gain est petit — la gaussienne 15,0 → 13,5 ms,
le tout **40,5 ms** en 1080p (73,2 en 1440p) — : dans cette étape, c'est la réduction à pleine
taille (`reduire_des_octets`, quatre lectures bornées par pixel) qui pèse, à mesurer à part ensuite.

**La réduction, par rangées** (vers 8 h 40) : mesurée à part, la première réduction coûtait 8,7 ms
des 13,5 de la gaussienne. Chaque rangée d'octets se convertit une fois, les quatre voisins bornés de
chaque colonne et de chaque rangée se calculent une fois (`fenetre`), le filtre binomial est le même
(`pese`, la ligne que `flou-137` sabote) : au bit près
(`test_flou_reduire_rend_au_bit_pres_la_lecture_voisin_par_voisin`, des octets comme d'une image,
tailles paires, impaires, d'un pixel). `meler-140.json` **4 sur 4**, `flou-137.json` **16 sur 16**,
`voies_suite` 34.

```text
                     tout (médiane)            réduction   gaussienne   mélange
1920 x 1080, ×1      65,6 → 41,9 → 37,5 ms     8,7 → 5,2   15,0 → 10,6  50,6 → 26,9
2560 x 1440, ×1     118,0 → 75,7 → 68,7 ms        → 10,0   26,8 → 19,9  91,1 → 48,8
```

**Reste** près de quatre fois le plancher. Le mélange (27 ms) n'a plus rien à pré-calculer : ce
sont ses flottants — par canal, deux produits, un `round`, deux `clamp`. La suite est une loi en
entiers, qui ne sera plus au bit près de l'ancienne : elle devra garder l'accord des deux voies à 4
niveaux, et se juger au banc.

**Les rangées lissées, gardées** (vers 8 h 55, `6487cac`) — le paragraphe d'au-dessus se trompait, il
restait à pré-calculer : une rangée du masque sert à une maille
entière de rangées de l'écran, une rangée du flou à quatre ; chacune se lisse désormais le long de x
une seule fois (`Lissees`), au lieu d'une fois par rangée de l'écran. Au bit près ;
`meler-140.json` **6 sur 6**, `flou-137.json` **16 sur 16**, `voies_suite` 34. Remesuré par le relais
le 24/11 à 9 h 10 :

```text
                     tout (médiane)     mélange
1920 x 1080, ×1      37,5 → 30,1 ms     26,9 → 19,6
2560 x 1440, ×1      68,7 → 55,8 ms     48,8 → 35,8
2560 x 1440, ×2             51,9 ms            35,3
```

**Reste** trois fois le plancher (10 ms en 1080p), et le mélange en pèse encore les deux tiers :
il n'a plus rien à pré-calculer, ce sont ses flottants — par canal, deux produits, un `round`, deux
`clamp`. La suite est la loi en entiers, telle que dite ci-dessus.

**Un `clamp` de trop** (le relais, 9 h 15) : la dernière ligne du mélange bornait `v.round()` entre 0
et 255 avant de le convertir en octet — mais en Rust, `as u8` sur un flottant sature déjà (et NaN
donne 0) : le `clamp` ne changeait aucun résultat, au bit près, quelle que soit la valeur. Retiré :
le mélange **19,6 → 17,5 ms** en 1080p (35,8 → 31,9 en 1440p), le tout **30,1 → 28,0 ms** (55,8 →
51,8). Les épreuves au bit près du flou (9 au noyau, 7 au bureau) et `voies_suite` (34) passent ;
aucun sabotage ne visait la ligne. Le `round` reste : `round_ties_even` (qui se vectorise) ne
donnerait pas les mêmes nombres sur un demi exact — c'est déjà la loi en entiers.

**L'arrondi sans appel de fonction** (le relais, 9 h 25) : sur la base x86-64 (SSE2, aucune cible
fixée), `f32::round` n'a pas d'instruction — quatre appels à `roundf` par pixel. `octet` tronque
(`as i32`, une instruction), puis ajoute 1 si ce qu'il a retiré vaut au moins un demi : cette partie
se calcule exactement (lemme de Sterbenz), et l'octet est celui de `v.round() as u8` pour toute
valeur. `test_flou_octet_arrondit_comme_round` l'éprouve à chaque entier et chaque demi de −4 à
300, à un ulp de part et d'autre, tous les 1/1024, à l'infini et à NaN — elle a trouvé d'emblée un
débordement à l'infini (`i32::MAX + 1`), d'où `saturating_add`. `meler-140.json` **9 sur 9** (trois
sabotages neufs), `voies_suite` 34, cliquets, Clippy du noyau :

```text
                     tout (médiane)            mélange
1920 x 1080, ×1      30,1 → 28,0 → 24,6 ms     19,6 → 17,5 → 14,1
2560 x 1440, ×1      55,8 → 51,8 → 45,6 ms     35,8 → 31,9 → 25,7
```

**Reste** deux fois et demie le plancher ; la gaussienne (10,5 ms) pèse désormais presque autant que
le mélange.

