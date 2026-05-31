# 🩺 Stabilité — Tableau de bord V1-beta

> **But de ce doc :** la liste *vivante* de ce qui doit être impeccable avant de publier
> la V1-beta sur GitHub. Pas une vision figée — un état réel, trié, qu'on vide un à un.
>
> **Règle d'or :** on attaque **un bug à la fois**, chacun verrouillé par un test pour
> qu'il ne revienne **jamais**.
>
> Légende : 🔴 bloquant · 🟠 visible/important · 🟡 cosmétique · ✅ réglé+testé
>
> **Dernière mise à jour :** 2026-05-31

---

## 🔴 Bloquants V1 (le logiciel n'est pas « à peu près fonctionnel » sans ça)

### NAV-1 — Auto-entrée de dossier : trop « large », fait entrer dans le mauvais dossier
**Statut :** ✅ réglé+testé (2026-05-31) — règle « seul dossier visible » dans `src/canvas/navigation.ts` (pur, 7 tests), branché dans `checkAutoNavigate`. Cause réelle : l'ancien code visait le dossier sous le *centre écran* alors que le zoom s'ancre sous le curseur → le voisin dérivait au centre. Désormais : on n'entre que si **un seul** dossier croise le viewport.
**Symptôme :** un dossier à gauche de l'écran, un autre à sa droite. Je vise le gauche,
je zoome vers lui — au passage on entre dans l'autre. Sur la frame d'avant, les **deux**
dossiers étaient visibles à l'écran.
**Cause racine (diagnostic user) :** le code ne tient pas compte de la **taille réelle**
des dossiers — un gros dossier déclenche au même zoom qu'un petit. Le critère actuel
(couverture ≥62 %) est donc faux.
**Règle voulue :** on entre dans un dossier **uniquement quand il est le seul à l'écran**
(aucun autre dossier frère ne croise le viewport). Géométrique, pas un scale fixe.
**Concerne :** `checkAutoNavigate` dans `src/canvas/GlucoseCanvas.tsx`.

### NAV-2 — Pavé tactile : impossible de naviguer comme sur le web
**Statut :** ✅ réglé (2026-05-31) — `classifyWheel` (`src/canvas/navigation.ts`, 7 tests).
**Pan 2 doigts (G/D/H/B) : OK, confirmé.** **Zoom = Ctrl + glisse 2 doigts** (et molette
souris), confirmé.
**Limite matérielle constatée :** sur ce pavé tactile (WebView2/Windows), le **pincement
n'émet AUCUN event** wheel (intercepté par l'OS, jamais transmis à la WebView) → techniquement
irrécupérable. Le geste zoom officiel est donc **Ctrl+glisse**, fiable partout (modèle Figma).
**Idée future :** rappel discret du raccourci à l'écran (non fait, pas demandé).
**Voulu (gestes standard) :** 2 doigts qui glissent → pan (droite/gauche/haut/bas) ;
pincement (doigts qui se rapprochent) → dézoom ; écartement → zoom.
**Impact :** frustrant en permanence pendant le test (je suis sur pavé tactile).

