# 75 — La porte des agents : Protéo écrit dans Glucose

> Session du 17/10/2026. Sa parole : *« connectique entre Glucose et Protéo »* — le système qui
> permet à Protéo d'écrire directement sur le document, l'affichage des pages web en miniature
> qui ouvrent le navigateur au clic, les flèches entre les références et le document qu'il
> remplit, avec le surlignage et le pointage de la flèche dès qu'il fait un rapprochement ; une
> bulle « demande », une bulle « recherche » qui la précise. Et le pourquoi : *« si une IA
> spécialisée dans le travail et dans les projets utilise à 99 % Glucose lors de ses workflows
> […] alors Glucose aura enfin atteint son stade »* — l'objectif 2 de la VISION, et l'origine
> même de Glucose.

## 1. Ce qui existait, et ce qui manquait

* **Glucose Tauri avait un pont MCP** (`mcp/glucose-mcp.mjs`, Node, onze outils : lire, chercher,
  ajouter une note, relier deux notes *depuis une phrase* — `sourceSel`). Il lisait et écrivait le
  **fichier** : Glucose devait être fermé (*« l'app écraserait l'écriture du pont »*), rien
  n'apparaissait sous les yeux, rien ne s'annulait. Glucose Rust l'avait perdu ; la configuration
  Claude de son PC pointait encore vers lui (le serveur `glucose` échouait à chaque session).
* **Le modèle de Glucose Rust avait déjà la grammaire** : les flèches ancrées à un passage
  (`TextAnchor`, FLECHE-4 — le modèle du W3C Web Annotation), qui suivent l'édition ; les
  membranes et leur appartenance (MEMB-1) ; une photo garde l'adresse d'où elle vient
  (`source_url`, lue de Tauri, jamais montrée) ; le lanceur, qu'un double-clic ouvre.
* **Protéo** (`Documents\PROTEO`, Rust, Ollama `qwen2.5:7b`) a un organe de recherche dont les
  **citations sont vérifiées mot pour mot** dans la page lue, un client MCP (sonde des deux
  époques du protocole), et un navigateur sans tête (`chrome-headless-shell`). Son bilan du
  16/10 dit sa faiblesse : le petit modèle invente des appels d'outils et dérive de langue.

## 2. La recherche

* **MCP, version du 28/07/2026** (modelcontextprotocol.io) : sans état — chaque requête porte sa
  version et son client dans `_meta` ; `server/discover` obligatoire ; erreur `-32022` pour une
  version inconnue, avec la liste des versions parlées ; un serveur « dual-era » sert aussi la
  poignée de main `initialize` des clients anciens. Le transport stdio : un message par ligne,
  rien d'autre sur la sortie. **Un transport sur un flux d'octets (prise Unix, TCP) doit
  reprendre ce cadrage** (« Custom transports … SHOULD reuse the stdio framing »).
* **tldraw** (Steve Ruiz, AI Engineer Europe 2026, « Agents on the canvas ») : l'écart entre
  *générer* un artefact et *collaborer* dessus. Son serveur MCP communautaire passe par un
  serveur de canevas et une diffusion WebSocket ; un autre connecteur dit ses limites : **écritures
  non idempotentes** (un appel retenté duplique les formes), **ni suppression ni annulation**.
* **Les aperçus de liens** (Open Graph, oEmbed) : une carte titrée, une image `og:image`, et la
  capture d'écran comme recours — qu'un navigateur sans tête fait seul.

## 3. La conception

### Une porte, pas un fichier

Le pont de Tauri écrivait le fichier ; ici l'agent parle **au Glucose ouvert**, et chaque appel
devient **un geste** comme ceux de la main (R1 : branché, visible, annulable, enregistré) :

```text
agent ──stdio──► glucose --mcp (le relais) ──porte locale──► fil de la connexion
                                                                 │ protocole (pur)
                                        boîte aux lettres ◄──────┘ un appel d'outil
                                               │ réveil
                        boucle de Glucose : l'accord, puis l'outil, en un geste du journal
```

* **La porte n'est pas un port réseau** : un tuyau nommé sous Windows, une prise Unix ailleurs.
  Aucune page web ne peut l'atteindre (le piège des serveurs locaux en HTTP, que la spécification
  défend par l'en-tête `Origin`).
