# 14 — L'état réel, et tout ce qui reste

> **Rôle de ce document.** Les fiches [`12`](12-PLAN-D-EXECUTION.md) et
> [`13`](13-PLAN-DE-CORRECTION.md) disent *ce que je veux faire* et *dans quel ordre*. Celui-ci
> dit **où on en est vraiment**, mesuré sur l'arbre de travail, et **tout ce qui reste** — sans
> trier par envie, sans omettre ce qui est pénible.
>
> **Date** : 2026-09-16 · mesures prises au commit `adaf415`.
> **État vérifié** : `cargo test --workspace` exit 0, **1 026 tests verts**, 1 ignoré,
> clippy strict à zéro, huit cliquets mécaniques.
>
> Il existe parce que la fiche 12 est **en retard sur la réalité** : elle annonce comme « à
> faire » des choses livrées depuis (le texte riche inline et blocs, les poignées vivantes, la
> barre d'action, le menu contextuel). Une roadmap qui se trompe sur le passé se trompera sur
> l'ordre.

---

## 1. Le chiffre, et pourquoi il y en a deux

### Compté en fonctions — 34 %

La fiche [`03`](03-PARITE-FONCTIONNELLE.md) décompose Glucose Tauri en **279 fonctions**. Elle
en donnait 22 % en juillet. Recompté aujourd'hui, domaine par domaine :

| Domaine | Fonctions | Parité | Ce qui a bougé depuis la fiche 03 |
|---|---:|---:|---|
| 1. Canvas & caméra | 14 | 50 % | le pan trackpad, le cran de zoom, la grille |
| 2. Sélection & manipulation | 18 | **72 %** | poignées vivantes, rotation, barre d'action, menu contextuel |
| 3. Images | 21 | 29 % | le dépôt et le routage |
| 4. Texte & Markdown | 19 | **74 %** | inline, blocs, tableaux, liens, LaTeX |
| 5. Stickies | 8 | 50 % | — |
| 6. Flèches | 16 | 50 % | **tout**, de 12 % à 50 % en une session |
| 7. Membranes | 17 | 12 % | — |
| 8. Rideaux | 11 | **0 %** | — |
| 9. Dossiers & sous-canvas | 15 | 27 % | le badge, l'entrée |
| 10. Miroirs | 7 | **0 %** | — |
| 11. Domaines sémantiques | 9 | 70 % | le panneau vit |
| 12. Temporalité | 8 | **0 %** | — |
| 13. Storyboard | 8 | **0 %** | — |
| 14. Presets & zones | 7 | **0 %** | — |
| 15. Undo / historique | 9 | 70 % | le journal par geste |
| 16. Persistance | 12 | 60 % | ouvrir, enregistrer, enregistrer-sous |
| 17. Export | 8 | **0 %** | — |
| 18. Interface & panneaux | 22 | 35 % | le dock, les domaines |
| 19. Entrées & plateforme | 16 | 40 % | trackpad, presse-papiers |
| 20. Collaboration | 12 | **0 %** | — |
| 21. Plugins & App Bridge | 9 | **0 %** | — |
| 22. Recherche & navigation | 7 | 15 % | — |
| 23. Divers | 6 | **0 %** | — |
| **TOTAL** | **279** | **≈ 34 %** | (22 % en juillet) |

### Compté en travail restant — plutôt 25 %

Le premier chiffre flatte, et il faut le dire. Il compte une fonction de rideau comme une
fonction de zoom, alors que l'une est un sous-système de 1 178 lignes et l'autre trois lignes.

Pondéré par les lignes de TypeScript à porter, les **neuf domaines à 0 %** pèsent à eux seuls
**≈ 8 900 lignes** — collaboration, plugins, rideaux, temporalité, export, storyboard, presets,
miroirs, divers. Aucune n'est commencée. La part réellement faite du travail est donc plus
proche de **25 %** que de 34.

**Les deux chiffres sont vrais et ne disent pas la même chose :** un tiers des *gestes* de
Glucose existent, un quart du *logiciel* est écrit.

---

## 2. Ce qui est déjà écrit, testé, et que personne n'appelle

C'est le gisement le moins cher du projet : du code qui existe, passe ses tests, et attend un
geste.

| Module | l. | Ce qu'il débloque | Ce qui manque pour l'allumer |
|---|---:|---|---|
| `timeline` | 406 | toute la temporalité | l'interface : règle, filtre, invite de saisie |
| `export` | 334 | SVG et Markdown | **l'écriture sur le disque**, rien d'autre |
| `membrane_stretch` | 234 | la membrane qui grandit avec son contenu | dépend de `membrane_id`, jamais écrit |
| `text_anchors` | 224 | flèche partant d'une **phrase** | la sélection de texte persistante |
| `curtain_model` | 222 | géométrie des rideaux | le panneau et le geste |
| `curtain_panel` | 204 | panneau de rideau | son branchement au dock |
| `mirror_graph` | 118 | anti-inception des miroirs | le geste « créer un miroir » |
| **Total** | **1 742** | | **+ leurs suites de tests** |

À quoi s'ajoutent `arena` (3 918 l.) et `fixed` (273 l.) — la fondation 10⁷, morte **par
construction** en attendant la vague 4, pas par oubli.

> **Une faiblesse du cliquet 3, nommée ici pour ne pas la redécouvrir.** Il compte les modules
> *cités par un autre module*, pas les fonctions *atteignables depuis l'application*.
> `membrane_space` (722 l.) a donc quitté la liste des admis sans qu'aucun geste ne l'atteigne :
> il est cité par `membrane_focus` et `membrane_stretch`, qui sont eux-mêmes en attente. Un
> module mort qui en appelle un autre reste mort. Le cliquet devra mesurer la **joignabilité
> depuis `glucose-desktop`**, pas la citation.

---

## 3. Le tableau complet — tout ce qui reste

Poids : ○ un geste · ◐ un chantier · ● un sous-système · ●● une vague entière.

### A — Le texte *(74 %, le plus avancé)*

| # | Ce qui reste | Poids | Note |
|---|---|:--:|---|
| A.1 | **Sélection de texte** à la souris et au clavier (`Maj`+flèches, double-clic = mot, triple = ligne) | ◐ | `Selection` existe dans le noyau ; le geste non |
| A.2 | **IME** — `set_ime_allowed`, `WindowEvent::Ime` | ◐ | aucune langue à composition n'est possible aujourd'hui |
| A.3 | Copier-coller **dans** un texte en édition | ○ | |
| A.4 | **Ancres de texte** — brancher `text_anchors` | ◐ | débloque A.5 et la flèche depuis une phrase |
| A.5 | **PGFPlots** — traceur natif, `\addplot` de coordonnées / fonction / table, `axis` | ●● | **dette nommée**, fiche 12 § 7.1 |
| A.6 | Largeur de carte libre jusqu'à 600 px (défaut 13 de la fiche 13) | ○ | reporté avec sa raison |

### B — La manipulation *(72 %)*

| # | Ce qui reste | Poids | Note |
|---|---|:--:|---|
| B.1 | **Déplacement au clavier** (flèches, `Maj` = pas large) | ○ | |
| B.2 | Alignement et distribution d'une sélection multiple | ◐ | |
| B.3 | Groupes (autres que membranes) | ◐ | |

### C — Les flèches *(50 %, de 12 % hier)*

| # | Ce qui reste | Poids | Note |
|---|---|:--:|---|
| C.1 | **Découvrabilité des prédicats** — le menu contextuel doit proposer les six | ○ | aujourd'hui il faut connaître les chiffres 1–6 |
| C.2 | Attache à un sous-bloc / à une **sélection de texte** | ◐ | dépend de A.4 |
| C.3 | **Flèche-portail** vers un autre board | ◐ | |
| C.4 | Épaisseur, couleur, bidirectionnelle, **courbes de Bézier** | ◐ | les coudes sont des segments droits |

### D — Les membranes *(12 % — 1 442 lignes écrites, presque rien de branché)*

| # | Ce qui reste | Poids | Note |
|---|---|:--:|---|
| D.1 | Dessiner une membrane **par glisser** | ○ | aujourd'hui : 320 × 240 fixe |
| D.2 | **Écrire `membrane_id` au dépôt** — l'invariant MEMB-1 | ◐ | **aucune membrane ne possède rien aujourd'hui** ; tout D en dépend |
| D.3 | Brancher `membrane_space` : adoption, coordonnées relatives, mode minimisé | ◐ | 722 l. prêtes |
| D.4 | Brancher `membrane_stretch` : grandir avec son contenu, alerte d'étirement | ○ | 234 l. prêtes |
| D.5 | **Mode focus** au geste + son tween 200 ms | ◐ | la courbe existe |
| D.6 | Panneau d'options, imbrication, suppression en cascade | ◐ | |
| D.7 | **Couleur dérivée des domaines** (`symbiotic_hue` × poids) | ◐ | la charte la désigne pour naviguer dans l'immense |

### E — Dossiers et miroirs *(27 % et 0 %)*

| # | Ce qui reste | Poids | Note |
|---|---|:--:|---|
| E.1 | **Miroirs** : geste, marquage visuel, propagation, remontée à l'original | ● | le vol de caméra 400 ms est prêt |
| E.2 | Brancher `mirror_graph` : détection de cycle | ○ | 118 l. prêtes |
| E.3 | **Miroir d'un dossier OS** : scan snapshot/live, récursif, paresseux (49 k fichiers), glob, 7 tris, vignettes | ● | |
| E.4 | Poignée gravée à 45°, **mini-carte de contenu** | ◐ | |

### F — Images et fichiers *(29 %)*

| # | Ce qui reste | Poids | Note |
|---|---|:--:|---|
| F.1 | **Mipmaps et LOD de rendu** | ● | la charte le demande explicitement : « des milliards d'images sans bug » |
| F.2 | **Cache borné** + décodage asynchrone + chargement progressif | ● | aujourd'hui tout est décodé d'un coup, en mémoire |
| F.3 | **Glisser depuis un navigateur** + position exacte du dépôt | ◐ | un `IDropTarget` à nous — même cause, même correction |
| F.4 | Vidéos | ● | |
| F.5 | Déduplication par `sha256`, tags, compteur d'images | ◐ | `sha256` est branché |

### G — Rideaux *(0 % — 426 l. écrites)*

| # | Ce qui reste | Poids | Note |
|---|---|:--:|---|
| G.1 | Créer un rideau, languette nommée et colorée | ● | **la fonctionnalité la plus originale de Glucose** |
| G.2 | Rideau = board complet, visibilité privé/partagé, droits, ratios | ● | |

### H — Temporalité *(0 % — 406 l. écrites)*

| # | Ce qui reste | Poids | Note |
|---|---|:--:|---|
| H.1 | Ancrage d'un nœud dans le temps, invite de saisie | ◐ | |
| H.2 | Règle temporelle, filtre, plages, **années négatives** | ● | |

### I — Export et persistance *(0 % et 60 %)*

| # | Ce qui reste | Poids | Note |
|---|---|:--:|---|
| I.1 | **Écriture sur le disque** des exports SVG et Markdown | ○ | **les deux moteurs sont prêts** — c'est le geste le moins cher du tableau |
| I.2 | Export HTML et PNG | ◐ | |
| I.3 | **Autosave**, versions automatiques, compaction | ◐ | |
| I.4 | Récupération après crash | ◐ | |
| I.5 | Migration du format v1 TypeScript | ◐ | sans elle, les anciens documents sont perdus |

### J — Interface *(35 %)*

| # | Ce qui reste | Poids | Note |
|---|---|:--:|---|
| J.1 | **Minimap cliquable** — elle est dessinée et inerte | ○ | |
| J.2 | Minimap : annotations et membranes, pas seulement les images | ○ | |
| J.3 | **Recherche `Ctrl+F`** plein texte + navigation vers un résultat | ◐ | rien n'existe |
| J.4 | Dock animé : rebond + FLIP | ◐ | les courbes sont prêtes, les gestes non |
| J.5 | **Sélecteur de couleur** | ◐ | |
| J.6 | Panneau Ordonner complet : 8 tris × 5 dispositions | ◐ | |
| J.7 | Renommer / fermer / réordonner les boards | ○ | |
| J.8 | Signets de vue 1–9, zoom clavier `+`/`−`/`0`, pavé numérique | ○ | |
| J.9 | Infobulles, barre de statut, HUD | ◐ | |
| J.10 | **Réaction du glow au survol** (15 % → 40 % en 0,2 s) | ○ | demande un état « qui est survolé » que l'application n'a pas |
| J.11 | Storyboard, presets, zones | ● | maquettes aujourd'hui |

### K — Le moteur *(la vague 4 — rien n'est commencé)*

| # | Ce qui reste | Poids | Note |
|---|---|:--:|---|
| K.1 | **Substitution du noyau vers l'arène** — 209 accès dans `store/` | ●● | 3 918 l. d'arène attendent ; la règle S rend l'opération locale |
| K.2 | Étaler la reconstruction de l'index sur plusieurs frames | ◐ | 16 ms tombent aujourd'hui d'un coup |
| K.3 | **Cache pyramidal de tuiles** + invalidation exacte | ●● | |
| K.4 | Couche d'arêtes longues | ● | |
| K.5 | **Présentation GPU** avec repli CPU testé par défaut | ●● | l'effacement du fond à 3,1 ms en 4K ne se réduit que là |
| K.6 | Non-régression visuelle au pixel entre CPU et GPU | ◐ | |

### L — Les plateformes *(la charte les pose comme contrainte de conception)*

| # | Ce qui reste | Poids | Note |
|---|---|:--:|---|
| L.1 | **Couche d'abstraction GPU** : Vulkan + Metal + D3D + GL ES | ●● | OpenGL ne couvre pas la liste ; Apple l'a figé à 4.1 |
| L.2 | **Android** : multi-touch de première classe, cycle de vie d'activité, perte du contexte graphique, mémoire contrainte | ●● | |
| L.3 | **macOS** (Apple Silicon), **Linux** | ● | |
| L.4 | Couche d'entrée unifiée — c'est là que F.3 doit être écrit | ● | |

### M — Le lointain

| # | Ce qui reste | Poids | Note |
|---|---|:--:|---|
| M.1 | Collaboration : CRDT, curseurs, canal d'assets | ●● | 1 920 l. de TS |
| M.2 | Plugins et App Bridge | ● | 1 471 l. de TS |
| M.3 | Jonction `glucose-brain` — le canva Wikipédia | ●● | **c'est le but du projet**, et rien n'y touche encore |
| M.4 | Pomodoro, télémétrie, mise à jour | ◐ | |

---

## 4. Ce que je ferais, dans cet ordre, et pourquoi

Trois principes de tri, dans cet ordre de priorité.

**1. Ce qui est écrit et éteint passe avant ce qui n'existe pas.** `I.1` — l'écriture disque des
exports — allume 334 lignes testées pour quelques dizaines de lignes de code. `E.2` en allume 118
pour moins encore. C'est le meilleur rendement du dépôt, et c'est aussi ce qui rembourse R-18.

**2. Ce qui bloque plusieurs chantiers passe avant ce qui n'en bloque qu'un.** `D.2` (écrire
`membrane_id`) débloque tout le domaine 7 à lui seul. `A.4` (les ancres de texte) débloque `C.2`.
`F.3` (l'`IDropTarget`) débloque deux manques à la fois.

**3. Ce que la charte nomme passe avant le confort.** `D.7` (couleur des domaines) et `J.3`
(recherche) sont les deux outils que la charte désigne pour **naviguer dans l'immense** — c'est
le but déclaré du projet, pas une amélioration.

Ce que cela donne :

| Rang | Lot | Pourquoi ici |
|---:|---|---|
| 1 | `C.1` la découvrabilité des prédicats | finit proprement le chantier en cours |
| 2 | `I.1` l'écriture disque des exports, `E.2` `mirror_graph` | le meilleur rendement du dépôt |
| 3 | `D.2` → `D.4` les membranes possèdent enfin | débloque un domaine entier à 12 % |
| 4 | `A.1` la sélection de texte, puis `A.4` les ancres | débloque `C.2` |
| 5 | `J.3` `Ctrl+F` et `D.7` la couleur des domaines | ce que la charte nomme |
| 6 | `G` les rideaux | le plus original, et 426 l. attendent |
| 7 | `K` + `L` le moteur et les plateformes | quand on saura ce qu'il y a dans une tuile |

---

## 5. Ce qui n'est pas dans le tableau, et qui devrait inquiéter

Trois choses que ce document ne sait pas mesurer, et qui pèsent plus lourd que n'importe quelle
ligne ci-dessus.

1. **Trois vérifications à l'écran n'ont jamais été faites** — `Ctrl`+clic ouvre-t-il vraiment un
   lien, le pavé tactile déplace-t-il à deux doigts, les saccades ont-elles baissé. Elles sont
   marquées « corrigé, à confirmer » depuis la fiche 13. Tant qu'elles ne le sont pas, trois
   corrections sont des hypothèses.

2. **Aucun document réel n'a jamais été ouvert dans Glucose Rust.** La migration du format v1
   (`I.5`) n'existe pas, donc le logiciel n'a jamais affronté autre chose que des documents
   synthétiques et ce qu'on y tape à la main. Ce que ça cache est inconnu par construction.

3. **Le but du projet — le canva Wikipédia — n'a pas de premier pas planifié.** `M.3` est en bas
   du tableau alors que c'est lui qui justifie les 10⁷ nœuds, l'arène, le GPU et les domaines.
   Rien ne dit aujourd'hui à quoi ressemble la plus petite version utile de cette jonction.

---

*Suite immédiate : `C.1`, la découvrabilité des prédicats — le menu contextuel doit proposer les
six quand une flèche est sélectionnée.*
