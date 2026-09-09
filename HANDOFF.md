# HANDOFF — Glucose (pour le prochain Claude)

> Réécrit le **2026-09-09**, fin de la session « le rideau devient un canvas ».
> Branche **`main`** · HEAD **`31f3624`** · ⚠️ **14 commits NON POUSSÉS** (voir §1).

---

## 0. TL;DR

Glucose = **Tauri v2** (Rust + WebView2) / **React 19** / **PixiJS 8** / **Zustand** / **Automerge 3**.
North star : le `.glucose` **indestructible**, et *« poser, relier, zoomer, explorer — rien d'autre »*.

**Le plan membranes est terminé.** Les six étapes (repère local → focus → rideaux
→ appartenance → étirement → rideau-canvas) sont livrées et branchées. Il ne
reste pas d'étape suivante connue : la prochaine session part d'une demande
neuve, pas d'un reste.

**État technique** : `tsc --noEmit` 0 erreur · **947 tests TS verts** (64 fichiers) ·
`biome check src` 0 erreur (14 warnings `any` **pré-existants**).

### Les trois premières choses à faire

1. **Pousser.** Quatorze commits attendent. Le token GitHub des sessions
   précédentes a été refusé (`Invalid username or token`) : **demander un PAT à
   jour** à l'user. Aucune autre méthode d'authentification n'est prévue — ne
   pas improviser. Rien n'est perdu : arbre propre, 14 commits d'avance.
2. **Lire le §3** (les six invariants). Ils portent toute l'architecture, et
   trois d'entre eux sont contre-intuitifs — les casser sans le savoir est facile.
3. **Faire tourner l'application.** Tout ce qui suit est prouvé par des tests
   d'intégration qui montent les vraies couches et relisent le store, mais
   **rien n'a été vu tourner** : PixiJS reste hors tests, et le ressenti (les
   temporisations, la fluidité, la générosité des poignées) appartient à l'user.

---

## 1. Git — état exact

```
31f3624  test: le test instable est identifie                ← HEAD, non poussé
b91c15f  feat(rideaux): monde de clic a part entiere         ← non poussé
04dcfc6  feat(rideaux): largeur de la languette              ← non poussé
72fb2cf  feat(rideaux): le panneau monte le canvas           ← non poussé
f7e6593  feat(rideaux): le rideau porte un board             ← non poussé
eb285e6  feat(membranes): le passage fluide                  ← non poussé
9fc92d6  fix(membranes): les fleches suivent ce qu'elles relient
38477c4  refactor(membranes): retirer le code mort
bb45246  feat(membranes): le mode etire pousse pour de vrai
4082b73  docs: passation de session
3bb8db2  feat(membranes): les boutons de mode
93d1033  feat(membranes): le resolveur pilote le rendu
b8a0743  feat(membranes): l'appartenance devient reelle
9c399f8  feat(membranes): le rideau, en vrai
```

Workflow de push observé (à reprendre tel quel) :
`git fetch <url-avec-token> main` → vérifier `git rev-list --left-right --count HEAD...FETCH_HEAD`
→ `git push <url-avec-token> main:main`. **Jamais de force-push.** Le token
passe en URL ponctuelle et n'est **jamais** écrit dans `.git/config`.

---

## 2. Ce qui existe

### Sélection au clic — `hitPriority.ts` + `pickArbiter.ts` ✅

Ordre : poignée → bord de conteneur → flèche → image → note → texte → corps de
conteneur. À rang égal entre conteneurs, **le plus petit gagne**.

Deux points non évidents :

- **Le cycle « re-clic = cible suivante » avance au RELÂCHEMENT**, jamais à
  l'appui. À l'appui on ne sait pas encore si le geste sera un clic ou un
  glisser ; avancer là faisait que « je clique mon image, puis je la tire »
  attrapait la membrane.
- **Le texte est un TERMINUS.** Le cycle s'y arrête, parce qu'un double-clic sur
  un bloc éditable ouvre l'éditeur : en faire une étape intermédiaire ferait
  manger le clic suivant par l'éditeur.

Poignées : préhension de **24 px écran** (le carré dessiné en fait 9), plafonnée
à 35 % du petit côté pour qu'un bloc minuscule reste déplaçable.

**Le registre est indexé par (PORTÉE, type de couche)** — `main` pour la scène,
`curtainScope(boardId)` pour chaque rideau. C'est ce qui permet à deux canvas
d'exister sans se voler les clics. Le drapeau de détournement (`markHijack`)
est scopé pour la même raison : global, un clic détourné dans un rideau aurait
avalé le double-clic de la scène.

