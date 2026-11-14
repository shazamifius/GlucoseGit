# 116 — La porte publique : Claude.ai dans le document ouvert, par Collaborer ; le relais en ligne ; ce que le co-working demande au journal

> Session du 14/11/2026, de 1 h 46 à 4 h 33, arrêtée à sa demande (*« la session commence
> malheureusement à être trop longue »*) : la porte publique jusqu'à 3 h 45, puis les fondations du
> co-working. Sa parole d'ouverture : *« j'aimerais vraiment
> travailler sur les protocoles MCP de liaison de IA ainsi que le système de collaboration »* —
> la porte publique (fiche 111) et le co-working (fiche 113), dont les pages attendaient sa parole
> sans commentaire. Ses mots valent feu vert sur mes avis des deux pages (le relais sur son
> compte, A2 ; son accord gardé jusqu'à ce qu'il le retire, B1 ; le document ouvert seulement,
> C1 ; Claude d'abord ; la porte avant le co-working), **avec une remise en question** (§ 2).

## 1. L'ouverture

* **Sa session de 1 h 45** (sa chronique et sa sortie, copiées) : 2 min 20 sur un petit document,
  fenêtre 936 × 996, son VivoBook. **La première vraie session lue par PARU-1** : 3 285 images
  confiées, 3 165 montrées, 120 jetées (3,7 %) ; **19,1 % des battements sautés** alors qu'une image
  était due ; **de l'envoi à l'écran, 46 ms en médiane, 93 au p90, 110 au p99** ; « écouter la
  main » prend jusqu'à 31 ms d'un coup (107 à 112 s). La mesure qu'attendait le chantier de son lag
  (SUITE § 2) — non traitée ce soir.
* **La télémétrie** : 176 sessions, 10 installations, 36 fins anormales (170 et 34 le 13/11). Le
  Linux à RTX 4070 SUPER est passé en 2.0.4-beta.1 le 13/11 ; son téléphone, toujours en beta 8.

## 2. Remis en question

* **Le relais ne fait que passer** (contre la page de la fiche 111, qui y mettait l'autorisation).
  Chaque Glucose est **son propre serveur d'autorisation** : il donne les clés, les vérifie, les
  retire — à la seconde, chez lui. Trois raisons : le relais ne garde ni ne fabrique aucune clé ;
  une nouvelle version de MCP (deux cette année) ne demande jamais de le redéployer ; toute la
  logique s'éprouve et se sabote en Rust. Il ne sert qu'**une** page à lui (« va dans Glucose »,
  le code) et refuse de servir quoi que ce soit d'autre que du JSON au nom d'un Glucose — personne
  ne fait de son domaine une page de hameçonnage.
* **Deux outils touchaient le disque**, que la page n'avait pas vus : `glucose_poser_page` lisait
  n'importe quelle image par son chemin (`capture`), `glucose_domaines` lit les autres fichiers et
  dit leurs chemins. D'où **la portée** (`outils::Portee`) : une IA du nuage n'a que le document
  ouvert ; **la liste des outils ouverts au nuage permet, elle n'interdit pas** (un outil neuf reste
  à la machine tant qu'on ne l'y ajoute pas exprès), de même pour « Lire seulement ».
* **Le bouton Collaborer existait**, et mentait (« pas encore disponible ») — la dette nommée dans
  le cliquet des messages. La page disait que le co-working passerait par la même porte : c'est
  donc lui qui l'ouvre. Il s'allume quand elle l'est.
* **La spécification MCP du 28/07/2026 a durci le transport** depuis la fiche 111 : `Mcp-Method`
  et `Mcp-Name` en en-têtes, vérifiés contre le corps (`HeaderMismatch`, -32020) ; `GET` et `DELETE`
  rendent 405 ; aucune session. Glucose sert aussi un client de l'époque ancienne, sans session.

## 3. PUBLIQUE-1 — la porte, telle qu'écrite

```text
IA du nuage ──https──► relais (outils/porte, Cloudflare) ──WebSocket sortant──► lien (un fil)
                                                                                  │ http (pur)
                                       guichet (OAuth) ◄──────────────────────────┤
                                       boîte aux lettres ◄── un appel d'outil ────┘
                                               │
                        boucle de Glucose : l'outil, en un geste du journal, signé de son nom
```

* **`agents/publique/`** : `accords.rs` (le secret, la porte ouverte ou non, les IA qui l'ont — par
  l'empreinte de leur clé seulement, `porte-publique.json`, lisible de l'utilisateur seul) ;
  `oauth.rs` (le guichet, pur : CIMD — le document d'identité lu sur le web, son `client_id` et son
  domaine vérifiés —, DCR sans rien garder — l'identifiant **est** l'inscription —, PKCE S256, un
  code à usage unique de dix minutes, une clé **sans expiration ni rafraîchissement** : un Glucose
  fermé ne pourrait pas en rafraîchir une, et B1 la garde jusqu'au retrait) ; `http.rs` (chaque
  requête : la découverte RFC 9728 et 8414, la page, l'échange, la porte MCP ; `401` et
  `resource_metadata` sans clé, ce qui lance l'autorisation chez Claude) ; `lien.rs` (le fil : un
  `ping` toutes les 20 s, mort sans `pong` en 10, l'attente qui double d'une seconde à une minute,
  au plus seize requêtes à la fois).
* **Le nom qui signe est celui de la clé** (« Claude (claude.ai) ») : ni le `_meta` d'une requête
  moderne, ni la poignée de main ancienne ne le remplacent (`Connexion::publique`).
* **Dans la boucle** (`app/porte_publique.rs`) : Collaborer → la question de la porte (fermée : ce
  qu'elle ferait ; ouverte : son état, **son adresse**, qui l'a ; copier, voir les IA, fermer) ; la
  question d'accord — « « Claude », depuis claude.ai, demande l'accès au document ouvert… Code :
  9 5 3 — le même que sur la page de ton IA ? » —, *Lire et écrire*, *Lire seulement*, *Refuser*
  (Échap refuse) ; retirer une IA ; **le témoin** : « Claude (claude.ai) travaille dans ton
  canevas », une fois par reprise après une minute de silence (le site du message qui mentait).
* **Ouverte, elle se rouvre au lancement**, chez le seul Glucose qui tient la porte locale — deux
  Glucose du même compte se prendraient l'adresse l'un à l'autre.
* **L'adresse ne change jamais** : `https://glucose-porte.ferme-nilslamber.workers.dev/g/<32
  chiffres>/mcp`, les chiffres étant le début du SHA-256 du secret ; `GLUCOSE_RELAIS` nomme un autre
  relais (un fork, une école, une épreuve).

## 4. Le WebSocket ([`decisions/16`](decisions/16-LE-WEBSOCKET-DE-LA-PORTE-PUBLIQUE.md))

`ureq` ne passe pas en WebSocket. Écrits ici : les trames et la poignée de main (RFC 6455, pur),
SHA-1 pour l'acceptation et le base64 dans les deux sens (le noyau ; le décodeur de Tauri y a
déménagé : une seule loi), le hasard du système (`/dev/urandom`, `BCryptGenRandom`). **Hors de
Windows**, `rustls` et `webpki-roots`, déjà là par `ureq` (le verrou ne gagne que deux liens) ;
**sous Windows**, WinHTTP, qui prend le proxy et les certificats du système (le PC de l'école) —
relu contre les signatures exactes de la caisse `windows`, puis **éprouvé sur la CI de Windows** :
l'épreuve de la vraie connexion contre un relais local y passe par WinHTTP.

## 5. Le relais, en ligne

`outils/porte` : un Worker et un Durable Object par Glucose (SQLite, l'offre gratuite) ; **aucun
stockage écrit, aucun journal** (`observability` coupé : il porterait l'adresse IP de chaque IA).
Le `ping` de Glucose reçoit son `pong` sans réveiller l'objet (`setWebSocketAutoResponse`) : une
porte ouverte toute la journée ne coûte rien. **Déployé le 14/11 à 3 h 15**
(`npx wrangler deploy`, son compte, celui de la télémétrie), vérifié de l'extérieur par
`essai/faux_glucose.mjs` : une demande passe avec sa clé, le cookie du navigateur ne passe pas, une
page HTML est refusée (502), un second Glucose prend la porte du premier (4000), un Glucose parti
rend 503.

## 6. L'essai réel

`outils/porte/essai/essai_pilote.sh` : le vrai Glucose hors écran, la porte ouverte **par la
palette** (`Ctrl+Maj+P`, « collaborer »), puis `faux_claude.mjs` qui fait tout ce que fait
claude.ai — la découverte, la page, la demande (**Glucose a lu le vrai document d'identité de
claude.ai sur le web** : « Claude », claude.ai), l'échange, les outils —, chaque étape capturée et
regardée. Contre le relais local (`wrangler dev`), puis **en ligne, chiffré** : la porte reliée en
`wss://`, le chemin entier en 4 s, chaque outil en moins d'une seconde ; le texte posé, signé
« agent Claude (claude.ai) » dans l'histoire ; la capture refusée, `glucose_domaines` fermé ; le
témoin à l'écran, Collaborer allumé.

**Ce que l'essai a trouvé et que les épreuves ne voyaient pas** : `glucose_lire` rendait le
**chemin complet du document** (`/home/…/brouillons/….glucose`). Une IA du nuage lit désormais
« le document ouvert » ; l'épreuve de bout en bout le garde.

## 7. Les épreuves et les sabotages

* **29 épreuves neuves en Rust, 4 en JavaScript** — **3 258 en tout, toutes vertes** (la suite entière,
  bornée, le 14/11 à 3 h 41) : 7 du WebSocket (dont une vraie connexion contre un relais écrit dans
  l'épreuve, qui exerce WinHTTP sur la CI de Windows), 16 de la porte (tout le chemin d'un accord
  sans réseau), **une de bout en bout dans l'application** (`publique_1_…` : un faux relais, la
  porte ouverte, une IA inscrite, la question et son code, la clé, un geste signé et le témoin, la
  capture d'une vraie image refusée, le domaine d'un fichier voisin tu, le chemin tu, retirée, fermée),
  SHA-1, base64, le hasard ; le bouton Collaborer par la souris (réécrite) ; 4 du relais.
