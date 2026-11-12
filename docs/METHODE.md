# Comment on travaille sur Glucose

> **Au 11/11/2026** (fiche 104). Les règles du code, la façon de prouver, les principes que ce dépôt a
> payés, et les gestes de cette machine. **La charte** (100 images par seconde, aucune machine exclue,
> deux voies, l'élégance) est dans [VISION](VISION.md) § 2 ; **la façon de travailler avec lui**, dans
> [`CLAUDE.md`](../CLAUDE.md). Les leçons une par une, avec leurs cas : [`carnet/lecons.md`](carnet/lecons.md).

---

## 1. Les règles du code

* **Rien à moitié** (R1) : une fonctionnalité est finie quand elle est **branchée, visible, annulable et
  enregistrée**. Un bouton qui ne fait rien encore le dit ; il ne fait jamais semblant.
* **Glucose Tauri dit *quoi*, jamais *comment*** (R2) : les fiches 06 à 10 en sont la spécification ; son
  code n'est pas un modèle.
* **Les cliquets** (`crates/glucose-desktop/tests/cliquets_suite.rs`) : **80 lignes par fonction, 600 par
  fichier**, le couplage du bureau au modèle (passer par le noyau), les toasts, les modules sans appelant,
  les marques de mesure. Un cliquet qui proteste demande une **extraction**, jamais un plafond relevé.
* **La fiche 05** reste la référence du reste : un gestionnaire d'événements ne contient aucune logique ;
  toute modification passe par le journal ; le rendu lit, il ne calcule pas, et ne touche jamais le disque.
* **Le noyau (`glucose-core`) n'a aucune dépendance** ; une dépendance ailleurs exige sa note dans
  `docs/carnet/decisions/`.
* **Une constante arbitraire qui peut disparaître disparaît** ; une qui ne le peut pas se dit « nombre de
  ressenti », avec le journal de ses valeurs essayées (exemple : `elan.rs`).
* **Les invariants portent un nom** (MEMB-1, LETTRES-1…) et un commentaire qui dit **pourquoi** ; le code
  cite les fiches par leur numéro, qui ne change jamais.

## 2. Prouver

* **« Réglé » seulement vérifié** : sur une copie de ses documents, sur GitHub, ou à son écran — jamais
  parce que ça compile, ni parce qu'une épreuve isolée passe.
* **Une épreuve par invariant, sabotée** : `outils/saboter.py` casse la garde, lance les épreuves, exige
  qu'elles tombent, restaure. Un fichier de sabotages par chantier, dans `outils/sabotages/`.
* **Une épreuve porte sa preuve à l'envers** : elle rejoue l'ancienne loi sur les mêmes données et vérifie
  qu'elle concluait l'inverse.
* **Les deux voies au bit près**, ou à un écart mesuré, borné et expliqué (`tests/voies_suite.rs`).
* **Regarder l'image** : au pilote, dans le vrai Glucose sans fenêtre, chaque capture regardée.
* **Les machines qu'on n'a pas** : la CI (dix tâches) les compile et les éprouve — avant une publication,
  pas à chaque envoi.

## 3. Ordonner son travail

* **Regarder avant de saboter** : une loi que l'image va corriger, sabotée d'abord, c'est du travail perdu
  (fiche 103).
* **La recherche avant la conception** : sur internet, ses sources rapportées dans la fiche.
* **Une seule grosse compilation à la fois** (7,6 Go) : la suite, puis Nix, puis un prototype, puis les
  sabotages ; jamais deux.
* **Un point en français simple toutes les 30 à 45 minutes** ; l'heure se **lit** (`date`), ne s'estime
  pas.
* **Prototyper hors du dépôt** (un arbre détaché dans le scratchpad, un correctif sauvé dans
  `~/.cache/glucose-bancs/`) ; prendre sa parole **sur une image** avant d'écrire ce qui engage sa vision.

## 4. Les principes — ce que les leçons ont appris

Chacun résume des fautes réelles ; les fiches citées en gardent les cas ([`carnet/lecons.md`](carnet/lecons.md)).

**Les épreuves**

1. **Une épreuve doit tomber quand on casse ce qu'elle garde.** Toute épreuve neuve se lance d'abord sur le
   code intact, **puis** se sabote ; la commande du sabotage doit **exécuter** l'épreuve visée (fiches 71,
   75). Un sabotage qui passe : chercher **l'entrée** qui le ferait tomber, et **le lecteur actuel** de la
   garde, avant de la retirer — il a parfois caché un vrai défaut (fiches 57, 65, 99, 104).
