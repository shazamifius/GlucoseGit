# 16 — L'adaptativité : utiliser tout ce que la machine a, à l'instant où elle l'a

> **Rôle de ce document.** La fiche [`15`](15-PLAN-DE-PERFORMANCE.md) dit ce qui empêche
> d'atteindre la cadence. Celui-ci dit comment Glucose **s'adapte** — à la machine, et à ce que
> cette machine est en train de faire par ailleurs.
>
> **L'exigence, en trois moitiés qui n'en font qu'une :**
>
> 1. personne n'est exclu — tout CPU, tout GPU, toute machine du monde ;
> 2. rien n'est gaspillé — « à quoi ça sert de se priver de matériel lorsqu'on le possède ? » ;
> 3. et cela s'ajuste **pendant** l'exécution — si un jeu occupe la carte graphique, Glucose
>    doit s'en rendre compte et changer de chemin.
>
> **Date** : 2026-09-17.

---

## 1. L'erreur à ne pas refaire

J'avais conclu de « personne n'est exclu » que le matériel récent devenait secondaire. C'est
faux, et c'est même l'inverse de ce qui est demandé : **ne pas exclure n'est pas ne pas
exploiter.** Un logiciel qui tourne partout et qui, partout, va aussi lentement que sur la
machine la plus faible, n'a pas résolu le problème — il l'a distribué.

La bonne formulation :

> **Le plancher se tient par l'algorithme. Le plafond se prend sur le matériel.**

Ne pas refaire ce qui n'a pas changé (fiche 15, vague A) vaut sur un portable de 2012 comme
sur une machine de 2026 — c'est le plancher. Par-dessus, chaque machine donne tout ce qu'elle
a : AVX2 si elle l'a, la carte graphique si elle est libre, la mémoire si elle est là.

---

## 2. Le piège, et pourquoi la solution évidente ne marche pas

La solution évidente est un `if` par capacité :

```rust
if avx2_disponible() { rapide() } else { lent() }          // ← non
if gpu_disponible()  { par_le_gpu() } else { par_le_cpu() } // ← non
if memoire > 16_Go   { gros_cache() } else { petit_cache() } // ← non
```

Trois défauts, et ils sont rédhibitoires :

* **La combinatoire.** Trois capacités donnent huit chemins, quatre en donnent seize. Chacun
  doit être écrit, testé, maintenu — c'est exactement la duplication redoutée.
* **La question est mal posée.** « Le GPU est-il disponible ? » a une réponse (oui, le pilote
  répond) qui ne dit rien de ce qu'on veut savoir : *est-il libre ?* Un jeu en plein écran ne
  rend pas le GPU indisponible, il le rend **lent**. Et aucune plateforme ne répond
  portablement à « y a-t-il un jeu en cours ».
* **Les seuils.** `memoire > 16_Go` est exactement la constante arbitraire que la charte
  interdit.

---

## 3. L'invariant central — ADAPT-1 : on n'interroge pas la machine, on l'observe

**Un travail a plusieurs *voies*. Chaque voie a un débit, mesuré en continu. On prend la
meilleure constatée.**

C'est tout. Et cela répond aux trois défauts d'un coup :

* **Pas de combinatoire** : un seul mécanisme, quel que soit le nombre de voies ou de
  capacités. Ajouter une voie n'ajoute pas de chemin à écrire, juste une voie de plus à mesurer.
* **La bonne question** : on ne demande pas si le GPU est libre, on constate que présenter
  coûte 12 ms au lieu de 0,8. Le jeu se voit dans la mesure, sans jamais avoir été cherché.
  Une thermique qui bride, une machine virtuelle, un bureau distant, une batterie faible : tout
  cela se voit pareil, par le même mécanisme, sans une ligne de plus.
* **Pas de seuil** : la comparaison est entre deux mesures, pas entre une mesure et un nombre
  choisi.

### Ce qui fait que ça n'oscille pas

Deux voies proches en performance pourraient se relayer à chaque image. Le remède n'est pas une
hystérésis choisie au doigt : c'est de **comparer les mesures à leur propre bruit**. Chaque
voie porte sa moyenne et sa dispersion ; on ne bascule que si l'écart des moyennes dépasse ce
que la dispersion explique. Le seuil de bascule est donc **calculé sur les données**, pas réglé.

### Ce qui fait qu'on ne reste pas coincé

Si on ne prend jamais la voie qu'on croit moins bonne, on ne verra jamais que le jeu s'est
fermé. Il faut ré-essayer — mais pas « de temps en temps » (constante), et pas au hasard.

La formulation juste est celle des bandits à plusieurs bras : **on essaie la voie dont
l'incertitude pourrait encore la rendre gagnante**. L'incertitude décroît en `1/√n` avec le
nombre de mesures ; une voie qu'on n'a pas testée depuis longtemps voit son incertitude
remonter, et elle est ré-essayée d'elle-même. Le rythme d'exploration n'est pas choisi : il
**se déduit** de la confiance qu'on a dans chaque mesure.

C'est aussi ce qui rend le système réactif sans être nerveux : plus une voie est stable, moins
on la remet en question ; plus elle est erratique, plus on la surveille.

### La garantie qui rend tout cela sûr — VOIE-1

> **Deux voies d'une même opération produisent les mêmes pixels, au bit près.**

L'adaptation change **comment** on arrive au résultat, jamais le résultat. Sans cet invariant,
l'écran changerait d'aspect selon la charge de la machine — ce serait la dégradation
silencieuse que la charte interdit, et le rendu cesserait d'être testable.

Avec lui, c'est un test mécanique : même scène, chaque voie, empreintes identiques. Une voie
qui dévie est un bug, pas un compromis.

---

