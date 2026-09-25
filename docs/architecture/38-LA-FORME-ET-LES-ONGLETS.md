# 38 — La forme, et les onglets

> **Rôle de ce document.** La session du 25/09 a d'abord lu **son second essai** (§ 1) : sa
> chronique accusait les membranes (23 ms par image), la couche du dessous envoyée entière, et
> le liseré de la Time Machine ; la reprise après plantage rendait le texte, pas le curseur.
> Tout cela est corrigé (§ 2 à § 4). Puis **les onglets**, selon sa réponse : se renommer, se
> ranger, se supprimer, et faire entrer un autre document dans un onglet neuf, par un geste à
> part de `Ctrl+O` (§ 5). Enfin, un seul traceur de rectangle arrondi, en vrais arcs de
> cercle, et une borne d'épreuve qui n'en était pas une (§ 6). **Les membranes s'arrêtent
> là** : il a demandé qu'on en parle avant de continuer (§ 8).
>
> **Date** : 2026-09-25 · commits `77c3e12` à `ef68e6a`, et celui de cette fiche.
> **État vérifié** : `cargo test --workspace` exit 0, **1 695 tests verts**, clippy strict à
> zéro, `cargo fmt --check` à zéro ; après chaque suite, `%LOCALAPPDATA%\Glucose` n'a rien
> reçu.
>
> **Vu à l'écran** : rien de cette session encore. Le § 10 dit quoi regarder.

---

## 0. En une page

| | ce qui change pour lui | commit |
|---|---|---|
| **Le dossier réel** | les épreuves déposaient des aperçus dans son dossier `Glucose` : plus rien n'y entre | `77c3e12` |
| **MEMB-FORME-1** | une membrane qui remplit l'écran : **18-24 → 3,2-3,7 ms** au processeur ; sur la voie graphique, la carte la peint | `1bc8309` |
| **BANDE-2** | la couche du dessous ne part plus entière : seulement titres, poignées, réglettes | `1bc8309` |
| **La reprise** | après un plantage, l'édition revient **telle qu'on l'a laissée** : curseur, sélection, carte sélectionnée | `b135a85` |
| **BOARDS-1** | les onglets se **renomment** (double-clic), se **rangent** (glisser), se **suppriment** (croix, `Ctrl+Z` les rend), un **menu** au clic droit ; le contenu d'un dossier n'est plus un onglet | `ff99d97`, `7388eb5` |
| **BOARDS-2** | **importer un document** dans un onglet neuf : par le menu d'un onglet, ou en le lâchant sur la barre ; ses images viennent avec lui | `e5e6472`, `2171fe0` |
| **Le liseré** | regarder le passé dans la Time Machine : panneaux **11,7 → 0,5 ms** par image | `36c87af` |
| **ARC-1** | les coins des cartes, pastilles et boutons sont de **vrais arcs**, comme ceux de Tauri et des membranes | `ef68e6a` |

---

## 1. Son second essai, lu

Ses cinq résultats : le lancement rouvre le document, parfaitement. Le plantage : *« tout a été
enregistré et ma caméra était bien au bon endroit, juste […] ça n'a pas repris exactement le
mode édition de texte et exactement où était précisément mon pointeur de texte »*. La réglette
tenue fonctionne. Les membranes fonctionnent — *« mais continue pas forcément maintenant, on
doit en discuter avant »*. Le zoom : fait.

Sa chronique (`sortie-chronique-2026-09-25-essai2.txt`, 127 s, 1 950 images, 139 nœuds, écran
240 Hz) :

* **26 images par seconde** entre deux images consécutives ; 74 % des images au-dessus du
  plancher de 10 ms.
* Le premier poste, et de loin : **`membranes`, 23,2 ms en médiane** sur les images qui ratent
  leur balayage, 48 au pire. Une membrane qui remplit l'écran se peignait trois fois, au
  processeur, pixel par pixel.
* Derrière elle, `blit` (8 à 10 ms) et `effacer` (2,4 ms) : la couche du dessous, que la
  membrane remplissait, partait **entière** à la carte à chaque image.
