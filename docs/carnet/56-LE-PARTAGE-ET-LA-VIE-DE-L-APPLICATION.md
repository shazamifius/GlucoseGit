# 56 — Le partage vers Glucose, la vie de l'application, les questions et les documents du téléphone

> Session du 08/10/2026. Sa liste du 08/10 (fiche 55 § 4), dans son ordre : **le partage
> d'images vers Glucose** (« uniquement les images »), **le double toucher** sur le vide, puis
> **les documents au téléphone**. Rien de ce qui suit n'a encore tourné sur un téléphone : le
> § 8 dit quoi essayer.

---

## 1. Ce que la lecture a trouvé avant d'écrire

* **La surface perdue (VIE-1).** Sous Android, passer dans une autre application **détruit**
  la surface de la fenêtre ; le retour en donne une neuve. `winit` le dit (`Suspended`,
  `Resumed`), Glucose ne l'écoutait pas : `resumed` ne faisait rien quand la fenêtre existait,
  `suspended` n'était pas traité. Or le partage est exactement ce chemin — on est dans
  Pinterest, on partage, Glucose revient. Le partage ne pouvait que tomber sur une surface morte.
* **Le bouton « images » du rail ne faisait rien au téléphone** : il ouvre le sélecteur de
  fichiers du bureau (`rfd`), absent sous Android. Un bouton qui ment.
* **Aucune question ne se pose au téléphone.** Les cinq dialogues de Glucose (nouveau
  document, travail non enregistré, mise à jour, journal technique deux fois) sont ceux du
  système, et synchrones ; sous Android, `oui_ou_non` répond « non », `oui_non_ou_annuler`
  « annuler ». D'où deux défauts :
  * **« Nouveau document » ne faisait rien** sur son téléphone, et un document sans nom ne
    pouvait jamais être quitté ;
  * **le journal technique a été refusé à sa place** : au premier lancement, la question a
    reçu « non » sans s'afficher, et ce refus s'est **écrit** comme une réponse. Pas une fuite
    — un refus —, mais un consentement qu'il n'a jamais exprimé, et la question ne se serait
    plus posée.
* **Le toast sort de l'écran du téléphone** : il fait la largeur de son texte, sur une ligne ;
  « Glucose reçoit les images et les liens : ce partage n'en porte aucun » fait 450 points,
  l'écran du Redmi 9 en a 360.
* **Aucun texte ne peut s'écrire au téléphone** : Glucose ne demande jamais le clavier
  (`set_ime_allowed`), et `winit` 0.30 **jette** le texte que `GameActivity` reçoit du clavier
  virtuel (`InputEvent::TextEvent` finit en « Unknown android_activity input event », lu dans
  sa source). Non corrigé : § 7.
* **La télémétrie** : la base du serveur est vide (`SELECT … FROM sessions` : aucune ligne).
  Rien d'anormal — personne n'a encore lancé un Glucose qui la porte et répondu « oui ».

## 2. VIE-1 — la surface qu'Android reprend et rend

* **Seule la surface part** (`present/gpu/vie.rs`) : le périphérique, ses textures (toutes les
  photos) et ses nuanceurs restent ; au retour, une surface neuve se crée sur **le même**
  périphérique, au format de l'ancienne, à la taille de la fenêtre. Refaire tout aurait coûté
  des secondes sur un téléphone, photos floues pendant ce temps — c'est la façon des exemples
  de `wgpu` et de Bevy. Si la surface neuve refusait le format, une présentation neuve s'ouvre.
  La voie processeur (`softbuffer`) refait sa surface sur le même contexte.
* **Une seule vérité** : « en arrière-plan » n'est pas un état tenu à côté, c'est « la
  présentation n'a pas de surface » (`Presenter::a_sa_surface`). Entre les deux, Glucose ne
  dessine rien. La première écriture avait un champ dans l'application ; le cliquet des 80
  lignes l'a refusé, et le retirer valait mieux qu'extraire.
* La sortie le dit : `[Glucose] vie : au premier plan, la surface revenue en N ms`.

