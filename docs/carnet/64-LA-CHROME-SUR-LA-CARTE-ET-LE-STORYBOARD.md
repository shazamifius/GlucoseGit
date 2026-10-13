# 64 — La chrome sur la carte, et le Storyboard

> Session du 13/10/2026. Il a dit, en ouvrant : *« le zoom et le flou, j'ai l'impression de ne
> plus du tout l'avoir, c'est assez beau, c'est très clean »* et, au téléphone, *« le système
> d'export a l'air de parfaitement fonctionner, sans aucun problème »*. Puis, en fin de session :
> la session devient trop longue — tout enregistrer au propre, et une passation.

## 1. Ce qui est jugé, ce qui est vérifié

* **NETTETE-1 — jugé à son écran** : « plus du tout de flou, très clean ».
* **PARTAGE-2 — jugé à son téléphone** : l'export par la feuille de partage marche.
* **La CI sur `7c60593`** (exécution 38306469681) : **les dix tâches vertes**, Linux et les deux
  Mac compris — rien ne les avait vérifiés depuis le 09/10.
* **La 2.0.3-beta.1 en brouillon** (Actions → Publier, exécution 38361915501, sur `7c60593`) :
  les quatre tâches vertes, onze fichiers. **Elle ne porte ni CHROME-1 ni STORYBOARD-1**, qui
  viennent après. Reste son essai, puis son clic « Publish release ».
* **Les postes du téléphone (POSTES-1) ne sont pas encore arrivés** : une session ne part qu'au
  lancement **suivant**. La session reçue le 13/10 avait commencé le 12/10 à 9 h 55, sous
  l'ancien APK (aucune ligne `poste`) ; celle de son essai d'export, sous l'APK neuf, partira
  à sa prochaine ouverture de Glucose au téléphone.

## 2. CHROME-1 — la carte pose la chrome (`6e53d09`)

**La mesure** : la couche du dessus — que le processeur efface, relève et envoie à la carte à
chaque image — portait toute la chrome : 297 lignes au PC, 556 au téléphone rail replié, 1 865
rail déplié (2,4 à 8 Mo par image). Sur son PC, effacer + relever + envoyer faisaient la moitié
du travail d'une image de déplacement (sa chronique : 0,22 + 0,86 + 0,43 ms sur ~2,5).

