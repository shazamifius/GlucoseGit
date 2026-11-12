# 109 — Les flèches de l'humain jamais jugées, le zoom qui ne refait plus tout

> Session du 12/11/2026, de 11 h 26 à 12 h 50. La passation (fiche 108) laissait PARENTE-4 non
> commité, une épreuve qui tombait, et la suite : la carte reposée, les documents, le journal des
> rectifications, le zoom. Envoyé : `1c3c754`, `365c9a8` (les flèches), son Glucose installé à
> `365c9a8` ; `0e90263` (le zoom) et les documents.

## 1. PARENTE-4 fini (le détail : fiche 108 § 5)

L'épreuve de l'agent rouvrait son document pendant que la première fenêtre le tenait — un seul
scribe par fichier : elle la ferme d'abord. 3 152 épreuves, puis **16 sabotages sur 16**
(`outils/sabotages/provenance-108.json`) après trois épreuves aveugles écrites. Envoyé, et son
Glucose installé mis à jour (`nix profile upgrade --refresh GlucoseGit`, 11 minutes).

## 2. Le zoom : deux rendus complets par cran (REPOS-4)

**Mesuré d'abord** (la mesure d'hier, image par image) : à chaque cran de molette, de loin sur sa
carte, **deux images refaisaient presque toutes les textures** (186 à 304 sur 385) — celle du
mouvement au palier dyadique, puis celle du repos, exactes. Le diagnostic d'hier (« chaque cran
refait les textures ») était juste sur le fond, faux sur le compte.

**REPOS-4** (`renderer/composants/passage.rs`) : tant que la vue reste à **une demi-octave** de
l'échelle du dernier repos — la garantie même du palier, le plus proche à racine de deux près —,
chaque texture se compose **dans la vue du repos** (même échelle, même phase, donc même clé), et la
carte graphique la pose par la transformation exacte du repos à l'écran (`écran = k × point + d`).
La translation entière de la fiche 100 en est le cas exact ; une translation fractionnaire ne
refait plus rien non plus — l'épreuve de la fiche 100 qui l'exigeait est réécrite (MÉTHODE § 4,
principe 7 : au palier comme au repos la carte interpole à une position fractionnaire, mais la
texture du repos est exacte et déjà rendue). `Regard::translation_entiere` devient
`Regard::repos`.

**CASCADE-4** (`present/scene_gpu/cascade.rs`) : le coût prévu d'une texture est une droite — un
prix fixe plus un prix par pixel —, par moindres carrés cumulés (Welford). Par pixel seulement,
la prévision gonflait les grandes textures après beaucoup de petites. **Mesuré sans gain visible
sur le zoom** : la cascade y bute de toute façon sur son budget. Gardée parce que l'ancienne loi
est biaisée par construction ; le commentaire qui annonçait un gain a été corrigé.

**Mesuré en paires entrelacées** (son Glucose installé `365c9a8` contre la version neuve, deux
fois, `~/.cache/glucose-bancs/lag-107/m109-*`, sway sans écran) :

| zoom de loin | installé | neuf |
|---|---|---|
| une image du mouvement | 19-21 ms, 240 à 304 textures | **4-5 ms, une texture** |
| la latence de navigation (médiane) | 19-20 ms | **6,6-6,7 ms** |
| l'image médiane du segment | 19,0-19,5 ms | 11,8-14,4 ms |
| p90 | 20,4-21,0 ms | 20,4-21,6 ms (inchangé) |

**Ce qui reste** : l'image du repos qui suit chaque cran (≈ 20 ms). Sa cause, mesurée : la cascade
a le droit de remplir **la période de l'écran** (`Cadence::tranche_de_fond` : le plus grand de la
période et du plancher de 10 ms — 12 ms à 60 Hz), puis l'envoi à la carte graphique qui suit
(`soumettre`, 1,7 à 4 ms, qui grandit avec le nombre de textures envoyées) déborde la marge fixe de
0,3 ms (`cadence::MARGE`) : un battement d'écran manqué, la netteté d'un cran arrive une image
plus tard. Deux pistes : prévoir la queue de l'image (ce qui suit la cascade) au lieu d'une marge
fixe ; ou ne donner que le plancher au second tour de la cascade (un flou attend, un trou non).

**Prouvé** : 3 157 épreuves, clippy et fmt à zéro ; sabotages `outils/sabotages/zoom-109.json`
(15 sur 15, § 5) ; `gel-100.json` mis à jour (ses deux gardes ont changé de place). **Pas regardé** :
une image capturée pendant un zoom (le mouvement ne se capture pas au pilote) ; à l'arrêt, le
chemin est celui d'avant.

## 3. Le journal des rectifications — conçu, pas écrit (RECTIF-1)

La première marche de la page « Le cercle de l'IA », qu'il a acceptée (VISION § 8 bis). **Rien
n'est écrit dans le code.** La conception, à remettre en question :