2. **Une épreuve exerce son cas** : des données où les deux lois divergent (une scène de 26 liens ne
   distingue pas « compter les liens » de « compter les nœuds »), des flottants qui ne s'écrivent pas en
   binaire, plusieurs colonnes quand on éprouve des coupes (fiches 66, 74, 96, 104).
3. **Une épreuve passe par les vrais chemins** : la souris (`handle_cursor_moved`), la touche
   (`handle_shortcut_input`), l'image redessinée entre deux gestes (`une_image_sans_fenetre`) — une épreuve
   qui joue une image par pas ne voit pas les images immobiles du vrai programme (fiches 65, 67, 99, 103).
4. **Une épreuve tombe, elle ne bloque pas, et ne croit pas l'horloge** : ce qui attend s'éprouve sur un fil
   avec un délai ; un temps qui n'avance pas se fixe (fiches 64, 66, 68, 75).
5. **Une forme a une seule loi, que les deux voies appellent** ; une égalité promise s'obtient en appelant,
   jamais en récrivant ; comparer deux voies qui appellent la même fonction ne la voit pas (fiches 71, 73).
6. **Une borne se déduit de ce que la construction ou le matériel garantit** (1/2⁸ de texel, le huitième de
   pixel), jamais supposée ; une épreuve de temps qui tombe dit que la manière doit changer, jamais la
   borne (fiches 73, 88).
7. **Une loi neuve qui contredit une épreuve ancienne** : trancher, écrire, réécrire l'attendu — jamais
   relâcher la tolérance (fiche 99).
8. **Des épreuves vertes ne prouvent pas que ça marche** : un essai réel avant « fini » — le vrai programme,
   sur de vrais fichiers, la vraie carte de Wikipédia (fiches 82, 88, 97, 99, 103).

**Mesurer**

9. **Mesurer avant de supposer** : chronométrer chaque étape, profiler **avec les appelants**, compter
   (demandes, ratés) ; la première hypothèse est souvent fausse — dire « probablement, à confirmer »
   (fiches 53, 71, 97, 103, 104).
10. **Une marque de mesure absorbe ce qui la précède ; un compteur jamais lu vaut zéro** ; lire médiane,
    99ᵉ centile et pire, ne jamais additionner des centiles (fiches 53 à 63).
11. **Comparer en paires entrelacées**, dans la même exécution : le même programme varie d'un facteur 3
    d'un jour à l'autre (secteur, batterie) ; comparer à la version **publiée**, et regarder l'image de
    l'écart avant de mettre une empreinte témoin à jour (fiches 87, 95, 103).
12. **Une mesure trop courte fabrique une conclusion** ; mesurer au-delà de ce que l'hypothèse annonce, et
    exiger que la mesure ait **eu lieu** (zéro réponse illisible) avant de la juger (fiches 76, 77, 98).

**Concevoir**

13. **Regarder la vue d'ensemble ET la profondeur, et le geste entier** : un prototype qui corrige une vue en
    casse souvent une autre ; une loi qui passe ses épreuves peut faire une pelote sur la vraie carte
    (fiches 99, 102, 103).
14. **Une borne choisie se périme** : préférer une loi sans capacité (« oublier ce que l'image d'avant n'a
    pas demandé ») et une formule exacte à un seuil — un seuil qui tranche signale une formule qui manque
    (fiches 54, 63, 72, 104).
15. **Une seule source pour un état, une seule liste, un seul ordre des couches, une seule question au
    noyau** ; ce qui se compare se nomme pareil des deux côtés (fiches 59, 60, 67, 68, 88).
16. **Une règle à la sortie plutôt qu'une garde à chaque entrée** ; une garde redondante, ou que la
    mathématique rend vide, se retire (fiches 57, 65, 68, 73).
