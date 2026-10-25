# La suite, mise à jour par mise à jour

> **Au 25/10/2026** (fiches 73 à 82 ; la 2.0.3-beta.6 publiée le 23/10 ; **Protéo en pause,
> la partie 1 d'abord**). **Le plan complet** de ce qui attend Glucose Rust, rangé en étapes :
> ce que chacune porte, et ce qui la déclare finie. Tout y est à remettre en question — ses
> paroles comprises. La direction vit dans [VISION](VISION.md), l'état vérifié dans
> [ÉTAT](ETAT.md), chaque fonction et son bouton dans [INVENTAIRE](INVENTAIRE.md), la cible
> PureRef dans [PUREREF](PUREREF.md).
>
> **Comment s'en servir** : prendre le premier chantier non fait de la première étape non finie.
> Fini = **prouvé** (sur une copie, sur GitHub, ou à son écran), jamais « ça compile ». Le cocher
> ici, mettre ÉTAT à jour, passer au suivant. Une version se publie quand ses chantiers sont
> prouvés — **le nom et le moment restent les siens**.

`[x]` fini et prouvé · `[~]` écrit, éprouvé, saboté — **attend son écran** · `[ ]` à faire

---

## 0. La partie 1 — sa parole du 25/10

*« J'aimerais mettre une pause à Protéo et essayer de définitivement finir la partie 1 de
Glucose : le logiciel complet, complet, en Rust, ultra optimisé, avec des utilisateurs et une
ergonomie parfaite. »*

**« Parfait » n'a pas de ligne d'arrivée** ; sans elle, rien ne se finit « définitivement ». La
session du 25/10 propose celle-ci (à remettre en question) : la partie 1 est finie, et s'appelle
**2.1.0, la première version stable**, quand ces six choses sont vraies :

1. **Un utilisateur de Glucose Tauri n'y perd plus rien** ([INVENTAIRE](INVENTAIRE.md) § 3 vide,
   sauf ce qu'il repousse lui-même).
2. **Aucun tableau perdu** : la mort du téléphone du 15/10 expliquée et corrigée ; Glucose finit
   proprement quand le système le ferme.
3. **100 images par seconde au PC, 60 sur un téléphone courant**, lues dans la télémétrie au pire
   (le 99ᵉ centile), pas en médiane — ou pixelisées sans que cela se voie.
4. **Chaque fonction a son bouton**, au PC et au téléphone.
5. **L'ergonomie** : le preset de Glucose et l'interface du téléphone refaite, jugés par lui.
6. **Toutes les machines qu'on peut servir sans argent.**

**Cinq étapes, dans cet ordre** : ne rien perdre ; tout ce qui manque ; fluide partout ; toutes
les machines ; l'ergonomie — en dernier, comme il l'a toujours dit (VISION : « l'ergonomie se fait
à la fin, quand toutes les fonctionnalités seront là »). Ce qui garde l'ergonomie possible
pendant les étapes 2 à 4 : chaque fonction neuve entre dans la table des commandes, ses
raccourcis dans la couche de texte, ses couleurs dans le thème (P4) — rien en dur.

**Ordre de grandeur, honnête** : 25 à 35 sessions comme celles d'octobre.

**Ce qui attend sa parole** (les défauts de la session, qu'il peut renverser) :

* **L'iPad par le web** dans la partie 1 ? Défaut : oui, en dernier des machines (la plus grande
  inconnue : Glucose dans Safari).
* **Les presets Figma et Miro** : défaut, la partie 2 ; la partie 1 porte le preset de Glucose et
  celui de PureRef.
* **« Avec des utilisateurs »** : lu comme « des gens qui s'en servent sans rien perdre ». Des
  comptes, une connexion, ce serait la collaboration : défaut, la partie 2 (le bouton dit
  « bientôt »).

### Pourquoi cet ordre — quatre principes, à contredire

1. **Ce qu'il voit d'abord.** Une version publiée sans son essai est un pari : chaque étape
   commence par juger la précédente à son écran, et par lire sa télémétrie.
2. **Ne jamais perdre un tableau** passe avant toute fonction neuve — sa panique devant une perte,
   et deux téléphones l'ont vécu.
3. **Le manque de fonctions bloque plus que la vitesse** (la charte : « le blocage actuel n'est
   pas la performance ») — mais **100 images par seconde, toujours**.
4. **Relire avant de bâtir** : chaque panneau contre Tauri (P6) *avant* que le thème et la
   disposition deviennent des données (les presets) — sinon on transcrit des panneaux qu'il
   faudra refaire.

---

## 1. Le plan

### Publiées

* **La 2.0.3-beta.5** (16/10) : P8, P8 bis, ses raccourcis (fiches 73, 74).
* **La 2.0.3-beta.6** (23/10, sur `4a111b4`) : la porte des agents et tout ce que les fiches 75 à
  81 ont fait (lire par niveaux, ce que l'humain désigne, FLECHE-6, ACCUEIL-1). **Ses notes sont
  celles de la beta 5 recopiées** (« depuis la 2.0.3-beta.4 », rien sur les agents) : à corriger
  à sa parole.

| | chantier publié | ce qui le déclare fini |
|---|---|---|
| [~] | **P8 — Le texte de près par la carte graphique** (fiche 73) : Slug, CADRE-1, SLUG-1 ; ~1 ms par image de près, contre ~20 et le flou | de près, aucun flou, sur sa machine et au téléphone |
| [~] | **P8 bis — Un nœud immense, de près** (fiche 73 § 7) : LIGNE-1 et la mémoire des mises en page — 10,5 ms → 0,72 ms à 17 600 mots | un très long texte, de près ; un tableau plein de textes |
| [~] | **P4 — Tes raccourcis** (fiches 72, 74) : la table des commandes, la palette, la couche de l'utilisateur | `Ctrl+Maj+P` ; changer un raccourci, relancer |
| [~] | **P10 — La porte des agents** (fiche 75) : `glucose --mcp`, huit outils, chaque geste annulable, rien sans son accord ; lire par niveaux, `selection`, `empreinte`, `document`, `glucose_ouvrir_tableau` (fiches 79, 80) | un agent travaille dans son Glucose, sous ses yeux |

### Étape 1 — Ne rien perdre

| | chantier | ce qui le déclare fini |
|---|---|---|
| [~] | **FIN-SESSION-1 — Glucose finit quand le système le ferme** (fiche 82) : sous Windows, `winit` ne transmet pas `WM_ENDSESSION` — l'extinction tuait Glucose, et la boîte noire lisait « interrompue » (deux sessions de 3 h 37 et 7 h chez un utilisateur, les 21 et 22/10) ; sous Linux, `SIGTERM` et `Ctrl+C` tuaient aussi. **FIN-1** : la fin, quelle qu'elle soit, s'écrit une fois (`finir`) | éteindre son PC avec Glucose ouvert : la session suivante dit « fermée proprement » ; la télémétrie ne compte plus d'extinctions parmi les fins anormales |
| [ ] | **P0 — Ne jamais perdre un tableau au téléphone** : fait côté code (fiche 66 : MEMOIRE-1 à 3 ; fiche 72 : LIBERE-1, SORTIE-1, REPRISE-1). **Le 15/10, sur deux téléphones, un tableau de plus de 6 000 images s'est fermé seul et n'est pas revenu** : cause inconnue. Ses lignes `sortie`, `reprise` et `precedente` la diront — **au 25/10, toujours aucune session de son téléphone depuis le 15/10** (encore en beta 3) : **ouvrir Glucose sur son téléphone** (il proposera la beta 6). **Ne pas désinstaller** : le brouillon est peut-être encore là. Piste à mener ici : reproduire sur un émulateur Android (un tableau de 6 000 images, la mémoire bornée comme son moto g54). Relevé : `node outils/telemetrie/tableau.mjs <dossier>` (7403 → `npx wrangler whoami`, puis une fois encore) | la cause nommée par ses lignes ; corrigée, éprouvée sur une copie |
| [ ] | **Ses essais de la beta 6** (§ 2) : chaque défaut qu'il voit, corrigé — en premier, le texte de près | sa parole |
| [~] | **Les boutons qui se taisent** : Trans-domaines dit « bientôt — en recherche » **depuis la fiche 71** (épreuve `test_the_trans_domain_button_says_it_is_coming` ; SUITE et INVENTAIRE l'ignoraient jusqu'au 25/10) ; la carte d'accueil ne naît plus choisie (ACCUEIL-1, fiche 81). Tous deux dans la beta 6 | à son écran |
| [x] | **Le repos à zéro** — mesuré le 18/10 : 0,2 image par seconde sans la main, toutes pour le fondu du message de bienvenue | une chronique au repos sans image hors un message qui paraît |

### Étape 2 — Tout ce qui manque (la parité avec Glucose Tauri, et ses demandes)

| | chantier | ce qui le déclare fini |
|---|---|---|
| [ ] | **Le domaine** (sa définition du 22/10, [VISION](VISION.md)) : un domaine **traverse** les blocs, les membranes et les fichiers (« l'eau, ça mouille », « eau = H₂O », « l'eau dans un jeu vidéo » : trois blocs loin les uns des autres, un seul domaine). Ce qui existe : le catalogue d'un document, l'appartenance pondérée de chaque nœud, le panneau, les badges. Ce qui manque : **traverser les fichiers** (un catalogue qui ne soit plus celui d'un seul document) ; **une IA qui lit et pose les domaines** (`glucose_lire` ne les dit pas — contraire à « lisible pareil par l'humain et par l'IA », une règle de Glucose, pas de Protéo). Sa visualisation (le voir, le filtrer, sa couleur) va à l'étape 5 | un domaine relie des nœuds de deux fichiers ; un agent lit et pose les domaines |
| [ ] | **La recherche, `Ctrl+F`** (sa décision du 13/10 : tout cadrer est passé à `Alt+F`) : chercher dans tout le document — textes, titres, noms de fichiers —, voler d'un résultat au suivant ; sur le noyau, pour tenir 10⁷ nœuds | à son écran, sur un gros tableau |
| [ ] | **P6 — Relire chaque panneau et chaque menu contre Tauri, champ par champ** : Pomodoro, Domaines, Time Machine, les menus du clic droit, la bande, Ordonner, Mood board | la liste de ce qui manque, puis fait |
| [ ] | **La couleur** : un sélecteur ; la couleur d'un post-it | à son écran |
| [ ] | **Les opérateurs des post-it** (ET, OU, MAIS, PARCE QUE — `Alt+0…4`) : ils se dessinent déjà, aucun geste ne les pose | à son écran |
| [ ] | **Les miroirs** (un même nœud à deux endroits, `Ctrl+Maj+M` ; le noyau en partie) | à son écran |
| [ ] | **La temporalité** : ancrer un nœud dans le temps, la réglette de −10 000 à 2 100, le filtre (`Maj+T`, `Maj+R` ; le noyau écrit) | à son écran |
| [ ] | **Les GIF animés et les vidéos** (un GIF se montre figé) | à son écran |
| [ ] | **L'export HTML** (pan, zoom, descriptions, un fichier) ; **le mode zen** (`F11`) | à son écran |
| [ ] | **Les choix à l'utilisateur** (sa parole : « TOUJOURS CHOISIR ») : la planche claire ou sombre, les champs sous les cases, « 2.3 », le numéro dans la bande ou le coin, l'export PNG à 100 % ou aux originaux, les photos du SVG en originaux ou réduites | chaque choix là où l'on agit, enregistré, saboté |
| [ ] | **Ordonner, « très très grandement »** (sa proposition du 15/10) : **à concevoir avec lui**. Ce qui manque déjà (PureRef) : par nom, par date d'ajout, en ligne, en colonne, aligner avec un espacement, distribuer, empiler, normaliser la largeur, la taille, l'aire, l'échelle ; aléatoire | sa parole, puis à son écran |
| [ ] | **Le dossier, à retravailler** (sa parole du 15/10) ; « double-clic + zoom comme Tauri », au PC et au téléphone | à son écran |
| [ ] | **La provenance des images** (SauceNAO) | **sa clé** |
| [ ] | **Ce qui attend sa parole** : les rideaux (un tableau dans une membrane — le noyau écrit), les membranes de Mary (étirées, minimisées — le noyau écrit) | sa parole |

### Étape 3 — Fluide partout (100 images par seconde)

| | chantier | ce qui le déclare fini |
|---|---|---|
| [ ] | **P3 — Ce que son téléphone dira** : 16,4 ms par image ordinaire sur sa PowerVR (la télémétrie), 885 ms au pire ; POSTES-1 (les lignes `poste`), puis DESSUS-1 et RASTER-1 si elles les désignent ; et **P8 au téléphone**, jamais mesuré | les postes lus, le coupable nommé par la mesure, puis tenu |
| [ ] | **La fluidité de son PC** : ses zooms chargés à 11-24 ms (345 images sur 13 886 au-dessus de 10 ms) ; **sur batterie**, 30 images par seconde par moments ; les gels de l'Intel Arc ; la coupure du pavé de 7 à 10 s ; le plantage d'une épreuve dans `vulkan-1.dll` (fiche 49 § 4) | chaque point mesuré, puis tenu ou pixelisé — jamais une dégradation qui se voit |
| [ ] | **Le texte de près, la suite** (fiche 73 § 7) : les titres des membranes, des zones et des cases en formes ; la frontière « plus grand que l'écran » remise en question par la mesure | mesuré, puis aucun flou pendant un zoom |
| [ ] | **Le modèle de coût qui prédit** (la charte) — COUT-1 ne connaît encore que les pixels des photos | l'écart prédit-mesuré lu dans sa chronique |
| [ ] | **La mémoire des formules** (`MathRenderer`) ne s'oublie jamais — la même loi que celle des mises en page | éprouvé, saboté |

### Étape 4 — Toutes les machines

| | chantier | ce qui le déclare fini |
|---|---|---|
| [ ] | **Android, ce qui manque** : le presse-papiers du système, les photos HEIC, les champs du dock qui se tapent, les panneaux qui s'écartent des bords ; les rayures de sa PowerVR ; la mise à jour automatique d'Android, prouvée par une version qui arrive d'elle-même | chacun à son écran |
| [ ] | **Les doigts sous Windows** (tablettes) ; le dépôt d'une page compressée hors Windows (`ureq` sans `gzip`) | à son écran, ou sur une copie |
| [ ] | **P9 — Le Mac publié** (un `.dmg`, son copier, sa mise à jour — sans compte Apple : non signé) | une version publiée qui s'installe |
| [ ] | **L'iPad par le web** (Glucose dans Safari — la personne qui compte pour lui) — **à sa parole** pour la partie 1 | **le modèle de l'iPad**, puis à son écran |

### Étape 5 — L'ergonomie (la 2.1.0)

Un preset est **l'interface entière** ([VISION](VISION.md)) : la disposition, le comportement, les
couleurs, les raccourcis, les fenêtres.

| | chantier | ce qui le déclare fini |
|---|---|---|
| [ ] | **Le thème comme donnée**, puis **la disposition des panneaux comme donnée** (après P6) | rien n'a changé à l'œil ; un thème changé par la table |
| [ ] | **Le preset de Glucose** — *« extrêmement travaillé, qui n'a jamais été créé »* : toutes les fonctions, ergonomique, rapide pour les professionnels, propre à l'œil | sa parole |
| [ ] | **Le preset PureRef** ([PUREREF](PUREREF.md)) : la hiérarchie, le miroir horizontal et vertical, l'opacité, les niveaux de gris, le plus proche voisin, la pipette, le dessin à main levée, les formes, le magnétisme à 45°, réinitialiser une transformation, image suivante et précédente, un diaporama, les modes de fenêtre | sa liste PureRef, cochée |
| [ ] | **Choisir son preset**, importer et exporter ses réglages (un preset exporté *est* un fichier) ; les raccourcis de la souris et les raccourcis globaux | à son écran |
| [ ] | **La visualisation des domaines** : leur couleur sur le canevas, le filtre par domaine (fiche 03 § 11) — de l'ergonomie, sa parole du 22/10 | à son écran |
| [ ] | **L'interface du téléphone, toute refaite** : ultra rapide, fluide, pratique | à son écran |
| [ ] | **Le dessin de la sélection** (E pour l'instant) | sa parole |

---

## 1 bis. Après la partie 1

### En pause : Protéo (sa parole du 25/10)

Tout ce qui suit est écrit, éprouvé et saboté au point où il est, et attend. La porte des agents
(P10), elle, est dans Glucose et publiée (beta 6) : rien à défaire.

| | chantier | où il en est |
|---|---|---|
| [~] | **L'agent et l'humain, dans les deux sens** (fiches 79, 80) : la relecture à chaque message, le carnet « Ce que Protéo a appris », le « je ne sais pas ». **Reste** : les commentaires ancrés, la provenance par nœud, les membranes repliables, le statut et la preuve écrits par le programme | vu en essai réel hors écran |
| [~] | **La toile, rangée** (fiche 77 § 5) : RANGEE-1, SUITE-2 | à voir en essai réel |
| [~] | **Le tri de la demande** (fiche 76) : réponse courte, outils, ou workflow et sa taille | 23 justes sur 28 |
| [~] | **Le cerveau de Protéo** (fiches 77, 78) : **qwen3.5:4b démontré meilleur que qwen2.5:7b (15/20 contre 3/20, p = 0,0005)**. **Reste** : finir les essais réels, **lui proposer la bascule**, remplacer `proteo.exe` | la bascule reste sa parole |
| [~] | **La machine qui choisit seule** (fiches 77, 78) : écrite et sabotée ; **reste** à la brancher | — |
| [~] | **L'organe de recherche** (fiche 77 § 4) : PROGRES-1, SUJET-1, DEPEND-1 ; **reste** les pages dans une autre langue | — |
| [ ] | **Le flou** (sa parole du 17/10) : quand la recherche découvre qu'elle ne sait pas assez, Protéo se relance seul | — |

### La partie 2 — l'échelle, ensemble

| | chantier | ce qui le déclare fini |
|---|---|---|
| [ ] | **GLISSER-2** (défaire un glisser exactement) ; **les glyphes des formules calés au pixel entier** | regardé |
| [ ] | **La fondation des 10⁷ nœuds** : l'arène, écrite, branchée ; l'index, la mémoire, le disque à cette échelle | un tableau de dix millions de nœuds, à 100 images par seconde |
| [ ] | **Le canevas de Wikipédia de `glucose-brain`**, et des jeux de données d'IA (sa parole du 22/10) : naviguer ultra facilement par les domaines et la couleur, jamais en appauvrissant le sens | à son écran |
| [ ] | **Les presets Figma et Miro** (défaut de la session du 25/10 : partie 2) | sa parole |
| [ ] | **Collaborer** (obligatoire) : le co-working sur le journal du document | à son écran, à deux |
| [ ] | **Les plugins** ([PLUGINS.md](../PLUGINS.md)), **l'IA locale**, **le MCP réécrit en Rust** | sa parole |
| [ ] | **D'humain à IA** : des graphes de questions et de réponses au lieu d'un fil ; le plugin de la langue inventée | sa parole |
| [ ] | **Le document de recherche** en mathématiques et en physique, non linéaire | sa parole |
| [ ] | **Trans-domaines** (de la R&D : une vue en perspective 3D de tous les domaines) | sa parole |
| [ ] | **Glucose en briques** : le noyau, le rendu, la typographie, les formules, le format, chacun lisible seul ; **les dépendances vers zéro**, sans jamais exclure une machine | une brique prise par un autre projet |

### La recherche, à côté

* **L'améliorateur d'image à l'import** (sa parole du 15/10) : comprendre une image avant d'y
  toucher (filtres de Gabor, GLCM, LBP — `recherche/cerveau-image/`), pour débruiter, déflouter,
  agrandir sans inventer.

---

## 2. Ce qui attend son écran — une seule liste

Tout est écrit, éprouvé et saboté ; **rien de ceci n'a été jugé par lui**. Sa parole du 15/10 :
*« si j'ai absolument tout testé, normalement 100 % des fonctionnalités sont validées »*.

**Au PC** — la 2.0.3-beta.6 installée, ou `cargo run --release > sortie-essais.txt 2>&1` :

* **Éteindre le PC avec Glucose ouvert** (fiche 82), puis rouvrir Glucose : le travail est là, et
  rien ne doit avoir demandé quoi que ce soit à l'extinction.
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
* **Un agent dans Glucose** (fiches 75, 80) : « Autoriser » ; une demande, une recherche — ses
  pages (le double-clic les ouvre), les flèches qui arrivent à côté de chaque citation ;
  sélectionner un nœud et lui dire « ça ».
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

**Au téléphone** — **ouvrir Glucose** : il proposera la beta 6 (sinon, l'APK zippé sur son
Bureau) :

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

* **Ouvrir Glucose sur son téléphone** (encore en beta 3, muet depuis le 15/10) : la seule façon
  de savoir pourquoi le tableau de 6 000 images est mort (P0).
* **Ses trois réponses de la partie 1** (§ 0) : l'iPad, les presets Figma et Miro, « avec des
  utilisateurs ».
* **Les notes de la beta 6**, recopiées de la beta 5 : les corriger sur GitHub, à sa parole.
* **Sa clé SauceNAO** (la provenance des images).
* **Le modèle de l'iPad**, les téléphones de ses testeurs Android (le plus ancien d'abord).
* **Ordonner** : ce qu'il entend par « très très grandement ».
* **Les définitions** : ~~« domaine »~~ (**défini le 22/10**, [VISION](VISION.md) — ne plus le lui
  demander) ; Trans-domaines (R&D), les rideaux, les membranes de Mary (étirées, minimisées),
  « optimiser » dans la Time Machine.
* **Sa liste PureRef** (ce qu'il veut retrouver de PureRef), pour le preset.
* **Le dessin de la sélection** : E pour l'instant, à reprendre avec l'ergonomie.
* **Le défilement au bord** (« moche, pas fluide ») : plus tard, à sa parole.
* **Les modes de la fenêtre** (opacité, clics qui traversent) : pas pour l'instant.
* **La bascule du cerveau de Protéo** (qwen3.5:4b) : quand Protéo reprendra.
* **Le nom des versions** : la 2.1.0 comme première stable, à sa parole.

## 4. Fait (mémoire courte)

* **P5 — L'export SVG complet** (fiches 69 à 71) — reste ses yeux (§ 2).
* **La 2.0.3-beta.3** et **la 2.0.3-beta.4** — publiées par lui le 15/10 ; son essai de la beta 4 :
  plus de 24 000 nœuds au téléphone, « ça n'a pas trop lagé, plutôt cool ».
* **La 2.0.3-beta.5** (16/10) et **la 2.0.3-beta.6** (23/10).
