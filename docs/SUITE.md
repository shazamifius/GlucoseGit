# La suite

> **Au 19/11/2026, 18 h 45** (fiches 108-126). **Ce qu'on fait ensuite, dans l'ordre** — une ligne par chantier ouvert ;
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

## 2. Maintenant, dans l'ordre — ses paroles du 12 et du 19/11

**Sa parole du 12/11 au soir** (VISION § 3) : des versions qu'il teste en conditions réelles, puis
**les boutons qui manquent — d'abord le co-working et les connexions MCP avec des IA publiques**. Les
deux sont faits ; ce qui suit les termine, puis reprend le reste du tableau.

| | chantier | fini quand |
|---|---|---|
| [~] | **Le compte Cloudflare de Glucose** (sa parole du 19/11 : *« fermenislamber […] c'est un autre projet ; il faudrait vraiment switch ça »*) : le plan, fiche 123 § 7 ; **la question est dans son Glucose** (son clic *Create Account*, le nom — `glucose-canevas` proposé, les serveurs nommés `porte` et `boite-noire`). Ensuite, **le serveur avant le client** : la boîte noire et la porte sur le compte neuf, l'ancienne adresse de la boîte noire qui transmet (`~/.cache/glucose-bancs/session-123/outils/ancienne-adresse/`), la base recopiée (`INSERT OR IGNORE`), `ADRESSE` (`telemetrie.rs`), `RELAIS` (`agents/publique.rs`), `verifier_serveur.py` et les documents changés ; l'ancienne porte reste ouverte pour les versions installées ; l'identifiant du compte par `CLOUDFLARE_ACCOUNT_ID`, jamais dans un fichier | les deux serveurs prouvés en ligne sur le compte neuf ; une session de télémétrie d'une version neuve reçue ; ses connecteurs réajoutés |
| [ ] | **Les sabotages pas encore joués** (avec le témoin, deux fils, un fichier par commande, MÉTHODE § 5) : le dernier de `liste-123` (la puce en vecteurs — il compile les épreuves d'intégration), `detourage-121` (25 — l'optimisation du dépôt, sa propre cible), `veille-126` (2 — la borne a refusé, sa machine pleine). Joués à la fiche 126 : sept fichiers et un échantillon de 24 anciens, **tous tombent** (trois gardes aveugles réparées). Un sabotage qui passe : chercher l'entrée qui le ferait tomber, écrire l'épreuve | chacun TOMBE |
| [ ] | **Son Glucose installé à la 2.0.4-beta.4** (publiée le 19/11 à 19 h 15, son clic) : `bash outils/mettre_a_jour.sh` — la compilation de Nix est hors de la borne, huit à la fois : **sous chien de garde, et seulement quand sa machine laisse de la mémoire** (le 19/11 au soir, Brave et VS Code n'en laissaient que 3 Go). **Jamais `gh release delete` sur une version sans l'avoir lue (`gh release view … --json isDraft`)** | son Glucose dit 2.0.4-beta.4 |
| [~] | **Les émojis en couleur** (EMOJI-2 et 3, fiches 125-126, `decisions/18`) : **faits, publiés dans la beta.4** — Twemoji 17 en COLRv0, et les 34 signes d'encre dans la palette du fond sombre ; sa parole sur les ✔ : *« non, c'est parfait comme ça ! »*. **Reste** : son écran | à son écran |
| [~] | **Les rangées d'un tableau coupé** (TABLE-4, fiche 125) : **fait** — un filet entre deux rangées, de l'encre du filet de l'en-tête à 70 % (Primer, `borderColor.muted`), sur les trois dessins ; 7 sabotages sur 7, regardé au pilote et sur la scène témoin. **Reste** : son écran | à son écran |
| [ ] | **Les tiroirs qui défilent quand la hauteur manque** — la vraie fin de TIROIR-3 (fiche 122 § 7 : Storyboard et Domaines se croisent de 10 pixels à 1440 × 900 avec huit domaines). **Conçu à la fiche 126, pas écrit** : (1) **le rangement** (`dock/rangement.rs`, sorti de `dock.rs`, à 595 lignes) — le « first fit » de chaque côté, puis, tant que les colonnes ne tiennent pas dans la largeur, la dernière colonne du côté qui en a le plus se fond dans la moins pleine du sien ; une colonne trop haute partage sa hauteur par **répartition équitable** (max-min : seuls les plus hauts cèdent, jusqu'à une hauteur commune — le minuteur reste entier) ; (2) **le défilement, une seule loi pour les sept** — un panneau se range dans un cadre virtuel à sa hauteur naturelle, décalé de son défilement (`PanelLayoutBox::contenu`), et se dessine dans un tampon de son contenu **visible** (la poignée intacte, TIROIR-1) : le dessin et le clic lisent le même cadre (L4) ; `panneau_sous` ne donne le contenu qu'au pointeur qui est dedans ; la molette défile le panneau qui cache quelque chose ; une marque d'un trait dit ce qui reste. Le défilement propre à Domaines (`decalage`, `hidden`, DOM-UI-2) deviendrait du code mort : à retirer avec ses épreuves. **Les deux moitiés partent ensemble** : un tiroir rétréci qui ne défile pas déborderait sur sa poignée | aucun tiroir l'un sur l'autre, au pire cas ; regardé au pilote |
| [ ] | **Distinguer une veille d'un gel** dans la télémétrie (l'image de 415 s, fiche 123 § 6). **La loi, corrigée (fiche 126 § 2)** : seul l'`Instant` de Windows (QueryPerformanceCounter) compte la veille — sous Linux et Android il lit `CLOCK_MONOTONIC`, qui ne la compte pas, et retrancher « BOOTTIME moins MONOTONIC » y donnerait une durée négative. **Mesurer la durée d'une image à une horloge qui ne compte pas la veille** — sous Windows `QueryUnbiasedInterruptTimePrecise`, ailleurs `Instant` — là où la boucle remplit `Instantane::duree_us` ; aucune ligne neuve, donc rien à déployer avant. Compter la veille à part serait une ligne neuve : le serveur d'abord | une session réelle sous Windows, mise en veille, sans image géante |
| [~] | **Le détourage, la suite** (fiche 121, `decisions/17`) : **regardé dans le vrai Glucose** (le 18/11 sur la boule, le 19/11 sur une pomme photographiée sur fond blanc, fiche 126 § 4 bis : le sujet entier, l'ombre partie) — **un liseré clair de huit à dix pixels de l'image reste au bord** (le premier plan garde du blanc du fond : DETOURER-2 ne le retire pas jusqu'au bout) et de fines traces des rayures restent au-dessus de la boule : à mesurer puis corriger ; ISNet selon la machine (sa licence d'abord) ; la voie de la carte graphique ; un clic pour désigner l'objet | regardé, puis à son écran |
| [~] | **La porte, la suite** : l'adresse de ce Glucose reconnaissable (dire la machine dans la question de la porte — Mistral appelait celle d'un autre de ses Glucose) ; B2 en option (redemander l'accord à chaque lancement) ; une icône de Glucose pour le relais ; le téléphone ; la « server card » que Mistral demande (`/.well-known/mcp/server-card/…`, 404) — **pas mûre** au 19/11 : la SEP-2127 est passée dans un dépôt *experimental*, et les sources donnent cinq chemins (fiche 126 § 5) | chacun dans `main`, saboté |
| [~] | **Son lag, à son écran** (fiche 112) : le rythme n'était pas la cause (le « 32 % » était le compteur du tempo) ; **PARU-1** dit désormais ce que l'écran montre, et `outils/pilote/essai_ecran.sh` mesure ses vrais gestes (pavé virtuel). **Ensuite** : (1) **le calage sur le battement** — envoyer chaque image « au prochain battement moins une avance » efface la gigue d'un battement (prototype : `~/.cache/glucose-bancs/rythme-112/calage-et-sonde-prototype.patch`) ; l'avance n'est pas la bonne grandeur : l'image suivante de Glucose occupe la carte graphique pendant que niri compose — le levier est de **dessiner juste à temps** (fiche 112 § 5), puis mesurer en paires à son écran ; (2) la latence de deux battements de l'envoi à l'écran, sur une carte graphique partagée avec niri ; (3) **ses vraies sessions lues par PARU-1** — le 14/11 à 1 h 45 (fiche 116 § 1) : 19 % de battements sautés alors qu'une image était due, 46 ms de l'envoi à l'écran en médiane, 93 au p90 ; **le 14/11 à 4 h 37 sur sa carte de Wikipédia, sur batterie** (fiche 117 § 1) : **46,5 % de battements sautés**, 39 / 55 / 78 ms, « ornements » jusqu'à 33 ms, « écouter la main » jusqu'à 72 ms d'un coup — à reprendre là | à son écran, PARU-1 : la gigue nulle, aucun battement sauté de plus qu'aujourd'hui, la latence mesurée |
| [~] | **Le co-working sur un canevas** (CLIENT-1, fiche 117) : **fait** — deux vrais Glucose dans le même canevas, chaque geste vu chez l'autre, un retardataire qui rattrape tout, un vrai conflit montré des deux côtés, `Ctrl+Z` par main, les curseurs ; la salle **en ligne** (fiche 117 § 5). **Les photos d'un participant à l'autre** (B1) : **faites** (fiche 119 — ce que je pose part avec mon geste, ce qui manque se demande en entrant ; prouvées par deux vrais Glucose à travers le relais en ligne, redéployé ; la 2.0.4-beta.2 en brouillon les porte). **Reste** : (1) l'alerte « des photos attendent quelqu'un qui les a » (sa question 18) ; (2) **le partage qui survit à un relancement** (aujourd'hui on rejoint de nouveau par le code), et un instantané dans la salle pour qu'un long canevas se rattrape vite (Secsync) ; (3) son nom changé dans l'interface ; (4) le conflit **par propriété** (fusion à trois par le format à étiquettes) ; (5) le téléphone ; (6) **son essai** — il demande une version publiée sur ses deux machines (la 2.0.4-beta.2, son geste) | à son écran : son PC et son téléphone, ou deux Glucose sur son VivoBook, dans le même canevas |
| [~] | **Son lag au zoom** (*« pour ce qui est capital et nécessaire, il faut pouvoir optimiser »*) : REPOS-4 (fiche 109) — une image du mouvement 19-21 → 4-5 ms ; QUEUE-1 (fiche 110) — la cascade compte la fin de l'image : battements ratés 8-10 → 3 par série de crans, p90 24 → 17 ms. **Reste** : trois images par série à 18-22 ms (la fin de l'image qui varie, une texture plus chère que sa droite au premier cran) ; **remettre en question l'objectif** — « sous 10 ms au p99 » contredit la règle qui donne la période de l'écran à la cascade ; le critère qui se voit est « aucun battement raté », les 10 ms pèsent sur la latence : mesurer la latence d'un geste pendant une image de cascade avant de trancher (fiche 110 § 4). Ensuite, le tracé des flèches pour la carte graphique (2,5 ms de loin) | aucun battement raté au zoom de loin, mesuré en paires ; la latence mesurée |
| [~] | **Le cercle de l'IA, sur la Time Machine** (VISION § 8 bis ; la page : https://claude.ai/artifact/8WJTkJLvv7HvDDrJMng7CL — sa source : `~/.cache/glucose-bancs/cercle-ia/`) : la Time Machine signe chaque geste et porte des écrits (fiche 108) ; **le journal des rectifications** est écrit (RECTIF-1, fiche 110) — une couleur, une flèche retouchée, retirée ou suivie, sans seuil, avec le pourquoi (B3, exact). **Ensuite** : regarder ce que le journal récolte chez lui (pas d'apprentissage avant, sa règle) ; le pourquoi d'une flèche ; ses propres flèches, que l'IA ne juge pas mais dont elle doit *« comprendre l'importance »* ; **puis** les couleurs et l'interface de la Time Machine pour tout y voir — un écrit y tient aujourd'hui sur une ligne coupée (*« d'abord le fonctionnement, l'esthétisme et l'ergonomie après »*) | à son écran : ses rectifications lues dans la Time Machine |
| [~] | **Les flèches de Wikipédia** (VISION § 6) : **PARENTE-4 dans `main`** (fiche 108) — l'IA juge les flèches qu'un agent ou un script a posées, la souris ne rapproche que celles de sens ; jamais les siennes. Sa carte signée (`~/Documents/Glucose-essai-wikipedia-signee.glucose`, fiche 110) le montre : la souris près d'un texte n'éclaire qu'une poignée de flèches. **Ensuite** : une couleur qui dit quelque chose (le gris n'est que temporaire), partir des **mots** qui citent (les ancres de texte existent), des liens partiels, un tracé à logique de câblage. La recherche : `~/.cache/glucose-bancs/halo-106b/recherche-fleches.md` | à son écran, sur la carte signée |
| [~] | **Les membranes en carte** (fiches 105 et 107) : écrites, éprouvées, sabotées, regardées ; 11 % (*« vraiment vraiment parfait »*). Reste **l'allure de la frontière**, à sa parole (https://claude.ai/artifact/7L4zAY5U4mnebV2qvHgSK4) | sa parole sur la frontière, puis son écran |
| [~] | **Le halo d'un texte dit sa parenté** (fiche 106) : écrit, éprouvé, regardé sur la carte. Reste **son écran** ; sa couleur et ce que l'IA en apprend passent par le cercle de l'IA, plus haut | son écran |
| [ ] | **Les boutons qui manquent** (INVENTAIRE § 3), un par un, chacun conçu après une recherche (une fonction « de Tauri » se vérifie d'abord dans Tauri — « filtrer par domaine » n'y existait pas, fiche 115 § 7) : ~~l'export HTML~~ — **dans `main`** (fiche 117 ; reste l'iPad, Safari jamais essayé) ; d'abord les miroirs (`Ctrl+Maj+M`), les GIF animés et les vidéos, la provenance des images… | chacun dans `main`, saboté, regardé dans le vrai Glucose |

