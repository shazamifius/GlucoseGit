# 117 — Le co-working : deux Glucose dans le même canevas ; l'export en page web dans `main`

> Session du 17/11/2026, à partir de 9 h 28, sans parole neuve de sa part : il a collé la passation
> de la fiche 116 et laissé travailler. Le plan : lire ses sessions, **brancher le co-working dans
> Glucose** (fiche 116 § 8), finir la page web. Le co-working est fait jusqu'au bout — deux vrais
> Glucose dans le même canevas, le conflit montré, `Ctrl+Z` par main, les curseurs —, sauf les photos
> d'un participant à l'autre (B1) et le partage qui survit à un relancement.

## 1. L'ouverture

* **Sa dernière session** (le 14/11 à 4 h 37, sa chronique et sa sortie copiées) : 84 s sur sa carte de
  Wikipédia (1 672 nœuds), **sur batterie**. PARU-1 : **46,5 % des battements sautés** alors qu'une
  image était due (19 % dans sa session de 1 h 45) ; de l'envoi à l'écran 39 ms en médiane, 55 au p90,
  78 au p99 ; « ornements » jusqu'à 33 ms ; « écouter la main » jusqu'à 72 ms d'un coup. La matière du
  chantier de son lag (SUITE § 2), non traitée ce matin. **Pas d'essai de la porte avec son compte.**
* **La télémétrie** : 190 sessions, 10 installations, 40 fins anormales (176 et 36 le 14/11). **Son
  téléphone est en 2.0.4-beta.1** depuis le 16/11, sa première session dans cette version finie
  proprement ; le PC de l'école (GTX 1660 SUPER) aussi. Les fins anormales récentes sont toutes celles
  du Linux à RTX 4070 SUPER, l'appareil redémarré des heures après — un PC éteint Glucose ouvert, pas
  un plantage.

## 2. EXPORT-HTML-1 dans `main`

Le correctif de la fiche 115 § 7 ter s'appliquait encore tel quel. Un seul changement : la page lisait
la liste des annotations du tableau en direct, et le cliquet du couplage au modèle a monté (39 contre
38) — le noyau sait désormais donner les flèches d'un tableau (`Board::fleches`). **8 sabotages sur
8.** La suite entière a montré que **le cliquet 3 tombait déjà sur `main`** : CHIFFRE-1 était entré
sans appelant dans l'application, et la suite n'avait pas tourné après — le client l'a branché.

## 3. Remis en question, avant de concevoir

La recherche (§ 8) a confirmé l'architecture de la fiche 113 (le greffier aveugle : la salle ordonne
sans lire, chaque Glucose rejoue) et précisé ce qu'un client doit faire :

* **La salle, plus mes gestes en attente** (Weidner 2024-2025, Replicache, tldraw sync) : un geste
  s'applique tout de suite chez soi ; quand celui d'un autre arrive, on défait les siens en attente,
  on applique le sien, on refait les siens. **Rejoué entièrement**, sans l'optimisation qui
  appliquerait directement un geste qui ne touche pas les miens : une seule loi, la plus rare
  (un geste en attente vit ~100 ms) ne justifie pas la seconde.
* **`Ctrl+Z` à plusieurs** (Liveblocks, Figma, Google Slides) : chacun ne défait que les siens ; un
  nœud qu'un autre a retiré : rien. Et Figma : *« an undo operation modifies redo history at the time
  of the undo »* — défaire, puis tout rétablir, ne change rien.
* **Contre la passation — deux codes indépendants** : **un seul code, deux pouvoirs, par dérivation**
  (CODE-1). Le code « écrire » donne par empreinte le code « lire », qui donne la salle et la clé ; le
  secret d'écriture ne se tire que du code « écrire ». Rien à garder nulle part.
* **Contre la passation — la base seule en morceaux** : **toute lettre** en morceaux d'un mébioctet.
  Un geste aussi dépasse les 2 Mo d'une valeur de la salle : retirer un tableau entier de sa carte de
  Wikipédia porte tout ce qu'il contenait (JRN-1).
* **Rejoindre lit le code dans le presse-papiers** : copier le code reçu (Discord), cliquer — aucun
  champ à remplir.
* **Un autre humain n'est ni « l'humain » ni un agent** (`Main::Participant`) : son geste ne se défait
  pas ici, ne rectifie pas l'IA d'ici, et l'IA de Glucose ne juge pas ses flèches.
