# 77 — Le cerveau de Protéo : choisir un modèle sur la mesure

> Session du 18/10/2026 (la nuit), suite des fiches 75 et 76. La fiche 76 concluait : *la mécanique
> tient ; l'intelligence du qwen2.5:7b, non* — et la machine qui devait le remplacer n'avait tourné
> qu'une fois, le 01/10. Sa parole, dans son texte du 17/10 : Protéo *« va comprendre ton PC, ses
> composants, ses capacités, puis commencer à fetch internet dans l'optique de trouver THE
> intelligence artificielle généraliste faite pour ton PC ; à partir de là ce petit LLM va
> complètement dégager pour laisser place au nouveau »*. Et, ce soir : *« c'est plus à toi de tester
> […] tu seras sûrement meilleur juge que moi »*.

## 1. Pourquoi la machine avait retenu *ZDTaichu5.0-9B* (le 01/10)

Le journal de Protéo du 01/10 le dit pas à pas (`modele.choisir`, `modele.inspecter`) :

* **Le vivier** : les cent modèles en tendance sur Hugging Face et le classement BFCL. Ni Qwen3.5-4B
  ni Qwen3.5-9B n'y étaient ce jour-là ; *ZDTaichu5.0-9B* était 40ᵉ de la tendance.
* **L'ordre** : `log2(paramètres) + jours / 95` — la loi de densification (Xiao et al., 2025),
  appliquée à **chaque** modèle comme s'il était à la frontière de densité. Elle décrit le meilleur
  modèle d'une date, pas n'importe lequel : un dépôt créé en septembre valait, à taille égale,
  2^(205/95) ≈ 4,5 fois un Qwen3.5 de février. *ZDTaichu5.0-9B* est un Qwen3.5-9B (sa fiche le dit)
  auquel on a greffé un encodeur de vision ; il ne déclarait pas sa base, et passait pour des poids
  neufs de septembre.
* **Le verdict** : un seul candidat est essayé par campagne — le premier éligible. L'a priori, qui
  ne devait fixer que l'ordre d'examen, devenait donc le choix.
* **Ce qu'il est** : un modèle de vision spatiale pour la robotique, dont la fiche déclare
  l'anglais et le chinois. Pas un agent qui parle français.
* **L'échec du téléchargement** (« hf.co : no such host ») : le réseau, ce matin-là — `hf.co`
  répond aujourd'hui. Mais l'erreur n'étant pas classée « réseau bloqué », l'essai fut conclu
  « raté » et ses poids marqués essayés.

## 2. La recherche

