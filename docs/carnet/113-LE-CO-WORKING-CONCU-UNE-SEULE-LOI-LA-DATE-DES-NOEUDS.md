# 113 — Le co-working conçu, une seule loi pour ce qui s'applique, la date des nœuds commencée

> Session du 12/11/2026, de 22 h 42 au 13/11 vers 0 h 50, arrêtée à sa demande (*« la session
> commence malheureusement à être trop longue »*). La passation (fiche 112) donnait l'ordre : la
> publication, son Glucose mis à jour, la page du co-working, puis la porte publique et son lag.
> **Envoyé** : APPLIQUE-1 (`fba34da`), puis les documents. **Pas dans `main`** : le bleu des liens
> et la date des nœuds, gardés en correctif (§ 6).

## 1. La publication, son Glucose

* **La 2.0.4-beta.1 est toujours en brouillon** (`/releases/latest` rend la 2.0.3-beta.10 ; le
  brouillon v2.0.4-beta.1, onze fichiers) : son clic, rappelé.
* **Son Glucose installé passe à `b44113c`** (PARU-1) par `outils/mettre_a_jour.sh` : 7 minutes
  de compilation dans le service de Nix, le chien de garde jamais déclenché (2,1 Go libres au plus
  bas). Ses prochaines sessions écrivent « Ce que l'ecran a montre ».
* **La télémétrie** : inchangée (166 sessions, 9 installations, 33 fins anormales) ; l'erreur
  7403 s'est levée par `npx wrangler whoami`, comme MÉTHODE § 5 le dit.

## 2. Le co-working — la page, à sa parole

<https://claude.ai/artifact/SLp5cJzTb7FvsKp7iEtUpS> (source : `~/.cache/glucose-bancs/coworking-113/`).

* **Ce que Glucose Tauri faisait** (relu dans `tauri-v1.0.1`, `src/multiplayer/`) : le panneau
  « Collaborer » (créer une chaîne, la rejoindre par un code `automerge:…`, « Mon identité » — un
  nom tiré au sort, modifiable, une couleur parmi dix, `localUser.ts`) ; les curseurs des pairs
  (`PeerCursorsLayer.tsx`) ; les octets des photos par un second document Automerge
  (`assetChannel.ts`, nommés `sha256(octets)[..16]`) ; et, en Rust, un multijoueur de réseau local
  (mDNS et WebSocket, `src-tauri/src/multiplayer.rs`). **Tout passait par `sync.automerge.org`**,
  le serveur public d'Automerge, qui gardait le document entier en clair — ses auteurs le disent
  fait pour les démos, « no reliability or data safety guarantees ». Les commentaires de son code
  racontent ce qu'Automerge a coûté : un gel de 5 s quand les photos étaient dans le document, et
  un `Ctrl+Z` figé par la réentrance du WASM.
* **La thèse** : partager un canevas, c'est partager sa Time Machine. Le journal garde déjà le
  nœud entier avant et après (`Slot`), chaque geste est signé (MAIN-1), chaque photo a son
  empreinte (`core/hash.rs`), et l'histoire du disque est une suite chaînée en ajout seul
  (`persist/histoire.rs`) — la forme d'un historique de git.
* **Trois façons de faire** : A1, l'arbitre qui lit (Figma, tldraw) ; A2, sans arbitre, une version
  par nœud (Excalidraw : la salle ne garde rien, mais chacun voit les gestes dans son ordre, donc
  autant de Time Machines que de participants) ; **A3, le greffier aveugle** (proposé) : la salle
  numérote les gestes et les garde chiffrés sans les lire ; chaque Glucose rejoue les mêmes gestes
  dans le même ordre — la réplication d'une machine à états par un ordre total, la « server
  reconciliation » de Weidner et de Replicache, la réconciliation faite par chaque client puisque
  la salle ne lit rien.
* **Les conflits** : le geste dit le nœud par son nom et seulement ce qui change ; le dernier
  numéro gagne (Figma) ; **l'avant que le journal garde pour `Ctrl+Z` détecte le conflit** — un
  avant qui ne correspond plus au canevas, c'est que quelqu'un est passé ; la Time Machine le
  montre. Un texte écrit à deux : une fusion à trois comme git. Aucune horloge, aucun compteur par
  nœud.
* **Remis en question, contre la passation** : les indices fractionnaires de l'ordre de dessin
  ne servent que sans ordre total ; avec une salle qui ordonne, « mets-le devant » se rejoue pareil
  partout — la machinerie disparaît. Et les identifiants des nœuds sont un compteur (`txt-43`,
  `store/ids.rs`) : deux Glucose donneraient le même ; chaque participant aura sa série (Figma).
