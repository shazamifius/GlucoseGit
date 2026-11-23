**Glucose {version}** — Glucose, réécrit en Rust.

**Si vous venez de la 2.0.4-beta.1 ou d'avant : mettez à jour toutes vos machines.** Depuis la 2.0.4-beta.2, Glucose écrit vos documents dans une forme plus récente (une image peut être retournée) : les versions d'avant disent « ce document vient d'une version plus récente de Glucose » et n'y touchent pas. Rien n'est perdu — il s'ouvrira dès qu'elles seront à jour. Au téléphone, installez l'APK **par-dessus** l'ancien : ne désinstallez jamais Glucose, vos brouillons vivent dans l'application.

**Ce qui change depuis la 2.0.4-beta.4**

* **Discuter avec une IA gratuite dans Glucose** : Collaborer → « L'API publique… » installe une clé Google AI Studio (gratuite) et teste les modèles gratuits ; le tiroir DISCUSSION lui parle, et elle peut lire et écrire dans votre document. Quand un modèle atteint sa limite, Glucose passe au suivant et vous le dit, puis revient au meilleur dès qu'il est de nouveau libre.
* **Un canevas partagé le reste après un redémarrage** : Glucose rejoint sa salle tout seul au lancement, sans retaper le code.
* **Les dossiers** : le rectangle de sélection les prend aussi, `Maj`+clic en ajoute un, et plusieurs dossiers se choisissent ensemble. Renommer un dossier (`F2`) renomme aussi ce qu'on voit en y entrant, et ce que `Ctrl+F` trouve.
* **L'opacité d'un dossier, à votre choix** : quand un dossier est choisi, la roue de couleur montre un second curseur, « opacité ». Une version plus ancienne qui partage le même canevas montre ce dossier en gris jusqu'à sa mise à jour.
* **Une flèche se tire à l'intérieur d'une membrane** : on vise une membrane par son bord, et son intérieur reste à vous.
* **Les tiroirs restent dans l'écran** : quand la hauteur manque, ils se partagent la place, et chacun fait défiler à la molette ce qu'il ne montre plus. À 110 % d'interface, la Time Machine ne perd plus un jalon sur deux, et le Storyboard montre toutes ses cases.
* **Les IA qui travaillent dans votre document** (par la porte, ou `glucose --mcp`) : elles voient vos dossiers et peuvent les renommer ; une lecture n'attend plus jamais que vous ayez fini un geste ; « Toujours, dans ce document » leur donne l'accès une fois pour toutes, même après un redémarrage, et vous pouvez le retirer.
* **Sous Windows**, une mise en veille n'est plus comptée comme un gel dans le journal technique.

**Encore connu** : dans un canevas partagé, une photo dont personne de connecté n'a les octets reste grise jusqu'au retour de quelqu'un qui les a ; une famille d'émojis (👨‍👩‍👧) s'affiche en personnes côte à côte, et un drapeau en deux lettres ; sur les très grandes cartes, le zoom a encore des à-coups.
