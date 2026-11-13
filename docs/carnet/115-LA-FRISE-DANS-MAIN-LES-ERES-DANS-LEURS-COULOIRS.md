# 115 — La frise chronologique dans `main` ; les ères dans leurs couloirs ; une flèche suit ce qu'elle relie ; regarder un domaine

> Session du 13/11/2026, à partir de 20 h 41. La passation (fiche 114) donnait l'ordre : lui
> rappeler son clic de publication et lire ses vraies sessions ; **finir la frise et l'estompe**
> (le correctif de la fiche 114 § 6 : la suite entière, clippy, les sabotages, l'essai dans le
> vrai Glucose, `main`, son Glucose) ; puis filtrer par domaine ; le co-working et la porte
> publique dès sa parole.

## 1. L'ouverture

* **La 2.0.4-beta.1 est publiée** — par son clic, le 13/11 à 18 h 23 (heure de Paris). La fiche
  114 l'avait vue en brouillon à 17 h 12 ; `/releases/latest` la rend désormais.
* **Son téléphone** (la télémétrie) : le 6/11 à 17 h 53, la beta 4 s'est fermée « pour une mise à
  jour » et la session suivante était en **beta 8** — il l'avait donc mis à jour ce jour-là. Rouvert
  le 13/11 à 18 h 23, la minute même de la publication, **toujours en beta 8** : la 2.0.4-beta.1
  l'attend, posée par-dessus. Une fin anormale de plus (la session de la beta 8 du 6/11, finie en
  arrière-plan, l'appareil redémarré depuis).
* **La télémétrie** : 170 sessions, 10 installations, 34 fins anormales. **Aucune vraie session de
  lui au PC** depuis le 12/11 à 18 h 16 (sa chronique et sa sortie, inchangées).

## 2. Le correctif de la fiche 114, repris

Appliqué depuis `main` (`git apply`, propre). La première suite entière : **3 216 vertes, une
tombée — un cliquet**, et il avait raison :

* `handle_mouse_wheel` à 81 lignes : ce qui prend la molette **avant** le canevas (la palette, le
  texte lié, la roue, la frise, les domaines) est une seule question — extraite,
  `molette_prise_avant_le_canevas`.
* `poser_en_tournant` à 83 lignes : la frise l'avait alourdie d'un choix du mélange (« estompée,
  elle compose »). Or `mode_de_report` est **la** loi qui dit quand une photo peut remplacer les
  pixels plutôt que composer — opaque, pas tournée ; **pleine** en est la troisième condition. La
  loi a pris sa condition, la fonction a perdu ses lignes (et un sabotage la garde).
* clippy : une assertion sur une constante devenue `const { assert!(…) }` ; un tuple dans un tuple
  de l'épreuve d'APPLIQUE-1 devenu un champ nommé (`frise`).

## 3. FRISE-2 — les ères dans leurs couloirs (remis en question)

La passation laissait contestable « la loi de la plus petite ère ». **Ce n'était pas seulement une
hiérarchie perdue : la bande disait des dates fausses.** Rejouée sur le catalogue :

* entre 1760 et 1789, les Lumières (74 ans) sont plus courtes que la révolution industrielle (80
  ans) : la bande montrait les Lumières, puis la Révolution, puis l'Empire ; la révolution
  industrielle ne paraissait qu'**à partir de 1800**, en deux morceaux (1800-1803, trop étroit pour
  un nom, puis après l'Empire) — l'image de la fiche 114 la montre ainsi ;
* de même la Grèce antique semblait finir en -509 (la République romaine, plus courte, la couvrait),
  et la Renaissance commencer en 1453.

La loi des membranes vaut dans l'espace parce que les régions **s'emboîtent** ; les ères du
catalogue **se chevauchent** sans s'emboîter. Une bande qui ment sur une date est pire qu'une bande
muette (VISION § 2, *tout a une signification*).

