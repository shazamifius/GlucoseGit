# 139 — L'arbitre sans rival, et l'IA qui déplace comme la main

> Le 24/11/2026, de 7 h à 8 h, session du relais (compte b). Sa consigne :
> *« fixer TOUS les problèmes de Glucose […] rendre moins brouillon toute la carte Glucose […]
> archiver le plus possible »*. La passation donnait : la suite de l'arbitre (fiche 138 § 3), puis
> ses réponses sous l'image E (aucune encore), puis l'archive.

## 1. ARBITRE-6 — une seule carte, pas d'arbitre

**Ce qui se passait** : son poste n'a qu'une carte visible sous Vulkan, la RTX (fiche 137 § 5).
« Économe » et « rapide » y ouvrent la même ; l'arbitre s'installait pourtant, et sa sortie du 23/11
disait *« cette carte gèle — la carte rapide s'ouvrira au prochain lancement »* : il retenait
« l'autre », qui était la même. Les téléphones, les Mac et la plupart des PC de bureau sont dans ce
cas. ECRAN-1 le savait sous Windows (DXGI range les cartes par préférence) ; ailleurs, rien.

**La loi** : au lancement, on demande **à `wgpu`** quelle carte ouvrirait l'autre préférence — la
même instance, la même surface (`ouverture::deux_cartes`). On appelle sa loi au lieu de la récrire :
`wgpu-core` 30 trie les adaptateurs par sorte (intégrée d'abord pour l'économe, dédiée d'abord pour
la rapide, `instance.rs`, `request_adapter`). Deux cartes sont la même quand leur `AdapterInfo` l'est
— la place sur le bus comprise, ce qui distingue deux cartes identiques. Aucune carte pour l'autre
préférence : il n'y en a qu'une. `Presenter::deux_cartes` le dit (faux sans carte graphique : la voie
du processeur n'a rien à choisir), et `faut_il_un_arbitre(deux_cartes, imposée, écran)` ne l'installe
que s'il y a deux cartes et que personne n'a tranché.

**Prouvé** : `test_arbitre_6_une_seule_carte_ne_laisse_rien_a_arbitrer` (la loi d'avant
l'installait) ; **au pilote**, dans une maison qui garde le souvenir « rapide » comme la sienne : le
binaire neuf dit *« arbitre : une seule carte graphique, rien a arbitrer »*, celui d'avant, sur le
même souvenir, *« la carte rapide a ete retenue lors d'une session precedente »* et installait
l'arbitre (`~/.cache/glucose-bancs/arbitre-139/`). Le coût : une demande d'adaptateur de plus au
lancement, sans périphérique ouvert.

**Laissé, avec ses raisons** : **ECRAN-1 sous Linux** (`/sys/class/drm`, la sortie du moniteur de
la fenêtre). Il ne sert qu'aux portables hybrides sous Linux ; aucun de ses utilisateurs connus n'en
a (son poste et l'autre Linux à RTX sont des tours à une carte, son VivoBook n'a que l'Intel), et
sans un tel portable on l'écrirait à l'aveugle. Sous Wayland, le nom d'une sortie (`DP-1`) est celui
du connecteur DRM, mais deux cartes peuvent porter chacune un `DP-1` : la lecture devrait alors se
taire. À reprendre le jour où une machine hybride sous Linux apparaît dans la télémétrie.

## 2. ARBITRE-5, par l'application

La fiche 138 § 3 le notait : l'appel `battre` dans `arbitrer_la_carte` n'avait aucune épreuve.
`test_arbitre_5_l_application_dit_le_battement_de_son_ecran` : l'application, écran à 60 Hz, deux
cents images d'échauffement puis six cents présentations à 15 ms (sa session du 23/11) — rien n'est
retenu, l'arbitre veille. Sans l'appel, il ne connaîtrait que le plancher de 10 ms.
`arbitre-139.json` : **2 sabotages sur 2**.

## 3. QUEUE-1 hors de l'horloge

`test_queue_1_la_cascade_compte_la_fin_de_l_image` tombait parfois sous la suite entière (la
passation le notait). **La cause** : la cascade lit la vraie horloge (`Instant`), et l'épreuve lui
donnait 5 ms ; sous la charge de la suite, le premier téléversement dépassait à lui seul ce budget,
et la seconde texture attendait aussi « sans la fin apprise ». La loi ne dépend pas de l'échelle des
durées : **la même scène en secondes** (une fin de 4 et 8 s, un budget de 5 s) garde son sens, et le
bruit de la machine n'y pèse plus. Les quatre sabotages de `queue-110.json` tombent toujours ; la
suite entière : **3 593 épreuves, aucune tombée** (MÉTHODE § 4, principe 4 : une épreuve ne croit pas
l'horloge).

## 4. DEPLACER-1 — une IA déplace comme la main

**Vu en rangeant son archive** (§ 5) : `glucose_deplacer` sur une membrane ne bougeait qu'elle — les
nœuds d'un « Groupe » sont restés à 4 000 unités de leur membrane. Sous sa main, une membrane emporte
ce qu'elle porte (MEMB-1) ; une IA doit pouvoir faire la même chose de la même façon (VISION § 1 : lu
et manié *« par un humain ET par une IA, de quasi la même façon »*). Deux trous au même endroit : un
dossier — qu'une IA voit depuis MCP-DOSSIER — était « inconnu » à qui voulait le déplacer ; une photo
verrouillée, que sa main ne bouge pas, bougeait sous une IA.

**La recherche** (consultée le 24/11/2026) : chez tldraw, déplacer un parent emporte ses enfants, et
toutes les transformations de l'API respectent la parenté
([parenting](https://tldraw.dev/sdk-features/parenting), [frame](https://tldraw.dev/sdk-features/frame-shape)) ;
chez Miro, des utilisateurs se plaignent d'un cadre qui n'emporte pas ses cadres imbriqués
([forum](https://community.miro.com/ask-the-community-45/frames-within-frames-container-frame-doesn-t-move-nested-children-frames-20785)).

**La loi** : le nœud devient la sélection le temps du geste (`geste_d_agent` rend la sienne ensuite,
AGENT-2), et c'est **la translation du glisser** qui bouge (`move_selected`,
`translation_de_la_selection`) : ce qu'une membrane emporte, les bouts des flèches qui suivent, un
dossier ; puis le dépôt (MEMB-1). Une photo verrouillée est refusée, et le refus le dit
(`Refus::Verrouillee`) au lieu de répondre « fait ». La description de l'outil le dit aux IA.

**Prouvé** : `deplacer_1_une_membrane_emporte_ce_qu_elle_porte` (le membre suit, ce qui est dehors
reste, un seul `Ctrl+Z` ; à l'envers, la loi d'avant laissait le membre) et
`deplacer_1_un_dossier_bouge_une_photo_verrouillee_non` ; `deplacer-139.json` : **3 sabotages sur 3**.

## 5. Son Glucose : l'archive, et ce qui était périmé

* **Les deux dossiers qu'il avait étiquetés « archive » portent ce nom** (sa note `text-f5fn-365` :
  *« demain et uniquement demain le 24/11 […] on pourra renommer archive juste ici »* — les dossiers
  se renomment par le MCP depuis MCP-DOSSIER, publié dans la beta.5) : celui du centre, où ses trois
  règles d'archivage pointent et où vivent les archives du 19-20/11, et celui du relais.
* **Rangé dans l'archive du centre** (une membrane « 🗄️ Archive · Réglé les 23 et 24/11 ») : le groupe
  des émojis, qu'il avait demandé d'archiver une fois le MCP corrigé (`text-kpll-360`), et notre
  réponse ; la question de la beta.5 (publiée) ; Q12 et nos deux réponses (tranchée, construite — la
  discussion continue sous l'image E, qui reste). **Dans celle du relais** (« 🗄️ Archive · Le relais
  réparé, le 23/11 ») : le gel « Chrome » et ses deux réponses, la question des permissions et sa
  réponse, nos trois anciens points d'étape.
* **Laissé en place** : ses bogues corrigés sur `main` qu'il n'a pas encore essayés — sa règle la plus
  récente (`text-f5fn-373`) archive *« une fois que c'est testé et déjà approuvé »* : ils attendent
  son essai, et sa parole y répond déjà — aucune question à poser.
* **Périmé, corrigé** : la liste de ses gestes lui demandait encore de publier la beta.5 — **publiée
  par son clic le 23/11 à 19 h 39** (`gh release view v2.0.4-beta.5` : `publishedAt`
  18:39:46 UTC), ce qu'ÉTAT et SUITE disaient encore « en brouillon ». Les trois sessions d'après ne
  l'avaient pas vu : une ligne d'ÉTAT qui parle d'un geste attendu se relit contre GitHub.

## 6. À contester

* **Deux dossiers « archive »** : ses deux étiquettes disaient ce mot. Une IA les distingue par leur
  place ; « archive du relais » pour le second serait plus clair — à sa parole.
* `deux_cartes` n'a pas d'épreuve qui ouvre une vraie surface : elle demande une fenêtre. Le pilote la
  regarde ; un sabotage de sa comparaison ne ferait tomber aucune épreuve automatique.
* `glucose_ranger` garde les écarts d'origine des nœuds rangés : venus de coins éloignés du tableau,
  ils arrivent éparpillés dans le dossier (le 24/11 : sur 5 700 unités). Les rapprocher en arrivant,
  en gardant la disposition de chaque groupe, serait plus propre.
* **ARROW-4 n'est raconté nulle part** (`c0d093a`, le 23/11 à 0 h 11, dans la beta.5) : son code
  cite la fiche 130, qui n'en dit rien. C'est la réponse à sa note `text-312` (*« impossible de placer
  une flèche […] à l'intérieur d'une membrane »*) : une membrane ne se vise plus que par sa frontière.
  Réponse posée dans son Glucose le 24/11 (`relais-rep-312`). **Reste** : à moins de 120 unités du
  bord, la frontière attire encore — dans un petit « Groupe », presque partout.

## 7. Ses deux autres notes sans réponse (24/11)

`text-321` (*« une IA peut écrire des trucs DANS la Time Machine, endroit que l'utilisateur ne peut
tout simplement pas accéder […] ça peut être dangereux »*) et `demande-codes-actifs` (voir et
supprimer les codes de salle et les accès) n'avaient ni réponse ni trace dans les documents. Une
proposition chacune dans son Glucose (`relais-rep-321`, `relais-rep-codes`), en attente de sa
parole : la première touche au format de l'histoire, que sa beta.5 doit pouvoir relire ; la
seconde, à ce qu'il veut voir réuni.

## 8. Les GitHub Pages, retirées

Sa réponse sous `q-pages` (lue le 23/11, VISION § 3) : *« ah oui à 100 %, il faudrait faire ça »* — à
la question « enlever l'ancienne version web de Tauri ? ». Le 24/11 vers 8 h :
`gh api -X DELETE repos/shazamifius/GlucoseGit/pages` ; `https://shazamifius.github.io/GlucoseGit/`
répond 404, et aucun workflow de `main` ne la redéploie. **L'environnement `github-pages` est gardé**
(son historique) : réactiver les Pages rendrait le site, et ce qu'un visiteur gardait dans son
navigateur (la même origine). Dit dans son Glucose (`relais-ou-on-en-est`).

