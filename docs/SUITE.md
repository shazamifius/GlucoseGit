# La suite, dans l'ordre

> **Au 07/10/2026.** La feuille de route de Glucose Rust après la V2. Elle remplace les plans des
> fiches 36, 44 et 46 du [carnet](carnet/00-INDEX.md), qui en gardent le raisonnement. Chaque
> chantier dit **pourquoi il est là**, et ce qui le déclare fini. Tout y est à remettre en question.
>
> **Comment s'en servir** : prendre le premier chantier non fait. Quand il est fini (prouvé, pas
> « ça compile »), le cocher ici, mettre à jour [ÉTAT](ETAT.md), et passer au suivant.

---

## 1. Ses retours sur la V2, et la 2.0.2 bêta — **d'abord**

Fiches 51 et 52. Au soir du 07/10, **presque tout est jugé à son écran** : « tout est absolument
parfait ». Restent le pincement, le dessin de la sélection, et la publication.

> **La CI n'a pas vu les commits du 07/10 après `20deb6f`.** Il a demandé l'envoi **sans**
> déclencher la vérification (le dernier commit porte `[skip ci]`). Or ils touchent du code
> propre aux autres systèmes (`plateforme/ecran.rs`, `plateforme::glisser_un_lot`) : **avant la
> publication**, lui proposer un envoi qui lance la CI, et en lire le résultat.

`[x]` fini et prouvé · `[~]` écrit, éprouvé hors écran, **attend son jugement à l'écran** · `[ ]` à faire

| | chantier | où en est-il |
|---|---|---|
| [x] | **La version de travail en `2.0.2-dev`** | l'épreuve compare à tout ce qui est publié (sabotée : elle tombe) |
| [x] | **La souris instantanée** : deux portes dans l'élan, la souris montrée à l'image suivante | essayée à la souris le 07/10 : « tout est absolument parfait » ; il navigue surtout au pavé |
| [x] | **Copier, couper, coller des nœuds, d'une fenêtre à l'autre** : le lot `.glucose`, le format « Glucose.Lot », le collage au curseur en un geste ; le glisser entre fenêtres (`DoDragDrop`) | **« absolument parfait »**, et le glisser entre deux fenêtres aussi (07/10) |
| [x] | **Clic droit sur une image : « Copier l'image » et « Enregistrer l'image sous… »** | **« parfait »**, Discord compris (07/10) |
| [x] | **`Ctrl+N` : un nouveau document**, et l'entrée du menu | **demande toujours** (NOUVEAU-1) : « fonctionne parfaitement » (07/10) |
| [x] | **Le mode référence, à la PureRef** : `Ctrl+Maj+A`, sans interface, sans cadre, au premier plan, retenu ; **au pavé, `Alt` + glisser déplace la fenêtre et `Alt` + pincer la redimensionne** (REFERENCE-2) ; il se désagrandit en entrant | « absolument et vraiment sublime » (07/10), signets compris. Plus tard, s'il le veut : opacité, clics qui traversent |
| [x] | **Le repos à zéro image** : la chronique du 07/10 le dit raison par raison | **aucune raison spontanée** : les messages qui s'effacent et les glissades qui finissent |
| [x] | **La CI de `20deb6f`** (la voie hors Windows du presse-papiers) | neuf tâches vertes |
| [x] | **ECRAN-1 — la carte qui tient l'écran** : le lag du 07/10 (gels de 500 ms dans `present`) venait d'un balancier de l'arbitre entre la RTX, qui tient son écran, et l'Intel (fiche 52 § 1) | **vérifié** sur sa session suivante : la RTX, « celle qui tient l'écran », `mailbox`, pire image 48 ms au lieu de 505 |
| [x] | **POIGNEE-1 — les poignées, le cadre et l'anneau de sélection suivent la place du nœud** à l'écran (son retour du 07/10, deux fois) | « parfait » (07/10) |
| [~] | **Le pincement au pavé** : la conduite sans la glissade, 0,22 d'octave par unité — « nette amélioration, mais pas encore ça » ; la réponse de fond est *Direct Manipulation* (chantier 5) | « légèrement trop lent, et pas fluide, comme s'il sautait » ; PureRef et un navigateur sont « instantanés et fluides » → **chantier 1 bis, *Direct Manipulation*** |
| [x] | **La molette** : un tiers d'octave par cran (« pas du tout assez rapide » au huitième) | « parfait » (07/10) |
| [x] | **Le mode référence, suite** : les panneaux retirés aussi de la voie graphique ; SIGNET-2 (les signets retiennent le centre) ; REFERENCE-3 (la fenêtre se redimensionne une fois par image — le gel qui a « failli planter » son PC, cause probable) | « parfait » (07/10) ; le gel n'est pas revenu |
| [ ] | **Le dessin de la sélection** : « pas esthétique du tout » ; quatre directions monochromes lui sont montrées (fiche 52 § 9) | **son choix** |
| [x] | **Le glisser vers une autre fenêtre** : la fenêtre d'origine se peint avant de partir, et ne fait plus tourner un cœur à vide | « parfait » (07/10) |
| [ ] | **Publier la 2.0.2-beta.1** : notes écrites (`outils/publication/notes.md`) ; son geste (Actions → Publier, `2.0.2-beta.1`, brouillon coché, relire, publier) | **après** ses essais à l'écran |