### VID-1 — Vidéos affichées comme image fixe
**Statut :** 🟡 implémenté, à confirmer à l'œil (2026-05-31) — cause : `autoPlay=false`
pour les vidéos `fit:"contain"` (folder) → poster figé (choix anti-lag d'origine).
Fix : vidéos chargées en boucle muette, lecture **pilotée par le culling** (`applyCulling`)
→ seules les vidéos **visibles** jouent, les autres en pause (règle VID-1 **et** anticipe
PERF-1 côté vidéo). `videoElsRef` (Map id→`<video>`), pause+cleanup au retrait du sprite.
**À valider** : une vidéo dans un dossier joue (en boucle), et défiler vite ne fait pas
ramer (les off-screen se mettent en pause).
**Symptôme :** une vidéo dans un dossier s'affiche figée — l'image ne bouge pas, ne se lit pas.
**Concerne :** texture vidéo dans `folderMirror.ts` / rendu sprite (`makeLinkedMedia`).

### UNDO-1 — Ctrl+Z / Ctrl+Maj+Z pas fiables
**Statut :** ✅ réglé+testé (2026-05-31) — 3 causes racines trouvées et corrigées,
verrouillées par **39 tests** (`src/store/undo-redo.test.ts`).
**Cause racine #1 (la pire) :** **la navigation polluait la pile d'undo.**
`setViewport` (pan/zoom/fit, en continu), `setActiveBoardId`, `enterFolder`/
`exitFolder`/`exitToRoot` et le scan paresseux `expandFolder` passaient tous par
`mutate` → chaque geste empilait une entrée d'undo **et vidait le redo**. Donc
Ctrl+Z annulait un mouvement de caméra au lieu de l'action, et le moindre pan
détruisait le redo. → Nouveau `mutateView` (applique le change Automerge SANS
toucher `_undoStack`/`_redoStack`) ; toute la navigation y est routée.
**Cause racine #2 :** **un drag = 1 entrée par frame.** `moveSelected` /
`updateAnnotation` / `updateFolder` étaient appelés à chaque `pointermove`, donc
glisser un objet créait des dizaines d'entrées (Ctrl+Z ne reculait que d'1 px). →
Transaction d'interaction `beginLiveEdit()` / `endLiveEdit()` : **1 seul snapshot**
au 1ᵉʳ mouvement réel, mutations live agrégées ensuite. Branché sur les 5 chemins de
drag/resize (sprites Pixi, texte/sticky/membrane, flèches, dossiers ; les zones
commitaient déjà au pointerup).
**Cause racine #3 :** **un simple clic créait une entrée no-op + vidait le redo**
(`pushHistory()` au pointerdown). → Supprimé ; la transaction s'ouvre paresseusement
au 1ᵉʳ vrai mouvement, donc un clic n'empile rien.
**Cause racine #4 (2ᵉ retour user, 2026-05-31) :** **éditer du texte = 1 entrée par
frappe/auto-fit.** Taper dans un bloc texte (ou nommer le bloc cible d'une flèche
tracée dans le vide) générait une entrée d'undo par redimensionnement auto + une au
commit → il fallait ~15 Ctrl+Z, et le texte « redevenait vide » avant de disparaître.
→ La **session d'édition entière** (ouverture overlay → frappe → auto-fit → commit)
est enveloppée dans UNE transaction `beginLiveEdit`/`endLiveEdit`, refermée à la
fermeture de l'overlay. Création-dans-le-vide d'une flèche : la transaction du tracé
reste ouverte jusqu'à la fin de la frappe → flèche+bloc effacés en 1 Ctrl+Z. 4 tests
ajoutés. **Membrane :** confirmé OK par l'utilisateur après rechargement propre
(le « bug » venait d'un état non rechargé) ; create/delete/undo verrouillés par 2 tests.
**Cause racine #5 (3ᵉ retour user) — LE vrai coupable du texte :** le `ResizeObserver`
de `HtmlAnnotationLayer` réécrivait la taille mesurée (`offsetWidth/Height`) via
`updateAnnotation` → chaque reflow du markdown empilait un Ctrl+Z **fantôme APRÈS** la
fermeture de la transaction (async, quand le bloc se rend post-commit). D'où « l'undo
de texte ne fait rien » (le 1ᵉʳ Ctrl+Z annulait un ajustement de taille invisible).
→ Nouvelle action `syncAnnotationSize` (via `mutateView`, **non annulable** : la taille
auto-fit est dérivée du rendu, pas une édition). 2 tests de régression. La transaction
d'édition (#4) + cette réconciliation hors-undo (#5) = ensemble nécessaire.
**Bonus fiabilité :** undo()/redo() renvoient un booléen → le toast « Annulé » ne
s'affiche que si une vraie action a eu lieu (plus de faux feedback) ; et undo/redo
**ne téléportent plus la caméra** ni ne te sortent du dossier courant (`preserveView`
+ `buildFolderStack`).
**Concerne :** `src/store/index.ts` (mutate/mutateView/beginLiveEdit), `GlucoseCanvas.tsx`,
`HtmlAnnotationLayer.tsx`, `SvgAnnotationLayer.tsx`, `FolderSvgLayer.tsx`, `App.tsx`.

### PERF-1 — Lag du rendu couleur/glow/fumée pendant zoom/dézoom
**Statut :** 🔴 à faire
**Symptôme :** tout ce qui émet de la couleur (fumées, glow, dégradés, flèches) fait
saccader le zoom/dézoom. Or c'est **le cœur esthétique** de Glucose — il faut le garder
ET le rendre ultra-fluide.
**Portée V1 :** fluidité dans les cas **normaux**. (La masse extrême → voir « Plus tard ».)
**Concerne :** `MembraneRenderer.ts`, `ZoneRenderer.ts`, couches de glow.

---

## 🟠 Visible / important (V1)

### TXT-1 — Fichiers texte : vrai texte lisible, packé sans superposition
**Statut :** 🟡 implémenté, à confirmer à l'œil (2026-05-31) — tailles **variables**
estimées du contenu (`estimateTextTileSize` dans `folderMirror.ts`, bornées 200–360×120–300)
+ **flow-pack** dans la zone basse (gauche→droite, retour à la ligne). Rendu = **vrai
markdown + LaTeX** (react-markdown/KaTeX) clippé à la tuile (`clipTextForTile`) → le
ResizeObserver relit la taille fixe (pas de croissance/chevauchement). 2 tests sizing.
**⚠️ Renverse le clamp 210×180 précédent + réutilise le pipeline markdown pour les tuiles
folder** (cf. mémoire `annotation-layer-no-culling`, maj) — coût borné par le clip ; masse
extrême = PERF-1. **À valider** : textes lisibles, pas de superposition, .md avec LaTeX.
**Demande réelle (mal comprise avant) :** afficher le texte **comme du vrai texte**
(rendu LaTeX/markdown, taille extensible), PAS de petits rectangles fixes.
1. Utiliser la **taille la plus optimale/propre** pour visualiser chaque texte.
2. **Packer tous les textes entre eux** de gauche→droite ; si trop large, **retour à la
   ligne en dessous** et on repart de gauche→droite (flow layout).
3. La superposition d'avant venait des tailles extensibles + collider box complexe →
   la vraie solution est un **algo de placement (packing)**, pas le clamp fixe.
**Tension à gérer :** rendu joli (markdown/LaTeX) **vs** fluidité (le pipeline markdown
est coûteux — cf. mémoire `annotation-layer-no-culling`).
**Concerne :** `folderMirror.ts` (placement) + `HtmlAnnotationLayer.tsx` (rendu).

### TXT-2 — Coloration syntaxique pour les scripts (.py & co)
**Statut :** 🟡 implémenté, à confirmer à l'œil (2026-05-31) — coloriseur **léger
sans dépendance** (`highlightCode` dans `HtmlAnnotationLayer.tsx`) : mots-clés (bleu),
chaînes (orange), nombres (vert clair), commentaires (vert), façon VSCode Dark.
Commentaire de ligne choisi selon le langage (`#` py/ruby/sh, `//` C-like, `--` sql/lua).
Branché via les composants `pre`/`code` de react-markdown → s'applique aux tuiles folder
ET aux annotations texte. 7 tests. **À valider** : un `.py` est coloré comme dans VSCode.

### LAYOUT-1 — Auto-rangement spatial à l'import d'un dossier
**Statut :** 🟡 implémenté, à confirmer à l'œil (2026-05-31) — disposition en **croix**
dans `folderMirror.ts` (`buildLevelNode`) : apps **gauche**, sous-dossiers **centre**,
images/vidéos **droite**, textes **bas**. Chaque catégorie a sa zone (cellules CELL=220,
gap 160), centrée sur (0,0). 4 tests croix. Bonus : la séparation spatiale règle aussi
l'ancien chevauchement médias↔icônes (z-order).
**⚠️ S'applique aux NOUVEAUX imports** (calculé au scan). Un dossier déjà importé garde
son ancienne disposition tant qu'on ne le ré-importe pas.
**À valider** : importer un dossier mixte → voir la croix.

---

## 🧰 Infrastructure de test

### TEST-1 — Scripts de tests « immenses » pour les features critiques
**Statut :** ✅ fait pour undo/redo + navigation (2026-05-31).
**Pourquoi :** impossible d'inventorier à la main tout ce qui marche/casse. Il faut une
batterie de tests qui re-vérifie **à chaque fois** au minimum : **undo/redo** (UNDO-1) et
**zoom/dézoom + entrée/sortie de dossier** (NAV-1). Ce sont les features non-régressables.
**Couverture actuelle :** `src/store/undo-redo.test.ts` (39 tests) — navigation transparente
(A), chaque mutation de contenu annulée+refaite (B), caméra/dossier préservés + feedback
honnête (C), drag groupé en 1 entrée (D). `navigation.test.ts` (14) couvre NAV-1/NAV-2.
**Total suite : 298 tests verts.**

---

## 🕓 Plus tard (explicitement reporté — PAS pour la V1)

- **Tri par date de dernière modification.**
- **Renommage** de fichiers.
- **« Ouvrir avec »** / lancement d'application configurable (clic droit → exécuter avec
  python, etc.). *(NB : le double-clic→lancer actuel reste, c'est le système configurable
  qui est reporté.)*
- **Optimisation extrême :** milliers de fichiers dans un dossier, masse d'images, masse
  de vidéos simultanées. (Le plafond perf est documenté : `annotation-layer-no-culling`.)

### 🌌 Vision long terme (cap, pas une spec)
Reprendre **tout** le fonctionnement de Windows et l'adapter dans une **grille 2D spatiale**
plus pratique, lisible et rapide que Windows lui-même.
