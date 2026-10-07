# La suite, dans l'ordre

> **Au 07/10/2026.** La feuille de route de Glucose Rust après la V2. Elle remplace les plans des
> fiches 36, 44 et 46 du [carnet](carnet/00-INDEX.md), qui en gardent le raisonnement. Chaque
> chantier dit **pourquoi il est là**, et ce qui le déclare fini. Tout y est à remettre en question.
>
> **Comment s'en servir** : prendre le premier chantier non fait. Quand il est fini (prouvé, pas
> « ça compile »), le cocher ici, mettre à jour [ÉTAT](ETAT.md), et passer au suivant.

---

## 1. Ses retours sur la V2, et la 2.0.2 bêta — **d'abord**

Il a approuvé l'ordre le 06/10 et ajouté trois retours le même jour. **Tout est écrit et éprouvé
hors écran depuis la session des 06-07/10 (fiche 51)** ; ce qui reste, c'est **son écran** — la
liste de la fiche 51 § 9 —, puis la publication.

`[x]` fini et prouvé · `[~]` écrit, éprouvé hors écran, **attend son jugement à l'écran** · `[ ]` à faire

| | chantier | où en est-il |
|---|---|---|
| [x] | **La version de travail en `2.0.2-dev`** | l'épreuve compare à tout ce qui est publié (sabotée : elle tombe) |
| [~] | **La souris instantanée** : deux portes dans l'élan, la souris montrée à l'image suivante | **il n'a pas de souris** (07/10) : ce sont d'autres mains qui le diront. La roue libre reste un doute pour elles ; la réponse est *Direct Manipulation* (chantier 5) |
| [x] | **Copier, couper, coller des nœuds, d'une fenêtre à l'autre** : le lot `.glucose`, le format « Glucose.Lot », le collage au curseur en un geste ; le glisser entre fenêtres (`DoDragDrop`) | **« absolument parfait »**, et le glisser entre deux fenêtres aussi (07/10) |
| [x] | **Clic droit sur une image : « Copier l'image » et « Enregistrer l'image sous… »** | **« parfait »**, Discord compris (07/10) |
| [~] | **`Ctrl+N` : un nouveau document**, et l'entrée du menu | quitter un document nommé le dit désormais (« « fuser » est enregistré — nouveau document ») : son écran |
| [~] | **Le mode référence, à la PureRef** : `Ctrl+Maj+A`, sans interface, sans cadre, au premier plan, retenu ; **au pavé, `Alt` + glisser déplace la fenêtre et `Alt` + pincer la redimensionne** (REFERENCE-2) ; il se désagrandit en entrant | son écran : les couleurs (RTX HDR et la vibrance de NVIDIA s'y appliquaient), et **les signets `1`, `Ctrl+1`, qui ne répondaient pas** — non reproduit en épreuve (fiche 52 § 8) |
| [x] | **Le repos à zéro image** : la chronique du 07/10 le dit raison par raison | **aucune raison spontanée** : les messages qui s'effacent et les glissades qui finissent |
| [x] | **La CI de `20deb6f`** (la voie hors Windows du presse-papiers) | neuf tâches vertes |
| [x] | **ECRAN-1 — la carte qui tient l'écran** : le lag du 07/10 (gels de 500 ms dans `present`) venait d'un balancier de l'arbitre entre la RTX, qui tient son écran, et l'Intel (fiche 52 § 1) | **vérifié** sur sa session suivante : la RTX, « celle qui tient l'écran », `mailbox`, pire image 48 ms au lieu de 505 |
| [~] | **POIGNEE-1 — les poignées, le cadre et l'anneau de sélection suivent la place du nœud** à l'écran (son retour du 07/10, deux fois) | son œil, au dézoom : la grille blanche du cadre restait après les poignées |
| [~] | **Le pincement au pavé** : direct, sans conduite ni glissade, 1/5 d'octave par unité (« trop smooth, pas rapide » au 1/12 lissé) | son doigt |
| [~] | **Le glisser vers une autre fenêtre** : la fenêtre d'origine se peint avant de partir, et ne fait plus tourner un cœur à vide | son écran (fiche 51 § 9, 3) |
| [ ] | **Publier la 2.0.2-beta.1** : notes écrites (`outils/publication/notes.md`) ; son geste (Actions → Publier, `2.0.2-beta.1`, brouillon coché, relire, publier) | **après** ses essais à l'écran |

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
