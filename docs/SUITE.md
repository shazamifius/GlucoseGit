# La suite

> **Au 24/11/2026, 8 h** (fiches 108-139). **Ce qu'on fait ensuite, dans l'ordre** — une ligne par chantier ouvert ;
> le détail d'un chantier vit dans sa fiche du [carnet](carnet/00-INDEX.md), pas ici. Ce qui est fini
> sort de ce document (il est dans sa fiche et dans [ÉTAT](ETAT.md)). Ses paroles : [VISION](VISION.md).
>
> **S'en servir** : prendre le premier chantier ouvert de § 2, le **remettre en question** (ses paroles
> comprises), chercher sur internet avant de concevoir, le faire. Fini = **prouvé** (sur une copie, sur
> GitHub, ou à son écran), jamais « ça compile ». Le retirer d'ici, mettre ÉTAT à jour.

`[ ]` à faire · `[~]` en cours, ou fait et en attente de son écran ou de sa parole

---

## 1. Le cap — la partie 1 (sa parole du 25/10)

La partie 1 est finie, et s'appelle **2.1.0, la première version stable**, quand ces six choses sont
vraies (la ligne d'arrivée proposée le 25/10, à remettre en question) :

1. un utilisateur de Glucose Tauri n'y perd plus rien ([INVENTAIRE](INVENTAIRE.md) § 3 vide) ;
2. aucun tableau perdu ;
3. 100 images par seconde au PC, 60 sur un téléphone courant, **au pire** (le 99ᵉ centile de la
   télémétrie) — ou pixelisées sans que cela se voie ;
4. chaque fonction a son bouton, au PC et au téléphone ;
5. l'ergonomie : le preset de Glucose et l'interface du téléphone, jugés par lui ;
6. toutes les machines qu'on peut servir sans argent.

## 2. Maintenant, dans l'ordre — ses paroles du 12 et du 19/11

**Sa parole du 12/11 au soir** (VISION § 3) : des versions qu'il teste en conditions réelles, puis
**les boutons qui manquent — d'abord le co-working et les connexions MCP avec des IA publiques**. Les
deux sont faits ; ce qui suit les termine, puis reprend le reste du tableau.

| | chantier | fini quand |
|---|---|---|
| [~] | **SELECTION-MEMBRANES — choisir et écrire un texte posé dans des membranes** (sa note du 23/11 au soir, `text-kpll-356`, sa capture dans `membrane-299` ; VISION § 6 ; fiche 134 § 12). **Faits** : **MEMB-9** (`3f54241`) — l'outil membrane prend ce qu'il entoure, par la loi du dépôt (le centre, comme le dossier ; ses « Groupe » restaient vides), six sabotages sur six ; **PICK-3** — le bord d'une membrane ne se prend que là où il se voit : là où un texte, une note ou une photo le couvre, ils passent devant (son texte était à 27 unités du bord d'un « Groupe » : dézoomé, la bande du bord le couvrait presque entier), épreuve sur sa géométrie exacte avec l'envers. regardée au pilote (l'avant et l'après). **Reste** : ses cinq « Groupe » vides (sa réponse à `relais-rep-356`) ; le bord des dossiers, s'il gêne | à son écran : son texte se prend d'un clic et s'ouvre d'un double-clic, dézoomé |
| [~] | **ANCRE-1 — la flèche qu'on ne choisit plus** (sa note du 23/11 au soir, `text-kpll-375` ; VISION § 6 ; fiche 135) : **fait** (`af53d41`, `046a5ca`) — tout ce qui cherche une flèche lit son tracé ancré, jamais la copie de ses bouts ; 10 sabotages sur 10, regardé au pilote. **Reste** : son écran ; une flèche qui contourne, ou courbe, peut sortir de la boîte de ses nœuds (fiche 135 § 6) | à son écran : la flèche glissée avec un texte, ou rangée vers un dossier, se reprend d'un clic |
| [~] | **EDIT-PRISE — sélectionner une partie d'un texte « quitte le bloc »** (sa note `text-kpll-370` ; VISION § 6 ; fiche 136) : **fait** (`7ee455e`) — en écriture, tout appui dans la boîte que le texte montre est au texte (la partie qui a grandi, que l'arbitre ne connaissait pas — ou la source d'un texte mis en forme, plus longue que son rendu —, une poignée du bord, une flèche qui y arrive) ; 5 épreuves, 4 sabotages sur 4, l'avant et l'après au pilote. **Reste** : son écran ; hors saisie, un texte peut-il se montrer plus haut que sa boîte (fiche 136 § 4) | à son écran : glisser dans un texte qu'on écrit sélectionne, jusqu'en bas |
| [~] | **PIPETTE — prendre la couleur qu'on voit** (sa note du 23/11, `text-f5fn-368` ; VISION § 5 ; fiche 134 § 13) : **faits** — PIPETTE-1 (`8f1c989`) : viser la lueur d'un texte coloré prend sa couleur ; PIPETTE-2 (`57dedf5`) : armée, **la croix** au curseur, **la case « après » et le code de la roue montrent ce qu'elle prendrait**, « Aucune » quand il n'y a rien ; PIPETTE-3 : **un texte sans couleur choisie donne celle de sa famille** (celle que sa lueur montre), jamais le gris ; l'icône, une vraie pipette. 13 sabotages sur 13, regardés au pilote. **Reste** : son écran — et sa parole si la couleur de famille ne lui va pas | à son écran |
| [~] | **REJOINDRE-1 — rejoindre un canevas par un champ** (sa note du 23/11, `text-kpll-346`, sa capture du téléphone ; fiche 134 § 14) : **fait** (`a8ef6c4`) — « Rejoindre… » ouvre un champ où l'on colle ou tape le code, déjà rempli du code copié ; sous Android, Glucose ne lisait que son propre presse-papiers. 9 sabotages sur 9, regardé au pilote à la taille du téléphone. **Reste** : le clavier d'Android lui-même (par la CI et une version publiée) | à son téléphone : un code collé d'un message rejoint le canevas |
| [~] | **EDITION-LONGUE — écrire un long texte** (sa note `text-wiov-325` ; fiche 134 § 15) : (d) **la vue suit ce qu'on écrit** — SUIVRE-1 fait (`91b6674`), 6 sabotages sur 6, regardé au pilote ; (b) le texte qui disparaît zoomé et (c) `Ctrl+Z` qui efface tout : **pas reproduits** au pilote (à son écran). **Reste (a), TEXT-FIT-3** : un texte naît à 240 et ne grandit qu'en hauteur — chez Glucose Tauri, **« libre jusqu'à 600 px, ou fixé si redimensionné »** (fiche 06 § 5.1) : la largeur suit le texte jusqu'à 600, puis il passe à la ligne ; un texte redimensionné garde la sienne (un état « libre » à porter par le document). **La recherche** (23/11, compte a) : **tldraw** porte `autoSize` sur le texte — vrai par défaut, la largeur suit le texte et ne passe jamais à la ligne ; tirer un bord gauche ou droit le rend faux, et le texte garde cette largeur et passe à la ligne ; `w` n'est lu que faux, la hauteur se mesure toujours ([tldraw, *Text shape*](https://tldraw.dev/sdk-features/text-shape), [*Programmatic text shape creation*](https://tldraw.dev/examples/text-shape-configuration)) ; **Excalidraw** porte `autoResize` (d'abord nommé `wrapped`, PR #7999) : les poignées de côté passent à la ligne au lieu d'étirer, la largeur est gardée quand il est faux, et **le menu contextuel rend la taille automatique** ([commit 971b4d4](https://gitea.maison43.duckdns.org/gilles/excalidraw/src/commit/971b4d4ae61bad7fd1c73e7238bf3d56ef40f543)) ; créer en glissant donne d'emblée une largeur fixe ([PR #8038](https://github.com/excalidraw/excalidraw/pull/8038)). **Ce qu'on en tire** : un booléen dans le document, comme eux (vrai pour un texte neuf, faux dès qu'on tire un côté), et un geste pour revenir à « libre » (Excalidraw) — *rien à moitié* ; le plafond de 600 de Tauri n'existe ni chez tldraw ni chez Excalidraw, mais un paragraphe Markdown sans plafond tiendrait sur une ligne : c'est un nombre de ressenti, à dire comme tel. **Le format, vérifié** : il est positionnel (`persist/annotation.rs`, PERSIST-1) — un champ neuf monte le schéma, lu sous `version >= …_DEPUIS` comme `storyboard.rs`, et **les versions publiées refuseront les documents neufs** (« mets-la à jour », comme PERSIST-5) : à faire avec une publication, et à dire dans ses notes. « Libre » ne peut pas être `width: None` (le texte la porte déjà en `Option`) : le noyau ne mesure pas, son `rect()` a besoin de la largeur mesurée, qu'un texte libre doit donc garder comme sa hauteur (TEXT-FIT-1) — il faut un booléen à part | au pilote : une longue phrase s'élargit jusqu'à 600 puis descend ; saboté |
| [~] | **Le dessin de la sélection** (Q12 ; sa réponse finale du 23/11 : *« les carrés avec le flou, mais alors au moins 60 % moins »* ; fiche 137) : **fait** — **les équerres** (`927c809`), un seul langage pour toutes les sortes de nœuds, les poignées logées dans leurs angles, 10 sabotages sur 10 ; **le flou du reste** (`ffce09a`, `6348f20`, `e25ea44`) : une gaussienne exacte en pyramide (`glucose_core::flou`, la loi des variances de Vello, le nombre d'étages par la loi de Nyquist), σ = 5,6 px (40 % de l'image D), sur les deux voies (l'accord à 4 niveaux), la main au repos, l'image entière tant qu'il vit, « Ne plus flouter le reste » au menu de la sélection ; regardé au pilote (l'image E, posée dans son Glucose à côté de D). **Reste** : son écran (sa relance — voir ARBITRE-DÉMARRAGE) ; son ressenti sur la force du flou (un seul nombre) | à son écran : la sélection en équerres, le reste flouté, rien pendant un glisser |
| [~] | **RANGER — ranger des nœuds dans un dossier, l'archive** (ses notes du 23/11 au soir : *« archive les bugs une fois qu'ils sont corrigés »*, *« éviter de supprimer, mais plutôt archiver »* ; fiche 134 § 10) : **faits** — RANGER-1, la loi et `glucose_ranger` pour les IA (une flèche dont un seul bout part reste dehors et vise le dossier) ; RANGER-2, **glisser des nœuds sur un dossier** (il s'entoure pendant le glisser) ; un seul `Ctrl+Z` ; regardé au pilote ; 22 sabotages. **Reste** : (1) **son geste** : relancer son Glucose (recompilé sur `f7c7420`) et « Toujours, dans ce document » (⚡ Gestes) ; (2) **archiver** selon sa règle, par le relais : un bogue corrigé et publié, une question dont la fonction est publiée et vue par lui ; son dossier `folder-wiov-332` devient « archive » le 24/11 (son geste, ou le nôtre s'il le demande) ; (3) entrer dans un dossier qui ne montre rien de ce qu'il porte : le cadrer (fiche 134 § 8) | ses bogues corrigés rangés dans « archive » par le relais, annulable |
| [~] | **RELAIS-H24** (sa parole du 23/11 : *« ton rôle, c'est de réparer les sessions Claude, qui doivent avoir un lancement automatique et du code H24 dans les limites des sessions, et de jamais avoir 2 sessions de code en même temps »* ; fiches 132 et 134) : **réparé et relancé** — une seule session, sa consigne signée de n'importe quel appareil à lui, ses notes données à la session, jusqu'au mur faute de relais, la relance sous le plafond de Claude Code. **Le 23/11 au soir** (fiche 134) : deux gels réglés — **`--no-chrome`** (la question de Chrome au démarrage, 1 h 48) et **le bypass, jamais le mode auto** (sa réponse) ; le ❤️ dit « figée » ; **la mémoire du modèle à 90 %** demande une conversation neuve, la semaine à 90 % ; **ses textes** (départ, passation, relance) dans son Glucose, sa version envoyée dès qu'il l'a écrite. **Reste** : regarder une nuit entière ; **toute autre question de Claude Code au démarrage** fige encore une session (la leçon de la fiche 134 § 1 : une option à chaque lancement, pas une réponse gardée par un compte) | une nuit H24 sans un trou, une seule session à la fois |
| [~] | **ACCÈS-H24** (sa demande du 22/11 : *« rendre tout ça vraiment H24 »* ; le nœud 🔑 de son Glucose, membrane du relais ; fiche 131) : **faits** — les deux jauges du relais (passation en plein tour, session refusée fermée, plus lointaine remise), le ❤️ compté contre GitHub, **« Toujours, dans ce document »**, **`glucose --mcp` qui survit à Glucose**, **le partage qui rejoint sa salle au lancement** ; chacun saboté et regardé au pilote. **Reste** : (1) ~~son geste~~ — **fait le 23/11 à 21 h 47** : il a fermé Glucose et cliqué « Toujours, dans ce document » (`agents-toujours.txt`) ; le relais peut désormais relancer son Glucose pour une version neuve sans que l'accord se perde ; (2) **ARBITRE-DÉMARRAGE** (point 6) : **sur son poste, plus d'arbitre** — une seule carte, rien à arbitrer (ARBITRE-6, fiche 139), et l'attente de l'écran n'est plus un gel (ARBITRE-5, fiche 138). **Reste**, sur une machine à deux cartes seulement : le démarrage du PC (tout dix à vingt fois plus lent) pris pour un défaut ; une loi sans constante à choisir (deux témoins, ou `present` jugé contre le travail de l'image ; fiche 131 § 6), et ECRAN-1 sous Linux (fiche 139 § 1) ; (3) le partage : la reprise après un arrêt net (repartir de la salle, dans le même fichier) ; (4) DOSSIER-2 **fait** (`d86f52f`), sauf l'épreuve de Maj+clic sur un dossier (son sabotage passe) | son clic, puis une nuit H24 sans clic ; l'arbitre mesuré |
| [~] | **La discussion avec une IA gratuite** (sa demande du 21/11, VISION § 3 : *« de manière très rapide »* ; fiches 128-129) : **écrite, commitée (`48094df`), 25 sabotages sur 25** — Collaborer → « L'API publique… » (discuter, tester les modèles gratuits, la clé Google AI Studio), le tiroir DISCUSSION, les outils de la porte par la boîte des agents (DISCUSSION-1 et 2), **la bascule** qui dort le temps que Google dit et qui le dit (BASCULE-1 et 2), `plateforme::echanger` sur les deux voies. **Regardée au pilote avec sa clé** (fiche 130 § 6 : Gemini lit, pose un nœud signé de son nom, répond). **Reste** : (1) **son écran** ; (2) **la CI** — la voie WinHTTP ne se compile pas ici ; (3) **un second fournisseur** au format d'OpenAI (Groq, OpenRouter, Mistral, Cerebras : un seul branchement), à la place de GitHub Models, fermé le 30/07/2026 ; (4) la réponse au fil de l'eau (`streamGenerateContent` : le délai d'attente de deux minutes disparaît au profit de celui de la porte) ; (5) ce que la bascule apprend, gardé d'un lancement à l'autre — au pilote, le tiroir a nommé « Gemini 4 Hydrogen » puis « 3.6 Flash » a répondu sans un mot (fiche 130 § 6) : garder ce qu'elle sait efface la surprise ; (6) la discussion **sur le canevas** — un graphe de questions et de réponses (VISION § 1, objectif 3 ; tldraw *Branching chat*) | à son écran : une question qui pose un nœud, une bascule dite ; sous Windows, par une version publiée |
| [~] | **Le compte Cloudflare de Glucose** (sa parole du 19/11 : *« fermenislamber […] c'est un autre projet ; il faudrait vraiment switch ça »*) : le plan, fiche 123 § 7 ; **la question est dans son Glucose** (son clic *Create Account*, le nom — `glucose-canevas` proposé, les serveurs nommés `porte` et `boite-noire`). Ensuite, **le serveur avant le client** : la boîte noire et la porte sur le compte neuf, l'ancienne adresse de la boîte noire qui transmet (`~/.cache/glucose-bancs/session-123/outils/ancienne-adresse/`), la base recopiée (`INSERT OR IGNORE`), `ADRESSE` (`telemetrie.rs`), `RELAIS` (`agents/publique.rs`), `verifier_serveur.py` et les documents changés ; l'ancienne porte reste ouverte pour les versions installées ; l'identifiant du compte par `CLOUDFLARE_ACCOUNT_ID`, jamais dans un fichier | les deux serveurs prouvés en ligne sur le compte neuf ; une session de télémétrie d'une version neuve reçue ; ses connecteurs réajoutés |
| [ ] | **La mémoire d'une compilation** (fiche 127) : le `rustc` des épreuves du bureau monte à 2,4 Go, et la borne refuse de compiler dès que Brave et VS Code laissent moins de 3,58 Go — le 20/11 au soir, une heure d'attente. Le profil de développement garde **toutes** les informations de débogage (`[profile.dev]`, `debug` par défaut ; 45 Go de cible) quand personne ne lance de débogueur ici et que les traces de plantage n'ont besoin que des lignes. **Mesurer** la pointe du `rustc` du bureau avec `debug = "line-tables-only"`, dans une cible à part, `CARGO_INCREMENTAL=0`, `/usr/bin/time -v`, contre le profil d'aujourd'hui ; aucune mesure publiée n'existe (Kobzol, *Disable debuginfo to improve Rust compile times*, 2025, ne mesure que le temps). Si la pointe baisse nettement : le profil change, et `outils/borne.sh` sa part par compilation | la pointe mesurée ; la borne refait ses comptes |
| [ ] | **Son Glucose installé à la 2.0.4-beta.4** (publiée le 19/11 à 19 h 15, son clic) : `bash outils/mettre_a_jour.sh` — la compilation de Nix est hors de la borne, huit à la fois : **sous chien de garde, et seulement quand sa machine laisse de la mémoire** (le 19/11 au soir, Brave et VS Code n'en laissaient que 3 Go). **Jamais `gh release delete` sur une version sans l'avoir lue (`gh release view … --json isDraft`)** | son Glucose dit 2.0.4-beta.4 |
| [~] | **Les émojis en couleur** (EMOJI-2 et 3, fiches 125-126, `decisions/18`) : **faits, publiés dans la beta.4** — Twemoji 17 en COLRv0, et les 34 signes d'encre dans la palette du fond sombre ; sa parole sur les ✔ : *« non, c'est parfait comme ça ! »*. **Reste** : son écran | à son écran |
| [~] | **Les rangées d'un tableau coupé** (TABLE-4, fiche 125) : **fait** — un filet entre deux rangées, de l'encre du filet de l'en-tête à 70 % (Primer, `borderColor.muted`), sur les trois dessins ; 7 sabotages sur 7, regardé au pilote et sur la scène témoin. **Reste** : son écran | à son écran |
| [~] | **Les tiroirs qui se rangent et défilent** (TIROIR-5 et 6, fiche 127) : **faits** — écrits à la fiche 126 sans compilateur, compilés, éprouvés (la bibliothèque du bureau entière, les huit suites d'intégration), **13 sabotages sur 13**, **regardés au pilote** (les sept tiroirs à 100, 110 et 125 %, le Mood board qui défile à la molette, le survol à travers le cache) ; sur GitHub (`8525291`). **Et la loi de l'écart** (TIROIR-2, `596e783`) : deux défauts de la beta.4, vus au pilote — un jalon perdu à 110 %, une case de moins sur deux pages dans le Storyboard —, 5 sabotages sur 5. **Reste** : son écran — et le filet d'accent qui dit la part montrée, très discret (gris 59 sur 30) : sa force est à son ressenti | à son écran |
| [~] | **Distinguer une veille d'un gel** (VEILLE-1, fiches 126-127) : **fait** — sous Windows, le temps endormi (`QueryInterruptTimePrecise` moins `QueryUnbiasedInterruptTimePrecise`, relus dans `windows` 0.62.2 : Windows 10, comme Rust depuis 1.78) se retranche de l'image ; ailleurs nul par construction ; **2 sabotages sur 2**. **Jamais compilé sous Windows avant la CI de `596e783`** (fiche 127 § 9). Rien à déployer avant (aucune ligne neuve) | une session réelle sous Windows, mise en veille, sans image géante |
| [~] | **Le détourage, la suite** (fiche 121, `decisions/17`) : **regardé dans le vrai Glucose** (le 18/11 sur la boule, le 19/11 sur une pomme photographiée sur fond blanc, fiche 126 § 4 bis : le sujet entier, l'ombre partie) — **un liseré clair de huit à dix pixels de l'image reste au bord**, **diagnostiqué le 20/11** (fiche 127 § 4) : ce n'est pas DETOURER-2 — le bord de la pomme reflète le fond blanc (le *spill*), et le masque le garde à 0,98-0,99, là où la couleur du premier plan n'a rien à corriger. Le filtre guidé n'y change presque rien (mesuré) ; la ligne de couleur de Levin sur une bande de ± 4 pixels du masque l'éteint en partie (3,6 s, un solveur creux à écrire). **Ensuite** : mesurer sur de vraies photos avec leur vérité (les vérités du banc sont de synthèse, sans reflet) avant de choisir ; et de fines traces des rayures au-dessus de la boule ; ISNet selon la machine (sa licence d'abord) ; la voie de la carte graphique ; un clic pour désigner l'objet | regardé, puis à son écran |
| [~] | **La porte, la suite** : l'adresse de ce Glucose reconnaissable (dire la machine dans la question de la porte — Mistral appelait celle d'un autre de ses Glucose) ; B2 en option (redemander l'accord à chaque lancement) ; une icône de Glucose pour le relais ; le téléphone ; la « server card » que Mistral demande (`/.well-known/mcp/server-card/…`, 404, un chemin de brouillon) — **le chemin est fixé** : la SEP-2127 est **Final**, et réserve `<adresse MCP>/server-card` (le catalogue `/.well-known/ai-catalog.json` facultatif) ; mais le format, dans `experimental-ext-server-card`, se dit encore « Protocol Revision: draft », et aucun client connu ne lit ce chemin (Mistral sonde celui d'un brouillon) — à faire avec la suite de la porte, pas avant (fiche 127 § 2) | chacun dans `main`, saboté |
| [~] | **Son lag, à son écran** (fiche 112) : le rythme n'était pas la cause (le « 32 % » était le compteur du tempo) ; **PARU-1** dit désormais ce que l'écran montre, et `outils/pilote/essai_ecran.sh` mesure ses vrais gestes (pavé virtuel). **Ensuite** : (1) **le calage sur le battement** — envoyer chaque image « au prochain battement moins une avance » efface la gigue d'un battement (prototype : `~/.cache/glucose-bancs/rythme-112/calage-et-sonde-prototype.patch`) ; l'avance n'est pas la bonne grandeur : l'image suivante de Glucose occupe la carte graphique pendant que niri compose — le levier est de **dessiner juste à temps** (fiche 112 § 5), puis mesurer en paires à son écran ; (2) la latence de deux battements de l'envoi à l'écran, sur une carte graphique partagée avec niri ; (3) **ses vraies sessions lues par PARU-1** — le 14/11 à 1 h 45 (fiche 116 § 1) : 19 % de battements sautés alors qu'une image était due, 46 ms de l'envoi à l'écran en médiane, 93 au p90 ; **le 14/11 à 4 h 37 sur sa carte de Wikipédia, sur batterie** (fiche 117 § 1) : **46,5 % de battements sautés**, 39 / 55 / 78 ms, « ornements » jusqu'à 33 ms, « écouter la main » jusqu'à 72 ms d'un coup — à reprendre là | à son écran, PARU-1 : la gigue nulle, aucun battement sauté de plus qu'aujourd'hui, la latence mesurée |
| [~] | **Le co-working sur un canevas** (CLIENT-1, fiche 117) : **fait** — deux vrais Glucose dans le même canevas, chaque geste vu chez l'autre, un retardataire qui rattrape tout, un vrai conflit montré des deux côtés, `Ctrl+Z` par main, les curseurs ; la salle **en ligne** (fiche 117 § 5). **Les photos d'un participant à l'autre** (B1) : **faites** (fiche 119 — ce que je pose part avec mon geste, ce qui manque se demande en entrant ; prouvées par deux vrais Glucose à travers le relais en ligne, redéployé ; la 2.0.4-beta.2 en brouillon les porte). **Reste** : (1) l'alerte « des photos attendent quelqu'un qui les a » (sa question 18) ; (2) **le partage qui survit à un relancement** (aujourd'hui on rejoint de nouveau par le code), et un instantané dans la salle pour qu'un long canevas se rattrape vite (Secsync) ; (3) ~~son nom changé dans l'interface~~ — **fait** (IDENTITE-2, fiche 140 : son nom et sa couleur demandés la première fois, « Ton nom et ta couleur… » dans Collaborer) ; (4) le conflit **par propriété** (fusion à trois par le format à étiquettes) ; (5) le téléphone ; (6) **son essai** — il demande une version publiée sur ses deux machines (la 2.0.4-beta.2, son geste) | à son écran : son PC et son téléphone, ou deux Glucose sur son VivoBook, dans le même canevas |
| [~] | **Son lag au zoom** (*« pour ce qui est capital et nécessaire, il faut pouvoir optimiser »*) : REPOS-4 (fiche 109) — une image du mouvement 19-21 → 4-5 ms ; QUEUE-1 (fiche 110) — la cascade compte la fin de l'image : battements ratés 8-10 → 3 par série de crans, p90 24 → 17 ms. **Reste** : trois images par série à 18-22 ms (la fin de l'image qui varie, une texture plus chère que sa droite au premier cran) ; **remettre en question l'objectif** — « sous 10 ms au p99 » contredit la règle qui donne la période de l'écran à la cascade ; le critère qui se voit est « aucun battement raté », les 10 ms pèsent sur la latence : mesurer la latence d'un geste pendant une image de cascade avant de trancher (fiche 110 § 4). Ensuite, le tracé des flèches pour la carte graphique (2,5 ms de loin) | aucun battement raté au zoom de loin, mesuré en paires ; la latence mesurée |
| [~] | **Le cercle de l'IA, sur la Time Machine** (VISION § 8 bis ; la page : https://claude.ai/artifact/8WJTkJLvv7HvDDrJMng7CL — sa source : `~/.cache/glucose-bancs/cercle-ia/`) : la Time Machine signe chaque geste et porte des écrits (fiche 108) ; **le journal des rectifications** est écrit (RECTIF-1, fiche 110) — une couleur, une flèche retouchée, retirée ou suivie, sans seuil, avec le pourquoi (B3, exact). **Ensuite** : regarder ce que le journal récolte chez lui (pas d'apprentissage avant, sa règle) ; le pourquoi d'une flèche ; ses propres flèches, que l'IA ne juge pas mais dont elle doit *« comprendre l'importance »* ; **puis** les couleurs et l'interface de la Time Machine pour tout y voir — un écrit y tient aujourd'hui sur une ligne coupée (*« d'abord le fonctionnement, l'esthétisme et l'ergonomie après »*) | à son écran : ses rectifications lues dans la Time Machine |
| [~] | **Les flèches de Wikipédia** (VISION § 6) : **PARENTE-4 dans `main`** (fiche 108) — l'IA juge les flèches qu'un agent ou un script a posées, la souris ne rapproche que celles de sens ; jamais les siennes. Sa carte signée (`~/Documents/Glucose-essai-wikipedia-signee.glucose`, fiche 110) le montre : la souris près d'un texte n'éclaire qu'une poignée de flèches. **Ensuite** : une couleur qui dit quelque chose (le gris n'est que temporaire), partir des **mots** qui citent (les ancres de texte existent), des liens partiels, un tracé à logique de câblage. La recherche : `~/.cache/glucose-bancs/halo-106b/recherche-fleches.md` | à son écran, sur la carte signée |
| [~] | **Les membranes en carte** (fiches 105 et 107) : écrites, éprouvées, sabotées, regardées ; 11 % (*« vraiment vraiment parfait »*). Reste **l'allure de la frontière**, à sa parole (https://claude.ai/artifact/7L4zAY5U4mnebV2qvHgSK4) | sa parole sur la frontière, puis son écran |
| [~] | **Le halo d'un texte dit sa parenté** (fiche 106) : écrit, éprouvé, regardé sur la carte. Reste **son écran** ; sa couleur et ce que l'IA en apprend passent par le cercle de l'IA, plus haut | son écran |
| [ ] | **Les boutons qui manquent** (INVENTAIRE § 3), un par un, chacun conçu après une recherche (une fonction « de Tauri » se vérifie d'abord dans Tauri — « filtrer par domaine » n'y existait pas, fiche 115 § 7) : ~~l'export HTML~~ — **dans `main`** (fiche 117 ; reste l'iPad, Safari jamais essayé) ; d'abord les miroirs (`Ctrl+Maj+M`), les GIF animés et les vidéos, la provenance des images… | chacun dans `main`, saboté, regardé dans le vrai Glucose |

