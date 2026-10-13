# 65 — Le storyboard tracé, le champ qui écrit, et le téléphone qui redémarre

> Session du 13/10/2026, l'après-midi. Ses retours, en ouvrant : le téléphone a **redémarré
> en urgence** ; `Ctrl+F` doit servir la recherche, et tout cadrer passe à `Alt+F` ; l'outil
> Storyboard, comme l'outil Membrane, « ne fonctionne pas au survol » et ne se trace pas ; le
> storyboard doit se remplir **selon un rectangle dessiné** ; « Décrire » n'écrit rien ; et
> *« je trouve que l'outil n'est pas pratique et incomplet : à toi de faire des propositions »*.

## 1. Ses réponses, mot pour mot ou presque

* **Le téléphone** : *« j'ai carrément crashé total, mon téléphone a dû faire un redémarrage
  d'urgence »*.
* **`Ctrl+F`** : *« mieux de l'utiliser comme outil de recherche — du coup on utilisera
  `Alt+F` à la place »* (la question de la fiche 64 § 5).
* **Le survol et le tracé** : *« quand on ajoute un élément, on voit sa prévisualisation suivre
  la souris jusqu'à ce qu'on pose, ou qu'on crée un rectangle en maintenant le clic — les
  membranes, ça ne fonctionne pas, il faut juste cliquer »*.
* **Le storyboard** : *« faire en sorte de créer un système pareil de dessin de rectangle, et le
  storyboard se remplit automatiquement et crée des cases ou les supprime selon le rectangle
  dessiné par l'utilisateur »*.
* **Décrire** : *« le bouton Décrire ne fonctionne absolument pas, la zone de texte n'écrit
  rien »*.

## 2. Le sabotage de STORYBOARD-1 — et ce qu'il ne gardait pas

`outils/sabotages/storyboard-1.json` : **24 gardes tombent sur 25**. La vingt-cinquième, « poser
ne numérote pas », passait : aucune épreuve ne vérifiait que la grille posée numérote ses
cases. Mais surtout, **les deux défauts qu'il a vus n'étaient gardés par rien** :

* **La grille armée ne suivait pas la souris** : son fantôme se calculait bien sous le
  curseur, mais seul le gabarit armé redemandait une image au mouvement (`suivre_le_gabarit`).
  La grille ne bougeait que quand autre chose repeignait. L'épreuve vérifiait le calcul, pas le
  dessin.
* **Le champ de la question n'écrivait rien au PC** : la question jetait toute touche autre que
  Tab, les flèches, Entrée et Échap (`_ => {}`). Le champ n'avait jamais été rempli que par le
  clavier du téléphone (le miroir, CLAVIER-1). Et l'épreuve écrivait dans le champ **à la main**
  (`champ.texte = …`), sans une seule touche.

Leçon (METHODE § 4) : une épreuve d'interaction passe par le geste réel — la souris qui bouge et
l'image redemandée, la touche frappée —, jamais par l'état qu'il devrait produire.

## 3. La recherche

