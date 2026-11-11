# Ce que Glucose veut devenir — ses paroles

> **Le seul endroit où vivent ses paroles et la charte.** Chaque phrase en italique est la sienne,
> datée. Ce qui n'est pas en italique les range ; une interprétation se dit telle. Quand deux de ses
> paroles se contredisent, **la plus récente l'emporte**, et l'ancienne est barrée ou dite remplacée.
> Rangé par thème le 11/11/2026 (fiche 104) ; ses mots, dans l'ordre où il les a dits, restent dans les
> fiches du carnet.
>
> Tout se remet en question, ses paroles comprises — avec des arguments.

---

## 1. Le but

**Remplacer entièrement la feuille de papier** (15/10/2026) : *« En 2026, une feuille de papier est et
reste indispensable. Mon défi : créer un système qui s'adapte à la technologie, pour les étudiants, les
développeurs, les artistes, les joueurs, ou qui que ce soit. »* Le canevas infini n'est pas la fin :
c'est la feuille, enfin sans bord.

**Le pont entre l'humain et l'IA** (22/10) : *« Depuis le début, Glucose visait à faire le pont entre
l'humain et l'IA. »* **Un Glucose doit pouvoir être lu par un humain ET par une IA, de quasi la même
façon** — *« ATTENTION : impérativement »*. Ce qu'un humain voit, une IA doit pouvoir le lire, et
l'inverse.

**Quatre objectifs**, dans l'ordre où il les a dits (15/10) :

1. **Le premier vecteur d'échange d'humain à humain** : tout Wikipédia dans Glucose, et beaucoup de
   bases de données — mais un humain ordinaire doit pouvoir **le consulter de façon tout à fait
   naturelle**, par la disposition, les domaines, les flèches, les membranes et la couleur. *« Autant te
   dire que c'est un défi super compliqué. »* Et pas seulement Wikipédia (22/10) : *« Pourquoi se limiter
   à Wikipédia ? Certains datasets d'IA sont super utiles, MÊME pour un humain : les avoir sur Glucose, ça
   doit pouvoir être incroyable. »*
2. **Le premier vecteur d'IA à IA** : l'idée de **Protéo** (un projet à côté, en pause depuis le 25/10) —
   donner à des IA un socle *« immensément meilleur que de simples jetons dans le vide »*. Un Glucose que
   tous les Protéo remplissent ensemble, où un Protéo **apprend** (22/10) : *« pour que Protéo lui-même
   démonte Maya, le comprenne — en fait il va apprendre plein de trucs avant de le remonter pour finaliser
   l'addon ; tout ça peut être écrit sur Glucose. »*
3. **Le premier vecteur d'humain à IA et d'IA à humain** : des **graphes de questions et de réponses**
   plutôt qu'un fil de discussion ; des IA qui remettent un projet en forme (exemple : un plugin qui
   comprend une langue inventée et propose des mots neufs cohérents).
4. **Réinventer le document linéaire de physique et de mathématiques** pour la recherche : *« Que Glucose
   soit extrêmement performant pour la recherche et le développement, d'IA à IA comme d'humain à humain ;
   qu'il soit utilisé par les plus grands ingénieurs du monde. »*

**Pour les IA** (22/10) : savoir **où se situe quoi** dans la carte (la géographie, pas seulement la
liste des nœuds) ; *« un système de déplacement et de recherche ULTRA avancé pour les IA, pour qu'elles
retrouvent ce dont elles ont besoin directement, sans trop galérer — attention : Glucose est là pour faire
des recherches, mais des fois on ne sait pas ce qu'on recherche »* ; une IA peut être le
sous-agent d'une autre, avec peu de mémoire et peu de temps. **Un usage qui attend** : l'arbre complet
d'un projet Rust de plusieurs millions de lignes (web3) — *« une arborescence mathématique super
complexe »* —, construit par Protéo, posé sur Glucose, puis travailler sur ce projet avec Protéo, depuis
Glucose.

