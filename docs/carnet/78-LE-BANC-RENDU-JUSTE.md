# 78 — Le banc rendu juste, et le cerveau choisi sur la mesure

> Session du 18/10/2026 (l'après-midi), suite de la fiche 77. Elle devait conclure la campagne
> interrompue et basculer de cerveau. Avant de mesurer, elle a lu au journal pourquoi les mesures
> d'hier étaient ce qu'elles étaient — et quatre défauts du banc et de la boucle sont apparus, dont
> deux touchaient aussi la production. Sa parole de la veille : *« c'est plus à toi de tester […] tu
> seras sûrement meilleur juge que moi »*.

## 1. Ce que le journal disait des mesures d'hier

* **Les deux échecs `page` de qwen3.5:4b n'étaient pas les siens.** La fiche 77 les comptait
  « deux réponses illisibles » ; le journal dit `fin = length` à **1 536 jetons pile**, deux fois,
  texte vide. La boucle bornait la sortie à un quart de la fenêtre (512 à 1 536 jetons) — une
  constante d'avant les modèles qui réfléchissent : la réflexion, puis une page HTML entière, n'y
  tenaient pas. Et Protéo disait ensuite au modèle « ton JSON était mal formé », parce qu'une
  réponse sans rien de visible était classée « perdue » avant qu'on regarde si elle était coupée.
* **Une hypothèse démentie en route** : l'analyseur des appels d'outils de Qwen3.5 sous Ollama
  (l'issue [ollama#14493](https://github.com/ollama/ollama/issues/14493), toujours ouverte : un
  format JSON au lieu du XML de Qwen3-Coder). À la version 0.40.6, le code d'Ollama rend et lit bien
  le XML (`model/renderers/qwen35.go`, `model/parsers/qwen35.go`) : l'issue ne s'applique plus.
* **L'épreuve `scripts` mesurait un chemin mort** : l'« action directe » du chat d'avant le
  01/10/2026. `routes/propositions.rs` et `consignes::action_directe` ne vivaient plus que pour
  elle (« pour que ses mesures restent comparables »). Ses cinq familles de tâches, elles, sont des
  demandes d'utilisateur (« combien de fichiers .log… écris ce nombre dans… ») jugées sur les
  fichiers produits, quel que soit le chemin.
* **L'épreuve `outils` n'exerce pas non plus le chat** : il ne montre pas au modèle les outils du
  catalogue MCP. Son catalogue demandait 131 072 jetons de contexte à un Qwen3.5 — d'où les 16 Go
  de la fiche 77 — et sa préparation lançait chaque serveur MCP connu, même quand la campagne ne la
  jouait pas.

## 2. Le banc rendu juste (Protéo)

* **SORTIE-1 — la sortie prend toute la place que la fenêtre laisse.** La fenêtre est déjà la plus
  grande qui tienne sur la carte graphique (`contexte::mesurer`) ; la réserve ne sert plus qu'à
  décider ce que l'élision garde de l'entrée. Une réponse coupée par la fenêtre avant tout texte est
  dite **coupée** (« ta réflexion a pris la place »), et deux de suite arrêtent le tour.
* **`scripts` disparaît** ; l'épreuve `agent` passe de 10 à **20 cas** : ses cinq familles, plus
  les cinq des tâches de fichiers, par la vraie boucle du chat. Le reste de l'ancien chat part avec.
* **Le verdict pesé par le chemin** (`bilan::VERDICT` : `agent`, `tri`, `analyse`) : la règle de
  bascule — meilleur quelque part, moins bon nulle part, mesuré partout où la référence l'est — ne
  lit plus que les épreuves qui exercent la production. `outils` se joue sur demande, hors verdict ;
  une campagne sans argument joue le verdict, et peut proposer.
* **Un cerveau est un modèle et le régime de sa boucle** (`Cerveau` :
  `qwen3.5:4b~sans-reflexion`). Deux régimes d'un même modèle sont deux candidats d'une même
  campagne, entrelacés — comparer deux exécutions, c'est comparer du bruit (METHODE § 4).
* **FENETRE-1 — un modèle, une fenêtre.** Le journal d'Ollama comptait **34 rechargements du
  modèle pour 46 requêtes** : chaque cas du banc remesurait sa fenêtre (la place libre varie : une
  fenêtre neuve à chaque cas), et le tri et la suite posaient leurs questions à 4 096 jetons entre
  deux pas de la boucle à la sienne. Chaque rechargement coûte deux secondes et le cache de
  l'invite — **en production aussi, à chaque demande**. Le tri, la suite et le plan lisent la
  fenêtre retenue du modèle ; le banc la retient comme le chat.
* **ANNULE-1 — une annulation se garde** : le canal naissait sans récepteur, et `send` n'écrit rien
  quand personne n'écoute. Un clic sur « Annuler » pendant le tri ou l'ouverture d'une mission était
  perdu. Trouvé parce que la charge de la campagne a fait tomber une épreuve de minutage qui passait
  depuis des semaines par la chance de ses 200 ms.
* **Une course entre épreuves** : `etat_de_test` nommait ses dossiers à l'heure, et deux épreuves
  lancées dans le même instant partageaient conversation et mission. Un compteur.
* **La loi de la réflexion** : aucune constante ne la décide pour tous les modèles. Elle se mesure
  modèle par modèle, et s'adopte avec lui : `PROTEO_MODELE` et « adopter » prennent un cerveau ;
  la variable `PROTEO_RAISONNEMENT`, un réglage de mesure, disparaît.

## 3. La machine qui choisit seule — les premières briques

* **La bibliothèque d'Ollama** (`vigie/bibliotheque.rs`) : `ollama.com/library` liste 256 modèles en
  une page servie sans script ; `/library/<nom>/tags` leurs étiquettes et empreintes (celle de
  `bonsai-2:9b` est celle du modèle installé ici) ; le **registre** (`registry.ollama.ai`, en JSON)
  la taille exacte de chaque couche et la version d'Ollama exigée (`requires`). Les attributs
  `x-test-*` qu'une session d'avant y avait lus n'existent plus. **Les badges sont une déclaration
  incomplète** : granite4.2 appelle des outils et sa page n'en porte aucun ; ils ordonnent,
  ils n'écartent pas — 208 modèles à examiner.
* **La tête d'un fichier GGUF** (`gguf.rs`) : le registre sert des poids bruts, mais accepte une
  lecture partielle ; les 16 premiers Mo de granite4.2:8b donnent ses 41 paires, dont un gabarit de
  conversation de 9 013 caractères qui nomme les outils. Ce qui se juge **avant** de télécharger.
* **Ce qui reste, et pourquoi pas tout de suite** : son disque n'a qu'environ 70 Go de libres —
  faire courir ensemble tous les modèles qui tiennent est impossible. La forme juste est une
  **course contre le champion** (*racing* : Maron et Moore, 1994 ; F-Race,
  [Birattari et al., GECCO 2002](https://cs.ubc.ca/labs/algorithms/Courses/CPSC536H-10/Materials/BirEtAl02.pdf)) :
  chaque prétendant passe les cas du champion, un par un, et s'arrête dès qu'un test apparié le
  démontre moins bon ; ses poids partent ensuite. Aucun nombre de survivants à choisir.

## 4. Les mesures

**Le criblage** (une répétition, les 20 cas `agent` par la vraie boucle, le tri et l'analyse) :

| cerveau | boucle du chat (20) | tri (41) | durée médiane d'un cas | verdict |
|---|---|---|---|---|
| **qwen3.5:4b** | **14** | 36 | 42 s | en tête |
| gemma4:e4b (QAT) | 13 | 36 | 93 s | aussi bon, deux fois plus lent |
| bonsai-2:9b | 12 | 37 | 54 s | proche second |
| qwen2.5:7b (en service) | 5 | 36 | 36 s | la référence |
| granite4.2:8b | 1 sur 9 | — | 127 s | **écarté** : 5,9 Go sur la carte, une fenêtre de 5 632 jetons — « contexte plein », réflexion coupée faute de place |

**La campagne fine** (le banc corrigé, FENETRE-1 comprise ; une répétition sur deux, arrêtée à
20 h 30 à sa demande d'éteindre), comparaison appariée exacte sur les 20 cas :

| | réussites | cas gagnés / perdus | p |
|---|---|---|---|
| **qwen3.5:4b contre qwen2.5:7b** | **15 contre 3** | 12 / **0** | **0,0005** |
| qwen3.5:4b, avec et sans réflexion | 15 contre 10 | 6 / 1 | 0,125 |
| qwen3.5:4b contre bonsai-2:9b | 15 contre 12 | 6 / 3 | 0,51 |

* **qwen3.5:4b est démontré meilleur que le cerveau en service**, pour 13 % de temps en plus au
  total (2 502 s contre 2 215 s).
* **La réflexion semble aider** — elle gagne les deux cas « corriger », et le régime sans réflexion
  s'est enlisé quinze minutes dans une boucle (`paquet#2`) — mais 20 cas ne le démontrent pas. Une
  réserve, dite : sans réflexion, le modèle garde l'échantillonnage du Modelfile (température 1,
  top_p 0,95), celui que la fiche de Qwen recommande **avec** réflexion ; sans, elle recommande 0,7
  et 0,8. La seconde répétition reste à faire.

## 5. Les essais réels (interrompus)

Le vrai Protéo (qwen3.5:4b à sa manière, `PROTEO_MODELE`), le vrai web, un Glucose sans fenêtre.
Six demandes sur huit avant qu'il éteigne :

* « salut » : court, en français, aucun outil (15 s) ; ses outils : listés, les vrais (73 s) ; le
  disque C : **120,5 Go libres**, lus sur la machine (33 s).
* Le script des nombres premiers : « la première tentative n'a pas fonctionné » — **à relire** dans
  sa conversation avant de conclure.
* **Les abeilles : répondu en 18 s, de mémoire, sans chercher ni citer** — le défaut de la fiche 76
  (le tri a dû dire « réponse »). À regarder au prochain essai : le tri, ou la consigne.
* Le grand prix : interrompu pendant sa recherche.
* **La toile** : RANGEE-1 vu en vrai — chaque demande sous la précédente, au bord gauche, la
  recherche à sa droite. Mais la première colonne d'une recherche s'étire très loin (une vingtaine
  de pages pour une sous-question) : la colonne grandit sans borne vers le bas. SUITE-2 n'a pas été
  vu (la demande de suite n'est pas partie).

## 6. Prouver

Protéo : **154 épreuves du serveur** et 182 des autres caisses, vertes ; **25 sabotages sur 25**
pour la session (`PROTEO/outils/sabotages-cerveau.json`, généré par `faire_sabotages_cerveau.py`),
et les 49 d'hier rejoués sur le code changé, tous tombés. Deux épreuves aveugles trouvées par le
sabotage (la réflexion du banc, le filtre de la fenêtre par modèle), corrigées. **Rien n'est
enregistré dans git côté Protéo** (51 fichiers modifiés depuis le 16/10) : à sa parole ; une
archive des sources est dans `Documents\Proteo-sauvegardes`. Le `proteo.exe` de son raccourci
**n'est pas remplacé**.

## 7. Ce qui reste, dans l'ordre

1. **Les essais réels du 4B à finir** (le grand prix, le sodium-ion, la suite — `PROTEO/outils/essais_cerveau.md`),
   la conversation du script relue, les abeilles comprises ; puis **lui proposer la bascule**
   (« adopter » `qwen3.5:4b`), et remplacer `proteo.exe` en renommant l'ancien.
2. **La seconde répétition** de la campagne fine, pour trancher la réflexion (et l'échantillonnage
   du régime sans réflexion).
3. **La machine qui choisit seule** : brancher la bibliothèque à la vigie, l'inspection GGUF au
   registre, la course contre le champion (`banc/course.rs`, la décision écrite) et le tirage par nom.
4. **La toile** : borner la hauteur d'une colonne de recherche.
5. **L'organe** : les pages dans une autre langue, le flou, une étape de recherche cochée sur un bilan.
