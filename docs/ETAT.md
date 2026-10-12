# Où en est Glucose

> **Au 12/10/2026**, après la session de ses retours sur la V2 (fiche 51), celle de la
> carte de l'écran (fiche 52), celle de Pinterest et du pavé (fiche 53), celle de la similitude
> des doigts, de la télémétrie et d'Android (fiches 54 et 55), celle du partage vers Glucose
> et des documents du téléphone (fiche 56), celle du clavier, de l'appui long et des
> documents qu'on renomme (fiche 57), celle des questions dessinées partout, des noms du
> rail et des emojis (fiche 58), celle de ses retours du 09/10 — la question au-dessus des
> panneaux, le canevas qui suit, le copier unique (fiche 59) —, et celle de ses essais du soir :
> les dossiers, la carte au journal, le presse-papiers tenu, et le recul du copier sous Linux et
> Mac qui retient la beta 3 (fiche 60), et celle du copier unique sous Linux, la voie B qu'il a
> choisie (fiche 61), et celle des boutons qui ne faisaient rien — les tris par la couleur, l'export
> en image, les Presets et leurs zones, les champs du dock (fiche 62). Ce document dit l'état **vérifié** du projet,
> en une lecture. Il se réécrit à chaque fin de session qui change quelque chose d'important ;
> l'historique, lui, vit dans le [carnet](carnet/00-INDEX.md).

---

## 1. En une page

* **Glucose Rust est publié.** La **2.0.2-beta.2** est la release « latest » du dépôt depuis le
  09/10/2026 (construite sur `06f4217`, la fiche 58 — relevé sur GitHub le 09/10 au soir, aucun
  document ne le disait) : installeur Windows, AppImage, `.deb`, `.rpm`, **l'APK Android**, et
  `nix run github:shazamifius/GlucoseGit`. La 2.0.2-beta.1 (07/10) et la 2.0.1-beta.1 (30/09) la
  précèdent. **Les fiches 59 et 60 ne sont dans aucune publication** : ce sera la 2.0.2-beta.3.
* **Glucose Tauri bascule de lui-même.** Chaque Glucose Tauri installé (depuis la 1.0.1-beta.9)
  trouve la 2.0.1 dans le même `latest.json`, par le popup habituel. Ses documents s'ouvrent dans
  Glucose Rust, et ses données ne sont jamais touchées. Cette bascule a été répétée de bout en bout
  sur GitHub : onze secondes de Tauri ouvert à Glucose Rust vivant.