**La partie 1 d'abord** (25/10) : *« mettre une pause à Protéo et essayer de définitivement finir la
partie 1 de Glucose : le logiciel complet, complet, en Rust, ultra optimisé, avec des utilisateurs et une
ergonomie parfaite. »*

## 2. La charte — ses exigences permanentes

Chaque décision, chaque ligne de code se juge contre elles.

* **La fluidité** (16 et 18/09) : *« rappelle TOUJOURS avoir 100 fps minimum ; si on est en dessous alors
  go pixeliser tout »* — **100 images par seconde au moins, constant, quoi qu'il se passe** ; au-delà, ce
  que l'écran sait afficher (sa fréquence se lit, ne se suppose pas) ; 60 sur un téléphone courant, 45
  sur un très vieux. Une action lourde a le droit de durer, **en cascade**, par tranches, sans jamais
  faire attendre l'image. **Mais une dégradation qui se voit est un échec** (21/09) : *« l'objectif d'une
  pixelisation et d'une optimisation, c'est qu'elle ne se voie pas à l'œil nu »*. Et le logiciel
  **pré-optimise** au lieu de réagir : *« que mathématiquement le logiciel développe une intelligence et
  comprenne la machine afin de pré-optimiser des rendus et de pas recalculer les fps h24 »* — un modèle
  de coût qui prédit. **Son ordre, le 11/11** : *« 100 ou 140, ce n'est pas trop la question : l'idée,
  c'est avancer sur les fonctionnalités et optimiser ce qui est capital, avancer de manière propre sur le
  dépôt et de manière pro ; si besoin, l'optimisation d'objectifs comme 100 fps minimum viendra après,
  une fois qu'on aura terminé les betas et fini toutes les fonctionnalités »*.
* **Mais pas une église de l'optimisation** (7/11) : *« Glucose n'a pas pour objectif d'être une église
  infime de finesse dans l'optimisation graphique, son but premier n'est pas là »* — le blocage de
  Glucose est le manque de fonctions. **Précisé le 11/11** : *« l'idée n'est pas de construire une église de
  l'optimisation, MAIS pour ce qui est capital et nécessaire, il faut pouvoir optimiser »* — un ralentissement qu'il sent (*« je lag plutôt beaucoup »*) se règle.
  ~~Le chantier nommé le 17/09 : la pixelisation au zoom sur processeur~~ — remplacé par ces deux
  paroles.
* **Aucune machine exclue** (17/09) : *« tous les CPU, tous les GPU, toutes les cartes du monde »* ;
  *« jamais personne ne doit manquer de quoi que ce soit — de fluidité, de contrôle sur le canva,
  d'interface »*. Windows, Linux, macOS (Apple Silicon compris), Android, puis l'iPad par le web, dès la
  conception. Une optimisation qui dépend d'un jeu d'instructions se prend **à l'exécution**, jamais à la
  compilation.
* **Rien de gaspillé** (17/09) : *« à quoi ça sert de se priver de physique et de matériel lorsqu'on le
  possède ? »*, *« utiliser 650 Mo lorsqu'on a 32 Go ? »* — utiliser ce que la machine a, et s'adapter en
  temps réel à ce qu'elle fait déjà.
* **Deux voies, ni pro-CPU ni anti-GPU** (21/09) : *« idée n'a jamais été être réfractaire au GPU, idée
  c'est être réfractaire à ne pas être spécialisé presque parfait sur 1 seul point mais sur TOUS les
  points en même temps »* ; le processeur et la carte graphique sont *« littéralement des scripts
  différents »*. **Interdit** : basculer sur une voie **parce qu'on a échoué** sur l'autre — *« la PIRE
  des mentalités »* (17/09). Glucose **se niche là où il reste des ressources** — *« se mettre là où il y
  a de la place, pour surtout jamais gêner l'utilisateur »* (un artiste fait tourner Blender ou Adobe en
  même temps). La mémoire par étages (23/09) : *« toujours prio la VRAM, ensuite la RAM, ensuite le
  SSD »*, chaque étage borné par ce que la machine laisse **maintenant**. Le bridage thermique d'un
  téléphone est le régime normal : on observe le débit qui baisse, on n'interroge pas la température.
