# La suite, dans l'ordre

> **Au 14/10/2026** (fiches 59 à 66). La feuille de route de Glucose Rust après la V2. Elle remplace les plans des
> fiches 36, 44 et 46 du [carnet](carnet/00-INDEX.md), qui en gardent le raisonnement. Chaque
> chantier dit **pourquoi il est là**, et ce qui le déclare fini. Tout y est à remettre en question.
>
> **Comment s'en servir** : prendre le premier chantier non fait. Quand il est fini (prouvé, pas
> « ça compile »), le cocher ici, mettre à jour [ÉTAT](ETAT.md), et passer au suivant.

> **Son ordre, le 07/10 au soir** (fiche 53 § 7) : *« une version ultra stable avec tout le mode
> Ctrl+Maj+A, compatible tablette, Linux, Windows et Mac ; un build complet pour tous les Linux,
> tous les Mac, et SURTOUT Android ; et une télémétrie de tous les utilisateurs, quel que soit
> leur appareil »*. D'où l'ordre ci-dessous : finir le chantier 1 (le déplacement au pavé, la
> sélection, la CI, la 2.0.2), puis **Android** (chantier 2), les versions complètes pour Linux
> et Mac (chantier 3), la télémétrie (chantier 4, qui attend sa décision sur le serveur).
>
> **Le 08/10 (fiche 54)** : la télémétrie est écrite des deux côtés et n'attend que son compte
> Cloudflare ; Android compile, avec les doigts, et attend son premier téléphone — l'APK se
> construira sur GitHub au prochain envoi avec CI.

---

## 0 octies. Le programme — **d'abord** (sa parole du 13/10 au soir, fiche 65 § 13)

Ses mots : les huit propositions du Storyboard **validées** ; **le téléphone, c'est bien Glucose
qui a figé tout le système** (même le bouton d'alimentation), après des saccades qui montaient,
puis un redémarrage seul — jamais vu avec une autre application ; et **le fantôme toujours en
haut à gauche du curseur**. Dans cet ordre :

| | chantier | ce qui le déclare fini |
|---|---|---|
| [~] | **P0 — ANDROID-MEMOIRE** — **fait côté code (fiche 66)** : MEMOIRE-1 (la mémoire se mesure sous Android, une ligne `memoire` par record dans la boîte noire qui voyage, le serveur redéployé), MEMOIRE-2 (`onTrimMemory` → la carte rend ce qui se reconstruit, le magasin offre tout derrière une autre application), MEMOIRE-3 (aucun décodage n'entre s'il ne tient pas dans la moitié de ce que la machine laisse ; décodé dans la page de la pyramide : 445 → 254 Mo pour une photo de 50 Mpx). **Reste** : lire le téléphone par le câble (`outils/android/lire_le_telephone.sh`), un APK neuf, et sa longue session | **son téléphone** (ci-dessous) |
| [ ] | **P0 (le libellé d'origine) — Glucose ne doit jamais figer un téléphone.** D'abord **lire** : le téléphone branché (débogage USB) — `adb shell getprop sys.boot.reason`, `adb shell dumpsys dropbox --print`, `adb logcat -b crash`, puis Glucose ouvert et `adb shell dumpsys meminfo com.glucose.app` pendant qu'on navigue. Puis la recherche (`onTrimMemory`/`ComponentCallbacks2`, `ActivityManager.getMemoryInfo`, `VK_EXT_memory_budget`, ce que font Chrome et les moteurs de jeu sur Android), et **la mémoire par étages sous Android** : un budget lu du système, relu en continu, `onTrimMemory` qui fait rendre les textures, jamais plus que ce que la machine laisse — la charte (« chaque étage borné par ce que la machine laisse **maintenant** »). Les saccades qui précèdent sont un signal à lire (POSTES-1, BOITE-ANDROID) | **son téléphone** : une longue session sans saccade qui monte, ni gel ; ses sessions arrivées et lues (le tableau de bord, et l'écart « redémarré après ») ; une épreuve qui simule la pression mémoire et exige que les textures se rendent |
| [~] | **P1 — FANTOME-1** — **fait (fiche 66 § 6 bis)** : la grille armée du Storyboard part du curseur par le coin haut gauche de sa première case, au survol comme au clic (la loi du centrage de Tauri retirée) ; la membrane et le dossier l'étaient déjà, une épreuve le garde ; regardé sur l'aperçu, éprouvé à la vraie souris, 3 sabotages | **à son écran** |
| [ ] | **GLISSER-2 — défaire un glisser exactement** (trouvé par P1, fiche 66 § 6 bis) : un glisser s'écrit en un pas, et défaire soustrait ce que faire a ajouté — une image revient à deux ulps de son départ. Invisible à l'œil ; mais l'annulation exacte demande au journal les départs eux-mêmes (une édition neuve : `DOCUMENT_VERSION` monte, un témoin écrit par la build d'avant) | quand il touchera au journal, avant 10⁷ nœuds |
| [~] | **P2 — Les huit propositions du Storyboard, validées** — **les 8 écrites** : 1 à 3 (fiche 66) ; 4, 5, 6 (fiche 67 : la durée et l'animatique, les champs, le plan) ; **8 — plusieurs storyboards par tableau (fiche 68)** : la case qui sait son storyboard, SEQUENCE-1, le storyboard courant, « ‹ 2 / 3 › », « + Nouveau », en un **schéma 5** (il avait lancé la build du schéma 4 : témoins v4 de `5bb82ec`) ; **7 — l'export en planches (fiche 68)** : la planche est le canevas, ses rangées déduites du format A, PNG ou PDF écrit à la main, rendu au pixel près par MuPDF et PDFium. Éprouvé, saboté (`p2-4`, `p2-8`, `p2-7`), regardé hors écran. **À sa parole** : la planche sombre comme le canevas, ou claire pour l'impression ? les champs (dialogue, action, notes) sous chaque case, comme Storyboarder ? le numéro « 2.3 » quand il y a plusieurs storyboards ? | **à son écran** (fiche 68 § 9) ; un bouton au PC et au téléphone (INVENTAIRE) |
| [ ] | **P3 — Ce que son téléphone dira** : POSTES-1 (lire les lignes `poste`), puis DESSUS-1 et RASTER-1 si elles désignent leur coupable | les postes lus, le coupable nommé par la mesure |
| [ ] | **P4 — « Créer un preset custom »** (INVENTAIRE § 2) — **attend sa parole** : Tauri n'ouvrait qu'une alerte « bientôt » ; deux sens possibles — enregistrer les zones du tableau telles qu'on les a disposées (leur géométrie : à mettre dans le schéma 5 tant qu'aucune build ne l'a porté chez lui), ou inventer un gabarit de zéro (un éditeur de catégories) | branché, enregistré dans le document, saboté |
| [ ] | **P5 — L'export SVG complet** — **commencé (fiche 69)** : une sœur vectorielle pour chaque dessin (membranes, images, cartes, post-it, flèches, libellés, badges), branchée sur « Exporter » ; **ni regardé dans un lecteur, ni saboté** ; manquent les dossiers, les textes des zones et des cases, les réglettes, les formules en vecteurs | fiche 69 § 3, dans l'ordre |
| [ ] | **P6 — Relire chaque panneau et chaque menu contre Tauri, champ par champ** : Pomodoro, Domaines, Time Machine, les menus du clic droit, la bande | la liste de ce qui manque, puis fait |
| [ ] | **P7 — Les fonctions de Tauri qui n'existent pas encore** (INVENTAIRE § 3), une session chacune environ : **la recherche** (`Ctrl+F`, qui l'attend), les opérateurs des post-it (ET, OU, MAIS, PARCE QUE — ils se dessinent déjà), la couleur des post-it et des domaines (un sélecteur de couleur), filtrer par domaine, les miroirs, la temporalité, l'export HTML, le mode zen, les vidéos, puis — à sa parole — les rideaux, les membranes étirées, Trans-domaines | chacun à son écran |
| [x] | **P8 — La 2.0.3-beta.2** : **publiée** par lui le 13/10 à 14 h 50 UTC (sur `82c774b`, CI dix vertes) ; il l'a dit le 14/10 | 14/10 |
| [ ] | **P8 bis — La 2.0.3-beta.3** : P0 (et ce qui suivra), quand son téléphone aura tenu une longue session ; la CI, puis le brouillon ; son clic | après P0 |
| [ ] | **P9 — Le Mac publié** ; **Collaborer** (obligatoire, plus tard) ; **l'UX du téléphone**, toute refaite, en dernier | |