* **Le prix**, relu sur les pages de Cloudflare : 100 000 requêtes par jour, un message entrant
  compte pour 1/20, les sortants sont gratuits ; 5 Go pour tous les Durable Objects. Environ 160
  heures de co-working par jour pour tous les utilisateurs (un curseur à 10 messages par seconde,
  qui bouge un tiers du temps) ; sa carte de Wikipédia pèse 2,1 Mo, histoire comprise : 2 400
  canevas comme elle. Les photos : d'un participant à l'autre par la salle, sans y être gardées
  (R2 : 10 Go pour tous, plein en un mois).
* **Ses cinq questions** : A3 ? le dernier gagne et le conflit montré (la revue des conflits en
  option) ? les photos B1 ? deux codes (écrire, lire seulement) et son identité comme dans Tauri ?
  la porte publique d'abord ?

## 3. APPLIQUE-1 — une commande s'applique-t-elle ? (`commandes/applique.rs`)

Le bouton qui ment de la fiche 111 § 4 : la palette proposait « Couleur » sur une flèche, qui ne
fait rien. En cherchant, **trois règles pour une seule question** : le menu du clic droit avait ses
conditions (`ui/context_menu/entrees.rs`), les gestes les leurs (rendre `false` en silence), la
palette aucune.

* **Une seule loi**, sans joker : chaque commande est rangée — toujours offerte, selon le document
  (annuler, rétablir, les vues, les cases d'un storyboard, le mode référence, le journal), ou selon
  le choix (le reste). `s_applique(store, ui, commande)` pour le menu, qui se dessine depuis le
  document et l'interface ; `GlucoseApp::s_applique` y ajoute ce que Glucose seul sait (le texte
  qu'on écrit, dont `Ctrl+Z` défait la frappe ; le focus d'une membrane).
* **Trois lecteurs** : la palette n'offre que ce qui s'applique (pour changer un raccourci, toutes
  restent) ; le menu filtre ses entrées par la loi, puis retire les filets qui ne séparent plus rien
  — ses conditions à lui ont disparu, sauf ce qui est propre à un endroit (le dossier seul, le
  libellé du verrou) ; l'exécution refuse ce que la loi tait.
