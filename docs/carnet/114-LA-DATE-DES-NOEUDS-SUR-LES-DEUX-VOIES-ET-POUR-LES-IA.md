# 114 — La date des nœuds, sur les deux voies, dans l'export et pour les IA

> Session du 13/11/2026, à partir de 17 h 10. La passation (fiche 113) donnait l'ordre : lui
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

## 6. La réglette du temps — conçue

La conception (`~/.cache/glucose-bancs/date-113/reglette-conception.md`), à remettre en question
avant d'écrire :

* **Tauri** (`TemporalRuler.tsx`, `screens/12_temporal_ruler.png`) : `Maj+R`, une barre en bas, une
  bande des ères, des graduations, une fenêtre ambre entre deux poignées, la molette qui zoome
  autour du curseur ; ce qui sort de la fenêtre s'estompe à 0,12, un nœud sans date reste visible.
  Sur sa capture, la minimap recouvre le bout droit de la réglette.
* **Gardé** : estomper plutôt que cacher (Palantir Vertex, le « Fade Mode » de Gephi Lite, et
  VISION § 2 : rien ne se cache) ; l'échelle linéaire et le zoom profond (ChronoZoom).
* **Changé, avec ses raisons** : les ères en gris neutres alternés plutôt qu'une teinte tirée d'un
  hachage de leur nom — *« tout a une signification »* ; l'ambre seul dit le temps. **Ajouté** :
  la répartition des nœuds datés du document le long de l'échelle (l'histogramme de KeyLines et de
  Kepler.gl) — sans elle, sur une échelle de -100 Ma à 3000, on ne retrouve pas ses trois dates.
  La vue initiale tirée des dates du document plutôt que -1000..2050. **La place** : une barre du
  bas (`ui/bas.rs`) — *la minimap ne bouge jamais* : la réglette s'arrête avant elle.
* **Les deux voies** : la carte graphique pose déjà une texture avec une opacité (`Pose::opacite`) ;
  les flèches, les lueurs et les membranes ont chacune leur alpha. Le filtre **multiplie** ce qui
  existe, par une seule loi que chaque voie appelle.

## 7. Où le travail en est

* **Dans `main`** : DATE-1 et le bleu des liens (`ade07ee`), puis ce document.
* **Ensuite** : la réglette et le filtre (§ 6), commencés dans le noyau — la loi de présence
  (`timeline::presence`, `ESTOMPE`), l'étendue et la répartition des dates, le filtre en
  `TemporalFilter` (une seule source).

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
