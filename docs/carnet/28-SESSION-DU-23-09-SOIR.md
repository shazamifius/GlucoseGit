# 28 — Le liseré avait deux causes, et le seuil se calcule

> **Rôle de ce document.** La session du 23/09 au soir s'ouvre sur une capture de l'utilisateur :
> *« le système de Ctrl+B ne fonctionne toujours pas assez bien, on voit clairement un liseré
> blanc autour »*. Cette fiche dit ce que ses pixels ont tranché, les deux corrections qui en
> sont sorties — l'une dans le détecteur, l'autre dans le rendu —, les quatre chantiers qui ont
> suivi, et l'étape 2 de la fiche 27 : l'empreinte perceptuelle, dont le seuil se **calcule**.
>
> **Date** : 2026-09-23, soirée · commits `5129b0c` à `d890f6c`, poussés sur `main`.
> **État vérifié** : `cargo test --workspace` exit 0, **1 509 tests verts**, clippy strict à
> zéro, douze cliquets. Aucun plafond de taille relevé ; la borne `POSTES` du cliquet 10 passe
> de 40 à 41 pour une marque de mesure nouvelle, ce que ce cliquet prescrit lui-même (§ 7).
>
> Une **session sœur** a travaillé en même temps sur la documentation publique (README, GUIDE,
> `.github/`). Les deux ont committé sur le même arbre sans se toucher, chacune ses fichiers,
> coordonnées par message ; elle a relevé deux défauts de code, traités ici (§ 3).

---

## 1. Ce qui a été fait, en une table

| Chantier | Ce que c'est | Vérifié par |
|---|---|---|
| **BORDURES-4, le rendu** (`Recadrage::texels_lisibles`, `Vue::avec_fenetre`, bornes de la pose GPU) | Les deux voies ne lisent plus ce que le recadrage retire : les bords de la fenêtre se prolongent, comme ceux de l'image | Un test du noyau et un test **sur la carte**, chacun avec sa preuve à l'envers ; chaque bord entier possible de quatre largeurs vérifié un à un ; la forêt rendue par le vrai code et regardée |
| **BORDURES-4, la détection** (`bordures::est_une_transition`) | Le fondu se lit pixel par pixel ; il se juge sur les pixels qui **peuvent** en montrer un | Trois tests, chacun vérifié à l'envers ; **49 images réelles**, le Rust et un portage Python d'accord sur toutes |
| **OUTILS** (`tools::add_and_edit`) | Une carte neuve naît son texte sélectionné : la première touche remplace « Nouveau texte » | Un test par outil, vérifié à l'envers |
| **TRANS-DOMAINES** (`arrow::est_trans_domaine`) | Le bouton qui mentait fait ce qu'il dit : pointillés, masquage, et clic | Règle, dessin et clic, chacun vérifié à l'envers ; la scène témoin **au pixel près** |
| **DEPOT-WEB-5** (`Depot`, `renderer::arrivage`) | Une épingle déposée répond dans l'instant : un marqueur là où elle arrivera, et elle s'y pose même si la vue a bougé | Deux tests d'application, un de dessin ; le marqueur regardé |
| **EMPREINTE** (`bench_empreinte`) | L'étape 2 de la fiche 27 : pHash mesuré sur de vraies paires, seuil calculé | 6 épingles × 3 tailles, 84 variantes, 2 193 paires négatives |
| **BORDURES-5** | Compter au lieu de trier : la même décision, quatre fois moins cher | Décisions identiques au bit près sur les 49 images |
| **RYTHME** (`Rythme::presentee`) | Le sommeil d'un toast n'est plus une image qui reste à l'écran | Un test, vérifié à l'envers |
| **BANDE** (`bench_bande`, marque `reperes`) | La bande coûte 0,49 ms ; ses 20 ms du terrain étaient une marque mal posée | Le banc ; la cause réelle attend la prochaine chronique |

---

## 2. Le liseré : deux causes, et la capture les séparait

Sa forêt venait de `fuser.glucose`, image 9 (1 200 × 576) — reconnue au rapport largeur sur
hauteur de la capture, 2,09. Le détecteur retirait `14, 12, 15, 17` pixels. Mesuré ensuite
**sur la capture elle-même**, bord par bord :

