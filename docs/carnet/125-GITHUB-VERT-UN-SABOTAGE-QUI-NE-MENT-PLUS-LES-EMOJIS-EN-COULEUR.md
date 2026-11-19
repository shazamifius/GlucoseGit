# 125 — GitHub vert sur trois sessions, un outil de sabotage qui ne ment plus, les émojis en couleur

> Session du 19/11/2026, depuis 14 h 35, après la fiche 124 (son travail sur la branche
> `verification`, le tour 3 de GitHub encore en cours). La passation : lire le verdict, fusionner
> ou corriger ; jouer les sabotages restants ; le compte Cloudflare (sa question dans son
> Glucose) ; la beta.3 ; les émojis en couleur.

## 1. En arrivant

* **Le tour 3 de GitHub est rouge** : une seule épreuve tombe, la même sous Linux, Windows et Mac
  Intel — `test_text_fit_2_a_l_ouverture_un_texte_grandi_pousse_sa_membrane` ; le Mac à puce
  Apple n'a jamais démarré (*« The job was not acquired by Runner »* : la capacité de GitHub, pas
  notre code).
* **Son Glucose** (par sa porte, les deux onglets) : rien de neuf ; `text-19`, la question du
  compte Cloudflare, sans réponse. `wrangler whoami` : un seul compte.
* **La mémoire** : 3,1 Go libres à 14 h 35, 3,7 à 14 h 38 — assez pour une compilation bornée.

## 2. Remis en question

* **« TEXT-FIT-2 : 9 sabotages sur 9 »** (fiche 124 § 5 et § 8) : faux pour l'épreuve de
  l'ouverture. Sa précondition exigeait que le texte grandisse « de plus de 100 » ; il grandit de
  78,4 (169,2 contre 90,8), ici comme sur GitHub — les polices sont embarquées, la mise en page est
  la même partout. L'épreuve **n'avait jamais passé, nulle part** : le reste de son corps n'avait
  jamais tourné, et ses sabotages « tombaient » sur la précondition. Trouvé sans compiler, en
  rejouant le binaire d'épreuves que la cible des sabotages gardait.
* **`outils/saboter.py` comptait « TOMBE » pour n'importe quel code de sortie non nul** — et ne
  jouait jamais le code intact d'abord. MÉTHODE § 4.1 le demandait déjà (*« toute épreuve neuve se
  lance d'abord sur le code intact, puis se sabote »*) : la règle existait, l'outil ne la tenait pas.
  Une règle à la sortie plutôt qu'une garde à chaque entrée (§ 4.16) : c'est l'outil qui l'exige
  désormais (§ 3).
* **« LISTE-1 : 6 sur 6 joués »** (fiche 124) : les deux derniers, jamais joués, **passent** — la
  place de la puce d'un enfant n'était gardée ni en pixels ni en vecteurs (§ 4).
* **« Les couches d'un émoji se rastérisent par `fontdue` »** (le chemin que la fiche 124 § 6
  supposait) : faux — `fontdue` ne prépare que les glyphes que la `cmap` atteint ; les couches lui
  seraient vides. Lu dans ses sources avant d'écrire (§ 5).

## 3. Le verdict de GitHub, et l'outil des sabotages

* **La précondition devient exacte** : le cas est exercé quand le texte neuf sort de la membrane
  d'avant, qui le contenait à l'écart du cadre près — `neuve > d_avant + ECART`. Le « + 100 »
  disparaît. Joué ici : les deux épreuves de TEXT-FIT-2 du bureau passent.
* **Le tour 4** (`d46f5e0`, lancé à 14 h 41) : **dix tâches sur dix vertes**, le Mac à puce Apple
  compris, à 14 h 56. `main` avancé sur GitHub, `verification` retirée : deux branches.