## 3. Ce qui attend son écran

**Avec la 2.0.4-beta.4, quand elle sera publiée** (fiches 120 à 126 — la beta.3, publiée, en porte déjà
une partie : le texte net, les modes de la fenêtre, `F2` ; regardé par nous au pilote, jamais par lui) : dans son document « ce qui attend ma parole », l'onglet « Textes optimisés » — les tableaux de
Claude.ai dans leur nœud, les listes décalées, les textes dans leurs membranes, les émojis en couleur — les ✔ clairs dans un nœud, foncés sur un post-it ; ouvrir les sept tiroirs à
la fois (chacun du côté de son bouton, collé au bord, plat) ; « Agrandir l'interface » deux fois au menu
du vide, puis « Réduire » ; une photo sur fond blanc, clic droit, « Détourer : la forme principale », puis
`Ctrl+Z` ; le texte de la bande et des icônes, de près (nets ? trop fins ?) ; `F11`, `Ctrl+Maj+A`,
`Maj+F11` ; un dossier, `F2`. **À l'école, sous Windows** : la même chose, et « Barre de Windows » au
menu du vide, deux fois.

**Sa capture du 17/11** (fiche 118) — **à l'école, sous Windows, avec la 2.0.4-beta.2** : chaque bouton de
la bande, surtout ceux de droite (Collaborer, Exporter, Plugins, Mood board, Domaines) ; un clic entre
deux boutons ne sélectionne rien. **Partout** : ouvrir tous les tiroirs à la fois (aucun sur un autre, le
Storyboard et la Time Machine à la hauteur de ce qu'ils montrent, « + Nouveau domaine » au-dessus de la
poignée) ; entrer dans un dossier, cliquer le premier nom du fil d'Ariane ; choisir une flèche, tirer
un de ses bouts sur un autre nœud, puis dans le vide ; clic droit sur un point orange, « Retirer ce
coude » ; « Redresser la flèche » ; une photo choisie, `Alt+Maj+H` puis `Alt+Maj+V`, et `Ctrl+Z`. **Le co-working
et ses photos** (fiche 119) : deux Glucose dans le même canevas (son PC et son téléphone, ou deux
fenêtres) ; une photo collée chez l'un apparaît chez l'autre, et un canevas plein de photos rejoint par
l'autre les montre toutes.

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

