**Glucose {version}** — Glucose, réécrit en Rust.

**Ce qui change depuis la 2.0.3-beta.8**

* **Un très grand tableau vu en entier est bien plus fluide.** Avec des milliers d'images à l'écran, Glucose refaisait le même travail pour chaque photo à chaque image, quand seule la vue bougeait. Il ne le refait plus tant que rien ne change.
* **Moins d'à-coups pendant un zoom sur un grand tableau.** Ce qui quitte la mémoire ne bloque plus le dessin.
* **Le journal technique part à la fin de la session**, et non plus au lancement suivant, pour ceux qui ont dit oui : une session sur un ordinateur où l'on ne revient pas arrive aussi. Il contient la même chose qu'avant.
* **Le journal dit aussi ce que l'ordinateur fait** pendant que Glucose dessine (en pourcentages), et combien de nœuds a le tableau.

**Encore connu** : sélectionner plusieurs milliers d'images d'un coup reste lent à chaque image.
