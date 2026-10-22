# 80 — Ce que l'humain désigne : l'agent relit, et range ce qu'il apprend

> Session du 22/10/2026, menée depuis Protéo. Sa parole, ce jour-là : *« Protéo DOIT utiliser
> Glucose comme un outil, et Glucose doit avoir des critiques et des ajouts — Glucose n'a pas assez
> d'utilisateurs, et mon objectif est de remplacer la feuille blanche, donc il nous faut des
> personnes avec des projets ; Protéo nous apporte une expérience utilisateur »* ; *« pour tout ce
> qu'il apprend et qu'il fait avec succès, il en laisse une trace quelque part (Glucose) »* ; et
> *« voir comment Glucose a besoin d'être modifié pour que Protéo l'utilise comme un outil, et que
> l'interaction avec l'humain se passe parfaitement »*. Les objectifs 2 et 3 de la
> [VISION](../VISION.md).

## 1. Ce qui manquait

La porte des agents (fiche 75) n'allait que dans un sens : un agent posait, il ne savait **ni ce
que l'utilisateur montrait du doigt, ni ce qu'il avait corrigé**, sauf à relire tout le tableau
et à le comparer lui-même. Protéo, de fait, ne relisait jamais rien : sa toile posait, rien ne
revenait. Et un agent ne pouvait pas **ouvrir un onglet à lui** : tout ce qu'il rangeait tombait
sur le tableau que l'utilisateur regarde.

## 2. La recherche (consultée le 22/10/2026)

* *Clarify the User or Verify the World?* (arXiv 2609.32255, 26/09/2026) : le doute d'un agent a
  deux sources — l'humain (ce qu'il veut) et le monde (ce qui est) ; à chacune sa réponse.
* *Conversations in Space* (CanvasConvo, arXiv 2605.15848) : 24 personnes, cinq jours de travail
  réel — le canevas **complète** le fil de discussion, il ne le remplace pas. Glucose et le chat
  de Protéo vont ensemble, synchronisés.
* *Filesystem-Based Memory for LLM Agents* (arXiv 2607.26637, 29/07/2026) : une mémoire organisée
  divise par deux le coût de lecture quand la matière est grande, mais n'améliore pas les
  réponses à elle seule, et un agent faible la laisse s'éroder. **La valeur propre de Glucose est
  du côté de l'humain** : voir le travail se faire, montrer du doigt, corriger, annuler.
* tldraw, « Agents can't point » (11/08/2026) : l'humain montre « ça » au lieu de le décrire.
* La mémoire partagée entre agents (arXiv 2606.24535 ; OWASP 2026, ASI06 ; MELD, arXiv
  2608.16357) : provenance par nœud, contradictions montrées et non tranchées en silence — la
  condition, dès aujourd'hui, du Glucose que tous les Protéo rempliraient.

## 3. La conception

* **`selection`** dans `glucose_lire` : les nœuds que l'utilisateur a sélectionnés sur le tableau
  qu'il regarde (rien sur un autre) — lue, jamais touchée (AGENT-2). Quand il écrit « ça » à
  l'agent, c'est eux.
* **`empreinte`** sur chaque nœud qui a un texte : les seize premiers chiffres de son sha256 (le
  sha256 du noyau, aucune dépendance), calculée à la lecture et jamais rangée — la porte reste
  sans état, le fichier ne change pas. L'agent la garde ; à la lecture suivante, il ne relit que
  les nœuds dont l'empreinte a changé.
* **`glucose_ouvrir_tableau`** : l'onglet qui porte ce nom, créé s'il n'existe pas, en un geste
  annulable — l'onglet que l'utilisateur regarde ne change jamais, et le même nom rend toujours
  le même onglet (l'esprit d'AGENT-4). Un ajout **né de l'usage de Protéo** : son carnet n'avait
  nulle part où vivre.
