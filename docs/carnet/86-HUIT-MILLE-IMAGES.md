# 86 — Huit mille images : ce que coûte un tableau entier à l'écran

> Enquête du 6/11/2026, **en cours**. Sa parole, après un essai de la beta 7 sur le PC de l'école :
> *« j'étais vraiment à 3 fps sur un plutôt bon PC, avec le CPU à 18 %, la RAM minuscule et le GPU à
> 1 % »* ; *« les focus, le F ou le 1234 : de très gros lags, alors qu'à la souris drastiquement
> moins »* ; sélectionner a mis *« un temps énorme à répondre »*. Et il demande que Glucose s'adapte :
> au GPU ou au CPU selon ce qui tourne à côté, la chaleur, la batterie, la machine.

## 1. Ce qui s'est appris sur les données

* **Sa session n'était pas sur Cloudflare.** Un journal ne part qu'au lancement **suivant**, sur la même
  machine. Ses deux fichiers copiés de l'école pèsent 4 et 28 Ko : ce n'est pas le poids (POIDS-1).
  **À revoir** : une session unique sur une machine prêtée n'arrive jamais.
* **L'« utilisateur inconnu » à la GTX 1660 SUPER (`5280efb2`, depuis le 12/10) est très probablement
  lui, à l'école** : même carte, et le démarrage de l'appareil de ses fichiers correspond à celui des
  sessions reçues. Les fiches 82 et suivantes le comptaient parmi des inconnus.
