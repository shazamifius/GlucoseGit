**Glucose {version}** — Glucose, réécrit en Rust.

**Ce qui change dans cette version**

* **Le texte et les membranes redeviennent nets dès qu'on s'arrête** : à certains niveaux de
  zoom, des textes restaient flous, parfois par carrés, jusqu'au geste suivant. Glucose finit
  désormais chaque image avant de s'endormir.
* **Chaque commande a son entrée dans le clic droit** — au téléphone, deux touchers sur le
  vide : annuler et rétablir, tout cadrer, cadrer la sélection, les **signets** (y aller, en
  poser un, en remplacer un), enregistrer, enregistrer sous, exporter. Plus rien ne demande un
  clavier. Sur un petit écran, le menu se range en colonnes au lieu de sortir de l'écran.
* **Un seul « Copier »**, et une image reste une image : collée dans Discord, un navigateur ou
  Paint, une image arrive comme une image, plusieurs comme des fichiers. Sous Windows, on peut
  aussi **glisser** des nœuds vers Discord, Google Docs ou le bureau. Sous Linux, le même
  copier, et une copie ne disparaît plus quand Glucose la quitte.
* **Ordonner** : les tris **par la couleur** (couleur, du sombre au clair, du clair au sombre),
  le champ « Colonnes », et des champs qu'on règle en glissant ou en tapant.
* **Les Presets** : six gabarits (CharaDesign, Environment, Creature Design, Props & Items,
  Storyboard, Mood Board) posés où l'on veut ; leurs zones se déplacent et se redimensionnent par leur bandeau ; « Par slot preset »
  range les images de chaque zone dans la zone.
* **Exporter en image** : la sélection, ou tout le tableau, en PNG.
* **La question passe par-dessus les panneaux**, et aucun panneau ne s'ouvre au lancement.
* **Le canevas défile** quand on tient un nœud près du bord de la fenêtre (une première
  version).
* **Au téléphone** : exporter et enregistrer une image passent par le **partage** d'Android
  (Fichiers, la galerie, une conversation…) ; « Enregistrer » range un travail sans nom,
  « Enregistrer sous… » le renomme, « Importer un document… » ajoute un autre document en
  onglets ; un dossier choisi a son menu ; les grandes photos ne dépassent plus ce que la puce
  graphique accepte.
* **Le journal technique, si tu l'as accepté**, dit désormais la carte graphique (par ses
  numéros) et, quand un geste a été trop lent, quelle étape du dessin l'a ralenti — toujours
  aucun contenu de tes documents. Tout est décrit dans le `LISEZ-MOI.txt` de son dossier.

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
