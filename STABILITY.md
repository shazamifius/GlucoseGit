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
**Statut :** 🔴 à faire
**Symptôme :** comportement incertain de l'annuler/refaire.
**Exigence :** non-négociable pour la V1. Doit être **couvert par des tests** solides
(voir TEST-1) pour qu'on sache à chaque commit s'il marche encore.
**Concerne :** undo/redo par snapshots de doc dans `src/store/index.ts`.

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
**Statut :** 🟠 à faire — ⚠️ **annule mon correctif précédent** (carte fixe 210×180)
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
**Statut :** 🟠 à faire
**Voulu :** colorer variables/mots-clés comme dans VSCode pour les `.py` (et autres scripts).

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
**Statut :** 🟠 à faire
**Pourquoi :** impossible d'inventorier à la main tout ce qui marche/casse. Il faut une
batterie de tests qui re-vérifie **à chaque fois** au minimum : **undo/redo** (UNDO-1) et
**zoom/dézoom + entrée/sortie de dossier** (NAV-1). Ce sont les features non-régressables.

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
