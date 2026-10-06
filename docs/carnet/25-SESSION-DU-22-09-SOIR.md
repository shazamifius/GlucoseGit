# 25 — La porte d'entrée : quatre fonctionnalités, un arbitre réparé, et sept fois où la mesure a eu raison contre moi

> **Rôle de ce document.** La fiche [`24`](24-SESSION-DU-22-09.md) laissait un plan de
> performance et un arbitre qui venait d'être écrit. Entre les deux, l'utilisateur a tranché :
> *« finir complètement le logiciel, publier la V1 publique, avoir des retours et améliorer. Il
> faut surtout pas qu'on s'attarde à tailler la pierre d'une cathédrale qui ne possède pas de
> porte d'entrée. »* Cette fiche dit ce que la session en a fait : **sa liste de quatre
> fonctionnalités, entière**, plus le défaut que la session précédente avait introduit — et,
> comme toujours, la liste de ce que la mesure a refusé.
>
> **Date** : 2026-09-22, soirée · six commits, de `2c87d71` à `5d5a8a4`.
> **État vérifié** : `cargo test --workspace` exit 0, **1 451 tests verts**, clippy strict à
> zéro, douze cliquets, aucun plafond relevé. Poussé sur `main`.
>
> **Le point de départ** : une chronique de quarante-huit secondes sur l'Intel Arc où
> `present` a gelé quatre fois — 478, 221, 194 et 156 ms — sans que l'arbitre ne bouge.

---

## 1. Ce qui a été fait, en une table

| Chantier | Ce que c'est | Vérifié par |
|---|---|---|
| **ARBITRE-2** (`present/arbitre.rs`) | L'arbitre compte du **temps perdu**, plus des images gelées ; il observe **en continu** au lieu de fermer la porte au bout d'une seconde ; sa fenêtre à lui disparaît au profit de l'échantillon du tempo | 12 tests, dont 4 rejouent la session du terrain ; 2 portent chacun un des deux anciens défauts et le montrent conclure l'inverse |
| **DEPOT-WEB-1** (`plateforme/`) | Glisser une image depuis un navigateur pose l'image : une cible de dépôt COM à nous, qui lit `FileGroupDescriptorW` + `FileContents` — **le navigateur télécharge**, nous lisons un flux | Le pont s'installe (bannière) ; 8 tests sur ce qui décide ; **le dépôt réel n'est pas vérifié** |
| **EMPREINTE-1** (`plateforme/empreinte.rs`, `chronique/veille.rs`) | La chronique dit ce que le **processus** coûte : mémoire de travail, part de cœur **quand la main ne touche à rien**, et combien d'images se dessinent alors | 10 tests ; deux sessions réelles |
| **RECADRAGE-1** (`types/recadrage.rs`, schéma v2) | Une image se recadre sans toucher à ses octets : quatre fractions, un neutre, un invariant ; rendu sur **les deux voies** ; **première migration chaînée du format**, prouvée sur un vrai fichier v1 | 7 + 12 tests ; l'épreuve des deux voies sur une photo cadrée ; le nuanceur inversé fait tomber l'épreuve |
| **BORDURES-1** (`glucose_core::bordures`, `Ctrl+B`, menu contextuel) | Les bandes unies d'un lot se détectent et se retirent, en une entrée d'annulation, sans déplacer ce qu'on garde | 9 + 4 tests ; le témoin du menu regardé avant de mettre l'empreinte à jour |
| **RECADRAGE-2** (`interactions/resize.rs`) | `Alt` sur un **côté** d'image recadre, comme `Alt` sur un coin tourne | 5 tests, dont un qui disait l'inverse et est devenu plus exigeant |

---

## 2. ARBITRE-2 — le brief avait trouvé un défaut, et c'est l'autre qui décidait

La fiche 24 § 14 et le brief posaient le diagnostic : l'arbitre comptait des **images gelées**
— quatre sur 2 546, 0,16 %, sous le pour cent toléré — là où un gel de 480 ms n'est pas un
raté mais **quarante-sept images entièrement perdues**. Vrai, et corrigé : la grandeur est le
temps perdu au-delà du plancher, ramené en images. Sur la même session, 101 images perdues pour
2 546, soit **3,97 %** — vingt-cinq fois l'ancienne lecture.

