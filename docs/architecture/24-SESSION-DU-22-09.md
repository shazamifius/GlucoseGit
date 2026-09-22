# 24 — L'instrument était aveugle, et trois des quatre chantiers du plan en dépendaient

> **Rôle de ce document.** La fiche [`23`](23-SESSION-DU-21-09-SOIR.md) laissait trois
> questions ouvertes et un plan de six chantiers. Deux sessions de terrain devaient trancher
> la première — `acquerir` à 19,5 ms. Elles l'ont tranchée, et en la tranchant elles ont
> montré que **la chronique cachait le poste qui décide de tout**. Cette fiche dit ce que le
> terrain a répondu, les deux défauts d'instrument que cela a mis au jour, et la correction
> que la mesure a refusée.
>
> **Date** : 2026-09-22 · sept commits, de `7c02f25` à `c1430d3`.
> **État vérifié** : `cargo test --workspace` exit 0, **1 371 tests verts**, clippy strict à
> zéro, douze cliquets, aucun plafond relevé.
>
> **Une troisième session de terrain** a suivi les cinq premiers commits, et c'est elle qui a
> répondu : les § 9 à 11 disent ce qu'elle a dit et ce qui en est sorti.
>
> **Le point de départ** : deux chroniques jouées sur le même document de 482 nœuds sans
> photo, mêmes gestes, sur la RTX 5070 en `mailbox`, l'une avec deux images en vol et l'autre
> avec trois.

---

## 1. `acquerir` : l'hypothèse est morte, et pas comme prévu

La fiche 23 § 7 donnait `acquerir` comme premier poste de la session, à **19,5 ms en médiane
au repos**. Le commit `40758b4` avait ajouté `GLUCOSE_IMAGES` pour tester la profondeur de la
chaîne : si la chaîne n'a pas d'image libre, la demander attend qu'il s'en libère une.

| | deux images | trois images |
|---|---:|---:|
| images par seconde | **51** | 43 |
| une image reste à l'écran, médian | **19,48 ms** | 23,17 ms |
| latence médiane | **19,5 ms** | 23,2 ms |
| au-dessus de 10 ms | **8 %** | 10 % |
| judder | **16 %** | 19 % |
| tempo à 5 balayages | **53 %** | 66 % |

**Trois images en vol dégradent six indicateurs sur six.** Mais le fait qui tranche n'est pas
là : `acquerir` **n'apparaît dans aucune des deux chroniques** — ni au repos, ni en
déplacement, ni au zoom. Le sixième poste affiché vaut 0,36 ms, donc `acquerir` vaut moins.
Il n'y avait rien à gagner.

**Les 19,5 ms du 21/09 étaient une attente de synchronisation, pas une pénurie d'images.** En
`Fifo`, `get_current_texture` attend le balayage — ce que la documentation de `succession.rs`
disait déjà, et que personne n'avait rapproché du chiffre. En `Mailbox`, que la RTX offre et
que l'Arc n'offre pas, il ne bloque plus. La profondeur de la chaîne ne tenait rien.

Le réglage reste, parce qu'il est un **instrument** et non un choix de production : il a
tranché une question en une session, et une autre machine pourra la reposer.

---

## 2. Le défaut central : le tableau des postes triait par médiane

En cherchant ce qui tient le tempo, un fait saute aux yeux des deux chroniques : **`textures`
n'apparaît dans aucun des trois gestes**, alors que les **douze** images les plus lentes de
chacune sont toutes dominées par lui — jusqu'à 24,88 ms pour une seule texture.

La cause tenait en deux lignes de `rapport.rs` :

```rust
    tries.sort_by_key(|(_, h)| Reverse(h.centile(0.5)));   // par MEDIANE
    for (nom, h) in tries.into_iter().take(6) { … }        // et on coupe a six
```