* **Le nonce** : 96 bits du hasard du système par morceau. Deux messages sous la même clé ne
  partagent un nonce qu'avec une chance de l'ordre de N²/2⁹⁷ — un milliard de messages, une chance sur
  cent milliards. XChaCha20 (192 bits, qu'Epicenter prend) n'a pas de vecteurs dans une norme publiée ;
  inutile ici.

## 4. Le client, tel qu'écrit

| | où | ce qu'il fait |
|---|---|---|
| **CODE-1** | `core/salle/code.rs` | `glucose-ecrire-…` / `glucose-lire-…` (43 signes) ; `H(étiquette ‖ secret)`, chaque dérivée son domaine (NIST SP 800-56C) |
| **La lettre** | `core/salle/lettre.rs` | qui parle (marque, nom, couleur, sa main ou un de ses agents), puis la base, un geste ou un curseur ; préfixée du schéma de l'émetteur — un schéma plus neuf se refuse en le disant |
| **SCELLE-1** | `core/salle/morceaux.rs` | les morceaux scellés (ChaCha20-Poly1305) ; données associées : la salle et le nom local `<marque>-<n>_<rang>`, le rang et le total **dans** le scellé ; le réassemblage garde ses morceaux à travers une reconnexion |
| **CLIENT-1** | `core/store/participation.rs` | en attente, rejouer l'ordre de la salle, une pile par main et par le nom, la règle de Figma, rien pendant un geste ouvert ; **appliquer, défaire et écrire au disque commencent par faire partir ce qui part** (sinon un geste pas encore parti n'est pas rejoué, et les deux divergent) ; un lecteur voit ses gestes se défaire |
| **NOMMER-1** | `core/store/journal/partir.rs` | un déplacement part nommé dans l'état qui le suivait (la file se remonte à rebours jusqu'au plus ancien déplacement) |
| **SELECTION-1** | `core/store/undo.rs` | ce qu'un geste appliqué retire sort de la sélection — au coût du geste, le résultat net (vrai aussi pour Glucose seul) |
| **Le participant** | `core/store/journal/mains.rs`, `persist/histoire.rs` | `Main::Participant { marque, nom }` ; `Auteur::d_un_participant` (identifiant tiré de la marque, pas un agent) ; une seule loi pour le scribe, `Auteur::d_une_main` |
| **Le fil** | `desktop/salle/lien.rs` | comme `publique/lien.rs` : ping, reprise ; une lettre gardée jusqu'à son numéro (renvoyée à la connexion suivante) ; un trou redemandé une fois par position ; les lettres lues hors de l'image |
| **Collaborer** | `desktop/app/collaborer.rs` | la question (partager, rejoindre, la porte aux IA ; partagé : les deux codes, qui est là, quitter) ; son nom et sa couleur comme dans Tauri (`identite.txt`) ; le tour de boucle ; les curseurs qui partent |
| **Les curseurs** | `desktop/renderer/curseurs.rs` | une flèche de sa couleur et son nom, par-dessus tout, sur les deux voies, avec « ce qui vient d'ailleurs » |
| **Le relais** | `outils/porte/src/salle.js` | la salle répond au `ping` sans se réveiller |

* **Un seul endroit qui parle** : le témoin de la porte et le co-working disent par
  `dire_ce_qui_arrive_au_canevas` ce qu'un autre vient de faire dans le canevas — le plafond des
  toasts reste à 46 ; copier un code, un partage impossible, l'absence de code se disent **dans la
  question**, là où l'on agit.
* **La Time Machine** met la participation de côté pendant un voyage ; ce qui arrive attend dans le
  fil, et rien du passé ne part.
* **L'état du partage vit avec la porte** (`lancement.agents.partage`) : le constructeur de
  l'application était à 80 lignes.

## 5. L'essai réel

`outils/porte/essai/essai_salle_pilote.sh` : deux vrais Glucose hors écran, chacun dans son compositeur
(`piloter.sh` mène plusieurs Glucose par `PILOTE_ETAT`, et lit ou écrit le presse-papiers de son
compositeur), Bob derrière **un réseau lent** (`essai/reseau_lent.py` : ce qui descend vers lui arrive
3 s plus tard), contre la salle locale (`wrangler dev`). Joué, chaque image regardée :

* Alice partage — « Canevas partagé », le code copié, « Tu es « Ours Émeraude » » ; Bob colle le code
  et rejoint, **rattrape la base et chaque geste** par le réseau lent ;
* un texte de chacun, vu chez l'autre au même endroit, le texte tapé compris ;
* **les curseurs** : la souris de chacun chez l'autre, à sa place, de sa couleur, nommée ;
* **le conflit** : Alice colore « Alice » en rouge ; une seconde plus tard Bob, qui ne voit pas encore
  le rouge, le colore en émeraude — **émeraude chez les deux**, et « Deux gestes ont touché le même
  nœud : le plus récent l'emporte » chez Alice tout de suite, chez Bob quand le rouge lui arrive ;