* **La suite entière** : 3 253 vertes et 4 tombées au premier passage — toutes justes : le cliquet
  qui veut qu'un fichier se remplace par `persist::atomic` seulement ; celui des 80 lignes
  (`agent_poser_page` : la garde est allée dans `lire_une_capture`, une règle à la sortie) ;
  l'épreuve des polices (un émoji dans mon épreuve, qu'Inter n'a pas) ; mon épreuve du bouton (le
  mot d'accueil).
* **La vérification de GitHub** (dix tâches) : la première est tombée sur **clippy de Rust 1.100**
  (Nix, ici, a 1.99) — une boucle indexée d'une **ancienne** épreuve des familles (fiche 106), où il
  s'arrêtait avant `glucose-desktop` : la voie WinHTTP n'avait pas été vérifiée. Corrigée, la
  seconde est **verte sur les dix** (`2e63713`), Windows x64 compris.
* **Les sabotages** (`outils/sabotages/porte-publique-116.json`) : **32 sur 32**, après deux
  passes. La première a trouvé une épreuve aveugle — le contrôle « le document d'identité dit sa
  propre adresse » était couvert par celui du domaine de retour ; le cas de l'imposteur (un
  document qui renvoie chez lui mais se dit celui de claude.ai) est écrit. En préparant le fichier,
  quatre autres gardes se sont montrées sans épreuve (un retour hors du domaine du document, une
  ressource étrangère, une autre clé valide, la capture — mon épreuve passait une image illisible,
  refusée de toute façon) : écrites avant de saboter.