**La loi** (`timeline::couloirs`) : le partage d'intervalles — la coloration d'un graphe
d'intervalles, les rangées empilées des frises (l'option `stack` de vis-timeline) et des échelles
géologiques (une rangée par niveau). Les ères prises par leur début, la plus longue d'abord à début
égal (le Moyen Âge passe avant ses parties, donc au-dessus d'elles), chacune au premier couloir
libre. **Le nombre de couloirs est le plus petit possible** : quand un couloir s'ouvre, tous les
autres portent une ère qui contient ce début — autant d'ères se chevauchent à cet instant. Aucune
constante. Le catalogue en demande **cinq** (de -509 à -146 : l'Holocène, l'Antiquité, l'Égypte, la
Grèce et Rome) ; rangés du plus large au plus fin : les âges géologiques, puis les grandes périodes,
puis leurs parties et ce qui les chevauche (la révolution industrielle, l'Ère numérique), la Grèce
et la Renaissance, la République romaine.

* **Une limite, pas une année partagée** : le catalogue écrit l'année du passage deux fois (476
  finit l'Antiquité et commence le Moyen Âge) ; dans la bande, une ère va de la limite où elle
  commence à celle où elle finit, et deux ères qui se touchent partagent un couloir.
* **Une frise de hauteur fixe** : une rangée par couloir du catalogue (`profondeur`). Une hauteur
  qui suivrait la vue ferait sauter la minimap et les barres pendant qu'on zoome la frise.
* **Regardé dans le vrai Glucose, puis corrigé** : les couloirs vides de la vue (ceux de
  l'Antiquité, en 1700-2000) laissaient un trou **entre les ères et la piste** qu'elles nomment,
  comme une erreur de mise en page — trois rangées vides une fois la frise zoomée sur le XXᵉ siècle.
  Les couloirs de la vue se serrent désormais contre la piste (`ui::frise::rangees`), leur ordre
  gardé, le vide sous l'en-tête. Une ère ne change de rangée que quand un couloir paraît ou
  disparaît de la vue.
* **Prouvé** : deux épreuves du noyau — chaque ère entière (la révolution industrielle de 1760 à
  1839 dans la vue), **avec la preuve à l'envers** (la loi d'avant, rejouée, la faisait naître en
  1800) ; les couloirs (deux ères qui se chevauchent n'en partagent jamais un, deux qui se touchent
  oui, la plus longue au-dessus à début égal, le rang suit le temps, autant de couloirs que d'ères
  qui se chevauchent au plus, cinq) ; une épreuve de pixels dans la vraie image (la révolution
  industrielle peinte dès 1770 dans sa rangée, vide avant elle, les rangées serrées contre la piste,
  toutes au-dessus de la piste).

## 4. ESTOMPE-2 — une flèche sans date suit ce qu'elle relie

Trouvé en lisant le correctif : une flèche **sans date** entre deux nœuds que la frise écarte
restait pleine — un trait vif entre deux fantômes, qui attire l'œil vers ce que la frise écarte.
**Une flèche est une relation** : sans date à elle, elle appartient au temps de ce qu'elle relie.

* **La loi** (`timeline::presence_d_une_relation`, une seule, dans le noyau) : datée, une flèche
  suit sa date ; sans date, elle prend la présence **du moins présent de ses deux bouts** ; un bout
  sans date, ou libre, ne retire rien (hors du temps, il n'est hors d'aucune période).
* **Les deux chemins qui dessinent une flèche** l'appellent (`renderer::estompe::de_la_fleche`) :
  son champ (le noyau et le nuanceur), et la couche du dessus (son étiquette, sa pastille). Les bouts
  se retrouvent **par l'index**, en temps constant ; sans frise, rien à chercher.
* **Prouvé** : une épreuve du noyau (sept cas, dont la preuve à l'envers : la seule date de la
  flèche la disait pleine) ; une épreuve des deux voies, la carte graphique exigée — entre deux
  textes de la Révolution, et vers un texte sans date, la flèche pâlit ; la frise qui garde la
  Révolution la laisse pleine ; chaque flèche mesurée contre la même scène sans flèches, **sous la
  même frise** (sinon les lueurs des textes estompés comptent avec elle).

## 5. L'essai dans le vrai Glucose

Le pilote hors écran (`outils/pilote/piloter.sh`, qui sait désormais `maj <touche>`), la version de
sortie, un dossier neuf. **Le document posé par la commande du terminal** — c'est aussi l'essai de
l'agent —, vers la porte du Glucose piloté, **nommée exprès** (`GLUCOSE_PORTE` : la porte par défaut
vit dans le dossier d'exécution de la session, le même que le sien) :

* la question « Un agent veut travailler ici » a paru ; « Autoriser » ; puis `poser-texte` (cinq
  textes), `relier` (deux flèches sans date), `dater` (« 1789-1799 », « Révolution industrielle »,
  « 1969 », « 1914-1918 ») — douze commandes, douze codes 0 ; `glucose lire` rend chaque date, son
  mot et ses années (`{"texte": "Révolution industrielle", "debut": 1760, "fin": 1840}`) ;
* **regardé** : les pastilles ambre ; l'IA du sens range les deux « révolutions » dans une même
  famille (la lueur bleue) ; `Maj+R` ouvre la frise, la minimap monte au-dessus d'elle ; **glisser
  la poignée du début jusqu'à 1900** : les deux révolutions pâlissent (pastilles et lueurs
  comprises), **la flèche sans date de la Révolution vers la Grande Guerre pâlit avec elles**, celle
  de la Lune vers « Sans date » reste pleine ; la molette zoome l'échelle (la fenêtre ne bouge pas,
  les ères suivent) ; la croix ferme, tout redevient plein ;