* **`docks`** — la question de CEDER-1 — : **5,8 ms en médiane au repos, 23,2 au p99**. La mesure
  a trouvé la cause, et ce n'était pas l'atelier : le liseré de la Time Machine (§ 3). Ce qui
  reste des panneaux en zoomant (11,6 ms au p99) ne se départagera qu'avec sa prochaine
  chronique.
* **`etages`** n'apparaît dans aucune liste : la chronique ne montre que les six postes les plus
  lourds de chaque geste, par médiane et par p99. Ce n'est donc qu'une **borne**, pas une
  mesure : au p99, moins de 1,2 ms en glissant un nœud, moins de 3,4 au repos, moins de 8,2 en
  zoomant.
* **Pas traité** : `textures`, jusqu'à 23 ms sur les pires images de zoom — le téléversement des
  photos quand le zoom demande un niveau plus fin (2,3 Mpx d'un coup).

---

## 2. Les membranes sur la carte (MEMB-FORME-1, BANDE-2)

### 2.1 La loi

Une membrane est quatre couches de la **même teinte** : deux halos, le fond, le bord. Une même
teinte composée sur elle-même n'est qu'une opacité : `A = 1 − Π (1 − aₖ · couvertureₖ)`. Une
membrane n'est donc pas quatre dessins mais **un seul champ**, constant partout où aucune couche
n'a de bord — presque toute sa surface.

Chaque couche est un rectangle aux coins **circulaires** (le `rx` de Tauri). Sa distance signée
est exacte, et calculée depuis les bords, pas depuis le centre : loin du centre, au fort zoom,
une distance calculée depuis le centre perd sa précision en `f32`. Le pointillé se lit sur
l'**abscisse curviligne** du bord, accumulée en `f64`.

La loi vit dans le noyau (`glucose_core::membrane_forme`), avec deux instruments :

* **la carte** (`present::membranes_gpu`) : un nuanceur qui évalue la loi en chaque pixel ;
* **le processeur** : un rasteriseur par segments — une seule composition par pixel là où `A`
  est constant, le calcul complet seulement le long des bords.

Accord des deux voies : **un niveau sur 255**. Les segments rendent la loi en chaque pixel, au
bit près. Dix-huit sabotages tombent.

### 2.2 Mesures

`bench_membranes`, 2 160 × 1 350, voie processeur :

| cas | avant | après |
|---|---|---|
| une membrane entière à l'écran | 13-19 ms | **3,8 ms** |
| une membrane qui remplit l'écran | 18-24 ms | **3,2-3,7 ms** |
| un coin à ×40 | ≈ 13,5 ms | ≈ 13,5 ms |

Le coin à ×40 n'a pas bougé, et je ne le cache pas : la bande du pointillé y fait 80 pixels de
large, et elle reste calculée pixel par pixel (§ 7). Sur la voie graphique — la sienne —, c'est
la carte qui la peint.

**BANDE-2** : la couche du dessous ne porte plus que ce que la carte ne peint pas (titres,
poignées, réglettes, dossiers), et elle s'efface et part par ses bandes, comme le dessus. Une
épreuve tient que **deux images de suite laissent le dessous qu'une seule aurait laissé**, au bit
près : sans elle, le titre d'une membrane laisserait une traînée en glissant.

---

## 3. Le liseré du passé

Quand la Time Machine montre le passé, un trait ambré borde la fenêtre et un voile glisse vers
l'intérieur. Peint au processeur, il coûtait **11,7 ms par image**, et ses bords gauche et
droit touchaient toutes les lignes : la couche du dessus repartait entière — 1 350 lignes sur
1 350 — **exactement pendant qu'on glisse la réglette**.

La carte le peint maintenant (`present::lisere_gpu`) : un triangle plein écran, la distance aux
bords, la même loi qu'au processeur. Le trait est la réunion de quatre bandes, couverte de
`1 − Π (1 − cₖ)`, exacte aux coins intérieurs ; une première version ne lisait que la bande la
plus proche, et l'épreuve l'a refusée à l'échelle 1,5. Accord : deux niveaux, trois dans les
coins, mesurés et localisés. Après : **`docks` 0,5 ms, 744 lignes**.

---

## 4. La reprise, telle qu'on l'a laissée

Son document le dit (`examples/lire_histoire`, un lecteur de l'histoire, nouveau) : le texte
tapé avant l'arrêt **avait** été rendu en édition au lancement suivant, puis fermé par un
`Ctrl+S`. Mais l'édition ne ressemblait pas à celle qu'il avait quittée : la carte rouverte
n'était pas sélectionnée, et le curseur revenait au bout du texte.

La saisie gardée passe au format `SAISIE02` et porte la **sélection** (ancre et curseur) ; elle
s'écrit aussi quand seul le curseur bouge. Une `SAISIE01` se relit encore, curseur au bout. La
carte rouverte est sélectionnée, et une carte ouverte sans rien taper se rouvre aussi en
édition.

---

## 5. Les onglets (BOARDS-1, BOARDS-2)

### 5.1 Ce que font les autres

* **Figma** : les pages se renomment d'un double-clic, se rangent en glissant ; déplacer une
  page vers un autre fichier n'existe pas — on copie son contenu.
* **Miro** : un tableau se sauvegarde en fichier et se restaure **comme un tableau neuf**.
* **tldraw** : des pages qu'on renomme, range et supprime ; **Excalidraw** : une API pour
  charger une scène, pas pour en fusionner deux.
* **Excel** : déplacer une feuille vers un classeur qui porte déjà ses noms ouvre le dialogue
  des **conflits de noms**.
* **Blender** : *Fichier › Ajouter* — prendre ce qu'on veut d'un autre fichier, sans l'ouvrir.

Sa réponse a tranché : un geste **à part** de `Ctrl+O`, qui reste « ouvrir ».

### 5.2 Ce qui est fait

* **Les onglets sont les tableaux racines.** Le contenu d'un dossier s'affichait comme un
  onglet ; Tauri ne montrait que les racines. L'onglet qui porte le tableau actif s'allume,
  dossier ouvert compris.
* **Double-clic** : le nom s'ouvre dans un champ ; `Entrée` pose, `Échap` renonce, un clic
  ailleurs pose. **Glisser** : l'onglet s'efface à moitié, un trait marque sa place d'arrivée.
  **La croix** supprime l'onglet et ses dossiers, sans confirmation : un message dit que
  `Ctrl+Z` le rend. Pas de croix sur le dernier onglet. **Clic droit** : Renommer, Supprimer,
  Nouveau board, Importer un document…
* Le geste de rangement n'écrit **que l'ordre** : un retrait suivi d'une insertion aurait écrit
  deux copies complètes du tableau dans l'histoire.
* **Un tableau vit tant qu'on l'atteint** depuis les onglets. Cette règle remplace deux défauts
  anciens : supprimer le **miroir** d'un dossier supprimait le contenu de l'original (ils
  partagent un tableau), et supprimer un dossier laissait orphelins les tableaux de ses
  dossiers imbriqués.
* **Importer un document** (Rust ou Tauri) : son **présent**, jamais son histoire, entre dans des
  onglets neufs, en **un seul geste** qu'un `Ctrl+Z` défait. Chaque élément reçoit un
  identifiant neuf et chaque référence suit (propriétaire, miroir, flèches, dossiers, rideaux,
  domaines) — Tauri ne renommait que le tableau, et deux imports du même document auraient eu
  des cartes jumelles. Les images se lisent dans le document d'origine jusqu'à ce que le scribe
  les copie ici ; une clé déjà prise par d'autres octets est renommée d'après leur empreinte.
  Un document lâché **sur la barre d'onglets** s'importe ; lâché sur le canevas, il se pose
  comme avant.

Chaque type du document est déstructuré en entier dans l'importeur : un champ ajouté plus tard
fera échouer la compilation au lieu d'être oublié. Six épreuves de bout en bout sur de vrais
fichiers ; trente-neuf sabotages tombent, sur les quatre commits.

---

## 6. Un seul rectangle arrondi (ARC-1)

Trois copies du même traceur — le rendu, les panneaux, les icônes —, identiques au caractère
près : exactement ce que la charte interdit. Il n'en reste qu'une (`renderer/arrondi.rs`).

Ses coins étaient des **paraboles** qui bombaient de 6 % du rayon au milieu du coin. Ce sont des
cubiques dont les points de contrôle sont à `4/3 · (√2 − 1)` du rayon : un quart de cercle à
0,03 % près, comme le `border-radius` de Tauri, et comme les membranes depuis le § 2. Le nombre
est déduit, pas choisi. Une épreuve géométrique le tient ; faussée à 0,5, elle tombe. Les
images témoins changent de 4 000 à 4 300 pixels, tous aux coins, regardés agrandis.

**Ce que ce changement a révélé.** Quatre épreuves du cache des panneaux sont tombées : deux
pixels à deux niveaux d'écart entre le panneau réutilisé et le panneau peint directement. Leur
commentaire disait l'écart « borné par construction à une unité ». Ce n'était pas une borne :
chaque couche transparente superposée — deux ombres qui se croisent, ou l'ombre, le fond et le
trait d'un panneau au même coin — ajoute un arrondi, et le tampon du cache en ajoute un. Les
paraboles avaient simplement placé les pixels partiels ailleurs.

Plutôt qu'élargir la tolérance, je l'ai fait **disparaître** : le rendu sans cache passe
maintenant par un cache neuf, qui ne garde rien — le même chemin, les mêmes arrondis. L'égalité
est stricte ; un panneau périmé d'un seul niveau se verrait. Les épreuves comparent désormais
l'image **réutilisée** (un cache neuf contre un cache neuf ne prouvait plus rien). Ignorer la
sélection ou le survol dans la clé : les deux sabotages tombent.

Honnêteté sur le chiffre : `bench_chrome` dit maintenant **6× (7,18 → 1,24 ms)** pour les six
panneaux, là où il disait 1,1×. Ce n'est pas le cache qui a gagné : c'est la référence qui paie
maintenant son tampon, comme un vrai panneau qui change.

---

## 7. Vu en chemin, pas corrigé

* **Glisser un nœud** écrit une translation par mouvement de souris, dans un seul geste :
  l'histoire grossit plus que nécessaire. Une translation par geste suffirait.
* **La couche du dessus** part sur 744 lignes sur 1 350 à cause des panneaux latéraux : des
  rectangles vaudraient mieux que des lignes entières.
* Le **titre d'une membrane** à ×40 coûte 3,7 à 5 ms : il pourrait devenir une texture posée
  par la carte. Le **pointillé à ×40** sur la voie processeur, environ 10 ms : le remède serait
  de segmenter par tirets.
* La **réunion des bandes côté carte** (ce qui a été envoyé à l'image d'avant, ajouté à ce qui
  est envoyé maintenant) n'a pas d'épreuve à elle.
* L'**IME** (la saisie des langues à caractères composés) n'existe pas ; les **textes des
  flèches** ne sont pas branchés ; les **presets** d'un document importé ne viennent pas.

---

## 8. Les membranes : ce qu'il faut décider ensemble

Il l'a demandé : *« tout ce qui touche aux membranes c'est une très très grosse partie et je ne
souhaite pas que tu continues sans en parler »*. Rien du lot 4.2 n'a donc été commencé. Voici ce
qui est sur la table, pour la discussion.

**Ce qui reste du lot 4.2** (fiche 37 § 8.3) : les modes **minimisée** et **étirée** (et
l'alerte d'étirement), dessiner une membrane **en glissant**, le **panneau d'options**, la
**couleur dérivée des domaines**, puis les **rideaux**.

**Ce qui s'écarte de Tauri aujourd'hui**, relevé dans `SvgAnnotationLayer.tsx` :

| | Glucose Tauri | Glucose Rust |
|---|---|---|
| le bord | **2 pixels d'écran**, quel que soit le zoom | 2 unités du monde : **80 pixels à ×40** |
| les tirets | 10 pixels d'écran | 10 unités du monde |
| les halos | deux, chacun **doublé de son flou** (σ = 25), à 1,2 % et 1,8 % | deux, **nets** seulement, à 3,1 % et 5,5 % |
| le fond | 2 % | 3,1 % |
| le bord sélectionné | 90 % | 92 % |
| déplacer la membrane | le contenu **reste** | le contenu **suit** (MEMB-1, fiche 37) |

Le premier écart compte plus qu'il n'en a l'air : au fort zoom, le bord de Tauri reste un fil
fin, le nôtre devient une bande épaisse — et c'est précisément cette bande qui coûte au
processeur (§ 2.2, le coin à ×40). Suivre Tauri le rendrait **à la fois plus fidèle et moins
cher**. Mais c'est un choix de ressenti, et c'est le sien.

---

## 9. Ce qui n'est pas fait, ou pas prouvé

* **Vu à l'écran** : rien de cette session. La fluidité des membranes, la reprise avec le
  curseur, les onglets, l'import et la Time Machine attendent son essai.
* **CEDER-1** reste à confirmer : sa chronique a montré le liseré, pas l'atelier ; la prochaine
  dira ce que valent les panneaux sans lui.
* `etages` : une borne (§ 1), pas une mesure.
* `textures` : jusqu'à 23 ms sur les pires images de zoom, pas traité.
* Le coin de membrane à ×40 sur la voie processeur : ≈ 13,5 ms, inchangé.
* L'import d'un document ne prend ni son histoire (voulu) ni ses presets (pas encore).

---

## 10. Ce qu'il faut regarder à l'écran

```text
cargo run --release > sortie-onglets.txt 2>&1
```

1. **La fluidité des membranes** : un document avec une grande membrane ; zoomer dessus
   jusqu'à ce qu'elle remplisse l'écran, dézoomer, glisser la vue. Puis, **avant de relancer**,
   copier `%TEMP%\glucose-chronique\derniere-session.txt` en
   `sortie-chronique-2026-09-25-onglets.txt`.
2. **Un plantage, curseur au milieu** : taper un texte dans une carte, **cliquer au milieu** du
   texte pour y mettre le curseur, puis tuer Glucose (Gestionnaire des tâches, onglet
   **Détails**, clic droit sur `glucose-desktop.exe`, « Fin de tâche »). Relancer : la carte
   en édition, sélectionnée, le curseur **à la même place**.
3. **Les onglets** : double-cliquer sur un nom, le changer, `Entrée` ; glisser un onglet
   ailleurs dans la barre ; le supprimer par sa croix, puis `Ctrl+Z` ; clic droit sur un onglet
   pour voir le menu.
4. **Importer un document** : clic droit sur un onglet › « Importer un document… », choisir un
   ancien `.glucose` (de Rust ou de Tauri) ; il arrive dans un onglet neuf, images comprises.
   `Ctrl+Z` le retire. Puis lâcher un `.glucose` depuis l'Explorateur **sur la barre
   d'onglets**.
5. **La Time Machine** : `Ctrl+H`, glisser la réglette — cela doit être plus fluide qu'avant.
6. **Les dossiers** : ouvrir un dossier ; son contenu ne doit plus apparaître comme un onglet.

---

## 11. Ce qui attend sa parole

1. **Les membranes** (§ 8) : la discussion qu'il a demandée, avant tout le reste du lot 4.2.
2. **Le MCP** (phase 6) vient après le lot 4.2, donc après cette discussion.
3. Toujours en attente, de la fiche 37 : la **signature de code Windows**, et **ce que ses
   utilisateurs emploient le plus** (l'ordre de la phase 4).

---

## 12. Les sources

* Figma, [centre d'aide, les pages d'un fichier](https://help.figma.com/hc/en-us/articles/8403626871063) ;
  forum Figma, [*Moving pages to another file*](https://forum.figma.com/t/moving-pages-to-another-file/23080).
* Miro, [*How to save board backup*](https://help.miro.com/hc/en-us/articles/360017572774-How-to-save-board-backup).
* Microsoft, [*Why am I seeing the Name Conflict dialog box in Excel?*](https://support.microsoft.com/en-us/office/why-am-i-seeing-the-name-conflict-dialog-box-in-excel-f9251985-dbde-4030-86d8-e90775e79952).
* tldraw, [*Pages*](https://tldraw.dev/sdk-features/pages) ; Excalidraw, [*excalidrawAPI*](https://docs.excalidraw.com/docs/@excalidraw/excalidraw/api/props/excalidraw-api).
* Spencer Mortensen, [*Approximate a circle with cubic Bézier curves*](https://spencermortensen.com/articles/bezier-circle/) — le problème du § 6, et ses constantes.
* Glucose Tauri, `src/canvas/SvgAnnotationLayer.tsx` et `BoardTabs.tsx` — la référence du § 5 et
  du § 8.

---

**Retour** : [`00-INDEX.md`](00-INDEX.md)
