# 72 — Le téléphone qui se taisait, le texte de près, le zoom déduit, et l'interface comme donnée

> Session du 15/10/2026, après la fiche 71. Le programme était **P4, la fondation des presets**,
> toute la session. Son message d'ouverture a changé l'ordre : **sur deux téléphones, un tableau
> de plus de 6 000 images s'est fermé tout seul, et tout a été perdu** ; et **de près, le texte
> devient flou en gros pixels, illisible** (deux captures). Ses données et la lisibilité passent
> avant les presets : P4 est venu après, et sa fondation est posée.

## 1. Ce qui est vérifié en ouvrant

* **La 2.0.3-beta.3 est publiée** (« Latest », le 15/10 à 11 h 56 UTC).
* **La télémétrie** : 73 sessions, 5 installations. Les trois sessions Android reçues le 15/10
  datent toutes du **13/10**, version `2.0.2-dev`, sans une ligne `memoire` : **rien de la beta 3
  n'est arrivé du téléphone**, et **aucun second téléphone** n'apparaît.
* Cloudflare ne garde aucun journal des requêtes (voulu : aucune adresse IP) : un refus du
  serveur ne laisse pas de trace.

## 2. Ses retours, mot pour mot ou presque

* *« Après plus de 6 000 images, mon téléphone n'a pas du tout crashé, il a juste fermé de
  lui-même le board, et j'ai perdu toutes mes images et mon board ; sinon tout était
  parfaitement fluide. »* — *« J'ai testé sur un autre téléphone, et pareil. »*
* Au PC, `cargo run --release` : *« tout a fonctionné parfaitement »*.
* *« Le niveau de zoom s'arrête trop proche ; ce serait bien de pouvoir beaucoup plus zoomer. »*
* La R&D (pas pour maintenant) : un améliorateur d'image à l'import, filtres de Gabor ; son
  ancien projet `main.cpp` à garder sur GitHub.
* Sa proposition : **améliorer très, très grandement le système pour ordonner les images**.
* *« Truc extrêmement important !! »* : de près, le texte et les membranes font un flou
  *« absolument illisible et impossible à travailler »* ; *« au bout d'un moment ça s'enlève […]
  comment ça se fait qu'on a besoin d'autant de pixelisation sur un simple texte ? »*

## 3. Pourquoi le téléphone se taisait — POIDS-1

Le serveur refuse une session de plus de **512 Kio** ; une session de trois minutes au téléphone
en pèse 72. Une longue session — exactement celle des 6 000 images — était refusée, retenue
« refusée » dans `telemetrie-envoyees.txt`, et **ne repartait jamais**. Les sessions les plus
précieuses étaient précisément celles qu'on ne voyait pas.

* Une session trop lourde perd désormais **ses plus anciennes mesures** (épisodes, postes,
  records, signes du pavé) jusqu'à tenir ; ce qui précède une mort reste ; une ligne `coupee`,
  juste après le début, dit combien sont parties. Ce qu'il faut retirer compte la ligne `coupee`
  à son plus long (`ligne_coupee(u64::MAX, u64::MAX)`) : aucune marge choisie — et une épreuve
  exhaustive sur chaque poids le prouve, car « retirer juste assez » l'oubliait une fois sur deux.
* Les sessions qu'un Glucose d'avant s'est vu refuser **pour leur poids** repartent une fois :
  la liste ne disait pas pourquoi elle les retenait, leur poids le dit.
* **Un refus se retient avec sa version** (`cle refusee 2.0.3-beta.4`) : la version suivante le
  retente une fois — le serveur aura peut-être appris la ligne qui le faisait refuser. La même
  leçon, appliquée à demain.

## 4. Pourquoi il s'est fermé — on ne le sait pas encore ; désormais, on le saura

Lu dans le code, sans pouvoir le prouver sans le téléphone :

* **Rien n'efface un brouillon à la mort du processus.** Le brouillon vit dans le dossier privé
  (`internal_data_path`, jamais vidé par Android) ; la reprise le rouvre au lancement ; le verrou
  est un `flock`, relâché à la mort ; « Enregistrer sous » copie d'un bloc et ne retire le
  brouillon qu'après. **Son tableau est peut-être encore dans le téléphone** : ne pas désinstaller.
* **Trouvé : `winit` ignore la destruction de l'activité** (« TODO: forward onDestroy
  notification », `platform_impl/android`) **et refuse de recréer sa boucle**
  (`RecreationAttempt`) : si Android recrée l'activité dans un processus vivant, `android_main`
  échoue et `GameActivity_finish` la ferme — à chaque toucher, tant que le processus vit.
  **VIE-2** : le processus se quitte alors, et le toucher suivant repart neuf. *Écrit, compilé
  pour Android ; impossible à éprouver sur ce PC.* Rien ne dit que ce soit son cas.