| bord | ce que l'écran montrait | ce que le fichier recadré porte |
|---|---|---|
| haut | un rang à 245,241,222 | le rang 12 gardé, luminosité 227 : un rang de fondu |
| bas | un rang à 177 | le rang 558 gardé, **blanc à 94 %** |
| droite | une colonne à **123** | la colonne 1 184, à **78** — propre |
| gauche | rien | propre |

En haut et en bas, le détecteur gardait des rangs clairs. **À droite, le fichier était propre et
l'écran ne l'était pas** : 123 pour 78 dans l'image et 209 dans la colonne retirée, soit un tiers
de mélange. C'était le rendu.

### 2.1 Le rendu lisait ce qu'on venait de retirer

La carte reçoit la texture native et lit le sous-rectangle du recadrage par un filtre
bilinéaire, **sans borner la lecture** à ce sous-rectangle : le pixel du bord mêle jusqu'à moitié
de la colonne coupée. Le report du processeur faisait de même — il ne bornait ses lectures qu'aux
bords de l'image — et, plus loin, un niveau réduit de la pyramide dont un texel chevauche le bord
ramène la bande par un autre chemin.

La règle naît **une fois**, dans le noyau (`Recadrage::texels_lisibles`), et chaque voie
l'exécute :

1. un pixel natif est gardé si son **centre** tombe dans la fenêtre — la règle du rastériseur,
   que le report applique déjà à la destination. Elle ne demande **aucune tolérance** : un bord
   entier, tel que `Ctrl+B` le pose, tombe à un demi-pixel de tout arrondi ;
2. un texel d'un niveau réduit `f` fois moyenne les pixels natifs `[k·f, (k+1)·f[` : il n'est
   lisible que si **tous** sont gardés.

Le processeur l'applique par `Vue::avec_fenetre` — `voisins` et `colonnes_interieures` bornent à
la fenêtre au lieu de l'image, sans un test de plus dans la boucle intérieure. La carte, par des
bornes dans la pose, où le nuanceur ramène sa lecture. L'image tournée et cadrée pose un motif
taillé à la fenêtre. Sur la forêt, sans la fenêtre, le bord gauche montait à **104** puis
**122** au zoom proche pour 52 à l'intérieur ; avec, il colle à son voisin à tous les zooms.

### 2.2 Le détecteur supposait un fondu uniforme

BORDURES-3 lisait un rang de fondu comme `a · bande + (1 − a) · suivant` avec **un seul** `a`
pour la ligne, et le jugeait déplacé dès qu'un centième de ses pixels l'était. Sur la forêt, le
bord de la peinture **ondule** : son premier rang gardé mêlait 79 à 90 % de blanc selon l'endroit,
et son dernier portait deux coups de pinceau débordant sur la marge (x = 13 à 71 et 765 à 774).
Le mélange uniforme n'expliquait que 94 % des pixels ; il en fallait 99.

Chaque pixel a désormais **son** coefficient — sa projection sur le segment qui va de son voisin
à la bande —, et le rang est un fondu si le mélange explique 99 % de ses pixels **et** si le
pixel typique a bougé vers la bande au-delà du bruit. Trois essais y ont mené, et la mesure a
refusé les deux premiers :

| essai | ce que la mesure a dit |
|---|---|
| un `a` par pixel, le déplacement jugé sur **un centième** des pixels | trop permissif : il mangeait **3 rangs de vraie peinture** au bas de la forêt, un éclaircissement doux de 5 niveaux par rang — la tolérance au bruit, appliquée pixel par pixel, « explique » n'importe quel rang lisse |
| la **médiane** du déplacement, sur tous les pixels, avec une garde « le rang suivant est du contenu » | la garde **laissait une ligne presque noire** sur fuser-04 (bande noire), et elle était inutile ; sans elle, la médiane gardait une colonne de fondu dans une peinture **sombre**, où un tiers des pixels sont déjà presque noirs et ne peuvent montrer aucun fondu |
| la médiane **parmi les pixels qui peuvent montrer un fondu** — ceux dont le voisin n'est pas déjà de la couleur de la bande —, qui doivent être la majorité | retenu |