17. **Demander lequel commande le symptôme**, pas lequel le produit (les cercles vicieux) ; une voie par
    source, jamais un réglage commun ; ce qui bouge la caméra avance au moment de l'image (fiche 53).
18. **Tout travail « pour l'image suivante » demande cette image, et finit de lui-même** ; au repos, le
    travail se fait hors des images, et une seule image montre le tout (fiches 63, 95, 98).
19. **Une ressource qui se rend passe devant celle qui se prend** ; tout état d'une ressource entre dans un
    compte ; ce qu'on demande à la carte graphique se borne par ce qu'elle annonce (fiches 59, 66, 83).
20. **Lire la plateforme et la bibliothèque jusqu'au bout** avant d'écrire un pont : le cycle de vie
    d'Android, le `Drop` d'un presse-papiers, `cfg(target_os = "linux")` qui n'est pas Android, une fenêtre
    cachée qui bloque l'extinction (fiches 56, 60, 66, 82).
21. **Chaque chemin qui écrit des données a son chemin de retour** ; un repli ne répond jamais à la place de
    l'utilisateur ; une identité ne se tire pas d'un nom qu'on peut changer (fiches 56, 58).
22. **Relire ce qu'un chantier retire, plateforme par plateforme, et ce qu'il réveille dans l'ancien**
    (fiches 60, 76) ; comparer un panneau à Tauri champ par champ avant de le dire porté (fiche 62).
23. **Un document ou un commentaire qui affirme se vérifie dans le code** : un nombre se compte (« 78
    commandes » : 105), un défaut se relit avant d'être pris (fiches 71, 72, 82, 104).
24. **Aux modèles d'IA** (Protéo) : des questions fermées sur un seul axe ; lire au journal à quel modèle
    chaque question est allée ; chercher d'abord dans le catalogue de ce qui exécute (fiches 76 à 78).

## 5. Les gestes de cette machine

Son VivoBook sous **NixOS, Wayland et niri** (depuis le 6/11/2026) ; la carte de la machine :
`~/Projets/LISEZ-MOI.md`. Le PC Windows d'avant : [`carnet/ancien-pc-windows.md`](carnet/ancien-pc-windows.md).

**Compiler, éprouver**
* Rust n'existe que dans `nix develop -c …` ; `cargo test --workspace` compte 5 à 8 minutes de compilation.
* **Toute commande lourde est bornée** : `bash outils/borne.sh <commande>` — 60 % de la mémoire, sans
  swap, une compilation par 1,5 Go de borne. Le 12/11, une suite nue (huit `rustc` à la fois) a noyé
  les 7,6 Go dans le swap compressé, où Linux ne tue rien : sa machine figée de 14 h 36 à 17 h 05
  (fiche 110). Bornée, elle meurt seule, en une seconde (code 137) — alors baisser `CARGO_BUILD_JOBS`.
* **Nix ne voit que ce que git connaît** : un fichier neuf que le paquet nomme empêche `nix develop` de
  s'ouvrir tant qu'il n'est pas ajouté (`git add`).
* **Pendant qu'une suite tourne, n'écrire aucun `.rs` dans le dépôt** : les cliquets lisent tous les
  fichiers, même ceux qu'aucun `mod` ne nomme. Préparer dans le scratchpad, recopier d'un coup.
* **Les sabotages, à part** : `git worktree add --detach <scratchpad>/sab HEAD`, puis dans l'arbre
  `CARGO_TARGET_DIR=~/.cache/glucose-bancs/sabotage-cible CARGO_PROFILE_DEV_OPT_LEVEL=0
  CARGO_PROFILE_DEV_DEBUG=0 nice -n 19 nix develop /home/shaza/Projets/GlucoseGit -c python3
  outils/saboter.py outils/sabotages/<x>.json`, en fond, avec un réveil sur sa sortie et sa durée dite
  avant ; retirer l'arbre ensuite. **Jamais éditer un fichier qu'un sabotage touche** : il restaure ce
  qu'il a lu. Un script de remplacement s'écrit **après** `cargo fmt`.
