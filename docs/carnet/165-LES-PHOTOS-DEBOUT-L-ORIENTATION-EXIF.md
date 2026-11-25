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
HEIC (§ 3.5). Avant de coder EXIF-1, trancher § 3.3.