## 8. Le co-working — ce que la lecture du journal a appris

Le cœur existe (fiche 113) : chaque geste porte l'avant et l'après de ce qu'il change, se signe,
s'écrit sur le disque tel quel (`persist/histoire/geste.rs`). **Mais il désigne un élément par son
rang dans une liste** (`Slot::index`, JRN-2). Rejoué dans le même ordre partout, c'est juste — le
greffier aveugle (A3) donne cet ordre. Le défaut vient du geste qu'on applique **tout de suite chez
soi** puis qu'on rejoue derrière ceux des autres (le « rebase », ou le retour d'une session hors
ligne) : ses rangs sont alors faux. Avant toute salle, donc :

1. **les gestes désignent un élément par son nom**, le rang n'étant qu'un indice (PERSIST-4 : une
   édition neuve, aucun numéro qui bouge ; les documents d'avant se relisent) ;
2. **les numéros des nœuds par participant** (`store/ids.rs` : un compteur ; deux Glucose donneraient
   `txt-43` au même instant) ;
3. **le chiffrement** des gestes que la salle garde sans les lire : ChaCha20-Poly1305 (RFC 8439),
   écrit dans le noyau sans dépendance et éprouvé sur les vecteurs de la norme — la même voie sur
   toutes les machines, l'iPad compris ;
4. puis la salle (un Durable Object de plus sur le même relais), la question de la porte qui
   devient « Partager ce canevas / Rejoindre », les curseurs.

**Ce que la lecture de `journal/edit.rs` a appris ensuite, et qui allège le premier point** : chaque
`Slot` porte **l'élément entier** avant et après — et chaque élément (photo, texte, dossier, case,
storyboard, tableau, domaine, preset) porte son identifiant ; les éditions en bloc (`Whole`) visent
leur tableau par son identifiant. **Le geste connaît donc déjà le nom de ce qu'il touche** :
« désigner par le nom » n'est pas un nouveau format de journal ni de disque, mais **une seconde
façon d'appliquer le même geste** — trouver l'élément par son nom, le rang n'étant qu'un indice pour
une insertion ; l'avant gardé, comparé à ce qui est là, dit le conflit. Deux exceptions seulement :

