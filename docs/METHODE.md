# Comment on travaille sur Glucose

> **Au 25/10/2026** (fiches 59 à 82). Les règles qui font foi, la façon de prouver, et les leçons que ce dépôt a
> payées — écrites pour qu'une session neuve ne les repaie pas. La **charte** elle-même (100 images
> par seconde, aucune machine exclue, deux voies, élégance mathématique) vit dans les mémoires de
> Claude, qui font autorité : les lire **en entier** avant tout.

---

## 1. Avec lui

* **Il ne code pas.** Il porte la vision et le ressenti ; la technique se décide et s'exécute
  seule, et s'explique dans le rapport. Il parle français, et veut être **contredit** quand on a
  un argument, lui compris.
* **Il ne voit pas la console.** Ce qu'il lance s'écrit dans un fichier à la racine
  (`cargo run --release > sortie-x.txt 2>&1`, ignoré par git), qu'on lit soi-même.
* **C'est à nous de tester** (sa parole du 18/10 : *« tu seras sûrement meilleur juge que
  moi »*) : les essais réels se mènent et se jugent ici, chaque image regardée ; une fenêtre sur
  son écran est permise pour un essai (puis fermée), les bancs restent hors écran. Lui garde le
  ressenti et les choix de vision, avec une liste courte de quoi regarder.
* **Ses données avant tout.** Une perte le fait paniquer. On diagnostique sur une **copie** de ses
  documents (`examples/verifier_images`, `examples/lire_histoire` lisent sans écrire), et
  `%LOCALAPPDATA%\Glucose` (ses brouillons, sa boîte noire) ne se touche jamais.
* **Ses mots** : « nœud », jamais « carte » pour un élément du tableau ; « carte graphique » en
  entier. « Glucose Tauri » et « Glucose Rust », tels quels.
* **Les questions se posent en prose**, jamais en menu à choix. Les sessions sont longues : on
  enchaîne les chantiers, on ne s'arrête que pour un essai à l'écran ou un jugement de ressenti.

## 2. Les règles de code qui font foi

* **R1 — Rien à moitié.** Une fonctionnalité est finie quand elle est **branchée, visible,
  annulable et enregistrée**. Un bouton qui ne fait rien encore le dit ; il ne fait jamais semblant.
* **R2 — Glucose Tauri dit *quoi*, jamais *comment*.** Les fiches 06 à 10 du carnet en sont la
  spécification ; son code n'est pas un modèle. On recrée, on ne traduit pas.
* **Les cliquets** (`crates/glucose-desktop/tests/cliquets_suite.rs`) l'emportent sur les seuils
  écrits dans la fiche 05 : **80 lignes par fonction, 600 par fichier**, le couplage au modèle, les
  toasts, les modules sans appelant, les marques de mesure. Un cliquet qui proteste demande une
  extraction, jamais un plafond relevé.
* La fiche 05 reste la référence pour le reste (§ 1.7 : un gestionnaire d'événements ne contient
  aucune logique ; § 3 : toute modification passe par le journal ; § 4 : le rendu lit, il ne
  calcule pas, et ne touche jamais le disque). Sa règle « deux dépendances au plus » est
  dépassée : la charte dit « minimales mais assumées », et chaque dépendance a sa note.
* **Une constante arbitraire qui peut disparaître disparaît.** Une qui ne le peut pas se dit comme
  un « nombre de ressenti », avec le journal de ses valeurs essayées (exemple : `elan.rs`).
* **Les invariants portent un nom** (MEMB-1, PICK-2…) et un commentaire qui dit **pourquoi**.

## 3. Prouver

1. **Une épreuve par invariant**, et elle doit **tomber** quand on casse ce qu'elle garde :
   `outils/saboter.py` remplace un passage, lance les épreuves, exige qu'elles tombent, restaure.
   Le 29/09, quatre épreuves aveugles trouvées ainsi.
2. **Une épreuve porte sa preuve à l'envers** : elle rejoue l'ancienne loi sur les mêmes données et
   vérifie qu'elle concluait l'inverse. Plus fort qu'un `git stash`, qui ne prouve qu'une fois.
3. **Les deux voies au bit près**, ou à un écart mesuré, borné, et expliqué.
4. **Regarder l'image**, pas seulement les nombres : trois régressions de rendu n'ont été vues qu'en
   posant deux captures côte à côte.
5. **Vérifier sur les machines qu'on ne possède pas** : la CI tourne sur six systèmes. Grouper les
   envois (un envoi plus récent annule le précédent) et suivre par
   `outils/suivre_ci.sh <empreinte complète>`.
6. **« Réglé » seulement vérifié** : sur une copie de ses documents, sur GitHub, ou à son écran.
   Jamais parce que ça compile.

## 4. Les leçons qui ont coûté des sessions

