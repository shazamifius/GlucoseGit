# 50 — Le toucher : Android, et l'iPad par le web

> **Rôle de ce document.** La partie D du plan de la fiche [`46`](46-LA-SUITE-DANS-L-ORDRE.md) :
> Glucose chez ses **dix testeurs Android**, et chez **une personne qui compte énormément pour
> lui, sur iPad** — sans argent, donc l'iPad **par le web**, dans Safari. **Aucun code dans cette
> fiche** : la recherche faite le 30/09, ce qu'elle impose, et l'ordre des sessions à venir. Tout y
> est à remettre en question.
>
> **Date** : 2026-09-30.

---

## 0. En une page

* **Le toucher est commun** : des doigts, le pincement, aucun survol — puis le crayon sur iPad.
  Une couche de gestes au-dessus de ce que `winit` rend, la même pour Android et le web.
* **L'iPad par le web** : WebGPU est dans Safari 26, sur **tout iPad qui reçoit iPadOS 26** (puce
  A12 et après : iPad 8ᵉ génération, mini 5, Air 3, Pro 3ᵉ génération). Un iPad plus ancien
  passerait par WebGL2. **Deux obstacles d'architecture**, pas de détail :
  1. **les fils n'existent pas dans un navigateur** — le travail de fond de Glucose (scribe, atelier,
     boîte noire) doit y tourner **en tranches, dans le temps libre de chaque image** : la cascade
     de la charte, devenue une voie à part entière ;
  2. **Safari efface les données d'un site** qu'on n'a pas touché depuis un temps — sauf, *peut-être*,
     en mode persistant. Une version web qui perdrait un document serait pire que rien : chaque
     document devra vivre **aussi** dans les Fichiers de l'iPad.
* **Android** : `winit` y passe par `android-activity`. Le **clavier virtuel** — Glucose est fait
  d'écriture — impose `GameActivity` (et une construction Gradle), pas `NativeActivity`.
* **Ce qui attend sa parole** : le modèle de l'iPad ; les téléphones de ses testeurs.

---

## 1. Où en est le code, mesuré

| | combien | ce que cela veut dire |
|---|---:|---|
| endroits qui lancent un fil (`thread::spawn`) | **8** | l'atelier, le scribe, la boîte noire, la mise à jour, le presse-papiers, le rapatriement, la priorité, la carte graphique |
| fichiers qui touchent au disque (`std::fs`) | **33** | sous un navigateur, aucun ne marche tel quel |
| passages propres à Windows (`cfg(windows)`) | **40** | déjà rangés dans `plateforme/` pour la plupart |
| le noyau (`glucose-core`, `glucose-math`) | **0 dépendance** | compilé pour Android ARM 64 et 32 bits à chaque envoi (fiche 45 § 1) |

Le noyau voyage déjà. L'application, non : elle suppose des fils et un disque.

## 2. L'iPad par le web

### 2.1 La carte graphique

**Safari 26 active WebGPU par défaut** sur iOS et iPadOS 26 (WebKit). iPadOS 26 exige la puce
**A12** : iPad 8ᵉ à 11ᵉ génération, iPad mini 5ᵉ et après, iPad Air 3ᵉ et après, iPad Pro 3ᵉ
génération et après. Il ne perd que l'iPad 7ᵉ génération (A10). Sur un iPad resté sous iPadOS 18,
`wgpu` sait prendre **WebGL2** — moins puissant, mais de plein droit : aucune exclusion matérielle.

### 2.2 Les fils — le premier obstacle

En 2026, **`std::thread` ne fonctionne toujours pas** dans un navigateur : les fils y demandent la
version instable du compilateur (`atomics`), des *Web Workers* montés à la main, et des en-têtes
d'isolation (COOP/COEP) que GitHub Pages ne sait pas poser (web.dev ; `wasm-bindgen`, issue 2433).

Deux voies :