## 1 bis. Le pincement par *Direct Manipulation* — **ensuite**

**Pourquoi** : son jugement du 07/10, après trois réglages — « légèrement trop lent, et pas
fluide, comme s'il sautait », alors que PureRef et un navigateur pincent « instantanément, de
manière fluide ». Glucose reçoit le pincement d'un pavé de précision sous la forme que Windows
donne aux applications qui ne savent pas mieux : un `Ctrl` + molette fractionné, par paquets
(fiche 33, `interactions/pincement.rs`). Aucun réglage de gain ni de lissage ne rend la
continuité qu'il a perdue en route. Chromium et Blender prennent le pavé par *Direct
Manipulation* (`IDirectManipulationManager`, viewport, `DM_POINTERHITTEST`) : l'échelle, le
déplacement et l'inertie arrivent tels que le doigt les fait, à la cadence du pavé (fiche 51
§ 1, ses sources).

**Fini quand** : à son écran, le pincement est « instantané et fluide » comme dans PureRef ; et
tout ce qui arrive encore en molette vient d'une molette (ce qui règle aussi le doute des roues
libres). Une recherche d'abord : le code de Chromium (`direct_manipulation_helper_win.cc`) et le
commit de Blender cité en fiche 51 § 1 ; puis une dépendance éventuelle à défendre dans
`carnet/decisions/` (la caisse `windows` a la fonctionnalité `Win32_Graphics_DirectManipulation`).

## 1 ter. Le dessin de la sélection

**Pourquoi** : « pas esthétique du tout » (07/10). `style.md` impose une chrome monochrome,
héritée de Glucose Tauri. Quatre directions lui ont été dessinées hors écran : `docs/carnet/
screens/52-selection-quatre-directions.png` — A aujourd'hui, B des équerres aux coins, C un fil
et quatre points, D un cadre unique pour le groupe. **Il ne sait pas où regarder les fichiers** :
la lui **montrer** (l'envoyer à l'écran), recueillir son choix, puis l'écrire dans la loi des
ornements (`hit_priority::chrome_ratio`, `renderer/handles.rs`, `scene/image/ornement.rs`) et
dans `style.md`. Si c'est la couleur qui lui manque, c'est `style.md` qui change — sa décision.

## 2. La boîte noire qui voyage

**Pourquoi maintenant** : deux installeurs téléchargés, zéro paquet Linux, et aucun moyen de savoir
si la bascule a échoué en silence chez quelqu'un. Sans elle, on ne voit que sa machine.

Conçu, pas écrit (fiche 49 § 2) : éteint par défaut, une question claire au premier lancement,
« voir ce qui part » (exactement les lignes de la boîte noire, qui ne peuvent porter aucun mot de
l'utilisateur), des lots envoyés au lancement suivant en HTTPS, un identifiant tiré au hasard et
renouvelable, effacement sur demande, **aucune adresse IP gardée**, une page publique qui dit tout.

* **Bloqué par sa décision** : le serveur. Sa box NixOS derrière un nom gratuit (deSEC) et Caddy,
  **mon avis**, parce que personne d'autre ne voit les données ; ou Cloudflare Workers, sans entretien.
* Les plantages hors de Windows : Linux en a deux registres (`systemd-coredump` chez Fedora, `apport`
  chez Ubuntu) ; Android a `ApplicationExitInfo` (11 et plus).

## 3. Le toucher : Android, et l'iPad par le web

Le plan est dans la fiche 50 ; l'ordre :

1. **L'exécuteur des tranches** : dans un navigateur, il n'y a pas de fils. Le travail de fond
   (scribe, atelier, boîte noire) doit pouvoir tourner **en tranches, dans le temps libre de chaque
   image**, comme une voie de plein droit, éprouvée d'abord sous Windows (mêmes résultats par les
   deux voies).