Pour savoir :

* **SORTIE-1** : au lancement, la raison qu'Android garde de chaque mort d'un processus de
  Glucose (`ApplicationExitInfo`, Android 11 et après : mémoire basse, plantage Java ou natif,
  gel, arrêt par l'utilisateur… dix-sept raisons, une liste fermée), avec la mémoire occupée à la
  mort (PSS, RSS), reconnue par le numéro de processus et l'instant — la même règle que les
  plantages de Windows. Une fin neuve, **`faute_de_memoire`** : tuée sous les yeux. Tuée en cache,
  c'est ordinaire. Une ligne `sortie`.
* **REPRISE-1** : une ligne `reprise` — ce que le lancement a trouvé et rouvert (rien, un
  brouillon, le dernier document, un échec), combien d'images, quel poids.
* Le serveur les connaît, **déployé** (version `2a274567`) et **prouvé en ligne** : une session
  jetable portant `coupee`, `sortie` et `reprise` reçue (201), effacée (200), la base relue (0).
  Un premier essai, trente secondes après le déploiement, avait été refusé : la propagation de
  Cloudflare prend jusqu'à une minute. Réessayer avant de chercher une cause.

**25 sabotages, 25 chutes** (`outils/sabotages/telephone-72.json`).

### 4 bis. Sa réponse, le soir — et LIBERE-1

*« Je ne l'ai pas rouvert, ça a juste dépop »* ; *« un canevas vide avec le message de bienvenue
de Glucose de base »*. La carte de bienvenue n'existe qu'au **démarrage** : le processus est
mort, et Android a relancé l'application qu'on regardait. Au relancement, la reprise n'a rien
rouvert — sinon il aurait lu « Travail non enregistré retrouvé », ou un échec.

Deux hypothèses, aucune prouvée :

* **le verrou du mourant** : la reprise saute tout brouillon qu'un autre Glucose tient encore.
  Au bureau, c'est une autre fenêtre ; au téléphone, le seul autre Glucose possible est celui
  qui meurt — et un processus qui tient des gigaoctets d'images met du temps à finir de mourir ;
* **le disque plein** : les photos sont copiées dans le document, et 6 000 photos de téléphone
  pèsent des gigaoctets.

La ligne `reprise` les départage désormais : `tenus` (les brouillons sautés parce que tenus),
`brouillons_mo` (ce que pèsent tous les brouillons), `libre_mo` et `total_mo` (la place du
disque, lue par `StatFs` sous Android ; absente ailleurs). Le serveur, redéployé (version
`50aee6c6`) et prouvé en ligne.

**LIBERE-1** (`persist/reprise/liberes.rs`), la défense : chaque brouillon tenu au lancement a
son fil, qui **attend son verrou** (`File::lock`) — c'est la mort ou la fermeture de l'autre qui
le réveille, aucune horloge. Libéré, il se rouvre si le document d'accueil est intact (« Ton
travail est revenu ») ; sinon il attend le prochain lancement, et le dit. Sous Windows, le
verrou est un mode de partage qu'on ne peut pas attendre : un brouillon tenu y appartient à une
fenêtre vivante. Éprouvé **sous Linux** (la distribution `Glucose-Linux`), le scénario même du
téléphone ; 4 sabotages, 4 chutes (`outils/sabotages/libere-1.json`, lancés par `wsl`).

## 5. Le texte de près — GRAND-1

**Le mécanisme** (lu dans le code, puis mesuré) : sur la voie graphique, une carte de texte est
une texture rendue par le processeur (COMPOSANT-1) ; de près, elle se découpe en carrés de
256 px (DE-PRES-1) que la cascade rend dans le temps libre de chaque image (CASCADE-2) ; en
attendant, le **repli** est la carte entière au plus grand palier qui tient dans l'écran,
agrandie dix ou vingt fois — ses gros pixels. Et chaque lettre se rastérisait **entière**, à sa
taille à l'écran : un carré qui touchait une lettre de mille pixels la payait toute.

`bench_de_pres` (neuf, son écran, une carte qui le couvre, à froid ; le pire carré entre
parenthèses) :

| zoom | avant | après |
|---|---|---|
| ×16 | 33,0 ms (7,2) | 21,3 ms (3,0) |
| ×32 | 65,6 ms (26,4) | 19,7 ms (2,5) |
| ×50 | 53,0 ms (21,2), **lettres réduites** | 18,7 ms (3,8), à leur taille |

