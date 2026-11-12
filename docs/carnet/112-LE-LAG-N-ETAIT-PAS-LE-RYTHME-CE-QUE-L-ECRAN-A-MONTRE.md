# 112 — Le lag n'était pas le rythme : ce que l'écran a montré

> Session du 12/11/2026, de 20 h 42 à la nuit. La passation (fiche 111) laissait le chantier du
> rythme : un prototype (`pre_present_notify`, niri donne le rythme) écrit, ni compilé ni mesuré,
> sur la foi de « 32 % des images en mouvement ratent leur balayage ». **Ce chiffre était faux** :
> c'était le compteur du tempo, pas l'écran. Mesuré à son écran par une sonde qui demande à niri
> ce qu'il a montré, le prototype est écarté, et la sonde entre dans Glucose (PARU-1).
> La 2.0.4-beta.1 attend toujours son clic.

## 1. Le danger annoncé, vérifié avant tout

La passation craignait qu'avec `pre_present_notify`, un appel d'agent attende indéfiniment
quand Glucose est caché. Lu dans winit 0.30.13 et dans niri :

* winit **retient** `RedrawRequested` tant que le rappel de niri n'est pas arrivé
  (`FrameCallbackState::Requested`) ; et `request_redraw` ne réveille la boucle **que si** son
  drapeau n'était pas déjà levé (`compare_exchange(false, true)`) — un drapeau retenu fait taire
  tous les réveils qui passent par lui ;
