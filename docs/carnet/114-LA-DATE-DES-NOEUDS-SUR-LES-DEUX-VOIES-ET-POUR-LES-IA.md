# 114 — La date des nœuds, sur les deux voies et pour les IA ; la frise chronologique et l'estompe

> Session du 13/11/2026, de 17 h 10 à 20 h 10, arrêtée à sa demande (*« la session commence
> malheureusement à être trop longue »*). **Envoyé** : DATE-1 (`ade07ee`), les documents. **En
> correctif, pas dans `main`** : la frise chronologique et l'estompe (§ 6), écrites, éprouvées sur
> les deux voies, pas encore sabotées. La passation (fiche 113) donnait l'ordre : lui
> rappeler son clic de publication et lire ses vraies sessions ; **finir le correctif** de la
> fiche 113 (le bleu des liens, DATE-1) ; puis la réglette du temps ; le co-working et la porte
> publique dès sa parole.

## 1. L'ouverture

* **La 2.0.4-beta.1 est toujours en brouillon** (vérifié à 17 h 12 : `/releases/latest` rend la
  2.0.3-beta.10, le brouillon a ses onze fichiers) : son clic, rappelé.
* **Aucune vraie session de lui depuis le 12/11 à 18 h 16** (déjà lue, fiche 111) : sa chronique et
  sa sortie copiées dans le scratchpad, inchangées.
* **Ses deux pages** (le co-working, la porte publique) : ni réponse, ni commentaire.
* **La télémétrie** : 168 sessions, 10 installations, 33 fins anormales (166, 9 et 33 la veille).
  **Une installation neuve** : un Windows (carte inconnue), deux sessions le 12/11, en
  **2.0.2-beta.2** (publiée le 9/10) — la mise à jour automatique existe depuis la fiche 48, elle
  lui proposera la dernière.

## 2. Le correctif de la fiche 113, fini

Appliqué depuis `main` (`git apply`, propre), puis :

* **(a) Le menu d'une sélection** attend désormais « Dater… » : l'épreuve accordée.
* **(c) Les deux épreuves de la pastille qui tombaient visaient mal** — l'image rendue, regardée
  avant de toucher quoi que ce soit, montrait la pastille au bon endroit :
  * celle de la photo supposait que `BoardImage::new(…, 500, 0, 100, 100)` posait son **coin** en
    (500, 0) ; elle pose son **centre** (450 à 550). L'épreuve lit désormais le coin par la boîte
    que le noyau donne aux flèches (`arrow::emprise`), comme pour le texte — aucune zone écrite en
    dur ;
  * celle « de loin » exigeait un rouge au-dessus de 150 au dixième, où la pastille fait **deux
    pixels de haut** : son ambre s'y mêle au fond — (53, 48, 41) sur (23, 23, 23), regardé. **Ce qui
    se garde de loin, c'est la couleur** : un pixel « chaud » (le rouge passe le bleu), alors que
    tout le reste de la scène est gris, donc rouge = bleu, ou froid. **Un critère sans seuil**, qui
    vaut de près comme de loin ; et la preuve à l'envers dans chaque épreuve : sans date, aucun
    pixel chaud au même endroit.
  * Décidé, et à dire : **la pastille suit le zoom comme son nœud** (Tauri faisait de même) ; elle ne
    se cache jamais (VISION § 2), mais au dixième elle n'est plus qu'une tache ambre. La réglette
    et son filtre seront le moyen de lire le temps de loin.