Et la mesure a trouvé au passage un **défaut plus ancien** : sur sept illustrations posées sur un
fond de la couleur de la marge, BORDURES-3 prenait pour des fondus les rangs où seule la
**pointe d'un objet** paraît — un centième de pixels déplacés suffisait —, et rognait un à trois
rangs d'objet. La médiane les garde : leur pixel typique est du fond, immobile.

Aucun nombre nouveau : le bruit (24) et la part aberrante (1 %) existaient ; la médiane est la
définition du pixel typique.

### 2.3 Ce qui reste, et c'est à l'œil de le juger

Au bas de la forêt, la peinture **elle-même** se désature vers le gris sur six rangs, de 75,60,39
à 102,95,87 — 4 à 8 niveaux par rang. Rendue, elle ressemble à un bord plus clair. Le détecteur
la garde volontairement : c'est la règle « un fondu doux ne se mange pas », qui protège aussi les
vignettages. Si l'utilisateur la voit encore comme un liseré, c'est la question à reposer, avec
un critère à l'échelle de plusieurs rangs — pas un seuil.

---

## 3. Deux défauts relevés par la session sœur

Elle relisait le guide geste par geste dans le code, et en a trouvé deux :

* **« Nouveau texte » naissait le curseur à la fin.** Le commentaire de `tools.rs` disait *« le
  texte de départ est là pour être remplacé »*, et il fallait `Ctrl+A` avant d'écrire — ce que le
  guide enseignait. Il naît sélectionné ; le guide dit maintenant ce qui se passe.
