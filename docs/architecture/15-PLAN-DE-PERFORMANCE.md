# 15 — Le plan de performance : 400 fps, et ce qui l'empêche vraiment

> **Rôle de ce document.** La charte demande trois choses simultanées : ~400 fps sur PC,
> 120 fps sur une tablette au bon écran et au processeur faible, 45-60 fps sur un vieux
> téléphone — et, quoi qu'il arrive, **pouvoir toujours se déplacer** pendant qu'une action
> lourde se fait en cascade.
>
> Ce document dit ce qui empêche cela **aujourd'hui, mesuré**, et dans quel ordre le lever.
> Il ne propose rien qui ne soit chiffré.
>
> **Date** : 2026-09-17 · mesures prises au commit `b423b01`, profil `release`,
> Intel Core Ultra 9 + Arc 140T.

---

## 1. Le mur, et il est arithmétique

Le premier réflexe, devant « c'est lent », est de chercher l'algorithme coupable. Ici, il n'y
en a pas. Voici ce qu'une image coûte **avant qu'un seul pixel de contenu soit dessiné** :

| Définition | Mo par image | Effacer le fond | Recopier l'image | Débit atteint |
|---|---:|---:|---:|---:|
| 1440 × 900 | 4,9 | 0,29 ms | 0,33 ms | 16,6 Go/s |
| 2560 × 1600 | 15,6 | **0,36 ms** | 0,58 ms | 42,9 Go/s |
| 3840 × 2160 (4K) | 31,6 | **2,90 ms** | 3,02 ms | 10,7 Go/s |
| 7680 × 4320 (8K) | 126,6 | 10,38 ms | 8,20 ms | 11,9 Go/s |

Deux fois plus de pixels entre 2560 × 1600 et la 4K, mais **huit fois plus cher**. Ce n'est pas
une anomalie de mesure : 15,6 Mo tiennent encore largement dans le cache de dernier niveau,
31,6 Mo n'y tiennent plus. Le débit s'effondre de 42,9 à 10,7 Go/s au franchissement.

La conséquence se lit directement :

| Définition | Reste pour dessiner à 400 fps | à 144 fps | à 60 fps |
|---|---:|---:|---:|
| 2560 × 1600 | 1,96 ms | 6,41 ms | 16,13 ms |
| 3840 × 2160 (4K) | **−0,19 ms** | 4,25 ms | 13,97 ms |

**En 4K, 400 images par seconde sont hors d'atteinte avant même qu'on ait dessiné quoi que ce
soit.** Effacer le fond coûte déjà plus que la période entière. Aucune optimisation
d'algorithme ne change cela, parce qu'il ne s'agit pas d'un algorithme : il s'agit d'écrire
31,6 Mo en mémoire, quatre cents fois par seconde, soit 12,6 Go/s **rien que pour du noir**.

Le banc des étapes le confirme sur une scène réelle : **4K, dix mille nœuds, aucun visible à
l'écran — 5,47 ms par image.** Pas un nœud dessiné, et déjà deux fois le budget.

```
clear=2.79  halos=0.97  ui=0.72  grid=0.12  annotations=0.10  images=0.00
```

**C'est le constat central de ce document : tant qu'une image est intégralement réécrite en
mémoire centrale, la cadence visée est un problème de bande passante, pas de code.**

---

## 2. Trois machines, trois murs différents

La charte vise trois régimes. Ils n'ont pas le même goulot, et un seul plan ne les sert que
s'il s'attaque au facteur commun.

| Machine | Cible | Pixels | Ce qui bloque en premier |
|---|---:|---:|---|
| PC, écran 400 Hz | 400 fps | 4-8 Mpx | **la bande passante mémoire** : 2,5 ms par image, 31 Mo à écrire |
| Tablette, bon écran, processeur faible | 120 fps | 4-8 Mpx | **la bande passante, plus durement** : beaucoup de pixels, peu de cache, mémoire lente |
| Vieux téléphone | 45-60 fps | 1-2 Mpx | **le processeur et la mémoire disponible** : peu de pixels, mais tout est lent et il faut tenir dans la RAM |

Le cas le plus dur n'est pas le PC à 400 Hz : c'est **la tablette**. Un bon écran impose autant
de pixels qu'un PC, et le processeur qui doit les écrire est trois à cinq fois plus lent, avec
un cache plus petit et une mémoire partagée avec le processeur graphique. Une tablette à
120 fps sur 4 Mpx demande d'écrire 2 Go/s en continu — sur un appareil qui chauffe et se bride.

**Le facteur commun aux trois est le même : le nombre d'octets écrits par image.** C'est donc
là, et nulle part ailleurs, que le plan doit frapper en premier.

---

## 3. L'invariant central — le canevas est un tore (TORE-1)

### Le raisonnement

