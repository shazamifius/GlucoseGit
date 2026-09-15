# 13 — Le plan de correction : ce que l'application fait mal, et dans quel ordre

> **Rôle de ce document.** La fiche [`12`](12-PLAN-D-EXECUTION.md) dit ce qu'il reste à
> **construire**. Celui-ci dit ce qu'il faut **réparer** — et il passe devant, parce que les
> objets de la vague 2 (membranes, flèches, miroirs) réutiliseront tous la sélection, le clic,
> les poignées et la saisie. Un défaut de conception laissé là se dupliquerait dix-neuf fois.
>
> **Source** : une session d'essai réelle, faite à la main sur l'application compilée, puis
> l'instruction de chaque point dans le code des deux versions.
> **État au départ** : 915 tests verts, clippy strict à zéro — et quatorze défauts qu'aucun
> d'eux ne voyait.

---

## 1. Pourquoi 915 tests ne voyaient rien

C'est la leçon la plus chère de la session, et elle doit changer la méthode, pas seulement le
code. Les défauts trouvés se rangent en trois familles, et **aucune n'est atteignable par un
test unitaire tel qu'on les écrit ici** :

| Famille | Pourquoi le test est aveugle | Exemples trouvés |
|---|---|---|
| **Le matériel** | aucun test n'a de main, de pavé tactile ni de clavier physique | le pan à deux doigts, `Entrée`, les touches mortes |
| **Le système** | les tests vivent dans le processus ; le presse-papiers et le navigateur sont dehors | `Ctrl+C` vers une autre application, l'ouverture d'un lien |
| **Le temps** | un test mesure un appel, jamais une cadence ni une sensation | les saccades, le zoom qui « bugue » |

Il y a pire qu'un angle mort : un test qui **affirme une règle fausse**. La touche `Entrée`
avait le sien, vert, citant une « fiche 08 § 2.1 » qui parle de rendu de texte et ne dit rien
d'`Entrée`. Le test ne vérifiait pas la règle : il la fabriquait.

> **Règle de méthode retenue.** Toute correction de ce plan livre **le test qui aurait attrapé
> le défaut**. Quand aucun test ne le peut — c'est le cas des trois familles ci-dessus — elle
> livre à la place **un protocole de mesure reproductible**, et le dit explicitement. Une
> correction sans l'un des deux n'est pas finie.

---

## 2. L'inventaire complet — quatorze défauts

| # | Ce qui se voit | Ce que j'ai établi | État |
|---|---|---|---|
| 1 | `Entrée` va à la ligne **une fois sur deux** | `Maj+Entrée` sautait la ligne ; une majuscule de début de phrase tient `Maj` enfoncé | **corrigé** `bad603f` |
| 2 | Il faut **maintenir** Espace pour naviguer | Tauri **bascule** l'outil Pan et l'y laisse | **corrigé** `bad603f` |
| 3 | Le pavé tactile est « infiniment moins bon » | le pan vertical à deux doigts était **inatteignable** : Windows livre tout en `LineDelta`, que le code prenait pour une molette | **corrigé** `bad603f` |
| 4 | **Ça saccade énormément** | mesuré : le pointillé des membranes et la conversion du tampon | **corrigé, à confirmer à l'écran** |
| 5 | Zoomer/dézoomer vite « bugue » | le cran valait +12 % contre +4 % dans Tauri (corrigé) ; reste le cache de la minimap, dont la clé contient le cadrage | partiellement corrigé |
| 6 | `Ctrl+C` ne sort pas de l'application | la table des raccourcis n'avait ni `c` ni `x` | **corrigé** `5358fd7` |
| 7 | Le collé venu d'ailleurs ne marche pas | même racine que 6 | **corrigé** `5358fd7` |
| 8 | `Ctrl`+clic n'ouvre pas un lien | un test innocente la géométrie ; deux fautes corrigées dans l'ouverture système | **corrigé, à confirmer à l'écran** |
| 9 | Les poignées sont trop dures à attraper | **mesuré** : 24 px sur une image, mais une carte de texte n'avait que six poignées — même défaut que le 12 | **corrigé par le 12** |
| 10 | Un texte long devient insélectionnable | non instruit | à instruire |
| 11 | Le LaTeX n'est capté qu'entre `$` | demande explicite : une commande doit être reconnue sans délimiteur | à concevoir |
| 12 | On ne peut pas tirer la **hauteur** d'une carte | choix assumé (TEXT-FIT-1) qui divergeait de Tauri ; c'est aussi lui qui faisait passer le 9 pour un défaut de zone de clic | **corrigé** |
| 13 | Une carte naît à 240 px | la fiche 06 § 5.1 dit « libre jusqu'à 600 » | à corriger |
| 14 | Le badge d'un dossier affiche zéro | `"0"` était écrit en dur dans le rendu | **corrigé** `13708ad` |