* **Les épreuves tournent en parallèle** : deux qui écrivent le même fichier temporaire tombent au hasard
  (nommer par l'empreinte). Une épreuve graphique se saute sans carte ; `GLUCOSE_EXIGER_UNE_CARTE` la fait
  tomber sur GitHub. Les épreuves de la mémoire offerte tombent sous forte pression mémoire : les rejouer
  seules.
* **Un témoin de format s'écrit par la version d'avant** : un arbre détaché au dernier commit qui produit ce
  schéma, une épreuve jetable, sa propre cible, puis `git worktree remove --force`.
* `undefined reference` après une coupure : `cargo clean -p glucose-desktop` ; `target/` grossit vite
  (`cargo clean` ne coûte qu'une recompilation).

**Le vrai Glucose, hors écran**
* **Le pilote** (fiche 99) : `cargo build --release -p glucose-desktop --bin glucose-desktop`, puis `bash
  outils/pilote/piloter.sh demarrer <maison> <sortie> [VAR=valeur…]` (le document : le dernier brouillon de
  `<maison>/state/glucose/brouillons`), puis `clic|double|droit|aller <x> <y>`, `presser|relacher`,
  `milieu-presser|milieu-relacher`, `glisser <x0> <y0> <x1> <y1> <ms> <hz>` (sans bouton : un simple
  mouvement), `molette <crans> [<ms>]` (négatif : zoomer), `taper`, `touche|ctrl|alt|altmaj <t>`, `capture
  <f.png>`, `arreter`. **`glisser` et `molette` rendent la main tout de suite** : attendre leur durée.
  Attendre que Glucose soit prêt (une capture noire = trop tôt). `GLUCOSE_EXE=<binaire>` pour un autre
  programme ; `GLUCOSE_PERF=1` : une ligne par image dans `sortie.txt`.
* **Forcer la voie du processeur au pilote** : écrire `version=2.0.3-dev\ntombe=vulkan\ntombe=gl\n` dans
  `<maison>/state/glucose/moteurs.txt` ; sans cela, il dessine par la carte graphique.
* **La copie de sa carte de Wikipédia avec ses liens** : `~/Documents/Glucose-essai-wikipedia-liens.glucose`.
  Le squelette se pose par `outils/wikipedia/essai_squelette.sh`.
* **Le rythme ne se juge qu'à son écran** (fiches 111-112) : sous sway sans écran, présenter ne
  bloque jamais ; sous niri imbriqué, niri bloque sur son hôte et date ses retours à l'envoi — il
  ne sait pas juger un rythme. Le juge est le compositeur : la section « Ce que l'ecran a montre »
  de la chronique (PARU-1). Les « images qui manquent l'heure du tempo » sont celles du tempo,
  sur sa grille, **pas l'écran**.
* **Sous niri** : `PILOTE_COMPOSITEUR=niri` ; **à la taille d'un téléphone** : `PILOTE_ECRAN=1080x2400
  PILOTE_ECHELLE=2.75` (son moto g54). **Le presse-papiers et le glisser de Wayland** : `bash
  outils/pilote/essai_wayland.sh <dossier>`.
* **Mesurer sans le déranger** : `bash outils/banc_hors_ecran.sh <scénario> <sortie>` ; comparer deux
  programmes par `GLUCOSE_EXE`, en paires entrelacées. Les scénarios : `outils/scenarios/` ; le tableau de
  8 000 images : `~/.cache/glucose-bancs/` (`fabriquer_un_grand_tableau`).
* **Profiler** : `nix shell nixpkgs#perf -c perf record -e cpu-clock:u --call-graph dwarf,16384
  --user-callchains -p <pid>`, puis `perf report -G` ; `strace -T` sur le fil qui dessine dit ce que
  `perf` ne voit pas.
* **Comparer des images** : `nix shell --impure --expr '(import (builtins.getFlake "nixpkgs")
  {}).python3.withPackages (p: [p.pillow])' -c python3 …`, en RVB (`convert('RGB')`).
* **Les commandes du terminal, en réel** : `nix develop -c bash outils/essai_terminal.sh <dossier jetable>`
  (une prise Unix tient en 108 caractères : un chemin du scratchpad est trop long).
* **Faire tomber un moteur** : `GLUCOSE_TOMBER=vulkan` (ou `gl`).

**À son écran**
* **Ses vrais gestes, mesurés** (fiche 112) : `bash outils/pilote/essai_ecran.sh <dossier>
  [VAR=valeur…]` — une fenêtre ~75 s, la souris par le pointeur virtuel de niri, **le pavé par un
  pavé virtuel** (`pave_virtuel.py`, uinput : pincer, deux doigts), sa carte copiée, la chronique
  et sa section PARU-1 dans le dossier ; `LOURD='pincer 1.8 700\n…'` zoome d'abord. Annoncer le
  nombre de fenêtres ; l'outil met Glucose au premier plan, flottant, et vérifie le focus (par le
  numéro du processus) avant chaque geste.