* **Le bouton Trans-domaines mentait** : il basculait un booléen que seule la barre lisait, et
  affichait « activé ». Glucose Tauri en faisait trois choses, et les trois sont branchées : la
  règle dans le noyau (les deux bouts portent des domaines et n'en partagent aucun), les
  pointillés 6-4 en unités monde, et le masquage — étiquette comprise — **y compris au clic** :
  une flèche qu'on ne voit pas ne vole pas le geste. Une flèche pleine garde **exactement** son
  tracé d'avant : séparer la pointe de la tige posait deux fois leur jonction anticrénelée, et la
  scène témoin l'a vu au pixel près.

Vérifiés au passage : l'**Aimant** coupe bien le magnétisme au déplacement comme au
redimensionnement ; **Collaborer** dit honnêtement « pas encore » (la fiche 05 § 5.4 préférerait
un bouton grisé — c'est une décision de produit, laissée telle).

---

## 4. DEPOT-WEB-5 — la seconde d'attente d'une épingle

Mesuré sur trois épingles réelles : **781, 1 036 et 1 404 ms**, dont l'essentiel **avant le
premier octet de l'image** — la connexion, la page, l'original. Une barre de progression
resterait vide les quatre cinquièmes du temps puis se remplirait d'un coup ; un cadre à la forme
d'une image inconnue devrait se redimensionner à l'arrivée, c'est-à-dire avouer qu'il mentait.
La recherche ([Nielsen](https://www.nngroup.com/articles/response-times-3-important-limits/)) donne
la vraie exigence : **un dixième de seconde** pour qu'un geste direct semble répondre.

Le canal des dépôts porte deux messages : l'**annonce**, envoyée à l'instant du lâcher, et la
**livraison**, qui la rejoint par son numéro — toujours, vide s'il le faut, sans quoi le marqueur
resterait pour toujours. Le marqueur est un toast posé là où l'image arrivera : *« Téléchargement
depuis fr.pinterest.com… »*.

Et un défaut que l'annonce a corrigé en passant : le point de lâcher restait en pixels d'écran
jusqu'à la livraison, et se convertissait avec la vue **du moment de la livraison**. Déplacer la
vue pendant l'attente envoyait l'image ailleurs — le test à l'envers la trouve en
**(11 440, −5 022)** au lieu de (720, 489). Le point se fige désormais dans le monde à l'annonce.

---

## 5. L'empreinte perceptuelle — étape 2 de la fiche 27

`bench_empreinte`, sur six épingles de l'utilisateur en 236, 736 et original, et les 49 images de
ses deux documents en négatif :

| | la même image | deux images différentes (2 193 paires) |
|---|---|---|
| **pHash** (référence phash.org) | **0 bit** dans les six groupes | au plus proche **20**, médiane 32 |
| dHash (Krawetz) | 0 à 2 | au plus proche 9 |

pHash sépare trois fois mieux. Et sur des variantes fabriquées depuis les originaux, au pire des
six :

| variante | pHash |
|---|---:|
| JPEG qualité 10 et 30, niveaux de gris, miniature de 64 px | 0 |
| contraste +30 % | 2 |
| lumière +20 % | 4 |
| recadrage de 2 % par bord | 8 |
| recadrage de 5 % par bord | 12 |
| cadre blanc de 5 %, filigrane blanc dans un coin | 14 |
| recadrage de 10 % d'un seul côté | 22 |
| recadrage de 10 % par bord | 26 |
| miroir | 34 |

### 5.1 Le seuil ne se choisit pas : il se calcule

Les distances entre images différentes suivent la loi du hasard pur, **Bin(64, ½)** : 4 paires
attendues à 20 bits ou moins sur 2 193, **5 observées** ; 1,7 attendue à 19 ou moins, 0 observée.
La probabilité qu'une image différente tombe sous `t` est donc connue, et le « sûr à 99 % » de
l'utilisateur donne le seuil directement : parmi `N` candidats d'une recherche,

```
    N · P(Bin(64, ½) ≤ t)  ≤  1 %
```

Pour 20 candidats, **t = 18** (0,62 %) ; 19 dépasserait (1,6 %). Tout ce qui est recompression,
réduction, gris, lumière, contraste, cadre ou filigrane passe ; un recadrage de 10 % ou un miroir
ne passe pas, et c'est juste — ce n'est plus la même image, c'en est une dérivée.

Aucun module de production n'est écrit : il n'aurait pas d'appelant tant que l'étape 3 — la
recherche SauceNAO — n'existe pas (fiche 05 § 7.6).

---

## 6. Trois instruments réparés ou mesurés

* **`Ctrl+B` compte au lieu de trier.** « Le centile qui laisse un centième au-dessus de lui est
  sous le bruit » et « au plus un centième des écarts dépassent le bruit » sont la même
  proposition ; la seconde se vérifie en comptant. Décisions identiques au bit près sur les 49
  images ; la forêt passe de 1,20 à 0,28 ms, la plus lourde de ses images de 5,73 à 1,56 ms, une
  synthétique de 24 Mpx à bandes de 400 px de 53 à 40 ms. Les « 160 ms » de la fiche 27 ne se
  reproduisent sur aucune image seule : c'est un lot qu'il faudra mesurer.
* **Le sommeil d'un toast n'est plus une image qui reste à l'écran.** « Pire 1 874 ms » dans la
  chronique du jour : le toast dort jusqu'à son fondu, et l'image de son réveil est attendue.
  L'histogramme compte désormais depuis l'**échéance** quand elle est plus tardive que la
  présentation précédente — la règle de GEL-1, qu'il avait gardée de côté. Pendant un mouvement,
  l'image suivante est due dès la précédente (le tempo attend *après* le rendu) : rien n'y change.
* **La bande coûte 0,49 ms.** `bench_bande` la rend à la taille de la fenêtre de l'utilisateur :
  2,25 ms au tout premier dessin (glyphes froids), 0,49 ms refaite, 0,12 ms en cache. Les 14,95,
  32,28 et 20,66 ms du terrain n'étaient pas elle : sur la voie graphique, la marque qui la
  précédait était `ornements`, et tout ce qui suit — guides, boîte de sélection, marqueurs, début
  de l'interface — se facturait à la bande. **Cinquième marque mal posée de ce dépôt.** Une marque
  `reperes` les sépare ; **ce qui paie ces 20 ms n'est pas établi**, la prochaine chronique le
  dira.

---

## 7. Ce que cette session a démenti — la liste

1. **« Le fichier recadré est propre, donc l'écran aussi »** : à droite, le rendu ramenait la
   colonne retirée. Seule la mesure de la capture l'a montré.
2. **Mon premier soupçon des mipmaps** : la carte n'en a pas (`mip_level_count: 1`). Le mécanisme
   était voisin — le filtre bilinéaire non borné —, et le bon niveau de pyramide y ajoute sa part
   sur la voie processeur.
3. **Le fondu uniforme de BORDURES-3**, par le bord qui ondule.
4. **Mon critère pixel par pixel au centième**, par trois rangs de vraie peinture mangés.
5. **Ma garde « le rang suivant est du contenu »**, par une ligne noire gardée sur fuser-04.
6. **La médiane sur tous les pixels**, par une peinture sombre.
7. **BORDURES-3 ne mangeait pas de contenu** : il rognait la pointe des objets sur fond uni.
8. **Mon test de la pointe d'objet**, faux la première fois : un objet seul, une fois les marges
   latérales retirées, occupe toute la largeur — son sommet flou est alors un vrai fondu.
9. **La bande à vingt millisecondes** : 0,49 ms mesurée.
10. **« 160 ms pour `Ctrl+B` »** : 0,3 à 5,7 ms sur une image réelle, avant même la réécriture.

Six fautes de mes propres critères ou tests, attrapées chacune par une mesure ou un test qui
protestait — et un cliquet des 80 lignes qui a eu raison trois fois (§ 1, `draw_arrow`,
`GlucoseApp::new`, `peindre_ce_qui_a_change`) : chaque fois, l'extraction a nommé une vraie unité
de sens — le crayon d'une flèche, les `Arrivees`, la scène réduite de la `Resolution`.

---

## 8. Ce qui reste, chiffré

| # | Ce que c'est | Chiffre | Statut |
|---|---|---|---|
| 1 | **Le liseré, vu à l'écran** | — | Les deux causes corrigées ; **à confirmer à l'œil** (§ 9) |
| 2 | **Qui paie les 20 ms facturées à la bande** | 14,95 / 32,28 / 20,66 ms | La marque `reperes` le dira |
| 3 | **Les dépôts « Chromium Web Custom MIME Data Format » seul** | six sur dix | Ce format est déjà fouillé pour ses adresses ; un essai `GLUCOSE_DEPOT=1` tranchera |
| 4 | **La recherche d'origine** (fiche 27, étape 3) | seuil calculé | **Attend une clé SauceNAO** |
| 5 | **`Ctrl+B` sur un lot** | ≈ 1 à 2 ms par image | À mesurer sur cinquante images avant de le mettre en cascade |
| 6 | **L'adoucissement propre d'une peinture** | 6 rangs, 4-8 niveaux par rang | Gardé ; à rediscuter si l'œil le juge liseré |
| 7 | Mémoire de la carte (≈ 325 Mo), l'arbitre qui alternerait | fiche 27 | Inchangés |

---

## 9. Ce qu'il faut tester à l'écran

1. **Le liseré** : ouvrir `fuser.glucose`, sélectionner la forêt (ou toute image à bord clair),
   `Ctrl+B`, puis zoomer de près sur ses quatre bords. Aucun trait clair ne doit rester ; le bas
   peut garder un adoucissement propre à la peinture.
2. **Pinterest** : `set GLUCOSE_DEPOT=1` puis `cargo run --release > sortie-depot.txt 2>&1`,
   glisser trois épingles depuis la grille puis une depuis sa page. Un marqueur « Téléchargement
   depuis… » doit paraître **aussitôt** au point de lâcher, puis l'image à sa place ; déplacer la
   vue pendant l'attente ne doit pas la faire atterrir ailleurs.
3. **Une carte de texte** (`T`, clic) : taper un mot doit **remplacer** « Nouveau texte ».
4. **Trans-domaines** : deux nœuds de domaines différents reliés par une flèche → pointillés ;
   le bouton coupé → la flèche disparaît et ne s'attrape plus.

---

**Retour** : [`00-INDEX.md`](00-INDEX.md)