### Ce que le témoin a révélé en cessant de planter

La capture de la scène **sélectionnée** ne se faisait plus du tout : `tiny-skia` paniquait sur
la barre d'une citation, et l'exemple s'arrêtait là. Deux captures de plus — la scène dézoomée
et la vitrine — ne se produisaient donc jamais non plus. Le filet de sécurité était coupé aux
trois quarts sans que rien ne le dise. Réparé ; et dès la première image rendue, deux défauts
neufs se voyaient :

| # | Ce que j'ai cru voir | Ce que la mesure a dit | État |
|---|---|---|---|
| 15 | La barre d'action touche le bord bas | elle garde sa marge à toutes les échelles et définitions ; la référence la pose à ~20 px, le Rust à 12 | **pas un défaut** (écart de 8 px à confirmer) |
| 16 | Une image tournée n'a aucun cadre | elle pose bien son encre : le centre des trois images est éclairci de la même façon | **pas un défaut** |

> **Et c'est la leçon de cette paire.** J'ai passé la session à répéter qu'il faut *regarder le
> PNG* — c'est ainsi qu'on a trouvé le test LaTeX vert sur un écran faux. Mais **regarder une
> capture réduite n'est pas mesurer** : deux défauts sur deux étaient des erreurs de lecture,
> et j'allais corriger du code qui marchait. Les deux méthodes se complètent et ne se
> remplacent pas : l'œil trouve ce qu'aucun test ne regarde, la mesure dit si l'œil a raison.
> Les deux tests écrits pour trancher restent — ils tiennent désormais ces deux questions.

À quoi s'ajoutent deux écarts trouvés en lisant Tauri, que personne n'avait signalés :

* **`F`** recentre la vue en Rust ; dans Tauri c'est l'outil **dossier**, et le recentrage est
  `Ctrl+Maj+F`.
* **L'IME** est absent : ni `set_ime_allowed`, ni `WindowEvent::Ime`. Les accents et les touches
  mortes passent — vérifié à la main — mais aucune langue à composition n'est possible.

---

## 3. L'ordre, et la raison de cet ordre

### Vague A — les fonctions qui mentent *(défauts 6, 7, 8, 14)*

Une fonctionnalité absente est honnête ; une fonctionnalité **présente et inerte** est un
mensonge, et c'est la faute que le dépôt passe son temps à rembourser (R-18, R-33). Elle passe
en premier parce qu'elle coûte le moins cher à réparer et le plus cher à laisser vivre : chaque
jour qui passe, on construit par-dessus en croyant le socle bon.

### Vague B — la fluidité *(défauts 4, 5, 9)*

« Tout est smooth sur Tauri » n'est pas un détail de confort : c'est ce qui fait qu'un canevas se
manipule sans y penser. Cette vague passe avant le texte parce qu'elle peut **changer
l'architecture du rendu**, et qu'il vaut mieux le découvrir avant d'écrire la mise en page des
formules par-dessus.

### Vague C — le texte *(défauts 10, 11, 12, 13)*

Le cœur de Glucose, et la partie où les décisions sont des décisions de conception, pas des
corrections. Elle vient après B parce qu'elle en dépend : la hauteur élastique d'une carte touche
sa boîte, donc l'invalidation, donc ce que la vague B aura décidé.

### Vague D — le reste du portage

