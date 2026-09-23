# Guide d'utilisation de Glucose

> Ce guide décrit **Glucose Rust tel qu'il est aujourd'hui**, geste par geste. Chaque ligne a
> été vérifiée dans le code. Ce qui n'existe pas encore est dit à la fin, sans détour.
>
> Vous cherchez l'ancienne version, **Glucose Tauri** ? Elle se télécharge dans l'onglet
> *Releases* ; ce guide ne la décrit pas.

**Sommaire** : [Premier lancement](#1-premier-lancement) ·
[Se déplacer](#2-se-déplacer) · [Poser](#3-poser) ·
[Sélectionner et manipuler](#4-sélectionner-et-manipuler) · [Les images](#5-les-images) ·
[Écrire](#6-écrire) · [Relier](#7-relier) · [Organiser](#8-organiser) ·
[Enregistrer et exporter](#9-enregistrer-et-exporter) · [Pas encore là](#10-pas-encore-là) ·
[Aide-mémoire](#aide-mémoire)

---

## 1. Premier lancement

Glucose s'ouvre sur une feuille noire, infinie, quadrillée de points, avec une carte
d'accueil au centre. Tout se pose sur cette feuille.

En haut, la **barre d'outils** ; juste en dessous, les **onglets** des tableaux. En bas à
droite, la **minimap**.

Une règle vaut pour tout le logiciel : **ce qui modifie le document s'annule par `Ctrl+Z`**,
un geste entier à la fois (un glisser, une saisie, un lot d'images déposées).

---

## 2. Se déplacer

| Pour… | Faites… |
|---|---|
| **Zoomer** | la molette de la souris. Le point sous le curseur reste sous le curseur |
| **Se déplacer** | glisser avec le bouton du milieu, ou avec le bouton droit |
| **Au pavé tactile** | glisser à deux doigts pour se déplacer, pincer pour zoomer |
| **Passer en mode main** | `Espace` ou `H`. On y **reste** : `V` ou `Échap` pour en sortir |
| **Voir tout le contenu** | `F` : la vue vole jusqu'à cadrer tout ce qui est posé |
| **Poser un signet** | `Ctrl+1` à `Ctrl+9` : la vue actuelle est retenue dans ce signet |
| **Revenir à un signet** | `1` à `9` : la vue y vole, à la bonne échelle |
| **Voyager par la minimap** | cliquer dedans, ou **maintenir et glisser** : la vue suit en douceur |
| **Changer de tableau** | cliquer un onglet ; `+` crée un nouveau tableau |

Les signets sont propres à chaque tableau et s'enregistrent avec le document. Les chiffres
se lisent sur la touche elle-même : sur un clavier AZERTY, pas besoin de `Maj`.

---

## 3. Poser

### Les outils

Choisissez un outil dans la barre ou par sa touche, puis cliquez sur la feuille. Après avoir
posé, Glucose revient tout seul à l'outil de sélection.

| Outil | Touche | Ce qui naît |
|---|---|---|
| **Sélection** | `V` | rien : c'est l'outil pour prendre et déplacer |
| **Main** | `H` ou `Espace` | rien : c'est l'outil pour se déplacer |
| **Texte** | `T` | une carte de texte, prête à écrire |
| **Note** | `N` | une note adhésive jaune, prête à écrire |
| **Flèche** | `A` | une flèche. Un clic la pose, **un glisser la trace** là où vous voulez |
| **Membrane** | `M` | un cadre en pointillés, nommé « Groupe » |
| **Dossier** | bouton de la barre | un dossier, qui contient un tableau à lui |

Une carte et une note naissent avec un texte provisoire déjà sélectionné : tapez pour le
remplacer.

### Les images

| Pour… | Faites… |
|---|---|
| **Choisir des images sur le disque** | `Ctrl+I`, ou le bouton `Images` |
| **Déposer depuis l'explorateur** | glissez un ou plusieurs fichiers sur la feuille |
| **Déposer depuis un navigateur** *(Windows)* | glissez l'image depuis la page : Glucose la **télécharge** lui-même. Une épingle Pinterest arrive dans sa pleine résolution d'origine |
| **Coller** | `Ctrl+V` : une image copiée, une capture d'écran, ou du texte (qui devient une carte) |

Formats lus : PNG, JPEG, WebP, GIF et BMP.

### Les autres fichiers

Déposer autre chose qu'une image marche aussi :

- un **fichier texte ou de code** devient une carte, son contenu mis en forme ;
- **tout autre fichier**, ou un dossier, devient une **tuile** à son nom. Un double-clic sur
  la tuile ouvre l'explorateur **sur** ce fichier, sans le lancer ;
- un **lien** glissé depuis la barre d'adresse *(Windows)* devient une carte qui porte
  l'adresse, cliquable.

---

## 4. Sélectionner et manipuler

| Pour… | Faites… |
|---|---|
| **Sélectionner** | un clic |
| **Ajouter à la sélection** | `Maj` + clic |
| **Sélectionner une zone** | glisser dans le vide : un rectangle se dessine |
| **Tout sélectionner** | `Ctrl+A` |
| **Atteindre l'élément du dessous** | recliquer au même endroit : chaque clic descend d'un cran dans la pile |
| **Déplacer** | glisser. L'aimant aligne sur les bords et les centres des voisins, avec des guides |
| **Déplacer au clavier** | les flèches `← ↑ → ↓` ; avec `Maj`, dix fois plus loin |
| **Redimensionner** | tirer l'une des huit poignées. Les coins d'une image gardent ses proportions, `Maj` les libère. `Échap` pendant le geste rend la taille de départ |
| **Dupliquer** | `Ctrl+D` |
| **Supprimer** | `Suppr` ou `Retour arrière` |
| **Copier, couper** | `Ctrl+C`, `Ctrl+X` : le texte de la sélection part dans le presse-papiers |
| **Passer devant, derrière** | `Ctrl+]` et `Ctrl+[`, ou le menu du clic droit |
| **Annuler, rétablir** | `Ctrl+Z` ; `Ctrl+Y` ou `Ctrl+Maj+Z` |

**Le menu du clic droit.** Un clic droit *sans glisser* ouvre un menu sur place. Sur une
sélection : dupliquer, verrouiller, retirer les bordures, premier plan, arrière-plan,
supprimer. Dans le vide : coller, tout sélectionner. `Échap` ou un clic ailleurs le referme.

**La barre d'action.** Quand quelque chose est sélectionné, une barre apparaît en bas de
l'écran : le nombre d'éléments, verrouiller, supprimer.

**L'aimant** se coupe et se rallume par le bouton `Aimant` de la barre d'outils.

---

## 5. Les images

| Pour… | Faites… |
|---|---|
| **Faire tourner** | `Alt` + glisser un **coin**. Avec `Maj`, l'angle se cale par huitièmes de tour |
| **Recadrer** | `Alt` + glisser un **côté**. L'image d'origine n'est pas touchée : le recadrage se défait à tout moment |
| **Retirer les bordures** | `Ctrl+B` : les bandes unies (noires, blanches…) autour des images sélectionnées disparaissent, sur tout le lot d'un coup |
| **Verrouiller** | `L` : l'image ne bouge plus et se cerne de rouge. `L` à nouveau pour la libérer |

Le compte-rendu de `Ctrl+B` dit ce qu'il a fait : combien d'images recadrées, combien
n'avaient pas de bordure.

---

## 6. Écrire

### Ouvrir et fermer une carte

| Pour… | Faites… |
|---|---|
| **Éditer une carte ou une note** | double-clic : le mot visé est déjà sélectionné |
| **Aller à la ligne** | `Entrée` |
| **Valider et sortir** | `Ctrl+Entrée` ou `Échap`, ou cliquer ailleurs |
| **Annuler pendant la saisie** | `Ctrl+Z`, mot par mot ; `Ctrl+Y` pour rétablir |

`Ctrl+S` enregistre même pendant qu'on écrit.

### Au clavier et à la souris

| Pour… | Faites… |
|---|---|
| **Sélectionner** | glisser ; double-clic sur un mot ; triple-clic sur un paragraphe |
| **Étendre la sélection** | `Maj` + clic, ou `Maj` avec n'importe quel déplacement |
| **Avancer par mot** | `Ctrl` + `←` `→` |
| **Monter, descendre** | `↑` `↓` : le curseur garde sa colonne |
| **Début et fin de ligne** | `Début`, `Fin` (la ligne visible) ; avec `Ctrl`, tout le texte |
| **Effacer par mot** | `Ctrl` + `Retour arrière` ou `Suppr` |
| **Copier, couper, coller** | `Ctrl+C`, `Ctrl+X`, `Ctrl+V` ; `Ctrl+A` sélectionne tout |

Les accents et les caractères composés passent par le clavier du système : tout ce que
votre disposition sait taper, Glucose le reçoit.

### La mise en forme

Les cartes comprennent le Markdown. Au repos, les signes disparaissent ; pendant l'édition,
ils réapparaissent en gris.

| Écrivez… | Pour obtenir… |
|---|---|
| `# Titre` à `###### Titre` | des titres, du plus grand au plus petit |
| `- élément` ou `1. élément` | une liste à puces, une liste numérotée |
| `> citation` | une citation |
| ` ``` ` sur une ligne, puis le code, puis ` ``` ` | un bloc de code |
| `---` | un séparateur |
| `-# texte` | un petit texte |
| `**gras**`, `*italique*`, `~~barré~~`, `` `code` `` | la mise en forme dans la ligne |
| `[texte](https://…)` | un lien : `Ctrl` + clic l'ouvre dans le navigateur |
| des lignes comme `\| a \| b \|` | un tableau, colonnes alignées |
| `$…$` ou `$$…$$` | une **formule LaTeX** |

Les formules sont rendues par Glucose lui-même, sans navigateur : elles restent nettes à
n'importe quel zoom. Pendant la saisie, leurs délimiteurs sont **verts** si la formule est
juste, **rouges** sinon, et un aperçu s'affiche à côté de la carte. Une formule fausse reste
visible, en rouge, avec sa source.

Seuls les liens `http://` et `https://` s'ouvrent : rien de ce qu'un document contient ne peut
lancer un programme.

---

## 7. Relier

| Pour… | Faites… |
|---|---|
| **Tracer une flèche** | `A`, puis glisser d'un élément à un autre : les deux bouts s'y accrochent |
| **Écrire sur une flèche** | double-clic dessus |
| **Couder une flèche** | sélectionnez-la, puis tirez le losange au milieu d'un tronçon |
| **Retirer un coude** | double-clic sur le coude |
| **Dire ce que la flèche signifie** | sélectionnez une ou plusieurs flèches, puis une touche de `1` à `6` |
| **Retirer ce sens** | `0` |

Les six relations :

| Touche | Relation |
|:-:|---|
| `1` | est précurseur de |
| `2` | contredit |
| `3` | hérite de |
| `4` | inspire |
| `5` | dépend de |
| `6` | illustre |

Quand aucune flèche n'est sélectionnée, les chiffres reprennent leur autre rôle : voler vers
un signet.

---

## 8. Organiser

### Membranes et dossiers

Une **membrane** regroupe visuellement ce qu'elle entoure. Double-clic dessus pour la
renommer.

Un **dossier** est une porte vers un autre canevas. Posé par-dessus des éléments, il les
**emporte** à l'intérieur. Double-clic pour y entrer : la vue plonge dedans. Le **fil
d'Ariane**, en haut, permet de remonter. Un clic pendant la plongée la termine aussitôt.

### Les panneaux

Les panneaux s'ouvrent depuis la barre d'outils et se rangent sur le côté. On les déplace par
leur poignée `⠿⠿` ; les tirer hors de l'écran les ferme.

| Panneau | Ce qu'il fait |
|---|---|
| **Ordonner** | range les images du tableau d'un geste, annulable. Tris : ordre actuel, grand vers petit, petit vers grand, portrait, paysage. Dispositions : rangées compactes, colonnes façon Pinterest, grille alignée, même hauteur. Les options grisées ne sont pas encore construites |
| **Timer** | un minuteur Pomodoro : 25, 15 ou 5 minutes |
| **Domaines** | crée des domaines (des thèmes), les renomme, change leur couleur et leur sigle, les supprime après confirmation. On les assigne à la sélection avec un poids, de 20 % à 100 %, et chaque élément affiche une jauge par domaine |

---

## 9. Enregistrer et exporter

| Pour… | Faites… |
|---|---|
| **Enregistrer** | `Ctrl+S` : un dialogue la première fois, en silence ensuite |
| **Enregistrer sous** | `Ctrl+Maj+S` |
| **Ouvrir** | `Ctrl+O` |
| **Exporter le tableau** | `Ctrl+E`, ou le bouton `Exporter` : en Markdown (`.md`) ou en image vectorielle (`.svg`), selon le nom choisi |
| **Fermer** | si le document a changé, Glucose demande avant de fermer. Un enregistrement raté n'est jamais suivi d'une fermeture |

Le fichier `.glucose` range chaque image **une seule fois**, même posée cent fois. Il est
écrit de façon sûre : un plantage pendant l'enregistrement laisse intact le fichier
précédent.

L'export SVG contient les cartes et les flèches, pas encore les images ni les membranes.

### La fenêtre

`Alt+T` garde Glucose **au premier plan**, au-dessus de Blender, Krita ou Photoshop.
`Alt+T` à nouveau pour revenir à une fenêtre normale.

---

## 10. Pas encore là

Ces boutons existent déjà dans la barre, et **disent eux-mêmes** qu'ils ne font rien
encore. Aucun ne fait semblant.

| Bouton | Où il en est |
|---|---|
| **Storyboard** | le panneau montre ses réglages ; rien ne se dessine encore sur la feuille |
| **Preset** | les gabarits s'affichent ; les appliquer n'est pas encore possible |
| **Plugins** | aucun plugin, aucune IA locale pour l'instant |
| **Collaborer** | pas de travail à plusieurs pour l'instant |

Les fichiers enregistrés par Glucose Tauri ne s'ouvrent pas encore. Les notes adhésives
affichent un opérateur (ET, OU, MAIS, PARCE QUE) quand elles en portent un, mais on ne peut
pas encore le choisir.

---

## Aide-mémoire

| Touche | Action | | Touche | Action |
|---|---|---|---|---|
| `V` | sélection | | `Ctrl+Z` | annuler |
| `H` / `Espace` | main | | `Ctrl+Y` | rétablir |
| `T` | texte | | `Ctrl+A` | tout sélectionner |
| `N` | note | | `Ctrl+C` / `X` / `V` | copier, couper, coller |
| `A` | flèche | | `Ctrl+D` | dupliquer |
| `M` | membrane | | `Suppr` | supprimer |
| `F` | voir tout | | `Ctrl+B` | retirer les bordures |
| `L` | verrouiller | | `Ctrl+]` / `[` | devant, derrière |
| `1`–`9` | aller au signet | | `Ctrl+1`–`9` | poser un signet |
| `1`–`6` | sens d'une flèche | | `0` | retirer ce sens |
| `← ↑ → ↓` | déplacer | | `Ctrl+I` | importer des images |
| `Échap` | sortir, annuler le geste | | `Ctrl+S` | enregistrer |
| `Alt+T` | premier plan | | `Ctrl+Maj+S` | enregistrer sous |
| | | | `Ctrl+O` | ouvrir |
| | | | `Ctrl+E` | exporter |

---

<div align="center">

[Retour au README](README.md)

</div>
