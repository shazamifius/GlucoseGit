# La suite, dans l'ordre

> **Au 15/10/2026, le soir** (fiche 72). La feuille de route de Glucose Rust. Chaque chantier dit **ce qui
> le déclare fini**. Tout y est à remettre en question — ses paroles comprises. La direction
> vit dans [VISION](VISION.md), l'état vérifié dans [ÉTAT](ETAT.md), chaque fonction et son
> bouton dans [INVENTAIRE](INVENTAIRE.md), la cible PureRef dans [PUREREF](PUREREF.md).
>
> **Comment s'en servir** : prendre le premier chantier non fait. Fini = **prouvé** (sur une
> copie, sur GitHub, ou à son écran), jamais « ça compile ». Le cocher ici, mettre ÉTAT à jour,
> passer au suivant. L'histoire des tableaux d'avant (§ 0 à 0 octies, du 07 au 14/10) vit dans
> les fiches 51 à 70 ; ce qui en restait ouvert est repris ci-dessous.

`[x]` fini et prouvé · `[~]` écrit, éprouvé, saboté — **attend son écran** · `[ ]` à faire

---

## 1. Le programme

| | chantier | ce qui le déclare fini |
|---|---|---|
| [x] | **P5 — L'export SVG complet** (fiches 69 à 71) : tout ce que le canevas montre, dans son ordre, le texte en tracés ; les flous bornés à leur portée (28 s → 62 ms) ; le rectangle net par la règle du canevas ; 30 sabotages, 30 chutes ; regardé sur une copie de son document « UwU » (superposé au PNG) | reste **ses yeux** : un SVG ouvert dans son navigateur et dans Inkscape (§ 2) |
| [x] | **La 2.0.3-beta.3** — publiée par lui le 15/10 | publiée par lui |
| [~] | **La 2.0.3-beta.4** (fiche 72) — la télémétrie qui ne perd plus les longues sessions, la raison de mort d'Android (SORTIE-1), ce que la reprise rouvre et ce qu'elle saute (REPRISE-1), **LIBERE-1** (un brouillon tenu au lancement revient), VIE-2, le texte net de près (GRAND-1), le zoom ×256, **la palette qui défile** et les raccourcis comme donnée (P4). Le brouillon, refait sur le dernier commit après sa CI ; l'APK zippé sur son Bureau. Reste **son essai au téléphone** (le 16/10), puis **son clic** « Publish release » | publiée par lui |
| [ ] | **P0 — Glucose ne doit jamais figer ni perdre un tableau au téléphone** : fait côté code (fiche 66 : MEMOIRE-1 à 3). **Le 15/10, sur deux téléphones, un tableau de plus de 6 000 images s'est fermé seul et n'est pas revenu** (fiche 72 § 4) : la cause n'est pas connue. La beta 4 la dira : ses lignes `sortie` (la raison d'Android, la mémoire à la mort) et `reprise` (ce que le lancement a rouvert), et les longues sessions qui partent enfin. **Ne pas désinstaller** : le brouillon est peut-être encore là | la cause nommée par ses lignes ; puis corrigée, éprouvée sur une copie |
| [~] | **P4 — La fondation des presets** (fiche 72) : **fait** — le noyau des raccourcis (`glucose-core::raccourcis` : l'accord et sa loi ACCORD-1, le clavier français ; la couche en texte ; la pile qui remplace ou retire), **la table des 75 commandes** (`commandes.rs`), les raccourcis de Glucose en couche (`assets/raccourcis/glucose.txt`), **le menu qui exécute les commandes et dit leurs raccourcis lus dans le clavier**, **la palette** (`Ctrl+Maj+P`) ; un raccourci changé par la table, éprouvé ; rien n'a changé à l'écran (le témoin du menu). **Reste** : la couche de l'utilisateur, lue sur le disque et modifiable ; le thème et la disposition des panneaux comme données ; puis les presets PureRef, Figma, Miro comme couches | à son écran : `Ctrl+Maj+P` ; puis la couche de l'utilisateur |
| [ ] | **P8 — Le texte par la carte graphique** (fiche 72 § 5) : de près, le texte se rastérise au processeur et la cascade montre un repli flou le temps de refaire les carrés. **Slug** (Eric Lengyel), dans le domaine public depuis le 17/03/2026, nuanceurs MIT : les lettres rendues par la carte depuis leurs courbes, nettes à tout zoom sans re-rastériser. Les contours sont déjà lus (GRAND-1) ; les fontes CFF (Inter Italic) demandent les cubiques en quadratiques ; le texte riche, les formules et les cadres à porter en primitives | de près, aucun flou, sur sa machine et au téléphone |
| [ ] | **Ordonner, très très grandement** (sa proposition du 15/10) : à concevoir avec lui — ce qui manque (PureRef : par nom, par date, en ligne, en colonne — dans le noyau sans bouton ; aligner, distribuer, empiler, normaliser), et ce qu'il entend par « grandement » | sa parole, puis à son écran |
| [ ] | **P3 — Ce que son téléphone dira** : POSTES-1 (les lignes `poste`), puis DESSUS-1 et RASTER-1 si elles les désignent. Son téléphone tournait à **19,5 ms** par image en médiane — sous le plancher presque tout le temps | les postes lus, le coupable nommé par la mesure |
| [ ] | **Les choix à l'utilisateur** (sa parole : « TOUJOURS CHOISIR ») : la planche claire ou sombre, les champs (dialogue, action, notes) sous les cases, « 2.3 », le numéro dans la bande ou le coin, l'export PNG à 100 % ou aux originaux, les photos du SVG en originaux ou réduites | chaque choix dans la question ou le panneau où l'on agit, enregistré, saboté |
| [ ] | **P6 — Relire chaque panneau et chaque menu contre Tauri, champ par champ** : Pomodoro, Domaines, Time Machine, les menus du clic droit, la bande ; la carte d'accueil qui naît choisie au premier lancement | la liste de ce qui manque, puis fait |
| [ ] | **P7 — Les fonctions qui manquent** ([INVENTAIRE](INVENTAIRE.md) § 3, [PUREREF](PUREREF.md)) : **la recherche** (`Ctrl+F`) d'abord ; les opérateurs des post-it ; la couleur (un sélecteur), la couleur des domaines, le filtre par domaine ; les miroirs ; la temporalité ; l'export HTML ; le mode zen ; les vidéos et les GIF animés ; la provenance (sa clé SauceNAO) ; de PureRef : la hiérarchie, le miroir, l'opacité, les niveaux de gris, la pipette, le dessin, les modes de fenêtre | chacun à son écran |
| [ ] | **Le dossier, à retravailler** (sa parole du 15/10) ; « double-clic + zoom comme Tauri » à vérifier au PC et au téléphone | à son écran |
| [ ] | **GLISSER-2** — défaire un glisser exactement (une édition neuve du journal, un format neuf) ; **les glyphes des formules calés au pixel entier** au canevas (le texte l'est au quart de pixel) | avant 10⁷ nœuds ; regardé |
| [ ] | **Le repos à zéro** : sa chronique redessine pour les messages (1,6 image par seconde) — la charte dit zéro | une chronique au repos sans image |
| [ ] | **La fluidité** (la charte : 100 images par seconde, toujours) : ses zooms chargés à 11-24 ms (sa chronique du 14/10 : 345 images sur 13 886 au-dessus de 10 ms, un tressaut à 91 px au p99 quand la durée d'une image change) ; **sur batterie**, 30 images par seconde par moments (le bridage de la RTX, à séparer par une session branchée et une débranchée) ; les gels de l'Intel Arc (probablement l'écran sur la RTX, à confirmer en mode Optimus) ; la coupure du pavé de 7 à 10 s (la boîte noire dira où à la prochaine) ; le plantage reproductible d'une épreuve dans `vulkan-1.dll` (fiche 49 § 4) | chaque point mesuré, puis tenu ou pixelisé — jamais une dégradation qui se voit |
| [ ] | **Android, ce qui manque** : le presse-papiers du système, les photos HEIC, les champs du dock qui se tapent (le miroir du clavier étendu), les panneaux qui s'écartent des bords ; **les rayures** de sa PowerVR (« je surveillerai ») ; la mise à jour automatique d'Android, à prouver quand une version suivante arrivera à un téléphone qui a la beta 3. Hors Android : les doigts sous Windows (tablettes), l'iPad par le web, le dépôt d'une page compressée hors Windows (`ureq` sans `gzip`) | chacun à son écran |
| [ ] | **P9 — Le Mac publié** (un `.dmg`, son copier, sa mise à jour) ; **Collaborer** (obligatoire, plus tard) ; **l'ergonomie, puis l'UX du téléphone**, toutes refaites quand toutes les fonctionnalités seront là | |

## 2. Ce qui attend son écran — une seule liste

Tout est écrit, éprouvé et saboté ; **rien de ceci n'a été jugé par lui**. Sa parole du 15/10 :
*« si j'ai absolument tout testé, normalement 100 % des fonctionnalités sont validées »*.

**Au PC** — `cargo run --release > sortie-essais.txt 2>&1` :

* **Le texte de près** (fiche 72) : zoomer très près d'un texte, d'un titre de membrane, d'une
  formule — net bien plus vite ; et **le zoom jusqu'à ×256**.
* **La palette** : `Ctrl+Maj+P`, écrire « dupl », Entrée ; les flèches pour choisir ; Échap ;
  sans rien écrire, faire défiler toute la liste (molette, pavé, flèches, Page bas).
* **Le menu du clic droit** : ses raccourcis viennent désormais du clavier — rien ne doit avoir
  changé à l'œil.

* **Le Storyboard** (fiches 64 à 68) : « Activer », la grille qui suit la souris par son coin,
  un clic ou un rectangle tracé ; la poignée qui le retrace ; une case lâchée ailleurs ; décrire
  (deux clics sur la bande), `Tab` vers la suivante ; plusieurs images lâchées sur une case ;
  « Storyboard de la sélection » ; les deux « + » ; « ▶ Lire » (← →, Espace, Échap) ; « CASE n » :
  durée, plan, champs ; « + Nouveau », ‹ 2 / 3 › ; « Exporter en planches » en PDF et en PNG.
* **L'export SVG** : l'ouvrir dans son navigateur, puis dans Inkscape s'il l'a.
* **Le dossier** : son titre et son compteur ; un dossier vide dit « Double-clic pour ouvrir ».
* **`Alt+F`** cadre tout, même un tableau qui ne porte qu'un storyboard.
* **Mood board** (l'ancien Presets) : poser un gabarit, ses zones (bandeau, coins, croix), « Par
  mood board » dans Ordonner.
* **Ordonner** : les trois tris par la couleur ; les champs (glisser, cliquer et taper).
* **L'export PNG** : une sélection, puis tout le tableau.
* **Le clic droit** : Annuler, Rétablir, Tout cadrer, Signets…, Enregistrer, Exporter ; le menu
  en colonnes sur un écran bas.
* **La membrane et le dossier tracés** d'un glisser ; « Décrire » écrit au clavier.
* **La chrome** : rien ne doit avoir changé à l'œil, sauf la minimap (un trait fin, un voile).
* **Trans-domaines** dit « bientôt ».
* De plus loin : transformer une sélection entière (origine commune ou individuelle) ; le cadre
  du groupe qui suit ; Pinterest (la copie d'abord, puis l'original) ; les sélecteurs de fichiers
  qui ne figent plus Glucose ; les flèches qui contournent et leur coude ; les formules seules
  sur leur ligne ; la Time Machine pendant un zoom ; `Ctrl+Z` après un glisser ; l'aimant sur un
  tableau chargé ; un PDF glissé d'une page web.

**Au téléphone** — l'APK de la beta 4 (zippé sur son Bureau) :

* l'installer **par-dessus** (ne pas désinstaller : un brouillon perdu est peut-être encore là),
  l'ouvrir une fois, puis une **longue session** ; si Glucose se ferme seul, le rouvrir deux
  fois — ses lignes partent au lancement suivant ;
* le clavier (écrire dans un texte), l'appui long (le clic droit), deux doigts touchés annulent,
  trois rétablissent ; renommer, dupliquer, supprimer un document (appui long dans la liste) ;
* le Storyboard au doigt ; un dossier choisi (son menu, sa barre) ; le partage vers Glucose
  depuis la galerie ou Pinterest ; les emojis ; les noms du rail ; le bord à bord.

**Sous Linux** (son NixOS, quand il est allumé) : le copier vers Discord, un bloc d'un Glucose
à l'autre, après fermeture (fiches 60, 61) ; et la liste de « tant de choses ne marchent pas ».

## 3. Ce qui n'attend que lui

* Son **essai de l'APK** et son **clic** « Publish release » de la beta 3.
* **Sa clé SauceNAO** (la provenance des images).
* **Le modèle de l'iPad**, les téléphones de ses testeurs Android (le plus ancien d'abord).
* **Les définitions** : Trans-domaines et « domaine » (en recherche), les rideaux, les membranes de
  Mary (étirées, minimisées), « optimiser » dans la Time Machine.
* **Le dessin de la sélection** : E pour l'instant, à reprendre avec l'ergonomie.
* **Le défilement au bord** (« moche, pas fluide ») : plus tard, à sa parole.
* **Les modes de la fenêtre** (opacité, clics qui traversent) : pas pour l'instant.

## 4. Plus loin

**L'améliorateur d'image à l'import** (sa parole du 15/10) : comprendre une image avant d'y
toucher (filtres de Gabor, GLCM, LBP — son cerveau d'image, `recherche/cerveau-image/`) pour
débruiter, déflouter, agrandir sans inventer · Le co-working (sur le journal du document) · le MCP réécrit en Rust · les plugins selon sa vision
([PLUGINS.md](../PLUGINS.md)) et l'IA locale · Glucose en briques réutilisables (des milliers de
forks, [VISION](VISION.md)) · la fondation des 10⁷ nœuds (l'arène, écrite, en attente) · le modèle
de coût qui **prédit** ce qu'une image va coûter · et le but : le canevas de Wikipédia de
`glucose-brain`, puis les graphes d'humain à IA et d'IA à IA.