* **Le relais** `glucose --mcp` : ce qu'un client MCP lance (Claude Code, Protéo). Il recopie les
  lignes ; **il ouvre Glucose s'il ne l'est pas** — sans lui confier ses flux (sa sortie
  corromprait le protocole), et hors de tout *Job* Windows (`CREATE_BREAKAWAY_FROM_JOB`) : Protéo
  range ses programmes dans un *Job* qui tue sa descendance ; le Glucose de l'utilisateur, et son
  travail, n'en sont pas.
* **Le protocole est pur** (`agents/protocole.rs`) et sert les deux époques sur la même
  connexion ; **le JSON est écrit ici** (`agents/json.rs`, ~300 lignes, borné en profondeur) plutôt
  que `serde_json` et ses trois caisses : aucune dépendance neuve.

### Les invariants

* **AGENT-1 — un appel est un geste** (`Store::geste_d_agent`) : une entrée du journal, que
  `Ctrl+Z` défait ; **atomique** — un appel qui échoue en route (un passage introuvable) ne laisse
  rien ; et refusé pendant un geste de l'utilisateur (son `Ctrl+Z` défairait les deux) : l'appel
  attend son relâchement.
* **AGENT-2 — un agent ne touche jamais sa sélection** : gardée autour du geste, rendue sans les
  nœuds retirés.
* **AGENT-3 — un passage cité est mot pour mot dans le nœud** : la flèche part d'une phrase exacte
  et arrive à une phrase exacte, surlignées ; sinon l'appel est refusé et dit pourquoi.
* **AGENT-4 — poser un identifiant qui existe le met à jour** : un appel retenté ne double rien (le
  défaut de tldraw). La mise à jour garde l'appartenance du nœud, et écrit son texte par la porte
  qui fait suivre les ancres des flèches.
* **AGENTS-1 — rien sans son accord** : le premier appel d'un agent pose une question dessinée
  (*« Un programme qui se présente comme « proteo » demande à lire ce tableau et à y travailler
  sous tes yeux… »* — **Autoriser**, **Refuser**, **Plus tard**) ; l'appel attend. Le choix vaut
  jusqu'à la fermeture ; « Plus tard » refuse cet appel et redemande au suivant ; la question ne
  chasse jamais une autre question.
* **AGENTS-2 — la porte n'est qu'à lui** : sous Windows, le tuyau ne s'ouvre qu'au compte qui l'a
  créé (`D:P(A;;GA;;;<SID>)`), refuse le réseau, et le relais **vérifie que le processus qui le
  tient est du même compte** ; ailleurs, la prise vit dans un dossier `0700`.
* **JSON-1** : une entrée hostile rend une erreur, jamais une panique ; l'imbrication est bornée
  (128) et une ligne aussi (16 Mio).

### Les outils (huit)

