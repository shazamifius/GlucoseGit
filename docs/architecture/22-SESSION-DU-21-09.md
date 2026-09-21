# 22 — Le fond et les lueurs sur la carte, et ce que la capture a montré

> **Rôle de ce document.** La fiche [`21`](21-LES-VOIES.md) posait quatre étapes et disait
> l'étape 1 faite. Celle-ci dit ce que l'après-midi du 21/09 a fait de l'étape 3 — le fond
> et les lueurs sur la carte —, ce qu'elle a **refusé** de faire du plan qu'on lui donnait,
> et surtout ce qu'une image mise à côté d'une autre a montré que trois commits et mille
> trois cents tests n'avaient pas vu.
>
> **Date** : 2026-09-21 · trois commits, de `2e547ca` à `7257acf`.
> **État vérifié** : `cargo test --workspace` exit 0, **1 346 tests verts**, clippy strict à
> zéro, onze cliquets, aucun plafond relevé.
>
> **Le point de départ** : la chronique de terrain de midi, sur un document de 482 nœuds et
> zéro photo. Elle désignait le poste sans ambiguïté, dans les quatre gestes mesurés.

---

## 1. Ce qui a été fait, en une table

| Chantier | Ce que c'est | Vérifié par |
|---|---|---|
| **Les lueurs sur la carte** (`present/lueurs_gpu.rs`) | Un quad par carte, `α = A·P(x)·P(y)` évalué **analytiquement** par `erf` | Écart **1 niveau** sur 255 contre le processeur, 2 pour deux lueurs superposées |
| **Le fond sur la carte** (`present/fond_gpu.rs`) | Un triangle, une couleur, et le point de grille le plus proche par un arrondi | Écart **7 niveaux**, exactement la borne calculée, sur quelques millièmes de l'écran |
| **La couche du dessous qui cesse d'exister** | Ne porte plus que membranes et dossiers ; vide, elle ne se téléverse ni ne se compose | Compteur `dessous_televerse` ; la passe **compte** ce qu'elle dessine |
| **La scène en cinq temps** (`present/gpu/cinq_temps.rs`) | fond → lueurs → dessous → photos → dessus | `tests/voies_suite.rs` : la scène entière, deux voies, **3 niveaux d'écart, zéro canal au-delà de 3** |
| **Les photos en chemin** | Elles ne se dessinaient plus du tout sur la voie graphique — depuis trois commits | Une capture côte à côte, puis un test qui **compte** les pixels de cadre |
| **BLINK-1** | Le dessin du curseur lisait l'horloge : le même état rendait deux images | Un test qui porte sa preuve : deux horloges en opposition de phase, même image |
| **La capture de la voie graphique** (`examples/capture_voie_gpu.rs`) | Les cinq temps composés hors fenêtre, écrits en PNG | C'est elle qui a trouvé les photos en chemin |

---

## 2. Ce que la chronique de midi disait, poste par poste

Document de 482 nœuds, 240 Hz, 223 images, **43 images par seconde**, 86 % au-dessus de
10 ms. Le tempo calé à **cinq balayages** — 20,8 ms — parce que le p99 du travail réel
(16,4 ms) ne tenait pas dans quatre.

| geste | `halos` | `blit` | `clear` | `recolte` | `annotations` |
|---|---:|---:|---:|---:|---:|
| repos | **4,10** | 2,05 | 1,45 | 1,45 | 1,02 |
| déplacer la vue | **4,10** | 2,05 | 1,22 | 1,22 | 0,86 |
| glisser un nœud | **5,00** | 2,15 | 1,58 | 1,61 | 0,59 |
| sélectionner | **4,63** | 2,74 | 1,65 | 1,60 | — |

`halos` est le premier poste réel partout — après `tempo`, qui est une attente et non un
coût. Et il l'est pour une raison géométrique que la fiche du module nomme : une ombre
déborde de sa carte de la portée du flou, donc elle touche bien plus de pixels qu'elle n'en
désigne. Seize fils l'avaient fait tomber de 8,86 à 4,43 ms, et c'est là que le
processeur s'arrête : le découpage est horizontal, et des cartes alignées verticalement
n'occupent que trois bandes sur seize.