* **(d) La voie graphique, vérifiée** : la pastille se pose dans la moitié que les deux voies
  partagent ; sur la carte graphique, c'est **la couche du dessus**, dont ARCHITECTURE § 4 dit
  qu'elle est vide sans rien de choisi. Lu dans `present/bandes.rs` : cette couche ne déclare pas
  ce qu'elle porte, elle **balaie ses pixels** (BANDE-1) — une date posée part donc forcément.
  **Prouvé** par une épreuve neuve de `tests/voies_suite.rs` : un texte, une flèche, une membrane
  et une photo en chemin, tous datés, **rien de choisi** — la carte graphique montre l'ambre au coin
  de chacun, rien sans date, et les deux voies rendent la même image (l'écart admis). Lancée avec
  `GLUCOSE_EXIGER_UNE_CARTE=1` : sur sa carte Intel, elle a vraiment tourné.
* **(b) Le témoin**, regardé avant/après. L'« avant » sans arbre détaché : la cible du dépôt pèse
  37 Go, une copie était exclue ; le travail a été sauvegardé en correctif
  (`~/.cache/glucose-bancs/date-113/sauvegarde-114.patch`), mis de côté (`git stash -u`), l'ancien
  témoin capturé **dans la même cible** — ses empreintes étaient exactement celles de `main` —, puis
  repris, identique à la sauvegarde. L'écart : **247 pixels**, le seul mot « lien » (plus clair,
  toujours bleu et souligné) ; le menu : **17 620 pixels**, « Dater… Maj+T » dans le groupe de
  l'édition ; vu de loin, 80 pixels. Les trois empreintes mises à jour, avec leur ligne d'histoire.
* **(f) L'export SVG** : la pastille ne faisait que remplir un tracé, tracer son contour et écrire
  — les quatre gestes de la `Surface` (P5, fiche 70). Elle est désormais **écrite une fois**,
  générique : le canevas lui donne ses pixels, l'export ses vecteurs, **le même tracé**, posé au
  même rang (après les nœuds, avant les réglettes des photos). Une épreuve : un filet ambre par
  nœud daté et l'encre du mot dans le SVG, rien sans date.
* **(f) Les IA** — la passation proposait « un champ `date` sur chaque outil qui pose ».
  **Remis en question** : le geste de l'humain est **un** geste sur une sélection (`Maj+T`) ; son
  pendant est **un** outil, `glucose_dater` (des identifiants, une date dans la langue de la
  question de l'humain ; vide, il la retire), qui vaut pour toutes les sortes de nœuds, photos
  comprises, sans alourdir six schémas. **Et surtout la lecture** : ce qu'un humain voit, une IA
  doit le lire (VISION § 1, *« impérativement »*) — `glucose_lire` rend la date de chaque nœud
  daté, **son mot et ses années exactes** (`{"texte": "1789 – 1799", "debut": 1789, "fin": 1799}`) :
  le mot arrondit avant -10 000 (« 12.3 ka » pour -12 345), les années non, et des années entières
  renvoyées se relisent sans perte. Un nom qui n'est pas un nœud du tableau est **refusé par son
  nom**, et rien ne change — l'humain date ce qu'il a choisi, qui existe ; un agent peut se tromper
  de nom, et doit le savoir. Quatre épreuves (`agents/executer/tests/date_1.rs`) ; `glucose dater`
  existe au terminal de lui-même.

## 3. Ce que les cliquets ont demandé

La première suite entière : 3 190 vertes, **3 tombées, les trois cliquets** — et ils avaient
raison :

* **`timeline` a enfin un appelant** : le module du noyau qui attendait son geste depuis la fiche
  12 sort de la liste des modules sans appelant (du terrain gagné).
* **`annotation.rs` à 620 lignes** : sa cause est écrite dans son propre commentaire — un champ
  recopié dans les quatre sortes de nœuds (le standard § 2.1 voudrait un tronc commun), donc un
  `match` à quatre bras par accesseur. **La répétition a disparu plutôt que déménagé** : une macro
  documentée (`champ_commun!`) écrit ce `match` une fois ; dix accesseurs (l'identité, la place, la
  membrane, les domaines, la date, en lecture et en écriture) tiennent chacun en trois lignes.
  569 lignes.
* **`Theme::dark` à 82 lignes** : les trois couleurs de la pastille sont allées là où le thème
  range déjà **l'ambre du temps** (`CouleursDuTemps` : les jalons, le curseur du passé, le liseré
  — « pour cela seulement »). Épinglées dans l'épreuve du thème, comme les autres valeurs de Tauri.
