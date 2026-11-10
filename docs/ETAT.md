# Où en est Glucose

> **Au 10/11/2026** (fiches 73 à 102). L'état **vérifié** du projet, en une lecture. Il se réécrit à chaque
> fin de session qui change quelque chose d'important ; l'histoire, session par session, vit dans
> le [carnet](carnet/00-INDEX.md) (fiches 51 à 84 pour la V2). La direction : [VISION](VISION.md).
> La suite : [SUITE](SUITE.md). Chaque fonction et son bouton : [INVENTAIRE](INVENTAIRE.md).

---

## 1. En une page

* **Les liens : de loin une trace, `Ctrl`+clic les suit, une fenêtre les amène** (fiche 102, 10/11 au soir ; sa parole du 9/11, sa seule exception de LOD ; ses quatre questions de la fiche 99 toujours sans réponse — bâti sur des défauts argumentés, § 7). **Le squelette porte ses vrais liens** (`GLUCOSE_LIENS=1`, 1 128 flèches de Wikipédia entre les cent articles vitaux). **LIEN-1** : plus loin que la taille réelle (une unité par pixel logique), un lien rétrécit avec le monde **et pâlit d'autant** — deux lois d'abord, rejetées chacune à l'image (une pelote) : rétrécir seulement gardait la densité d'encre de la lecture ; aucune constante ; ce qu'on regarde reste plein (choisi, survolé, et dans la fenêtre) ; le survol ne redessine que si un lien change. **LIEN-2** : `Ctrl`+clic sur une flèche vole jusqu'au bout où l'on n'est pas, `Alt+Z` ramène ; « Suivre le lien » au menu, `lien.suivre`. **LIEN-3, la fenêtre des liens** (le Bring & Go de Moscovich et al., CHI 2009, lu dans l'article) : `B`, le menu, la palette ; chaque voisin glisse sur la demi-droite de sa vraie place, au plus près sans rien chevaucher (exact, `glucose_core::amener`) ; un tableau de copies **à côté du document** (rien ne l'atteint, rien ne l'enregistre) ; chacun dans une membrane titrée de sa provenance, à la couleur de sa région, sans le début que tous partagent ; un clic y mène, `Maj`+clic la refait, glisser déplace la vue, `Échap` et tout autre geste la ferment. **Regardé au pilote** : `Ctrl+F`, `B`, les articles cités par « Eau » autour de lui, un clic vole à « Transport », `Alt+Z` ramène. **Le sens des couleurs des membranes, v2** : la proximité par les liens de la carte (Milne et Witten) — 0,31 à 0,71 entre thèmes contre 0,05 à 0,14 par les mots ; image dans `squelette-sens2/images/`. **3 073 épreuves, sabotages 68 sur 68** (le cliquet des 80 lignes a fait extraire la porte des commandes). **Pas vu à son écran.**
* **Le bas de l'écran — rien ne sort, rien ne couvre la minimap** (fiche 101, 10/11). À la taille de son téléphone (le pilote sait la prendre, `PILOTE_ECRAN`), la barre de couleur sortait de l'écran (« ouleur », la roue coupée) et les barres couvraient la minimap ; une propriété a trouvé la barre des flèches à 699 points. **BAS-1** (`ui/bas.rs`) : une barre passe à la ligne comme un texte — la barre des flèches groupe par groupe —, et monte au-dessus de la minimap, qui ne bouge jamais ; un bouton à deux états a la largeur de son plus long mot (basculer ne déplace plus rien sous la main) ; `ecran_visible` retire aussi la navigation de droite. Éprouvé de 320 à 2 560 points aux échelles 1 à 2,75, regardé au moto g54, au Redmi 9 et au bureau (inchangé). **La CI est verte** pour la fiche 100 (dix tâches sur dix : le pont Windows compile).
* **Ce que niri a vu : six défauts sous Linux, réparés** (fiche 100, 9/11 au soir, sa capture de six défauts). **Le pont Wayland** (`plateforme/selection_wayland`) : Glucose ne parlait au presse-papiers que par X11, que `xwayland-satellite` ne relaie qu'avec le clavier, et `winit` ne transmet aucun glisser-déposer sous Wayland — copier un texte vers Discord, coller une image, glisser une photo dans les deux sens marchent désormais, éprouvés par `outils/pilote/essai_wayland.sh` (cinq gestes jugés seuls ; l'image reçue identique au pixel près). **La boîte montrée** : la lueur d'un texte suit la hauteur que pousse la saisie — le contour ne se décroche plus. **LOT-2 et PHOTO-2** : la minimap défaisait le lot des photos, une image sur deux sans photo au clic molette — revu, plus aucune. **TEMPO-3** : le tempo ne monte plus au-dessus du plancher de la charte (à 60 Hz, il passait à 30 images par seconde pour des images de 10 ms — sa propre chronique le montrait) ; **SOURIS-1** : la souris avance la vue par pixels entiers, et une translation entière montre l'image du repos décalée — aucune texture à refaire, le texte net pendant le geste ; sur une copie de sa carte, un balayage par image pendant tout le déplacement (130 images au lieu de 61 à 81). **Le README** réécrit, chaque chiffre vérifié (269 caisses sous Linux, dites ; GTK 3 n'était plus vrai ; Android publié), et le guide corrigé. **Vu sous un vrai niri réglé comme son écran** (le 10/11, imbriqué dans le banc sans écran) : `essai_wayland.sh`, six contrôles sur six, la photo déposée centrée sous le curseur. **Reste** : un nœud ne se glisse pas vers une fenêtre posée par-dessus Glucose (sa parole) ; une fenêtre très étroite fait chevaucher la barre de couleur et la minimap. **Sabotages : 15 sur 15**, 3 032 épreuves. **Tout éprouvé sous sway sans écran : rien sous niri, rien à son écran** ; le pont Windows (`ScreenToClient`) n'est compilé que par la CI.
* **Ce qu'on écrit se voit ; la couleur d'un texte se voit ; `Alt+Z` ; la roue de couleur** (fiche 99, 9/11 après-midi et soir, sa parole du même jour). **Un banc qui clique et qui tape** (`outils/pilote/` : un pointeur virtuel Wayland et `wtype` dans un compositeur invisible) a reproduit ce qu'il a vu : sur un texte et un post-it, la frappe se voyait ; **sur le titre d'une membrane, jamais** — il se dessinait depuis le document. **SAISIE-1** : le titre montre la frappe, son curseur (une seule loi du curseur pour le texte, le post-it et le titre), sa sélection ; un titre vide qu'on écrit a sa place. **SAISIE-2** : `Ctrl+G` sur ce qu'on voit en haut de l'écran posait la place du titre **sous la barre des onglets** — la vue glisse du strict nécessaire. **COULEUR-3** : la couleur d'un texte ne touchait qu'une brume à 3 % ; elle teinte désormais son papier **à sa clarté perçue** (Oklch, le bord exact de l'écran) — les lettres se lisent pareil. **RETOUR-1** : `Alt+Z` revient au lieu d'où la caméra est partie (F, tout cadrer, signets, `Ctrl+F`, Focus, minimap), `Alt+Maj+Z` y repart, « Vue précédente » au menu du vide ; l'histoire vit dans le magasin. **ROUE-1** : la roue de couleur à la Blender, **dans Oklch** (la case au disque, au bout de la barre de couleur ; `Ctrl` cale la teinte, `Maj` affine, la molette change la clarté d'une différence à peine visible, `Retour arrière`, `Échap` ; le code qu'on tape, posé exactement ; la pipette, qui prend la couleur d'un nœud ou la moyenne d'une photo) ; un seul geste. Tout regardé dans le vrai programme hors écran. **Le sens des couleurs des membranes, en image** (§ 12) : un agent colore le squelette — la teinte par la ressemblance des régions (sériation circulaire exacte), la clarté par leur lisibilité ; l'opacité mesurée, pas peinte. **Proposés en prose, attendent sa parole** : le sens de la couleur des textes (A : le domaine dont il parle — mon avis ; B : son rôle dans un raisonnement ; C : sa sûreté) ; les liens (le Bring & Go de Moscovich et al., CHI 2009 : la transparence suit la lisibilité, le clic cadre, le « show » amène les nœuds liés). **Suite : 3 008 épreuves passées au dernier passage, et les trois tombées réparées puis rejouées (deux empreintes témoins regardées — la barre et son petit disque —, le cliquet du couplage : la pipette passe par le noyau) ; sabotages : 52 sur 52** (SAISIE 10, COULEUR-3 7 — rejoués après le bord de l'écran —, RETOUR-1 11, ROUE-1 24). **Un sabotage aveugle a fait trouver un vrai défaut** : le gamut n'est pas convexe dans Oklab, et la roue rendait `(0, 0, 85)` en `(0, 9, 76)` — le bord se cherche désormais depuis le dehors, l'aller-retour est exact (fiche 99 § 8 bis). **Pas vu à son écran.**
* **Glucose se pose sur une carte de Wikipédia, et six chantiers en autonomie** (fiche 98, 9/11, il était épuisé : rien ne lui a été demandé). **REPOS-2** (sa parole : *« yep »*) : la fiche 97 disait « ne se pose jamais » — **faux** : 120 s de banc montrent une convergence ruineuse en 30 s (une teinte et une texture par image complète de 22 ms, la peinture des membranes mangeant tout le budget) ; le cache des teintes paie ses dettes au repos sans image, une seule image les montre, les 352 textures se posent en une image : **293 → 26 images en 30 s de repos, un cœur 21,5 % → 2,3 %** (paires entrelacées), 7 sabotages sur 7. **AGENT-8** : un agent qui recolorait une membrane effaçait son titre, qui réécrivait un texte effaçait ce que l'humain y avait mis — une mise à jour ne change plus que ce qu'elle nomme (3 sur 3). **B, C, D présentées** en prose (fiche 98 § 3), **D en image** (`~/.cache/glucose-bancs/squelette-d/images/`) : Tree Colors par un agent — les neuf thèmes se reconnaissent d'un regard, mais la roue s'épuise dès le deuxième niveau ; trois suites à son choix. **OPERATEUR-1** : ET, OU, MAIS, PARCE QUE ont enfin un geste (`Alt+1…4`, `Alt+0`, la palette, le menu d'un post-it ; 7 sur 7). **OPERATEUR-2** : une IA ne voyait pas la pilule que l'humain voit — elle la lit et la pose (`operateur=parce-que` ; 6 sur 6). **COULEUR-1** : la barre de couleur — huit teintes et « Par défaut » pour les textes, post-it, membranes et dossiers choisis, une commande par teinte ; un opérateur ne se colore pas (la capture l'a montré) ; 12 sur 12. **COULEUR-2** : une IA lit la couleur que l'humain voit, et colore un post-it comme lui (son papier). **GROUPER-1** : `Ctrl+G` (et le menu, la palette) entoure la sélection d'une membrane neuve, dans l'ancêtre commun, les villes gardant leur pays — le geste qui fait une région ; une IA aussi (`glucose_grouper`). Tout regardé dans le vrai programme hors écran. **2 967 épreuves, 0 tombée ; 47 sabotages, 47 tombés.** **GRIS-3** (les traits fins hors des nœuds) examiné, décrit pour la suite (fiche 98 § 7). **Pas vu à son écran.**
* **Le squelette de Wikipédia, posé et regardé** (fiche 97) : les cent articles vitaux en neuf thèmes, « Eau » et « Hydrogène » entiers dedans, 542 nœuds rangés en colonnes, par les seules commandes, dans le vrai Glucose sans fenêtre (`outils/wikipedia/essai_squelette.sh`). **La première image montrait un arbre faux** : un article qui grandit recouvrait ses voisins, et ce qu'un agent posait « dans » lui tombait chez un voisin plus petit. **MEMB-8** : ce qu'un agent pose dans une membrane lui appartient ; deux refus neufs (pas une membrane, un cycle). Reposé : chaque section dans son article. **Jugé** : ça se lit comme une carte ; « Sciences et médecine » écrase les autres thèmes (deux articles entiers contre cent résumés) ; des vides sous les blocs géants ; un seul bleu. Images et document : `~/.cache/glucose-bancs/squelette-wikipedia/`. **Dans le vrai programme** (fiche 97 § 4 bis) : GRIS-1 avait fait passer l'image médiane de 23 à 33 ms — non par le gris, mais par le fond lu sous chaque nœud dans chaque membrane (**FOND-1**, réparé : 23 ms) ; la loi des membranes n'est pas exacte sous le pixel (**RECT-1**, un peintre exact des rectangles). **Défaut ancien, le plus grave** : sur cette carte, Glucose ne se pose jamais (« la netteté revient », 99 % des images, déjà vrai avant la session) — un cœur tourne en permanence ; **sa parole du 9/11 : « yep », le prochain chantier** (REPOS-2). Le déséquilibre : garder la loi, *« je crois bien »* ; B, C, D : *« prochaine session »*. **Pas vu à son écran.**
* **Le gris des lignes, le Focus par un geste choisi, et rien ne se tait de loin** (fiche 96, 8/11 au soir, sa parole : *« le mode Focus, je préférais que ce soit un geste choisi »*, *« carrément d'accord pour le gris des lignes »*). **FOCUS-1** : « Entrer dans la membrane » (menu d'une membrane, palette) ; `Échap`, « Sortir de la membrane » ou dézoomer en sortent ; zoomer n'y fait plus entrer. **GRIS-1** : de loin, un nœud de texte et un post-it montrent le gris de chaque ligne — **mesuré : la couleur de loin vaut celle de près à 0,1 niveau** —, avec ce qui n'est pas une lettre (plaque de code, trait, citation, puces, passages qui brillent), peint à la couverture exacte. **GRIS-2** : neuf dessins se taisaient encore de loin (noms des zones et des dossiers, étiquettes des flèches, mots des opérateurs, sigles des domaines, cases, prédicats, deux lueurs) — la loi vit à un seul endroit, la typographie : une lettre de moins de 2,8 pixels montre le gris de son encre ; l'export SVG garde ses lettres. **COLONNES-2** : « Disposer en colonnes » envoyait parfois le dernier bloc seul dans une colonne de trop (un arrondi, depuis la fiche 92) — réparé, et l'épreuve aveugle de la fiche 95 voit enfin des coupes. **Pas vu à son écran.** **Reste**, dit : les titres des membranes ne suivent pas encore la loi du gris ; ce qui est plus fin qu'un pixel hors des nœuds de texte reste trop lourd de loin.
* **Le nom d'une région à la taille de la région** (fiche 95, 8/11 au soir, sa parole : *« pas de LOD, pas de simplification »*, *« revoir les membranes pour qu'elles soient plus intéressantes »*) : **TITRE-1** — chaque membrane consacre à son nom la même part de sa surface qu'une membrane neuve (`s = √(A/A₀)`, jamais moins que 1 : rien de petit ne change) ; **aucun titre ne se cache** à aucune distance ; un nom tient dans la largeur de sa région ; l'encre d'une membrane (son titre compris) pour le culling, le cadrage, l'export, le vol, la sélection et le glisser ; la bande de clic grandit ; « Disposer en colonnes » réserve la place. **Regardé** : « Eau » se lit comme une carte, ses huit sections et leurs sous-sections. **CHANTIER-2** : depuis la fiche 86 (betas 8 et 9), un cache des teintes en retard faisait redessiner Glucose à vide, jusqu'à 128 images par seconde tableau immobile — réparé. **TITRE-2** : un texte détouré se compose en deux passes (le halo, `1 − Π(1 − cₖ)`), plus neuf. **Reste**, dit honnêtement : pendant un zoom sur une très grande région, l'image médiane ~19 ms contre ~9 pour `main` (fiche 95 § 8, deux routes chiffrées). **Sa parole du soir** : *le mode Focus, un geste choisi* ; *d'accord pour le gris des lignes* — **faits depuis** (fiche 96). Sabotages de la fiche 95 : 24 sur 25 ; l'aveugle (la partition) tombe depuis la fiche 96, et son épreuve qui tombait sur le code juste avait trouvé un vrai défaut (COLONNES-2). **Les propositions B à E attendent sa parole** (fiche 95 § 4).
* **Comprendre Wikipédia d'un regard** (fiche 94, 8/11, sa parole : *« comment le cerveau humain serait capable de comprendre Wikipédia tout entier sur un canva 2D sans que ce soit trop brouillon »*) : la recherche (quatre morceaux en mémoire de travail, les articles vitaux par niveaux de dix, la hiérarchie des noms des cartes, la mémoire spatiale) donne quatre principes, et tranche ses deux questions. **DOM-6 est fait** : un domaine posé sur une membrane vaut pour ce qu'elle contient, à la lecture, sans rien écrire sur les nœuds — le sommaire de « Eau » passe de 135 376 à 70 287 caractères ; 8 sabotages sur 8. **Reste** : les titres des membranes à la taille de leur région, un squelette de Wikipédia. **Sa parole du 8/11 au soir** (VISION) : *pas de LOD, pas de simplification possible* ; *les membranes peuvent être revues pour être plus intéressantes*.
* **Le pincement du pavé sous Linux** (fiche 93, 8/11, sa demande) : il ne zoomait pas du tout — `winit` 0.30 ne le transmet que sous macOS. Un pavé natif Wayland (`pointer-gestures`, niri le donne) remplit désormais le même contrat que *Direct Manipulation* sous Windows : le zoom et le déplacement des doigts ensemble, en une similitude. Sous macOS, `PinchGesture`, que Glucose n'écoutait pas non plus. Éprouvé, 5 sabotages sur 5, dix vertes sur GitHub ; **vu à son écran le 8/11 : « ça fonctionne parfaitement »**. Sous X11, rien encore.
* **Disposer en colonnes** (fiche 92, 8/11) : une membrane range ce qu'elle porte — sous-membranes comprises — en colonnes de journal, dans l'ordre de lecture, à la proportion de l'écran ; un seul `Ctrl+Z`. Au menu d'une membrane, à la palette, pour un agent (`glucose_disposer`) et au terminal (`glucose disposer eau`). L'optimum exact, prouvé contre la force brute ; **« Eau », 251 nœuds, une commande de 8 ms : l'article tient dans un écran**. 2 886 épreuves, clippy et les cliquets à zéro ; sabotages : 9 sur 9 (`colonnes-92.json`). **Pas vu à son écran** ; les titres des membranes, illisibles de loin, attendent sa parole.
* **Un article de Wikipédia posé par les commandes** (fiche 91, 8/11) : « Eau », 8 680 mots, 259 commandes, 3 s. Il a trouvé **MEMB-7** (une membrane qui grandit ne faisait pas grandir la sienne : l'article à trois niveaux s'emboîtait sur huit) — réparé, éprouvé, saboté, rejoué ; donné à l'agent **un plan** (`glucose_lire niveau=plan` : les membranes seules, 12 000 caractères au lieu de 140 000) et des coordonnées courtes sans perte (COORD-1) ; 2 877 épreuves, clippy à zéro. **Pour un humain, l'article est illisible** (une colonne de 60 000 de haut) : un prototype en colonnes de journal, fait de l'extérieur par les commandes, le fait tenir dans un écran. **Sa question** : un domaine posé sur une membrane vaut-il pour ce qu'elle contient ?
* **Ses commandes au terminal** (fiche 90, 8/11, sa demande du 7/11) : `glucose document.glucose` et un double-clic ouvrent un document ; **chaque outil de la porte des agents est une commande** (`glucose aide`, `lire`, `chercher eau`, `domaines`, `poser-texte "…"`, `cat notes.md | glucose poser-texte -`) — une seule table, la même porte, le même accord, chaque geste annulable ; et **l'agent cherche comme `Ctrl+F`** (`glucose_chercher`, douze outils). **Essai réel, tout passe** (`outils/essai_terminal.sh` : le vrai Glucose sans fenêtre, le vrai programme, un document ouvert par son nom dans un écran invisible, la capture regardée) ; 2 874 épreuves, clippy à zéro ; **sabotages : 21 tombés sur 21** (`terminal-90.json`, MEMB-7, le plan et COORD-1 compris). **Pas vu à son écran** ; sous Windows, la sortie ne se lit que redirigée.
* **Le domaine traverse les fichiers, et une IA le lit comme l'humain** (fiche 88, 7/11). Chaque fichier garde ses domaines ; Glucose met côte à côte ceux des documents qu'il connaît, sans jamais y écrire. **Sa règle du 7/11 : deux domaines ne portent jamais le même nom** — « même sujet », ou préciser le mot (« Python code », « Python serpent »). Un agent lit les domaines de chaque nœud, la table de tout ce que Glucose connaît, les nœuds d'un domaine dans tous les fichiers, et les pose (`glucose_poser_domaine`, `glucose_domaines`) ; le panneau montre les autres fichiers, pose la question, et va à chaque place (« où »). **Essai réel hors écran, tout passe** (`outils/essai_domaine.py`) ; 2 845 épreuves, clippy à zéro, 34 sabotages tombés sur 34 (`domaine-88.json`), **dix vertes sur GitHub** (`218d7b3`, exécution 42411174857 ; Windows 2 951 épreuves, dont les 43 du domaine) ; **pas vu à son écran**, pas publié.
* **`Ctrl+F`** (fiche 89, enregistré le 8/11, `58657ab`) : chercher dans tout le document, sans majuscules ni accents, Entrée y va, `F3` au suivant ; « Chercher… » au menu du vide. Une lettre tapée coûte 4 à 8 ms sur 24 000 textes (110 à 218 avant l'index). **VOL-1**, trouvé en relisant : une flèche trouvée ne faisait pas voler la vue. **Sabotages : 12 tombés sur 12** (`chercher-89.json`). **Pas vu à son écran.**
* **CONSOLE-1** (fiche 88 § 10) : sa session du 5/11 sur ce PC est tombée d'une panique, `println!` sur une console disparue ; reproduit sans écran (le programme d'avant meurt deux fois sur deux, le réparé vit deux fois sur deux), réparé partout, gardé par le cliquet 13.
* **Sa décision du 7/11 : on arrête l'optimisation graphique ici** (*« ce n'est pas le but premier de Glucose »*, fiche 87 § 7 ter). **La beta 9, publiée par lui le 7/11** (`b04ccef`) : la fiche 87 et JOURNAL-1, sans le domaine.
* **Un tableau entier à l'écran ne coûte plus par photo à chaque image** (fiche 87) : ATLAS-1, PHOTO-1, LOT-1, JETER-1. En paires entrelacées, 8 000 images, Intel UHD 620, **sur secteur** : un vol 18 → 4,9 ms, un zoom 23 → 9,7 ms, un focus revenu au tableau 33 → 4,9 ms ; un focus pendant son zoom reste à 16 ms, le pire d'un zoom à 28 ms. **Ni sa GTX 1660, ni son téléphone, ni la batterie n'ont été mesurés.** **Dans la beta 9, publiée le 7/11.**
* **Sélectionner 8 000 images coûte encore 28 ms par image** (3,9 sinon), reproduit, **pas réparé** : fiche 87 § 7 bis.
* **Une session part dès sa fermeture** (JOURNAL-1, fiche 87), et non plus au lancement suivant ; essayé en réel contre un faux serveur. **Dans la beta 9, publiée le 7/11.**
* **La beta 8, publiée par lui le 6/11**, retire la barre blanche de Windows et les coûts au carré du texte (fiches 85 et 86).
* **Depuis le 6/11, on travaille sur son VivoBook sous NixOS** (i7-8550U, 7,6 Go, Intel UHD 620 et
  MX150, écran 60 Hz, Wayland) : le PC Windows des fiches précédentes est cassé. Le code n'a rien
  perdu. **Windows ne s'essaie plus qu'à travers la CI**, alors que ses utilisateurs connus y sont
  presque tous. Mode d'emploi : METHODE § 5 ; la carte de la machine : `~/Projets/LISEZ-MOI.md`.
