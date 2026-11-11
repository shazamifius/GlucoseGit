# 15 — Le sens des textes, par `static-similarity-mrl-multilingual-v1`

> Sa parole du 11/11/2026 : *« une intelligence artificielle semi from scratch, ultra optimisée et
> minimale, capable de comprendre une seule et unique chose : est-ce que ce texte a un rapport avec
> ce texte-là, et à combien de % »*. Cette note dit le modèle embarqué pour cela — **une donnée, pas
> une caisse** : le noyau le lit sans aucune dépendance —, et pourquoi celui-là.
>
> **Date** : 2026-11-11 · **Portée** : `crates/glucose-core/donnees/sens-multilingue-128.bin`,
> `crates/glucose-core/src/sens*` · **Fiche** : 106.

---

## Ce qu'il faut

* Dire **à combien deux textes se ressemblent**, en français d'abord, et dans les langues de ses
  utilisateurs ;
* **sur toutes les machines** (*« aucune machine exclue »*) : le téléphone, le vieux portable, sans
  carte graphique, sans réseau ;
* dans le noyau, qui n'a **aucune dépendance** ;
* assez vite pour relire un document de milliers de textes à chaque texte écrit.

## Pourquoi celui-là

Un **modèle statique** : une table qui donne à chaque morceau de mot une position dans un espace du
sens ; le sens d'un texte est la moyenne de ses morceaux. Aucun réseau à faire tourner : une lecture
de table et une moyenne — des microsecondes par texte, sur n'importe quel processeur.

* **Entraîné pour la ressemblance de phrases**, dans une cinquantaine de langues dont le français
  (sentence-transformers, janvier 2025 : de 100 à 400 fois plus rapide sur processeur que
  `multilingual-e5-small`, pour au moins 85 % de sa justesse) ;
* **entraîné « en poupées russes »** (Matryoshka) : ses 128 premières dimensions portent l'essentiel.
  Glucose les garde, en octets signés — **15 Mo** au lieu de 434 ; mesuré sur la carte de Wikipédia,
  aussi juste que le modèle entier (fiche 106) ;
* **son découpeur est WordPiece** (celui de BERT multilingue) : quelques centaines de lignes, réécrites
  dans le noyau à l'identique de la référence de Hugging Face, texte piège par texte piège ;
* **licence Apache 2.0** : son texte vit à côté du fichier
  (`donnees/LICENSE-static-similarity-mrl-multilingual-v1.txt`), sa provenance et ses empreintes dans
  `donnees/LISEZ-MOI.md`.

## Pourquoi pas autre chose

| | ce que ça donne | pourquoi non, ou pas encore |
|---|---|---|
| les sacs de mots d'avant | rien à embarquer | moins justes sur la carte (0,63 contre 0,69 pour « même thème », fiche 106) |
| `multilingual-e5-small`, MiniLM | des transformeurs, plus justes | un réseau à faire tourner : une caisse d'inférence (`candle`, `tract`, `ort`) ou un transformeur réécrit, plus de 100 Mo, et des secondes sur un téléphone pour un document |
| `potion-multilingual-128M` (Model2Vec, mai 2025) | statique aussi, 101 langues, distillé de BGE-M3, le plus juste des statiques multilingues | **pas encore mesuré** sur la carte ; environ quatre fois plus lourd (un vocabulaire de quelque 250 000 morceaux), et un découpeur Unigram à réécrire. Le premier candidat pour une IA « portable » plus forte — ses trois IA selon la machine (VISION § 8 bis) |
| télécharger le modèle au premier lancement | un programme plus léger | une machine hors ligne, ou derrière le pare-feu d'une école, en serait privée |

## Ce que ça engage

* **15 Mo dans le programme**, sur toutes les plateformes, et **15 Mo dans l'histoire du dépôt** à
  chaque version du fichier : on ne le change qu'avec une mesure qui le justifie.
* **Les trois IA selon la machine** (sa parole du 11/11) : toutes répondront à la même question —
  à combien deux textes se ressemblent —, pour que les couleurs restent comparables d'une machine à
  l'autre. Celle-ci est la plus petite.

## Sources

* Le modèle : <https://huggingface.co/sentence-transformers/static-similarity-mrl-multilingual-v1>
* *Train 400x faster Static Embedding Models with Sentence Transformers* (Hugging Face, 15/01/2025) :
  <https://huggingface.co/blog/static-embeddings>
* Model2Vec et `potion-multilingual-128M` : <https://github.com/MinishLab/model2vec>,
  <https://huggingface.co/minishlab/potion-multilingual-128m>
* Arora, Liang et Ma (2017), *A Simple but Tough-to-Beat Baseline for Sentence Embeddings* : la
  moyenne des mots, base solide du sens d'une phrase.