* **`dessiner_sur_les_photos` à 81 lignes** : l'appel aux pixels du canevas est sorti dans
  `renderer/date.rs` (`poser_les_dates`).
* `executer.rs` (les outils des agents) était à **600 lignes pile** : la description d'un nœud en
  JSON est partie dans son module (`agents/executer/decrit.rs`), avec la date.

## 4. Les sabotages

`outils/sabotages/date-114.json` : **21 gardes** — le canevas, la carte graphique et l'export qui
tairaient les dates, les photos ou les nœuds sans pastille, la pastille loin de son coin ou taillée
trop tôt, une photo que le noyau ne date pas, ce qui porte déjà la date compté comme changé, un
agent qui daterait un nom inconnu, `glucose_lire` qui tairait la date ou arrondirait ses années,
une date vide illisible pour l'agent, « Dater » qui ne s'appliquerait jamais ou ne demanderait
rien, `Maj+T` retiré, une date illisible qui ne se redemande pas, les ères qui ne suivent pas ce
qu'on tape, « Retirer la date » sans objet, le menu sans « Dater… », l'ambre de Tauri changé.
**21 sur 21 tombent** (dans un arbre détaché, à sa propre cible, la carte graphique exigée ; six
minutes). La suite entière : **3 193 épreuves vertes**, clippy et la mise en forme propres.

## 5. Trouvé en chemin