* **Ses journaux** (14 et 27 images ouvertes, **pas les 8 000** : ce test-là n'est pas arrivé) : au pire
  17 à 19 images par seconde, `composants` 16 ms, `textures` 7, `soumettre` 5, `cull` 3,5, sur le fil
  qui dessine ; et, une fois, `membranes` à 698 ms pour une seule image, **pas encore expliqué**.
* **Son téléphone a reparlé le 6/11** (`1c287c59`, beta 4) : `sortie` « demandée par l'utilisateur »,
  et 2,3 Go tenus (`rss_mo`), à regarder.

## 2. Reproduit ici

`fabriquer_un_grand_tableau` : 8 000 JPEG de 600 × 900, 2,3 Go. **`GLUCOSE_SCENARIO`** (`app/scenario.rs`) :
le vrai Glucose, sa vraie carte graphique, des commandes et des vols minutés, sa chronique et son
journal à la fin. Le VivoBook : i7-8550U, Intel UHD 620, 60 Hz.

| ce qu'on regarde | une image | |
|---|---|---|
| les 8 000 à l'écran, **rien à décoder**, processeur à 5 % | **46 ms** | 21 images/s |
| les 8 000 à l'écran, en décodage | 46 à 55 ms, pire 339 | |
| un signet rapproché, le même tableau | 8 ms | 126 images/s |
| le vol du départ d'un focus | 23 ms, p99 85 à 120, pire étape `etages` 102 ms | |
| `tout-choisir` : **la commande seule** | **0,9 ms** | **pas elle** |

**Le coût suit le nombre de nœuds visibles, et il se refait à chaque image** : `composants`, `textures`,
`cull`, `poses`. Une caméra qui change d'échelle à chaque image paie tout, à chaque image : c'est le
lag des focus. C'est aussi le « rien à 100 % » : le travail tient sur un fil, et la carte attend.

## 3. Le profil (`perf`, `cpu-clock`, scénario complet)

* **Le fil qui dessine n'a aucun gros coupable : une mort à petits coups.** Hachage SipHash de chaînes
  (10 % de ses échantillons), `malloc` et `free`, `memcmp`, copies d'identifiants : les nœuds sont
  nommés par des `String`, clonées et hachées par nœud et par image (`a_revoir`, `Retenues.cles`,
  `HashMap<String, Televersee>`).
* **Les ouvriers décodent massivement, y compris à la troisième ouverture** : `zune_jpeg`,
  `Pyramide::batir`, `etaler` ; et `hash::compress` à 9,5 %, à expliquer. **Le décodage à la taille
  montrée n'existe toujours pas** (fiche 83 § 6) : à la vue d'ensemble, Glucose décode chaque original
  pour en garder la vignette.
* **Ce que le profil a démenti** : « les ouvriers étouffent le dessin ». `ouvriers()` en laisse déjà un
  cœur, et leur priorité est basse (7 fils sur 8, pas 8). J'ai écrit le contraire avant de relire.

## 4. Ce qu'il reste à prouver avant de construire

* **Un vol demande-t-il des niveaux à ses échelles intermédiaires ?** Sa destination est connue d'avance :
  ne décoder que pour elle serait le « pré-optimiser » de la charte. **Hypothèse, à compter.**
* **Pourquoi les aperçus n'évitent-ils pas le décodage à la deuxième ouverture ?**
* **Le temps énorme de la sélection** : ce n'est pas `tout-choisir`. Probablement l'image qui suit, ou un
  rectangle tracé à la souris, que le scénario ne sait pas encore jouer.

## 4 bis. Son essai sur du texte, le même jour

Ses journaux de 15 h 25, en texte seul : **`lueurs` à 716 ms**, **`annotations` à 378 ms**, un pic à
1,1 s, et un coût qui monte au fil de la session (1,7 ms par image au début, 50 ms à la fin). Des postes
que le banc des images ne faisait pas parler. `examples/bench_noeuds_choisis` les reproduit :

| 4 000 textes serrés (voie processeur) | avant | après |
|---|---|---|
| le document change à chaque image | 1 840 ms | **296 ms** |
| tout choisi | 411 ms | 354 ms (353 à 2 000 : ne dépend plus du nombre de choisis) |

* **TEINTES-1** (`renderer/hue.rs`) : le carré. Chaque changement oubliait toutes les teintes, et chacune
  mélange tout son voisinage (rayon de 1 200). La cascade de la charte : une image recalcule dans sa
  tranche, montre la teinte d'avant pour le reste, et le réveil finit le travail. **Sans tranche, le
  calcul reste entier** : épreuves et bancs déterministes. Le mélange est aussi le même au bit près.
* **CHOIX-1** (`store/choix.rs`) : la sélection reste une liste à la lecture, et répond en temps constant.
* **Ce que CHOIX-1 n'a pas réglé** : sur les 8 000 **images**, sélectionnées ou non, le vrai Glucose coûte
  pareil : `composants` 25 ms, `textures` 23, `soumettre` 16 à 23. Le « temps énorme » de sa sélection
  d'images reste à reproduire. La barre d'action comparait bien images × choisies (64 millions à huit
  mille), mais seulement là où elle se montre : **à revoir à son écran**.
* **`atelier` à 43-66 ms pendant les vols** : ce repère compte tout depuis le repère d'avant. **À expliquer**,
  pas à conclure.
* **Il reste environ 290 ms** dans ce banc pour 864 textes à l'écran : c'est le processeur qui les dessine.
  Le chemin par la carte graphique se juge à son écran.

**Vérification** : 2 718 épreuves vertes, 7 ignorées, clippy à zéro ; `huit-mille.json`, 9 sabotages sur 9.

**La télémétrie ne disait ni la charge ni le nombre de nœuds**, et il l'a remarqué. **Le serveur refuse
toute session portant un champ ou une sorte de ligne inconnus** (`valider.js`) : ce qui est neuf doit donc
lui être appris **avant** la version qui l'écrit, sinon ses journaux cessent d'arriver. **CHARGE-1** est une
sorte de ligne à part, `charge`, plutôt que des champs optionnels de `memoire`, qui ne s'écrit qu'aux
records : la charge se lit dans le temps. **Le serveur est déployé le 6/11**, avant la version qui
l'écrit ; `verifier_serveur.py` l'interroge sans rien écrire (une session qu'il refuse à sa dernière
ligne), et a d'abord été vu tomber. **Sa limite** : cinq secondes moyennent le repos entre deux vols
d'une seconde (fil à 1-24 % sur le scénario, sans rien démentir) : elle lira les lags longs, pas un focus
isolé. **La carte graphique n'y est pas**, faute d'une lecture commune aux systèmes.

## 4 ter. Le profil qui tranche, et la forme du coût

`perf`, le vrai Glucose, des vols autour de la vue d'ensemble : les 8 000 images restent à l'écran, **rien à
décoder**. Le fil qui dessine : **16 803 échantillons** ; chaque ouvrier : **environ 300**. Il travaille
56 % du temps total, tempo et attente de l'écran compris. Son « rien n'est à 100 % » est donc bien un seul
fil saturé.

Où vont ses échantillons : **51 % dans Glucose, 28 % dans la libc, 19 % dans le pilote Vulkan.** Comparer et
hacher des chaînes (`memcmp` 7 %, SipHash 8 %), allouer et rendre (`malloc` et les siens 13 %), et le suivi
par `wgpu` de chaque liaison.

**La forme** : pour chaque photo visible et à chaque image, un nom fabriqué (`format!("{src}@{facteur}")`),
rangé dans deux `HashMap<String>` ; `connait` qui le cherche ; une liaison et un dessin par photo
(`set_bind_group` puis `draw`, huit mille fois). **Le code disait d'avance quand changer** : *« un atlas ne
se prend que si un banc la réclame »*. Le banc la réclame.

## 4 quater. La conception

**La vue d'ensemble : un atlas, un seul dessin.** Quand une photo tient en quelques dizaines de pixels :

* **Un tableau de textures** (`texture_2d_array`), dont chaque cellule porte le petit niveau d'une photo : un
  seul objet pour la carte, une seule liaison. Pas de `binding_array`, absent de trop de machines.
* **Les poses en coordonnées du document**, gardées sur la carte, et la vue en un seul uniforme : tant que
  le document ne change pas, **le fil qui dessine n'écrit rien par photo**, à 100 images ou à 8 000.
* **Un seul dessin instancié par suite consécutive** : l'ordre du modèle reste la loi, et une photo vue de
  près, hors de l'atlas, coupe la suite à sa place.
* **De près, rien ne change** : le chemin actuel est le bon pour peu de grandes photos.
* **Toujours la mémoire par étages** : l'atlas se borne à ce que la carte accorde. Une photo sans cellule passe
  par le chemin actuel : jamais de trou, seulement un coût.

**À prouver à chaque étape, sinon on s'arrête** : les mêmes pixels que le chemin actuel (l'épreuve des deux
voies) ; 8 000 images sous 10 ms sur cette Intel UHD 620 ; aucune régression de près.