* **Ce qui tient sur la machine** — mesuré, pas lu : son bureau garde **2,4 Go** des 8 de la RTX 5070
  (le fond d'écran animé, Brave, Discord, Steam, Glucose…) ; il reste **5,5 Go**, cache de contexte
  compris. La mémoire vive : 14,8 Go libres sur 31,4.
* **Les candidats d'octobre 2026**, relevés à la source (fiches Hugging Face, bibliothèque d'Ollama) :
  Qwen3.5 (février 2026 : 0,8, 2, 4, 9, 27 et 35 milliards ; la 3.6 et la 3.8 n'ont pas de petit
  modèle dense — [Qwen3.8-27B](https://huggingface.co/Qwen/Qwen3.8-27B) est le plus petit) ;
  [Gemma 4](https://huggingface.co/google/gemma-4-12B-it) (E2B, E4B, 12B, 26B-A4B, 31B ; appel
  d'outils natif) ; [Granite 4.2](https://huggingface.co/ibm-granite/granite-4.2-8b) (3B, 8B ;
  le français déclaré, l'appel d'outils) ; [Ministral 3](https://huggingface.co/mistralai/Ministral-3-8B-Instruct-2512)
  (8B, le français) ; LFM2.5 (Liquid AI). Les chiffres d'appel d'outils publiés sont **tous
  déclarés par leurs auteurs** ([BFCL-V4 sur llm-stats](https://llm-stats.com/benchmarks/bfcl-v4) :
  aucun vérifié) : Qwen3.5-9B 66,1, Granite 4.2-8B 52,4, Qwen3.5-4B 50,3. Le guide le plus cité pour
  8 Go ([localllm.in](https://localllm.in/blog/best-local-llms-8gb-vram-2025)) donne Qwen3.5-9B —
  sur une carte **vide**.
* **Deux pistes écartées sur pièces** : *Ternary-Bonsai-2-27B* (un Qwen3.8-27B en poids ternaires,
  5,95 Go, [sa fiche](https://huggingface.co/prism-ml/Ternary-Bonsai-2-27B-gguf)) exige une version
  modifiée de llama.cpp qu'Ollama ne charge pas ; les GGUF de `hf.co` séparent souvent le texte et la
  vision, et certaines architectures n'y sont pas prises en charge par Ollama
  ([la discussion d'unsloth](https://huggingface.co/unsloth/Qwen3.5-9B-GGUF/discussions/4)).
* **La réflexion** : *AgentFloor* ([arXiv 2605.00334](https://arxiv.org/abs/2605.00334)) — les
  petits modèles ouverts tiennent l'appel d'outils court et structuré ; ils cèdent sur le long
  horizon. Rien de publié ne dit si leur réflexion aide un agent : à mesurer.

## 3. Trois défauts qui rendaient le banc aveugle aux modèles récents

Avant de mesurer quoi que ce soit, il a fallu que la mesure soit possible.

* **RAISONNEMENT-1 — une question au format imposé ne délibère pas.** Les modèles récents
  réfléchissent par défaut (Qwen3.5, Gemma 4, Granite 4.2). Sous les 32 jetons du tri, la réflexion
  seule les épuisait : **41 réponses illisibles sur 41**, pour Qwen3.5-4B comme pour Granite 4.2-8B.
  Le client d'Ollama porte désormais la réflexion (`avec_raisonnement`) ; sous un schéma, elle est
  coupée sauf si on la demande (`think: false`, qu'un modèle sans réflexion accepte). Tout l'organe
  de recherche, le tri, la suite et le plan passent par là.
* **MODELE-1 — un tour n'a qu'un modèle, celui de son cadre.** Dans l'épreuve `agent` du banc, seule
  la boucle parlait au candidat : **le tri, la suite, le plan de la mission et tout l'organe de
  recherche parlaient au modèle en service**. Un candidat y était en partie le modèle qu'il devait
  remplacer. `Ctx::modele` le porte ; l'épreuve `au_banc_chaque_question_va_au_candidat` lit au
  journal le modèle de chaque question.
* **LANGUE-1 — un modèle qui déclare ses langues sans le français n'est pas éligible** : toutes les
  consignes de Protéo, et l'utilisateur, sont en français. Une fiche muette n'écarte pas. Avec la
  preuve à l'envers : sans la règle, *ZDTaichu* passait.

Et une épreuve neuve, **`tri`** (`banc/tri.rs`) : le tri de ses 31 demandes réelles et la suite des
10 cas de la mission de Rennes, posés exactement comme la production les pose — la source unique de
ces cas, que l'épreuve à la main du criblage (`le_tri_et_la_suite_sur_ses_demandes`) lit aussi.

## 4. L'organe de recherche : trois lois, lues au journal du 17/10

Chaque défaut a d'abord été retrouvé dans le journal de Protéo, puis une épreuve l'a reproduit sur le
code intact (elle tombait), puis la correction l'a fait passer.

* **PROGRES-1 — un tour progresse quand il fait avancer le statut de sa sous-question** (ouverte →
  répondue → recoupée). La cause des 116 pages sur l'histoire de la Bretagne : un tour comptait comme
  fécond dès qu'il citait *une* phrase, même qui ne répond pas ; chaque page en citant une, le compte
  des tours stériles revenait sans cesse à zéro, et les liens se suivaient jusqu'à « Portail :
  Pays-Bas ». J'avais d'abord pensé à la loi de la valeur marginale (Charnov, 1976 ; la quête
  d'information de Pirolli et Card) — quitter une sous-question quand le rendement de son dernier tour
  tombe sous le rendement moyen de la recherche. **Elle n'aurait rien arrêté** : le rendement en
  citations ne baissait pas, c'est leur pertinence qui s'effondrait. La bonne mesure du gain était le
  but de la sous-question, pas le nombre de phrases. L'épreuve : une chaîne de 40 pages qui citent
  sans répondre — 41 pages lues avant, 2 après ; les citations restent pour le rapport.
* **SUJET-1 — des mots-clés ne partent seuls que s'ils nomment le sujet.** Les sources spécialisées
  (Wikipédia, arXiv, GitHub) reçoivent les mots-clés d'une sous-question au lieu de ses requêtes. Le
  journal : « art culture politique » pour l'histoire de la Bretagne, « ambition déception
  hégémonie », « IA stratégie Cour_des_comptes » — des dictionnaires en retour. Les requêtes, elles,
  gardaient le sujet (« manifestations Rennes 17 octobre 2026 »). Sans terme commun avec la question
  de l'utilisateur, la source reçoit désormais les requêtes. Une épreuve d'avant (le parallélisme des
  sources) s'appuyait sans le dire sur l'ancienne règle — son mot-clé « x » n'avait rien de commun
  avec sa question « Q ? » ; sa donnée nomme désormais son sujet, ce qu'elle prouve n'a pas changé.
* **DEPEND-1 — une dépendance qui échoue ne bloque plus.** Le modèle déclare des dépendances entre
  sous-questions ; quand la première restait introuvable, toutes les suivantes devenaient « bloquées »
  en cascade (sept colonnes sur huit le 17/10, « non cherchée — dépend de la sous-question 1 »), alors
  que le plan leur avait écrit leurs propres requêtes. Une dépendance est un **contexte**, pas une
  condition : la sous-question attend que les siennes soient *réglées* (répondues ou introuvables),
  puis cherche avec ce qu'on sait — sans réécrire ses requêtes quand il n'y a rien à y ajouter.
  L'état « bloquée » disparaît, de l'organe, du rapport et de la toile.
* **Les pages dans une autre langue** : la recherche du sodium-ion n'a vu aucun rejet — le 7B n'avait
  **proposé** aucune citation sur trois articles d'arXiv très pointus (l'anode en carbone dur) pour
  une question sur le principe général. Un jugement de modèle et un choix de source : remesuré avec
  le nouveau cerveau avant d'y toucher (§ 6).

## 5. La toile : une demande, une rangée ; une suite, une flèche

Il m'a dit, ce soir, que je serais meilleur juge que lui. J'ai donc regardé l'image du tableau de
son essai du 17/10 (le Glucose sans fenêtre, sept demandes) avec l'œil d'un utilisateur :

* **Tout tenait sur une seule ligne.** Chaque bulle se posait « à droite de tout le tableau » : un
  ruban de quinze bulles où rien ne disait quelle recherche répondait à quelle demande, et des
  flèches « précise la demande » qui traversaient le vide en biais (le chantier 5 de la passation).
  **RANGEE-1 — une demande, une rangée** : sa bulle Demande s'ouvre **sous tout le tableau, au bord
  gauche** (Protéo lit le tableau, `glucose_lire`, et prend le bas de son nœud le plus bas) ; ses
  recherches et ses rapports se posent à sa droite, dans l'ordre où ils arrivent, la rangée
  s'élargissant avec les colonnes ; la demande suivante ouvre la rangée d'en dessous. L'écart entre
  deux rangées est la place d'un titre de bulle et deux marges — aucun nombre neuf. La flèche
  « précise la demande » devient horizontale : la Recherche est à côté de sa Demande. Le sabotage a
  trouvé une épreuve aveugle (« y > 500 » laissait passer une rangée posée sur le *haut* du dernier
  nœud) ; elle est exacte désormais.
* **SUITE-2 — une demande qui poursuit la précédente se relie à elle** (le chantier 6 ; l'objectif
  3 de la VISION, des graphes de questions plutôt qu'un fil). Protéo pose déjà, quand une mission est
  en cours, la question fermée « la nouvelle demande poursuit-elle ce travail ? » ; il la pose
  désormais aussi contre la demande d'avant. Oui : une flèche « et ensuite » descend de la bulle
  d'avant à la nouvelle. Une question de plus par demande (~0,5 s), seulement quand la toile est
  posée. Les notes de Protéo, qui parlent dans la voix de l'utilisateur, ne sont jamais « la demande
  d'avant ».

## 6. Les mesures

**Ollama mis à jour** (0.32.5 → 0.40.6, installeur signé par Ollama Inc. vérifié). Neuf modèles
téléchargés depuis la bibliothèque d'Ollama, puis trois de plus trouvés en route (bonsai-2, ornith).

**Le criblage : le tri et la suite** (l'épreuve `tri` à la main, la production même, date figée au
17/10) — sur la même machine, le même jour :

| modèle | tri (31) | suite (10) | fautes graves | place mesurée | par question |
|---|---|---|---|---|---|
| qwen2.5:7b (en service) | 27 | 9 | 1 | 4,8 Go, tout sur la carte | 0,3 s |
| qwen3.5:4b | 27 | 9 | **0** | 3,1 Go | 0,4 s |
| **bonsai-2:9b** (Qwen3.5-9B ternaire) | 27 | **10** | 1 | **2,8 Go** | 0,5 s |
| gemma4:e4b (QAT) | 27 | 9 | 1 | 3,1 Go | 0,4 s |
| granite4.2:8b | 26 | 9 | 1 | 5,9 Go | 0,4 s |
| ornith:9b | 27 | 10 | 1 | 5,6 Go (fichier) | 2,1 s |
| qwen3.5:9b | 24 | 10 | 0 | 6,2 Go, **déborde** | 0,6 s |
| gemma4:12b (QAT) | 28 | 9 | 1 | 8,6 Go, **déborde** | 2,6 s |
| ministral-3:8b | 26 | 9 | 2 | 6,0 Go, **déborde** | 0,8 s |
| granite4.2:3b | 22 | 8 | 5 | 2,5 Go | 0,2 s |
| lfm2.5 | 19 | 5 | 0 | — | 0,2 s |
| bonsai-2:4b | 19 | 8 | 0 | 1,7 Go | 0,7 s |

**Ce tableau départage mal**, et c'est une leçon : sept modèles sur sept ratent les trois mêmes cas
(« écris un script… exécute-le… lis le fichier » — tous disent « workflow », l'étiquette dit
« outils ») et le même cas de suite (« non, je voulais parler de Rennes en Bretagne » — une
correction, donc une suite). La consigne du tri a été réglée phrase par phrase sur qwen2.5:7b le
17/10 : le banc favorise le sortant. Le criblage élimine (granite 3B, lfm2.5, bonsai 4B), il ne
choisit pas.

**La campagne du banc** (`scripts`, `agent`, `analyse`, `tri`, répétitions alternées) — **en cours
à la fin de cette fiche** ; une première campagne, interrompue, a donné une répétition complète pour
deux modèles :

| | qwen2.5:7b | qwen3.5:4b |
|---|---|---|
| **`agent`** (la boucle d'outils du chat, 10 cas) | **3 / 10** (puis 3 / 8 à la reprise) | **7 / 10** |
| `analyse` (6) | 5 / 6, puis 6 / 6 | 6 / 6 |
| `scripts` (10) | 5 / 10, puis 6 / 10 | 3 / 10 |
| `tri` (41) | 36 | 36 |
| durée médiane d'un cas `agent` | 45 s | **109 s** |

Lecture provisoire, à confirmer par la campagne entière : **qwen3.5:4b fait deux fois mieux sur ce
que le chat fait vraiment** (la boucle d'outils), au prix d'un temps doublé — il réfléchit avant
chaque pas. Il rate davantage `scripts`, mais **cette épreuve mesure l'« action directe »**, un
chemin que la production n'emprunte plus (le chat passe par la boucle) ; or la règle de bascule de
Protéo refuse tout candidat « moins bon » sur une épreuve, quelle qu'elle soit — elle doit être
revue avant de conclure. qwen3.5:9b a été écarté en route : 50 s par cas `scripts`, il déborde de la
carte et mange le processeur de l'utilisateur (l'inverse de se nicher là où il reste de la place).

## 7. Prouver

* **Protéo** : 146 épreuves du serveur, 100 + 15 des autres caisses, vertes ; **49 sabotages**
  (`PROTEO/outils/sabotages-tri-et-toile.json`, généré par `faire_sabotages_tri.py`) : les 32 de la
  fiche 76 remis à jour, et 17 neufs (MODELE-1, LANGUE-1, RAISONNEMENT-1, PROGRES-1, SUJET-1,
  DEPEND-1, SUITE-2, RANGEE-1, la campagne restreinte). Le sabotage a trouvé une épreuve aveugle
  (RANGEE-1), corrigée. Chaque épreuve neuve est d'abord tombée sur le code intact.
* **Protéo n'est toujours pas enregistré dans git** — ses changements s'ajoutent à ceux des sessions
  d'avant : à sa parole. Le `proteo.exe` de son raccourci **n'a pas été remplacé** : les lois de la
  toile neuves (RANGEE-1, SUITE-2) n'ont pas encore été vues en essai réel.

## 8. Ce qui reste, dans l'ordre

1. **Lire la campagne finie** (`/api/banc` du Protéo de banc, port 7861, ou le journal), puis
   mesurer le régime **sans réflexion** (`PROTEO_RAISONNEMENT=non`) pour le meilleur, et trancher la
   loi de la réflexion dans la boucle.
2. **La règle de bascule** : peser les épreuves par le chemin qu'elles exercent (retirer `scripts`
   du verdict, ou la refaire sur la boucle).
3. **Les essais réels** du meilleur candidat (sept demandes fixées : salut, ses outils, le script des
   nombres premiers, le disque, les abeilles, le dernier grand prix, le sodium-ion), le Glucose sans
   fenêtre, **les images regardées** — RANGEE-1 et SUITE-2 en vrai ; puis lui proposer la bascule.
4. **La machine qui choisit seule** : un troisième vivier, la bibliothèque d'Ollama (ses pages, où
   bonsai-2 était) ; tirer sans `hf.co` ; un criblage mesuré de tout ce qui tient (une répétition de
   l'épreuve `agent`), puis le banc complet pour les deux meilleurs — au lieu d'un seul candidat
   choisi par un a priori.
5. **L'organe** : les pages dans une autre langue (remesurer avec le nouveau cerveau) ; **le flou**
   (des recherches parallèles quand le savoir manque — la loi de Charnov y sert, pour répartir
   l'effort) ; la mission (une étape de recherche qui se coche sur un bilan).
