# 68 — Les gardes de la fiche 67, plusieurs storyboards, et les planches

> Session du 14 au 15/10/2026 (après la fiche 67). Le téléphone n'était pas branché : P0
> attend toujours sa lecture par le câble. Le programme a fini la fiche 67, puis fait **P2.8 —
> plusieurs storyboards par tableau**, en un **schéma 5**, et **P2.7 — l'export en planches**,
> PNG et PDF. Les huit propositions du Storyboard sont écrites.

## 1. Ce qui est vérifié en ouvrant

* Le téléphone n'est pas branché (`adb devices` vide).
* **Il a lancé la build du schéma 4** : `target/release/glucose-desktop.exe` construit le 14/10 à
  17 h 13, après `5bb82ec` (17 h 05) ; sa chronique (`derniere-session.txt`, 17 h 16 min 20 s) et
  `sortie-67.txt` disent une session de 187 s, et 73 images où le document a changé. Ses
  documents peuvent donc porter des gestes écrits au schéma 4 : P2.8 ne pouvait plus se glisser
  dans le schéma 4 — **schéma 5**, et des témoins v4 écrits par la build d'avant (§ 5).

## 2. Les gardes aveugles de la fiche 67 — et celles que le sabotage a trouvées ensuite

* « le storyboard / une case écrit sans changement » (`store/storyboard/case.rs`) : **retirées**.
  Lu à la source : `Journal::record` jette toute édition nulle avant même de regarder la
  transaction, et son commentaire le dit — « c'est le journal qui tranche, une fois pour
  tous ». L'économie de la garde (un clone par case inchangée, au clic) ne se mesure pas.
* « l'arène ne pèse plus ses tables » : **une épreuve manquait** — la seule épreuve du poids ne
  posait que du texte. `test_le_poids_du_document_compte_ses_tables` pose une rotation, un
  recadrage (la table que la liste écrite deux fois avait perdue) et une case.
* « une durée tenue part au document » (`interactions/champ.rs`) : la garde était
  **inatteignable** — aucun des trois appelants de `pousser_les_durees` ne peut tenir le champ.
  Retirée ; le sabotage vise désormais ce qui compte : un glisser qui pousserait à chaque pas.
* Trois aveugles de plus, trouvés en finissant les 28 sabotages restants :
  * « la durée de la case sans case choisie » (`dock/champ.rs`) : **pas redondante** — elle sert
    à `Tab`, qui sans elle ouvrirait la frappe sur un champ absent. Son épreuve :
    `test_tab_ne_passe_jamais_a_un_champ_absent`.
  * « la caméra libre en lecture » et « la molette passe en lecture » : **le cas d'école des
    gardes qui se couvrent** — quatre mécanismes pour un même effet (l'élan remis à zéro au
    début et à la fin, la molette refusée à l'entrée, la caméra tenue). Chacun, retiré seul, ne
    faisait rien tomber. Et en les lisant, **un vrai défaut** : un pincement à deux doigts (au
    téléphone, ou au pavé) pendant une lecture s'accumulait dans `toucher.attente`, que rien ne
    vidait — à Échap, la vue d'avant revenait, puis sautait de tout ce qu'on avait pincé.
    **Une seule règle** les remplace : pendant une lecture, la caméra est la sienne, et ce
    qu'on lui demande se perd à chaque image (`oublier_ce_qu_on_demande_a_la_camera` : l'élan,
    les doigts et le pavé, un vol en route). Trois épreuves par les vraies voies, qui regardent
    **après la fin** ; la quatrième (le vol) lit l'état du vol — dans une épreuve, l'horloge
    n'avance pas, un vol vivant n'aurait rien déplacé.
* **47 sabotages, 47 chutes** (`outils/sabotages/p2-4.json`). Clippy pour Android, que la fiche
  67 n'avait pas relancé : propre.

## 3. La recherche — plusieurs storyboards

* **Storyboarder** : une scène est un fichier `.storyboarder` qui **possède** ses cases
  (`boardData.boards`), et porte son format (`aspectRatio`) et sa durée par défaut
  (`defaultBoardTiming`) ; une case ne sait rien de sa scène. Plusieurs scènes passent par un
  script (`.fountain`), une scène à la fois (`main-window.js` : `scriptData`, `currentScene`).