| la faute | ce qu'il faut faire |
|---|---|
| **Une marque de mesure absorbe ce qui la précède** — cinq fois (`occlusion`, `recolte`, `blit`, `minimap`, `bande`), et chaque fois le mauvais coupable pendant des sessions | poser la marque là où le travail change de nature ; les postes de l'entracte se somment au bit près |
| **Un compteur déclaré et jamais lu vaut zéro, et un zéro se lit comme une mesure** | cliquet 9 : aucun champ de la chronique ne reste vide |
| **Trier par médiane cache ce qui tient le p99**, et sommer laisse une image aberrante décider du typique | lire médiane, p99 et pire ; ne jamais additionner des centiles |
| **Un accusé de réception n'est pas une livraison** : `present` répondait « réussi » pendant que l'écran restait figé | mesurer ce que l'œil reçoit, pas ce que l'API répond |
| **Les cercles vicieux** — six : un mécanisme qui s'adapte à un coût qu'il ne commande pas (la finesse asservie à sa propre sortie, la résolution qui lisait les gels du pilote…) | demander « lequel **commande** ce symptôme », pas « lequel le produit » |
| **Un test qui simule ce que le code suppose ne teste que l'arithmétique** | simuler ce que la machine fait vraiment |
| **Comparer deux exécutions, c'est comparer du bruit** (2,3, 6,2 puis 5,7 ms pour la même mesure) | les deux régimes dans la **même** exécution, entrelacés |
| **Deux sessions de terrain ne jouent pas le même geste** | ne pas conclure d'une comparaison qui n'en autorise aucune |
| **L'œil a raison contre le banc** (le lissage à 5 ms « mesuré bon » était lagy) | un nombre de ressenti se juge à l'écran, par lui |
| **La première hypothèse est souvent fausse** (l'antivirus de GitHub, le rapporteur éteint, mes instances restées vivantes…) | diagnostiquer avant de corriger ; dire « probablement, à confirmer » |
| **Lire un titre de la charte et sous-lire la phrase d'après** : une session perdue à gagner des millisecondes au processeur pendant que la carte graphique faisait cinq fois mieux sans abîmer l'image | relire les mémoires en entier ; demander « quelle voie » avant d'optimiser celle en place |
| **Un réglage hérité d'un autre outil** : tout le lissage de la navigation a été réglé au pavé tactile, et la souris en a hérité | une voie par source, jamais un réglage commun |
| **Une décision globale lue par une collecte locale** : l'arbitre de clic par l'index ne voit que les nœuds proches du curseur, et ne voyait donc jamais le groupe d'une sélection (fiche 53 § 10) | ce qui dépend de toute la sélection se calcule avant de filtrer ; l'épreuve passe par la vraie souris |
| **Un mouvement appliqué au réveil de la boucle et non dans l'image** : une image reçoit deux pas, la suivante aucun — le tressaut ×5,8 du pavé (fiche 53 § 8) | tout ce qui bouge la caméra avance au moment de l'image, une fois |
| **Une cause supposée sans la mesurer** : le pincement « réglé » trois fois ; Pinterest pris pour un réseau en panne alors que la page arrivait nue, sans compression (fiche 53 § 1) | chronométrer chaque étape (`essai_rapatriement`), lire la sortie, comparer à curl |
| **Classer un geste au lieu d'appliquer ce qu'il fait** : un déplacement à 0,13 % d'écart d'échelle pris pour un pincement, bloqué jusqu'à la fin — la règle de Chromium et de Blender (fiche 54 § 3) | appliquer la transformation entière ; une règle qui tranche par seuil est le signe qu'une formule exacte manque |
| **Additionner des transformations au lieu de les composer** : deux doigts dans la même image, et le point entre eux dérivait de 12,5 unités (fiche 54 § 7) | une seule transformation par image, composée exactement ; l'épreuve qui bouge deux doigts l'un après l'autre |
| **Un résumé de moteur de recherche cite une version** : « le plugin Android 8.13 est le dernier » — il en était à 9.4 (fiche 54 § 7) | lire la version à la source (le dépôt Maven, `services.gradle.org`) |
| **Un repli qui répond à la place de quelqu'un** : sous Android, le dialogue absent répondait « non », et ce refus du journal technique s'est écrit comme un consentement (fiche 56 § 1) | un repli ne répond jamais pour l'utilisateur : il pose la question autrement (QUESTION-1), ou ne décide rien ; et une réponse dit qu'elle a été vue (VUE-1) |
| **Brancher un geste qui traverse une plateforme sans lire la vie de celle-ci** : le partage revient d'une autre application, et Android détruit la surface au départ — Glucose ne l'écoutait pas (fiche 56 § 1) | avant d'écrire un chemin, suivre l'utilisateur de bout en bout sur la plateforme : où il part, ce que le système détruit, ce qu'il rend |
| **Une fonctionnalité qui rend un document injoignable** : « Nouveau document » rangeait le travail sous « Canevas 1 », et rien au téléphone ne permettait de le rouvrir — vu en relisant, avant l'essai (fiche 56 § 6) | chaque chemin qui écrit ses données a son chemin de retour, écrit dans la même session |
| **Écrire dans un état qu'une file servira plus tard, puis le relire** : le clavier d'Android rend l'état d'avant l'envoi, et un miroir naïf défaisait ce que Glucose venait d'écrire (fiche 57, CLAVIER-2) | lire à la source si l'appel est fait quand il rend la main (`write_work`) ; retenir ce qui peut encore revenir, et l'ignorer ; l'épreuve sert la file à la main |
| **Un geste qui en ouvre un autre** : le menu agit à l'appui et ouvre une question ; si la question répondait au relâchement sans mémoire de l'appui, le doigt qui se lève aurait choisi pour lui (fiche 57, APPUI-1) | une réponse part au relâchement **sur la réponse pressée** ; un relâchement qu'on n'a pas reçu termine le geste qu'il terminait |
| **Une garde redondante passe au sabotage** : deux lignes empêchaient le même effet, et chacune, retirée seule, ne faisait rien tomber (fiche 57) | retirer celle qui n'a pas à exister, ou affirmer le contrat de celle qu'on garde — jamais la laisser aveugle |
| **Une boîte du système qui tient la boucle** : posée à l'ouverture, avant la première image, la question du journal technique laissait une fenêtre noire et une boîte derrière — « Tab puis Entrée, dans le noir » (fiche 58, POPUP-1) | toute question se dessine dans Glucose, et sa réponse a une **suite** au lieu d'un retour ; une boîte du système ne se pose que là où aucune fenêtre n'existe (DIAL-5) |
| **Une bande qui remplace ce qu'elle couvre par du vide** : avec le rail, la barre d'état n'était plus peinte, et la composition « remplacer » y laissait passer n'importe quoi — vu seulement en regardant l'aperçu (fiche 58 § 3) | regarder l'image ; lire le pixel qui étonne (alpha nul) ; une surface qui remplace doit tout peindre |
| **Une carte de gestes écrite sur le papier** : « un doigt tient un nœud pendant qu'un autre touche » se heurtait à l'appui long, dont le menu s'ouvrait à 0,4 s — vu en lisant le code, après la proposition (fiche 58 § 9) | lire le code des gestes **avant** de proposer une carte ; un geste neuf se décide par ce qui le **suit**, pas par une course contre un délai |
| **Renommer ce qu'un serveur a déjà reçu** : l'identifiant d'une session venait du nom de son fichier ; renommer aurait tout fait repartir en double (fiche 58 § 9) | une identité ne se tire pas d'un nom qu'on peut vouloir changer ; la migration se joue sur une **copie** de ses données |
| **Renommer ce qu'une répétition surveille** : la migration du journal rangeait le témoin que la répétition de la bascule plante dans `boite-noire` — rouge sur GitHub seulement (fiche 58 § 9) | chercher le vieux nom **partout** (scripts compris) avant de renommer ; une épreuve lie les scripts au nom que le code emploie |
| **Une épreuve où le choix revient tout de suite** : un sélecteur réel rend la main plus tard ; l'épreuve synchrone aurait laissé passer une fermeture qui n'attend pas son choix (fiche 58 § 9) | l'épreuve passe par la même boîte aux lettres que la vraie, et tourne la boucle pour la relever |
| **Un clippy local plus vieux que celui de GitHub** : Rust 1.99 y voyait un champ jamais lu que la 1.98 laissait passer — quatre tâches rouges (fiche 58 § 9) | `rustup update stable` avant un envoi : clippy doit dire ici ce qu'il dit là-bas |
| **Une taille citée de mémoire** : « Noto Emoji, de l'ordre d'un mégaoctet » — la police actuelle est variable et pèse 1,98 Mo ; son instance fixe, 887 Ko (fiche 58 § 5) | télécharger, mesurer, lire la `cmap` ; vérifier ce que le moteur lit (`fontdue` ignore les variations) |
| **Des captures diagnostiquées sans être datées** : la moitié de ses captures du téléphone venait de l'APK du 07/10, d'avant le bord à bord et l'appui long — « pas de Supprimer » n'était plus vrai (fiche 59 § 2) | dater chaque capture (l'horloge du téléphone, la télémétrie, les commits) avant d'en tirer une cause |
| **Un ordre de dessin qui contredit l'ordre du clic** : la question se peignait sous les panneaux, et le clic allait à la question — on touchait ce qu'on ne voyait pas (fiche 59, DECISION-1) | un seul ordre des couches, écrit une fois ; le dessin suit le clic (loi L4) |
| **Une interaction choisie sur le papier** : *Push-Edge*, meilleur à la souris, ne mène qu'à un passage du doigt au pavé bouton tenu, et le bord du bas d'une fenêtre plein écran est la barre des tâches (fiche 59, DEFILE-1) | lire l'outil de l'utilisateur (le pavé, sans débrayage) et le système avant de choisir ; le dire quand le plan change |
| **Écrire à chaque geste ce qu'on ne lira peut-être jamais** : les originaux de toutes les images à chaque `Ctrl+C`, pour recoller dans Glucose, qui ne les lit pas (fiche 59, COPIER-1) | fabriquer à la demande (`IDataObject::GetData`) ; ne garder que le dernier |
| **Un panneau porté sans être comparé à Tauri champ par champ** : Ordonner avait perdu son champ « Colonnes », et écrivait « LARGEUR CIBLE » là où la valeur est une hauteur — depuis le portage, sans que personne le voie (fiche 62 § 5) | ouvrir le composant de Tauri à côté du panneau de Rust, et cocher chaque champ, chaque libellé, chaque borne |
| **Une mise en page éprouvée qui déborde** : « Par slot preset » rangeait bien, mais une zone trop pleine débordait sur sa voisine, et un second rangement aurait changé ses images de zone — trouvé en relisant, aucune épreuve ne le disait (fiche 62 § 4) | se demander ce qui arrive **au-delà** de la capacité ; un geste de rangement se rejoue deux fois dans une épreuve : il doit être stable |
| **Une limite du matériel jamais lue** : seule la fenêtre se mesurait au plafond des textures de la carte ; une grande photo demandait ce qu'un téléphone refuse (fiche 59, PLAFOND-1) | toute ressource demandée à la carte se borne par ce qu'elle annonce (`Limits`) |
| **Une épreuve qui compare des chemins quand il fallait prouver une écriture** : « une seule écriture » passait aussi avec une réécriture aux mêmes noms (fiche 59) | changer entre deux appels ce que le second ne doit pas toucher, et vérifier qu'il n'y a pas touché |
| **Un chantier qui ajoute sur un système et retire sur les autres** : COPIER-1 a retiré « Copier l'image » partout et ne l'a remplacé que sous Windows — Linux et Mac reculaient, la CI verte (fiche 60 § 3) | relire ce qu'un chantier **retire**, plateforme par plateforme, avant de publier |
| **La durée de vie d'une bibliothèque jamais lue** : un `arboard::Clipboard` ouvert puis lâché à chaque geste — juste sous Windows, faux sous X11, où le copieur sert le collage ; la copie disparaissait (fiche 60, TENU-1) | lire la documentation et la source jusqu'au `Drop` ; tenir ce que la plateforme veut vivant |
| **Un compte refait à chaque endroit** : le menu, la barre et le clavier comptaient chacun la sélection, et deux oubliaient le dossier (fiche 60, DOSSIER-1) | une seule question au noyau (`compte_de_la_selection`), lue partout |
| **Un diagnostic bâti sur le nom d'un appareil** : les documents disaient Redmi 9, le téléphone branché est un moto g54 5G (fiche 60 § 4) — et ses rayures avaient été cherchées sur une puce Mali, alors que la sienne est une PowerVR (fiche 61 § 6) | demander au système ce qui est branché (`Get-PnpDevice`, `adb devices`) avant de raisonner sur lui ; et à lui, quel appareil a montré le défaut |
| **Un code de plateforme écrit sans jamais tourner** : le copier de Linux n'était que relu, et la CI ne fait que compiler ce qu'aucune épreuve n'exerce (fiche 61) | la distribution `Glucose-Linux` (§ 5) : compiler, clippy, et éprouver contre un vrai serveur X, avec un demandeur qui n'est pas le nôtre (`xclip`) |
| **Une épreuve qui prend la sélection X de l'écran** prend le presse-papiers de la personne qui la lance (fiche 61) | ne viser qu'un serveur X nommé exprès (`GLUCOSE_ECRAN_X_D_EPREUVE`), jamais `DISPLAY` |
| **L'ordre entre deux clients X** : une prise envoyée avant une question peut être traitée après ; et un client qui ferme sa connexion juste après sa dernière requête peut la voir jetée — deux blocages d'épreuve, une fois sur deux (fiche 61 § 5) | un aller-retour avec le serveur (une requête qui attend sa réponse) avant de se dire prêt, et avant de partir |
| **Un serveur X d'épreuve qui se réinitialise** dès que son dernier client part, et coupe un moment les connexions : une épreuve sur vingt tombait (fiche 61 § 5) | `Xvfb … -noreset` ; rejouer une série vingt fois de suite pour débusquer une course |
| **Deux signes de fin pour une même attente** : la remise au gestionnaire finissait sur sa réponse **ou** sur la sélection prise — chacun, retiré seul, n'aurait rien fait tomber (fiche 61) | chercher l'évènement qui vaut chez tous (la sélection prise, KDE compris), et retirer l'autre |
| **Des épreuves de droits jouées en `root`**, qui écrit même dans un fichier en lecture seule — et des dossiers d'épreuve laissés par `root`, interdits ensuite à l'utilisateur (fiche 61) | éprouver en utilisateur ordinaire, comme sur GitHub |
| **Deux compilations à la fois** (Windows et la machine virtuelle Linux) : « le fichier d'échange est trop petit », et des erreurs « can't find crate » qui ne disent pas leur cause (fiche 61) | une seule grosse compilation à la fois ; éteindre la machine virtuelle (`wsl --shutdown`, quand aucune autre distribution ne tourne) avant la suite Windows — arrêter la distribution seule lui laissait 14 Go engagés |
| **Un travail qui s'endort avant d'être fini** : la dernière image d'un geste, la finesse qui remonte par moitiés, les textures reportées — aucune raison de réveil ne les rappelait, et le flou restait jusqu'au geste suivant (fiche 63, NETTETE-1) | tout ce qui « se fera à l'image suivante » doit **demander** cette image, et finir d'elle-même ; l'épreuve joue l'endormissement (`prochain_reveil`) et exige qu'une image soit demandée, puis plus aucune |
| **Une règle d'arrêt bâtie sur une somme qui vaut le tout** : « les postes jusqu'à couvrir la médiane » les prenait tous, puisqu'ils se somment à la durée (fiche 63, POSTES-1) | se demander ce que la règle rend **dans le cas ordinaire** avant de l'écrire ; ici, couvrir l'**excès** au-dessus du plancher |
| **Deux suites d'épreuves lancées l'une sur l'autre** : `LNK1104`, l'exécutable verrouillé, pris pour une épreuve bloquée (fiche 63) | attendre la fin d'une vérification (ou la tuer) avant d'en lancer une autre ; `Get-CimInstance Win32_Process` dit qui tient l'exécutable |
| **Un commit pris pendant un sabotage** (l'envoi « urgent ») : le saboteur modifie des fichiers le temps d'une épreuve (fiche 63) | après tout commit qu'on n'a pas fait, vérifier qu'aucune garde sabotée n'y est entrée (`git show HEAD:<fichier>`) et qu'il compile ses épreuves |
| **Un témoin planté sous un nom de version** que le code a changé : `apercus/v2`, retiré au lancement par la version 3 — la répétition de la bascule rouge sur GitHub seulement (fiche 63) | une épreuve lie le script au nom que le code emploie (`apercu::VERSION`) : elle tombe ici d'abord |

| **Un banc qui recopie ce qu'il éprouve** : `banc_gpu` préparait les passes de la carte de son côté — une passe oubliée dans la vraie présentation ne faisait tomber aucune épreuve (fiche 64) | une seule définition, que le code et le banc appellent (`present/passes.rs`) |
| **Une affirmation sur sa machine écrite avant la mesure** : « le rail couvre la minimap sur son téléphone » — faux, il s'arrête 500 lignes au-dessus (fiche 64) | mesurer d'abord ; une épreuve qui dépend d'une géométrie **vérifie** cette géométrie, sinon elle devient aveugle en silence |
| **Une cause devinée deux fois** : la « tolérance de la main », puis rien — c'était la poignée de coin de l'image choisie (fiche 64) | afficher l'état juste après l'appui (`eprintln!` dans l'épreuve), puis le retirer |
| **Une épreuve qui dépend du temps** : un message naît transparent (fondu), et s'efface déjà sous la charge de la suite entière (fiche 64) | fixer l'horloge de ce qu'on éprouve (une vie longue, une naissance passée) ; une épreuve qui passe seule et tombe dans la suite est souvent celle-là |
| **Un seuil relatif à ce qui change** : « 5 % de la couche du dessus », et la couche a perdu la chrome (fiche 64) | borner par la grandeur qui sépare les deux fautes (ici la surface de la carte), jamais par une part de quelque chose d'autre |
| **Une épreuve d'interaction qui écrit l'état au lieu de faire le geste** : le champ « Décrire » rempli à la main (`champ.texte = …`) — il n'écrivait rien au clavier ; le fantôme de la grille calculé — il ne se redessinait pas au survol (fiche 65) | passer par le geste réel : la touche frappée (avec son texte composé), la souris qui bouge **et l'image redemandée** (`prendre_la_salissure`) |
| **Une partie entière posée sur un bord exact** : « les cases qui tiennent » — sept rangées au format Scope donnent 6,999999999999999, et saisir une poignée pile au coin aurait retiré une rangée (fiche 65) | choisir une loi sans bord à l'état de repos (l'étendue la plus proche : `round`) ; l'éprouver sur des étendues exactes, à tous les formats |
| **Qui possède quoi, décidé au moment où deux boîtes se recouvrent** : la case lâchée recouvrait une voisine, et l'image partait avec la mauvaise (fiche 65) | ce qu'une main porte se décide au début du geste et se garde jusqu'au bout |
| **Une condition lue à la place d'une autre** : j'ai cru qu'un glisser ne se refermait que si ses pas étaient les derniers écrits (`==`, la condition pour le *continuer*) ; celle qui le *referme* dit `<=` — ma garde en plus était redondante, et le sabotage l'a montrée aveugle (fiche 65) | lire la fonction même qu'on croit protéger ; une garde dont le sabotage ne fait rien tomber se retire, elle ne se « renforce » pas d'une épreuve inventée |
| **Une plateforme que le `cfg` ne nomme pas** : l'empreinte du processus était écrite pour `target_os = "linux"`, et Android est `android` — le téléphone, la machine qui a figé, ne mesurait rien (fiche 66) | à chaque `cfg(target_os = "linux")`, se demander si Android en est ; une mesure absente se dit « absente », et une plateforme qui devrait répondre a son épreuve (`ici()`) |
| **Un signal du système reçu, et jeté** : `onTrimMemory` arrivait jusqu'à `winit` (`memory_warning`) et aucune ligne n'y répondait (fiche 66) | avant d'écrire un pont vers le système, lire ce que le cadre transmet déjà, et chercher les méthodes de `ApplicationHandler` que l'application n'implémente pas |
| **Une borne tirée du papier** : « le décodeur garde chaque coefficient sur seize bits, six octets par pixel » — la mesure dit 0 pour un JPEG ordinaire, 3,8 au pire ; la borne comptait 300 Mo de trop (fiche 66) | mesurer la grandeur avant de la borner, chaque chemin dans son propre processus (`bench_decodage`) ; écrire d'où vient chaque terme |
| **Une épreuve qui bloque au lieu de tomber** : si la règle d'admission cassait, l'épreuve attendait sans fin, et le saboteur avec elle (fiche 66) | ce qui peut attendre s'éprouve dans un fil, avec un délai : une règle cassée fait tomber, jamais bloquer |
| **Une durée de vie tenue par un nom** : `let _admis = porte.entrer(…)` garde le jeton, `let _ = …` le lâche aussitôt — aucune épreuve ne voit la différence (fiche 66) | que le type l'interdise : `porte.pendant(octets, || travail)` tient le jeton exactement le temps du travail |
| **Une épreuve qui passe par la chance de ses données** : « défaire rend l'image exacte » tenait pour une grille centrée, et tombait pour la même grille posée par son coin — l'annulation soustrait ce que faire a ajouté (fiche 66, GLISSER-2) | une promesse au bit près s'éprouve sur des données qui l'exercent (plusieurs, choisies pour arrondir) ; si elle ne tient pas en général, l'épreuve dit ce qui est vrai, sans tolérance inventée, et le défaut devient un chantier |
| **Une règle de glisser choisie sur le papier** : la durée d'une case, linéaire à 100 ms le pas, montait de 2 à 14 s en 60 pixels (fiche 67) | demander comment la grandeur se **perçoit** : une durée en rapport — règle logarithmique, arrondi à deux chiffres significatifs, plus de pas à choisir |
| **Un miroir qui compare au « dernier vu »** : le document A → B → A entre deux images, et le champ du panneau gardait B (fiche 67) | après avoir écrit, relire tout de suite la source ; l'épreuve défait avant toute image |
| **Une liste tenue à la main en deux endroits** : les tables de l'arène, pesées et vérifiées, avaient perdu `crop` des deux côtés (fiche 67) | une seule liste, tenue par une déstructuration exhaustive : un oubli ne compile pas |
| **Une épreuve qui ne peint pas entre deux gestes** : à l'écran, le panneau se resynchronise à chaque image (fiche 67) | peindre (`une_image_sans_fenetre`) entre deux mouvements, comme la vraie boucle |
| **Une liste d'épreuve écrite à la main** : l'épreuve du `LISEZ-MOI` comptait onze sortes de lignes ; BOITE-ANDROID en a ajouté deux sans les y mettre, et rien n'est tombé (fiche 66) | lier l'épreuve à la source de vérité — ici la liste fermée du serveur, qui est ce qui part —, jamais à un nombre recopié |
| **Une garde à chaque entrée au lieu d'une règle à la sortie** : quatre mécanismes tenaient la caméra pendant une lecture, chacun aveugle au sabotage — et une entrée oubliée (deux doigts qui pincent) faisait sauter la vue à la fin (fiche 68) | demander ce que l'état promet (« pendant une lecture, la caméra est la sienne »), l'écrire une fois là où il se consomme, et retirer les gardes d'entrée ; l'épreuve regarde **après** la fin |
| **Une épreuve qui croit l'horloge** : un vol vivant ne déplace rien quand le temps de l'épreuve n'avance pas — le sabotage qui laissait le vol en route ne tombait pas (fiche 68) | lire l'état qui promet le mouvement (`vol.en_cours()`), pas seulement la vue |
| **Deux sources pour un même état** : le storyboard courant se lisait sur la case choisie, puis sur le dernier touché — les deux divergeaient après « + Nouveau » (fiche 68) | une seule source ; chaque geste qui change l'une tient l'autre, ou l'autre disparaît |
| **Une page qui prend tout ce qui tombe dans son cadre** : la planche d'un storyboard montrait l'autre storyboard, posé dessous — aucune épreuve ne le disait, l'aperçu si (fiche 68) | regarder l'image produite avant d'y croire ; une page ne prend que ce qui est à elle, par la règle d'appartenance qui existe déjà |
| **Une adaptation par script qui garde la forme et perd le sens** : une lecture construite avec un nom de storyboard vide compilait, et ne lisait rien (fiche 68) | après un remplacement mécanique, lire chaque épreuve qui tombe avant de la « réparer » ; une aide d'épreuve qui exige l'hypothèse (« un seul storyboard ») tombe au lieu de choisir |
| **Une lenteur attribuée sans chronomètre** : « probablement le flou de `resvg` » — juste, mais la cause était dans **notre** SVG (une région de filtre de ±100 000 unités), et tout lecteur la payait (fiche 71) | chronométrer chaque étape avant de supposer ; une lenteur d'épreuve peut être un défaut du produit |
| **Un commentaire qui affirme ce que le code ne fait pas** : « trois écarts-types de chaque côté » au-dessus de ±100 000 ; « les pixels mêmes que `fill_crisp` » au-dessus d'une autre règle (fiche 71) | relire le code sous chaque commentaire qui promet une égalité ; une égalité promise s'obtient en **appelant** la règle, jamais en la récrivant |
| **Une épreuve globale qui dilue ce qui est petit ou peu contrasté** : la superposition du SVG ne voyait ni un texte bleu sur bleu absent, ni un glyphe grossi de 5 %, ni un décalage d'un pixel (fiche 71) | une épreuve globale borne le gros ; chaque petite garde a son épreuve ciblée, jugée si possible par un autre que nous (`fontdue`, `resvg`) |
| **Un brouillon de release déposé sans le dire** : une `v2.0.3-beta.3` du 14/10, sur un commit d'avant le storyboard et l'export SVG, attendait sur GitHub ; la publication du 15/10 a refusé de la remplacer (fiche 71) | une fiche dit chaque brouillon déposé (sa version, son commit) ; avant de publier, lister les releases et lire la cible de celle qu'on va publier |
| **Une commande de sabotage qui ne lance pas l'épreuve visée** : BORNES-2 « aveugle » — son épreuve vit dans `tests/store_suite.rs`, la commande ne lançait que `--lib` (fiche 71) | avant de saboter, lancer la commande sur le code intact et vérifier qu'elle **exécute** l'épreuve visée (son nom dans la sortie) |
| **Une limite du serveur payée en silence par le client** : le serveur refusait toute session de plus de 512 Kio, le client retenait le refus pour toujours — les longues sessions, celles qui comptaient, ne partaient jamais (fiche 72) | une donnée trop grosse se raccourcit avant de partir ; un verdict se retient **avec ce qui l'a causé** (la version), et se retente quand la cause a pu changer |
| **Un essai lancé avant que le déploiement soit arrivé** : le serveur refusait encore, trente secondes après `wrangler deploy` (fiche 72) | attendre la minute de propagation de Cloudflare avant de chercher une cause |
| **Une garde qui cache un défaut** : `MAX_FONT_SIZE` empêchait le gel d'une lettre géante, et réduisait les lettres de près depuis toujours (fiche 72) | quand le coût cesse de dépendre de la grandeur gardée, retirer la garde — et regarder ce qu'elle tordait |
| **Un seuil emprunté** : 256 px, celui de Skia, laissait ×16 au pire ; le croisement mesuré est vers 100 (fiche 72) | mesurer le croisement des deux coûts sur sa machine, écrire le journal des valeurs essayées |
| **Une loi implicite perdue en mettant en table** : le clavier français marchait parce que cinq `match` ignoraient Alt et Maj par endroits (fiche 72) | avant de remplacer un aiguillage, écrire la loi qu'il appliquait sans le dire (ACCORD-1), et l'éprouver par les claviers qu'elle sert |
| **Un nombre écrit de tête dans un document** : « 89 commandes » — le compte est 75 (fiche 72) | compter dans le code, ou s'appuyer sur ce que le type tient (la longueur d'un tableau) |
| **Une épreuve qui reconnaît le cas qu'on avait sous les yeux** : SLUG-1 appelait « bord » un mélange du fond de la carte et de l'encre blanche — sous un passage ou une sélection, chaque bord de lettre devenait une faute, et l'épreuve tombait sur un dessin juste (fiche 73) | enregistrer les deux images, les regarder, puis la carte des écarts **avant** toute hypothèse ; définir la grandeur par ce qu'elle est (un mélange de ses voisins), pas par les couleurs d'un exemple |
| **Une loi qui excuse « au bord » excuse un bord déplacé** : les pixels qu'une lettre décalée d'un pixel déplace sont eux aussi des pixels de bord (fiche 73) | une loi de bord se double d'une **loi de place** — le déplacement d'ensemble par moindres carrés, borné par ce que la construction garantit (le huitième de pixel de GLYPH-1) |
| **Deux lois pour un même bord** : le cadre d'un passage par `tiny-skia` au processeur, par la loi des membranes sur la carte ; et son bord débordait de la lueur découpée par la loi exacte (fiche 73) | une forme a une seule loi, que les deux voies **appellent** ; une fois la géométrie partagée, la comparaison des voies ne la voit plus : une épreuve à part la lit contre la référence (Tauri) |
| **Une affirmation vraie d'avance** : « tout doit repartir » (`envoyees == (0, 0)`) sur un atlas qui n'avait jamais rien envoyé (fiche 73) | mettre l'état dans la situation contraire avant d'affirmer qu'il en sort ; et jouer le nombre d'images que le mécanisme lit vraiment (le resserrement lit l'image d'avant) |
| **Une garde que la mathématique rend vide** : un `abs` devant une fonction paire (fiche 73) | démontrer la propriété, l'écrire dans la doc de la fonction, l'éprouver au bit près, et retirer la garde |
| **Une garde qui ne sert qu'à un cas qu'on ne sait pas produire** : comparer le texte derrière une empreinte, pour les seules collisions (fiche 73, la mémoire des mises en page) — aucune épreuve ne peut la voir | changer de structure pour que la garde disparaisse (la table prend le texte pour clé) |
| **Une optimisation qui écarte ce qui « ne se verra pas »** (LIGNE-1) | l'éprouver par **équivalence** : le résultat avec et sans elle, identique, en balayant les places critiques plus finement que le plus petit débordement mesuré |
| **Deux messages d'affilée dans un emplacement qui n'en tient qu'un** : la ligne fausse et la commande inconnue d'un même fichier — seul le second se voyait (fiche 74) | ce qui se dit en même temps se dit en **un** message ; l'épreuve demande tout ce qu'il doit dire |
| **Une épreuve dont la donnée n'exerce pas le cas** : « `Ctrl+  rien` », prise pour une ligne fausse, se lit très bien (fiche 74) | l'épreuve affirme d'abord qu'elle part du cas voulu (« un fichier protégé »), puis le juge |
| **Un outil d'écriture qui décode les échappements** : `\u00e9\ud83d\ude00` écrit dans une épreuve est devenu « é😀 » — l'épreuve du décodage JSON passait sans rien décoder (fiche 75) ; c'est l'épreuve des glyphes, tombée sur l'émoji, qui l'a dit | construire les échappements dans le code (`let bs = char::from(92u8)`), et relire les octets écrits (`cat -A`) d'une épreuve qui porte des barres obliques |
| **Une épreuve sabotée qui tombait déjà sur le code intact** : deux épreuves du double-clic ont « fait tomber » leur sabotage alors qu'elles tombaient sans lui (la carte d'accueil prenait le clic) — ajoutées après la dernière suite verte (fiche 75) | toute épreuve neuve se lance d'abord sur le code intact, **puis** se sabote |
| **Une épreuve qui se fige au lieu de tomber**, encore : une lecture qui attend une réponse tue ou bloquée (TUYAU-1, RELAIS-1) — deux sabotages « tombés » parce que je les avais arrêtés à la main (fiche 75) | toute lecture qui peut attendre se fait sur un fil, avec un délai (`recv_timeout`) ; un « TOMBE » obtenu en tuant le processus ne compte pas : corriger l'épreuve, rejouer |
| **Une garde dont l'effet est rattrapé par une autre** : sans le suivi des ancres, Glucose retrouve un passage par sa citation et son contexte — l'épreuve qui ne regardait que le texte désigné était aveugle (fiche 75) | éprouver ce que la garde produit elle-même (la place enregistrée de l'ancre), pas seulement le résultat qu'un filet rattrape |
| **Deux fils sur un même tuyau synchrone sous Windows** : l'écriture attend la lecture en cours — le relais se figeait après la première réponse ; l'épreuve passait parce qu'elle écrivait tout avant de lire (fiche 75, TUYAU-1) | un flux lu et écrit par deux fils s'ouvre en E/S superposée ; l'épreuve joue l'ordre réel (une lecture en attente, puis l'écriture) |
| **Une mesure contre un service réel qui ne compte que les fautes graves** : le tri « passait » avec 0 sur 28 — Ollama était mort avec Protéo, chaque réponse illisible, et une réponse illisible ne fait aucune faute grave (fiche 76) | exiger que chaque mesure ait eu lieu (zéro illisible) avant de juger ce qu'elle dit |
| **Une question à plusieurs valeurs, dont le sens tient dans des phrases, posée à un petit modèle** : deux phrases ajoutées aux définitions, et 24 justes sur 28 devenaient 19, neuf fautes graves (fiche 76) | des questions fermées sur un seul axe chacune (vrai/faux), et leur produit écrit dans le code ; ce que le programme sait sûrement (la date), il le décide lui-même |
| **Une règle d'avant qu'un chantier déclenche soudain souvent** : « une demande reprend une mission en pause » dormait ; le tri ouvrant des missions, une mission sur Rennes a pris pour elle trois demandes sans rapport (fiche 76) | relire ce que le neuf réveille dans l'ancien ; l'essai réel enchaîne des demandes **différentes**, pas une seule |
| **Une note du programme dans la voix de l'utilisateur** : la mission annoncée par une longue note `user` ouvrait une conversation où le 7B écrivait ses appels en JSON dans son texte, cinq fois sur cinq (fiche 76) | dire au modèle ce qui s'est passé dans la forme qu'il connaît — un appel d'outil fait pour lui, et son résultat |
| **Un essai réel qui agit sur sa machine** : « crée un serpent dans Blender », envoyée par le pilote, a ouvert Blender deux fois sur son écran et écrit un fichier à la racine de `C:` (fiche 76) | un essai réel de Protéo ne porte que des demandes sans programme à fenêtre, ou tourne sous `PROTEO_CONFINEMENT` ; ce qu'il a lancé se ferme et ce qu'il a écrit se retire, dans la même session |
| **Un pilote d'essai qui abandonne avant le travail** : 25 minutes ne suffisaient pas à une recherche qui dérivait (116 pages) ; la demande suivante partait pendant que Protéo travaillait encore (fiche 76) | lire l'état réel avant de conclure ; un temps dépassé est une mesure à rapporter, pas une fin de tour |
| **Un journal qui ne dit que les échecs** : une toile posée et une toile qui attendait l'accord laissaient le même journal, vide — son essai du 17/10 est resté illisible (fiche 76) | écrire aussi ce qui a été fait (`toile.pose`) : un silence ne doit avoir qu'un sens |
| **Une épreuve qui compare deux voies appelant la même fonction** ne voit pas une faute dans cette fonction (`inverse()` dans la loi et dans la carte) | saboter ce que la règle interdit vraiment (ici : que la carte divise elle-même), et voir la fonction contre une voie qui ne l'appelle pas |
| **Un a priori qui devient le verdict** : la machine des candidats de Protéo ne devait que fixer l'ordre d'examen, mais n'essayait que le premier éligible — et sa loi (la densification, appliquée à chaque modèle comme s'il était à la frontière) préférait le plus récent, un modèle de vision robotique en anglais et chinois (fiche 77 § 1) | quand un ordre décide de ce qui sera mesuré, il est un verdict : le juger comme tel, ou mesurer plusieurs candidats à bas prix avant le banc complet |
| **Un banc aveugle aux modèles récents** : les modèles de 2026 réfléchissent par défaut, et la réflexion épuisait les 32 jetons du tri — 41 réponses illisibles sur 41 (fiche 77, RAISONNEMENT-1) | avant de comparer, vérifier que la mesure **a eu lieu** pour chaque candidat (zéro illisible) ; une question au format imposé ne délibère pas |
| **Une épreuve qui parle en partie au modèle en service** : dans l'épreuve `agent`, seule la boucle visait le candidat ; le tri, la suite, le plan et la recherche visaient le sortant (MODELE-1) | lire au journal **à qui** chaque question est allée ; un cadre de travail porte son modèle, et chaque organe le lit là |
| **Un banc qui favorise le sortant** : la consigne du tri avait été réglée phrase par phrase sur qwen2.5:7b ; sept modèles sur sept ratent les mêmes trois cas (fiche 77 § 6) | dire quel modèle a servi à régler une consigne ; un cas que tous ratent interroge l'étiquette avant les modèles |
| **Une épreuve qui mesure un chemin mort** : `scripts` mesure l'« action directe », que le chat n'emprunte plus — et la règle de bascule refusait tout candidat « moins bon » sur elle | avant de peser une épreuve, demander quel chemin de la production elle exerce |
| **Une recherche qui oublie le catalogue de l'outil** : j'ai cherché les modèles sur le web et Hugging Face, et manqué ceux que la bibliothèque d'Ollama venait d'ajouter (bonsai-2, en poids ternaires) — le seul catalogue que Protéo sait installer sans détour | chercher d'abord dans le catalogue de ce qui exécute, puis ailleurs |
| **Une loi plus belle que le cas** : la valeur marginale de Charnov, quitter quand le rendement tombe sous la moyenne, n'aurait rien arrêté — le rendement en citations ne baissait pas, leur pertinence si (PROGRES-1) | mesurer le gain au but de la chose (ici, faire avancer le statut d'une question), pas à son volume ; reproduire le cas réel dans une épreuve **avant** de choisir la loi |
| **Une campagne restreinte qui mesurait encore ce qu'elle ne jouait pas** : la fenêtre des outils MCP menait un 9B à 262 144 jetons et 16 Go pour une épreuve écartée | ce qu'on écarte s'écarte partout, la préparation comprise |
| **`python -I` ignore les variables d'environnement**, `PYTHONIOENCODING` comprise : un script qui écrit « ✗ » tombait sous Windows | `sys.stdout.reconfigure(encoding="utf-8")` dans le script lui-même |
| **Un échec attribué au modèle sans lire sa réponse** : les deux cas `page` de qwen3.5:4b — « illisible » — étaient coupés à 1 536 jetons pile, la borne de sortie de la boucle, que sa réflexion épuisait (fiche 78, SORTIE-1) | lire au journal la fin de chaque génération ratée (`fin`, jetons de sortie) avant de juger le modèle ; une borne qui ne vient d'aucune mesure se retire |
| **Une épreuve morte qui garde du code en vie** : `scripts` mesurait l'« action directe », et `routes/propositions.rs` ne survivait que pour elle — « exactement celui qu'il était pour que ses mesures restent comparables » (fiche 78) | une épreuve exerce un chemin de la production, ou elle disparaît ; ses cas passent par le chemin vivant |
| **Un signal envoyé quand personne n'écoute** : `watch::Sender::send` n'écrit rien sans récepteur — une annulation pendant le tri était perdue ; l'épreuve passait par la chance de ses 200 ms, et la charge d'une campagne l'a fait tomber (fiche 78, ANNULE-1) | lire la documentation d'un canal jusqu'à son cas « fermé » ; une épreuve de minutage qui tombe sous charge désigne souvent un défaut, pas le minutage |
| **Une taille de contexte par appel** : Ollama recharge un modèle à chaque `num_ctx` neuf — 34 rechargements pour 46 requêtes au banc, le tri à 4 096 jetons entre deux pas de la boucle (fiche 78, FENETRE-1) | un modèle, une fenêtre : mesurée une fois, lue par toute question qui lui est posée ; compter les `starting llama-server` du journal d'Ollama |
| **Un catalogue d'attributs recopié d'une session d'avant** : les `x-test-*` de la bibliothèque d'Ollama n'existent plus ; `/library` liste 256 modèles en une page servie sans script (fiche 78) | relire la page à la source avant de bâtir un lecteur dessus |
| **Des épreuves vertes et des sabotages tous tombés, et rien ne marche en vrai** : la fenêtre témoin tenait la place de `winit` ; le Restart Manager, lui, n'atteignait jamais notre fenêtre (fiche 82) | un essai réel avant « fini » — ici un vrai processus sur un dossier jetable (`LOCALAPPDATA`, `TEMP`, `TMP` redirigés), fermé par `RmShutdown`, sa boîte noire relue |
| **Une bibliothèque qui pose une fenêtre de premier niveau sans jamais lire ses messages** : le moteur OpenGL de `wgpu-hal` — le Restart Manager échoue, l'extinction attend (fiche 82) | énumérer les fenêtres du processus et parler à chacune (`SendMessageTimeout`) : FENETRES-1, une épreuve dans un processus à elle |
| **Ouvrir un moteur qu'on n'emploie pas** : tous les moteurs de `wgpu` d'office, alors qu'il choisit Vulkan dès qu'il existe (fiche 82) | n'ouvrir que ce qui sert, dans l'ordre où ça sert (`present::moteurs`) |
| **Un document qui affirme un défaut déjà corrigé** : SUITE et INVENTAIRE disaient Trans-domaines muet, il dit « bientôt » depuis la fiche 71 (fiche 82) | lire le code (et son épreuve) avant de prendre un chantier |
| **Git Bash convertit les chemins `/mnt/c/…`** passés à `wsl` en `C:/Program Files/Git/mnt/c/…` : rien n'a tourné (fiche 82) | lancer `wsl … glucose-faire` par PowerShell |
| **Un travail qui rend une ressource attendait derrière celui qui la consomme** : l'offre des niveaux passait après six mille décodages — « rien ne l'attend », disait le code ; la mémoire l'attendait (fiche 83) | demander, pour chaque file, ce que son retard coûte, et à qui |
| **Un état intermédiaire que personne ne compte** : « en chemin » n'était ni tenu ni offert, et le journal disait 2,7 Go « inconnus » (fiche 83) | tout état d'une ressource entre dans un compte ; `dumpsys meminfo` (tas natif, « Unknown », propre ou sale) dit lequel manque |
| **Une cause qui dort faute d'appareil** : dix jours sans son téléphone ni son câble (fiche 83) | le téléphone virtuel de ce PC (§ 5), et un tableau fabriqué par le vrai dépôt |

## 5. Les pièges pratiques

* **Git Bash mange les barres obliques inverses** dans un heredoc : écrire un script ou un message
  de commit avec l'outil d'écriture, dans le scratchpad, puis `git commit -F fichier`.
* **Git Bash refuse aussi certains heredocs** (apostrophes, guillemets français) avec « unexpected
  EOF » : tout script Python long s'écrit dans un fichier du scratchpad, puis `python -I fichier`.
  Et Python sous Windows écrit en CRLF : ouvrir avec `newline='\n'` (le dépôt est en LF).
* **Même entre apostrophes (`<<'EOF'`), Git Bash change `\\` en `\`** : trois scripts de la fiche 73
  en ont perdu leurs `\\n` et leurs `\\sum`. La règle qui suit vaut sans exception.
* **Git Bash mange aussi les barres obliques inverses d'un `python - <<'EOF'`** (fiche 56 : des
  `\n` devenus de vrais sauts de ligne dans une chaîne Rust, et un script qui ne s'est jamais
  écrit). Tout script qui en porte s'écrit avec l'outil d'écriture, puis `python -I fichier`.
* **Un fichier Rust qu'aucun `mod` ne nomme n'est pas compilé — mais les cliquets le lisent** :
  on peut préparer un module pendant qu'un sabotage tourne ; il doit déjà respecter les
  cliquets (fiche 56 : deux `fill_rect`).
* **Ne jamais éditer un fichier pendant qu'un sabotage le touche** : le saboteur restaure ce
  qu'il a lu au départ, et effacerait la modification (fiche 54 : elle a survécu, de peu).
* **Git Bash exécute ce qu'on met entre accents graves**, même dans une chaîne destinée à
  Python : `` `docs/INVENTAIRE.md` `` dans un `python -c "…"` a été lancé comme un script
  (fiche 63, sans dégât). Tout texte qui en porte s'écrit par l'outil d'écriture ou d'édition.
* **Les fichiers de sabotage se rangent dans `outils/sabotages/`** (`chrome-1.json`, `p5.json`, `mood-board.json`,
  `storyboard-1.json`, `storyboard-2.json`, `memoire.json`, `memoire-3.json`, `p2-4.json`, et
  depuis la fiche 72 `telephone-72.json`, `grand-1.json`, `zoom-72.json`, `p4.json`, la fiche 73 `p8.json` et `p8-bis.json`, la fiche 74 `p4-utilisateur.json`, la fiche 75 `agents.json` (36 gardes), la fiche 76 `fleche-6.json` (et, pour Protéo, `PROTEO/outils/sabotages-tri-et-toile.json`), la fiche 82 `fin-session-1.json` et `fenetres-1.json`, la fiche 83 `memoire-4.json`, et
  `libere-1.json`, dont la commande passe par `wsl … glucose-lancer` : ce qui ne se joue que
  sous Linux se sabote sous Linux) : le scratchpad disparaît avec la session, et la suivante doit pouvoir les
  rejouer. Un script de remplacement s'écrit **après** `cargo fmt` (sinon il ne trouve plus son
  texte).
* **L'APK signé, depuis ce PC** : `outils/android/construire.sh` avec, **en chemins Windows**,
  `JAVA_HOME=C:/Program Files/Java/jdk-21.0.12`, `ANDROID_HOME=C:/Users/Administrator/Android/Sdk`,
  `GRADLE=C:/Users/Administrator/Android/gradle/gradle-9.8.1/bin/gradle.bat`, et
  `GLUCOSE_KEYSTORE` / `GLUCOSE_KEYSTORE_PASSWORD` lus dans `Documents\Glucose-cle-android` ;
  vérifier l'empreinte (`apksigner verify --print-certs` : `3f2af005…`) avant de le poser sur
  son Bureau.
* **Android** : les objets de construction dans `target/android` (`CARGO_TARGET_DIR`) pour ne pas
  recompiler le bureau ; `cargo ndk -t arm64-v8a -P 21 clippy -p glucose-android -p
  glucose-desktop -- -D warnings` voit ce que Windows ne compile pas (les doigts, `ureq`).
* **`SetWindowSubclass` empêche le programme de démarrer** sans manifeste des contrôles communs v6
  (`STATUS_ENTRYPOINT_NOT_FOUND`) : le sous-classement passe par `SetWindowLongPtrW` (fiche 53 § 2).
* **PowerShell ne distingue pas les majuscules** dans les noms de variables (`$tauri` est
  `$Tauri`) : un script d'épreuve s'y est trompé.
* **Les épreuves tournent en parallèle** : deux qui écrivent le même fichier temporaire tombent
  au hasard (nommer par l'empreinte du contenu).
* **Une épreuve graphique se saute sans carte** ; sur GitHub, `GLUCOSE_EXIGER_UNE_CARTE` la fait
  tomber au lieu de mentir en vert.
* **Une épreuve de temps** qui tombe sur une machine de GitHub dit que la manière doit changer,
  jamais que la borne doit monter.
* **Les épreuves de la mémoire offerte** (`plateforme::offre`, le `Ctrl+B` qui attend ses
  originaux) tombent sous une forte pression mémoire : le système jette alors les pages offertes.
  Les rejouer seules avant de conclure (fiche 59).
* **Linux, sur ce PC : la distribution WSL `Glucose-Linux`** (Ubuntu 24.04, à part de
  NixOS-Proteo, qui est un autre projet). Un utilisateur ordinaire `glucose`, Rust stable, et un
  serveur X **invisible** (Xvfb `:99 -noreset`) — rien ne s'affiche sur son écran ; WSLg est
  écarté (`DISPLAY`, `WAYLAND_DISPLAY`). Deux commandes, en `root` :
  `glucose-lancer '<commande>'` recopie le dépôt de Windows (`rsync`, sans `target`) puis lance
  la commande dans la copie ; `glucose-faire <sortie.txt> <commande>` fait de même et écrit tout
  dans le fichier, fini par `CODE <n>`. Les épreuves de la sélection X11 veulent
  `GLUCOSE_ECRAN_X_D_EPREUVE=:99` (et `GLUCOSE_EXIGER_UN_SERVEUR_X=1` pour tomber sans lui).
  Exemple : `wsl -d Glucose-Linux -u root -- glucose-faire /mnt/c/…/sortie.txt cargo clippy
  --workspace --all-targets -- -D warnings`. `cargo check --target …-linux-gnu` depuis Windows ne
  passe toujours pas (`ring`).
* **La distribution grossit** : un dossier `target` y pèse près de 30 Go. Pour rendre la place à
  Windows : `fstrim -av` dedans, `wsl --shutdown`, puis `diskpart` (`select vdisk file=…ext4.vhdx`,
  `attach vdisk readonly`, `compact vdisk`, `detach vdisk`) — le disque en mode normal, pas
  « clairsemé », que `diskpart` refuse (fiche 61 : 58 Go ramenés à 33).
* **Le Mac ne se compile pas ici** : seule la CI le fait.
* **Compiler pendant qu'il essaie** (fiche 76) : un exécutable ouvert ne se remplace pas sous
  Windows, mais **il se renomme** — `target/release/glucose-desktop.exe` **et** son lien dur
  `deps/glucose_desktop.exe` (`ls -i`, `find -inum`), puis `cargo build --release` ; son Glucose
  continue, le suivant sera le neuf. Pour Protéo, `proteo.exe` est un lien dur vers
  `target/release/proteo.exe` : compiler ailleurs (`CARGO_TARGET_DIR` hors du dépôt — un dossier
  dans le dépôt apparaît à git). Jamais tuer ce qu'il a ouvert.
* **Regarder ce qu'un agent pose, sans fenêtre** (fiche 75) : `cargo build -p glucose-desktop --bin
  glucose-desktop --example porte_sans_fenetre`, puis, **par PowerShell** (Git Bash déforme
  `\\.\pipe\…`), `porte_sans_fenetre.exe \\.\pipe\<nom> <arrêt> <dossier>` ; l'agent se branche
  par `glucose-desktop.exe --mcp` avec `GLUCOSE_PORTE=\\.\pipe\<nom>`. Créer le fichier `<arrêt>`
  rend le tableau, chaque bulle et une flèche survolée en PNG. Protéo : `PROTEO_GLUCOSE=<exe>`, et
  ses épreuves à la main `rejouer_une_vraie_recherche_sur_la_toile` (`PROTEO_RAPPORT=<preuve>`) et
  `recherche_neuve_posee_en_direct` (`PROTEO_QUESTION=…`).
* **Un témoin de format s'écrit par la build d'avant** (fiche 65, `riche-v2.glucose`) : un
  arbre de travail détaché au dernier commit qui produit ce schéma (`git worktree add --detach
  <scratchpad>/vN HEAD`), une épreuve jetable qui y écrit la fixture, sa propre cible
  (`CARGO_TARGET_DIR`), puis `git worktree remove --force`. Un témoin écrit par la build du
  jour ne prouverait que sa propre idée de l'ancien schéma.
* **Lire le téléphone par le câble** (fiche 66) : `bash outils/android/lire_le_telephone.sh
  <dossier>` — la raison du dernier redémarrage, les morts de Glucose qu'Android a notées
  (`exit-info`), la boîte des incidents, les journaux, la mémoire, le pilote, et un rapport de
  bogue complet. Rien n'est écrit sur le téléphone, et Glucose n'y est pas ouvert.
* **Sans câble** (il a perdu le sien, fiche 71) : le **débogage sans fil** d'Android 11 et après
  (`adb pair <adresse:port>` et son code à six chiffres, puis `adb connect <adresse:port>`, un
  autre port) — le même script marche. Il demande que téléphone et PC soient sur **un même
  Wi-Fi** : quand le PC passe par le partage de connexion du téléphone, c'est probablement
  refusé. Le repli : l'APK dans un zip, envoyé par Drive (Discord refuse plus de 10 Mo sans
  abonnement), extrait par l'application Fichiers ; la télémétrie dit le reste.
* **Le téléphone virtuel de ce PC** (fiche 83) : l'appareil `glucose-telephone` (Android 14
  x86_64, 4 Go, 1080 × 2400), lancé sans fenêtre — `~/Android/Sdk/emulator/emulator.exe -avd
  glucose-telephone -no-window -no-audio -no-boot-anim -no-snapshot -gpu host` en fond, prêt
  quand `adb shell getprop sys.boot_completed` rend 1 (son processus s'appelle
  `qemu-system-x86_64-headless`). L'APK : `GLUCOSE_PROCESSEURS=x86_64 bash
  outils/android/construire.sh`, puis `adb install -r`. Sous Git Bash, **`MSYS_NO_PATHCONV=1`
  devant `adb`**. Les données de Glucose : `adb shell run-as com.glucose.app …` dans
  `files/glucose/` ; un document se dépose par `adb push` dans `/data/local/tmp`, puis `run-as …
  cp` (un brouillon : `files/glucose/brouillons/<nombre>-1.glucose`). Le grand tableau :
  `cargo run --release -p glucose-desktop --example fabriquer_un_grand_tableau -- <sortie> 6000`.
  Répondre **Non** à la question du journal technique : le téléphone virtuel n'envoie rien.
* **Le pilote Vulkan du téléphone** : `adb shell cmd gpu vkjson` (dans `lire_le_telephone.sh`,
  `vulkan.json`) dit ses extensions — `VK_EXT_memory_budget` y est-il ? vulkan.gpuinfo.org ne
  répond pas hors de son navigateur (fiche 67).
* **Le jeton de `wrangler` expire** : une erreur 7403 (« account not authorized ») sur une
  requête à la base se règle par `npx wrangler whoami`, qui le rafraîchit.
* **Le serveur avant le client** : une ligne neuve de la boîte noire se déploie d'abord
  (`npx wrangler deploy` dans `outils/telemetrie`), se prouve en ligne une minute plus tard (une
  session jetable, puis son effacement, puis la base relue — le script de la fiche 72), et
  seulement ensuite part dans une version : un client neuf devant un serveur ancien voit chacune de
  ses sessions refusée.
* **Les épreuves du serveur** : `node --test outils/telemetrie/test/valider.test.js` — nommer le
  fichier, le dossier seul échoue sur ce Node. **Cloudflare refuse un client sans en-tête
  d'identité** (erreur 1010) : un essai en ligne donne un `user-agent`, s'efface ensuite
  (`DELETE /v1/installations/<id>`), et la base se relit pour le prouver (fiche 60).
* **`cargo test --workspace` compile beaucoup** : `target/` a atteint 94 Go avant le rangement du
  06/10. `cargo clean` de temps en temps ne coûte qu'une recompilation.

## 6. À la fin d'une session

1. Mettre à jour [ÉTAT](ETAT.md) (ce qui est vrai maintenant) et cocher [SUITE](SUITE.md).
2. Si le travail est important, écrire une fiche datée dans le carnet (numéro suivant), et sa ligne
   dans [`carnet/00-INDEX.md`](carnet/00-INDEX.md). Le code cite les fiches par leur numéro : on
   ne renumérote jamais.
3. Lui donner **une seule** liste d'essais à l'écran, courte, avec la commande qui écrit la sortie
   dans un fichier.
4. Ne jamais écrire un jeton dans le dépôt.
