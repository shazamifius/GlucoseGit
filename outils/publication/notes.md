**Glucose {version}** — Glucose, réécrit en Rust.

**Ce qui change depuis la 2.0.3-beta.7**

* **Sous Windows, la barre blanche disparaît.** La bande de Glucose monte jusqu'en haut de la fenêtre : on
  la glisse pour déplacer la fenêtre, et elle porte réduire, agrandir et fermer. Coller aux bords de
  l'écran, double-cliquer pour agrandir et `Win + flèches` restent les gestes de Windows.
* **Beaucoup de textes proches les uns des autres ne ralentissent plus autant.** La teinte d'un texte
  dépend de ses voisins, et tout se recalculait d'un coup à chaque changement : plus d'une seconde pour
  une image sur un très grand tableau de textes. Elles se recalculent maintenant par petites tranches,
  sans jamais arrêter l'image ; une teinte peut donc arriver à sa valeur exacte quelques images plus tard.
* **Sélectionner beaucoup de nœuds ne coûte plus davantage à chaque image.** Le dessin parcourait toute
  la sélection pour chaque nœud visible.

**Encore connu** : un tableau de plusieurs milliers d'images entièrement à l'écran reste lent, surtout
pendant un focus (`F`, signets). C'est le prochain chantier.