* **Toon Boom Storyboard Pro** : une hiérarchie stricte — la case dans la scène, la scène dans
  la séquence (« une suite de scènes à regrouper, d'ordinaire par lieu »), les actes au-dessus.
* **Boords** : un projet porte plusieurs storyboards, et **le format se choisit par
  storyboard**, à sa création.
* **L'export PDF de Storyboarder** (`exporters/pdf/generate.js`, `group-by-page.js`) : **une
  page ne mélange jamais deux scènes** ; un en-tête (titre, nombre de cases, durée, format, la
  date), « n / m » en bas ; par case : l'image, le plan, la durée, dialogue, action, notes. Pour
  la proposition 7.

Sources : [Storyboarder](https://github.com/wonderunit/storyboarder) (`src/js/window/main-window.js`,
`src/js/models/board.js`, `src/js/exporters/pdf/`),
[Toon Boom — la structure d'un storyboard](https://docs.toonboom.com/help/storyboard-pro-22/storyboard/structure/about-storyboard-structure.html),
[Boords — les formats](https://help.boords.com/en/articles/6929959-frame-sizes),
[Krita — Storyboard Docker](https://docs.krita.org/en/reference_manual/dockers/storyboard_docker.html).

## 4. La structure : la case qui sait son storyboard (remise en question)

Deux formes : **A**, la case porte l'identifiant de son storyboard (le tableau garde la liste
plate des cases et la liste des storyboards) ; **B**, le storyboard contient ses cases — la
forme de Storyboarder, qui rend la case orpheline impossible par construction. **A est
retenue**, pour trois raisons propres à Glucose :

1. **L'arène est plate** : une case y est un nœud, le storyboard une donnée du tableau (comme
   les zones) — la forme B y redeviendrait un lien de toute façon.
2. **Le journal garde `Edit::Panel`** : défaire une case coûte une case (JRN-1) ; sous B, régler
   une grille clonerait ses cases, ou il faudrait trois sortes d'éditions.
3. **Les gestes d'avant se relisent sans traduire leurs cases.**

Le prix de A est un invariant — **SEQUENCE-1 : toute case nomme un storyboard de son tableau,
et tout storyboard a au moins une case** — tenu par chaque geste et ramené à la lecture par une
seule règle (`types::rattacher`), et deux épreuves (§ 7).

## 5. Le schéma 5

* `StoryboardPanel::storyboard`, `Storyboard::id`, `Board::storyboards` (une liste).
* **Un document d'avant** (schémas 1 à 4) et **Glucose Tauri** : leur unique storyboard, et
  chacune de ses cases, prennent le nom `STORYBOARD_D_AVANT` (« storyboard ») — un nom propre au
  tableau, qu'un geste de son histoire lui donne aussi ; des cases sans grille (schémas 1 et 2)
  le reçoivent à la grille de Tauri. **Une grille sans aucune case ne fait plus de storyboard** :
  il ne montrerait rien, et rien ne le désignerait (Tauri pouvait l'avoir).
* **Au journal, un storyboard est une case de la liste** (`Edit::Storyboard`, un `Slot`). Une
  coïncidence de format : l'ancien « avant / après » optionnel des schémas 3 et 4 est
  **exactement** cette case au rang 0, sans son rang écrit — le schéma 5 n'ajoute que le rang.
* Un identifiant de storyboard est propre à son tableau : l'import en onglets ne le renomme pas,
  le lot (copier-coller) ne porte ni cases ni storyboards ; ID-1 les compte.
* **Témoins écrits par la build d'avant** (`5bb82ec`, la seule à écrire du v4) : `riche-v4.glucose`
  et `storyboard-v4.geste`. La fixture riche porte désormais **deux** storyboards : une seule
  fonction la ramène à ce qu'un document d'avant pouvait dire (`d_avant_les_sequences`).

## 6. Les gestes

* **Le magasin** : chaque geste qui vise un storyboard le nomme (`disposer((board, cible), d)`
  — sans cible, il en naît un ; `ajouter_une_case`, `effacer_le_storyboard`, les réglages de
  durée et de champs…) ; ceux qui partent d'une case le lisent sur elle (`retirer_la_case`,
  `deplacer_la_case`). **Ranger un storyboard ne touche que ses places** dans la liste
  (`store/storyboard/ranger.rs`, `remplacer_la_suite`) : les cases des autres ne changent pas
  de rang, et le journal ne les écrit pas.
* **Le storyboard courant** (`storyboard_courant`) : le dernier touché — choisir une case le
  règle — ou posé, sinon le premier. **Une seule source** : la première version lisait d'abord
  la case choisie, et les deux divergeaient après « + Nouveau » (la case de l'ancien restait
  choisie). Désormais, poser un storyboard neuf rend la case choisie.
* **Le panneau** : « ‹ 2 / 3 › » à côté du titre quand il y en a plusieurs ; le bouton se
  dédouble — « Mettre à jour » (le courant) et « + Nouveau » ; la lecture, « + Ajouter », la
  croix, les champs visent le courant. La grille armée sait sa **cible** (`Placement::cible`).
* **Un storyboard de la sélection en fait naître un** à côté des autres (jusqu'ici, ses cases
  s'ajoutaient à l'unique) ; plusieurs images lâchées remplissent le storyboard de la case sous
  elles ; la lecture lit le storyboard de sa case, et s'arrête à sa dernière.
* `add_panel` et `remove_panel` (des cases sans storyboard, l'un sans appelant, l'autre appelé
  par une seule épreuve) et `placer_la_case` (appelé par les épreuves seules) : **retirés**.
* **Le numéro d'une case reste celui qu'elle a dans son storyboard** : les séquences sont
  séparées sur le canevas, et préfixer « 2.3 » dans un badge prévu pour deux chiffres est un
  choix visuel — à sa parole.
* **Regardé** : `apercu_storyboard` pose désormais deux storyboards — le panneau dit « 1 / 2 »,
  « Mettre à jour » et « + Nouveau » côte à côte, la séquence verticale numérotée de 1 à 6.

## 6 bis. P2.7 — l'export en planches

* **Ce que le canevas montre est la planche.** Un storyboard est rangé par sa grille : sa
  planche de rang `p` est le rectangle du canevas qui porte ses rangées `[p·R, (p+1)·R)`, toutes
  ses colonnes, bordé d'un écart (`Disposition::planches`). Elle se rend par l'écrivain de
  l'export en image, au même 100 % — aucun second moteur de mise en page, ce que Storyboarder
  et Krita ont.
* **Le nombre de rangées par planche n'est pas un réglage : il se déduit du format A** (ISO
  216 — le rapport √2, la seule feuille qui garde sa proportion pliée en deux). `R` est le
  nombre de rangées dont la hauteur est la plus proche de la largeur sur √2 ; les colonnes sont
  celles de la grille. Imprimée, une planche remplit sa feuille, A4 comme A3. Un storyboard plus
  court qu'une feuille tient sur une planche à sa mesure (vu sur l'aperçu : deux rangées, une
  planche de quatre, la moitié vide).
