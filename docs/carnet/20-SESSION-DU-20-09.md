# 20 — La salissure, les cercles vicieux, et quinze coeurs qui dormaient

> **Rôle de ce document.** La fiche [`19`](19-SESSION-DU-19-09.md) désignait l'**étape 1 du
> plan [`18`](18-PLAN-R-ET-D.md)** — la salissure — comme le chantier suivant. Celle-ci dit ce
> que la journée en a fait. Elle dit aussi, et c'est sa partie la plus utile, **deux
> corrections que j'ai écrites et que la mesure a démenties**, dont une régression visible à
> l'écran que l'utilisateur a signalée en photo — et **un chiffre faux que la chronique
> elle-même m'a fait écrire dans la première version de cette fiche** (§ 4.5).
>
> **Date** : 2026-09-20 et 21 · seize commits, de `af2ffa4` à `5f542db`.
> **État vérifié** : `cargo test --workspace` exit 0, **1 327 tests verts** (29 binaires),
> clippy strict à zéro, **onze** cliquets mécaniques. Poussé sur `main`.
>
> **Le point de départ, mot pour mot** : « sa fait des gros gros carrée noir derrnier » et
> « lorsque on moove ces extremement pixeliser […] mon ordinateur es plutot puissant pourquoi
> cette pixelisation a elle une utiliter ».

---

## 1. Ce qui a été fait, en une table

| Chantier | Ce que c'est | Vérifié par |
|---|---|---|
| **Le tempo qui ne dérive plus** | La grille des soumissions se fixe quand l'image est **prête**, pas quand elle est présentée | Terrain : judder 34 % → **2 %** |
| **La chronique qui ne ment plus** | Un dialogue de 19,8 s n'est pas un gel ; l'élan n'est pas du repos | 15 tests |
| **Les plages** (`report/plages.rs`) | Opaque / transparente / mixte **par ligne**, lues une fois au rangement | Composition 4,8 ms → **1,7 ms** |
| **La bande du haut en cache** | Barre et onglets dans un tampon, gardé tant que ce qu'ils montrent ne change pas | Banc entrelacé : **+0,29 ms** quand redessinée |
| **Le fond sauté** (`Couverture`) | `clear` + `grid` ne se peignent pas quand les tuiles recouvrent tout | Banc : **−1,25 ms**, et l'aspect prouvé identique sur fond rouge / fond noir |
| **Le lecteur des cliquets** | Il lisait `ui.rs` sur 279 de ses 1 650 lignes | Son propre test |
| **`ui.rs` découpé** | 1 650 → 350 lignes de production, en quatre modules | Cliquet 4a |
| **Le banc du pincement** | L'échelle avance en **octaves**, et compte les tuiles peintes par image | Pic isolé : **77 tuiles, 43 ms** |

---

## 2. La salissure — étape 1 du plan 18, à moitié faite

Le plan demandait trois choses. Deux sont faites, une ne l'est pas.

### 2.1 Séparer la chrome de la scène — fait, et modeste

Le banc disait que la barre coûtait 13 à 15 % de l'image pendant un glissement, **identique à
l'image précédente sur 119 images sur 119**. Elle se rend désormais dans un tampon à sa taille.

La clé du cache n'est pas devinée : la barre et les onglets sont déjà des **layouts**, donc le
dessin ne dépend que de ce dont le layout dépend — largeur, échelle, outil actif, bascules,
tableaux, badge — plus le survol. Et le survol n'est pas la position du pointeur mais **ce
qui** est survolé.

Le gain honnête est **+0,29 ms** par image quand la chrome se redessine, soit 6 % d'une image
à 5 ms. Le premier lancement isolé en annonçait 1,1 ms ; il mesurait surtout un cœur ralenti
(voir § 5.1).

### 2.2 Ne pas effacer sous les tuiles opaques — fait, et net

La grille relève, **avant** le fond, ce que l'écran porte, et répond à une question fermée :
toutes les tuiles sont-elles déjà peintes, opaques, et couvrantes sur leur surface entière ?
Si oui, le fond ne se peint pas. Aucune zone n'est calculée, aucun bord n'est arrondi : une
seule tuile manquante, transparente ou partielle, et on efface comme hier.

