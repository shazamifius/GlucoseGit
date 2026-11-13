# La suite

> **Au 13/11/2026, 23 h 58** (fiches 108-115). **Ce qu'on fait ensuite, dans l'ordre** — une ligne par chantier ouvert ;
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

## 2. Maintenant, dans l'ordre — ses paroles du 11 et du 12/11

**Sa parole du 12/11 au soir** (VISION § 3) : publier la **2.0.4-beta.1**, qu'il teste en conditions
réelles (des journaux, de la télémétrie), puis **les boutons qui manquent — d'abord le co-working sur un
canevas Glucose et les connexions MCP avec des IA publiques**. Elle passe devant le reste de ce tableau.

| | chantier | fini quand |
|---|---|---|
| [~] | **La 2.0.4-beta.1 est publiée** (son clic, le 13/11 à 18 h 23). Reste à l'installer **au téléphone, par-dessus** (jamais désinstaller ; il est en beta 8) et au PC de l'école (beta 9) — son geste ; puis lire ses vraies sessions (PARU-1 dans sa chronique) | installée chez lui et au téléphone, une vraie session lue |
| [~] | **Son lag, à son écran** (fiche 112) : le rythme n'était pas la cause (le « 32 % » était le compteur du tempo) ; **PARU-1** dit désormais ce que l'écran montre, et `outils/pilote/essai_ecran.sh` mesure ses vrais gestes (pavé virtuel). **Ensuite** : (1) **le calage sur le battement** — envoyer chaque image « au prochain battement moins une avance » efface la gigue d'un battement (prototype : `~/.cache/glucose-bancs/rythme-112/calage-et-sonde-prototype.patch`) ; l'avance n'est pas la bonne grandeur : l'image suivante de Glucose occupe la carte graphique pendant que niri compose — le levier est de **dessiner juste à temps** (fiche 112 § 5), puis mesurer en paires à son écran ; (2) la latence de deux battements de l'envoi à l'écran, sur une carte graphique partagée avec niri ; (3) ses journaux après la beta (PARU-1 dans sa chronique, une fois son Glucose à jour) | à son écran, PARU-1 : la gigue nulle, aucun battement sauté de plus qu'aujourd'hui, la latence mesurée |
| [~] | **Les connexions MCP avec des IA publiques** (« la porte publique ») : la recherche est faite et **la page attend sa parole** — https://claude.ai/artifact/36hpNTPEH2mCEENhKjyvzF (fiche 111 § 3) : un relais Cloudflare à lui, l'accord donné dans Glucose (un code), ses quatre questions | une IA publique lit et écrit son canevas, chaque geste signé et annulable |
| [~] | **Le co-working sur un canevas** (Collaborer, obligatoire) : **la page attend sa parole** — https://claude.ai/artifact/SLp5cJzTb7FvsKp7iEtUpS (fiche 113 § 2 ; source `~/.cache/glucose-bancs/coworking-113/`) : partager un canevas, c'est partager sa Time Machine ; **le greffier aveugle** proposé (une salle sur son relais numérote les gestes et les garde chiffrés sans les lire ; chacun rejoue le même ordre ; le dernier gagne, l'avant que le journal garde détecte le conflit, montré dans la Time Machine) ; les photos d'un participant à l'autre ; deux codes ; ses cinq questions. Dès sa parole, sans attendre le relais : les numéros des nœuds par participant (`store/ids.rs` est un compteur) et les gestes par nom | à sa parole sur la page, puis écrit |
| [~] | **Son lag au zoom** (*« pour ce qui est capital et nécessaire, il faut pouvoir optimiser »*) : REPOS-4 (fiche 109) — une image du mouvement 19-21 → 4-5 ms ; QUEUE-1 (fiche 110) — la cascade compte la fin de l'image : battements ratés 8-10 → 3 par série de crans, p90 24 → 17 ms. **Reste** : trois images par série à 18-22 ms (la fin de l'image qui varie, une texture plus chère que sa droite au premier cran) ; **remettre en question l'objectif** — « sous 10 ms au p99 » contredit la règle qui donne la période de l'écran à la cascade ; le critère qui se voit est « aucun battement raté », les 10 ms pèsent sur la latence : mesurer la latence d'un geste pendant une image de cascade avant de trancher (fiche 110 § 4). Ensuite, le tracé des flèches pour la carte graphique (2,5 ms de loin) | aucun battement raté au zoom de loin, mesuré en paires ; la latence mesurée |
| [~] | **Le cercle de l'IA, sur la Time Machine** (VISION § 8 bis ; la page : https://claude.ai/artifact/8WJTkJLvv7HvDDrJMng7CL — sa source : `~/.cache/glucose-bancs/cercle-ia/`) : la Time Machine signe chaque geste et porte des écrits (fiche 108) ; **le journal des rectifications** est écrit (RECTIF-1, fiche 110) — une couleur, une flèche retouchée, retirée ou suivie, sans seuil, avec le pourquoi (B3, exact). **Ensuite** : regarder ce que le journal récolte chez lui (pas d'apprentissage avant, sa règle) ; le pourquoi d'une flèche ; ses propres flèches, que l'IA ne juge pas mais dont elle doit *« comprendre l'importance »* ; **puis** les couleurs et l'interface de la Time Machine pour tout y voir — un écrit y tient aujourd'hui sur une ligne coupée (*« d'abord le fonctionnement, l'esthétisme et l'ergonomie après »*) | à son écran : ses rectifications lues dans la Time Machine |
| [~] | **Les flèches de Wikipédia** (VISION § 6) : **PARENTE-4 dans `main`** (fiche 108) — l'IA juge les flèches qu'un agent ou un script a posées, la souris ne rapproche que celles de sens ; jamais les siennes. Sa carte signée (`~/Documents/Glucose-essai-wikipedia-signee.glucose`, fiche 110) le montre : la souris près d'un texte n'éclaire qu'une poignée de flèches. **Ensuite** : une couleur qui dit quelque chose (le gris n'est que temporaire), partir des **mots** qui citent (les ancres de texte existent), des liens partiels, un tracé à logique de câblage. La recherche : `~/.cache/glucose-bancs/halo-106b/recherche-fleches.md` | à son écran, sur la carte signée |
| [~] | **Les membranes en carte** (fiches 105 et 107) : écrites, éprouvées, sabotées, regardées ; 11 % (*« vraiment vraiment parfait »*). Reste **l'allure de la frontière**, à sa parole (https://claude.ai/artifact/7L4zAY5U4mnebV2qvHgSK4) | sa parole sur la frontière, puis son écran |
| [~] | **Le halo d'un texte dit sa parenté** (fiche 106) : écrit, éprouvé, regardé sur la carte. Reste **son écran** ; sa couleur et ce que l'IA en apprend passent par le cercle de l'IA, plus haut | son écran |
| [ ] | **Les boutons qui manquent** (INVENTAIRE § 3), un par un, chacun conçu après une recherche (une fonction « de Tauri » se vérifie d'abord dans Tauri — « filtrer par domaine » n'y existait pas, fiche 115 § 7) : **l'export HTML d'abord : écrit, en correctif** (`~/.cache/glucose-bancs/export-html-115/export-html.patch`, fiche 115 § 7 ter — `git apply`, la suite entière, les 8 sabotages de `outils/sabotages/export-html-115.json`, `main`, son Glucose) ; puis les miroirs (`Ctrl+Maj+M`), les GIF animés et les vidéos, la provenance des images… | chacun dans `main`, saboté, regardé dans le vrai Glucose |

