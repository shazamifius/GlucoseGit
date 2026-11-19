# 18 — Les émojis en couleur, par Twemoji 17 construit en COLRv0

> Sa parole (fiche 122 § 6) : *« je suis d'accord avec tes propositions »* — devant 🟢 et 🔴
> dessinés du même rond rayé, quand Claude.ai s'en sert pour dire « fait » et « reste à faire ».
> Cette note dit la police qui remplace Noto Emoji monochrome (`decisions/10`), **aucune caisse
> nouvelle**, et l'attribution que sa licence demande.
>
> **Date** : 2026-11-19 · **Portée** : `crates/glucose-desktop/assets/GlucoseEmoji-Couleur.ttf`,
> `NotoEmoji-Reste.ttf`, `typography/choix.rs`, `typography/trace/couleur.rs`,
> `outils/emojis/` · **Fiches** : [`124`](../124-DEUX-SESSIONS-VALIDEES-UN-TEXTE-QUI-GRANDIT-POUSSE-SA-MEMBRANE-LES-EMOJIS-EN-COULEUR.md) § 6 (la recherche), `125`.

---

## Ce qu'il faut

Un émoji de la couleur que son auteur a voulue, sur les trois dessins de Glucose (pixels, carte
graphique, SVG), net à tout zoom — un canevas infini zoome —, sans changer la place d'un seul
caractère des textes déjà posés.

## Ce qu'on prend

* **Les dessins de Twemoji 17** (`jdecked/twemoji`, v17.0.3, l'archive du tag épinglée par son
  empreinte SHA-256) : 4 009 dessins SVG, **aucun dégradé** ; sous **CC-BY 4.0**
  (`assets/LICENSE-Twemoji-graphics.txt`).
* **Construits par nous en police COLRv0** (`outils/emojis/construire.sh`), par **nanoemoji** 0.16
  (Google, Apache 2.0) et fontTools, ceux de nixpkgs — des outils de construction, jamais dans
  l'application. COLRv0 : un émoji est une **pile de glyphes ordinaires**, chacun d'une couleur de
  la palette (`COLR`, `CPAL`).
* **Lus par `ttf-parser`**, déjà là (`decisions/13`) : `Face::paint_color_glyph` annonce, couche
  par couche, le glyphe puis sa couleur. **Aucune caisse nouvelle.**
* **Ce que Twemoji n'a pas reste à Noto Emoji** : un sous-ensemble de 56 caractères (le cadre de
  touche U+20E3, les joints et les sélecteurs invisibles, les étiquettes des drapeaux régionaux),
  4 Ko, sous OFL 1.1 comme avant — deux polices plutôt qu'une, pour que les deux licences ne se
  mêlent pas dans un même fichier.

## Ce que l'outil garantit, et vérifie

* **Les métriques de Noto** : l'em (2 048), l'avance (2 600), la montante et la descendante ; le
  dessin posé sur la boîte du 😀 de Noto, à surface égale (une homothétie déduite des deux boîtes,
  aucun nombre choisi). Aucun texte déjà posé ne se remet en page.
* **Un dessin tient dans le carré de l'em** : la vue de 🍉 (36 × 25,22) débordait l'avance
  (3 426) — trouvé par la vérification de l'outil, corrigé par la loi, pas par une exception.
* **Rien de perdu** : chaque caractère que Noto dessinait (hors usage privé) est dans l'une des
  deux polices, à la même avance ; 🟢 et 🔴 sont de deux couleurs. L'outil s'arrête sinon.
* **Un caractère, un glyphe** : sans moteur de mise en forme, une séquence (famille, drapeau, ton
  de peau) ne serait jamais appelée ; elle n'est pas construite (`decisions/10`).
* **Reproductible au bit près** : l'heure inscrite dans la police est celle de la source
  (`SOURCE_DATE_EPOCH`), pas celle de la construction.

## Ce que ça coûte

| | Noto Emoji (avant) | Twemoji en couleur + le reste |
|---|---|---|
| octets embarqués | 886 628 | 851 172 + 4 004 |
| caractères | 1 489 | 1 427 + 56 |
| glyphes lus par `fontdue` au chargement | 1 489 | 56 (les couches passent par leur contour) |

* `fontdue` ne charge que les glyphes que la `cmap` atteint : il verrait les couches **vides**. Les
  couches se rastérisent donc par leur contour (le chemin des grandes lettres, GRAND-1) et entrent
  dans le même cache que les lettres, sous (police, indice) — deux visages qui retombent sur le
  même émoji partagent son image.
* Un émoji compte 6,4 couches en moyenne (9 181 pour 1 427), 44 au plus : autant de compositions
  par image qu'il en a — son image est en cache, chaque couche ne coûte qu'une composition.

## Pourquoi pas autre chose

| | ce qu'elle est | pourquoi non |
|---|---|---|
| **Twemoji de Mozilla** (COLRv0, 2022) | la même famille, déjà construite | figée à Unicode 14 ; ses métriques ne sont pas celles de Noto |
| **Fluent Emoji plat** (Microsoft, MIT) | la seule licence sans attribution | 27 Mo |
| **Noto Color Emoji** (OFL) | COLRv1 | des dégradés à dessiner sur les deux voies |
| **OpenMoji** | CC BY-SA | une licence qui contamine ce qu'on en dérive |
| **La police du système** | rien à embarquer | différente sur chaque machine (FONT-1), absente d'Android en COLR |

## L'attribution (CC-BY 4.0)

*« Twemoji graphics Copyright 2019-2026 Twitter, Inc and other contributors, licensed under CC-BY
4.0 — converted to a COLRv0 font for Glucose, scaled to the metrics of Noto Emoji. »* Elle est à
trois endroits : `assets/LICENSE-Twemoji-graphics.txt` (le texte de la licence, depuis l'archive),
la section Licence du `README.md`, et **les champs de la police même** (`name` 0, 13 et 14), qui
voyagent avec le programme. Glucose n'a pas d'écran « À propos » : le jour où il en aura un, les
licences des polices y vont.

## Sources (consultées le 19/11/2026)

* [`jdecked/twemoji`](https://github.com/jdecked/twemoji) v17.0.3 — `LICENSE-GRAPHICS` (CC-BY 4.0).
* [nanoemoji](https://github.com/googlefonts/nanoemoji) 0.16 — lu dans ses sources :
  `color_glyph.map_viewbox_to_font_space` (la vue posée sur montante − descendante, centrée dans
  l'avance), `write_font._draw_glyph_extents` (la base d'un glyphe COLRv0, un trait sans surface).
* `ttf-parser` 0.25.1, `tables/colr.rs` — `paint_v0` : `outline_glyph` puis `paint(Solid)` par
  couche, l'indice 0xFFFF à l'encre du texte.
* `fontdue` 0.9.4, `Font::from_bytes` — seuls les glyphes de la `cmap` (et de `GSUB` sur demande)
  sont préparés.
* [Reproducible builds, `SOURCE_DATE_EPOCH`](https://reproducible-builds.org/docs/source-date-epoch/).
