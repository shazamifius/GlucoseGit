# 79 — Lire par niveaux : un agent ne reprend que ce dont il a besoin

> Session du 21/10/2026, menée depuis Protéo. Sa parole, ce jour-là : *« je propose aussi que
> Glucose soit un simplificateur de tokens pour que plusieurs agents travaillent en parallèle et
> qu'on n'atteigne jamais la limite de tokens ; l'idée de Glucose, c'est que tout soit lisible
> quelque part, tout soit écrit, mais au propre, et que l'IA ne récupère QUE les trucs dont elle a
> besoin »*. L'objectif 2 de la [VISION](../VISION.md), et la première brique de la R&D écrite côté
> Protéo (`docspers/memoire-sans-fin.md` de son dépôt, § 5).

## 1. Ce qui manquait

`glucose_lire` rendait **le texte entier de chaque nœud** du tableau. Pour un petit modèle de
langage dont la fenêtre se remplit, c'est l'inverse d'une mémoire. Le constat qui l'a montré, côté
Protéo : le matin du 21/10, sa mission « recréer MeshForge en addon Maya » a lu un manuel de
1 037 lignes d'un bloc ; sa fenêtre (28 160 jetons ce jour-là) était pleine avant d'écrire, et le
petit modèle a inventé 22 commandes Maya sur 26.

## 2. La recherche

* **Le tableau noir** (*blackboard*) : des agents qui lisent et écrivent un tableau commun font
  mieux avec moins de jetons (arXiv 2507.01701, juillet 2025 ; Google, arXiv 2510.01285,
  septembre 2025) — c'est ce que Glucose est déjà.
* **Le système de recherche d'Anthropic** (13/06/2025) : les sous-agents rangent leur travail
  dehors et ne repassent que des références légères.
* **Le masquage des observations** (JetBrains, arXiv 2508.21433, NeurIPS 2025) : ne pas porter
  le vieux détail est aussi bon que le résumer, pour moitié moins cher.
* **A-Mem** (NeurIPS 2025) : des notes liées, chacune avec sa description courte.

## 3. La conception

Trois façons de lire, dans `agents/lecture.rs` (pur, sans le tableau) :

* **le sommaire, par défaut** : chaque nœud dit ce qu'il est (identifiant, sorte, boîte,
  membrane, bouts d'une flèche, adresse d'une photo), **sa première phrase** (`debut`) et sa
  **taille** — pas son texte ;
* **`ids`** : le texte entier des nœuds que l'agent choisit ;
* **`cherche`** : la **phrase** de chaque nœud qui contient les mots cherchés (`passage`),
  sans tenir compte des majuscules.

`dans` garde une membrane ; `niveau: "complet"` rend tous les textes, comme avant.

**Aucune longueur n'est fixée** : pas « les 80 premiers caractères », mais la phrase — coupée
après `.`, `!` ou `?` suivis d'un blanc, et à chaque ligne. L'unité est celle du texte lui-même
(la charte : une constante qui peut disparaître disparaît).

Protéo ne lisait du tableau que les positions (`toile.rs`, `rangee_sous`) : rien ne change pour
lui. Les instructions données aux agents disent la manière : *commence par le sommaire, lis en
entier seulement ce dont tu as besoin, ou cherche une phrase*.

## 4. Prouver

* `lecture/tests.rs` : les phrases (un point sans blanc ne coupe pas : « v2.0 »), le passage, le
  choix des nœuds (sommaire, complet, ids, cherche, dans).
* `executer/tests.rs`, `lire_par_niveaux_ne_rend_que_ce_qu_on_demande` : par l'outil lui-même,
  sur un manuel de 1 037 lignes — le sommaire n'en porte que la première phrase et la taille, et
  il est **plus de vingt fois plus court** que la lecture complète ; `ids` rend un seul texte
  entier ; `cherche` rend la seule phrase qui contient « polydissolve ».
* **Quatre sabotages, quatre chutes** (`outils/saboter.py`) : le sommaire qui porte le texte ;
  `ids` qui ne choisit rien ; `cherche` qui rend n'importe quelle phrase ; des phrases qui ne se
  coupent qu'aux lignes.
* `fmt`, `clippy -D warnings`, toute la suite (cliquets compris) : propres.

**Pas encore vu** : à son écran, par un vrai agent, sur un vrai tableau.

## 5. Ce qui reste (côté Glucose, de la même R&D)

1. un **résumé par nœud**, tenu par celui qui écrit, distinct du texte (RAPTOR, A-Mem) ;
2. des **membranes repliables = des sous-tâches repliées** (AgentFold) : fermée, elle montre son
   bilan ;
3. un **statut et une preuve** sur un nœud, écrits par le programme, jamais par le modèle ;
4. des **territoires d'agents** et l'attribution visible (qui a écrit quoi) ;
5. des **commentaires ancrés**, parallèles au canevas (tldraw, « Agents can't point »,
   11/08/2026) : il pointe, l'agent reçoit l'ancre ;
6. les **relations typées** parcourues par voisinage ; des **éditions incrémentales** avec
   l'historique du nœud (ACE, arXiv 2510.04618).