* **L'étiquette d'une photo en chemin ne suit pas le zoom** : au quart, « Image [i] » déborde de sa
  photo (sa taille est celle de l'écran, pas du monde). Ancien, sans rapport avec la date ; à
  reprendre avec les ornements des photos.
* **Deux types pour un seul filtre du temps** : `Store::temporal_filter` (une `TemporalAnchor`) et
  `timeline::TemporalFilter` — une seule source à garder quand viendra la réglette.

## 6. La frise chronologique et l'estompe — écrites, en correctif

`~/.cache/glucose-bancs/frise-114/frise-estompe.patch` (`git apply` depuis `main`, vérifié ; 56
fichiers, 39 sabotages compris ; les images regardées à côté). La conception d'abord écrite :
`reglette-conception.md`, au même endroit.

* **Le nom : la frise chronologique**, pas « la réglette du temps » — la Time Machine a déjà sa
  réglette (`interactions/temps.rs`, `TempsIntent::Reglette`) : le passé du **document**, geste par
  geste. La frise montre les **dates** des nœuds. `Maj+R`, la palette (« Frise chronologique »),
  toujours offerte ; FRISE-1 et ESTOMPE-1.
* **Ce que Tauri faisait** (`TemporalRuler.tsx`, `screens/12_temporal_ruler.png`) : une barre en
  bas, une bande des ères **teintées d'un hachage de leur nom**, des graduations, une fenêtre ambre
  entre deux poignées, la molette ; ce qui sort de la fenêtre s'estompe à 0,12 ; sur sa capture, la
  minimap recouvre le bout de la barre.
* **Ce qui est écrit** :
  * **le noyau** (`timeline.rs`) : `presence` et `ESTOMPE` (0,12, le nombre de ressenti de Tauri) ;
    `etendue` des dates d'un document ; `repartition` (le compte des dates par graduation) ;
    `eres_de_la_vue` — **à chaque année, la plus petite ère qui la contient**, la loi des régions
    des membranes (REGION-1) appliquée au temps : les ères du catalogue s'emboîtent, Tauri les
    empilait ; `Store::temporal_filter` devient un `TemporalFilter` (une seule source),
    `Store::presence`, `Store::dates_du_tableau` ; le champ d'une flèche porte son opacité
    (`Champ::opacite` : la couleur prémultipliée multipliée — exactement une opacité de groupe) ;
  * **la frise** (`ui/frise.rs`, `interactions/frise.rs`) : une **bande du bas, comme le clavier
    d'Android** — l'écran visible s'en raccourcit (`UiState::sous`, la seule source de ce que le bas
    cache), donc la minimap, les barres d'une sélection et le message se posent au-dessus d'elle
    par leurs propres lois ; l'en-tête et ses bornes en ambre, la croix ; la bande des ères en gris
    neutres (*« tout a une signification »* : le hachage de Tauri ne disait rien), un nom coupé
    garde son début et « … » ; les graduations, une majeure sur cinq nommée, jamais au-delà de la
    piste ; **la répartition des dates du document** en barres grises (KeyLines, Kepler.gl) ; la
    fenêtre ambre et ses poignées, prises à 24 points (la moitié d'une cible de doigt) ; la molette
    et le pavé **par la loi du canevas** (`pan_zoom::geste` : un tiers d'octave par cran, autour de
    la main) ; la vue d'ouverture tirée des dates du document (une graduation majeure de part et
    d'autre), celle de Tauri sans date ; dans « ce qui attend une décision » (DECISION-1), sur les
    deux voies ;
  * **l'estompe** (`renderer/estompe.rs`) sur chaque chemin : ce qui est **d'une seule couleur**
    multiplie son alpha (les régions des membranes et leur frontière — sauf l'anneau d'une membrane
    choisie —, les lueurs par `halo::alpha_montre` que les deux voies appellent, le champ d'une
    flèche dans le noyau et dans son nuanceur — une place libre de son réglage, sans changer le
    tampon —, la pastille d'une date, l'étiquette et la pastille de relation d'une flèche) ; ce qui
    est **composé** prend une opacité de groupe : sur la carte graphique `Pose::opacite` (photos,
    leur repli, photos en chemin, cartes de texte, titres) et les formes d'une carte plus grande que
    l'écran à leur couleur prémultipliée ; au processeur `estompe::en_groupe` (dessiner à part, dans
    la boîte de la texture — `card::marge_de_la_texture`, une seule mesure pour les deux voies —,
    puis composer) pour les cartes de texte, les titres (la boîte d'encre du texte détouré) et les
    photos en chemin ; une photo décodée estompée passe par la pose générale (`draw_pixmap` à son
    opacité, en composant), **ne compte plus comme un cache** dans l'occlusion, et sa tuile la nomme
    avec sa présence ; **le lot des photos** (LOT-1) retient la frise ; la couche du dessus (les
    post-it, leur ombre et leur pilule ; les réglettes des domaines par **une surface qui estompe**,
    `Estompee`, un adaptateur de `Surface`) ;
  * deux exceptions, tranchées : **ce qu'on écrit se voit entier** (la carte, le post-it, le titre
    qu'on édite) ; **l'anneau et les poignées d'un nœud choisi restent entiers**, pour le prendre ;
  * **un export ignore la frise** : sa copie du document (`copie_a_exporter`) naît sans période —
    la frise est une façon de regarder, pas le document.
* **Prouvé** (dans le correctif) : le noyau (la présence, l'étendue, la répartition, la plus petite
  ère, le champ estompé en groupe) ; les gestes par les vrais chemins (`Maj+R`, l'appui, le glisser
  d'une poignée puis de la fenêtre, le relâchement, la molette par `handle_mouse_wheel`, la croix,
  l'appui qui ne descend pas au canevas) ; la mise en page (la minimap et une barre au-dessus) ; la
  piste (la barre de 1969, l'ambre dans la fenêtre seulement) ; **sur les deux voies, la carte
  graphique exigée** : le texte, le post-it, la membrane, son titre, la photo, la flèche et la
  lueur s'estompent, le processeur rend la même image, le texte sans date ne bouge pas, une frise
  qui garde la Révolution ne change rien ; **une photo décodée estompée laisse voir la photo
  qu'elle couvre** des deux côtés (l'occlusion), et la couvre sans frise ; le lot défait par la
  frise ; la pastille pâlie ; la coupe des noms ; l'export. **Regardé** : `frise-toute.png` et
  `frise-1780-1820.png` (la Grande Guerre et la Lune pâles, le reste plein) — deux défauts vus et
  réglés (les ères sans nom, le libellé « 2000 » qui débordait).
* **Pas encore** : la suite entière sur la dernière version (la précédente n'a pas compilé : une
  épreuve du noyau posait le filtre avec l'ancien type, corrigé ; clippy a refusé deux signatures,
  allégées), clippy, **les 39 sabotages** (`outils/sabotages/frise-114.json`, chaque « avant »
  vérifié), **l'essai dans le vrai Glucose** (le pilote : `Maj+R`, glisser une poignée, une
  capture), puis `main`. Au doigt : la frise passe par les mêmes gestes ; pincer la frise pour la
  zoomer n'est pas éprouvé.

## 7. Où le travail en est

* **Dans `main`, envoyé** : DATE-1 et le bleu des liens (`ade07ee`), les documents (`4f0c167`, puis
  celui-ci). **Son Glucose installé : `4f0c167`** (mis à jour à 18 h 42, 21 minutes, le chien de
  garde jamais déclenché).
* **Le correctif** `~/.cache/glucose-bancs/frise-114/frise-estompe.patch` : la frise et l'estompe
  (§ 6).
* La 2.0.4-beta.1 attend toujours son clic.

## 8. La recherche (consultée le 13/11/2026)

* Estomper plutôt que cacher : [Palantir Vertex, la frise](https://palantir.com/docs/foundry/vertex/timeline/)
  (« nodes that do not match the time filter are faded out ») ; [Gephi Lite, PR 280](https://github.com/gephi/gephi-lite/pull/280)
  (« Fade Mode ») ; [la navigation temporelle des graphes dynamiques](https://arxiv.org/pdf/2008.12747).
* Montrer la répartition des données sur la barre de temps : [KeyLines, la barre de
  temps](https://developers.cambridge-intelligence.com/docs/keylines/time-bar) (« each bar
  represents a period in time and is a cumulative count of all items within the range ») ;
  [Kepler.gl, la lecture dans le temps](https://docs.kepler.gl/docs/user-guides/h-playback).
* L'échelle : [ChronoZoom, du concept à la réalité](https://sociostudies.org/almanac/articles/from_concept_to_reality_developing_a_zoomable_timeline_for_big_history)
  (déjà dans la fiche 113) ; [le projet chez Microsoft Research](https://www.microsoft.com/en-us/research/project/chronozoom/)
  (une interface d'abord trop compliquée, simplifiée ensuite).

## 9. Les leçons

* **Regarder l'image avant de toucher l'épreuve ou le dessin** : les deux épreuves qui tombaient
  se trompaient, pas le dessin — une photo se place par son centre.
* **Un critère qui tient à toutes les distances vaut mieux qu'un seuil** : « chaud » (le rouge passe
  le bleu) reconnaît l'ambre de près comme au dixième, là où un seuil de luminosité ne tenait qu'à
  une échelle.
* **Une épreuve sautée se tait** : la sortie d'une épreuve qui passe est gardée par le banc ;
  l'absence de « sautée » ne prouve rien. `GLUCOSE_EXIGER_UNE_CARTE=1` la fait tomber s'il n'y a
  pas de carte.
* **Un cliquet qui proteste désigne parfois une répétition à faire disparaître**, pas des lignes à
  déménager : la macro des champs communs.
* **`commande | tail` rend le code de `tail`** : le premier « clippy 0 » de la session mentait ;
  écrire la sortie dans un fichier et lire `$?` de la commande elle-même.
* **Chercher un nom avant de le poser** : `glisser_la_reglette` existait (la Time Machine) ; le
  conflit du compilateur a révélé deux « réglettes du temps » — d'où la frise.
* **Écrire les sabotages révèle les épreuves aveugles** : neuf gardes de l'estompe n'avaient aucune
  épreuve (la photo décodée, le titre, la lueur, le lot, la pastille, la molette par son vrai
  chemin…) ; une loi partagée par les deux voies échappe à leur comparaison — il faut mesurer le
  nœud lui-même.
* **Une boîte se trouve par l'image** quand on ne la connaît pas (le titre d'une membrane : là où il
  change des pixels), plutôt qu'écrite en dur.