## 3. PARTAGE-1 — Glucose dans « Partager »

* **Le manifeste** : `SEND` et `SEND_MULTIPLE` pour `image/*`, `SEND` pour `text/plain`.
* **Java, à la frontière seulement** (`MainActivity.java`) : l'intention arrive (`onCreate`,
  `onNewIntent`) ; un fil à part ouvre chaque fichier comme `openInputStream` l'ouvrirait
  (`openAssetFileDescriptor` — vérifié dans le code d'Android : `openFileDescriptor`, lui,
  refuse un morceau de fichier) et confie à Rust le descripteur, son début et sa longueur. Une
  intention **rejouée** (activité recréée, ou rouverte depuis les applications récentes) est
  ignorée : sans cela, le même partage se reposait.
* **Rust** (`glucose-android`, `jni` 0.22 — déjà dans l'arbre, `decisions/09`) lit les octets,
  bornés à 256 Mo, et confie le partage à `plateforme::partage` :
  * des images se posent **en un geste** — un `Ctrl+Z` les retire toutes —, au centre de la
    vue ; le texte qui les accompagne n'est qu'une légende ;
  * sans image, les **adresses** du texte partent au rapatriement — celui qui trouve l'image
    entière d'une épingle depuis un glisser sur le bureau (la page d'une épingle porte ses
    `i.pinimg.com/originals/…`, vérifié par `curl`) ;
  * un fichier que le système n'a pas laissé ouvrir **se compte** (« 2 éléments posés,
    1 illisible ») ; un texte sans lien **le dit**. Deux retouches au dépôt pour cela : un lot
    vide se taisait, et « Aucun des 1 fichiers » devient « Le fichier n'a pas pu être posé ».
* **La boîte aux lettres** : un partage qui lance Glucose arrive avant sa fenêtre ; il attend,
  et part quand le pont se branche. Une activité fermée puis rouverte par un partage en fait
  naître une autre : le pont se débranche avec la fenêtre, et le partage attend la suivante.
* Vérifié dans l'APK : les deux bibliothèques exportent `recevoirUnPartage` (`llvm-nm`), le
  manifeste porte les trois filtres (`aapt2`).

## 4. Le double toucher, et « Ajouter des images… »

* **Deux touchers rapprochés sur le vide** ouvrent le menu du canevas — la main n'a pas de
  clic droit. Les conditions d'un double clic : 350 ms, 8 points.
* **Au doigt, le menu prend la taille du doigt** : 48 points par entrée, 16 de texte, sans
  raccourcis clavier. La mesure vient de la main qui l'ouvre, pas d'un « mode téléphone » : le
  clic droit rend le menu de la souris.
* **« Ajouter des images… »** en tête du menu du vide. Au bureau, le sélecteur de fichiers ;
  au téléphone, **le sélecteur de photos d'Android** (`PickMultipleVisualMedia` : celui
  d'Android 13, ou celui que les services de Google ajoutent depuis Android 4.4, sinon le
  sélecteur de documents — aucune permission de stockage). Ce qu'on y choisit revient par le
  chemin d'un partage. **Le bouton « images » du rail** ouvre le même sélecteur.
* Au téléphone, le menu tait « Mode référence » et « Voir ce qui part » (une fenêtre sans
  cadre, un dossier à ouvrir dans l'explorateur : rien de cela n'y existe).

## 5. QUESTION-1 — la question que Glucose dessine, et VUE-1

* **Une question, deux voies** (`ui/question.rs`, `interactions/question.rs`) : là où le système
  a ses dialogues, elle s'y pose et sa **suite** part aussitôt ; au téléphone, Glucose la
  **dessine** — un voile, une carte plate, le texte coupé à la largeur de l'écran (WRAP-1), des
  réponses en rangées de 48 points — et la suite part au toucher. Elle prend tout toucher
  jusqu'à sa réponse. Chaque question n'est écrite qu'une fois.