* **Glucose excluait une machine, et on l'a appris par la télémétrie** (fiche 84) : un utilisateur
  Windows inconnu, le 29/10, quatre lancements sans une image, tué par `igvk64.dll`, le pilote Vulkan
  d'Intel. **MOTEUR-1** : un moteur qui fait tomber Glucose à son ouverture ne s'ouvre plus, jusqu'au
  processeur s'il le faut, sans liste de pilotes. **Joué en réel sous Linux, de vrais plantages, chaque
  image regardée** ; **pas prouvé sur le pilote fautif**, que personne n'a ici. Au passage : OpenGL
  était mort sous Wayland (corrigé, vu dessiner), et le nœud de bienvenue naissait sous la barre
  (ACCUEIL-2). **Attend la beta 7.**
* **La beta 7 est publiée par lui le 6/11** (`0138982`). **Sa surprise du même jour** : l'« utilisateur Windows inconnu » à la GTX 1660
  (`5280efb2`, 39 sessions depuis le 12/10) a la même carte et le même démarrage d'appareil que **ses
  journaux du PC de l'école** : c'est très probablement lui. Son contenu : MEMOIRE-4 (les 6 000 images), FIN-SESSION-1,
  MOTEUR-1, ACCUEIL-2, `rustls` 0.23.46 (ferme les 4 alertes de GitHub, aucune n'atteignait Glucose).
  Le garde-fou NOTES-1 arrête une publication dont les notes parlent d'une autre version.
* **La télémétrie au 8/11** : 142 sessions, 8 installations. **Son téléphone a reparlé le 6/11** (3 sessions, encore en beta 4, aucune mort) ; la « panique » de ce PC le 7/11 est celle que CONSOLE-1 a réparée.
* **La télémétrie au 6/11** : 132 sessions, 7 installations, 29 fins anormales. L'utilisateur à la
  GTX 1660 revient toujours (39 sessions, dernière le 4/11, en beta 6). **Son téléphone reste muet
  depuis le 15/10.** Son PC Windows : dernière session le 23/10.
* **La clé de mise à jour a retrouvé des copies** (7/11), toutes sur ce PC : voir § 4.
* **Sa parole du 25/10 : Protéo en pause, la partie 1 de Glucose d'abord** — le logiciel complet,
  ultra optimisé, des utilisateurs, une ergonomie parfaite. SUITE § 0 en propose la ligne
  d'arrivée (la 2.1.0, six conditions) et cinq étapes ; trois questions attendent sa parole.
* **Glucose Rust est publié.** La **2.0.3-beta.6** est la release « latest » depuis le 23/10
  (sur `4a111b4` : tout jusqu'à la fiche 81, la porte des agents comprise) : installeur Windows, AppImage, `.deb`,
  `.rpm`, APK Android, et `nix run github:shazamifius/GlucoseGit`. Chaque Glucose Tauri installé
  (depuis la 1.0.1-beta.9) bascule de lui-même vers Glucose Rust ; ses documents s'ouvrent, ses
  données ne sont jamais touchées. Son essai : **plus de 24 000 nœuds au téléphone, « ça n'a pas
  trop lagé, plutôt cool »** ; au PC, *« tout fonctionne merveilleusement bien »*.
* **Glucose finit quand le système le ferme** (fiche 82, FIN-SESSION-1, FENETRES-1, 25/10) :
  éteindre Windows, un installeur ou Windows Update qui ferme les applications, `SIGTERM` ou
  `Ctrl+C` sous Linux — Glucose écrit tout, sans question, et sa boîte noire dit « fermée
  proprement ». Jusqu'ici il était tué, et deux sessions de 3 h 37 et 7 h d'un utilisateur se
  lisaient comme des plantages. **La cause cachée** : le moteur OpenGL de `wgpu`, ouvert d'office
  et jamais employé, posait une fenêtre sourde qui bloquait le Restart Manager (et, selon toute
  vraisemblance, faisait attendre l'extinction) ; OpenGL ne s'ouvre plus que sur une machine qui
  n'a rien d'autre. **Prouvé en réel** sous Windows : un vrai Glucose fermé par le Restart Manager
  en 287 ms, sa boîte noire close — la beta 6, dans les mêmes conditions, refuse (351) et reste.
* **La mort du tableau de 6 000 images, reproduite et réparée** (fiche 83, MEMOIRE-4, 25/10) :
  un **téléphone virtuel** sur ce PC (Android 14, 4 Go, sans fenêtre, piloté par `adb`), un
  tableau de 6 000 images fabriqué par le vrai dépôt ; « Tout cadrer » tuait Glucose à 3,1 Go —
  les niveaux à rendre au système attendaient derrière les décodages, et personne ne les
  comptait. Ce qui rend la mémoire passe désormais devant ce qui en prend : Glucose survit, les
  6 000 images dessinées. **Pas encore sur son téléphone** : la prochaine version l'apportera.
* **La télémétrie au 25/10** : 125 sessions, 6 installations. Un utilisateur Windows inconnu de
  lui (GTX 1660) s'en sert presque chaque jour — 12 sessions du 20 au 23/10, un document de 373
  images, passé seul à la beta 6 ; un second (GTX 1660, encore en beta 3) ; **un testeur Android
  neuf** (puce Mali, le 18/10, 8,2 ms par image ordinaire) ; son téléphone, **muet depuis le
  15/10** (beta 3, 16,4 ms par image ordinaire, 885 ms au pire).
* **P8, le texte net de près** (fiche 73) : écrit, éprouvé, saboté, **publié dans la beta 5** — **il attend son
  écran**, puis son téléphone (aucune mesure n'y a été faite). Et **P8 bis** : un nœud immense vu de
  près coûtait 10,5 ms par image au processeur (17 600 mots) ; 0,72 ms désormais, quelle que soit sa
  longueur — et la mise en page de chaque nœud visible n'est plus refaite deux fois par image.
* **Lire par niveaux** (fiche 79, 21/10, sa parole : « que l'IA ne récupère QUE ce dont elle a
  besoin ») : `glucose_lire` rend d'abord le sommaire (chaque nœud, sa première phrase, sa taille),
  puis le texte entier des nœuds choisis (`ids`) ou la phrase qui contient des mots (`cherche`).
  Éprouvé et saboté ; **pas encore vu à son écran**.
* **Ce que l'humain désigne** (fiche 80, 22/10, sa parole : « Protéo utilise Glucose comme un outil,
  et Glucose reçoit ses critiques ») : `glucose_lire` dit aussi ce que l'utilisateur a
  **sélectionné** (« ça »), l'**empreinte** de chaque texte (ce qu'il a récrit se voit sans tout
  relire) et l'**identité du document** ; `glucose_ouvrir_tableau` donne à un agent un onglet à
  lui sans changer celui qu'on regarde. Côté Protéo : il **relit** ce que l'humain fait à chaque
  message, et range ce qu'il réussit dans son **carnet**, « Ce que Protéo a appris ». Prouvé hors
  écran (le vrai Protéo, un Glucose sans fenêtre dont une « main d'épreuve » joue l'humain) ;
  **attend une version publiée** : la beta 5 n'a pas la porte. **Le carnet vu en réel** (fiche 81) :
  une leçon posée par un calcul voté, corrigée depuis l'autre onglet, relue au message suivant ;
  l'image a montré la carte d'accueil **choisie d'office**, que Protéo prenait pour un geste de
  l'utilisateur (« il désigne : Bienvenue dans Glucose ») — corrigé, ACCUEIL-1. Son usage réel,
  compté dans son journal : 444 gestes, aucun refus, un seul manque — la porte publiée.
* **La porte des agents** (fiche 75, P10, sa demande du 17/10) : `glucose --mcp`, le protocole MCP,
  huit outils (douze depuis la fiche 90, et chacun une commande du terminal) ; **Protéo pose sa recherche dans Glucose, en direct** — sa demande, une colonne par
  sous-question avec les pages lues en capture (le double-clic les ouvre), les citations exactes,
  le rapport relié phrase par phrase aux citations, qui brillent au survol. Chaque geste s'annule
  par `Ctrl+Z` ; rien sans son accord. Prouvé hors écran sur deux recherches neuves du vrai Protéo
  (images regardées) ; **attend son écran**, et une version publiée (la beta 5 n'a pas la porte).
  Envoyée sur GitHub (`1265c9b`, sans CI). **Le tri de la demande** (fiche 76) : Protéo dit pour chaque
  demande s'il faut une réponse courte, des outils ou un workflow — et sa taille, mesurée par le
  plan et tenue à jour — et en tire sa route ; mesuré sur ses 28 demandes réelles : 23 justes, aucune
  faute grave. **FLECHE-6** : une flèche ancrée arrive à la hauteur de sa citation, par le côté qui
  fait face. **Son essai du 17/10 au soir**, puis cinq séries d'essais réels (le vrai Protéo, le vrai web,
  un Glucose sans fenêtre) : un tri qui privait d'outils (« je n'ai aucun outil ») — il ne retire plus
  rien ; une page lue par son seul titre — le corps se juge sans lui ; une mission en pause qui prenait
  pour elle les demandes suivantes — elle ne reprend que si la demande la poursuit ; des appels
  d'outils écrits dans le texte quand la mission s'annonçait en note — elle s'écrit en échange
  d'outils. **La mécanique tient ; l'intelligence du qwen2.5:7b, non** (il demande des sources au
  lieu de chercher, n'extrait rien d'une page en anglais, et sa recherche dérive) : le chantier
  suivant est son remplacement, mesuré (fiche 76 § 7). **Le cerveau, la nuit du 18/10** (fiche 77) :
  la machine des candidats retenait un modèle de vision robotique (un a priori devenu verdict) ; le
  banc était aveugle aux modèles de 2026 (leur réflexion épuisait les sorties bornées : 41 réponses
  illisibles sur 41) et parlait en partie au modèle en service — corrigé. Douze modèles criblés sous
  Ollama 0.40.6. **Le banc rendu juste** (fiche 78) : les échecs du 4B venaient en partie de la boucle
  (la sortie coupée à 1 536 jetons, SORTIE-1 — en production aussi), l'épreuve `scripts` mesurait un
  chemin mort (retirée, ses tâches dans la boucle : 20 cas), Ollama rechargeait le modèle à presque
  chaque question (FENETRE-1), une annulation pendant le tri se perdait (ANNULE-1). **Mesuré : qwen3.5:4b
  réussit 15 cas sur 20 de la boucle du chat, qwen2.5:7b 3 — démontré (p = 0,0005), aucun cas perdu**,
  pour 13 % de temps en plus ; la réflexion semble aider (15 contre 10), non démontré. Aucune bascule
  encore : les essais réels ont été interrompus (il devait éteindre), et la bascule reste sa parole. L'organe de recherche : les 116 pages
  (PROGRES-1), les mots-clés sans sujet (SUJET-1), les cascades de blocages (DEPEND-1). La toile :
  une demande par rangée (RANGEE-1), une suite reliée par une flèche (SUITE-2) — pas encore vues en
  essai réel.
* **Ses utilisateurs** : 5 de Glucose Tauri (Windows, Linux) ; une vingtaine de testeurs qui
  attendent Glucose Rust, dont 10 sous Android ; une personne qui compte énormément pour lui, sur
  iPad (Glucose dans Safari, plus tard). **La télémétrie** (avec l'accord de chacun, un serveur
  Cloudflare de son compte, `docs/TELEMETRIE.md`) : 89 sessions, 5 installations au 17/10, dont
  deux utilisateurs Windows inconnus de lui — l'un est passé **seul** à la beta 5 (la mise à jour
  automatique marche chez un autre que lui) ; **toujours aucune session du téléphone depuis le
  15/10** (il est encore en beta 3).
