# 142 — Les jetons : relire la conversation coûtait 78 %

> Le 24/11/2026, de 12 h à 12 h 20, compte b (le relais). Sa consigne pointe d'une flèche sur
> `relais-q-h24-quotas`, où il répond : *« ces URGENT, la perte de token es ma prioriter actuel !!! vraiment
> je ne shouaite pas switch sur sonnet »*, *« relire la converssation ces vraiment a ne jamais faire »*, et
> *« ce serai aussi interressant que claude arret de parler dans le vide […] moi je ne vois QUE les
> interaction avec glucose »*.

## 1. Mesuré, pas estimé

Les journaux des conversations (`~/.claude/projects/-home-shaza-Projets-GlucoseGit/*.jsonl`) donnent, pour
chaque requête au modèle, ses jetons : lus du cache (la conversation relue), écrits au cache (ce qui
s'ajoute), écrits par le modèle (pensée comprise). Depuis le 23/11 : **3 191 requêtes**, 21 conversations,
Opus 5.5 partout.

Pondérées comme l'API compte leur prix (lire le cache 0,1 ; l'écrire pour une heure 2 ; écrire 5) :

| poste | part |
|---|---|
| **relire la conversation** (lecture du cache) | **77,7 %** |
| écrire au cache ce qui s'ajoute | 11,1 % |
| ce que le modèle écrit (pensée, outils, texte) | 11,2 % |

**Le contexte moyen d'une requête : 318 000 jetons, 784 000 au plus.** La fenêtre d'Opus est d'un
million : le seuil de 90 % (sa règle du 23/11) laissait une conversation grandir jusqu'à 900 000 jetons,
chaque réponse relisant tout. Le texte écrit dans le terminal n'est qu'une petite part des 11 % que le modèle écrit (sa pensée en
est l'essentiel) : le taire est juste, mais ce n'était pas là que fuyaient les jetons.

**Le démarrage d'une conversation lisait ~110 000 jetons** (de 45 000, ce que Claude Code charge seul, à
154 000 après 25 requêtes) : ses règles déjà chargées relues, toutes les mémoires en entier (7 900 jetons),
VISION, ÉTAT, SUITE, MÉTHODE en entier (~40 000). Coût d'un démarrage : ~612 000 jetons pondérés.

**Doute honnête** : la façon dont un abonnement Pro compte ces jetons n'est pas publiée ; la pondération est
celle des prix de l'API. La preuve viendra de la jauge : les heures de travail par fenêtre de 5 h, ~1 h 30
avant (fiche 141), à relever après.

## 2. Le seuil : simulé sur les vraies sessions

Rejouer les 15 sessions longues avec une conversation neuve à chaque seuil (le démarrage payé à chaque
fois) — coût moyen par requête, en milliers de jetons pondérés :

| seuil | 150 k | 200 k | 250 k | 300 k | 400 k | 600 k | 900 k |
|---|---|---|---|---|---|---|---|
| sans réapprendre | 637 | 47,6 | 38,1 | **36,8** | 37,9 | 41,2 | 46,4 |
| 150 k à réapprendre | 794 | 679 | 76,5 | 46,3 | **41,4** | 42,5 | 46,6 |

Sous 200 k, le démarrage (154 k) dépasse déjà le seuil : une boucle sans fin. Le creux est large, de 250 k à
400 k. **Choisi : 300 000 jetons** — le meilleur sans réapprendre, et à 12 % du meilleur si une session
neuve doit relire 150 000 jetons de code. Une fenêtre plus petite (200 000) garde 90 %.

## 3. Fait

* `loi.SEUIL_JETONS = 300_000` ; `contexte_plein` le calcule de la taille de la fenêtre, que la barre
  d'état donne (`context_window_size`, écrite dans `contexte.json`).
* **Le démarrage maigrit** : plus de relecture de ce que Claude Code charge (ses règles, l'index des
  mémoires) ; de LISEZ-MOI, d'une mémoire, de VISION, ÉTAT, SUITE ou MÉTHODE, la seule section qu'un travail
  demande. Le `CLAUDE.md` du dépôt le dit pour les sessions du relais ; une session à lui lit tout.
* **Les règles du relais** (`REGLES`, ajoutées à son invite) : aucun compte rendu dans le terminal, lire au
  plus juste.
