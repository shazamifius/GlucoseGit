**Glucose {version}** — Glucose, réécrit en Rust.

**Ce qui change dans cette version**

* **Fini les gels d'une demi-seconde sur les portables à deux cartes graphiques** : Glucose
  dessine désormais sur la carte qui tient l'écran. Il pouvait dessiner sur l'autre, une
  session sur deux, et chaque image devait alors traverser d'une carte à l'autre.
* **Au dézoom, les poignées et le cadre d'une sélection suivent la taille des nœuds** : ils ne
  recouvrent plus des vignettes plus petites qu'eux, et une vignette sélectionnée se déplace
  au lieu de se redimensionner.
* **Au pavé tactile, Glucose reçoit les gestes comme un navigateur** : le pincement et le
  déplacement à deux doigts arrivent tels que les doigts les font, avec l'inertie de Windows.
  Le pincement s'arrête avec les doigts.
* **Les images glissées depuis Pinterest arrivent bien plus vite** (la page est demandée
  compressée : une seconde au lieu de vingt). Si une image ne vient vraiment pas, le lien posé
  à sa place dit pourquoi, et le clic droit « Remplacer par l'image » réessaie.
* **`Ctrl+N` demande toujours** avant de commencer un nouveau document : un raccourci tapé par
  erreur ne fait plus rien disparaître de l'écran.
* **Le mode référence au pavé tactile** : `Alt` + glisser déplace la fenêtre, `Alt` + pincer
  l'agrandit ou la rétrécit ; les panneaux s'effacent avec le reste de l'interface.
* **Les signets ramènent au même lieu dans une fenêtre de n'importe quelle taille** : ils
  retiennent le centre de la vue. Ceux posés avec une version précédente sont à reposer une fois.
* **La molette zoome bien plus vite** : trois crans doublent la taille.
* **À la souris, tout est instantané** : un cran de molette zoome tout de suite, le glisser au
  bouton du milieu suit la souris au pixel, et rien ne glisse quand on lâche. Au pavé tactile,
  le déplacement à deux doigts garde son élan.
* **Copier, couper, coller des nœuds** — textes, images, flèches, membranes —, y compris d'une
  fenêtre de Glucose à une autre : `Ctrl+C`, `Ctrl+X`, `Ctrl+V`, ou le clic droit. Le collage se
  pose sous la souris et se défait d'un seul `Ctrl+Z`.
* **Clic droit sur une image** : « Copier l'image », pour la coller dans Discord, un navigateur
  ou un logiciel de dessin ; « Enregistrer l'image sous… », qui rend le fichier d'origine, tel
  qu'il avait été posé.
* **`Ctrl+N`** : un nouveau document vierge (aussi au clic droit sur le vide). Un travail sans
  nom pose la question habituelle avant.
* **Le mode référence**, comme PureRef : `Ctrl+Maj+A` (ou le clic droit) retire toute
  l'interface et le cadre de la fenêtre, et la garde au premier plan — pour garder sa
  référence au-dessus de Blender. On la déplace au **bouton droit**, on la redimensionne par
  ses bords, et Glucose s'en souvient à la relance. Le même geste le défait.
* Supprimer une sélection qui mêle images et textes se défait désormais d'un seul `Ctrl+Z`.

| système | fichier |
|---|---|
| Windows 10 et 11 | `Glucose_{version}_x64-setup.exe` — installé pour l'utilisateur seul, sans droits d'administrateur |
| Linux, toute distribution | `Glucose_{version}_amd64.AppImage` |
| Debian, Ubuntu | `Glucose_{version}_amd64.deb` |
| Fedora | `Glucose-{version}-1.x86_64.rpm` |
| NixOS | `nix run github:shazamifius/GlucoseGit` |
| Android 5.0 et après | `Glucose_{version}_android.apk` — **une première version d'essai** : on y navigue et on y pince, le clavier et l'ouverture de fichiers viendront |

**Sous Windows**, un avertissement peut paraître (« Windows a protégé votre ordinateur ») :
Glucose n'est pas signé par un certificat payant. « Informations complémentaires », puis
« Exécuter quand même ».

**Sous Android**, ouvrir l'APK depuis le téléphone : il demande d'autoriser l'installation
d'applications inconnues pour l'application qui l'ouvre (Fichiers, Drive, le navigateur), puis
peut dire qu'elle n'est pas vérifiée — elle n'est pas sur le Play Store. « Installer quand même ».

**Avec Glucose Tauri déjà installé**, la mise à jour arrive d'elle-même, par la fenêtre
habituelle. Les documents et les images restent à leur place.

Les fichiers `.sig` et `latest.json` servent la mise à jour automatique.
