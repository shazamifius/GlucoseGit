# 164 — Les dossiers miroirs du disque : l'étude (25/11 vers 6 h 15, le relais, compte a)

Le plan du relais, point 15 : « les boutons qui manquent, sans sa parole » (INVENTAIRE § 3),
d'abord **les dossiers miroirs du disque** : un dossier du PC, vivant, dans le canevas (fiche 08
§ 5.3). Une étude : rien n'est codé. Sa question est dans son Glucose (`relais-q-dossier-disque`).

## 1. Ce que Tauri faisait (lu dans `tauri-v1.0.1`, `src/canvas/folderMirror.ts`, `scan_tree`)

* **Un dossier lâché sur la fenêtre** devient un dossier de Glucose, dont le tableau montre **un
  seul niveau** : les sous-dossiers en dossiers vides marqués `pendingScan`, lus à l'entrée
  (`expandFolderIfPending`, aussi par le fil d'Ariane). Plafond : 20 000 entrées par niveau ;
  4 Mo de texte lus en tout.
* **Une disposition en croix** : les sous-dossiers au centre, les « applications » (le reste) à
  gauche, les images et vidéos à droite, les textes en bas (tuiles de 200 à 360 de large, à la
  taille de leur contenu). Le tri (nom, type, taille, date) s'applique à chaque niveau, dossiers
  d'abord.
* **Les images liées par leur chemin, jamais embarquées** : *« un folder mirror ne doit pas gonfler
  le .glucose »*. Les textes lus en entier dans le nœud (`sourceFile` gardé) ; le reste en tuile
  qui lance le fichier, sauf une liste d'extensions qui exécutent (`FORBIDDEN_OPEN_EXTS`) ; masqués :
  les fichiers cachés, `Thumbs.db`, `desktop.ini`, les binaires (`SCAN_HIDE_EXTS`).
* **« Vivant » n'a jamais existé** : `mode` vaut toujours `"snapshot"`. Rien ne surveillait le
  disque, rien ne relisait un niveau déjà lu. Le miroir de Tauri est une **photographie** prise
  une fois, niveau par niveau.

## 2. Ce que Glucose Rust a déjà

* **Le format porte tout** : `FolderMirrorSource` (chemin, `mode` « snapshot » ou « live »,
  dernière lecture, motif, tri, `pending_scan`) est écrit et relu (`persist/document.rs`) ;
  `AssetRef::Link { href }` aussi (`persist/image.rs`, depuis le 11/09 : toutes les betas le
  lisent) ; `source_file` sur les textes et les post-it. **Aucun champ neuf** : ce chantier
  n'attend pas le clic « Publish » de la beta.6.
* **Le dépôt d'un dossier** fait aujourd'hui une tuile qui le **révèle** dans le gestionnaire de
  fichiers (`drop.rs`, `place_launcher` ; `pick.rs`, `reveal_launcher_target`) — jamais ouvrir ce
  qui s'exécute. Le dépôt marche sous Wayland (fiche 100, `selection_wayland.rs`).
* `rfd` choisit un dossier (`pick_folder`) : de quoi faire l'« outil Dossier » de l'INVENTAIRE.
* **Le cache des aperçus nomme déjà une photo non scellée par sa taille et sa date**
  (`renderer/apercu.rs`, `chemin`) : un fichier changé sur le disque a un autre aperçu. Exactement
  ce qu'une photo liée d'un dossier vivant demande.
* **Le nœud du problème** : toute image qui a un `src` est **scellée** dans le document à
  l'enregistrement (`persist/ecriture.rs`, `sceller`) — ses octets entrent dans l'histoire. Un
  dossier de 2 000 photos ferait un document de plusieurs gigaoctets. Une image du miroir doit
  porter `asset = Link`, et `sceller` la laisser dehors.

## 3. Ce que la recherche dit (25/11)

* **`notify`** est la surveillance de fichiers de Rust : inotify sous Linux et Android, FSEvents
  sous macOS, `ReadDirectoryChangesW` sous Windows ; la 9.0 en préversion (Rust 1.88 au moins), des
  « debouncers » à côté. Sources : [lib.rs/crates/notify](https://www.lib.rs/crates/notify),
  [docs.rs, notify 9.0.0-rc.3](https://docs.rs/crate/notify/9.0.0-rc.3),
  [Debian, notify-debouncer-full](https://tracker.debian.org/pkg/rust-notify-debouncer-full).
* **Une surveillance ne voit pas tout** : chokidar (la bibliothèque de Node.js) conseille de relire
  par intervalles sur un disque réseau, et prend un fichier supprimé puis recréé en 100 ms (ce que
  fait un éditeur qui enregistre) pour un fichier changé
  ([chokidar](https://unpkg.com/chokidar@6.0.0/README.md)) ; Eagle ne surveille que des dossiers
  locaux, « évitez les NAS », et **copie** ce qui arrive dans sa bibliothèque
  ([Eagle, l'import automatique](https://docs-en.eagle.cool/article/1287-the-auto-import)).
  Rien trouvé sur Allusion, dont les « emplacements » sont, de mémoire, liés et non copiés (non
  vérifié).

Conclusion : **une relecture doit exister de toute façon** (disque réseau, Android, un Glucose
fermé pendant que le dossier changeait) ; la surveillance n'est qu'un déclencheur de plus.

## 4. La conception proposée : une loi, des déclencheurs

**Une seule loi, dans le noyau, sans dépendance : réconcilier un niveau.** Elle prend ce que le
tableau du dossier montre (chaque nœud né d'un fichier, par son chemin) et ce que le disque vient
de rendre (nom, taille, date, inode sous Unix), et rend des gestes ordinaires — donc annulables,
enregistrés, partagés comme les autres :

* **un fichier nouveau** : un nœud posé dans sa zone (la croix de Tauri), à la suite des autres ;
* **un fichier changé** (taille ou date) : le texte relu, la photo redécodée (son aperçu change
  de nom tout seul, § 2) ;
* **un fichier renommé ou déplacé dans le niveau** : le même nœud, son chemin changé — ses
  flèches et ses domaines restent. Reconnu par l'inode sous Unix ; ailleurs, par une paire
  unique (disparu, apparu) de même taille, même date, même extension ;
* **un fichier disparu** : le nœud reste, marqué absent (la bordure pointillée de la fiche 08
  § 8), s'il porte quelque chose de sa main (une flèche, un domaine, un texte changé) ; sinon il
  part. Ce que sa main a ajouté ne se perd jamais à cause du disque.

**Un nœud qu'il retire ne revient pas.** Retirer un nœud du miroir laisse le fichier ; à la
relecture suivante, un fichier sans nœud serait pris pour un nouveau. La loi compare donc aussi à
**la liste lue la fois d'avant** : seul ce qui n'y était pas est nouveau. Cette liste vit dans
l'état de Glucose sur cette machine (`~/.local/state/glucose`), pas dans le document : le chemin
n'a de sens qu'ici. Perdue (une autre machine, le cache effacé), les nœuds retirés reviennent une
fois — le défaut est dit, il ne détruit rien.

**Les déclencheurs**, dans le bureau : entrer dans le tableau du dossier (le `pending_scan` de
Tauri, étendu à chaque entrée), la fenêtre qui reprend la main (`WindowEvent::Focused`) — deux
évènements, aucune constante, aucune dépendance — puis, si l'usage le demande, `notify` sur le
seul niveau ouvert (une surveillance, pas une par sous-dossier ; une note `decisions/21`).

**Jamais d'écriture sur son disque.** Retirer un nœud du miroir ne supprime aucun fichier ;
déplacer un nœud ne déplace rien. **Les photos liées** (`asset = Link`) ne se scellent pas ; leurs
aperçus vivent dans le cache, comme aujourd'hui.

**Ce qui reste ouvert, technique, décidé plus tard** : le partage (un miroir vit sur une machine :
dans un canevas partagé, l'autre voit les nœuds, et les photos liées doivent voyager comme les
photos de B1, fiche 119, ou se dire absentes) ; Android (un dossier par le SAF,
`ACTION_OPEN_DOCUMENT_TREE`, sans surveillance : les deux premiers déclencheurs suffisent) ; un
niveau de 20 000 photos et les 100 images par seconde (les aperçus d'abord, à mesurer).

## 5. Le chemin, en chantiers

1. **DISQUE-1** — le geste : un dossier lâché (et « Dossier du disque… » au menu du canevas, par
   `pick_folder`) devient un dossier miroir ; un niveau lu, les sous-dossiers à l'entrée ; photos
   liées, non scellées ; textes lus ; le reste en tuile qui révèle. La parité avec Tauri.
2. **DISQUE-2** — réconcilier (§ 4) à chaque entrée et quand la fenêtre reprend la main : le
   « vivant » sans dépendance. Ses épreuves sur un dossier fabriqué (ajout, changement, renommage,
   disparition, un nœud qui porte une flèche).
3. **DISQUE-3** — `notify`, si sa réponse dit qu'un dossier change pendant qu'il le regarde (des
   rendus de Blender, des captures qui tombent).

## 6. Ce qui attend sa parole (`relais-q-dossier-disque`)

Un seul sens (le disque vers Glucose), ou les deux : un texte du miroir changé dans Glucose
s'écrit-il dans son fichier ? Le défaut proposé : **un seul sens** — changer le texte d'un nœud du
miroir le détache de son fichier (il devient un texte à lui, que le disque ne touche plus) ; écrire
dans ses fichiers depuis Glucose ferait de Glucose un éditeur de son disque (le chemin d'Obsidian),
une autre ambition, à décider par lui. Et : à quoi lui servira-t-il — des références, des rendus
qui tombent pendant qu'il travaille, un projet entier ? La réponse dit si DISQUE-3 vient tout de
suite.