Un poste dont la médiane est nulle et le p99 vaut vingt millisecondes n'y paraît jamais. Or
c'est **exactement** celui qu'on cherche quand on demande pourquoi le tempo ne descend pas :
le tempo se cale sur le centile de la distribution, **jamais sur son typique**.

> **C'est la faute de la fiche 20 § 4.5 prise à l'envers, et elle avait déjà été corrigée une
> fois.** La fiche 23 § 6 raconte le même filtre retiré de `bench_texte` la veille — « un
> poste dont la médiane est nulle et le pire vaut vingt millisecondes est exactement celui
> qu'on cherche ». Réparé dans le banc, laissé dans l'instrument qui tourne chez
> l'utilisateur.

Le tableau montre désormais l'**union** des six plus gros en médiane et des six plus gros au
p99, triée par le p99. Aucun seuil de durée n'a eu à être choisi — un seuil aurait été une
constante arbitraire — et les deux lectures restent : la médiane dit ce qu'une image coûte
d'ordinaire, le p99 dit ce qui la fait geler.

Le test porte sa preuve : il rejoue le tri par médiane sur les mêmes données et vérifie qu'il
jetait le poste, comme celui du gel isolé le fait depuis la fiche 20.

### 2.1 Ce que cela veut dire de tout ce qu'on croyait savoir

Tout ce que le projet sait de `textures` vient de la **table des images lentes**. Or la fiche
17 § 3.2 interdit précisément cette lecture : *« les images les plus lentes sont un échantillon
biaisé »*. Une image dont la texture est prête devient rapide, donc elle **quitte** cette
liste ; on n'y regardait que les cas où le mécanisme avait échoué.

**On ne sait donc pas aujourd'hui ce qui tient le p99.** Les chiffres disponibles le disent
sans ambiguïté :

```
    image mediane (zoom)   6,89 ms
    postes medians visibles  docks 1,02 + minimap 0,86 + relever 0,86
                           + blit 0,86 + effacer 0,72 + present 0,61  =  4,93 ms
```

Près de **deux millisecondes sur sept** sont dans des postes que le tableau ne montrait pas,
et 152 images sur 1 985 dépassent dix millisecondes alors que les douze dominées par
`textures` n'en font que 0,6 %. La cause des cent quarante autres n'a jamais été affichée.

---

## 3. Deux marques de plus, et la quatrième du genre

`minimap` mesurait depuis `bande`, et `draw_breadcrumb` tombe entre les deux : le fil
d'Ariane était dans la minimap.

> **Quatrième marque de ce dépôt à absorber ce qui la précède**, après `occlusion` (fiche 19
> § 4.4), `recolte` (fiche 22 § 5.4) et `blit` (fiche 23 § 2). Les trois premières ont chacune
> désigné le mauvais coupable pendant plusieurs sessions.

`ui` était pire : la marque se posait **chez l'appelant**, après le retour de `render_ui`.
Elle couvrait donc la barre d'action, le toast et le menu contextuel — et, dans le chemin du
rendu réduit, tout ce que la fonction venait de faire. Un poste qui nomme quatre choses ne
désigne rien. Elle est descendue dans `render_ui`, `ariane` est née, et `ui` ne nomme plus que
ce qui paraît sur décision de l'utilisateur.

### 3.1 Et l'échelle de l'interface n'était écrite nulle part

`bench_texte` mesure la minimap à **0,05 ms**. Le terrain la donne à **0,86**. Dix-sept fois.

Les postes de la chrome couvrent des rectangles dont la surface est proportionnelle au
**carré** de l'échelle de l'interface : à 175 %, la minimap occupe trois fois plus de pixels
qu'à 100 %. Aucune trace ne portait ce nombre, donc l'écart entre le banc et le terrain était
illisible — et c'est mot pour mot la leçon de la fiche 19 § 6.1, *une mesure qui ne dit pas où
elle a été prise ne se relit pas*, celle-là même qui avait laissé `Immediate` passer inaperçu
pendant des semaines.

