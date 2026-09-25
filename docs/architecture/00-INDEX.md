# Glucose — Dossier d'architecture

> **Rôle de ce dossier** : servir d'architecte. Il ne contient aucun code. Il contient l'état
> réel du projet, les problèmes nommés et vérifiés, l'architecture cible et le plan pour
> y arriver.
>
> **Date** : 2026-09-10 — audit au commit `7a13c86`, **re-vérifié** au commit `81aea31`
> **Périmètre audité** : `crates/glucose-core` + `crates/glucose-desktop` (14 322 lignes Rust)
> **Référence de parité** : la version TypeScript (`src/`, 45 218 lignes, 180 fichiers)

---

## Les documents

| # | Document | Ce qu'il contient | Quand le lire |
|---|---|---|---|
| **01** | [Audit du code Rust](01-AUDIT-CODE-RUST.md) | **33 constats vérifiés** (dont **4 déjà corrigés** pendant la rédaction), avec gravité, fichier:ligne et correctif. Ce qui n'est pas pro, ce qui n'est pas optimisé, et pourquoi. | Maintenant. C'est le diagnostic. |
| **02** | [Architecture cible](02-ARCHITECTURE-CIBLE.md) | Le découpage en crates, les 10 lois, la politique de dépendances, le modèle, l'undo, le rendu, la persistance. **Et la vraie solution à ton problème de glisser-déposer.** | Avant d'écrire la première ligne. |
| **03** | [Parité fonctionnelle](03-PARITE-FONCTIONNELLE.md) | **279 fonctionnalités inventoriées**, avec leur état exact. Le chiffre réel : **22 %**, pas 3 %. | Pour savoir où tu en es, vraiment. |
| **04** | [Roadmap](04-ROADMAP.md) | 13 phases, chacune avec ses travaux et ses **critères de sortie**. De 20 % à 100 %. | Pour savoir quoi faire lundi matin. |
| **05** | [Standards de code](05-STANDARDS-DE-CODE.md) | Les règles qui empêchent le code de redevenir ce qu'il est. Diagnostic précis de « pourquoi c'est moche ». | Avant chaque merge. |
| **06** | [Rendu visuel & Design System](06-SPEC-RENDU-VISUEL-DESIGN-SYSTEM.md) | **La bible visuelle pixel-perfect** : tokens HEX/RGBA exacts, grille infinie ($G, R, \alpha, P$), cadre 3px, poignées 9px (hitbox 48px), symbiose chromatique (djb2 + Perlin 2D + moyenne vectorielle 1200px), flèches multicouches, minimap, chrome. | Indispensable pour coder le rendu Rust identique. |
| **07** | [Animations & Interactions](07-SPEC-ANIMATIONS-ET-INTERACTIONS.md) | **La cinétique exacte** : table TIMING, cubic ease-out, rebond de dock (overshoot 156%), FLIP 250ms, l'arbitre de clic PICK-1 (rangs 0 à 60, cycle 8px), magnétisme SNAP-1 (8px écran), tweening des membranes (200ms). | Pour régler le "feel & polish" et les animations. |
| **08** | [Nœuds & Fonctionnalités Canvas](08-SPEC-FONCTIONNALITES-CANVAS-ET-NOEUDS.md) | **Inventaire technique exhaustif** : images (culling hystérésis, LOD progressif), texte GFM + KaTeX + undo unique, stickies + opérateurs (AND/OR/BUT/BECAUSE), flèches (évitement d'obstacles), dossiers miroir OS, miroirs vivants anti-cycle, membranes. | Pour implémenter chaque entité sans rien oublier. |
| **09** | [Systèmes Métier & Persistance](09-SPEC-SYSTEMES-METIER-ET-PERSISTANCE.md) | **Les moteurs sous le capot** : schéma universel, undo/redo (transactions atomiques), format binaire `.glucose` v2, bundle portable dédupliqué SHA-256, filet anti-corruption, domaines sémantiques, timeline temporelle, collaboration CRDT. | Pour la persistance et la robustesse globale. |
| **10** | [Spécification Visuelle & UI Panneaux](10-SPEC-VISUELLE-PANNEAUX-ET-UI.md) | **Atlas visuel pixel-perfect et 14 captures réelles** : dimensions exactes, tokens hexadécimaux, tous les panneaux (Ordonner, Pomodoro, Storyboard, Domaines, Presets, Plugins, Collab, Time Machine, Réglette temporelle, Diagnostics HUD) avec captures d'écran authentiques Playwright et atlas HTML interactif. | Pour coder le rendu des panneaux et du chrome en Rust. |
| **11** | [Plan de marche](11-PLAN-DE-MARCHE.md) | **La charte confrontée aux fiches 02 et 04** : 100 fps (budget 10 ms), netteté à l’arrêt, cible 10^7 nœuds (canva Wikipédia), dépendances minimales mais assumees. Mesures refaites a la main, 6 remises en question chiffrées (modèle 512 o/noeud, blit 4K a 9,6 ms = 96 % du budget, tuiles vs LOD, parité TS invalide comme étalon, les arêtes Wikipédia = le vrai milliard, auto-critique du zéro-dépendance), ordre de marche A→G. | Pour les six remises en question. **Son § 4 est remplacé à partir de l'étape C par la fiche 12.** |
| **12** | [Plan d'exécution](12-PLAN-D-EXECUTION.md) | **Tout ce qui reste, dans l'ordre.** Pourquoi l'étape C (tuiles, GPU) cède sa place : un cache de tuiles se conçoit quand on sait ce qu'il y a dans une tuile. La règle S qui rend cet ordre possible, l'état mesuré au commit `13d0b9c` (2 868 lignes de noyau sans appelant), cinq vagues de chantiers avec leur sortie mesurable, et la seule vraie inconnue : le LaTeX. | **Avant de choisir quoi faire ensuite.** |
| **13** | [Plan de correction](13-PLAN-DE-CORRECTION.md) | **Quatorze defauts qu'aucun des 915 tests ne voyait**, trouves en se servant de l'application a la main. Les trois familles que le test unitaire n'atteint pas — le materiel, le systeme, le temps — et la regle de methode qui en decoule : toute correction livre le test qui aurait attrape le defaut, ou le protocole de mesure quand aucun test ne le peut. | **Quand l'application se comporte mal a l'usage.** |
| **14** | [L'etat reel, et tout ce qui reste](14-ETAT-ET-RESTE.md) | **Le tableau complet**, mesure au commit `adaf415` : 34 % comptes en fonctions, 25 % comptes en travail restant, et pourquoi les deux chiffres sont vrais. Les 1 742 lignes ecrites, testees et eteintes. Treize blocs A a M avec le poids de chaque ligne, l'ordre que je defends, et les trois choses qui devraient inquieter et qu'aucun tableau ne mesure. | **Pour voir d'un coup ou on en est.** |
| **15** | [Plan de performance](15-PLAN-DE-PERFORMANCE.md) | Ce qui empechait la cadence au 17/09, poste par poste, et l'ordre des vagues A a G. **Ses vagues A et B sont faites** ; le goulot a change trois fois depuis. | Pour l'historique du diagnostic. |
| **16** | [Adaptativite](16-ADAPTATIVITE.md) | Utiliser tout ce que la machine a, a l'instant ou elle l'a : voies mesurees plutot qu'interrogees, bornes qui suivent le materiel, bridage thermique comme regime normal. | Avant de toucher a un choix qui depend du materiel. |
| **17** | [La journee du 18/09](17-SESSION-DU-18-09.md) | REPORT-1, CASCADE-1, COUT-1 : ce que la mesure a trouve, ce qu'elle a **dementi**, et les trois lecons de methode qui ont coute cher — dont « un compteur declare et jamais lu vaut zero, et un zero se lit comme une mesure ». | Pour ne pas refaire les memes erreurs de lecture. |
| **18** | [Cent quarante images par seconde, toujours](18-PLAN-R-ET-D.md) | **Le plan de R&D, et il remet en cause l'architecture actuelle.** Le mur arithmetique mesure (en 4K, effacer et recopier coutent 4,77 ms — il reste 2,37 ms a 140 fps). Trois lois qui remplacent tous les seuils choisis : la finesse suit la **vitesse** et non le budget (psychophysique du glissement retinien), le quadtree canonique memoise a la Hashlife, et le debit observe. Puis l'arbre de possibilites : on n'optimise plus le dessin, **on a deja dessine**. | **Avant tout nouveau travail sur la cadence ou le rendu.** |
| **19** | [Le temps que l'ecran montre](19-SESSION-DU-19-09.md) | **Ce que la session du 19/09 a trouve.** Le judder du freinage venait de l'HORLOGE, pas du modele du mouvement : la camera avancait du temps de calcul et etait vue pendant l'intervalle de presentation. L'application tournait sans synchronisation verticale depuis toujours. La chronique mesure enfin ce que l'ecran montre et CONCLUT. TUILE-1 branche : trois fois moins cher, un saut trois fois plus petit sur `bench_freinage`. Puis le TEMPO : un nombre constant de balayages par image, et deux sessions reelles qui confirment que la regularite prime sur la cadence. Six choses que la mesure a dementies. | **Pour lire une chronique, et avant de toucher au rythme.** |
| **20** | [La salissure, et quinze coeurs qui dormaient](20-SESSION-DU-20-09.md) | **Ce que la session du 20-21/09 a trouve.** L'etape 1 du plan 18 aux deux tiers : la chrome se redessine quand ELLE change (+0,29 ms), et le fond ne se peint plus sous des tuiles opaques qui le recouvrent (-1,25 ms). Le lecteur des cliquets lisait `ui.rs` sur 279 de ses 1 650 lignes -- deux plafonds remontent a la mesure vraie, sans qu'un seul acces ait ete ajoute. Et surtout **deux cercles vicieux que j'ai ecrits moi-meme** : une cible de finesse asservie a sa propre sortie, et un raffinement progressif qui coutait plus qu'il ne rapportait. La reduction de resolution ne gagne que 6 a 13 % pour un pixel qui en montre quatre. **Et la chronique sommait ses postes** : une seule image de gel lui faisait annoncer « blit 62,4 % » la ou blit coute 0,94 ms -- chiffre que j'ai cru au point de l'ecrire dans cette fiche avant de le demasquer. **ET LA DECOUVERTE** : la composition des pixels tournait sur UN coeur en scalaire, sur une machine qui en annonce seize. Interpoler sur seize fils (2,38 ms) coute MOINS que pixeliser sur un (3,19 ms) -- la pixelisation n'etait pas un compromis, c'etait le prix de n'avoir jamais utilise la machine. Le SIMD reste entier. | **Avant de croire un banc, avant de croire la chronique, et avant de toucher a la finesse.** |
| **21** | [Les voies : deux moteurs, une machine](21-LES-VOIES.md) | **Le plan qui corrige une erreur de lecture de la charte.** « Le GPU ne remplace jamais le CPU » interdisait de basculer PAR ECHEC, pas de construire une voie graphique -- la charte disait deja « les deux chemins sont de plein droit ». Mesure : 429 photos coutent 4,2-6,3 ms au processeur **et pixelise**, 1,22 ms sur GPU integre et 0,26 sur carte dediee, **nettes**. L'architecture qui en decoule : un socle commun (le quoi), deux executants VRAIMENT distincts (le comment), un arbitre qui place Glucose **la ou il reste des ressources** -- jamais en concurrence avec Adobe ou Blender. Quatre etapes, chacune avec ce qui la valide. | **Avant tout travail sur le rendu ou la cadence.** |
| **22** | [Le fond et les lueurs sur la carte](22-SESSION-DU-21-09.md) | **Ce que l'apres-midi du 21/09 a fait de l'etape 3.** Les lueurs et le fond descendent sur la carte -- `halos` et `clear` disparaissent du profil, et la couche du dessous **cesse d'exister** quand elle est vide, ce qui vaut mieux qu'un cache que le plan proposait et qui n'aurait rien gagne pendant un geste. Les deux voies prouvees d'accord, borne par borne, chaque borne baissee a zero pour lire la mesure : 1 niveau pour une lueur, 7 pour la grille -- exactement la borne calculee --, **3 sur la scene entiere et zero canal au-dela**. Une capture cote a cote trouve la TROISIEME regression de l'etape 1, invisible a mille trois cents tests : les photos en chemin ne se dessinaient plus. BLINK-1 : le dessin du curseur lisait l'horloge. Et le plan du texte comme composant, fonde sur ce que la recherche dit : un atlas pour la chrome, MSDF pour les cartes zoomees, jamais l'un pour l'autre. **Le gain a l'ecran n'est pas mesure.** | **Avant de toucher au texte, et pour lire la prochaine chronique.** |
| **23** | [Les deux cartes, la rampe, et trois refus](23-SESSION-DU-21-09-SOIR.md) | **Ce que la soiree du 21/09 a tranche, et surtout ce qu'elle n'a PAS pu corriger.** Les gels de `present` etaient bien le chemin hybride : ils disparaissent sur la carte dediee. `blit` etait un FAUX coupable -- la marque absorbait le rendu des textures ; separee, elle vaut 1 ms. CASCADE-2 : un composant garde son ancien palier pose tant que le nouveau n'est pas rendu, et le budget vaut pour les deux tours. BANDE-1 : la couche du dessus ne s'efface et ne part que par les bandes reellement ecrites, 12 % de l'ecran. **Mais son § 4 vaut plus que ses gains** : trois corrections ecrites, trois refusees par la mesure -- dont une qu'un cliquet vieux de deux sessions a eu raison de refuser contre moi. | **Pour la methode du doute, avant tout chantier de cadence.** |
| **24** | [L'instrument etait aveugle](24-SESSION-DU-22-09.md) | **Ce que le terrain du 22/09 a repondu, et le defaut que la reponse a mis au jour.** L'hypothese `acquerir` est MORTE : trois images en vol degradent six indicateurs sur six, et le poste n'apparait dans aucune des deux chroniques -- les 19,5 ms etaient une attente de synchronisation en `Fifo` que `Mailbox` supprime. **Et en le cherchant, on a trouve que le tableau des postes triait par MEDIANE et coupait a six** : `textures`, nul en median et a 25 ms au p99, n'y paraissait jamais, alors que c'est le p99 que le tempo suit. Meme faute que la fiche 20 § 4.5 a l'envers, corrigee dans un banc la veille et laissee dans l'instrument de l'utilisateur. Quatrieme marque mal posee (`minimap` mangeait le fil d'Ariane), `ui` en nommait quatre, et l'echelle de l'interface n'etait ecrite nulle part -- le banc mesure la minimap a 0,05 ms, le terrain a 0,86. Une texture de composant coute du remplissage pur, ~5 ms/Mpx, un tiers de cadre et deux tiers de texte ; **le tampon ne coute rien, mon estimation se trompait d'un facteur cent**. Et la trancher en bandes est REFUSE : le cout tient, les octets non. Une fois l'instrument repare, une troisieme session a designe le geste le plus cher de toute la session, que personne n'avait jamais pu expliquer : **editer du texte coute 19,48 ms par image**, cinquante et une par seconde pendant qu'on ecrit. COMPOSANT-2 y repond -- la carte qu'on edite devient une texture comme les autres, et cent images de saisie immobile n'en font plus que DEUX au lieu de cent, prouve sans chronometre. | **Avant de lire la prochaine chronique, et avant de croire un banc.** |
| **25** | [La porte d'entree](25-SESSION-DU-22-09-SOIR.md) | **Ce que la session du 22/09 au soir a fait de sa decision : « finir le logiciel, publier, avoir des retours ».** Sa liste de quatre fonctionnalites, entiere. Le glisser-deposer depuis un navigateur, par une cible de depot COM a nous : la recherche a DEMENTI la premisse « un navigateur ne donne qu'une adresse » -- Windows a un format pour les fichiers promis depuis 2009, c'est le navigateur qui telecharge, et l'arbitrage HTTPS-ou-COM n'existait pas. La mesure de ce que le PROCESSUS coute -- 461 a 598 Mo sur un noeud, 1,6 image par seconde sans qu'on touche a rien --, dont le premier critere a ete dementi en vingt secondes de terrain. Le recadrage non destructif en fractions, rendu sur les deux voies sans un pixel de plus, avec la premiere migration chainee du format prouvee sur un vrai fichier v1. La detection de bordures suit FFmpeg moins deux criteres qu'un test a refuses chacun : la moyenne mange le haut des lettres, la mediane mange un damier, on garde le defaut qui ne se voit pas. `Alt` sur un cote recadre comme `Alt` sur un coin tourne. Et l'arbitre repare : le brief avait trouve un defaut, c'est l'autre qui decidait -- la porte se fermait a la 240e image, le premier gel tombait a la 833e. **Neuf choses dementies, dont six sont des fautes de TEST.** | **Avant de tester a l'ecran, et avant d'ecrire un test qui prouve une egalite.** |
| **26** | [Le gel avait un nom](26-SESSION-DU-23-09.md) | **Ce qu'une lecture attentive de la MEME chronique a tranche.** Le pire gel du 22/09 au soir est la bascule de carte de l'arbitre : la banniere annonce 138 + 606 = 744 ms, la chronique 748,7 ms a ne pas dessiner, et l'image suivante paie 406 ms de `present` -- une seconde de canevas fige en plein zoom. On ne peut pas rendre la bascule rapide (ouvrir une carte coute une demi-seconde partout, c'est pourquoi les logiciels professionnels demandent un redemarrage) : ARBITRE-3 la rend INVISIBLE, elle attend que la vue s'immobilise. **La piste principale du brief est dementie** : apres la bascule, la RTX presente normalement (p99 2,90 ms) ; le tableau de la fiche 25 § 9.3 comparait une session sur une carte a une session sur deux. ENTRACTE-1 decoupe enfin le temps « a ne pas dessiner » en cinq postes qui se somment au bit pres. **Et le « tressaut x85 », premier du verdict depuis quatre sessions, etait un gel compte deux fois** -- l'image figee puis celle qui rattrape : 1 024 / 12 = 85,3. Le gel a desormais son constat, le tressaut ne se juge que sur le rendu. Enfin, deux panneaux montraient une selection perimee : leur cle etait trop ETROITE avant d'etre trop large. | **Avant d'attaquer un constat du verdict : verifier qu'il ne compte pas deux fois la meme chose.** |
| **27** | [L'image et sa provenance](27-L-IMAGE-ET-SA-PROVENANCE.md) | **L'objectif que l'utilisateur appelle « ultime et final » : la meilleure qualite, l'origine, l'auteur.** Ce qui est fait : une epingle Pinterest se pose en image, dans sa pleine resolution d'origine -- Glucose telecharge ce qu'on depose, par WinHTTP, sans caisse de plus, verifie sur quatre epingles reelles (le premier essai a rapporte une ICONE, vue en ouvrant l'image). Ctrl+B emporte enfin les rangs de transition du liseré, et ne mange plus de contenu -- l'ancienne boucle coupait la manche d'un kimono. Ce qui existe en 2026 : SauceNAO pour l'illustration (auteur, lien), Google Web Detection pour la photo, Bing ferme depuis aout 2025. **Aucun service ne sait la toute premiere publication** : on construit la plus ancienne connue. Et le « 99 % sur » ne vient pas du service mais d'une empreinte perceptuelle comparee chez nous, avec un seuil mesure. | **Avant de construire la recherche d'origine.** |
| **28** | [Le liseré avait deux causes](28-SESSION-DU-23-09-SOIR.md) | **Le Ctrl+B de la capture du soir, tranché sur ses pixels.** Le liseré avait deux causes : le DETECTEUR gardait des rangs clairs -- il supposait un fondu uniforme, et le bord d'une peinture ondule --, et le RENDU lisait la colonne que Ctrl+B venait de retirer (123 a l'ecran pour 78 dans le fichier). Les deux voies bornent desormais leur lecture a la fenetre, par une regle nee une fois dans le noyau, sans tolerance. Le fondu se lit pixel par pixel, sur les pixels qui PEUVENT en montrer un : deux criteres refuses par la mesure, et un defaut plus ancien trouve -- BORDURES-3 rognait la pointe des objets. Le bouton Trans-domaines qui mentait fait ses trois choses ; une carte neuve nait son texte selectionne ; une epingle deposee repond dans l'instant et se pose ou on l'a lachee. **L'empreinte perceptuelle : pHash a 0 bit pour la meme image, 20 au plus proche pour deux differentes, et un seuil qui se CALCULE -- N P(Bin(64, 1/2) <= t) <= 1 %.** La bande coutait 0,49 ms : ses 20 ms etaient une cinquieme marque mal posee. | **Avant de construire la recherche d'origine, et avant de croire un poste de la chronique.** |
| **29** | [La session suivante](29-LA-SESSION-SUIVANTE.md) | **Le dossier de travail de la session qui suit : ce qui est su, ce qui ne l'est pas, et toutes les pistes -- pas de solutions.** Ce que l'utilisateur a vu et dit le 23/09 au soir : Ctrl+B parfait sur la forêt, imparfait à droite sur une gravure ; Trans-domaines FAUX de bout en bout -- et son bouton n'avait jamais bascule ; le texte qui disparait de pres (la fiche 24 § 13 l'avait predit) ; les images qui arrivent en vagues et 1,3 Go de memoire -- la carte recoit toujours la texture NATIVE ; l'application qui dessine 13 images par seconde au repos ; ses deux longues sessions a 164-194 noeuds, perdues sauf son resume. | **Au debut de la session suivante -- et pour la contredire.** |
| **30** | [De près, au repos, et en vagues](30-SESSION-DU-23-09-NUIT.md) | **Trois pistes de la fiche 29, et Trans-domaines retiré de bout en bout.** DE-PRES-1 : une carte plus grande que l'écran se découpe en tuiles au lieu de disparaître -- gouttière d'un texel, repli au palier où elle tient ; et la texture entière d'avant n'était « au bit près » qu'à l'échelle 1. Le repos : l'instrument était AVEUGLE, le masque des réveils effacé par `frame_begin` avant d'être lu ; chaque image dit maintenant d'où elle vient, et la barre ne se redessine plus à chaque mouvement. NIVEAU-GPU-1 : la carte reçoit le niveau qui couvre la photo, prêté sans copie -- la copie coûtait 10,5 ms, plus que l'envoi, et une photo réduite crénelait. La gravure introuvable, 486 images examinées. Rien encore vu à l'écran. | **Avant sa prochaine longue session, et pour lire sa chronique.** |
| **31** | [La longue session lue](31-LA-LONGUE-SESSION-LUE.md) | **Sa longue session (243 noeuds, 45 épingles) lue, et la mémoire par étages qu'il demande : VRAM, puis RAM, puis SSD, adaptée en temps réel.** La provenance marche : les images « au repos » sont des animations, le système n'en demande que 0,4 %. Un gel de 2,46 s dans un seul événement de la main -- `Ctrl+B` innocent, une fenêtre de Windows soupçonnée. ORNEMENTS-2 : 243 photos sélectionnées passent de 21,6 à 0,48 ms, et `fill_crisp` écrit lui-même, au bit près de `tiny-skia` -- dont un défaut trouvé au passage. VRAM-1 : la sonde lit le budget que Windows accorde sur la carte, et la carte garde ce qui quitte l'écran dans la moitié de ce budget. Le décodage JPEG réduit démenti (-20 % seulement). | **Avant de poursuivre la mémoire par étages.** |
| **32** | [La mémoire par étages](32-LA-MEMOIRE-PAR-ETAGES.md) | **Les sept étapes approuvées le 24/09, et les restes des fiches 29-31.** La mémoire, c'étaient les images (1 186 Mo de pyramides sur 243 photos) — après une erreur de ma part sur le mauvais document. ETAGES-1 : tenu ce que l'écran montre et la vue d'ensemble, offert à Windows tout le reste — 1 248 Mo → 164 Mo, tout revient en 12 ms. ETAGES-2 : la carte se rend même quand Glucose dort. ETAGES-3 : un cran commun quand la carte manque de place, le processeur si rien ne tient. ETAGES-4 : les aperçus sur le disque, un écran de pixels par document quelle que soit sa taille — ouvrir `fuser` passe de 462 à 8 ms. Et SURVOL-2, CASCADE-3, COLLER-1, la provenance qui ne compte plus les messages endormis. Un plantage rare, sa cause probable corrigée. | **Avant sa prochaine session, et avant l'étape 7.** |
| **33** | [Le filet, et l'histoire](33-LE-FILET-ET-L-HISTOIRE.md) | **Ses deux images de `Ctrl+B`, sa sortie du 24/09, et l'enregistrement à la git.** BORDURES-5 : un filet gris d'un pixel au bord cachait toute une marge — un seul filet, devant une bande épaisse, part désormais ; mesuré sur ses 275 images, exactement les quatre qui en portent un changent. L'emboîtement libre des bandes mangeait le fond d'une photo : refusé. Les épreuves de BORDURES-3 ne lisaient qu'un bord, et leurs images finissaient en un trait. La suite annoncée verte ne l'était pas (une épreuve du cran paniquait). Sa sortie : la mémoire tient (448 Mo au lieu de 1 543), la cadence tombe à 43 i/s — TEMPO-2 dira si c'est une queue de coûts ou un cercle. L'enregistrement : `Ctrl+S` réécrit 180 Mo pour un document de 96 Ko, une image sur deux vit dans le dossier temporaire ; la conception à la git, à lui présenter. | **Avant sa prochaine session, et avant tout code d'enregistrement.** |
| **34** | [La tache sur la marge](34-LA-TACHE-SUR-LA-MARGE.md) | **Sa gravure : `Ctrl+B` laissait encore la marge de droite.** Pas un filet : trois petites taches sur le papier, chacune un peu plus d'un centième d'une colonne. BORDURES-6 : une tache s'enjambe si la marge reprend, reste sous le centième en tout, et finit sur un bord net atteint par une rampe qui monte. Sans le bord net, 48 de ses 286 images changeaient en perdant du contenu ; avec, deux, justes. Reste un liseré de 2 à 4 px au coin d'une plaque penchée : question de ressenti. Sa sortie : les gels de 2 s sont ses `Ctrl+S` ; TEMPO-2 dit que ce n'est pas un cercle — une image ordinaire coûte 7 à 8 ms, c'est le chantier de la cadence. L'enregistrement : direction approuvée, deux décisions à lui. | **Avant sa prochaine session, et avant tout code d'enregistrement.** |
| **35** | [La frange et le vol](35-LA-FRANGE-ET-LE-VOL.md) | **Les contours clairs de ses gravures, et l'animation de focus.** BORDURES-7 : la frange d'un bord de plaque — le papier recule de plus d'un centième par ligne, sur au plus un centième de la ligne, jusqu'à un contenu sans papier ; 31 images sur 308 changent de 1 à 9 px, toutes regardées. VOL-2 : `F` et les signets suivent le chemin de van Wijk et Nuij (zoom et translation ensemble, dézoom puis rezoom quand c'est loin), au profil de secousse minimale ; les dossiers aussi — une seule géométrie de vol. Une épreuve de la carte tombe au hasard (2 sur 10), préexistante, à enquêter. | **Avant sa prochaine session.** |
| **36** | [La route vers la V1](36-LA-ROUTE-VERS-LA-V1.md) | **La carte des sessions à venir, dans l'ordre que je choisirais.** Où l'on en est : ≈ 25 % du logiciel, la fluidité pas encore tenue (43 i/s, les panneaux à 34 ms). Ce que la bascule depuis Tauri exige, vérifié : le canal de mise à jour est la release « latest » de ce dépôt, la clé de signature existe, Tauri tourne sur trois systèmes, ses documents ne s'ouvrent pas encore. Neuf phases : le document (le journal comme colonne vertébrale), la fluidité, la distribution, la parité, le co-working, le MCP en Rust, plugins et IA, la bascule, macOS et Linux. | **Au début de chaque session, jusqu'à la V1.** |
| **37** | [Le document, et le fil qui dessine](37-LE-DOCUMENT-ET-LE-FIL-QUI-DESSINE.md) | **La phase 1 de la fiche 36, entière, et la phase 2 jusqu'où elle se mesure sans lui.** Un seul fondement, le geste : le journal du noyau contre Automerge et Loro sur son document — 1,4 ms pour 1 312 gestes contre 826 et 54, 124 octets par geste contre 356 et 375. Les documents de Tauri s'ouvrent (`Ctrl+O`, un lecteur à nous, identique à la référence). Chaque geste s'écrit à la suite du fichier, sommes chaînées : plus de `Ctrl+S` qui fige, les images dans le fichier, ouvrir 1 683 → 0,67 ms. Brouillons et texte en cours de frappe survivent à un plantage. La Time Machine (`Ctrl+H`). Trois défauts trouvés en chemin, dont deux fenêtres qui écrivaient le même document. CEDER-1 : les pics de 21 à 34 ms des panneaux étaient ceux de l'atelier réveillé — reproduits sur sa machine, 21,6 → 3,0 ms. L'épreuve au hasard : un fichier partagé. Puis la phase 4 commence : une membrane possède ce qu'on y dépose (la déplacer emporte son contenu), et le mode Focus — décider en redécrivant tout le tableau coûtait 8 ms par image à 10 000 nœuds, une carte des membranes le ramène à 0,0003 ms. Son premier essai (import, `Ctrl+S`, Time Machine validés) : le lancement rouvre toujours le dernier document, la carte d'un plantage revient en édition, un bouton Time Machine et une réglette qu'on glisse ; la barre ne cède plus ses libellés à des largeurs écrites en dur. | **Avant sa prochaine session : le § 11 dit quoi regarder, le § 12 ce qui attend sa parole.** |
| **38** | [La forme, et les onglets](38-LA-FORME-ET-LES-ONGLETS.md) | **Son second essai lu, et les onglets selon sa réponse.** Sa chronique accusait les membranes (23 ms par image) : une loi unique dans le noyau — une membrane est un seul champ d'opacité, sur des coins circulaires à distance exacte —, peinte par la carte ou par segments au processeur, 18-24 → 3,2-3,7 ms ; la couche du dessous ne part plus entière (BANDE-2) ; le liseré de la Time Machine passe sur la carte, panneaux 11,7 → 0,5 ms. La reprise après plantage rend le curseur et la sélection. Les onglets se renomment, se rangent, se suppriment (`Ctrl+Z` les rend) ; un tableau vit tant qu'on l'atteint, ce qui corrige deux défauts anciens (un miroir supprimé emportait l'original) ; importer un document dans un onglet neuf, en un geste à part de `Ctrl+O`, tous ses identifiants renommés. Un seul rectangle arrondi, en vrais arcs ; une tolérance d'épreuve qui n'était pas une borne disparaît. Les membranes s'arrêtent là : il a demandé qu'on en parle. | **Avant sa prochaine session : le § 8 pour la discussion des membranes, le § 10 pour quoi regarder.** |
| **39** | [Le registre de Tauri](39-LE-REGISTRE-DE-TAURI.md) | **Tout ce qui ne va pas dans Glucose Tauri, relevé par lui le 25/09, à ne pas reproduire dans Rust.** Dix-huit entrées, ses mots gardés : les ancres de texte des flèches (un mot désigné par son texte, pas par sa place), l'alignement intelligent (membranes, dossiers, premier placement), les tiroirs de Plugins/Preset/Domaines, sa vision des plugins, le texte en retard quand la vue bouge, l'ordre de priorité au clic et les poignées trop petites, Ollama, le bouton Trans-domaines, la Time Machine (enlever « compacter », des jalons datés, une optimisation définie), l'idée de Mary sur les membranes (classique, minimisée, étirée), les rideaux. Chaque ligne dit son état dans Rust. | **Avant de toucher à l'un de ces sujets ; à tenir à jour.** |
| **40** | [Les flèches](40-LES-FLECHES.md) | **Sa demande : « revoir complètement les flèches, et même leur design, car sur Tauri elles sont trop belles ».** L'aspect de Tauri (dégradé de la source à la cible, halo, pastille, trait constant à l'écran) ; par le traceur générique, 34 ms pour 80 flèches — par la loi de leur champ, 9,3 ms au processeur, et sur la carte graphique chez lui, sans toucher la couche du dessus. Une barre d'options (droite ou courbe, double sens, épaisseur, relation). Les ancres de texte, l'entrée 1 de son registre : un passage désigné par ses octets et non par son mot (une position n'est pas une identité — l'épreuve a retrouvé son défaut), qui suit l'écriture, d'où la flèche part, qui brille seul au survol, et qu'on choisit dans la carte même. Le module porté de Tauri plantait sur un accent. | **Avant sa prochaine session : le § 8 pour quoi regarder.** |
| **41** | [La flèche qui se voit, et la frappe](41-LA-FLECHE-QUI-SE-VOIT-ET-LA-FRAPPE.md) | **Ses retours sur la fiche 40, une session coupée par lui, et sa reprise.** La flèche qu'on tire se voit pendant le glisser (l'index ne connaissait que le document publié) ; la carte prend l'habit de Tauri — le texte sur fond noir, le contour en lueur — ; les tailles d'écran du canevas en pixels logiques à 150 % ; les barres du bas à la mesure de Tauri ; une vraie fenêtre d'ancrage ; des passages encadrés sur la police. Puis ce qu'une frappe coûte : le curseur, la pastille d'une formule et l'anneau de sélection deviennent des ornements — sélectionner ne refait plus une texture, et les deux voies s'accordent **au bit près** (deux tolérances disparaissent) ; la brume par la loi des membranes. Son terrain (11,6 ms) reste cinq fois plus cher que le banc : un compteur départagera. Le registre de Tauri vérifié entrée par entrée : l'ordre au clic se reproduisait (PICK-2), l'aimant au premier placement manquait (PLACEMENT-1), les jalons sont datés, Trans-domaines est expliqué. Quatre références de logiciels pour les rideaux. | **Avant sa prochaine session : le § 13 pour quoi regarder, le § 14 pour ce qui attend sa parole.** |

À côté, [`docs/heritage/`](../heritage/README.md) garde la roadmap et la spécification de
**Glucose Tauri** — l'inventaire du *quoi* et la vision, jamais un modèle du *comment*. Ce
qu'elles déclarent « fait » l'est dans `src/`, pas dans les crates.

---

## Le diagnostic en cinq phrases

1. **Le code n'est pas mauvais, il est débranché.** 13 des 18 modules de `glucose-core` sont
   écrits, testés — et jamais appelés par l'application. C'est ≈ 3 700 lignes de travail réel
   qui ne sert à rien pour l'utilisateur.

2. **Tu n'es pas à 3 %, tu es à 22 %.** Et si tu ne faisais que **brancher** ce qui existe déjà,
   sans écrire un seul algorithme nouveau, tu passerais à **26 %**.

3. ~~**L'application ne sait pas enregistrer.**~~ **Corrigé depuis.** Le format `.glucose` v2 est
   écrit et relu, avec somme de contrôle par section et écriture atomique (`Ctrl+S`, `Ctrl+O`).

4. **La teinte symbiotique est recalculée en O(n) par nœud visible, par frame, deux fois** —
   pour un résultat identique à la frame précédente. *(Les deux autres fautes de performance,
   la grille en O(surface) et l'absence de culling, viennent d'être corrigées par le commit
   `81aea31`.)*

5. **L'interface calcule sa géométrie deux fois** — une fois pour dessiner, une fois pour
   cliquer, avec les mêmes nombres recopiés. Le commit `81aea31` vient de régler ça **pour la
   barre d'outils**, avec des tests, et c'est exactement le bon patron. **La barre d'onglets et
   la minimap ont suivi depuis** (`layout_tabs`, `layout_minimap`, chacune avec son test de clic).

---

## Tes questions, mes réponses d'architecte

### « Est-ce que je dois continuer, ou tout reprendre ? »

**Continue — mais restructure.** Ce n'est pas la même chose que « réécris ».

Ce qui est bon et doit être **gardé tel quel** : `hit_priority`, `smart_align`, `symbiotic_hue`,
`quadtree`, `mirror_graph`, `bundle`, `export`, les modules de membranes et de rideaux, et
surtout les **3 194 lignes de tests**. Soit ≈ 2 800 lignes de logique métier valide.

Ce qui doit être **refait** : `store.rs` (snapshots d'undo, scans linéaires, collisions d'ids),
`app.rs` (objet-dieu de 1 029 lignes), `ui.rs` + `renderer.rs` (géométrie en double, aucun cache).

**Ce n'est pas un redémarrage à zéro. C'est la même matière, dans une structure qui la laisse
enfin servir.**

### « Le "0 dépendance", c'est réaliste ? »

**Presque. Vise 2, pas 0** — et surtout, place-les au bon endroit.

Écrire ton propre rastériseur 2D (2 000-3 000 lignes) : **oui**, c'est faisable, agréable et
ça t'appartient. Écrire ton propre décodeur WebP : **non** — c'est un décodeur VP8 complet,
plusieurs mois, et une surface d'attaque sur des octets hostiles pour un gain de compréhension
nul.

Détail complet dans [`02-ARCHITECTURE-CIBLE.md` § 0](02-ARCHITECTURE-CIBLE.md).

Note au passage : le commit `d7e460a` affirme avoir éliminé `tiny-skia`. **C'est faux** — il est
importé dans 5 des 6 modules du desktop, et l'application dépend aujourd'hui de 7 crates directs
et ≈ 200 transitifs. Un historique qui ment t'empêche de savoir où tu en es (R-32).

### « Le glisser-déposer web ET fichiers, c'est impossible ? »

**Non. Et c'est le meilleur argument "from scratch" de tout ton projet.**

Ce n'est pas une limite de Rust ni de Tauri : c'est une limite de `winit`, qui n'expose que
`DroppedFile` avec un chemin, et **jette tout le reste**.

Or, quand tu fais glisser une image depuis un navigateur, Windows te propose **simultanément**
`CF_HDROP`, `CFSTR_INETURL`, `CF_HTML`, `CF_UNICODETEXT`, `CFSTR_FILECONTENTS` et `CF_DIB`.
En implémentant toi-même `IDropTarget` — environ 400 lignes — tu obtiens **fichiers locaux ET
images web dans le même geste, avec l'URL d'origine préservée** (le champ `source_url` de ton
modèle, qui n'a jamais été rempli).

Ici, écrire soi-même n'est pas une question de fierté : **c'est la seule façon d'avoir la
fonctionnalité.** Détail en [`02-ARCHITECTURE-CIBLE.md` § 0.1](02-ARCHITECTURE-CIBLE.md),
phase 5 de la roadmap.

### Les deux règles qui priment sur tout le reste

Elles sont détaillées dans [`05-STANDARDS-DE-CODE.md`](05-STANDARDS-DE-CODE.md), en tête des règles.

**R1 — Rien à moitié.** Une fonctionnalité livrée à moitié est pire que pas livrée : elle a l'air
de marcher. Une fonctionnalité n'est finie que si elle est **branchée**, **visible**,
**annulable** et **conservée à l'enregistrement**. Quatre conditions. Ce dépôt est un catalogue de
ce que coûte l'inverse — voir R-18, R-33, R-47.

**R2 — L'ancien code TypeScript n'est pas un modèle.** Il sert d'inventaire de ce qui existe, et
à rien d'autre. Son implémentation n'est ni portée, ni imitée, ni citée comme justification. Une
décision se défend par un raisonnement et une mesure. On refait tout en Rust, **mieux**.

### « Pourquoi j'ai l'impression d'être à 3 % ? »

Deux raisons, toutes les deux mesurées :

1. **10 boutons sur 19 n'ont aucun effet observable** — ils affichent un toast qui *décrit* une
   action qui n'a pas lieu. « 🎨 Préréglage PureRef appliqué » : rien n'est appliqué. Il y a
   **24 appels à `show_toast`** dans `app.rs`. Tu regardes l'écran et tu vois Glucose ; tu
   cliques et il n'y a rien derrière.

2. **24 fonctionnalités sont écrites mais débranchées.** Tu as fait le travail, il ne compte pas.
   C'est démoralisant précisément parce que l'effort a été fourni. Mesure exacte au commit
   `e2cd410` : **12 des 20 modules de `glucose-core` ne sont appelés par personne**, soit
   **3 601 lignes** écrites, testées, inatteignables (R-18).

3. **Ce qui s'affiche s'affiche mal.** Deux défauts dégradent *tout* le rendu, indépendamment des
   fonctionnalités : le texte ne suit pas le zoom (onze `clamp` bornent le contenu des cartes
   pendant que leur boîte se met à l'échelle librement — R-45), et les glyphes sont posés à des
   coordonnées entières tronquées, sans positionnement sous-pixel (R-46). Résultat : le
   « LOD » involontaire en dézoomant, et l'impression de flou.

4. **Une ligne du tableau de parité peut valoir 2 000 lignes de code.** « Collaboration : 0 % »
   pèse autant visuellement que « Storyboard : 0 % », alors que la première représente 1 920
   lignes de CRDT et la seconde 464. Le décompte par volume est dans
   [`03-PARITE-FONCTIONNELLE.md`](03-PARITE-FONCTIONNELLE.md) § *Ce que le score ne dit PAS* :
   **quatorze sous-systèmes, 19 670 lignes de TypeScript, dont aucun n'est utilisable.**

**Décision** : un bouton dont la fonction n'existe pas est retiré ou grisé. Jamais un toast qui
simule. C'est la phase 0, et elle ne coûte presque rien.

### « Pourquoi c'est si long de trouver et corriger un bug ? »

Parce que `window_event` fait **490 lignes avec 7 niveaux d'imbrication**, et que `app.rs`
mélange fenêtre, souris, clavier, presse-papiers, décodage d'images et algorithmes de mise en
page. Il n'existe aucune frontière où poser un point d'arrêt mental.

Et parce qu'il n'y a **aucun test** dans `glucose-desktop` (2 dans `canvas.rs`), **aucun type
d'erreur** dans tout le projet, et **aucune erreur visible** par l'utilisateur : quand une image
ne s'affiche pas, tu vois un rectangle gris et rien ne te dit pourquoi.

Règles correctives en [`05-STANDARDS-DE-CODE.md`](05-STANDARDS-DE-CODE.md).

### « Le code est moche. C'est réparable ? »

Oui, et le diagnostic est précis : **ce n'est ni le nommage ni l'indentation** — ils sont
corrects. C'est :

- **60 %** de répétition structurelle : créer une annotation demande 14 lignes dont 8 disent
  « rien », et ce bloc apparaît 4 fois dans `app.rs`. Cause : le modèle recopie ses champs
  communs dans les 4 variantes.
- **30 %** d'imbrication profonde.
- **10 %** de nombres magiques en double.

**La laideur est un symptôme de la structure, pas du style.** Une fois le modèle en composition
et l'UI en layout-unique, écrire du code beau devient le chemin le plus facile — pas un effort
en plus.

---

## Par où commencer, concrètement

Les points 1 à 8 de la liste d'origine sont **faits** : transform des images, grille adaptative,
culling, sélection élastique, layout unique, teinte mémorisée, boucle de rendu unique,
`smart_align` branché, générateur d'id en O(1), `app.rs` éclaté (1 126 → 438 l.). La frame est
passée de 240 ms à 17 ms. Voici la suite réelle, mesurée au commit `e2cd410`.

| Ordre | Action | Pourquoi maintenant | Poids |
|:--:|---|---|---|
| 1 | **La persistance** (R-01, phase 2) | **Rien n'est encore écrit sur disque.** Tant que ça dure, tout le reste est du travail qu'on perd en fermant la fenêtre. C'est la seule tâche qui transforme un prototype en logiciel. | 2 572 l. TS |
| 2 | **La fidélité du rendu** (R-45, R-46) | Ce que tu vois est faux à tout zoom ≠ 1, et le texte est flou. Ça dégrade les 22 % déjà acquis, donc ça coûte plus cher que ça n'en a l'air. | ~200 l. |
| 3 | **Brancher les 12 modules morts** (R-18, R-47) | **3 601 lignes déjà écrites et testées.** Les domaines sont l'exemple type : le noyau sait tout faire, l'interface écrit dans une liste fantôme. Meilleur rapport résultat/effort du projet. | déjà payé |
| 4 | **Retirer les 10 boutons qui mentent** (R-33, phase 0) | Sans ça tu ne peux pas mesurer ton avancement — et c'est la cause directe du ressenti « 3 % ». | quelques heures |
| 5 | **Rectangles sales** (R-42, tâche 1.13) | `docks` + `ui` = 59 % de la frame passés à redessiner l'immobile. | ~300 l. |
| 6 | **Texte riche : Markdown + KaTeX** | 2 229 l. TS, aucune ligne portée. C'est ce qui fait de Glucose un outil de pensée et pas un tableau d'images. | 2 229 l. TS |
| 7 | **Membranes** (3 modules morts + tween) | 2 661 l. TS ; 1 442 l. de noyau déjà écrites mais mortes. | 2 661 l. TS |
| 8 | **Flèches, dossiers, miroirs, temporalité, rideaux** | 5 167 l. TS ; noyau partiellement écrit, intégralement mort. | 5 167 l. TS |
| 9 | **Collaboration, plugins, télémétrie** | 4 179 l. TS, rien de porté. À traiter en dernier : ce sont les seuls sous-systèmes qui ne bloquent aucun usage solo. | 4 179 l. TS |

**Ce que cette table dit, et qu'il faut regarder en face** : les points 1 à 5 représentent
quelques milliers de lignes et rendraient le logiciel *utilisable*. Les points 6 à 9 représentent
**environ 14 000 lignes de TypeScript à porter** — c'est là qu'est le gros du chemin restant, et
aucune optimisation de performance ne le raccourcira.

Les points 2 à 6 représentent **moins de 400 lignes** et changent radicalement la sensation du
logiciel. Ne commence pas par le rastériseur maison : commence par les gains visibles.

⚠️ **Le point 7 mérite une alerte.** En deux commits, `window_event` a gagné 238 lignes et
`redraw()` 15 appels : les bonnes corrections s'entassent dans l'objet-dieu **faute d'un endroit
où les mettre**. À structure constante, chaque correction rend la suivante plus coûteuse.
C'est la seule dynamique de ce dossier qui va dans le mauvais sens.

---

## Les chiffres à retenir

*Au commit `81aea31`.*

| | |
|---|---:|
| Lignes Rust | 14 322 |
| Lignes de tests | 3 194 |
| **Modules de noyau morts** | **13 / 18** |
| Lignes de noyau inutilisées | ≈ 3 700 |
| **Parité fonctionnelle réelle** | **22 %** |
| Parité si on branche l'existant | **26 %** |
| Fonctionnalités inventoriées | 279 |
| Fonctionnalités « maquette » | 18 |
| Plus grosse fonction (`window_event`) | **728 lignes** ⚠️ |
| Appels directs à `redraw()` | **45** ⚠️ |
| Types d'erreur définis | **0** |
| **Chemins d'écriture de projet** | **0** |
| Boutons qui agissent | **9 / 19** |

---

## Comment maintenir ce dossier

- **`03-PARITE-FONCTIONNELLE.md`** est mis à jour à chaque fin de phase, avec les vrais chiffres.
  C'est ton tableau de bord.
- **`01-AUDIT-CODE-RUST.md`** : chaque constat `R-xx` est barré quand il est corrigé, avec le
  commit qui l'a fait. Le document devient un historique de dette remboursée.
- **`04-ROADMAP.md`** : les critères de sortie sont cochés. Une phase n'avance pas sans eux —
  c'est exactement le mécanisme qui manquait quand `smart_align` a été écrit puis oublié.
- **`docs/architecture/decisions/`** : une note par décision structurante, notamment toute
  nouvelle dépendance.

---

*Ce dossier ne vaut que s'il reste vrai. Un document d'architecture qui ment est pire que pas de
document du tout — c'est exactement le problème que ce dossier existe pour corriger.*
