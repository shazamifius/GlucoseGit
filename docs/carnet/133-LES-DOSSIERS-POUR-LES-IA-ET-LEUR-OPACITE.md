# 133 — Les dossiers pour les IA, et leur opacité

> Deux sessions du relais (compte a), le 23/11/2026 : de 14 h 15 à 14 h 50 (MCP-DOSSIER, `9accdfe`),
> puis de 14 h 55 à la limite du compte (OPACITE-1, `378239d` ; la sélection, Q12, commencée). Ses
> notes du 23/11, écrites dans son Glucose depuis son autre appareil (« Aigle Émeraude ») : *« les
> dossiers sont très transparents par défaut ; ce serait bien, avec le système de couleur, d'avoir
> une option pour changer son opacité »* ; le dossier qu'il voulait renommer « archive » (demain, le
> 24/11) ; et **la beta.5 avant ce soir**.

## 1. MCP-DOSSIER — une IA voit les dossiers (`9accdfe`, la session de 14 h 15)

Avant, quand une IA lisait son tableau, ses trois dossiers n'y étaient pas (seule la recherche les
trouvait), tous s'appelaient « Dossier », et aucun outil ne savait les renommer.

* **MCP-DOSSIER-1** : `glucose_lire` décrit chaque dossier — sorte « dossier », son nom, ce qu'il
  ouvre (`ouvre`), le chiffre de son badge (`contient`) —, au sommaire comme au plan.
* **MCP-DOSSIER-2** : l'outil `glucose_renommer` renomme un dossier ou un onglet, en un geste ; un nom
  vide et un inconnu sont refusés.
* **MCP-DOSSIER-3** : la lecture dit où est le tableau lu (`ou` : « Canvas Principal › archive »).
* **MCP-DOSSIER-4** : un identifiant demandé qui n'est pas sur ce tableau est dit (`introuvables`).
* **RENOMMER-2** : renommer un dossier (`F2` comme un agent) renomme aussi le tableau qu'il ouvre.
* La consigne aux IA ne dit plus que les émojis s'affichent en noir et blanc (faux depuis la beta.4) ;
  le relais ne prend plus le nom d'un dossier pour un identifiant.

Sabotages `mcp-dossier-133` 12 sur 12, `relais-133` 2 sur 2. Clippy strict et la suite entière,
joués par la session suivante sur `0b7a7ad` : verts, **3 513 épreuves**.

## 2. OPACITE-1 — remis en question

**Le plan de la passation** : l'opacité rangée dans la couleur (`#rrggbbaa`), 6/255 à la naissance,
choisi = min(2a + 1, 255), un second curseur dans la roue. Relu contre le code :

* **Pourquoi pas un champ à part ?** Vérifié : le format du document est positionnel (PERSIST-1,
  `persist/document.rs`, chaque champ dans l'ordre, une version par manifeste). Un champ de plus rend
  le document illisible pour la 2.0.4-beta.4 — celle de son autre appareil, qui partage ce canevas en
  direct. L'argument tient. `#rrggbbaa` est la forme de CSS Color 4 (W3C, *CSS Color Module Level 4*,
  § 5.2, la notation hexadécimale à huit chiffres) ; la beta.4 la lit comme une couleur inconnue et
  dessine le dossier au gris de sa naissance — rien ne se perd.
* **Mais le code de la couleur fuyait** : la pipette aurait pris `#rrggbbaa` et l'aurait posé sur un
  texte ; « Par défaut ou non » comparait le texte entier. D'où **trois fonctions du noyau, seules
  lectrices des deux derniers chiffres** (`teinte_de`, `opacite_de`, `couleur_de_dossier`) : la roue
  et la pipette lisent la teinte, colorer garde l'opacité, opacifier garde la teinte ; à l'opacité de
  la naissance, six chiffres, comme toutes les versions les lisent.
* **Le défaut que le plan ne voyait pas** : le titre, l'icône et le badge d'un dossier s'écrivent
  dans sa teinte. Un cadre opaque de cette teinte les efface (à 100 %, couleur sur couleur). **Le
  bandeau ne dépasse jamais l'opacité de la naissance ; le corps prend celle qu'on choisit.** Aucun
  seuil : le bandeau suit l'opacité vers le bas, jamais vers le haut. Le corps se compose par-dessus
  le voile du bandeau exactement — `1 − (1 − corps) / (1 − bandeau)`, l'opérateur « par-dessus » de
  Porter et Duff (*Compositing Digital Images*, SIGGRAPH 1984), retourné.
* **La sélection** : 2a + 1 « double », et sature à 255 dès 50 % (un dossier à 60 % devenait opaque
  une fois choisi). **Choisi, le cadre est peint deux fois** : 1 − (1 − α)² — sans plafond, il se
  distingue à toute opacité sous 100 %, et redonne les 5 % de la fiche 06 § 8 à 1/255 près (12 au
  lieu de 13).
* **Le curseur** montre, à chaque hauteur, la couleur posée **sur le fond du canevas** à cette
  opacité — ce que le dossier sera —, plutôt qu'un damier que le dossier ne recouvre jamais. La
  molette y avance d'une différence à peine visible (0,02) : le corps posé sur le fond change de
  clarté de Δα fois l'écart entre la teinte et le fond, jamais plus que Δα.