* **Net à l'arrêt** : à l'arrêt, net à quasi 100 % ; pendant le mouvement, un arbitrage à **mesurer**,
  jamais à trancher sur le papier.
* **L'élégance mathématique** : une constante arbitraire qui peut **disparaître** ne se règle pas, elle
  disparaît ; une formule exacte plutôt qu'un flou heuristique. **Viser la perfection** ; un code
  *irréprochable* (pas d'objet-dieu, pas de module mort, pas de bouton qui ment), réécrit au propre dès
  qu'il est vu.
* **Les dépendances** (15/10) : *« oui 1 dépendance reste une dépendance, mais si on ne peut pas faire
  sans, alors on fera intelligemment avec »* — aujourd'hui minimales mais assumées ; **à terme, zéro
  absolu**, chacune réécrite quand on sait faire mieux, jamais au prix d'une machine exclue.
* **Pas de niveau de détail sur le sens** (8/11, pour la carte de Wikipédia) : *« pas de système de LOD
  et pas de simplification possible »* — rien ne se cache, ne se remplace ou ne s'efface selon la
  distance. **Une seule exception** (9/11) : *« c'est la première fois que j'autorise les niveaux de
  LOD »* — **les liens** (§ 6). Le niveau de détail **du rendu des images**, lui (mipmaps, pixelisation
  invisible, décodage progressif), est souhaité.
* **Tout a une signification** (9/11) : *« il faut que TOUT ait une signification »* — une teinte, une
  clarté, une opacité ne varient jamais pour décorer.
* **L'échelle visée** : des **dizaines de millions de nœuds** (10⁷), pas pour tout de suite. Le but de
  Glucose Tauri, à dépasser : *des millions d'images ouvertes sans laguer*.
* **Glucose Tauri, la référence** : visuellement et fonctionnellement, Glucose Rust part de ce que
  faisait Glucose Tauri (les fiches 06 à 10 du carnet le relèvent au pixel), **jamais de son code** — on
  recrée, on ne traduit pas. **Ses paroles plus récentes l'emportent sur Tauri** : la teinte symbiotique
  des textes était *« temporaire »* (11/11), les membranes changent (§ 5).

## 3. Une application mère, ouverte à tous

* **Des centaines de plugins**, et des modifications du code lui-même ([PLUGINS.md](../PLUGINS.md)).
* **Open source, en Rust**, pour que **n'importe qui puisse forker** : l'arborescence doit se lire par un
  inconnu. **Glucose devient lui-même un ensemble de briques** (15/10) : *« qu'il y ait des milliers de
  forks, adaptés à tout besoin »* — le noyau sans dépendance (`glucose-core`) est la première.
* **Les presets sont l'interface entière** (15/10) : la disposition, les couleurs, les raccourcis, toutes
  les fenêtres — **pas** des gabarits de zones. Un preset de Glucose *« extrêmement travaillé, qui n'a
  jamais été créé »*, qui regroupe absolument toutes les fonctions, extrêmement ergonomique, rapide pour
  les professionnels et très propre à l'œil — *« pousser l'outil au maximum dans son ergonomie »* ; un
  preset PureRef, un Figma, un Miro, chacun avec 100 % (ou presque) de ses fonctions et sa façon de faire.
* **L'utilisateur choisit toujours** (15/10) : *« je suis d'accord avec le système qui permettrait à
  l'utilisateur de TOUJOURS CHOISIR ! »* — devant deux réponses légitimes, un défaut argumenté, et le
  choix.
* **Tous les boutons sont obligatoires** (12/10) : *« 100 % des boutons existants, ainsi que TOUTES les
  fonctionnalités à venir, sont quasi obligatoires »* ; un bouton qui ne marche pas se fait marcher,
  jamais ne se retire. **Collaborer est obligatoire** (plus tard).
* **L'ergonomie se fait à la fin** (15/10), une grosse session quand toutes les fonctions seront là ;
  avant, chaque fonction a son bouton, au PC et au téléphone. **Nuancé le 10/11** : *« actuellement le
  Wikipédia est illisible et horrible à travailler dessus »* — il veut travailler l'ergonomie de la carte
  de Wikipédia maintenant.

## 4. Le domaine — sa définition (22/10)

*« Toutes les sessions de code me demandent ce qu'est un domaine : c'est le moment de répondre. »* **Ne
plus jamais le lui demander.**

> **Le domaine, c'est l'idée de créer un fork.** Dans le domaine **eau**, on peut très bien avoir :
> « l'eau, ça mouille » ; « eau = H₂O » ; « l'eau dans les jeux vidéo : il faut des systèmes de noise map
> et de displacement pour créer un effet de 3D ». Ce sont **trois choses très séparées**, dans des
> **blocs de texte distincts**, qui peuvent eux-mêmes être dans des **membranes**, elles-mêmes peut-être
> dans des **fichiers** différents — **mais le domaine, lui, va traverser tout ça : tout fait partie du
> domaine de l'eau.**

* **Sa règle du nom** (7/11) : *« on ne peut pas inscrire deux domaines avec le même nom ; ils doivent
  impérativement demander à l'IA ou à l'humain la spécification ou la spécialisation du mot — ça donnerait
  python code et python serpent. »*
* **Sa visualisation** (22/10) : *« ça va dans la catégorie ergonomie de l'application »*.
* **Trans-domaines** : *« une espèce de vue en perspective 3D de tous les domaines… mais c'est de la R&D,
  je ne suis pas sûr »* ; le bouton dit « bientôt ».
* **Les dimensions de Glucose** (sa piste de R&D) : un espace latent de 4 096 dimensions posé sur un
  canevas 2D deviendrait *« absolument PLUS exploitable »* ; x et y, la couleur (*« autant de couleurs
  visibles que de dimensions »*), la membrane (une dimension, plus une couleur), la flèche (*« une infinité de
  dimensions ? c'est un système qui a énormément d'options »*), le domaine (une infinité de dimensions,
  plus une couleur).

## 5. Les couleurs

### Les membranes

* **Le sens** (9/11) : **la teinte** dit la **proximité des domaines** (*sciences, société et sciences
  sociales ont des couleurs proches*) ; **la clarté**, l'**accessibilité** (un sujet accessible à tous ou
  non) ; **l'opacité**, l'**appartenance** (un sujet qui entre pile dans sa matière est opaque ; un sujet
  qui en dérive, plus transparent). À terme, **une option** : l'IA de Glucose colore d'elle-même, que
  l'utilisateur peut ne pas utiliser. *« Je ne sais pas trop »* (10/11), devant l'image qu'on lui en a
  montrée.