### Le repère des membranes — `membraneSpace.ts` ✅

Trois modes : `classic` (implicite, comportement historique), `minimized`,
`stretched`. Voir §3 pour les invariants. Branché sur : sprites Pixi, hachage
spatial / culling, collision au clic, curseur de survol, les trois couches
d'annotation, et `moveSelected`.

### Mode Focus — `membraneFocus.ts` ✅

Zoomer assez sur une membrane → la caméra se cale dessus, le fond prend sa
couleur teintée, tout le reste disparaît. Dézoomer de 20 % en sort.

**L'entrée et la sortie sont volontairement ASYMÉTRIQUES** : on entre sur la
couverture d'écran (≥ 92 %), on sort sur le dézoom relatif à l'échelle du
cadrage. Une fois entré, la couverture n'est **plus jamais** consultée — sinon
le recadrage, en ajoutant ses marges, provoquerait sa propre annulation. Un test
mesure que la couverture après cadrage (0,774) est sous le seuil d'entrée : le
piège est réel, l'asymétrie est ce qui l'évite. L'animation de cadrage (320 ms)
tient **strictement** dans le temps mort de la décision (400 ms).

### Mode étiré — `membraneStretch.ts` + `membraneStretchRuntime.ts` ✅

Une membrane étirée grandit quand son contenu déborde, et **bute sur les
obstacles** : elle ne les recouvre pas et ne les capture pas (décision prise avec
l'user). `stretchPlan` rend `allowed` et `desired` ; **on applique toujours
`allowed`**. Les bloqueurs sont entourés en pointillé et l'avertissement offre un
saut vers le premier — il est souvent hors écran.

L'étirement se joue **après** l'appartenance, jamais avant : un élément qu'on
vient de déposer doit compter dans l'étendue, sinon la membrane grandirait avec
un geste de retard.

### Passage fluide — `membraneTween.ts` ✅

Une image qui entre ou sort d'une membrane minimisée change d'échelle en
douceur au lieu de sauter. **L'animation vit côté rendu, pas dans les données** :
le document reçoit la valeur finale tout de suite, c'est l'affichage qui rattrape.
Un pair ne voit donc jamais une position intermédiaire.

### Rideaux — `curtainModel.ts` + `curtainPanel.ts` + `CurtainCanvas.tsx` ✅

Panneau personnel au bord droit, visible **uniquement en mode focus**. Survol
pour déployer, sans clic. Languettes empilées, une par personne, à sa couleur.

**Le survol ne peut pas battre, et c'est géométrique** : se déployer pousse la
frontière vers la gauche, donc elle *fuit* le curseur qui a déclenché le
déploiement ; se replier la pousse à droite, même raison. L'animation renforce
toujours la condition qui l'a déclenchée. La temporisation (90 ms / 40 ms) est là
contre l'**ouverture accidentelle**, pas contre le battement.

Permissions, deux champs pour trois usages :
`private` → **carnet** · `shared`+`owner` → **vitrine** · `shared`+`everyone` → **atelier**.
Un rideau neuf est **privé**. `canEdit` revalide la cohérence plutôt que de faire
confiance à la combinaison stockée.

**Le contenu d'un rideau est un BOARD.** C'est la décision qui commande tout le
reste : dans Glucose, tout outil travaille sur un board, donc en donner un au
rideau lui offre membranes, flèches, images, alignement, undo et synchro sans en
réimplémenter un seul. `ensureCurtainBoard` le fabrique à l'ouverture — le même
appel sert le rideau neuf et celui d'avant, il n'y a pas de code de migration à
faire vivre à côté.

Le rideau est **un monde de clic autonome** : sa caméra, son board, sa sélection,
son arbitre. Les couches ont appris trois choses pour ça, et rien de plus —
`viewportEvent` (quelle caméra suivre), `pickScope` (sous quelle portée
s'inscrire), `onMove` (qui déplacer). Les images y sont en DOM et leur
préhension est portée par `CurtainCanvas` : il n'y a pas de sprite pour la
porter.

### Modes de membrane — `MembraneOptions.tsx` ✅

Barre contextuelle quand une membrane est seule sélectionnée. Classique →
minimisée / étirée, **aller sans retour** (bouton désactivé, pas caché).

### Identité — `multiplayer/localUser.ts` ✅

Nom + couleur modifiables, persistés en `localStorage`, avec un **identifiant
stable** (`id`) qui porte la propriété des rideaux — le nom ne peut pas jouer ce
rôle, il change.

---

## 3. Les six invariants — à lire avant de toucher aux membranes

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
**Ne jamais court-circuiter ce chemin rapide.**

**⑤ Toute écriture dans le document doit être DÉTACHÉE.**
Automerge refuse qu'un objet déjà présent y soit réinséré
(`Cannot create a reference to an existing document object`). Réécrire un tableau
réinsère fatalement les éléments non touchés. `detachCurtains` recopie champ par
champ — **et un garde-fou compare les clés du rideau à celles de sa copie**,
parce que cette liste avait déjà oublié un champ (`boardId`) ajouté après elle.
**Ce bug ne se voit qu'en collaboration, au deuxième élément.**

**⑥ Une session d'alignement ne fait pas qu'aimanter.**
Elle convertit le déplacement **total depuis le grab** (ce que les couches savent
calculer) en delta **incrémental** (ce que les déplaceurs attendent). La
court-circuiter fait partir les éléments deux fois trop loin. Une couche qui doit
se passer de l'aimantation prend `beginRawMoveSession`, pas rien.

---

## 4. Ce qui n'a pas été fait, et pourquoi

- **Le mouvement d'échange en profondeur** (une seconde façon d'ouvrir le rideau,
  gardée sur la maquette) n'est pas dans l'app : l'user a choisi le survol.
- **L'aimantation ne joue pas dans un rideau.** `beginSelectionSnap` lit la
  sélection globale et le board actif ; y brancher le rideau demanderait une
  seconde source de cibles. Le geste y est simplement libre.
- **Le rideau n'a pas de dossiers.** Sa couche n'est pas montée, et l'arbitre du
  rideau passe donc `folders: []`. Rien ne l'interdit, ça n'a pas été demandé.
- **Renommer `curtain`.** L'user hésitait entre *rideau*, *coulisse* et *loge*.
  Le code dit `curtain` partout. Renommer coûtera peu maintenant, beaucoup plus
  tard.
- **« Réservé »** est implémenté comme *réservé au propriétaire*. S'il voulait
  dire « réservé à une personne nommée que je désigne », c'est un champ de plus.

---

## 5. Dettes et pièges connus

- **PixiJS n'est pas couvert par les tests** (jsdom n'a pas de WebGL). Tout ce
  qui touche aux sprites se raisonne, ne se vérifie pas. Les images du RIDEAU
  font exception : elles sont en DOM, et ce sont les seules dont la préhension
  soit testée.