> **Une lecture fausse dans le brief de la session, corrigée en la lisant.** Il donnait
> « tempo 23,17 ms » comme un poste. Ce chiffre est la **durée médiane qu'une image reste à
> l'écran** — 5,56 balayages — et non le poste `tempo`, qui vaut 8,19 ms. Les deux disent la
> même chose sous deux angles, mais l'un est une attente et l'autre un résultat, et les
> confondre aurait fait chercher huit millisecondes qui n'existent pas.

---

## 3. Ce que la carte fait de mieux, et ce n'est pas « aller plus vite »

### 3.1 Les lueurs

HALO-1 établit qu'une ombre de boîte gaussienne est **séparable** — un produit de deux
profils d'une dimension, et c'est exact, pas approché. Le processeur échantillonne `Φ`
dans une table au pixel, parce qu'un masque pré-calculé doit être discret. La carte n'a pas
cette contrainte : elle évalue `Φ(t) = ½(1 + erf(t/σ√2))` par l'approximation d'Abramowitz
& Stegun 7.1.26, dont l'erreur maximale vaut 1,5·10⁻⁷ — un millième du demi-niveau de huit
bits. Ce n'est donc pas une voie qui abîme pour aller vite : c'est **la même loi, écrite
pour un matériel qui sait l'évaluer par million**.

Le culling, la teinte symbiotique — qui dépend du voisinage — et la décision de ce qui se
dessine restent au processeur. C'est le socle de la fiche 21, et rien n'en descend.

### 3.2 Le fond

La grille ne descend jamais sous trente-deux pixels de pas, et le rayon d'un point
plafonne à deux et demi. **Un pixel ne peut donc être touché que par un point** — celui de sa
maille. Le nuanceur n'a rien à parcourir : un arrondi donne le point, une distance donne la
couverture, et c'est la même formule analytique que le processeur, `clamp(r + ½ − d, 0, 1)`,
sans la quantification en seize phases qu'il doit s'imposer pour pré-calculer ses masques.

### 3.3 L'écart entre les deux voies, mesuré et tenu

| passe | borne calculée | mesure | borne du test |
|---|---:|---:|---:|
| lueurs, une seule | 4 | **1** | 2 |
| lueurs, deux superposées | 5 | **2** | 2 |
| grille | 7 | **7** | 7 |
| scène entière | 7 | **3**, et zéro canal au-delà de 3 | 5 |

La borne des lueurs était écrite à quatre par raisonnement ; la mesure a donné un et deux,
et c'est **deux** qui est dans le test. Celle du fond, calculée à `0,125 × 0,45 × 123 ≈ 7`,
est atteinte **exactement** — la meilleure confirmation possible, et aucune marge où une
dérive pourrait se cacher. Sur la scène entière, la borne est à cinq : trois de mesure, deux
pour qu'une autre carte graphique, qui n'arrondit pas forcément au même bit, ne fasse pas
échouer une épreuve où rien n'est faux. La charte interdit d'exclure une machine, et cela
vaut aussi de ses tests.

**Un test qui passe avec une borne large ne prouve rien.** Chaque borne a été baissée à
zéro pour lire la mesure, puis remontée à la mesure.

---

## 4. Ce que j'ai refusé de faire, et pourquoi c'est le vrai sujet

Le plan de la session proposait, en deuxième chantier : *« ne pas re-téléverser la couche
du dessous quand elle n'a pas changé — le mécanisme existe déjà pour la bande du haut »*.

Il suffit de regarder pour voir que cela ne vaut rien. Pendant un déplacement ou un zoom —
**les seuls moments où la cadence est en jeu** — le fond se décale et les lueurs suivent
leurs cartes. Tout change à chaque image, donc un cache de salissure y gagnerait exactement
zéro. Il aurait servi sur une scène immobile, où il n'y a pas de problème de cadence.