* **Choisir sa couleur** (9/11) : *« si possible le même système de palette que Blender, avec la roue —
  elle est extrêmement ergonomique »*.
* **Plus de couleurs** (9/11, devant une teinte par région) : *« c'est magnifique, c'est presque parfait,
  MAIS tu n'as pas créé assez de couleurs »* ; **pas de clair ou de sombre décoratif**.
* **Revoir les membranes** (8/11) : *« j'accepte de revoir le fonctionnement des membranes afin qu'elles
  soient plus intéressantes »*.
* **Emboîtées** (10/11) : *« plus on met des membranes dans des membranes, moins on voit grand-chose ; on
  arrive à lire, mais… c'est vraiment pas beau »*. **La loi des régions** — chaque endroit prend la
  couleur de la plus petite région qui le contient (fiche 104) : *« je suis d'accord »* (11/11). **Les
  frontières qui grandissent avec la région** : *« trouver quelque chose de plus léger ; sinon oui, je
  suis d'accord »*. **Une membrane sans couleur choisie devient neutre** : *« oui aussi »*. Et (11/11) :
  la carte n'est *« pas encore super lisible, à cause de l'opacité des membranes »*.
* **Plus transparentes** (11/11 au soir, devant les images du halo, où les régions sont déjà en
  carte) : *« non, je trouve que les membranes ne sont pas encore assez transparentes ; il faudrait les
  rendre plus transparentes »*.