* **Cliquer ou tracer** : tldraw (`BaseBoxShapeTool`, son état
  [`Pointing`](https://github.com/tldraw/tldraw/blob/main/packages/editor/src/lib/editor/tools/BaseBoxShapeTool/children/Pointing.ts))
  — un clic crée la forme à sa taille par défaut, **centrée** sur le clic ; un glisser la crée à
  1 × 1 au point d'appui et passe au redimensionnement par le coin bas droit. Sketch
  ([frames](https://go.sketch.com/docs/designing/frames.md)) : un clic insère un cadre de la
  taille du dernier, un glisser le trace. Figma : le cadre se trace.
* **Le storyboard** : Boords [renumérote seul quand on réordonne](https://boords.com/iterate/automatic-frame-numbering)
  et [déplace plusieurs cadres à la fois](https://help.boords.com/en/articles/2307477-moving-multiple-frames) ;
  Storyboarder ([Wonder Unit](https://www.wonderunit.com/storyboarder)) porte dialogue, action,
  notes, durée et type de plan, et joue l'animatique ; Toon Boom Storyboard Pro fixe les
  [champs de légende au niveau du projet](https://docs.toonboom.com/help/storyboard-pro-24/storyboard/caption/about-default-panel-caption.html) ;
  Krita ([Storyboard Docker](https://docs.krita.org/en/reference_manual/dockers/storyboard_docker.html))
  a des champs de commentaire ajoutables, une durée par scène, et l'export en PDF ou SVG par
  rangées et colonnes par page ; Figma range par l'*auto layout* qui [revient à la ligne](https://forum.figma.com/suggest-a-feature-11/launched-wrap-elements-in-auto-layout-to-multiple-lines-20107/index14.html).
* **Le téléphone** (§ 8) : les forums d'Imagination sur la BXM-8-256 —
  [une longue liste de défauts de pilote](https://forums.imgtec.com/t/bxm-8-256-long-list-of-driver-issues/3891),
  [Vulkan qui plante sur cette puce](https://forums.imgtec.com/t/on-the-img-bxm-8-256-graphics-card-the-vulcan-driver-does-not-work-i-try-to-run-the-game-on-vulcan-and-it-does-not-work-it-crashes-immediately/4101) ;
  et la [liste des défauts de pilotes de Chromium](https://chromium.googlesource.com/chromium/src.git/+/HEAD/gpu/config/gpu_driver_bug_list.json).

## 4. TRACE-1 — un clic pose, un glisser trace

`interactions/trace.rs`. Sous l'outil Membrane, l'outil Dossier, et la grille armée du
Storyboard, **l'appui ne pose rien** : il commence un tracé (`Trace` : le départ, dans le monde
et à l'écran, et le coin que la main tire).

* Tant que la main reste dans le tremblement d'un double-clic, c'est un **clic** : le
  relâchement pose comme avant — la membrane et le dossier à leur taille de naissance sous le
  curseur (leur fantôme aimanté), le storyboard centré.
* Au-delà, le **rectangle suit la main** : sous une membrane ou un dossier, le fantôme *est* le
  rectangle ; sous la grille, les cases qui le remplissent, et le rectangle lui-même dessiné
  comme celui de la sélection. Le relâchement pose à la taille tracée — vers le haut et la
  gauche aussi.
* Échap renonce au tracé, l'outil reste armé ; un second Échap le rend.
* La chronique compte un tracé comme un dessin (`Geste::Dessiner`).

## 5. STORYBOARD-2 — une suite, une loi, une grille dans le document

**La grille est une donnée du tableau** (`Board::storyboard`, schéma **3**). STORYBOARD-1 l'avait
laissée au panneau : un rectangle tracé à six colonnes serait retombé à quatre au « + Ajouter »
suivant, et un document rouvert n'aurait plus su la sienne. Glucose Tauri la gardait avec le
tableau (`StoryboardSettings`), et le lecteur des documents de Tauri la lit désormais (il
l'omettait). La migration : un document v2 se lit sans grille — prouvé sur **de vrais octets**
écrits par la build de `58ff58c` (`tests/persist_fixture/riche-v2.glucose`), comme le témoin
v1 ; l'histoire en ajout seul porte la version de chaque geste, et une édition neuve
(`Edit::Storyboard`, numéro 13) l'écrit avant et après.

**Le storyboard est une suite**, et **une seule loi** place une case : celle de rang `i` est à la
colonne `i mod c`, la rangée `i div c` (`Grille::coin`). Tout geste ne fait que changer la suite
ou sa **disposition** (la grille, l'origine, le nombre — `Disposition`), et `Store::ranger`, seul,
en tire où va chaque case et ses images, en un geste :

* **poser d'un clic** (centré), **tracer un rectangle**, **tirer la poignée** — une disposition ;
* **« + Ajouter »** — une case de plus à la suite, à la grille enregistrée ;
* **retirer une case** — les suivantes avancent d'une place ; retirer la dernière retire le
  storyboard ;
* **lâcher une case** tenue par sa bande — elle prend la place sous le centre de son cadre, les
  autres glissent d'un cran (Boords). Le numéro suit donc **toujours** l'ordre qu'on voit.

**Ce qu'un rectangle décide** : les colonnes et les rangées **dont l'étendue est la plus proche
de la sienne** — `n = round((L + e) / p)`. Pas « celles qui tiennent » (la partie entière) : sur
une étendue exacte, `(n·p − e + e) / p` peut valoir `n − 1 ulp` — sept rangées au format Scope
donnent 6,999999999999999 —, et saisir la poignée pile au coin aurait retiré une rangée sans que
la main bouge. L'arrondi n'a de bord qu'à mi-case ; une épreuve le prouve sur les cinq formats,
et à l'envers. **Une case qui porte quelque chose** — une description, une image — ne se retire
jamais ainsi : la grille s'allonge plutôt.

**Ce qui suit une case** : ses images, par la similitude qui envoie l'ancien cadre dans le
nouveau, contenu et centré (`k = min(l'/l, h'/h)`) — à taille égale, une translation exacte ; la
largeur ou le format changés, la composition de la case est gardée.

**Un piège trouvé en route** : lâchée sur la troisième place, la case tenue **recouvre** encore
la troisième case ; la première case qui contenait l'image la prenait — l'image partait avec la
mauvaise. Trouvé par l'épreuve, lu dans l'état : les images que la case tenue emportait restent
les siennes, les autres cherchent leur case sans elle. Le lâcher enchaîne le glisser des images
et le rangement dans un même geste : le glisser se referme à la fin du geste en **une**
translation exacte, même suivi d'autres écritures (GLISSER-1) — une épreuve défait et refait le
geste au bit près.

**La poignée** : quand une case est choisie, le coin bas droit de la boîte du storyboard porte
la poignée des nœuds. La tirer, c'est **un tracé parti du coin de la première case** dont la
main tient le coin opposé (avec l'écart de la main au coin, gardé) : le fantôme montre en
direct les cases qui apparaissent et disparaissent, et le relâchement les pose.

**Le panneau** suit la grille du document quand elle change (un tracé, un autre tableau, un
`Ctrl+Z`) et garde ce qu'on y règle entre deux, pour le prochain « Mettre à jour » ; sous le
bouton, une ligne dit le geste : « UN CLIC POSE, UN GLISSER TRACE » (un toast ne dit que ce qui
vient d'avoir lieu, cliquet 2).

**« Re-numéroter les panels »** reste (tous les boutons sont obligatoires) : il renumérote dans
l'ordre de lecture **et range** — ce qui remet en suite un storyboard venu de Tauri ou de
STORYBOARD-1, aux cases éparses. Sur une suite déjà rangée, il ne change rien, et c'est juste :
la suite est toujours numérotée.

## 6. CHAMP-QUESTION-1 — le champ d'une question écrit au clavier

`interactions/question/frappe.rs`. Une frappe s'y traduit par **la même loi que dans un nœud**
(`Command::of`) — une seule traduction du clavier : lettres, Retour arrière et Suppr (par mot
avec `Ctrl`), les flèches, Début et Fin, `Ctrl+A`, copier, couper, coller. Le **texte composé**
de la boucle passe désormais jusqu'à la question : sans lui, « ê » (une touche morte sur un
clavier AZERTY) ne s'écrivait pas. Entrée, Échap, Tab et les flèches verticales restent à la
question. Le champ sert aussi « Renommer » un document au téléphone avec un clavier branché.

## 7. `Alt+F` tout cadrer, `Ctrl+F` à la recherche

Sa décision. `Alt+F` cadre tout, `F` la sélection ; `Ctrl+F` ne fait plus rien en attendant la
recherche (INVENTAIRE § 3). winit consomme `WM_SYSCHAR` lui-même : pas de « bip » de Windows,
et `Alt+T` y vivait déjà. Le menu dit `Alt+F`.

## 8. Le téléphone qui redémarre — ce qu'on sait, ce qu'il faut

**Ce que la télémétrie dit : rien de neuf.** La dernière session reçue de son téléphone
(`1c287c59`) est celle du 12/10, version 2.0.2-dev, sous l'ancien APK. **Aucune session de l'APK
du 12/10 au soir (2.0.3-dev) n'est arrivée** : ni celle de son essai d'export, ni celle du
plantage — une session ne part qu'au lancement suivant, et Glucose n'a pas été rouvert depuis,
ou n'a pas vécu assez pour envoyer.

**Ce qu'une application ne peut pas faire seule, c'est redémarrer un téléphone.** Il y faut le
noyau ou le système : un pilote graphique qui se bloque et que la surveillance du noyau abat, ou
une mémoire épuisée au point que le système lui-même tombe. **Deux pistes, aucune prouvée** :

* **Le pilote graphique de sa puce** (PowerVR BXM-8-256) : réputé fragile sous Vulkan, sans la
  couche de robustesse d'autres pilotes (une faute d'application y devient une faute du pilote,
  dit Imagination) ; les rayures venaient déjà de cette puce (fiche 61 § 6).
* **La mémoire graphique** : sous Android, aucun budget ne se lit (la sonde ne parle qu'à DXGI) ;
  la carte garde « ce que l'écran demande ». Sur un téléphone, cette mémoire est celle du
  système.

**Un test qui tranchera, sans rien lui demander de plus** : chaque session note quand l'appareil a
démarré (`demarrage_appareil_ms`). Si le téléphone a redémarré **pendant** Glucose, le démarrage
noté par la session suivante tombe quelques dizaines de secondes après la dernière trace de la
session interrompue ; des heures après, c'est un redémarrage sans lien. Les deux redémarrages
déjà connus (son PC le 08/10, son téléphone le 09/10) sont tombés **14 h 32 et 10 h 46 après** :
sans lien avec Glucose. La colonne « après » du tableau de bord le montre désormais pour chaque
session.

**Ce qu'il faut pour savoir** : le téléphone branché au PC par le câble, le débogage USB allumé,
puis lire ce que le système a gardé du redémarrage (`adb shell getprop sys.boot.reason`,
`adb shell dumpsys dropbox`, `adb logcat -b crash`) — c'est moi qui le lis. Et rouvrir Glucose
**une fois** pour que les deux sessions partent, ce qui n'est sûr qu'après avoir lu le
téléphone.

## 8 bis. Le tableau de bord de la télémétrie — et ce qu'il a déjà dit

Sa demande du 13/10 (« voir les statistiques et les PC »). `outils/telemetrie/tableau.mjs` lit la
base **depuis son compte**, comme toujours (`wrangler d1 execute` : le Worker n'a aucune porte
pour lire, et n'en a toujours pas), et la résume en trois fichiers — les installations (système,
carte graphique **par son nom**, tiré de la base publique `pci.ids`, sessions, fins, pire image),
les sessions reçues par jour, les sessions qui ont mal fini. Le tableau de bord est une page
privée de son compte claude.ai, qui cite chaque nombre à son fichier ; il se refait en relançant
le script (`node outils/telemetrie/tableau.mjs <dossier>`) puis en remplaçant les fichiers.

**Comment une session a fini** ne se lit pas dans ses lignes : c'est la **suivante** qui le dit
(sa ligne `precedente`). Au 13/10 : 48 sessions, 4 installations, 16 sessions mal finies.

**Ce qu'il a montré tout de suite — un défaut de la boîte noire** : au téléphone, **toutes** les
sessions dont on connaît la fin sont « interrompues » (14 sur 14). Lu dans le code : la ligne de
fin ne s'écrit qu'à la sortie de la boucle (`exiting`), et Android tue l'application en
arrière-plan sans jamais l'appeler. Un vrai plantage s'y confond donc avec une fermeture
ordinaire — seul le champ « appareil redémarré » y distingue encore quelque chose (vrai une
fois, le 09/10 : l'appareil a redémarré entre deux sessions, sans qu'on sache pourquoi). La
réparation (BOITE-ANDROID) : noter le passage en arrière-plan (`Suspended`), pour qu'une mort
en arrière-plan se lise comme une fin ordinaire — un nom de plus dans la liste fermée du
serveur.

## 9. Propositions pour le Storyboard — à sa parole

Ce qui est fait cette session répond à ce qu'il a demandé. Voici ce que les meilleurs outils
font et que Glucose n'a pas, par ordre de ce que j'en pense :

1. **Tab passe à la case suivante en décrivant** — *fait* (petit, sans risque, sans bouton neuf).
2. **Plusieurs images d'un coup remplissent les cases vides à la suite** : en lâcher dix sur la
   première case les range une par case, et le storyboard s'allonge s'il le faut (Boords,
   Milanote). C'est le « se remplit automatiquement » appliqué au contenu.
3. **Un storyboard de la sélection** : choisir des images, puis « Storyboard » au clic droit —
   une case par image, dans l'ordre de lecture.
4. **Insérer une case entre deux** : un « + » entre deux cases au survol, ou « Insérer avant /
   après » au clic droit (Boords).
5. **La lecture** (l'animatique) : une durée par case, et « Lire » montre les cases une à une, en
   plein écran (Storyboarder, Krita, Boords).
6. **Plusieurs champs par case** — dialogue, action, notes —, définis pour tout le storyboard
   (Toon Boom, Storyboarder, Krita).
7. **Le type de plan** (large, moyen, gros plan) à choisir par case (Storyboarder).
8. **L'export en planches** : PNG ou PDF, tant de rangées et de colonnes par page (Krita).
9. **Plusieurs storyboards par tableau** (une séquence chacun) — un changement du modèle.

## 10. Les erreurs de la session — ce que j'ai cru et que la vérification a démenti

* **« Une case qui tient » — la partie entière** : j'avais écrit la loi du rectangle « sans
  aucune tolérance à régler ». Le calcul l'a démentie : au bord exact, elle perd une rangée.
* **Le lâcher d'une case** : j'avais pensé que « la première case qui contient l'image » suffisait
  pour dire à qui elle est ; la case tenue en recouvre une autre au moment même du lâcher.
* **Une garde redondante** : j'avais cru qu'un glisser ne se refermait que si ses pas étaient les
  derniers écrits du geste, et ajouté `clore_le_glisser` au rangement. J'avais lu la condition
  qui le *continue* (`==`) ; celle qui le *referme* dit `<=`. Le sabotage l'a montrée aveugle :
  retirée.
* **Git Bash et les apostrophes**, encore : un heredoc d'épreuves refusé — passé par un fichier.

## 11. Ce qui attend sa parole

1. **Le téléphone** : le brancher au PC (débogage USB), pour que je lise pourquoi il a redémarré
   — avant de le rouvrir.
2. **Les propositions du § 9** : lesquelles, dans quel ordre.
3. **Le numéro d'une case** dans sa bande, plutôt que dans le coin (fiche 64 § 5).
4. **Trans-domaines**, **l'export PNG**, **la 2.0.3-beta.1** (fiche 64 § 5).

## 11 bis. Ce qui est prouvé

**2 422 épreuves** passent sous Windows ; clippy strict à zéro sous Windows et pour Android.
**57 sabotages, 57 chutes** : `outils/sabotages/storyboard-2.json` (34) et `storyboard-1.json` mis
à jour (23 ; deux de ses gardes sont couvertes par la loi unique de STORYBOARD-2). Le seul qui
passait était la garde redondante, retirée (§ 10). **Rien n'est jugé à son écran.**

## 12. À essayer à l'écran

`cargo run --release > sortie-65.txt 2>&1` :

* **Le Storyboard** : Storyboard → Activer → la grille suit la souris → **glisser** un rectangle :
  les cases le remplissent ; relâcher. Choisir une case (un clic sur sa bande) → tirer la
  **poignée** du coin bas droit : des cases apparaissent, disparaissent. Décrire une case (deux
  clics sur sa bande), taper, **Tab** pour la suivante. Prendre une case par sa bande et la
  lâcher ailleurs : les autres se décalent.
* **La membrane et le dossier** : l'outil, puis glisser un rectangle.
* **`Alt+F`** : tout cadrer.