Le relevé est **gratuit** — et c'est la partie qui a demandé deux essais. La première version
recalculait les empreintes de toutes les tuiles, et le poste du fond montait de 0,98 à
1,11 ms : un coût net sur un document dont les photos ne pavent pas l'écran, donc dans le cas
courant. C'est la géométrie calculée deux fois que la charte interdit. Les empreintes sont
désormais relevées **une** fois et lues deux.

| | Résultat |
|---|---|
| Photos jointives | `clear` et `grid` **disparaissent** du profil (98 % des images), **−1,25 ms** |
| Photos espacées | Fond peint sur 100 % des images, relevé 0,06 ms |

### 2.3 Ne téléverser que ce qui a changé — **pas fait**

C'est le tiers restant de l'étape 1. Rien n'a été tenté ici.

> **Correction.** La première version de cette fiche justifiait ce chantier par « `blit` à
> 62,4 % du temps au repos », lu dans la chronique. **Ce chiffre était faux**, et c'est la
> chronique qui mentait : elle sommait les postes, si bien que l'unique image du gel de
> démarrage — 335 ms de `blit` — écrasait les trente autres, à 0,94 ms. Le § 4.5 raconte le
> défaut et sa correction. La vraie raison de traiter `blit` est plus modeste et suffit : il
> coûte **environ 1 ms par image**, sur un budget de 4,17 ms à 240 Hz, et il est désormais le
> plus gros poste fixe une fois `clear` sauté et la chrome mise en cache.

---

## 3. Les deux défauts signalés à l'écran

### 3.1 Les gros carrés noirs — une régression que j'avais introduite

`Portee::de` lisait l'opacité d'une tuile comme « aucune plage **mixte** », ce qui acceptait
les plages **transparentes**. Une tuile portant deux photos séparées par du vide était donc
déclarée opaque, composée par `Remplacer`, et son trou **écrasait le fond en transparent** —
du noir pur derrière les photos.

L'opacité utile est celle de la **boîte** : c'est elle qui borne la composition, donc c'est
sur elle seule que la question se pose. Un trou à l'intérieur suffit à répondre non ; ce qui
est autour ne décide de rien, puisque rien ne l'y compose.

**Ce que le test d'égalité au bit près ne pouvait pas voir** : il comparait `reporter` à
lui-même, avec et sans plages connues. Les deux voies composaient les mêmes pixels ; c'est le
**mélange choisi en amont** qui était faux. *Un test de composition ne dit rien du choix de
composer.* Le nouveau test échoue sur l'ancienne implémentation — la seule preuve qui compte.

### 3.2 La pixelisation — l'utilisateur avait raison, et deux fois

La chronique chiffrait : **48 % des images** rendues à un facteur 2,86 — un pixel d'écran pour
8,2 du canevas — pendant que le tempo tenait **quatre balayages, soit 16,7 ms** par image. On
abîmait donc pour tenir dix millisecondes alors qu'on en avait seize.

Ma première correction visait « un cran sous ce que le tempo tient déjà ». **C'était un
cercle**, et la session suivante l'a démenti en une lecture (§ 4.1).

La cible est désormais le **plus grand nombre entier de balayages qui tienne le plancher de la
charte** : deux à 240 Hz, un seul à 60 Hz. Elle ne dépend que de la période lue et du
plancher, donc rien ne la fait dériver. Un test rejoue le cercle — `k` monte à sept, la cible
ne bouge pas d'une nanoseconde.

> **Dix millisecondes n'était d'ailleurs pas une bonne cible.** L'écran ne sait pas montrer une
> image pendant dix millisecondes : il la montre un nombre **entier** de balayages. À 240 Hz,
> viser dix, c'est viser entre deux crans, et laisser une zone morte où l'on dégrade sans
> jamais descendre d'un cran.

---

## 4. Ce que la mesure a démenti — dont deux fois moi

### 4.1 Premier cercle : la cible de finesse qui suivait le tempo

Viser « un cran sous ce que le tempo tient » faisait monter la cible **avec** `k` : on dégradait
moins, donc les images coûtaient plus, donc `k` montait encore, donc la cible montait.

| | Avant ma correction | Après |
|---|---|---|
| Images au-dessus de 10 ms | 13 % | **46 %** |
| Images par seconde | 51 | **43** |
| Latence p99 | 23,2 ms | **50,2 ms** |
| Tempo | 4 balayages | 5 à 9 |

Un asservissement dont la consigne dépend de la sortie n'est pas un asservissement.