* **`Translation`** ne garde que des rangs (`Vec<u32>`, pour déplacer 750 000 nœuds en quelques
  millisecondes, JRN-4) : nommée au moment où le geste part, pas dans le journal ;
* **`ActiveBoard`** (l'onglet regardé) est un geste du journal ; en co-working, chacun regarde le sien :
  il ne part pas aux autres.

**Écrit dans la foulée — PARTAGE-1** (`core/store/journal/partage.rs`) : `Transaction::a_partager`
(le geste tel qu'il part : le déplacement nommé dans l'état qui le suit, l'onglet regardé laissé à
soi), `GestePartage::appliquer` (par le nom ; l'insertion au rang d'origine s'il tient ; l'avant
comparé à ce qui est là : **le conflit**, et le dernier gagne ; un élément retiré est **absent**, sa
modification ne le ressuscite pas ; l'ordre des onglets range ceux qu'il nomme, puis ceux qu'un autre
a ajoutés) et `GestePartage::inverse` (pour défaire son geste en attente). **Six épreuves** où Alice
et Bob font chacun leur geste chez eux puis rejouent l'ordre de la salle (le rebase) : ils
convergent dans les deux ordres — et la preuve à l'envers, l'ancienne loi par les rangs qui écrit le
geste d'Alice sur le c de Bob. **9 sabotages sur 9** (`outils/sabotages/partage-116.json`). Rien du
journal ni du disque n'a changé.

**Puis CHIFFRE-1** (`core/chiffre.rs`) : ChaCha20-Poly1305 (RFC 8439) — le chiffre de TLS 1.3 et de
WireGuard, rapide et sans fuite par le temps sans aucune instruction du processeur : la même voie
partout. `sceller` et `ouvrir`, le sceau comparé en entier. **Éprouvé sur les vecteurs de la norme,
recopiés du texte de la RFC par un script** (rien de mémoire) : l'exemple § 2.8.2 scellé au bit près,
l'annexe A.5 ouverte (265 octets), **les sept cas limites de Poly1305** (A.3, 5 à 11 : la réduction
finale, le report de `s`, les membres pleins — là où se trompent les implémentations faites à la main)
— tous passés du premier coup ; un seul bit changé n'ouvre rien. **10 sabotages sur 10**
(`outils/sabotages/chiffre-116.json`).