* **à la taille de son téléphone** (1080 × 2400 à 2,75) : la frise prend le bas, la minimap se pose
  au-dessus, les noms trop longs se coupent avec « … ».

Vu, laissé : quand une poignée tombe pile sur une graduation majeure, son trait ambre couvre le
premier chiffre du libellé (« 1900 » se lit « 900 »).

## 6. Les sabotages

`outils/sabotages/frise-114.json` : **51 gardes** — les 39 de la fiche 114 (une remplacée : la
plus petite ère n'est plus la loi), sept de FRISE-2 (les couloirs partagés par deux ères qui se
chevauchent, la plus courte au-dessus, une année de trop, la bande qui ignore les couloirs, les
rangées superposées, une seule rangée de place, le vide entre les ères et la piste), cinq
d'ESTOMPE-2 et une de la photo estompée qui remplacerait ce qu'elle couvre. **51 sur 51
tombent** (un arbre détaché à sa propre cible, la carte graphique exigée ; treize minutes). Un
sabotage ne compilait pas d'abord (`let n = 1;` laissait le compilateur sans type pour
`saturating_sub`) : écrit `1_usize`, rejoué seul, il tombe. La suite entière, sur la version
envoyée : **3 219 épreuves vertes**, clippy et la mise en forme à zéro.

**Puis la lentille de domaine** (§ 7) : `outils/sabotages/domaine-115.json`, **16 sur 16** ; et les
51 de la frise **rejoués** sur la loi neuve de la présence (chaque appel passe désormais ce que le
nœud montre) : **51 sur 51** — un « avant » mis à jour (le bout d'une flèche rend son regard, plus
sa seule date). La suite entière : 3 224 vertes et un cliquet tombé — un message de plus que permis,
le mien, retiré (§ 7) — puis les seize cliquets verts.

## 7. DOMAINE-1 et LENTILLE-1 — regarder un domaine

Le chantier suivant de la passation : « filtrer par domaine (Tauri) ». **Tauri ne le faisait
pas** — la parité le marque « — » côté Tauri (fiche 03 § 11.7), son panneau Domaines n'a aucun
filtre. Une fonction à concevoir, pas à recopier.

* **La recherche** : la « vitrine » de Kumu (*Showcase*) fait ce qu'il faut — ce qui n'est pas
  choisi pâlit au lieu de disparaître (à 0,1), les liens entre éléments choisis restent ; son
  *Filter* cache, son *Focus* cache le reste : Glucose ne cache rien (VISION § 2). Obsidian colore
  par groupes sans estomper.
