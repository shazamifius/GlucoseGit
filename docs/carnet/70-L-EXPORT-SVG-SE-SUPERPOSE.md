# 70 — L'export SVG se superpose au canevas : les glyphes en tracés, la surface, l'oracle

> Session du 15/10/2026, à la suite de la fiche 69. Arrêtée à sa demande (« la session devient
> trop longue ») au milieu de P5. Le téléphone n'était pas branché (`adb devices` vide) : P0
> attend toujours sa lecture par le câble.

## 1. Ce que l'image a montré

La fiche 69 n'avait jamais regardé son SVG. Rendu sans fenêtre — Brave sans tête ne rend pas
la main (son processus graphique tombe ; `--virtual-time-budget` n'y change rien) ; **`resvg`**
(construit dans le scratchpad : `cargo install resvg --root <scratchpad>/outils`) le rend —, puis
comparé pixel à pixel au PNG de l'export (`comparer.py`, `voisinage.py`, `loupe.py` du
scratchpad, à réécrire au besoin) :

* **juste** : les membranes et leur pointillé, les lueurs, les flèches, la photo recadrée, la
  photo tournée, les post-it ;
* **faux** : les polices (Inter absente chez le lecteur : remplacée, et chaque passage en gras
  laissait un trou) ; les formules écrites en source LaTeX ; le dossier absent ; le titre des
  membranes dans une autre police.

## 2. Ce qui est fait

* **Les glyphes en tracés** (`renderer/svg/glyphes.rs`) : chaque glyphe défini une fois
  (`<path>` dans `<defs>`, dans les unités de sa police) et posé par un `<use>` à la place même
  du canevas — la plume, la ligne de base à `y + corps`, l'avance de `Typography`. Aucun
  `<text>`, aucune police demandée au lecteur ; chaque ligne garde ses mots dans un
  `aria-label`. C'est le mode par défaut de matplotlib et l'« Outline text » de Figma ; tldraw et
  Excalidraw embarquent une `@font-face`, que `resvg`, Inkscape et Illustrator ignorent.
  `Typography::source` dit la police nommée (`Police`) et l'indice du glyphe — le même choix
  qu'EMOJI-1. Les contours viennent de **`ttf-parser`**, déjà compilé par `fontdue` (même
  version, sans fonctions par défaut : rien de neuf ne se compile). **`decisions/13` reste à
  écrire** (ttf-parser, et resvg ci-dessous).
* **Les formules en vecteurs** (`renderer/math/svg.rs`) : les glyphes des fontes de KaTeX, les
  filets par la même boîte que le tracé (`boite_du_filet`), les formes étirables par le chemin de
  KaTeX rogné à sa boîte. La pose d'une formule dans une carte est partagée
  (`card/ornament.rs`, `poser_la_formule`).
* **La surface** (`renderer/surface.rs`) — remise en question des sœurs de la fiche 69 : une
  sœur recopie les appels de dessin, et une couleur peut diverger sans que la mise en page
  partagée le voie. Cairo dessine le même code sur une image, un PDF, un SVG. Un dessin qui
  n'emploie que remplir un tracé, tracer un contour, remplir un rectangle net et écrire est
  désormais **écrit une fois**, générique sur `Surface` (`Pixels` ou `Vecteurs`), et le SVG reçoit
  le même `tiny_skia::Path` que le canevas. Y sont passés : **le dossier**, **la réglette des
  domaines**, **les textes des zones et des cases**. Gardent leur sœur : le texte riche, les
  post-it, les flèches et leurs libellés, les badges, les photos, les membranes.
* **L'ordre exact du canevas** (`renderer/export.rs`) : sous les photos (fond, premier rang,
  membranes puis leurs ornements — titre détouré par les huit copies de `DETOURAGE`, réglette —,
  textes des zones et des cases, dossiers), les photos, puis **toutes les lueurs avant toutes
  les cartes** (LUEUR-3 : la fiche 69 posait chaque lueur sous sa carte, celle d'une carte B
  aurait couvert une carte A), les annotations (réglettes des cartes et post-it), les réglettes
  des photos. Les lueurs prennent `halo_geometry` (`halo/svg.rs`) au lieu de la recalculer.
* **`rayon_tenu`** : le canevas borne un coin par la plus petite demi-dimension ; un `rx` seul
  donnait des coins d'ellipse à un rectangle bas.

## 3. Des défauts du canevas trouvés en écrivant le SVG

* **Le dossier, relu contre `FolderSvgLayer.tsx`** : le titre (ligne de base à 23) et le
  chiffre du badge (11,5) avaient été pris pour des **hauts** de texte — le titre touchait le
  trait du bandeau, le « 0 » passait sous sa pastille. Corrigé ; et comme Tauri : le badge ne
  se montre que si le dossier contient quelque chose, un dossier vide dit « Double-clic pour
  ouvrir », l'icône est le tracé de Tauri, le titre a ses opacités (55 %, 85 %), le chiffre du
  badge est centré par sa largeur mesurée (`BADGE_CHASSE` a disparu). **À juger à son écran**
  (et « Double-clic » au téléphone ?).
