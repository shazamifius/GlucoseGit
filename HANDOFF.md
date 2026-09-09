# HANDOFF — Glucose (pour le prochain Claude)

> Réécrit le **2026-09-09**, fin de la session « membranes : focus, rideaux, modes ».
> Branche **`main`** · HEAD **`3bb8db2`** · ⚠️ **3 commits NON POUSSÉS** (voir §1).

---

## 0. TL;DR

Glucose = **Tauri v2** (Rust + WebView2) / **React 19** / **PixiJS 8** / **Zustand** / **Automerge 3**.
North star : le `.glucose` **indestructible**, et *« poser, relier, zoomer, explorer — rien d'autre »*.

Cette session a livré, en 11 commits, **un second repère de coordonnées** pour les
membranes (mode focus + membranes minimisées), **les rideaux personnels**, et
l'**identité collaborative modifiable**.

**État technique** : `tsc --noEmit` 0 erreur · **845 tests TS verts** (57 fichiers) ·
`biome check src` 0 erreur (14 warnings `any` **pré-existants**) · `cargo check` OK.

### Les trois premières choses à faire

1. **Pousser.** Trois commits attendent, le token GitHub de la session précédente
   a été refusé (`Invalid username or token`). **Demander un PAT à jour** à l'user.
   Aucune autre méthode d'authentification n'est prévue — ne pas improviser.
   Rien n'est perdu : arbre propre, 3 commits en avance sur `origin`.
2. **Lire le §3** (les cinq invariants). Ils portent toute l'architecture, et
   deux d'entre eux sont contre-intuitifs — les casser sans le savoir est facile.
3. Demander à l'user **par quelle étape du §4** il veut continuer. Le plan
   1→2→3 est terminé ; il reste 4, 5, 6 et il les connaît.

---

## 1. Git — état exact

```
3bb8db2  feat(membranes): les boutons de mode          ← HEAD, non poussé
93d1033  feat(membranes): le resolveur pilote le rendu ← non poussé
b8a0743  feat(membranes): l'appartenance devient reelle← non poussé
9c399f8  feat(membranes): le rideau, en vrai           ← poussé
2313dd2  feat(collab): mon nom et ma couleur           ← poussé
d384e3e  feat(membranes): rideau — modele d'interaction← poussé
85d222d  feat(membranes): mode Focus                   ← poussé
6cf266a  feat(membranes): repere local — la fondation  ← poussé
d2a887c  fix(selection): cycle de priorite au relachement
152f83c  fix(fleches): ancrer une selection par position
5be958a  feat(plugins): moteur "Cours magistral" integre
```

Workflow de push observé cette session (à reprendre tel quel) :
`git fetch <url-avec-token> main` → vérifier `git rev-list --left-right --count HEAD...FETCH_HEAD`
→ `git push <url-avec-token> main:main`. **Jamais de force-push.** Le token
passe en URL ponctuelle et n'est **jamais** écrit dans `.git/config` (vérifié).

---

## 2. Ce qui existe maintenant

### Sélection au clic — `src/canvas/hitPriority.ts` ✅ branché

Ordre de priorité : poignée → bord de conteneur → flèche → image → note → texte
→ corps de conteneur. À rang égal entre conteneurs, **le plus petit gagne**.

Le point non évident : **le cycle « re-clic = cible suivante » avance au
RELÂCHEMENT, jamais à l'appui.** À l'appui on ne sait pas encore si le geste sera
un clic ou un glisser ; avancer là faisait que « je clique mon image, puis je la
tire » attrapait la membrane. Trois tests verrouillent ça.

Poignées : préhension de 24 px **écran** (le carré dessiné fait 9), plafonnée à
35 % du petit côté pour qu'un bloc minuscule reste déplaçable.

### Le repère des membranes — `src/canvas/membraneSpace.ts` ✅ branché

Le cœur. Trois modes : `classic` (implicite, comportement historique),
`minimized`, `stretched`. Voir §3 pour les invariants.

Branché sur : sprites Pixi, hachage spatial / culling, test de collision au clic,
curseur de survol des poignées, `HtmlAnnotationLayer`, `SvgAnnotationLayer`, et
`moveSelected` côté store.

### Mode Focus — `src/canvas/membraneFocus.ts` ✅ branché

Zoomer assez sur une membrane → la caméra se cale dessus, le fond prend sa
couleur teintée, tout le reste disparaît. Dézoomer de 20 % en sort.

