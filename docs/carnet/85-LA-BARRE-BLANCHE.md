# 85 — La barre blanche : la fenêtre sans la barre du système

> Session du 6/11/2026, suite de la fiche 84. Sa demande : *« la barre blanche de Windows pour mettre
> plus petit, plus grand et fermer, ce serait bien qu'elle n'apparaisse tout simplement pas du tout »*.

## 1. Le choix : une fenêtre ordinaire dont la barre n'occupe plus de place

Le geste facile était `set_decorations(false)`, que le mode référence emploie déjà. **Rejeté, et par
une mesure de chez lui** : le 07/10, agrandie et sans cadre, la fenêtre ressemblait à un jeu en plein
écran, et l'application NVIDIA lui a appliqué RTX HDR, toutes ses couleurs changées (fiche 52). Elle
perdrait aussi son ombre et ses animations.

**BARRE-1** garde donc `WS_CAPTION` et répond à Windows comme Chrome et le Terminal :

* `WM_NCCALCSIZE` : la zone cliente monte jusqu'en haut (agrandie, elle descend de la bordure qui
  dépasse de l'écran) ;
* `WM_NCHITTEST` : le vide de la bande rend `HTCAPTION`. Glisser, coller aux bords, double-cliquer,
  `Win + flèches` restent les gestes de Windows, sans une ligne à nous ;
* la bande dit ce qui attend un clic à chaque image ; **un menu ou une question ouverts par-dessus
  coupent le déplacement**, sinon leur clic serait volé.

**Si Windows refuse** la procédure, la barre du système reste et aucun bouton n'est dessiné : jamais
une fenêtre qu'on ne sait plus fermer. **Fermer suit le chemin de la croix** : un document modifié
pose sa question. Les trois boutons sont monochromes (`style.md`) : la croix ne vire pas au rouge.

## 2. Ce que ça coûte

* **Sur Windows seulement** : Linux et Mac gardent ce que leur système décide. Son niri n'a de toute
  façon pas de barre. `GLUCOSE_CADRE` les montre ailleurs, pour regarder leur dessin.
* **La barre réserve 138 pixels** : entre 900 et 1 040 pixels environ, les outils passent au rail plus tôt.
* **Pas de menu d'agencement de Windows 11** au survol du bouton d'agrandissement : il demanderait
  `HTMAXBUTTON` et de refaire le survol en messages de cadre. Glisser vers un bord et `Win + Z` restent.
* **Hors de la table des commandes** : ces gestes sont ceux de la fenêtre, et `Alt+F4` reste celui de
  Windows.

## 3. Ce qui se prouve sans Windows

Cette machine n'a plus Windows, et la leçon de la fiche 61 est connue : un code de plateforme qui ne
tourne jamais ment. Donc :

* **`tests/cadre_suite.rs`, sur la vraie CI Windows** : le vrai programme ; `WS_CAPTION` gardé ; la
  barre partie de la mesure ; le titre rend `HTCAPTION`, fermer et le canevas `HTCLIENT` ; agrandir,
  restaurer, réduire, fermer par de vrais messages de souris, et une sortie propre.
* **Une image** : la CI lance Glucose, capture l'écran et la dépose en artefact (`glucose-sous-windows`).
* **Sur cette machine** : 2 707 épreuves vertes, 7 ignorées ; sabotages `barre-1.json`, 5 sur 5 tombés ;
  le dessin des boutons regardé (`GLUCOSE_CADRE=1`).
