**Glucose {version}** — Glucose, réécrit en Rust.

**Ce qui change depuis la 2.0.3-beta.2**

* **Au téléphone, Glucose ne prend plus la mémoire sans regarder ce qui reste** : une image ne
  commence à se décoder que si l'ensemble des décodages en cours tient dans la moitié de ce que
  l'appareil laisse libre à cet instant ; et une photo décodée occupe jusqu'à 40 % de mémoire de
  moins pendant son décodage (254 Mo au lieu de 445 pour une photo de 50 mégapixels).
* **Quand Android demande de la mémoire, Glucose la rend** : derrière une autre application, la
  carte graphique lâche ses images et le reste est offert au système, qui le reprend s'il en a
  besoin ; au retour, tout revient de lui-même.
* **Le journal technique, si tu l'as accepté**, dit désormais la mémoire — ce que Glucose occupe
  et ce que l'appareil laisse, en mégaoctets, quand c'est un record de la session — et les
  instants où le système en demande. Toujours aucun contenu de tes documents.
* **Le Storyboard, complet** :
  * plusieurs images lâchées sur une case remplissent les cases vides à la suite, et le
    storyboard s'allonge s'il le faut ;
  * « Storyboard de la sélection » au clic droit : une case par image choisie ;
  * les deux « + » d'une case choisie insèrent une case avant ou après elle ;
  * **la lecture** (« ▶ Lire ») : chaque case plein écran, une durée par case, ses textes en
    sous-titre, ← → pour avancer, Espace pour la pause ;
  * par case : sa durée, son plan (du plan général au très gros plan), des champs nommés
    (Dialogue, Action, Notes…) ;
  * **plusieurs storyboards par tableau** (« + Nouveau », ‹ 2 / 3 ›) ;
  * **l'export en planches**, en PDF ou en images, au format A.
* **La grille du Storyboard part du coin de la souris**, comme quand on la trace.
* **L'export SVG porte tout ce que le canevas montre** : photos, membranes, dossiers, formules,
  zones et cases, dans le même ordre ; le texte y est en tracés, lisible tel quel par n'importe
  quel logiciel (navigateur, Inkscape, Illustrator), sans police à installer.
* **Le dossier, comme dans Glucose Tauri** : son titre et son compteur à leur place, et un dossier
  vide dit « Double-clic pour ouvrir ».
* **`Alt+F` cadre aussi un tableau qui ne porte qu'un storyboard ou des zones.**
* **Le panneau « Presets » s'appelle « Mood board »** : ses six gabarits de zones.

**À savoir avant d'installer**

* **Au téléphone, une prudence** : sur un téléphone, une longue session a fini par figer tout le
  système, jusqu'à son redémarrage. Cette version empêche Glucose de prendre plus de mémoire que
  l'appareil n'en laisse — la cause la plus probable, **pas encore prouvée**. Si Glucose se met
  à saccader de plus en plus, ferme-le.

| système | fichier |
|---|---|
| Windows 10 et 11 | `Glucose_{version}_x64-setup.exe` — installé pour l'utilisateur seul, sans droits d'administrateur |
| Linux, toute distribution | `Glucose_{version}_amd64.AppImage` |
| Debian, Ubuntu | `Glucose_{version}_amd64.deb` |
| Fedora | `Glucose-{version}-1.x86_64.rpm` |
| NixOS | `nix run github:shazamifius/GlucoseGit` |
| Android 5.0 et après | `Glucose_{version}_android.apk` — au doigt : deux touchers sur le vide ouvrent le menu, un appui long fait le clic droit, deux doigts touchés annulent, trois rétablissent |

**Sous Windows**, un avertissement peut paraître (« Windows a protégé votre ordinateur ») :
Glucose n'est pas signé par un certificat payant. « Informations complémentaires », puis
« Exécuter quand même ».

**Sous Android**, ouvrir l'APK depuis le téléphone : il demande d'autoriser l'installation
d'applications inconnues pour l'application qui l'ouvre (Fichiers, Drive, le navigateur), puis
peut dire qu'elle n'est pas vérifiée — elle n'est pas sur le Play Store. « Installer quand même ».
Les versions suivantes arrivent d'elles-mêmes.

**Avec Glucose Tauri ou une version précédente déjà installée**, la mise à jour arrive
d'elle-même, par la fenêtre habituelle. Les documents et les images restent à leur place.

Les fichiers `.sig` et `latest.json` servent la mise à jour automatique.
