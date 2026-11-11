# La suite

> **Au 11/11/2026** (fiche 107). **Ce qu'on fait ensuite, dans l'ordre** — une ligne par chantier ouvert ;
> le détail d'un chantier vit dans sa fiche du [carnet](carnet/00-INDEX.md), pas ici. Ce qui est fini
> sort de ce document (il est dans sa fiche et dans [ÉTAT](ETAT.md)). Ses paroles : [VISION](VISION.md).
>
> **S'en servir** : prendre le premier chantier ouvert de § 2, le **remettre en question** (ses paroles
> comprises), chercher sur internet avant de concevoir, le faire. Fini = **prouvé** (sur une copie, sur
> GitHub, ou à son écran), jamais « ça compile ». Le retirer d'ici, mettre ÉTAT à jour.

`[ ]` à faire · `[~]` en cours, ou fait et en attente de son écran ou de sa parole

---

## 1. Le cap — la partie 1 (sa parole du 25/10)

La partie 1 est finie, et s'appelle **2.1.0, la première version stable**, quand ces six choses sont
vraies (la ligne d'arrivée proposée le 25/10, à remettre en question) :

1. un utilisateur de Glucose Tauri n'y perd plus rien ([INVENTAIRE](INVENTAIRE.md) § 3 vide) ;
2. aucun tableau perdu ;
3. 100 images par seconde au PC, 60 sur un téléphone courant, **au pire** (le 99ᵉ centile de la
   télémétrie) — ou pixelisées sans que cela se voie ;
4. chaque fonction a son bouton, au PC et au téléphone ;
5. l'ergonomie : le preset de Glucose et l'interface du téléphone, jugés par lui ;
6. toutes les machines qu'on peut servir sans argent.

## 2. Maintenant, dans l'ordre — ses paroles du 11/11

| | chantier | fini quand |
|---|---|---|
| [~] | **Les membranes en carte** (fiches 105 et 107) : écrites, éprouvées, sabotées, regardées ; leur corps deux fois plus transparent (11 %, le voile d'une membrane seule de Tauri : *« vraiment vraiment parfait »*). Reste **l'allure de la frontière**, à sa parole (le trait continu, qui ferait partir le pointillé de toutes les membranes : https://claude.ai/artifact/7L4zAY5U4mnebV2qvHgSK4) | sa parole sur la frontière, puis son écran |
| [~] | **Le halo d'un texte dit sa parenté** (fiche 106) : écrit, éprouvé, regardé sur la carte ; sa parole : les flèches grises *« seulement si c'est temporaire »*, et les membranes plus transparentes (fait, fiche 107). Reste **son écran**. **Ensuite, à concevoir avec lui** : sa couleur et ce que l'IA en apprend — la méta-compréhension (VISION § 8 bis : l'IA aide, l'humain corrige, l'IA enregistre et apprend), une page de conception d'abord ; la recherche est rangée dans `~/.cache/glucose-bancs/membranes-107/recherche-meta.md` | son écran ; la page de conception |
| [~] | **Les flèches de Wikipédia** (VISION § 6) : **PARENTE-4, prototype** (fiche 107) — l'IA juge chaque flèche (le sens d'une région, le hasard de la cible hors d'elle) ; la souris ne rapproche que les flèches de sens. Sauvé : `~/.cache/glucose-bancs/fleches-107/prototype-parente-4.patch` (sur `5e48642`). **Attend sa réponse** (https://claude.ai/artifact/M5YZbYFWmwH3jScdcKwWws) : l'IA juge-t-elle aussi les flèches qu'il tire lui-même ? (défaut proposé : non — retenir qui a posé chaque flèche). Puis : une couleur qui dit quelque chose, partir des **mots** qui citent (les ancres de texte existent), des liens partiels, un tracé à logique de câblage. La recherche : `~/.cache/glucose-bancs/halo-106b/recherche-fleches.md` | sa réponse, puis à son écran, sur la carte de Wikipédia |
| [ ] | **Son lag** (*« pour ce qui est capital et nécessaire, il faut pouvoir optimiser »*) : sur son VivoBook, 13,8 ms l'image typique, 2,6 s la pire (la télémétrie du 11/11) ; au fond d'une région, REGION-1 a retiré l'attente de la carte graphique (17,6 → 7,8 ms, fiche 105) ; de loin, 15,7 ms restent ; lire sa chronique, nommer les postes au profil, régler ce qui pèse | sa chronique sous 10 ms au 99ᵉ centile, sur sa carte |
| [ ] | **Les liens bleus des textes** : contraste APCA 52, sous 60 (fiche 104) — la même teinte, à la clarté qui se lit ; après le fond gris | mesuré, regardé |

## 3. Ce qui attend son écran

Écrit, éprouvé, saboté — **jamais jugé par lui**. Sous Linux : `glucose > ~/sortie-glucose.txt 2>&1`, puis
on lit nous-mêmes la sortie et sa chronique (`/tmp/glucose-chronique/derniere-session.txt`, copiée
d'abord).

**Sur la carte de Wikipédia** (`~/Documents/Glucose-essai-wikipedia-liens.glucose`, une copie) :

* le halo de la parenté (fiche 106) : au fond de « Eau », les textes de l'eau d'une même couleur, ceux
  de la chimie d'une autre, les autres au contour gris ; écrire un texte, attendre une seconde : sa
  couleur suit ; colorer un texte (la barre de couleur) : sa lueur prend la couleur, pleine ;

* les membranes en carte (fiche 105) : de loin, une couleur par région, aucune boue ; zoomer au fond
  de « Eau » : les blocs gris sombres sur la couleur de leur région, plus fluide qu'avant ; le grand
  cadre « Wikipédia » gris ;

* dézoomer, bouger la souris : plus fluide (LETTRES-1, fiche 104) ;
* plusieurs coups de molette avec des pauses d'une seconde, puis `Alt+Z` : un pas par série — le veut-il
  ainsi ? (RETOUR-2, fiche 103) ;
* survoler un nœud, `Ctrl`+clic sur une flèche, `Alt+Z`, choisir un nœud, `B`, cliquer un amené (les
  liens, fiches 102-103) ; l'effet de la souris est doux (sa courbe) — le veut-il plus net ?
* « Entrer dans la membrane », `Échap` (FOCUS-1) ; les titres à la taille de leur région (TITRE-1) ;
  « Disposer en colonnes ».

**Ailleurs, au PC** :

* écrire dans un texte, un post-it, le titre d'une membrane ; un texte qui passe à la ligne (le contour
  suit) ; `Ctrl+G` sur deux nœuds (SAISIE, GROUPER) ;
* déplacer au clic molette sur beaucoup de photos : aucune ne clignote, aucun micro-gel (fiche 100) ;
* la roue de couleur : une membrane, la case au disque, `Ctrl`, `Maj`, la molette, un code, la pipette ;
* `Ctrl+F` sur un gros tableau ; `F3` ;
* ses commandes au terminal : `glucose aide`, `glucose chercher eau` ; un double-clic sur un `.glucose` ;
* le domaine dans deux documents : « où », la question de deux « Python » ;
* les opérateurs des post-it : `Alt+1…4` ;
* copier un texte vers Discord, coller une image, glisser depuis Nautilus et vers lui (le pont Wayland) ;
* un très long texte vu de près ; zoomer très près d'une formule, d'un emoji (P8) ;
* la palette (`Ctrl+Maj+P`), changer un raccourci ;
* un agent dans Glucose (`glucose --mcp`) : « Autoriser », une demande ;
* le Storyboard, Mood board, Ordonner, l'export SVG et PNG, le mode référence.

**Au téléphone** (le sien est encore en **beta 4**, 17 fins anormales sur 22 sessions) : **l'ouvrir et le
mettre à jour par-dessus** — jamais désinstaller, un brouillon perdu y est peut-être encore ; une longue
session ; le texte de près ; les doigts (deux touchers annulent, trois rétablissent) ; le Storyboard au
doigt ; la pile des barres du bas (BAS-1 : gêne-t-elle ?).

**Quand une machine Windows existera** : la bande qui remplace la barre blanche (BARRE-1), éteindre le PC
Glucose ouvert (FIN-SESSION-1), un dépôt posé sous le curseur.

## 4. Ce qui attend sa parole

* **Le cap** : l'iPad par le web dans la partie 1 ? (défaut : oui, en dernier) ; les presets Figma et Miro
  (défaut : partie 2) ; « avec des utilisateurs » = des gens qui s'en servent sans rien perdre (des comptes
  seraient la collaboration, partie 2) ; le nom des versions (2.1.0 la première stable).
* **Glisser un nœud vers une fenêtre posée par-dessus Glucose** (fiche 100 § 10) : le veut-il, par quel
  geste ?
* **B et C** (fiche 98 § 3) : une membrane dit ce qu'elle porte ; un fil « où je suis ».
* **Le sens des couleurs des membranes posé par un agent** (fiche 102 § 6, l'image
  `~/.cache/glucose-bancs/squelette-sens2/images/`) : *« je ne sais pas trop »* (10/11) — à reprendre avec
  l'IA de similarité du halo, qui peut aussi dire la proximité des régions.
* **« Créer un sous-dossier »** (fiche 98 § 7) : ranger la sélection dans un dossier neuf — que deviennent
  les flèches qui traversent ? avec le dossier « à retravailler ».
* **Ordonner « très très grandement »** ; **sa liste PureRef** ; **le dessin de la sélection** ; **le
  défilement au bord** (« moche, pas fluide », plus tard) ; **les modes de la fenêtre** (pas pour
  l'instant) ; **les rideaux**, **les membranes de Mary**, « optimiser » dans la Time Machine.
* **Ses clés et ses appareils** : une copie de la clé de mise à jour sur une clé USB à lui ;
  `sauvegarde-profil-2026-10-31.7z` (ses clés de serveur, son identité web3), introuvable ici ; sa clé
  SauceNAO ; le modèle de l'iPad ; les téléphones de ses testeurs.
* **Protéo**, quand il reprendra : la bascule de son cerveau (qwen3.5:4b, démontré meilleur, fiche 78).

## 5. Plus tard — la partie 1, par étapes

**Ne rien perdre**
* [~] P0 — le tableau de 6 000 images mort au téléphone : reproduit et réparé (MEMOIRE-4, fiche 83) ;
  reste qu'une version récente arrive sur son téléphone et que ses fins anormales cessent.
* [~] MOTEUR-1 (fiche 84) : prouvé seulement quand la télémétrie montre une installation touchée qui
  dessine.

**Tout ce qui manque** (la parité avec Tauri, et ses demandes)
* [ ] P6 — relire chaque panneau contre Tauri, champ par champ : Pomodoro, Domaines, Time Machine, la
  bande, Ordonner, Mood board.
* [ ] Les miroirs (`Ctrl+Maj+M`) ; la temporalité (la réglette de −10 000 à 2 100) ; les GIF animés et
  les vidéos ; l'export HTML ; le mode zen (`F11`) ; la provenance des images (SauceNAO).
* [ ] Les choix à l'utilisateur : la planche claire ou sombre, l'export PNG à 100 % ou aux originaux, les
  photos du SVG, la numérotation « 2.3 », la place du numéro.
* [ ] Le domaine : ses synonymes (« eau », « water ») ; sa visualisation (ergonomie).
* [ ] Le dossier, à retravailler ; « double-clic + zoom comme Tauri ».
* [ ] L'encre d'un post-it ; les titres et le gris de loin (GRIS-3, fiche 98 § 7).

**Fluide partout**
* [ ] Son téléphone : lire ses postes (16,4 ms l'image typique), nommer le coupable, le tenir ; P8 au
  téléphone, jamais mesuré.
* [ ] Le modèle de coût qui prédit (COUT-1 ne connaît que les pixels des photos).
* [ ] Les titres des membranes, des zones et des cases en formes, de près (fiche 73 § 7).
* [ ] Choisir 8 000 images (28 ms par image) ; le fondu d'un message (REPOS-3) ; la mémoire des formules
  qui ne s'oublie jamais ; le zoom sur une très grande région (fiche 95 § 8).

**Toutes les machines**
* [ ] Android : le presse-papiers du système, HEIC, les champs du dock, sa mise à jour automatique
  prouvée ; les rayures de sa PowerVR.
* [ ] Les doigts sous Windows ; le Mac publié (`.dmg`, non signé) ; **l'iPad par le web**.
* [ ] Le `.deb` qui dit ses vraies dépendances (GTK 3 retiré, fiche 100) ; la sortie des commandes du
  terminal dans une console Windows.

**L'ergonomie (la 2.1.0)**
* [ ] Le thème, puis la disposition des panneaux, comme données (après P6).
* [ ] Le preset de Glucose ; le preset PureRef ([PUREREF](PUREREF.md)) ; choisir son preset, l'exporter.
* [ ] L'interface du téléphone, toute refaite ; la visualisation des domaines.

## 6. Après la partie 1

* **Protéo, en pause** (sa parole du 25/10) : l'agent et l'humain dans les deux sens, la toile rangée, le
  tri de la demande, la machine qui choisit seule, l'organe de recherche — écrits et éprouvés au point où
  ils sont (fiches 75 à 81).
* **La partie 2** : les 10⁷ nœuds (l'arène, écrite, à brancher) ; le canevas de Wikipédia de
  `glucose-brain` et des jeux de données d'IA ; les presets Figma et Miro ; **Collaborer** (obligatoire) ;
  les plugins, l'IA locale ; les graphes de questions et de réponses ; le document de recherche ;
  Trans-domaines ; Glucose en briques, les dépendances vers zéro ; GLISSER-2.
* **La recherche, à côté** : l'améliorateur d'image à l'import (`recherche/cerveau-image/`).

## 7. Publiées

La **2.0.3-beta.10** (10/11) est la dernière ; chaque version a sa fiche (les notes : `outils/publication/`).