* niri n'envoie ses rappels qu'aux surfaces visibles ; les autres en reçoivent un par une
  minuterie de secours, **toutes les une à deux secondes** (`FRAME_CALLBACK_THROTTLE` = 995 ms,
  minuterie d'une seconde) — et **aucun quand l'écran est éteint** (`monitors_active`) ;
* la boîte des agents réveille la boucle par le **proxy** (`reveil_de` → `send_event`), qui réveille
  toujours, et la réponse se fait dans `about_to_wait` : elle n'aurait pas attendu. Mais
  **`consigner`** (JRN-5 : écrire au journal ce que le dernier geste a changé) vit dans `redraw()` :
  les gestes d'un agent dans un Glucose caché auraient attendu une à deux secondes pour s'écrire,
  écran éteint jusqu'à son retour. Le prototype écarté (§ 4), rien n'a été déplacé ; si un jour le
  compositeur donne le rythme, `consigner` sort d'abord du chemin de l'image.

## 2. La sonde — et pourquoi niri imbriqué ne pouvait pas juger

`wp_presentation` (stable ; niri, GNOME, KDE, sway le proposent) : pour chaque image confiée, le
compositeur répond **montrée** (l'instant, le numéro du battement, la période) ou **jetée**. Le
prototype de la sonde s'est branché comme le pincement (fiche 93) : la connexion de la fenêtre, une
file et un fil à elle, la surface de winit par `ObjectId::from_ptr`, la demande juste avant de
présenter.

**Sous niri imbriqué** (le banc de la fiche 111), en paires entrelacées : le nouveau rythme ne
jetait plus rien (un tiers avant) mais niri ne montrait qu'**une image toutes les 34 ms**. La
source de niri l'explique (`src/backend/winit.rs`) : imbriqué, il dessine dans une fenêtre de son
hôte, son échange de tampons EGL bloque jusqu'au battement de l'hôte, et le rappel à Glucose ne
part qu'après — un cycle sur deux perdu ; ses retours de présentation sont datés à l'envoi
(`Refresh::Unknown`, battement 0). **Niri imbriqué ne sait pas juger un rythme réglé par ses
rappels** ; seul son vrai écran le peut.

## 3. À son écran : le banc qu'il fallait

* **Un pavé tactile virtuel** (`uinput` : son compte y a droit par une ACL de logind) qui imite le
  sien (ELAN1200 04F3:303E, I2C, « buttonpad », cinq doigts ; plages plausibles, 3209 × 2097 à 31
  par mm, ses vraies ne se lisant pas sans le groupe `input`) : niri le traite par libinput comme
  le vrai — **156 à 222 pincements et 378 glissements à deux doigts** reçus par Glucose à chaque
  essai. Pour la première fois, le geste qu'il fait vraiment se mesure.
* Le pointeur virtuel de niri pour la souris, la molette et le clic du milieu.
* **L'incident — ma faute** : au premier essai, la fenêtre neuve n'a pas pris le focus (niri
  26.04 ne le donne pas sans jeton d'activation), s'est ouverte en colonne hors de la vue, et niri
  ne dit pas où est une colonne (`tile_pos_in_workspace_view` vide) : mon script a pris (0, 0), et
  une vingtaine de secondes de gestes sont tombées sur **VS Code** (un survol, un clic du milieu
  glissé, trois crans de molette, le début d'un pincement). Aucun fichier n'a changé sur le disque
  (vérifié) ; un clic du milieu a pu coller une sélection dans un onglet non enregistré — dit à lui.
  Et mon `pkill -f` s'est tué lui-même (le piège de MÉTHODE § 5). Depuis : la fenêtre de Glucose
  est mise au premier plan (`focus-window`), flottante à une place connue
  (`move-floating-window`), et **chaque geste vérifie d'abord qu'elle a le focus**, sinon tout
  s'arrête ; le scénario ferme Glucose de lui-même (garde-fou).
* Sept fenêtres en tout, chacune annoncée, d'une minute et demie.

## 4. Ce que l'écran a montré

`~/.cache/glucose-bancs/rythme-112/` (`e112-*`, lecteur `presentation.py`). « Sautés » : la part
des battements où l'écran a remontré l'image d'avant, entre deux images montrées de suite ;
« latence » : de l'envoi à l'instant où l'image a paru.

**Toute la carte de loin** (images légères, 4-5 ms de travail) :

| geste | ancien rythme (tempo) | nouveau (niri donne le rythme) |
|---|---|---|
| clic du milieu | 0 % sautés, 1,3 % jetées, 34,7 ms | **16 % sautés**, 0 jetée, 29,0 ms |
| pincer | ~0 %, 4 % jetées, 26,9 ms (p90 36,5) | ~1 %, 0 jetée, 30,0 ms (p90 30,3) |
| deux doigts | 2 %, 2,7 % jetées, 26,6 ms (p90 40,2) | 1 %, 0 jetée, 29,7 ms (p90 30,0) |

**Zoomé dans Sciences, Chimie et Eau** (des dizaines de blocs et de flèches ; les membranes 4 à
6 ms) :

| geste | ancien | nouveau |
|---|---|---|
| pincer | 3,6 % sautés, 7 % jetées, 39 ms | **13 % sautés**, 0, 22 ms |
| deux doigts | 0,7 %, 1,6 %, 33,5 ms (p90 42,6) | 4 %, 0, 26,5 ms (p90 29,5) |
| clic du milieu | 6,2 %, 7,6 %, 42 ms | **16 %**, 0, 29,5 ms |

* **Le « 32 % d'images ratées » de sa session n'était pas l'écran** : c'est le compteur du tempo,
  une image prête après l'heure que visait SA grille, qui n'est pas celle de l'écran. Avec l'ancien
  rythme, l'écran montre une image neuve à 93-99 % des battements.
* **Le prototype échange des saccades contre de la latence** : 5 à 15 ms de moins entre l'envoi
  et l'écran, aucune image jetée, mais jusqu'à 16 % de battements sautés. Relu image par image :
  il envoie bien une image par période, mais une sur six arrive juste après que niri a composé la
  sienne — niri dessine dès qu'un client (ou son curseur, ou VS Code) envoie, et le rappel de niri
  place l'envoi de Glucose n'importe où dans la période. **Écarté.**

## 5. Le calage sur le battement — un prototype, pas encore une loi

La sonde donne l'instant exact de chaque battement (la période annoncée : 16,673016 ms, son écran
bat à 59,977 Hz). Le prototype (`GLUCOSE_RYTHME=calage`) : après avoir dessiné, attendre « le
prochain battement moins une avance », puis envoyer. Sur la scène zoomée, en face du tempo :

| geste | tempo | calage, avance 4 ms | calage, 7 ms |
|---|---|---|---|
| clic du milieu | 25 % sautés, 28,3 ms (p99 52,9) | **0 %**, 37,3 ms (p90 37,4) | 0 %, 40,3 ms |
| pincer | 0 % (pauses comprises 2,9), 34,3 ms (p90 41,2) | 1,8 %, **37,3 (p90 37,3)** | 1,8 %, 40,3 |
| deux doigts | 0,7 %, 37,4 ms (p90 40,1) | 1,1 %, **37,3 (p90 37,3)** | 2,3 %, 40,2 |

* **La gigue disparaît** : la latence est la même à chaque image (médiane = p90), là où le tempo
  passe d'un battement à l'autre — ce qui se voit comme un tressaut.
* **La latence ne baisse pas** : une image envoyée juste avant un battement paraît **deux**
  battements plus tard. Relu sur les envois au hasard du tempo : envoyée moins de 8 ms avant un
  battement, une image paraît presque toujours au troisième ; 10 à 14 ms avant, le plus souvent au
  deuxième. Sa carte graphique est partagée : quand Glucose dessine l'image suivante pendant que
  niri compose, niri prend du retard (le prototype où niri donnait le rythme, qui attend le rappel
  avant de redessiner, paraissait au deuxième battement presque à coup sûr).
* **Pourquoi pas encore dans `main`** : une seule série par avance (une mesure trop courte
  fabrique une conclusion), et l'avance est une constante choisie. **Ce n'est pas la bonne
  grandeur** : le prototype où niri donnait le rythme (Glucose ne dessine qu'après le rappel)
  paraissait au deuxième battement quelle que soit son avance — ce n'est pas notre image qui
  arrive tard, c'est **l'image suivante de Glucose qui occupe la carte graphique pendant que niri
  compose**. Et le rappel `on_submitted_work_done` de wgpu ne part qu'au prochain `poll` : il
  daterait le `poll`, pas la carte. Le levier est donc de **dessiner plus tard** — commencer
  l'image juste à temps pour le battement, ce que le tempo refusait faute de savoir ce qu'une
  image coûte (`app/presentation.rs`) ; PARU-1 donne désormais le juge pour l'apprendre. Une
  refonte de la boucle, à concevoir puis mesurer en paires à son écran.
* Le banc du jour l'écrit en clair : **la latence de l'envoi à l'écran est de deux battements
  (33 à 42 ms) avec l'ancien rythme** — davantage que tout le travail d'une image. Ce que sa main
  attend est là, autant que dans le travail.

## 6. PARU-1 — ce qui a paru à l'écran, dans la chronique (`chronique/ecran.rs`)

* **La sonde** (`plateforme/presentation_wayland.rs`) : sous Wayland, branchée à la naissance de
  la fenêtre, une demande `feedback` juste avant chaque présentation, datée sur **l'horloge que
  le compositeur annonce** (`clock_id`). Aucune caisse de plus (`decisions/14`). Hors de Wayland,
  rien : la section se tait.
* **Ce qui se compte** : les images montrées et jetées ; **les battements sautés alors qu'une
  image était due** — son échéance (GEL-1) précède l'instant où la précédente a paru ; hors de ce
  cas, un battement sans image neuve est le repos, sans seuil à choisir ; l'écart en battements se
  lit au compte de l'écran, sinon au temps rapporté à sa période, **arrondi** ; le temps de l'envoi
  à l'écran au centile, dont l'écart médiane-p90 est la gigue. Les verdicts arrivent dans l'ordre
  des envois : un envoi resté sans verdict quand un plus récent en reçoit un sort de l'attente,
  qui ne grandit jamais ; une image que la surface refuse (cachée, perdue) ne compte pas.
* **La section** « Ce que l'écran a montré » suit « Le rythme » ; la ligne du tempo dit
  désormais ce qu'elle mesure : « image(s) ont manqué l'heure que visait le tempo ».
* **Prouvé** : neuf épreuves (dont le vrai chemin de la présentation, un présentateur témoin et
  une fausse sonde, jusqu'au rapport), la suite entière, clippy ; sabotages
  `outils/sabotages/paru-112.json` (§ 7). Deux épreuves étaient aveugles avant d'être écrites :
  l'arrondi au battement le plus proche (des écarts de 33,4 ms se lisent aussi bien tronqués) et le
  compte de l'écran (il coïncide avec le temps tant que la période est fixe) — leurs données
  exercent désormais le cas. La sonde elle-même ne s'éprouve qu'à son écran (§ 3).
* « ECRAN-1 » existait déjà (le choix de l'écran et de la carte, fiche 60) : l'invariant s'appelle
  PARU-1.
* **Essayé à son écran, le programme de `main`** (`outils/pilote/essai_ecran.sh`, zoomé dans
  « Eau », `~/.cache/glucose-bancs/rythme-112/main-1`) : « ecran : le compositeur dit ce qu'il
  montre », puis dans la chronique : 811 images confiées, 688 montrées, 123 jetées (15,2 % — le
  survol de la souris à 125 Hz en jette le plus) ; **35 battements sautés sur 670 dus (5,2 %)** ;
  de l'envoi à l'écran, médiane 39 ms, p90 46, p99 55. Le tempo, juste au-dessus : « 3 images ont
  manqué l'heure (0,5 %) » — l'écart entre les deux est toute la leçon de cette fiche.

## 7. Les sabotages

`outils/sabotages/paru-112.json` : **18 sur 18** — l'échéance et son âge, l'attente (un envoi qui n'attend pas, une image refusée qui reste, des envois sans verdict qui ne sortent pas, un verdict orphelin compté), les comptes (montrées, jetées, latence), les sauts (une pause comptée, les battements dus un par image, les sauts non comptés, l'image montrée non retenue, le compte de l'écran ignoré, le temps tronqué), la vraie présentation (rien demandé au compositeur, une image cachée ou perdue qui compte) et le rapport qui tairait l'écran. La suite entière : 3 173 épreuves, une seule tombée — elle lisait l'ancien libellé du tempo, accordée au nouveau.

## 8. La recherche (consultée le 12/11/2026)

* winit 0.30.13 : `Window::pre_present_notify`, `FrameCallbackState`, `request_redraw`
  (`platform_impl/linux/wayland/window/{mod,state}.rs`, `event_loop/mod.rs`).
* niri : `src/niri.rs` (`FRAME_CALLBACK_THROTTLE`, `send_frame_callbacks`,
  `send_frame_callbacks_on_fallback_timer`), `src/backend/winit.rs` (le rendu imbriqué) ;
  [la boucle de rendu](https://niri-wm.github.io/niri/Development:-Redraw-Loop.html) ;
  [la PR 642](https://github.com/YaLTeR/niri/pull/642) (les rappels hors écran).
* [Fission UI, PR 156](https://github.com/fission-ui/fission/pull/156) : le même piège —
  `pre_present_notify` qui retient les images quand le rappel ne vient pas.
* Le protocole [presentation-time](https://wayland.app/protocols/presentation-time) ;
  [Firefox, bug 1563075](https://bugzilla.mozilla.org/show_bug.cgi?id=1563075) (prédire le
  prochain battement par les retours ; Chromium sous Wayland mêle rappels et retours).
* Mutter : [« Compute max render time heuristically »](https://git.subgraph.com/brl/mutter/commit/592fbee065f3c90c105559bcd80a05da3c78434e)
  (Ivan Molodetskikh — l'auteur de niri) et [la MR 1762](https://gitlab.gnome.org/GNOME/mutter/-/merge_requests/1762) :
  composer le plus tard possible avant le battement, en apprenant le temps qu'il faut — côté
  compositeur ; niri ne retarde pas ses rappels, c'est à Glucose de se caler.
* Mesa : [la présentation fifo sous Wayland (24.3)](https://phoronix.com/news/Mesa-24.3-Vulkan-FIFO),
  par `commit-timing` et `fifo`, que niri 26.04 n'offre pas.

## 9. Les leçons

* **Un compteur interne n'est pas l'écran** : « ratée » disait « prête après l'heure que je
  visais », et deux sessions l'ont lu comme « l'écran a sauté ». Demander au juge (le compositeur)
  avant de concevoir un remède.
* **Un banc imbriqué ne juge pas un rythme qu'il produit lui-même** : niri imbriqué bloque sur son
  hôte ; ses retours de présentation sont datés à l'envoi.
* **Avant d'envoyer un geste à son écran, savoir où il tombera** : une fenêtre neuve sous niri ne
  prend pas le focus et peut s'ouvrir hors de la vue ; vérifier le focus avant chaque geste.
* **Un nom d'invariant se cherche avant de se poser** (ECRAN-1 était pris).

## 10. Le co-working — la recherche commencée, pas encore de page

Commencée en fin de session (chantier 4 de la passation ; la porte publique attend sa parole) :

* **Le journal de Glucose s'y prête déjà** : chaque geste garde l'élément **entier**, avant et
  après, à une position dans une liste (`Slot`, `core/store/journal.rs`), et chaque élément porte
  son identifiant. Un geste se traduit donc directement en « l'élément X devient R » : partagé, il
  désigne l'élément par son identifiant, et la position (l'ordre de dessin) devient un **indice
  fractionnaire**, comme Excalidraw l'a fait en 2024.
* **Deux familles** : un arbitre (Figma, tldraw : la salle ordonne, le dernier changement d'une
  propriété gagne, le client rejoue ses gestes en attente par-dessus — la « rebase » de
  Replicache/Zero, des gestes **nommés** rejoués sur l'état du serveur) ; ou sans arbitre (Excalidraw :
  une version par élément, la plus haute gagne, l'égalité tranchée par un nombre tiré au hasard —
  et la salle ne voit que du chiffré). Glucose travaille hors ligne : la version par élément
  fusionne au retour sans serveur d'état ; un texte écrit à deux en même temps, lui, perd l'un des
  deux (comme chez Figma) — un CRDT de texte plus tard.
* **Le chiffrement de bout en bout** (la clé dans le lien d'invitation, comme Excalidraw) tient sa
  règle « rien ne sort sans son accord » : le relais transmet sans lire. Keyhive et Beelay (Ink &
  Switch, pré-alpha) : les droits délégués et le chiffrement de groupe en local-first.
* À concevoir sur sa page, avant une ligne : les photos (adressées par leur empreinte SHA-256, que
  le noyau sait calculer), l'invitation, la présence (les curseurs), la Time Machine partagée
  (chaque geste signé de son auteur, MAIN-1).

Sources (consultées le 12/11/2026) : [tldraw sync, une analyse](https://grida.co/docs/wg/research/crdt/tldraw) ;
[You might not need a CRDT](https://jamsocket.xyz/blog/you-might-not-need-a-crdt) ;
[Excalidraw P2P](https://plus.excalidraw.com/blog/building-excalidraw-p2p-collaboration-feature) ;
[Replicache, comment ça marche](https://doc.replicache.dev/concepts/how-it-works) ;
[Rocicorp, Ready Player Two](https://rocicorp.dev/blog/ready-player-two) ;
[Keyhive](https://www.inkandswitch.com/keyhive/notebook/) ; et, déjà dans la fiche 111 § 5, Figma
(Evan Wallace, 2019) et tldraw-sync-cloudflare.