La fiche 12 reprend la main, inchangée : membranes, flèches, dossiers et miroirs, puis rideaux,
export, temporalité, images, interface, recherche, persistance.

---

## 4. Le détail, chantier par chantier

### A.1 — Le presse-papiers *(défauts 6 et 7)*

**Ce qui est établi.** En édition, `Ctrl+C` produit bien `Command::Copy`, qui appelle `arboard`.
Mais `copy_selected_text` sort immédiatement si aucune session d'édition n'est ouverte : **une
carte simplement sélectionnée ne copie donc rien vers le système**. Tauri n'a pas ce trou, pour
une raison qui n'a rien d'une vertu — son éditeur est un `textarea`, et c'est le navigateur qui
copie.

**Ce que je fais.** `Ctrl+C` écrit toujours dans le presse-papiers système : le texte de la
sélection en édition, le texte des nœuds sélectionnés sinon. Le duplicata interne des nœuds
reste, il ne s'y substitue pas.

**Comment je le vérifie sans écran.** Un test d'intégration qui écrit par l'application, puis
relit le presse-papiers **depuis un second processus** — c'est exactement ce que fait un collage
dans le Bloc-notes, et c'est reproductible.

### A.2 — Les liens *(défaut 8)*

**Ce qui est établi.** `click_link_at` est bien appelé, en tête de la chaîne de clic, et `Ctrl`
n'entre en conflit avec rien : la sélection multiple est sur `Maj`. Le défaut est donc soit dans
`link_under` — qui traduit une position écran en offset d'octet dans la source, en mode *rendu*,
là où l'adresse est masquée — soit dans `open_url`.

**Comment je tranche.** Un test qui parcourt la chaîne complète depuis une coordonnée écran
jusqu'à l'URL. S'il passe, le défaut est dans l'ouverture système et se corrige côté Windows ;
s'il échoue, c'est la géométrie. Les deux cas sont réparables, et le test dit lequel.

### A.3 — Le badge d'un dossier *(défaut 14)*

Le compte se tient dans le noyau, au moment où le tableau enfant change, et non au moment du
dessin, qui ne peut pas le savoir.

### B.1 — Mesurer avant de toucher *(défauts 4 et 5)*

**Ce qui ne colle pas.** Le banc annonce 0,90 ms pour une image de mille nœuds en 1080p, et
l'application est décrite comme « laggy de fou » sur une bonne machine. L'un des deux se trompe,
et c'est presque sûrement le banc — parce qu'il mesure le **dessin**, et que l'image complète
contient deux postes qu'il ne voit pas :

* `surface.resize(w, h)` est appelé **à chaque image**, y compris quand rien n'a changé ;
* `blit_and_present` convertit tout le tampon, pixel par pixel, du format de `tiny-skia` vers
  celui de la fenêtre — 8,3 millions de pixels en 4K, à chaque image.

**Le protocole.** L'instrument existe déjà (`perf::stage`). Je l'étends à ces deux postes, je
rejoue une rafale d'événements de molette — ce qu'est un zoom continu — et je publie la
décomposition. **Aucune optimisation n'est écrite avant ce tableau.** C'est la règle que la
vague 0 a établie et qui a déjà rapporté quatre fois.

**Ce que la mesure a dit, et pourquoi mon soupçon était faux.**

Je soupçonnais le cache de la minimap : sa clé contient le cadrage, donc elle ratait à chaque
image pendant un zoom. C'était plausible et c'était **faux**. Le banc du geste
(`examples/bench_geste.rs`, neuf) mesure une image pendant que la caméra bouge — ce qu'aucun
banc ne faisait, tous rendant toujours la même image :

| déf. | nœuds | arrêt ×1 | **arrêt ×0,02** | en zoomant | en dézoomant | pire image |
|---|---:|---:|---:|---:|---:|---:|
| 1080p | 1 000 | 0,66 ms | **11,21 ms** | 0,61 ms | 2,38 ms | 16,29 ms |
| 1080p | 10 000 | 2,50 ms | **27,96 ms** | 2,34 ms | 4,14 ms | 29,32 ms |
| 4K | 1 000 | 6,65 ms | **15,63 ms** | 4,05 ms | 12,76 ms | 26,68 ms |
| 4K | 10 000 | 6,45 ms | **81,45 ms** | 6,71 ms | 11,21 ms | 83,41 ms |

