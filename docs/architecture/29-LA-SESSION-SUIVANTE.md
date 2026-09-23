# 29 — La session suivante

> **Rôle de ce document.** C'est le dossier de travail de la prochaine session Claude sur
> **Glucose Rust**. Il ne dit pas quoi coder : il dit ce qui est su, ce qui ne l'est pas, ce que
> l'utilisateur a vu et dit, et **toutes les pistes** qu'on a — avec ce qui les appuie et ce qui
> les démentirait. Le travail, c'est toi qui le fais.
>
> Écrit le 23/09/2026 au soir, à la fin de la session de la fiche 28. Tête : `2603f1a` (vérifie).

---

## 0. Avant tout : remets TOUT en question

C'est la règle la plus importante de ce dépôt, et l'utilisateur la répète à chaque session.
Remets en question **le code, les fiches, ce dossier, et les paroles de l'utilisateur**. Il a
raison sur le *quoi* — ce qu'il voit à l'écran prime sur tous tes chiffres — et rarement sur le
*pourquoi* : cela ne veut pas dire qu'il se trompe, cela veut dire qu'il faut mesurer chez lui.

Ce dossier est écrit par la session qui vient de se tromper **trois fois** devant lui ce soir
(§ 3). Tout ce qu'il affirme est une hypothèse de travail, pas un fait. Quand une piste ci-dessous
te paraît fausse, elle l'est peut-être : dis pourquoi, et va voir.

Et une chose que la session précédente a apprise à ses dépens : **le plus gros danger n'est pas
de ne pas trouver, c'est de croire avoir trouvé.** « Probablement la cause, à confirmer » tant que
l'utilisateur ne l'a pas vu à l'écran.

---

## 1. Ce qu'il faut lire, et vraiment lire

* **Les mémoires Claude**, toutes, en entier (`~/.claude/projects/…/memory/`) : la charte, l'architecture
  adaptative, les sessions longues, la politique des tokens, les journaux par fichier. Et le
  `CLAUDE.md` global : il ne code jamais, tu exécutes tout, tu parles français — y compris dans
  tes réflexions visibles —, jamais de menus à choix multiples.