* **La main d'épreuve** du Glucose sans fenêtre (`bench/main_d_epreuve.rs`) : des gestes de
  l'utilisateur (sélectionner, récrire, poser un post-it, retirer) lus dans un fichier et joués
  par les chemins de la main — pour éprouver ce qu'un agent voit de ce que l'humain fait, sans
  personne devant l'écran. Elle n'existe que pour les bancs.

**Côté Protéo** (son dépôt, conception § 4.20) : à chaque message, il relit le tableau, compare
les empreintes à ce qu'il avait vu (gardé par tableau, sur le disque), et dit au modèle ce que
l'utilisateur **désigne**, a **récrit**, a **ajouté** dans ses bulles ou relié à ses nœuds, en a
**retiré** — comme un appel à `glucose_lire` fait pour lui. Et il tient un **carnet**, l'onglet
« Ce que Protéo a appris » : chaque réussite vérifiée (un calcul voté, une réponse de
l'utilisateur, ce qu'il a tranché après une recherche) y devient un nœud écrit par le programme,
dont la première phrase dit quand s'en servir ; s'il corrige une leçon, Protéo le voit.

## 4. Prouver

* **Épreuves** : la sélection (annotations et photos, le bon tableau, lue sans être touchée) ;
  l'empreinte (la même à tout niveau de lecture, changée par une récriture, gardée par ce qui ne
  change pas, le vecteur du vide) ; l'onglet (le même nom, le même onglet ; un geste qu'un
  `Ctrl+Z` défait ; refusé pendant un geste de la main ; un nom vide refusé ; on y pose avec
  `tableau` sans changer de regard).
* **Sabotages** : `agents-designer.json` (6 sur 6), `agents-tableau.json` (3 sur 3).
* **La suite entière** : fmt, clippy `-D warnings`, toutes les épreuves (cliquets compris).
* **L'essai réel** (le vrai Protéo, le vrai `qwen3.5:4b`, le vrai web, ce Glucose sans fenêtre, moi
  dans le rôle de l'humain par la main d'épreuve) : « Explique-moi en deux phrases ce qu'est le
  glucose » → la toile posée ; je sélectionne sa réponse et pose un post-it « Donne aussi sa
  formule chimique » dans sa bulle Demande ; puis « Développe ça, s'il te plaît » → le tour reçoit
  la réponse désignée et le post-it en entier ; Protéo cherche en y incluant « formule chimique »
  (14 citations vérifiées) et développe sa réponse avec C₆H₁₂O₆.

**Pas encore vu** : à son écran — sa version installée (beta 5) n'a pas la porte.

## 5. Ce que l'usage de Protéo dit de Glucose (ses critiques, la liste qu'il voulait)

* **Un agent n'avait pas d'onglet à lui** → `glucose_ouvrir_tableau` (fait).
* **Un agent ne voyait ni la sélection ni les corrections** → `selection`, `empreinte` (fait).
* **L'auteur d'un nœud n'est pas dans le fichier** : Protéo reconnaît les siens à leur préfixe
  (`proteo-`), ce qui suffit pour un agent seul, pas pour plusieurs — ni pour le Glucose que tous
  les Protéo rempliraient. Il faudra la provenance par nœud (qui, depuis quelle preuve, quand).
* **Un post-it n'appartient à une bulle que par sa position** (MEMB-1) : l'humain qui veut ajouter
  à la bulle d'un agent doit viser dedans. Naturel à la souris ; à confirmer à son écran.
* **Le nom de la porte d'essai** (`GLUCOSE_PORTE`) veut le chemin complet du tuyau sous Windows
  (`\\.\pipe\…`) ; l'exemple sans fenêtre disait seulement « la porte ne s'ouvre pas ».

## 6. Ce qui reste

1. **Une version publiée** qui ait la porte : rien de ceci ne se voit chez lui avant (son geste).
2. Les **commentaires ancrés**, parallèles au canevas (fiche 79 § 5) ; la **provenance par nœud** ;
   les **membranes repliables** ; le **statut et la preuve** écrits par le programme.