La bannière dit désormais la taille de la fenêtre, l'échelle et la taille en points.

**Ce n'est pas une explication, c'est une piste** : l'échelle est la seule condition du terrain
que les bancs ne reproduisent pas, mais rien ne prouve encore qu'elle suffise à expliquer le
facteur dix-sept. La prochaine bannière donnera le nombre.

---

## 4. Ce qu'une texture de composant coûte, mesuré pour la première fois

`bench_composant` rend la **même** carte à huit échelles avec trois corps — vide, court, long
— et sépare ainsi le tampon, le cadre et le texte **sans poser une seule marque** dans le code
de production. Trois marques imbriquées auraient eu le défaut du § 3 ; trois contenus, non.

| échelle | texture | kpx | vide | long | tampon | ms/Mpx |
|---|---|---:|---:|---:|---:|---:|
| 4,0 | 967 × 527 | 510 | 0,47 | 2,10 | 0,00 | 4,13 |
| 6,0 | 1449 × 789 | 1143 | 1,31 | 3,72 | 0,00 | 3,26 |
| 8,0 | 1931 × 1051 | 2029 | 2,26 | 9,78 | 0,01 | 4,82 |

Trois faits, et le premier est une estimation fausse de plus :

1. **Le tampon ne coûte rien** — 0,01 ms pour deux mégapixels. Je lui donnais une
   milliseconde, le temps de mettre neuf mébioctets à zéro : **facteur cent**. C'est la
   troisième estimation d'ordre de grandeur démentie en deux sessions, après le facteur
   soixante et le facteur six de la fiche 23 § 8.
2. **Le coût est du remplissage pur**, trois à six millisecondes par mégapixel, dont environ
   un tiers pour le cadre et deux tiers pour le texte. Il n'y a **pas de gaspillage caché à
   supprimer** : une texture coûte ce que coûte de peindre sa surface.
3. Le premier rendu à une échelle donnée coûte plus que les suivants — c'est la rastérisation
   des glyphes à leur nouvelle taille, et c'est le cas de l'application quand un zoom franchit
   une octave.

---

## 5. La correction que la mesure a refusée : trancher le rendu d'une texture

Le budget de CASCADE-2 borne le **nombre** de textures par image. Il ne peut rien contre une
seule qui coûte treize millisecondes, que la garantie « au moins une par image » laisse
toujours passer. La suite paraissait écrite : **CASCADE-2 au grain de la ligne**, comme
CASCADE-1 l'avait fait pour les vignettes — *« quelle plus petite tranche : la ligne, parce
qu'étaler ne suffit pas si le grain reste une vignette entière »* (fiche 17 § 2.2).

`Composant::peindre(cible, depart)` a donc été écrit : la cible est une tranche du tampon,
`depart` dit à quelle ligne elle commence, la vue s'en décale d'autant.

**Le coût tient.** De deux à seize bandes, le surcoût se perd dans le bruit — c'est l'inverse
du raffinement progressif de la fiche 20 § 4.2, qui avait été retiré parce que composer
coûtait presque autant que peindre.

**Mais les octets changent.** Jusqu'à **quarante-huit niveaux** sur les bords verticaux du
cadre, et dès **deux** bandes sur une carte sans texte — donc sans le texte pour l'expliquer.
Le rastériseur de `tiny-skia` accumule la couverture d'un bord le long du chemin et n'est pas
invariant par découpe, exactement comme la fiche 22 § 11.5 le décrit d'une translation entière.

J'ai d'abord cru à une perte de précision en `f32` dans le décalage de la bande — un `f32`
porte sept chiffres, et le rastériseur travaille au 2⁻¹⁶ de pixel. **Refait en `f64`, l'écart
est identique au bit près** : 2 467, 5 929, 6 552 et 10 290 octets, les mêmes nombres. Ce n'est
pas la précision, c'est le rastériseur.