* **La fiche 28**, entière : ce que la dernière session a fait, démenti, laissé ouvert.
* **La fiche 27** : l'objectif que l'utilisateur appelle « ultime et final » (la meilleure qualité,
  l'origine, l'auteur d'une image).
* **La fiche 24** : la plus riche en méthode — et son § 13, qui annonçait il y a deux jours le
  défaut que l'utilisateur vient de montrer (§ 4.2 ici).
* Les fiches 26, 25, 23, 21, 20 à la demande, la fiche 05 (standards : R1 « rien à moitié »,
  80 lignes par fonction, 600 par fichier), et `tests/cliquets_suite.rs` : **ils ont raison quand
  ils protestent** ; on extrait, on ne relève jamais un plafond de taille.

L'état se vérifie : `git status`, `git log`, `git ls-remote https://github.com/shazamifius/GlucoseGit.git main`,
`cargo test --workspace` (1 509 verts au départ), `cargo clippy --workspace --all-targets -- -D warnings`.
Le token GitHub vient de l'utilisateur ; s'il manque ou s'il est refusé : « il me faut un token »,
une phrase, rien d'autre.

---

## 2. Ce que l'utilisateur a vu et dit, à la fin de la session du 23/09

Ses mots, à peine resserrés. C'est ta matière première.

* **`Ctrl+B` sur la forêt** : *« parfait — l'image elle-même a des contours légèrement blancs, la
  façon dont tu la vois est PARFAITEMENT crop »*.
* **`Ctrl+B` sur une gravure** (un paysage à la pointe sèche, nuages et forêt, signature au
  crayon en bas) : *« sur le côté droit, ce n'est pas tout à fait parfaitement crop »*.
* **Le dépôt Pinterest** : fait, avec `GLUCOSE_DEPOT=1`. Quatre épingles, quatre originaux
  rapatriés (§ 4.8).
* **La carte de texte neuve** : *« absolument parfait, tout fonctionne »*.
* **Trans-domaines** : *« impossible de désactiver, en plus ce n'était ABSOLUMENT PAS sa
  fonction. Touche pas Trans-domaines, voire tu peux retirer complètement ce que tu pensais que
  c'était, tu l'as très très mal compris »*.
* **De lui-même** :
  1. *« Lorsqu'on est trop proche d'un texte, celui-ci ne veut tout simplement pas s'afficher,
     c'est horrible. »* (capture : un aplat vert, la grille de points, plus rien de la carte)
  2. *« Les images apparaissent en vague, par cascade, c'est assez insupportable ; j'ai encore à
     peu près 30 fps, mais que ça apparaisse par cascade et que les images soient aussi mal
     optimisées, c'est assez complexe. »*
* **Deux longues sessions** qu'il a jouées pendant que la session précédente travaillait — donc
  **peut-être sans toutes ses modifications**. Il les dit *« SUPER intéressantes et trop
  importantes »*. Leurs chroniques ont été écrasées ; il en reste son résumé (§ 5).

Captures et données copiées dans le scratchpad de la session précédente — perdues pour toi. **Les
données de l'utilisateur vivent chez lui** : `%TEMP%\glucose-chronique\derniere-session.txt`
(écrasée à chaque session : copie-la dès qu'elle arrive), `%TEMP%\glucose_depose\` (les dépôts,
dont ~60 images rapatriées), ses documents `.glucose` (`Desktop\Blender\…` : `fuser.glucose`,
`random photo !!!!.glucose` ; en-tête de 16 octets, table de 56 octets par section, images en
nature 3 — Python et PIL les extraient en dix lignes).

---

## 3. Ce que la dernière session a mal fait, et qu'il faut défaire ou vérifier

### 3.1 Trans-domaines — à retirer, puis à redemander

La session a lu Glucose Tauri (`ArrowSvgLayer.tsx`) : une flèche « trans-domaine » y relie deux
nœuds sans domaine commun, se dessine en pointillés, et le bouton la masque. Elle l'a branché
(commit `bbc496b` : `arrow::est_trans_domaine`, pointillés, masquage au dessin et au clic).
**L'utilisateur dit que ce n'est pas du tout la fonction.** La référence Tauri n'était donc pas la
bonne source pour cette fonctionnalité — ou elle-même ne faisait pas ce qu'il voulait.

Et pire : **le bouton n'a jamais basculé.** Aucune ligne du code n'écrit `ui.trans_domain` :
`handle_ui_click` bascule l'Aimant, pas lui. Le toast disait « affichés » à chaque clic. Les tests
de la session posaient l'état **directement** — ils prouvaient le dessin, jamais le geste. C'est
la leçon de la fiche 20 § 5.4 refaite : *un test qui prouve une égalité ne prouve pas un choix* ;
un test qui ne passe pas par le clic ne prouve pas le bouton.

Il a dit « touche pas » et « tu peux retirer complètement ». Ce qu'il veut que Trans-domaines
soit est une **décision de produit** : c'est à lui de la dire, en prose. Réfléchis à ce que « trans-
domaines » peut vouloir dire dans sa vision (fiche mémoire : des millions de nœuds, des
**domaines** et la **couleur** pour naviguer dans la carte de toute la connaissance) avant de lui
poser la question — et ne présume pas de la réponse.

### 3.2 Ce que personne n'a vérifié à l'écran

* le **marqueur de téléchargement** (« Téléchargement depuis fr.pinterest.com… » au point de
  lâcher) : la sortie prouve que les quatre images sont arrivées, pas qu'il a vu le marqueur ;
* l'image qui se pose **là où on l'a lâchée même si la vue bouge** pendant l'attente ;
* le **liseré** sur d'autres images que la forêt et la gravure.

---

## 4. Les pistes

Chacune dit ce qu'on sait, ce qu'on croit, et où regarder. Aucune n'est une solution. L'ordre
n'est pas un ordre de priorité imposé : l'utilisateur a dit *« il ne faut surtout pas qu'on
s'attarde à tailler la pierre d'une cathédrale qui ne possède pas de porte d'entrée »* — les
fonctionnalités et ce qu'il voit passent avant la performance fine. Mais deux de ces pistes
(4.2, 4.3) sont **ce qu'il voit**, et le cap est la V1 publique.

### 4.1 Trans-domaines

Voir § 3.1. Défaire d'abord ; demander ensuite ; construire seulement après.

### 4.2 Le texte qui disparaît quand on s'en approche

La fiche 24 § 13 l'avait prédit sans pouvoir l'établir : `Regime::composer`
(`renderer/composants.rs`) **refuse** de faire une texture d'une carte plus grande que l'écran
(`pixels.0 > self.ecran.0 || …` → `None`), et la documentation promet qu'elle se dessine alors
« en direct, comme une photo en zoom proche ». Sur la voie graphique, **personne ne semble la
dessiner**. La capture de l'utilisateur le montre : la lueur de la carte, la grille à travers, et
rien d'autre. Un commentaire qui décrit une intention passe pour une description.

Questions : qui devrait la dessiner, et sur quelle voie ? Le refus est-il la bonne règle — une
texture plus grande que l'écran est absurde, mais la carte en zoom proche n'a besoin que de sa
partie visible ? Que fait la voie processeur au même zoom ? Le texte est rendu par `tiny-skia`
sur le processeur puis posé en texture (COMPOSANT-1) : à quel zoom et à quel coût se rend-il en
direct ? Un test d'aspect qui zoome une carte jusqu'à dépasser l'écran, sur les deux voies,
aurait attrapé ce défaut : pourquoi n'existait-il pas ?

### 4.3 Les images qui arrivent « en vagues », et 1,3 Go de mémoire

Ce qu'on sait du code :

* **La carte reçoit toujours la texture NATIVE** (`app/presentation.rs`, la source des textures :
  `e.pyramide.native().clone()`), quelle que soit la taille à l'écran. Une épingle rapatriée en
  original fait jusqu'à 2 297 × 3 062 pixels : **28 Mo par texture**, téléversés même pour une
  vignette de 200 pixels.
* Le téléversement est budgété (CASCADE-2, `present/scene_gpu.rs`, `assurer`) : ce qui ne tient
  pas dans le temps libre de l'image attend la suivante — **d'où les vagues**. Le poste
  `textures` monte à 27 ms dans ses sessions.
* Côté processeur, chaque image garde sa **pyramide entière** (native + niveaux ≈ 1,33 ×) en
  mémoire. Trente originaux Pinterest : l'ordre de grandeur du 1,3 Go qu'il a mesuré.
* La fiche 26 § 10.1 : sur une carte intégrée, ce que la carte détient vit dans la RAM du
  processus — la mémoire double.

La tension, et elle est vraie : fiche 27 veut **la meilleure qualité** d'une image — et la charte
veut *« des millions d'images sans laguer et sans occuper trop de place »*. Posséder l'original
n'oblige pas à le **montrer** en entier ni à le **tenir** en mémoire. `Recadrage::texels_lisibles`
prend déjà un facteur de réduction : la voie graphique pourrait recevoir le niveau qui couvre la
taille affichée — mais alors chaque changement de palier est un téléversement, et c'est
exactement le mécanisme qui fait les vagues. Des mipmaps sur la carte ? Un atlas ? Montrer
**tout de suite** un petit niveau (quelques kilo-octets, la « pixelisation assumée » de la charte)
puis affiner, plutôt qu'un cadre vide qui se remplit en vague ? Ce que font les visionneuses qui
tiennent des dizaines de milliers d'images (tuiles, niveaux, décodage progressif) mérite une vraie
recherche avant toute ligne.

Et une question que personne n'a mesurée : **combien coûte réellement** un téléversement de
28 Mo sur sa RTX, contre un de 1 Mo ? Le banc d'abord.

### 4.4 L'application qui ne dort pas

La courte session du dépôt (60 s) : *« processeur pendant qu'on ne le touche pas : 7,0 % d'un
cœur, et il dessine 13,4 images par seconde »* — alors que la même section dit *« aucune : chaque
image a été demandée par un geste »*. Ses longues sessions : 3,6 à 12,5 images par seconde au
repos, jusqu'à 8 % d'un cœur. La fiche 25 § 9.1 avait mesuré **0,1 image par seconde** au repos :
**quelque chose a régressé**, ou quelque chose n'était pas là à l'époque.

Où regarder : `app/reveil.rs`, `prochain_reveil` — curseur, toast, animation, élan, vol,
décodage, **chantier**, **pomodoro** ; le compteur `reveil_masque` existe déjà, il dit quelles
raisons étaient actives. Et l'instrument lui-même se contredit : pourquoi dit-il « aucune » ?

### 4.5 Les gels hors du rendu, et la session qui s'est fermée pendant un gel

Son résumé : `acquerir` à **293 ms au repos** ; « écouter la main » et « attendre Windows » qui
bloquent **103 à 195 ms** ; `composants` à 31,8 ms sur un zoom ; et la deuxième session qui
**s'est fermée pendant un gel** — *« une image était due depuis 104 ms et n'a jamais paru »*.

Rien de cela n'a de cause établie. « Écouter la main » à 100-200 ms, c'est un gestionnaire
d'événement qui travaille — un dépôt qui décode ? un `Ctrl+B` sur un lot ? un enregistrement ?
L'entracte (ENTRACTE-2) garde chaque gel **daté et décomposé** : la prochaine chronique d'une
longue session le dira, à condition de la **copier avant qu'elle soit écrasée**. Et le gel de
fermeture : est-ce un vrai gel, ou l'instrument qui compte l'instant de la fermeture elle-même ?
La leçon de la fiche 26 § 11 vaut ici : un accusé de réception n'est pas une livraison, et
inversement un instrument peut porter en tête ce qu'il devait corriger.

### 4.6 La cadence à 164-194 nœuds

Médiane vers 61 images par seconde (3 à 6 balayages sur un écran à 240 Hz), 17 à 26 % des images
au-dessus de 10 ms, du judder sur 16 à 21 % des images. `ornements` à 26 ms, `textures` à 27 ms
quand la vue bouge ou zoome. Et le verdict constant : **100 % des images redessinent tout**, 0 %
passent par le chemin le moins cher.

C'est la première fois qu'il joue avec **autant de nœuds**. Les fiches précédentes jouaient avec
21. Tout ce qui a été mesuré à 21 nœuds est à remesurer. `ornements` (fiche 24 § 14.3) n'a jamais
été instrumenté finement. Le « travail refait » est un constat du verdict depuis des sessions :
est-il vrai, ou mal compté ?

### 4.7 La gravure et son bord droit

Mesuré sur sa capture : au bord droit, 3-4 pixels d'écran de papier clair, précédés d'un dégradé
(34 → 80 → 144 → 186 → 216) vers la plaque. En bas, la bande de papier reste — la signature au
crayon l'arrête, et c'est assumé (« un filigrane dans la bande l'arrête à sa ligne »). Il ne s'est
plaint que du côté droit.

Le fichier n'a pas été retrouvé (ni dans les dépôts, ni dans ses documents du jour) : **demande-le
lui**, ou cherche mieux. Deux hypothèses, à départager sur le fichier et pas avant :
* le bord de la plaque est **adouci sur plusieurs rangs**, chacun sous le bruit — le cas que la
  fiche 28 § 2.3 a laissé ouvert, et que la règle « un fondu doux ne se mange pas » protège
  volontairement (pour les vignettages) ;
* la gravure a été **scannée légèrement de travers** : le bord de la plaque est oblique, aucune
  colonne n'est ni bande ni fondu, et un recadrage droit ne peut pas tout retirer sans rogner.

Sur la forêt, il a jugé l'adoucissement propre de la peinture « parfait » ; sur la gravure, le
papier restant le gêne. Qu'est-ce qui les sépare, mesurablement ? `bench_bordures` (chronométré),
`apercu_recadrage` (rend par le vrai code, avec et sans la fenêtre de lecture) et le portage
Python de la session précédente sont tes outils.

### 4.8 Pinterest — ce qui est réglé, et ce qui reste

Réglé, vérifié sur sa sortie : depuis la grille, le dépôt porte le lien de l'épingle et le
fragment HTML avec les quatre tailles de l'image ; depuis la page d'une épingle, **seulement** le
format `Chromium Web Custom MIME Data Format` avec l'adresse en 736, remontée à l'original. Quatre
sur quatre. Le « cas ouvert » des fiches 26 et 27 est fermé — à confirmer sur d'autres sites.

Reste : la seconde d'attente (le marqueur, § 3.2), et **la mémoire des originaux** (§ 4.3).

### 4.9 La recherche d'origine — fiche 27, étape 3

L'étape 2 est faite (`bench_empreinte`, fiche 28 § 5) : pHash distingue la même image (0 bit)
de deux images différentes (20 au plus proche), et le seuil **se calcule** depuis son « sûr à
99 % » : `N · P(Bin(64, ½) ≤ t) ≤ 1 %`. Aucun module n'existe encore : il n'aurait pas
d'appelant.

L'étape 3 attend **une clé SauceNAO** : la question lui a été posée, il n'a pas répondu. Ne
harcèle pas ; repose-la une fois, au bon moment, en prose. À chercher avant de construire : le
format JSON de l'API SauceNAO et ses limites, comment ArtStation et pixiv lient les autres comptes
d'un artiste, ce que X et Instagram ferment.

### 4.10 Le reste, en vrac, et chiffré

* **« Une image reste à l'écran : pire 522 ms »** encore, dans la session du dépôt, après la
  correction du sommeil des toasts. Quel autre réveil n'a pas d'échéance ?
* **`bande` à 20 ms** : une marque `reperes` sépare désormais ce qui suit `ornements` (fiche 28
  § 6). La prochaine chronique dira qui paie.
* **`docks` à 5-7 ms** quand un panneau se refait (sélection, document modifié) : légitime mais
  cher, deux panneaux à 150 %.
* **`Ctrl+B` sur un lot** : 1 à 2 ms par image mesurées seules ; jamais mesuré sur cinquante.
* **L'arbitre** qui alternerait son souvenir si deux cartes gelaient toutes deux.

---

## 5. Les deux longues sessions, telles qu'il les a résumées

À prendre comme un résumé, pas comme une chronique : les chiffres sont les siens.

| | session 1 | session 2 |
|---|---|---|
| durée, images | 798 s, 12 035 | 340 s, 6 784 |
| nœuds au plus | 164 | 194 |
| fin | normale | **pendant un gel** (104 ms dus, jamais parus) |

Machine : RTX 5070 Laptop, écran 240 Hz, `mailbox`, interface à 150 %. Médiane autour de
61 images par seconde. 17 à 26 % des images au-dessus de 10 ms. Judder sur 16 à 21 % des images ;
le contenu à 64-77 px de sa trajectoire au p99. `ornements` et `textures` jusqu'à 26-27 ms dès que
la vue bouge. `acquerir` à 292,99 ms au repos (session 1). Gels hors rendu de 103 à 195 ms.
`composants` à 31,81 ms sur un zoom. 100 % des images redessinent tout. Au repos : 3,6 à 12,5
images par seconde, jusqu'à 8 % d'un cœur. Mémoire : jusqu'à **1,3 Go** après une trentaine
d'images Pinterest glissées.

**Demande-lui une longue session de plus** — avec le document chargé de ses ~190 nœuds —, sortie
dans un fichier, et copie la chronique dès qu'elle arrive. Ce sont les données les plus
précieuses qu'il ait jamais produites, et elles sont perdues.

---

## 6. La méthode, en bref — le détail est dans les fiches

* **Un banc de plus plutôt qu'un raisonnement.** Ce soir, `bench_bande` a démenti en une
  exécution une hypothèse vieille de trois sessions.
* **Chaque correction vérifiée à l'envers** : remets l'ancien code, regarde le test tomber, note
  le chiffre. Et **un test doit passer par le geste** qu'il prétend prouver (§ 3.1).
* **Les données de l'utilisateur, tôt** : ses pixels, ses fichiers, ses documents. Ce soir, le
  liseré a été résolu en mesurant **sa capture**, pas son fichier : le fichier était propre à
  droite, l'écran non.
* **Regarde le résultat, pas le journal** : rends l'image, ouvre-la.
* **Une marque de mesure absorbe tout ce qui la précède.** Cinq fois dans ce dépôt, dont `bande`
  ce soir. Quand un poste est énorme et inexplicable, regarde d'abord la marque d'avant.
* **Aucune constante nouvelle** quand une grandeur existante suffit ; une constante qui peut
  disparaître disparaît. Ce soir, le seuil de l'empreinte n'a pas été choisi : il est sorti de la
  loi binomiale et du « 99 % » de l'utilisateur.
* **Cherche sur internet avant de concevoir** : il y tient beaucoup (*« c'est trop trop
  intéressant, sur les défis, sur nos objectifs »*). Ce soir : les limites de Nielsen pour
  l'attente, pHash et ses seuils.

Pièges pratiques, payés :
* sous Windows, Python écrit en **CRLF** par défaut : `open(p, 'w', encoding='utf-8',
  newline='\n')` ; le dépôt impose `eol=lf` ;
* écris les scripts d'édition dans un fichier, **vérifie toutes les assertions avant d'écrire le
  moindre fichier** ; un motif peut attraper un paramètre de fonction homonyme d'un champ ;
* les apostrophes françaises cassent les heredocs de bash : passe par un fichier ;
* **d'autres sessions peuvent travailler sur le même dépôt en même temps** (ce soir, une session
  sur la documentation). `git status` avant de committer, ajoute tes fichiers **un par un**, et
  coordonne-toi par message ;
* ne lance l'application toi-même que si c'est nécessaire, et vérifie qu'aucun processus
  `glucose-desktop` ne survit.

---

## 7. Ce qu'il attend de toi

* **Des sessions longues** : enchaîne, tranche seul la technique, explique tout en bloc à la fin.
  Ne t'arrête que pour un test à l'écran, un ressenti, ou une décision de produit (Trans-domaines
  en est une).
* **Parle-lui simplement**, en français. Il ne code pas ; « les images partaient et n'arrivaient
  pas à l'écran » l'aide plus qu'un nom de structure.
* **Une question à la fois, en prose**, jamais de menu à choix.
* **Il ne voit pas la console** : toute sortie passe par `cargo run --release > sortie-<nom>.txt 2>&1`
  à la racine du dépôt, et c'est toi qui lis le fichier. Donne-lui la commande exacte et ce qu'il
  doit faire à l'écran.
* Il est très perfectionniste, et il a raison de l'être. Mais **la porte d'entrée de la
  cathédrale d'abord** : la V1 publique, des retours, puis mieux.

---

## 8. Ce dont la session précédente n'est pas sûre — commence par là si tu veux la contredire

* **Le critère du fondu** (fiche 28 § 2.2) : « la majorité des pixels qui peuvent montrer un
  fondu, et leur médiane déplacée au-delà du bruit ». Il a passé 49 images et trois tests
  construits pour faire tomber ses prédécesseurs. Il n'a jamais vu une gravure scannée de travers.
* **La lecture bornée à la fenêtre** : exacte sur les deux voies en test, mais une image
  **tournée et cadrée** copie désormais sa fenêtre à chaque image sur la voie processeur — jamais
  mesuré.
* **Le seuil de l'empreinte** suppose que les distances entre images différentes suivent
  Bin(64, ½). Vrai sur 2 193 paires de **ses** images ; faux peut-être sur deux dessins d'un même
  artiste, qui se ressemblent davantage que deux images au hasard.
* **L'histogramme depuis l'échéance** : il reste un « pire 522 ms » que la règle n'explique pas.
* **Trans-domaines** : faux de bout en bout (§ 3.1).

---

**Retour** : [`00-INDEX.md`](00-INDEX.md)
