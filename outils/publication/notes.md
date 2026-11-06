**Glucose {version}** — Glucose, réécrit en Rust.

**Ce qui change depuis la 2.0.3-beta.6**

* **Un pilote de carte graphique défaillant n'empêche plus Glucose de s'ouvrir.** Sur certaines
  machines, le pilote fermait Glucose avant la première image, à chaque lancement. Glucose retient
  désormais le chemin qui l'a fait tomber et en prend un autre au lancement suivant, jusqu'à
  dessiner sans la carte graphique s'il le faut. Une nouvelle version vérifie discrètement si
  l'ancien chemin est redevenu sûr.
* **Au téléphone, un très grand tableau vu en entier ne ferme plus Glucose.** Sur un tableau de
  6 000 images, « Tout cadrer » épuisait la mémoire et Android tuait l'application. Réparé et
  vérifié sur un téléphone virtuel, avec les mêmes gestes : Glucose survit et les 6 000 images se
  dessinent.
* **Glucose finit proprement quand le système le ferme** : éteindre l'ordinateur, Windows Update,
  ou la fermeture d'une session sous Linux. Il écrit tout, sans rien demander, au lieu d'être tué.
* **Le nœud de bienvenue se voit en entier au premier lancement** : son titre se cachait sous la
  barre d'outils.
* **Sous Linux avec Wayland**, OpenGL fonctionne quand Vulkan n'est pas utilisable.
* **Sécurité** : mise à jour de `rustls`, la brique des connexions internet hors Windows. Les
  quatre failles corrigées ne concernaient pas la façon dont Glucose s'en sert.

**Bon à savoir** : si le pilote de ta carte fermait déjà Glucose, la mise à jour se cherche avant
d'ouvrir la carte. Relance Glucose une fois ; s'il se ferme encore, relance-le une seconde fois.
