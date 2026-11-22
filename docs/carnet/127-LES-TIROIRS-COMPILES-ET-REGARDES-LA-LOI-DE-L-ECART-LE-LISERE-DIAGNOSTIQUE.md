# 127 — Les tiroirs compilés et regardés ; la loi de l'écart ; le liseré diagnostiqué

> Session du 20/11/2026, depuis 21 h 49, après la fiche 126 (arrêtée le 19/11 à 19 h 40, à sa
> demande). La passation : compiler les tiroirs (TIROIR-5 et 6, écrits sans compilateur), les
> sabotages en attente, son Glucose à la beta.4, le compte Cloudflare, le liseré du détourage.

## 1. En arrivant

* **La machine** : 3,1 Go disponibles (Brave près de 4 Go, VS Code 2 Go, son Glucose ouvert) — la
  borne refuse une compilation des épreuves du bureau (3,58 Go demandés).
* **GitHub** : `main` à `539bc88` des deux côtés ; la 2.0.4-beta.4 « Latest » (lue par
  `gh release list`, pas dans nos documents).
* **La télémétrie** (22 h) : 219 sessions, 10 installations, 40 fins anormales — **aucune neuve
  depuis le 17/11**. **La beta.4 est arrivée seule** sur son PC de l'école et chez l'utilisateur
  Windows à carte AMD (14 sessions, la dernière le 20/11, aucune fin anormale ; sa « pire image »
  reste celle de 415 s, la veille d'avant VEILLE-1) ; son téléphone est passé en beta.2.
* **Son Glucose** (par sa porte, les deux onglets nommés) : voir § 2.

## 2. Remis en question

* **« Ses dernières réponses sont lues ; une seule chose attend : le compte Cloudflare »** (la
  passation) : faux. **`text-24`**, à lui, sans réponse ni flèche : *« OULA sur le github javais pas
  vue mais… en fait il y avais lancien fichier tauri toujours publique sur les github page […] je
  propose de enlever »*. Aucun document du dépôt n'en parlait. Et une membrane « en attente » porte
  toute l'ancienne page de questions (1 à 19), recopiée par lui.
* **« La server card MCP n'est pas mûre »** (fiche 126 § 6, SUITE) : faux. Le texte officiel de la
  SEP-2127 porte **« Status: Final »** et réserve `<adresse MCP>/server-card` comme emplacement
  recommandé ; le catalogue d'un domaine, `/.well-known/ai-catalog.json`, est facultatif. Les
  « cinq chemins » étaient ceux des brouillons ; le dépôt du format s'appelle encore
  `experimental-ext-server-card` (il change de nom à l'acceptation). Mistral demandait un chemin de
  brouillon (`/.well-known/mcp/server-card/…`).
* **« Le liseré : le premier plan garde du blanc du fond, DETOURER-2 ne le retire pas jusqu'au
  bout »** (fiche 126 § 4 bis, SUITE) : faux (§ 5). Le bord clair est **dans la photo**, sous une
  opacité de 0,98-0,99 : la pomme reflète le fond blanc à son contour (le *spill* des
  incrustateurs). Là où l'opacité vaut presque 1, la couleur du premier plan n'a rien à corriger.
* **Ma propre première mesure du liseré, fausse** : « le masque déborde le bord réel de 2 à 10
  pixels » — je prenais pour bord l'endroit où la couleur est à mi-chemin du rouge et du blanc ;
  or le bord de la pomme s'éclaircit lui-même avant son contour. Dit à 22 h, corrigé à 22 h 03
  devant l'image.
* **SUITE § 7** disait encore « la 2.0.4-beta.2 est la dernière », et § 3 « la beta.4, quand elle
  sera publiée » : corrigés.

## 3. TIROIR-5 et 6, compilés

**La relecture d'abord** (pendant que la mémoire manquait) : chaque nom que le diff appelle existe,
avec la visibilité et les dérivations qu'il faut (`Brush::origin` public, `TabId` `Hash + Eq + Copy`,
`ScaledRect` `PartialEq + Copy`, `composition::poser` en `(f32, f32)`, `deltas` à trois champs) ;
Domaines range ses rangées dans le cadre qu'on lui donne (`layout_domains_panel`), l'exemption
tient. Les cliquets, lus sans compiler : 0 faute, le couplage à 38 sur 38.

**Deux gardes qui manquaient, trouvées à la lecture** :

* **Le sabotage du survol (SURVOL-2) était aveugle deux fois** : son filtre (`tiroir_ survol
  dock_cache`) ne jouait même pas l'épreuve du survol à travers le cache
  (`test_the_cached_dock_is_identical_while_hovering`), et aucune épreuve ne survolait un tiroir
  **défilé**. Écrite : `test_tiroir_6_un_tiroir_defile_se_survole_a_travers_le_cache` — Ordonner
  défilé, le cache rempli le pointeur dehors, puis sur un bouton de la part visible ; l'épreuve
  cherche elle-même un point où le survol change l'image.
* **Le cache suit le défilement par construction** (la clé porte le cadre entier,
  `geometrie: PanelLayoutBox`, défilement compris), mais rien ne le gardait : toutes les épreuves
  de TIROIR-6 dessinaient sans cache, quand le vrai Glucose dessine par lui. Écrite :
  `test_tiroir_6_le_cache_suit_le_defilement`.

**La recherche** — Apple, *Scroll views* (HIG) : *« Avoid putting a scroll view inside another
scroll view with the same orientation »* (Domaines, qui défile par rangées, n'est pas mis dans un
second défilement) ; *« displaying partial content at the edge of a view indicates that there's
more content in that direction »* (le contenu coupé net au-dessus de la poignée) ; l'indicateur
dit *« near the beginning, middle, or end »* (le filet d'accent, par sa place et sa longueur).

**La première compilation** (22 h 13 → 22 h 20, après une heure à attendre que la borne la laisse
partir) : le code écrit sans compilateur **compile du premier coup** ; 36 épreuves passent, **3
tombent**, chacune pour une raison à elle :

* **La molette** — *« aux sept ouverts, un tiroir défile »* : à 1440 × 900 sur un document vide,
  aucun tiroir ne déborde ; l'épreuve espérait son cas au lieu de le construire. Elle le construit :
  l'interface agrandie deux fois (Ordonner ne tient plus sous la bande), puis le minuteur seul à
  l'échelle 1 pour « montré entier, il laisse passer la molette » — la branche était sous un
  `if let`, aveugle quand aucun tiroir n'était entier.
* **Le contenu défilé s'écarte de 9 niveaux** du contenu entier (1 permis). **Ma première cause
  était fausse**, et je l'ai corrigée sans l'avoir vérifiée : « le tampon transparent, deux
  arrondis » — la seconde compilation a redonné exactement 9. **La vraie, trouvée par le calcul** :
  le fond d'un tiroir vaut `#111111`, son filet `#222222` (17 niveaux) ; un filet d'un pixel à
  cheval sur le bord (y = 78) couvre la moitié de la rangée 78, soit 8,5 au-dessus du fond. La
  région comparée commençait sur le filet du haut, qui reste au bord quand le contenu défile.
  **Et la cause disait un vrai défaut** : le tampon du contenu commençait sur cette rangée — le
  contenu défilé **recouvrait le filet**. Le tampon s'arrête désormais au bord intérieur du filet ;
  il part d'une copie du fond du tiroir et se recopie à sa place (`Melange::Remplacer` — un seul
  arrondi, celui du dessin direct), et l'épreuve exige **0 niveau**, hors du filet ; une épreuve
  neuve vérifie, défilement après défilement, que la rangée du filet reste celle d'un tiroir
  immobile.
* **TIROIR-1 à l'échelle 1,5** — *« le contenu couvre la poignée »* : l'épreuve mesurait
  `contenu()`, qui est désormais par construction le cadre **virtuel** de tout le contenu, et passe
  sous la poignée quand le tiroir défile. Ce qui ne doit jamais la couvrir, c'est ce qui se voit
  (`contenu_visible()`) ; la poignée intacte se garde au pixel dans l'épreuve de TIROIR-6. Une loi
  neuve qui contredit une épreuve ancienne : tranché, l'attendu réécrit (MÉTHODE § 4.7). **Et
  TIROIR-1 n'avait plus de sabotage vivant** : celui de `capture-118` visait du code disparu depuis
  la fiche 122 (« INTROUVABLE ») — un sabotage neuf sur `contenu_visible`.

**Ensuite** (22 h 36 → 22 h 46) : les 40 épreuves des tiroirs, l'écart **mesuré à 0** ; la
bibliothèque du bureau **2 175 sur 2 175** ; les huit suites d'intégration (les 16 cliquets, les
33 des deux voies) ; clippy strict et `fmt` à zéro. Enregistré en local : `8525291`.

