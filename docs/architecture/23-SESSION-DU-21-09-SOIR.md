# 23 — Les deux cartes, la rampe du démarrage, et trois hypothèses que la mesure a refusées

> **Rôle de ce document.** La fiche [`22`](22-SESSION-DU-21-09.md) laissait deux questions
> ouvertes : les gels de `present` — l'hybride intégré/dédié est-il en cause ? — et le pic des
> composants. L'utilisateur a joué **deux sessions de terrain** sur le même document, les
> mêmes gestes, sur les deux cartes graphiques de sa machine. Cette fiche dit ce qu'elles ont
> tranché, ce qu'un banc neuf a chiffré pour la première fois, et surtout **les trois
> corrections que j'ai écrites et que la mesure a refusées l'une après l'autre**.
>
> **Date** : 2026-09-21, soirée · quatre commits, de `ad65716` à `ef23bd9`.
> **État vérifié** : `cargo test --workspace` exit 0, **1 355 tests verts**, clippy strict à
> zéro, onze cliquets, aucun plafond relevé.
>
> **Le point de départ, mot pour mot** : *« au niveau utilisation je trouve que tout est
> fluide MAIS un logiciel lent […] au commencement d'une action, lorsque tu as 0 vélocité et
> que tu souhaites te déplacer, que ce soit avec la minimap, le pad ou autre, on a une
> accélération progressive — et bien cette accélération-là est BEAUCOUP trop lente. »*

---

## 1. Les gels de `present` : l'hypothèse de la fiche 22 § 12 est confirmée

`GLUCOSE_CARTE=rapide` a fait basculer l'application sur la RTX 5070. Le poste `present`,
poste par poste :

| | Intel Arc 140T (intégré) | RTX 5070 (dédiée) |
|---|---|---|
| `present` médian | absent du profil | **0,72 ms** |
| `present` p99 | — | **1,22 – 1,72 ms** |
| `present` pire | **301 / 303 / 65 ms** | **2,35 ms** |
| succession | `fifo` | `mailbox` |

**Les gels de `present` disparaissent sur la carte dédiée.** C'était bien le chemin hybride :
une image rendue sur l'intégrée doit traverser le bus pour être composée par la carte qui
pilote l'écran, et ce chemin gèle. Trois sessions l'avaient nommé « jamais élucidé » ; une
variable d'environnement et deux minutes de terrain l'ont tranché.

**Ce que cela ne règle pas, et il faut le dire.** Le pire gel de la session RTX vaut encore
**725 ms, dont 717 à ne pas dessiner** — donc hors de tout code de rendu. Et le tempo est
*pire* sur la RTX : huit balayages sur **68 %** des images contre 35 % sur l'Arc. Supprimer
les gels de `present` n'a pas fait descendre le tempo, parce que ce n'est pas `present` qui
le tient.

---

## 2. Ce qui tient le tempo à huit balayages, et c'était mal nommé

Les deux chroniques désignent le même poste sur les deux cartes :

```
    blit    median 2,44 ms    p99 32,77 ms    pire 71,76 (RTX) / 150,43 (Arc)
```

Un p99 treize fois la médiane. Le tempo se cale sur le p99 ; le p99 est ce poste ; donc
**huit balayages, 33 ms par image, 39 à 65 ms de latence** — pour un rendu qui coûte 9,7 ms
en médiane et tiendrait dans trois balayages.

Sauf que `blit` ne mesurait pas ce que son nom dit. La marque était posée en fin de
`cinq_temps::presenter` et absorbait **tout** depuis l'entrée de la fonction :

```text
    scene.assurer(...)      <- cree et televerse les textures MANQUANTES
    scene.preparer(...)        les poses, les uniformes, les sommets
    fond.preparer(...)
    lueurs.preparer(...)
    couches.televerser(...)    <- le seul vrai televersement
    stage("blit")
```

`assurer` est exactement l'endroit où les **composants** se rendent quand ils changent de
palier ou entrent à l'écran — le pic de trente millisecondes que la fiche 22 § 11.2 annonce.
Il tombait dans `blit`, où il était méconnaissable.

Un téléversement de seize mébioctets à 71 ms ferait 230 Mo/s, ce qu'aucun bus ne justifie :
**le chiffre disait déjà que la marque mentait**, et personne ne l'avait lu ainsi.

> **Troisième marque mal posée de ce projet**, et la forme est toujours la même :
> `occlusion` absorbait la composition des tuiles (fiche 19 § 4.4), `recolte` l'effacement
> d'un tampon de seize mébioctets (fiche 22 § 5.4), `blit` le rendu des textures. Chaque fois
> elle a désigné le mauvais coupable pendant plusieurs sessions. Trois marques remplacent
> désormais celle-là : `textures`, `poses`, `blit`.

---

## 3. La rampe du démarrage, chiffrée pour la première fois