`glucose_lire` (le tableau qu'il regarde : chaque nœud, sa boîte, son texte, ses bouts ; la partie
visible ; les tableaux — `dans` pour une membrane), `glucose_poser_texte` (texte ou post-it ; la
hauteur mesurée par la typographie de Glucose), `glucose_poser_membrane`, `glucose_ecrire`
(remplacer ou **allonger** — un document qu'on remplit ; la hauteur suit, la membrane grandit),
`glucose_relier` (les six relations, les passages), `glucose_poser_page`, `glucose_deplacer`,
`glucose_retirer`. **Sans position, un nœud se pose sous le contenu de sa membrane — qui grandit
pour le contenir —, ou à droite de tout le tableau** : un agent qui ne dit pas où poser n'empile
jamais rien.

### La page web en miniature

**Une photo qui garde son adresse** — pas un nouveau genre de nœud : tout ce qui sait déjà dessiner,
copier, exporter une photo le sait pour elle. Sa capture (que l'agent fait — Protéo avec son
navigateur sans tête, 1280 × 800) est **scellée dans le document** comme une image qu'une page
livre (DEPOT-4) : le fichier de l'agent peut disparaître. Sous elle, un texte : son titre en gras,
son adresse en lien, puis ce qu'on y a lu.

**Le double-clic l'ouvre dans le navigateur** — la règle du lanceur, dont le double-clic ouvre le
fichier : une photo n'a rien d'autre à ouvrir. Et `Ctrl`+clic aussi, comme un lien dans un texte.
Le clic seul la choisit, comme tout nœud : *« lorsqu'on clique, ça ouvre le navigateur »* aurait
interdit de l'attraper — c'est la raison même pour laquelle les liens des textes demandent `Ctrl`
(`links.rs`). Seuls `http` et `https` s'ouvrent.

## 4. Protéo

**Le programme pose, pas le modèle.** La toile (`proteo-server/src/toile.rs`) traduit les
événements de l'organe de recherche — son plan, ses pages lues, ses citations vérifiées, son
rapport numéroté — en gestes dans Glucose. Aucun texte du modèle n'est interprété en geste : la
règle d'or de `PLUGINS.md`, *« l'IA décide le sens, le code décide la géométrie »*, et le bilan de
Protéo, qui a mesuré ce que le 7B fait des appels d'outils.

* une bulle **Demande** : la demande, puis *« Ce que Protéo en comprend »* (les sous-questions) ;
  une flèche de la Recherche vers elle, *« précise la demande »* ;
* une bulle **Recherche**, une **colonne par sous-question** (côte à côte : chacune grandit vers le
  bas sans recouvrir la suivante), et dans chacune les pages lues en capture, avec sous chacune
  ses citations, `> « … » [n]` ;
* une bulle **Rapport** : le rapport rédigé ; de **chaque phrase qui cite `[n]`** part une flèche
  vers **la citation n** sous sa page — les deux passages surlignés.

Le client (`proteo-connectors/src/glucose.rs`) trouve Glucose (`PROTEO_GLUCOSE`, puis
`%LOCALAPPDATA%\Programs\Glucose`, puis le `PATH`), **lit sa version d'abord** — un Glucose sans
porte ignorerait `--mcp` et ouvrirait simplement sa fenêtre — et tient une **session** MCP
(`mcp/session.rs`) le temps du travail. La recherche n'attend jamais Glucose : le projecteur
travaille sur un fil à lui. `PROTEO_SANS_GLUCOSE` l'éteint.

### Deux défauts trouvés en route, et leurs invariants

* **TUYAU-1 — lire et écrire en même temps ne se bloquent pas.** Le relais lit les réponses sur
  un fil pendant qu'il écrit les requêtes sur un autre ; sur un tuyau ouvert en mode synchrone,
  Windows sérialise les opérations d'un même objet fichier, et l'écriture attendait une lecture
  qui attendait une réponse à cette écriture. Vu au premier essai réel : la découverte passait,
  `tools/list` jamais. Le bout du relais est donc ouvert en **E/S superposée**
  (`FILE_FLAG_OVERLAPPED`, `porte/tuyau.rs`), chaque opération avec son `OVERLAPPED`. L'épreuve du
  relais passait par chance — elle écrivait tout avant de lire.
* **RELAIS-1 — le relais n'ouvre Glucose qu'au premier outil appelé.** Un client lance ses
  serveurs au début de chacune de ses sessions (Claude Code) : un relais branché tout de suite
  ouvrirait Glucose à chaque fois. Il répond donc seul à la découverte, à la poignée de main et à
  la liste des outils — par le même protocole pur —, et ne se branche qu'au premier appel ; il
  rejoue alors la poignée de main d'un client ancien, et tait les réponses déjà données.
* **Retirer l'accord** : une commande, « Retirer l'accord des agents… » (palette `Ctrl+Maj+P`),
  dit qui l'a et le retire sur sa parole — sans elle, l'accord ne se reprenait qu'en fermant
  Glucose.

## 5. Prouver

* **Les épreuves** : 19 dans le noyau (`store/agents/tests.rs`), une cinquantaine dans
  l'application (le JSON, le protocole des deux époques, chaque outil sur une vraie application,
  l'accord, la vraie porte, TUYAU-1, RELAIS-1), et le relais de bout en bout
  (`tests/agents_suite.rs`) : le vrai programme `glucose --mcp`, lancé comme un client le lance.
* **La suite entière** : 2 666 épreuves vertes sous Windows ; clippy strict à zéro sous Windows,
  **sous Linux** (la distribution `Glucose-Linux` : la prise Unix et le relais de bout en bout, 41
  épreuves) et **pour Android** (qui a trouvé la ligne `--mcp` du programme de bureau, gardée
  depuis hors d'Android). Le Mac ne se compile qu'à la CI.
* **Les sabotages** (`outils/sabotages/agents.json`) : 36 gardes cassées exprès, chacune fait
  tomber une épreuve. Ils ont trouvé **une épreuve aveugle** — le suivi des ancres à une mise à
  jour : Glucose retrouve un passage par sa citation et son contexte même quand sa place est
  fausse, et l'épreuve ne regardait que le texte désigné ; elle lit désormais la **place
  enregistrée** de l'ancre. Et deux épreuves qui **se figeaient** au lieu de tomber (TUYAU-1,
  RELAIS-1) : toute lecture qui peut attendre se fait sur un fil, avec un délai.
* **Le bout en bout réel**, sans aucune fenêtre sur son écran : un Glucose sans fenêtre
  (`examples/porte_sans_fenetre`, `bench/porte.rs` — la vraie application, la vraie boucle), et
  Protéo qui travaille à travers `glucose --mcp` :
  * le **rejeu** de sa vraie recherche du 16/10 (« modélisation moléculaire ») : 31 gestes, aucun
    refusé ;
  * deux **recherches neuves** menées en direct par le vrai organe, le vrai `qwen2.5:7b` et le vrai
    web (`recherche_neuve_posee_en_direct`) : « Qu'est-ce que le glucose… » (2 pages, 4 citations,
    18 gestes) et « Comment les abeilles communiquent-elles entre elles ? » (10 pages,
    19 citations, 49 gestes, quatre colonnes) ;
  * **les images regardées** : les bulles, les captures, les citations sous chaque page, les
    flèches du rapport qui contournent les bulles ; et, une flèche survolée, **la phrase du
    rapport qui brille en violet et la citation en vert** — le surlignage et le pointage demandés.