Ce qui marche est plus simple et définitif : une fois le fond et les lueurs sur la carte, la
couche du dessous ne porte plus que les membranes et les dossiers. Sur un document qui n'en
a pas, elle est **entièrement vide**, et rien ne part — ni ses quinze mébioctets, ni le
dessin qui composerait du transparent. **Un coût supprimé vaut mieux qu'un coût mis en
cache.** C'est la règle de la charte sur les constantes, appliquée à un téléversement.

Et c'est la passe elle-même qui répond à la question, en comptant ce qu'elle dessine.
Balayer les pixels coûterait un écran entier pour la même réponse.

---

## 5. Ce que la capture a trouvé, et que mille trois cents tests ne pouvaient pas voir

### 5.1 Les photos en chemin

`examples/capture_voie_gpu.rs` compose les cinq temps hors fenêtre et écrit l'image. Mise
à côté de `capture_temoin`, la différence saute aux yeux : deux cadres portant
« Image [img-libre] » et « Image [img-verrouillee] » d'un côté, **rien** de l'autre.

Sur la voie processeur, une photo dont les octets ne sont pas encore là se dessine comme un
cadre gris portant son identifiant, au moment où la pose échoue. Quand les photos sont
descendues sur la carte, cette pose a cessé d'avoir lieu : la carte ne connaît pas la photo,
donc elle ne dessine rien, **et plus rien ne la dessinait**. Un commentaire de `voies.rs`
promettait pourtant, mot pour mot, que « le processeur porte le cadre en chemin dans la
couche du dessus ». Personne ne l'avait écrit.

C'est la **troisième** régression de l'étape 1, après l'écran noir et les poignées, et elle
a survécu à trois commits parce qu'elle ne se voit que pendant un décodage — deux secondes
à l'ouverture d'un document de quatre cents photos — ou sur un fichier disparu. Les tests
d'aspect montent tous leurs scènes avec des photos déjà là.

Les trois ont la même forme : **une passe qui cesse d'être appelée**. Aucun test de passe
ne peut les voir, parce qu'un test de passe appelle la passe. Seule une comparaison de
l'image entière les attrape, et c'est `tests/voies_suite.rs`.

**L'écart d'ordre, assumé.** Le cadre se pose dans la couche du dessus, donc après les
photos que la carte a posées ; sur la voie processeur il se pose à son rang. Deux photos qui
se chevauchent, dont celle du dessous est en chemin, montrent son cadre par-dessus sa
voisine pendant le décodage. La réponse exacte serait que la carte pose ce cadre elle-même
— un quad uni et une bordure, donc les formes de l'étape 3 — et le libellé demandera un
atlas de glyphes (§ 7). Tant que ce n'est pas fait, un artefact transitoire vaut mieux
qu'une photo invisible.

### 5.2 Le rendu par région écrivait dans la couche du dessous

Sur la voie graphique, le tampon principal n'est plus l'image : il est la couche du
dessous. Y repeindre une région au cadrage `Tout` y aurait posé un carré d'image complète
— fond opaque, photos, annotations — au milieu d'une couche transparente, composé sous les
vraies photos. Rien ne l'a montré parce que la salissure ne sait presque jamais se
localiser : « 100 % des images redessinent » est dans chaque chronique. **Le défaut
attendait le jour où l'étape 1 du plan 18 aboutirait** — c'est-à-dire le jour où l'on
croirait avoir gagné. La garde est posée.

### 5.3 BLINK-1 — le dessin du curseur lisait l'horloge

Le test d'aspect des cartes échouait au hasard depuis trois sessions ; la fiche 17 le
nomme « instable » et s'arrête là. La cause : la visibilité du curseur se déduisait de
`blink_timer.elapsed()` **dans le dessin**, à deux endroits. Le test posait un
`blink_timer` reculé d'exactement une demi-seconde ; quand le second de ses deux rendus
arrivait plus d'une demi-seconde après, la phase avait tourné.