Quand la vue se déplace de `(dx, dy)`, **l'immense majorité de ce qui était à l'écran y est
encore** : seule une bande de largeur `|dx|` et une bande de hauteur `|dy|` sont neuves. Un
déplacement de dix pixels par image en 4K découvre 10 × 2160 + 3840 × 10 ≈ 60 000 pixels, soit
**0,23 Mo — contre 31,6 Mo pour tout réécrire. Cent trente-sept fois moins.**

La réponse évidente serait de recopier la partie commune et de ne dessiner que les bandes. Mais
le tableau du § 1 l'interdit : recopier 4K coûte 3,02 ms, **aussi cher qu'effacer**. Déplacer
les pixels annule le gain.

Donc on ne les déplace pas. **On déplace l'origine.**

### La formule

Le tampon de rendu est traité comme un **tore** : continu en `x` et en `y`, refermé sur
lui-même. Un point du monde s'écrit toujours à la même adresse modulo la taille du tampon :

```
adresse(px, py) = ( (px − origine_x) mod W , (py − origine_y) mod H )
```

Un déplacement de la vue est alors **une soustraction sur `origine`, et rien d'autre**. Aucun
pixel ne bouge. Les seules régions à redessiner sont celles que le modulo vient de faire
basculer de l'autre côté — exactement les bandes neuves, et exactement une fois.

La présentation sait déjà lire une texture ; lui donner une origine torique est une
modification du nuanceur, où `fract()` fait le modulo gratuitement, sur du matériel dont c'est
le métier. Le coût d'un déplacement passe de *la surface de l'écran* à **la surface découverte**.

### Ce que cela règle, et ce que cela ne règle pas

| Geste | Avant | Avec le tore |
|---|---|---|
| Se déplacer | tout réécrit | les bandes neuves seules — proportionnel à la vitesse |
| Ne rien faire | tout réécrit | rien du tout |
| Une carte qui change | tout réécrit | son rectangle seul |
| **Zoomer** | tout réécrit | **tout réécrit — le tore n'y peut rien** |
| Redimensionner la fenêtre | tout réécrit | tout réécrit, une fois |

Le zoom est le cas que le tore ne couvre pas : il ne préserve aucune adresse. C'est traité au
§ 5, et c'est le seul endroit de ce plan où une question reste **ouverte et à mesurer**, pas
tranchée sur le papier.

### Pourquoi c'est cet invariant et pas un cache

Un cache de scène classique garderait une copie de l'image précédente et la recopierait. Il
paierait la recopie. Le tore ne garde rien de plus que le tampon qui existe déjà, n'alloue rien,
et **aucune constante n'y apparaît** : ni taille de cache, ni seuil de déplacement, ni
heuristique de « ça vaut le coup ». Le gain est exactement le rapport entre la surface
découverte et la surface totale — une quantité géométrique, pas un réglage.

C'est la forme que la charte demande : la constante arbitraire ne rétrécit pas, elle n'existe
pas.

---

## 4. Le plan, par ordre de rapport gain / risque

### Vague A — Le nombre d'octets écrits par image

| # | Ce qu'on fait | Gain attendu | Risque |
|---|---|---|---|
| **A.1** | **Suivre ce qui a changé** — chaque passe déclare le rectangle qu'elle salit ; une image ne redessine que l'union. Une scène immobile ne coûte plus que la présentation. | immobile : 5,5 ms → **~0,1 ms** | faible, mais il faut qu'**aucune** passe ne mente sur son rectangle — c'est un invariant à tester mécaniquement |
| **A.2** | **Le tore (TORE-1)** — le déplacement devient une origine, pas une recopie. Dépend de A.1. | déplacement 4K : 5,5 ms → **~0,4 ms** | modéré : touche la présentation et toutes les passes |
| **A.3** | **Effacer seulement ce qui ne sera pas recouvert** — le fond est déjà écrit par-dessus dans la plupart des cas. | 4K : −2,9 ms quand la scène couvre | faible |

**A.1 est le prérequis de tout le reste, et c'est aussi le plus gros changement
d'architecture** : aujourd'hui `render()` dessine tout, inconditionnellement, dans l'ordre. Il
doit devenir : « voici ce qui a changé, redessine cela ».

### Vague B — Ce qui bloque l'image sans être du rendu

| # | Ce qu'on fait | Pourquoi |
|---|---|---|
| **B.1** | **Décoder les images hors de l'image** — `load_image_impl` décode dans la boucle de rendu. Une photo de 12 Mpx coûte **351 ms** : c'est cent quarante images perdues. | c'est ce qui fait « figer » à l'import, et c'est indépendant de tout le reste |
| **B.2** | **Borner le cache d'images** — il n'a aucune limite ; la trace de l'utilisateur montrait **650 Mo**. Sur un téléphone, c'est la mort du processus. | la cible Android l'exige ; la borne se déduit de la mémoire disponible, elle ne se choisit pas |
| **B.3** | **L'ordonnanceur en cascade (CASCADE-1)** — `cadence.rs` sait déjà calculer le temps libre d'une image. Personne ne l'appelle. | c'est littéralement la promesse de la charte, et le module qui la tient est mort |
| **B.4** | **Brancher `Cadence` sur l'écran réel** — `MonitorHandle::refresh_rate_millihertz()`. Aujourd'hui tout suppose 60 Hz. | un budget faux fait optimiser dans la mauvaise direction |

