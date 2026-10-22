# La suite, mise à jour par mise à jour

> **Au 22/10/2026, le soir** (fiches 73 à 81 ; la 2.0.3-beta.5 publiée ; le domaine défini). **Le plan complet** de ce qui attend
> Glucose Rust, rangé en mises à jour : ce que chacune porte, et ce qui la déclare finie. Tout y
> est à remettre en question — ses paroles comprises. La direction vit dans [VISION](VISION.md),
> l'état vérifié dans [ÉTAT](ETAT.md), chaque fonction et son bouton dans
> [INVENTAIRE](INVENTAIRE.md), la cible PureRef dans [PUREREF](PUREREF.md).
>
> **Comment s'en servir** : prendre le premier chantier non fait de la première mise à jour non
> publiée. Fini = **prouvé** (sur une copie, sur GitHub, ou à son écran), jamais « ça compile ».
> Le cocher ici, mettre ÉTAT à jour, passer au suivant. Une mise à jour se publie quand ses
> chantiers sont prouvés — **le nom et le moment restent les siens**.

`[x]` fini et prouvé · `[~]` écrit, éprouvé, saboté — **attend son écran** · `[ ]` à faire

---

## 0. Pourquoi cet ordre

Quatre principes, tirés de la charte et de ce que les sessions ont appris — à contredire :

1. **Ce qu'il voit d'abord.** Une version publiée sans son essai est un pari : chaque mise à
   jour commence par juger la précédente à son écran, et par lire sa télémétrie.
2. **Ne jamais perdre un tableau** passe avant toute fonction neuve (P0) — sa panique devant une
   perte, et deux téléphones l'ont vécu.
3. **Le manque de fonctions bloque plus que la vitesse** (la charte : « le blocage actuel n'est
   pas la performance ») — mais **100 images par seconde, toujours**. Les mises à jour alternent
   donc : des fonctions, puis la fluidité de tout ce qui existe.
4. **Relire avant de bâtir** : chaque panneau contre Tauri (P6) *avant* que le thème et la
   disposition deviennent des données (les presets) — sinon on transcrit des panneaux qu'il
   faudra refaire.

**Les noms proposés** : les betas de la 2.0.3 continuent tant que la parité avec Glucose Tauri
n'est pas atteinte ; la **2.1.0** serait la première version que Glucose Rust appelle stable — le
jour où un utilisateur de Tauri n'y perd plus rien. À sa parole.

---

## 1. Le plan

### La 2.0.3-beta.5 — publiée le 16/10 (fiches 73, 74)

| | chantier | ce qui le déclare fini |
|---|---|---|
| [~] | **P8 — Le texte de près par la carte graphique** (fiche 73) : un nœud plus grand que l'écran se dessine depuis sa description vectorielle — la même que l'export SVG (`Vectoriel`) — : les lettres et les formes de KaTeX par **Slug** (le noyau `courbes` : CUBIQUE-1, BANDES-1, BANDES-2 ; RACINES-1), les fonds, plaques, filets et liserés par la loi des membranes ; le découpage en carrés retiré ; **CADRE-1** ; SLUG-1 et sa loi de place ; ~1 ms par image de près, contre ~20 et le flou ; CI verte sur les dix tâches | de près, aucun flou, sur sa machine et au téléphone |
| [~] | **P8 bis — Un nœud immense, de près** (fiche 73 § 7) : **LIGNE-1** et **la mémoire des mises en page** — 10,5 ms → **0,72 ms** à 17 600 mots ; un tableau de mille textes ne refait plus deux mille mises en page par image | un très long texte, de près ; un tableau plein de textes |
| [~] | **P4 — Tes raccourcis** (fiches 72, 74) : la table des 77 commandes, la palette, la couche de Glucose, **la couche de l'utilisateur** (« Changer un raccourci… », retirer, rétablir, `raccourcis.txt` jamais écrasé s'il porte une ligne fausse) | `Ctrl+Maj+P` ; changer un raccourci, relancer |

### La 2.0.3-beta.6 — « Ce que les écrans diront »

Petite et rapide : juger la beta 5, nommer les causes que la télémétrie apporte, réparer.

