# 55 — Glucose sur son téléphone, le serveur en ligne, la 2.0.2-beta.1, et le rail

> Suite de la session de la fiche [54](54-LA-NUIT-DU-PAVE-ET-ANDROID.md), le 08/10/2026. Son mot :
> *« oui, ça fonctionne sur téléphone »*. Puis sa liste pour le téléphone, et sa décision sur la
> vidéo : *« les vidéos Insta et TikTok n'ont rien à faire sur PC, c'est fait pour téléphone…
> pour l'instant, se concentrer uniquement sur le transfert d'images »*.

---

## 1. Ce qui est en ligne, et vérifié

* **La CI de `89510e2` : dix tâches vertes** — Linux, les deux Mac, NixOS, l'APK sur GitHub.
  La tâche des paquets Linux s'était figée une heure et demie à installer ses dépendances chez
  GitHub, avant notre code : annulée, relancée seule, verte.
* **Le serveur de la boîte noire** : un Worker dans son compte Cloudflare,
  `https://glucose-boite-noire.ferme-nilslamber.workers.dev`, sa base D1 en Europe de l'Ouest
  (`5d5141eb-…`, dans `outils/telemetrie/wrangler.jsonc`). Éprouvé sur Internet : reçue, déjà
  reçue, mot refusé (422), aucune porte pour lire (404), effacée. Glucose connaît l'adresse,
  pour le vrai programme seulement (`telemetrie::serveur()` vaut `None` en épreuve).
  * `wrangler login` attend **120 s**, figées dans son code : trop court sur sa connexion. Un
    script chargé avant lui (`--require`, dans le scratchpad, jamais dans le dépôt) a allongé ce
    seul délai à 30 minutes.
  * Le sous-domaine `ferme-nilslamber` est celui de son compte : visible de tous — à lui de dire
    s'il le garde.
  * Un fichier du cache de `wrangler` (le nom du compte, son e-mail) a été commité par erreur,
    retiré **avant** tout envoi, et `outils/telemetrie/.wrangler/` est ignoré.
* **Glucose tourne sur son Redmi 9** (Helio G80, Android 10) : l'APK construit ici (Gradle 9.8.1
  arrivé à 5 Ko/s, puis 30 minutes de construction), posé sur son Bureau. *« Oui, ça fonctionne »*.
* **La clé de Glucose pour Android** : RSA 4096, cent ans, empreinte `3f2af005…5310359fcad`, dans
  `Documents\Glucose-cle-android` (avec son `LISEZ-MOI.txt`) et dans les secrets du dépôt
  (`ANDROID_SIGNING_KEYSTORE`, `ANDROID_SIGNING_PASSWORD`). **Il doit en garder une copie hors
  du PC.** Gradle signe l'APK de publication quand elle lui est donnée, et calcule un
  `versionCode` qui monte avec la version.
* **La publication de la 2.0.2-beta.1, en brouillon**, lancée à sa demande : installeur, trois
  paquets Linux, **l'APK signé — sa signature prouvée par l'empreinte du certificat** —,
  `latest.json`. Le clic « Publish release » est **son** geste ; au 08/10 il ne l'a pas encore
  fait (`latest` est toujours la 2.0.1-beta.1).

## 2. La skill du téléphone

Il a demandé d'installer les skills de conception mobile. Celles qu'on trouve (wshobson/agents,
`mobile-android-design`) enseignent **Jetpack Compose et Material You** — l'inverse de `style.md`.
Lue en entier, **non installée**. À la place, `.claude/skills/glucose-telephone/SKILL.md` : ce que
le téléphone impose au dessin de Glucose (48 dp, aucun survol, bord à bord imposé par Android 16,
le pouce) et ce que le système offre (le partage `ACTION_SEND`, le sélecteur de photos, le SAF, le
retour prédictif).

## 3. Le rail (`ui/rail.rs`)

Sa demande : les boutons du haut sur le côté, avec une flèche pour déplier et replier.

* **La mesure décide** : quand même les icônes seules ne tiennent plus, la barre devient un rail ;
  la bande ne garde que les onglets (`topbar_height` vaut 0, `header_height` les onglets seuls).
* **Une seule liste, deux mises en page** (L4) : `boutons::les_boutons` produit la liste de la
  barre, le rail la repose en grille ; `effet_du_bouton` (l'aimant, la collaboration) sert les
  deux.
* **Au doigt** : cases de 48 dp ; rangées tant que l'écran en tient, colonnes qu'il faut — la
  première version fixait deux colonnes et sortait de l'écran couché (vu sur l'aperçu).
* **Décidé avant la scène** (`Renderer::synchroniser_les_caches`) : décidé dans l'interface, il
  arrivait une image trop tard — l'épreuve des caches a vu 120 pixels d'écart.
* `examples/apercu_telephone.rs` rend Glucose à 720 × 1600 à 200 %, debout et couché.
* **Non vu à son écran.** Doute à lui soumettre : des icônes sans libellé au doigt, sans survol
  pour les nommer — peut-être une colonne avec libellés, à faire défiler.

## 4. Ce qui reste de sa liste, dans l'ordre

1. **Le partage d'images vers Glucose** (sa priorité) : un filtre `ACTION_SEND` /
   `ACTION_SEND_MULTIPLE` (`image/*`, `text/plain`) dans le manifeste ; `MainActivity.java` reçoit
   l'intention et confie le contenu à Rust (un fichier dans le dossier de l'application, ou JNI —
   la caisse `jni` est déjà dans l'arbre d'Android, tirée par une autre) ; Rust le pose par le
   chemin du dépôt (`interactions::depot_web`). Un lien partagé passe par le rapatriement, qui
   sait déjà chercher l'image d'une épingle.
2. **Le double toucher** sur le vide : un menu, « Ajouter des images » → le sélecteur de photos
   (`ACTION_PICK_IMAGES`, `ACTION_OPEN_DOCUMENT` avant Android 13).
3. **Les documents au téléphone** : la liste des canevas, créer, renommer, ouvrir, supprimer.
4. **La vidéo** : remise à plus tard, à sa demande. Le diagnostic : Glucose Tauri téléchargeait
   par **yt-dlp** (programme externe, épinglé, vérifié) et lisait par le **navigateur** de Tauri ;
   Glucose Rust lirait par les décodeurs des systèmes (Media Foundation, MediaCodec, VideoToolbox
   — aucune caisse) et GStreamer sous Linux ; yt-dlp ne tourne pas sous Android. **Il a dit qu'il
   accepterait quelques dépendances exceptionnelles.**
5. **La mise à jour automatique sous Android** : télécharger l'APK, vérifier sa signature de
   Glucose (minisign, à ajouter à la publication), demander l'installation au système (Android
   confirme toujours).

## 5. Prouvé, et pas prouvé

* **Prouvé** : 2 119 épreuves ; clippy strict sous Windows et pour Android ; douze sabotages du
  rail, douze chutes (un mal posé, réécrit) ; la publication verte ; le serveur sur Internet ;
  Glucose qui démarre sur son téléphone.
* **Pas prouvé** : le rail à son écran ; la télémétrie depuis un vrai Glucose (personne n'a encore
  répondu à la question) ; la question de la télémétrie sous Android (aucun dialogue n'y existe :
  `oui_ou_non` répond « non »).

---

**Retour** : [`00-INDEX.md`](00-INDEX.md)