**L'entrée et la sortie sont volontairement ASYMÉTRIQUES** : on entre sur la
couverture d'écran (≥ 92 %), on sort sur le dézoom relatif à l'échelle du
cadrage. Une fois entré, la couverture n'est **plus jamais** consultée — sinon le
recadrage, en ajoutant ses marges, provoquerait sa propre annulation. Un test
mesure explicitement que la couverture après cadrage (0,774) est sous le seuil
d'entrée : le piège est réel, l'asymétrie est ce qui l'évite.
L'animation de cadrage (320 ms) tient **strictement** dans le temps mort de la
décision (400 ms) — un test verrouille la relation entre ces deux constantes.

### Rideaux — `curtainPanel.ts` + `curtainModel.ts` + `MembraneCurtainLayer.tsx` ✅ branché

Panneau personnel au bord droit, façon console Quake / Slide Over, visible
**uniquement en mode focus**. Survol pour déployer, sans clic. Languettes
empilées, une par personne, à sa couleur (variante A, validée sur maquette).

**Le survol ne peut pas battre, et c'est géométrique** : se déployer pousse la
frontière vers la gauche, donc elle *fuit* le curseur qui a déclenché le
déploiement ; se replier la pousse à droite, même raison. L'animation renforce
toujours la condition qui l'a déclenchée. La temporisation (90 ms / 40 ms) est là
contre l'**ouverture accidentelle**, pas contre le battement.

Permissions, deux champs pour trois usages :
`private` → **carnet** · `shared`+`owner` → **vitrine** · `shared`+`everyone` → **atelier**.
Un rideau neuf est **privé**. `canEdit` revalide la cohérence plutôt que de faire
confiance à la combinaison stockée.

### Modes de membrane — `src/components/MembraneOptions.tsx` ✅ branché

Barre contextuelle quand une membrane est seule sélectionnée. Classique →
minimisée / étirée, **aller sans retour** (bouton désactivé, pas caché).

### Identité — `src/multiplayer/localUser.ts` ✅ branché

Nom + couleur modifiables (panneau Collaboration), persistés en `localStorage`,
avec un **identifiant stable** (`id`) qui porte la propriété des rideaux — le nom
ne peut pas jouer ce rôle, il change. Un `USER_CHANGED_EVENT` fait rediffuser la
présence immédiatement.

---

## 3. Les cinq invariants — à lire avant de toucher aux membranes

**① L'échelle du contenu ne se stocke pas, elle se déduit.**
`k = min(1, largeur/étendueX, hauteur/étendueY)`. D'où, gratuitement : le `min`
des deux axes (étirer en longueur seule ne fait pas regrossir une image), le
plafond à 1, et l'impossibilité qu'une échelle stockée se désynchronise de la
taille réelle. **Ne jamais introduire de champ `scale`.**

**② L'appartenance, elle, SE STOCKE** (`membraneId`), et c'est contre-intuitif.
Une membrane minimisée est *par construction* plus petite que son contenu à
l'échelle 1 : un test d'inclusion géométrique déclarerait le contenu sorti à
l'instant même où elle le réduit — elle se viderait en rangeant. L'appartenance
est donc un **événement** : dépôt (`reconcileMembership`) ou conversion
(`containedIn`). Jamais un prédicat continu.

**③ La conversion est le seul moment où la géométrie peut décider.**
En quittant `classic`, l'échelle vaut encore 1 : naturel == effectif, l'inclusion
est sans ambiguïté. Après, le test s'inverse. C'est pour ça que l'instantané ne
se prend **qu'en quittant classique**, jamais à chaque bascule.

**④ `hasScaling()` est le garde-fou de toute la migration.**
Tant qu'aucune membrane n'est minimisée, `projectBoard` rend le board **tel quel,
au même objet près** — zéro calcul, zéro copie, chemin d'avant à l'identique.
Trois tests d'intégration le vérifient en montant les vraies couches. **Ne
jamais court-circuiter ce chemin rapide.**

**⑤ Toute écriture dans le document doit être DÉTACHÉE.**
Automerge refuse qu'un objet déjà présent y soit réinséré
(`Cannot create a reference to an existing document object`). Réécrire un tableau
réinsère fatalement les éléments non touchés. `detachCurtains` recopie champ par
champ. **Ce bug ne se voit qu'en collaboration, au deuxième élément** — un test
d'intégration l'a attrapé, pas la relecture.

---

## 4. Ce qui reste

### Étape 4 — Mode étiré et avertissements de collision
`stretchPlan()` est **écrit et testé mais appelé nulle part**. Aujourd'hui le
bouton « Étirée » écrit le mode, le résolveur le respecte (échelle 1), mais la
membrane **ne grandit pas toute seule** et rien n'avertit.