**Reste**, dans l'ordre — avec deux décisions que la lecture du journal impose, à ne pas découvrir
trop tard :

1. **Un geste reçu entre au journal avec ses rangs réels** — **écrit** : `appliquer` rend la
   transaction **réalisée** (les rangs de l'instant, l'avant de ce qui était vraiment là), que le
   journal recevra comme les autres, signée de son auteur (MAIN-1). Éprouvé : rejouée par les rangs
   sur l'état d'avant, elle redonne le même canevas ; défaite, elle rend l'état d'avant — la valeur
   d'un autre comprise en cas de conflit. JRN-2 reste vrai, l'histoire du disque (JRN-5) se rejoue
   dans l'ordre. **11 sabotages sur 11** (une épreuve aveugle trouvée : un rang d'insertion qui ne
   changeait jamais dans mes cas).
2. **`Ctrl+Z` à plusieurs défait *mon* dernier geste**, pas le dernier de la pile (qui peut être celui
   d'un autre) : par son inverse **par le nom** (`GestePartage::inverse`), qui part à son tour comme un
   geste — Figma, Google Slides ; si l'élément a disparu entre-temps, rien (fiche 113). La pile LIFO du
   journal ne le sait pas : en partage, c'est une pile par main.
3. **Les numéros des nœuds par participant** — **écrit** (ID-2, `store/ids.rs`) : sa marque entre
   dans chaque nom qu'il crée (`txt-a7-44`, `txt-k3f9-44`) ; le numéro reste le dernier morceau,
   qu'ID-1 relit au chargement ; seul, aucune marque. **3 sabotages sur 3**. Et le geste partagé
   **s'écrit en octets** (`persist/histoire/geste.rs` : `ecrire_partage`, `lire_partage` — la forme
   des gestes de la Time Machine, un déplacement par ses noms), relu à l'identique (2 sabotages).
4. **La salle** — **écrite, éprouvée en local, pas déployée** (SALLE-1, `outils/porte/src/salle.js`,
   `messages.js`) : un Durable Object par canevas partagé (`/s/<id>/salle`), qui numérote, garde (SQLite)
   et passe des gestes scellés sans les lire ; la base est le premier geste, **en morceaux** (une valeur
   tient 2 Mo ; sa carte de Wikipédia en pèse 2,1) ; un retardataire rattrape `depuis n` ; les curseurs
   passent sans être gardés. **Écrire se prouve** : le code « écrire » porte un secret dont la salle ne
   garde que l'empreinte, fixée par le premier écrivain, **recalculée par le relais, jamais crue du
   client** ; le code « lire » n'a que la clé du chiffre. `essai/salle_essai.mjs` : neuf verdicts (un
   écrivain, un lecteur qui ne peut pas écrire, deux retardataires, un faux secret, une empreinte
   usurpée) ; **7 sabotages sur 7** (`outils/sabotages/salle-116.json`). Elle se déploiera **avant** la
   version de Glucose qui s'en servira.
