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
  dossier de 2 000 photos ferait un document de plusieurs gigaoctets. `sceller` doit laisser les
  photos du miroir dehors — **par leur lieu, pas par `asset = Link`** (§ 7).

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
déplacer un nœud ne déplace rien. **Les photos du miroir** ne se scellent pas ; leurs aperçus
vivent dans le cache, comme aujourd'hui.

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

## 7. DISQUE-1, fait (25/11 vers 6 h 40, compte a)

**Un dossier lâché devient son miroir**, son premier niveau lu dans le geste du dépôt (un seul
`Ctrl+Z`) ; un dossier qu'on ne peut pas lire reste la tuile qui y mène. Un sous-dossier se lit au
moment d'y entrer. Le code : `core/miroir_disque.rs` (ce qui se tait, l'ordre), `core/store/
miroir_disque.rs` (le remplissage, un geste), `desktop/interactions/miroir_disque.rs` (le disque).
Ce qui a changé en codant, et pourquoi :

* **Le signe d'une photo qui reste sur le disque est son lieu**, pas `asset = Link` : la suite l'a
  dit — un document *portable* de Tauri marque ses images `Link` (`asset:…`), et elles doivent se
  sceller (`test_un_document_tauri_portable_…`). Une photo reste dehors **tant qu'elle vit dans le
  tableau d'un miroir** (`tableaux_du_disque`) ; glissée ou copiée sur son canevas, elle devient
  sienne et se scelle. `Transaction::images_posees_et_ou` donne le tableau de chaque photo posée.
* **La lecture à l'entrée est un geste** de l'histoire (on verra dans la Time Machine quand le
  disque a été lu ; elle s'enregistre et se partage), **déclenchée au moment d'entrer seulement** :
  déclenchée par l'état (« ce dossier attend »), un `Ctrl+Z` qui vide le dossier où l'on est l'aurait
  relu à l'image suivante — la main enfermée, sans pouvoir défaire plus loin. Épreuve et sabotage.
* **La disposition est COLONNES-1** (la loi des membranes, la proportion de l'écran) : la croix de
  Tauri et ses quatre constantes ont disparu. À regarder à son écran.
* **Tout ce qui commence par un point se tait**, `.env` compris (Tauri le montrait) : un canevas se
  partage et des IA le lisent.
* Un dossier illisible à l'entrée le dit par le message du dépôt (le cliquet 2 refuse un site de
  plus ; lire un sous-dossier est la suite différée du dépôt) et reste « à lire ».
* **« Dossier du disque… »**, au menu du vide et à la palette : le sélecteur du système
  (`rfd::pick_folder`, `dialogue::Mode::Dossier`), puis le dossier choisi se lâche là où l'on
  regarde — le même geste que le dépôt. Pas sur le téléphone (le SAF, plus tard).
* Les photos à sceller : `images_hors_du_disque` (le noyau) — le cliquet 1 refusait que l'écriture
  lise les tableaux elle-même.

Épreuves : 8 dans le noyau, 5 dans le bureau ; l'épreuve du dépôt d'un dossier réécrite (« son
miroir, pas une carte »). Sabotages : `outils/sabotages/disque-1.json`. **Piège payé encore** : deux
sabotages « ne compilent pas » — le cache incrémental (`target/debug/incremental/glucose_desktop-*`),
vidé, ils tombent.

Suite entière verte (3 742) ; Windows, Mac et Android verts (`verifier_cibles.sh`). **Piège payé** :
`saboter.py` joue dans l'arbre du dépôt — l'éditer pendant qu'il tourne fait tomber un sabotage en
« ne compile pas » ; attendre sa fin.

**Pas encore** : le lire hors du fil de l'image (mesuré au § 8 : à faire avec DISQUE-2) ; son écran ; DISQUE-2 (le vivant) ; le partage (les photos des gestes partent par B1,
lues à leur clé — ici leur chemin : à essayer).


## 8. DISQUE-1 au pilote : la première vue, et l'en-tête seul (25/11 vers 7 h 50, compte a)

Le vrai Glucose hors écran (`piloter.sh presse-fichiers` puis `Ctrl+V`) : un petit dossier (4 photos,
2 textes, 2 sous-dossiers, un `.zip`, un `.env`) se lit comme prévu — le `.env` se tait, le `.zip` en
tuile, les sous-dossiers lus à l'entrée, le fil d'Ariane suit. **Deux défauts trouvés, corrigés :**

1. **La caméra d'un miroir lu posait l'origine sous la bande** : un tableau neuf n'a jamais eu de
   vue, et le haut du dossier se cachait sous le fil d'Ariane ; 600 photos n'en montraient que six.
   **La première vue d'un miroir lu montre tout son contenu, jamais plus grand que nature**
   (`premiere_vue` : le cadrage d'`Alt+F`, ramené à l'échelle 1 quand il grossirait, centré sous la
   bande), posée par la lecture qui le remplit — au dépôt comme à l'entrée ; une vue qu'on s'est faite
   ne se reprend jamais. `remplir_le_miroir` (le noyau) rend le tableau rempli au lieu d'un booléen :
   le bureau n'a pas à le chercher dans le modèle (le cliquet 1 le refusait).