* **`Ctrl+Z`** : Alice pose « A2 », Bob « B2 » (le dernier geste) ; Alice défait deux fois — « A2 » part
  chez les deux, « B2 » reste.

**Ce que l'essai a trouvé** :

* **SELECTION-1** : après `Ctrl+Z`, la barre disait « 1 sélectionné » sur un nœud qui n'existait plus —
  vrai aussi pour Glucose seul, et fréquent à plusieurs (un autre retire ce qu'on a choisi).
* **Geler un Glucose ne fabrique pas un conflit** : les frappes envoyées pendant le gel se perdent (le
  clavier virtuel du pilote disparaît avant que Glucose ne les lise). D'où le réseau lent — ce que vit
  vraiment quelqu'un sur une mauvaise connexion.
* **Le premier conflit scripté n'en était pas un** : la palette de Bob s'ouvrait trop tard, le rouge
  lui était déjà arrivé — deux gestes successifs, convergents, sans message. Seule la lecture de
  chaque image l'a montré ; le script ouvre désormais les deux palettes ensemble.
* Rejoindre depuis un brouillon rouvert pose la question du travail non enregistré (BROUILLON-1) :
  c'est juste.

**Son essai, seul, sur son VivoBook** : `outils/porte/essai/un_second_glucose.sh` lance un second
Glucose (« Bob ») dans un dossier jetable (ses fichiers temporaires à part : sa chronique reste
intacte), sans télémétrie ni mise à jour ; la porte des agents reste au sien. Essayé hors écran avec
**son Glucose installé** (`21acb0c`, mis à jour à 11 h 37) : partager dans l'un, rejoindre dans l'autre
par le même presse-papiers, un texte et les curseurs des deux côtés, par le relais en ligne.

**En ligne** — la salle déployée le 17/11 à 11 h 19 (`npx wrangler deploy`, son compte ; ses dix
verdicts de l'extérieur, la porte publique vérifiée de nouveau par `faux_glucose.mjs`), puis l'essai
entier contre le relais public : **tout passe**, le conflit dit des deux côtés, les curseurs, `Ctrl+Z`.
Le premier essai en ligne avait échoué **par l'outil** : le réseau lent d'un essai d'avant vivait
encore (`kill` n'arrêtait que l'enveloppe `nix shell`), relié à la salle locale — Bob y attendait une
base qui n'existait pas. Il a montré **un défaut de Glucose** : un code qui mène à une salle vide
laissait attendre pour toujours, sans rien dire. La question de Collaborer le dit désormais (« Ce code ne
mène encore à aucun canevas… »), jamais par-dessus une autre question.

## 6. Les épreuves et les sabotages

* **Le noyau** : 6 épreuves des codes, de la lettre et des morceaux ; 15 de la participation (deux
  participants et une salle simulée : la convergence dans les deux ordres, le conflit des deux côtés,
  `Ctrl+Z` par main, la règle de Figma, le nœud retiré, le geste ouvert, le nommage exact, la
  signature, le lecteur, la sortie, SELECTION-1, le geste pas encore parti, `Ctrl+Z` dans la même
  image, le participant qui ne rectifie pas). **L'application** : le bouton Collaborer par la souris
  (réécrit), l'identité, le curseur peint à sa place, les curseurs montrés, la phrase de ce qui n'a pas
  tenu.
* **Écrire les sabotages avant de les lancer** a montré **six gardes sans épreuve** (faire partir avant
  d'appliquer, avant de défaire, avant le disque ; le participant qui rectifierait ; les curseurs d'un
  autre tableau ; la phrase) — écrites — et **une garde redondante** : la longueur d'un code, que les
  32 octets et l'alphabet imposent déjà — retirée.
* **La suite entière** : 3 301 épreuves vertes (bornée, le 17/11 à 11 h 11) ; clippy à zéro ; les 16
  cliquets — le 2 (les toasts) et le 4a (80 lignes) ont protesté en route, à raison : un seul site dit
  ce qu'un autre a fait dans le canevas, et l'état du partage vit avec la porte.
* **Les sabotages** : **29 sur 29** (`outils/sabotages/client-117.json`), **1 sur 1** contre
  `wrangler dev` (`salle-117.json`) ; et les **8 sur 8** de la page web (`export-html-115.json`).

## 7. Ce qui reste, et ce qui reste contestable

* **Les photos d'un participant à l'autre (B1)** : une photo posée chez Alice arrive chez Bob **sans ses
  octets** (le scribe de Bob n'écrit pas l'objet, rien ne casse). Sa clé n'est pas un chemin
  (`depot:<instant>-<rang>-<nom du fichier>`) ; mais un fichier texte ou une tuile déposés gardent
  **leur chemin complet** (`source_file`) — dans le document lui-même, donc chez qui le reçoit, par un
  `.glucose` envoyé comme par un canevas partagé.
* **Le partage ne survit pas à un relancement** : on rejoint de nouveau par le code, et l'on rattrape
  tout depuis la base. Sans instantané dans la salle (Secsync), un canevas qui vit longtemps se
  rattrape de plus en plus lentement.
* **Le conflit est au niveau du nœud entier**, pas de la propriété (Figma) : Alice change la couleur
  d'un texte, Bob son texte, en même temps — le dernier l'emporte en entier. Une fusion à trois champ
  par champ, par le format à étiquettes du disque (`persist/tags`), le réglerait sans code par genre.
* **Un participant au même code peut parler au nom d'un autre** (aucune signature par participant ;
  Secsync en signe) ; la salle peut retenir ou réordonner (le client compte les numéros).
* **Le fil n'est éprouvé que par l'essai réel**, pas par des épreuves sabotables (les redemandes, les
  renvois après une reconnexion).
* La question de Collaborer **ne se met pas à jour** pendant qu'elle est ouverte (« Il se relie… »), comme
  celle de la porte ; le curseur d'un participant parti reste jusqu'à la prochaine image (30 s).
* **Le nom tiré** (« Loup Émeraude » sur une pastille bleue) : un nom de couleur qui ne dit pas la
  couleur, comme dans Tauri — à son goût ; le nom se change dans `identite.txt`, pas encore dans
  l'interface.
* Un lecteur voit ses gestes se défaire, au lieu d'un canevas qui ne se laisse pas toucher.
* **Le téléphone** : jamais essayé.

## 8. La recherche (consultée le 17/11/2026)

* Matthew Weidner, [Architectures for Central Server Collaboration](https://mattweidner.com/2024/06/04/server-architectures.html)
  (2024) et [sa conférence de 2025](https://docs.google.com/presentation/d/1WHUKnht-RxZ9lrMLBMwrs8BpW7ifN7y6iillhju2rKw/htmlpresent)
  (la « server reconciliation », défaire les gestes en attente, appliquer, refaire).
* Evan Wallace, [How Figma's multiplayer technology works](https://www.figma.com/blog/how-figmas-multiplayer-technology-works/)
  (l'annulation qui réécrit la pile de rétablissement au moment où l'on défait ; les identifiants par
  client ; à la reconnexion, une copie fraîche puis les gestes hors ligne).
* Liveblocks, [How to build undo/redo in a multiplayer environment](https://liveblocks.io/blog/how-to-build-undo-redo-in-a-multiplayer-environment)
  (une pile par client ; un nœud retiré : rien, comme Figma et Google Slides).
* tldraw, [TLSyncClient](https://tldraw.dev/reference/sync-core/TLSyncClient) (push, pull, rebase) ;
  [une lecture de son architecture](https://grida.co/docs/wg/research/crdt/tldraw).
* [Secsync](https://github.com/serenity-kit/secsync) : un serveur aveugle, des instantanés et des mises
  à jour chiffrées ; ce qu'un serveur aveugle peut encore faire (omettre, rejouer).
* Ce qui a paru en 2026 : [NextGraph au FOSDEM 2026](https://fosdem.org/schedule/event/J3ZBYC-nextgraph-sync-engine-sdk-reactive-orm),
  [Evolu](https://toolradar.com/tools/evolu-local-first), Epicenter (XChaCha20-Poly1305 par valeur).
* Glucose Tauri, `src/multiplayer/localUser.ts` (COLLAB-1 : le nom tiré, la couleur parmi dix,
  gardés, modifiables).

## 9. Les leçons

* **Écrire le fichier des sabotages avant de les lancer** a encore montré des gardes sans épreuve —
  six — et une garde que la mathématique rend vide.
* **Un `pkill -f` dont le motif est dans sa propre commande la tue** : la passation le disait, je l'ai
  fait quand même (code 144). `ps -eo pid,args | grep '[m]otif'`, puis `kill`.
* **Un remplacement de texte à l'aveugle déborde** : `self.partage` → `self.lancement.agents.partage` a
  aussi changé `self.partager_ce_canevas()`. Et `cargo fmt` remet en page : relever les motifs après.
* **Un essai scripté peut ne pas faire ce qu'il prétend** : le premier « conflit » n'en était pas un.
  Regarder chaque image, pas seulement le code de retour.
* **`nix shell -c` laisse vivre son enfant** : tuer le numéro de `nix` laisse tourner le programme. Un
  outil de fond se retrouve par `ps`, et un essai refuse de démarrer si son port est déjà pris.
* **Un essai qui échoue par l'outil peut quand même trouver un défaut** : la salle vide qui ne disait
  rien.