> **Pourquoi c'est rédhibitoire, et pas un détail d'un demi-niveau.** Le nombre de bandes
> dépend du budget, donc du temps. L'aspect d'une carte aurait donc dépendu de l'instant où
> elle se rend : **le même état rendrait deux images différentes.** C'est BLINK-1 sous une
> autre forme, et la fiche 05 § 4.4 l'interdit — le renderer *lit*, il ne calcule pas. C'est
> aussi ce qui rend l'épreuve des deux voies possible, donc on ne l'abandonne pas.

Tout a été retiré de la production. Le banc dit le refus, pour que personne ne le recommence.

### 5.1 Ce que ce refus désigne, et c'est plus intéressant que ce qu'il refuse

On ne peut pas découper le rendu d'une **forme anti-crénelée** sans changer ses bords. La
sortie n'est donc pas de mieux la découper : c'est qu'elle **ne soit pas rastérisée par le
processeur du tout**.

Le cadre d'une carte est un rectangle arrondi uni avec un trait — une forme analytique, et la
carte graphique en fait des millions pour rien. Le projet l'a déjà prouvé deux fois : le fond
de grille et les lueurs (fiche 22 § 3), avec des écarts de sept et un niveau, mesurés et
bornés. Un `sdRoundedBox` est plus simple qu'une gaussienne évaluée par `erf`.

C'est l'étape 4 de la fiche 22 § 7.4 — *les formes sur la carte, à leur rang* — et elle prend
alors un sens qu'elle n'avait pas : elle ne déplace pas un coût, **elle en supprime un tiers**,
et elle laisse à la texture un tampon presque entièrement transparent où seuls les glyphes
écrivent.

---

## 6. Ce qui reste, chiffré — révisé

| # | Ce que c'est | Chiffre | Statut |
|---|---|---|---|
| 1 | **Ce qui tient le p99** | 152 images sur 1 985 au-dessus de 10 ms, dont 12 par `textures` | **Inconnu.** L'instrument était aveugle ; la prochaine chronique répond |
| 2 | **La chrome** : `docks` + `minimap` + `bande` + `ariane` + `ui` | ~2 ms sur une image de 6,89 | Marques séparées ; l'échelle de l'interface manquait. **Le chantier suivant si le terrain le confirme** |
| 3 | **Le cadre des cartes sur la carte** | un tiers du coût d'une texture, **supprimé** | Fiche 22 § 7.4 étape 4, justifiée par le § 5.1 |
| 4 | **L'arbitre** (fiche 21, étape 2) | l'Arc gèle `present` à 300 ms, la RTX non | Non commencé ; `GLUCOSE_CARTE=rapide` reste à taper à chaque lancement |
| 5 | **Le texte à grande échelle** | deux tiers du coût d'une texture | MSDF, fiche 22 § 7.2. Non commencé |
| 6 | **Le SIMD à l'exécution** | facteur 3 à 4 | `is_x86_feature_detected` toujours absent du dépôt |
| 7 | **Le gel de démarrage** | ~1 s à la première seconde | Fiche 19 § 5.1, non résolu |
| 8 | **Le constat « travail refait » du verdict** | annonce x1,1 en permanence | **Dette nommée, voir § 7** |

---

## 7. Une dette d'instrument trouvée et non remboursée

Le verdict annonce, à chaque session : *« 100 % des images redessinent alors que l'image
d'avant aurait pu suffire »*. La section du dessous dit, deux lignes plus loin : *« aucune :
chaque image a été demandée par un geste »*. **Les deux ne peuvent pas être vraies.**

Le constat se calcule ainsi :

```rust
    let gravite = refait / 0.9;   // « une image sur dix qui n'a rien a redessiner
                                  //   est le minimum qu'une scene immobile devrait produire »
```

