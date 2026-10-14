# 67 — La lecture du storyboard : le schéma 4, l'animatique, les champs et le plan

> Session du 14/10/2026 au soir (après la fiche 66). Le téléphone n'était pas branché : P0
> attend toujours sa lecture par le câble. Le programme a donc avancé sur **P2, propositions 4,
> 5 et 6**, en un seul schéma. Arrêtée à sa demande (« la session devient trop longue »).

## 1. Ce que la recherche a changé à la conception de la fiche 66 § 7 bis

* **La durée par défaut est une donnée de la scène** : Storyboarder la garde dans son fichier
  (`scene.defaultBoardTiming`) et prend `board.duration ?? scene.defaultBoardTiming`
  (`src/js/models/board.js`, `boardDuration`). Son propre code se contredit : sa frise
  (`main-window.js`, `renderSceneTimeline`) prend `0` pour « par défaut », son modèle non. Ici,
  **une seule loi** (`Storyboard::duree`), et 2 000 ms n'est qu'une valeur de naissance,
  réglable pour tout le storyboard. Storyboarder suggère aussi une durée tirée du dialogue
  (`durationOfWords(dialogue, 300) + 300`) : non retenu, c'est une heuristique.
* **Les champs ne sont pas figés à trois** : Toon Boom (Dialogue, Action Notes, Slugging, Notes —
  « réglés pour le projet ; un champ ajouté apparaît vide dans chaque case, en retirer un
  retire son texte de toutes ») et Krita (des commentaires qu'on nomme) les définissent pour
  tout le projet. La liste des noms devient une **donnée du storyboard** (Dialogue, Action,
  Notes à la naissance — ceux de Storyboarder) : la constante disparaît.
* **Un schéma par livraison, pas par proposition** : tant qu'aucune build ne l'a porté chez lui,
  le schéma 4 peut encore accueillir la proposition 8 (plusieurs storyboards). Deux schémas
  qu'aucune version n'a portés obligeraient Glucose à lire un format intermédiaire pour toujours.
* **L'animatique** : Boords règle la durée totale ou étire chaque case sur une frise ; Krita et
  Storyboarder ont un lecteur à part. Ici, **le canevas lui-même**, cadré case après case.

Sources : [Storyboarder](https://github.com/wonderunit/storyboarder) (`board.js`,
`main-window.js`, `utils/index.js`), [Toon Boom — caption fields](https://docs.toonboom.com/help/storyboard-pro-24/storyboard/caption/about-default-panel-caption.html),
[Boords — animatics](https://boords.com/docs/animatics),
[Krita — Storyboard Docker](https://docs.krita.org/en/reference_manual/dockers/storyboard_docker.html).

## 2. Le schéma 4 (`DOCUMENT_VERSION` = 4, `ANIMATIQUE_DEPUIS`)

* Une case (`StoryboardPanel`) porte `duree_ms: Option<u32>`, `plan: Option<Plan>`,
  `textes: Vec<String>` (au rang des champs, sans vide au bout : une forme unique).
* Le tableau porte un `Storyboard { grille, duree_ms, champs }` (au lieu de la grille seule).
* `Plan` : l'échelle française en huit plans, numérotés par leur rang (PERSIST-4).
* Le format vit dans `persist/storyboard.rs` (sorti de `document.rs`) ; l'histoire lit chaque
  geste à son propre schéma.
* **Témoins écrits par la build d'avant** (`ea8a690`, la dernière du v3, l'arbre de travail était
  propre) : `riche-v3.glucose` et `storyboard-v3.geste` — un document, et un geste de l'histoire
  qui pose un storyboard et décrit une case. Les deux se relisent.
* L'arène (fondation des 10⁷ nœuds) porte la lecture d'une case dans une table éparse
  (`CaseTrait`). **Un défaut ancien trouvé** : la table `crop` (RECADRAGE-1) n'était ni pesée ni
  vérifiée — la liste des tables était écrite deux fois à la main. Désormais **une seule liste**,
  `Doc::tables()`, tenue par une déstructuration exhaustive : une table oubliée ne compile pas.
  `check` est repassée sous 80 lignes (le cliquet a descendu).

## 3. Les gestes du magasin (`store/storyboard/case.rs`)

Écrire un texte (`Texte::Description` ou `Texte::Champ(i)`), régler la durée d'une case ou du
storyboard (au moins 1 ms), dire le plan, nommer, ajouter, retirer un champ (son texte part de
chaque case dans le même geste). Chacun est un `Ctrl+Z`. `update_panel` est retiré : une seule
écriture. Une case qui porte un plan, une durée ou un texte ne se retire jamais d'un tracé.

## 4. Le panneau, et les durées

* Le panneau est un **miroir** du document, rafraîchi à chaque image (`StoryboardState::suivre`) :
  le dessin, le clic et les champs lisent la même chose (L4).
* Section neuve : « ▶ Lire » et **DURÉE PAR CASE** ; puis **CASE n** : sa durée (et « ↺ » pour la
  rendre au storyboard), son plan (un sélecteur qui tourne, comme le format), sa description et
  chaque champ (toucher le texte l'écrit, toucher le nom renomme ou retire), « + Champ ».
* **Une durée se lit et se tape en secondes** (« 2,5 s », virgule ou point), se garde en ms.
* **Sa règle est logarithmique** : la première version, linéaire à 100 ms le pas, faisait passer
  une case de 2 à 14 s en 60 pixels (l'épreuve l'a montré). Une durée se perçoit en rapport :
  chaque pixel multiplie par `(max/min)^(1/largeur)`, l'arrondi est à deux chiffres significatifs.
* **Une faute trouvée par l'épreuve** : après avoir écrit, si `Ctrl+Z` arrivait avant l'image
  suivante, le champ gardait la valeur défaite (le miroir comparait au dernier état *vu*). Le
  panneau relit le document aussitôt après avoir écrit.

## 5. L'animatique (`interactions/lecture.rs`, `renderer/lecture.rs`)

La case remplit l'écran (marges du système ôtées), **tout le reste est voilé de noir**, ses textes
passent en sous-titre (description, puis « Nom : texte »), un compteur « 3 / 12 · Gros plan »,
une croix. Une **coupe** à chaque durée ; entre deux, la boucle dort (`Raison::Animation`).
Échap ou le retour du téléphone, ← →, Origine, Espace (pause) ; au doigt : tiers gauche recule,
le reste avance, la croix finit. L'interface est cachée par un prédicat unique,
`UiState::sans_interface()` (la référence ou la lecture), qui remplace sept `ui.reference`.
Les prises (ornements, « + », croix) ne se dessinent pas pendant la lecture. « Lire le
storyboard » est aussi au menu du vide (PC et téléphone). Le sigle du plan (PG, GP…) s'écrit
dans la bande de la case, après le numéro. **Regardé** : la lecture rendue hors écran, le
panneau dans `apercu_storyboard`.

## 6. Ce qui est prouvé, et ce qui ne l'est pas

* Toute la suite passait (2 497) avant la dernière correction du cliquet ; arène, cliquets,
  lecture, panneau relancés ensuite et verts ; clippy strict sous Windows propre.
  **Pas rejoué** : la suite entière après les derniers changements, clippy pour Android.
* **Sabotages** (`outils/sabotages/p2-4.json`, 46) : au moment de l'arrêt, les premiers donnaient
  TOMBE, **sauf deux aveugles** — « le storyboard écrit sans changement » et « une case écrite
  sans changement » (`store/storyboard/case.rs`, `regler` et `changer`) : le journal jette déjà
  les éditions nulles, ces gardes sont redondantes. **À trancher d'abord** à la session suivante
  (les retirer, selon la règle) ; le reste des sabotages est dans
  `sortie-sabotages-67.txt` s'il a fini (sinon : les relancer).
* **Rien n'est jugé à son écran.**

## 7. Les erreurs de la session

* Une règle de glisser choisie sur le papier (linéaire) pour une grandeur qui se perçoit en
  rapport.
* Un miroir qui compare au « dernier vu » : faux dès que le document fait A → B → A entre deux
  images.
* Quatre épreuves écrites aveugles (le glisser sans image entre deux mouvements, le réveil
  « ≤ 2 s », les prises en lecture, `Tab` dans un champ), vues en préparant les sabotages,
  renforcées avant de saboter.
* Encore un heredoc refusé par Git Bash (apostrophes) : tout script passe par un fichier.

## 8. À essayer à l'écran

`cargo run --release > sortie-67.txt 2>&1`, un storyboard posé :
choisir une case → le panneau montre CASE n ; régler sa durée (glisser, ou cliquer et taper
« 3,5 ») ; « ↺ » ; le plan qui tourne (son sigle apparaît sous la case) ; écrire un Dialogue,
renommer un champ, en ajouter un ; « ▶ Lire » : plein écran, sous-titres, coupe à chaque durée,
← →, Espace, Échap ; clic droit sur le vide → « Lire le storyboard ».