À faire : appeler `stretchPlan` quand le contenu d'une membrane étirée dépasse,
appliquer `allowed` (jamais `desired`), **entourer les `blockers`** et offrir un
saut vers l'élément fautif. Décision déjà prise avec l'user : **l'étirement bute
sur l'obstacle**, il ne le recouvre pas et ne le capture pas.

### Étape 5 — Animation de dépôt
Quand un élément entre ou sort d'une membrane minimisée, sa taille change d'un
coup. L'user veut un passage **fluide** (image qui traverse d'un point A à un
point B). Piste : animer `k` côté rendu sur ~200 ms, sans toucher aux données.

### Étape 6 — Le rideau devient un vrai canvas *(le gros morceau)*
Il ne contient aujourd'hui que des **notes texte**. L'user a confirmé vouloir
« un canvas complet ». **Décision à lui soumettre d'abord** : seconde instance
PixiJS dans le panneau (images comprises, coûteux) **ou** rendu DOM seul (textes,
notes, flèches, membranes ; images en `<img>`). Recommandation de la session
précédente : **DOM seul**.

Le modèle est prêt à l'accueillir : les rideaux vivent sur la membrane et passent
par `updateAnnotation`, donc ils héritent de l'undo et de la synchro sans surface
de store supplémentaire.

---

## 5. Décisions ouvertes — à demander à l'user

- **« Réservé »** : implémenté comme *réservé au propriétaire*. S'il voulait dire
  « réservé à une personne nommée que je désigne », c'est un champ de plus.
- **Le nom** : le code dit `curtain` partout. L'user hésite entre *rideau*,
  *coulisse* et *loge*. Renommer coûtera peu maintenant, beaucoup plus tard.
- **Le bouton « créer un rideau »** vit sur la languette, au bord droit. L'user
  l'avait imaginé à côté des boutons de mode — qui existent désormais
  (`MembraneOptions`). À déplacer si c'est ce qu'il veut.
- **Le flou de la languette** entre en tension avec `style.md`, qui proscrit le
  glassmorphisme. Il a été gardé parce que l'user l'a demandé explicitement, et
  parce que ce qu'on floute est du *contenu*. Son arbitrage, pas le nôtre.

---

## 6. Dettes et pièges connus

- **Code mort** : `stretchPlan`, `naturalDelta`, `toNatural`, `membraneAtPoint`
  ne sont appelés **nulle part** hors tests. `stretchPlan` sert à l'étape 4 ; les
  trois autres ont été rendus inutiles par des solutions plus simples
  (l'écriture inverse se fait dans `moveSelected` via `scaleOf`). À supprimer ou
  à employer, mais ne pas les laisser traîner en l'état.
- **Un test instable non identifié** : une exécution de la suite a échoué
  (815/816) puis **cinq exécutions consécutives sont passées**. Ni le test ni la
  cause n'ont été retrouvés. Si ça resurgit, noter le nom du test *immédiatement*.
- **Flèches non projetées** : `projectBoard` laisse les flèches telles quelles.
  Une flèche dont les extrémités sont dans une membrane minimisée sera mal
  ancrée. Pas encore rencontré à l'usage, mais c'est un vrai trou.
- **PixiJS n'est pas couvert par les tests** (jsdom n'a pas de WebGL). Tout ce
  qui touche aux sprites se raisonne, ne se vérifie pas.
- Les 14 warnings `biome` sur `any` sont **antérieurs** à cette session.

---

## 7. Comment cet user travaille

- **Il écrit en français, vite, sans ponctuation.** Prendre le temps de
  reformuler ce qu'on a compris **avant** de coder : ça a évité deux
  contresens coûteux cette session.
- **Il valide sur du concret.** Quand une intention visuelle n'était pas claire,
  une **maquette interactive publiée en Artifact** a tranché en un échange là où
  trois paragraphes n'y arrivaient pas. À refaire.
- **Il fait confiance au déterminisme** : *« tu n'es pas obligé de lancer le
  logiciel tant que mathématiquement tu es certain »*. En contrepartie il attend
  de **vraies preuves** — modules purs, simulations, tests d'intégration qui
  montent les vraies couches et relisent le store.
- **Il apprécie qu'on signale ce qu'on n'a pas vérifié.** Les réserves honnêtes
  (« Pixi non couvert », « le ressenti reste à toi ») ont été bien reçues.
- **Commits séparés par chantier**, messages en français qui expliquent le
  *pourquoi*, pas le *quoi*. Attribution :
  `Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>`.
- **`style.md` fait loi** : Glucose est brutaliste, chrome monochrome strict, la
  couleur appartient au contenu de l'utilisateur — jamais à l'interface.