## 0 septies. Le téléphone, le Storyboard tracé (fiche 65)

Ses retours du 13/10 après-midi : le téléphone a **redémarré en urgence** ; `Ctrl+F` à la
recherche, tout cadrer à `Alt+F` ; le Storyboard et la membrane qui ne suivent pas au survol et
ne se tracent pas ; le storyboard qui se remplit selon un rectangle ; « Décrire » qui n'écrit
rien ; des propositions pour un outil « pas pratique et incomplet ».

| | chantier | où en est-il |
|---|---|---|
| [ ] | **Le téléphone qui redémarre** — **c'est Glucose** (sa parole : tout le système figé, le bouton d'alimentation compris, après des saccades qui montaient) | programme **P0** (§ 0 octies) |
| [x] | **Saboter STORYBOARD-1** — 24 sur 25 ; l'aveugle (« poser ne numérote pas ») est gardée par STORYBOARD-2 | fiche 65 § 2 |
| [~] | **STORYBOARD-2** — la grille dans le document (schéma 3, témoin v2), une suite et une loi, le rectangle tracé, la poignée, la case lâchée qui prend la place, Tab vers la case suivante | **à son écran** (fiche 65 § 12) |
| [~] | **TRACE-1** — la membrane et le dossier se tracent ; un clic pose comme avant | **à son écran** |
| [~] | **CHAMP-QUESTION-1** — le champ d'une question écrit au clavier du PC (accents compris) | **à son écran** : « Décrire » |
| [~] | **`Alt+F`** tout cadrer ; `Ctrl+F` gardé à la recherche | **à son écran** |
| [~] | **Le tableau de bord de la télémétrie** (sa demande du 13/10) : `outils/telemetrie/tableau.mjs` résume la base, une page privée de son compte la montre — installations, systèmes, cartes par leurs noms, fins | fiche 65 § 8 bis — **à ses yeux** |
| [~] | **BOITE-ANDROID** : passer derrière s'écrit ; une session tuée en arrière-plan finit ordinairement, et un vrai plantage s'y distingue ; le serveur redéployé | fiche 65 § 8 bis — **à son téléphone**, avec un APK neuf |
| [x] | **Les propositions du Storyboard** (fiche 65 § 9) — **toutes validées** (13/10 au soir) | programme P2 (§ 0 octies) |

## 0 sexies. La chrome, le Storyboard (fiche 64)

| | chantier | où en est-il |
|---|---|---|
| [x] | **Saboter STORYBOARD-1** : 24 gardes sur 25 tombent ; l'aveugle est désormais gardée (fiche 65 § 2) | 13/10 |
| [~] | **CHROME-1** — la carte pose la chrome ; le rectangle de la vue fidèle à Tauri ; `present/passes.rs` (`6e53d09`, 21 sabotages) | **à son écran** : rien ne doit changer à l'œil, sauf la minimap |
| [~] | **STORYBOARD-1** — le panneau entier, les cases sur les deux voies, une image ajustée dans sa case, la case glissée avec ses images, décrite, retirée — **repris par STORYBOARD-2** (§ 0 septies) | **à son écran et à son téléphone** |
| [ ] | **Les postes du téléphone** : qu'il ouvre Glucose une fois au téléphone, puis lire les lignes `poste` (fiche 64 § 1) | **après** la lecture du téléphone qui a redémarré (§ 0 septies) |
| [ ] | **DESSUS-1** — la couche du dessus balayée en entier à chaque image, même vide (0,7-0,8 ms à 2560 × 1600) : que chaque passe dise ce qu'elle dessine, vérifié contre le balayage en `debug_assert` | **si les postes du téléphone le désignent** (`relever`) |
| [ ] | **RASTER-1** — les textes rendus sur plusieurs fils (fiche 63 § 7) | idem |
| [ ] | **Le panneau Storyboard prend toute la hauteur** (comme la Time Machine) : le borner à son contenu demanderait que le dock connaisse le nombre de cases | à juger à son écran |
| [x] | **La 2.0.3-beta.1** : **publiée** par lui (13/10, 8 h 26 UTC, sur `7c60593`) ; la suite est la 2.0.3-beta.2 (§ 0 octies, P8) | 13/10 |
| [~] | **Un tableau de bord de la télémétrie** (les installations, les cartes par leurs noms) — sa demande du 13/10 « voir les statistiques et les PC » | **fait** (fiche 65 § 8 bis) |