### Vague C — Ce qui ne peut venir que du processeur graphique

Même toutes les vagues A et B faites, un cas reste hors d'atteinte : **un zoom continu en 4K**.
Il réécrit tout, par nature, et tout réécrire coûte 2,9 ms d'effacement plus le contenu.

C'est là que le dessin par le processeur graphique cesse d'être une optimisation pour devenir
la seule réponse. Un GPU intégré dispose de 100 à 200 Go/s contre 10 Go/s mesurés ici, et
efface une image 4K en quelques dizaines de microsecondes parce qu'il a du matériel dédié à
cela.

| # | Ce qu'on fait | Note |
|---|---|---|
| **C.1** | **La scène est dessinée par le GPU** (`vello`, validé précédemment) | supprime en même temps le téléversement de 31 Mo par image, qui est un second mur du même ordre |
| **C.2** | **Atlas de textures** pour les images et les glyphes | conséquence de C.1, pas un chantier séparé |

**Ordre recommandé : A.1 → B.1 → A.2 → B.3/B.4 → A.3 → B.2 → C.** Les vagues A et B donnent
l'essentiel du gain ressenti pour un risque contenu, et elles restent utiles **après** le
passage au GPU : ne pas redessiner ce qui n'a pas changé est vrai sur toutes les machines, et
c'est ce qui fera tenir le vieux téléphone et la tablette.

---

## 5. La seule question que ce plan laisse ouverte

**Le zoom continu, et la netteté pendant le mouvement.**

Pendant un zoom à la molette, rien n'est réutilisable : chaque image a une échelle différente.
Deux réponses existent :

* redessiner net à chaque image — correct, mais c'est le cas qui tient le moins la cadence ;
* rééchantillonner l'image précédente pendant le geste, et redessiner net à l'arrêt — la
  cadence tient toujours, au prix d'un flou transitoire.

La charte est explicite : **à l'arrêt, net à quasi 100 % ; pendant le mouvement, c'est à peser,
tester et mesurer — surtout pas à trancher sur le papier.** Ce document ne tranche donc pas. Il
propose un protocole : implémenter les deux derrière un même invariant, capturer une séquence
de zoom dans les deux régimes, et regarder — l'œil décide, pas le raisonnement.

---

## 6. Ce que ce plan refuse de faire

* **Aucun LOD sémantique.** Décidé définitivement, et rappelé ici parce que c'est la première
  chose qu'un plan de performance propose habituellement. Dégrader ce qui est *montré* trahit
  la carte. Le LOD de rendu sur les images, lui, est acquis et souhaité.
* **Aucun seuil empirique.** Pas de « au-delà de N nœuds, on simplifie ». Chaque gain de ce
  plan est une quantité géométrique — la surface découverte, le rectangle sali, le niveau de
  pyramide — et se calcule.
* **Aucune dégradation silencieuse.** L'application s'adapte à la machine par ce qu'elle
  *évite de refaire*, jamais par ce qu'elle renonce à montrer.

---

## 7. Comment on saura que ça marche

Les bancs existants mesurent un rendu complet et ne verront **rien** des vagues A et B, puisque
leur objet est précisément de ne pas rendre. Il faut donc, avant de commencer :

| Banc | Ce qu'il doit mesurer |
|---|---|
| `bench_bande_passante` | le plancher par définition d'écran — **écrit, il fonde ce document** |
| `bench_immobile` | une scène qui ne change pas, cent images de suite — doit tendre vers zéro |
| `bench_deplacement` | un déplacement continu, en fonction de la vitesse — doit être proportionnel à la vitesse, pas à la surface |
| `bench_import` | le temps entre le lâcher d'un fichier et sa première image — et surtout **la cadence pendant** |

Le dernier est le seul qui mesure ce que l'utilisateur a décrit. Aucun banc actuel ne le fait,
et c'est pourquoi une journée d'optimisation du rendu n'a pas touché sa cause réelle.

---

## 8. Deux défauts vus en chemin, non corrigés

* **`coller_image` écrit un PNG dans `%TEMP%` et ne le nettoie jamais**, et le document
  enregistré référence ce chemin temporaire. Un projet rouvert après un nettoyage de Windows
  aura perdu ses images collées. C'est un défaut de persistance, pas de performance, mais il
  est grave.
* **`bench_presentation` est cassé** — « Surface is not configured for presentation ». Un banc
  qui ne tourne plus ne garde rien.