* **Une rectification est un écrit** (NOTE-1) que l'IA de Glucose attache au geste de l'humain qui
  la contredit : un fait, un seul endroit (l'histoire) ; la Time Machine et `glucose_histoire` le
  montrent déjà. Son auteur : un agent déclaré « l'IA de Glucose » (`Auteur::d_un_agent`), montré
  sans le préfixe « agent » (deux endroits l'écrivent : `interactions/temps.rs`,
  `agents/executer/temps.rs`).
* **NOTE gagne deux champs en queue** : sa sorte (écrit libre, rectification) et ses sujets (les
  nœuds dont il parle). NOTE n'a jamais été publié — son format peut s'allonger ; le lecteur prend
  les champs absents par défaut (`Reader::remaining`). Construit à trois endroits :
  `persist/histoire.rs` (`lire_note`), `persist/ecriture.rs` (`noter`), `tests/histoire_suite.rs`.
* **Les gestes qui comptent** (sa page) : il colore un texte que l'IA avait rangé dans une
  famille ; il retouche une flèche d'agent que l'IA jugeait d'habitude ; il suit (`Ctrl`+clic)
  une telle flèche. « D'habitude » = sûreté sous 0,5 : la sûreté est une probabilité a posteriori
  (1 − lfdr d'Efron), 0,5 la frontière de Bayes à coûts égaux — pas un nombre choisi.
* **Où les voir** : à la sortie, dans `GlucoseApp::ecrire_les_gestes` (`persist/disque.rs`) — chaque
  `Ecrit` y devient un geste au rang que le scribe connaît ; le journal dit, par `Ecrit`, les nœuds
  que ce geste de l'humain a repris aux agents (`suivre_les_mains` le rend) ; l'avis de l'IA est
  dans `renderer.sens_des_textes` (`avis_du_tableau`, `surete_d_un_lien`). Le suivi d'une flèche :
  `interactions/suivre.rs`, puis demander à la parenté de se refaire (son `vu` ne regarde que la
  version du document).
* **Les faits seulement** (Tucker et al., ICML 2024 : une retouche est une préférence faible, pas
  une vérité) : ce que l'IA pensait (sa famille, deux de ses voisins, sa sûreté), ce que l'humain a
  fait.
* **D1, se souvenir sur ce document** : une flèche suivie ou retouchée n'est plus jugée —
  retouchée, MAIN-1 le fait déjà ; suivie, la rectification la retire de la provenance, et la
  relecture de l'histoire (`histoire/ouvrir.rs`) relit les rectifications. Le suivi n'est pas
  annulable (ce n'est pas un geste du document) : le dire.
* **Pas encore** : le « pourquoi » au mot près (§ 4), les flèches retirées, l'apprentissage.

## 4. La recherche (consultée le 12/11/2026)

* Tucker et al., *Coactive Learning for Large Language Models using Implicit User Feedback*
  (ICML 2024, <https://cs.cornell.edu/~tj/publications/tucker_etal_24a.pdf>) : quand l'humain
  retouche ce qu'un modèle propose, sa retouche dit « mieux que la proposition », pas « la bonne
  réponse ».
* Opitz, Möller, Michail, Padó et Clematide, *Interpretable Text Embeddings and Text Similarity
  Explanation: A Survey* (EMNLP 2025, <https://arxiv.org/abs/2502.14862>).
* Model2Vec, la moyenne pondérée des vecteurs de jetons
  (<https://eryawww.github.io/blog/model2vec-explained/2026/05/>) ; sa limite : « not good food »
  ressemble à « good food » (<https://llamaindex.ai/blog/exploring-static-embedding-retrieval>).
* **Mon raisonnement, à vérifier** : l'IA du sens de Glucose est un modèle statique — un texte est
  la moyenne des vecteurs de ses jetons, donc sa ressemblance au centre d'une famille est une
  **somme exacte** de la part de chaque mot ; les mots qui pèsent le plus sont le « pourquoi »
  (B3 : l'IA qui explique), sans heuristique.

## 5. Les sabotages du zoom

`outils/sabotages/zoom-109.json` : quinze — la demi-octave (jamais, toujours), le passage, la
pose, l'écran lu dans la vue du repos, la photo en chemin, le titre (échelle, rapport), l'échelle
de rendu, la droite du coût (sa pente, par pixel seulement, Welford), et les deux gardes de la
fiche 100 déplacées. **15 sur 15 tombent.**

## 6. Les leçons

* **Mesurer le compte avant la cause** : « chaque cran refait les textures » était vrai, mais il y
  en avait deux par cran, et l'image du mouvement en était une — la moitié du remède se voyait
  dans le compte image par image, pas dans la médiane.
* **Lire le budget avant de régler ce qui le dépense** : j'ai écrit qu'une image de zoom rendait
  12 à 16 ms « au lieu des 5 que son budget lui laissait » — le budget était la période de
  l'écran (12 ms). Le commentaire mentait ; la mesure l'a dit.
* **Un champ renommé vit aussi dans les exemples, les épreuves d'intégration et les sabotages** :
  `cargo test --workspace` compile les exemples (sept bancs) ; deux sabotages anciens visaient le
  code déplacé. Chercher dans tout le dépôt, `outils/` compris.
* **Une durée annoncée se mesure** : seize sabotages en six minutes, pas quarante-cinq ; quinze du
  rendu en plus de dix (chacun recompile la bibliothèque du bureau sans optimisation).
* **L'heure se lit** : j'ai écrit « 11 h 55 » à 11 h 44.
