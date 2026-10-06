# 37 — Le document, et le fil qui dessine

> **Rôle de ce document.** La session du 24 au 25/09 a déroulé la fiche
> [`36`](36-LA-ROUTE-VERS-LA-V1.md) dans son ordre : la **phase 1 — le document** entière (un
> seul fondement, le geste ; l'importeur des fichiers de Tauri ; les images dans le fichier ;
> l'enregistrement continu ; la reprise après plantage ; la Time Machine), puis la **phase 2 —
> la fluidité** jusqu'où elle se mesure sans lui : la cause probable des pics de 21 à 34 ms, et
> celle de l'épreuve qui tombait au hasard. Puis, sur son « continue », le début de la **phase
> 4** : les membranes possèdent ce qu'on y dépose, et le mode Focus (§ 8). La phase 3 attend sa
> parole (§ 12). Enfin, **son premier essai** et ce qu'il a changé (§ 9).
>
> **Date** : 2026-09-25 · commits `f86a85e` à `4c2c513`, et ceux de cette fiche.
> **État vérifié** : `cargo test --workspace` exit 0, **1 646 tests verts**, clippy strict à
> zéro, `cargo fmt --check` à zéro ; l'épreuve de la carte qui tombait au hasard passe 80 fois
> sur 80 (§ 7).
>
> **Vu à l'écran** : l'import de Tauri, `Ctrl+S`, la Time Machine (§ 9.1). Le reste attend son
> prochain essai ; le § 11 dit quoi regarder.

---

## 0. En une page

| | ce qui change pour lui | commit |
|---|---|---|
| **IMPORT-1, -2** | `Ctrl+O` ouvre un document de Glucose Tauri — binaire Automerge, JSON d'avant, dossier portable — avec ses images ; le fichier de Tauri n'est jamais réécrit | `f86a85e`, `4420d54` |
| **HISTOIRE-1, -2** | chaque geste s'écrit à la suite du fichier : **plus de `Ctrl+S` qui fige** ; les images vivent **dans** le fichier, plus dans le dossier temporaire ; un document sans nom vit dans un brouillon qu'un plantage ne perd pas. Ouvrir `fuser.glucose` : **1 683 ms → 0,67 ms** | `9973e5b`, `4420d54` |
| **HISTOIRE-3** | `Ctrl+H` : la **Time Machine** — une réglette d'un trait par geste, l'aperçu du passé bordé d'ambre, « Restaurer cet état » (un geste, que `Ctrl+Z` défait), les jalons | `a5db11d` |
| **HISTOIRE-4** | le texte **en cours de frappe** survit à un arrêt brutal ; deux fenêtres ne peuvent plus écrire le même document ; fermer la fenêtre pendant une frappe ne perd plus le texte | `d0aff3f` |
| **CEDER-1** | les ouvriers de l'atelier cèdent le pas au fil qui dessine : une image de 2 ms passait à **21,6 ms** pendant une rafale de décodages, elle reste à **3,0** — à confirmer par sa chronique | `a6eda2e` |
| **L'épreuve au hasard** | ni la carte ni le processeur : deux épreuves réécrivaient le même fichier en même temps | `0c1ae32` |
| **MEMB-1** | une membrane **possède** ce qu'on y dépose : la déplacer emporte son contenu, la supprimer le libère | `2b6881e` |
| **MEMB-2** | le **mode Focus** : zoomer sur une membrane jusqu'à ce qu'elle remplisse l'écran, et n'avoir plus qu'elle, sur un fond à sa couleur | `014cb10` |
| **Son premier essai** | le lancement rouvre **toujours** le dernier document ; après un plantage, la carte revient **en édition**, caméra dessus ; une seconde fenêtre ne double plus un document ; un **bouton** Time Machine, une réglette qu'on **glisse** | `4c2c513` |

---

## 1. Une conception unique : le geste (fiche 36, 1.1)

### 1.1 Ce que font les autres

* **Figma** ([billet](https://www.figma.com/blog/how-figmas-multiplayer-technology-works/)) : le
  document est une table `objet → propriété → valeur` ; un serveur central ordonne, et entre
  deux écritures simultanées d'une même propriété, **la dernière reçue gagne**. Ni OT ni CRDT
  complet : *« OTs were unnecessarily complex for our problem space »*. Les identifiants
  portent celui du client, l'ordre des frères est une fraction.
* **Automerge** ([format binaire](https://automerge.org/automerge-binary-format-spec/)) : des
  morceaux, chacun sa somme SHA-256 ; l'histoire entière des opérations en colonnes codées par
  plages. C'est le format de Tauri, lu ici (§ 2).
* **Loro** : co-édition et voyage dans le temps intégrés, le plus rapide des comparatifs
  publics ([PkgPulse](https://www.pkgpulse.com/guides/yjs-vs-automerge-vs-loro-crdt-libraries-2026),
  [crdt-benchmarks](https://github.com/dmonad/crdt-benchmarks)). Sa propre page de performances
  a refusé la lecture (403) : **d'où la mesure du § 1.2, sur son document plutôt que sur les
  leurs.**
* **SQLite** ([WAL](https://www.sqlite.org/fileformat2.html)) : chaque trame porte une somme
  **cumulative** sur tout ce qui précède ; une écriture interrompue ne valide pas, et la
  lecture s'arrête à la dernière trame saine. C'est la chaîne des sommes du § 3.
* **Redis** ([AOF](https://redis.io/docs/latest/operate/oss_and_stack/management/persistence/)) :
  un journal en ajout seul, rejoué au démarrage, synchronisé selon une période choisie (une
  seconde). Ici, aucune période : la cadence suit le disque (§ 3.3).
* **git** : les objets nommés par l'empreinte de leur contenu. C'est le magasin d'images.
* **Ink & Switch**, *[Local-first software](https://www.inkandswitch.com/essay/local-first/)* :
  la page n'a rien rendu à la lecture ; son principe — le document appartient à l'utilisateur,
  sur son disque — est celui de ce dossier depuis la fiche 02.

### 1.2 Trois voies, mesurées sur son document

Le même scénario rejoué sur `fuser.glucose` (254 images) : 1 000 photos posées, un
déplacement toutes les vingt, une phrase de 74 lettres tapée lettre à lettre trois fois, vingt
cartes ajoutées puis retirées — **1 312 gestes**. Banc exploratoire, **resté hors du dépôt** : il
tire `automerge` et `loro`, soixante caisses, pour une décision prise une fois.

| | construire | 1 312 gestes | p99 d'un geste | écrit par geste | ouvrir | remonter à mi-chemin | mémoire |
|---|---:|---:|---:|---:|---:|---:|---:|
| **le journal du noyau** | 0,25 ms | **1,4 ms** | 0,012 ms | **124 o** | 0,45 ms | **0,25 ms** | **611 Ko** |
| Automerge 0.12 | 37,6 ms | 825,6 ms | 17,3 ms | 356 o | 24,5 ms | 58,9 ms | 1 202 Ko |
| Loro 1.16 | 5,7 ms | 53,7 ms | 0,94 ms | 375 o | 0,62 ms | 17,9 ms | 4 051 Ko |

(Deuxième passage : mêmes ordres de grandeur.) Un geste d'Automerge coûte **au p99 deux images
à 120 Hz** ; celui du journal, un centième de milliseconde.

### 1.3 Ce que cela décide, et ce que cela ne décide pas

**Décidé** : l'enregistrement, le retour dans le temps et bientôt le MCP écrivent et lisent la
même chose — la transaction du journal (JRN-1), avec son avant et son après. Aucune caisse.

**Pas décidé** : la fusion de deux mains. Le banc ne mesure aucun conflit, et c'est ce qu'un
CRDT apporte. Le co-working (phase 5) partira de ce que fait Figma — un serveur qui ordonne, la
dernière écriture d'une propriété qui gagne —, que les tranches `avant / après` du journal
portent déjà ; il reste à le concevoir, pas à le supposer.

---

## 2. Lire les documents de Tauri (IMPORT-1, IMPORT-2)

Un lecteur à nous, dans le noyau, **sans aucune caisse** (note
[`decisions/03`](decisions/03-LIRE-LES-DOCUMENTS-DE-TAURI.md)) : les morceaux d'Automerge, les
colonnes par plages lues paresseusement, l'état reconstruit par les règles du format
(visibilité par les successeurs, horloge de Lamport, arbre RGA), DEFLATE écrit ici (RFC 1951),
un lecteur JSON sans récursion. La bibliothèque de référence n'entre que dans les épreuves,
comme **oracle**.

* Sur ses quatre documents Automerge : **identique à la référence, arbre pour arbre**, et
  **274 images sur 274** retrouvées dans le magasin de Tauri, chacune vérifiée contre son nom.
* Ses fichiers réels n'ont ni frères concurrents ni conflits : des sabotages de l'ordre RGA ne
  les changeaient pas. D'où des documents Automerge **fabriqués** pour les épreuves (frères,
  conflits dans les deux ordres d'acteurs, deltas, compression, 20 000 niveaux d'imbrication).
  Dix sabotages, dix épreuves qui tombent.
* `Ctrl+O` reconnaît le format à sa signature. Le document importé n'a pas de nom ; ses images
  sont scellées dans son premier fichier Rust ; **le fichier de Tauri n'est jamais réécrit.**

---

## 3. L'histoire dans le fichier (HISTOIRE-1, HISTOIRE-2)

### 3.1 La forme

```text
.glucose
├── la BASE    le conteneur v2 inchangé : en-tête, table, manifeste, document, images
└── la QUEUE   des entrées à la suite, en ajout seul :
               [nature][0;3][longueur u32][somme 8 octets] puis le contenu
               objet · geste · instantané · jalon · lien · vue
```

* **La somme chaînée** (la leçon du WAL) : `somme(k) = SHA-256(somme(k−1) ‖ nature ‖ longueur ‖
  H)[..8]`, graine prise dans la table de la base. Une fin déchirée s'arrête à la dernière
  entrée saine ; une queue ne se relit que derrière **sa** base.
* **Les objets** (la leçon de git) : les octets d'une image, nommés par leur empreinte, écrits
  une fois ; un **lien** rattache une clé du document à une empreinte. Ouvrir ne lit pas les
  images : leurs octets se vérifient le jour où on les lit.
* Un fichier v2 **est** un fichier de ce format, à la queue vide ; la première entrée fait
  passer l'en-tête à la version 3, pour qu'une build plus ancienne refuse au lieu de montrer une
  base périmée.

### 3.2 Chaque geste s'écrit (JRN-5)

Toute transaction appliquée au document — validée, défaite, refaite — passe par la file de
sortie du journal ; à chaque image, elle devient un **geste** confié au scribe. L'épreuve qui
fonde tout : un vrai `Store`, piloté par ses propres gestes, écrit après chacun ; le fichier
relu redonne exactement le document. `mutate_board_layout` vidait le journal quand le nombre
d'éléments changeait : le disque ne l'aurait jamais su. Il écrit maintenant le tableau.

### 3.3 Le scribe, et quand synchroniser — sans constante

Un fil à part écrit ce qu'on lui confie, puis `sync_data`, puis se rendort ; ce qui arrive
pendant qu'il synchronise s'écrit d'un bloc au tour suivant. La fréquence suit **la vitesse du
disque** — chaque geste sur un disque rapide, par paquets sur un lent. Redis choisit une
seconde ; ici, rien n'est choisi.

**Un instantané** s'écrit quand les gestes écrits depuis **pèsent autant que lui** : rouvrir ne
rejoue jamais plus qu'un instantané, l'histoire ne grossit que d'un facteur borné. C'est la
règle d'Automerge pour ses morceaux ; aucun seuil.

### 3.4 Ce que l'utilisateur voit

* Un document qui a un nom est **enregistré au fil de l'eau**. `Ctrl+S` pose un jalon
  (« ici, c'était bien ») et ne fige plus rien ; « Enregistrer sous » copie le fichier et son
  histoire.
* Un document sans nom écrit dans un **brouillon** (`%LOCALAPPDATA%\Glucose\brouillons`), né au
  premier geste ; le lancement suivant rouvre le plus récent qu'aucune autre fenêtre ne tient.
* Les images sont **scellées** par le scribe — lues, hachées, écrites —, jamais sur le fil qui
  dessine, puis relues par leur tranche, empreinte vérifiée. `persist/assets.rs`, qui
  restaurait les images dans `%TEMP%`, n'existe plus.
* **Mesuré** sur `fuser.glucose` (180 Mo) : ouvrir passait de **1 683 ms à 0,67 ms**.

**Sa question de la fiche 34 — où vit l'histoire** — a reçu la réponse recommandée, **dans le
fichier**. Elle reste la sienne (§ 12).

---

## 4. La Time Machine (HISTOIRE-3)

`Ctrl+H`, le raccourci de Tauri. Un panneau à droite :

* **la réglette** : un trait par geste ; les jalons plus épais, **ambrés s'ils sont nommés**,
  gris s'ils viennent de `Ctrl+S` ;
* **l'aperçu** : cliquer un point montre l'état après ce geste, relu **du fichier** depuis
  l'instantané le plus proche ; un liseré ambré borde l'écran. C'est un **bac à sable** : le
  présent et son journal sont mis de côté, rien ne s'écrit tant qu'on regarde ; `Échap`, `←`,
  `→` ;
* **« Restaurer cet état »** ne calcule aucune différence : les gestes qui ont suivi, chacun
  retourné, forment **un** geste — `Ctrl+Z` le défait, et ce qu'il défait reste dans
  l'histoire ;
* **« + Marquer un jalon »** : un nom, `Entrée`.

L'épreuve du noyau : pour **chaque** point du passé, l'état relu est celui qu'avait le document,
restaurer y ramène, et défaire rend le présent. Le panneau a été peint hors écran dans ses trois
états et regardé (`examples/apercu_temps.rs`). Six sabotages tombent.

---

## 5. Ce qui se tapait, et trois défauts trouvés en chemin (HISTOIRE-4)

### 5.1 Le texte en cours de frappe

Une saisie n'entre dans l'histoire qu'une fois validée : un arrêt perdait **tout** ce qui avait
été tapé. L'écrire à chaque touche ferait de chaque lettre un geste. Le texte en cours vit donc
**à côté** : un petit fichier `.saisie` par document, dans le dossier des brouillons, réécrit à
chaque changement (écrit à côté puis renommé : toujours entier), effacé une fois la saisie
devenue un geste.

**À quoi il s'accroche, sans horloge.** Pendant qu'on tape, rien d'autre ne change le document
— la caméra n'est pas dans le journal. Une saisie porte donc la somme de la chaîne **après le
dernier geste** : elle ne vaut que si aucun geste ne l'a suivie ; sinon, c'est que sa validation
est écrite. Une image scellée, une vue, un jalon ne la rendent pas caduque.

**L'ordre, garanti par un type.** Le scribe écrit la saisie dans la même file que les gestes,
**après** la synchronisation de son tour : une validation est sur le disque avant que son texte
en cours ne s'efface. Aucune épreuve ne peut voir l'inverse (il y faudrait une coupure de
courant à la microseconde) : toucher au fichier d'une saisie demande donc un jeton que **seule
la synchronisation fabrique**. Le sabotage ne compile plus.

Rouvrir le document rend le texte par le chemin d'une saisie validée — un geste, que `Ctrl+Z`
retire ; au lancement, un document nommé qu'une saisie attend se rouvre de lui-même. Un document
sans fichier en reçoit un dès que la frappe y change quelque chose.

### 5.2 Trois défauts, dont un antérieur

1. **Deux fenêtres sur le même document y écrivaient chacune sa suite** — la chaîne cassait à
   la première écriture de la seconde, et tout ce qu'elle écrivait ensuite était perdu à la
   relecture. Le fichier s'ouvre maintenant pour y écrire **seul** (`persist/verrou.rs`) : sous
   Windows en ne partageant que la lecture, ailleurs sous un verrou exclusif. La seconde fenêtre
   ouvre le document comme un fichier en lecture seule, ses changements vont dans un brouillon,
   et elle le dit. *Conséquence visible* : comme avec Word, l'Explorateur refuse de supprimer
   ou de renommer un document ouvert.
2. **Fermer la fenêtre pendant une frappe perdait le texte** : il n'était pas validé, et le
   document passait pour propre. Fermer — ou ouvrir un autre document — valide d'abord la carte
   en édition.
3. **Ouvrir un document pendant un aperçu du passé laissait l'écriture suspendue** : plus rien
   ne se serait écrit pour le nouveau document. Fermer revient d'abord au présent, et la
   réglette suit le document ouvert.

Sept épreuves de bout en bout (`persist::frappe`), trois dans `persist::disque` ; treize
sabotages tombent, le quatorzième ne compile plus.

---

## 6. Le fil qui dessine passe devant l'atelier (CEDER-1)

### 6.1 Le poste qui mentait

Les pires images d'un zoom portaient **21 à 34 ms** sur `docks`, qui en coûte un d'ordinaire,
alors que la chronique ne connaissait aux panneaux que **26 raisons** de se redessiner sur 7 761
images. Le poste est un chronomètre **par différence** : il comptait tout ce qui suivait la
marque `ui` — y compris `magasin.fermer()`, qui étage les niveaux d'images, évince, et
**réveille les ouvriers de l'atelier**, un par ordre. Ce coût a désormais son poste, `etages`.

### 6.2 Trois explications, une mesure

Les pics suivaient l'activité des images (8,7 ms au pire avant la mémoire par étages, sauf à
la session « dépôt » : 27,8 ms). Trois causes possibles : libérer de la mémoire sur ce fil
(mesuré : 33 µs par Mo — il faudrait 600 Mo d'un coup), des fautes de page, ou **le fil
préempté**. L'atelier lance un ouvrier par cœur sauf un, « pour que celui qui reste tienne la
cadence » — mais rien ne réservait ce cœur : même priorité que le fil qui dessine, Windows dope
un fil qu'on réveille, et les bandes de chaque image attendent derrière eux un quantum de
l'ordonnanceur, 16 à 31 ms.

Reproduit **sur sa machine** (`bench_ceder`, 16 fils logiques, 15 ouvriers pleins, une image
de 2 ms qui confie vingt ordres) :

| les ouvriers… | médiane | p99 | pire |
|---|---:|---:|---:|
| à la priorité du fil qui dessine (avant) | **21,6 ms** | **37,8 ms** | 51,7 ms |
| un cran dessous | 3,0 ms | 3,6 ms | 18,2 ms |
| **un cran dessous, sans dopage** (CEDER-1) | **3,0 ms** | **3,5 ms** | 20,1 ms |

Les ouvriers cèdent désormais (`plateforme/priorite.rs`) : ils prennent tout ce que les autres
laissent, et cèdent **aussi à Blender ou Adobe** — la charte. Pas plus bas : le mode
« arrière-plan » de Windows baisserait aussi la priorité de leurs lectures et de leurs pages.

**Prouvé** : le mécanisme et sa correction, reproduits à part. **À confirmer** : que ce soit la
cause de **ses** pics. Sa prochaine chronique doit montrer `docks` redescendu autour d'une
milliseconde, et `etages` avec son coût propre. Le tempo (5 à 6 balayages, ≈ 43 i/s) suit le
**p99** et non la médiane — 6,9 ms en zoom, sous la cible de 8,3 : si les pics tombent, c'est
lui qui devrait tomber.

### 6.3 Ce que je n'ai pas fait, exprès

Les passes en bandes font naître seize fils par image (**0,51 ms en médiane, 1,36 au p99** sur
sa machine) : un groupe de fils permanent les économiserait. Mais aucune n'apparaît dans sa
chronique — sur la voie graphique qu'il utilise, elles ne tournent pas. À faire pour la voie
processeur, le jour où elle comptera.

---

## 7. L'épreuve qui tombait au hasard (fiche 35 § 4)

`photo_damier()` et `photo_a_bande()` réécrivaient **la même photo, sous le même nom** du
dossier temporaire, à chaque appel — et les épreuves de `voies_suite` tournent en parallèle.
L'atelier de l'une lisait parfois un fichier que l'autre venait de vider : une texture de
**65 × 65** au lieu de 64, ou des gris qui n'étaient pas ceux du processeur. Ni la carte, ni le
processeur. L'hypothèse de la fiche 35 (des vignettes construites selon la charge) était
fausse : l'épreuve ne fait jamais avancer leur chantier.

Les photos d'épreuve sont maintenant **nommées par l'empreinte de leurs pixels** : une photo
qui existe est déjà la bonne. Avant : 2 échecs sur 20. Après : **80 sur 80**, dont 40 avec tous
les cœurs occupés.

---

## 8. Les membranes possèdent, et le mode Focus (MEMB-1, MEMB-2)

La phase 4 commence par le lot 4.2 de la fiche 36 : les rideaux — la fonctionnalité la plus
originale de Glucose — n'existent qu'en mode Focus, et le focus n'a de sens que si une membrane
possède quelque chose. Le modèle portait `membrane_id` depuis toujours ; **rien ne l'écrivait**.

### 8.1 MEMB-1 — posséder (`2b6881e`)

La règle vient de Tauri, déjà portée dans le noyau (`membrane_space::reconcile_membership`) : un
élément appartient à la **plus petite membrane qui contient son centre**, telle qu'on la voit,
jamais à sa propre descendance. L'appartenance est **écrite**, pas redéduite à chaque image —
sinon une membrane minimisée perdrait son contenu en le rangeant — et ne change qu'à des
moments explicites :

* **à la naissance** : une carte ou une image posée dans une membrane lui appartient ;
* **au dépôt** : lâcher un glisser, pousser d'un cran au clavier — dans le même geste que le
  déplacement, qu'un seul `Ctrl+Z` défait ;
* **déplacer une membrane emporte son contenu**, et celui de ses membranes ; les flèches
  accrochées suivent, la zone redessinée couvre ce qui déborde, une membrane ne s'aimante pas
  sur ses propres membres. **C'est un écart assumé avec Tauri**, où le contenu restait sur
  place — la moitié de ce que « posséder » veut dire ;
* **supprimer une membrane libère** son contenu vers celle qui la contenait : le cadre part,
  jamais ce qu'il portait.

Tout passe par le journal : l'histoire du document rejoue ces gestes à l'identique (épreuve
fondatrice étendue). Sept épreuves sur le vrai `Store`, deux par les gestes de l'application ;
dix sabotages tombent.

### 8.2 MEMB-2 — le mode Focus (`014cb10`)

« Zoomer assez sur une membrane et n'avoir plus qu'elle. » La décision était portée dans le
noyau (`membrane_focus`) ; l'application ne l'appelait pas. À chaque image où la vue a bougé :
on **entre** quand une membrane couvre 92 % de l'écran et en contient le centre — la caméra se
cale sur elle par le vol de `F` —, on **sort** en dézoomant sous 80 % de l'échelle d'entrée ou
en s'éloignant, et un temps mort de 400 ms empêche d'osciller (les réglages de Tauri, repris
tels quels : c'est le ressenti qu'il connaît). En focus, seules la membrane et son contenu se
dessinent, le clic et le rectangle n'attrapent que ce qui se voit, et le fond prend 16 % de la
couleur de la membrane. Rien n'est retiré du document.

**Mesuré avant d'être branché** — et c'est ce qui a changé la conception :

| sur un tableau de… | redécrire tout, à chaque image (Tauri) | la carte, une fois par état | décider, par image | le masque, par modification |
|---|---:|---:|---:|---:|
| 10 000 nœuds | **8,1 ms** | 0,2 ms | **0,0003 ms** | 7,2 → **0,41 ms** |
| 100 000 nœuds | **165 ms** | 2,5 ms | **0,0004 ms** | 78 → **4,5 ms** |

Pendant un zoom, seule la vue change : la décision lit une **carte des membranes**, calculée une
fois par état du document, et ne parcourt que les membranes. Le masque se calcule en un passage
qui n'emprunte que les identifiants ; les chaînes d'appartenance — une boucle, être sous la
membrane focalisée — se règlent sur les seules membranes, puisque seule une membrane peut être
parente. Une épreuve confronte ce masque à l'ancienne description complète, sur un tableau
fabriqué pour en éprouver les recoins (imbrication, boucle, appartenances invalides, cartes
mesurées ou non, flèches) ; ses sabotages tombent. Une montée dans une chaîne est **bornée** par
le nombre de membranes : une erreur rendrait une réponse fausse, jamais une application figée.

### 8.3 Ce qui reste du lot 4.2

Les modes **minimisée** et **étirée** (la barre de modes d'une membrane sélectionnée, fiche 10),
l'alerte d'étirement, le dessin d'une membrane par glisser, son panneau d'options, sa couleur
dérivée des domaines — puis les **rideaux**, sur le focus.

---

## 9. Son premier essai, et ce qu'il a changé

### 9.1 Ce qu'il a vu, et validé

*« Tout ce qui est importation d'un ancien projet se passe parfaitement, et le Ctrl+S aussi. »*
Un document de Tauri ouvert par `Ctrl+O`, ses images, `Ctrl+S` sans gel : **vu, et juste**. La
Time Machine — réglette, « Restaurer cet état », `Ctrl+Z`, « + Marquer un jalon » — *« s'est
parfaitement déroulée »*.

### 9.2 Ce qu'il a appris — et ce que j'avais mal écrit

**Son « plantage » n'en était pas un.** « Fin de tâche » dans le Gestionnaire des tâches ne tue
pas une application qui a une fenêtre : Windows lui demande d'abord de se fermer. Glucose a
donc reçu une fermeture propre — qui, depuis HISTOIRE-4, valide la carte en cours de frappe et
l'enregistre. Son texte était sauf jusqu'à la dernière lettre ; mais au lancement suivant, il
n'y avait rien à *récupérer*, et Glucose s'ouvrait sur le document d'accueil. Pour un vrai
plantage : onglet **Détails**, clic droit sur `glucose-desktop.exe`, « Fin de tâche » — là, le
processus meurt sans prévenir.

### 9.3 Ce qui a changé (`4c2c513`)

| sa demande | la réponse | preuve |
|---|---|---|
| *« qu'au lancement Glucose ouvre automatiquement le fichier qu'il a dernièrement utilisé, ou qui a crashé, tout le temps »* | Glucose **retient le dernier document** ouvert ou enregistré ; le lancement rouvre le plus récent de ce qui l'attend — ce document, un brouillon laissé par un plantage, le document d'un texte en cours. Jamais un document qu'une autre fenêtre tient (`persist/reprise.rs`) | 3 épreuves |
| *« retomber directement sur le texte en mode édition avec la caméra dessus »* | après un vrai plantage, la carte se **rouvre en édition**, curseur au bout, la vue posée dessus à la taille où on l'écrivait (échelle 1, réduite seulement si elle ne tient pas) ; rien n'est validé, le texte reste gardé à côté | épreuve réécrite |
| les deux fenêtres, *« je savais pas fermer laquelle »* | un document qu'une autre fenêtre de Glucose écrit **ne s'ouvre plus** ailleurs : la seconde garde le sien et dit pourquoi. La sonde est l'épreuve même du scribe — l'ouvrir seul en écriture — : un antivirus qui lit le fichier ne la trompe pas. Rouvrir le document de sa propre fenêtre reste permis | 2 épreuves ; 12 épreuves anciennes rouvraient un fichier encore tenu, elles le ferment d'abord |
| quatre « Enregistré » au même geste | un `Ctrl+S` sans geste depuis le dernier jalon n'en pose pas un second | 1 épreuve |
| *« pourquoi la Time Machine n'a pas de bouton »* | un bouton **Time Machine** dans la barre, à côté du Timer, avec son icône (une horloge qu'une flèche ramène) | 1 épreuve |
| *« pouvoir glisser en maintenant la souris »* | la réglette **se tient** : tant que le bouton l'est, le passé suit le curseur, jusqu'au présent comme jusqu'au début | 1 épreuve par la souris |

**Le bouton a révélé un défaut plus ancien.** La barre cédait ses libellés à des largeurs de
fenêtre écrites en dur (1 320 et 1 050 pixels) : un bouton de plus, et le groupe de droite
sortait de l'écran avant que le seuil ne tombe. Elle se pose maintenant du plus riche au plus
sobre — tous les libellés, sans ceux de droite, des icônes seules — et garde la première qui
tient : **les deux constantes ont disparu**. Regardée à 1 440, 1 100 et 900 pixels ; une
épreuve vérifie qu'aucun bouton ne dépasse, de 900 à 2 400.

**Et une trace chez lui.** Après la suite des épreuves, un `dernier-document.txt` est apparu dans
son vrai dossier Glucose, pointant vers un fichier d'épreuve : toute application créée prenait
le dossier de l'utilisateur par défaut, et chaque épreuve devait penser à en donner un autre.
C'est l'inverse désormais : **seul le vrai lancement** (`main.rs`) y met ce dossier ; toute autre
application travaille dans un dossier temporaire qui n'appartient à personne. La trace est
effacée, et son dossier reste vide après toute la suite.

### 9.4 Les boards — son idée, mon avis (en attente de sa parole)

Un fichier `.glucose` porte **déjà** plusieurs boards : ce sont les pages d'un même document,
une seule histoire, un seul enregistrement. Ce qui manque est de pouvoir les **renommer, les
réordonner et les supprimer** (un `Ctrl+Z` les rend, la Time Machine les garde). Son idée
d'**importer un autre document dans un nouveau board** est bonne et compatible avec tout le
reste, à deux conditions : que ce soit un geste à part de `Ctrl+O` (qui doit rester « ouvrir »),
et que l'import soit **un seul geste** — ses boards, ses images scellées dans le fichier, ses
identifiants renommés pour ne jamais en heurter un — qu'un `Ctrl+Z` défait d'un coup. L'histoire
du document importé ne vient pas avec lui : la réglette y verrait deux passés entremêlés ; son
présent seul entre, et son fichier d'origine garde le sien.

---

## 10. Ce qui n'est pas fait, ou pas prouvé

* **Vu à l'écran** : seulement l'import, `Ctrl+S` et la Time Machine ; la reprise au lancement,
  la carte rouverte en édition, les membranes, le focus et la fluidité attendent son essai.
* L'instantané s'encode sur le fil qui dessine : invisible pour ses documents (un document de
  quelques centaines de Ko), à déplacer vers le scribe pour un document énorme.
* Ouvrir un autre document laisse le brouillon d'un document sans nom sur le disque — il se
  retrouve —, et les brouillons plus anciens qu'un lancement ne rouvre pas s'accumulent.
* Une saisie se retrouve par le chemin de son document : un document déplacé après un
  plantage, avant la relance, ne retrouve pas son texte.
* L'auteur d'un geste est un identifiant **par lancement** ; le co-working en demandera un par
  installation.
* Le verrou et la priorité **hors de Windows** : le verrou est écrit (`flock`) mais n'a jamais
  tourné ; la priorité n'y fait rien encore.
* **Un piège pour la phase 3** : l'état de Glucose Rust (brouillons, textes en cours, aperçus)
  vit dans `%LOCALAPPDATA%\Glucose` — le dossier où **Glucose Tauri est installé**. Vérifié sur
  le modèle NSIS de Tauri : son désinstalleur retire ses fichiers un par un puis un `RMDir` non
  récursif, et ne vide récursivement que `com.glucose.app` ; les brouillons survivent. Mais
  l'installeur de Glucose Rust ne devra **jamais** vider récursivement son dossier
  d'installation.
* La phase 3 n'a pas commencé : elle attend sa parole.
* **Les membranes** : le focus entre de lui-même, avec les réglages de Tauri — s'il surprend, ce
  sont eux qu'on ajuste. Les modes minimisée et étirée ne sont pas encore montrés (§ 8.3) : un
  document de Tauri qui en porte s'affiche en mode classique.

---

## 11. Ce qu'il faut regarder à l'écran

```text
cargo run --release > sortie-document.txt 2>&1
```

1. **Un document de Tauri** : `Ctrl+O`, un de ses anciens `.glucose` (par exemple
   `Bureau\Blender\Projet\en cours\tst.glucose`). Il s'ouvre, les images paraissent.
2. **L'enregistrement** : `Ctrl+S`, un nom. Puis travailler normalement — aucun gel.
3. **Un plantage** : taper un long texte dans une carte, **sans cliquer ailleurs**, puis tuer
   Glucose — Gestionnaire des tâches, onglet **Détails**, clic droit sur `glucose-desktop.exe`,
   « Fin de tâche » (l'onglet Processus, lui, ferme proprement). Relancer : le document revient,
   la carte en édition, la caméra dessus.
4. **La Time Machine** : `Ctrl+H`, cliquer sur la réglette (liseré ambré), « Restaurer cet
   état », puis `Ctrl+Z` ; « + Marquer un jalon ».
5. **Deux fenêtres** sur le même document : la seconde refuse de l'ouvrir, garde le sien, et
   dit pourquoi. **Le lancement** : fermer Glucose normalement, relancer — le dernier document
   revient.
6. **Les membranes** : poser une membrane (`M`), y lâcher des cartes et des photos, puis
   glisser la membrane — son contenu suit ; supprimer la membrane — le contenu reste. Zoomer
   sur une membrane jusqu'à ce qu'elle remplisse l'écran : on entre en focus (fond teinté, le
   reste disparaît) ; dézoomer en sort. Dire si cela se sent juste.
7. **La fluidité** : zoomer beaucoup sur un document plein de photos. Puis, avant de relancer,
   copier `%TEMP%\glucose-chronique\derniere-session.txt` en
   `sortie-chronique-2026-09-25-document.txt` : elle dira si `docks` est redescendu, et ce que
   coûte `etages`.

---

## 12. Ce qui attend sa parole

1. **Les boards** (§ 9.4) : que `Ctrl+O` reste « ouvrir », et que l'import d'un document dans un
   nouveau board soit un geste à part.
2. *Réglé* : **la V1 et la bascule** se feront en **deux moments** — une bêta Windows à côté de
   Tauri, puis la bascule — et **pas maintenant** : *« pour l'instant ce n'est pas encore
   mature »*. La phase 3 attend.
3. **Où vit l'histoire** : dans le fichier (fait ainsi, recommandé), sans objection de sa part.
3. **La signature de code Windows** (phase 3.5) : un certificat payant.
4. **Ce que ses utilisateurs emploient le plus** : l'ordre de la phase 4.

Et une fois : **sauvegarder `C:\Users\Administrator\.tauri\glucose_updater.key` hors de cette
machine** — perdue, plus aucune mise à jour n'atteindrait un utilisateur de Tauri.

---

## 13. Les sources

* Figma, [*How Figma's multiplayer technology works*](https://www.figma.com/blog/how-figmas-multiplayer-technology-works/).
* Automerge, [spécification du format binaire](https://automerge.org/automerge-binary-format-spec/).
* SQLite, [format du WAL](https://www.sqlite.org/fileformat2.html).
* Redis, [persistance](https://redis.io/docs/latest/operate/oss_and_stack/management/persistence/).
* Comparatifs de CRDT : [PkgPulse](https://www.pkgpulse.com/guides/yjs-vs-automerge-vs-loro-crdt-libraries-2026),
  [crdt-benchmarks](https://github.com/dmonad/crdt-benchmarks), [Loro](https://loro.dev/).
* Ink & Switch, [*Local-first software*](https://www.inkandswitch.com/essay/local-first/).
* Tauri, [modèle de l'installeur NSIS](https://github.com/tauri-apps/tauri/blob/dev/crates/tauri-bundler/src/bundle/windows/nsis/installer.nsi).
* Microsoft, [`SetThreadPriority`](https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-setthreadpriority)
  et [`SetThreadPriorityBoost`](https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-setthreadpriorityboost).

---

**Retour** : [`00-INDEX.md`](00-INDEX.md)