* **La loi** (`glucose-core/src/lentille.rs`, une seule, dans le noyau) :
  * **graduée par l'appartenance** : un nœud porte chaque domaine avec un poids de 0 à 1 ; sous le
    domaine regardé, sa présence va de 0,12 (il ne le porte pas) à 1 (pleinement), en ligne droite.
    C'est sa parole du 9/11 sur les membranes — *l'opacité dit l'appartenance*. Aucun seuil (Tauri
    ne comptait un domaine qu'au-delà de 0,1) ;
  * **deux lentilles se combinent par le minimum**, le « et » de la logique floue — **contre la
    passation**, qui proposait leur produit : un nœud écarté par la frise et par le domaine serait
    tombé à 0,12 × 0,12 = 1,4 %, en pratique caché ; par le minimum, jamais sous 12 % ;
  * **une flèche** suit ce qui lui appartient en propre (sa date ; le domaine regardé si elle le
    porte), sinon le moins présent de ses deux bouts — ESTOMPE-2 étendue aux deux lentilles ;
  * le domaine regardé vit dans le document ouvert, **hors du journal** (une lentille, comme la
    période de la frise) ; NAV-1 le retire s'il n'est plus au catalogue (supprimé, ou défait par
    `Ctrl+Z`) — sinon tout le document pâlirait ; un export l'ignore (sa copie naît sans lentille).
* **Chaque appel de la présence** passe désormais ce que le nœud montre (`Annotation::regard`,
  `BoardImage::regard` : sa date et ses domaines) ; la pastille d'une flèche suit sa relation ; le
  lot des photos (LOT-1) et le cache des panneaux (une raison nommée, « domaine regarde ») se
  refont quand le domaine regardé change.
* **Le bouton** : « voir », sur chaque domaine du document dans le panneau Domaines — le bouton du
  dock, allumé (fond, contour, gras) quand on le regarde ; un second clic rend tout. **Aucun
  message** : j'en avais écrit un (« Tu regardes… ») ; le cliquet des messages (46 sites, pas un de
  plus) l'a refusé, et il avait raison — le bouton allumé et le canevas qui pâlit le disent. La palette offre **« Ne plus regarder le domaine »** tant qu'une lentille est posée
  (APPLIQUE-1). Un domaine d'un autre fichier ne se regarde pas d'ici : aucun nœud de ce document
  ne le porte.
* **Tranché seul** : le bouton « Domaines » de la bande **ne s'allume pas** quand un domaine est
  regardé — il faudrait faire passer l'état du document par la bande, le rail et leurs caches ; la
  rangée allumée du panneau suffit pour l'instant. À reprendre si, panneau fermé,
  on oublie pourquoi le canevas pâlit.
* **Prouvé** : trois épreuves du noyau (la présence suit le poids, sans seuil ; le minimum, et le
  produit qui aurait caché ; la relation) ; le vrai clic dans le panneau sur le vrai Glucose (voir,
  revoir, la palette, la suppression du domaine regardé, `Ctrl+Z`) ; « voir » s'allume en pixels
  sur sa rangée seule ; le cache du panneau nomme la raison ; le lot ; **sur les deux voies, la
  carte graphique exigée** : le texte pleinement de l'eau ne bouge pas d'un pixel, celui à moitié
  pâlit moins que celui du feu, la flèche sans domaine vers le feu pâlit, les deux voies rendent la
  même image. **Regardé dans le vrai Glucose** (le document posé par le terminal, les domaines par
  `poser-texte domaines=…`) : « voir » sur l'eau — « L'eau est H₂O » (100 %) pleine, « La pluie »
  (60 %) un peu pâle, « La vapeur » (20 %) davantage, le volcan et « Sans domaine » presque effacés,
  leurs réglettes de domaine avec eux.
* **Les sabotages** : `outils/sabotages/domaine-115.json`, seize gardes — le résultat au § 6.

## 8. Ce qui reste contestable

* **Les cinq rangées** : une frise plus haute que celle de Tauri (88 points) ; au XXᵉ siècle, deux
  rangées vides sous l'en-tête. Une hauteur qui suit la vue serait plus compacte, au prix d'une
  minimap qui saute pendant qu'on zoome. Gardé : fixe.
