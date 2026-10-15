# Chaque fonction, et où la main la trouve

> **Au 15/10/2026** (fiches 63 à 72). Sa demande : *« qu'il y ait bien un bouton pour absolument chaque
> fonctionnalité, sur PC comme au téléphone »*, et *« la liste des fonctionnalités et des boutons
> qui ne sont pas encore créés mais qu'on prépare »*. Ce document y répond, et se tient à jour :
> une fonction neuve y entre avec son bouton, ou elle n'est pas finie (R1).
>
> **La règle** : tous les boutons et toutes les fonctionnalités à venir sont obligatoires. Un bouton
> qui ne fait rien encore se fait marcher ; on n'en retire jamais un.

**Au PC** : la bande du haut (ou le rail sur le côté quand elle ne tient pas), le menu du clic
droit (sur le vide, sur une sélection, sur un onglet), la barre d'action d'une sélection, les
panneaux, le clavier. **Au téléphone** : le rail, le menu au doigt (deux touchers sur le vide, ou
un appui long sur un nœud), les gestes (deux doigts touchés annulent, trois rétablissent), les
panneaux. Le menu se range en colonnes quand l'écran est trop bas (ACCES-1).

✅ marche · 🟡 marche en partie · ❌ ne marche pas encore là · — sans objet là

---

## 1. Ce qui existe, et par où on y arrive

### Le document

