# 58 — Les questions dessinées partout, les noms du rail, les emojis, et la mise à jour d'Android

> Session du 08/10/2026, tard le soir. Ses demandes du 08/10 au soir (`SUITE.md` § 0), dans
> leur ordre : **POPUP-1** d'abord — l'urgence —, puis la carte des gestes à plusieurs doigts
> (une proposition, rien d'écrit : § 4), **les noms dans le rail**, **les emojis** ; puis,
> dans l'ordre de la suite, **la mise à jour sous Android** — sa condition pour publier.

---

## 1. POPUP-1 — ce que la lecture a trouvé

Ses mots : *« souvent des black screen total et des freeze de l'app ; il faut voyager dans le
noir total, faire Tab puis Entrée pour autoriser la télémétrie »*.

* **Une boîte du système tient la boucle de Glucose.** `rfd::MessageDialog::show` ouvre une
  boîte modale de Windows : elle fait tourner **sa** boucle de messages jusqu'à la réponse.
  Pendant ce temps, l'évènement de `winit` qui l'a ouverte n'est pas fini — aucune image ne se
  dessine, aucune ne se présente. Qt connaît le même défaut sous Windows ([QTBUG-4245](https://bugreports.qt-project.org/browse/QTBUG-4245)),
  et `rfd` lui-même déconseille la boîte synchrone avec `winit` sur macOS ([son README](https://github.com/parasyte/rfd)).
* **Le noir total vient de l'ordre** : la question du journal technique se pose **à
  l'ouverture de la fenêtre** (`demander_la_telemetrie`), avant que Glucose ait présenté une
  seule image. La fenêtre est noire — rien n'y a jamais été dessiné —, et la boîte peut se
  retrouver derrière elle : il faut alors deviner qu'elle attend, Tab, Entrée. C'est
  exactement son récit.
* **Les questions du bureau** passaient toutes par là : le travail non enregistré (la croix,
  `Ctrl+N`, `Ctrl+O` — BROUILLON-1), la mise à jour (deux fois), le journal technique,
  « Nouveau document ». La question dessinée existait déjà (QUESTION-1, fiche 56), mais
  seulement au téléphone.
* **Les sélecteurs de fichiers** (ouvrir, enregistrer sous, exporter, ajouter des images)
  tiennent la boucle de la même façon, mais ils s'ouvrent après un clic, sur une fenêtre déjà
  dessinée : la dernière image reste à l'écran. À lui de dire s'ils lui posent le même problème.

**Ce que font les autres** : Blender dessine ses popups dans sa fenêtre, et ne prend la boîte
du système que là où il ne peut pas dessiner ; son « bouton actif par défaut », qu'Entrée
déclenche, est né pour la question d'enregistrer en quittant ([le commit](https://developer.blender.org/rB2d34420648e5feacf1237abc975f8ff2a0c2a907),
[la question en quittant](https://lists.blender.org/pipermail/bf-blender-cvs/2018-March/105781.html)).
Figma et tldraw, dans un navigateur, n'ont que des boîtes dessinées. Le motif de dialogue du
W3C ([APG](https://www.w3.org/WAI/ARIA/apg/patterns/dialogmodal/)) fixe le clavier : Tab
tourne dans la boîte, Échap la ferme, et l'évidence va à l'action la moins destructrice quand
la plus évidente détruit.

## 2. POPUP-1 — ce qui est fait

* **Une seule voie** : toute question se dessine dans Glucose, au bureau comme au téléphone
  (`interactions/question.rs`). Glucose continue de se dessiner pendant qu'elle attend : plus
  rien ne tient sa boucle.
* **Une suite, pas un retour** : la croix ne demande plus « puis-je fermer ? » en attendant la
  réponse ; elle pose la question avec ce qui doit suivre (`persist::close::Apres` — quitter, ou
  laisser l'installeur prendre la place), et la boucle s'arrête quand la réponse le permet
  (`UiState::fermer_la_fenetre`). Même chose pour le travail sans nom qu'on quitte
  (`Ensuite` — ouvrir un document, ou un vierge) et pour la mise à jour.
* **Les réponses disent ce qu'elles font** : « Enregistrer », « Ne pas enregistrer »,
  « Annuler » — au lieu de « Oui / Non / Annuler » et d'un texte qui expliquait les boutons, la
  boîte du système ne sachant pas les nommer. « Installer », « Plus tard » pour la mise à jour.
* **Au clavier** : Entrée donne la réponse en évidence, cernée du filet blanc de la sélection
  (`style.md`) — la première, sauf « Supprimer », qui laisse l'évidence à « Annuler » ; Tab,
  Maj + Tab et les flèches la déplacent ; **Échap donne « Annuler »**, la réponse qui ne change
  rien — et le retour d'Android aussi. Une question sans « Annuler » (le journal technique) se
  retire **sans réponse** : on ne répond jamais à sa place (VUE-1), elle se reposera.
* **Rien ne passe sous le voile** : ni touche (un `Suppr` n'efface plus rien derrière), ni clic
  droit, ni molette. Un clic droit commencé avant la question se termine sans ouvrir de menu.
* **Les réponses restent à l'écran** : un texte trop long — les notes d'une mise à jour — se
  coupe à ce que l'écran laisse, et finit par « … ».
* **Une seule boîte du système reste** (DIAL-5) : la mise à jour cherchée **avant** la fenêtre,
  après une session qui a mal fini. C'est voulu : ce qui tombe au démarrage (la carte
  graphique, le document) ne doit pas empêcher de recevoir la correction, et Glucose n'a alors
  rien où dessiner. Aucune fenêtre de Glucose n'existe à noircir ni à geler. Le type le tient :
  `dialogue::oui_ou_non` exige un `SansFenetre`, que seul `GlucoseApp::sans_fenetre` construit,
  et seulement sans fenêtre.
* `ui.questions_dessinees` disparaît : il mêlait deux choses. Dessiner la question vaut
  partout ; **ranger les documents sans sélecteur de fichiers** reste propre au téléphone, et
  porte désormais son nom (`ui.documents_ranges`).
* **Prouvé** : vingt épreuves neuves ou réécrites, jouées par la vraie souris et le vrai
  clavier de Glucose ; **dix-huit sabotages, dix-huit chutes** ; l'aperçu hors écran
  (`examples/apercu_questions.rs`) au format de son portable (1 920 × 1 200 à 150 %).
  **Pas prouvé** : rien n'a tourné sur son écran. Le chemin de la croix jusqu'à la sortie de la
  boucle (`entretenir`) et le branchement du clavier (`handle_key`) ne se jouent qu'avec une
  vraie boucle ; la preuve qu'aucune boîte ne s'ouvre sur une fenêtre (`sans_fenetre`) est un
  type, qu'une épreuve sans fenêtre ne peut pas saboter.

## 3. Les noms dans le rail

Sa demande, acceptée : *« une icône et son nom, quitte à faire défiler »* ; trois icônes y sont
identiques. Ce sont **Ordonner, Storyboard et Preset** : quatre carrés presque pareils, repris
tels quels des icônes de Glucose Tauri. Les redessiner l'écarterait de Tauri ; leur nom suffit.

* **Chaque bouton porte son nom**, quelle que soit la densité (`TopbarButtonDef::nom`) ; les
  outils ont le leur (`ActiveTool::nom`, les infobulles de Tauri dans les mots de Glucose :
  « Post-it », pas « note sticky »).
* **Le rail l'écrit à droite de l'icône quand les colonnes tiennent dans la largeur**, sinon il
  garde les icônes seules — la règle de la barre du haut, qui cède ses libellés quand elle ne
  tient plus. Je n'ai pas fait défiler : un bouton caché derrière un défilement est un bouton
  qu'on ne trouve pas, et sur son Redmi 9, debout comme couché, tout tient.
* **Colonne après colonne**, chacune à la largeur de **son** plus long nom : une liste se lit
  de haut en bas, les outils ensemble. La première version donnait à toutes les colonnes la
  largeur du plus long nom, et debout il manquait 7 pixels ; mesurés, pas devinés.
* Les noms sont au corps 12, le bas de la fourchette de `style.md` pour l'interface : à 13,
  « Déplacer la vue » et « Trans-domaines » ne tenaient pas côte à côte sur 360 points.
* **Vu en regardant l'aperçu** : avec le rail, la bande gardait la hauteur de la barre d'état
  (BORD-1) sans la peindre, et la **remplaçait** par du vide — selon la voie de rendu, du noir
  ou ce qui traînait dans la surface. Son fond la couvre désormais, comme quand la barre est là.
* Six épreuves neuves ; les sabotages au § 7.

## 4. GESTES-1 — la carte proposée, rien d'écrit

Sa demande : *« impossible de faire une multi-sélection et un clic droit ; avec 2 doigts on crée
des raccourcis, et avec 3 aussi ; le plus simple possible »*. Le clic droit existe déjà depuis la
fiche 57 (l'appui long), mais son APK ne l'a pas encore montré à un téléphone. La recherche :

* **Procreate** : toucher à deux doigts, annuler ; à trois, rétablir ; tenir deux doigts annule
  en rafale ; trois doigts glissés vers le bas, le menu copier-coller ([son manuel](https://help.procreate.com/procreate/handbook/interface-gestures/gestures),
  [annuler et rétablir](https://help.procreate.com/articles/tvicQm-undo-and-redo)).
* **Infinite Painter** : deux doigts annulent, trois rétablissent ([sa documentation](https://docs.infinitestudio.art/painter/studio/gestures/)).
* **Concepts** : deux doigts annulent (réglable) ; pour choisir plusieurs traits, un doigt tient
  et **un second doigt touche** chaque trait à ajouter ([sa sélection](https://concepts.app/en/android/manual/selection)).
* **FigJam sur iPad** : un appui long, puis glisser, trace le rectangle de sélection ; l'appui
  long ouvre aussi « Sélectionner des objets », où chaque toucher ajoute ou retire ([son aide](https://help.figma.com/hc/en-us/articles/4502073572247)).
  Affinity Designer et Amadine : un doigt tient, les autres touchent pour ajouter ([le forum de Figma](https://forum.figma.com/t/multi-select-on-ipad/55766)).
* **tldraw** : l'appui long ouvre le menu contextuel ([sa version 5.2](https://tldraw.dev/releases/v5.2.0)).

La carte que je lui propose, en quatre lignes, sans un bouton de plus : **deux doigts touchés**
annulent, **trois** rétablissent ; **un doigt qui tient un nœud pendant qu'un autre en touche
d'autres** les ajoute à la sélection (ou les retire) ; **un appui long sur le vide puis glisser**
trace le rectangle de sélection — le menu du vide reste aux deux touchers, et l'appui long sur un
nœud reste son clic droit. Elle attend sa parole.

## 5. EMOJI-1 — les emojis, par Noto Emoji

* **La police à la source** (`decisions/10`) : la version monochrome actuelle est une police
  **variable** de 1,98 Mo chez Google Fonts — pas « un mégaoctet » ; son instance fixe de graisse
  400, tirée par `fontTools`, pèse **887 Ko** et couvre **1 489 caractères jusqu'à Unicode 15**.
  `fontdue` 0.9.4 compile `ttf-parser` sans les variations (lu dans son `Cargo.toml`) : il ne
  lisait de toute façon que l'instance par défaut.
* **En repli, seulement** (`Typography::police`) : le visage demandé s'il a le caractère, sinon
  Noto Emoji s'il l'a, sinon la case d'Inter. La mesure et le dessin passent par le même choix.
  Un caractère qu'Inter a déjà reste à lui : le cœur U+2764, par exemple — vu par l'épreuve.
* **Ce qui manque** : sans moteur de mise en forme, une famille composée s'affiche en personnes
  côte à côte, un drapeau en deux lettres (« F R »), un ton de peau en carré après la main.
* Vu à l'œil, sur un texte rendu hors écran : les emojis sont nets, à la taille du texte.

## 6. MAJ-ANDROID-1 — la mise à jour sous Android

Sa condition pour publier la 2.0.2 : *« si la mise à jour automatique fonctionne pour tout le
monde »*. Sous Windows et Linux, la CI la répète ; sous Android, il n'y en avait aucune.

**Ce que la source dit** (lue, pas résumée) :

* une application **ne se remplace pas elle-même** : elle confie l'APK à l'installeur du système,
  `PackageInstaller` — Android 5 et après, donc tous ses téléphones ; la mise à jour ne passe
  que si l'APK porte **la même clé** (celle de Glucose pour Android) et un code de version plus
  haut (`codeDeVersion`, déjà écrit) ([`PackageInstaller`](https://developer.android.com/reference/android/content/pm/PackageInstaller)) ;
* il faut la permission `REQUEST_INSTALL_PACKAGES`, et, depuis Android 8, que l'utilisateur
  autorise une fois Glucose à « installer des applications inconnues »
  (`canRequestPackageInstalls`, la page `ACTION_MANAGE_UNKNOWN_APP_SOURCES`) ;
* **Android 12 et après** installe **sans confirmation** une application qui se met à jour
  elle-même, si elle vise une API récente et déclare `UPDATE_PACKAGES_WITHOUT_USER_ACTION` — lu
  dans le code d'AOSP (`PackageInstaller.java`, `setRequireUserAction`) ; ailleurs, et sur son
  Redmi 9 (Android 10), le système demande ; la confirmation arrive par
  `STATUS_PENDING_USER_ACTION`, qu'on doit toujours savoir traiter.

**Ce qui est écrit** :

* **Le chemin du bureau, tel quel** : la veille lit `latest.json`, la question dessinée propose
  (« Installer », « Plus tard »), un fil télécharge et **vérifie la signature de Glucose** avant
  tout — puis « poser » l'APK, sous Android, c'est **le confier au système**
  (`Installation::Apk`, `plateforme::installation`). Rien à relancer : Android remplace Glucose.
* **Glucose ne se ferme pas lui-même** : son document s'écrit déjà geste après geste ; ce qu'il
  tient part sur le disque au moment où l'APK est confié. Le fermer avant que l'utilisateur ait
  confirmé aurait laissé, s'il refusait, une application ouverte sur un document fermé.
* **Java, à la frontière seulement** (`MainActivity.installerUnApk`) : la session de
  `PackageInstaller`, la page d'autorisation au premier usage (l'installation repart au retour),
  la confirmation du système quand elle est demandée. Ce qu'Android en dit revient à Rust par
  un nombre (`recevoirLInstallation`), et Glucose le dit dans ses mots.
* **La publication** signe aussi l'APK de la clé de Glucose, et le manifeste l'inscrit sous
  `android-aarch64` et `android-armv7` (avec et sans « -apk ») : un seul APK porte les deux.
  L'exemple `publication manifeste` le relit et vérifie sa signature, comme les autres.
* Un piège évité en écrivant : Java répond « il faut autoriser » **pendant** l'appel, sur le même
  fil ; la porte relâche son verrou avant d'appeler, sans quoi Glucose se serait figé — une
  épreuve le joue.

**Ce qui n'est pas prouvé, et ne peut l'être qu'à une publication** : aucune release ne porte
encore d'APK sous ces clés. Le premier Glucose qui sait se mettre à jour doit s'installer à la
main ; la version **suivante** sera la première à arriver seule. La 2.0.2-beta.1 en brouillon a
été construite avant : il faudrait relancer la publication pour qu'elle porte ce chemin. Le Java
n'a tourné sur aucun téléphone.

## 7. Ce qui reste, et ce qui n'est pas prouvé

* **Rien de cette fiche n'a tourné sur son écran ni sur son téléphone.**
* Les **sélecteurs de fichiers** (ouvrir, enregistrer sous, exporter, ajouter des images) restent
  ceux du système, et tiennent la boucle de la même façon : à lui de dire s'ils lui posent le même
  problème. Le remède existerait sans les redessiner : `rfd` sait ouvrir un sélecteur sans bloquer
  (sa voie asynchrone), et Glucose continuerait de se dessiner derrière.
* La liste des documents au téléphone (dix documents et « Annuler ») ne tient pas sur un téléphone
  **couché** : onze rangées de 48 points pour 360. Le texte se coupe désormais, les rangées non.
* La télémétrie : une neuvième session est arrivée, de son téléphone — du 07/10 au soir, envoyée au
  lancement suivant ; rien encore de l'APK qui porte le clavier.
* La porte de l'installation relâche son verrou avant d'appeler Java ; le sabotage contraire ferait
  **pendre** l'épreuve au lieu de la faire tomber : il n'est pas lancé, l'épreuve le garde quand
  même (elle ne finirait pas).

## 8. À essayer

**Au bureau** (`cargo run --release > sortie-essais-58.txt 2>&1`) :

1. Fermer Glucose par la croix avec un texte neuf, jamais enregistré : la question se dessine dans
   Glucose ; Échap la retire, la croix revient ; « Ne pas enregistrer » ferme.
2. `Ctrl+N` : Entrée crée, Échap annule ; Tab déplace le filet blanc.
3. Clic droit sur le canevas → « Journal technique… » : la question se lit nettement, Échap la
   retire sans rien changer.

**Au téléphone** (l'APK neuf de son Bureau) : le rail déplié, chaque icône avec son nom ; un emoji
tapé dans un texte ; puis ce que la fiche 57 § 7 demandait déjà (le clavier, l'appui long, les
documents).

---

**Retour** : [`00-INDEX.md`](00-INDEX.md)