* **`saboter.py`** : chaque commande se joue d'abord sur le code intact (**le témoin**) ; s'il
  tombe, ses sabotages ne sont pas joués et l'outil le dit, avec la ligne qui tombe. Et « TOMBE »
  exige la ligne d'échec d'une épreuve (libtest, `node --test`, un essai qui écrit « NON ») : une
  édition de liens ratée, un processus tué par la borne ou Nix qui échoue disent désormais
  « ÉCHOUE SANS ÉPREUVE TOMBÉE ». Rejoué sur `texte-fit-124` : 9 sur 9, vrais cette fois.
* **L'arbre des sabotages a un chemin fixe** (`~/.cache/glucose-bancs/sabotage-arbre`, détaché) :
  sa cible (`sabotage-cible`) resert d'une session à l'autre.

## 4. LISTE-1 — deux épreuves aveugles

* **En pixels** : l'épreuve lisait la première encre sur **une seule rangée**, le milieu de la
  ligne ; la puce, un petit disque centré sur la hauteur d'œil, n'y passait pas — l'épreuve
  mesurait le texte de l'enfant, décalé lui aussi. Elle lit désormais toute la bande de la ligne :
  la première encre à gauche est la puce.
* **En vecteurs** : aucune épreuve ne lisait la puce que le SVG et la carte graphique reçoivent. Un
  peintre qui ne retient que les puces (`Puces`, `card/vecteurs/tests.rs`) : celle de l'enfant est
  un retrait (14) plus loin que celle d'un frère.

## 5. EMOJI-2 — les émojis en couleur

La décision, ses raisons et sa licence : `decisions/18`. Ici, ce qui a été construit et trouvé.

* **La police se construit** (`outils/emojis/construire.sh`) : les 4 009 dessins de Twemoji 17,
  dont 1 426 d'un seul caractère (Glucose ne dessine pas les séquences) ; nanoemoji les pose en
  COLRv0 aux métriques de Noto (em 2 048, avance 2 600, montante 1 900, descendante −500) ; le
  centre du 😀 tombe déjà au centre de celui de Noto (1 300, 700), l'échelle est le rapport des
  deux boîtes à surface égale (0,8646). Résultat : 851 172 octets (Noto : 886 628), 10 066 glyphes,
  9 181 couches, 513 couleurs ; et 4 004 octets pour ce que Twemoji n'a pas (56 caractères de Noto).