Aucune mesure du dépôt ne disait ce que l'utilisateur décrit. Les vingt et un bancs
chronomètrent ce qu'une image **coûte** ; `bench_demarrage` ne chronomètre rien. Il **simule**
une main qui pousse à vitesse constante sur une horloge d'affichage régulière, et lit la
courbe de ce que l'écran montre. Deux exécutions donnent le même chiffre au bit près — c'est
la seule raison pour laquelle il a le droit de comparer deux exécutions, ce que la fiche 20
§ 5.1 interdit partout ailleurs.

Main à 1 200 px/s, source à 100 Hz, écran à 240 Hz :

| tempo | montée à 90 % | retard en régime | grain (livraison régulière) |
|---|---:|---:|---:|
| 1 balayage | 100,0 ms | 59,1 px | 9,1 % |
| 3 balayages | 100,0 ms | 57,0 px | 8,0 % |
| 8 balayages | 100,0 ms | 39,9 px | 6,9 % |

**Cent millisecondes** pour atteindre neuf dixièmes de la vitesse demandée, et cinquante
pixels de retard permanent. C'est ce que l'utilisateur ressent, et c'est chiffré.

---

## 4. Trois corrections écrites, trois refusées par la mesure

C'est la partie utile de cette session, et elle vaut plus que ses gains.

### 4.1 La constante de conduite — la doc du module promettait autre chose que le code

Le haut de `elan.rs` dit, et depuis trois sessions : *« la constante de temps vaut
l'**intervalle d'émission mesuré** »*. Le code applique cinquante millisecondes fixes. C'est
le piège de la fiche 22 § 9.1 — *un commentaire qui décrit une intention passe pour une
description* — et il avait tenu trois sessions dans un module que quatre tentatives avaient
déjà retravaillé.

J'ai donc rétabli la mesure, en corrigeant au passage ce qui l'avait fait retirer : elle
divisait un **horizon fixe** par un nombre d'événements qui, lui, varie ; elle doit diviser
l'**étendue observée** par le nombre d'intervalles qu'elle contient, ce qui ne varie plus.

**Le banc a refusé.** Le grain — de combien la vitesse apparente saute d'une image à l'autre —
est passé de 34 à **192 %** en livraison groupée : soit exactement le défaut d'origine, *« des
sauts d'image comme si on avait 15 fps »*. Cette constante **achète** un facteur trois sur le
grain, et ma lecture « elle n'achète rien » était fausse faute de point de comparaison.

Essai suivant : `τ = max(pas de l'image, intervalle d'émission)` — le plus grossier des deux
quanta, puisqu'en dessous ni la source ni l'écran ne portent d'information. Refusé aussi : à
huit balayages, il ne gagne **rien** sur la montée et dégrade le grain de 6,9 à 10,0 %.

### 4.2 Et c'est en cherchant pourquoi qu'on trouve la vraie contrainte

À huit balayages, la première image d'un geste arrive 33 ms après lui et la deuxième 67 ms.
**Aucune valeur de τ ne descend sous deux images.** La montée mesurée à 100 ms est donc
d'abord une affaire de tempo, et seulement ensuite de constante de temps.

> **Le tempo est le plancher de la réactivité.** Tant qu'il tient huit balayages, retoucher
> l'élan ne peut rien donner. Ce n'est pas un réglage qui change, c'est l'ordre des chantiers.

Et la contrainte est réelle, pas contournable : à 3,3 événements de source par image, la
quantification est irréductible sans soit du retard (un lissage), soit de la prédiction. La
charte demande la seconde — « pré-optimiser », la prédiction de pose de la fiche 18 § 4 — et
c'est un chantier à part entière, avec son risque propre : une prédiction fausse fait
**reculer** le contenu, ce qui se voit bien plus qu'un retard.

### 4.3 Le tempo — un cliquet a eu raison contre moi

Puisque monter `k` n'évite pas un gel de soixante-et-onze millisecondes — il rate à huit
balayages comme il raterait à neuf —, j'ai fait que seuls les ratés **qu'un cran de plus
aurait évités** fassent monter le tempo. La symétrie exacte de `rateraient_en_dessous`, qui
commande la descente.

`test_la_cible_de_la_finesse_ne_derive_pas_avec_le_tempo` a protesté, **et il avait raison** :
un rendu systématiquement à 30 ms n'est évitable à aucun cran intermédiaire, et doit pourtant
faire monter `k` à huit — sinon **toutes** les images ratent en permanence, ce qui est le pire
des mondes. Ma règle confondait un gel rare et un coût systématiquement élevé.

**Le tempo fait déjà ce qu'il faut** : il prend le p99 de la distribution. C'est la
distribution qui est mauvaise, et son p99 s'appelait `blit`.

---

## 5. CASCADE-2 — ce qui a été livré

