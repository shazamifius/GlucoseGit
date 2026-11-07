# 87 — Une photo est un numéro : huit mille images sous dix millisecondes

> Sessions du 6 et du 7/11/2026, suite de la fiche 86. Sa parole : *« j'étais vraiment à 3 fps sur un
> plutôt bon PC, avec le CPU à 18 %, la RAM minuscule et le GPU à 1 % »*. La fiche 86 avait reproduit, mais
> sa conclusion se trompait de coupable.

## 1. Ce que cette session a retourné

**La fiche 86 voyait le gain dans les poses gardées sur la carte, et appelait « rustine » le passage de
chaînes à des entiers. La mesure dit l'inverse.** Le profil du fil qui dessine, pile d'appels comprise
(8 000 images, le vrai Glucose) :

| par photo et par image | part du fil |
|---|---|
| `poses_des_photos` (`format!`, tables de `String`, `reclamer`) | 21 % |
| `Magasin::fermer` | 9,7 % |
| `SceneGpu::assurer` (deux tours, `connait`) | 7,9 % |
| `query_rect_ranks`, **trois fois la même requête** | 7,8 % |
| `Confie::textures` (clones) | 6,9 % |
| `rendre_la_region`, `preparer`, `poser`, libérer `Confie` | 17 % |
| **ce qui part vraiment vers la carte** (`finish`, `submit`) | **3,4 %** |

Écrire les poses ne coûtait presque rien : c'était **nommer chaque photo par une chaîne** qui coûtait. Et
des poses gardées n'auraient rien fait pour un vol où l'ensemble visible change à chaque image.

**Un doute levé** : « 110 → 65 ms » n'était pas un effet de l'écran. La présentation est en `mailbox`, sans
attente, et les postes additionnés donnent bien le total.

**Une prudence qui reste** : **le même programme faisait 78 ms le 6/11 au soir et 23 ms le 7/11**. Le second
jour : sur secteur, régulateur `performance`, 3,5 GHz ; le premier, probablement sur batterie. Un facteur 3 à
code identique. **Seules les paires entrelacées valent**, et les chiffres ci-dessous sont ceux d'une machine
au mieux de sa forme.

## 2. Un banc sans fenêtre

`outils/banc_hors_ecran.sh <scénario> <sortie> [VAR=valeur…]` : le vrai Glucose et sa vraie carte graphique dans
un compositeur Wayland sans écran (`sway`, moteur `headless`), en 1920 × 1080 à 60 Hz. La fenêtre existe, Vulkan
présente, l'horloge bat ; **rien ne prend le focus, rien ne se voit**, et une capture reste possible
(`GLUCOSE_CAPTURE_A=<secondes>`). Il répond à la fiche 86 : quatre fenêtres pour deux annoncées. On peut
donc répéter des paires autant qu'il faut, pendant qu'il travaille.

`perf` accepte la pile d'appels sans toucher aux réglages de la machine avec `--user-callchains`
(`perf_event_paranoid` vaut 2 ici). `strace -T` dit ce que `perf` ne voit pas : ce qui **attend**.

## 3. ATLAS-1 — les petites photos en pages

Commencé à la fiche 86 § 4 quinquies. **Confirmé : trois paires entrelacées, un vol 110 → 78 ms, `soumettre` 23 →
1,7 ms.** Le 65 ms d'alors venait d'une seule paire.

**La comptabilité a tranché pour des cellules en puissances de deux**, de 1 à 64 pixels, une page portant
toujours 32 cellules de côté : une page suit donc ce qu'elle range. Le « 16 Ko fixe » faisait pire que gaspiller :
il aurait menti au **cran de la carte**, qui aurait cru qu'une photo réduite ne coûtait plus rien.
`octets_d_une_texture` est la seule règle, que le cran lit aussi. `octets()` compte les pages entières.

**Une éviction par cellule reste juste** : le budget lit la mémoire que le système voit, et se contracte tant que
les pages ne se vident pas. **Limite assumée** : une photo à l'écran garde sa page entière. Elles sont voisines
dans le document, donc dans la page, et le mal reste borné par ce que la carte a déjà tenu.

**Les épreuves** : les mêmes pixels avec et sans atlas (écart ≤ 1 sur 255), sur des photos de bruit voisines,
tournées, recadrées, transparentes ; une suite par lot, dans l'ordre du modèle.

## 4. PHOTO-1 — une photo est un numéro

* **`Cle`** : une photo est `Niveau(numéro, facteur)` ; tout ce qui n'a pas de numéro garde son nom. Plus de
  `format!`, plus de tables `niveaux` ni `replis` : ce qu'une photo est se lit sur elle.
* **`Cache`** : les images par numéro. Un chemin ne se cherche qu'une fois par image. **Un numéro ne sert jamais à
  deux fichiers**, même après une éviction : la carte nomme ses textures par lui, et une photo réattribuée y
  montrerait les pixels d'une autre.
