# 39 — Le registre de Tauri

> **Rôle de ce document.** Le 25/09, il a donné ce qu'il appelle *« EXTRÊMEMENT utile »* : le
> relevé, sur un tableau de Glucose Tauri, de **tout ce qui ne va pas** dans Tauri — des défauts,
> des gestes qui manquent, des idées. *« Il faut aussi éviter de les reproduire sur Rust, et tout
> corriger. »* Ce registre les garde, fidèlement, un par un, avec ce que Glucose Rust en fait.
> Il se tient à jour : chaque ligne dit son état, et le commit qui l'a changé.
>
> **Source** : une capture de son tableau (8 000 × 3 926), lue en pleine résolution, morceau par
> morceau. Ses mots sont gardés entre guillemets là où la reformulation risquerait de trahir.
>
> **États** : *à vérifier* (pas encore regardé dans Rust) · *absent de Rust* (le défaut ne s'y
> reproduit pas, vérifié) · *présent* (il s'y reproduit) · *corrigé* (commit) · *à discuter*
> (demande sa parole).

---

## 1. Les ancres de texte des flèches — un mot désigné par son texte, pas par sa place

Deux cartes : « bonjours / test / test / bonjours » et « aurevoire / test / test / aurevoire »,
une flèche entre elles. Dans le panneau « Source — sélectionnez le texte exact », il sélectionne
**le premier** « bonjours » : **les deux** s'allument. Au survol de la flèche, seul le premier
brille. Puis un second défaut : cette fois, seul le premier mot est pris.

*« Le problème vient du système de sélection qui prend les mots à part entière et pas leur…
token identitaire. Il faudra complètement analyser, comprendre et penser à une solution. Je
propose le truc du token, mais ce n'est absolument pas la seule solution : à toi d'imaginer. »*

**État** : *à vérifier*. Tauri avait tenté la correction (`utils/textAnchors.ts` : position +
citation + contexte, le modèle du W3C Web Annotation) ; son essai montre qu'elle ne tenait pas de
bout en bout. Relève du chantier des flèches.

## 2. L'alignement intelligent

Il ne fonctionne que pour les post-it, le texte et les images. *« J'aimerais bien continuer pour
le scale et le positionnement des membranes, ainsi que pour les folders. »* Et reprendre toute
la mécanique, *« qui n'est pas super bien codée »*, pour qu'elle soit optimisée et propre.

**État** : *à vérifier*.

## 3. L'alignement, au premier placement aussi

*« Lorsqu'on est en mode édition, on peut activer le snap intelligent […] mais il ne s'active
qu'une fois qu'on édite le placement. Pourquoi pas DÈS qu'on souhaite réellement placer une
première fois — genre on a créé notre texte, on attend juste de le positionner ? »* Pareil pour
tous les éléments : membrane, dossier, note, texte.

**État** : *à vérifier*.

## 4. Les panneaux Plugins, Preset, Domaines se superposent

Ordonner, Timer, Storyboard apparaissent comme des pages, avec un système de bascule et des
poignées. Il veut la même chose pour Plugins, Preset et Domaines, **mais en haut à gauche** :
des tiroirs qui descendent, qu'on fait **remonter** pour les fermer (l'inverse du bas à droite).
*« Attention à ne pas faire de duplicata : je veux QUE des poignées, surtout pas de bouton
fermer »* — et il reste aujourd'hui un petit « x » à enlever.

**État** : *à vérifier*.

## 5. Sa vision des plugins — à écrire en `.md`

Le bouton « choisir un texte ou une IA » ne devrait exister que **dans** le plugin. Sa vision :

* la page Plugins a, tout en haut, un bouton **« Installer un plugin »** ;
* au départ, **absolument rien** n'est préinstallé, aucun affichage particulier ;
* installer un plugin crée un **tiroir** : on y retrouve ses paramètres, et les boutons qu'il
  crée lui-même ;
* des systèmes **entre plugins**, qui disent d'eux-mêmes *« je PEUX servir aux autres »* — par
  exemple, pour le plugin d'IA, tout le système : installer Ollama, choisir la bonne IA ;
* *« l'idée, c'est que les plugins se connaissent bien entre eux et interagissent au maximum
  ensemble, pour avoir un maximum d'optimisation »*.

**État** : *à écrire* (un document de vision, à sa demande), avant la phase 7.

## 6. Les icônes en couleur

Le message « Alignement intelligent désactivé » porte un aimant en couleur. *« Pas d'icône en
couleur, juste en SVG. »*

**État** : *à vérifier*.

## 7. Le texte arrive en retard quand la vue bouge

*« Je bouge le canva : les images bougent avec, instantanément, mais le texte prend du temps à
s'afficher au bon endroit — comme en retard. »* Ce qui bouge instantanément : image, membrane,
dossier, icône, flèche. *« Il y a vraiment QUE le texte. »* Il le soupçonne plus grave qu'il n'y
paraît.

**État** : *à vérifier* (dans Tauri, le texte est une couche HTML à part ; dans Rust, il se peint
dans la même image — à confirmer à l'écran).

## 8. L'ordre de priorité au clic

Des images dans une membrane : cliquer sur une image sélectionne **la membrane**. Il demande un
système complet d'ordre de priorité, **selon la place de la souris** :

* au centre d'une image posée dans une membrane → l'image ; vers les arêtes, sur le pointillé →
  la membrane ;
* en plein dans un texte posé dans une membrane → le texte ;
* une image **sous** un texte : aujourd'hui, même la souris sur le texte sélectionne l'image —
  c'est faux ;
* **recliquer sans bouger** passe à l'élément suivant dessous (la membrane, puis l'image, puis
  le texte…), *« à ordre suivant, puis suivant »* ;