### Les textes : un fond toujours gris, un halo qui dit le sens (11/11)

* **Le fond d'un texte** : *« il faudrait abandonner l'idée du background d'un texte : je préfère qu'il
  soit TOUJOURS gris »*. Le geste qui colore un bloc de texte devait changer **le halo lumineux** autour
  du bloc — *« PAS son background ! »*. (Son *« les couleurs de text box ne fonctionnent pas »* du 9/11
  parlait donc du halo ; la fiche 99 a coloré le fond.) Quel gris : *« yep, le gris sombre et neutre des
  blocs d'aujourd'hui »* (11/11).
* **Le halo lumineux** : la *noise map* qui y met des couleurs aléatoires (la teinte symbiotique, héritée
  de Glucose Tauri) *« n'était que temporaire »*. Son idée : *« une intelligence artificielle semi from
  scratch, ultra optimisée et minimale, capable de comprendre une seule et unique chose : est-ce que ce
  texte a un rapport avec ce texte-là, et à combien de % »* — des modèles faits *« uniquement pour des
  % »*, beaucoup plus rapides, petits et optimisés. La règle :
  * des textes qui ont un rapport ont des **couleurs proches** ; si l'utilisateur les **éloigne** dans le
    canevas, cela s'applique *« en mode smooth »* ;
  * si l'IA est **sûre** d'un rapport fort avec un autre texte : garder la couleur, mais appliquer
    **légèrement une moyenne des couleurs des blocs voisins** — *« peu importe la distance »* ;
  * si l'IA **n'est pas sûre**, et qu'ils sont vraiment distants : **pas** de liaison de couleur ;
  * *« et du coup le système d'opacité et de niveaux de blanc ou noir prend sens ! Avant, il n'était pas
    utilisé »*.
* **Ses choix, devant l'image du halo** (11/11 au soir,
  [la page](https://claude.ai/artifact/ADmG6kEagEFNuRHprpRVhY)) : les couleurs **par familles** (des
  textes que l'IA juge parents sans doute possible partagent une teinte, les familles voisines des
  teintes voisines, un texte sans parent sûr reste gris) ; le halo à **76 au repos, 102 au survol** (il
  était à 38, invisible sur une membrane colorée) ; l'**opacité dit la sûreté de l'IA**, la **clarté
  l'accessibilité** — *« parfaitement d'accord »*. Sur la couleur qu'on choisit soi-même pour un texte, et
  ce que l'IA en fait : *« c'est là le truc le plus complexe »* (§ 8 bis).
* **Une utilité à la couleur des textes** (9/11) : *« c'est toute une question aussi »* — au sein d'une
  membrane il y a souvent du texte, et du texte de couleur : il faut lui proposer une signification.
* ~~La couleur d'un texte dit le domaine dont il parle (*« j'avoue, je n'en ai aucune idée ; je partage ton
  avis, je pense »*, 10/11)~~ —
  remplacé par l'idée du halo ; sur la question « textes sans domaine neutres, liens gris » : *« je n'ai
  pas trop idée, je n'ai pas tout compris »* (11/11).

## 6. Les liens

* **Sa seule exception de niveau de détail** (9/11) : de base, les liens sont **très transparents** ; dès
  qu'on s'approche d'un bloc de texte, ils apparaissent. Un clic sur un lien : soit **un focus** sur ce à
  quoi il est rattaché, soit **un « show »** (*« isolation ? »*) — une **fenêtre temporaire** qui rassemble
  le nœud et **tous** ses liens, les blocs placés **à côté**, gardant leur couleur, avec leur provenance ;
  on y zoome, on y lit ; un clic sur un nœud défait la fenêtre et vole vers sa place.