## 0 quinquies. Le flou, chaque bouton, et la 2.0.3-beta.1 — **d'abord** (fiche 63)

Ses mots, le 12/10 au soir : des textes et des membranes « qui restent ultra floutés, même pour
du texte — des carrés qui ne souhaitent pas du tout se déflouter » ; « un bouton pour absolument
chaque fonctionnalité, sur PC ou téléphone » ; « regarde les performances » ; et publier la
2.0.3-beta.1 quand « absolument tout sera parfait ».

| | chantier | où en est-il |
|---|---|---|
| [x] | **NETTETE-1** — une image qui n'est pas finie demande la suivante (`c94cc60`) | **jugé** (13/10) : « plus du tout de flou, très clean » |
| [~] | **ACCES-1** — chaque commande dans le clic droit, au PC et au téléphone ; les signets sans clavier ; le menu en colonnes (`aabd7da`) | **à son écran** |
| [x] | **PARTAGE-2** — exporter et enregistrer une image au téléphone, par la feuille de partage (`aabd7da`) | **jugé** (13/10) : « a l'air de parfaitement fonctionner » |
| [~] | **POSTES-1** — la boîte noire dit quelle étape ralentit un épisode trop lent (`939eab7`, serveur redéployé) | la session de son essai partira à sa **prochaine ouverture** au téléphone (fiche 64 § 1) |
| [x] | **La CI** sur `7c60593` — **dix vertes** (exécution 38306469681), Linux et les deux Mac compris | 13/10 |
| [x] | **La 2.0.3-beta.1** — **publiée** par lui le 13/10 au matin (sur `7c60593`, onze fichiers) | la 2.0.3-beta.2 suit (§ 0 octies) |
| [~] | **CHROME-1** — la bande, le rail, la minimap posés par la carte | **fait** (`6e53d09`, fiche 64 § 2) — à son écran |
| [ ] | **RASTER-1** — les textes rendus sur plusieurs fils, comme Chromium (chaque fil sa typographie) | idem |
| [ ] | **Trans-domaines dit « bientôt »** comme Collaborer — le seul bouton qui ne dit pas qu'il ne fait rien | **à sa parole** (fiche 63 § 9) |
| [ ] | **Mac** : une version publiée, et la mise à jour qui sait la remplacer | chantier à part (§ 3) |

## 0 quater. Les boutons qui ne faisaient rien (fiche 62)

Ses mots, le 12/10 : *« il y a encore des dizaines d'options et de boutons qui ne fonctionnent
pas »* ; *« 100 % des boutons existants, ainsi que toutes les fonctionnalités à venir, sont
quasi obligatoires — je n'en délaisserai pas un seul »*. **Ne jamais proposer d'en retirer un.**
La méthode qui marche : relire le composant de Tauri **champ par champ** à côté du panneau de
Rust (c'est elle qui a trouvé le champ Colonnes perdu et le libellé faux d'Ordonner).

| | chantier | où en est-il |
|---|---|---|
| [~] | **ORDONNER-2** — les trois tris par la couleur (Oklab, la moyenne exacte de la pyramide) | `64b5922` — **à son écran** |
| [~] | **EXPORT-PNG-1** — la sélection, ou le tableau, en PNG, à 100 %, par bandes | `9fca9ca` — **à son écran** ; la taille (100 % ou pleine résolution) **à sa parole** |
| [~] | **PRESET-1** — six gabarits, zones sur les deux voies, posées par un fantôme, prises par leur bandeau ; « Par slot preset » | `c442ea2` à `367fa73` — **à son écran** |
| [~] | **CHAMP-1** — les champs de nombre du dock (glisser, ou cliquer et taper) ; « Colonnes » d'Ordonner | `82911ea` — **à son écran** |
| [x] | **L'envoi** des neuf commits (`64b5922` à `82911ea`, plus la documentation) | **fait le 12/10** (« urgent », `58ad6ea`) ; sa règle désormais : envoyer sans la CI, la CI avec raison |
| [~] | **STORYBOARD-1** — le panneau entier : poser la grille par un fantôme, dessiner les cases sur les deux voies, une image posée dans une case s'y ajuste, la description qui se tape, ajouter, retirer, renuméroter, « ✕ » ; les cases prises par leur bandeau | **écrit** (fiche 64 § 3) — à saboter, envoyer, puis à son écran |
| [ ] | **« Créer un preset custom »** — les zones du tableau enregistrées comme gabarit du document (`project.presets` se lit déjà dans le panneau) | après le Storyboard |
| [ ] | **L'export SVG complet** — les images et les membranes | |
| [ ] | **Relire chaque panneau et chaque menu contre Tauri, champ par champ** : Pomodoro, Domaines, Time Machine, les menus du clic droit, la bande | la liste exécutable de ce qui manque |
| [ ] | **Les champs du dock au téléphone** : glisser marche, taper non — le miroir du clavier (CLAVIER-1) ne sert ni les champs de nombre, ni le nom d'un jalon, ni celui d'un domaine | avec **l'UX du téléphone, toute à refaire** (sa demande : ultra rapide, fluide, pratique), l'une des dernières étapes |
| [ ] | **Collaborer** — « absolument obligatoire » | **plus tard, à sa parole** |
| [ ] | Trans-domaines ; l'IA locale (« Télécharger » du panneau Plugins) | plus tard |

## 0. Ses demandes du 08/10 au soir — **avant tout le reste**