La colonne « arrêt ×0,02 » est celle qui tranche, et elle n'existait que parce que j'ai failli
conclure sans elle : **au fort dézoom, une image coûte plus cher à l'arrêt qu'en mouvement.**
Le mouvement n'ajoute donc rien — le cache n'est pas en cause. Ce qui coûte, c'est le dézoom
lui-même, où le culling ne retient plus rien et où tout le document se dessine.

**Et un poste que le banc n'avait jamais compté.** `blit_and_present` traduit chaque pixel du
format de `tiny-skia` vers celui de la fenêtre, un par un, à chaque image :

| définition | conversion | pixels |
|---|---:|---:|
| 1080p | 1,07 ms | 2,1 M |
| 1440p | 2,11 ms | 3,7 M |
| **4K** | **4,28 ms** | 8,3 M |

Ce qui donne le vrai total d'une image, budget de 10 ms en regard :

| | rendu | conversion | **total** | |
|---|---:|---:|---:|---|
| 1080p, 10 000 nœuds, ×1 | 2,50 | 1,07 | **3,57 ms** | tenu |
| **4K, 10 000 nœuds, ×1** | 6,45 | 4,28 | **10,73 ms** | **dépassé, sans rien faire** |
| 1080p, 10 000 nœuds, ×0,02 | 27,96 | 1,07 | **29,03 ms** | dépassé |
| 4K, 10 000 nœuds, ×0,02 | 81,45 | 4,28 | **85,73 ms** | dépassé huit fois |

**Deux coûts distincts, deux natures.** Le dézoom est un coût de **contenu** : il est
algorithmique, donc réductible — `WorldScale::draws_detail` existe déjà et décide du niveau de
détail, reste à savoir ce qu'il laisse passer à 0,02. La conversion est un coût de **surface** :
aucun culling, aucun cache ne la réduira jamais, et c'est l'argument chiffré de la présentation
GPU que la fiche 12 range en vague 4.

**Ce que la décomposition a désigné, et qui n'était dans aucune hypothèse.** Le banc par
étapes, étendu au dézoom, nomme le poste dominant en une ligne :

```
1080p, 10 000 nœuds, échelle 0,02
total=47,54 ms   halos=7,02   membranes=20,58   images=8,06   annotations=8,07
```

Les **membranes** consomment la moitié d'une image. Et même à l'échelle 1 sur mille nœuds :
`membranes=1,12 ms` contre `annotations=0,02 ms` — cinquante fois plus cher par élément.

La cause est un travail invisible. Le tiret du pointillé est mis à l'échelle (SCALE-1), donc
leur **nombre ne dépend pas du zoom** : une membrane de cinq mille unités de périmètre en porte
deux cent cinquante à toute échelle, chacun avec deux bouts arrondis. Au fort dézoom, on
rastérisait cinq cents arcs **plus fins qu'un pixel**, par membrane et par image.

La correction ne choisit aucun seuil : *un tiret sous le pixel ne peut pas être vu comme un
tiret*. L'œil n'y lit qu'un trait continu, à moitié moins dense puisque la moitié du parcours
est vide — alors on dessine exactement cela, un trait plein d'opacité moitié. C'est le même
résultat à l'œil, et c'est tout ce que l'écran sait montrer.

Mesuré dos à dos, même machine, même charge — la première comparaison ne valait rien, la
machine s'étant chargée entre les deux exécutions, ce que la colonne de conversion a
immédiatement trahi :

| au fort dézoom | avant | après | |
|---|---:|---:|---|
| 1080p, 10 000 nœuds | 59,13 ms | **37,41 ms** | −37 % |
| 4K, 10 000 nœuds | 163,74 ms | **104,20 ms** | −36 % |
| **pire image**, 1080p | 70,28 ms | **38,10 ms** | −46 % |
| **pire image**, 4K | 209,43 ms | **121,07 ms** | −42 % |