* **Ce que la vérification de l'outil a trouvé** : 🍉 — sa vue de 36 × 25,22 débordait l'avance
  (3 426) ; la loi « un dessin tient dans le carré de l'em » l'a corrigé, sans exception écrite. Et
  deux constructions identiques donnaient deux empreintes : l'heure de construction était écrite
  dans la police — elle vient désormais de la source (`SOURCE_DATE_EPOCH`, la date la plus récente
  de l'archive) : deux constructions, une empreinte (`a2309380…`).
* **Le choix d'un glyphe** (`typography/choix.rs`) : le visage, puis Twemoji, puis le reste de
  Noto, puis le `.notdef`. La clé du cache devient (police, indice).
* **Un émoji, deux lectures** (`typography/trace/couleur.rs`) : dessiné, ses couches, chacune de sa
  couleur (lues par `ttf-parser`, deux fois — d'une encre noire transparente puis blanche opaque —
  pour reconnaître une couche « de la couleur du texte » sans couleur sentinelle) ; lu d'une seule
  encre, sa silhouette. Une seule fonction pose un caractère en pixels (`Typography::poser`), que la
  ligne et le titre détouré appellent ; le tracé des grandes lettres et le peintre vectoriel (SVG,
  carte graphique) parcourent les mêmes couches, chacune dans un groupe de sa couleur.
* **Un émoji pâlit avec son texte** : la couleur d'une couche prend l'alpha de l'encre du texte —
  sans quoi un émoji resterait vif dans un nœud que la frise ou un domaine estompent.
* **Épreuves** : le vert est vert et le rouge rouge (16, 32 et 300 de corps — l'image et le tracé),
  l'émoji qui pâlit, la silhouette lue de loin (et la base seule, sans encre : la preuve à l'envers),
  les métriques de Noto, le SVG relu par resvg, les deux voies de près (SLUG-1). La suite entière
  du bureau : 2 151 sur 2 153 (§ 7 : les deux qui tombent ne viennent pas de là).

### Regardé dans le vrai Glucose

Au pilote (sway sans écran), le binaire de ce commit sur une copie de son document
(`~/.cache/glucose-bancs/pilote-125/`, captures 1 à 5) : l'onglet « Textes optimisés » — les
émojis des titres de membrane en couleur (🗄️, 🤝, 🚪) ; le nœud des tiroirs cadré : ✅, 🟢 et 🔴
distincts, 🔵 🟠 🟣, ❓, 💡 ; de près, 🖼️ et ses quatre couches (ciel, nuage, maison, cadre) sans
défaut, sur la voie où la carte graphique dessine les lettres depuis leur forme. La même vue que la
capture de la fiche 124, où 🟢 et 🔴 étaient le même rond rayé.

**Un défaut d'allure, à son jugement** : ✔️ (écrit avec son sélecteur d'émoji, U+FE0F — l'auteur a
demandé l'émoji), Twemoji le dessine gris foncé, comme Apple, Google et Discord ; sur le gris sombre
de Glucose il se lit mal. Fidèle, comme partout ailleurs ; la question est dans son Glucose
(`claude-code-coches`), avec mon avis : le garder, et en faire un choix le jour des presets.

### Deux erreurs de mes épreuves, trouvées en les jouant

* FONT-1 est tombée : j'avais écrit 🟢 et 🔴 **en clair** dans les messages des épreuves ; la règle
  veut chaque caractère littéral du code dessiné par Inter.
* L'épreuve des deux voies ne trouvait ni vert ni rouge : 🔴 tombait hors de l'image, et le seuil de
  « vert dominant » (+60) excluait le vert même de Twemoji (#78B159, +57). Trouvé **en faisant écrire
  les deux images par l'épreuve** et en les regardant, pas en devinant.

## 6. TABLE-4 — un filet entre deux rangées

Vu au pilote (fiche 124 § 3) : coupées en lignes, deux rangées ne se distinguaient plus. GitHub
pose un filet au-dessus de chaque rangée (`border-top` sur chaque `tr`, `primer/css`) ; Glucose
Tauri n'en avait aucun. La loi (`TextLayout::separe_une_rangee`) : la première ligne d'une rangée
qui suit une autre rangée. Le filet est de la largeur du texte, à la frontière des deux rangées, de
l'encre du filet de l'en-tête à 70 % : il dit « une autre rangée », pas « la fin de l'en-tête ». Le
rapport n'est pas choisi : dans Primer, `borderColor.muted` est `borderColor.default` à 0,7
d'alpha (`primer/primitives`, `borderColor.json5`, lu le 19/11). Les trois dessins l'appellent.

## 7. Les sabotages, tous avec témoin

Dans l'arbre à chemin fixe, sa cible, deux fils d'épreuves, un fichier par commande :

* `texte-fit-124` : **9 sur 9**, rejoué avec le témoin et la preuve d'une épreuve tombée ;
* `emoji-125` : **8 sur 8** (la base seule, la couleur oubliée, l'émoji qui ne pâlit pas, la
  silhouette réduite à la base, une grande lettre d'une seule encre, le SVG à l'encre de la ligne,
  Twemoji ou le reste de Noto jamais consultés) ;
* `table-125` : **7 sur 7** ;
* `liste-123` : **7 sur 8**, dont la puce en pixels qui **tombe enfin** (l'épreuve réparée) ; le
  dernier, la puce en vecteurs, arrêté avec la session (il compilait les épreuves d'intégration) ;
* **pas joués** : `temoin-123`, `table-123`, `tiroirs-122`, `porte-122`, `applique-122`,
  `echelle-121`, `detourage-121`.

**La machine** : à quatre fils, la suite entière du bureau a été tuée par sa borne (2,7 Go) — trois
épreuves en vol, légères chacune seule (100 à 430 Mo) ; à deux fils, elle passe, 1,45 Go au plus,
la mémoire du processus montant au fil des épreuves. Pas mesuré sur le code d'avant : une
accumulation qui existait peut-être déjà. L'épreuve du détourage dépasse son délai de 60 s sans
l'optimisation du dépôt (le profil des sabotages force `opt-level 0`) ; elle est verte sur GitHub.

## 8. La recherche (consultée le 19/11/2026)

* nanoemoji 0.16, ses sources : `color_glyph.map_viewbox_to_font_space`,
  `write_font._draw_glyph_extents`, `ninja.py` (`--noexec_ninja`), `codepoints.from_filename`.
* `ttf-parser` 0.25.1, `tables/colr.rs` (`paint_v0`) ; `fontdue` 0.9.4, `Font::from_bytes`.
* [`jdecked/twemoji` v17.0.3](https://github.com/jdecked/twemoji) — l'archive du tag, son empreinte
  vérifiée deux fois.
* GitHub, la feuille de style des tableaux Markdown
  ([`primer/css`, `markdown/tables.scss`](https://github.com/primer/css/blob/main/src/markdown/tables.scss)) :
  un filet d'un pixel sur chaque cellule, `padding: 6px 13px`, une rangée sur deux d'un fond
  plus doux — pour le chantier des rangées d'un tableau coupé.
* [Reproducible builds, `SOURCE_DATE_EPOCH`](https://reproducible-builds.org/docs/source-date-epoch/).

## 9. Les leçons

* **Un outil de preuve qui ne joue pas le cas témoin prouve n'importe quoi** : « 9 sur 9 » comptait
  huit fois l'échec d'une précondition. La règle était écrite (MÉTHODE § 4.1) ; c'est l'outil qui la
  tient désormais.
* **Une constante qui « exerce le cas » se dit par le cas** : « + 100 » est devenu « sort de la
  membrane d'avant » — et l'épreuve a passé partout.
* **Une épreuve qui lit une seule rangée de pixels lit autre chose que ce qu'elle croit** : la puce
  n'y passait pas, le texte si.
* **Lire la bibliothèque avant d'écrire le pont** (§ 4.20) : `fontdue` ne charge que la `cmap` — les
  couches d'un émoji lui seraient vides ; trouvé dans ses sources, pas au premier écran noir.
* **Une épreuve qui ne trouve rien se fait montrer ce qu'elle voit** : l'image écrite par l'épreuve
  des deux voies a montré le rond rouge hors du cadre en une seconde.
* **Une police construite est un produit : reproductible, vérifiée** — l'outil a trouvé 🍉 et l'heure
  de construction avant qu'aucun texte ne bouge.

## 10. Où la session s'arrête (16 h 20, à sa demande)

* **Sur `main`, sur GitHub** : `78ed630` (EMOJI-2, TABLE-4, LISTE-1, et la correction de clippy) ;
  sa CI (`gh run view 44551546465`) tournait — sept tâches vertes à 16 h 20.
* **Rien ne tourne sur sa machine.** Le dernier sabotage a été arrêté (un `rustc` tué : si la
  compilation suivante tombe sur des « undefined reference », `cargo clean -p glucose-desktop` dans
  la cible des sabotages) ; l'arbre des sabotages est propre, à `78ed630`.
* **Dans son Glucose** : deux questions sans réponse — le compte Cloudflare (`text-19`) et les ✔
  foncés (`claude-code-coches`).
* **Hors du dépôt** : `~/.cache/glucose-bancs/emojis-125/` (la construction, rejouable),
  `~/.cache/glucose-bancs/pilote-125/` (sa maison jetable, le binaire, les captures 1 à 5).