Ses mots, après la fiche 57 : les 12 commits envoyés **avec la CI** ; la clé Android copiée hors
du PC (« c'est fait, c'est enregistré ») ; la publication de la 2.0.2-beta.1 seulement **si la
mise à jour automatique fonctionne pour tout le monde**.

| | chantier | pourquoi, et ce qui le déclare fini |
|---|---|---|
| [x] | **POPUP-1 — toutes les questions dessinées dans Glucose, partout** (urgent) — **« tout fonctionne » (09/10)** : la croix, `Ctrl+N`, `Ctrl+O`, la mise à jour, le journal technique ; Entrée, Tab, Échap ; une seule boîte du système reste, la mise à jour cherchée avant la fenêtre (DIAL-5) | *« souvent des black screen total et des freeze ; il faut voyager dans le noir total, faire Tab puis Entrée pour autoriser la télémétrie »*. Les dialogues du système (`rfd`, `dialogue::oui_ou_non`, `oui_non_ou_annuler`, `poser`, appelés sous `sous_un_dialogue`) bloquent la boucle pendant qu'ils sont ouverts : la fenêtre ne se repeint plus. La question dessinée (QUESTION-1, `ui::question`) existe déjà et marche au téléphone : la faire servir partout (`ui.questions_dessinees` à vrai), et convertir les questions encore synchrones — le travail non enregistré (`persist/close.rs`, BROUILLON-1, qui attend une réponse avant de fermer), la mise à jour (`app/lancement.rs`). Les sélecteurs de **fichiers** (ouvrir, enregistrer sous, exporter, ajouter des images) restent ceux du système : à lui de dire s'ils posent le même problème. Fini quand plus aucune question oui/non ne passe par `rfd`, et quand il l'a jugé à son écran. |
| [~] | **GESTES-1 — des raccourcis à deux et trois doigts** — acceptés le 09/10, **écrits (fiche 58 § 9)** : deux doigts touchés annulent, trois rétablissent ; l'appui long prend, puis lever ouvre le menu, glisser trace un rectangle, un autre doigt ajoute des nœuds ; **à son téléphone** | *« impossible de faire une multi-sélection et un clic droit ; avec 2 doigts on crée des raccourcis, et avec 3 aussi ; le plus simple possible, le moins de boutons et de pages possible »*. Attention : deux doigts **déplacent et pincent** déjà la vue (la similitude, fiche 54) — un toucher à deux doigts (sans mouvement) est libre, comme dans Procreate (deux doigts = annuler, trois = rétablir) ; chercher ce que font Procreate, Infinite Painter, Concepts, tldraw, Figma au doigt, et lui proposer **une** carte de gestes courte avant d'écrire. |
| [~] | **Les noms dans le rail** : une icône et son nom — il est d'accord ; « tout fonctionne » (09/10), le téléphone compris ? | **écrit (fiche 58 § 3)** : colonne après colonne, sans défiler, les icônes seules quand la place manque ; **à son téléphone** |
| [~] | **Les emojis** : la police monochrome Noto Emoji (OFL) en repli d'Inter — d'accord | **écrit (fiche 58 § 5, `decisions/10`)** : 887 Ko, Unicode 15 ; les familles et les drapeaux se décomposent ; **à son téléphone** |
| [~] | **Les sélecteurs de fichiers ne tiennent plus Glucose** (DIAL-2, sa réponse « d'accord avec tout », 09/10) : ouverts sur un fil à eux, Glucose se dessine derrière ; « Enregistrer » un travail sans nom à la fermeture attend le choix de l'endroit | fiche 58 § 9 — **à son écran** |
| [x] | **Le journal technique lisible** (sa remarque du 09/10) : le dossier `journal-technique`, des fichiers à l'heure de leur session, un `LISEZ-MOI.txt` en colonnes ; rien de ce qui est parti ne repart (joué sur une copie de ses données) | fiche 58 § 9 |
| [~] | **La mise à jour automatique, vérifiée pour tout le monde** avant de publier : Windows et Linux (la bascule est répétée par la CI) ; **Android : écrite (MAJ-ANDROID-1, fiche 58 § 6)** — l'APK confié à `PackageInstaller`, signé de la clé de Glucose et inscrit au manifeste. **Ne se prouve qu'à une publication** : le premier Glucose qui la porte s'installe à la main, le suivant arrive seul ; la 2.0.2-beta.1 en brouillon ne la porte pas (à republier) | sa condition à la publication |

## 0 ter. La 2.0.2-beta.3 — **d'abord** (fiche 60)

Sa demande du 09/10 au soir : *« envoie tout sur GitHub avec la CI, plus la publication de la
2.0.2 beta 3 »*. L'envoi est fait (dix tâches vertes sur `3fbf383`) ; la publication est retenue
par un recul trouvé en relisant COPIER-1.

| | chantier | où en est-il |
|---|---|---|
| [~] | **Le recul de COPIER-1 sous Linux** : **la voie B, son choix** — Glucose sert la sélection de X11 lui-même (`selection_x11`, `decisions/11`) : le lot, le texte, l'image seule ou les fichiers, `INCR`, `MULTIPLE`, l'heure du serveur, la remise au gestionnaire | **écrit, éprouvé contre un vrai serveur X** (fiche 61 : neuf épreuves, quinze sabotages, vingt tours) ; **à son Linux** : Discord, un bloc d'un Glucose à l'autre, après fermeture |
| [ ] | **Le recul de COPIER-1 sous Mac** : `NSPasteboard` à plusieurs types, les fichiers fournis à la demande | aucune version Mac publiée : personne ne recule ; **le prochain chantier du copier**, après la beta 3 (fiche 61 § 7) — seule la CI le compile |
| [~] | **DOSSIER-1 — un dossier choisi a son menu et sa barre** (« les dossiers ça ne fonctionne pas ») | en local (`abae87b`) ; **à son téléphone, avec un APK neuf** — celui de son Bureau ne le porte pas |
| [~] | **CARTE-1 — la carte graphique au journal technique, par ses numéros** | en local (`e869805`) ; le serveur **redéployé et vérifié en ligne** ; la première vraie session la montrera dans la base |
| [~] | **TENU-1 — le presse-papiers du système tenu toute la vie de Glucose** (cause probable de « tant de choses ne marchent pas » sous Linux) | en local (`b72965a`) ; **à son Linux** |
| [ ] | **Son Linux** : une machine NixOS à part (`nixos.home`), pas NixOS-Proteo — **éteinte pendant la fiche 61** ; la liste de ce qui ne marche pas, et l'accès SSH qu'il croit ouvert | **son NixOS allumé**, puis la liste ; pour compiler et éprouver, la distribution `Glucose-Linux` de ce PC suffit (METHODE § 5) |
| [~] | **L'envoi** des commits des fiches 60 et 61 : **fait le 12/10, à sa demande, sans la CI** (`[skip ci]`). Reste la CI — la tâche Linux éprouvera la sélection contre son Xvfb —, puis la publication (Actions → Publier, `2.0.2-beta.3`, en brouillon) ; son clic « Publish release » | la CI et la publication **à sa parole** |

## 0 bis. Ses retours du 09/10 — **les problèmes d'abord, l'ergonomie ensuite** (fiche 59)

Son conseil, suivi : *« ne t'y concentre pas d'abord ; corrige plutôt les problèmes — l'ergonomie,
la praticité, ce sont des domaines beaucoup plus longs et complexes »*.

| | chantier | où en est-il |
|---|---|---|
| [x] | **DECISION-1 — la question par-dessus les panneaux**, sur les deux voies ; **aucun panneau au lancement** | **jugé** au PC et au téléphone (09/10 au soir, fiche 60) |
| [x] | **« Supprimer » au téléphone** — l'APK de ses captures (07/10) n'avait ni appui long ni marges ; l'actuel les a | **« ça fonctionne »** (09/10 au soir) — sauf sur un dossier : DOSSIER-1 (§ 0 ter) |
| [ ] | **Les rayures du téléphone** — non reproduites sur ce PC (Vulkan, GL, DX12) ; PLAFOND-1 (jamais une texture plus grande que la carte) fait ; le journal du téléphone dit sa carte au lancement | **« pour l'instant rien, je surveillerai »** (09/10 au soir) ; si elles reviennent : le débogage USB, puis `adb logcat -s Glucose` ; la ligne « carte » part au journal technique (CARTE-1). **C'est le moto g54 5G** (sa réponse, fiche 61) — une **PowerVR BXM-8-256**, pas une Mali : d'abord la version du pilote (fiche 61 § 6) |
| [ ] | **DEFILE-1 — le canevas défile quand on tient un nœud près du bord** (une bande, la cible d'un doigt ; *Push-Edge* lu et écarté) | **« moche, pas fluide, pas très pratique »** (09/10 au soir) — remis à plus tard par lui ; à reprendre avec sa chronique |
| [x] | **COPIER-1 — un seul copier, et une image reste une image** : le lot, le texte, les images (une en PNG, plusieurs en fichiers fabriqués à la demande) ; le glisser vers Discord, Docs, le bureau | **sous Windows, jugé** : « absolument impressionnant », recoller et glisser « parfaits » (09/10 au soir) ; **Linux et Mac reculent** (§ 0 ter) ; Android : le presse-papiers du système (§ 2) |
| [ ] | **La provenance des images** (son point 1 ; la fiche 27, étapes 1 à 4) : la page d'origine gardée au dépôt, l'empreinte perceptuelle mesurée sur ses documents (une copie), « Trouver l'origine… » par SauceNAO, la provenance dans le document | **sa clé SauceNAO** pour l'étape 3 ; la carte « Origine » lui sera montrée en image avant d'être écrite |
| [ ] | **L'ergonomie** — le rail qui prend tout l'écran, les boutons, les fenêtres | **après**, à sa demande : des planches, regardées par lui |
| [~] | **L'export PNG de la sélection** — son idée de repli (les images composées dans la disposition de l'auteur) | **fait** (EXPORT-PNG-1, fiche 62) — à son écran |

## 1. Ses retours sur la V2, et la 2.0.2 bêta — **d'abord**

Fiches 51 et 52. Au soir du 07/10, **presque tout est jugé à son écran** : « tout est absolument
parfait ». Restent le pincement, le dessin de la sélection, et la publication.

> **La CI n'a pas vu les commits du 07/10 après `20deb6f`.** Il a demandé l'envoi **sans**
> déclencher la vérification (le dernier commit porte `[skip ci]`). Or ils touchent du code
> propre aux autres systèmes (`plateforme/ecran.rs`, `plateforme::glisser_un_lot`) : **avant la
> publication**, lui proposer un envoi qui lance la CI, et en lire le résultat.

`[x]` fini et prouvé · `[~]` écrit, éprouvé hors écran, **attend son jugement à l'écran** · `[ ]` à faire

| | chantier | où en est-il |
|---|---|---|
| [x] | **La version de travail en `2.0.2-dev`** | l'épreuve compare à tout ce qui est publié (sabotée : elle tombe) |
| [x] | **La souris instantanée** : deux portes dans l'élan, la souris montrée à l'image suivante | essayée à la souris le 07/10 : « tout est absolument parfait » ; il navigue surtout au pavé |
| [x] | **Copier, couper, coller des nœuds, d'une fenêtre à l'autre** : le lot `.glucose`, le format « Glucose.Lot », le collage au curseur en un geste ; le glisser entre fenêtres (`DoDragDrop`) | **« absolument parfait »**, et le glisser entre deux fenêtres aussi (07/10) |
| [x] | **Clic droit sur une image : « Copier l'image » et « Enregistrer l'image sous… »** | **« parfait »**, Discord compris (07/10) |
| [x] | **`Ctrl+N` : un nouveau document**, et l'entrée du menu | **demande toujours** (NOUVEAU-1) : « fonctionne parfaitement » (07/10) |
| [x] | **Le mode référence, à la PureRef** : `Ctrl+Maj+A`, sans interface, sans cadre, au premier plan, retenu ; **au pavé, `Alt` + glisser déplace la fenêtre et `Alt` + pincer la redimensionne** (REFERENCE-2) ; il se désagrandit en entrant | « absolument et vraiment sublime » (07/10), signets compris. Plus tard, s'il le veut : opacité, clics qui traversent |
| [x] | **Le repos à zéro image** : la chronique du 07/10 le dit raison par raison | **aucune raison spontanée** : les messages qui s'effacent et les glissades qui finissent |
| [x] | **La CI de `20deb6f`** (la voie hors Windows du presse-papiers) | neuf tâches vertes |
| [x] | **ECRAN-1 — la carte qui tient l'écran** : le lag du 07/10 (gels de 500 ms dans `present`) venait d'un balancier de l'arbitre entre la RTX, qui tient son écran, et l'Intel (fiche 52 § 1) | **vérifié** sur sa session suivante : la RTX, « celle qui tient l'écran », `mailbox`, pire image 48 ms au lieu de 505 |
| [x] | **POIGNEE-1 — les poignées, le cadre et l'anneau de sélection suivent la place du nœud** à l'écran (son retour du 07/10, deux fois) | « parfait » (07/10) |
| [x] | **Le pincement au pavé** : *Direct Manipulation* (chantier 1 bis, fiche 53 § 2) | « dans l'idée c'est bien, le zoom maintenant » (07/10) |
| [~] | **Le déplacement à deux doigts** : dix-neuf déplacements pris pour des pincements et bloqués (sa chronique du 07/10) ; la similitude entière des doigts, sans bascule (fiche 54 § 3) | Windows zoome sous le curseur (**vérifié**, 2 px) ; **son ressenti** sur le biais |
| [~] | **GROUPE-1 — le cadre du groupe suit le groupe** pendant le geste (fiche 54 § 2) | **à son écran** |
| [ ] | **La coupure du pavé de 7 à 10 s**, aussi dans Blender | la boîte noire en note chaque étage (fiche 54 § 4) : **la prochaine coupure le dira** |
| [~] | **Pinterest, DEPOT-WEB-6** : la page compressée, un repli qui dit pourquoi, « Remplacer par l'image », une session gardée, les variantes en course, **la copie posée tout de suite puis remplacée par l'original** (fiche 53 § 1, 7, 9) | la copie en 0,3 à 0,9 s ; **à son écran** |
| [~] | **`F` cadre la sélection (ou tout), `Ctrl+F` tout** — sa demande (fiche 53 § 7) | éprouvé par la vraie touche ; **à son écran**. `Ctrl+F` prend la place de la future recherche : à lui dire |
| [x] | **La molette** : un tiers d'octave par cran (« pas du tout assez rapide » au huitième) | « parfait » (07/10) |
| [x] | **Le mode référence, suite** : les panneaux retirés aussi de la voie graphique ; SIGNET-2 (les signets retiennent le centre) ; REFERENCE-3 (la fenêtre se redimensionne une fois par image — le gel qui a « failli planter » son PC, cause probable) | « parfait » (07/10) ; le gel n'est pas revenu |
| [~] | **Transformer une sélection entière**, origine commune ou individuelle, comme Blender (fiche 53 § 10) : poignées du groupe, `Alt` + coin pour tourner, bouton dans la barre | **à son écran** |
| [ ] | **Le dessin de la sélection** : il penche pour **E** (un cadre et un fil), C et B ses préférées — mais « leur design ne me convient pas encore » (fiche 53 § 10) | **à concevoir avec lui** : planches de variantes de E |
| [x] | **Le glisser vers une autre fenêtre** : la fenêtre d'origine se peint avant de partir, et ne fait plus tourner un cœur à vide | « parfait » (07/10) |
| [ ] | **Publier la 2.0.2-beta.1** : notes écrites (`outils/publication/notes.md`) ; son geste (Actions → Publier, `2.0.2-beta.1`, brouillon coché, relire, publier) | **après** ses essais à l'écran |

## 1 bis. Le pincement par *Direct Manipulation* — **écrit le 07/10, à juger à son écran**

**Pourquoi** : son jugement du 07/10, après trois réglages — « légèrement trop lent, et pas
fluide, comme s'il sautait », alors que PureRef et un navigateur pincent « instantanément, de
manière fluide ». Glucose reçoit le pincement d'un pavé de précision sous la forme que Windows
donne aux applications qui ne savent pas mieux : un `Ctrl` + molette fractionné, par paquets
(fiche 33, `interactions/pincement.rs`). Aucun réglage de gain ni de lissage ne rend la
continuité qu'il a perdue en route. Chromium et Blender prennent le pavé par *Direct
Manipulation* (`IDirectManipulationManager`, viewport, `DM_POINTERHITTEST`) : l'échelle, le
déplacement et l'inertie arrivent tels que le doigt les fait, à la cadence du pavé (fiche 51
§ 1, ses sources).