## 4 quinquies. L'atelier, là où la session du 6/11 s'est arrêtée

**ATLAS-1 est écrit et n'est PAS sur GitHub.** Il vit dans l'arbre de travail de ce PC (`present/scene_gpu/atlas.rs`,
`scene_gpu.rs`, `scene_gpu/memoire.rs`), et dans `~/.cache/glucose-bancs/atlas-en-cours.patch`.

La conception du § 4 quater s'est **simplifiée en écrivant** : pas de tableau de textures, pas de nuanceur de
plus. Une page est une texture ordinaire de 2048 pixels en cellules de 64 ; le nuanceur lit déjà une fenêtre
et des bornes ; les poses consécutives d'une page sont un seul dessin. Toutes les cartes le savent, OpenGL
compris. **La pose reste écrite à chaque image** : les poses gardées sur la carte, en coordonnées du
document, restent à faire, et c'est là qu'est le reste du gain.

**Mesuré, une seule paire, à confirmer** (`outils/scenarios/vols-vue-d-ensemble.txt`, 8 000 images, Intel UHD 620) :

| | sans atlas | avec |
|---|---|---|
| une image pendant le vol, médiane | 110 ms | **65 ms** |
| au repos, médiane | 78 ms | 65 ms |
| `soumettre`, médiane | 16,4 ms | **1,7 ms** |
| `composants` | 23 ms | 19 ms |
| `textures` | 14 ms | 12 ms |
| `etages` | 12 ms | 7 ms |

**Très loin des 10 ms.** Ce qui reste : `composants` (`format!` et `HashMap<String>` par photo), `textures`
(`assurer`, `connait`, deux tours), `etages`, `cull`. Et **`soumettre` a des pics à 71-86 ms au p99**, à expliquer.

**Il reste à faire, dans l'ordre** :

1. Confirmer l'avant-après (`GLUCOSE_SANS_ATLAS=1` coupe l'atlas) : plusieurs paires.
2. **3 épreuves tombent**, toutes sur la comptabilité (une petite texture compte désormais une cellule, 16 Ko) :
   `test_la_carte_compte_ce_qu_occupent_ses_photos_a_l_ecran`, `test_memoire_2_rendre_garde_l_ecran_…`,
   `test_vram_hors_de_l_ecran_…`. Décider si la comptabilité est juste, puis les adapter, sans les affaiblir.
3. `Atlas::pages()` est morte (clippy refuse) : lui donner son épreuve (une page vide se rend entière).
   Une épreuve qui compte les dessins ; le `GLUCOSE_SANS_ATLAS` et le mélange atlas/grandes photos dans
   l'ordre du modèle. Sabotages. `scene_gpu.rs` est à 598 lignes : le cliquet est à 600, extraire.
4. Une page partiellement vide occupe 16 Mo mais compte moins : à dire au budget de la carte.
5. **Les poses gardées sur la carte** : plus d'écriture par photo tant que le document ne change pas.
6. Reprendre le § 4 : un vol décode-t-il à ses échelles intermédiaires ? Les aperçus à la deuxième ouverture ?

## 5. Les options

La piste qui ferme le sujet : **quand seule la vue change, la carte graphique n'a besoin que d'une
transformation**. Les poses des photos restent dans sa mémoire, et le fil qui dessine ne fait presque
rien, quel que soit le nombre de nœuds.

| | quoi | ce que ça vise | coût |
|---|---|---|---|
| **A** | Des entiers au lieu de chaînes par image | de l'ordre de 10 à 30 % du fil qui dessine (à mesurer) ; ne change pas la forme du coût | 1 session |
| **B** | **Les poses gardées sur la carte, la vue en un seul uniforme** | focus, zoom et déplacement à coût presque constant ; le niveau d'une image ne se revoit que quand il change | 2 à 4 sessions |
| **C** | Un vol ne décode que pour sa destination | la charge pendant un focus, si § 4 le confirme | 1 session |
| **D** | Décoder à la taille montrée | l'ouverture et la vue d'ensemble, surtout sur de vraies photos | 1 à 2 sessions |
| **E** | **L'adaptation qu'il décrit** : placer le travail selon ce qui est libre, la chaleur, la batterie | sa demande entière ; aujourd'hui seuls l'arbitre des cartes et la finesse existent | à concevoir après B : sans B, le CPU n'a rien à céder |

**Recommandation : C d'abord s'il est confirmé, puis B ; A seulement si B le laisse visible.** A seul
serait une rustine : il rétrécit le coût par nœud sans le faire disparaître.