* **Ce que sa consigne désigne** (sa note `text-g420-2321` : que le démarrage « regarde le block
  important ») : les flèches que **sa main** tire depuis `relais-consigne`, puis depuis leurs cibles, sont
  données à lire au démarrage (`loi.ses_fleches`, lues de toute l'histoire de Glucose). Une flèche d'IA ne
  compte pas. Aujourd'hui : `relais-q-h24-quotas`. Pour son bloc URGENT, une flèche de sa consigne vers lui.
* Son nœud `relais-texte-depart` portait l'ancien texte, signé de sa main par un simple déplacement (le
  23/11 à 10 h 29, « déplace des nœuds » et « change le texte ») : récrit par Claude au texte neuf, il ne
  vaut plus signé et le texte neuf part.
* Épreuves : 75 sur 75 ; `outils/sabotages/relais-142.json`, 8 sabotages sur 8 tombent.
* **La mise à jour de Claude** qu'il voyait dans VS Code : 2.1.331 (sortie le 23/11 à 22 h 49) est la
  dernière du marché, déjà installée ; le relais la lance depuis 12 h. VS Code ne l'affiche qu'après le
  rechargement de sa fenêtre ; le relais n'en dépend pas (`claude_binaire` prend la plus récente à chaque
  lancement).

## 4. Ce qui reste

1. **Mesurer le gain à la jauge** : le pourcentage de la fenêtre de 5 h par heure de travail, avant et après.
   Premier relevé : la fenêtre du compte b ouverte à 12 h, **2 % à 12 h 00, 19 % à 12 h 25** — cette
   conversation, partie sous l'ancien démarrage, à 260 000 jetons de contexte à 12 h 25. La suivante sera la
   première sous les règles neuves : relever la jauge à son début et à sa fin.
2. ~~Le MCP plus court~~ : **fait à 12 h 24** — une lecture par `ids` ou `cherche` ne redit plus le
   document (deux fois), la partie visible, les onglets ni les domaines ; le sommaire, qui oriente, les
   garde (`executer/tests/courte.rs`, `mcp-court-142.json` 3 sur 3 ; 3 586 épreuves).
3. Ne relire à chaque démarrage que ce qui a changé de son bloc important (la proposition 6).

## 5. Le flou et les équerres retirés (12 h 13 - 12 h 18)

Ses paroles du 24/11 (`text-2231`, `text-g420-2313`, `relais-q-beta6`) : *« je reviens sur ma décision, c'est
définitivement une très mauvaise idée le flou […] terriblement nul, accumulation de problèmes »*, *« ne pas
toucher au système de sélection actuel de Glucose beta 5, c'est celui qui me plaît le plus »*, et pas de
beta.6.

**Défaits, du plus récent au plus ancien, le code seul** (`git show <c> -- . ':!docs' | git apply -R
--3way`, la documentation gardée comme journal) : MELER-1 (`7007ea3 4c1f0b3 6487cac 8dd48ee 0499e7b b5268f7
b2a8a75`), le flou (`92e0816 e25ea44 6348f20 ffce09a`), les équerres (`8e018fc 927c809`). Aucun conflit :
rien d'autre, depuis la beta.5, ne s'appuyait sur eux. `glucose-core/src/flou.rs` était né avec Q12 et ne
servait qu'à lui : il part aussi. Les corrections d'après la beta.5 restent (PICK-3, EDIT-PRISE, la flèche
qui se choisit là où elle se dessine…).

Preuves : `clippy -D warnings` propre, `fmt` propre, **3 585 épreuves, 0 échec** (cliquets compris). Les
sabotages `flou-137`, `equerres-137` et `meler-140` partent avec leur code.

**Ce qui se perd, à dire** : le travail de MELER-1 (le flou au processeur de 65,6 à 24,6 ms) n'a plus d'objet.
Ses leçons restent à la fiche 140 § 5.
4. **La parenté tue quand elle est vide** (12 h 35) : un texte seul et sans lien ne dit plus
   `"parente":{"famille":null,"surete":0}` — dans son document, presque chaque nœud du sommaire le disait.
   Gardé par `parente_3` (sabotage 4 de `mcp-court-142.json`). **Une épreuve manque** : une flèche entre deux
   textes de musique sûrs lit une sûreté de 0 dans l'épreuve, et aucune flèche de son document ne dit de
   parenté — la sûreté des flèches que promet la description du MCP est peut-être morte. À creuser.
