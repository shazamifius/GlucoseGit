**Glucose {version}** — Glucose, réécrit en Rust.

**Ce qui change dans cette version**

* **Le Storyboard** : « Activer », puis un clic pose la grille — ou un rectangle tracé à la
  souris se remplit de cases. Une poignée au coin l'agrandit ou le rétrécit ; une case qui porte
  une image ou une description n'est jamais retirée. Une image lâchée dans une case s'y ajuste.
  Une case se prend par sa bande et se lâche à une autre place : les autres se décalent avec
  leurs images, et les numéros suivent. Deux clics sur la bande pour la décrire, `Tab` pour
  passer à la suivante.
* **Les membranes et les dossiers se tracent** : l'outil choisi, un glisser leur donne leur
  taille ; un simple clic les pose comme avant.
* **Tout cadrer passe à `Alt+F`** ; `Ctrl+F` est gardé pour la recherche, qui vient. `F` cadre
  toujours la sélection.
* **Le texte et les membranes redeviennent nets dès qu'on s'arrête** : à certains niveaux de
  zoom, des textes restaient flous, parfois par carrés, jusqu'au geste suivant. Glucose finit
  désormais chaque image avant de s'endormir.
* **L'interface coûte moins à chaque image** : la bande, le fil d'Ariane, la minimap et le rail
  sont posés par la carte graphique au lieu d'être redessinés. Le rectangle de la minimap
  retrouve le trait fin de Glucose Tauri.
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
  Storyboard, Mood Board) posés où l'on veut ; leurs zones se déplacent et se redimensionnent par
  leur bandeau ; « Par slot preset » range les images de chaque zone dans la zone.
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
  numéros), quelle étape du dessin a ralenti un geste trop lent, et, au téléphone, quand Glucose
  passe en arrière-plan — toujours aucun contenu de tes documents. Tout est décrit dans le
  `LISEZ-MOI.txt` de son dossier.

**À savoir avant d'installer**

* **Un document enregistré par cette version ne s'ouvre plus dans une version plus ancienne** de
  Glucose Rust : il porte la grille de son storyboard. Tous les documents d'avant s'ouvrent.
* **Au téléphone, une prudence** : sur un téléphone, une longue session a fini par figer tout le
  système, jusqu'à son redémarrage. La cause est recherchée. Si Glucose se met à saccader de plus
  en plus, ferme-le.

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