Le test n'est que le symptôme. Le vrai défaut est que **le même état rendait deux images
différentes** — ce que la fiche 05 § 4.4 interdit (le renderer *lit*, il ne calcule pas), et
ce qui interdit de comparer deux voies pixel par pixel. La phase se décide désormais là où
le temps avance, dans la boucle de réveil qui la calculait déjà pour savoir quand se
réveiller ; le dessin lit un booléen.

Le test porte sa preuve : deux sessions à la même phase mais dont les horloges sont en
opposition — une demi-période d'écart — exigent la même image au bit près. Sur l'ancienne
implémentation, la première montrait le curseur et la seconde non.

### 5.4 `recolte` était une marque mal posée

`recolter()` est un `try_recv` sur un canal presque toujours vide — il ne peut pas coûter
1,22 ms. La marque absorbait tout ce qui la précédait, dont l'effacement du tampon du dessus :
seize mébioctets de zéros. Une marque `effacer` les sépare, et le premier lancement la
chiffre à **1,02 ms**. C'est la bande passante mémoire, et le § 7 dit ce qui la supprime.

---

## 6. Ce que le premier lancement montre, et ce qu'il ne montre pas

Seize images de démarrage, sans un geste, sur un canevas à un nœud :

```
    poste            median
    present          30,09 ms     ← le pilote au démarrage, connu (fiche 19 § 5.1)
    effacer           1,02 ms     ← le fill du dessus, nommé pour la première fois
    blit              0,86 ms     ← une seule couche : le dessous ne part plus
    annotations       0,61 ms
    docks             0,51 ms
```

**`halos` et `clear` ont disparu du profil.** C'est ce que l'étape prévoyait, et c'est tout
ce que ce lancement prouve : seize images sans geste ne disent rien de la cadence en
mouvement, ni du tempo, ni de la latence. **Le gain à l'écran n'est pas mesuré.** Il faut
une session sur le document de 482 nœuds — les gestes de midi, dans le même ordre — puis la
chronique en entier. Ce qu'elle doit montrer si le raisonnement tient : `halos` absent,
`clear` absent, `blit` autour d'une milliseconde, et le tempo descendu de cinq balayages à
deux ou trois.

---

## 7. Le chantier suivant : le texte comme composant

### 7.1 Ce que l'utilisateur demande, et ce que le profil désigne

*« Les interfaces texte subissent le traitement de la pixelisation alors qu'on pourrait
faire BEAUCOUP plus élégant mathématiquement en comprenant pas ce bloc texte comme une image
mais comme un composant. »* Jamais traité. Et une fois le fond et les lueurs partis, ce qui
reste au processeur est **presque entièrement du texte** : `annotations`, `ui`, `docks` —
plus `effacer` et `blit`, qui n'existent que parce que ce texte se téléverse en couche.

### 7.2 Ce que la recherche dit, et elle sépare deux cas

Glucose a deux sortes de texte, et elles ne se rendent pas pareil :

* **la chrome** — barre, onglets, panneaux, minimap — ne zoome jamais. Le bon outil est un
  **atlas de glyphes rastérisés à leur taille, quantifiés sur quatre phases sous-pixel**, et
  c'est exactement ce que `typography.rs` fait déjà sur le processeur (GLYPH-1). Le
  déplacer sur la carte revient à téléverser l'atlas une fois et à poser des quads. Rasmus
  Barringer décrit la variante qui pré-filtre les quatre phases dans les quatre canaux d'un
  texel, ce qui rend le filtrage linéaire exact entre phases ;
* **le texte des cartes** zoome avec la vue, et c'est lui que l'utilisateur voit pixeliser.
  Un atlas bitmap se dégrade au zoom ; un **champ de distance signé multi-canal** (MSDF)
  reste net à toute échelle et garde les coins. Son prix, et les guides le disent sans
  détour : **il perd le hinting à petite taille** — sous douze à quatorze pixels, un glyphe
  MSDF est moins net qu'un glyphe rastérisé. Il ne remplace donc pas l'atlas ; il le
  complète, au-delà d'une taille où le hinting ne compte plus.