### Ce que l'image a montré, et qui a été corrigé

* **Des citations illisibles** (« « 3Tw▒ » [6] ») : universalis.fr envoie sa page **compressée en
  gzip sans qu'on le demande**, et Protéo n'avait pas la décompression — il lisait des octets
  compressés comme du texte, et « vérifiait » des citations dans ce charabia. Corrigé côté
  Protéo (les fonctions `gzip`, `brotli`, `deflate` de `reqwest`, déjà là), avec l'épreuve qui
  tombe sans elles. **Un défaut que seule la toile a rendu visible.**
* `d&rsquo;alarme` : le lecteur de pages de Protéo ne décodait que cinq entités. Les 96 du
  latin-1 se décodent désormais par leur rang (la n-ième désigne U+00A0 + n), et la ponctuation
  typographique.
* Les adresses entre crochets que le lecteur garde après chaque lien encombraient les citations :
  retirées, du texte écrit comme du passage désigné — les deux restent le même texte.
* Les titres longs débordaient de leurs bulles : « Recherche », « Question 1 »…, et la question
  entière devient le premier texte de sa colonne.

### Ce que l'image montre encore

* Une flèche ancrée **s'arrête au bord** de la carte, sur la droite qui va vers la hauteur du
  passage : quand elle arrive très en biais, elle touche le bord loin au-dessus de sa citation
  (le comportement de Tauri, `findTextSelPosition`). La citation brille au survol ; la faire
  arriver *à sa hauteur*, par le bord, serait une loi neuve des flèches — à lui de juger.
* Une capture montre ce que la page montre : parfois sa bannière de cookies.

## 6. Ce qui reste

* **Le tri de la demande** (sa bulle : une réponse courte, des outils, un workflow complet — et sa
  taille) : Protéo ne sait aujourd'hui que « projet / pas projet » (`mission/juge.rs`).
* **La mise en page « très designée » pilotée par Protéo**, ses autres outils, Glucose comme
  support premier de ses projets — sa parole : *« plus tard »*.
* **La porte au téléphone** : aucune (aucun agent n'y tourne à côté de Glucose).
* **Une page posée par l'utilisateur** (un lien collé) n'a pas encore de miniature : Glucose sait
  télécharger une image (`rapatrier.rs`) ; lire l'`og:image` d'une page serait le pas suivant.