* **Un essai réel** : `bash outils/essai_isole.sh <secondes> [capture.png]` — dossier neuf, télémétrie coupée
  d'avance ; annoncer la fenêtre (elle prend le focus) et le nombre **exact** de fenêtres.
* **Sa chronique** : `/tmp/glucose-chronique/derniere-session.txt`, et son journal technique
  `~/.local/state/glucose/journal-technique/` — les **copier**, puis les lire.
* **Son Glucose installé** : le profil Nix `GlucoseGit` ; après un envoi qui change le programme, `bash
  outils/mettre_a_jour.sh` — `nix profile upgrade --refresh GlucoseGit` sous un chien de garde : la
  compilation se fait dans le service de Nix, hors de `borne.sh` (fiche 110).
* **Son mot de passe** (une commande système) : `pkexec bash <script>` ouvre la fenêtre
  « Authentification » au milieu de l'écran ; le script commence par `source /etc/set-environment`
  (pkexec vide l'environnement), et se simule d'abord dans `env -i`.

**GitHub, la télémétrie, les autres machines**
* **Envoyer** : `git push "https://x-access-token:$JETON@github.com/shazamifius/GlucoseGit.git" main:main`,
  avec `[skip ci]` ; le message de commit dans un fichier du scratchpad, puis `git commit -F`. **Suivre la
  CI** : `GH_TOKEN=… bash outils/suivre_ci.sh <empreinte complète> <sortie.txt>`. Avant d'envoyer un
  fichier de `.github/workflows/` : `nix shell nixpkgs#actionlint -c actionlint`.
* **La télémétrie** : `node outils/telemetrie/tableau.mjs <dossier>`, en début de session. Une erreur 7403 :
  `npx wrangler whoami` rafraîchit le jeton ; la connexion, dans `CLAUDE.md`. **Le serveur avant le
  client** : une ligne neuve se déploie (`npx wrangler deploy`), se prouve en ligne une minute plus tard,
  et seulement ensuite part dans une version (`outils/publication/verifier_serveur.py` arrête l'oubli).
  Les épreuves du serveur : `node --test outils/telemetrie/test/valider.test.js`.
* **Une commande qui déploie ou publie ne s'enchaîne qu'avec `if`**, jamais `;` (fiche 86).
* **Windows, Mac, Android** : seule la CI les compile et les éprouve ; une correction propre à l'un part
  donc sans essai réel, et se dit telle. L'APK : `bash outils/android/construire.sh` (le SDK et le NDK sont
  absents ici : la CI le construit). `/dev/kvm` existe pour un téléphone virtuel ; les 4 Go de l'appareil
  seront serrés.
* **Glucose Tauri** : `git worktree add --detach <scratchpad>/tauri origin/tauri-v1.0.1`, retiré en sortant.
* **Arrêter un processus** : pas de `pkill -f` dont le motif figure dans sa propre commande.

## 6. À la fin d'une session

1. **ÉTAT** se réécrit au présent (pas une ligne de journal en plus) ; **SUITE** perd ce qui est fini.
2. Une **fiche** datée dans le carnet (le numéro suivant, jamais renuméroter), et sa ligne dans
   [`carnet/00-INDEX.md`](carnet/00-INDEX.md) ; ses paroles nouvelles, dans **VISION**, à leur thème.
3. **Une seule** liste d'essais à son écran, courte, avec la commande qui écrit la sortie dans un fichier.
4. S'il demande une passation : un prompt qui dit quoi lire (`CLAUDE.md` mène au reste), les pièges payés,
   ce qui attend sa parole — le jeton GitHub qu'il donne, sans remarque, et **jamais écrit dans le dépôt**.
