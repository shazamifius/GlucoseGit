# 21 — Les voies : deux moteurs, une machine, et la place qui reste

> **Rôle de ce document.** La fiche [`18`](18-PLAN-R-ET-D.md) posait le plan de R&D en
> supposant un seul moteur, celui du processeur. Cette fiche dit pourquoi cette supposition
> était une **erreur de lecture de la charte**, ce que la mesure en dit, et l'architecture qui
> en découle.
>
> **Date** : 2026-09-21. Rien n'est encore construit : ce document est un plan, et il le reste
> tant qu'un banc n'a pas validé chaque étape.
>
> **Le point de départ, mot pour mot** : « en fait explication de pourquoi on a dit plus
> jamais le GPU ? […] idée n'a jamais été être réfractaire au GPU, idée c'est être réfractaire
> à ne pas être spécialisé presque parfait sur 1 seul point mais sur TOUS les points en même
> temps ».

---

## 1. L'erreur, et elle est à moi

La charte porte deux phrases voisines. J'ai lu la première et sous-lu la seconde :

> « Le processeur graphique ne REMPLACE jamais le processeur. »
>
> « Le GPU reste légitime **en plus**, jamais à la place. **Les deux chemins sont de plein
> droit**, et c'est le mécanisme des voies qui choisit selon ce qui est libre. »

Ce qui est interdit, c'est **basculer parce qu'on échoue**. Construire une voie parce qu'elle
est la bonne pour ce matériel-là en est l'exact opposé.

Le coût de cette erreur est chiffrable : une session entière passée à gagner un facteur deux
sur le processeur — parallélisme des tuiles, des lueurs, cache du fond — pendant que la carte
graphique donnait cinq fois mieux **sans dégrader l'image**.

---

## 2. Ce que la mesure dit

`bench_voie_gpu`, 429 photos, 2560 × 1600, échelle non entière — le travail que le processeur
paie le plus cher :

| voie | coût par image | image |
|---|---|---|
| processeur, 16 fils, tuiles et bandes comprises | 4,2 – 6,3 ms | **pixelisée** |
| Intel Arc 140T — **GPU intégré** | **1,22 ms** | nette |
| RTX 5070 Laptop — carte dédiée | **0,26 ms** | nette |

Le filtrage bilinéaire est **câblé dans le silicium** graphique. La carte rend donc net, et
pour un cinquième du prix, ce que le processeur ne tient qu'en abîmant.

> **Ce que ce chiffre ne dit pas**, et il ne faut pas le lui faire dire : ni le texte, ni les
> formes, ni les lueurs. Le banc emploie aussi une seule texture source ; il en faudra un
> atlas ou un tableau, ce qui change la liaison des ressources, pas le nombre de pixels
> écrits.

---

## 3. L'architecture : un socle, des exécutants, un arbitre

### 3.1 Le socle — le *quoi dessiner*

Commun aux deux voies, et il existe déjà : le modèle, la géométrie, le quadtree et le culling,
les niveaux dyadiques et les empreintes de tuiles, la perception, le tempo, la chronique.

**Rien de ce qui décide ne descend dans un exécutant.**

### 3.2 Les exécutants — le *comment*

Deux algorithmes **distincts**, pas un chemin paramétré :

| | ce qu'on lui donne |
|---|---|
| **Processeur** — traitement séquentiel rapide, faible latence, branchements | des décisions, des caches, du travail irrégulier — les tuiles mémoïsées, les bandes, le report exact |
| **Carte** — parallélisme massif, débit par lot, filtrage câblé | des lots homogènes sans branche — des quads texturés, un atlas, un nuanceur |

La charte l'exige dans les termes de l'utilisateur : *« les 2 sont littéralement des scripts
différents car ne fonctionnent absolument pas du tout de la même manière »*.

**La garantie qui rend le choix sûr** reste celle de la charte : deux voies d'une même
opération produisent le même résultat. Au bit près quand c'est possible ; à défaut, avec un
écart **borné et mesuré** — un filtrage matériel n'arrondit pas comme le nôtre, et prétendre
l'inverse serait un mensonge de plus dans un test.