* **Le 15/10, sur deux téléphones, un tableau de plus de 6 000 images s'est fermé seul et n'est
  pas revenu** (« parfaitement fluide » jusque-là). **La mort est reproduite et réparée** sur le
  téléphone virtuel (fiche 83, MEMOIRE-4) : un grand tableau vu en entier épuisait la mémoire.
  Aucune session de la beta 3 n'est arrivée du téléphone — le serveur refusait toute session de
  plus de 512 Kio (réparé, POIDS-1). Sa précision du soir : l'application a « juste dépop » et a
  redémarré seule sur la carte de bienvenue — la reprise n'a rien rouvert ; sur le téléphone
  virtuel, elle rouvre le brouillon après chaque mort, et LIBERE-1 (beta 4) défend la cause la
  plus probable du « non rouvert ». Rien dans le code n'efface un brouillon à la mort : **son
  tableau est peut-être encore dans le téléphone**.
* **Android** : Glucose tourne sur son **moto g54 5G** (une PowerVR BXM-8-256). **Le 12/10, une
  longue session a figé tout le téléphone**, bouton d'alimentation compris, après des saccades qui
  montaient, puis un redémarrage seul — *« c'est bien Glucose »*. Cause la plus probable : la
  mémoire (Glucose n'y bornait pas ses décodages, n'écoutait pas `onTrimMemory`, ne mesurait pas
  sa propre empreinte) — **corrigé côté code (fiche 66), pas prouvé** : il faut une longue session
  sur son téléphone. **Il a perdu son câble** ; le débogage sans fil est probablement impossible
  (son PC passe par le partage de connexion du téléphone) : l'APK passera par un zip (fiche 71 § 3).
  Sur son téléphone, une image coûtait **19,5 ms** en médiane — sous le plancher presque tout le
  temps.