2. **La couche de gestes** commune : doigts, pincement, aucun survol, puis le crayon.
3. **Le web** (WebGPU dans Safari 26 sur iPad à puce A12 et après, WebGL2 en dessous) : chaque
   document écrit **aussi** dans les Fichiers de l'iPad, parce que Safari efface les données d'un
   site inactif et recharge une page trop gourmande. Une version web qui perd un document serait
   pire que rien.
4. **Android** : `GameActivity` (le clavier virtuel ne marche pas avec `NativeActivity`), la vie
   d'une application suspendue, l'APK et **sa clé à ne jamais perdre**, de vrais téléphones anciens
   par Firebase Test Lab (gratuit).

## 4. Ce qui manque de Glucose Tauri

D'abord **remesurer la parité** (le tableau de la fiche 03 date du 10/09). Puis, dans l'ordre de la
fiche 36 § 2 (phase 4) : ce qui est écrit dans le noyau et pas branché (`timeline`, `mirror_graph`,
rideaux, `membrane_stretch` : le cliquet 3 les nomme), naviguer dans l'immense (`Ctrl+F`, la couleur
des domaines), dossiers et miroirs, rideaux et temporalité, storyboard, presets et zones,
l'interface (sélecteur de couleur, infobulles), vidéos et provenance (fiche 27), exports PNG et HTML.

**Le registre de Tauri** (fiche 39) garde ses défauts relevés, à ne pas reproduire. Encore ouverts :
les membranes de Mary (13), les rideaux (14), « recopie image » (15) et la carte « aaaa » (18) à
comprendre avec lui, les niveaux de boards (16) à confirmer, l'optimisation de la Time Machine (12).

## 5. La fluidité

* Les zooms chargés à 11-21 ms (session du 05/10) : le plancher de 100 images par seconde.
* Les gels de la carte Intel Arc (fiche 43 § 8) : très probablement l'écran branché sur la RTX
  (fiche 52 § 1) — à confirmer, puis à mesurer en mode Optimus (l'écran sur l'Intel).
* **La boîte noire ne note ni la carte ni la succession des images** : à ajouter, c'est elle
  qui voyage (fiche 52 § 1).
* Un plantage reproductible **dans les épreuves** : trois fois dans `vulkan-1.dll` au même octet
  (fiche 49 § 4), probablement une épreuve graphique qui referme la carte.
* **Le pavé par *Direct Manipulation*** (fiche 51 § 1) : ce que font Chromium et Blender — les
  gestes du pavé avec leurs phases, leur point focal et leur inertie ; et tout ce qui arrive
  encore en molette vient alors d'une molette, ce qui règle le doute des roues libres.
* Puis l'étalonnage : le modèle de coût qui **prédit** ce qu'une image va coûter, appris de ce que la
  boîte noire aura vu sur les vraies machines.

## 6. Plus loin

Le co-working (sur le journal du document) · le MCP réécrit en Rust (`rmcp`) · les plugins selon sa
vision ([PLUGINS.md](../PLUGINS.md)) et l'IA locale · le Mac · la fondation des 10⁷ nœuds (l'arène,
écrite et en attente) · et le but : le canevas de Wikipédia de `glucose-brain`.

---

## Ce qui n'attend que lui

1. **Le serveur de la boîte noire** (chantier 2).
2. **Une copie de sa clé de signature hors de ce PC.**
3. **Le modèle de l'iPad**, et **les téléphones de ses testeurs Android** (le plus ancien d'abord).
4. ~~Le modèle de sa souris~~ : **il n'en a pas** (07/10) ; il navigue au pavé tactile.
5. Sait-il qui, parmi ses cinq utilisateurs, a basculé ? Surtout sous Linux.
6. Les membranes de Mary, les rideaux, Trans-domaines, « optimiser » dans la Time Machine.

## Les essais à l'écran jamais faits

Écrits dans les fiches 42, 43 et 47, jamais rapportés. À lui redonner en **une liste courte** à la
fin du chantier 1 (avec le copier-coller entre deux fenêtres et le collage dans Discord) : les flèches qui contournent (et leur coude qu'on glisse), les formules seules sur
leur ligne, la Time Machine ouverte pendant qu'on zoome, `Ctrl+Z` après un glisser ou un
redimensionnement, l'aimant sur un tableau chargé, coller puis `Ctrl+C`/`Ctrl+V` une image, glisser
un PDF depuis une page web.