C'est une spécialisation par cas, pas un outil unique — la fiche 21 le demande.

### 7.3 Ce que cela supprime, et ce que cela ne supprime pas

Si la chrome et les annotations se posent en quads depuis un atlas, la couche du dessus ne
porte plus que les ornements et les repères du geste — quelques rectangles. Elle ne se
téléverse plus en entier ; `effacer` et `blit` disparaissent avec elle, comme `halos` et
`clear` viennent de le faire. Ce qui reste au processeur est ce qu'il fait de mieux : la mise
en page, le culling, les décisions.

Ce que cela ne supprime pas : le rastériseur de glyphes lui-même, qui reste le
processeur — il remplit l'atlas. Et la voie processeur reste entière, comme la charte
l'exige.

### 7.4 L'ordre, et ce qui valide chaque pas

1. **L'atlas de la chrome sur la carte** — le cas simple, taille fixe, et c'est un poste
   mesuré (`ui`, `docks`). *Preuve* : la chrome de la voie graphique et celle du processeur
   au bit près, puisque c'est le même atlas.
2. **Les annotations par atlas, aux tailles où le hinting compte** — le texte des cartes à
   l'échelle 1 est du texte de treize pixels. *Preuve* : `tests/voies_suite.rs` tient.
3. **MSDF au-delà** — pour les cartes zoomées, où le bitmap floute. *Preuve* : un banc qui
   zoome une carte de ×1 à ×8 et mesure la netteté d'un bord.
4. **Les formes** — le cadre d'une photo en chemin, les rectangles de sélection — sur la
   carte, à leur rang. *Preuve* : l'écart d'ordre du § 5.1 disparaît.

---

## 8. Ce qui reste, chiffré

| # | Ce que c'est | Chiffre | Statut |
|---|---|---|---|
| 1 | **Le gain à l'écran** | attendu : tempo de 5 à 2-3 balayages | **Non mesuré** — c'est la première chose à faire |
| 2 | **Le texte** | `annotations` + `ui` + `docks` + `effacer` + `blit` ≈ 4 ms | Plan § 7, non commencé |
| 3 | **L'arbitre** (fiche 21, étape 2) | l'application tourne sur l'Arc intégré, pas sur la RTX | Non commencé ; le choix d'adaptateur est aujourd'hui celui de `wgpu` |
| 4 | **Le SIMD à l'exécution** | facteur 3 à 4 sur la voie processeur, rien de fait | `is_x86_feature_detected` absent du dépôt |
| 5 | **Le gel de démarrage** | `present` 30 ms sur les seize premières images | Fiche 19 § 5.1, non résolu |
| 6 | **Le cas sRGB** | une surface sans format linéaire assombrit les couches ET les passes | Cohérent entre elles, faux par rapport au processeur ; jamais rencontré sur cette machine |
| 7 | **`let _ = theme;`** dans `folder.rs` | interdit par la fiche 05 § 6.1 | Vu, non corrigé |

---

## 9. Trois leçons de plus, pour la liste des fiches 17, 19 et 20

1. **Un commentaire qui décrit une intention passe pour une description.** « Le processeur
   porte le cadre en chemin dans la couche du dessus » était vrai dans la tête de qui l'a
   écrit, et faux dans le code pendant trois commits. Un commentaire ne casse pas la build ;
   un test qui compte, si.
2. **Une image à côté d'une autre vaut mille tests de passe.** Les trois régressions de
   l'étape 1 ont la même forme, et aucune n'était visible autrement qu'en regardant l'image
   entière. La capture de la voie graphique est désormais un outil du projet, au même titre
   que le témoin.
3. **Un test qui passe avec une borne large ne prouve rien.** Toutes les bornes de cette
   session ont été baissées à zéro pour lire la mesure, puis remontées à la mesure. La borne
   du fond, calculée à sept, a été mesurée à sept : c'est la seule fois où un raisonnement a
   été confirmé au niveau près, et c'est parce qu'on a regardé.