À l'échelle 1, où le pointillé reste visible, rien ne bouge (12,66 → 12,19 ms, du bruit) et
l'empreinte de la scène témoin est inchangée : le gain ne se paie d'aucune régression.

**Ce qui reste à faire sur ce poste.** Les halos, les images et les annotations se partagent
maintenant le reste à parts égales, autour de 7 ms chacun à l'échelle 0,02. Aucun ne domine
plus : la suite n'est plus une correction ponctuelle mais la question du niveau de détail
(`WorldScale::draws_detail`), qui se pose pour l'élément entier.

**Le coût de surface, réduit d'un tiers sans rien céder.** La conversion lisait trois octets
et les recomposait à coups de décalages et de « ou ». Elle se dit en une seule opération : un
pixel vaut `r,g,b,a` en mémoire, donc `a<<24 | b<<16 | g<<8 | r` lu comme un mot ; l'échanger
bout à bout donne `r<<24 | g<<16 | b<<8 | a`, et un décalage de huit bits laisse exactement ce
que la fenêtre attend. Le processeur a une instruction pour ça, et le compilateur peut la
vectoriser — ce qu'une recomposition octet par octet lui interdit.

| définition | octet à octet | échange de mot | |
|---|---:|---:|---|
| 1080p | 1,58 ms | **1,06 ms** | −33 % |
| 1440p | 3,42 ms | **2,29 ms** | −33 % |
| 4K | 7,83 ms | **5,47 ms** | −30 % |

Un test parcourt **chaque valeur possible de chaque canal** et vérifie que le pixel rendu est
identique à l'octet près, alpha compris : une optimisation qui change une couleur n'est pas une
optimisation, c'est un défaut plus rapide.

**Ce qui reste hors de ma portée.** `present()` lui-même — la remise du tampon au système — ne
se mesure qu'avec une fenêtre. Il s'ajoute aux totaux ci-dessus, il ne s'en retranche pas.

### B.2 — Les poignées *(défaut 9)*

**Ce que dit le code.** La tolérance vaut `min(24/s, max(6/s, min(l,h) × 0,35))`. Le premier
terme est en pixels écran — il suit le zoom. Le second est en unités monde et **ne le suit pas**.
Conséquence : à zoom 1 sur une carte de 48 px de haut la cible fait 16,8 px, mais à zoom 0,25
elle fait encore 16,8 unités monde, soit **4,2 px à l'écran** — alors que la poignée dessinée,
elle, garde ses 9 px constants. Le dessin et la cible divergent, et c'est exactement la faute que
la rotation venait de nous coûter.

**Ce que je fais.** La cible devient une taille **écran**, comme le dessin, et sa seule limite
est géométrique : deux poignées voisines ne doivent pas se recouvrir, donc le rayon ne dépasse
pas le quart du plus petit côté. Le ratio `0,35`, inventé, disparaît au profit d'un `1/4` qui se
démontre. `HANDLE_SLOP_MIN_PX` disparaît avec lui.

**Le test qui le tient.** Sur une gamme de zooms et de tailles, la tolérance exprimée **en pixels
écran** ne descend jamais sous celle du dessin. C'est une propriété, pas un exemple.

### C.1 — La hauteur d'une carte *(défaut 12)*

La règle est venue de l'usage, et elle est meilleure que les deux versions existantes :

> **La liberté n'empêche pas la contrainte.**
> `hauteur affichée = max(hauteur tirée, hauteur du texte)`

TEXT-FIT-1 ne disparaît pas : il devient un **plancher**. On tire où l'on veut, la carte suit la
main ; le texte grandit, la carte pousse et refuse de descendre sous lui. Tauri obtient le même
résultat par un `ResizeObserver` qui recale après coup — ici c'est une seule expression, sans
rattrapage ni aller-retour.

Les deux poignées manquantes — `Top` et `Bottom` — reviennent, et la largeur de naissance passe à
ce que dit la fiche 06 *(défaut 13)*.

