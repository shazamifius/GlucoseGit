# 124 — Deux sessions validées par GitHub ; un texte qui grandit pousse sa membrane ; les émojis en couleur, à portée

> Session du 19/11/2026, depuis 13 h 43, après la fiche 123 (son travail et celui de la 122 restés dans
> l'arbre, jamais compilés : sa machine ne laissait que 2 Go). La passation demandait de valider
> l'arbre, de jouer les sabotages, de regarder au pilote, de préparer le compte Cloudflare de Glucose,
> de refaire la beta.3 et de commencer les émojis en couleur.

## 1. En arrivant

* **La mémoire** : 2,4 Go libres (VS Code, une application Electron, une autre session de Claude) —
  sous la borne, rien ne compile. La voie de GitHub d'abord ; une demi-heure plus tard, 3,6 Go : la
  compilation et les sabotages ont pu se faire ici.
* **Son Glucose** (lu par sa porte, les deux onglets) : rien de neuf de sa part depuis le matin. La
  question du compte Cloudflare, posée la veille dans la conversation, n'était pas dans son
  document : posée dans son Glucose à 14 h (`text-19`, à droite de la porte), en prose.
* **La télémétrie** : 205 sessions, 10 installations, 40 fins anormales — rien depuis 12 h 10.

## 2. Remis en question

* **« `richtext.rs` est à 592 »** (la passation) : 601 lignes de production, une de trop pour le
  cliquet. Trouvé sans compiler, par un portage en Python de la lecture du cliquet (`sans_les_tests`,
  la mesure des fichiers et des fonctions, puis celle du couplage), vérifié d'abord sur `aa72ab8c`
  (zéro faute, comme sa CI). Extrait : le genre d'un bloc (`richtext/genre.rs` — taille, graisse,
  encre, retrait, la place de l'ornement), la seule frontière entre ce qu'un bloc est et ce qu'il
  devient.
* **« Le travail de la 122 est validé par la 123 »** (la fin de la fiche 122) : faux, rien n'avait été
  compilé ; une correction datée y est ajoutée, et à la 121 pour ses sabotages.
* **L'ordre de la passation** (sabotages, puis pilote) contre MÉTHODE § 3 (*« regarder avant de
  saboter »*) : la méthode l'a emporté — et l'image a trouvé un défaut que les sabotages n'auraient pas
  cherché (§ 3).
* **« Les schémas en caractères se cassent »** : celui du co-working, dans un nœud de 1 000, ne se
  coupe pas ; son bord droit est décalé d'une cellule sur la ligne de « 🏠 ». La source le dit : ses
  55 caractères comptent l'émoji pour **un** — un terminal le dessine sur deux, un navigateur (donc
  Glucose Tauri) à l'avance de sa police d'émojis. Le schéma était faux chez son auteur ; Glucose le
  dessine comme un terminal. Reste la vraie question, un bloc de code plus large que son nœud.
* **Les choix que la 123 avait tranchés seule** : gardés, chacun avec son argument. Une rangée de
  tableau en édition montre sa source, comme tout le texte en édition (MODE-1). Sous la largeur
  minimale, les colonnes se réduisent et les mots se coupent : c'est la règle de `wrap_paragraph`
  partout ailleurs, et « un texte ne sort jamais de son nœud » vaut mieux que le débordement de la
  CSS. Les listes suivent CommonMark (W + N), que GitHub suit aussi. Le témoin se redit à chaque
  changement d'onglet : c'est ce qui manquait le 19/11.
* **ACCES-1 tenait « un écran de 900 pixels tient le menu »** : vrai sous Linux, faux sous Windows,
  où « Barre de Windows » s'ajoute aux deux crans d'ECHELLE-1 (§ 4). L'écran qui tient le menu se
  mesure désormais au menu lui-même.

## 3. Regardé dans le vrai Glucose, sur une copie de son document

Le binaire release du dépôt, au pilote (sway sans écran), sa copie dans une maison jetable
(`~/.cache/glucose-bancs/pilote-124/`, les captures numérotées) — l'onglet « Textes optimisés » :

* **les tableaux tiennent dans leur nœud** : chaque cellule se coupe dans sa colonne, la colonne des
  libellés à son mot le plus large — ce que fait la combinaison de *CSS Tables 3* ;
* **mais les rangées ne se distinguent plus** : une rangée de trois lignes suit la précédente sans
  écart ni filet. Glucose Tauri n'avait aucun style de tableau (le défaut du navigateur : deux pixels
  entre les cellules) — à concevoir, pas une régression ;
