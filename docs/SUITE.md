# La suite, dans l'ordre

> **Au 06/10/2026.** La feuille de route de Glucose Rust après la V2. Elle remplace les plans des
> fiches 36, 44 et 46 du [carnet](carnet/00-INDEX.md), qui en gardent le raisonnement. Chaque
> chantier dit **pourquoi il est là**, et ce qui le déclare fini. Tout y est à remettre en question.
>
> **Comment s'en servir** : prendre le premier chantier non fait. Quand il est fini (prouvé, pas
> « ça compile »), le cocher ici, mettre à jour [ÉTAT](ETAT.md), et passer au suivant.

---

## 1. Ses retours sur la V2, et la 2.0.2 bêta — **d'abord**

Il a approuvé l'ordre le 06/10 (la souris, `Ctrl+N`, le mode référence) puis ajouté trois retours
le même jour (copier-coller, clic droit sur une image). Le copier-coller passe juste après la
souris parce que `Ctrl+X` peut aujourd'hui détruire ce qu'il ne sait pas recoller : ses données
d'abord. Ce sont les premiers retours d'usage de la V2 : rien ne passe avant.

| | chantier | fini quand |
|---|---|---|
| [ ] | **La version de travail en `2.0.2-dev`** (fiche 48 § 15.4, oubliée après la publication) | l'épreuve de version passe |
| [ ] | **La souris instantanée.** Deux voies d'entrée, pas un réglage commun : à la souris, chaque cran de molette zoome dans l'image même et le glisser (bouton du milieu, `Espace`) suit le curseur au pixel, **rien ne glisse au lâcher** ; au doigt (pavé, écran tactile, pincement), l'élan actuel, inchangé. Aujourd'hui tout passe par `interactions/elan.rs`, réglé **au pavé** : conduite 10 ms, glissade de 0,45 s (déplacement) et 0,28 s (zoom). La source se reconnaît déjà (`pan_zoom::source_continue`). **Doute à vérifier** : une molette à défilement libre (Logitech MX…) envoie des fractions de cran, comme un pavé | un cran de souris donne tout son zoom dans l'image suivante et rien après ; un glisser lâché ne laisse aucun reste ; le pavé ne change pas d'un pixel ; **lui** le juge à l'écran |
| [ ] | **Copier, couper, coller des nœuds, d'une fenêtre à l'autre.** D'abord le piège : `Ctrl+X` détruit aujourd'hui ce que `Ctrl+C` ne sait pas emporter (il ne copie que du texte). Puis le vrai geste : la sélection entière part dans le presse-papiers sous **un format à Glucose** (les nœuds, leurs places relatives, les flèches qui les relient, les octets des images par leur empreinte), **avec** un texte de repli pour les autres logiciels ; le collage pose le lot au curseur, en un geste annulable, **chaque identifiant renommé** — `Store::importer_un_document` (BOARDS-2) sait déjà renommer un lot et faire suivre chaque référence. Le presse-papiers traverse les processus : deux Glucose ouverts s'échangent ainsi leurs nœuds. Ensuite **le glisser d'une fenêtre à l'autre** : la cible de dépôt existe (`plateforme/depot_windows.rs`), il manque la source (`DoDragDrop`) | une sélection mêlée (textes, images, flèches, membrane) copiée dans une fenêtre et collée dans une autre revient identique, au bit près, sous des identifiants neufs ; couper puis coller ne perd rien |
| [ ] | **Clic droit sur une image : « Copier l'image » et « Enregistrer l'image sous… ».** Copier met l'image dans le presse-papiers du système comme une image (Discord, un navigateur, Photoshop la collent) ; enregistrer écrit **ses octets d'origine**, scellés dans le document, sans recompression, avec leur extension | coller dans Discord montre l'image ; le fichier enregistré est identique, octet pour octet, à celui qui avait été posé |
| [ ] | **`Ctrl+N` : un nouveau document**, par le raccourci et une entrée visible. Rien ne se perd (le document s'écrit déjà au fil de l'eau) ; un travail sans nom pose la question habituelle (BROUILLON-1) | épreuve de bout en bout ; aucun brouillon perdu |
| [ ] | **Le mode référence, à la PureRef.** Un geste qui enlève toute l'interface (bande, onglets, minimap, panneaux), enlève le cadre de la fenêtre (déplacer et redimensionner restent possibles), garde la fenêtre **toujours au premier plan**, et s'en souvient d'une session à l'autre ; le même geste le défait. Regarder d'abord **les gestes exacts de PureRef** pour déplacer une fenêtre sans cadre. Plus tard, s'il le veut : opacité, clics qui traversent | il garde sa référence au-dessus de Blender et la juge à l'écran |
| [ ] | **Le repos à zéro image.** Glucose dessine encore 1 à 2,5 images par seconde quand personne n'y touche (fiche 43 § 8). Une fenêtre de référence posée sur Blender doit coûter **zéro** : c'est « se nicher là où il reste des ressources » | la chronique dit 0 image au repos |
| [ ] | **Publier la 2.0.2-beta.1** : son geste (Actions → Publier, brouillon coché, relire, publier) | ses testeurs la reçoivent |

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
* Les gels de la carte Intel Arc (fiche 43 § 8).
* Un plantage reproductible **dans les épreuves** : trois fois dans `vulkan-1.dll` au même octet
  (fiche 49 § 4), probablement une épreuve graphique qui referme la carte.
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
4. **Sa souris** (le doute du chantier 1).
5. Sait-il qui, parmi ses cinq utilisateurs, a basculé ? Surtout sous Linux.
6. Les membranes de Mary, les rideaux, Trans-domaines, « optimiser » dans la Time Machine.

## Les essais à l'écran jamais faits

Écrits dans les fiches 42, 43 et 47, jamais rapportés. À lui redonner en **une liste courte** à la
fin du chantier 1 (avec le copier-coller entre deux fenêtres et le collage dans Discord) : les flèches qui contournent (et leur coude qu'on glisse), les formules seules sur
leur ligne, la Time Machine ouverte pendant qu'on zoome, `Ctrl+Z` après un glisser ou un
redimensionnement, l'aimant sur un tableau chargé, coller puis `Ctrl+C`/`Ctrl+V` une image, glisser
un PDF depuis une page web.
