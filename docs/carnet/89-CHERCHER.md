# 89 — Chercher : `Ctrl+F`, et le résultat suivant

> Session du 7/11/2026, après la fiche 88. Sa parole, ce soir-là : *« Ctrl+F, ça me va
> entièrement »*. SUITE le voulait ainsi : *« chercher dans tout le document — textes, titres, noms de
> fichiers —, voler d'un résultat au suivant ; sur le noyau, pour tenir 10⁷ nœuds »*.

## 1. Ce qui existait

* **Glucose Tauri** avait une fenêtre au milieu de l'écran (`SearchPanel.tsx`) : les noms des
  tableaux, les étiquettes des images, le texte des nœuds ; vingt résultats, Entrée allait au premier.
* **Glucose Rust** n'avait rien : `Ctrl+F` lui était gardé depuis le 13/10 (tout cadrer est passé à
  `Alt+F`). Un agent cherchait déjà (`glucose_lire`, `cherche`, fiche 79), sans les accents.

## 2. La recherche (consultée le 7/11/2026)

* **Peu de canevas le font bien** : les utilisateurs d'[Obsidian](https://forum.obsidian.md/t/local-find-search-within-canvas/51936)
  et de [Figma](https://forum.figma.com/t/jumping-to-a-specific-page-frame-using-a-search-box/347/31)
  le réclament ; [Spine](https://docs.getspine.ai/app/basics/quick-search) ouvre une fenêtre au milieu
  de l'écran par `Ctrl+F`, et un clic va au bloc.
* **Chercher sans les accents** est la règle des moteurs de recherche : décomposer, retirer les marques,
  puis mettre en minuscules ([le manuel Unicode du développeur](https://unicodefyi.com/series/developers-handbook/search-that-works/),
  [OpenSearch](https://aws.amazon.com/blogs/big-data/perform-accent-insensitive-search-using-opensearch)).
  Sans table Unicode, la table du latin s'écrit à la main.
* **Pour des millions de nœuds** : l'index de trigrammes de [Russ Cox](https://en.wikipedia.org/wiki/Trigram_search)
  (Google Code Search). Pas aujourd'hui : il viendra sans changer ce que la recherche rend.

## 3. La conception

* **La palette, une autre source** : `Ctrl+F` ouvre la même question que `Ctrl+Maj+P` (le champ, la
  fenêtre qui défile, les flèches, Page haut et bas, la molette, Entrée, Échap), dont les rangées sont
  des nœuds — comme `Ctrl+P` et `Ctrl+Maj+P` dans VS Code. Rien n'est écrit deux fois.
* **PLI-1** (`glucose_core::recherche::pli`) : sans majuscules ni accents, tout le latin (« ete »
  trouve « Été », « oeuvre » « Œuvre », « lodz » « Łódź »). La palette des commandes y passe aussi, et
  un agent qui cherche (`glucose_lire`) : **une seule loi**. Chercher n'est pas nommer : un nom de
  domaine garde ses accents (DOM-4), parce qu'il désigne.
* **TROUVE-1** : chaque mot cherché doit être dans le texte, dans n'importe quel ordre ; le début d'un
  mot passe devant (« eau » trouve « l'eau » avant « bureau ») ; à égalité, l'ordre du document. Ce
  qu'on cherche : les noms des tableaux et des dossiers, le texte des nœuds (textes, post-it, titres de
  membranes, ce que disent les flèches), les étiquettes et l'adresse des photos.
* **TROUVE-2** : Entrée va au résultat — son onglet, même rangé dans un dossier, le nœud choisi, la
  caméra qui vole jusqu'à lui — sans rien écrire au journal. **`F3`** va au suivant et **`Maj+F3`** au
  précédent, en boucle, sans rouvrir la palette, dans le document tel qu'il est devenu.
* **Au doigt** : « Chercher… » au menu du vide.

## 4. Le chiffre

Un document de 24 000 textes d'environ deux cents lettres (la taille de son tableau au téléphone),
`examples/bench_recherche` :

| | la première version | l'index |
|---|---|---|
| une lettre tapée | 110 à 218 ms | **4 à 8 ms** |
| refaire l'index (une fois par version du document) | — | **23 ms** (109 avant le chemin direct de l'ASCII) |

La première version relisait et repliait tout le document à chaque lettre, et calculait la phrase de
20 000 résultats pour en montrer dix. **L'index** plie une fois par version du document ; la phrase ne
se calcule que pour les rangées montrées ; une lettre ASCII se met en minuscule sans la table d'Unicode.

**Limite assumée** : sur un très gros document, la première lettre après un changement paie l'index
(23 ms à 24 000 textes, bien moins d'une milliseconde sur ses documents ordinaires). Le faire sur un fil
à part viendra si un usage réel le demande.

## 5. Prouver

* **Le noyau** : PLI-1 (le latin, l'accent écrit à part, ce qui n'est pas latin), TROUVE-1, l'index qui
  rend ce que rend une recherche d'un coup et ne voit pas ce qui change après lui.
* **L'application** : `Ctrl+F` qui trouve dans tous les onglets et dit combien ; Entrée qui va au
  résultat sans rien écrire ; `F3` en boucle ; le document qui change entre deux `F3` ; un résultat
  rangé dans un dossier ; l'index qui suit le document ; les raccourcis ; le menu du vide.
* **Sabotages** (`outils/sabotages/chercher-89.json`) : voir ÉTAT.

## 6. VOL-1 — trouvé en relisant, le 8/11

La session suivante a relu ce chantier avant de l'enregistrer : **une flèche trouvée par ce qu'elle dit
ne faisait pas voler la vue**. Entrée la choisissait, puis rien — pas même une image redessinée. « Voler
jusqu'à un nœud » était écrit trois fois (ici, le panneau Domaines, l'éditeur d'ancres), chacune par
`node_rect`, la boîte où une flèche **s'accroche** ; une flèche n'en a pas. Une seule fonction désormais
(`GlucoseApp::voler_vers_le_noeud`), qui vole vers l'**emprise** du nœud (`glucose_core::arrow::emprise` :
sa boîte, ou l'enveloppe de l'encre d'une flèche — ce que `F` cadre déjà). Les épreuves
`entree_vole_jusqu_a_une_fleche` et `l_emprise_d_une_fleche_est_son_encre` la gardent ; la suite entière
(2 865 épreuves) et clippy passent. Ses sabotages se jouent avec ceux de la fiche 90.