* **Branchées** : le journal technique (au lancement, et au menu), et « Nouveau document »
  (NOUVEAU-1, qu'il a voulu le 07/10).
* **VUE-1** : toute réponse s'écrit désormais avec `vue=oui` ; sous Android, une réponse sans
  cette marque est celle que l'ancien Glucose a donnée à sa place — **la question se reposera
  sur son téléphone**, dessinée cette fois. Au bureau, la même ligne était une vraie réponse.
* Les textes de la question du journal portaient des suites d'espaces au milieu des phrases
  (des barres obliques inverses mangées par Git Bash, METHODE § 5) : réécrits, et le geste qui
  y revient est celui de la main qu'on a — « clic droit », ou « deux touchers sur le vide ».
* **Le toast se coupe en lignes** à la largeur de l'écran : au bureau, une ligne, à sa place de
  toujours.

## 6. DOCUMENTS-1 — les documents du téléphone

**La recherche** : Procreate, Infinite Painter et Concepts gardent les œuvres dans l'appareil,
dans une galerie qui est « un simple gestionnaire de fichiers » (Infinite Painter) ; renommer,
dupliquer, supprimer passent par un appui long (Infinite Painter, Concepts) ou un glissé
(Procreate) ; supprimer demande confirmation, et c'est définitif.

* **Rien ne se demande, tout se garde** : au téléphone, un travail sans nom qu'on quitte se
  range dans `documents/`, sous le premier nom libre — « Canevas 1 », « Canevas 2 »… —, et s'y
  écrit désormais geste après geste comme tout document nommé. « Nouveau document » marche.
* **« Ouvrir un document… »** au menu du vide : au bureau le sélecteur de fichiers (`Ctrl+O`) ;
  au téléphone **la liste** des documents rangés, du plus récent au plus ancien (les dix
  premiers), dessinée comme une question. Un toucher sur un nom range le travail en cours et
  ouvre celui-là. Sans elle, un document rangé par « Nouveau document » aurait été injoignable.
* `hasFragileUserData` : désinstaller Glucose propose de **garder** ses données — ses documents
  vivent dans le dossier privé de l'application, qu'une désinstallation efface sinon.
* **Pas encore** : renommer (il faut le clavier, § 7), supprimer (l'appui long, et la question
  « Supprimer ? » qui est prête), dupliquer, des vignettes.

## 7. Ce qui reste, et ce que l'ordre doit devenir

* **Le clavier virtuel passe avant le reste du téléphone** : sans lui, on n'écrit aucun texte
  — Glucose est fait d'écriture — et on ne renomme aucun document. Deux morceaux : demander le
  clavier quand une saisie s'ouvre (`Window::set_ime_allowed`, que `winit` traduit en
  `show_soft_input`), et lire ce qu'il écrit, que `winit` 0.30 jette — l'état du texte reste
  lisible par `AndroidApp::text_input_state()`, à relever après chaque évènement, ou par une
  version de `winit` qui le transmet (à chercher).
* Puis l'appui long (le clic droit du doigt : supprimer, renommer un document), la mise à jour
  sous Android, les barres du système (bord à bord), les doigts sous Windows.
* **HEIC** : les photos de certains téléphones (Samsung, Pixel en option) sont en HEIC, que
  `image` ne lit pas ; partagées, elles se compteront « illisibles ». Android sait les décoder
  (`ImageDecoder`, Android 9 et après) : une voie système, comme la vidéo plus tard.

## 8. Prouvé, et pas prouvé

* **Prouvé** : 2 147 épreuves ; clippy strict sous Windows et pour Android ; quarante et une
  gardes sabotées, quarante et une chutes (une épreuve aveugle vue avant le sabotage — l'ordre
  des documents — et écrite) ; l'APK signé de sa clé, ses symboles et son manifeste.
* **Pas prouvé** : **rien n'a tourné sur un téléphone.** La surface rendue, le partage depuis
  Pinterest, la galerie et Chrome, le sélecteur de photos, la question dessinée, la liste des
  documents : son essai le dira.

---

**Retour** : [`00-INDEX.md`](00-INDEX.md)
