# 131 — ACCÈS-H24 : les deux jauges, « Toujours, dans ce document », le relais qui survit à Glucose

> Session du relais (compte a), le 23/11/2026 depuis 8 h 23 — lancée par le veilleur 27 s après le
> redémarrage du PC (la session s'est ouverte seule, Glucose aussi : le « H24 » de la nuit tient). Sa
> consigne (signée à 0 h 14) : *« Avant le plan : le chantier ACCÈS-H24 (le nœud 🔑 de Claude) »*.
> À 8 h 31, la session interactive a tapé dans la mienne : *« ce matin, il a dû cliquer « Autoriser »
> TROIS fois dans son Glucose, à la main, après le redémarrage du PC — il en a assez, à juste titre.
> Finis le point 4 […], puis prends IMMÉDIATEMENT le point 1 (AGENTS-TOUJOURS) […]. Le choix
> « Toujours, dans ce document » doit valoir pour chaque agent de la machine — le terminal comme
> claude-code et tout autre nom — dès qu'il l'a choisi une fois. »*

## 1. En arrivant

* **Le PC a redémarré** entre 0 h 28 et 8 h 22 (`/tmp` vidé, le veilleur démarré à 8 h 23) : la
  relance du compte a « vers 4 h » n'a pas eu lieu. Rien d'autre n'avait bougé depuis la passation de
  0 h 16 ; DOSSIER-2 attend dans `stash@{0}`.
* **Sa session interactive** (VS Code, compte a) a repris à 8 h 24 : elle diagnostiquait la chronique
  du démarrage (le point 6, ARBITRE-DÉMARRAGE, qu'elle a ajouté au nœud 🔑). Elle mange la même
  jauge de 5 h que le relais. Je n'ai commité que mes fichiers, nommés un par un.

## 2. Remis en question

* **« Une jauge jamais lue compte pour libre »** (le nœud 🔑) : vrai, mais la lire d'avance n'est
  pas possible sans toucher aux identifiants de son compte (l'adresse `/api/oauth/usage` que
  `/usage` appelle n'est pas documentée, et son jeton expire). **Le lancement est la sonde** : la
  documentation de la barre d'état dit que `rate_limits` *« appears only after the first API
  response in the session »* ; le 23/11, la jauge du compte b (semaine à 100 %) s'est écrite trois
  minutes après son lancement. Le trou n'était donc pas de lancer : c'était de **garder ouverte** une
  session que le serveur refuse. Et **une jauge ne fait que monter jusqu'à sa remise** : ce qu'elle
  dit de plein reste vrai même lue tôt ; elle ne peut que sous-estimer (le compte servi ailleurs,
  claude.ai compris).
* **Lire l'écran pour voir la limite** (« You've hit your … limit », mon premier réflexe, les préfixes
  relevés dans le binaire de Claude Code : `You've hit your`, `You've reached your`, `You're out of
  usage credits`) : **écarté**. L'écran montre aussi ce que la session affiche — mes propres sorties
  de `strings` contenaient ces phrases ce matin même : une session qui travaille sur le relais se
  serait fait fermer. La jauge est la seule source.
* **« Toujours » par nom d'agent** : un nom n'est pas une identité (n'importe quel programme peut se
  dire « claude-code », toute commande d'un terminal se dit « le terminal ») ; ce qui distingue un
  agent de la machine, c'est d'atteindre la porte locale, que le système n'ouvre qu'à son
  utilisateur (AGENTS-2). Un accord durable par nom ne protégerait de rien et le ferait cliquer à
  chaque nom neuf. **L'accord durable porte sur le document** — c'est aussi ce que demandait la
  session interactive.
* **Où le retenir** : dans le dossier de Glucose (comme la clé des IA), jamais dans le `.glucose`
  (un fichier envoyé n'emporte pas l'accord) — le nœud 🔑 le disait. Son dossier Glucose ne se
  touche jamais : **je n'ai pas posé l'accord pour lui** ; c'est son clic, une fois.
* **Le relais MCP qui « se rebranche »** : se rebrancher tout de suite, et ouvrir Glucose s'il ne
  répond pas (la règle du premier appel, RELAIS-1), ferait naître **un second Glucose** à côté de
  celui que son service relance en dix secondes. Au rebranchement, Glucose était là un instant plus
  tôt : on lui laisse le temps d'un lancement (la borne qui existait, `LANCEMENT_MAX`) avant de
  l'ouvrir soi-même. Aucune constante neuve.

## 3. La recherche (consultée le 23/11/2026)

* Claude Code, [*Status line*](https://code.claude.com/docs/en/statusline) : `used_percentage` de 0
  à 100, `resets_at` en secondes Unix, *« Each window […] may be independently absent, and Claude
  Code drops a window once its `resets_at` time passes »* ; la barre se relance à chaque message de
  l'assistant (mais pas pendant un long appel d'outil).
* Claude Code, [*Hooks*](https://code.claude.com/docs/en/hooks) : un crochet « Stop » qui rend
  `decision: block` relance la session avec sa raison. Des tickets signalent que ce retour peut
  heurter les blocs de réflexion ([63287](https://claudeissues.com/issue/63287-opus-4-8-extended-thinking-stop-hook-block-re-entry-corrupts-thinking-blocks-400)) :
  rien vu ici, à surveiller.
* **Aucun rebranchement d'un serveur stdio dans Claude Code** :
  [43177](https://claudeissues.com/issue/43177-mcp-stdio-servers-never-auto-reconnect-after-disconnect),
  [82746](https://claudeissues.com/issue/82746-auto-reconnect-or-provide-claude-mcp-reconnect-name-for-dead-stdio-mcp-servers-t)
  (« not planned ») — un serveur stdio mort le reste jusqu'à `/mcp`. Un relais stable devant un
  serveur qu'on relance est une pratique connue (le *dev proxy* de
  [mcp-debugger](https://glama.ai/mcp/servers/@debugmcpdev/mcp-debugger/blob/1f43b32dbb97078498b56f1ed2ce202ca7cb8ac8/tools/dev-proxy/README.md)).
* JSON-RPC 2.0 § 5.1 : −32000 à −32099 réservés aux erreurs du serveur ; le kit TypeScript de MCP
  nomme −32000 `ConnectionClosed` — le code de la requête que la fermeture de Glucose emporte.

## 4. Ce qui est écrit

* **RELAIS-JAUGES** (`outils/relais/relais.py`, `6b0007b`) — le veilleur, à chaque tour :
  `garder()` dit « fermer » (une fenêtre à 100 % : le serveur refuse ; la session est close et
  l'autre compte lancé **dans le même tour**) ou « passation » (un seuil passé, rien demandé encore :
  la demande est **tapée dans la session**, sans attendre la fin du tour) ; le crochet « Stop » ne
  laisse partir la session qu'une fois la passation **récrite** (son empreinte a changé), ou après
  trois arrêts en deux minutes ; la demande dit aussi : *« si tu n'as rien changé, ajoute seulement
  en tête une ligne qui la confirme »* (une session qui vient d'arriver n'écrase pas la bonne
  passation par une pauvre). `plein()` rend **la plus lointaine** remise des fenêtres pleines. Le
  journal dit, à chaque fermeture, si la passation a été écrite.
* **Le ❤️** (`travail`) : ce que GitHub n'a pas, demandé à GitHub (`git ls-remote`, sans jeton : le
  dépôt est public) — l'envoi par l'adresse au jeton ne met jamais `origin/main` à jour.
* **`saboter.py` lit `unittest`** (« FAIL: … », « ERROR: … », « Ran N tests … OK »).
* **AGENTS-TOUJOURS** (`agents/toujours.rs`, `app/agents.rs`, `c8a3350`) : un quatrième choix,
  « Toujours, dans ce document », **offert seulement à un document qui vit dans un fichier** (sinon
  il mentirait) ; il vaut pour tout agent de la machine dans ce document, d'un lancement à l'autre ;
  `agents-toujours.txt` dans le dossier de Glucose (une identité de document par ligne, écrite par
  `persist::atomic` — le cliquet 11 me l'a rappelé) ; les IA du nuage gardent leur propre accord ;
  « Retirer l'accord des agents… » le dit et le reprend ; une écriture ratée vaut pour la session
  et s'y dit (sans toast : leur nombre ne grandit pas).
* **RELAIS-2** (`agents/relais.rs`, `agents/relais/lien.rs`) : le relais relaie ligne par ligne et
  tient **la dette** de chaque porte — les requêtes parties sans réponse. Une porte qui se ferme
  répond à chacune par une erreur qui le dit (son effet est incertain) ; une ligne qui n'est pas
  entrée se renvoie à la porte suivante ; la poignée de main rejouée se tait, comme au premier
  branchement. La dette distingue la requête **en vol** (son envoi n'est pas encore confirmé :
  l'envoyeur décide) de la requête **partie** (la fermeture lui répond) : chaque requête reçoit une
  réponse, jamais deux, jamais aucune.

* **PARTAGE-SURVIT** (`app/collaborer/reprise.rs`) : pour chaque document partagé (la clé : son
  fichier, brouillon compris), Glucose retient dans son dossier **le code de la salle** (lisible
  de lui seul : `persist::atomic::poser_un_secret_d_un_bloc`, une seule loi pour les secrets — la
  clé des IA y passe aussi), « en cours » pendant le partage ; **à la fermeture du document**,
  tout écrit et synchronisé, **le numéro du dernier geste de la salle que le fichier contient** —
  seulement si aucun de mes gestes n'attend le sien. Au lancement, le document rouvert rejoint sa
  salle à ce numéro, sous une marque neuve, et rattrape. Après un arrêt net, ou un geste parti
  sans retour, il **ne reprend pas tout seul et le dit** (une fois). « Quitter le partage »
  l'oublie. **Pourquoi exact** : un déplacement voyage comme un vecteur (`Deplacement::delta`) —
  rejoué deux fois, il déplacerait deux fois ; le numéro se lit donc à la fermeture du document,
  pas à la fin du programme (la boucle peut encore appliquer un geste entre les deux).

## 5. Vérifié

* **Le relais** : 31 épreuves (dont un vrai dépôt Git jetable pour le ❤️, avec la preuve à
  l'envers : l'ancienne loi y dit 2 quand la vérité est 1), **12 sabotages sur 12**
  (`relais-131`). Le veilleur relancé sur ce code (`KillMode=process` : ma session a survécu). Contre
  le vrai GitHub : « 0 pas encore sur GitHub » (l'ancienne loi disait 1).
* **AGENTS-TOUJOURS** : 9 épreuves neuves par les vrais chemins (la vraie question, la vraie
  réponse, un Glucose « relancé » qui relit le même fichier), dont la preuve à l'envers (« Autoriser »
  ne survit pas au relancement) ; **12 sabotages sur 12** (`agents-toujours-131`). **Au pilote**,
  le vrai Glucose sur une copie de `riche-v5.glucose` (sa maison à part) : la question montre le
  quatrième choix ; « Toujours » choisi pour « le terminal », un client MCP qui se dit
  « claude-code » écrit **sans question** ; Glucose relancé, la lecture répond **en 68 ms sans
  question** et le texte de « claude-code » est là ; **le binaire d'avant**, relancé sur la même
  maison, **repose la question**. (Ma première lecture a expiré à 90 s : c'était mon clic, venu six
  minutes après — les heures des fichiers le disent.)
* **RELAIS-2** : 5 épreuves devant une fausse porte (une prise Unix qui répond puis se ferme), 50
  fois de suite sans une chute ; **7 sabotages sur 7** (`relais-2-131`) — le septième passait d'abord
  (la branche « l'envoi échoue » n'était jamais jouée : le fil d'écoute voit toujours la fermeture
  avant) ; l'épreuve qui la joue directement est écrite. **Une branche n'est gardée que par le
  raisonnement** : la requête partie au moment exact où la porte se ferme (une course qu'aucune
  épreuve ne force sans crochet). **Au pilote** : un vrai `glucose --mcp` branché sur le vrai
  Glucose ; ce Glucose fermé (SIGTERM, comme son service) puis relancé dans le même compositeur — le
  relais vit, la demande suivante passe **525 ms** après la relance, sans erreur. Le relais d'avant,
  même essai : il répond, puis **meurt avec Glucose**.
* **PARTAGE-SURVIT** : 6 épreuves par la vraie application (enregistrer, partager, la croix,
  relancer ; le fil vers une adresse morte, jamais le réseau ni son presse-papiers), **10
  sabotages sur 10** (`partage-survit-131`). Le dixième passait d'abord : mon épreuve écrivait le
  geste arrivé après la fermeture, ce qui faisait naître un brouillon — le sabotage écrivait sous
  sa clé à lui. L'entrée juste : le geste appliqué sans être écrit, comme la boucle avant `finir`.
  Le branchement au lancement (`ouvrir_la_porte_des_agents`) ouvre une vraie porte : seul l'essai
  réel le garde. **L'essai réel, à travers la vraie salle en ligne** (deux Glucose hors écran,
  `~/.cache/glucose-bancs/acces-h24-131/essai_partage.sh`, captures dans `partage-apres/`) : Alice
  partage, Bob rejoint et voit « Alice avant » ; Alice fermée par le signal de fin retient le
  numéro 3 ; Bob pose « Bob pendant » ; **Alice relancée dit « repris au geste n° 3 de sa salle »
  et montre « Bob pendant »** ; son « Alice après » arrive chez Bob, avec son curseur. **Le binaire
  d'avant**, même essai : Alice relancée n'a que « Alice avant », et n'est plus dans la salle.
* La suite entière : 3 498 épreuves, et les 16 cliquets après correction.

## 6. Vu, pas réglé

* **ARBITRE-DÉMARRAGE (le point 6 du nœud 🔑)** : ce matin, l'arbitre a condamné la RTX ; son
  souvenir dit « économe », et le prochain lancement de son Glucose ouvrira la puce du Ryzen.
  **Ma première lecture était fausse, et la mesure l'a démentie** : je croyais que `present`
  attendait la place que l'écran à 60 Hz libère (13-14 ms au 99ᵉ centile ce matin), et que
  l'arbitre comptait ce temps comme perdu. **Mesuré sur son écran** (une fenêtre d'essai de 22 s,
  la caméra qui vole par le scénario, la RTX, 60 Hz, mailbox ; `arbitre-ecran/` dans le banc) :
  `present` **0,09 ms** en médiane, 0,36 au p90, 1,54 au p99, aucune image au-dessus de 10 ms ;
  l'arbitre garde la carte. Au pilote (sway sans écran) : 0,03 ms — la sortie sans écran ne retient
  rien, et ne peut pas servir. **Ce matin, tout était dix à vingt fois plus lent** (l'image au repos
  3,4 ms en médiane, contre 0,46 maintenant) : c'est le moment — le PC qui démarre — que l'arbitre
  a pris pour un défaut de la carte, comme la session interactive le pensait. **Pas réglé** : il
  faut une loi qui distingue une carte qui gèle d'une machine qui démarre, sans constante. Pistes
  à peser : (a) deux témoins — une carte n'est condamnée que si elle gèle dans deux sessions (le
  prix : une session de plus sur une vraie mauvaise carte) ; (b) juger `present` contre le travail
  de l'image elle-même (au démarrage, tout s'allonge ; une carte qui gèle n'allonge que `present`) ;
  (c) mesurer le prochain démarrage image par image avant de choisir. Je n'ai touché ni à la loi,
  ni à son souvenir.
* **Trois outils d'essai à son écran visaient son Glucose** (`essai_isole.sh` : `pkill -f
  "^$exe"` ; `essai_ecran.sh` et `console_disparue.sh` : le premier processus du binaire, le plus
  ancien — le sien) : depuis le 22/11, son Glucose (un service) et les relais des agents tournent
  ce même binaire. Corrigés : le Glucose d'essai se retrouve par son binaire **et** son dossier
  personnel (`le_glucose_de`).
* **Un geste vide de la salle salit le document** : il fait monter la version sans rien écrire, et
  la croix demande alors d'enregistrer un document nommé. Vu dans une épreuve ; un vrai geste
  s'écrit et laisse le document propre.
* **Après la fermeture du document par la croix**, un geste de la salle écrit avant la fin du
  programme ferait naître un brouillon (le document fermé n'a plus d'écriture). Une fenêtre de
  quelques images ; pas réglé.

## 7. Choix pris seul, à contester

* **« Toujours » l'emporte sur un « Refuser » donné plus tôt dans la session** à un autre nom : le
  plus récent de ses choix gagne (sa règle), et « Retirer l'accord des agents… » reste le seul
  retrait. « Autoriser » reste le choix mis en avant (la touche Entrée) : le plus prudent.
* **L'identité du document est son chemin** (fiche 80) : le même fichier ouvert par un autre chemin
  (un lien, un dossier renommé) redemande — l'échec qui se voit, jamais celui qui ouvre trop.
* **Une requête emportée par la fermeture reçoit une erreur, pas un renvoi** : la renvoyer pourrait
  écrire deux fois (les écritures d'agent sont idempotentes par identifiant, mais pas toutes).