* **Ses formules** (10/11) : *« ça fonctionne bien, mais j'aimerais rajouter quelque chose »* — quand les
  liens abondent, leur opacité suit leur **nombre à l'écran** (un
  lien : 100 % ; vingt : 30 %, en ligne droite) et **la proximité de la souris** aux nœuds qui les portent
  (`100 / (1 + distance)`) ; *« je n'aimerais jamais tomber en dessous de 8 % »* — ses mots tels quels,
  même de très loin : *« oui »* (11/11).
* **Ce qui ne va pas** (11/11, sur la carte de Wikipédia) :
  * **trop de flèches partout** : *« impossible de pouvoir naviguer »*, *« on ne comprend pas trop »* ;
  * elles pointent des **blocs** ; or, dans Wikipédia, un article fait référence à un autre depuis des
    **mots précis** — *« ce n'est pas du tout utilisé convenablement, on a juste des flèches random »* ;
  * l'algorithme qui les affiche sans brouillon est plutôt mauvais, surtout en mode *smooth* : revoir le
    système pour qu'un humain puisse **suivre** une flèche, avec une logique de **câblage** — *« mais pas
    du pur câblage, on ne s'y retrouverait pas non plus ; il faut une logique, et là encore passer par
    l'intelligence artificielle pour que ça fasse sens »*.
* **Les flèches grises** (11/11 au soir) : depuis la fiche 106, une flèche prend la couleur des lueurs
  qu'elle relie, et beaucoup deviennent grises — *« si c'est temporaire qu'elles soient grises, ça me
  va ; seulement si c'est temporaire »*. Le gris d'une flèche n'est accepté qu'en attendant que les
  flèches disent quelque chose.

## 7. La caméra et la carte de Wikipédia

* **`Alt+Z`** (9/11) : revenir en arrière dans les placements de la caméra — *« tu focus sur un bloc qu'il
  ne fallait pas, Alt+Z, et tu es de retour instantanément à ton lieu d'origine »* ; (10/11) *« c'est
  vraiment juste le Ctrl+Z, mais pour la caméra »* — chaque geste de caméra se défait, un voyage à la
  molette compris.
* **Le mode Focus** (8/11) : *« je préférais que ce soit un geste choisi »* — on y entre par un geste ; le
  zoom ne cache plus rien.
* **De loin, un texte montre le gris de ses lignes** (8/11) : *« carrément d'accord pour le gris des
  lignes »*.
* **Glucose se repose sur la carte** (9/11) : *« yep »*.
* **Le déséquilibre** (9/11) : la surface d'une région dit la quantité de ce qu'elle porte, *« je crois
  bien »* ; les régions se reconnaîtront par la couleur.
* **Comprendre Wikipédia d'un regard** (8/11) : *« comment le cerveau humain serait capable de comprendre
  Wikipédia tout entier sur un canva 2D sans que ce soit trop brouillon, avec le maximum de facilité »*.
