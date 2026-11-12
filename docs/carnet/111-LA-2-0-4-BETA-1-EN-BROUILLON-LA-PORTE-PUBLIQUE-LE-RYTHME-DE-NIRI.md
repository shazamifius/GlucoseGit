# 111 — La 2.0.4-beta.1 en brouillon, la porte publique, le rythme de niri

> Session du 12/11/2026, de 18 h 40 à 19 h 20, arrêtée à sa demande (*« la session commence
> malheureusement à être trop longue »*). Envoyé : `f9027fd` (la préparation de la publication,
> CI entière), puis les documents. Rien d'autre dans le programme.

## 1. La 2.0.4-beta.1, en brouillon

* **Vérifié avant de publier** : la comparaison des versions de Glucose (`mise_a_jour/version.rs`)
  suit semver, le noyau d'abord — 2.0.3-beta.10 → 2.0.4-beta.1 se propose ; le code de version
  Android (`android/app/build.gradle.kts`) monte : beta 10 = 20003210, 2.0.4-beta.1 = 20004201, son
  téléphone en beta 4 = 20003204 ; la boîte noire et la télémétrie n'ont pas bougé depuis la beta 10
  (`git diff aebb3f7`), et `verifier_serveur.py` répond que le serveur connaît tout ; la beta 10 a été
  publiée comme « latest », pas comme pré-version.
* **La version de travail** passe à **2.0.4-dev** (fiche 48 § 15.4 : au-dessus de la publiée) — le
  bureau, Android, le défaut de Gradle, le `Cargo.lock` ; l'épreuve
  `test_la_version_depasse_tout_ce_qui_est_publie` inscrit la 2.0.3-beta.10 et la 2.0.4-beta.1.
  Le flux « Publier » écrit lui-même la version publiée dans `Cargo.toml` (`sed`), le dépôt ne la
  porte jamais.
* **Les notes** (`outils/publication/notes.md`) : d'abord le prix de PERSIST-5 (mettre à jour toutes
  ses machines, l'APK par-dessus), puis les membranes en carte, la lueur de parenté, les corrections
  que Glucose garde (RECTIF-1), la Time Machine signée, les flèches, `Alt+Z`, la fluidité, et ce qui
  reste connu (dont « Couleur » sans effet sur une flèche).
* **La CI entière, verte** : dix tâches sur dix sur `f9027fd` (exécution 43273119497, environ
  18 minutes). **Le flux « Publier »** (exécution 43276302684) : vert, onze fichiers, **en
  brouillon**. Le clic « Publish release » est le sien : laisser « Set as the latest release »
  coché, **ne pas** cocher « pre-release » (la mise à jour lit `/releases/latest`, qui ignore les
  pré-versions).

## 2. Son essai de 18 h 16 (le chantier 2 commencé)

Son Glucose installé a été mis à jour à 18 h 15 (`c9b746f`) ; il l'a lancé à 18 h 16, **70 secondes**,
sur l'**ancienne** carte (`~/Documents/Glucose-essai-wikipedia-liens.glucose`, rouverte par la
reprise), au **pavé tactile** : 322 pincements, 772 déplacements à deux doigts — **pas une molette**,
alors que nos bancs ne mesurent que la molette et le clic du milieu. Sa sortie, sa chronique et son
journal technique sont copiés dans `~/.cache/glucose-bancs/rythme-111/`.

* 61 % des images au-dessus de 10 ms ; **32 % des images en mouvement ratent leur balayage** ;
  latence geste → écran : médiane 19,5 ms ; le tressaut : le contenu se pose à 77 px de sa
  trajectoire au p99.
* Sur les images ratées, le premier poste est **`acquerir`** (médiane 6,9 ms, p90 13,8) — l'attente
  d'une image libre de la chaîne —, devant le travail.

