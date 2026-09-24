# 32 — La mémoire par étages

> **Rôle de ce document.** L'utilisateur a approuvé, le 24/09, le plan en sept étapes de la
> mémoire par étages — *« utiliser et la VRAM et la RAM et le SSD, et dès qu'une application
> utilise trop la RAM ou la VRAM, on switch ; toujours prio la VRAM, ensuite la RAM, ensuite
> le SSD ; et que le logiciel s'adapte en temps réel tout le temps »* — et demandé qu'on
> vérifie aussi tout ce que les fiches 29 à 31 avaient laissé ouvert. Cette fiche dit ce qui
> a été construit, ce que la mesure a établi et démenti, et ce qui reste.
>
> **Date** : 2026-09-24 · commits `bb28ef6` à `ab928db`, et celui de cette fiche.
> **État vérifié** : `cargo test --workspace` exit 0, **1 549 tests verts** (1 523 au départ),
> clippy strict à zéro, `cargo fmt --check` à zéro, aucun plafond relevé — cinq extractions à
> la place (le nuanceur des photos, la pose d'une photo, le mot d'accueil, la vue d'ensemble,
> le relevé du survol).
>
> **Rien de ceci n'a encore été vu à l'écran.** Tout est prouvé par des épreuves
> déterministes, chacune sabotée puis restaurée ; rien n'est « réglé » au sens de la charte
> tant qu'il n'a pas joué une session (§ 12).

---

## 1. Ce que l'utilisateur a dit, et sa session de 35 secondes

* *« Absolument tout me va dans ce que tu as proposé. »*
* *« Plus du tout d'images en cascade, tout est d'une fluidité complètement folle. »* —
  NIVEAU-GPU-1 (fiche 30) est donc vu, et bon.
* Sa chronique (`sortie-courte-2026-09-24.txt`, 35 s, 1 850 images) : **1 543 Mo** de mémoire
  de travail **dès la fin de l'ouverture** — pas une croissance —, 595 Mo sur la carte, un
  budget de 7 123 Mo accordé ; 72 images par seconde entre deux images consécutives, le tempo
  à 3-4 balayages.

---

## 2. La mesure d'abord — et une erreur de ma part

J'ai d'abord pesé les épingles de sa longue session (235 Mo décodées) et le document « random
photo » (131 Mo), puis écrit `bench_memoire` : avec la vraie carte et ces 77 images, **505 Mo**.
J'en ai conclu, et je le lui ai dit, qu'*« un gigaoctet n'est pas les images »*. **C'était
faux** : je pesais le mauvais document. Celui qu'il ouvre, `fusée CS\fuser.glucose` (180 Mo),
porte **243 images toutes différentes** — 222 mégapixels, **1 186 Mo** de pyramides. Avec la
carte, `bench_memoire` le reproduit : 1 248 Mo. La mémoire, c'étaient les images.

Ce que le banc a établi au passage :

| | |
|---|---:|
| ouvrir la carte, allocateur en réglage `Performance` | **+198 Mo** de mémoire graphique, pour rien |
| le même, en réglage `MemoryUsage` | +18 Mo |
| redécoder une image sur un fil | 13-16 ms par mégapixel |
| redécoder les 243 par l'atelier (15 fils) | 0,43 s |

---

## 3. La recherche