**Fini quand** : à son écran, le pincement est « instantané et fluide » comme dans PureRef ; et
tout ce qui arrive encore en molette vient d'une molette (ce qui règle aussi le doute des roues
libres). **Fait** (fiche 53 § 2, `decisions/07`) : reste son écran — le geste, le sens du
déplacement, l'élan. Une recherche d'abord : le code de Chromium (`direct_manipulation_helper_win.cc`) et le
commit de Blender cité en fiche 51 § 1 ; puis une dépendance éventuelle à défendre dans
`carnet/decisions/` (la caisse `windows` a la fonctionnalité `Win32_Graphics_DirectManipulation`).

## 1 ter. Le dessin de la sélection — **la planche lui est envoyée (fiche 53 § 3)**

**Pourquoi** : « pas esthétique du tout » (07/10). `style.md` impose une chrome monochrome,
héritée de Glucose Tauri. Quatre directions lui ont été dessinées hors écran : `docs/carnet/
screens/52-selection-quatre-directions.png` — A aujourd'hui, B des équerres aux coins, C un fil
et quatre points, D un cadre unique pour le groupe. **Il ne sait pas où regarder les fichiers** :
la lui **montrer** (l'envoyer à l'écran), recueillir son choix, puis l'écrire dans la loi des
ornements (`hit_priority::chrome_ratio`, `renderer/handles.rs`, `scene/image/ornement.rs`) et
dans `style.md`. Si c'est la couleur qui lui manque, c'est `style.md` qui change — sa décision.