* **Sa machine** (Windows 11, RTX 5070 Laptop et Intel Arc 140T, écran 2 560 × 1 600 à 240 Hz et
  150 %) : 3 à 7 ms par image en médiane ; **le pire monte à 11-24 ms dans les zooms chargés**
  (sa chronique du 14/10 : 345 images sur 13 886 au-dessus de 10 ms). **Le plancher de 100 images
  par seconde n'est pas tenu partout.**
* **La parité avec Glucose Tauri est loin d'être atteinte** (un quart au comptage du 24/09, bien
  plus aujourd'hui ; [INVENTAIRE](INVENTAIRE.md) § 3 dit ce qui manque) ; le cœur est solide.
* **Il ne peut dépenser aucun argent** : pas de certificat Windows, pas de compte Apple, pas de
  compte Play à 25 $. Android passe par un APK installé à la main (Play Protect : « Installer quand
  même » ; la vérification des développeurs n'arrive en France qu'en 2027).

## 2. Ce que fait Glucose Rust

| domaine | ce qui marche |
|---|---|
| **Naviguer** | canevas infini, **zoom jusqu'à ×256** (déduit de la précision du document, fiche 72) ; souris instantanée ; pavé par *Direct Manipulation* (la similitude entière des doigts — « tellement mieux qu'avant », 15/10) ; `F` cadre la sélection, `Alt+F` tout ; signets `Ctrl+1..9` ; minimap ; onglets ; le canevas défile au bord ; les doigts au téléphone |
| **Le document** | `.glucose` écrit geste après geste, images scellées, brouillons, reprise au lancement, Time Machine (`Ctrl+H`), documents de Tauri lus ; `Ctrl+N`, `Ctrl+O` ; au téléphone, les documents rangés et rouverts |
| **Images** | PNG, JPEG, WebP, GIF, BMP ; coller, déposer (Pinterest en pleine résolution sous Windows) ; rotation, recadrage non destructif, `Ctrl+B` ; la mémoire par étages (carte graphique, mémoire, disque, bornés par ce que la machine laisse) |
| **Texte** | Markdown, tableaux, liens, LaTeX fidèle à KaTeX, annulation mot par mot ; le clavier du téléphone ; **de près, la carte graphique dessine les lettres depuis leur forme** (Slug, P8, fiche 73) : nettes à tout zoom jusqu'à ×256, ~1 ms par image ; au processeur, les grandes lettres se tracent (GRAND-1) |
| **L'interface** | **les raccourcis sont une donnée** (une couche de texte, `assets/raccourcis/glucose.txt`), **78 commandes** dans une table, le menu qui les exécute et dit leurs raccourcis, **la palette** (`Ctrl+Maj+P`) — la fondation des presets (P4, fiche 72) ; **ses raccourcis se changent dans Glucose**, écrits dans `raccourcis.txt` (fiche 74) |
| **Flèches** | droites ou courbes, double sens, six relations, coudes, contournement, ancres de texte — qui arrivent à la hauteur de leur passage, par le côté qui fait face (FLECHE-6) |
| **Copier, coller** | la sélection entière d'une fenêtre à l'autre ; un seul « Copier » qui donne aussi les images (Discord, navigateur) ; le glisser vers d'autres applications sous Windows ; sous Linux, la sélection X11 servie par Glucose |
| **Organiser** | membranes (tracées ou posées) ; dossiers ; domaines ; aimant ; **Ordonner** (cinq dispositions, huit tris dont trois par la couleur) ; **Mood board** (six gabarits de zones, « Par mood board ») ; **le Storyboard** : la grille tracée, retracée, ses cases glissées, décrites, plusieurs images qui remplissent les cases, un storyboard de la sélection, les « + », la lecture plein écran, durée, plan et champs par case, plusieurs storyboards par tableau, les planches en PDF |
| **Exporter** | Markdown ; **SVG complet** (le texte en tracés, lisible par tout logiciel) ; PNG de la sélection ou du tableau ; planches d'un storyboard ; au téléphone, par la feuille de partage |
| **Fenêtre** | le mode référence (`Ctrl+Maj+A`) : sans interface, sans cadre, au premier plan |
| **Les agents** | `glucose --mcp` : Protéo, Claude ou tout client MCP lit le tableau et y pose textes, membranes, pages web, flèches ancrées à des phrases — en direct, chaque geste annulable, avec son accord (« Retirer l'accord des agents… » le reprend) ; Protéo y pose chaque demande, son tri et sa réponse |
| **Mise à jour** | automatique et signée, sous Windows et Linux ; sous Android, écrite, à prouver par la prochaine version |

**Jugé à son écran** (fiches 51, 52, 58, 60, 64) : le copier-coller et le glisser entre fenêtres,
Discord ; `Ctrl+N` ; le mode référence et ses signets ; les poignées ; la molette ; le lag réglé
(ECRAN-1) ; les questions dessinées ; aucun panneau au lancement ; NETTETE-1 (« plus du tout de
flou ») ; l'export au téléphone par la feuille de partage ; le déplacement au pavé. **Tout ce qui
est venu depuis la fiche 62 attend son écran** : la liste unique est dans [SUITE](SUITE.md) § 2.

## 3. La qualité, mesurée

* **Au 8/11 au soir : `1282358` (le pincement), dix vertes** (exécution 42504658154, lue dans le journal) : le Mac à puce Apple 2 859 épreuves, dont le pincement de macOS. Avant : **`a10eb80`, dix vertes** (exécution 42494464628, lue dans le journal) : Windows 2 984 épreuves, aucune tombée — les fiches 89 à 91 (Ctrl+F, les commandes du terminal, MEMB-7, le plan), le paquet NixOS et les paquets Linux qui portent le type `.glucose`. **Sabotages** : 12 sur 12 (`chercher-89.json`), 21 sur 21 (`terminal-90.json`).
* **Au 7/11, dernière vérification GitHub : `39894bf`, dix vertes** (exécution 42402082754, lue dans le journal) : Windows 2 878 épreuves, aucune tombée. Windows était rouge deux fois de suite : `bench_memoire`, un banc réservé à Windows, puis l'épreuve de l'atlas, dont la borne de 1 sur 255 était une supposition ; WARP lit 2, ce que la précision des cartes permet (1/2⁸ de texel par axe). Sur ce PC, une épreuve de temps tombe encore sous la charge de toute la suite et passe seule (`test_relire_la_memoire_est_assez_bon_marche_pour_chaque_image`, 20 µs pour une borne de 25, **sa borne non relevée**). **Sabotages** (`outils/sabotages/huit-mille-2.json`, 75 gardes) : **tous joués** ; les 20 restants, 16 tombés et 4 aveugles (CHOIX-1, JOURNAL-1, PHOTO-1, LOT-1), corrigés par une meilleure épreuve et rejoués.
* **Au 6/11** : **`0138982` : dix vertes** (exécution 42216997831) : Windows 2 710 épreuves, Linux
  2 701, les deux Mac 2 677, aucun échec ; **84 épreuves de rendu par Direct3D 12 nommé** (WARP), et un
  vrai plantage qui fait tomber le moteur, sous Windows. **COURSE-1** : le plantage des épreuves dans
  le chargeur Vulkan, reproduit sous `gdb` et réparé (fiche 84 § 8). Le Mac à puce Apple passe à
  `macos-15` : GitHub a retiré `macos-14`.
* **2 680 épreuves** sous Windows, toutes vertes, 7 ignorées (22/10, fiche 81) ; clippy strict à zéro sous
  Windows (Rust 1.99.1) et pour Android ; la suite Linux tourne aussi sur ce PC (WSL `Glucose-Linux`, METHODE
  § 5), rejouée pour la dernière fois à la fiche 61.
* **Les sabotages** : chaque garde est cassée exprès et doit faire tomber une épreuve
  (`outils/sabotages/`, un fichier par chantier) — 30 sur 30 pour l'export SVG (fiche 71) ; à la
  fiche 72, 25 sur 25 pour le téléphone, 4 sur 4 pour LIBERE-1 (sous Linux), 10 sur 10 pour
  GRAND-1, 2 sur 2 pour le zoom, 20 sur 20 pour P4 ; à la fiche 73, **37 sur 37 pour P8** — après
  qu'une première passe en eut trouvé cinq aveugles, chacun corrigé par l'épreuve qui le voit ; 9 sur 9 pour P8 bis ;
  à la fiche 74, **13 sur 13 pour la couche des raccourcis de l'utilisateur**, et les 20 de P4
  rejoués ; à la fiche 75, **36 sur 36 pour la porte des agents** — après une épreuve aveugle et deux
  qui se figeaient, chacune corrigée ; à la fiche 76, **5 sur 5 pour FLECHE-6**, et côté Protéo
  **24 sur 24** pour le tri et la toile ; aux fiches 79 et 80, 4 + 6 + 3 + 4 pour les agents ; à la
  fiche 81, 1 sur 1 pour ACCUEIL-1.
* **Sur GitHub, dix tâches** à chaque envoi avec la CI : Windows, Linux, les deux Mac, le noyau
  pour Android et le web, l'APK, NixOS, l'installeur et la mise à jour, la répétition de la
  bascule depuis un vrai Glucose Tauri, les paquets Linux. Dernières vertes : `82c774b` (13/10) ;
  **`58478d6` (15/10) : dix vertes** — la CI groupée de la beta 3 (exécution 38717823204) ;
  **`3e1ee42` (15/10) : dix vertes** — celle de la beta 4 (exécution 38749250295) ;
  **`a6689c8` (16/10) : dix vertes** — P8, ses nuanceurs neufs traduits pour Metal, Vulkan et
  l'APK (exécution 38811883089). P8 bis, du Rust sans rien de propre à une plateforme, n'a pas
  relancé la CI : la publication l'a passée — **`141499c` (16/10) : dix vertes** (exécution
  38820378998), puis la construction signée de la beta 5 (exécution 38821870908).