| ce qui existe | ce qu'on en retient |
|---|---|
| Unreal, *texture streaming* ([doc](https://dev.epicgames.com/documentation/unreal-engine/texture-streaming-metrics-in-unreal-engine)) | une petite version **toujours résidente**, la finesse voulue quand le budget le permet |
| Microsoft, *Residency* ([doc](https://learn.microsoft.com/en-us/windows/win32/direct3d12/residency)) | un processus au-delà de son budget est *« gelé par intermittence »* ; le budget se signale par un événement |
| Vulkan `VK_EXT_memory_budget`, Metal `recommendedMaxWorkingSetSize` | la même question, posée ailleurs ; aucune n'est encore câblée ici |
| `OfferVirtualMemory` / `ReclaimVirtualMemory` ([doc](https://learn.microsoft.com/en-us/windows/win32/api/memoryapi/nf-memoryapi-offervirtualmemory)) | le système reprend une mémoire offerte **au moment exact** où il en a besoin, sans l'écrire sur le disque |
| Chromium ([mémoire](https://chromium.googlesource.com/chromium/src.git/+/master/docs/memory/key_concepts.md)) | la même idée (*discardable*) — et sa limite : une page offerte reste **engagée** |
| Figma ([images](https://help.figma.com/hc/en-us/articles/360052988373-Image-loading-and-performance)) | la haute résolution pour ce qui est à l'écran, une réduite ailleurs |
| PureRef ([forum](https://www.pureref.com/forum/read.php?2,1947)) | tout non compressé en mémoire — ce qu'on doit battre |
| Lightroom ([aperçus](https://www.lightroomqueen.com/lightroom-performance-previews-caches/)) | des aperçus sur le disque, pour ne jamais redécoder à l'ouverture |
| BC7 / ASTC ([bc7enc](https://github.com/richgel999/bc7enc)) | quatre fois plus d'images dans la même carte, presque invisible — étape 7 |

---

## 4. ETAGES-1 — tenu ce que l'écran montre, offert tout le reste (`bb28ef6`, `7fcece1`)

**La règle, et elle ne choisit aucun nombre** : chaque niveau de chaque pyramide est **tenu**
tant que l'écran le montre (le niveau voulu de chaque image visible) ou qu'il appartient à la
**vue d'ensemble** (l'échelle où le tableau entier tient dans l'écran, déduite des bornes du
document) ; tout le reste est **offert** à Windows, qui le jette s'il manque de place. Offrir et
reprendre coûtent 3 et 4,6 ms pour 10 Mo (`bench_offre`) : ce sont les ouvriers de l'atelier qui
le font, par trois files lues par urgence — reprendre, décoder, offrir. Un niveau perdu se
redécode. C'est l'ordre qu'il demandait, chiffré : revenir de la carte ne coûte rien, de la
mémoire offerte ~2,4 ms par mégapixel, du disque ~16.

| `bench_memoire`, ses 243 images | mémoire de travail | tenu | offert |
|---|---:|---:|---:|
| tout décodé, avant | **1 248 Mo** | 1 131 Mo | 0 |
| l'écran les montre à 150 px | **164 Mo** | 45 Mo | 1 087 Mo |
| plus rien à l'écran | **120 Mo** | 1 Mo | 1 130 Mo |
| tout revient à 150 px | 164 Mo | 45 Mo | 1 087 Mo — en 12 ms d'atelier |

**La limite, dite** : la mémoire *engagée* ne baisse pas — une page offerte reste promise tant
que le système ne l'a pas reprise. ADAPT-1 reste donc la seconde ligne, sur l'ensemble.

Ce qui a changé autour :

* la pyramide : un état par niveau (tenu, offert, en chemin, perdu), et lire un niveau le
  marque **voulu** ; `meilleur_pour` prend le voulu, sinon un plus petit, sinon un plus grand ;
* la carte demande toujours le niveau **voulu** ; le meilleur tenu ne se pose que sous une
  identité à lui, quand elle ne détient rien d'autre — sous celle de la photo, il aurait
  remplacé une texture nette par une floue au retour d'une photo sortie de l'écran (trouvé en
  relisant, avant tout test) ;
* `Ctrl+B` lit l'original, presque toujours offert : le lot **attend** ses originaux sans
  geler, puis s'applique d'un bloc, un seul `Ctrl+Z`. Deux défauts trouvés en l'écrivant — une
  photo hors de l'écran n'était jamais revue, et rien ne réveillait l'application pour
  appliquer le lot — et **l'épreuve ne les voyait pas** : les deux sabotages passaient, l'un
  parce qu'un retour d'offre récent remettait la photo à revoir par hasard, l'autre parce que
  le mot d'accueil réveillait l'application. Durcie, elle les attrape ;
* la chronique dit la mémoire vive étage par étage : tenu, offert, allers-retours, lettres.

---

## 5. ETAGES-2 — Glucose rend la carte même quand il dort (`303ad84`)

Le budget ne se relisait qu'à chaque image dessinée ; au repos, jamais. Un fil dort maintenant
sur l'événement que Windows signale à chaque changement de budget
(`RegisterVideoMemoryBudgetChangeNotificationEvent`) et réveille la boucle, qui rend ce que le
cache de textures garde au-delà de sa part. L'allocateur de la carte passe en réglage économe :
l'écart de temps est dans le bruit (243 textures en 176-219 ms contre 179-188).

---

## 6. ETAGES-3 — quand la carte manque de place, un cran pour toutes (`a30753b`)

Ce que la carte **montre** n'était borné par rien. Les photos à l'écran perdent désormais le
même nombre de crans — le plus petit qui fasse tenir leur demande dans la part que le budget
leur laisse (le budget, moins tout ce que Glucose tient d'autre sur la carte, cache excepté).
Si aucun cran ne suffit, l'image se compose sur le processeur, et se rejuge à chaque image.
**Il ne se déclenchera pas sur sa RTX** (7 Go de budget pour ~0,6 Go à l'écran) : c'est la
garde des machines à petite carte, et des jours où Blender la prend.

---

## 7. ETAGES-4 — une image vue une fois s'ouvre déjà montrée (`1fad4ca`)

La vue d'ensemble de chaque image se garde sur le disque, dans le dossier de l'application.
**Ce qui rend l'idée juste** : plus un document a d'images, plus chacune est petite dans sa vue
d'ensemble — la somme vaut **environ un écran de pixels**, qu'il en compte cent ou dix
millions. La place disque est bornée par l'écran, et aucune borne n'a été choisie. Un aperçu
donne une pyramide partielle : ses grands niveaux se disent perdus, et l'écran qui les voudra
les fera redécoder — le chemin d'une page que Windows a jetée.

| `bench_apercus`, `fuser.glucose`, écran 2160 × 1350 | toutes les images ont de quoi se montrer |
|---|---:|
| première ouverture (tout décoder) | **462 ms** |
| ouvertures suivantes | **8 ms** |
| sur le disque | 11,8 Mo pour 243 images |

---

## 8. Les restes des fiches 29 à 31, tranchés

| | ce qu'on croyait | ce qui est établi |
|---|---|---|
| **A** | *« les images au repos sont des animations — les messages qui s'effacent »* (fiche 31 § 1) | **non établi** : la provenance comptait tout message *présent*, même endormi sur son plateau. Elle ne compte plus qu'une raison qui a **demandé** l'image (`mark_dirty`), et raisons et attentes forment une seule table (`a30753b`) |
| **B** | le gel de 2,46 s dans « écouter la main » vient d'une fenêtre de Windows | les fenêtres sont déjà oubliées de la chronique (DIAL-2) — **ce n'est pas elles**. En cherchant : coller une image l'encodait en PNG dans le geste, 15 à 150 ms — **COLLER-1** (`ab928db`). La cause des 2,46 s reste inconnue : **question à lui poser** |
| **C** | la cadence à 190 nœuds | non attaquée de front ; D et E en retirent deux causes de queue |
| **D** | les panneaux à 5,8 ms sur une sélection | la clé du cache gardait la **position exacte** du pointeur : 76 redessins en 35 s. Un panneau ne dépend du pointeur que par ses questions de survol ; mêmes réponses, mêmes pixels — **SURVOL-2** (`6daba0e`) |
| **E** | `textures` à 28 ms sur une image de zoom | la cascade vérifiait qu'il restait du temps, sans prévoir ce que coûterait la texture. Elle prévoit maintenant sur son débit observé, par pixel posé — **CASCADE-3** (`a30753b`) |
| **F** | la latence « p90 = p99 = pire » : instrument cassé | **non** : l'histogramme range par tranches de 19 %, et le centile est le haut de la tranche plafonné au pire. La latence ne dépasse jamais ~18 ms |
| **G** | le cache de lettres | borné en nombre, pas en octets : 54 Mo mesurés à un zoom ×14. Désormais dans la chronique ; pas changé |

Trans-domaines : **pas touché**. La gravure et la clé SauceNAO : toujours à lui demander.

---

## 9. Un plantage rare

La suite complète a planté **une fois** (`STATUS_ACCESS_VIOLATION`, épreuves de la
bibliothèque), et pas dans les trente passages suivants de l'ancien exécutable. En relisant le
code `unsafe` : la reprise passait à Windows une **tranche Rust** sur des pages qu'il avait
rendues inaccessibles — et une référence autorise le compilateur à y lire par anticipation,
ce qui fait exactement tomber le processus. Corrigé (`7fcece1`) : l'offre et la reprise ne
parlent plus que par pointeur brut. **Probablement la cause, pas prouvé** — il ne se
reproduisait pas : un plantage en trente-cinq passages avant la correction, zéro en vingt
passages de l'exécutable corrigé (850 épreuves chacun). À surveiller : s'il revient, la
référence n'était pas la cause.

L'application elle-même a été lancée huit secondes, sans y toucher, pour vérifier que son
démarrage — le réveil de la boucle, le veilleur de la carte, l'allocateur économe, les aperçus —
ne tombe pas : vivante, 193 Mo sur le document d'accueil.

---

## 10. Ce que cette session a démenti — la liste

1. **« Un gigaoctet n'est pas les images »** — le mien, sur le mauvais document (§ 2).
2. **« Les images au repos sont des animations »** (fiche 31) — l'instrument comptait les
   messages endormis (§ 8 A).
3. **« La latence p90 = p99 = pire est un instrument cassé »** — c'est sa résolution (§ 8 F).
4. **« Le collage est le gel de 2,46 s »** — 150 ms au plus (§ 8 B).
5. **Mon épreuve de `Ctrl+B`** — elle passait avec les deux sabotages (§ 4).
6. **Mon épreuve de la prévision** — l'historique semé se diluait dans la première mesure,
   ce qui est justement le comportement voulu (CASCADE-3).
7. **« Le code consigné est propre »** — il n'était pas passé par `rustfmt`, alors que le
   dépôt l'est depuis `a40250b` (`d0e62fd`). **Pour la suite : `cargo fmt --all` avant chaque
   commit.**

---

## 11. Ce qui reste, chiffré

| # | Piste | Ce qu'on sait | Ce qu'il faut |
|---|---|---|---|
| 1 | **Étape 7 : textures compressées** | BC7 ×4 de place, BC1 visible ; ASTC sur téléphone, ETC2 presque absent sur ordinateur ; détection à l'exécution. Un encodeur BC7 rapide « mode 6 seul » laisse des **blocs visibles** ([bc7enc_rdo](https://github.com/richgel999/bc7enc_rdo), [BC7E](https://github.com/BinomialLLC/bc7e)) — ce que la charte refuse | un banc sur **ses** images : qualité et coût d'un encodeur modes 1 + 6 écrit ici, contre une dépendance éprouvée. C'est un arbitrage de dépendance, à trancher par la mesure |
| 2 | Le gel de 2,46 s | ni fenêtre, ni `Ctrl+B`, ni collage | **ce qu'il faisait vers 4 min 16 s** |
| 3 | La cadence | tempo à 3-4 balayages (72 i/s) pour des images de 5-7 ms ; plancher de la charte : 100 | sa prochaine chronique, après SURVOL-2 et CASCADE-3 |
| 4 | Les autres plateformes | l'offre n'existe que sous Windows ; ailleurs, un tampon ordinaire | `vm_purgable_control` (macOS), `ashmem` (Android) ; les budgets Vulkan et Metal |
| 5 | Le dossier des aperçus | un écran par document, jamais nettoyé | une règle de nettoyage sans constante |
| 6 | Les panneaux à 20,9 ms | une image, juste après l'ouverture du document | non analysé |
| 7 | Le cache de lettres en octets | 54 Mo à ×14 | la chronique réelle |
| 8 | La gravure, SauceNAO, Trans-domaines | — | **sa parole** |

---

## 12. Ce qu'il faut tester à l'écran

```text
cargo run --release > sortie-etages.txt 2>&1
```

1. **Ouvrir `fuser.glucose`**, et garder le gestionnaire des tâches ouvert à côté : la mémoire
   de Glucose devrait rester **quelques centaines de Mo**, et non 1,5 Go.
2. **Se promener, zoomer très près d'une photo, dézoomer jusqu'à tout voir** : rien ne doit
   apparaître vide ; dire si une photo est floue un instant avant de devenir nette.
3. **`Ctrl+B` sur une grande sélection** : il doit s'appliquer après un battement, sans figer.
4. **`Ctrl+V` d'une grande image** (une capture d'écran) : plus de gel au collage.
5. **Garder la souris sur un panneau pendant un zoom au pavé** : le dire si c'est plus fluide.
6. **Fermer, puis rouvrir `fuser.glucose`** : les images doivent être là tout de suite.

Puis copier la chronique (`%TEMP%\glucose-chronique\derniere-session.txt`) avant de relancer :
la section de la mémoire dira tenu et offert, et la provenance dira enfin d'où viennent les
images au repos.

---

## 13. Ce que l'utilisateur a dit en fin de session

* *« Tout ça en une seule session, ça fait énorme, c'est parfait. »* — dit avant le test à
  l'écran : ce n'est pas une validation de ce qui s'y verra.
* **Le gel de 2,46 s était l'enregistrement** (`Ctrl+S`, un document de 180 Mo et 243 photos).
  L'enregistrement est aujourd'hui fait dans le geste, d'un bloc, sur le fil qui dessine.
  Sa parole : *« le système d'enregistrement de Glucose est clairement pas fini — juste
  fonctionnel, absolument pas là où je voudrais aller »*. Ce qu'il veut, plus tard : **un
  système à la git**, avec tout un *wayback* — revenir en arrière et voir l'état du projet à
  n'importe quel moment. **Glucose Tauri l'avait déjà implémenté** : les jalons durables
  (`.glucose.versions/`, fiche 09 § 2.1 et § 3.3 — un dossier `tst.glucose.versions/` écrit
  par Tauri existe encore sur son bureau), la Time Machine (fiche 09 § 7, fiche 10 § 5.7,
  `GlucoseTauri/src/components/TimelinePanel.tsx`). À étudier avant de concevoir, et à
  concevoir entier plutôt qu'à rapiécer : pour l'instant, l'enregistrement doit juste
  rester fonctionnel.
* **`Ctrl+B` « fonctionne extrêmement mal »**. Il donnera deux images à la session suivante
  qui l'illustrent. Les fiches 26 à 28 racontent sa construction et ses critères : à
  remettre en question sur ses images, pas à défendre.

---

**Retour** : [`00-INDEX.md`](00-INDEX.md)