* **Ce qui ne va pas** (11/11) : pas encore lisible (l'opacité des membranes), trop de flèches, des
  flèches qui ne partent pas des mots, un tracé brouillon, et *« je lag plutôt beaucoup »*.

## 8. Ce qui reste à définir — à sa parole

* **Le dossier**, à retravailler (15/10) ; **le dessin de la sélection** (E, pour l'instant).
* **Trans-domaines** (R&D).
* **Les rideaux** (un tableau dans une membrane), **les membranes de Mary** (étirées, minimisées),
  « optimiser » dans la Time Machine.
* **Ordonner, « très très grandement »** (15/10) : à concevoir avec lui.
* **Sa liste PureRef**, pour le preset.

## 8 bis. L'IA de Glucose : la méta-compréhension (11/11 au soir)

Ses mots, à propos de la petite IA du halo :

* **Des IA selon la machine** : *« je pense que l'IA est plutôt OK pour des téléphones, mais il faut
  qu'on ait des IA plus puissantes selon ce que l'utilisateur a comme machine »*.
* **Une seconde interface** : *« Glucose, c'est quoi ? Pour un humain, c'est un canva 2D qui a pour but
  d'être clair pour tous les humains ; mais pour une IA, il ne sera jamais parfait. Je propose de créer
  une seconde interface à Glucose, que l'humain ne voit pas forcément mais à laquelle il peut
  accéder. »*
* **Le hub** : *« depuis la création de Glucose, le logiciel avait pour objectif de R&D l'injection de
  mémoire, l'espace latent et la propagation de hub par des centaines d'IA. L'idée, c'est quoi ?
  Glucose, c'est le hub ; des millions de plugins peuvent être attachés. »*
* **Une IA locale qui évolue, en trois versions** : *« une IA locale, semi from scratch, qui évolue ;
  elle a trois versions : une pour téléphone ou PC très mauvais, une pour un PC portable plutôt bon, une
  pour des PC tours extrêmement puissants. TOUS doivent évoluer, et pour évoluer on a besoin de data,
  et c'est là que l'utilisateur sera le meilleur pour ça — uniquement avec son accord. »*
* **Le journal des rectifications** : *« un espèce d'énorme journal en arrière-plan, qui est là
  uniquement pour la rectification — pas dans l'obligation de tout comprendre du canva 2D, mais
  vraiment pour écrire des rectifications : l'utilisateur a mis cette couleur-là alors que l'IA pensait
  que c'était ça ; alors elle essaie de chercher, et note son erreur ou tout ce qu'elle pense »* ; et
  *« un document complet qui a pour but de comprendre la MÉTA-compréhension d'un canva lors de sa
  création »*.
* **Le cercle : l'IA aide, l'humain corrige, l'IA apprend** (11/11, plus tard dans la soirée) : *« il
  faut que le maximum de choses fasse sens sans que l'humain doive trop y réfléchir ; l'IA doit pouvoir
  aider l'humain, et l'humain doit corriger l'IA ; et l'IA doit enregistrer, apprendre, et ensuite un
  re-fine-tuning, ou la création d'une vraie IA from scratch, plus tard »*.
* **Ce qu'il en pense lui-même** : *« ce n'est pas du tout innovant, pas du tout dans les dernières
  technos, rien de bien impressionnant — c'est toi le pro, c'est toi qui seras mieux que moi »*.
* **Le rappel** : *« l'idée que ce soit LE logiciel pour tous les projets du monde, et aussi pour les
  équipes ; rénover la feuille de papier au goût de 2027 ; et rendre bien meilleure l'utilisation de
  l'IA, comme on le voit rarement sur des projets »*.

La réponse proposée le même soir (fiche 106) : apprendre **chez lui**, sans récolter — une petite
matrice posée sur le modèle commun, apprise des gestes de l'utilisateur ; le journal comme une seconde
colonne de l'histoire que Glucose écrit déjà ; la seconde interface comme le récit lisible du canevas
et de sa construction ; les trois IA répondant à la même question. À concevoir avec lui.

## 9. Sur la façon de travailler

* *« Je ne code pas »* : il porte la vision, les valeurs, le ressenti ; la technique se décide et
  s'exécute pour lui. Il parle français, ne voit pas la console, veut être **contredit** quand on a un
  argument.
* *« C'est plus à toi de tester plutôt que de me demander à moi ; tu seras sûrement meilleur juge que
  moi »* (18/10).
* *« Tu fais un truc, tu t'arrêtes, tu dis tout, puis tu attends un message de ma part et tu recommences
  ça en boucle — j'appelle pas ça travailler »* (septembre) : des sessions entières, les chantiers
  enchaînés.
* **Les documents** (11/11) : *« trop de choses qui se contredisent, d'autres qui se répètent »* ; les
  sessions suivantes *« se retrouvent vite perdues »*.