**La porte aux IA publiques** (fiche 116), **avec ton compte Claude.ai** : Collaborer → « Ouvrir la
porte » → « Copier l'adresse » ; dans Claude.ai : Personnaliser → Connecteurs → Ajouter un
connecteur personnalisé, coller l'adresse ; la page dit un code et « va dans Glucose » ; dans
Glucose, le même code, « Lire et écrire » ; puis demander à Claude de lire le tableau et d'y poser
un texte — il apparaît, signé « Claude (claude.ai) » dans la Time Machine ; Collaborer → « Les IA qui
ont ta porte… » pour la retirer.

**Le canevas partagé** (fiche 117), **sur son VivoBook seul** : dans son Glucose, Collaborer → « Partager
ce canevas » (le code est copié) ; puis, dans un terminal, `bash ~/Projets/GlucoseGit/outils/porte/essai/un_second_glucose.sh`
— une seconde fenêtre, « Bob », dans un dossier jetable ; dans celle-ci, Collaborer → « Rejoindre (le code
copié) ». Poser un texte dans l'un, le voir dans l'autre ; bouger la souris : son curseur, nommé, dans
l'autre ; colorer le même texte des deux côtés presque en même temps ; `Ctrl+Z` dans l'un ne défait que
ses gestes. **Entre son PC et son téléphone** : il faut une version publiée (la 2.0.4-beta.2, son geste).