## 2. Le toucher : **Android d'abord**, et l'iPad par le web — ce qu'il attend le plus

**Fait les 07 et 08/10 (fiches 54 à 56, `decisions/08` et `09`)** : Glucose compile pour Android et
**tourne sur son moto g54 5G** (fiche 61) ; la couche de gestes ; la clé de signature et l'APK dans la
publication ; **le rail** (la barre sur le côté quand elle ne tient plus — à juger à son écran).
**Sa liste du 08/10, dans son ordre** (fiche 55 § 4), la vidéo remise à plus tard — et ce que
la session du 08/10 (fiche 56) en a fait :

| | chantier | où en est-il |
|---|---|---|
| [~] | **VIE-1 — la surface qu'Android reprend et rend** : sans elle, revenir d'une autre application (le chemin même du partage) tombait sur une surface morte | écrit, éprouvé hors écran ; **à son téléphone** |
| [~] | **PARTAGE-1 — le partage d'images vers Glucose** : `SEND`, `SEND_MULTIPLE`, les images en un geste, un lien au rapatriement, Java à la frontière seulement (`decisions/09`) | **à son téléphone** : Pinterest, la galerie, Chrome |
| [~] | **Le double toucher** sur le vide : le menu à la taille du doigt, « Ajouter des images… » par le sélecteur de photos ; le bouton « images » du rail, qui ne faisait rien, aussi | **à son téléphone** |
| [~] | **QUESTION-1** — la question que Glucose dessine (aucun dialogue sous Android) ; **VUE-1** — le refus du journal technique donné à sa place se repose | **à son téléphone** |
| [~] | **DOCUMENTS-1** — les documents du téléphone : un travail sans nom se range sous « Canevas N », « Nouveau document » marche, « Ouvrir un document… » liste les rangés | **à son téléphone** |
| [~] | **CLAVIER-1 à 3 — le clavier du téléphone** (fiche 57) : la saisie et le clavier tiennent le même état (le miroir) ; les positions UTF-16 de Java ; l'état d'avant l'envoi ignoré ; la ligne qu'on écrit reste au-dessus du clavier ; la touche retour valide | écrit, éprouvé hors écran (treize sabotages) ; **à son téléphone** |
| [~] | **APPUI-1 — l'appui long** (fiche 57) : au délai que règle l'accessibilité d'Android, le menu du clic droit à la taille du doigt, le mot sous le doigt dans un texte, une vibration ; la question répond au relâchement | **à son téléphone** |
| [~] | **DOCUMENTS-2 — renommer, dupliquer, supprimer** par un appui long dans la liste (fiche 57) ; un champ dans la question, que le clavier remplit | **à son téléphone** |
| [~] | **Les emojis** : Noto Emoji en repli (EMOJI-1) | fiche 58 § 5 — **à son téléphone** |
| [~] | **BORD-1 — le bord à bord** (fiche 57 § 4 bis), avancé devant la mise à jour : partout, les marges du système lues dans `MainActivity`, la barre qui couvre la barre d'état, *l'écran visible* au-dessus de la navigation et du clavier | écrit, éprouvé hors écran ; **à son téléphone** |
| [~] | La **mise à jour automatique** sous Android — écrite (fiche 58 § 6), à prouver à la prochaine publication | |
| [~] | **La batterie sous Android** — écrite (fiche 58 § 9) : la télémétrie de son prochain lancement le dira |
| [ ] | Le presse-papiers du système ; les panneaux qui s'écartent des bords | |
| [ ] | Les doigts sous Windows (écarter la souris simulée, `GetMessageExtraInfo`) et l'iPad par le web. La vidéo (fiche 55 § 4) quand il la redemandera ; les photos **HEIC** par le décodeur d'Android | |

