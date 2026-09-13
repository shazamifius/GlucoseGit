# 📖 Guide d'utilisation de Glucose

> Ce guide décrit **ce que le logiciel fait aujourd'hui** — pas ce qu'il fera. Chaque geste
> ci-dessous est branché, annulable et enregistré. Ce qui manque encore est listé dans le
> [README](README.md#-où-en-est-le-portage-honnêtement), et détaillé dans le
> [dossier d'architecture](docs/architecture/00-INDEX.md).

---

## 1. Le canvas

À l'ouverture, Glucose montre une feuille noire infinie, quadrillée de points, avec une carte
d'accueil. Tout se pose dessus.

| Geste | Comment |
|---|---|
| **Se déplacer (pan)** | clic du milieu ou clic droit glissé · `Espace` + clic gauche glissé · outil main (`H`) |
| **Zoomer** | molette — le point du monde sous le curseur ne bouge pas |
| **Recentrer** | `F` ramène la caméra à l'origine, à l'échelle 1 (le cadrage sur le contenu n'est pas encore branché) |
| **Minimap** | en bas à droite ; un clic dedans recentre la vue à cet endroit |
| **Tableaux** | la barre d'onglets sous la barre d'outils ; `+` crée un tableau |

---

## 2. Poser

| Outil | Touche | Ce qui se passe |
|---|---|---|
| **Images** | `Ctrl+I` ou bouton `+ Images` | un dialogue natif ; PNG, JPEG, WebP, GIF, BMP |
| | dépôt depuis l'explorateur | un fichier à la fois, posé sous le curseur |
| | `Ctrl+V` | colle une image du presse-papiers (capture, navigateur) |
| **Carte texte** | `T` puis clic | une carte en édition, avec Markdown : `# titre`, `## sous-titre`, `- puce`, et des formules LaTeX (`$…$`, `$$…$$`) |
| **Note adhésive** | `N` puis clic | une note jaune ; les opérateurs ET / OU / MAIS / PARCE QUE sont affichés quand une note en porte un |
| **Flèche** | `A` puis clic | une flèche droite, à taille fixe pour l'instant |
| **Membrane** | `M` puis clic | un cadre pointillé nommé, à taille fixe pour l'instant |
| **Dossier** | bouton de la barre d'outils, puis clic | un sous-canvas ; double-clic pour y entrer, fil d'Ariane pour remonter |

Une carte texte ou une note s'édite au **double-clic** : `Entrée` valide, `Maj+Entrée` saute
une ligne, `Échap` sort. La hauteur d'une carte suit son texte. Une formule s'affiche rendue
au repos et montre sa source pendant l'édition ; une formule fausse s'affiche en rouge, avec
sa source.

---

## 3. Sélectionner et manipuler

| Geste | Comment |
|---|---|
| **Sélectionner** | clic · `Maj`+clic ajoute · glisser dans le vide dessine une sélection élastique · `Ctrl+A` |
| **Déplacer** | glisser la sélection ; l'aimant aligne sur les bords et les centres des voisins et dessine des guides |
| **Redimensionner** | huit poignées ; les coins gardent le rapport, `Maj` le libère ; `Échap` annule le geste |
| **Dupliquer** | `Ctrl+D` — la copie est décalée de 20 px et sélectionnée |
| **Supprimer** | `Suppr` ou `Retour` |
| **Annuler / rétablir** | `Ctrl+Z` · `Ctrl+Y` ou `Ctrl+Maj+Z` — un geste entier (un glisser, une saisie) est une seule entrée, et la caméra ne bouge pas |

Quand plusieurs éléments se superposent, le clic va au plus précis : une poignée avant un
bord, un bord avant une flèche, puis l'image, la note, le texte, et enfin l'intérieur d'un
conteneur — une membrane ne vole jamais un clic à son contenu.

---

## 4. Les panneaux

| Bouton | Ce qu'il fait |
|---|---|
| **Ordonner** | huit tris et cinq dispositions ; `Appliquer` réarrange les images du tableau, en un geste annulable |
| **Timer** | un Pomodoro qui décompte pour de bon |
| **Domaines** | créer, renommer, colorer, supprimer des domaines ; les assigner à la sélection avec un poids ; chaque nœud porte une jauge par domaine |
| **Storyboard, Preset, Plugins** | les panneaux existent et **disent qu'ils ne font pas encore leur travail** — aucun bouton ne simule une action |
| **Collaborer, Exporter** | pas encore disponibles ; le bouton le dit |

Les panneaux se glissent par leur poignée `⠿⠿` ; les tirer au-delà du bord les ferme.

---

## 5. Enregistrer

| Geste | Comment |
|---|---|
| **Enregistrer** | `Ctrl+S` — la première fois, un dialogue ; ensuite, en silence |
| **Enregistrer sous** | `Ctrl+Maj+S` |
| **Ouvrir** | `Ctrl+O` |
| **Fermer** | la croix pose la question si le document est modifié ; un enregistrement raté ne ferme pas |

Le fichier `.glucose` est binaire, écrit de façon atomique (le fichier précédent survit à un
crash pendant l'écriture), avec une somme de contrôle par section et une seule copie de chaque
image, quel que soit le nombre de fois où elle est posée. Les fichiers de la version Tauri ne
s'ouvrent pas encore.

---

## 6. La fenêtre

| Geste | Comment |
|---|---|
| **Toujours au premier plan** | `Alt+T` — pour garder Glucose au-dessus de Blender, Krita ou Photoshop |

---

<div align="center">

**Glucose, c'est juste poser, relier, zoomer, explorer.**

[← Retour au README](README.md)

</div>
