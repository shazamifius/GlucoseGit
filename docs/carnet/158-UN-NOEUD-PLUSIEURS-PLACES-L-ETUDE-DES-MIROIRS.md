# 158 — Un nœud, plusieurs places : l'étude des miroirs (25/11 vers 3 h 30, le relais, compte b)

Le plan du relais, point 12 : « les boutons qui manquent » (SUITE § 2, INVENTAIRE § 3), d'abord
**les miroirs** de Tauri (`Ctrl+Maj+M`, « un même nœud à deux endroits »). Une étude : rien n'est
codé. Sa question est dans son Glucose (`relais-q-miroirs`).

## 1. Ce que Tauri faisait (lu dans `tauri-v1.0.1`)

* `Ctrl+Maj+M` (`App.tsx`, l. 290) : pour chaque texte et chaque image choisis, une **copie** à 40
  points en bas à droite, qui porte `mirrorOf` = l'original au bout de la chaîne
  (`findOriginalAnnotation`, au plus 16 maillons). Les dossiers : le magasin seul, sans bouton.
* Un badge ↻ au coin haut gauche (`MirrorBadge`) : un clic va à l'original.
* Supprimer l'original supprime ses miroirs (la cascade de `removeImages`, `removeAnnotations`).
* **Aucune propagation** : la copie est figée (fiche 03 § 10.4). Écrire dans l'original ne change pas
  le miroir. Le nom promettait un « lien vivant » ; le code faisait un duplicata avec une flèche de
  retour.

Ce qu'en garde Rust : `mirror_of` sur les images et les dossiers (écrit, lu, jamais posé) ; la
cascade d'images (`close_image_mirror_cascade`) ; `mirror_graph` (les cycles des dossiers). Rien
pour les textes.

## 2. Ce que font les autres

* **Heptabase** : une carte appartient à la bibliothèque, jamais à un tableau ; un tableau n'en tient
  qu'une **place**, et la même carte peut être posée sur plusieurs tableaux à la fois. Son panneau
  d'informations dit sur quels tableaux elle est ([fundamental elements](https://wiki.heptabase.com/fundamental-elements),
  [la mise à jour du 29/11/2024](https://wiki.heptabase.com/newsletters/2024-11-29), le panneau
  des emplacements ; [cartes réutilisables](https://wiki.heptabase.com/nested-whiteboard)). Qu'une
  modification se voie partout n'est écrit que par un tiers ([une revue](https://dev.to/pickuma/heptabase-review-visual-note-taking-for-thinking-through-problems-5daa)).
* **Obsidian** : rien d'officiel ; le greffon [Mirror](https://community.obsidian.md/plugins/mirror)
  dit l'idée : le bloc inclus reste attaché à sa source, écrire d'un côté écrit le même contenu.

## 3. La proposition : un contenu, plusieurs places

**Un miroir ne copie rien** : il est une place (son tableau, sa membrane, sa position, sa taille) qui
lit tout le reste — le texte, la couleur, les domaines, la date, la photo — dans l'original. Écrire
dans un miroir écrit l'original. **La propagation disparaît** au lieu d'être codée : il n'y a qu'un
contenu, rien ne peut diverger, rien à synchroniser — ni dans le partage, ni dans l'histoire, où un
geste d'écriture nomme l'original.

* **Pas de chaîne** : un miroir d'un miroir pointe l'original ; la borne de 16 maillons de Tauri
  disparaît avec elle.
* **Les flèches tiennent à une place** (comme on les voit) ; les domaines et la parenté, au contenu.
* **Retirer** un miroir retire sa place. **Retirer l'original** : proposé, **le plus ancien miroir
  devient l'original** — un contenu vit tant qu'il a une place (Heptabase) ; la cascade de Tauri
  effaçait toutes les places d'un geste. `Ctrl+Z` remet tout.
* **Le signe** : un ↻ discret au coin, comme Tauri ; un clic mène à la place suivante, en boucle (pas
  seulement à l'original : avec trois places, on les visite toutes).
* **`Ctrl+Maj+M`** pose le miroir de la sélection sous la main ; la palette et le menu d'un nœud
  aussi, et une IA par la porte (`glucose_lire` dit `miroir_de`).

## 4. Ce qui reste à trancher

* **Le mot** : « miroir » dit déjà deux autres choses dans Glucose — retourner une image (MIROIR-1,
  fiche 118) et le miroir de fichiers en Markdown (VISION § 8, 22 et 23/11). Proposé : **« reflet »**.
  À sa parole (`relais-q-miroirs`).
* **Le format** : les textes n'ont pas de champ pour cela. Un champ en fin d'enregistrement, avec une
  version de schéma (la façon de MIROIR-1, `persist/image.rs`) ; comme TEXT-FIT-3, **après son clic
  « Publish » de la beta.6** — ses autres appareils lisent encore la beta.5.
* Les images ont déjà `mirror_of` : la même lecture les sert (une photo à deux endroits, deux tailles).