Le plan d'origine (fiche 50) ; l'ordre :

1. **L'exécuteur des tranches** : dans un navigateur, il n'y a pas de fils. Le travail de fond
   (scribe, atelier, boîte noire) doit pouvoir tourner **en tranches, dans le temps libre de chaque
   image**, comme une voie de plein droit, éprouvée d'abord sous Windows (mêmes résultats par les
   deux voies).
2. **La couche de gestes** commune : doigts, pincement, aucun survol, puis le crayon.
3. **Le web** (WebGPU dans Safari 26 sur iPad à puce A12 et après, WebGL2 en dessous) : chaque
   document écrit **aussi** dans les Fichiers de l'iPad, parce que Safari efface les données d'un
   site inactif et recharge une page trop gourmande. Une version web qui perd un document serait
   pire que rien.
4. **Android** : `GameActivity` (le clavier virtuel ne marche pas avec `NativeActivity`), la vie
   d'une application suspendue, l'APK et **sa clé à ne jamais perdre**, de vrais téléphones anciens
   par Firebase Test Lab (gratuit).

## 3. Des versions complètes : tous les Linux, tous les Mac, le mode référence partout

**Pourquoi** : sa demande du 07/10 — « une version ultra stable avec tout le mode Ctrl+Maj+A,
compatible tablette, Linux, Windows et Mac ». Linux a déjà ses trois formes et NixOS (fiche 48) ;
**le Mac n'a aucune version publiée** : la CI le compile (puce Apple et Intel), rien ne
l'empaquette. Sans compte Apple (aucun argent), un `.dmg` non signé s'ouvre par « clic droit →
Ouvrir », comme PureRef l'a longtemps fait.

* Le mode référence hors de Windows : sans cadre et au premier plan passent par `winit` partout ;
  `Alt` + glisser (`drag_window`) est refusé par certains bureaux Wayland — à vérifier, et à dire
  quand il ne marche pas.
* Le pavé hors de Windows : macOS donne le pincement et le déplacement nativement à `winit` ;
  Linux selon le bureau (libinput sous Wayland, rien de propre sous X11).
* **Fini quand** : la CI verte sur les neuf tâches, un `.dmg` à la release, et un essai du mode
  référence sous Linux et Mac (par lui ou un testeur).

