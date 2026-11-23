# 134 — Le relais jamais figé, la flèche vers une membrane, la sélection en images

> Le 23/11/2026, compte b. La session du relais lancée à 16 h s'est figée 1 h 48 au démarrage ; reprise
> à 17 h 48, elle s'est figée de nouveau à 18 h ; il a repris **la même conversation** dans son VS Code
> à 18 h 43 (« continue »), et c'est elle qui finit. Sa consigne : *« mets à jour le Glucose sur tous les
> points et réponds aux questions ; je te demande d'être actif sur le Glucose, et aussi de mettre à jour
> le GitHub »*.

## 1. Les deux gels du relais

**16 h 00 → 17 h 48 : « Claude in Chrome extension detected ».** La session attendait une réponse à
cette question avant même son premier message ; le ❤️ disait « au travail depuis 16:00 » pendant tout
ce temps, et l'écran posé dans `relais-bloque` à 16 h 37 n'a été lu qu'à 17 h 43. Sa note tapée par le
veilleur l'a fermée (« No, keep browser tools off »). **La cause, lue dans le code de Claude Code
2.1.330** : l'offre ne se pose que si l'option `--chrome/--no-chrome` est absente **et** que le réglage
`claudeInChromeDefaultEnabled` du compte n'a jamais été défini (`xss()` : `w$e()!==!1 &&
…claudeInChromeDefaultEnabled===void 0`). Le compte a y avait répondu un jour, le compte b jamais.

* **RELAIS-CHROME** : le lancement passe `--no-chrome`.
* **RELAIS-FIGEE** : figée (35 minutes sans un changement d'écran), une session ne se dit plus « au
  travail » dans le ❤️ : « 🆘 figée, rien ne bouge depuis HH:MM » (`loi.dire_session`).

**18 h 00 → 18 h 43 : une permission du mode auto.** La session voulait regarder une image de
`~/.cache/glucose-bancs/` (hors du dossier de travail) ; le mode auto a demandé la permission, personne
n'était là. C'était la question posée à 15 h 25 (`claude-question-permissions`) ; **sa réponse** : *« on
fait tout en mode bypass, jamais de mode auto !!!!! Il faut que Claude ne soit JAMAIS bloqué »*.

* **RELAIS-BYPASS** : `--permission-mode bypassPermissions --allow-dangerously-skip-permissions`, comme
  son VS Code. **Pas de troisième gel par la question du bypass** : Claude Code ne la pose pas quand les
  réglages portent `skipDangerousModePermissionPrompt` (lu dans son code : le dialogue
  `BypassPermissionsModeDialog` est sauté si `zde()`), et les siens l'ont.

**Deux processus sur une conversation.** Sa session VS Code reprenait la conversation du relais pendant
que la copie figée vivait dans tmux. Je l'ai fermée (une seule session). Le veilleur avait tapé la
demande de passation dans le menu de permission **douze secondes avant** : son « Entrée » pouvait
accepter « toujours autoriser » à sa place. Rien n'a été accepté (`settings.json` daté de 14 h 12).

**La leçon** : chaque question que Claude Code pose au démarrage fige une session que personne ne
regarde. Ce qui la règle n'est pas la réponse gardée par un compte (l'autre ne l'a pas), mais **l'option
donnée à chaque lancement**.

## 2. RELAIS-CONTEXTE — la mémoire du modèle, et 90 % partout

Sa note (`text-kpll-351`) : *« on ne regarde jamais la limite de tokens d'un modèle […] on vise
toujours 90 % […] et on switch, ou on recrée un nouveau chat »*. Claude Code donne à la barre d'état
`context_window.used_percentage` (sa documentation et son code : `pgt()`).

* La barre d'état d'une **session du relais** (`RELAIS=1`) l'écrit dans `contexte.json` ; celle d'une
  session à lui, jamais.
* `loi.contexte_plein` : 90 %, et seulement une lecture faite **après** le lancement de cette session.
* Le veilleur (`garder` → « contexte ») et le crochet « Stop » demandent la passation ; écrite, la
  session se ferme et une conversation neuve est lancée (l'autre compte s'il est libre, sinon le même).
* La semaine passe la main à **90 %**, comme les cinq heures (elle était à 95).

## 3. RELAIS-TEXTES — ses textes dans son Glucose

Sa note : *« voir le script de passation envoyé à l'ouverture d'une session […] il ne me convient pas du
tout ; de même pour le script de stop »*. Les trois textes — **au démarrage**, **la passation**, **la
relance** — vivent dans trois nœuds de sa membrane « relais » (`relais-texte-depart`,
`relais-texte-passation`, `relais-texte-relance`), chacun dans une membrane titrée, sous une explication
(`relais-textes-explication`). Ils y ont été posés **depuis le code** (`loi.TEXTES`), mot pour mot.

* Le relais envoie **sa** version dès que sa main l'a écrite en dernier (`signee`, généralisée de la
  consigne aux textes) ; changée par une IA ou retirée, celle d'origine revient.
* Ce qui change d'une fois à l'autre s'ajoute seul : le compte, ses notes à lire, le motif de la
  passation, sa consigne.

## 4. FLECHE-7 — une flèche vers une membrane va droit à son bord

Sa note : *« pour relier un texte à une membrane, la flèche fait absolument n'importe quoi »*, avec sa
capture : la flèche entrait dans la membrane, la traversait jusqu'au bord gauche, et revenait se poser
sur le bord droit. **La cause** : l'itinéraire (FLECHE-5) va du centre de la source au **centre** de la
cible et contourne tout ce que l'index range sur le chemin — vers une membrane, les nœuds qu'elle porte ;
le trait est coupé au bord ensuite.

**La loi** : *ce que contient un bout de la flèche n'est pas un obstacle* (`arrow::contourner`, filtre
de la requête ; `Rect::contains_rect`). Le tracé s'arrête au bord, rien de l'intérieur ne se voit. Elle
ne change pas d'un tronçon à l'autre : le problème reste le plus court chemin entre des boîtes, et la
recherche paresseuse reste juste (`contour`, « pourquoi la source et la cible ne sont pas des
obstacles »). Un nœud **à cheval** sur le bord se contourne encore. Une photo qui porte une carte en
profite aussi.

* Trois épreuves (`contour_membrane_tests.rs`) : vers une membrane, droit au bord gauche — **avec sa
  preuve à l'envers** (l'ancienne loi faisait un détour sur la même scène) ; depuis une membrane ; un
  nœud à cheval sur le bord contourné. **Ma première scène ne reproduisait pas son bogue** : le nœud
  « dedans » couvrait le centre de la membrane, et un obstacle qui contient un bout ne se contournait
  déjà pas — c'est la preuve à l'envers qui l'a dit, en tombant.
* `fleche-134` : **5 sabotages sur 5**.

## 5. La sélection (Q12) — A, B, C dans son Glucose

Sa lettre sous l'image 0 : *« j'ai pas compris ce que je dois faire ? ce que je dois juger ? et où sont
les 3 propositions »* — la session d'avant s'était arrêtée avant de les dessiner. Le prototype
(`~/.cache/glucose-bancs/selection-q12/`, compilé dans une cible à part, `cible-q12`) a montré **deux
défauts à l'image**, corrigés avant de lui montrer :

* le cadre de la photo décalé d'une demi-photo : `x, y` d'une photo est son centre (`BoardImage::bounds`) ;
* dans C, le voile qui retombait là où deux trous se chevauchent (la règle pair-impair) : un masque, l'union.
* Et le fil de A passé de 1,5 à **2 pixels** (WCAG 2.2, 2.4.13, cité à la fiche 133).

Les quatre images sont dans sa membrane `relais-q12-images` (`relais-q12-a`, `-b`, `-c`), chacune avec
« ✍️ Ta lettre », et l'explication réécrite (`relais-q12-explique-1800`). `proto.patch` et
`selection_essai.rs` mis à jour dans le dossier du banc.

**L'image D** (le relais, 20 h 42 — sa lettre B, *« et du flou »*) : les équerres de B, et **le reste du
canevas flouté** — réduit au quart, trois flous de boîte (une gaussienne à peu de frais), agrandi,
posé hors de l'union des formes choisies (`GLUCOSE_ESSAI_SELECTION=d`). Posée dans sa membrane
(`relais-q12-d`, sa lettre sous elle), prise par `prendre-d.sh` (un cran dézoomé, pour que le texte
témoin se voie flouté). **Ce que le prototype triche** : il force la voie du processeur
(`la_carte_pose_les_photos` rend faux sous `d`) — par la carte graphique, le calque du dessus n'a pas les
pixels du canevas. Dans `main`, le flou vivra **sur la carte graphique** (Kawase double, une passe
de plus sur la texture de la scène), et sur la voie du processeur par la même réduction au quart ;
il s'efface pendant un glisser. L'arbre du prototype se refait : `git worktree add --detach
~/.cache/glucose-bancs/q12-arbre HEAD`, `git apply proto.patch`, recopier `selection_essai.rs`,
compiler avec `CARGO_TARGET_DIR=~/.cache/glucose-bancs/cible-q12`.

## 6. La 2.0.4-beta.5 — le brouillon est prêt

La vérification de GitHub de `378239d` : **verte sur les dix tâches**. Le brouillon : `gh workflow run
publier.yml -f version=2.0.4-beta.5 -f brouillon=true` (run 45221747706, sur `fcdd59f` — le même code :
seuls les docs et le relais diffèrent), **ses quatre tâches vertes** ; `verifier_notes.py` et
`verifier_serveur.py` joués ici d'abord. **Le clic « Publish release » est le sien.** FLECHE-7 et le
relais d'aujourd'hui n'y sont pas : la version d'après.

## 7. Vérifié

* Le relais : **69 épreuves** ; `relais-134` **20 sabotages sur 20** ; `relais`, `relais-131`,
  `relais-132`, `relais-133` rejoués — cinq entrées suivaient du code déplacé, mises à jour, aucune ne
  passe. Le veilleur relancé sur le code neuf, son premier tour lu (« en retrait », ses textes lus).
* `opacite-133` : le 27ᵉ sabotage, tué en plein jeu à 15 h 51 (le fichier `roue/opacite.rs` était resté
  **saboté** dans la copie de travail), rétabli puis rejoué : **27 sur 27**.
* FLECHE-7 : ses 3 épreuves, 5 sabotages ; Clippy strict à zéro ; **la suite entière : 3 532 épreuves, aucune tombée** (le pilote éteint).

## 8. Vu, pas réglé

* ~~Ranger des nœuds dans un dossier existant n'existe pas~~ — fait plus tard dans la session (§ 10).
* **Entrer dans un dossier ne cadre pas ce qu'il contient** : la caméra de son tableau est la sienne,
  et le texte rangé n'est pas à l'écran (vu au pilote, la mini-carte le montrait ; `Alt+F` le cadre).
  Pour l'archive, qui se remplit par la droite, c'est un piège : cadrer à l'entrée quand rien de ce
  qu'il porte n'est visible.
* **Son Glucose ouvert tourne sur un binaire d'avant MCP-DOSSIER** : la porte ne montre pas ses dossiers
  (`folder-wiov-332`, son futur « archive », est invisible au relais). Le relancer couperait l'accès tant
  que « Toujours, dans ce document » n'est pas donné (ACCÈS-H24).
* Les prototypes regardés hors du dossier de travail : en bypass, plus de permission à demander ; mais
  une session du relais sans garde-fou peut tout lancer, des heures sans personne — c'est son choix, dit
  une fois ici.

## 9. Choix pris seul, à contester

* **À 90 % du contexte, l'autre compte d'abord** (comme pour une jauge) plutôt qu'une conversation neuve
  sur le même : l'usage se répartit ; `choisir` reprend le même compte si l'autre est au seuil.
* **Ses textes n'ont pas de titre dedans** : le texte du nœud est le message entier ; le titre est la
  membrane autour. Il peut ainsi tout récrire sans casser une forme.
* FLECHE-7 ne compte plus comme obstacle ce qu'une **photo** porte quand une flèche la vise : le trait
  s'arrête à son bord, rien de dedans ne se voit.

## 10. RANGER-1 et 2 — ranger dans un dossier, l'archive

Ses notes du soir : *« archive les bugs une fois qu'ils sont corrigés »*, *« archive les textes ainsi
que les réponses une fois que c'est testé et déjà approuvé »*, *« éviter de supprimer, mais plutôt
archiver »* — ses flèches pointaient le dossier qu'il renommera « archive » le 24/11.

**La recherche** (consultée le 23/11/2026) : tldraw retire le lien d'une flèche dont un bout change de
page — elle devient libre, son sens perdu ([bindings](https://tldraw.dev/sdk-features/bindings),
[pages](https://tldraw.dev/sdk-features/pages)) ; Figma n'en dit rien d'officiel (des rapports de
connecteurs détachés sur son forum).

**La loi** (`store/ranger.rs`, RANGER-1, `662da67`) : ce qui part quitte le tableau pour celui que le
dossier ouvre, **à droite de ce qu'il porte déjà** (`place_pour_un_agent` : une seule loi de
placement), sa disposition gardée ; une membrane emporte ses membres, et les leurs ; un membre dont la
membrane reste arrive sans membrane. **Les flèches** : tous les bouts partent, elle part ; **un seul
bout part, elle reste dehors et vise le dossier** — le dossier tient lieu de ce qu'il contient, rien
ne pend, le lien garde son sens ; d'un nœud rangé vers le dossier, elle est arrivée et part avec le
geste. Un seul geste, que `Ctrl+Z` défait (les cases du journal du dernier au premier, JRN-2).
`glucose_ranger` (et `glucose ranger`) pour les IA, ouvert au nuage qui écrit ; il rend le tableau du
dossier (`ouvre`) — par l'API du noyau : la première version lisait les dossiers du modèle en direct,
et le cliquet du couplage l'a arrêtée (40 accès contre 38).

**Le geste à sa main** (RANGER-2, `f7c7420`) : pendant un glisser, le dossier sous le pointeur — celui
qu'un clic y prendrait, le même arbitre, jamais un dossier emporté — se dessine comme **le fantôme d'un
nœud à naître** (« ça arrivera ici », le langage de PLACEMENT-1) ; au lâcher, la sélection y entre dans
le même geste que le déplacement. `Échap` n'y range rien. L'état d'un glisser devient un seul champ,
`Glisser`, remis à zéro d'un coup (les deux fonctions qui passaient 80 lignes d'une ligne chacune ont
ainsi rendu la leur). **Aucun message** : j'en avais écrit un ; le cliquet des toasts (46) l'a refusé,
et il avait raison — la boîte annonce le rangement, le badge du dossier le compte.

**Regardé au pilote** (`~/.cache/glucose-bancs/ranger-134/`, une cible à part) : la boîte sur le
dossier pendant le glisser ; au lâcher, le texte parti, le badge à 1 ; `Ctrl+Z` le rend à (0, 0) ; dans
le dossier, le texte (après `Alt+F`, § 8).

**Vérifié** : 7 épreuves du noyau, 3 par la porte, 6 par la souris et la peinture — dont une qui a
d'abord mesuré son bruit : la barre du bas se redessine seule d'une image à l'autre, d'où deux épreuves
(la vraie peinture : la boîte apparaît sur le dossier ; un rendu direct : rien d'autre ne change) ;
`ranger-134` **13 sabotages sur 13**, `ranger2-134` **9 sur 9** ; Clippy strict, la suite entière :
**3 548 épreuves, aucune tombée**. Son Glucose (`target/release`) recompilé sur `f7c7420` : la relance
le lui donne (« Toujours, dans ce document » à donner alors, ⚡ Gestes).

**À contester** : une flèche vers le dossier depuis un nœud rangé disparaît (arrivée) — il les avait
tirées exprès ; `Ctrl+Z` les rend, mais un geste plus doux serait de la garder dans le dossier, libre.
La boîte du dépôt est discrète (1 pixel, le fantôme) : sa force est à son ressenti.


## 11. FLECHE-EDITION — les flèches d'un texte qu'on écrit partent du bord qu'on voit (23/11, 20 h)

**Sa note** (`text-kpll-364`, sa capture dans `membrane-299`) : *« toujours et encore le même problème :
les flèches déjà assignées à un texte qui est en mode édition, et donc en train de grandir, ne se
mettent pas à jour et passent par-dessus le texte »*.

**La cause, lue dans le code** : la hauteur d'un texte n'entre au document qu'à la validation
(`commit_editing` → `fit_text_card_height`) — un seul geste annulable pour toute la saisie. Le dessin
de la carte et sa lueur lisaient déjà la boîte **montrée** (`card::boite_montree`, fiche 100) ; les
flèches lisaient le document, donc l'ancien bord.

**La correction** : la boîte montrée du texte qu'on écrit est posée avant chaque image
(`Renderer::boite_vivante`, par `poser_la_boite_vivante`) et passe par `Contournement` — ce que le
rendu sait du geste en cours, la même porte qu'un glisser (GESTE-1). `NoeudsDuRendu::boite` la rend
pour ce texte : le dessin, le clic et les poignées lisent le même départ. Le cache des itinéraires
dépend déjà des bouts (FLECHE-5) : un bout qui bouge refait son chemin, rien d'autre.

**Écarté** : écrire la hauteur au document à chaque touche. C'est la loi d'un glisser, mais une saisie
dure des minutes : son geste ouvert engloberait ce qu'une IA écrit pendant ce temps (le relais écrit
dans son Glucose pendant qu'il tape), et chaque touche partirait au canevas partagé. `bloc()` (la
présence d'un lien près de la souris) garde la boîte du document : retouchée, elle n'avait pas
d'épreuve.

**Prouvé** : `renderer/arrow/edition_tests.rs` (huit lignes tapées : la flèche garde son écart de
naissance au bas qu'on voit ; validée, elle ne saute pas) ; trois sabotages
(`outils/sabotages/fleche-edition-134.json`).

**Fini par le relais** (compte a, 20 h 20-20 h 25) : la suite entière (3 547 épreuves) a fait tomber
**deux cliquets** — `peindre_ce_qui_a_change` à 82 lignes, et un accès direct aux annotations du
tableau (39 contre 38). Réparés (`f8d1043`) : le suivi d'avant l'image (le panneau Storyboard, la
palette, la date, la boîte vivante) devient `suivre_avant_l_image` ; la boîte vivante trouve son texte
par `Project::annotation`. Cliquets 16 sur 16, Clippy strict, **les trois sabotages rejoués dans
l'arbre détaché : 3 sur 3 tombent**. **Regardé au pilote** (`~/.cache/glucose-bancs/fleche-edition-134/`) :
deux textes reliés, huit lignes tapées dans celui d'où part la flèche — elle part du bas qu'on voit
(`2-pendant.png`), et ne saute pas à la validation (`3-apres.png`) ; **le binaire d'avant (19 h 31),
même saisie : la flèche traverse le texte** (`0-avant-pendant.png`) — sa capture, reproduite.

## 12. SELECTION-MEMBRANES — une membrane tracée prend ce qu'elle entoure (MEMB-9), et son bord ne se prend que là où il se voit (PICK-3) (23/11, 20 h 30)

**Sa note** (`text-kpll-356`, sa capture dans `membrane-299`) : *« j'ai eu beaucoup de mal à
sélectionner ce bloc texte, car trop de zones de clic, et la gestion d'intention est trop mal
faite ; du coup, pour éditer ce texte, j'ai dû vraiment galérer »*.

**Ce que son document montre** : le texte est entièrement dans un « Groupe » qu'il a tracé
(`membrane-kpll-362`, 935 × 217), mais n'en fait pas partie — comme les quatre autres « Groupe »
de sa membrane des bogues, tous vides.

**MEMB-9 — l'outil membrane prend ce qu'il entoure** (`3f54241`). `place_membrane` n'appelait
qu'`add_annotation`, qui donne à la neuve sa membrane d'accueil et rien d'autre ; « Grouper »
(GROUPER-1) et le dossier (FOLD-1) prenaient. Désormais `Store::adopter_ce_qu_elle_entoure` : les
sœurs de la neuve (les nœuds de sa membrane, ou libres comme elle) y passent quand **la loi du
dépôt** (MEMB-1, la plus petite membrane qui contient leur centre) les y range ; une sœur membrane
entre avec son contenu ; une appartenance écrite ailleurs ne se redéduit pas ; la pose et
l'adoption font un geste. Quatre épreuves (`appartenance_suite`, `memb_9`), une à la vraie souris
(`trace/tests.rs`) ; **six sabotages sur six** (`memb-9-134.json`).

**Choix pris seul, à contester** : le centre, et non « entièrement dedans ». tldraw (« the frame
tool reparents any sibling shapes that fall fully inside the frame's bounds »,
[Frame shape](https://tldraw.dev/sdk-features/frame-shape)) et Sketch
([Frames](https://go.sketch.com/docs/designing/frames/) : « if you draw a new frame around existing
layers, these will be reparented ») prennent ce qui est entièrement dedans. Glucose a déjà **une**
loi pour « être dans » : le centre — le dépôt d'un nœud (MEMB-1) et l'emprise d'un dossier
(`capture_content_under`). Trois gestes, une loi : l'appartenance qu'un nœud aurait si on l'y
lâchait (`test_memb_9_une_membrane_et_un_dossier_au_meme_endroit_prennent_les_memes`). Le prix : un
nœud qui déborde de moitié entre et dépasse du cadre.

**PICK-3 — le bord d'une membrane ne se prend que là où il se voit** (la cause probable de son clic
difficile, vérifiée sur sa géométrie). Le bord d'une membrane se saisit dans une bande de 14 px de
l'écran, au rang 10 — devant un texte (40). Son texte est à 27 unités du haut du « Groupe » : à
0,25, la bande en couvrait un tiers ; à 0,15, les quatre cinquièmes ; à 0,12, la membrane mesurait
moins de 28 px et n'était plus qu'un bord (`on_rect_edge`). Le clic prenait la membrane, le
double-clic aussi. **La loi** (`le_bord_cache_passe_apres`, `hit_priority/candidates.rs`) : une
membrane se peint **sous** ce qu'elle porte ; là où une photo, un texte ou une note couvre le point,
c'est eux qu'on voit (PICK-2), et le bord passe après eux, au rang du corps ; découvert, il garde son
rang. Le re-clic y descend toujours. C'est la règle de Figma et de tldraw (on prend l'enfant ; le
cadre, par son nom ou son vide). Épreuve neuve sur **sa géométrie exacte**, à 1, 0,3, 0,15 et 0,12,
avec l'envers (`test_pick_3_un_texte_pres_du_bord_d_un_groupe_se_prend_meme_dezoome`) ; deux
épreuves anciennes réécrites, parce que la loi neuve les contredit (MÉTHODE § 4.7) : les pointillés
couverts par deux photos, et le cycle des re-clics (texte, photo, puis le bord). Trois sabotages
(`pick-3-134.json`) — **le deuxième passait** (« le bord passe après tout, même découvert ») : rien ne
mettait un bord découvert en concurrence ; une flèche croise désormais les pointillés dans l'épreuve
(`ed86230`), **trois sur trois tombent**. **Regardé au pilote** (`~/.cache/glucose-bancs/pick-3-134/`) :
sa scène vue à 0,17 (le texte fait 41 × 12 px), un clic le prend, un double-clic l'ouvre, on y écrit ;
**le binaire d'avant, même clic : le « Groupe »**. Ses cinq « Groupe » vides restent vides : MEMB-9 ne
vaut que pour une membrane neuve ; la question est dans son Glucose. **Pas touché** : le bord d'un **dossier**, qui n'est pas peint sous ce qui le
recouvre de la même façon ; et le rang exact du bord couvert (le corps, 60, plutôt qu'un rang
entre la photo et le corps) n'a pas d'épreuve qui le distingue.