* **Une planche ne mélange jamais rien** — la règle de Storyboarder, trouvée **en regardant
  l'image** : la première version rendait le rectangle tel quel, et la moitié basse montrait
  l'autre storyboard, posé dessous. La copie qu'on exporte ne garde que ce storyboard, ses
  cases et ce qui est posé dedans (`Store::copie_du_storyboard` : une image, un texte par leur
  centre, la règle d'appartenance d'une image à sa case).
* **PNG ou PDF**, au nom choisi : `.pdf`, un fichier, une page par planche ; `.png`, une image
  par planche (`nom-1.png`…). Au téléphone, un PDF part à la feuille de partage.
* **Le PDF, écrit à la main** (`persist/export/pdf.rs`), sans dépendance : la méthode d'img2pdf
  — les blocs `IDAT` du PNG de la planche, mis bout à bout, posés tels quels sous
  `/FlateDecode` et `/DecodeParms << /Predictor 15 … >>` ; aucun pixel recompressé. L'écrivain
  de l'export PNG écrit désormais dans n'importe quelle sortie (`rendre_en_png`), et son fil, son
  moteur se prêtent (`lancer_le_travail`, `moteur`). **Éprouvé dans deux lecteurs
  indépendants** — MuPDF et PDFium (celui de Chrome et d'Edge), par `pymupdf` et `pypdfium2`
  dans un environnement du scratchpad : la page rendue à 96 points par pouce est **la planche
  PNG au pixel près** (écart maximal 0 sur 1 128 400 pixels, dans l'un et l'autre). Le
  prédicteur que Telerik dit mal lu par certains lecteurs est bien lu par ces deux-là.
* **Les boutons** : « Exporter en planches… » dans le panneau, sous « ▶ Lire », et au menu du
  vide après « Lire le storyboard » — au PC et au téléphone.
* **Un défaut d'EXPORT-PNG-1 trouvé en passant** : l'export de la sélection gardait les cases
  du storyboard et les zones d'un gabarit, dont les cadres se seraient dessinés autour des
  photos choisies. Retirés ; l'épreuve de la sélection en pose entre ses photos.
* **Ce qui reste à sa parole** : la planche est sombre comme le canevas — une encre lourde à
  l'impression ; Storyboarder imprime sur blanc. Et la planche ne porte que ce que la bande
  d'une case montre (le numéro, le sigle, la description) : ni dialogue, ni action, ni notes,
  que Storyboarder imprime sous chaque case.

## 7. Ce qui est prouvé

* Les témoins v1 à v4 se relisent, un geste v3 et un geste v4 aussi ; l'histoire rejoue deux
  storyboards rangés et effacés chacun à part ; SEQUENCE-1 tient après chaque geste, en défaisant
  et en refaisant ; six épreuves au bureau par les vrais clics.
* **2 526 épreuves** sous Windows ; clippy strict sous Windows et pour Android.
* **Sabotages** : `p2-4.json` (47, 47 chutes) ; `p2-8.json` (24 : 23 chutes, et ID-1, aveugle
  par la faute de son épreuve — un seul storyboard « storyboard-500 » ne collisionnait jamais —,
  réécrite et rejouée dans `p2-7.json`) ; trois gardes de P2.8 trouvées redondantes au sabotage
  et retirées (le nom du storyboard réécrit au rangement, les storyboards vidés d'un lot, la
  cible vidée en renonçant) ; `p2-7.json` (19, 19 chutes — une épreuve des planches lisait
  ses pixels par la vue même du rendu, et une vue fausse s'y lisait juste : elle les place
  désormais d'après la loi du noyau ; un sabotage mal écrit, de ma main, ne compilait pas :
  réécrit et rejoué).

## 8. Les erreurs de la session

* **Une adaptation mécanique qui retire le sens** : une épreuve de rendu construisait sa lecture
  avec un nom de storyboard vide — elle compilait, et tombait (le voile ne se posait sur rien).
  Après une adaptation par script, lire chaque épreuve qui tombe avant de la « réparer ».
* **Deux sources pour un même état** (le storyboard courant : la case choisie, puis le dernier
  touché) — vu en préparant les sabotages, avant qu'aucune épreuve ne le dise.
* **Un rectangle rendu tel quel** : la planche montrait l'autre storyboard, posé dessous —
  aucune épreuve ne le disait, l'image si. Une page ne prend que ce qui est à elle.
* **Une recherche d'octets dans une conversion en texte « avec pertes »** (l'épreuve du PDF) :
  un octet invalide y devient trois, et chaque position se décale — vu en relisant l'épreuve.
* Encore un heredoc refusé par Git Bash (des apostrophes) : tout script passe par un fichier.

## 9. À essayer à l'écran

`cargo run --release > sortie-68.txt 2>&1`, sur un tableau qui a un storyboard :

* panneau Storyboard → « + Nouveau », puis un clic ou un rectangle : un second storyboard ; le
  panneau dit « 2 / 2 » ; ‹ › passent de l'un à l'autre ; choisir une case de l'un en fait le
  courant ; « Mettre à jour », la croix, « + Ajouter », « ▶ Lire » n'agissent que sur lui ;
* « Exporter en planches… » (panneau, ou clic droit sur le vide) → un `.pdf`, puis un `.png` :
  l'ouvrir, l'imprimer ;
* rouvrir un document écrit pendant la session de la build du 14/10 (le schéma 4) : tout y est.