| | les fils du navigateur | **les tranches** |
|---|---|---|
| le compilateur | instable (*nightly*) | stable |
| l'hébergement | des en-têtes d'isolation (Cloudflare Pages) | n'importe lequel (GitHub Pages) |
| ce qu'il faut écrire | des *workers*, un pont par fil | un **exécuteur** : chaque travail de fond découpé, exécuté dans le temps libre de chaque image |
| la charte | — | *« une action lourde se fait en cascade, par tranches qui tiennent dans le temps libre de chaque image »* (le rendu n'attend jamais le travail) |

**Mon avis : les tranches**, comme une **voie** — sous Windows et Android, les fils restent ; sur le
web, les mêmes travaux passent par l'exécuteur. C'est le plancher tenu par l'algorithme, sans rien
d'instable. Les huit fils d'aujourd'hui se découpent inégalement : le scribe (des écritures) et la
boîte noire (des lignes) se tranchent naturellement ; le décodage d'une grande photo dans l'atelier
est le plus dur — c'est là que se mesurera la voie.

### 2.3 Le disque — le second obstacle

* **L'espace** : un site peut garder jusqu'à 60 % du disque (le système de fichiers privé de
  l'origine, OPFS) ; il n'existe pas en navigation privée.
* **L'éviction** : Safari efface **toutes** les données d'un site sans interaction depuis un temps
  (*Intelligent Tracking Prevention* ; MDN : sept jours d'usage du navigateur). WebKit : un site en
  **mode persistant** *« might be excluded »* — **peut** être épargné. Installé sur l'écran d'accueil,
  il a les mêmes quotas.
* **Donc, la règle** : `navigator.storage.persist()`, Glucose **installé sur l'écran d'accueil**, et
  **chaque document écrit aussi dans les Fichiers de l'iPad** — la seule copie que Safari ne peut
  pas effacer. La conception exacte (quand, comment, sans rien demander à chaque geste) est le
  premier travail de la phase web.

### 2.4 Ce qui reste à mesurer

* **La mémoire** qu'une page reçoit sur un iPad avant que le système la ferme — elle décide de la
  mémoire par étages sur le web.
* **Le crayon** : la pression et l'inclinaison, que les événements du navigateur donnent.

## 3. Android

* **La fenêtre et la boucle** : `winit` 0.30 passe par `android-activity`, avec deux portes. **Le
  clavier virtuel** ne marche bien que par **`GameActivity`** (et son `GameTextInput`) :
  `NativeActivity` demande le clavier, qui souvent ne vient pas (`android-activity`, issue 44 ;
  une amélioration des actions d'édition en 2026). `GameActivity` demande une construction Gradle.
* **La carte** : Vulkan ou OpenGL ES 3 par `wgpu` ; en deçà, la voie du processeur.
* **La vie d'une application** : suspendue, reprise, sa surface perdue et rendue.
* **Le disque** : son dossier privé ; les documents de l'utilisateur par le sélecteur du système.
* **La clé de l'APK** : Android n'installe une mise à jour que signée de la même clé — **à ne jamais
  perdre**, comme celle de Tauri (fiche 44 § 4).
* **Les épreuves** : les émulateurs sur GitHub, et de vrais téléphones anciens par Firebase Test Lab
  (gratuit, cinq par jour).

## 4. L'ordre proposé

1. **Voir sans posséder, pour le web** : le noyau compilé pour WebAssembly à chaque envoi, comme
   pour Android — une tâche de plus, sans coût.
2. **L'exécuteur des tranches** (§ 2.2), éprouvé d'abord **sous Windows** : les mêmes travaux, par
   l'une ou l'autre voie, doivent rendre les mêmes résultats — la règle des voies.
3. **Le toucher** : la couche de gestes, éprouvée par des événements fabriqués.
4. **Le web** : la fenêtre, la carte, le disque (§ 2.3) ; hébergé sur GitHub Pages.
5. **Android** : `GameActivity`, le clavier, la vie de l'application, l'APK et sa clé.

## 5. Ce qui attend sa parole

1. **Le modèle de l'iPad** (ou sa version d'iPadOS) : il décide entre WebGPU et WebGL2.
2. **Les téléphones Android de ses testeurs**, le plus ancien surtout : il fixe le plancher.

## 6. Les sources

* WebKit, [*WebKit Features in Safari 26.0*](https://webkit.org/blog/17333/webkit-features-in-safari-26-0/)
  — WebGPU ; Wikipédia, [*iPadOS 26*](https://en.wikipedia.org/wiki/IPadOS_26) — les iPad pris en
  charge, la puce A12.
* web.dev, [*Using WebAssembly threads from C, C++ and Rust*](https://web.dev/articles/webassembly-threads) ;
  `wasm-bindgen`, [*Current state of WASM threads*](https://github.com/rustwasm/wasm-bindgen/issues/2433).
* WebKit, [*Updates to Storage Policy*](https://webkit.org/blog/14403/updates-to-storage-policy/) —
  les quotas, le mode persistant ; MDN, [*Storage quotas and eviction criteria*](https://developer.mozilla.org/en-US/docs/Web/API/Storage_API/Storage_quotas_and_eviction_criteria).
* `winit`, [`platform::android`](https://docs.rs/winit/latest/winit/platform/android/index.html) ;
  `android-activity`, [issue 44](https://github.com/rust-mobile/android-activity/issues/44) ;
  Android, [*TextInput in GameActivity*](https://developer.android.com/games/agdk/game-activity/use-text-input).

---

**Retour** : [`00-INDEX.md`](00-INDEX.md)