| fonction | au PC | au téléphone |
|---|---|---|
| Nouveau document | menu du vide, `Ctrl+N` | ✅ menu |
| Ouvrir un document | menu du vide, `Ctrl+O` | ✅ menu (la liste des documents rangés) |
| Enregistrer (un jalon) | **menu du vide** (ACCES-1), `Ctrl+S` | ✅ menu (range un travail sans nom) |
| Enregistrer sous | **menu du vide** (ACCES-1), `Ctrl+Maj+S` | ✅ menu (un autre nom) |
| Renommer, dupliquer, supprimer un document | l'explorateur du système | ✅ appui long dans la liste |
| Importer un document en onglets | menu d'un onglet, ou le déposer sur les onglets | ✅ menu d'un onglet (la liste) — **ACCES-1** |
| Exporter (Markdown, SVG, PNG) — **le SVG porte tout ce que le canevas montre** (photos, membranes, dossiers, formules, zones, cases ; le texte en tracés, lisible par n'importe quel lecteur — fiches 69 à 71) | bouton Exporter, **menu du vide**, `Ctrl+E` | ✅ rail, menu : le format se demande, puis **la feuille de partage** (PARTAGE-2) |
| **Exporter le storyboard en planches** — PDF (une page par planche) ou images ; la planche est ce que le canevas montre, au format A (P2.7, fiche 68) | panneau Storyboard « Exporter en planches… », menu du vide | ✅ les mêmes : un PDF, à la feuille de partage |
| Time Machine (regarder, restaurer, jalons) | bouton, `Ctrl+H` | ✅ rail |
| Annuler, rétablir | **menu** (ACCES-1), `Ctrl+Z`, `Ctrl+Y` | ✅ menu, deux et trois doigts |
| Journal technique, voir ce qui part | menu du vide | ✅ menu (sans « voir ce qui part » : pas d'explorateur) |

### Naviguer

| fonction | au PC | au téléphone |
|---|---|---|
| Se déplacer, zoomer (jusqu'à ×256, fiche 72) | souris, pavé, `Espace`, outil Main | ✅ un doigt sur le vide, deux doigts |
| **La palette de commandes** — chaque commande cherchée par son nom, lancée par Entrée (fiche 72) | **menu du vide**, `Ctrl+Maj+P` | ✅ menu (deux touchers sur le vide) |
| Tout cadrer | **menu du vide** (ACCES-1), **`Alt+F`** (`Ctrl+F` jusqu'au 13/10, rendu à la recherche — fiche 65) | ✅ menu |
| Cadrer la sélection | **menu de la sélection** (ACCES-1), `F` | ✅ menu |
| Signets de vue (poser, y voler, remplacer) | **menu du vide « Signets… »** (ACCES-1), `Ctrl+1…9`, `1…9` | ✅ menu |
| Minimap | la minimap | ✅ |
| Onglets (choisir, ajouter, renommer, ranger, supprimer) | les onglets, leur menu | ✅ |
| Dossiers (entrer, sortir) | double-clic, fil d'Ariane | ✅ |
| Mode référence (sans cadre, au premier plan) | menu du vide, `Ctrl+Maj+A` | — (un téléphone n'a pas de fenêtres) |

### Les nœuds

| fonction | au PC | au téléphone |
|---|---|---|
| Poser texte, post-it, flèche, dossier, membrane | la bande, `T` `N` `A` `M` | ✅ rail |
| **Tracer** une membrane ou un dossier à la taille voulue (un clic : sa taille de naissance) | l'outil, puis glisser un rectangle (TRACE-1, fiche 65) | ✅ (le doigt glisse) |
| Ajouter des images | bouton Images, menu du vide, `Ctrl+I`, glisser-déposer, coller | ✅ rail, menu (sélecteur de photos), « Partager » vers Glucose |
| Copier, couper, coller, dupliquer | menu, `Ctrl+C` `X` `V` `D` | 🟡 dans Glucose seulement : le presse-papiers d'Android n'est pas encore pris |
| Copier vers Discord, un navigateur, Paint | `Ctrl+C`, glisser | ❌ Android (le presse-papiers du système) |
| Enregistrer une image sous | menu d'une image | ✅ menu : **la feuille de partage** (Fichiers, la galerie…) — PARTAGE-2 |
| Remplacer un lien par son image | menu | ✅ |
| Verrouiller | barre d'action, menu, `L` | ✅ |
| Retirer les bordures | menu, `Ctrl+B` | ✅ |
| Premier plan, arrière-plan | menu, `Ctrl+]` `Ctrl+[` | ✅ |
| Supprimer | barre d'action, menu, `Suppr` | ✅ |
| Transformer la sélection (origine commune ou individuelles) | poignées, barre d'action, `Alt` + coin | ✅ |
| Recadrer une image | double-clic | ✅ |
| Flèche : droite ou courbe, double sens, épaisseur, relation, ancres de texte | la barre de la flèche, `1…6` | ✅ |
| Déplacer d'un pas | les flèches du clavier | — (le doigt glisse) |
| Aimant | bouton Aimant | ✅ |

### Organiser

| fonction | au PC | au téléphone |
|---|---|---|
| Ordonner (cinq dispositions, huit tris, champs) | panneau Ordonner | 🟡 les champs se glissent, **ne se tapent pas** (le clavier d'Android ne sert pas les champs du dock) |
| **Mood board** (l'ancien « Presets » de Tauri, renommé à sa parole, fiche 71) : poser un gabarit, ses zones, « Par mood board » dans Ordonner | panneau Mood board | ✅ |
| **Storyboard** : poser la grille (« Activer », un clic), « Mettre à jour » le storyboard courant, effacer le courant (×) | panneau Storyboard | ✅ (STORYBOARD-1, fiche 64 — à juger) |
| **Plusieurs storyboards par tableau**, une séquence chacun : « + Nouveau » en pose un autre ; ‹ et › passent de l'un à l'autre (« 2 / 3 ») ; choisir une case fait le sien **courant** — celui que le panneau montre, que « ▶ Lire », « + Ajouter », « Mettre à jour » et × visent (P2.8, fiche 68) | panneau Storyboard, la bande d'une case | ✅ (les mêmes boutons au doigt) |
| Storyboard : **tracer un rectangle** que les cases remplissent | « Activer », puis glisser (STORYBOARD-2, fiche 65) | ✅ |
| Storyboard : **le retracer** — des cases en plus ou en moins, celles qui portent quelque chose gardées | la **poignée** au coin bas droit, une case choisie ; ou « Mettre à jour », puis glisser | ✅ |
| Une case : **la lâcher à une autre place** — les autres se décalent, chacune avec ses images | sa bande, glissée | ✅ |
| Une case : **passer à la suivante en la décrivant** | `Tab` dans la description (`Maj+Tab` : la précédente) | — (pas de touche Tab au doigt) |
| Une case : la prendre et la glisser avec ses images | sa bande | ✅ (le doigt sur la bande) |
| Une case : sa description | deux clics sur sa bande, ou sa ligne dans le panneau | ✅ (la question à champ, le clavier du téléphone) |
| Une case : la retirer | sa croix, `Suppr`, le × de sa ligne | ✅ (croix, panneau) |
| Une image dans une case : s'y ajuste | la lâcher dedans | ✅ |
| **Un storyboard de la sélection** : une case par image choisie, dans l'ordre de lecture — **un storyboard naît** à la place de la sélection, à côté de ceux qui sont là, et devient le courant (P2, fiches 66 et 68) | menu de la sélection, « Storyboard de la sélection » | ✅ menu (appui long sur la sélection) |
| **Plusieurs images** lâchées sur une case : une par case vide, à la suite, dans l'ordre de lecture — le storyboard s'allonge s'il le faut (P2, fiche 66) | les glisser ensemble, ou déposer des fichiers dessus | ✅ (les images choisies, glissées au doigt) |
| **La durée d'une case** (la sienne, ou celle du storyboard ; « ↺ » la rend) — en secondes, glissée ou tapée (fiche 67) | panneau Storyboard, « CASE n », DURÉE | 🟡 se glisse ; ne se tape pas (le clavier ne sert pas les champs du dock) |
| **La durée par case** de tout le storyboard (fiche 67) | panneau, DURÉE PAR CASE | 🟡 idem |
| **Le plan d'une case** (PG … TGP), son sigle sous la case (fiche 67) | panneau, PLAN (un sélecteur qui tourne) | ✅ |
| **Les champs d'une case** (Dialogue, Action, Notes…) : écrire, renommer, retirer, ajouter un champ (fiche 67) | panneau, « CASE n » : toucher le texte, toucher le nom, « + Champ » | ✅ (la question à champ) |
| **Lire le storyboard** — l'animatique plein écran, une coupe à chaque durée, sous-titres (fiche 67) | « ▶ Lire » du panneau, menu du vide « Lire le storyboard » ; Échap, ← →, Espace | ✅ (toucher : tiers gauche recule, le reste avance, la croix finit ; retour) |
| Ajouter une case, renuméroter | « + Ajouter », « Re-numéroter les panels » | ✅ |
| **Insérer une case entre deux** — les suivantes glissent d'un cran avec leurs images (P2, fiche 66) | les deux « + » de la case choisie, au milieu de ses bords : avant, après | ✅ (les mêmes, au doigt) |
| Domaines (créer, éditer, attribuer) | panneau Domaines | 🟡 le nom d'un domaine ne se tape pas (même cause) |
| Timer Pomodoro | panneau Timer | ✅ |

---

## 2. Les boutons qui ne font pas encore ce qu'ils annoncent

| où | quoi | ce qu'il faut | quand |
|---|---|---|---|
| Plugins | « Télécharger » le modèle d'IA | l'IA locale | plus tard |
| bande | **Collaborer** : dit « pas encore disponible » | la collaboration — obligatoire | plus tard, à sa parole |
| bande | **Trans-domaines** : ne fait rien, **et ne le dit pas** | sa définition — de la recherche, son raisonnement sur « domaine » n'est pas fini (15/10) | **dire « bientôt »**, comme Collaborer (fiche 71 : un bouton ne fait jamais semblant) |
| Mood board | « + Créer un mood board », grisé — et **un preset est désormais l'interface entière** (disposition, comportement, couleurs, raccourcis, fenêtres : Glucose, PureRef, Figma, Miro — `VISION.md`), pas des zones | la fondation : l'interface devenue donnée | SUITE P4 |
| le téléphone | les champs de nombre du dock, le nom d'un jalon, celui d'un domaine : on ne peut pas taper | le miroir du clavier (CLAVIER-1) étendu au dock | avec l'UX du téléphone, refaite en dernier |
| au premier lancement | la carte d'accueil naît **choisie** : le premier clic droit ouvre le menu d'une sélection | à vérifier contre Tauri | relecture des menus |

---

## 3. Ce qui n'existe pas encore, et qu'on prépare

De Glucose Tauri (la fiche 03 en est l'inventaire d'origine), ou de ses demandes. Chacun aura son
bouton, au PC et au téléphone, le jour où il existe.

| fonction | d'où | où elle se trouvera | où en est-elle |
|---|---|---|---|
| **Les presets d'interface** : un preset de Glucose jamais vu, un PureRef, un Figma, un Miro — la disposition, le comportement, les couleurs, les raccourcis, les fenêtres (sa parole du 15/10, `VISION.md`) | sa vision | à concevoir | la fondation d'abord (SUITE P4), sa liste PureRef à venir |
| **Les choix à l'utilisateur** : planche claire ou sombre, champs sous les cases, « 2.3 », le numéro dans la bande ou le coin, l'export PNG à 100 % ou aux originaux (fiche 71) | sa parole | la question d'export, le panneau Storyboard | à faire |
| **Rideaux** (un tableau entier dans une membrane, privé ou partagé) | Tauri | membrane, en mode Focus | noyau écrit (`curtain_model`), à brancher ; **à sa parole** |
| **Temporalité** : ancrer un nœud dans le temps, la réglette de −10 000 à 2 100, le filtre | Tauri (`Maj+T`, `Maj+R`) | barre d'action, panneau | noyau écrit (`timeline`), à brancher |
| **Recherche** dans tout le document | Tauri (`Ctrl+F`) | bande, menu, **`Ctrl+F`** (sa décision du 13/10 : tout cadrer passe à `Alt+F`) | à concevoir — `Ctrl+F` lui est gardé |
| **Miroirs** (un même nœud à deux endroits) | Tauri (`Ctrl+Maj+M`) | menu de la sélection | noyau en partie (`mirror_graph`) |
| **Dossiers miroirs du disque** (un dossier du PC, vivant) | Tauri | outil Dossier | — |
| **Opérateurs des post-it** : ET, OU, MAIS, PARCE QUE | Tauri (`Alt+0…4`) | menu d'un post-it | **se dessinent déjà**, aucun geste ne les pose |
| **Couleur d'un post-it**, sélecteur de couleur | Tauri | menu, barre d'action | — |
| **Couleur des domaines sur le canevas**, filtrer par domaine | Tauri, fiche 03 § 11 | panneau Domaines | — |
| **Membranes étirées et minimisées** (l'idée de Mary) | Tauri | options de la membrane | noyau écrit ; **à sa parole** |
| **Export HTML** (pan, zoom, descriptions, un fichier) | Tauri | Exporter | — |
| **Mode zen** (`F11`) | Tauri | menu | le mode référence en fait une partie |
| **Vidéos** | Tauri | comme les images | — |
| **Provenance des images** (SauceNAO) | sa demande | menu d'une image | **sa clé SauceNAO** |
| **Collaborer** | Tauri, obligatoire | bande | plus tard |
| **Plugins, IA locale, MCP** | Tauri, PLUGINS.md | panneau Plugins | plus tard |
| **Trans-domaines** | Tauri | bande | sa définition, à sa parole |
| **Le presse-papiers d'Android**, les photos HEIC | le téléphone | — | à faire |
| **Les doigts sous Windows** (tablettes) | toutes les machines | — | à faire |
| **Mac** (une version publiée), **iPad** (par le web) | toutes les machines | — | la CI compile le Mac ; l'iPad attend le web |
| **Le dessin de la sélection** (son choix entre les directions) | sa demande | — | planches à lui montrer |
| **L'UX du téléphone**, toute refaite : ultra rapide, fluide, pratique | sa demande | — | **en dernier** |
| **Le Storyboard, plus loin** : ~~plusieurs images qui remplissent les cases vides à la suite~~ (fait, fiche 66) ; ~~un storyboard de la sélection~~ (fait, fiche 66) ; ~~insérer une case entre deux~~ (fait, fiche 66) ; ~~la lecture, les champs, le plan~~ (faits, fiche 67) ; ~~plusieurs storyboards par tableau~~ (fait, fiche 68) ; ~~l'export en planches~~ (fait, fiche 68) — **les huit propositions sont écrites** | ses mots du 13/10, Boords, Storyboarder, Toon Boom, Krita | panneau Storyboard, clic droit | **validées toutes les huit** (13/10) — SUITE P2 |
| **Le défilement au bord** (« moche, pas fluide ») | son essai | — | à reprendre avec sa chronique |