### 4.2 Second cercle : le raffinement progressif

Quand le budget d'une image est épuisé, composer la tuile depuis l'octave **précédente** —
encore en cache, elle vient de servir — agrandie deux fois et clippée à la place de la tuile
absente ; puis peindre la vraie dans le temps libre que le tempo dégage. C'est ce que font les
canevas à tuiles, et le code tenait en deux fonctions sans géométrie nouvelle : le clip est
déjà le domaine d'itération du report.

Un test prouvait qu'aucun trou n'apparaissait — la seule chose qui compte : montrer plus
grossier est acceptable, montrer du vide ne l'est pas. Il passait, et le pic tombait de
**77 tuiles à 18, de 43 ms à 22**.

Et le banc, en comparant les deux régimes dans la **même** exécution :

| Geste | Avec | Sans | Écart |
|---|---|---|---|
| Pincement | 8,59 ms | 8,25 ms | **+0,34 ms** |
| Glissement | 6,32 ms | 5,35 ms | **+0,97 ms** |

Perdant. La raison est arithmétique : composer un ancêtre agrandi coûte presque autant que
peindre la tuile qu'il remplace, et il faut le recomposer à **chaque** image tant qu'elle
manque. Le temps libre devrait rattraper — mais s'il y avait du temps libre, l'image ne serait
pas en retard.

**Tout retiré**, environ une heure de travail. Ce qu'il faudrait pour que cela marche, et qui
reste à faire : que composer un ancêtre coûte **nettement** moins que peindre une tuile. Ce
n'est pas le cas aujourd'hui parce que les deux parcourent la même surface ; ce le serait si
l'ancêtre se composait par blocs, ou si la tuile se peignait par tranches.

### 4.3 La réduction de résolution est un mauvais marché

Mesuré, deux régimes alternés dans la même exécution :

| Geste | Pleine résolution | Réduite de moitié | Gain |
|---|---|---|---|
| Glissement | 5,87 ms | 5,49 ms | **−0,38 ms (6 %)** |
| Pincement | 9,10 ms | 7,89 ms | **−1,21 ms (13 %)** |

Pour un pixel d'écran qui en montre **quatre**. La cause est structurelle : réduire la scène
**désactive les tuiles** — `Cadrage::reduit` impose le régime direct. On économise sur le rendu
des photos, on perd le cache, et on paie `agrandir` par-dessus.

Le modèle de la réduction croit gagner un facteur *f²* parce qu'il suppose le coût de la scène
proportionnel à sa **surface**. C'était vrai avant TUILE-1. Avec les tuiles, composer coûte la
surface mais **peindre coûte le contenu**, et le facteur *f²* est faux.

**Question de produit, non tranchée** — voir § 7.

### 4.4 Le lecteur des cliquets mesurait un dixième du desktop

Il coupait un fichier au **premier** `#[cfg(test)]`, en supposant que c'était le `mod tests` de
la fin. Dix fichiers en ont un plus haut, et tout ce qui suivait disparaissait : `ui.rs` était
lu sur **279 de ses 1 650 lignes**, `typography.rs` sur 47 de ses 679.

Deux plafonds remontent donc à la mesure vraie — le couplage de 36 à 51, les toasts de 46 à
48 — **sans qu'un seul accès ni un seul toast ait été ajouté**. Ce n'est pas relâcher la règle,
c'est cesser d'effacer une dette : chacun porte désormais le compte de ce qui **existe**.

### 4.5 La chronique elle-même sommait ses postes — et m'a fait écrire un chiffre faux