* **BORNES-2** (`store/navigation.rs`) : la borne du contenu ne comptait ni les zones ni les
  cases du storyboard — l'export coupait un storyboard, et `Alt+F` ou la minimap ne pouvaient pas
  cadrer un tableau qui n'en porte qu'un. Une case compte avec sa bande de description
  (`StoryboardPanel::emprise`). Épreuve : `test_bornes_2_les_zones_et_les_cases_sont_du_contenu`.
* **Les glyphes des formules sont calés au pixel entier** au canevas (`dessine_glyphe`,
  `.round()`), quand le texte l'est au quart de pixel (GLYPH-1) : le SVG les pose exactement.
  **Non corrigé** (un chantier du canevas).

## 4. L'épreuve de superposition — et sa mesure

`resvg` 0.46, **en dépendance d'épreuve seulement** (l'oracle, comme `automerge`,
`decisions/03`) : notre `tiny-skia` 0.11.4, les mêmes `gif` et `zune-jpeg` qu'`image` — aucune
version double (la 0.45 en tirait deux, la 0.48 une seconde `tiny-skia`).
`test_p5_le_canevas_et_le_svg_se_superposent` rend la vitrine de l'export
(`bench/vitrine.rs` : la vitrine du noyau, deux photos, un domaine, un gabarit, un storyboard, un
dossier plein, une pilule « ET », une flèche à libellé et relation) par le canevas et par
`resvg`, et mesure deux grandeurs, choisies **en simulant des fautes** sur l'image :

| | forme (blocs 8 × 8, à un pixel près) | teinte (moyenne signée, blocs 16 × 16) |
|---|---|---|
| la paire juste | 26,6 | 6,3 |
| un mot absent | 87,9 | 53,0 |
| une ligne décalée de 2 px | 56,6 | 11,3 |
| une teinte faussée de 12 niveaux | 27,4 | 13,3 |
| une lueur absente | 26,6 | 24,9 |
| une puce absente (8 × 10 px) | 31,4 | 9,8 |

Bornes : le milieu entre le bruit et la plus fine faute (41,6 ; 9,8). **Une puce absente ne la
fait pas tomber** : les petits éléments ont besoin d'épreuves ciblées. **L'épreuve prend 80 s**
— pas la comparaison (accélérée sans effet) ; probablement le flou des lueurs dans `resvg`, dont
la région de filtre est immense (`x="-100000"…`) : à mesurer, puis borner la région à la portée
du flou (`portee_du_flou`), ce qui allégerait aussi le fichier pour tout lecteur.

## 5. Ce qui reste — dans cet ordre

1. **Mesurer et régler les 80 s** (§ 4). Puis **toute la suite** (`cargo test --workspace`) :
   elle n'a pas été rejouée après ces changements — seules les épreuves `p5_`, `folder::` et
   `bornes` l'ont été. Clippy strict : voir le commit.
2. **Saboter** (`outils/sabotages/p5.json`, à écrire) : chaque appel de `rendre_en_svg` et de
   `sous_les_photos_en_svg` retiré, l'échelle d'un glyphe, son avance, l'opacité d'un filet, le
   détourage, `rayon_tenu`, `remplir_net`, BORNES-2, le badge et l'indice du dossier. Ce qui ne
   tombe pas reçoit son épreuve ciblée (comme la puce).
3. **`decisions/13`** (ttf-parser, resvg), ARCHITECTURE § 3 « Exporter » et § 4 (la surface),
   INVENTAIRE (« le SVG ne porte ni les images ni les membranes » est faux désormais), SUITE P5,
   ÉTAT.
4. Ce que le SVG n'a pas encore : la lecture (voile), rien d'autre de connu — **regarder
   encore**, sur un vrai document de lui (une copie).
5. **Ses yeux** : ouvrir un SVG exporté dans son navigateur et dans un éditeur (Inkscape ?).

## 6. Les erreurs de la session

* **Une mesure trompeuse** : un « décalage systématique d'un demi-pixel » du texte, lu sur des
  centres d'encre de fenêtres qui coupaient des glyphes — mot par mot, les signes changeaient.
  Aucun décalage. Mesurer sur des régions qui ne coupent rien avant de conclure.
* **Un `cat` sans entrée** devant un script Python : la commande a attendu deux minutes. Tout
  script passe par l'outil d'écriture, puis `python -I`.
* **Une chaîne brute fermée par son contenu** (`r#"…="#g-"#`) : `r##"…"##`.
* **Le nom d'un utilisateur dans un en-tête** envoyé à crates.io : jamais rien qui le désigne.