**Au téléphone** (le sien est en **2.0.4-beta.1** depuis le 16/11 — mis à jour par-dessus) : une longue
session ; le texte de près ; les doigts (deux touchers annulent, trois rétablissent) ; le Storyboard au
doigt ; la pile des barres du bas (BAS-1 : gêne-t-elle ?).

**Quand une machine Windows existera** : la bande qui remplace la barre blanche (BARRE-1), éteindre le PC
Glucose ouvert (FIN-SESSION-1), un dépôt posé sous le curseur.

## 4. Ce qui attend sa parole

**Ses questions vivent dans son Glucose** depuis le 19/11 (sa parole, VISION § 9) : son document
« ce qui attend ma parole » (`~/Downloads/ce qui attend ma parole.glucose`), lu et écrit par sa porte
(`outils/porte/claude_code.mjs`) ; ses réponses vont dans VISION, et sortent d'ici. **Ouverte** : le
compte Cloudflare de Glucose (son clic, et le nom — posée le 19/11 à 14 h, à droite de la porte).
Restent, de [l'ancienne fiche de questions](https://claude.ai/artifact/KwCMGG6Keymuzr5V4DJ4gN) (fiche
118 § 5), celles qu'il n'a pas encore recopiées dans Glucose :

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
  défilement au bord** (« moche, pas fluide », plus tard) ; **les rideaux**, **les membranes de Mary**, « optimiser » dans la Time Machine.
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

La **2.0.4-beta.2** (son clic, le 18/11 à 11 h 49) est la dernière ; avant elle, la 2.0.4-beta.1 (13/11).
Chaque version a sa fiche (les notes : `outils/publication/`).