Le tableau « où va le temps » donnait un pourcentage par poste. Il venait d'une **somme** sur
toutes les images du geste. Sur la session de terrain du 20/09 — trente et une images, dont
**une** à 335 ms de `blit` (le gel d'initialisation du pilote, à la deuxième seconde) — il a
donc annoncé :

    blit  62,4 %

La vérité, poste par poste : `blit` coûte **0,94 ms** à une image typique. Une seule image
aberrante sur trente et une décidait du portrait du geste entier, et **j'ai écrit ce 62,4 %
dans la première version de cette fiche** comme justification d'un chantier.

Le plus dur à admettre : le module `histogramme.rs`, écrit trois jours plus tôt, s'ouvre sur
la phrase *« cent images à 2 ms et une à 200 ms donnent une moyenne de 4 ms — excellente —
alors que l'utilisateur a vu un gel »*. La leçon avait été tirée pour les **durées d'image** et
jamais appliquée à leur **décomposition** : le tableau des gestes se lisait au centile pendant
que la ligne du dessous, juste en dessous, se lisait en moyenne.

Chaque poste garde désormais sa **distribution** (356 octets par poste et par geste, 142 Ko
pour la session entière, quelle que soit sa durée). Le rapport donne médiane, p99 et pire :

    poste            median       p99      pire
    report            2.90ms   30.07ms     30.07ms  ############
    agrandir          0.86ms    8.02ms      8.02ms  ####........
    blit              0.86ms    6.89ms    335.44ms  ####........

Le gel n'est pas perdu — il est à sa place, dans le pire, où il se lit pour ce qu'il est.

**Et le pourcentage a disparu**, parce qu'il mentait une seconde fois : un pourcentage invite
à additionner, et des centiles ne s'additionnent pas — la médiane d'une somme n'est pas la
somme des médianes, et deux postes n'ont aucune raison d'être médians sur la même image. La
barre compare les postes **entre eux**, relative au plus coûteux ; personne ne peut la sommer.

Le test qui porte ce défaut **contient sa propre preuve** : il rejoue l'ancienne lecture — la
somme, que l'histogramme garde encore — sur les mêmes images, et vérifie qu'elle conclut
l'inverse. Une preuve permanente, là où un `git stash` n'aurait prouvé qu'une fois.

*Trouvé en relisant cette fiche, écrite une heure plus tôt.*

---

## 5. Trois leçons de méthode, pour la liste de la fiche 17

### 5.1 Comparer deux exécutions, c'est comparer le bruit

Trois lancements séparés du même banc ont donné **2,3 / 6,2 / 5,7 ms** de médiane pour la
**même** mesure. La fréquence de la machine varie plus que ce qu'on mesure. Tous les bancs de
cette session jouent désormais les deux régimes **alternés dans la même exécution**.

### 5.2 Un cache qui repeint tout rend les mêmes pixels

L'épreuve d'aspect ne peut pas voir qu'un cache ne sert à rien. Chaque cache porte donc un
**compte de dessins réels**, et un test le lit. C'est la leçon des vignettes, qui ont servi à
1 % pendant cinq sessions.

### 5.3 L'instrument de mesure se vérifie comme le reste

Trois sessions ont lu « où va le temps » et l'ont cru. Le tableau était faux depuis qu'il
existe. **Une mesure qu'on lit tous les jours est celle qu'on vérifie le moins**, parce que sa
familiarité tient lieu de preuve. Elle mérite un test au même titre que le code qu'elle juge —
et le sien devait être *« une image aberrante ne décide pas du typique »*, pas *« la somme est
correcte »*, qui était vrai et sans intérêt.

### 5.4 Un test qui prouve une égalité ne prouve pas un choix

Le test des plages comparait `reporter` à lui-même. Les deux voies étaient justes ; c'est la
décision prise **avant** l'appel qui était fausse (§ 3.1). Quand une correction change une
**décision**, le test doit porter sur la décision, pas sur son exécution.

---

## 6. Ce qui reste, chiffré

| # | Ce que c'est | Chiffre | Statut |
|---|---|---|---|
| 1 | **Le pic du changement d'octave** | 77 tuiles, **43 ms** contre 6,7 ms pour une image ordinaire, une image sur 240 | Isolé, non résolu ; le raffinement progressif a échoué (§ 4.2) |
| 2 | **Le téléversement partiel** (`blit`) | ~**1 ms** par image, soit un quart du budget à 240 Hz | Étape 1 du plan 18, **non commencée** |
| 3 | **Les halos** | **7,7 ms**, 46 % du temps sur une scène de cartes de texte | Jamais attaqué |
| 4 | **Le gel de démarrage** | ~1,26 s à la 2,3ᵉ seconde, dont 335 ms dans `blit` | Décrit fiche 19 § 5.1, non résolu |
| 5 | **COUT-1 sur les tuiles** | Prévoit à 27 % depuis la grille | Dette nommée fiche 19, non remboursée |
| 6 | **Le modèle de la réduction** | Suppose *f²*, faux depuis TUILE-1 | Constaté en commentaire ; corriger demande qu'il **apprenne** |
| 7 | **`NOT_YET_COLLAB`** | Un toast qui ment, que la fiche 05 § 5.4 interdit | Le retirer change ce que l'utilisateur voit : cela lui revient |

---

## 7. La question qui n'est pas technique

**Faut-il dégrader la résolution pour un gain de 6 à 13 % ?**

La charte dit : « 100 fps minimum constant quoi qu'il se passe, **sinon on pixelise** ». Le
mécanisme existe et obéit. Mais la mesure de cette session dit que **pixeliser n'achète pas
ces 100 fps** — ça les approche d'un dixième, en échange d'un pixel d'écran qui en montre
quatre.

Trois issues, et le choix engage la vision du produit :

1. **Garder la réduction** telle quelle : elle gratte 6 à 13 % et abîme visiblement.
2. **La supprimer** : image toujours nette, cadence qui baisse sur les scènes lourdes, en
   attendant que le pic des tuiles (§ 6.1) et le téléversement (§ 6.2) soient traités.
3. **La réparer** : faire qu'elle n'annule plus les tuiles — c'est un chantier à part entière,
   et rien ne garantit qu'elle rapporte davantage une fois réparée.

Ma recommandation, argumentée : **2, puis 3**. Un levier qui coûte visiblement et rapporte peu
n'est pas un levier ; et tant qu'il existe, il masque le vrai coût des scènes lourdes et
retire l'urgence de traiter les postes du § 6.

---

## 7bis. Le parallélisme — la découverte qui change la feuille de route

### La question de l'utilisateur, et elle était la bonne

> « Je ne sais pas comment Glucose Tauri fonctionne mais lui pouvait avoir des milliards
> d'images sans lager du tout, et de même sur téléphone. Comment ça se fait qu'on a une techno
> qui est mieux juste à côté ? »

**Ce que le navigateur fait et que nous ne faisons pas** : il ne recompose jamais les pixels.
Les images vivent en textures sur la carte, déplacer la vue change une matrice, et
l'interpolation bilinéaire est **câblée dans les unités de texture** — elle ne coûte rien.
Figma tient ses millions d'objets ainsi, par tuiles et par shaders.

La charte interdit d'y répondre par le GPU. Restait à savoir ce que le processeur peut.

### Ce que la mesure a trouvé, et c'est gênant

`bench_tuiles`, sur 2560 × 1600, agrandissement ×1,5 :

| | |
|---|---|
| au texel le plus proche | **3,19 ms** — pixelisé, ce qu'on faisait |
| interpolé | **20,68 ms** — net, six fois trop cher |

J'ai cru que ces 20 ms venaient de *tiny-skia*. **Faux** : le banc utilise notre propre
primitive, REPORT-1, déjà monomorphisée. Alors j'ai cherché où elles vont, et la réponse
tenait en deux recherches :

```text
is_x86_feature_detected  ->  aucune occurrence dans tout le dépôt
thread::spawn            ->  une seule, dans l'atelier de DÉCODAGE
```

**La composition des pixels tournait sur un cœur, en scalaire, sur une machine qui en annonce
seize.** C'est exactement ce que la charte refuse : *« à quoi ça sert de se priver de physique
et de matériel lorsqu'on le possède ? »*

### `bench_bandes` — le découpage de la destination

| fils | durée | facteur |
|---|---|---|
| 1 | 21,78 ms | — |
| 4 | 4,70 ms | 4,3× |
| 8 | 3,31 ms | 6,1× |
| **16** | **2,38 ms** | **8,5×** |

**Interpoler sur seize fils coûte moins que pixeliser sur un.** La pixelisation n'était pas un
compromis entre la vitesse et la beauté : c'était le prix de n'avoir jamais utilisé la machine.

La grille interpole donc **toujours**. Sur un pincement de 429 photos : **8,25 ms pixelisé
avant cette session, 4,70 ms net aujourd'hui**.

> **Le SIMD reste entier.** C'est l'autre moitié du facteur, et rien n'en est fait.

### Deux défauts trouvés par le test « au bit près », dont un qui dormait

Le test exige ce que la charte exige : deux voies, les mêmes pixels au bit près. Il a refusé
deux fois.

1. **À moi** : je retranchais le haut de la bande *avant* d'arrondir. `f32::round` s'éloigne de
   zéro, donc tout demi-pixel passant du côté négatif basculait. C'est la règle des « croix
   noires » transposée d'un axe à l'autre : **un bord partagé s'arrondit une fois, dans le
   repère où il est commun**.
2. **Dans REPORT-1, et il précède les bandes** : l'échantillonnage s'ancrait sur la **zone** —
   ce que le clip laisse voir — au lieu de la **pose**. Or les pas sont tronqués à 2⁻¹⁶ de
   texel et la boucle les additionne : **deux clips différents rendaient des pixels différents
   pour la même tuile au même endroit**. Silencieux, et contraire à tout ce sur quoi le cache
   de tuiles repose.

---

## 7ter. La réduction de résolution — quatre tours pour une réponse

### Ce que j'ai fait de travers, et c'est une leçon de méthode

J'ai corrigé le modèle de résolution **trois fois d'affilée** en jugeant sur des chroniques de
terrain. Elles ne mesuraient pas la même chose : l'une sans un pincement, la suivante avec
**272**, la dernière avec **280 images d'édition de texte**. Trois conclusions successives
tirées d'une comparaison qui n'en autorisait aucune.

**La raison pour laquelle je n'avais pas de banc** : `bench_salissure` appelait `render`, qui
ne réduit jamais — c'est `rendre_reduit` qui le fait, et seul `app/peinture.rs` l'appelle. La
réduction **n'avait jamais été mesurée par un banc**, alors qu'elle décide de 40 à 80 % des
images et qu'elle *est* ce que l'utilisateur voit comme « trop pixelisé ».

### Ce que le banc dit, une fois écrit

429 photos, un pincement, 2560 × 1600 :

| régime | médiane | pire image | tuiles au pic |
|---|---|---|---|
| **net** | **4,21 ms** | 32,93 ms | 77 |
| réduit ×2, avec tuiles | 4,70 ms | 15,43 ms | 24 |
| réduit ×4, avec tuiles | 5,58 ms | 8,89 ms | 8 |
| réduit ×2, **sans** tuiles | 6,76 ms | 15,79 ms | 0 |
| réduit ×4, **sans** tuiles | 6,12 ms | 9,98 ms | 0 |

1. **Faire passer la scène réduite par la grille est bon** : −2,06 ms à ×2. Le cache en était
   exclu au motif qu'une scène réduite « est déjà une pixelisation » — le raisonnement
   confondait un moyen de dégrader avec un **cache**.
2. **Réduire ne rapporte rien en régime permanent** : 4,21 → 4,70 → 5,58 ms. La courbe monte.
   `agrandir` coûte plus que ce que la grille économise.
3. **Réduire a un seul mérite, et il est grand : le pic.** 32,93 → 8,89 ms, et 77 tuiles → 8.

### Ce que cela désigne, et ce n'est pas ce que je croyais en commençant

La réduction **n'est pas à supprimer** : elle est le seul outil qui existe contre le pic. Mais
elle est déclenchée par un modèle qui regarde le **typique**, alors que son seul bénéfice est
sur le **pic** — d'où 80 % d'images abîmées pour un événement qui en concerne une sur 240.

Et le pic **se prévoit** : c'est le franchissement d'une octave, et le niveau dyadique est
connu *avant* de dessiner. Un modèle qui réduirait pour cette image-là, et pour elle seule,
rendrait la scène nette partout ailleurs.

**C'est le chantier suivant, et il est le plus prometteur de tous.** L'autre voie, qui se
mesure aussi : paralléliser la *peinture* des tuiles comme leur composition l'est maintenant —
77 tuiles sur seize fils, c'est cinq par fil.

---

## 8. Ce qui n'a pas été vérifié, et qui devrait l'être

La dernière chronique de terrain ne mesure **rien des deux corrections du § 3**. Elle porte sur
**31 images en 1 543 secondes**, toutes en « repos », sur un canevas à **un nœud et zéro
photo** : l'application est restée ouverte vingt-cinq minutes sans qu'on la touche.

Donc, honnêtement :

* **les carrés noirs** : corrigés par un test déterministe qui échoue sur l'ancien code, **pas
  encore revus à l'écran** ;
* **la pixelisation** : la cible ne dérive plus, prouvé par un test, **pas encore jugée à
  l'œil**.

Il faut une session sur le document de 429 photos — glissements qui s'éteignent, vols
`F` / `Ctrl+1`, pincements — puis la chronique en entier.