**Ce que le banc a montré** (`~/.cache/glucose-bancs/lag-107/m111-*`, son Glucose installé, la même
carte) : sous **sway sans écran** (tous nos bancs jusqu'ici), `acquerir` vaut 0,1 ms et 1,1 % des
images ratent ; sous **niri imbriqué** (`PILOTE_COMPOSITEUR=niri`), `acquerir` monte à 11-15 ms au
p90 et **31,1 % des images en mouvement ratent — chez lui, 32,1 %** —, avec seulement **4 ms** de
travail en médiane. **Le lag qu'il sent tient en bonne partie au rythme entre Glucose et niri, pas au
travail par image**, et nos mesures hors écran ne pouvaient pas le voir. *Probablement la cause
principale, à confirmer sur son vrai écran* : niri imbriqué produit 122 images par seconde (la
moitié jetées), sa vraie session 61.

* Deux réglages existants n'y font rien : trois images en vol (`GLUCOSE_IMAGES=3`, 28 % ratées) ;
  la présentation fifo (`GLUCOSE_PRESENT=fifo`, 92 % ratées, 51 images par seconde).
* **La cause probable** : le tempo (`tempo.rs`) a été réglé sur son ancien écran Windows à 240 Hz, où
  présenter ne bloque jamais. Sous Wayland, Glucose ne demande jamais à niri quand dessiner : il ne
  passe jamais par `Window::pre_present_notify` (winit 0.30.13 : *« Wayland: Schedules a frame
  callback to throttle RedrawRequested »*), dessine sur ses propres minuteries, attend son tempo,
  puis attend encore dans `acquerir` que niri rende un tampon.
* **Le prototype, écrit, pas compilé** (la session s'est arrêtée pendant la compilation) :
  `~/.cache/glucose-bancs/rythme-111/rythme-du-compositeur.patch` — sous Wayland, appeler
  `pre_present_notify` juste avant de présenter, et laisser le tempo de côté ;
  `GLUCOSE_RYTHME_TEMPO=1` rend l'ancien comportement au **même** programme, pour des paires
  entrelacées. Le banc aux coordonnées de la fenêtre de niri (960 px de large) :
  `lag-107/mesure-niri.sh` ; le lecteur du rythme de chaque passage : `lag-107/rythme.sh`.
* niri 26.04 offre `wp_presentation` (l'instant exact où chaque image a été montrée) et un pointeur
  virtuel (`zwlr_virtual_pointer_manager_v1`) : une mesure sur son vrai écran est possible, une
  fenêtre et sa souris pilotée une minute, à annoncer.

## 3. La porte publique — la page de conception, à sa parole

<https://claude.ai/artifact/36hpNTPEH2mCEENhKjyvzF> (source : `~/.cache/glucose-bancs/porte-111/`).
Une IA publique (Claude.ai, ChatGPT) ne parle qu'à une adresse `https://` et demande une
autorisation ; son ordinateur n'a pas d'adresse publique. La proposition :

* **A2, un relais sur son compte Cloudflare** (un Worker et un Durable Object par Glucose, gratuits,
  là où vit la télémétrie) : une adresse fixe par installation, que Glucose joint **de lui-même**
  par une connexion sortante (WebSocket, mise en sommeil — une connexion qui attend ne coûte rien) ;
  aucun port ouvert. Contre un tunnel (adresse qui change, programme en plus) et un relais par
  utilisateur (exclut les gens ordinaires). **Le prix** : le contenu lu traverse le relais (jamais
  gardé) ; les 100 000 requêtes par jour gratuites sont partagées par tous les utilisateurs de
  Glucose ; un fork peut avoir le sien.
* **L'accord dans Glucose**, sans mot de passe ni compte : la page d'autorisation dit « va dans
  Glucose », Glucose demande « Claude (claude.ai) demande l'accès à ce canevas — code 4 7 2 »
  (*Lire et écrire*, *Lire seulement*, *Refuser*) ; ce clic est la preuve. Le nom vient du document
  d'identité du client (CIMD), donc vérifié par son domaine.
* **Mes avis** (B, C) : l'accord gardé jusqu'à ce qu'il le retire (redemandé à chaque lancement en
  option), un témoin quand une IA publique agit ; elle ne voit que le document ouvert (pas les
  chemins de ses fichiers). Le risque qu'on ne supprime pas : l'injection d'instructions par un
  texte du canevas — borné par l'annulation, la signature, *Lire seulement*.
* **Le co-working sur le même relais** : une salle (Durable Object) par canevas partagé, un arbitre
  qui ordonne les gestes et où le dernier changement d'une propriété gagne (Figma, tldraw), chaque
  geste signé (MAIN-1) — une IA n'est qu'un membre de plus. À concevoir : les photos (5 Go pour
  tout le monde dans l'offre gratuite), l'invitation, et **les gestes du journal désignent un nœud
  par son rang dans une liste** (`Slot`, JRN-2) : partagés, ils devront le désigner par son
  identifiant.
* **Ses quatre questions** : A2 ? B et C ? quelle IA d'abord, avec quel compte ? la porte avant le
  co-working ?
* **Remarqué** : le protocole « moderne » (2026-07-28) est sans état — un relais n'a rien à retenir ;
  un client ancien (Claude.ai parle probablement 2025-06-18 ou 2025-11-25) s'y traduit, le relais
  donnant au geste le nom tiré du jeton, comme `glucose --mcp` rejoue la poignée de main
  (RELAIS-1). Glucose a déjà son SHA-256 sans dépendance (`core/hash.rs`) : l'identité d'une
  installation peut être l'empreinte d'un secret, sans rien stocker côté relais. Sous Windows,
  WinHTTP sait le WebSocket (déjà dans les fonctionnalités de la caisse `windows`) ; ailleurs,
  `rustls` est déjà là par `ureq`.

## 4. Trouvé en chemin, pas réglé

* **« Couleur » sans effet sur une flèche** : la palette (`commandes/palette.rs`, `trouvees_pour`)
  propose **toutes** les commandes sans regarder ce qui est choisi, et une commande qui ne
  s'applique pas rend `false` en silence (`commandes/executer.rs`) ; une flèche ne se colore pas
  (`Annotation::se_colore`). La fiche 05 § 5.4 : une action qui n'existe pas est **absente ou
  grisée**. Une question n'a pas de réponse grisée (`ui/question.rs`) : soit filtrer pour
  `But::Lancer`, soit apprendre le grisé aux questions — par une seule loi « s'applique-t-elle ? »
  que l'exécution et la palette partagent.
* **Le bleu des liens des textes** (`theme.rs`, `link: #60a5fa`) : APCA 52 sur le papier `#0d0d0d`
  (le corps `#dce1eb` : 88, recalculé). Un lien est souligné, il se distingue donc ; il doit se
  **lire** comme le corps : Lc 75 (minimum d'un texte courant) = `#a8cfff`, Lc 60 = `#7ab5ff` (même
  teinte en Oklch, h = 254,6). À trancher sur une image ; l'épreuve du thème fixe aujourd'hui
  `#60a5fa`.
* `automerge` est une dépendance d'épreuve (l'oracle du lecteur des documents de Tauri, qui étaient
  en Automerge depuis juillet 2026, `decisions/03`), jamais de l'application.
* Le segment « profond-souris » du banc ne rapporte rien sous sway : la souris au fond d'« Eau » ne
  demande aucune image (rien n'y change) — probablement juste, pas un défaut ; sous niri il en
  rapporte.

## 5. La recherche (consultée le 12/11/2026)

* MCP, la spécification du 28/07/2026 : [les changements](https://modelcontextprotocol.io/specification/2026-07-28/changelog)
  (sans état, `server/discover`, l'inscription dynamique dépréciée au profit des documents
  d'identité), [l'autorisation](https://modelcontextprotocol.io/specification/2026-07-28/basic/authorization)
  (OAuth 2.1, PKCE S256, RFC 8707, RFC 9728, `iss` de RFC 9207).
* Anthropic : [l'authentification des connecteurs](https://claude.com/docs/connectors/building/authentication)
  (sans autorisation, OAuth DCR ou CIMD — CIMD seulement si les métadonnées annoncent
  `client_id_metadata_document_supported` et `none` ; retour `https://claude.ai/api/mcp/auth_callback` ;
  Claude Code en boucle locale sur un port quelconque ; 10 s par point d'accès) ;
  [les connecteurs personnalisés](https://support.claude.com/en/articles/11175166-getting-started-with-custom-connectors-using-remote-mcp)
  (tous les comptes, **un seul en gratuit**, servis depuis le nuage même au téléphone).
* OpenAI : la page du mode développeur refuse les robots ; les sources secondaires se contredisent
  sur les formules ([coworker.ai](https://coworker.ai/blog/chatgpt-mcp),
  [le forum](https://community.openai.com/t/mcp-server-tools-now-in-chatgpt-developer-mode/1357233)) —
  à vérifier sur son compte.
* Cloudflare : [les limites des Workers](https://developers.cloudflare.com/workers/platform/limits/)
  (100 000 requêtes par jour, 10 ms de calcul), [les Durable Objects](https://developers.cloudflare.com/durable-objects/platform/pricing/)
  (100 000 requêtes et 13 000 Go-s par jour ; un message WebSocket entrant compte 1/20 ; la mise en
  sommeil ne facture pas la durée), [l'offre gratuite](https://blog.cloudflare.com/building-ai-agents-with-mcp-authn-authz-and-durable-objects/).
* Le co-working : Evan Wallace, [How Figma's multiplayer technology works](https://madebyevan.com/figma/how-figmas-multiplayer-technology-works/)
  (2019) ; [tldraw sync](https://tldraw.dev/docs/sync) et
  [tldraw-sync-cloudflare](https://github.com/tldraw/tldraw-sync-cloudflare) (une salle par Durable
  Object, ~50 collaborateurs) ; [les relais publics d'iroh](https://docs.iroh.computer/iroh-services/relays/public) ;
  Loro, Automerge, Yrs : aucun banc public sur une charge de canevas.
* Le rythme : winit 0.30.13, `Window::pre_present_notify` et `FrameCallbackState`
  (`platform_impl/linux/wayland/window/state.rs`) ; Mesa a la présentation fifo par
  `commit-timing` et `fifo` depuis 24.3 ([Phoronix](https://phoronix.com/news/Mesa-24.3-Vulkan-FIFO)),
  protocoles que niri 26.04 n'offre pas (`wayland-info`).

## 6. Les leçons

* **Un banc hors écran ne voit pas le compositeur** : sous sway sans écran, présenter ne bloque
  jamais ; sous niri, si. Mesurer sous le compositeur de l'utilisateur (`PILOTE_COMPOSITEUR=niri`),
  et lire sa vraie session poste par poste avant de croire le banc.
* **Mesurer le geste qu'il fait** : il zoome au pavé, nos bancs à la molette.
* **Un réglage fait pour une machine ne vaut pas pour une autre** : le tempo de son écran à 240 Hz
  sous Windows, transporté tel quel sous niri à 60 Hz.