2. **`into_dimensions` lit tout un JPEG pour en dire la taille** : le décodeur JPEG d'`image` 0.25
   commence par `read_to_end` (`codecs/jpeg/decoder.rs`, lu dans les sources ; `imagesize` existe pour
   cela, [docs.rs](https://docs.rs/crate/imagesize/latest/source/src/lib.rs)). Mesuré
   (`GLUCOSE_PERF=1`, la mesure `miroir_lu`, neuve) sur 150 photos d'appareil de 7,8 Mo : **540 ms
   d'image figée à froid** sur le disque NVMe de son poste (le cache chassé par `posix_fadvise`), 46 ms
   à chaud. La taille d'un JPEG est dans son marqueur `SOFn`, avant les données : **on y va de segment
   en segment en sautant chaque charge** (`interactions/dimensions.rs`, ITU T.81 annexe B ; une
   quarantaine de lignes, aucune dépendance). Les autres formats passaient déjà par leur en-tête. Un
   JPEG que la marche ne comprend pas (hauteur différée au marqueur DNL) retombe sur le décodeur entier.
   **Après : 17,6 ms à froid, 1,3 ms à chaud** (30 fois moins) ; 600 JPEG : 63,5 ms à froid, 4,8 à
   chaud. Le dépôt d'une photo seule (`place_image_file`) en profite aussi : lâcher 50 photos de
   téléphone ne lit plus 400 Mo sur le fil de l'image.

Épreuves : 1 de la première vue (une photo seule à l'échelle 1, 40 grandes photos en recul, tout sous
la bande et centré, une vue à soi gardée), 4 de l'en-tête (un squelette d'appareil — EXIF de 60 Ko,
remplissage, table de Huffman dans la plage des SOF, SOF progressif, 2 Mo de données — lu en moins de
64 Ko ; ce qui n'a pas de taille n'en invente pas ; un vrai JPEG a la taille du décodeur entier ; un
fichier sur le disque se mesure par la marche). Sabotages : `disque-1-vue.json` (3 sur 3),
`en-tete-jpeg.json` (6 sur 6). Suite verte (3 747), Windows, Mac et Android verts.

**Ce qui reste, et l'argument pour la suite** : 63 ms à froid pour 600 photos, c'est six images
perdues sur un NVMe — et des secondes sur un disque dur, une clé ou un partage réseau (aucune machine
exclue). **La lecture d'un niveau doit quitter le fil de l'image** : DISQUE-2 relira à chaque entrée,
elle deviendra fréquente ; c'est là qu'elle passe sur un fil (le dossier montré « en lecture » le
temps qu'il lise, le niveau posé comme un geste à son retour). À concevoir avec DISQUE-2.

**Vu au passage, sans rapport avec DISQUE-1** : une image de 12 ms à l'arrivée de 600 vignettes
(`textures` 10,2 ms, 443 faites dans la même image) — le budget de la cascade suit la période de
l'écran (16,7 ms à 60 Hz), pas le plancher de 10 ms de la charte. Rien n'a sauté de battement.

**Piège payé** : `~/.cache/glucose-bancs/cible-q12` s'était abîmé (une dépendance compilée illisible :
le compilateur panique en lisant ses métadonnées, « invalid enum variant tag while decoding
`Constness` », même sur le code commité). Un dossier de compilation neuf (`cible-q13`) compile ; l'ancien
est effacé.


## 9. DISQUE-2, le miroir vivant (25/11 vers 11 h 30, compte a)

Sans sa réponse à `relais-q-dossier-disque`, le défaut proposé : **un seul sens**. Le code :
`core/miroir_disque/reconcilier.rs` (la loi), `core/store/miroir_disque.rs` (le geste),
`desktop/interactions/miroir_disque/` (`atelier.rs`, le fil du disque ; `appliquer.rs`, du plan aux
nœuds).

**La loi** (`reconcilier`, pure, sans dépendance) prend le tableau, ce que le disque rend, et **ce que
le miroir avait vu** (chemin → taille, date, et pour un texte l'empreinte du Markdown qu'il y avait
posé), et rend un plan : ce qui entre, ce qui suit son fichier, ce qui part, et ce qu'il aura vu.

* **Les nœuds qu'elle gère** : ceux dont le fichier est de ce niveau (son dossier est la racine du
  miroir), ou que le miroir avait vus. Ses textes, ses flèches, une photo qu'il a collée là : à lui,
  jamais touchés.
* **Un fichier sans nœud** que le miroir avait vu à l'identique a été **refusé** (il a retiré son nœud)
  et ne revient pas ; **changé depuis** (un rendu refait sous le même nom), il revient.
* **Un renommage** : une paire unique (disparu, apparu) de même taille, même date, même extension —
  une seule règle pour tous les systèmes, sans l'inode d'Unix. Deux candidats pareils : la loi ne
  devine pas (le nœud part, les deux fichiers entrent). L'épreuve l'a montré d'elle-même : deux PNG
  identiques écrits dans la même seconde.