* **`Rapide`** : un hachage sans dépendance. **Le mélange de FxHash, essayé d'abord, a donné 640 collisions
  entières sur 8 000 chemins** : un écart dans les bits hauts d'un mot revenait, par le pivot, dans les bits bas
  du mot suivant, là où les chiffres diffèrent aussi. C'est son épreuve qui l'a vu. La multiplication repliée (la
  forme de `foldhash`, et de `rustc-hash` 2 pour les octets) n'en a aucune.
* **`etager`** ne réexamine plus une image qu'un examen vient de déclarer réglée.
* **Une seule requête spatiale par image** au lieu de trois, un seul tour pour voir ce qui manque.

## 5. LOT-1 — tant que rien ne change, une similitude

Si `p` est une pose écrite à la vue `(x₀, y₀, s₀)`, sa place à la vue `(x₁, y₁, s₁)` est **exactement** `r · p + t`,
avec `r = s₁ / s₀` et `t = v₁ − r · v₀`. **Un rapport et une translation pour toutes les photos.** La carte garde
donc les poses de la dernière image complète, et le nuanceur applique la similitude.

**Aucune coordonnée du monde**, contrairement à la conception de la fiche 86 : les poses restent celles d'une
image ordinaire, à l'écran, là où `f32` est le plus précis ; une image complète est le cas `r = 1, t = 0` ; et il
n'y a qu'une loi, écrite en une ligne de nuanceur.