* **Absente, pas grisée** : la loi du menu (« griser, c'est montrer une porte fermée ») ; la palette
  de VS Code tait de même une commande dont la condition est fausse.
* **« S'applique », c'est : ce qu'elle change existe, et elle le changerait.** Une couleur ou un
  opérateur que tout ce qui est choisi porte déjà est sans objet. **Deux exceptions, voulues** : la
  relation d'une flèche reste offerte même déjà portée — sinon la touche « 1 » volerait au signet 1,
  la commande suivante de son accord ; l'ordre (premier plan) et les colonnes s'appliquent dès
  qu'ils ont un objet, un ordre ou une disposition déjà atteints ne se calculant qu'en les faisant.
* **La fenêtre des liens** (LIEN-3) : une commande qui ne la regarde pas la ferme d'abord, même pour
  ne rien faire ensuite (`Suppr` la ferme) — l'exécution ferme, **puis** demande à la loi ; et la
  loi, fenêtre ouverte, tait ce qui dépend du choix, qui partira avec elle.
* **Une seule question au noyau** : `Store::peut_grouper_la_selection` (la boîte du groupe, la loi
  que `grouper` emploie), `Store::peut_montrer_les_liens` (que `montrer_les_liens` pose avant de
  construire), `HistoireDeLaVue::peut_avancer` ; au bureau, `candidats_a_montrer`,
  `noeud_montre_choisi`, `une_fleche_seule`, que le geste et la loi appellent tous deux.
* **Le menu, regroupé par sens** (l'édition ; « Grouper » et les régions ; l'image ; les
  opérateurs ; les images ; l'ordre ; « Supprimer ») : l'ancien découpage tenait au hasard des
  conditions — un filet n'existait que si l'entrée d'après existait. Le témoin du menu a changé de
  15 354 pixels, tous dans le menu : regardé avant/après, agrandi ; son empreinte mise à jour.
* **Prouvé** : cinq épreuves (`commandes/applique/tests.rs`) — le défaut rapporté par la vraie
  palette, écrite au clavier ; la palette et le menu lisent la loi dans neuf scènes (rien, un texte,
  la flèche, le post-it, la membrane, l'image, un texte et la flèche, la fenêtre des liens vide ou
  une copie choisie) ; **la loi dit vrai** — dans chaque scène, pour chaque commande qui n'est pas
  « toujours » (une liste écrite dans l'épreuve, pas lue dans la loi), le geste nu ne change rien
  quand la loi le tait et change quelque chose quand elle l'offre, et l'exécution refuse ce qu'elle
  tait ; les chiffres gardent leur loi ; le menu d'une sélection sans flèche ni post-it. Cette
  épreuve a trouvé deux vrais cas en route : `Suppr` ne fermait plus la fenêtre des liens, et
  « Journal technique » dépend du serveur du programme, qu'aucune épreuve n'a (rangé « système »
  dans l'épreuve, avec le presse-papiers, l'explorateur et le mode référence).
* **Sabotages** (`outils/sabotages/applique-113.json`) : **13 sur 13** — l'exécution, la palette et
  le menu qui ignoreraient la loi, la fenêtre des liens, une couleur ou un opérateur déjà portés,
  « Aller à ce nœud », « Vue suivante », sortir d'une membrane, les deux questions au noyau,
  « Suivre le lien », et l'ordre de l'exécution (la loi avant la fenêtre : `Suppr` ne la fermait
  plus). **Deux sabotages passaient, et avaient raison** : ils cassaient le retrait des filets en
  double ou en queue — or la mise en page du menu (`en_colonnes`) ne pose déjà un filet qu'entre
  deux groupes non vides. La garde était redondante ; elle est retirée (MÉTHODE § 4, principe 16).
* **Une leçon payée en route** : un arbre détaché qui partage la cible du dépôt l'empoisonne —
  Cargo a compilé le bureau contre le noyau de l'arbre détaché (quatre erreurs sur des fonctions
  qui existaient pourtant) ; `cargo clean -p glucose-core -p glucose-desktop` l'a réparé.
  MÉTHODE § 5 le dit désormais.

## 4. Le bleu des liens (LIEN-BLEU-1, `theme.rs`)

Fiche 111 § 4 : « à trancher sur une image ». Recalculé (APCA 0.0.98G-4g) sur le papier des textes
`#0d0d0d` : `#60a5fa` (le bleu de Tauri) Lc 52, `#7ab5ff` Lc 60, `#a8cfff` Lc 75, le corps
`#dce1eb` Lc 88. La norme (ARC, le mode « bronze ») : **un texte courant demande Lc 75 au moins**,
90 de préférence ; Lc 60 ne vaut que pour un texte qui n'est pas du corps. Un lien dans un
paragraphe est du corps. **Regardé** (`~/.cache/glucose-bancs/date-113/liens.png`, agrandi dans
`petit.png`) : un paragraphe en Inter à 13, 16 et 20 pixels, les trois
bleus côte à côte, agrandi — à 13 pixels, le bleu de Tauri se lit nettement moins bien que le
texte autour ; `#a8cfff` se lit comme lui, et reste un lien (bleuté, et souligné de sa couleur
par `draw_underline`). **Tranché : `#a8cfff`**, la même teinte (Oklch h = 254,6 → 253,9, l'arrondi
des octets). Le témoin (`synth::witness`) porte un lien : ses empreintes changeront — à regarder avant de les
mettre à jour (§ 6).

## 5. DATE-1 — dater les nœuds (la temporalité de Tauri), commencée

**Pourquoi maintenant** : sa parole du 12/11 (*« se focaliser sur les boutons manquants »*), et un
fait trouvé en chemin — **les dates que Glucose Tauri posait sont lues et enregistrées par Glucose
Rust** (`persist/tauri/projet/annotations.rs`, `persist/annotation.rs`), **et jamais montrées**. Un
utilisateur de Tauri qui datait ses nœuds les a perdues de vue : la première condition de la
2.1.0 (SUITE § 1). Le noyau savait déjà lire et écrire une date (`timeline.rs` : `parse_anchor`,
`format_anchor`, les ères `DEFAULT_ERAS`, `node_matches_temporal_filter`, `tick_step`) ; aucun
geste ne s'en servait.

* **Ce que Tauri faisait** (relu dans `tauri-v1.0.1`) : `Maj+T` — « ANCRAGE TEMPOREL · n nœuds »,
  un champ (« 1789 · -500 · Renaissance · 10 ka »), les ères en puces (six sans rien d'écrit, huit
  au plus, filtrées par ce qu'on tape), « Retirer l'ancrage » (`TemporalAnchorPrompt.tsx`) ; une
  pastille en bas à droite du nœud, débordant de 10, fond `rgba(15,15,25,.9)`, encre `#fde68a`,
  filet `#fbbf24` à 60 % (`AnnotationBadges.tsx`, `TemporalBadge`) ; `Maj+R` — la réglette en bas
  de l'écran (88 px, les ères en bandes de couleur, les années graduées, une fenêtre jaune entre
  deux poignées, la molette zoome ; `TemporalRuler.tsx`, l'image `screens/12_temporal_ruler.png`) ;
  ce qui sort de la fenêtre s'estompe (alpha 0,12 pour les photos ; `GlucoseCanvas.tsx`).
* **Écrit, dans le correctif** : le noyau — `Store::ancrer_dans_le_temps` et `dates_choisies`
  (`store/dates.rs`), `Annotation::temporal_anchor(_mut)`, trois épreuves (**vertes**) ; le bureau —
  la commande « Dater… » (`Maj+T`, la palette, le menu d'une sélection, la loi APPLIQUE-1 : selon
  le choix), la question (`interactions/dater.rs` : le champ porte la date du premier nœud daté,
  les ères suivent ce qu'on tape sans accents, « Retirer la date » quand quelque chose est daté,
  une date illisible se redemande avec ce qu'on avait tapé), quatre épreuves par les vraies
  entrées ; la pastille (`renderer/date.rs`) posée dans `voies::dessiner_sur_les_photos`, la moitié
  que les deux voies partagent (ORNEMENTS-1), à la boîte que le noyau donne aux flèches
  (`arrow::emprise`), à l'échelle du monde ; deux épreuves de pixels (l'ambre au coin du nœud daté,
  rien sans date, de loin il reste de l'ambre).
* **La convention** : celle des historiens, sans an zéro (« 500 av. J.-C. » = « -500 ») — l'ISO
  8601 et les astronomes en ont un ; Glucose suit le noyau, qui suit Tauri. À dire si un jour une
  date s'exporte en ISO.
* **Reste** : le verdict des épreuves du bureau sur le correctif (§ 6) ; les sabotages (DATE-1) ;
  la pastille sur la voie graphique **vérifiée** (`tests/voies_suite.rs` : poser un nœud daté dans
  une scène des deux voies) — la couche du dessus de la carte graphique est vide sans rien de
  choisi (ARCHITECTURE § 4) : **vérifier qu'elle porte bien les dates** ; l'export SVG (la date n'y
  est pas) ; un agent qui date (`glucose_poser_texte` et les autres : un champ `date`) ; puis la
  **réglette** (`Maj+R`) et le **filtre** qui estompe — sur les deux voies.

## 6. Où le travail s'est arrêté

* **Dans `main`, envoyé** : APPLIQUE-1 (`fba34da`), les documents.
* **Le correctif** `~/.cache/glucose-bancs/date-113/date-bleu-menu.patch` (`git apply`, depuis
  `main`) : le bleu des liens (§ 4), le retrait de la garde redondante des filets du menu (§ 3) et
  son sabotage mis à jour (13 sabotages au lieu de 15), DATE-1 (§ 5). Le noyau : 734 épreuves
  vertes. Le bureau : voir `verdict-bureau.txt` à côté du correctif. **Attendu** : les trois
  empreintes du témoin (`bench/tests.rs` : `EMPREINTE_TEMOIN`, `…_SELECTION`, `…_MENU`) tombent à
  cause du lien bleu — `cargo run -p glucose-desktop --example capture_temoin -- <dossier>`,
  regarder avant/après (l'ancien programme dans un arbre détaché **à sa propre cible**), puis les
  mettre à jour avec leur ligne d'histoire.

## 7. La recherche (consultée le 12 et le 13/11/2026)

Le co-working : les sources de sa page (Figma, Excalidraw, tldraw et son SDK 4.3, Replicache,
Matthew Weidner 2025, Eg-walker 2025, Liveblocks pour l'annulation et les curseurs,
perfect-cursors, Automerge et son serveur public, Cloudflare : prix et limites des Durable Objects,
R2, les relais publics d'iroh, Keyhive, Patchwork). Le bleu : [ARC, le mode bronze
d'APCA](https://a11yreadtech.github.io/ARC/tests/bronze-simple-mode) ; [l'introduction
d'APCA](https://github.com/Myndex/apca-introduction). La date : [ChronoZoom, du concept à la
réalité](https://sociostudies.org/almanac/articles/from_concept_to_reality_developing_a_zoomable_timeline_for_big_history)
(l'échelle logarithmique essayée puis abandonnée : rien ne se voyait entre deux puissances de dix ;
une échelle linéaire et un zoom profond) ; [Gephi Lite, le filtre de temps qui
estompe](https://github.com/gephi/gephi-lite/pull/280) (estomper plutôt que cacher, le clavier
pour déplacer la plage) ; [la numérotation astronomique des
années](https://en.wikipedia.org/wiki/Astronomical_year_numbering) (l'an zéro).

## 8. Les leçons

* **Une épreuve qui choisit quoi vérifier en lisant la loi qu'elle éprouve est aveugle** à la loi
  qui rendrait tout « toujours » : la liste des commandes « toujours offertes » s'écrit dans
  l'épreuve.
* **Un sabotage qui passe peut désigner une garde morte** : ici, la mise en page faisait déjà le
  travail ; la garde est partie, pas l'épreuve.
* **Un arbre détaché ne partage jamais la cible du dépôt** (MÉTHODE § 5).
* **Chercher ce que le modèle porte déjà avant de compter ce qui manque** : les dates de Tauri
  étaient lues et enregistrées depuis des semaines, invisibles.