### 3.3 L'arbitre — *où il y a de la place*

Il ne choisit pas la voie la plus rapide dans l'absolu : il choisit celle qui est **libre**.

* Blender sature la carte → Glucose va au processeur.
* Adobe monopolise le processeur → Glucose va à la carte.
* Le téléphone chauffe et bride → le **débit** le dit, on délie avant que le système ne
  dégrade tout.

Le principe ne change pas d'un iota par rapport à la charte : **on n'interroge pas le
matériel, on observe son débit.** Aucune table de matériels, aucun `if` par capacité.

---

## 4. Le découpage, et ce qui le valide

| # | Étape | Ce qui prouve qu'elle est faite |
|---|---|---|
| **1** | **Les photos sur la carte.** Elles deviennent des textures téléversées une fois ; la scène est un lot de quads. Le chrome, le texte et les lueurs restent au processeur, composés par-dessus. | La chronique : `report` et `agrandir` disparaissent du profil, et **plus une image n'est réduite** |
| **2** | **L'arbitre.** Les deux voies mesurent leur débit ; le choix se fait à chaud, et suit le bridage comme la concurrence. | Un banc qui occupe la carte et vérifie que Glucose passe au processeur sans saut visible |
| **3** | **Le reste de la scène sur la carte** : texte, formes, lueurs. **Le fond et les lueurs sont faits** (fiche [`22`](22-SESSION-DU-21-09.md)) ; le texte et les formes restent. | Égalité d'aspect avec la voie processeur, à un écart borné — **tenue par `tests/voies_suite.rs` : 3 niveaux sur la scène entière** |
| **4** | **La spécialisation par classe de machine** : des routines qui s'assemblent selon le débit constaté. | Sur une machine sans carte utilisable, la voie processeur reste au niveau d'aujourd'hui |

**L'étape 1 est la seule qui réponde au grief de l'utilisateur** — « une interface absolument
horrible pour travailler car toujours toujours flou ». Elle est aussi la moins risquée : les
photos sont le cas le plus simple d'un rendu GPU, et le chrome continue de fonctionner tel
quel.

---

## 5. Ce qui devra être tranché en chemin

1. **La composition des deux couches.** *Tranché, et la réponse a changé en chemin* : deux
   couches d'abord (fond + lueurs + contenants dessous, le reste dessus), puis **cinq
   temps** une fois le fond et les lueurs sur la carte — et la couche du dessous **cesse
   d'exister** quand elle est vide, ce qui vaut mieux que la mettre en cache (fiche 22 § 4).
2. **Les textures des photos.** Un atlas borné, ou un tableau de textures. Le choix dépend du
   nombre de photos visibles simultanément, que le quadtree connaît déjà.
3. **Ce que devient le cache de tuiles sur la voie GPU.** Probablement rien : la carte n'a pas
   besoin de mémoïser ce qu'elle refait pour un centième de milliseconde. À mesurer avant de
   le supprimer de ce chemin — jamais de conclusion sans banc.
4. **La netteté à l'arrêt.** Le filtrage matériel au plus proche donne l'exactitude que la
   charte demande ; le bilinéaire donne le mouvement. Le basculement doit être décidé par la
   même perception qu'aujourd'hui.

---

## 6. Ce qui ne change pas

* **La voie processeur reste un vrai chemin**, pas un secours. Tout le travail de la
  fiche [`20`](20-SESSION-DU-20-09.md) la sert : les tuiles, les bandes, les lueurs
  parallélisées, le fond sauté. C'est elle qui tournera quand la carte sera prise.
* **Aucune exclusion matérielle.** Une machine sans carte utilisable doit rester servie, et
  bien.
* **Le critère de l'utilisateur**, qui juge toute optimisation : *« l'objectif d'une
  pixelisation et d'une optimisation, c'est qu'elle ne se voie pas à l'œil nu »*.
