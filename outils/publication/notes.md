**Glucose {version}** — Glucose, réécrit en Rust.

**Si vous venez de la 2.0.4-beta.1 ou d'avant : mettez à jour toutes vos machines.** Depuis la 2.0.4-beta.2, Glucose écrit vos documents dans une forme plus récente (une image peut être retournée) : les versions d'avant disent « ce document vient d'une version plus récente de Glucose » et n'y touchent pas. Rien n'est perdu — il s'ouvrira dès qu'elles seront à jour. Au téléphone, installez l'APK **par-dessus** l'ancien : ne désinstallez jamais Glucose, vos brouillons vivent dans l'application.

**Ce qui change depuis la 2.0.4-beta.5**

* **Les flèches** : une flèche vers une membrane va droit à son bord, sans zigzaguer entre ce qu'elle contient. Les flèches d'un texte qu'on est en train d'écrire partent du bord qu'on voit, pas de l'ancien. Une flèche se choisit toujours là où elle se dessine : plus de zone de clic invisible après un glisser ou un rangement.
* **Écrire dans un texte** : choisir une partie du texte à la souris ne fait plus sortir du bloc ; la vue suit la ligne qu'on écrit quand elle dépasse l'écran.
* **Les membranes** : dans le vide d'une membrane qu'on n'a pas choisie, glisser trace le rectangle de sélection ; un clic la choisit. Son bord ne se prend que là où il se voit : un texte posé dessus se choisit et s'ouvre. Une membrane qu'on trace prend ce qu'elle entoure.
* **Ranger dans un dossier** : glissez des nœuds sur un dossier, ils y entrent (l'archive, par exemple) ; `Ctrl+Z` les rend.
* **Les tableaux Markdown** serrés dans un nœud étroit ne coupent plus que le mot trop long : les autres colonnes restent lisibles.
* **La pipette** : armée, la croix vise et la roue montre la couleur qu'elle prendrait ; viser la lueur d'un texte coloré prend sa couleur ; une vraie icône.
* **Le canevas partagé** : la première fois qu'on partage ou rejoint, Glucose demande votre nom et votre couleur. « Rejoindre… » ouvre un champ où coller ou taper le code — sur le téléphone aussi. Un Glucose fermé pendant qu'un de ses gestes voyageait encore rejoint bien sa salle à la relance.
* **Les IA qui travaillent dans votre document** : elles déplacent comme la main (une membrane emporte son contenu), répondent à un nœud en un geste, et lisent le document plus sobrement.

**Encore connu** : dans un canevas partagé, une photo dont personne de connecté n'a les octets reste grise jusqu'au retour de quelqu'un qui les a ; la Time Machine d'un appareil qui a rejoint un canevas ne montre que les gestes faits depuis qu'il l'a rejoint ; sous Windows, `F11` ne met pas encore en plein écran ; une famille d'émojis (👨‍👩‍👧) s'affiche en personnes côte à côte, et un drapeau en deux lettres ; sur les très grandes cartes, le zoom a encore des à-coups.