* le texte est souvent le dernier de l'ordre : un double-clic l'ouvre en édition, et la bascule
  ne marcherait plus ;
* le même ordre pour les flèches, les notes, les dossiers ;
* **les poignées** de redimensionnement : trop petites, il faut viser *« PILE »*. Il veut une
  zone de clic **beaucoup plus grande**, prioritaire ; cliquer ailleurs quitte le mode
  modification.

**État** : *à vérifier* (Rust porte déjà une priorité, `hit_priority.rs` ; à confronter à ce
cahier point par point).

## 9. Ollama et le plugin d'IA

Ollama installé n'était pas vu avant d'avoir relancé le logiciel ; le bouton « installer
qwen2.5:32b » ne marchait qu'après relance. Après avoir choisi un texte, le bouton **« Lancer »**
reste grisé, impossible à utiliser.

**État** : *à vérifier* (phase 7).

## 10. Le bouton Trans-domaines ne sert à rien

Coché ou décoché, rien ne change. *« Explore tout le code qui y fait référence et dis-moi
exactement tout ce qui touche à transdomaine, et ce que vient faire ce bouton. »*

**État** : *à expliquer*. (Fiche 29 § 3.1 : il avait déjà démenti la fonction de Tauri — masquer
les flèches entre domaines sans domaine commun.)

## 11. L'ordre des boutons de la barre

Mettre **Aimant** plus loin, et regrouper Ordonner, Timer et Storyboard — *« pour une question
esthétique uniquement »*. (Aujourd'hui : Ordonner, Timer, Aimant, Trans-domaines, Storyboard.)

**État** : *à vérifier*.

## 12. La Time Machine

Le bouton « compacter » et le bouton « marquer un jalon » : *« pas terrible »*. Elle enregistre
beaucoup de choses inutiles, de manière peu optimisée : revoir le script pour enregistrer le
minimum. « Compacter » réduit **tout** à une sauvegarde : *« je veux vraiment l'enlever »*. Les
jalons : **oui**, avec la date exacte. Et un bouton « optimiser » à repenser : choisir les
**grandes étapes**, garder l'essentiel des moments où les choses bougent — *« architecturalement
et mathématiquement définir une optimisation, et garder une quarantaine d'étapes clés »*.

**État** : *à vérifier* (la Time Machine de Rust est neuve, fiche 37 § 4 — à confronter).

## 13. Les membranes — l'idée de Mary

Trois modes : **classique**, **minimisée**, **étirée**.

* **Étirée** : zoomer assez sur une membrane l'étire jusqu'à remplir l'écran avec son contenu ;
  dézoomer la ramène, et le reste réapparaît. Comme les dossiers, mais sans entrer dans un autre
  espace : un **focus**, sur un fond à la couleur de la membrane. Un bouton d'**auto-étirement** :
  poser une image hors de la membrane l'étire dans la scène entière. Si l'étirement passe
  par-dessus un autre bloc, qui deviendrait membre sans l'avoir voulu : un **avertissement** —
  *« la membrane vient de passer sur un élément, pouvez-vous d'abord le placer ailleurs »* —,
  avec l'élément entouré, et un zoom sur lui.
* **Minimisée** : la membrane garde sa taille dans le canevas ; en focus, rien ne change ; hors
  focus, son contenu est simplement plus petit. Un élément glissé dedans prend
  instantanément la bonne taille, avec une animation fluide, et redevient grand en sortant.
  Les poignées : la membrane se **dézoome** d'abord jusqu'à l'échelle 1, puis s'étire. Une
  échelle **par direction** (longueur 0,4, largeur 0,6 : on garde la plus courte) — sinon une
  image ne pourrait pas regrossir quand on n'étire que dans un sens.
* On passe de minimisée à étirée et inversement, **jamais** d'une membrane spéciale à une
  classique ; l'avertissement vaut **toujours** quand quelque chose est sur le chemin.

**État** : *à discuter* — il a demandé qu'on n'y touche pas sans en parler (fiche 38 § 8), puis a
préféré commencer par les flèches.

## 14. Les rideaux

En focus sur une membrane, un bouton (à côté de minimiser et étirer) crée un **rideau** : un
canevas personnel, comme un dossier, qui ne se voit **qu'en focus** sur cette membrane — une
fenêtre flottante. On bascule entre le focus et le rideau (*« 1/10ᵉ du rideau »*) par une
animation fluide. En collaboration : chacun crée les siens, plusieurs ; on peut définir qu'un
rideau n'est **qu'à soi** — personne d'autre ne le modifie. *« Envoie-moi des références de ce à
quoi tu imagines ce logiciel, je te dis si on s'est bien compris — perso, je n'ai pas de
référence. »*

**État** : *à discuter* (avec les membranes).

## 15. Recopier une image

*« Problème de re-copie image »* — sans détail sur le tableau.

**État** : *à comprendre* avec lui.

## 16. Les niveaux de boards

*« Problème niveaux board TvT !!!!! »*, près d'une barre d'onglets qui ne montre que « Board
principal ».

**État** : *probablement corrigé* par BOARDS-1 (fiche 38 § 5 : les onglets sont les tableaux
racines, un dossier ouvert allume son onglet) — *à confirmer* avec lui.

## 17. La prévisualisation de l'écriture

*« Pré-visualisation à refaire complètement, au propre, et problème d'écriture. »* La capture
montre l'édition d'une carte : le texte brut s'écrit **par-dessus** la carte voisine, et un
panneau « Prévisualisation en direct » s'ouvre à côté.

**État** : *à vérifier* (Rust édite dans la carte elle-même, sans panneau à part).

## 18. Une carte « aaaaaaaaaaaa »

Une carte verte, en titre, sans commentaire. **État** : *à comprendre* avec lui.

---

**Retour** : [`00-INDEX.md`](00-INDEX.md)
