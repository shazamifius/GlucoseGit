# La suite, dans l'ordre

> **Au 15/10/2026** (fiche 71). La feuille de route de Glucose Rust. Chaque chantier dit **ce qui
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
| [~] | **La 2.0.3-beta.3** — P0 (la mémoire du téléphone), le Storyboard complet, l'export SVG, le dossier, Mood board. La CI groupée, le brouillon (Actions → Publier, brouillon coché) ; **son essai de l'APK au téléphone**, puis **son clic** « Publish release » | publiée par lui |
| [ ] | **P0 — Glucose ne doit jamais figer un téléphone** : fait côté code (fiche 66 : MEMOIRE-1 à 3). Reste **son téléphone** : l'APK installé (sans câble : le zip par Drive, fiche 71 § 3), une longue session, ses lignes `memoire` et `alerte_memoire` lues dans la télémétrie — et la session du gel, qui partira à la première ouverture | une longue session sans saccade qui monte ni gel ; ses lignes lues |
| [ ] | **P4 — La fondation des presets** (sa parole du 15/10, [VISION](VISION.md)) : **l'interface devient une donnée** — une table des commandes (chaque action nommée, une fois), une table des raccourcis qui les désigne, le thème, la disposition des panneaux ; puis la **palette de commandes** (`Ctrl+Maj+P`, PureRef), qui en sort d'elle-même. Les presets sont des **couches** qu'on mélange (les raccourcis de Figma avec l'allure de Glucose) — sa réponse du 15/10. Rien ne change à l'écran tant qu'il n'y a qu'un preset. **La grande session suivante** | chaque raccourci et chaque commande lus dans leur table ; une épreuve qui change un raccourci par la table ; saboté |
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

**Au téléphone** — l'APK de la beta 3 :

* l'ouvrir une première fois (la session du gel part), puis une **longue session** ;
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

Le co-working (sur le journal du document) · le MCP réécrit en Rust · les plugins selon sa vision
([PLUGINS.md](../PLUGINS.md)) et l'IA locale · Glucose en briques réutilisables (des milliers de
forks, [VISION](VISION.md)) · la fondation des 10⁷ nœuds (l'arène, écrite, en attente) · le modèle
de coût qui **prédit** ce qu'une image va coûter · et le but : le canevas de Wikipédia de
`glucose-brain`, puis les graphes d'humain à IA et d'IA à IA.