Le pic des composants est le p99, donc ce qui cale le tempo. Il s'étale désormais sur
plusieurs images, dans ce qui reste du plancher de la charte une fois l'image rendue
(`Cadence::tranche_de_fond`, qui existait déjà pour l'atelier de décodage). **Aucune constante
n'a été choisie.**

Ce qui a dû changer, et c'était le vrai travail : la mémoire de la carte était indexée par
**clé** — *ce que la texture montre*. Une clé nouvelle était une texture inconnue, et
l'ancienne disparaissait à la fin de l'image : impossible de garder quoi que ce soit. Elle est
indexée par **identité** — *ce que le composant est* —, portée explicitement par
`Composant::identite` et non déduite en coupant la clé, ce qui supposerait qu'aucun
identifiant ne contienne de deux-points.

**L'ordre des deux tours est ce qui rend la cascade sûre.** D'abord ce que la carte n'a pas du
tout, et celui-là ne se budgète pas — sans texture un composant ne se dessine pas, et un trou
est infiniment pire qu'un flou. Ensuite seulement ce qui a vieilli. C'est la priorité que
Chrome donne à ses tuiles, pour la même raison.

**Ce qui n'est pas mesuré : le gain.** `bench_texte` ne sait pas le montrer — chaque geste y
saute à une vue éloignée, donc sa première image voit entrer soixante-douze cartes d'un coup,
toutes urgentes et légitimement non budgétées, et c'est ce pic-là que `pire` mesure dans les
deux régimes. Le banc compare deux artefacts. Ce qui est prouvé est la **mécanique**, par
trois tests dont le premier rejoue l'ancienne loi et vérifie qu'elle concluait l'inverse.

---

## 6. Le banc du texte était aveugle, et depuis toujours

En cherchant si le pic des composants existe, `bench_texte` a répondu « le poste `textures`
n'apparaît nulle part ». Deux défauts de l'instrument, qui se couvraient l'un l'autre :

1. le tableau des postes filtrait par `med >= 0.05`. **Un poste dont la médiane est nulle et
   le pire vaut vingt millisecondes est exactement celui qu'on cherche**, et ce filtre le
   jetait. C'est la faute de la fiche 20 § 4.5 prise à l'envers : là, une somme laissait une
   image aberrante décider du typique ; ici, le typique effaçait l'aberrante ;
2. le compteur `textures_rendues` était rempli à chaque image et **affiché nulle part** —
   « un compteur déclaré et jamais lu vaut zéro », fiche 17 § 3.1, cliquet 9.

Une fois réparé, sur 480 cartes dont 72 visibles :

| geste | `textures` médian | `textures` pire | rendues d'un coup |
|---|---:|---:|---:|
| immobile | 0,01 ms | **20,02 ms** | 72 |
| zoom | 0,01 ms | **11,03 ms** | 72 |

---

## 7. Ce qui reste, chiffré — révisé

| # | Ce que c'est | Chiffre | Statut |
|---|---|---|---|
| 1 | **Le gain de CASCADE-2** | pic de 20 ms à étaler | **Non mesuré** — première chose à lire dans la prochaine chronique |
| 2 | **Ce que `blit` vaut une fois seul** | médiane 2,44 ms, p99 inconnu | Les marques sont séparées ; le terrain dira |
| 3 | **L'arbitre** (fiche 21, étape 2) | l'Arc gèle `present`, la RTX non | **Le besoin est maintenant prouvé.** Il choisit par le débit observé, jamais par une variable |
| 4 | **Les 717 ms « à ne pas dessiner »** sur la RTX | pire gel 725 ms | Hors de tout code de rendu ; jamais instrumenté |
| 5 | **La prédiction de pose** pour la rampe | montée 100 ms, plancher à 2 images de tempo | La seule voie sous le tempo ; risque de recul visible |
| 6 | **La chrome par rectangles** | `effacer` + `blit` | Dépend de ce que le point 2 dira |
| 7 | **Les pense-bêtes** comme composants | même mécanisme | Non commencé |
| 8 | **Le SIMD à l'exécution** | facteur 3 à 4 | Rien de fait |

---

## 8. Une leçon de plus, pour la liste des fiches 17, 19, 20 et 22

**Trois corrections écrites, trois refusées — et c'est la méthode qui a fonctionné, pas qui a
échoué.** Aucune des trois n'aurait été vue sans un banc ou un test qui protestait : la
première par un banc écrit le jour même, la deuxième par le même banc joué sur cinq tempos, la
troisième par un test vieux de deux sessions.

Ce qu'elles ont en commun : **chacune corrigeait un mécanisme qui n'était pas la cause.**
L'élan, le tempo et la constante de conduite subissaient tous les trois le même p99, venu
d'ailleurs. Devant un symptôme, la question n'est pas « quel mécanisme le produit » mais
« lequel le **commande** » — et c'est la même question que les six cercles vicieux posent
depuis le début.