5. « Partager ce canevas / Rejoindre » dans la question de Collaborer, son nom et sa couleur, les
   curseurs (hors de l'histoire), les photos d'un participant à l'autre (B1).

## 9. Ce qui reste contestable

* **Une clé sans expiration.** OAuth 2.1 préfère des clés courtes et rafraîchies ; mais Claude les
  rafraîchit cinq minutes avant leur fin, et un Glucose fermé ne répondrait pas — on ne sait pas
  ce que Claude ferait d'un rafraîchissement manqué. La clé vaut jusqu'au retrait (B1) ; à revoir si
  Claude se comporte bien avec un `503` au rafraîchissement.
* **Le témoin** ne dit que la reprise d'une IA après une minute de silence, en bas de l'écran. Il
  pourrait vivre dans la bande (Collaborer qui pulse) — à juger à son écran.
* **Le texte posé par une IA tombe à droite du tableau**, hors de la vue : le témoin le dit, rien ne
  le montre. Un vol vers lui serait-il bienvenu, ou volerait-il sa vue ?
* **La page d'accord n'a pas été vue dans un navigateur** : son script ne l'a été que par le faux
  Claude, qui en refait les appels. L'essai avec son compte Claude.ai la montrera.
* **La question de la porte ne se met pas à jour** pendant qu'elle est ouverte (« elle se relie… »
  reste jusqu'à la suivante).

## 10. La recherche (consultée le 14/11/2026)

* MCP, 28/07/2026 : [les transports](https://modelcontextprotocol.io/specification/2026-07-28/basic/transports),
  [Streamable HTTP](https://modelcontextprotocol.io/specification/2026-07-28/basic/transports/streamable-http)
  (en-têtes miroirs, `HeaderMismatch`, 405, aucune session, la compatibilité ancienne).
* Anthropic : [l'authentification des connecteurs](https://claude.com/docs/connectors/building/authentication)
  (`401` obligatoire, le premier `authorization_servers` seul, CIMD si `client_id_metadata_document_supported`
  et `none`, le retour `https://claude.ai/api/mcp/auth_callback`, 10 s par point d'accès, la sortie
  `160.79.104.0/21`) ; [l'authentification paresseuse](https://claude.com/docs/connectors/building/lazy-authentication)
  (la forme du `401`, la vérification d'un CIMD, la boucle locale au port près). **Les deux documents
  d'identité, lus** : `https://claude.ai/oauth/mcp-oauth-client-metadata` (« Claude », un seul
  retour) et `https://claude.ai/oauth/claude-code-client-metadata`.
* ChatGPT : le compte gratuit ne branche toujours aucun serveur MCP ; Pro en lecture, Business et
  Enterprise en plein (sources secondaires, qui se contredisent :
  [designrevision](https://designrevision.com/blog/add-mcp-server-to-chatgpt),
  [caipi](https://caipi.ai/blog/which-plans-support-mcp-2026)).
* Cloudflare : [les WebSockets des Durable Objects](https://developers.cloudflare.com/durable-objects/api/websockets)
  (32 Mio par message reçu depuis octobre 2025, la réponse automatique sans réveil, 32 768 connexions
  par objet) ; [les notes de version](https://developers.cloudflare.com/durable-objects/release-notes/index.md).
* La RFC 6455 (WebSocket), 7636 (PKCE), 7591 (DCR), 8252 (boucle locale), 8414, 8707, 9207, 9728 ;
  SHA-1 : RFC 3174 ; base64 : RFC 4648.

## 11. Les leçons

* **Un clippy plus récent ailleurs** : la CI prend la dernière version de Rust, Nix la sienne. Une
  vérification qui tombe au noyau ne dit rien du reste — elle s'est arrêtée avant.

* **L'essai réel trouve ce que l'épreuve ne cherche pas** : seize épreuves de la porte, et le chemin
  du document est parti au nuage dans le premier essai réel. Une épreuve vérifie ce qu'on a pensé
  à protéger ; l'essai montre ce qu'on n'a pas pensé.
* **Écrire le fichier des sabotages avant de les lancer** a montré quatre gardes sans épreuve : se
  demander « quelle épreuve tombe si je casse ceci ? » est déjà un sabotage.
* **Deux gardes qui se couvrent** rendent une épreuve aveugle : il faut le cas qui n'est arrêté que
  par l'une (l'imposteur).
* **Une garde se pose là où la chose arrive** : refuser la capture dans `lire_une_capture`, et non
  dans chaque outil qui l'appelle — le cliquet des 80 lignes l'a rappelé.