* **Aucun plantage ni gel** de Glucose noté par Windows depuis la publication. **Au repos**, Glucose
  redessine encore pour ses messages (1,6 image par seconde) : la charte dit zéro.

## 4. Points de vigilance

* **La clé Android** : dans les secrets du dépôt, et dans
  `/stockage/ancien-windows/bureau/sauvegarde-documents-2026-10-31.7z` (chiffrée, testée le 6/11).
* **La clé de mise à jour** (`glucose_updater.key`, celle de chaque version de Tauri) : le secret du dépôt, et **deux copies sur ce PC depuis le 7/11** (fiche 87 § 7 sexies), sur ses deux disques. **Aucune hors de cet ordinateur** : s'il disparaît, il ne reste que le secret, qui ne se relit que par une tâche. `sauvegarde-profil-2026-10-31.7z` reste introuvable : ses clés de serveur, son identité web3 et ses brouillons n'ont pas de copie connue ici.
* Son dossier Glucose (`~/.local/state/glucose`, `%LOCALAPPDATA%\Glucose` sous Windows) ne se touche
  jamais ; un diagnostic se fait sur une copie de ses documents.
* **Deux branches, et pas une de plus** (sa demande du 16/10) : `main` et `tauri-v1.0.1`, en local
  comme sur GitHub. On travaille sur `main` ; une branche de travail se retire dès sa fusion.
  L'ancienne branche locale `secours/avant-reattribution` (les commits d'avant la réécriture des
  auteurs) est sauvée dans `Documents\Glucose-sauvegardes\avant-reattribution.bundle` — vérifiée ;
  `git fetch <ce fichier> secours/avant-reattribution:secours` la rend.