## 3. Ce qui attend son écran

**La sélection en équerres, le reste flouté** (Q12, fiche 137 ; `main` depuis le 24/11 à 3 h, à sa
relance) : choisir un texte d'un clic — ses quatre équerres, le reste flouté ; le déplacer — le flou
s'efface pendant le glisser et revient au lâcher ; choisir plusieurs nœuds au rectangle ; clic droit
sur la sélection, « Ne plus flouter le reste », puis « Flouter le reste ». **Le flou est-il assez
léger ?** (40 % de l'image D : un seul nombre à changer.)

**Après la beta.5** (`main` depuis le 23/11 au soir ; **déjà dans son Glucose**, relancé le 24/11 à 7 h 30 sur `dc86ff2`) : **la pipette sur la lueur d'un texte coloré** — elle prend sa couleur (PIPETTE-1) ; **dézoomé, un clic sur un petit texte posé près du bord d'une membrane** — il prend le texte, un double-clic l'ouvre (PICK-3) ; **tracer une membrane autour de deux textes, puis la déplacer** — ils suivent (MEMB-9) ; **écrire plusieurs lignes dans un texte d'où part une flèche** — elle part du bas qu'on voit pendant qu'il grandit, au lieu de le traverser (FLECHE-EDITION, sa note du 23/11 au soir, fiche 134 § 11) ; **une flèche d'un texte vers une membrane qui porte des nœuds** — elle va droit au bord le plus proche (FLECHE-7, sa capture du 23/11) ; **glisser des nœuds sur un dossier** — il s'entoure, ils y entrent, `Ctrl+Z` les rend (RANGER-2) : la boîte est-elle assez visible ?

**Avec la beta.5, publiée le 23/11** : **un dossier choisi, la roue de couleur, le curseur de droite** — son opacité ; le nom reste-t-il lisible, le bandeau léger lui va-t-il (fiche 133 § 7) ? Puis :
ouvrir les sept tiroirs, puis « Agrandir l'interface » au menu du vide jusqu'à ce qu'un tiroir déborde
(le Mood board à 125 % en 1440 × 900) ; la molette dessus — son contenu défile sous ses bords, et le
filet de son bord intérieur dit la part montrée (**assez visible ?**) ; à 110 %, la Time Machine et
tous ses jalons ; un Storyboard à deux cases, sur une seule page.

**Avec la 2.0.4-beta.4, publiée le 19/11** (fiches 120 à 126 — la beta.3, publiée, en porte déjà
une partie : le texte net, les modes de la fenêtre, `F2` ; regardé par nous au pilote, jamais par lui) : dans son document « ce qui attend ma parole », l'onglet « Textes optimisés » — les tableaux de
Claude.ai dans leur nœud, les listes décalées, les textes dans leurs membranes, les émojis en couleur — les ✔ clairs dans un nœud, foncés sur un post-it ; ouvrir les sept tiroirs à
la fois (chacun du côté de son bouton, collé au bord, plat) ; « Agrandir l'interface » deux fois au menu
du vide, puis « Réduire » ; une photo sur fond blanc, clic droit, « Détourer : la forme principale », puis
`Ctrl+Z` ; le texte de la bande et des icônes, de près (nets ? trop fins ?) ; `F11`, `Ctrl+Maj+A`,
`Maj+F11` ; un dossier, `F2`. **À l'école, sous Windows** : la même chose, et « Barre de Windows » au
menu du vide, deux fois.

**Sa capture du 17/11** (fiche 118) — **à l'école, sous Windows, avec la 2.0.4-beta.2** : chaque bouton de
la bande, surtout ceux de droite (Collaborer, Exporter, Plugins, Mood board, Domaines) ; un clic entre
deux boutons ne sélectionne rien. **Partout** : ouvrir tous les tiroirs à la fois (aucun sur un autre, le
Storyboard et la Time Machine à la hauteur de ce qu'ils montrent, « + Nouveau domaine » au-dessus de la
poignée) ; entrer dans un dossier, cliquer le premier nom du fil d'Ariane ; choisir une flèche, tirer
un de ses bouts sur un autre nœud, puis dans le vide ; clic droit sur un point orange, « Retirer ce
coude » ; « Redresser la flèche » ; une photo choisie, `Alt+Maj+H` puis `Alt+Maj+V`, et `Ctrl+Z`. **Le co-working
et ses photos** (fiche 119) : deux Glucose dans le même canevas (son PC et son téléphone, ou deux
fenêtres) ; une photo collée chez l'un apparaît chez l'autre, et un canevas plein de photos rejoint par
l'autre les montre toutes.

Écrit, éprouvé, saboté — **jamais jugé par lui**. Sous Linux : `glucose > ~/sortie-glucose.txt 2>&1`, puis
on lit nous-mêmes la sortie et sa chronique (`/tmp/glucose-chronique/derniere-session.txt`, copiée
d'abord).

**Sur la carte de Wikipédia signée** (`~/Documents/Glucose-essai-wikipedia-signee.glucose`, une copie
posée par la version neuve — l'ancienne, `…-liens.glucose`, ne connaît pas les agents) :

* zoomer et dézoomer à la molette de loin : plus fluide qu'hier (REPOS-4, QUEUE-1) ;
* la souris près d'un texte : seules quelques flèches s'éclairent (PARENTE-4) ;
* colorer un texte qui a un halo de couleur, `Ctrl`+clic sur une flèche, puis la Time Machine
  (`Ctrl+H`) : sous ces gestes, l'écrit de l'IA de Glucose — ce qu'elle pensait, et pourquoi
  (RECTIF-1) ;

* le halo de la parenté (fiche 106) : au fond de « Eau », les textes de l'eau d'une même couleur, ceux
  de la chimie d'une autre, les autres au contour gris ; écrire un texte, attendre une seconde : sa
  couleur suit ; colorer un texte (la barre de couleur) : sa lueur prend la couleur, pleine ;

* les membranes en carte (fiche 105) : de loin, une couleur par région, aucune boue ; zoomer au fond
  de « Eau » : les blocs gris sombres sur la couleur de leur région, plus fluide qu'avant ; le grand
  cadre « Wikipédia » gris ;

* dézoomer, bouger la souris, déplacer la vue de loin : bien plus fluide (LETTRES-1, fiche 104 ; TITRE-3
  et les deux mémoires, fiche 108 : 17 → 7,5 ms par image) ;
* plusieurs coups de molette avec des pauses d'une seconde, puis `Alt+Z` : un pas par série — le veut-il
  ainsi ? (RETOUR-2, fiche 103) ;
* survoler un nœud, `Ctrl`+clic sur une flèche, `Alt+Z`, choisir un nœud, `B`, cliquer un amené (les
  liens, fiches 102-103) ; l'effet de la souris est doux (sa courbe) — le veut-il plus net ?
* « Entrer dans la membrane », `Échap` (FOCUS-1) ; les titres à la taille de leur région (TITRE-1) ;
  « Disposer en colonnes ».

**Ailleurs, au PC** :

* écrire dans un texte, un post-it, le titre d'une membrane ; un texte qui passe à la ligne (le contour
  suit) ; `Ctrl+G` sur deux nœuds (SAISIE, GROUPER) ;
* déplacer au clic molette sur beaucoup de photos : aucune ne clignote, aucun micro-gel (fiche 100) ;
* la roue de couleur : une membrane, la case au disque, `Ctrl`, `Maj`, la molette, un code, la pipette ;
* `Ctrl+F` sur un gros tableau ; `F3` ;
* ses commandes au terminal : `glucose aide`, `glucose chercher eau` ; un double-clic sur un `.glucose` ;
* le domaine dans deux documents : « où », la question de deux « Python » ;
* les opérateurs des post-it : `Alt+1…4` ;
* copier un texte vers Discord, coller une image, glisser depuis Nautilus et vers lui (le pont Wayland) ;
* un très long texte vu de près ; zoomer très près d'une formule, d'un emoji (P8) ;
* la palette (`Ctrl+Maj+P`), changer un raccourci ; **une flèche choisie : la palette ne propose
  plus « Couleur »**, et le clic droit montre un menu rangé par sens (APPLIQUE-1, fiche 113) ;
* **dater** : choisir un texte et une photo, `Maj+T`, taper « renais » puis choisir la Renaissance ;
  la pastille ambre à leur coin ; dézoomer ; `Ctrl+Z` ; un lien dans un texte, plus clair
  qu'avant (DATE-1, fiche 114) ;
* **la frise chronologique** : dater trois ou quatre textes à des époques différentes, relier deux
  d'entre eux par une flèche, `Maj+R` ; glisser une poignée de la fenêtre ambre — ce qui sort de la
  période pâlit, la flèche avec ; la molette sur la frise ; la croix (FRISE-1 et 2, ESTOMPE-1 et 2,
  fiches 114-115) ;
* **regarder un domaine** : donner l'eau à trois textes à des poids différents (le panneau Domaines,
  ses paliers), puis « voir » sur l'eau — ce qui n'en est pas pâlit, d'autant moins qu'il en est ;
  « voir » encore (DOMAINE-1, fiche 115) ;
* **le mode zen** : `F11` — plus que le canevas ; « ZEN · F11 » en haut à droite, ou `F11`, ou
  `Échap`, pour revenir (ZEN-1, fiche 115) ;
* un agent dans Glucose (`glucose --mcp`) : « Autoriser », une demande ; puis la Time Machine
  (`Ctrl+H`) : sous chaque geste de l'agent, son nom et ses écrits (MAIN-1, NOTE-1, fiche 108) ;
* le Storyboard, Mood board, Ordonner, l'export SVG et PNG, le mode référence.

**La porte aux IA publiques** (fiche 116), **avec ton compte Claude.ai** : Collaborer → « Ouvrir la
porte » → « Copier l'adresse » ; dans Claude.ai : Personnaliser → Connecteurs → Ajouter un
connecteur personnalisé, coller l'adresse ; la page dit un code et « va dans Glucose » ; dans
Glucose, le même code, « Lire et écrire » ; puis demander à Claude de lire le tableau et d'y poser
un texte — il apparaît, signé « Claude (claude.ai) » dans la Time Machine ; Collaborer → « Les IA qui
ont ta porte… » pour la retirer.

**Le canevas partagé** (fiche 117), **sur son VivoBook seul** : dans son Glucose, Collaborer → « Partager
ce canevas » (le code est copié) ; puis, dans un terminal, `bash ~/Projets/GlucoseGit/outils/porte/essai/un_second_glucose.sh`
— une seconde fenêtre, « Bob », dans un dossier jetable ; dans celle-ci, Collaborer → « Rejoindre (le code
copié) ». Poser un texte dans l'un, le voir dans l'autre ; bouger la souris : son curseur, nommé, dans
l'autre ; colorer le même texte des deux côtés presque en même temps ; `Ctrl+Z` dans l'un ne défait que
ses gestes. **Entre son PC et son téléphone** : il faut une version publiée (la 2.0.4-beta.2, son geste).

**Au téléphone** (le sien est en **2.0.4-beta.1** depuis le 16/11 — mis à jour par-dessus) : une longue
session ; le texte de près ; les doigts (deux touchers annulent, trois rétablissent) ; le Storyboard au
doigt ; la pile des barres du bas (BAS-1 : gêne-t-elle ?).

**Quand une machine Windows existera** : la bande qui remplace la barre blanche (BARRE-1), éteindre le PC
Glucose ouvert (FIN-SESSION-1), un dépôt posé sous le curseur.

## 4. Ce qui attend sa parole

**Ses questions vivent dans son Glucose** depuis le 19/11 (sa parole, VISION § 9) : son document
« ce qui attend ma parole » (`~/Downloads/ce qui attend ma parole.glucose`), lu et écrit par sa porte
(`outils/porte/claude_code.mjs`) ; ses réponses vont dans VISION, et sortent d'ici. **Ouvertes** : **ce qu'une IA écrit
dans la Time Machine** (sa note `text-321`, notre proposition `relais-rep-321` : « Retirer » un écrit,
des notes jamais des ordres pour les IA suivantes, la liste de tous les écrits — fiche 139 § 7) ;
**les accès à ce document en un seul endroit** (sa demande `demande-codes-actifs`, notre proposition
`relais-rep-codes`) ; le
compte Cloudflare de Glucose (son clic, et le nom — posée le 19/11 à 14 h, à droite de la porte) ;
**les GitHub Pages** (sa note du 19 ou du 20/11 : l'ancienne version web de Tauri encore en ligne, *« je
propose de enlever »* — notre réponse `claude-code-pages` : rien n'en dépend, sauf peut-être quelqu'un
qui s'en sert dans son navigateur ; à son « enlève », `gh api -X DELETE repos/shazamifius/GlucoseGit/pages`
puis l'environnement `github-pages`, fiche 127 § 5). **Il a recopié toute l'ancienne page de questions**
(1 à 19) dans une membrane « en attente », à droite de son document : ses réponses y viendront.
Restent, de [l'ancienne fiche de questions](https://claude.ai/artifact/KwCMGG6Keymuzr5V4DJ4gN) (fiche
118 § 5), ces questions — désormais aussi dans sa membrane « en attente » :

* **Le cap** : l'iPad par le web dans la partie 1 ? (défaut : oui, en dernier) ; les presets Figma et Miro
  (défaut : partie 2) ; « avec des utilisateurs » = des gens qui s'en servent sans rien perdre (des comptes
  seraient la collaboration, partie 2) ; le nom des versions (2.1.0 la première stable).
* **Glisser un nœud vers une fenêtre posée par-dessus Glucose** (fiche 100 § 10) : le veut-il, par quel
  geste ?
* **B et C** (fiche 98 § 3) : une membrane dit ce qu'elle porte ; un fil « où je suis ».
* **Le sens des couleurs des membranes posé par un agent** (fiche 102 § 6, l'image
  `~/.cache/glucose-bancs/squelette-sens2/images/`) : *« je ne sais pas trop »* (10/11) — à reprendre avec
  l'IA de similarité du halo, qui peut aussi dire la proximité des régions.
* **« Créer un sous-dossier »** (fiche 98 § 7) : ranger la sélection dans un dossier neuf — que deviennent
  les flèches qui traversent ? avec le dossier « à retravailler ».
* **Ordonner « très très grandement »** ; **sa liste PureRef** ; **le
  défilement au bord** (« moche, pas fluide », plus tard) ; **les rideaux**, **les membranes de Mary**, « optimiser » dans la Time Machine.
* **Ses clés et ses appareils** : une copie de la clé de mise à jour sur une clé USB à lui ;
  `sauvegarde-profil-2026-10-31.7z` (ses clés de serveur, son identité web3), introuvable ici ; sa clé
  SauceNAO ; le modèle de l'iPad ; les téléphones de ses testeurs.
* **Protéo**, quand il reprendra : la bascule de son cerveau (qwen3.5:4b, démontré meilleur, fiche 78).

## 5. Plus tard — la partie 1, par étapes

**Ne rien perdre**
* [~] P0 — le tableau de 6 000 images mort au téléphone : reproduit et réparé (MEMOIRE-4, fiche 83) ;
  reste qu'une version récente arrive sur son téléphone et que ses fins anormales cessent.
* [~] MOTEUR-1 (fiche 84) : prouvé seulement quand la télémétrie montre une installation touchée qui
  dessine.

**Tout ce qui manque** (la parité avec Tauri, et ses demandes)
* [ ] P6 — relire chaque panneau contre Tauri, champ par champ : Pomodoro, Domaines, Time Machine, la
  bande, Ordonner, Mood board.
* [ ] Les miroirs (`Ctrl+Maj+M`) ; les GIF animés et
  les vidéos ; l'export HTML ; le mode zen (`F11`) ; la provenance des images (SauceNAO).
* [ ] Les choix à l'utilisateur : la planche claire ou sombre, l'export PNG à 100 % ou aux originaux, les
  photos du SVG, la numérotation « 2.3 », la place du numéro.
* [ ] Le domaine : ses synonymes (« eau », « water ») ; sa visualisation (ergonomie).
* [ ] Le dossier, à retravailler ; « double-clic + zoom comme Tauri ».
* [ ] L'encre d'un post-it ; les titres et le gris de loin (GRIS-3, fiche 98 § 7).
* [ ] **L'aimant qui comprend l'intention** (sa note `text-316`) : une membrane pleine de petits textes donne des points
  d'aimant « chelous » — regarder au pilote ce qui aimante quand on glisse une membrane ; puis l'aimant qui ne
  prend que la main qui ralentit (*Oh Snap*, Fernquist et al., INTERACT 2011 ; *Snap-and-go*, Baudisch et al., CHI 2005).
* [ ] **Réorganiser après une écriture : trois propositions à côté** (sa note `text-wiov-327`) : un texte qui grandit
  recouvre ses voisins ; à la fin de l'écriture, trois aperçus — pousser le moins possible en gardant l'ordre
  (la carte mentale de Misue et al., 1995 ; *Fast node overlap removal*, Dwyer et al., 2005), agrandir les
  membranes, ou ranger en colonnes — un clic, un geste, `Ctrl+Z` ; l'IA de Glucose apprend lequel il prend.

**Fluide partout**
* [ ] Son téléphone : lire ses postes (16,4 ms l'image typique), nommer le coupable, le tenir ; P8 au
  téléphone, jamais mesuré.
* [ ] Le modèle de coût qui prédit (COUT-1 ne connaît que les pixels des photos).
* [ ] Les titres des membranes, des zones et des cases en formes, de près (fiche 73 § 7).
* [ ] Choisir 8 000 images (28 ms par image) ; le fondu d'un message (REPOS-3) ; la mémoire des formules
  qui ne s'oublie jamais ; le zoom sur une très grande région (fiche 95 § 8).

**Toutes les machines**
* [ ] Android : le presse-papiers du système, HEIC, les champs du dock, sa mise à jour automatique
  prouvée ; les rayures de sa PowerVR.
* [ ] Les doigts sous Windows ; le Mac publié (`.dmg`, non signé) ; **l'iPad par le web**.
* [ ] Le `.deb` qui dit ses vraies dépendances (GTK 3 retiré, fiche 100) ; la sortie des commandes du
  terminal dans une console Windows.

**L'ergonomie (la 2.1.0)**
* [ ] Le thème, puis la disposition des panneaux, comme données (après P6).
* [ ] Le preset de Glucose ; le preset PureRef ([PUREREF](PUREREF.md)) ; choisir son preset, l'exporter.
* [ ] L'interface du téléphone, toute refaite ; la visualisation des domaines.

## 6. Après la partie 1

* **Protéo, en pause** (sa parole du 25/10) : l'agent et l'humain dans les deux sens, la toile rangée, le
  tri de la demande, la machine qui choisit seule, l'organe de recherche — écrits et éprouvés au point où
  ils sont (fiches 75 à 81).
* **La partie 2** : les 10⁷ nœuds (l'arène, écrite, à brancher) ; le canevas de Wikipédia de
  `glucose-brain` et des jeux de données d'IA ; les presets Figma et Miro ; **Collaborer** (obligatoire) ;
  les plugins, l'IA locale ; les graphes de questions et de réponses ; le document de recherche ;
  Trans-domaines ; Glucose en briques, les dépendances vers zéro ; GLISSER-2.
* **La recherche, à côté** : l'améliorateur d'image à l'import (`recherche/cerveau-image/`).

## 7. Publiées

La **2.0.4-beta.4** (son clic, le 19/11 à 19 h 15) est la dernière ; avant elle, la 2.0.4-beta.3 (19/11,
16 h 20), la 2.0.4-beta.2 (18/11) et la 2.0.4-beta.1 (13/11). L'état réel se lit sur GitHub (`gh release
list`), jamais ici.
Chaque version a sa fiche (les notes : `outils/publication/`).
