# 165 — Les photos debout : l'orientation EXIF (l'étude)

> 25/11 vers 8 h 10, compte a. Trouvé en lisant le code de DISQUE-1 (fiche 164 § 8), puis **vu au
> pilote**. Rien n'est codé : le compte arrivait à sa limite, et ce chantier ne se fait pas à moitié.

## 1. Le défaut

**Une photo de téléphone prise en portrait s'affiche couchée dans Glucose Rust.** Un téléphone écrit
le plus souvent les pixels dans le sens de son capteur (en paysage) et une balise EXIF, « orientation »
(0x0112), qui dit comment les tourner : 6 pour un portrait tenu droit. Glucose Rust ne lit cette balise
nulle part (aucun `orientation()` ni `apply_orientation` dans le code, le 25/11).

Vu au pilote : deux JPEG identiques (une bande noire en haut, une rouge en bas, 400 × 300), l'un marqué
« orientation 6 » — Glucose les montre pareils, couchés. Le marqué devait paraître en 300 × 400, la
bande noire à droite.

**Tauri les montrait droites** : un navigateur applique l'orientation EXIF par défaut
(`image-orientation: from-image`, la valeur initiale depuis 2020 ;
[MDN](https://developer.mozilla.org/en-US/docs/Web/CSS/image-orientation)). C'est une perte de parité,
sur ce que Glucose montre le plus : ses photos.

## 2. Ce qu'`image` donne déjà

`image` 0.25 lit la balise : `ImageDecoder::orientation()` (par `exif_metadata`, pour JPEG, PNG, WebP,
TIFF) et `DynamicImage::apply_orientation`. Aucune dépendance de plus.

## 3. La conception proposée : des pixels droits, dès le décodage

Une seule place : **le décodage** (`renderer/atelier/ouvrier.rs::decoder`, puis
`Pyramide::depuis_le_decodeur` et `depuis_rgba`). L'orientation s'y lit avant `read_image`, et le niveau
natif se tourne avant que la pyramide se bâtisse : tout l'aval — les deux voies, l'export, le
storyboard, le recadrage, le détourage, les vignettes — voit une photo droite, sans rien savoir de
l'EXIF. L'autre voie (l'orientation comme une rotation du nœud) mettrait l'EXIF dans chaque rendu et
ferait d'un détail de stockage un geste qu'on peut défaire : écartée.

**Ce que cela demande, et où sont les pièges :**

1. **Les dimensions du dépôt et du miroir** (`interactions/dimensions.rs`) : celles de la photo tournée
   (largeur et hauteur échangées pour 5 à 8). La marche de `dimensions_jpeg` passe déjà par l'APP1
   « Exif » : y lire l'IFD0 (l'ordre des octets `II`/`MM`, la balise 0x0112) — une trentaine de lignes.
   Pour les autres formats, `into_decoder` puis `orientation()` lit l'en-tête seul.
2. **La mémoire** (MEMOIRE-3, fiche 66) : 2, 3 et 4 (miroirs) se font sur place ; 5 à 8 (une
   transposition) demandent une seconde page de la taille native — l'admission (`admission::pic`) doit
   la compter.
3. **Les documents déjà écrits** : un nœud posé avant la correction garde sa boîte en paysage ; ses
   pixels arriveraient en portrait, écrasés dans la boîte. Il faut décider : corriger la boîte au
   décodage quand ses `original_width`/`original_height` sont ceux de la photo transposée (un geste
   silencieux ? une migration à l'ouverture ?), ou laisser ces nœuds tels quels. **À concevoir avant
   de coder** — c'est la seule partie qui touche ses documents.

   **Tranché (25/11 vers 8 h 25) : les redresser, par un geste de Glucose, à l'arrivée des pixels.**
   Après EXIF-1, `original_width`/`original_height` d'un nœud sont toujours ceux de la photo droite ;
   un nœud dont ils sont **la transposée** des pixels décodés (et pas un carré) ne peut donc venir que
   d'avant — le signe est dans ses propres données, sans champ neuf ni lecture d'en-tête à
   l'ouverture. Quand l'atelier rend une photo, chaque nœud qui la montre ainsi se redresse : sa boîte
   tourne d'un quart autour de son centre (largeur et hauteur échangées), ses dimensions d'origine
   aussi, son recadrage suit la même orientation. **Un seul geste signé « Glucose »** : écrit dans le
   document, partagé aux autres mains, défaisable. Écartés : laisser ces nœuds écrasés dans leur boîte
   (ce qu'il verrait au premier lancement) ; les montrer couchés comme avant (une règle de rendu de
   plus, et l'erreur gardée) ; relire l'en-tête de chaque photo à l'ouverture (du disque pour rien).
   Les orientations 2, 3 et 4 ne changent pas la boîte : la photo paraît enfin droite ; seul un
   recadrage posé avant tomberait sur la région miroir — rare, à transformer de même.
   **EXIF-1 ne part dans son Glucose qu'avec ce redressement** (rien à moitié).

   **Revu en commençant à le coder (25/11 vers 9 h) — deux choses manquaient :**
   * **Le recadrage demande l'orientation exacte**, pas seulement « transposée » : ses quatre marges
     se permutent autrement pour 5, 6, 7 et 8. L'orientation doit donc **voyager avec les pixels** :
     un champ de la pyramide (posé par `depuis_le_decodeur` et le chemin seize bits), et un octet de
     plus dans l'en-tête des aperçus (leur version 4, déjà changée pour qu'ils se refassent debout) —
     une photo vue la première fois par son aperçu dans un autre document doit pouvoir se redresser.
   * **Un nœud qu'il a déjà tourné à la main** : s'il a mis un portrait couché à 90° pour le voir
     droit, le redresser sans rien d'autre le recoucherait de l'autre côté. La règle proposée :
     **composer** — la rotation du nœud perd le quart de tour que les pixels ont pris
     (`rotation − angle de l'orientation`), et son `miroir` de même pour 2, 4, 5 et 7 ; ce qu'il voyait
     ne change pas quand il l'avait redressé, et un nœud jamais touché paraît enfin debout… sauf que
     composer ainsi garde aussi couché celui qu'il n'a jamais touché. **Les deux ne tiennent pas
     ensemble par une seule formule** : à trancher, avec un argument. Proposé : un nœud dont la
     rotation est un multiple d'un quart de tour (0 compris) paraît debout, sa rotation ramenée à ce qui
     le montre droit ; un nœud incliné d'un autre angle (un choix d'artiste) garde ce qu'il voyait.
     **Tranché en le codant (§ 6)** : une règle plus simple tient les deux — des deux lectures de sa
     rotation, le nœud garde celle qui le laisse le moins tourné.
4. **Les aperçus en cache** (`renderer::apercu`, nommés par la taille et la date du fichier) : ceux
   faits avant seraient couchés — changer leur nom (une version) pour qu'ils se refassent.
5. **Le HEIC** (fiche 161) : ses transformations `irot`/`imir` passent par `heifer`, à vérifier à part ;
   sa vignette `thmb` aussi.
6. **Le partage** : les pixels ne voyagent pas tournés, les octets d'origine si ; chaque Glucose les
   tourne de la même façon.

**Les épreuves à écrire** : les huit orientations sur une image asymétrique (chaque coin à sa place) ;
les dimensions du dépôt échangées pour 5 à 8 ; une photo sans balise inchangée ; un nœud ancien (§ 3.3).
L'outil d'essai : un JPEG se marque par un segment APP1 « Exif » d'une seule entrée (0x0112, SHORT),
glissé après le SOI — une dizaine de lignes de Python.

## 4. Le chemin

**EXIF-1** : § 3.1 à 3.4, et les épreuves ; au pilote, la photo marquée paraît debout. **EXIF-2** : le
HEIC (§ 3.5). § 3.3 tranché : le redressement fait partie d'EXIF-1.

## 5. EXIF-1, première moitié : le décodage et le dépôt (25/11 vers 8 h 50, compte a)

* **Le décodage tourne** (`renderer/photo/orientation.rs`, `Niveau::oriente`) : `depuis_le_decodeur`
  lit l'orientation du décodeur avant `read_image`, et le niveau natif se tourne avant que la pyramide
  se bâtisse ; le chemin des seize bits passe par `apply_orientation` d'`image`. Une page neuve, quand
  le travail du décodeur est rendu : le pic d'admission ne change pas (ses quatre octets par pixel
  couvrent la page tournée, qui ne vit pas en même temps).
* **Le dépôt et le miroir disent la taille de la photo droite** (`interactions/dimensions.rs`) : la
  marche du JPEG lit l'orientation dans le seul IFD0 de l'EXIF (l'en-tête TIFF, ses entrées ; la
  vignette se saute — moins de 32 Ko lus d'un JPEG d'un Mo à EXIF de 50 Ko) ; les autres formats par
  `orientation()` du décodeur, l'en-tête seul.
* Épreuves : les huit orientations comparées pixel à pixel à `apply_orientation` ; un JPEG marqué « 6 »
  décodé debout (30 × 40, le haut noir passé à droite) ; un portrait mesuré debout en `II` comme en
  `MM`, une orientation hors norme ignorée ; un vrai JPEG marqué que le décodeur d'`image` lit
  « Rotate90 ». Sabotages : `outils/sabotages/exif-1.json` (6 sur 6). Suite verte (3 751).

**Pas encore, et rien de cela ne part dans son Glucose sans elle** : la seconde moitié — le
redressement des nœuds déjà posés (§ 3.3), les aperçus en cache (§ 3.4, leur nom à changer), le HEIC
(EXIF-2), et la photo marquée au pilote.

## 6. EXIF-1, seconde moitié : les nœuds déjà posés se redressent (25/11 vers 11 h, compte a)

* **L'orientation voyage avec les pixels** : un champ de la pyramide (`Pyramide::orientation`, posé
  par les deux chemins du décodage), et un octet de plus dans l'en-tête des aperçus (leur version 4) —
  une photo vue d'abord par son aperçu sait encore comment elle a été tournée.