**Mais ce n'est pas ce défaut qui tenait le résultat.** L'arbitre jugeait *une* fois, sur la
première seconde d'écran — 240 images — puis fermait la porte. Les quatre gels sont tombés aux
images 833, 1 204, 1 437 et 1 782. **Même avec la bonne grandeur, il n'aurait rien vu.** Deux
tests portent les deux défauts séparément, chacun montré en train de conclure l'inverse sur les
mêmes images — la méthode de la fiche 23 § 5, plutôt qu'un `git stash` qui ne prouve qu'une fois.

### 2.1 Une constante disparaît

La fenêtre « une seconde de l'écran » jugeait **la même tolérance que le tempo sur un autre
échantillon que lui**. C'est mot pour mot ce que `PART_TOLEREE` reprochait aux seuils en deux
exemplaires une fiche plus tôt, et je l'avais refait dans le même commit. `UNE_FREQUENCE` et
`ECHANTILLON` montent dans `cadence`, le tempo et l'arbitre les y lisent, et `Arbitre::nouveau`
ne demande plus la cadence de l'écran : son échantillon se compte en **images** — « à quarante
images par seconde, un pour cent cesse d'être mesurable », le tempo l'avait déjà établi.

### 2.2 Ce que la loi accepte, dit franchement

Trois images perdues sur deux cents suffisent : un hoquet isolé de quarante millisecondes dans
`present` fera essayer l'autre carte, une fois. C'est l'arbitrage du tempo, qui monte d'un cran
sur trois ratés, et son coût est borné — une reconfiguration, puis l'arbitre garde la meilleure.
L'inverse a été mesuré : une machine enfermée sur une carte qui gèle pendant toute une session.

Le premier échantillon est un **échauffement** : le gel de démarrage est daté (fiche 19 § 5.1),
et `present` vaut 30 ms sur les seize premières images (fiche 22 § 6). Sans lui, toute machine
du monde basculerait au lancement — le choix par étiquette que la charte refuse, sous un autre
nom.

---

## 3. DEPOT-WEB-1 — la recherche a démenti la prémisse, et évité une décision de charte

Le brief posait l'arbitrage ainsi : un pont Windows natif, **ou bien** télécharger l'image
depuis son adresse — donc HTTPS, donc une poignée de caisses, donc une décision qui engage la
charte, « à lui de trancher ». On lit partout qu'un navigateur « ne donne qu'une adresse ».

**C'est faux depuis 2009.** Windows a un format pour les fichiers qui n'existent pas encore —
`CFSTR_FILEDESCRIPTORW` pour les décrire, `CFSTR_FILECONTENTS` pour les lire — et Chrome comme
Firefox l'offrent pour toute image glissée hors d'une page. **C'est le navigateur qui
télécharge**, avec ses connexions, son cache et ses cookies ; nous lisons un flux. Zéro
dépendance réseau, et l'image arrive même depuis une page qui demande une authentification —
ce qu'un téléchargement de notre côté n'aurait jamais su faire. L'arbitrage n'avait pas lieu
d'être, et c'est une mesure — pas une préférence — qui l'a montré. La note de décision
[`decisions/01`](decisions/01-WINDOWS-POUR-LE-DEPOT-NATIF.md) le détaille.

### 3.1 Ce que le pont fait, et où s'arrête son `unsafe`

