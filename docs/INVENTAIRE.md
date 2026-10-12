# Chaque fonction, et où la main la trouve

> **Au 12/10/2026** (fiche 63). Sa demande : *« qu'il y ait bien un bouton pour absolument chaque
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
| Exporter (Markdown, SVG, PNG) | bouton Exporter, **menu du vide**, `Ctrl+E` | ❌ **le sélecteur n'existe pas au téléphone : rien ne s'ouvre** |
| Time Machine (regarder, restaurer, jalons) | bouton, `Ctrl+H` | ✅ rail |
| Annuler, rétablir | **menu** (ACCES-1), `Ctrl+Z`, `Ctrl+Y` | ✅ menu, deux et trois doigts |
| Journal technique, voir ce qui part | menu du vide | ✅ menu (sans « voir ce qui part » : pas d'explorateur) |

### Naviguer

| fonction | au PC | au téléphone |
|---|---|---|
| Se déplacer, zoomer | souris, pavé, `Espace`, outil Main | ✅ un doigt sur le vide, deux doigts |
| Tout cadrer | **menu du vide** (ACCES-1), `Ctrl+F` | ✅ menu |
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
| Ajouter des images | bouton Images, menu du vide, `Ctrl+I`, glisser-déposer, coller | ✅ rail, menu (sélecteur de photos), « Partager » vers Glucose |
| Copier, couper, coller, dupliquer | menu, `Ctrl+C` `X` `V` `D` | 🟡 dans Glucose seulement : le presse-papiers d'Android n'est pas encore pris |
| Copier vers Discord, un navigateur, Paint | `Ctrl+C`, glisser | ❌ Android (le presse-papiers du système) |
| Enregistrer une image sous | menu d'une image | ❌ **rien ne s'ouvre au téléphone** |
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
| Presets : poser un gabarit, ses zones, « Par slot preset » | panneau Preset | ✅ |
| Domaines (créer, éditer, attribuer) | panneau Domaines | 🟡 le nom d'un domaine ne se tape pas (même cause) |
| Timer Pomodoro | panneau Timer | ✅ |

---

## 2. Les boutons qui ne font pas encore ce qu'ils annoncent

| où | quoi | ce qu'il faut | quand |
|---|---|---|---|
| **le téléphone** | **Exporter** et **« Enregistrer l'image sous… »** : rien ne s'ouvre | la feuille de partage d'Android (ce que font Procreate et Concepts) | **PARTAGE-2, la suite immédiate** |
| bande | **Storyboard** : « Activer » refuse, rien ne se dessine | STORYBOARD-1 | prochain chantier de fonction |
| Preset | « Créer un preset custom », grisé | les zones du tableau, gardées comme gabarit du document | après le Storyboard |
| Plugins | « Télécharger » le modèle d'IA | l'IA locale | plus tard |
| bande | **Collaborer** : dit « pas encore disponible » | la collaboration — obligatoire | plus tard, à sa parole |
| bande | **Trans-domaines** : ne fait rien, **et ne le dit pas** | sa définition, à sa parole (« laisse le bouton ne rien faire », 23/09) | **une question pour lui** : dire « bientôt », comme Collaborer ? |
| Exporter | le SVG ne porte ni les images ni les membranes | l'export SVG complet | après « preset custom » |
| le téléphone | les champs de nombre du dock, le nom d'un jalon, celui d'un domaine : on ne peut pas taper | le miroir du clavier (CLAVIER-1) étendu au dock | avec l'UX du téléphone, refaite en dernier |
| au premier lancement | la carte d'accueil naît **choisie** : le premier clic droit ouvre le menu d'une sélection | à vérifier contre Tauri | relecture des menus |

---

## 3. Ce qui n'existe pas encore, et qu'on prépare

De Glucose Tauri (la fiche 03 en est l'inventaire d'origine), ou de ses demandes. Chacun aura son
bouton, au PC et au téléphone, le jour où il existe.

| fonction | d'où | où elle se trouvera | où en est-elle |
|---|---|---|---|
| **Storyboard** : la grille posée par un fantôme, les cases, une image par case, les descriptions | Tauri | bande, panneau Storyboard | Tauri lu (fiche 62 § 10) |
| **Preset personnel** | Tauri | panneau Preset | — |
| **Rideaux** (un tableau entier dans une membrane, privé ou partagé) | Tauri | membrane, en mode Focus | noyau écrit (`curtain_model`), à brancher ; **à sa parole** |
| **Temporalité** : ancrer un nœud dans le temps, la réglette de −10 000 à 2 100, le filtre | Tauri (`Maj+T`, `Maj+R`) | barre d'action, panneau | noyau écrit (`timeline`), à brancher |
| **Recherche** dans tout le document | Tauri (`Ctrl+F`) | bande, menu | à concevoir — `Ctrl+F` sert aujourd'hui à tout cadrer : **à lui dire** |
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
| **Le défilement au bord** (« moche, pas fluide ») | son essai | — | à reprendre avec sa chronique |
