---
name: glucose-telephone
description: Concevoir Glucose pour le téléphone et la tablette (Android, puis l'iPad par le web) — à charger avant tout travail sur l'interface tactile, les gestes, le partage entre applications, les fichiers ou la vie de l'application sous Android. Glucose dessine lui-même son interface en Rust, monochrome et brutaliste (style.md) : ni Jetpack Compose, ni Material You.
---

# Glucose au téléphone

Glucose ne prend **pas** l'interface d'Android : il la dessine lui-même, en Rust, comme sur un
bureau, dans le langage de `style.md` (monochrome, plat, la couleur au contenu). Ce qui suit est
ce que le téléphone **impose** à ce dessin, et ce que le système **offre** et qu'il faut prendre.

## 1. Toucher : la main n'est pas une souris

* **Une cible tactile fait au moins 48 dp** de côté (Android, Material) — 44 pt sur iPad. Un
  bouton de 24 px de la bande du bureau est inutilisable au doigt : soit il grandit, soit il
  part dans un panneau. Mesurer en points indépendants : `ui.scale_factor`.
* **Aucun survol** : rien ne doit n'apparaître qu'au passage de la souris (infobulle, poignée qui
  surgit). Ce qui se révèle au survol sur bureau se révèle au toucher, ou reste visible.
* **Les gestes** (`interactions::toucher`) : un doigt agit comme la souris, sur le vide il déplace
  le canevas avec son élan ; deux doigts font leur similitude exacte. Le double toucher, l'appui
  long (le clic droit), le glisser depuis un bord se décident là, jamais dans un module de rendu.
* **Le pouce** : ce qu'on touche souvent vit en bas ou sur les côtés, à portée — pas en haut.

## 2. L'écran n'est pas une fenêtre

* **Bord à bord, imposé** à toute application qui vise Android 16 (API 36, la nôtre) sur un
  appareil Android 16 : Glucose dessine sous la barre d'état, la barre de navigation et
  l'encoche. Rien d'interactif ne s'y pose — les *insets* viennent de Java
  (`android-activity` 0.6.1 ne les donne pas ; voir le § 4 bis) ; le canevas, lui, va jusqu'au
  bord.
* **Portrait d'abord** (un téléphone se tient debout), paysage et tablette ensuite : la mise en
  page se calcule de la taille, jamais d'un « mode téléphone ».
* **Une seule surface**, comme sur bureau (`style.md` : pas de modes) : les panneaux se replient
  sur les côtés, une flèche les ouvre, le canevas reste tout.

## 3. Le système donne, il faut prendre

* **Le partage** (Pinterest, Instagram, TikTok, le navigateur) : un filtre `ACTION_SEND` et
  `ACTION_SEND_MULTIPLE` dans `AndroidManifest.xml` (`image/*`, `video/*`, `text/plain`) — une
  application partage un **fichier** (`EXTRA_STREAM`, à lire par `ContentResolver` tant que
  l'intention vit) ou un **lien** (`EXTRA_TEXT`), et c'est au dépôt de Glucose d'en faire un nœud,
  par le même chemin qu'un glisser-déposer sur bureau (`interactions::depot_web`).
* **Les images de l'appareil** : le sélecteur de photos du système (`ACTION_PICK_IMAGES`,
  Android 13 et après ; `ACTION_OPEN_DOCUMENT` en deçà) — aucune permission de stockage.
* **Les documents** : le dossier privé de l'application (`internal_data_path`) pour le travail
  courant ; le Storage Access Framework (`ACTION_OPEN_DOCUMENT`, `ACTION_CREATE_DOCUMENT`) pour
  ouvrir ou exporter un `.glucose` ailleurs.
* **Le retour** : avec la cible API 36 sur Android 16, l'animation de retour prédictive est
  active par défaut — le geste retour doit fermer ce qui est ouvert (un panneau, une saisie)
  avant de quitter.

## 4. La vie d'une application

* **Suspendue sans prévenir** : `Suspended` / `Resumed` de `winit` — la surface graphique est
  perdue et rendue (VIE-1, fiche 56 : la présentation lâche sa surface et garde ses textures) ;
  le document s'écrit déjà geste après geste, rien ne doit attendre la fin. **Tout geste qui
  passe par une autre application** (partager, choisir une photo) revient par ce chemin.
* **Tuée sans prévenir** quand le système manque de mémoire : la boîte noire le dira « arrêtée
  sans rien dire » ; c'est le régime normal, pas un plantage.
* **Bridée** : le téléphone chauffe en quelques minutes (charte : on observe le débit qui baisse).

## 4 bis. Ce qui n'existe pas au téléphone, et ce qu'on fait à la place (fiche 56)

* **Aucun dialogue du système** qu'on appelle comme `rfd` : la question se **dessine**
  (`ui::question`, QUESTION-1) et sa suite part au toucher — jamais une réponse donnée à la
  place de l'utilisateur (VUE-1). `ui.questions_dessinees` choisit la voie.
* **Aucun sélecteur de fichiers** : les documents se rangent d'office dans `documents/`
  (DOCUMENTS-1) et se rouvrent par une liste ; les images viennent du sélecteur de photos.
* **Aucun clic droit** : deux touchers sur le vide ouvrent le menu, à la taille du doigt ; et
  **l'appui long** (APPUI-1, fiche 57), au délai que règle l'accessibilité d'Android
  (`plateforme::doigt`), ouvre le menu du clic droit. Une question dessinée répond **au
  relâchement**, sur la réponse pressée, comme un bouton d'Android. Aucun dossier à « ouvrir
  dans l'explorateur ».
* **Le clavier ne tape pas des touches, il réécrit un texte** (CLAVIER-1, fiche 57) : `winit`
  (0.30 comme 0.31) jette son évènement ; Glucose relit l'état (`plateforme::clavier`) et le
  reflète (`text_edit::miroir`). Ses positions sont en **UTF-16** ; ce qu'on lui écrit passe
  par une file (relire juste après rend l'ancien, CLAVIER-2) ; sans le drapeau multiligne,
  Entrée n'écrit rien. Un champ de texte dans une question passe par le même miroir.
* **Bord à bord partout** (BORD-1, fiche 57) : les marges du système viennent de
  `MainActivity.onApplyWindowInsets` (`plateforme::marges`) ; la barre du haut englobe la barre
  d'état, et tout ce qui se pose en bas passe par `UiState::ecran_visible` — sans la navigation,
  ni le clavier, qui **recouvre** la fenêtre au lieu de la rétrécir. Un élément neuf collé à un
  bord se cale là, jamais sur l'écran entier.
* **Un emoji tapé n'a pas de dessin** dans la police de Glucose (fiche 57 § 6).
* Un message plus large que l'écran se **coupe** (le toast, la question) : 360 points de large.

## 5. Comment travailler

* `bash outils/android/construire.sh` (SDK et NDK dans `~/Android/Sdk`, Gradle 9.6+), et
  `cargo ndk -t arm64-v8a -P 21 clippy -p glucose-android -p glucose-desktop -- -D warnings`
  qui voit ce que Windows ne compile pas. La CI construit l'APK (tâche « apk »).
* **Regarder sans téléphone** : rendre l'interface hors écran à la taille d'un téléphone (par
  exemple 720 × 1600 à 2,0) et **montrer l'image** à l'utilisateur avant de lui demander un essai.
* **Le journal du téléphone** : `adb logcat -s Glucose` (la sortie de Glucose y est redirigée).
* **Java seulement à la frontière** : `MainActivity.java` reçoit ce que seul Java reçoit (une
  intention de partage, le résultat d'un sélecteur) et le confie à Rust — par un fichier dans le
  dossier de l'application, ou par JNI. Aucune logique en Java.
* Le téléphone de référence : **Redmi 9** (Helio G80, Mali-G52, Android 10, 720 × 1600) — le
  plus ancien connu de ses testeurs.