* **Le magasin note chaque photo arrivée tournée** (`Magasin::tournees` : son orientation, ses
  dimensions droites ; gardées pour la session) ; avant la peinture suivante, l'application demande
  au noyau de redresser (`app/attentes.rs`), et le réveil repasse pour cela — sauf pendant un geste
  ouvert (une frappe de texte peut durer : la boucle dort, et le redressement attend qu'il se ferme).
  **Un autre document qu'on ouvre** se redresse sur les photos déjà tenues (son identité a changé) ; le
  passé de la Time Machine aussi, et rien de ce qui s'y fait ne s'écrit : le présent se redresse au
  retour. Une image écrasée paraît le temps d'une image, celle où la photo arrive. Le prix : un
  parcours des images du document à chaque image où une photo tournée arrive (une recherche par
  image, sans index) — rien pour un document sans photo de téléphone.
* **Au pilote** (l'ancien binaire, `a55d280`, puis le nouveau, sur le même brouillon) : une photo
  marquée « 6 » posée par l'ancien paraît couchée (la bande noire en haut, en paysage) ; rouvert par
  le nouveau, le nœud est en portrait, la bande noire à droite, le bleu en haut ; `Ctrl+Z` rend la boîte
  en paysage, la photo debout écrasée dedans.
* **Le noyau** (`types/image/redresser.rs`, `store/redresser.rs`) : un nœud est couché quand
  l'orientation échange largeur et hauteur (5 à 8), que la photo n'est pas carrée, et que ses
  dimensions d'origine sont exactement la transposée des pixels. Sa boîte et ses dimensions d'origine
  s'échangent autour du même centre, son recadrage suit les pixels (chaque marge suit son bord :
  `Recadrage::oriente`). **Sa rotation** : des deux lectures — un geste sur la photo telle qu'elle
  paraissait (elle reste, la photo paraît enfin debout), ou une correction de la photo couchée
  (elle s'efface, ce qu'il voyait ne change pas) —, le nœud garde **celle qui le laisse le moins
  tourné**. Jamais tourné : debout. Redressé à la main d'un quart : rotation zéro, rien ne bouge à
  l'écran. Redressé puis penché : il garde sa pente. Penché seulement : la photo debout, sa pente
  gardée. Pas de seuil, pas de cas à part pour « un choix d'artiste » : la proposition du § 3.3
  (« un multiple d'un quart de tour paraît debout, un autre angle garde ce qu'il voyait ») traitait mal
  un nœud redressé puis penché. Un quart de tour rangé en `f32` par le document n'est pas `π/2` : il
  se lit au grain du document, et revient exactement à zéro.
* **Un seul geste, signé « Glucose »** (`Main::Agent("Glucose")` : c'est son nom dans l'histoire et
  dans `glucose_quoi_de_neuf`), écrit, partagé aux autres mains, défaisable par `Ctrl+Z` — défait, le
  nœud reste couché jusqu'à la prochaine arrivée de sa photo. Un verrou n'y change rien : il garde le
  nœud de la main, et le laisser serait le montrer écrasé.
* Épreuves : la lecture la moins tournée sur les quatre orientations, huit pentes, deux miroirs (en
  matrices) ; un nœud jamais tourné, un nœud redressé à la main revenu exactement à zéro (rangé en
  `f32`), un redressé puis penché ; rien d'autre ne bouge (déjà debout, carré, 1 à 4, inconnue) ; le
  recadrage qui suit ses bords, **tenu contre `image`** (la même région de pixels, les huit
  orientations) ; le nœud ramené à zéro est bien celui qui montrait la photo debout (tenu contre
  `fliph`/`rotate90` d'`image`) ; un seul geste signé, défait d'un coup ; de bout en bout, une vraie
  photo marquée décodée par l'atelier, le réveil, un geste ouvert, un autre document. Sabotages :
  `outils/sabotages/exif-1-redresser.json` (14 sur 14). Suite verte (3 763), clippy strict, Windows
  vérifié ici. **Elle a attrapé un oubli de la première moitié** (`4f302e9`, après la beta.6) : les
  répétitions de l'installeur plantaient leur aperçu témoin dans `apercus/v3`, que la version 4 retire
  au lancement — ramenées à `v4` (`commun.ps1`, `une_forme.sh`), comme la fiche 63 l'avait payé.

**Limites, dites** : les orientations 2, 3 et 4, et un portrait carré, ne laissent aucun signe dans
les données — la photo y paraît enfin droite, mais un recadrage ou un demi-tour posés avant sur la
photo à l'envers restent tels quels (une photo tenue tête en bas, « 3 », redressée à la main d'un
demi-tour, se retrouve à l'envers). Les retrouver demanderait un champ du format, pour un cas rare. Le
HEIC reste EXIF-2.

## 7. EXIF-2, le HEIC : une garde, pas un correctif (25/11 vers 12 h, compte a)

**Ce que dit le format** : en HEIF, la rotation vit dans les propriétés de l'image (`irot`, en quarts
de tour dans le sens inverse des aiguilles ; `imir`), que le lecteur doit appliquer ; la balise EXIF
d'orientation n'y est qu'**informative** — l'iPhone écrit les deux (`irot` 3 et EXIF 6 pour un portrait),
et les suivre toutes deux tourne la photo deux fois ; mal accordées, elles font les photos d'iPhone
mal tournées de plusieurs visionneuses (les bogues KDE 461205, 473396 et 510990). ExifTool l'a relevé sur des fichiers d'iPhone : la vignette `thmb`, le
second aperçu et la carte de gain partagent l'`irot` de l'image première.
[forum ExifTool](https://exiftool.org/forum/index.php?msg=91969),
[bogue KDE 473396](https://bugs.kde.org/show_bug.cgi?id=473396). `heifer` le dit aussi
(`Exif::orientation` : « informative only in HEIF »).

**Ce que fait Glucose, lu** : `heifer` applique `irot`/`imir` en décodant (`decode_item_with`), à
l'image comme à la vignette, décodée comme image première d'une copie (HEIC-3) ; notre décodeur
(`Heic`) ne rend pas l'EXIF, donc `orientation()` dit « rien à tourner » au décodage
(`Pyramide::depuis_le_decodeur`) comme au dépôt et au miroir (`dimensions_du_fichier`) ; et
`transformer` dit la taille tournée sans décoder. **Rien à corriger.** Le piège est pour demain : qui
ferait rendre l'EXIF au décodeur (pour la date, le GPS) ferait tourner `image` une seconde fois, sans
bruit.

**La garde** : deux épreuves sur un HEIC fabriqué tel qu'un iPhone tenu en portrait l'écrit — l'encodeur
de `heifer`, puis un `irot` 3 ajouté à chaque image (l'image et sa vignette) et l'EXIF remis à 6
(`ajouter_un_irot`, la chirurgie des boîtes `ipco`, `ipma`, `iprp`, `meta`, et des étendues d'`iloc`) :
le dégradé paraît debout, tourné **une seule fois** et dans le bon sens (un coin chacun), à la taille
droite, sa vignette aussi. Sabotages (`exif-2.json`, 2 sur 2) : l'EXIF rendu par le décodeur (la photo
tournée deux fois), le quart de tour oublié dans la taille.

**Au pilote** (le binaire de `23c5656`, après EXIF-1) : `arrow.heic`, une vraie photo d'iPhone qui porte
`irot` 3 **et** l'EXIF 6, collée dans un Glucose hors écran — debout, en portrait, dès sa vignette.

**Reste** : `imir` n'a pas d'épreuve à nous (un miroir dans un HEIC est rare ; `heifer` l'applique).