**La recherche** : Chromium sépare l'*invalidation* (à rastériser de nouveau) du *dommage* (à
recomposer) — déplacer une couche ou changer son opacité ne rastérise rien
([`cc/README.md`](https://chromium.googlesource.com/chromium/src.git/+/HEAD/cc/)) ; Android
rejoue la liste d'affichage d'un `RenderNode` inchangé
([RenderNode](https://developer.android.com/reference/android/graphics/RenderNode)) ; Flutter
isole ce qui ne bouge pas derrière un `RepaintBoundary`
([isRepaintBoundary](https://api.flutter.dev/flutter/rendering/RenderObject/isRepaintBoundary.html)) ;
Blender marque à redessiner région par région, et a séparé ses gizmos de la vue
([commit](https://lists.blender.org/pipermail/bf-blender-cvs/2020-February/135701.html)).

**Ce qui est fait** :

* `ui/chrome.rs` : **l'ordre de la chrome écrit une fois** (`parcourir`) — la bande, le fil
  d'Ariane, le fond de la minimap, le rectangle de la vue, le rail —, avec ses marques de mesure ;
  la voie processeur compose chaque morceau, la voie graphique les reçoit (`ChromeConfiee`) en
  textures tenues par leurs caches, **lues sans copie** (`Rc`), refaites à leur changement
  seulement. Le fil d'Ariane a son cache.
* **Le rectangle de la vue est un nombre** : quatre filets et un voile, posés par la carte comme
  les zones. Et il est **fidèle à Tauri** (`Minimap.tsx`) : un trait d'**un** point (Rust en
  traçait 1,5) et un voile blanc à 0,04 (perdu ; la fiche 06 § 9 le notait « à vérifier »).
* **Ce qui attend une décision** passe toujours dans son tampon, au-dessus de la chrome — et ne se
  peint ni ne se relève sans rien à décider (`UiState::attend_une_decision`).
* `present/passes.rs` : **les passes de la carte en un seul endroit**, pour la présentation et le
  banc des épreuves. Le banc **recopiait** la préparation : une passe oubliée dans la vraie
  présentation ne faisait tomber aucune épreuve.

**Mesuré** (`examples/bench_chrome_carte`, l'ancienne loi rejouée sur le même document) : la
couche du dessus passe de 297 / 556 / 1 865 lignes à **0** sans rien de choisi — 2,57 / 2,40 /
8,06 Mo de moins par image. Les images témoins changent dans la minimap seule (8 838 pixels),
regardées avant/après. **21 sabotages, 21 chutes** (`outils/sabotages/chrome-1.json`).

**Ce qui reste — DESSUS-1** : la couche du dessus est toujours **balayée en entier** à chaque
image pour savoir ce qu'elle porte : 0,7 à 0,8 ms sur 2560 × 1600, même vide (`bench_dessus`).
La supprimer demande que chaque passe dise ce qu'elle dessine, vérifié contre le balayage dans
chaque épreuve (`debug_assert`). Les postes du téléphone diront si c'est le prochain coupable.

## 3. STORYBOARD-1 — le panneau entier (écrit, éprouvé, envoyé sans la CI — **non saboté**)

**Tauri lu champ par champ** (`StoryboardControls.tsx`, `StoryboardLayer.ts`,
`GlucoseCanvas.tsx` l. 1418-1530, `store/index.ts` l. 1117-1160) ; **la recherche** : Boords
renumérote seul et met les champs sous la case
([docs](https://boords.com/docs/creating-storyboards)) ; Storyboarder range dialogue, action et
notes à côté ([Wonder Unit](https://www.wonderunit.com/storyboarder)) ; Milanote fait glisser
les images dans des colonnes ([guide](https://milanote.com/guide/motion-design-storyboard)) ;
le rattachement automatique des sections de Figma est l'une des plaintes de son forum
([forum](https://forum.figma.com/suggest-a-feature-11/disable-auto-nesting-auto-reparenting-while-moving-objects-23638/index2.html)).

**Ce qui est fait** :

* **Le noyau** (`core/store/storyboard.rs`) : la grille centrée sur le clic (`Grille::coins`,
  `Store::coins_a_poser` — une définition pour le fantôme et le clic) ; poser, ou « Mettre à
  jour » (les mêmes cases rangées à la nouvelle grille) ; ajouter à la suite ; retirer (la suite
  reste continue) ; effacer ; renuméroter dans l'**ordre de lecture** (Tauri numérotait dans
  l'ordre de création, que rien ne montre) ; **une image lâchée dans une case s'y ajuste**
  (contenue, centrée) ; **glisser une case emporte ses images en deux éditions**, quelle que
  soit la durée du geste — la case réécrite à sa place dans le geste, ses images par le glisser
  généralisé (`Store::glisser_ces_images`, GLISSER-1). Chaque geste est un geste du journal :
  aucun changement du format de fichier.
* **Le dessin** (`renderer/storyboard.rs`) : les couleurs de Tauri ; les rectangles au premier
  rang avec les zones (`premier_rang`, les deux voies) ; numéro et description dans la couche du
  dessous ; le contour et la croix de la case choisie dans celle du dessus.
* **Les gestes** (`interactions/storyboard.rs`) : la grille armée suit le curseur (un fantôme
  par case), un clic la pose, Échap y renonce ; une case se prend **par sa bande** (son cadre
  reste au canevas) et se glisse avec ses images ; deux clics sur la bande la décrivent (la
  question à champ, que le clavier du téléphone remplit) ; sa croix ou Suppr la retire.
* **Le panneau** (`dock/storyboard.rs`) : celui de Tauri — format, largeur, colonnes,
  espacement, l'aperçu, « Activer » / « Mettre à jour », « × », « PANELS (n) », « + Ajouter »,
  chaque case (numéro, description, ×), « Re-numéroter les panels » ; la liste **par pages**
  (‹ ›) au lieu de son défilement. Armer ferme le panneau, comme les Presets.
* **Une différence voulue** : le numéro passe du coin du cadre à la **bande**, devant la
  description — une image ajustée couvre son cadre (dans Tauri, une case n'en recevait jamais).

**Éprouvé** : 9 épreuves du noyau, 4 du panneau, 7 des gestes (dont une par la vraie souris, de
« Activer » au `Ctrl+Z`), 3 du dessin, la comparaison stricte des deux voies sur les cases. La
suite entière : voir § 6. **Regardé** : `examples/apercu_storyboard` (cases décrites, deux
images ajustées, une case choisie, le panneau ; la grille armée). **Pas encore saboté** : les 25
sabotages sont écrits (`outils/sabotages/storyboard-1.json`) — la première chose à faire.

## 4. Les erreurs de la session — ce que j'ai cru et que la vérification a démenti

* **« Le rail couvre la minimap sur son téléphone »** : faux. Debout (1080 × 2400), le rail
  déplié s'arrête à la ligne 1 546, la minimap commence à 2 037. L'épreuve de l'ordre prend un
  écran de 1 800 lignes, où il la couvre — et vérifie elle-même ce recouvrement.
* **« Une image prise par la tolérance de la main autour d'elle »** : faux. Les images ne se
  prennent que dans leur boîte ; c'était la **poignée de coin** de l'image choisie (`add_image`
  la choisit), qui mord sur la bande — et c'est la règle voulue. Trouvé en affichant l'état
  après l'appui, pas en devinant.
* **Une épreuve qui dépend du temps** : un message neuf naît transparent (fondu d'entrée) ; sous
  la charge de la suite entière, un message de 2,4 s s'effaçait déjà. Épreuve rendue
  indépendante (une vie d'une heure).
* **Un seuil relatif à ce qui change** : l'épreuve de la carte qu'on écrit admettait « 5 % de la
  couche du dessus » ; la couche a perdu la chrome (205 918 → 38 250 pixels d'encre) et le
  curseur dépassait. La borne est désormais la surface de la carte elle-même.
* **Un cliquet trompé par un nom** : le champ `images` de ma propre structure comptait comme un
  accès au modèle — renommé `portees`.
* **Git Bash et les apostrophes** (encore) ; et un script de remplacement écrit avant
  `cargo fmt`, qui ne trouvait plus son texte — rien n'avait été écrit, le script s'arrête avant.

## 5. Ce qui attend sa parole

1. **Trans-domaines** dit-il « bientôt », comme Collaborer ? (fiche 63 § 9)
2. **`Ctrl+F`** : la recherche, quand elle viendra, ou tout cadrer ? (fiche 63 § 9)
3. **L'export PNG** : à 100 %, ou à la pleine résolution des originaux ? (fiche 62)
4. **Le numéro d'une case** dans sa bande (mon choix) plutôt que dans le coin du cadre (Tauri).
5. **La 2.0.3-beta.1** : son essai du brouillon, puis son clic — ou attendre une beta qui porte
   CHROME-1 et STORYBOARD-1 (il faudrait alors refaire le brouillon après la CI).

## 6. Où vivent les données de la boîte noire qui voyage

Dans **son compte Cloudflare** (celui de `shazamifius@gmail.com`) : une base **D1** nommée
`glucose-boite-noire` (Europe de l'Ouest), remplie par le Worker
`glucose-boite-noire.ferme-nilslamber.workers.dev`. Une ligne par session : l'installation (un
identifiant tiré au hasard), la session, le jour, la version, le système, l'architecture, et
les lignes du journal technique — **aucun contenu de ses documents**, seulement des nombres et des
mots fixés à la compilation (`docs/TELEMETRIE.md`).

* **La voir** : tableau de bord Cloudflare → *Storage & Databases* → *D1* →
  `glucose-boite-noire` → *Console* (on y tape du SQL), ou depuis `outils/telemetrie/` :
  `npx wrangler d1 execute glucose-boite-noire --remote --command "SELECT …"`.
* **Au 13/10** : quatre installations — son PC (`640513b7`, 22 sessions), son téléphone
  (`1c287c59`, 15), **un autre utilisateur sous Windows** (`5280efb2`, 9), une machine Linux
  (`13cff848`, 1 session le 09/10 — probablement son NixOS).
* **Les cartes graphiques** y sont par leurs numéros (lignes `carte` : fabricant, modèle — CARTE-1) :
  les traduire en noms demande une table PCI ; un tableau de bord est une piste pour plus tard.

## 7. À essayer à l'écran

`cargo run --release > sortie-64.txt 2>&1` :

* **La chrome** : déplacer et zoomer — rien ne doit avoir changé à l'œil ; la minimap a un trait
  plus fin et un léger voile dans le rectangle de la vue, comme Tauri.
* **Le Storyboard** : bouton Storyboard → « Activer » → la grille suit la souris → un clic la
  pose ; glisser une image dans une case ; prendre une case par sa bande et la glisser (l'image
  suit) ; deux clics sur la bande pour écrire la description ; la croix ; `Ctrl+Z` ; le panneau
  (+ Ajouter, ×, Re-numéroter, les pages avec beaucoup de cases).
* **Au téléphone** : ouvrir Glucose une fois — pour que la session d'hier parte, avec ses postes.