## 4. La boîte noire qui voyage — la télémétrie de tous ses utilisateurs

**Pourquoi maintenant** : deux installeurs téléchargés, zéro paquet Linux, et aucun moyen de savoir
si la bascule a échoué en silence chez quelqu'un. Sans elle, on ne voit que sa machine.

Conçu, pas écrit (fiche 49 § 2) : éteint par défaut, une question claire au premier lancement,
« voir ce qui part » (exactement les lignes de la boîte noire, qui ne peuvent porter aucun mot de
l'utilisateur), des lots envoyés au lancement suivant en HTTPS, un identifiant tiré au hasard et
renouvelable, effacement sur demande, **aucune adresse IP gardée**, une page publique qui dit tout.

* **En ligne le 08/10 (fiches 54 § 6 et 55 § 1)** : le Worker et sa base dans son compte
  Cloudflare, l'adresse dans Glucose. **Le 08/10 au soir, la base est vide** : personne n'a
  encore lancé un Glucose qui la porte et répondu « oui ». Reste : vérifier la première session
  reçue (`npx wrangler d1 execute glucose-boite-noire --remote --command "SELECT …"`, depuis
  `outils/telemetrie/`). **La question sous Android est faite** (QUESTION-1, VUE-1, fiche 56).
* Les plantages hors de Windows : Linux en a deux registres (`systemd-coredump` chez Fedora, `apport`
  chez Ubuntu) ; Android a `ApplicationExitInfo` (11 et plus).

## 5. Ce qui manque de Glucose Tauri

D'abord **remesurer la parité** (le tableau de la fiche 03 date du 10/09). Puis, dans l'ordre de la
fiche 36 § 2 (phase 4) : ce qui est écrit dans le noyau et pas branché (`timeline`, `mirror_graph`,
rideaux, `membrane_stretch` : le cliquet 3 les nomme), naviguer dans l'immense (`Ctrl+F`, la couleur
des domaines), dossiers et miroirs, rideaux et temporalité, storyboard, presets et zones,
l'interface (sélecteur de couleur, infobulles), vidéos et provenance (fiche 27), exports PNG et HTML.

**Le registre de Tauri** (fiche 39) garde ses défauts relevés, à ne pas reproduire. Encore ouverts :
les membranes de Mary (13), les rideaux (14), « recopie image » (15) et la carte « aaaa » (18) à
comprendre avec lui, les niveaux de boards (16) à confirmer, l'optimisation de la Time Machine (12).

## 6. La fluidité

* Les zooms chargés à 11-21 ms (session du 05/10) : le plancher de 100 images par seconde.
* **Sur batterie** (fiche 54 § 5) : le tempo à 7-8 balayages (30 images par seconde) sur 11 % des
  images en mouvement, « effacer » jusqu'à 27 ms. Une session branchée et une débranchée, le
  même geste, pour séparer le bridage de la carte de Glucose ; puis pixeliser plutôt que céder.
* Les gels de la carte Intel Arc (fiche 43 § 8) : très probablement l'écran branché sur la RTX
  (fiche 52 § 1) — à confirmer, puis à mesurer en mode Optimus (l'écran sur l'Intel).
* **La boîte noire ne note ni la carte ni la succession des images** : à ajouter, c'est elle
  qui voyage (fiche 52 § 1).
* Un plantage reproductible **dans les épreuves** : trois fois dans `vulkan-1.dll` au même octet
  (fiche 49 § 4), probablement une épreuve graphique qui referme la carte.
* ~~Le pavé par *Direct Manipulation*~~ : fait (fiche 53), à juger à son écran.
* **Le dépôt hors de Windows demande la page nue** : `ureq` sans sa fonctionnalité `gzip` (qui
  tirerait `flate2`, une note à écrire). Sous Linux, une page d'épingle peut mettre 20 s.
* Puis l'étalonnage : le modèle de coût qui **prédit** ce qu'une image va coûter, appris de ce que la
  boîte noire aura vu sur les vraies machines.

## 7. Plus loin

Le co-working (sur le journal du document) · le MCP réécrit en Rust (`rmcp`) · les plugins selon sa
vision ([PLUGINS.md](../PLUGINS.md)) et l'IA locale · le Mac · la fondation des 10⁷ nœuds (l'arène,
écrite et en attente) · et le but : le canevas de Wikipédia de `glucose-brain`.

---

## Ce qui n'attend que lui

0. **Ses retours du 09/10** (fiche 59 § 10) : redéployer le serveur de télémétrie pour la ligne
   « carte » ; brancher le téléphone en USB une fois ; sa clé SauceNAO ; l'envoi et la CI.
1. **Le clic « Publish release »** sur le brouillon de la 2.0.2-beta.1 (fiche 55 § 1).
1 bis. ~~Une copie de la clé Android hors du PC~~ : faite (08/10 au soir). Garder ou non le
   sous-domaine `ferme-nilslamber` du serveur de la boîte noire.
2. **Une copie de sa clé de signature hors de ce PC.**
3. **Le modèle de l'iPad**, et **les téléphones de ses testeurs Android** (le plus ancien
   d'abord) : un « Xiaomi 9 » — Mi 9 ou Redmi 9 ? — et une clé de signature Android à garder.
4. Le modèle de sa souris : **il en a une** (07/10), mais navigue surtout au pavé tactile.
5. Sait-il qui, parmi ses cinq utilisateurs, a basculé ? Surtout sous Linux.
6. Les membranes de Mary, les rideaux, Trans-domaines, « optimiser » dans la Time Machine.

## Les essais à l'écran jamais faits

Écrits dans les fiches 42, 43 et 47, jamais rapportés. À lui redonner en **une liste courte** à la
fin du chantier 1 (avec le copier-coller entre deux fenêtres et le collage dans Discord) : les flèches qui contournent (et leur coude qu'on glisse), les formules seules sur
leur ligne, la Time Machine ouverte pendant qu'on zoome, `Ctrl+Z` après un glisser ou un
redimensionnement, l'aimant sur un tableau chargé, coller puis `Ctrl+C`/`Ctrl+V` une image, glisser
un PDF depuis une page web.