* **Au-delà d'un demi-carré (128 px d'em), la lettre se trace** par son contour (`ttf-parser`,
  déjà là pour le SVG) au coût des pixels qu'elle couvre — ce que fait Skia, le moteur de Chrome
  (donc de Tauri), au-delà de 256 à 384 px. Le seuil est **mesuré** (32, 64, 128, 256 essayés ;
  le journal est dans `typography/trace.rs`).
* **`MAX_FONT_SIZE` disparaît** : il réduisait les lettres au-delà de ×32. Sa garde (une taille
  aberrante qui gèlerait) n'a plus rien à garder : un tracé coûte au plus le tampon.
* **Les formules** rastérisaient chaque glyphe **sans cache** : même traitement. Un glyphe de
  formule porte désormais un `char` — le pont de KaTeX pousse chaque caractère seul, et le type le
  dit (la documentation annonçait « parfois une ligature », c'était faux) ; l'avance qui ne
  servait jamais a disparu, une garde aveugle de moins.
* **10 sabotages, 10 chutes** (`outils/sabotages/grand-1.json`).

**Ce qui reste, et c'est le vrai remède** : tant que le texte est rastérisé par le processeur, un
zoom refait des carrés, et le repli se voit. **Slug** — le texte rendu par la carte graphique
directement depuis les courbes des lettres, net à tout zoom sans jamais re-rastériser — est
**dans le domaine public depuis le 17/03/2026** (Eric Lengyel a abandonné son brevet ; nuanceurs
de référence sous licence MIT). C'est le chantier suivant de la voie graphique (SUITE).

## 6. Le zoom, déduit

×20 à la molette (hérité de Tauri), ×50 dans le modèle. L'arène des 10⁷ nœuds place ses
positions au 1/256 de pixel (`FRAC_BITS = 8`, déduit de la portée) ; son épreuve exigeait « un
quart de pixel à ×50 ». Le quart de pixel servait le placement du texte, qui se calcule **dans**
le nœud, en `f64`. Ce qui se verrait vraiment, c'est un glisser qui saute de plus d'un pixel :
**au zoom maximal, une unité du document vaut un pixel d'écran**. Donc **×256 = 2⁸** — le maximum
de Figma, trouvé sans le chercher. La molette va jusqu'au modèle. Le module `fixed` est
désormais atteint par le noyau (le cliquet 3 l'a vu). 2 sabotages, 2 chutes.

## 7. P4 — l'interface devient une donnée

**La recherche** (sources au § 10) : VS Code (identifiants de commandes, `keybindings.json`,
`when`, `-commande`), Zed (`base_keymap` sous les liaisons de l'utilisateur, `null` qui retire,
le contexte le plus précis qui gagne), Blender (une *keyconfig* est une donnée ; « Industry
Compatible » en est une autre), Krita (des schémas `.shortcuts` Photoshop et SAI, à part des
espaces de travail), PureRef 2 (la palette qui cherche chaque commande, des raccourcis entièrement
personnalisables).

**Ce qui est fait :**

* **Le noyau** (`glucose-core::raccourcis`, sans dépendance) : l'**accord** et sa loi
  **ACCORD-1** — Ctrl compte toujours ; Alt seulement sans Ctrl (Ctrl+Alt est AltGr sous Windows) ;
  Maj pour une lettre ou une touche nommée, pas pour un signe ni un chiffre (Maj les a écrits) ;
  un chiffre est la touche physique. C'était la loi implicite qui faisait marcher l'AZERTY ; elle
  est écrite, et une épreuve la tient. La **couche** : un texte, une liaison par ligne, l'ordre
  du fichier gardé (le premier accord écrit pour une commande est celui que le menu dit). Le
  **clavier** : la pile ; la couche du dessus remplace ou retire (`-`).
* **La table des commandes** (`commandes.rs`) : 75 commandes, une énumération fermée — chacune
  son identifiant stable, son nom, sa répétition (KEY-2 n'est plus une liste de touches à part) —,
  exécutées par **un seul `match` sans joker** : une commande sans geste ne compile pas.
* **Les raccourcis de Glucose** sont une couche embarquée (`assets/raccourcis/glucose.txt`) :
  exactement ceux d'avant. Les cinq familles de `match` du clavier ont disparu ; un accord désigne
  une liste — `1  relation.1 | signet.aller.1`, la première qui s'applique l'emporte.
* **Le menu exécute la commande**, et **dit les raccourcis lus dans le clavier** : ses
  raccourcis écrits à la main ont disparu (deux sources pour un même fait). `Ctrl+S` au clavier et
  « Enregistrer » au menu passaient par deux fonctions ; la commande prend celle du menu, juste
  partout.
* **La palette** (`Ctrl+Maj+P`, et au menu du vide pour le doigt) : une question de Glucose — un
  champ, les commandes trouvées avec leur raccourci, Entrée qui lance —, au PC comme au
  téléphone ; elle suit son champ à chaque image ; elle cherche sans majuscules ni accents.
  **Elle défile** (sa demande du soir : *« pouvoir scroller et voir tout »*) : une fenêtre de dix
  commandes sur toutes celles trouvées ; la molette une rangée par cran, le pavé une rangée par
  hauteur de rangée (les fractions s'ajoutent), les flèches et Tab déplacent l'évidence et la
  fenêtre la suit, Page haut et Page bas d'une fenêtre ; au doigt, « ↑ n plus haut » et « ↓ n
  plus bas » se touchent (une réponse qui défile sans répondre). P4 : 20 sabotages, 20 chutes.
* L'épreuve demandée par SUITE : **un raccourci changé par la table** (`Ctrl+K` duplique,
  `Ctrl+D` retiré).

**Ce qui reste de P4** : la couche de l'utilisateur, lue sur le disque et modifiable (« enregistré ») ;
le thème et la disposition des panneaux comme données ; les presets PureRef, Figma et Miro comme
couches ; Collaborer avec les plugins sur la même table.

## 8. La R&D : le cerveau d'image

`recherche/cerveau-image/main.cpp`, à sa demande, avec son `LISEZ-MOI` : une image analysée en
flux, en arithmétique entière (un RP2040) — Welford, GLCM et Haralick, **une pyramide de filtres
de Gabor** (λ = 4, 16, 64 px, quatre orientations), LBP, autocorrélation, tuilage —, résumée en
une signature de 80 octets. La piste de l'améliorateur d'image, pour plus tard.

## 9. Les leçons

* **Une limite côté serveur se paie côté client, en silence** : 512 Kio refusaient les sessions
  qui comptaient, et le client retenait le refus pour toujours. Un verdict se retient avec ce qui
  l'a causé (la version), et se retente quand la cause peut avoir changé.
* **Un déploiement se propage** : un essai trente secondes après refusait encore. Attendre la
  minute avant de chercher.
* **Une garde peut cacher un défaut** : `MAX_FONT_SIZE` empêchait le gel, et réduisait les
  lettres de près depuis toujours. Quand le coût ne dépend plus de la taille, la garde disparaît
  et le défaut avec.
* **Un seuil se mesure, il ne s'emprunte pas** : 256 (celui de Skia) laissait ×16 au pire ; le
  croisement mesuré est vers 100.
* **Une affirmation d'épreuve qui tombe peut dire que le cas n'existe pas** : « un élément de
  formule de plusieurs lettres » n'existe pas ; le type le dit désormais.
* **Une loi implicite est une loi** : le clavier français marchait parce que cinq `match`
  ignoraient Alt et Maj par endroits. Mise en table, elle devait être écrite, ou elle se perdait.

## 10. Sources

* Slug, domaine public : Hackaday, « Slug Algorithm For On-GPU Rendering Of Fonts With Bézier
  Curves Now In Public Domain » ; Gigazine, 23/03/2026 ; le dépôt `EricLengyel/Slug`.
* Skia, les grandes lettres en tracés : `GrContextOptions.h` (« glyphs are drawn as individual
  paths », 384 Android, 256 macOS, 324 ailleurs), revues de code de Chromium 1183053005,
  1202433002.
* VS Code : « when clause contexts » (code.visualstudio.com/api/references/when-clause-contexts).
* Zed : « Key bindings » (zed.dev/docs/key-bindings).
* Blender : `bf-blender-cvs`, « Industry Compatible Keymap: initial keymap » (avril 2019) ;
  `bl_keymap_utils/io.py`.
* Krita : « Shortcut Settings » (docs.krita.org/Shortcuts).
* PureRef 2 : pureref.com/blog/pureref2, pureref.com/handbook/shortcuts.

## 11. La beta 4, refaite

Un premier brouillon avait été déposé l'après-midi (sur `b6a8ea4`). LIBERE-1, la reprise enrichie
et la palette sont venus le soir : ce brouillon, le nôtre, que personne ne voit, a été retiré, et
la publication relancée après la CI dix vertes de `a7893be` (exécution 38769058177) — le
brouillon sur `a7893be` (publication 38771020014). L'APK, signé `3f2af005…`, est zippé sur son
Bureau (`Glucose-2.0.3-beta.4-telephone.zip`). Son essai au téléphone le 16/10, puis son clic.