## 3. Ce qui est écrit (`378239d`)

* `glucose-core` : `types/folder.rs` (`OPACITE_DE_NAISSANCE`, les trois fonctions, `teinte()`,
  `opacite()`), `store/annotations.rs` (`colorer_la_selection` garde l'opacité,
  `opacifier_la_selection`), `store/selection.rs` (la roue et la pipette lisent la teinte),
  `store/agents/decrit.rs` (une IA lit la couleur entière).
* `glucose-desktop` : `renderer/folder.rs` (`opacites`, `remplir_le_corps`), `renderer/arrondi.rs`
  (`push_bas_arrondi`), `ui/roue.rs` (le curseur, `Endroit::Opacite`, `encoche`),
  `interactions/roue.rs` et `roue/opacite.rs` (`Glisse::Opacite`, `reposer` : chaque mouvement
  repart des couleurs d'avant et repose la couleur et l'opacité, chacune si la main l'a touchée),
  `ui/minimap.rs` (la teinte).
* `outils/publication/notes.md` : la phrase de la beta.5.

## 4. Vérifié

* **16 épreuves neuves** (noyau 4, dessin 5, roue 6, MCP 1), chacune lancée sur le code intact.
* **27 sabotages** (`outils/sabotages/opacite-133.json`) : RÉSULTAT À VENIR.
* **Regardé au pilote** (`~/.cache/glucose-bancs/opacite-133/`) : un dossier posé par la palette,
  coloré en bleu, la roue ouverte — deux curseurs —, l'opacité à 60 % : le corps bleu, **le nom
  lisible** dans le bandeau ; choisi, plus dense ; `Ctrl+Z` défait l'opacité en un geste (la couleur,
  posée avant, reste) ; lu par la porte d'essai : `#60a5fa`.
* Clippy strict, `cargo fmt` ; **la suite entière : 3 528 sur 3 529**. La 3 529ᵉ,
  `present::scene_gpu::tests::test_queue_1_la_cascade_compte_la_fin_de_l_image`, est tombée pendant
  que le Glucose d'essai du pilote tournait sur la même carte graphique ; seule, elle passe trois fois
  sur trois (§ 6).
* La vérification de GitHub (dix tâches) : run 45197129987, lancée à 15 h 42.

## 5. La sélection (Q12) — commencée

Sa réponse, le 23/11 : *« ah oui carrément, envoie les photos sur Glucose aussi, je choisirai »*. Le
15/10, devant cinq directions dessinées hors de Glucose (fiche 53), il avait pris E, *« pour
l'instant »*.

* **La « 0 », la sélection d'aujourd'hui**, capturée dans le vrai Glucose
  (`~/.cache/glucose-bancs/selection-q12/0-choisis.png`, la scène : `scene.sh`) : **quatre langages
  pour une seule idée** — un anneau blanc épais sur le texte, un fil presque invisible sur le post-it
  jaune, un cadre fin décollé sur la photo, des pastilles aux bouts de la flèche ; la membrane passe
  du pointillé au plein.
* **La recherche** (consultée le 23/11/2026) : tldraw trace le contour exact de chaque forme choisie
  sur **un seul calque**, plus le cadre du groupe ([indicators](https://tldraw.dev/sdk-features/indicators),
  [selection](https://tldraw.dev/sdk-features/selection)) ; le W3C demande qu'un indicateur de focus
  change ses pixels d'un contraste de 3:1 entre l'état choisi et l'autre, sur au moins le périmètre
  d'un trait de 2 px ([WCAG 2.2, 2.4.13](https://www.w3.org/WAI/WCAG22/Understanding/focus-appearance),
  niveau AAA) — le fil du post-it d'aujourd'hui n'y arrive pas.
* **À faire** : A, B, C dans le vrai Glucose (un prototype hors du dépôt, une variable
  `GLUCOSE_ESSAI_SELECTION`), posés dans sa membrane `relais-rep-rapides`. Aucun outil du MCP ne pose
  une photo (`poser_page` exige une adresse http) : à résoudre avant.

## 6. Vu, pas réglé

* **La cascade de `scene_gpu` croit l'horloge** : son « l'ancienne loi la faisait » dépend du temps
  réel de la carte graphique, et tombe quand un autre programme s'en sert (MÉTHODE § 4, principe 4).
* Son Glucose tourne encore sur l'ancien binaire : le relancer couperait l'accès du relais tant que
  « Toujours, dans ce document » n'est pas donné par lui (ACCÈS-H24).

## 7. Choix pris seul, à contester

* **Le bandeau reste léger à toute opacité.** Un dossier à 100 % ressemble à une carte pleine sous
  une étiquette : s'il veut le bandeau plein aussi, le titre doit changer d'encre (une encre claire
  ou sombre, celle qui contraste le plus — `contrast-color()` de CSS Color 5).
* **Pas de chiffre** : le curseur ne dit pas « 60 % ». L'encoche suffit-elle ?
* « Par défaut » rend la teinte de naissance et garde l'opacité choisie.
