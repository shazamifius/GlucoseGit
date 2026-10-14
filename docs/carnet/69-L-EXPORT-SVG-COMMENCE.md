# 69 — L'export SVG complet (P5) : commencé, pas fini

> Session du 15/10/2026, à la suite de la fiche 68. Arrêtée à sa demande (« la session devient
> trop longue ») au milieu de P5. **Ce qui est écrit compile, passe clippy et la suite ; rien
> n'est regardé dans un lecteur, rien n'est saboté.** C'est le premier chantier de la session
> suivante.

## 1. Pourquoi refaire l'export SVG

Le SVG du noyau (`export.rs`, « port de `toSvg.ts` ») n'écrivait qu'une ligne de texte par
carte, des flèches droites, ni post-it, ni membranes, ni images. Celui de Glucose Tauri
(`src/utils/export/toSvg.ts`, `scene.ts`) en faisait plus — mais en **recomposant sa scène de
son côté** : un second jeu de marges, de rayons, de couleurs, qui divergeait du canevas.

## 2. La conception retenue : une sœur vectorielle pour chaque dessin

Chaque module du rendu qui dessine un nœud porte, **à côté de son dessin**, une fonction qui
écrit la même chose en SVG — la même mise en page, les mêmes constantes, la même passe. Le SVG
d'un tableau est donc ce que le canevas montre, **dans le cadre de l'export PNG** (EXPORT-PNG-1 :
la sélection, sinon le tableau ; la même marge ; 100 %) : les deux exports se superposent.

| sœur | où | ce qu'elle reprend |
|---|---|---|
| le peintre (`Svg`, base64, échappement) | `renderer/svg.rs` | — |
| membranes (formes, halos, pointillé, titres) | `renderer/scene.rs`, `membranes_en_svg` | `pour_chaque_membrane`, `MembraneVue::forme` (MEMB-FORME-1) — le bord part du même point que celui d'un `<rect>`, le pointillé tombe pareil |
| images (octets d'origine, recadrage, rotation) | `renderer/scene/image/svg.rs` | `boite_ecran`, `boite_source` (RECADRAGE-1) |
| cartes (lueur, brume, lignes) | `renderer/card/svg.rs` | `poser`, `BRUME`, les constantes de `halo.rs` (la lueur est **exactement** l'ombre CSS de Tauri : `feGaussianBlur` d'écart-type 30) |
| ornements (puce, numéro, citation, filet, formule) | `renderer/card/ornament.rs`, `ornement_en_svg` | la même géométrie ; **une formule s'écrit par sa source** (le moteur de formules ne livre que des pixels) |
| une ligne de texte riche | `renderer/richtext/draw.rs`, `ligne_en_svg` | taquets, fond d'un `code`, lien, barré — lus dans la police |
| post-it et pilules d'opérateur | `renderer/note/svg.rs` | `StickyLayout`, `sticky_lines` |
| flèches | `renderer/arrow.rs`, `fleche_en_svg` | `Fleche` (le tracé exact en `L`/`C`, les trois teintes, `aspect`) |
| libellé, badge de relation | `arrow_label.rs`, `predicate.rs` | la pastille ; le sigle, déjà en traits |
| annotations, dans l'ordre | `renderer/pass/svg.rs` | `carte_de`, `Fleche::de`, l'ordre de `draw_annotations` |
| le chef d'orchestre | `renderer/export.rs`, `Renderer::rendre_en_svg` | la passe comme `rendre_la_region` : fond, `premier_rang` (zones, cases), membranes, images, annotations |
| l'export | `persist/export/svg.rs`, `GlucoseApp::rendre_le_svg` ; `try_export` | `png::preparer`, `disque.objets.lire_en_attendant` |

Retirés du noyau : `to_svg`, `scene_to_svg`, `SvgOptions`, `html_escape` et leurs quatre épreuves
(plus d'appelant). Le Markdown reste au noyau.

## 3. Ce qui reste — dans cet ordre

1. **Regarder l'image.** `cargo run --release -p glucose-desktop --example apercu_svg -- <dossier>`
   écrit `vitrine.svg` et `vitrine.png` (la vitrine, plus une photo recadrée et une tournée).
   Rendre le SVG sans fenêtre : Brave est installé (`C:\Program Files\BraveSoftware\Brave-Browser\
   Application\brave.exe --headless=new --screenshot=… --window-size=1914,1134 file:///…svg`) —
   **la commande n'a pas rendu la main en deux minutes** : lui donner `--no-first-run
   --disable-extensions --virtual-time-budget=5000`, ou essayer `resvg` (un exécutable) dans le
   scratchpad. Comparer pixel à pixel au PNG (`venv-pdf` du scratchpad a Pillow : à recréer).
2. Corriger ce que l'image montre ; vérifier en particulier le **pointillé** des membranes, la
   **lueur** des cartes, les **ordinaux**, le **recadrage tourné**.
3. **Ce qui manque encore au SVG** : les **dossiers** (`renderer/folder.rs`), les **textes des
   zones et des cases** (numéro, sigle, description — seuls leurs rectangles y sont), les
   **réglettes de domaines**, les **formules en vecteurs**, les tableaux (les taquets y sont, à
   regarder). À écrire, ou à dire dans INVENTAIRE.
4. **Les épreuves** (une seule existe : `persist/export/svg/tests.rs`) : une par sœur, sur un
   vrai document — membranes (le `stroke-dasharray`, les opacités des trois couches), cartes
   (une ligne par ligne visuelle, à la bonne ordonnée), flèches (`C` pour une courbe), images
   (recadrage, rotation) — puis **saboter** chacune (`outils/sabotages/p5.json`).
5. Mettre à jour INVENTAIRE (« Exporter » : le SVG porte maintenant…), ARCHITECTURE § 3
   (« Exporter »), SUITE P5.