## 3. Ce qui attend son écran

Écrit, éprouvé, saboté — **jamais jugé par lui**. Sous Linux : `glucose > ~/sortie-glucose.txt 2>&1`, puis
on lit nous-mêmes la sortie et sa chronique (`/tmp/glucose-chronique/derniere-session.txt`, copiée
d'abord).

**Sur la carte de Wikipédia signée** (`~/Documents/Glucose-essai-wikipedia-signee.glucose`, une copie
posée par la version neuve — l'ancienne, `…-liens.glucose`, ne connaît pas les agents) :

* zoomer et dézoomer à la molette de loin : plus fluide qu'hier (REPOS-4, QUEUE-1) ;
* la souris près d'un texte : seules quelques flèches s'éclairent (PARENTE-4) ;
* colorer un texte qui a un halo de couleur, `Ctrl`+clic sur une flèche, puis la Time Machine
  (`Ctrl+H`) : sous ces gestes, l'écrit de l'IA de Glucose — ce qu'elle pensait, et pourquoi
  (RECTIF-1) ;

* le halo de la parenté (fiche 106) : au fond de « Eau », les textes de l'eau d'une même couleur, ceux
  de la chimie d'une autre, les autres au contour gris ; écrire un texte, attendre une seconde : sa
  couleur suit ; colorer un texte (la barre de couleur) : sa lueur prend la couleur, pleine ;

* les membranes en carte (fiche 105) : de loin, une couleur par région, aucune boue ; zoomer au fond
  de « Eau » : les blocs gris sombres sur la couleur de leur région, plus fluide qu'avant ; le grand
  cadre « Wikipédia » gris ;

* dézoomer, bouger la souris, déplacer la vue de loin : bien plus fluide (LETTRES-1, fiche 104 ; TITRE-3
  et les deux mémoires, fiche 108 : 17 → 7,5 ms par image) ;
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
* la palette (`Ctrl+Maj+P`), changer un raccourci ; **une flèche choisie : la palette ne propose
  plus « Couleur »**, et le clic droit montre un menu rangé par sens (APPLIQUE-1, fiche 113) ;
* **dater** : choisir un texte et une photo, `Maj+T`, taper « renais » puis choisir la Renaissance ;
  la pastille ambre à leur coin ; dézoomer ; `Ctrl+Z` ; un lien dans un texte, plus clair
  qu'avant (DATE-1, fiche 114) ;
* **la frise chronologique** : dater trois ou quatre textes à des époques différentes, relier deux
  d'entre eux par une flèche, `Maj+R` ; glisser une poignée de la fenêtre ambre — ce qui sort de la
  période pâlit, la flèche avec ; la molette sur la frise ; la croix (FRISE-1 et 2, ESTOMPE-1 et 2,
  fiches 114-115) ;
* **regarder un domaine** : donner l'eau à trois textes à des poids différents (le panneau Domaines,
  ses paliers), puis « voir » sur l'eau — ce qui n'en est pas pâlit, d'autant moins qu'il en est ;
  « voir » encore (DOMAINE-1, fiche 115) ;
* **le mode zen** : `F11` — plus que le canevas ; « ZEN · F11 » en haut à droite, ou `F11`, ou
  `Échap`, pour revenir (ZEN-1, fiche 115) ;
* un agent dans Glucose (`glucose --mcp`) : « Autoriser », une demande ; puis la Time Machine
  (`Ctrl+H`) : sous chaque geste de l'agent, son nom et ses écrits (MAIN-1, NOTE-1, fiche 108) ;
* le Storyboard, Mood board, Ordonner, l'export SVG et PNG, le mode référence.

**Au téléphone** (le sien est en **beta 8**, 18 fins anormales sur 24 sessions) : **l'ouvrir et le
mettre à jour par-dessus** — jamais désinstaller, un brouillon perdu y est peut-être encore ; une longue
session ; le texte de près ; les doigts (deux touchers annulent, trois rétablissent) ; le Storyboard au
doigt ; la pile des barres du bas (BAS-1 : gêne-t-elle ?).

**Quand une machine Windows existera** : la bande qui remplace la barre blanche (BARRE-1), éteindre le PC
Glucose ouvert (FIN-SESSION-1), un dépôt posé sous le curseur.

## 4. Ce qui attend sa parole

* **Le co-working** (fiche 113, https://claude.ai/artifact/SLp5cJzTb7FvsKp7iEtUpS) : le greffier
  aveugle (A3) ; le dernier gagne et le conflit montré (la revue des conflits en option) ; les
  photos d'un participant à l'autre (B1) ; deux codes et son identité comme dans Tauri ; la porte
  publique d'abord ?

* **La porte publique** (fiche 111, https://claude.ai/artifact/36hpNTPEH2mCEENhKjyvzF) : le relais sur
  son compte Cloudflare (A2) ; l'accord gardé jusqu'à ce qu'il le retire (B1) ; le document ouvert
  seulement (C1) ; quelle IA d'abord et avec quel compte ; la porte avant le co-working.

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
* [ ] Les miroirs (`Ctrl+Maj+M`) ; les GIF animés et
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