| | chantier | ce qui le déclare fini |
|---|---|---|
| [ ] | **Le domaine** (sa définition du 22/10, [VISION](VISION.md) — *« pour l'instant, concentrons-nous sur domaine »*) : un domaine **traverse** les blocs, les membranes et les fichiers (« l'eau, ça mouille », « eau = H₂O », « l'eau dans un jeu vidéo » : trois blocs loin les uns des autres, un seul domaine). Ce qui existe : le catalogue d'un document, l'appartenance pondérée de chaque nœud, le panneau, les badges. Ce qui manque : **une IA qui lit et pose les domaines** (`glucose_lire` ne les dit pas : contraire à « lisible pareil par l'humain et par l'IA ») ; **traverser les fichiers** (un catalogue qui ne soit plus celui d'un seul document). **La visualisation d'un domaine** (le voir, le filtrer, sa couleur) est de l'**ergonomie : pas maintenant** (sa parole du 22/10) | Protéo lit et pose les domaines ; un domaine relie des nœuds de deux fichiers |
| [ ] | **Ses essais de la beta 5** (§ 2) : chaque défaut qu'il voit, corrigé — en premier, le texte de près | sa parole |
| [ ] | **P0 — Ne jamais perdre un tableau au téléphone** : fait côté code (fiche 66 : MEMOIRE-1 à 3 ; fiche 72 : LIBERE-1, SORTIE-1, REPRISE-1). **Le 15/10, sur deux téléphones, un tableau de plus de 6 000 images s'est fermé seul et n'est pas revenu** : cause inconnue. Ses lignes `sortie` (la raison d'Android, la mémoire à la mort), `reprise` (ce que le lancement a rouvert, la place du disque) et `precedente` la diront — **au 17/10, toujours aucune session du téléphone depuis le 15/10** (89 sessions, toutes de PC ; il est encore en beta 3) : rouvrir Glucose une seconde fois, au PC et au téléphone. **Ne pas désinstaller** : le brouillon est peut-être encore là. Relevé : `node outils/telemetrie/tableau.mjs <dossier>` (7403 → `npx wrangler whoami`, puis une fois encore) | la cause nommée par ses lignes ; corrigée, éprouvée sur une copie |
| [ ] | **P3 — Ce que son téléphone dira** : son téléphone tournait à **19,5 ms** par image en médiane, sous le plancher presque tout le temps. POSTES-1 (les lignes `poste`), puis DESSUS-1 et RASTER-1 si elles les désignent ; et **P8 au téléphone**, jamais mesuré (Slug sur sa PowerVR) | les postes lus, le coupable nommé par la mesure, puis tenu |
| [x] | **Le repos à zéro** — mesuré le 18/10 : une séance immobile, un dossier de données à part — **0,2 image par seconde sans la main, toutes pour le fondu du message de bienvenue**, rien d'autre. Les « 1,6 image par seconde » du 09/10 étaient, selon toute vraisemblance, les fondus de messages que sa main venait de faire naître. Réserve : une séance, et le pavé touché en route | une chronique au repos sans image hors un message qui paraît |
| [~] | **P10 — La porte des agents** (fiche 75, sa demande du 17/10) : `glucose --mcp`, le protocole MCP des deux époques, huit outils ; chaque appel est un geste annulable (AGENT-1 à 4), rien sans son accord (AGENTS-1), la porte n'est qu'à lui (AGENTS-2) ; la page web en miniature, que le double-clic ouvre ; **côté Protéo** : la toile — Demande, Recherche en colonnes, Rapport relié phrase par phrase aux citations | Protéo travaille dans son Glucose, sous ses yeux |
| [~] | **L'agent et l'humain, dans les deux sens** (fiches 79, 80) : lire par niveaux ; `selection`, `empreinte`, `document` dans `glucose_lire` ; `glucose_ouvrir_tableau` ; côté Protéo, la relecture à chaque message, le carnet « Ce que Protéo a appris », le « je ne sais pas » — éprouvés, sabotés, vus en essai réel hors écran. **Reste** : les commentaires ancrés, la provenance par nœud (qui l'a écrit, depuis quelle preuve), les membranes repliables, le statut et la preuve écrits par le programme ; relever par le programme ce que l'usage de Protéo dit de Glucose | il sélectionne un nœud de Protéo et dit « ça » ; il corrige une leçon du carnet ; Protéo le voit |
| [~] | **La toile, rangée** (fiche 77 § 5) : **RANGEE-1** (une demande, une rangée : la Demande sous tout le tableau, ses recherches et rapports à sa droite) ; **SUITE-2** (une demande qui poursuit la précédente se relie à elle — le début des graphes de questions) | vu en essai réel, puis à son écran |
| [~] | **Le tri de la demande** (fiche 76, sa demande du 17/10) : réponse courte, outils, ou workflow et sa taille — deux questions vrai/faux au modèle, la règle de l'année, la route qui en suit (sans outil, la boucle, la mission ouverte par le programme) ; la toile par demande ; **FLECHE-6** (la flèche ancrée arrive à la hauteur de sa citation) | ses demandes, dans son Glucose : le tri juste à ses yeux |
| [~] | **Le cerveau de Protéo** (fiches 77, 78) : le banc rendu juste (SORTIE-1, FENETRE-1, ANNULE-1, `scripts` retirée, le verdict pesé par chemin, un cerveau = un modèle et sa réflexion) ; **qwen3.5:4b démontré meilleur que qwen2.5:7b (15/20 contre 3/20, p = 0,0005)**. **Reste** : finir les essais réels (le grand prix, le sodium-ion, la suite ; relire le script et les abeilles), la seconde répétition (la réflexion), **lui proposer la bascule**, remplacer `proteo.exe` | un modèle meilleur que qwen2.5:7b, mesuré, en service |
| [~] | **La machine qui choisit seule** (fiches 77, 78) : **écrits et sabotés** — la bibliothèque d'Ollama lue (256 modèles, 208 à examiner ; les badges ordonnent, n'écartent pas), la tête GGUF lue au registre avant tout téléchargement, la décision de la course contre le champion. **Reste** : les brancher (la vigie, l'inspection, le tirage par nom), la course elle-même ; borner par la place du disque | Protéo retrouve seul le modèle que la mesure désigne |
| [~] | **L'organe de recherche** (fiche 77 § 4) : **PROGRES-1** (un tour progresse quand il fait avancer le statut de sa sous-question : les 116 pages), **SUJET-1** (des mots-clés sans le sujet ne partent pas seuls), **DEPEND-1** (une dépendance qui échoue ne bloque plus) — éprouvés, sabotés. **Reste** : les pages dans une autre langue (remesurer avec le nouveau cerveau), une étape de recherche qui se coche sur un bilan | une recherche réelle, juste et bornée |
| [ ] | **Le flou** (sa parole du 17/10) : quand la recherche découvre qu'elle ne sait pas assez, Protéo organise de lui-même des recherches plus intensives, en parallèle, qui précisent la Demande | une recherche floue qui se relance seule, sur la toile |
| [ ] | **Les boutons qui se taisent** : Trans-domaines ne fait rien et ne le dit pas (il doit dire « bientôt », comme Collaborer) ; ~~la carte d'accueil qui naît **choisie**~~ — **fait** (ACCUEIL-1, fiche 81 : Tauri n'a pas de carte d'accueil, rien n'y est choisi ; Protéo la prenait pour un geste de l'utilisateur) | à son écran |

### La 2.0.4 — « Retrouver, ranger, choisir »

Les gestes de tous les jours qui manquent le plus, et la relecture qui prépare les presets.

| | chantier | ce qui le déclare fini |
|---|---|---|
| [ ] | **La recherche, `Ctrl+F`** (sa décision du 13/10 : tout cadrer est passé à `Alt+F`) : chercher dans tout le document — textes, titres, noms de fichiers —, voler d'un résultat au suivant ; sur le noyau, pour tenir 10⁷ nœuds | à son écran, sur un gros tableau |
| [ ] | **Ordonner, « très très grandement »** (sa proposition du 15/10) : **à concevoir avec lui**. Ce qui manque déjà (PureRef) : par nom, par date d'ajout (dans le noyau, sans bouton), en ligne, en colonne (dans le noyau, sans bouton), aligner avec un espacement, distribuer, empiler, normaliser la largeur, la taille, l'aire, l'échelle ; aléatoire ; et ce qu'il entend par « grandement » | sa parole, puis à son écran |
| [ ] | **Les choix à l'utilisateur** (sa parole : « TOUJOURS CHOISIR ») : la planche claire ou sombre, les champs (dialogue, action, notes) sous les cases, « 2.3 », le numéro dans la bande ou le coin, l'export PNG à 100 % ou aux originaux, les photos du SVG en originaux ou réduites | chaque choix là où l'on agit, enregistré, saboté |
| [ ] | **P6 — Relire chaque panneau et chaque menu contre Tauri, champ par champ** : Pomodoro, Domaines, Time Machine, les menus du clic droit, la bande, Ordonner, Mood board | la liste de ce qui manque, puis fait |
| [ ] | **Le dossier, à retravailler** (sa parole du 15/10) ; « double-clic + zoom comme Tauri », au PC et au téléphone | à son écran |

### La 2.0.5 — « Le contenu vivant » (la parité avec Glucose Tauri)

Ce que Tauri faisait et que Rust ne fait pas encore ([INVENTAIRE](INVENTAIRE.md) § 3).

| | chantier | ce qui le déclare fini |
|---|---|---|
| [ ] | **La couleur** : un sélecteur ; la couleur d'un post-it ; **la couleur des domaines sur le canevas**, et le filtre par domaine (fiche 03 § 11 — de l'ergonomie, sa parole du 22/10 : pas tout de suite) | à son écran |
| [ ] | **Les opérateurs des post-it** (ET, OU, MAIS, PARCE QUE — `Alt+0…4`) : ils se dessinent déjà, aucun geste ne les pose | à son écran |
| [ ] | **Les miroirs** (un même nœud à deux endroits, `Ctrl+Maj+M` ; le noyau en partie) | à son écran |
| [ ] | **La temporalité** : ancrer un nœud dans le temps, la réglette de −10 000 à 2 100, le filtre (`Maj+T`, `Maj+R` ; le noyau écrit) | à son écran |
| [ ] | **Les GIF animés et les vidéos** (un GIF se montre figé) | à son écran |
| [ ] | **L'export HTML** (pan, zoom, descriptions, un fichier) ; **le mode zen** (`F11`) | à son écran |
| [ ] | **La provenance des images** (SauceNAO) | **sa clé** |
| [ ] | **Ce qui attend sa parole** : les rideaux (un tableau dans une membrane — le noyau écrit), les membranes de Mary (étirées, minimisées — le noyau écrit), Trans-domaines (de la R&D depuis sa parole du 22/10 : une vue en perspective 3D de tous les domaines) | sa parole |

### La 2.0.6 — « 100 images par seconde, partout »

Tout ce qui existe, tenu au plancher sur chaque machine — et ce que P8 a laissé ouvert.

| | chantier | ce qui le déclare fini |
|---|---|---|
| [ ] | **La fluidité de son PC** : ses zooms chargés à 11-24 ms (345 images sur 13 886 au-dessus de 10 ms, un tressaut à 91 px au p99) ; **sur batterie**, 30 images par seconde par moments (le bridage de la RTX : une session branchée, une débranchée) ; les gels de l'Intel Arc (l'écran sur la RTX, en mode Optimus ?) ; la coupure du pavé de 7 à 10 s ; le plantage d'une épreuve dans `vulkan-1.dll` (fiche 49 § 4) | chaque point mesuré, puis tenu ou pixelisé — jamais une dégradation qui se voit |
| [ ] | **Le texte de près, la suite** (fiche 73 § 7) : les **titres des membranes**, des zones et des cases en formes (par la `Surface`, qui sait écrire en vecteurs) ; **la frontière « plus grand que l'écran »** remise en question par la mesure — les formes comme repli au lieu de la texture agrandie pendant un zoom (un banc qui zoome sur 10, 100, 1 000 textes, à son PC puis à son téléphone) | mesuré, puis aucun flou pendant un zoom |
| [ ] | **Le modèle de coût qui prédit** (la charte : ne pas mesurer les images en boucle, prédire ce qu'une image va coûter) — COUT-1 ne connaît encore que les pixels des photos | l'écart prédit-mesuré lu dans sa chronique |
| [ ] | **La mémoire des formules** (`MathRenderer`) ne s'oublie jamais — la même loi que celle des mises en page | éprouvé, saboté |

### La 2.0.7 — « Toutes les machines »

| | chantier | ce qui le déclare fini |
|---|---|---|
| [ ] | **Android, ce qui manque** : le presse-papiers du système, les photos HEIC, les champs du dock qui se tapent (le miroir du clavier étendu), les panneaux qui s'écartent des bords ; **les rayures** de sa PowerVR ; la mise à jour automatique d'Android, prouvée par une version qui arrive d'elle-même | chacun à son écran |
| [ ] | **Les doigts sous Windows** (tablettes) ; le dépôt d'une page compressée hors Windows (`ureq` sans `gzip`) | à son écran, ou sur une copie |
| [ ] | **P9 — Le Mac publié** (un `.dmg`, son copier, sa mise à jour) | une version publiée qui s'installe |
| [ ] | **L'iPad par le web** (Glucose dans Safari — la personne qui compte pour lui) | **le modèle de l'iPad**, puis à son écran |

### La 2.1.0 — « Les presets » (la vision)

Un preset est **l'interface entière** ([VISION](VISION.md)) : la disposition, le comportement, les
couleurs, les raccourcis, les fenêtres.

| | chantier | ce qui le déclare fini |
|---|---|---|
| [ ] | **Le thème comme donnée**, puis **la disposition des panneaux comme donnée** (après P6) | rien n'a changé à l'œil ; un thème changé par la table |
| [ ] | **Le preset PureRef** ([PUREREF](PUREREF.md)) : la hiérarchie (l'arbre des membranes et des dossiers), le miroir horizontal et vertical, l'opacité, les niveaux de gris, le plus proche voisin (pixel art), la pipette, le dessin à main levée, les formes, le magnétisme à 45°, réinitialiser une transformation, image suivante et précédente, un diaporama, les modes de fenêtre (derrière tout, au premier plan d'une application, overlay) | sa liste PureRef, cochée |
| [ ] | **Le preset Figma**, **le preset Miro** — leurs façons de faire | sa parole |
| [ ] | **Le preset de Glucose** — *« extrêmement travaillé, qui n'a jamais été créé »* : toutes les fonctions, ergonomique, rapide pour les professionnels, propre à l'œil | sa parole |
| [ ] | **Choisir son preset**, importer et exporter ses réglages (un preset exporté *est* un fichier) ; les raccourcis de la souris et les raccourcis globaux | à son écran |

### La 2.2 — « L'échelle » (le canevas de Wikipédia)

| | chantier | ce qui le déclare fini |
|---|---|---|
| [ ] | **GLISSER-2** (défaire un glisser exactement : une édition neuve du journal, un format neuf) ; **les glyphes des formules calés au pixel entier** | regardé |
| [ ] | **La fondation des 10⁷ nœuds** : l'arène, écrite, branchée ; l'index, la mémoire, le disque à cette échelle | un tableau de dix millions de nœuds, à 100 images par seconde |
| [ ] | **Le canevas de Wikipédia de `glucose-brain`** : des millions de textes, membranes, flèches, domaines — **naviguer ultra facilement** par les domaines et la couleur, jamais en appauvrissant le sens | à son écran |

### La 3.0 — « Ensemble » (la vision, plus loin)

| | chantier | ce qui le déclare fini |
|---|---|---|
| [ ] | **Collaborer** (obligatoire) : le co-working sur le journal du document | à son écran, à deux |
| [ ] | **Les plugins** ([PLUGINS.md](../PLUGINS.md)), **l'IA locale**, **le MCP réécrit en Rust** | sa parole |
| [ ] | **D'humain à IA** : des graphes de questions et de réponses au lieu d'un fil ; le plugin de la langue inventée | sa parole |
| [ ] | **D'IA à IA** : l'idée de Proteo — **commencé** : la porte des agents et la toile de Protéo (fiche 75) ; la suite : le tri de la demande, la mise en page pilotée par Protéo, ses projets dans Glucose | sa parole |
| [ ] | **Le document de recherche** en mathématiques et en physique, non linéaire | sa parole |
| [ ] | **Glucose en briques** : le noyau, le rendu, la typographie, les formules, le format, chacun lisible seul, pour des milliers de forks ; **les dépendances vers zéro**, sans jamais exclure une machine | une brique prise par un autre projet |

### En dernier, toujours

* **L'ergonomie** : une grosse session, quand toutes les fonctions seront là ; **puis l'UX du
  téléphone**, toute refaite — ultra rapide, fluide, pratique.
* **Le dessin de la sélection** (E pour l'instant), avec l'ergonomie.

### La recherche, à côté

* **L'améliorateur d'image à l'import** (sa parole du 15/10) : comprendre une image avant d'y
  toucher (filtres de Gabor, GLCM, LBP — `recherche/cerveau-image/`), pour débruiter, déflouter,
  agrandir sans inventer.

---

## 2. Ce qui attend son écran — une seule liste

Tout est écrit, éprouvé et saboté ; **rien de ceci n'a été jugé par lui**. Sa parole du 15/10 :
*« si j'ai absolument tout testé, normalement 100 % des fonctionnalités sont validées »*.

**Au PC** — la 2.0.3-beta.5 installée, ou `cargo run --release > sortie-essais.txt 2>&1` :

* **Protéo dans Glucose** (fiches 75, 76) : le raccourci « Essai Proteo dans Glucose » de son
  Bureau ; « Autoriser » dans Glucose ; une demande courte (« salut »), une demande d'outils, un
  projet — la bulle Demande dit chaque tri, la taille d'un workflow, puis la réponse ; une recherche :
  ses pages (double-clic les ouvre), les flèches du rapport qui arrivent à côté de chaque citation.
  **Puis (fiche 80)** : sélectionner la réponse de Protéo et lui écrire « développe ça » ; poser un
  post-it dans sa bulle Demande ; ouvrir l'onglet « Ce que Protéo a appris » après un calcul, y
  corriger une leçon, et lui demander s'il l'a vue ; sous une question de Protéo, le bouton « Je ne
  sais pas ».

* **Le texte de près** (fiches 72 et 73) : zoomer très près d'un texte, d'une formule, d'un
  passage qu'une flèche fait briller, d'une sélection en cours d'écriture, d'un emoji, de
  l'italique — **nets dès la première image**, jusqu'à ×256 ; zoomer et dézoomer sans trou ni
  à-coup ; un titre de membrane (encore peint au processeur) ; **un très long texte** (des
  milliers de mots collés dans un seul nœud), vu de près, qu'on parcourt : fluide.
* **Changer un raccourci** (fiche 74) : `Ctrl+Maj+P`, écrire « raccourci », « Changer un
  raccourci… », choisir « Dupliquer », frapper `Ctrl+B` ; `Ctrl+B` duplique ; fermer et rouvrir
  Glucose : il duplique encore ; puis « Rétablir tous les raccourcis de Glucose ».
* **La palette** : `Ctrl+Maj+P`, écrire « dupl », Entrée ; les flèches pour choisir ; Échap ;
  sans rien écrire, faire défiler toute la liste (molette, pavé, flèches, Page bas).
* **Le menu du clic droit** : ses raccourcis viennent du clavier — rien ne doit avoir changé à
  l'œil.
* **Le Storyboard** (fiches 64 à 68) : « Activer », la grille qui suit la souris par son coin,
  un clic ou un rectangle tracé ; la poignée qui le retrace ; une case lâchée ailleurs ; décrire
  (deux clics sur la bande), `Tab` vers la suivante ; plusieurs images lâchées sur une case ;
  « Storyboard de la sélection » ; les deux « + » ; « ▶ Lire » (← →, Espace, Échap) ; « CASE n » :
  durée, plan, champs ; « + Nouveau », ‹ 2 / 3 › ; « Exporter en planches » en PDF et en PNG.
* **L'export SVG** : l'ouvrir dans son navigateur, puis dans Inkscape s'il l'a.
* **Le dossier** : son titre et son compteur ; un dossier vide dit « Double-clic pour ouvrir ».
* **`Alt+F`** cadre tout, même un tableau qui ne porte qu'un storyboard.
* **Mood board** : poser un gabarit, ses zones (bandeau, coins, croix), « Par mood board » dans
  Ordonner.
* **Ordonner** : les trois tris par la couleur ; les champs (glisser, cliquer et taper).
* **L'export PNG** : une sélection, puis tout le tableau.
* **Le clic droit** : Annuler, Rétablir, Tout cadrer, Signets…, Enregistrer, Exporter ; le menu
  en colonnes sur un écran bas.
* **La membrane et le dossier tracés** d'un glisser ; « Décrire » écrit au clavier.
* **La chrome** : rien ne doit avoir changé à l'œil, sauf la minimap (un trait fin, un voile).
* De plus loin : transformer une sélection entière (origine commune ou individuelle) ; le cadre
  du groupe qui suit ; Pinterest (la copie d'abord, puis l'original) ; les sélecteurs de fichiers
  qui ne figent plus Glucose ; les flèches qui contournent et leur coude ; les formules seules
  sur leur ligne ; la Time Machine pendant un zoom ; `Ctrl+Z` après un glisser ; l'aimant sur un
  tableau chargé ; un PDF glissé d'une page web.

**Au téléphone** — l'APK de la beta 5 (zippé sur son Bureau) :

* l'installer **par-dessus** (ne pas désinstaller : un brouillon perdu est peut-être encore là),
  l'ouvrir une fois, puis une **longue session** ; si Glucose se ferme seul, le rouvrir deux
  fois — ses lignes partent au lancement suivant ;
* le texte de près (P8 sur sa PowerVR, jamais mesuré) ;
* le clavier (écrire dans un texte), l'appui long (le clic droit), deux doigts touchés annulent,
  trois rétablissent ; renommer, dupliquer, supprimer un document (appui long dans la liste) ;
* le Storyboard au doigt ; un dossier choisi (son menu, sa barre) ; le partage vers Glucose
  depuis la galerie ou Pinterest ; les emojis ; les noms du rail ; le bord à bord.

**Sous Linux** (son NixOS, quand il est allumé) : le copier vers Discord, un bloc d'un Glucose
à l'autre, après fermeture (fiches 60, 61) ; et la liste de « tant de choses ne marchent pas ».

## 3. Ce qui n'attend que lui

* **Publier une version qui a la porte des agents** (la beta 5 ne l'a pas) : sans elle, rien des
  fiches 75 à 80 ne se voit chez lui ; le brouillon se prépare à sa demande, le clic « Publish
  release » est le sien.
* **Rouvrir Glucose une seconde fois**, au PC et au téléphone : la télémétrie de la beta 4 et de
  la beta 5 part au lancement suivant (P0).
* **Sa clé SauceNAO** (la provenance des images).
* **Le modèle de l'iPad**, les téléphones de ses testeurs Android (le plus ancien d'abord).
* **Ordonner** : ce qu'il entend par « très très grandement ».
* **Les définitions** : ~~« domaine »~~ (**défini le 22/10**, [VISION](VISION.md) — ne plus le lui
  demander) ; Trans-domaines (R&D : une vue en perspective 3D de tous les domaines), les rideaux, les
  membranes de Mary (étirées, minimisées), « optimiser » dans la Time Machine.
* **Sa liste PureRef** (ce qu'il veut retrouver de PureRef), pour le preset.
* **Le dessin de la sélection** : E pour l'instant, à reprendre avec l'ergonomie.
* **Le défilement au bord** (« moche, pas fluide ») : plus tard, à sa parole.
* **Les modes de la fenêtre** (opacité, clics qui traversent) : pas pour l'instant.
* **Le nom des versions** : la 2.1.0 comme première stable, à sa parole.

## 4. Fait (mémoire courte)

* **P5 — L'export SVG complet** (fiches 69 à 71) — reste ses yeux (§ 2).
* **La 2.0.3-beta.3** et **la 2.0.3-beta.4** — publiées par lui le 15/10 ; son essai de la beta 4 :
  plus de 24 000 nœuds au téléphone, « ça n'a pas trop lagé, plutôt cool ».
* **La 2.0.3-beta.5** — publiée à sa demande le 16/10 (§ 1).