* **🟢 et 🔴 sont le même rond rayé**, à côté d'« Aujourd'hui » et d'« Avec la loi » : la preuve à
  l'écran qu'il faut les émojis en couleur (§ 6) ;
* **les listes imbriquées** : « 1. Côté du bouton » porte ses deux puces décalées d'un retrait, puis
  « 2. » reprend au bord (LISTE-1, tel que voulu) ;
* **les textes sortent par le bas de leur membrane** : coupés en lignes, les tableaux font grandir
  leur texte, et la membrane reste — celui de la porte dépasse d'une centaine de pixels à ce zoom.
  D'où TEXT-FIT-2 (§ 5) ;
* **les sept tiroirs ouverts** à 1920 × 1080 : chacun du côté de son bouton, collé au bord, une
  largeur par colonne, plats, aucun l'un sur l'autre ; à 110 et 125 % (« Agrandir l'interface » deux
  fois au menu du vide), rangés de nouveau sans se croiser — la bande replie ses libellés de droite
  en icônes.

## 4. La vérification de GitHub, en trois tours

Sur une branche `verification` (la CI ne part pas d'elle-même hors de `main` : `gh workflow run`),
retirée dès la fusion.

1. **`0a1b5be`** : clippy de Rust 1.100 refuse `type_complexity` (273 pour un seuil de 250) sur
   `(&Block, &[Cellule], &[Vec<(usize, usize)>])` — les lignes d'une cellule coupée ont un nom,
   `Lignes`. Le reste de la bibliothèque passait la vérification des types : rien d'autre à reprendre
   dans le code non compilé.
2. **`8c4e579`** : tout compile partout ; deux épreuves tombent — FONT-1 (les émojis de ses tableaux,
   recopiés en clair dans l'épreuve de TABLE-3 : le code les écrit en échappements) et ACCES-1 sous
   Windows (§ 2).
3. **`b4a633e`** (TEXT-FIT-2 et les deux corrections) : lancé à 14 h 21 (`gh run view 44532678404`) ;
   à 14 h 29, sept tâches sur dix vertes (le noyau pour Android et le web, l'APK, NixOS, les paquets
   Linux, la répétition de la bascule, l'installeur…), Linux, Windows et les deux Mac encore en cours.
   **Son verdict n'est pas lu** : c'est la première chose à faire en reprenant.

## 5. TEXT-FIT-2 — un texte qui grandit pousse sa membrane

`glucose-core/src/store/pousser.rs`. **Le bas d'une membrane suit le bas de ce qui grandit en elle**,
avec l'écart du cadre (GROUPER-1, 30 unités), et celle qui la contient suit à son tour (MEMB-7).

* **Seul ce qui grandit pousse.** Une loi « s'il était contenu avant » aurait laissé dehors le texte
  du co-working, qui **dépassait déjà** sa membrane de 19,6 unités dans l'ancienne mise en page.
  L'événement est la croissance : un texte qui ne change pas ne touche à rien, même s'il dépasse.
* **Seul le bas bouge** : ni le titre, ni les côtés, ni les voisins (MEMB-7 ne les pousse pas).
* **L'appartenance est celle du document** (`membrane_id`) : un nœud déposé ailleurs change de
  membrane (MEMB-1, `reconcile_membership`), un texte tiré dehors ne l'étire donc jamais.
* **Une loi, deux appelants** : le chargement (`ajuster_les_cartes`, hors du journal — sorti de
  `resize.rs`, qui passait à 607 lignes, dans `resize/ajuster.rs`) et la saisie
  (`fit_text_card_height`, dans le geste ouvert : un seul `Ctrl+Z` défait le texte et sa membrane).
  Le cliquet du couplage a refusé que le bureau applique la poussée lui-même (deux lignes de plus
  sur `.annotations`) : c'est le noyau qui l'applique à un tableau (`pousser_dans_le_tableau`), et le
  plafond reste à 38.
* **Les épreuves** : cinq dans le noyau (la poussée, ce qui tient encore, la cascade, un cycle — sur
  un fil avec un délai, pour qu'une garde cassée fasse tomber l'épreuve au lieu de la bloquer —, le
  journal, avec une membrane dont le haut n'est pas zéro), deux dans le bureau (l'ouverture d'un
  document écrit quand le tableau ne se coupait pas, la saisie et son `Ctrl+Z`). **9 sabotages sur 9**
  (`outils/sabotages/texte-fit-124.json`, ACCES-1 compris).

## 6. Les émojis en couleur — la recherche, et le chemin

Sa parole : *« je suis d'accord avec tes propositions »* (fiche 122 § 6). Ce que Glucose dessine
aujourd'hui : Noto Emoji, monochrome (`decisions/10`).

* **Le format** : COLRv0 — un émoji est une **pile de glyphes ordinaires**, chacun d'une couleur de la
  palette (CPAL). COLRv1 ajoute dégradés et compositions. **L'argument décisif pour COLRv0** : les
  trois dessins de Glucose — les pixels (`blend_glyph`), la carte graphique (le relevé, qui pose un
  glyphe à l'encre de son groupe) et le SVG (`<g fill>` autour de `<use>`) — savent déjà poser un
  glyphe d'une couleur. Rien de neuf à leur apprendre que l'ordre des couches.
* **La lecture** : `ttf-parser` 0.25, déjà une dépendance du bureau (`decisions/13`), lit `COLR` et
  `CPAL` sans option (`Face::paint_color_glyph`, un `Painter` : `outline_glyph`, puis `paint`) —
  aucune caisse neuve.
* **Les polices, mesurées** :

  | | ce qu'elle est | pourquoi |
  |---|---|---|
  | **Twemoji Mozilla 0.7.0** (oct. 2022) | COLRv0, 1,47 Mo ; 13 723 glyphes, 1 418 caractères, 3 689 émojis en couleur, 33 179 couches (44 au plus), une palette de 1 063 couleurs ; 🟢 et 🔴 sont le même disque, d'une autre couleur | figée en 2022 (Unicode 14) |
  | **Twemoji 17** (`jdecked/twemoji`, v17.0.3, juin 2026) | 4 009 dessins SVG, **aucun dégradé** ; dessins sous CC-BY 4.0, code sous MIT | **pris** : à construire en COLRv0 par **nanoemoji** (Google, Apache 2.0 ; 0.16 dans nixpkgs) |
  | Fluent Emoji, plat (Microsoft, MIT) | la seule licence sans attribution ; Windows dessine ses émojis plats en COLRv0 | la police plate pèse **27 Mo** |
  | Noto Color Emoji (OFL) | en COLRv1, avec des dégradés | des dégradés à dessiner sur les deux voies |
  | OpenMoji | CC BY-SA | une licence qui contamine (`decisions/10`) |

* **Ce qu'il faut garder** : les métriques de Noto Emoji — une avance de 1,27 em, un 😀 dans une
  boîte d'environ 1,02 em, centrée à 0,34 em au-dessus de la ligne de base —, pour que les textes déjà
  posés ne se remettent pas en page.
* **Ce qui reste à mesurer avant d'écrire** : ce que coûte la police à `fontdue`, qui prépare chaque
  glyphe au chargement (13 723 pour celle de Mozilla) — sinon, les couches se rastérisent par leurs
  contours (`trace::Contours`, lus à la demande). Et le cache des glyphes, indexé aujourd'hui par
  (visage, caractère), devra l'être par (police, indice) : deux visages qui retombent sur le même
  émoji partageront son image.
* **Ce qui ne change pas** : sans moteur de mise en forme, une famille (👨‍👩‍👧) reste des personnes
  côte à côte et un drapeau deux lettres (`decisions/10`).

## 7. Une veille n'est pas un gel — la loi, trouvée

L'image de 415 s (fiche 123 § 6). Sous Windows, l'`Instant` de Rust lit QueryPerformanceCounter, qui
**compte** la veille ; `QueryUnbiasedInterruptTime` ne la compte pas. L'écart entre
`QueryInterruptTime` et `QueryUnbiasedInterruptTime` est donc **le temps passé endormi depuis le
démarrage** ; Linux et Android le donnent par `CLOCK_BOOTTIME` moins `CLOCK_MONOTONIC`, macOS par
`mach_continuous_time` moins `mach_absolute_time`. Une image pendant laquelle ce compteur a grandi
contenait une veille : la retrancher, sans aucun seuil. À écrire, le serveur avant le client.

## 8. Les sabotages

Dans un arbre détaché à `b4a633e`, sa propre cible (`~/.cache/glucose-bancs/sabotage-cible`), sous la
borne — la pression sur son bureau est restée à zéro (`session-124/sonde-du-bureau.txt`) :

* `texte-fit-124` : **9 sur 9** ;
* `liste-123` : **6 sur 6 joués** (les deux derniers, dont « le dessin vectoriel », interrompus par la fin de
  la session) ;
* **pas encore joués** : `temoin-123` (2), `table-123` (10), `tiroirs-122` (6), `porte-122` (3),
  `applique-122` (2), `echelle-121` (5), `detourage-121` (25). Chaque passage visé a été vérifié présent,
  une seule fois, dans le code de `b4a633e` (un sabotage de `liste-123` suit `indent_of` dans
  `richtext/genre.rs`).

## 9. Le compte Cloudflare, prêt à son clic

Le Worker qui transmettra l'ancienne adresse de la boîte noire est prêt
(`~/.cache/glucose-bancs/session-123/outils/ancienne-adresse/`, le sous-domaine à remplir). Une
conséquence à lui dire : l'ancienne porte reste ouverte pour les versions installées, et **pendant la
transition, deux personnes ne partagent un canevas que si elles ont la même version** — la salle vit
sur le relais de leur version. Un compte se voit par `npx wrangler whoami` : à 13 h 55, un seul.

## 10. La recherche (consultée le 19/11/2026)

* Twemoji : [`mozilla/twemoji-colr`](https://github.com/mozilla/twemoji-colr) (v0.7.0, sa licence :
  code Apache 2.0, dessins CC-BY 4.0) ; [`jdecked/twemoji`](https://github.com/jdecked/twemoji)
  (v17.0.3, `LICENSE` MIT, `LICENSE-GRAPHICS` CC-BY 4.0) ;
  [Emoji-COLRv0](https://github.com/Emoji-COLRv0/Emoji-COLRv0) (des conversions, des licences qui se
  contredisent entre son tableau et son texte).
* Fluent : [`tetunori/fluent-emoji-webfont`](https://github.com/tetunori/fluent-emoji-webfont) (les
  polices construites, `dist/FluentEmojiFlat.ttf` 26,9 Mo) ;
  [Microsoft, *Bringing new emoji to Windows 11*](https://microsoft.design/articles/bringing-new-emoji-to-windows-11/)
  (*« for the 2D Fluent emoji, we chose COLRv0 »*).
* nanoemoji : `nixpkgs#python3Packages.nanoemoji` 0.16.0 (`--color_format glyf_colr_0`).
* La veille : [`QueryUnbiasedInterruptTime`](https://learn.microsoft.com/windows/win32/api/realtimeapiset/nf-realtimeapiset-queryunbiasedinterrupttime)
  (*« does not include time the system spends in sleep or hibernation »*) ;
  [le temps d'interruption](https://learn.microsoft.com/windows/win32/sysinfo/interrupt-time) ;
  [Rippling, *Rust suspend time*](https://www.rippling.com/blog/rust-suspend-time) (l'`Instant` de Rust
  sous Windows avance pendant la veille) ; la caisse [`suspend-time`](https://docs.rs/suspend-time).

## 11. Les leçons

* **Un cliquet se lit sans compiler** : porté en Python et vérifié sur un commit vert, il a trouvé le
  fichier d'une ligne de trop et le couplage, avant toute CI — quand la machine est pleine, c'est la
  vérification la moins chère.
* **Une épreuve qui fixe une taille d'écran sans la justifier tombe à la première entrée neuve** :
  la taille qu'elle teste se mesure à ce qu'elle doit tenir.
* **Une loi « s'il était contenu avant » se vérifie sur les vraies données** : le texte du co-working
  dépassait déjà — la loi juste porte sur l'événement (grandir), pas sur l'état d'avant.
* **Une épreuve dont la garde pourrait boucler tourne sur un fil, avec un délai** : sinon, le sabotage
  de la garde bloque la suite au lieu de la faire tomber.

## 12. Où la session s'arrête (14 h 30, à sa demande)

* **Sur GitHub, branche `verification`** : `b4a633e` (le travail des fiches 122 et 123, TEXT-FIT-2, les
  corrections de la CI) et le commit des documents de cette fiche. **`main` n'a pas bougé** (`aa72ab8c`) :
  il attend le verdict du tour 3. Vert : `git push … verification:main`, puis retirer `verification`
  sur GitHub et en local. Rouge : corriger, renvoyer sur `verification`, relancer.
* **Pas fait** : les sabotages restants (§ 8) ; le compte Cloudflare (son clic) ; la beta.3 ; le rendu des
  émojis en couleur (la recherche est faite, § 6) ; la séparation des rangées ; les tiroirs qui
  défilent ; la veille dans la télémétrie (la loi est trouvée, § 7).
* **Hors du dépôt** : `~/.cache/glucose-bancs/session-124/` (le portage des cliquets en Python,
  `outils/cliquets-sans-compiler.py <racine du dépôt>`) ; `pilote-124/` (la copie de son document, les
  captures) ; `emojis-124/` (Twemoji de Mozilla, les dessins de Twemoji 17, les scripts fontTools).