**Ce que la mesure des poignées a dit, et pourquoi elle a changé l'ordre.** La tolérance,
tracée en pixels écran sur toute la gamme de zooms, innocente les images : **24 px à toutes
les échelles utiles**, deux fois la zone de Tauri. Sur une carte de texte elle vaut 16,8 px à
l'échelle 1, puis 8,4 et 6 px en dessous — la moitié de Tauri, parce que le plafond
`min(l,h) × 0,35` est en unités monde et ne suit pas le zoom.

Mais le défaut réel était ailleurs, et la mesure l'a désigné en éliminant l'autre : une carte
de texte n'avait que **six** poignées. Viser le milieu du bord haut ne trouvait rien. Les deux
remarques rapportées — « la zone de clic est trop petite » et « on ne peut pas
redimensionner, c'est particulier » — étaient **le même défaut**, et c'est le 12 qui les
portait. Corrigé ci-dessus ; le plafond `0,35` reste à revoir, mais il ne gêne plus personne.

### C.2 — Le LaTeX sans délimiteurs *(défaut 11)*

**La demande.** Écrire une commande doit suffire : les `$` sont une syntaxe de traitement de
texte, pas un geste de canevas.

**La règle proposée, et elle se démontre.** *Un paragraphe dont le premier caractère non blanc
est une barre oblique inverse suivie de lettres est une formule.* Un texte français ne commence
jamais ainsi. Les `$` continuent de marcher pour le LaTeX **au milieu** d'une phrase, où aucune
règle de position ne peut trancher.

**Le risque, nommé.** Une formule mal formée reste alors affichée comme formule plutôt que comme
du texte — c'est déjà le comportement du mode `$`, où les délimiteurs virent au rouge, et il se
transpose tel quel.

### C.3 — Les textes longs insélectionnables *(défaut 10)*

Non instruit. Je commence par le reproduire par un test — une carte de plusieurs milliers de
caractères, un clic à la fin — avant d'énoncer quoi que ce soit. Si la cause est un coût qui
croît avec la longueur, elle relève de la vague B et elle y retourne.

---

## 5. Ce que je ne peux pas savoir, et comment je m'en passe

Trois questions étaient restées sans réponse possible. Chacune se remplace par une mesure que je
peux faire seul — et c'est une meilleure méthode que la question, parce qu'une mesure laisse une
trace que la prochaine session peut relire :

| Question posée | La mesure qui la remplace |
|---|---|
| Sur quel objet les poignées ratent-elles ? | la tolérance **en pixels écran** tracée sur toute la gamme de zooms ; si elle passe sous le dessin, le défaut est là quel que soit l'objet |
| Qu'est-ce qui saccade ? | la décomposition d'une image réelle par `perf::stage`, présentation comprise |
| Le presse-papiers est-il mal écrit ou mal lu ? | un test en deux processus, qui sépare l'écriture de la lecture |

---

## 6. La carte des défauts de Glucose Tauri

Une carte complète des défauts connus de Tauri existe, photographiée. À 8 000 pixels de large
ramenés à 2 000, les notes ne sont pas lisibles — et deviner sur une liste de défauts est le plus
sûr moyen de corriger le mauvais.

Elle reste **au plan**, sous une forme exploitable : le fichier du tableau, ou les captures des
zones une par une. Tant qu'elle n'est pas lisible, ce document ne prétend pas la couvrir. C'est
la seule ligne du plan qui ne dépend pas de moi.

---

## 7. Ce qui ne change pas

La fiche 12 reste la carte du portage, et sa vague 1 est finie à deux lignes près :

* **1.A.6 — les courbes PGFPlots** : un moteur de tracé complet pour une syntaxe d'article
  académique. Ma position n'a pas changé : c'est la ligne la moins rentable du plan, sa place est
  après la vague 2.
* **1.A.8 — les ancres de texte** : 224 lignes écrites et testées qui attachent une flèche à une
  phrase. Elles se brancheront le jour où une flèche sera sélectionnable (chantier 2.B) ; les
  brancher avant créerait une fonctionnalité sans porte d'entrée — la faute même que la vague A
  de ce document rembourse.