* **Ce qu'on sait des utilisateurs : presque rien.** Au 06/10, l'installeur Windows a été téléchargé
  **2 fois**, les paquets Linux **0 fois**, et `latest.json` a été lu 14 fois, dont une bonne part par
  sa propre machine. **La télémétrie est en ligne** (fiche 54 § 6) : le serveur tourne dans son
  compte Cloudflare (`glucose-boite-noire.ferme-nilslamber.workers.dev`, base D1 en Europe de
  l'Ouest), éprouvé de bout en bout sur Internet ; Glucose y envoie avec l'accord de chacun,
  demandé au premier lancement. **Le 08/10 au soir, huit sessions y sont arrivées**, dont quatre
  de son téléphone (fiche 57 § 5) : la question dessinée a donc marché chez lui.
* **Android : Glucose tourne sur son moto g54 5G** (« oui, ça fonctionne sur téléphone », 08/10 ;
  **c'est lui qui a montré les rayures** ; une puce graphique PowerVR, pas une Mali — fiche 61
  § 6 ; le Redmi 9 des documents n'a jamais été essayé) —
  ARM 64 et 32 bits, Android 5.0 et après, avec les doigts. L'APK est signé de **la clé de
  Glucose pour Android** (créée le 08/10, `Documents\Glucose-cle-android`, et dans les secrets
  du dépôt) : la publication le joint à la release.
* **Le téléphone, fiche 56 — écrit, éprouvé hors écran, jamais essayé sur un téléphone** :
  Glucose dans « Partager » (images, et liens d'épingles rapatriés), la surface qu'Android
  reprend et rend, deux touchers sur le vide pour le menu, « Ajouter des images… » par le
  sélecteur de photos, la question dessinée (le journal technique, refusé à sa place par
  l'ancien Glucose, se reposera), les documents rangés sous « Canevas N » et rouverts par une
  liste.
* **Le téléphone, fiche 57 — écrit, éprouvé hors écran, jamais essayé sur un téléphone** :
  **le clavier** (le miroir de la saisie : on écrit enfin au téléphone), **l'appui long** (le
  clic droit du doigt, au délai que règle l'accessibilité d'Android), **renommer, dupliquer,
  supprimer** un document par un appui long dans la liste, et **le bord à bord** (l'interface
  évite la barre d'état, la navigation et le clavier, que Android 15 et 16 imposent).
  Un APK de cette version, signé de sa clé, est sur son Bureau (`Glucose.apk`).
* **Fiche 58 — « tout fonctionne » (09/10)** : **toutes les questions se
  dessinent dans Glucose**, au bureau aussi (POPUP-1 : ses écrans noirs venaient d'une boîte du
  système posée avant la première image) — Entrée, Tab, Échap ; **le rail écrit le nom de chaque
  icône** ; **les emojis se dessinent** (Noto Emoji monochrome, en repli d'Inter). Puis, à sa
  demande : **le journal technique lisible** (`journal-technique`, un `LISEZ-MOI.txt` en
  colonnes) et **les gestes à plusieurs doigts** (annuler, rétablir, choisir plusieurs nœuds).
* **Fiche 59 — ses retours du 09/10, écrits et éprouvés hors écran, jamais essayés par lui** :
  la question passe **par-dessus les panneaux**, et aucun panneau ne s'ouvre au lancement
  (DECISION-1) ; **le canevas défile** quand on tient un nœud près du bord (DEFILE-1) ; **un seul
  « Copier »**, qui emporte les images elles-mêmes — une en PNG, plusieurs en fichiers —, et le
  **glisser vers Discord, Google Docs ou le bureau** (COPIER-1) ; jamais une texture plus grande
  que la carte n'accepte (PLAFOND-1). Ses captures du téléphone étaient en partie celles de
  l'APK du 07/10 (sans appui long ni bord à bord) ; **les rayures** de l'APK actuel ne se
  reproduisent sur aucun moteur de ce PC (Vulkan, GL, DX12) : la cause est probablement propre au
  téléphone, et son journal par le câble la dira. **Un APK neuf**, signé de sa clé (`3f2af005…`),
  est sur son Bureau (`Glucose.apk`), et l'aperçu de la question au-dessus des panneaux dans
  `Glucose apercus`.
* **Fiche 60 — ses essais du 09/10 au soir** : **au PC, tout marche** (aucun panneau ; le
  copier vers Discord « absolument impressionnant », textes, post-it et images ; recoller et
  glisser parfaits) — sauf **le défilement au bord, « moche, pas fluide »**, qu'il remet à plus
  tard. **Au téléphone**, la question passe au-dessus et « Supprimer » marche, **pas sur un
  dossier** : corrigé (DOSSIER-1, en local — l'APK de son Bureau ne le porte pas). **La carte
  graphique** part désormais au journal technique par ses numéros (CARTE-1, en local ; le
  serveur redéployé et vérifié en ligne). **Sous Linux**, « tant de choses ne marchent pas » :
  Glucose lâchait le presse-papiers sitôt la copie faite, qui disparaissait là où aucun
  gestionnaire ne la reprend (TENU-1, en local, à prouver sur son Linux). **Et COPIER-1 fait
  reculer Linux et Mac** (plus de « Copier l'image » vers les autres applications) : la
  2.0.2-beta.3 attendait son choix entre A et B (fiche 60 § 3) : **il a choisi B**.
* **Fiche 61 — le copier unique sous Linux** : Glucose sert lui-même la sélection de X11 — le
  lot, le texte, l'image seule ou les fichiers des images, `INCR`, `MULTIPLE`, l'heure du
  serveur, la remise au gestionnaire du presse-papiers à la fermeture ; un bloc se recolle d'un
  Glucose à l'autre. **Compilé, passé à clippy et éprouvé sous Linux sur ce PC** (une
  distribution WSL à part, un serveur X invisible, `xclip` pour demander) : neuf épreuves, quinze
  sabotages, vingt tours verts. **Jamais essayé sur son Linux** (son NixOS était éteint). Le Mac
  attend son tour (aucune version Mac n'est publiée).
* **Fiche 62 — les boutons qui ne faisaient rien** (« tous les boutons sont obligatoires, je
  n'en délaisserai pas un seul ») : **les trois tris par la couleur** d'Ordonner (Oklab, la
  moyenne exacte de chaque image) ; **l'export en PNG** de la sélection ou du tableau, à 100 %,
  par bandes ; **les Presets** — les six gabarits de Tauri, des zones dessinées sur les deux
  voies, posées par un fantôme, prises par leur bandeau, et **« Par slot preset »** qui range
  chaque zone en elle-même ; **les champs de nombre du dock**, qui ne répondaient à rien (glisser
  ou taper), et le champ « Colonnes » d'Ordonner, perdu au portage. **Neuf commits en local, non
  envoyés.** Tout est éprouvé et saboté, **rien n'est encore jugé à son écran**. Ce qui reste en
  façade : fiche 62 § 8 — d'abord **le Storyboard**.
* **La parité avec Glucose Tauri est loin d'être atteinte.** Environ un quart du logiciel au dernier
  comptage (24/09), mais le cœur est solide : document, rendu, images, flèches, texte.
* **Il ne peut dépenser aucun argent.** Pas de certificat Windows, pas de compte Apple : l'iPad passera
  par le web, Android par un APK installé à la main.

## 2. Ses utilisateurs

| qui | combien | sur quoi | ce qui les attend |
|---|---|---|---|
| utilisateurs de Glucose Tauri | 5 | Windows et Linux | la bascule automatique (faite) |
| testeurs qui attendent Glucose Rust | une vingtaine | dont **10 sous Android** | le portage Android |
| une personne qui compte énormément pour lui | 1 | **iPad** | Glucose dans Safari |

## 3. Ce que fait Glucose Rust aujourd'hui

| domaine | ce qui marche |
|---|---|
| **Naviguer** | canevas infini ; **le canevas défile quand on tient un nœud, un coin ou un rectangle de sélection près du bord** (DEFILE-1, fiche 59) ; **à la souris, instantané** (molette, glisser au bouton du milieu : rien ne glisse) ; pavé tactile **par *Direct Manipulation*** : la similitude entière des doigts, sans bascule (fiche 54 § 3 — Windows zoome bien sous le curseur, **vérifié** ; le biais **à juger à son écran**), `F` (vol qui cadre la sélection, ou tout), `Ctrl+F` (tout), signets `Ctrl+1..9` / `1..9`, minimap qu'on tient, onglets (renommer, ranger, supprimer, importer un document dedans) ; **au doigt, sur un écran** hors Windows (fiche 54 § 7) |
| **Le document** | **`Ctrl+N`** (un document vierge) ; `.glucose` qui s'écrit **geste après geste** (plus de gel à l'enregistrement), images scellées dans le fichier, brouillons, texte en cours de frappe qui survit à un plantage, reprise au lancement (le dernier document, le curseur), Time Machine (`Ctrl+H` : regarder, restaurer, jalons datés), documents Tauri lus par un lecteur écrit ici |
| **Images** | PNG, JPEG, WebP, GIF, BMP ; collage, dépôt de fichiers, **dépôt depuis un navigateur** sous Windows (Pinterest en pleine résolution, la page demandée **compressée** ; un repli en lien **dit pourquoi**, et se rattrape au clic droit : « Remplacer par l'image ») ; rotation, recadrage non destructif, `Ctrl+B` (bordures) ; mémoire par étages |
| **Texte** | Markdown, tableaux, liens, **LaTeX** fidèle à KaTeX, annulation mot par mot |
| **Flèches** | l'aspect de Tauri, droite ou courbe, double sens, épaisseur, six relations, coudes, **contournement** des obstacles, **ancres de texte** (une flèche part d'une phrase précise) |
| **Copier, coller** | **la sélection entière** (textes, images, flèches, membranes) par `Ctrl+C`/`Ctrl+X`/`Ctrl+V`, d'une fenêtre de Glucose à l'autre, et **glissée** de l'une à l'autre ; **un seul « Copier »** (COPIER-1, fiche 59) : le lot pour Glucose, le texte, et les images elles-mêmes — une seule en PNG, plusieurs en fichiers, écrits au premier collage qui les veut — pour Discord, un navigateur, Paint ; **le glisser emporte les images en fichiers** (Discord, Google Docs, le bureau) ; au clic droit sur une image : **Enregistrer l'image sous…** (les octets d'origine). **Sous Linux** (X11, et Wayland par XWayland), le même copier unique, servi par Glucose (fiche 61) — sans le glisser vers une autre application ; sous Mac, le lot seul |
| **Organiser** | membranes qui **possèdent** ce qu'on y dépose, mode Focus, dossiers, domaines, aimant aux voisines, placement aimanté dès le premier clic ; **Ordonner** : cinq dispositions, huit tris dont **trois par la couleur** (fiche 62), des champs qui se glissent ou se tapent ; **les Presets** : six gabarits, leurs zones posées, déplacées, redimensionnées, retirées, et « Par slot preset » qui range chaque zone |
| **Exporter** | Markdown, SVG (sans les images ni les membranes), et **PNG** : la sélection ou le tableau, à 100 % (fiche 62) |
| **Fenêtre** | **le mode référence**, à la PureRef : `Ctrl+Maj+A` — sans interface, sans cadre, au premier plan, retenu à la relance ; déplacée au bouton droit, redimensionnée par un bord |
| **Mise à jour** | automatique, signée, au format de Tauri, sous Windows et Linux ; **sous Android, écrite** (fiche 58 § 6 : l'APK confié à l'installeur du système) — la 2.0.2-beta.2 la porte ; pas encore prouvée : il faut qu'une version suivante arrive à un téléphone qui a la beta 2 |

## 4. La qualité, mesurée

* **Épreuves** : **2 334** sous Windows, toutes vertes (fiche 62 ; la suite Linux de WSL n'a
  pas été rejouée depuis la fiche 61, où elle comptait **2 255**) ; clippy strict à zéro sous
  Windows et pour Android. **Les neuf commits de la fiche 62** (`64b5922` à `82911ea`) sont
  **en local**, non envoyés. Fiche 61 —
  la suite tourne désormais **aussi sous Linux sur ce PC**, dans la distribution WSL
  `Glucose-Linux` (METHODE § 5) ; clippy strict à zéro avec Rust 1.99, sous Windows, Linux **et
  pour Android** ; la fiche 61 a saboté quinze gardes, quinze chutes.
  **Les commits des fiches 60 et 61** (`abae87b` à `4deba20` : DOSSIER-1, CARTE-1, TENU-1, les
  cliquets tenus, le copier unique sous Linux et leur documentation) **sont partis le 12/10,
  à sa demande, sans la CI** (« envoyer tout sur GitHub, mais pour l'instant pas avec la CI ») :
  le dernier porte `[skip ci]`. Rejoués avant l'envoi : 2 261 épreuves sous Windows, clippy
  strict sous Windows et pour Android. **Linux et Mac ne sont donc pas vérifiés par GitHub** : la
  2.0.2-beta.3 attend un envoi avec la CI. **Les six commits de la fiche 59 sont partis avec la CI**
  (`3fbf383`, exécution 37946507360) : **les dix tâches vertes**, Linux et les deux Mac compris.
  **Sur GitHub, à
  chaque envoi, dix tâches** : Windows, Linux, Mac à puce Apple, Mac Intel, le noyau pour Android
  et le web, **l'application Android et son APK** (nouvelle, jamais lancée), NixOS, l'installeur et
  la mise à jour, la répétition de la bascule, les paquets Linux. **Les dix vertes sur
  `89510e2`** (08/10 ; la tâche des paquets Linux, figée une heure et demie à installer ses
  dépendances chez GitHub, relancée) ; **les commits suivants du 07/10 ont été envoyés sans CI**, à sa demande
  (`[skip ci]`), et ceux de la fiche 53 aussi, à sa demande (« je ne veux absolument pas que tu
  déclenches la CI », 07/10 au soir) : Linux et Mac **à vérifier avant la publication** (relus,
  fiche 53 § 4 ; seule la CI le prouvera). `a32e074`, `efab62f` et la documentation qui suit
  sont envoyés **sans CI** eux aussi, à sa demande (« tu peux parfaitement les envoyer sans
  déclencher la CI », 07/10 au soir), jusqu'à `d17e838` — vérifié : aucune exécution. Ceux de
  la fiche 54 sont partis **avec** la CI (`89510e2`, dix vertes) ; la télémétrie branchée et
  l'APK de la publication, **sans** (« fais pas marcher la CI », 08/10), jusqu'à `df1b854`
  (`[skip ci]`, vérifié : aucune exécution). Le rail, les fiches 55, 56 et 57 — douze
  commits, jusqu'à `57060c8` — sont partis **avec** la CI, à sa demande (08/10 au soir) : **les
  dix tâches vertes**, Linux et Mac compris (exécution 37801814665). **Les commits de la fiche 58** sont partis
  le 09/10, à sa demande, avec la CI (`2f4a9a3`, exécution 37836775328) : **neuf tâches vertes,
  la répétition de la bascule rouge** — son témoin était planté dans l'ancien dossier du journal
  (fiche 58 § 9). Deux envois plus tard, **`06f4217` : les dix tâches vertes** (exécution
  37850791104), Linux et les deux Mac compris.
* **Sa machine** (Windows 11, RTX 5070 Laptop et Intel Arc 140T, écran 240 Hz à 150 %), session du
  05/10 : une image coûte **5 à 7 ms** en médiane. Le pire monte à **11-21 ms pendant les zooms
  chargés** et à 31 ms au décodage de l'ouverture : le plancher de 100 images par seconde n'est
  **pas encore tenu partout**.
* **Le lag du 07/10 au matin** (gels de 300 à 500 ms dans `present`, pour des images de 3 ms) :
  son portable a l'écran branché sur la RTX, et Glucose dessinait sur l'Intel, qui n'affiche
  rien et s'endort au repos. L'arbitre alternait d'une session à l'autre. Depuis ECRAN-1
  (fiche 52), Glucose dessine sur la carte qui tient l'écran — **vérifié** sur sa session
  suivante : la RTX, `mailbox`, pire image 48 ms au lieu de 505.
  Les gels de l'Arc des fiches 43 et 21 avaient très probablement la même cause.
* **Aucun plantage ni gel** de Glucose noté par Windows depuis la publication.
* **Au repos**, aucun réveil spontané : la chronique du 07/10 le dit raison par raison — seuls
  dessinent les messages qui s'effacent et les glissades qui finissent.

## 5. Ce qui manque encore (de Tauri)

Rideaux · temporalité (réglette de −10 000 à 2 100) · **storyboard** (le prochain) · un preset
personnel (« Créer un preset custom ») · l'export HTML, et le SVG complet · miroirs et dossiers miroirs du disque · recherche (`Ctrl+F`) · couleur des domaines sur le
canevas · vidéos · provenance des images (SauceNAO) · sélecteur de couleur · collaboration · MCP ·
plugins et IA locale · Mac · iPad. **Android** : tourne sur son téléphone ; les questions s'y
dessinent (fiche 56) ; le clavier, l'appui long et la gestion des documents sont écrits (fiche
57), le bord à bord aussi ; **pas encore** de presse-papiers du système, de mise à jour
automatique prouvée ; la batterie est écrite (fiche 58) ; les emojis se dessinent (fiche 58), sans les
familles ni les drapeaux composés. **Les tablettes Windows** : les doigts n'y passent
pas encore (la souris simulée à écarter d'abord). Les membranes « étirées » et « minimisées » (l'idée de
Mary) **attendent sa parole**, comme les rideaux et Trans-domaines.

## 6. Ses retours sur la V2 — jugés à son écran le 07/10

Fiches 51 et 52. **Tout ce qui suit est vu et approuvé à son écran** (« tout est absolument
parfait », 07/10 au soir), sauf où c'est dit :

* le copier-coller de nœuds, entre deux fenêtres et glissé de l'une à l'autre ; « Copier
  l'image » dans Discord et « Enregistrer l'image sous… » ;
* `Ctrl+N`, qui **demande toujours** (NOUVEAU-1) ;
* le mode référence (`Ctrl+Maj+A`) : sans interface ni panneaux, au premier plan ; au pavé,
  `Alt` + glisser déplace la fenêtre et `Alt` + pincer la redimensionne (une fois par image,
  REFERENCE-3) ; les signets retiennent le **centre** de la vue (SIGNET-2) ;
* les poignées, le cadre et l'anneau de sélection qui suivent la taille des nœuds (POIGNEE-1) ;
* la molette : trois crans doublent ;
* **le lag réglé** (ECRAN-1, la carte qui tient l'écran).

**Ouvert** :

* **le pincement au pavé**, par *Direct Manipulation* (fiche 53 § 2) : « c'est bien, le zoom
  maintenant » (07/10, 15 h 40) ;
* **le déplacement à deux doigts** : « la translation, la multi-direction pose problème ». Cause
  trouvée dans sa chronique — dix-neuf déplacements pris pour des pincements, bloqués jusqu'à la
  fin du geste ; corrigé par la similitude entière (fiche 54 § 3) — **son ressenti attendu** ;
* **la coupure du pavé de 7 à 10 secondes**, aussi dans Blender : cause inconnue ; la boîte noire
  en note désormais chaque étage (fiche 54 § 4) — **la prochaine coupure dira où** ;
* **sur batterie**, la cadence est tombée à 30 images par seconde par moments (fiche 54 § 5) :
  probablement le bridage de la RTX, à confirmer ; la charte demande de pixeliser plutôt ;
* **le cadre du groupe qui restait sur place** pendant un glisser : corrigé (GROUPE-1) — à son
  écran ;
* **transformer une sélection entière** (sa demande « comme Blender ») : poignées du groupe,
  `Alt` + coin, bouton « Origine commune / individuelles » (fiche 53 § 10) — **à son écran** ;
* **Pinterest** : six épingles devenues six liens le 07/10 (fiche 53 § 1) — la page arrivait
  nue, 1,2 Mo au lieu de 127 Ko compressée. Corrigé : « ça fonctionne, mais plutôt long » → la
  copie se pose désormais en 0,3 à 0,9 s, et l'original la remplace (fiche 53 § 9) ;
* **le dessin de la sélection** : « pas esthétique du tout ». Cinq directions **envoyées à son
  écran** (`docs/carnet/screens/53-selection-cinq-directions.png`) ; mon avis : E, un cadre
  pour le groupe et un fil fin par image (PureRef, Figma, tldraw) — **son choix** ;
* le gel qui a « failli planter » son PC : cause probable, la rafale de redimensionnements de
  `Alt` + pincer, corrigée (REFERENCE-3) — pas revu depuis ;
* l'opacité et les clics qui traversent du mode référence, s'il les veut ; et ce qu'il regrette
  d'avoir perdu en mode référence (« dommage que tu les aies supprimés, c'était vraiment
  pratique ») — à lui demander.

**Il navigue surtout au pavé tactile**, et a une souris, essayée le 07/10.

## 6 bis. Ses retours du 09/10 — fiche 59

* **La question du journal technique cachée sous Ordonner et Pomodoro** : corrigé sur les deux
  voies (DECISION-1), et aucun panneau au lancement — vu sur l'aperçu du téléphone, **à son
  téléphone** ;
* **pas de « Supprimer » au téléphone** : l'APK de ses captures n'avait ni appui long ni marges ;
  dans l'actuel, l'appui long sur une membrane offre « Supprimer » (une épreuve le joue) et la
  liste des documents aussi — **à confirmer** ;
* **les glitchs** (blocs blancs, rayures) : les blocs blancs venaient de l'ancien APK ; les rayures
  de l'actuel ne se reproduisent pas ici ; PLAFOND-1 retire une cause possible ; **son journal
  par le câble** dira la carte et ses refus ;
* **le canevas qui ne suit pas un glisser** : DEFILE-1 — **à son écran** (la vitesse est un nombre
  de ressenti) ;
* **un seul copier, et une image reste une image** : COPIER-1 — **à son écran**, dans Discord et
  Google Docs ;
* **la provenance des images** (son point 1) : la suite, chantier 6 — il faudra sa clé SauceNAO ;
* **l'ergonomie** (le rail qui prend tout l'écran, les boutons, les fenêtres) : **après**, à sa
  demande — un chantier de conception, avec des planches.

## 6 ter. Ses essais du 09/10 au soir — fiche 60

* **Jugés à son écran** : aucun panneau au lancement ; le copier vers Discord (une et trois
  images, les textes, les post-it) ; recoller le bloc dans Glucose ; le glisser vers Discord,
  Docs et le bureau — **sous Windows** ; au téléphone, la question au-dessus des panneaux et
  « Supprimer » sur une membrane.
* **Le défilement au bord** : « moche, pas fluide, pas très pratique » — remis à plus tard par
  lui ; sa chronique (16 % des images au bord, un tressaut de 64 px au p99) est à relire avec.
* **Les dossiers au téléphone** : corrigés (DOSSIER-1), **à son téléphone** avec un APK neuf.
* **Son Linux** (2.0.2-beta.2) : « tant de choses ne marchent pas » — TENU-1 en est une cause
  probable ; **la liste est à lui demander**. C'est une machine NixOS à part, `nixos.home` sur son
  réseau (pas NixOS-Proteo) ; **éteinte pendant la fiche 61** : aucun accès, aucun essai.
* **Le recul de COPIER-1 sous Linux et Mac** : il a choisi **B** ; fait sous Linux (fiche 61),
  le Mac ensuite.
* **Au repos, sa chronique redessine pour les messages** (371 images, 1,6 par seconde) : la
  charte dit zéro — à reprendre.

## 7. Points de vigilance

* **Sa clé de signature** est dans les secrets du dépôt (elle a signé chaque version de Tauri) ;
  **il l'a sauvegardée** hors de ce PC (« ma clé est sauvegardée », 12/10).
* `%TEMP%\glucose_pasted` : vide (relevé le 12/10). Les brouillons de release de juillet et la
  branche `feat/mcp-glucose-bridge` sont **retirés de GitHub** (12/10, à sa demande) : il reste
  `main` et `tauri-v1.0.1`. Une branche locale, `secours/avant-reattribution`, garde 83 commits d'avant la
  réécriture des auteurs : une sauvegarde, à garder.