**Rien ne se décide par un seuil** : on compare ce dont une pose dépend. La version du document **et aucun geste
ouvert** (elle n'avance qu'à la fin d'un geste), le tableau, les rangs visibles, ce que le magasin tient, le cran,
le plafond, la part de la carte, et **l'échelle tant qu'elle reste dans l'intervalle où chaque photo garde son
niveau**, relevé photo par photo (`largeurs_du_meme_niveau`, la règle de `rang_pour` lue à l'envers). **Un lot
n'existe que complet.** La carte dit quel lot elle tient avant chaque image, comme elle dit sa part et son plafond.

Une image qui reprend le lot **n'ouvre pas de génération** : elle garde le numéro de la précédente. Rien ne paraît
hors de l'écran, et l'étagement, qui lirait des voulus que personne n'a écrits, ne passe pas.

**Les épreuves** : un lot repris donne les pixels d'une image complète (écart ≤ 2 sur 255), et à l'envers, sans
similitude, l'épreuve voit un écart ; **en balayant six octaves, chaque image reprise demande exactement ce qu'une
image complète aurait demandé** ; chaque dépendance, changée seule, défait le lot.

**Trouvé en l'éprouvant** : `rang_pour` portait un `|| n.largeur <= 1` qui faisait prendre le niveau d'un pixel à
une photo montrée sur deux, le niveau de deux n'étant jamais choisi. Retiré.

**Limites connues** : une photo sans octets (fichier perdu) reste « en chemin » et empêche tout lot sur son tableau ;
des photos de tailles très variées resserrent l'intervalle d'échelle, donc reconstruisent plus souvent pendant un
zoom. **À mesurer sur un vrai tableau**, le nôtre n'ayant qu'une seule taille.

## 6. JETER-1 — ce que `perf` ne voyait pas

`etages` montait à 100-170 ms sur des images où presque rien ne se passait, avec `Magasin::fermer` à 1,2 % du
processeur : **le fil ne calculait pas, il attendait.** Mon hypothèse, le verrou des files, était **fausse**. `strace`
sur le seul fil qui dessine :

* **1 300 à 2 900 `munmap` dans une seule image** : l'éviction libérait les pyramides sur le fil qui dessine, un appel
  par niveau. Elles partent désormais chez un ouvrier, **et restent comptées jusque-là** (la leçon de la fiche 83).
* **Un `close` de 166 ms** : la chronique s'écrivait sur le disque depuis le fil qui dessine, justement quand une image
  venait d'être lente. Elle a maintenant son fil.
* Les niveaux à offrir partent d'un coup, en un verrou et un réveil, au lieu d'un par un.

## 7. Le chiffre

Paires entrelacées, 8 000 images, Intel UHD 620, sur secteur, en médiane par geste :

| | avant PHOTO-1 | LOT-1 |
|---|---|---|
| vue d'ensemble, un vol | 18 ms | **4,9 ms** |
| vue d'ensemble, un zoom | 23 ms | **9,7 ms** |
| un focus, revenu au tableau entier | 33 ms | **4,9 ms** |
| un focus, pendant son zoom | 33 ms | **16 ms** |
| une échelle moyenne, un zoom | 16 ms | **13** (10,7 avec JETER-1) |
| `etages` au p99, échelle moyenne | 101 ms | **6,9 ms** (JETER-1) |

**Ce qui n'est pas prouvé** : la machine n'était pas chargée ni sur batterie pour ces mesures ; la GTX 1660 de son école,
et surtout son téléphone, n'ont rien mesuré de tout cela. **Le pire d'un zoom reste à 28 ms** : c'est une image
complète de plus, quand une photo change de niveau.

## 7 bis. CADRES-1 — sélectionner, reproduit et **pas réparé**

Son « temps énorme » : **tout choisir sur 8 000 images fait passer une image de 3,9 à 28 ms** (`outils/scenarios/choisir-tout.txt`),
et ce n'est pas la commande (3 ms). C'est l'image d'après : `blit` à 9-21 ms, `ornements` à 5 ms. Ce qui est **observé** : les cadres
se peignent au processeur, et la couche du dessus part en entier vers la carte.

Un essai a fait poser les filets des cadres droits par la carte, juste sous la couche du dessus. **Il n'a rien gagné** (`blit`
toujours à 20 ms, ce qui veut dire que quelque chose d'autre écrit encore cette couche, **non identifié**), et quatre épreuves des
deux voies tombent (écart de 74 niveaux). **Il n'est pas dans le code** : son correctif est gardé,
`docs/carnet/87-cadres-1-a-reprendre.patch`, sur `c8d727f`. De la session reste la marque de `Choix`, qui évite de hacher huit mille noms à chaque image.

## 7 ter. Sa décision : on s'arrête là

Sa parole du 7/11, en plein chantier : *« glucose n'a pas pour objectif d'être une église infime de finesse dans l'optimisation
graphique, son but premier n'est pas là ; TOUT est dans les docs »*, et *« je ne souhaite pas que ce soit encore de l'optimisation »*.
**Il avait attendu quatre heures sans comprendre où on en était** : des points en français simple, plus souvent.
La suite n'est donc pas CADRES-1 ni le reste du § 9 : c'est **le domaine** (SUITE, étape 2).

## 7 quater. Ce qui a été éprouvé

**2 768 épreuves et plus**, clippy à zéro. **Sabotages** (`outils/sabotages/huit-mille-2.json`, joués dans un arbre détaché sans
optimisation) : 53 sur 59 au premier passage, **6 aveugles**, toutes corrigées par une épreuve (une tranche qui dessine toute la
suite ; un numéro réattribué ; une image examinée deux fois ; le cadrage gardé d'une image à l'autre ; trois requêtes par image ;
le rendu qui ne reconduit pas le magasin). Le rejeu de ces six, avec les gardes de JETER-1 et JOURNAL-1, est dit dans ÉTAT.

## 7 quinquies. JOURNAL-1 — la session qui finit part tout de suite

Un journal ne partait qu'au lancement **suivant** : une session unique sur un PC prêté n'arrivait jamais (fiche 86 § 1). À la fermeture,
Glucose lance `glucose --envoyer-le-journal <fichier>` : un processus à part, comme `--sonder-moteur` ; la fenêtre n'attend pas le
réseau. **Il n'envoie que la session nommée** : un Glucose relancé entre-temps écrit la sienne, et l'envoyer à moitié la marquerait
partie pour toujours. Ce qui n'a pas pu partir part au lancement suivant, comme avant. Android attend le lancement : pas d'exécutable à relancer.

**Essayé en réel** : le vrai programme hors écran, un faux serveur local (`GLUCOSE_SERVEUR_JOURNAL`) : chaque session arrive close
(`fin` présente) juste après la fermeture, une seule fois. **La question d'accord et `docs/TELEMETRIE.md` le disent.** **Non prouvé** : un
AppImage, dont le montage disparaît quand son programme finit ; le pire est un report au lancement suivant.

## 8. Les leçons

* **Profiler avec la pile d'appels avant de concevoir.** Une session entière a conçu un atlas et des poses gardées à
  partir de profils plats, en se trompant de coupable.
* **`perf` dit ce qui calcule ; `strace` dit ce qui attend.** Un poste haut avec un processeur à 1 % n'est pas un calcul.
* **Mesurer deux fois le même programme à deux jours d'écart.** Le facteur 3 n'aurait jamais été vu autrement.
* **Une épreuve sur les données réelles voit ce que le papier promet** : 640 collisions, sous une doc qui en jurait zéro.
* **Une garde qui répond à une crainte plutôt qu'à un cas se prouve fausse** : `en_attente` supposait qu'un niveau pouvait
  devenir offrable ; c'est une propriété fixée à sa naissance, et elle aurait empêché presque toute photo d'être réglée.
* **Des objets de compilation cassés par une coupure** se lisent en `undefined reference` : `cargo clean -p glucose-desktop`.
* **Les sabotages bloquent l'arbre et prennent des heures** : dans un arbre de travail détaché, à basse priorité, sans
  optimisation (`outils/sabotages/`, METHODE § 5).

## 9. Ce qui reste, si un usage réel le demande un jour

Rien de ceci n'est le prochain chantier (§ 7 ter) :

* CADRES-1 (§ 7 bis). Un focus pendant son zoom à 16 ms, le pire d'un zoom à 28 ms.
* **Le premier lancement sur un gros tableau** : le fil qui dessine a fait ~18 000 `read` sur le document avant sa première image (5,5 s sous `strace`).
* 46 énumérations du PCI par le fil qui dessine en 30 s, d'origine inconnue (`/sys/devices/…/vendor`).
* Le % de la carte graphique manque au journal ; un vol décode-t-il à ses échelles intermédiaires ; `membranes` à 698 ms une fois, 290 ms pour 864 textes (fiche 86).