Deux défauts. D'abord **0,9 est une constante arbitraire**, et la charte demande qu'une
constante qui peut disparaître disparaisse. Ensuite et surtout, la mesure porte sur **toutes**
les images, alors qu'une image en mouvement *doit* redessiner : sur la session du 22/09,
1 890 images sur 1 985 sont en mouvement. Le constat signale comme un défaut ce qui est le
fonctionnement normal.

La mesure juste porterait sur les images **au repos** seules. Mais sa référence est alors
**zéro** — au repos, aucune image ne devrait redessiner — et le verdict est bâti sur un rapport
`mesure / référence` qui n'admet pas une référence nulle. **Ce n'est donc pas une retouche,
c'est une question de conception du verdict**, et la traiter à la va-vite aurait produit une
seconde constante arbitraire pour remplacer la première.

Nommée ici plutôt que corrigée de travers.

---

## 8. Ce que cette session ajoute à la liste des fiches 17, 19, 20, 22 et 23

1. **Un défaut d'instrument corrigé dans un banc reste entier dans l'instrument principal.**
   Le filtre sur la médiane a été retiré de `bench_texte` un soir, et il a passé la nuit et
   toute la journée suivante dans la chronique — celle qui tourne chez l'utilisateur, et dont
   toutes les décisions de cadence dépendent. **Quand une classe de défaut est trouvée quelque
   part, la chercher partout ailleurs est le même travail, et il coûte dix minutes.**
2. **Une marque mal posée, quatre fois — et la quatrième a été trouvée en lisant, pas en
   mesurant.** `minimap` absorbait le fil d'Ariane, ce qui se voyait en regardant l'ordre des
   appels. La leçon des trois premières était écrite ; personne n'était allé relire les autres
   marques.
3. **Une correction dont le coût tient peut être refusée par son aspect.** Le tranchage était
   bon au chronomètre et faux au pixel. Aucun banc de durée ne pouvait le dire ; c'est une
   comparaison d'octets qui l'a dit, en une ligne — la même méthode que les preuves d'égalité
   des deux voies.
4. **Une estimation d'ordre de grandeur de plus, fausse d'un facteur cent.** Trois en deux
   sessions, avec les facteurs soixante et six. Le protocole qui les attrape est toujours le
   même et il tient en une phrase : **écrire le banc d'abord.**


---

## 9. La troisième session, et ce que l'instrument réparé a montré

La bannière porte enfin de quoi relire les chiffres :

```
    fenetre : 2160 x 1350 pixels, interface a 150 % (1440 x 900 points)
```

**Cent cinquante pour cent.** La chrome couvre donc 2,25 fois plus de pixels que dans les
bancs, qui jouent tous à 100 %. Ce n'est pas tout l'écart — le banc donne la minimap à
0,05 ms et le terrain à 0,86 — mais c'est un facteur qu'aucune trace ne portait, et les bancs
ne reproduisent toujours pas cette condition.

Et le tableau, trié par le p99, désigne enfin ce qu'il cachait :

| geste | image médiane | les postes qui font le p99 |
|---|---:|---|
| repos | 5,79 ms | **`textures` 0,22 → 8,43**, `docks` 0,86 → 4,36 |
| déplacer la vue | 6,89 ms | **`textures` 0,22 → 9,74** (pire 12,69) |
| zoomer | 8,19 ms | **`annotations` 0,00 → 9,74**, **`textures` 0,26 → 9,74** |
| **éditer du texte** | **19,48 ms** | **`annotations` 9,74 → 12,22** |
| sélectionner | 21,54 ms | `textures` 8,04, `docks` 6,35 |

Trois faits que la session précédente ne pouvait pas voir :

1. **`acquerir` apparaît, et il est minuscule** : 0,08 ms en médiane au repos. L'hypothèse du
   § 1 est close pour de bon.
2. **`annotations` a une médiane de zéro et un p99 de 9,74 ms** au zoom. Un poste
   rigoureusement invisible dans l'ancien tableau, et il fait le p99 à lui seul.