### Regardé dans le vrai Glucose

Au pilote (sway sans écran, 1440 × 900), le programme de `8525291` sur une copie de son document
(`~/.cache/glucose-bancs/pilote-127/`, captures numérotées) :

* **les sept tiroirs à 100 %** : chacun du côté de son bouton, collés au bord, aucun l'un sur
  l'autre — Ordonner et Pomodoro, puis la Time Machine et le Storyboard à gauche ; Plugins et
  Domaines contre le bord droit, le Mood board à l'intérieur. Tout tient : rien ne défile ;
* **à 125 %** (« Agrandir l'interface » deux fois) : une seule colonne à droite (TIROIR-5), et
  **le Mood board déborde** — son contenu s'arrête net au-dessus de sa poignée ; **trois crans de
  molette** le remontent (« Creature Design », « Props & Items » entiers), le haut passe sous le
  filet du cadre, et **le survol éclaire la rangée sous le pointeur, à travers le cache** ;
* **le filet d'accent** dit la part montrée (gris 59 sur une centaine de pixels, contre 30) — **très
  discret** : le signe principal reste le contenu coupé net (Apple). Sa force est une question de
  ressenti, à lui ;
* **à 110 %, la Time Machine ne montrait qu'un jalon sur deux**, un vide à la place du second — un
  défaut **de la beta.4 publiée**, pas d'aujourd'hui (§ 3 bis).

## 3 bis. TIROIR-2 — à sa hauteur naturelle, un tiroir montre tout, à toute échelle

* **La cause** : la Time Machine comptait ses jalons par `⌊libre / pas⌋`, à la hauteur exacte de
  deux jalons — en `f32`, à 110 %, 1,9999. Et sa hauteur naturelle se mesurait dans un cadre d'**un
  million** de pixels, puis se soustrayait : un `f32` y perd un dixième de pixel. Depuis ECHELLE-1,
  qui permet les échelles qui ne tombent pas juste.
* **Le Storyboard, pire et partout** : l'épreuve neuve l'a trouvé — il comptait 8 sous ses cases
  pour les 6 de sa pose : `n − 2/24`, **une case de moins, sur deux pages**, à toute échelle, dès
  deux cases. Depuis TIROIR-2.
* **La loi de l'écart, sans aucune tolérance** : `k` lignes n'emploient que `k × pas − écart` — la
  dernière n'a pas besoin de l'écart qui la suivrait. Les jalons, les rangées de Domaines et les
  cases du Storyboard se comptent ainsi ; la hauteur naturelle garde l'écart, et à elle le compte
  tombe à `n + écart/pas`, loin de l'arrondi. Le compte d'avant comptait un écart de trop.
* **La mesure exacte** : la hauteur naturelle se mesure dans un cadre de hauteur nulle — la mise en
  page est affine en la hauteur du cadre —, comme Plugins le faisait déjà. Le million ne sert plus
  qu'au Storyboard, qui lit son bas sans soustraire.
* **Les épreuves** : toute échelle de 50 à 300 %, de 1 à 8 lignes, par le vrai rangement (la Time
  Machine et Domaines tenaient déjà avec la loi ; le Storyboard tombait 76 fois) ; la hauteur
  naturelle égale celle d'un cadre de mille pixels au millième. **Regardé** : à 110 %, les deux
  jalons. Enregistré : `596e783`.

## 4. Les sabotages — tous les fichiers en attente, joués

Dans l'arbre détaché (à `596e783`, puis `f9a8557`), deux fils, le code intact joué d'abord ; le
détourage dans une cible au profil du dépôt (`opt-level = 1` — sans lui, son modèle dépasse le délai
de ses épreuves), copiée de celle du dépôt pour ses dépendances (`deps`, `build`, `.fingerprint` :
9,5 Go ; seuls nos crates s'y recompilent) :

| fichier | tombent | ce qu'il garde |
|---|---|---|
| `tiroirs-126` | **13 sur 13** | le rangement (TIROIR-5 ×4), le défilement (TIROIR-6 ×8), ce qui se voit sur la poignée (TIROIR-1) |
| `tiroirs-127` | **5 sur 5** | la loi de l'écart (Time Machine, Domaines, Storyboard) et la mesure exacte (×2) |
| `veille-126` | **2 sur 2** | VEILLE-1 |
| `liste-123` | **8 sur 8** | les listes imbriquées — dont la puce en vecteurs, jamais jouée jusqu'ici |
| `detourage-121`, le noyau | **20 sur 20**, après une épreuve écrite | le moteur de u2netp (DETOURER-1 ×15), le premier plan (DETOURER-2 ×2), le fond uni (DETOURER-3 ×3) |
| `detourage-121`, le bureau | (§ 9) | le geste dans Glucose (×5) |

* **« DETOURER-2 — le premier plan sans sa correction » passait** : son épreuve (un bord de
  synthèse, un sujet **uni**) affirmait *« ce qu'aucune formule privée de sa correction ne
  donne »* — faux : sur un sujet uni, la moyenne floue sur 90 pixels, dominée par l'intérieur,
  retombe déjà sur sa couleur. Ce que la correction de Forte garantit, c'est qu'**à l'opacité 1,
  le premier plan est l'image même** (`fm + 1 · (I − fm)`) ; écrite sur un sujet rayé, avec la
  preuve à l'envers (le flou seul s'en écarte de plus de 0,2) : le sabotage **tombe**, sur elle
  seule. `f9a8557`.

* **Le survol à travers le cache** (SURVOL-2), que la passation craignait aveugle, tombe sur
  l'épreuve écrite pour lui ; **« la molette prise par un tiroir montré entier »** tombe sur la
  branche que l'ancienne épreuve laissait muette.
* **L'ancien compte des jalons seul tombe** — avec la mesure exacte : le bruit des flottants
  suffisait à perdre un jalon, le million n'était pas seul en cause. **La mesure du million
  seule** ne change plus aucun compte (la marge de l'écart l'absorbe) : sans l'épreuve « la
  hauteur naturelle est exacte », écrite pour elle, ce sabotage aurait été aveugle.

## 5. Le liseré du détourage, diagnostiqué (pas encore corrigé)

Sur `pomme.jpg` et son masque de u2netp à pleine taille (`detourage-121/sorties/`), en Python,
hors du dépôt (`~/.cache/glucose-bancs/session-127/liseré/`) :

* **La rangée du milieu** : de x = 1107 à 1111 le masque vaut 0,98-0,99, la photo y est déjà rose
  clair (153,101,103 → 159,117,119) ; posée sur du noir, elle donne le (152,109,111) que la fiche
  126 avait mesuré dans Glucose. Au-delà, une bande où l'opacité tombe de 0,9 à 0 sur du blanc :
  la traînée gris rosé.
* **Le filtre guidé** (He, Sun, Tang 2010 ; rayon déduit de la résolution du masque, 1 ou 2 fois
  le rapport image / 320) : presque rien — 3,71 → 3,55 % d'erreur d'opacité au mieux sur la boule,
  la pomme inchangée dans les quatre réglages. Il respecte ce que le masque affirme.
* **La ligne de couleur de Levin** (2008, `pymatting` sous MIT, l'option `alpha_matting` de rembg),
  une bande inconnue autour du bord du masque : à ± 2 pixels du masque (± 8 de l'image), le bord
  clair devient plus net — pire ; à **± 4 (± 16), il s'éteint en douceur** — le pic sur la rangée
  du milieu passe de (167,128,133) à (153,109,111). **Rien n'efface tout** : une part du rose est
  le vrai lustre de la pomme. 3,6 s en Python, et un solveur creux à écrire.
* **Ce qui manque pour choisir** : une mesure sur de **vraies photos avec leur vérité** (les
  vérités du banc sont des images de synthèse aux bords nets, sans reflet) — par exemple le jeu
  d'alphamatting.com, pour une mesure interne (ses conditions d'usage à lire d'abord).

## 6. Les GitHub Pages (sa note `text-24`)

* `https://shazamifius.github.io/GlucoseGit/` sert **la version web expérimentale de Glucose
  Tauri** (React, Pixi, Automerge, une PWA), déployée par un flux de juin (`db1b310`), le dernier
  déploiement le 10/09 (`6740c32`) ; le flux n'existe plus dans `main`, rien n'y renvoie.
* **La mise à jour de Glucose Tauri ne passe pas par elle** : `tauri.conf.json` de `tauri-v1.0.1`
  lit `releases/latest/download/latest.json`. L'enlever ne casse aucune mise à jour.
* **Le seul risque** : cette version gardait ses canevas dans le navigateur de chacun (IndexedDB) ;
  quelqu'un qui s'en sert encore les perdrait de vue. La question est dans son Glucose
  (`claude-code-pages`) ; son « enlève » suffit : `gh api -X DELETE repos/…/pages`, puis
  l'environnement `github-pages`.

## 7. La recherche (consultée le 20/11/2026)

* Apple, [*Scroll views*](https://developer.apple.com/design/human-interface-guidelines/scroll-views)
  (la version texte : `scroll-views.md`).
* VS Code, [*Sidebars*](https://code.visualstudio.com/api/ux-guidelines/sidebars) — rien sur le
  défilement des vues empilées ; GitKraken, une plainte contre un panneau par barre de défilement
  ([*undo the panel scrolling changes made in 8.8.0*](https://feedback.gitkraken.com/suggestions/321870/please-undo-the-panel-scrolling-changes-made-in-880-or-allow-us-an-option-to-tog)).
* Blender, [*Regions*](https://docs.blender.org/manual/en/latest/interface/window_system/regions.html)
  — une région sans zoom défile à la molette, au clic du milieu.
* MCP, [SEP-2127](https://github.com/modelcontextprotocol/modelcontextprotocol/blob/main/seps/2127-mcp-server-cards.md)
  (Final) ; [`experimental-ext-server-card`](https://github.com/modelcontextprotocol/experimental-ext-server-card).
* Le détourage : Forte, [*Approximate Fast Foreground Colour Estimation*](https://github.com/Photoroom/fast-foreground-estimation)
  (ICIP 2021, déjà DETOURER-2) ; Germer et al., [*Fast Multi-Level Foreground Estimation*](https://arxiv.org/abs/2006.14970)
  (ICPR 2020) ; [`pymatting`](https://github.com/pymatting/pymatting) (MIT) ; He, Sun, Tang,
  *Guided Image Filtering* (ECCV 2010) ; Levin, Lischinski, Weiss, *A Closed-Form Solution to
  Natural Image Matting* (TPAMI 2008).
* **Les séquences d'émojis** (familles, drapeaux, tons de peau), pour leur chantier :
  [`rustybuzz`](https://docs.rs/crate/rustybuzz/0.12.1/source/README.md) se dit *« effectively
  dead »* ; son successeur [HarfRust](https://docs.rs/crate/harfrust/latest), sous l'organisation
  HarfBuzz, lit les polices par `read-fonts` — un second lecteur à côté de `ttf-parser`.
  **`ttf-parser` 0.25.1, déjà là, lit les ligatures de `GSUB`** (`LigatureSubstitution` : couverture,
  ensembles, composants ; lu dans ses sources) : découper le texte en séquences (UTS #51), puis
  demander à la police la ligature de chaque séquence — **aucune caisse nouvelle** ; notre
  construction de Twemoji devra garder les séquences qu'elle écarte aujourd'hui.
* La server card : le document de découverte se dit encore *« Protocol Revision: draft »*
  ([`docs/discovery.md`](https://github.com/modelcontextprotocol/experimental-ext-server-card/blob/main/docs/discovery.md)),
  et aucun client connu ne lit le chemin final — faisable, pas prioritaire.
* Cloudflare : aucun transfert d'un Worker ni d'une base D1 entre comptes — on redéploie, et la
  base passe par [`wrangler d1 export` puis `d1 execute --file`](https://developers.cloudflare.com/d1/best-practices/import-export-data/).

## 8. Les leçons

* **Une passation qui dit « tout est lu » se vérifie nœud par nœud** : un nœud à lui sans flèche
  partante est une parole sans réponse.
* **Une cause se vérifie avant de se corriger** : j'ai « réparé » les 9 niveaux d'un tiroir défilé
  par une hypothèse plausible, et la compilation suivante a redit 9. Le calcul des deux couleurs
  (fond, filet) l'aurait dit en une minute — et il a montré un vrai défaut que l'hypothèse cachait.
* **Une épreuve qui espère son cas ne l'exerce pas** : « aux sept ouverts, un tiroir défile » ne
  défilait nulle part ; le cas se construit (l'échelle), et chaque branche s'exige (`if let` muet).
* **Le bord d'un objet photographié n'est pas le milieu de sa couleur** : un contour qui s'éclaircit
  (le reflet du fond) déplace ce milieu vers l'intérieur. Regarder l'image avant de dire un écart.
* **Un statut se lit dans le texte officiel** : « experimental » était le nom d'un dépôt, pas l'état
  de la SEP.
* **Regarder trouve ce que les épreuves ne cherchent pas** : 2 175 épreuves vertes, et cinq
  minutes au pilote ont montré deux défauts **publiés** (un jalon, une case) — MÉTHODE § 4.8, encore.
* **Un sabotage aveugle se prévoit** : quand une loi rend une autre garde invisible à ses épreuves
  (la marge de l'écart, la mesure exacte), la garde demande une épreuve à elle — écrite avant de
  saboter, pas après.

## 9. Où la session s'arrête

(Écrit par la session de la fiche 129, le 22/11, d'après le journal de celle-ci : elle s'est tue à
0 h 33 le 21/11 sans écrire ce paragraphe, ni commiter sa fiche, ÉTAT, SUITE, l'index et `CLAUDE.md`.)

À 0 h 33 : `main` à `596e783` sur GitHub (vert sur les dix tâches) ; `f9a8557` (DETOURER-2) commité
**en local seulement**. Les cinq sabotages du détourage côté bureau tournaient — **leur verdict n'est
écrit nulle part**. Son Glucose installé restait `aeea3ef` : la mise à jour attendait leur fin et un
envoi. Ses deux questions (les GitHub Pages, une beta.5) attendaient dans son Glucose ; le compte
Cloudflare, son clic. La session suivante (fiche 128) a pris sa demande du 21/11 sur les IA gratuites.