* **Un seul sens** : un texte dont le contenu n'a plus l'empreinte de ce que le miroir y avait posé a
  été récrit de sa main — détaché, il ne se relit plus jamais. Sans mémoire, un texte qui n'est pas ce
  que le fichier donne est tenu pour sien. L'empreinte : les huit premiers octets du SHA-256 du noyau
  (le hachage de la bibliothèque standard ne promet pas d'être le même d'une version à l'autre).
* **Un fichier disparu** emporte son nœud, sauf ce que sa main y a mis : une flèche, un domaine, une
  membrane, un texte récrit, une tuile renommée. **Un sous-dossier déjà lu reste** (son contenu peut
  porter son travail ; le parcourir récursivement est une question pour plus tard), un jamais lu part.
  Un miroir illisible (un disque débranché) ne perd rien : une lecture en erreur ne réconcilie pas.

**Le geste** (`Store::reconcilier_le_miroir`) : un seul, **signé « Glucose »** (comme EXIF-1),
défaisable ; le neuf se dispose par COLONNES-1 **sous ce qu'il a rangé**, sans rien déplacer ; une
lecture sans rien de neuf n'écrit **aucun** geste (sinon un geste à chaque retour dans la fenêtre) ;
pendant un geste ouvert, la lecture attend qu'il se ferme.

**Ce que le miroir a vu** se garde dans le dossier de l'application (`miroirs/`, un fichier par miroir,
nommé par l'empreinte du fichier du document et du tableau), pas dans le document : un chemin n'a de
sens que sur cette machine, et il faudrait un champ neuf. **Ses défauts, dits** : un « Enregistrer
sous » ou une autre machine perdent cette mémoire — les nœuds retirés reviennent une fois (jamais
l'inverse : rien ne se cache). Un `Ctrl+Z` d'une relecture vaut refus (le nœud qu'elle avait ajouté,
retiré par sa main, ne revient pas tant que son fichier ne change pas) — sauf la première lecture :
défaite, le miroir redevient « à lire », et **un miroir à lire ignore ce qu'il avait vu**. Un plantage
entre l'écriture de cette mémoire et celle de l'histoire ferait prendre les fichiers ajoutés juste
avant pour refusés : fenêtre de quelques millisecondes, non fermée.

**Les déclencheurs** : **chaque entrée** dans le tableau d'un miroir (plus seulement la première), et
**la fenêtre qui reprend la main** (`WindowEvent::Focused`). **Le dossier lâché n'est plus lu au
dépôt** : il l'est à l'entrée, comme ses sous-dossiers — une seule loi pour tous les niveaux, aucun
disque lu dans le geste, et un `Ctrl+Z` après le dépôt l'emporte entier. Fermé, il dit « Double-clic
pour ouvrir », ce qui est vrai.

**Hors du fil de l'image** (la fin du § 8) : un fil, « l'atelier des miroirs », lit le niveau, le
Markdown des seuls textes changés ou inconnus, et la mémoire d'un miroir quand l'application ne l'a
plus ; il écrit cette mémoire, dans l'ordre (une écriture et la lecture suivante ne se croisent pas).
Il réveille la boucle quand une lecture revient. **Un texte ne se lit plus qu'en tête** (100 001
octets) : un journal de deux gigaoctets se lisait en entier pour en montrer cent kilo-octets ; le
témoin est `/dev/zero`, qui sans la borne ne rendrait jamais la main.

**Un défaut de DISQUE-1 trouvé en codant, corrigé** : une photo **collée ou lâchée dans** un miroir ne
se scellait pas (tout le tableau d'un miroir restait dehors) — elle n'aurait vécu que dans un cache ou
un fichier qu'il peut effacer. Ne reste dehors que la photo qui montre un fichier du niveau
(`reste_dehors`). Aucune donnée perdue : son Glucose tourne une version sans miroirs.

**Un rendu refait sous son nom** se redécode : le magasin oublie la pyramide, les vignettes et
l'échec de ce fichier (`Magasin::oublier`) ; son aperçu sur le disque est déjà nommé par la taille et la
date. Une autre résolution : la largeur reste, la hauteur suit la forme.

Épreuves : 11 de la loi, 5 du geste, 11 du bureau sur de vrais fichiers (ajout, changement,
renommage, disparition, refus, détachement, mémoire relue de son fichier, rendu refait, `/dev/zero`).
Sabotages : `disque-2.json`, et `disque-1.json` remis aux lignes qui ont bougé. Le cliquet 1 a refusé
six accès au modèle : `Store::board`, `Store::active_board_id`, `Board::image`, `annotation`,
`dossier`.

**Pas encore** : **le fil du focus n'a pas d'épreuve** — le même évènement relit le réglage du
mouvement, un état global que les épreuves se partagent ; à voir au pilote ou à son écran. Le
marquage « absent » (la bordure pointillée) : un nœud gardé reste tel quel, une photo sans fichier
montre son échec. Rien ne dit « en lecture » (17 ms sur son NVMe ; à mesurer sur un disque lent).
DISQUE-3 (`notify`) attend sa réponse. Le partage d'un miroir, pas essayé.