## 4. Les trois grandeurs que l'on observe, et ce qu'elles pilotent

| Ce qu'on observe | Comment | Ce que ça pilote |
|---|---|---|
| **Le débit de chaque voie** | chronométré à l'usage, normalisé par le travail fait (ns/pixel, ns/nœud) | le choix de la voie : AVX2 ou scalaire, graphique ou processeur |
| **Le temps libre de l'image** | période de l'écran − durée de l'image (`cadence.rs`, déjà écrit) | combien de travail de fond on fait avancer à cette image |
| **La mémoire disponible du système** | lue à la source, sans dépendance (§ 5) | ce que les caches ont le droit de garder |

Les trois se mesurent, aucune ne se suppose. Et les trois se réévaluent en continu : c'est ce
qui fait l'adaptation *en temps réel* plutôt qu'au démarrage.

---

## 5. La mémoire — ne pas se priver de 32 Go, ne pas mourir sur 2

L'exemple donné est exact : **occuper 650 Mo est absurde sur une machine à 2 Go et ridiculement
timide sur une machine à 32.** Une borne fixe est fausse dans les deux sens.

### La règle, et pourquoi elle n'a pas de réglage

> **Un cache ne prend jamais plus que ce qu'il laisse.**

C'est-à-dire : au plus **la moitié de la mémoire actuellement disponible**. Le facteur n'est pas
choisi — c'est le **point fixe** de la règle énoncée. « Prendre autant que je laisse » n'a
qu'une solution, et c'est un demi.

Trois propriétés en découlent, gratuitement :

* sur 32 Go libres, le cache peut prendre 16 Go — rien n'est gaspillé ;
* sur 1 Go libre, il se contente de 500 Mo — rien ne meurt ;
* et surtout, **il se contracte** : si une autre application réclame la mémoire, le disponible
  baisse, la borne baisse avec, et le cache rend la place à l'image suivante. C'est
  l'adaptation en temps réel, sur l'axe mémoire.

### Mesure, sur la machine de developpement

    totale 33,8 Go, disponible 21,6 Go  ->  part d'un cache : 10,8 Go

Contre les 650 Mo que le cache occupait sans borne ni raison. Rien n'est gaspille, et rien
n'est mis en danger : sur une machine a 2 Go libres, la meme formule donne 1 Go.

Et la borne est **relue a chaque image**, ce qui n'a de sens que si la lecture est gratuite.
Elle l'est : **840 ns** mesurees, soit trois dix-millemes du budget d'une image a 400 fps. La
decision de relire en continu ne repose donc pas sur une intuition -- un test la mesure et
echouerait si elle cessait d'etre vraie.

### Ce qu'on garde quand il faut choisir

Pas « le plus ancien ». Une image se juge sur ce que sa reconstruction coûterait, rapporté à ce
qu'elle occupe : `temps_de_décodage / octets`. On garde par utilité décroissante, et **ce qui
est à l'écran ne s'évince jamais** — l'évincer obligerait à le redemander dans la seconde.

Là encore, rien n'est réglé : les deux termes sont mesurés (le décodage est chronométré par
l'atelier, la taille est connue).

---

## 6. Ce que cela impose au code, pour que l'adaptativité ne devienne pas de la duplication

C'est la réserve juste : un système qui a plusieurs chemins risque d'avoir plusieurs codes qui
divergent. Trois règles l'évitent.

1. **Une voie est une implémentation d'une fonction pure**, pas une branche dans un algorithme.
   La logique — quoi dessiner, où, dans quel ordre — n'existe qu'une fois. Ce qui se décline,
   ce sont les primitives : remplir un rectangle, reporter une image. Elles sont petites,
   nombreuses à l'appel et rares en nombre.
2. **VOIE-1 est testé mécaniquement.** Deux voies qui divergent ne compilent pas longtemps.
3. **Une voie sans gain mesuré n'existe pas.** On n'écrit pas une variante « au cas où » : on
   la mesure d'abord, et si elle ne gagne rien sur aucune machine, elle ne naît pas.

---

## 7. Ce que cela change dans l'ordre de la fiche 15

L'adaptativité ne remplace pas la vague A, elle la complète — et l'ordre ne change pas, pour une
raison arithmétique : **le tore divise le travail par cent trente-sept, AVX2 par deux.** Faire
le facteur deux avant le facteur cent trente-sept serait optimiser le mauvais terme.

Mais « ensuite » n'est pas « jamais », et c'est la correction à retenir :

| | |
|---|---|
| **Maintenant** | A.1 (ne pas refaire ce qui n'a pas changé), la borne mémoire adaptative (§ 5) |
| **Ensuite** | A.2 (le tore), l'ordonnanceur en cascade |
| **Puis** | les voies : AVX2 détecté à l'exécution, le choix graphique/processeur mesuré |

La borne mémoire remonte en tête parce qu'elle est autonome, sans risque visuel, et qu'elle
répond directement à ce qui est demandé : ne pas se restreindre quand la machine est grande.

---

## 8. Ce qui reste à trancher, et ne l'est pas ici

**Écrire nous-mêmes les primitives de rastérisation.** Le facteur deux mesuré avec AVX2 vient
de `tiny-skia`, qu'on ne recompile pas en deux variantes : le récupérer par détection à
l'exécution suppose que ces primitives soient **à nous**. C'est la direction du contrôle quasi
total, et c'est cohérent avec la préférence exprimée pour réécrire plutôt que dépendre.

Mais c'est un chantier considérable, et il ne se décide pas en passant. Il se posera pour de
bon quand la vague A sera faite — parce qu'à ce moment-là, le rendu ne sera plus dominé par ce
que la fiche 15 appelle le mur, et le poids relatif des primitives aura changé.