* **L'Holocène en haut de la bande moderne** : vrai, mais il ne dit rien de plus que « notre
  époque ». Le hisser dans l'en-tête comme un fil d'Ariane (ChronoZoom) libérerait une rangée.
* **Une flèche datée suit sa date seule**, même entre deux nœuds écartés : la date que l'humain lui
  a donnée l'emporte. De même pour une flèche qui porte le domaine regardé.
* **La présence graduée en ligne droite** (de 0,12 à 1 selon le poids) : une courbe ferait peut-être
  mieux voir la différence entre 0,8 et 1 ; à juger sur son écran.
* Et celles que la fiche 114 laissait : les ères en gris, 0,12, ce qu'on écrit et les poignées
  jamais estompés, l'export qui ignore la frise, la pastille qui suit le zoom.

## 9. La recherche (consultée le 13/11/2026)

* Le partage d'intervalles : [Algorithm::TimelinePacking](https://mirrors.edge.kernel.org/CPAN/authors/id/D/DM/DMOR/Algorithm-TimelinePacking-0.01.readme)
  (« solves the interval graph coloring problem ») ; [la visualisation des journaux
  d'événements](https://hn.jakechampion.name/reader/http://foldl.de/blogs/re/re-log-viz.html) (trier
  par début, le premier couloir libre, une fin libère son couloir avant un début au même instant) ;
  [thi.ng, les intervalles empilés](https://codeclimate.com/github/thi-ng/umbrella/packages/viz/src/plot/stacked-intervals.ts/source).
* Les frises qui empilent : [vis-timeline, l'option `stack`](https://visjs.github.io/vis-timeline/examples/timeline/groups/subgroups.html) ;
  [CoreUI, la vue en frise](https://coreui.io/scheduler/docs/views/timeline/) (« overlapping bars in a
  row stack into sub-lanes and the row grows to fit ») ; [les événements disjoints et
  parallèles](https://diglib.eg.org:443/bitstream/handle/10.2312/eurovisshort20181074/031-035.pdf)
  (EuroVis 2018).
* Estomper ce qu'on ne regarde pas : [Kumu, la vitrine](https://docs.kumu.io/guides/showcase)
  (ce qui n'est pas choisi s'estompe, à 0,1 ; les connexions entre éléments choisis restent),
  [son filtre](https://docs.kumu.io/guides/filter) et [son focus](https://docs.kumu.io/guides/focus)
  (qui cachent) ; [Obsidian, les groupes de la vue en graphe](https://obsidian.md/help/Plugins/Graph%2Bview)
  (colorer, sans estomper).
* Les échelles géologiques, une rangée par niveau : [le National Park Service](https://www.nps.gov/subjects/geology/time-scale.htm) ;
  [Wikipédia, le modèle de l'échelle géologique](https://cavac.at/cavacopedia/Template%3ATimeline%20geological%20timescale).

## 10. Les leçons

* **Une loi juste dans l'espace peut mentir dans le temps** : « la plus petite région l'emporte »
  suppose des régions emboîtées ; des intervalles qui se chevauchent en sortent coupés. Rejouer
  l'ancienne loi sur le vrai catalogue a montré la date fausse, chiffrée.
* **Regarder avant de saboter** a encore payé : le trou entre les ères et la piste ne se voyait que
  sur l'image du vrai Glucose ; vu avant les sabotages, il n'en a refait aucun.
* **Une épreuve d'une flèche estompée se mesure sous la même frise** : contre une scène sans frise,
  les lueurs des textes estompés comptaient avec la flèche.
* **Deux lentilles se combinent par le minimum, pas par le produit** : chacune a son plancher
  (« rien ne se cache ») ; leur produit le franchit (1,4 %). Le « et » d'appartenances graduées est
  le minimum de la logique floue.
* **Une fonction « de Tauri » se vérifie dans Tauri** : « filtrer par domaine » n'y existait pas ;
  le croire aurait fait recopier un fantôme au lieu de concevoir.
* **Une porte se nomme exprès dans un essai** : la commande du terminal joint par défaut la porte de
  la session — celle de son Glucose à lui, s'il est ouvert.
