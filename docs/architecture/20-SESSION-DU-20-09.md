# 20 — La salissure, et deux cercles vicieux que j'ai construits moi-même

> **Rôle de ce document.** La fiche [`19`](19-SESSION-DU-19-09.md) désignait l'**étape 1 du
> plan [`18`](18-PLAN-R-ET-D.md)** — la salissure — comme le chantier suivant. Celle-ci dit ce
> que la journée en a fait. Elle dit aussi, et c'est sa partie la plus utile, **deux
> corrections que j'ai écrites et que la mesure a démenties**, dont une régression visible à
> l'écran que l'utilisateur a signalée en photo.
>
> **Date** : 2026-09-20 et 21 · huit commits, de `af2ffa4` à `574a0a4`.
> **État vérifié** : `cargo test --workspace` exit 0, **1 321 tests verts** (29 binaires),
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

C'est le tiers restant de l'étape 1, et c'est le plus gros poste qui reste. La chronique de la
dernière session au repos donne `blit` à **62,4 %** du temps. Rien n'a été tenté ici.

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

### 5.3 Un test qui prouve une égalité ne prouve pas un choix

Le test des plages comparait `reporter` à lui-même. Les deux voies étaient justes ; c'est la
décision prise **avant** l'appel qui était fausse (§ 3.1). Quand une correction change une
**décision**, le test doit porter sur la décision, pas sur son exécution.

---

## 6. Ce qui reste, chiffré

| # | Ce que c'est | Chiffre | Statut |
|---|---|---|---|
| 1 | **Le pic du changement d'octave** | 77 tuiles, **43 ms** contre 6,7 ms pour une image ordinaire, une image sur 240 | Isolé, non résolu ; le raffinement progressif a échoué (§ 4.2) |
| 2 | **Le téléversement partiel** (`blit`) | **62,4 %** du temps au repos | Étape 1 du plan 18, **non commencée** |
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