- **Le redimensionnement d'une image dans une membrane minimisée** lit la
  géométrie naturelle et le curseur en coordonnées écran-monde : exact tant que
  `k == 1`, approximatif sinon. C'est le comportement de la scène depuis
  toujours ; le rideau le reproduit **volontairement**, plutôt que d'inventer une
  règle divergente.
- **Le test instable est identifié** : c'était le smoke test de
  `HtmlAnnotationLayer`, et un **timeout**, pas une assertion — son import
  paresseux tire react-markdown, remark, rehype et KaTeX. Délai élargi à 20 s.
- Les 14 warnings `biome` sur `any` sont **antérieurs** à ces sessions.

---

## 6. Comment cet user travaille

- **Il écrit en français, vite, sans ponctuation.** Prendre le temps de
  reformuler ce qu'on a compris **avant** de coder : ça a évité plusieurs
  contresens coûteux.
- **Il perd le fil entre les sessions** et le dit. Commencer par établir l'état
  réel — `git log`, la suite de tests, le code — plutôt que de faire confiance à
  un document, celui-ci compris : la passation précédente avait cinq commits de
  retard sur `HEAD`.
- **Il valide sur du concret.** Quand une intention visuelle n'était pas claire,
  une **maquette interactive publiée en Artifact** a tranché en un échange là où
  trois paragraphes n'y arrivaient pas. À refaire.
- **Il fait confiance au déterminisme** : *« tu n'es pas obligé de lancer le
  logiciel tant que mathématiquement tu es certain »*. En contrepartie il attend
  de **vraies preuves** — modules purs, simulations, tests d'intégration qui
  montent les vraies couches et relisent le store.
- **Il apprécie qu'on signale ce qu'on n'a pas vérifié.**
- **Commits séparés par chantier**, messages en français qui expliquent le
  *pourquoi*, pas le *quoi*. Attribution :
  `Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>`.
- **`style.md` fait loi** : Glucose est brutaliste, chrome monochrome strict, la
  couleur appartient au contenu de l'utilisateur — jamais à l'interface.
