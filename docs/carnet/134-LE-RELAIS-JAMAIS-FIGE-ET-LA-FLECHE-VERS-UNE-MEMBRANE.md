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

* **Ranger des nœuds dans un dossier existant n'existe pas** — ni pour lui (glisser sur un dossier), ni
  pour une IA. Seul « créer un dossier avec ce qu'il couvre » existe (`create_folder`,
  `capture_content_under`). Ses notes d'archive le demandent (RANGER-1, SUITE § 2) ; la question des
  flèches qui relient un nœud rangé à un nœud resté dehors est à trancher d'abord.
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