3. **Éditer du texte coûte 19,48 ms par image** — cinquante et une images par seconde pendant
   qu'on écrit, le geste le plus cher de toute la session. Personne ne l'avait jamais mesuré,
   parce que le tableau des gestes le montrait sans que celui des postes puisse l'expliquer.

---

## 10. COMPOSANT-2 — la carte qu'on écrit

La déduction ne demande **aucune** hypothèse : `annotations` vaut 0,00 ms en médiane au zoom
et 9,74 pendant l'édition, et la seule différence entre les deux est la carte en saisie.

COMPOSANT-1 avait fait de chaque carte de texte une texture, et en avait exclu celle qu'on
édite — son curseur clignote, sa prévisualisation déborde. C'était exclure **la seule qui se
redessine à chaque image**. Et ce n'est pas la frappe qui coûtait : c'est de refaire à
l'identique *entre* deux touches.

Ce qu'elle montre ne change qu'à la frappe et deux fois par seconde pour le curseur. Son
empreinte le dit désormais — le tampon de saisie, l'étendue sélectionnée, la phase du curseur.
`goal_x` et `blink_timer` n'y sont pas, et c'est voulu : ils décident de ce que le curseur
*fera*, jamais de ce qu'il montre.

**Le gain se prouve sans chronomètre.** Une texture se refait exactement quand sa clé change :
compter les clés distinctes sur cent images de saisie immobile donne le nombre de rendus, sur
n'importe quelle machine et sans bruit de mesure.

| | avant | après |
|---|---:|---:|
| cent images sans une frappe | 100 rendus | **2** |
| sept caractères tapés | 7 rendus | 7 |

Le second test est aussi nécessaire que le premier : une clé qui ignorerait le tampon de
saisie passerait le test des deux textures **en montrant du texte périmé**. C'est la faute que
la fiche 20 § 5.4 nomme — un test qui prouve une égalité ne prouve pas un choix.

### 10.1 Le seul cas qui reste au processeur, et le seul endroit qui en décide

Une carte dont le curseur est posé sur une formule garde sa prévisualisation, et celle-ci se
pose **hors de la boîte** : son placement lit `clip.width`, qui vaut l'écran dans une passe et
la texture dans un composant. Elle basculerait à gauche au lieu de se poser à droite.

`porte_une_previsualisation` est le **seul** endroit qui en décide, et les deux appelants la
lisent. Deux tests séparés qui doivent rester d'accord finissent par ne plus l'être : ou bien
les deux dessinent la carte, ou bien aucun.

### 10.2 L'écart de reposition, et pourquoi il n'est pas nouveau

Une carte en saisie rendue à part s'écarte de 23 niveaux sur 99 canaux — là où une carte au
repos s'écarte de zéro. **La mesure dit pourquoi sans qu'on ait à le supposer** : le même test
sans saisie donne zéro, et le seul changement entre les deux est que `draw_card_frame` traite
une carte en saisie comme une carte sélectionnée — son cadre fin devient l'anneau de deux
pixels.

C'est donc le cran de couverture de `tiny-skia` déjà mesuré et borné pour la sélection (fiche
22 § 11.5), et c'est **sa** borne qu'on reprend : une seconde constante pour la même cause
finirait par diverger de la première.

J'avais d'abord soupçonné le curseur. Le test avec la phase éteinte donne exactement le même
écart : **soupçon démenti en une exécution**, comme celui de la précision `f32` du § 5.

### 10.3 La garde, et elle porte sa preuve

Déplacer un dessin d'une voie à l'autre ouvre deux fautes opposées, qu'aucune comparaison
d'images ne sépare : si personne ne la dessine, la carte **disparaît** — la forme exacte des
quatre régressions de l'étape 1 ; si les deux la dessinent, elle se compose deux fois sur
elle-même, ce qui ne se voit presque pas sur un fond sombre et fausse pourtant l'épreuve des
deux voies.