Trois formats, du plus sûr au plus pauvre : `CF_HDROP` (les vrais fichiers, ce que `winit`
faisait — le reprendre est le prix de lui avoir pris sa place), le descripteur et les contenus
(les octets s'écrivent dans le répertoire temporaire, et le dépôt redevient **exactement** un
fichier glissé que `interactions::drop` route sans savoir d'où il vient), l'adresse seule (posée
en carte cliquable — un repli visible vaut mieux qu'un geste sans effet).

**Et la position**, que `winit` recevait et jetait : `drop.rs` le disait lui-même — *« la
position exacte viendra avec la couche `IDropTarget` propre au projet, qui est aussi ce qui
débloquera le glisser depuis un navigateur »*. Un fichier déposé se pose désormais là où le
curseur a lâché, en convertissant les pixels de l'écran vers ceux de la zone de dessin.

Tout ce qui **décide** — le nom qu'on accepte d'une page (seule la dernière composante, ce qui
rend `..\..\x` inoffensif sans reconnaître `..`), l'endroit où les octets s'écrivent, comment
une adresse s'écrit pour rester cliquable — est dans `plateforme/moisson.rs`, sans une ligne de
Windows, testé partout. `depot_windows.rs` lit des formats et copie des octets ; il ne juge de
rien.

### 3.2 Deux choses que la mesure a refusées

* **L'ordre révoquer-puis-enregistrer.** Une fenêtre n'a qu'une cible de dépôt. Révoquer celle
  de `winit` d'emblée laisserait la fenêtre **sans aucune** cible le jour où notre
  enregistrement échoue pour une autre raison — et le glisser-déposer de fichiers, qui
  marchait, cesserait de marcher. On tente donc d'abord, et on ne révoque que si Windows dit
  explicitement que la place est prise (`DRAGDROP_E_ALREADYREGISTERED`). La bannière prouve que
  c'est ce chemin qui s'exécute.
* **La parenthèse ouvrante.** J'avais encodé la seule parenthèse *fermante* d'une adresse, en
  raisonnant qu'elle seule pouvait fermer le lien Markdown trop tôt. **Le test l'a démenti au
  premier essai** : l'ouvrante suffit à ce que la tranche cesse d'être un lien, et `Ctrl`+clic
  ne suivait plus rien. La preuve va jusqu'à `url_at` — celui que le clic emploie — au lieu de
  s'arrêter à la forme de la chaîne : *une chaîne bien formée en apparence ne prouve pas qu'un
  lien s'ouvre*. `…/wiki/Paris_(homonymie)` n'a rien d'exotique.

### 3.3 Ce qui n'est pas vérifié

Le pont **s'installe**, c'est prouvé par la bannière. Ce qui arrive quand on glisse réellement
une image depuis Chrome ne l'est pas — cela demande une main. C'est la première chose à tester.

---

## 4. EMPREINTE-1 — le critère démenti en vingt secondes

*« Mesurer ce que Glucose coûte en arrière-plan — mémoire, processeur quand on ne le touche pas.
L'idée c'est que ce soit un logiciel ultra rapide et économe. »* Cinq sessions avaient mesuré ce
qu'une **image** coûte ; de ce que le **processus** coûte, rien n'était mesuré.

On n'interroge que le système : la mémoire de travail est ce que le gestionnaire des tâches
montre, le temps processeur est ce qu'il a compté, et sa différence entre deux relevés divisée
par le temps mural donne la part d'un cœur.

### 4.1 Le premier critère était faux, et la première session réelle l'a dit

J'avais écrit : *un intervalle pendant lequel aucune **image** ne s'est rendue est du sommeil*.
Lancée vingt secondes sans qu'on y touche, l'application a rendu vingt-six images, et la
section a annoncé **« 1,4 % d'un cœur pendant qu'on s'en sert »** — alors que personne ne s'en
servait. Une application qui se réveille toute seule rend des images : compter des images
revenait à appeler « usage » ce qu'on cherchait justement à mesurer.

Le critère est donc **ce que la main a demandé**, et ce que l'application rend pendant ce temps
devient un *résultat* : *« et il dessine 1,6 image par seconde pendant ce temps — zéro est la
seule bonne réponse »*. Le décodage de fond ne compte pas comme la main : un document ouvert
puis laissé là continue de décoder, et c'est exactement « ce que Glucose coûte en arrière-plan ».

### 4.2 Ce que la première mesure dit, et il faut le regarder en face

| | |
|---|---:|
| mémoire de travail, document à un nœud | **461 à 598 Mo** |
| images dessinées par seconde, personne ne touchant à rien | **1,6** |

Les deux sont des chiffres à expliquer, et c'est la première fois que le dépôt peut les voir.
Cinq cents mébioctets pour un nœud n'est pas « économe », quelle que soit la cadence ; et 1,6
image par seconde sans raison empêche un portable de descendre dans ses états de sommeil
profond. **Aucune hypothèse avant d'avoir mesuré** — c'est le § 6.

### 4.3 Un test instable a été refusé, et sa preuve a déménagé

La première version prouvait l'unité par une mesure de charge : une boucle de 120 ms, et le
compteur devait avoir avancé d'autant. Elle a échoué dans la suite complète, pour une raison
qui vaut d'être écrite : **`GetProcessTimes` compte tous les fils du processus, et `cargo
test` en lance seize à la fois.** La mesure ne parlait pas du test mais de la suite entière. Un
test instable est pire qu'aucun test (fiche 17 § 5). La preuve est descendue là où la faute vit
— la recomposition des deux moitiés d'un `FILETIME`, une fonction pure — et **vérifiée à
l'envers** : moitiés inversées, le relevé annonce **536 870 912 000 000 µs** pour 120 000 µs de
boucle. Un nombre positif, plausible pour qui le regarde seul, et faux d'un facteur quatre
milliards.

---

## 5. RECADRAGE et BORDURES — trois critères essayés, deux refusés par un test chacun

### 5.1 Le socle, et pourquoi il ne coûte rien

Le recadrage vit dans le document en quatre **fractions** de l'original ; sans `Option`, parce
que « pas de recadrage » et « recadrage qui garde tout » sont le même état. L'invariant — il
reste toujours quelque chose à voir — se tient au constructeur, seul chemin. Un test l'a
attrapé sur l'infini dès le premier essai : ∞ × 0 = NaN, et un NaN passe tous les `> 0` sans
en satisfaire aucun.

Le rendu change presque rien, et c'est ce qui le rend sûr. **Voie processeur** : la source
entière se pose sur une boîte plus grande dont la fenêtre visible coïncide avec la boîte du
nœud, et les parts — des morceaux de la boîte — restent le clip ; le report n'itère que sur le
clip (REPORT-1), donc rien ne coûte un pixel de plus. **Voie graphique** : le quad garde sa
boîte, le nuanceur lit un sous-rectangle — quatre nombres de plus dans la pose. Une image
tournée *et* cadrée remplit un chemin d'un motif, sans masque.

### 5.2 Une image à côté d'une autre a démasqué mon test avant mon code

La première épreuve des deux voies sur une photo cadrée échouait côté carte, et le pixel seul
accusait le recadrage. Les deux PNG côte à côte ont dit autre chose : **la carte ne dessinait
pas la photo du tout**. La source que je donnais au banc ne connaissait que les composants, pas
le magasin — la closure de l'application connaît les deux. C'est le test qui était faux, en une
lecture ; le raisonnement sur le pixel aurait cherché longtemps.

### 5.3 La détection de bordures : FFmpeg, moins deux choses qu'un test a refusées

`cropdetect` balaie depuis chaque bord et compare la **moyenne** de la ligne à **24/255**. On
garde la structure et le seuil — qui n'est pas à nous, c'est le noir du signal vidéo plus le
bruit — et on mesure la distance à la couleur du bord au lieu de la clarté : une bande blanche
se trouve comme une noire.

Puis deux critères, chacun refusé par un test :

| critère | ce qu'il fait de faux | le test qui l'a dit |
|---|---|---|
| **la moyenne** (FFmpeg) | mange le haut des lettres d'une capture d'écran de code — la première ligne de pixels porte 3 % d'encre, sa moyenne reste sous le seuil | du texte fin sur fond uni |
| **la médiane** (essayée pour tenir malgré un filigrane) | prend un **damier entier** pour une bande — la moitié des pixels d'une ligne de texte sont du fond | le damier des tests, par accident |
| **le pire pixel** (retenu) | s'arrête une ligne trop tôt sur un pixel de bruit isolé — une ligne de bande reste | aucun : c'est le défaut qu'on choisit |

Le défaut de la moyenne mange du contenu, ce qui se voit ; celui du pire pixel laisse une ligne
de bande, ce qui ne se voit pas. **On choisit le défaut qui ne se voit pas.** Et un filigrane
posé dans la bande l'arrête à sa ligne, comme chez FFmpeg — assumé, et un test le dit.

Le bord droit d'une image encadrée traverse les bandes du haut et du bas, qui n'ont aucune
raison d'être de sa couleur : les quatre bords se rognent donc **ensemble**, chacun à
l'intérieur de ce que les autres ont retiré, jusqu'à ce que plus rien ne bouge. Un test l'a dit
aussi — le bord droit valait zéro.

**Ce qui trompe est écrit** : un ciel uni en haut d'une photo se fait rogner, parce que ce sont
les mêmes pixels qu'une bande et que seule l'intention les sépare. FFmpeg a le même défaut.
L'action est explicite et annulable, jamais automatique.

### 5.4 La première migration chaînée, prouvée sur de vrais octets

Le schéma passe à **v2**. Le témoin `riche-v1.glucose` a été écrit par la build du commit
`9ffb376`, la dernière à produire du v1, depuis la même fixture — il n'est pas régénérable par
cette build, **et c'est tout son intérêt** : un test qui écrirait le v1 lui-même ne prouverait
que sa propre idée du v1. La version vient du manifeste, jamais des octets : relire un v2 en
croyant lire un v1 échoue au lieu de deviner.

### 5.5 Le geste à la main n'a demandé aucune poignée nouvelle

`Alt` sur un **coin** d'image faisait tourner, et le code disait pourquoi un côté ne pouvait
pas : « un côté n'a pas d'azimut propre ». `Alt` sur un côté était donc sans effet — il
**recadre** ce côté. Chaque poignée porte le seul geste qu'elle sait faire, et le même
modificateur dit « pas la taille, autre chose ». Le test *« `Alt` sur un côté redimensionne
toujours »* avait raison et a eu tort : il exige désormais que le bord droit tiré de soixante
retire trois dixièmes à droite, que la boîte perde soixante par la droite et **pas une** par
la gauche, et que l'angle ne bouge pas.

---

## 6. Ce que cette session a refusé ou démenti — la liste

1. **Le diagnostic du brief sur l'arbitre** : juste, et pas décisif. Le second défaut — la
   porte fermée à la 240ᵉ image — tenait seul le résultat.
2. **La prémisse « un navigateur ne donne qu'une adresse »** : fausse depuis 2009. L'arbitrage
   HTTPS-ou-COM n'existait pas.
3. **« Un intervalle sans image est du sommeil »** : démenti en vingt secondes de terrain.
4. **Un test de charge pour prouver une unité** : instable par construction, seize fils en
   parallèle ; la preuve a déménagé dans une fonction pure.
5. **La parenthèse fermante seule** : l'ouvrante casse aussi le lien.
6. **La moyenne de FFmpeg** : mange le haut des lettres. **La médiane** : mange un damier.
7. **Mon épreuve GPU** : la photo n'était pas dessinée, la source du banc ignorait le magasin.
8. **Mon test tourné** : `drag_by` part de la position écran ; l'ouvrir en monde sans poser le
   curseur fait partir le geste d'un coin de l'écran (0,99 au lieu de 0,25).
9. **Mon motif « sans bande »** : cinq colonnes unies à gauche, et le détecteur les a trouvées —
   il avait raison.

Neuf, dont six sont des fautes de **test**, pas de code. C'est nouveau dans ce dépôt, et c'est
la conséquence directe d'écrire les preuves avant ou avec le code : quand le test est écrit
d'abord, c'est lui qu'on écrit de travers en premier.

---

## 7. Ce qui reste, chiffré

| # | Ce que c'est | Chiffre | Statut |
|---|---|---|---|
| 1 | **Le dépôt depuis un navigateur, à l'écran** | — | Le pont s'installe ; le geste réel n'est pas vérifié |
| 2 | **461 à 598 Mo de mémoire de travail** sur un nœud | mesuré deux fois | **Inexpliqué.** Mesurer avant toute hypothèse |
| 3 | **1,6 image par seconde sans la main** | mesuré | Inexpliqué ; le curseur ne clignote pas hors édition |
| 4 | **`bande` à 14,95 ms, `dock` = 1-2, `ornements` à 4,78** | fiche 24 § 14 | Inchangé — la performance est passée au second plan, par sa décision |
| 5 | **La vignette d'une image cadrée** | passe par le report au lieu de la vignette | À mesurer avant de décider si une vignette de la fenêtre vaut la peine |
| 6 | **Le fantôme du recadrage** — montrer ce qu'on retire pendant qu'on tire | — | Raffinement, non commencé |
| 7 | **L'empreinte sur macOS** | `task_info` non écrit | La section ne paraît pas, plutôt que zéro |
| 8 | **Le SIMD, le gel de démarrage, le constat « travail refait »** | fiche 24 § 12 | Inchangés |

---

## 8. Ce qu'il faut tester à l'écran, dans l'ordre

1. **Glisser une image depuis Chrome** vers le canevas — c'est la fonction phare, et la seule
   partie de la session qu'aucune preuve ne couvre. Puis depuis Firefox. Puis un lien depuis
   la barre d'adresse : une carte cliquable doit apparaître.
2. **Glisser des fichiers depuis l'explorateur** — le pont a pris la place de `winit`, et ce
   qui marchait doit toujours marcher, **là où le curseur lâche** et non plus au centre.
3. **`Ctrl+B` sur une image en boîte aux lettres**, puis `Ctrl+Z`. Puis sur cinquante images.
4. **`Alt` + glisser un côté d'une image** : le bord recule, l'image se rogne. `Alt` + coin
   tourne toujours.
5. **Lancer sans `GLUCOSE_CARTE`, jouer une minute** : la bannière doit dire, à un moment,
   *« arbitre : la présentation passe sur la carte rapide »* — et les gels de `present`
   doivent avoir disparu de la chronique après ce moment.
6. **Laisser l'application ouverte une minute sans y toucher**, fermer, et lire la section
   *« Ce que Glucose coûte à la machine »*.
7. **SEL-MULTI-1** de la session précédente, toujours non vérifié : presser une image déjà
   sélectionnée et déplacer plusieurs images ensemble.

---

## 9. Trois sessions de terrain, et ce qu'elles ont démenti — y compris moi, deux fois

L'utilisateur a joué trois sessions après les six premiers commits. Elles ont validé quatre
chantiers sur cinq, et **démenti mes deux hypothèses successives sur le gel du canevas**.

### 9.1 Ce qu'il a validé à l'écran, sans réserve

> *« TOUT le système avec Alt ça fonctionne parfaitement, que ce soit pour crop, pour rotate ou
> autre. Le Ctrl+Z fonctionne lui aussi parfaitement. »*

`Alt` + côté recadre, `Alt` + coin tourne, un lot s'annule d'un coup. RECADRAGE-1 et 2 sont
tenus.

Et la première mesure d'empreinte en usage réel, sur la RTX imposée à la main :

| | |
|---|---:|
| images par seconde | **103** |
| p99 de tous les gestes | 13,78 ms |
| au-dessus du plancher | 2 % |
| mémoire de travail | **307 Mo** |
| processeur **sans la main** | **0,3 % d'un cœur** sur 399 s |
| images dessinées sans la main | **0,1 par seconde** |

C'est la meilleure mesure que ce dépôt ait produite, et elle valide EMPREINTE-1 : l'application
dort vraiment quand on la laisse.

### 9.2 Le gel du canevas — deux hypothèses, deux démentis

> *« Le canva ça a freeze, mais Ctrl+O, Ctrl+S et tout ça fonctionnent parfaitement encore. »*

**Première hypothèse, et elle était fausse par construction.** J'ai d'abord cru à mes propres
essais : j'avais lancé l'application quatre fois en la tuant par délai d'attente, et les
instances restantes tenaient la carte. C'était vrai — *« 1 test sur 4 que tu as fait
fonctionnait, sinon ils étaient tous freeze ; moi j'ai lancé comme ça et aucun freeze »* — mais
ce n'était pas **tout** : le gel est revenu sur ses propres lancements.

**Seconde hypothèse : la surface refusait les images.** Elle expliquait parfaitement les deux
faits — un canevas figé pendant que l'interface répond, et treize secondes « à ne pas
dessiner ». Le contrat de la présentation ne distinguait pas « présentée » de « refusée »,
l'application effaçait la salissure et s'endormait. [`crate::present::Issue`] sépare les trois
cas, et la chronique les compte.

**La chronique suivante n'a affiché aucune ligne de refus.** Le mécanisme est juste et le
compteur est utile, mais **ce n'est pas la cause**. Il ne faut pas le laisser croire.

### 9.3 Ce que trois sessions disent de la bascule de carte, et c'est la piste

Trois sessions, même machine, même document de vingt-deux nœuds :

| | `GLUCOSE_CARTE=rapide` **au lancement** | bascule à chaud | bascule à chaud |
|---|---:|---:|---:|
| durée | 437 s | 37 s | 17 s |
| `present` pire | **2,50 ms** | 127,54 ms | **406,14 ms** |
| `soumettre` pire | 0,43 ms | 50,83 ms | 48,82 ms |
| images par seconde | **103** | 61 | 73 |
| pire gel | — | 13 099 ms | 763 ms |

**La RTX ouverte au lancement ne gèle pas. La même RTX ouverte par bascule gèle.** Trois cent
fois pire sur `present`, et `soumettre` passe de moins d'une milliseconde à cinquante.

L'explication qui tient : sur un portable hybride, la fenêtre a été créée pendant que l'Arc
pilotait l'écran, et le système l'a associée à cette carte. Recréer une chaîne d'images NVIDIA
sur ce même `HWND` force la composition à traverser le bus — **exactement le chemin hybride que
la fiche 22 § 12 décrit, et qu'ARBITRE-1 existe pour fuir**. L'arbitre soigne donc le mal en
l'infligeant d'une autre façon.

**Ce n'est pas établi** : trois sessions, dont deux courtes, et aucun banc. Mais c'est la seule
hypothèse qui explique les trois colonnes ensemble, et elle se teste en une session.

**La sortie probable, si elle se confirme** : l'arbitre **persiste son verdict** au lieu de
basculer à chaud. Il observe, conclut, écrit son choix à côté de la chronique ; le lancement
suivant ouvre directement la bonne carte, sans jamais recréer de chaîne sur une fenêtre vivante.
L'adaptation prend effet une fois, au démarrage — ce que `GLUCOSE_CARTE=rapide` fait à la main,
et que personne n'aurait à taper. C'est moins « temps réel » que la charte ne l'ambitionne ;
c'est aussi la seule forme qui ne dégrade pas ce qu'elle répare.

### 9.4 Ce qui reste du gel, et qui n'est expliqué par rien

* **763 à 13 099 ms « à ne pas dessiner »**, hors de tout code de rendu. Le poste que la
  fiche 24 § 12 nomme « jamais instrumenté » depuis deux sessions, et dont on sait maintenant
  qu'il **n'est pas** un refus de surface.
* **Le tressaut, premier du verdict et qui empire** : ×76, puis ×25, puis **×85**. Le contenu se
  pose à 1 024 px de sa trajectoire au p99, 8 002 au pire, pour 12 px d'avance attendue. La
  fidélité la plus basse tombe à **0,05**.
* **`soumettre` à 48 ms**, deux fois, sans explication.
* **`docks` à 9,91 ms au repos** — la clé trop large, désignée fiche 24 § 14.1, jamais traitée.
* **633 Mo au pire** pour vingt-deux nœuds, quand la même session finit à 325.

### 9.5 Ctrl+B — le défaut n'est toujours pas reproduit

> *« Le crop avec Ctrl+B fonctionne mal, c'est souvent PAS que du noir mais aussi du blanc ou
> autre. »*

Le cas « liseré blanc puis bande noire » **marche** : `bench_bordures` le vérifie, compression
JPEG comprise. Sur l'illustration qu'il a fournie, les quatre bords se trouvent au pixel près.
Et la mesure de sa capture d'écran dit que l'illustration **touche le fond du canevas sur ses
quatre côtés** — le contour clair qu'on croit y voir est le cadre de sélection, blanc à
quatre-vingts pour cent, qui entoure toute image sélectionnée et que `Ctrl+B` laisse en place
pour qu'on puisse annuler.

Le critère est passé du pire pixel au **centile 99** — sur son illustration, le pire pixel
s'arrêtait trois lignes trop tôt et laissait un liseré. C'est une amélioration mesurée, pas une
réponse à son cas.

`GLUCOSE_BORDURES=1` écrit, pour chaque image d'un lot, ce que la détection a trouvé en pixels
et le fichier d'où elle vient. **Il n'a pas encore été lu** : la session où la variable était
posée ne porte aucune ligne de bordures, donc aucun `Ctrl+B` n'a eu lieu.

### 9.6 Pinterest — l'instrument est posé, jamais lu

Ce qu'on glisse depuis une grille de Pinterest est un **lien**, pas une image : il faut remonter
jusqu'à `i.pinimg.com` pour que le geste marche. `CF_DIBV5` et `CF_DIB` se lisent désormais — le
bitmap que beaucoup de pages posent dans le presse-papiers du glisser, et que `Ctrl+V` lisait
déjà —, et `GLUCOSE_DEPOT=1` écrit la liste des formats qu'un dépôt portait. **Cette liste n'a
jamais été rapportée.** Sans elle, toute correction est une supposition.