Le test **compte** au lieu de comparer, comme celui des photos en chemin. Et il a été vérifié
à l'envers : en rétablissant l'ancienne ligne dans `pass.rs`, l'encre de la couche du dessus
passe de **205 918 à 260 930 pixels** — cinquante-cinq mille de plus, soit exactement une
carte repeinte. Un test qui n'a jamais échoué ne prouve rien.

---

## 11. Trois compteurs pour ce qui reste, et une table qui cesse de mentir par omission

Les trois causes du haut de la distribution demandent des réponses opposées, et aucune durée
ne dit laquelle domine :

```
    3.8s  22.33ms zoomer   dont docks 7.70ms, bande 6.95ms, textures 1.81ms
   14.3s  20.03ms zoomer   dont textures 14.13ms, tempo 9.48ms
   67.1s  19.70ms zoomer   dont annotations 11.46ms   (text=0)
```

* **`dock`** — combien de panneaux ont été *réellement* redessinés. Le cache du dock dit
  lui-même que son gain est de 1,1× : composer un tampon coûte presque ce que coûte le dessin
  qu'il remplace. Le poste vaut 1,02 ms en médiane et 7,70 au pire, et les deux n'appellent pas
  la même réponse — une clé trop large d'un côté, une couche à part de l'autre.
* **`bande`** — la bande du haut a-t-elle été refaite. L'écart y est de trente entre le médian
  et le pire.
* **`direct`** — combien de cartes le processeur a dessinées *entières*. Depuis COMPOSANT-2, il
  ne devrait plus en rester que les pense-bêtes ; ce compteur le vérifie et chiffre ce qu'ils
  coûtent.

**Et la table cesse d'afficher ce qui ne dit rien.** Elle portait vingt-deux colonnes fixes
dont la moitié vaut zéro sur toute une session : `vign`, `file`, `perim`, `pret`, `orph`,
`abdn`, `tuiles` et `reprises` comptent des mécanismes de la voie processeur qui ne s'exécutent
pas ici. Huit colonnes de zéros occupaient la moitié de la largeur pendant que `text` et `kpx`
— celles qui désignaient la cause — se lisaient à l'autre bout de la ligne. Une colonne dont
aucune image retenue ne porte de valeur ne paraît plus, et aucun seuil n'a eu à être choisi.

---

## 12. Ce qui reste après cette session

| # | Ce que c'est | Chiffre | Statut |
|---|---|---|---|
| 1 | **La chrome** : `docks` + `minimap` + `bande` + `ariane` + `ui` | ~2 ms sur une image de 8,19, pics à 7,70 et 6,95 | Les compteurs diront si elle se **refait** ou si seule sa composition coûte |
| 2 | **`textures`** — p99 à 8-10 ms sur les trois gestes, y compris au repos | 5 ms/Mpx, un tiers de cadre | Le § 5.1 dit la sortie : le cadre analytique sur la carte |
| 3 | **Les pense-bêtes comme composants** | `direct` le chiffrera | Même mécanisme que COMPOSANT-1, jamais appliqué |
| 4 | **L'arbitre** (fiche 21, étape 2) | l'Arc gèle `present` à 300 ms | `GLUCOSE_CARTE=rapide` reste à taper à chaque lancement |
| 5 | **Une carte plus grande que l'écran** n'est plus un composant | falaise dans `Regime::composer` | **À vérifier : rien ne semble la dessiner alors** |
| 6 | **Le SIMD à l'exécution** | facteur 3 à 4 | `is_x86_feature_detected` toujours absent |
| 7 | **Les 185 ms « à ne pas dessiner »** au p99, pire 510 ms | hors de tout code de rendu | Jamais instrumenté |
| 8 | **Le constat « travail refait »** du verdict | annonce x1,1 en permanence | Dette nommée au § 7 |
