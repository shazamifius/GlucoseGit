# 66 — La mémoire du téléphone : la voir, la rendre, ne jamais prendre plus que ce qui reste

> Session du 14/10/2026. Le programme de la fiche 65 § 13 commence par **P0 —
> ANDROID-MEMOIRE** : son téléphone s'est figé tout entier — le bouton d'alimentation compris —
> après des saccades qui montaient, puis a redémarré seul ; *« c'est bien Glucose »*. En ouvrant,
> sa rectification : *« la 2.0.3-beta.2 est sortie […] peut-être que les logs pourront
> t'aider »*.

## 1. Ce qui est vérifié en ouvrant

* **La 2.0.3-beta.2 est publiée** : le 13/10 à 14 h 50 UTC, sur `82c774b` (sa CI : dix vertes,
  exécution 38412206828 ; la publication, 38414442808). Au 14/10 : l'installeur Windows
  téléchargé 3 fois, l'APK aucune, `latest.json` lu 4 fois. Les documents la disaient encore à
  faire : corrigés.
* **La télémétrie** (le tableau de bord refait) : cinq installations. **Un nouvel utilisateur
  Windows** (`8392c7af`), resté en 2.0.1-beta.1 jusqu'au 13/10, quatre sessions envoyées d'un
  coup — trois « interrompues, appareil redémarré », mais 14 h, 5 jours et 7 jours après leur
  dernière trace : un PC éteint sans fermer Glucose, sans lien avec lui ; sa version ne notait
  pas la carte. `5280efb2` (GTX 1660 SUPER) est monté en 2.0.3-beta.1 ; son PC en 2.0.3-beta.2.
  **Le téléphone n'a rien envoyé depuis le 13/10** : la session du gel est encore sur lui.
* **Le téléphone n'a pas été branché** pendant la session : sa lecture par le câble reste la
  première chose à faire (§ 7). Un script la fait d'un coup : `outils/android/lire_le_telephone.sh`.
* Le jeton de `wrangler` avait expiré (erreur 7403) : `npx wrangler whoami` le rafraîchit.

## 2. La recherche

* **Android ne parle plus à l'application qu'on regarde.** Depuis Android 14, `onTrimMemory` ne
  livre plus `RUNNING_MODERATE`, `RUNNING_LOW` ni `RUNNING_CRITICAL` : au premier plan, aucun
  signal de pression ; seulement `UI_HIDDEN` et `BACKGROUND`
  ([ComponentCallbacks2](https://developer.android.com/reference/android/content/ComponentCallbacks2)).
* **Le seuil que le système publie est trop tardif pour servir de budget.** Lu dans AOSP
  (`ProcessList.getMemoryInfo`) : `availMem` est exactement le `MemAvailable` de
  `/proc/meminfo` — ce que Glucose lisait déjà —, et `threshold` vient d'une table des années
  2010 (`mOomMinFree`), mise à l'échelle par la mémoire et l'écran : environ **216 Mo** sur un
  téléphone d'aujourd'hui.
* **Même Google n'a pas de budget à donner** : sa *Memory Advice API* (archivée) devinait la
  mémoire disponible par un modèle TensorFlow Lite, alertait sous 20 % et 15 %, et lançait un
  processus « canari » de 256 Mo pour voir s'il se faisait tuer
  ([la source](https://android.googlesource.com/platform/frameworks/opt/gamesdk/+/refs/heads/main/games-memory-advice/)).
* **Le gel lui-même** : lmkd, guidé par la pression (PSI), tue d'abord ce qui est caché ; mais
  « si l'application qui s'étouffe est celle avec laquelle l'utilisateur interagit, lmkd ne doit
  pas la tuer » (un commit d'AOSP), et la documentation d'Android prévient qu'en tuant ses propres
  processus sous la pression, « le téléphone peut sembler redémarrer »
  ([lmkd](https://source.android.com/docs/core/perf/lmkd),
  [memory-management](https://developer.android.com/topic/performance/memory-management)).
  C'est son récit mot pour mot — **à confirmer par le téléphone**, pas encore prouvé.
* **Borner les décodages** : Chromium compte ses images décodées et verrouillées contre un
  budget en octets (`ImageDecodeCache`, *working set*) ; Firefox a connu exactement notre défaut
  sous Android — décoder en pleine résolution, puis l'allocation qui échoue — et l'a réglé en
  décodant à la taille affichée (bug 1124084, *downscale-during-decode*) ; Glide dimensionne
  ses caches en écrans × octets, d'après `memoryClass`. `zune-jpeg` (le décodeur JPEG d'`image`)
  ne sait pas décoder à taille réduite ; `image` borne ses décodeurs à 512 Mio par défaut.

## 3. Ce que le code faisait sous Android — quatre trous

1. **Glucose ne voyait pas sa propre mémoire.** L'empreinte du processus (EMPREINTE-1) était
   écrite pour `target_os = "linux"` ; Android est `android` pour le compilateur : rien ne se
   mesurait, et les étages du magasin ne s'y relevaient jamais. Et sous Linux, la mémoire
   résidente se comptait en pages de 4 096 octets supposées — fausse d'un facteur quatre sur les
   téléphones à pages de 16 Kio qu'Android 15 introduit.
2. **Le signal d'Android arrivait, et Glucose l'ignorait.** GameActivity transmet tout
   `onTrimMemory` comme `APP_CMD_LOW_MEMORY` (son niveau se perd en route), `winit` l'appelle
   `memory_warning` — aucune ligne du dépôt n'y répondait.
3. **La sonde du budget de la carte ne parle qu'à DXGI.** Sous Android, la carte graphique garde
   « ce que l'écran demande » et rien de plus : borné, mais sans budget lu. (`wgpu` active déjà
   `VK_EXT_memory_budget` quand le pilote l'offre ; on ne sait pas si celui de sa PowerVR l'offre.)
4. **Les décodages n'étaient bornés par rien.** L'atelier décode sur tous les cœurs moins un —
   sept sur son téléphone —, et chaque photo passait par trois copies pleine taille vivantes
   ensemble : l'image décodée, sa copie en RGBA, la pyramide. Sept photos de 50 Mpx (celles de
   son appareil) demandaient plus de trois gigaoctets d'un coup, sans regarder ce qui restait.

## 4. MEMOIRE-1 — voir

* `plateforme::empreinte` mesure aussi sous Android, et lit `VmRSS` dans `/proc/self/status`,
  en kibioctets, sans taille de page supposée.
* **Une ligne `memoire` dans la boîte noire qui voyage** : ce que Glucose occupe
  (`processus_mo`), ce que l'appareil laisse (`disponible_mo`), les images tenues et offertes,
  les décodages en vol, ce qui est confié à la carte (`carte_mo` — sa propre comptabilité, la
  même partout). Écrite **quand c'est un record** — l'empreinte la plus haute de la session, ou
  ce qui reste le plus bas —, la règle de la ligne `pire` : aucune période choisie. Une session
  sage en écrit quelques-unes au début ; une session qui gonfle les écrit marche après marche.
* Le serveur les connaît (redéployé : versions `e29eaef9`, puis `19a590fe` avec
  `decodages_mo`), essayé en ligne puis effacé ; la page publique (`docs/TELEMETRIE.md`) et le
  `LISEZ-MOI.txt` du journal les décrivent. **Le `LISEZ-MOI` ne disait pas les lignes de
  BOITE-ANDROID** : son épreuve tenait une liste écrite à la main, à laquelle elles n'avaient pas
  été ajoutées. Elle est désormais liée à la liste fermée du serveur.

## 5. MEMOIRE-2 — obéir

`memory_warning` → `GlucoseApp::rendre_la_memoire` : la ligne `alerte_memoire`, ce que Glucose
tenait à cet instant (une ligne `memoire` forcée), puis — **au premier plan**, la carte rend tout
ce que l'écran ne pose pas, cache compris ; **derrière une autre application**, elle rend tout,
et le magasin **offre au système** chacun de ses niveaux (`Magasin::offrir_tout`) : rien n'est
jeté, le système reprend ce qu'il lui faut sans rien écrire, et le retour retrouve le reste. La
première image du retour reprend ce que l'écran et la vue d'ensemble demandent — par
`rentrer`, qui remet déjà l'image à revoir (ma ligne qui le faisait aussi était redondante : le
sabotage l'a montrée, elle est retirée).

## 6. MEMOIRE-3 — ne jamais prendre plus que ce qui reste

* **L'admission** (`renderer/atelier/admission.rs`) : avant de décoder, l'ouvrier lit les
  dimensions dans l'en-tête, en tire le **pic** — les octets encodés, la pyramide sommée niveau
  par niveau, et le travail du décodeur —, et ne commence que si l'ensemble des décodages en vol
  tient dans la moitié de ce que la machine laisse **maintenant** — la règle du cache
  (`Memoire::part_pour_un_cache`). Un décodage seul passe toujours. La porte est celle du
  processus, et elle n'offre que `pendant(octets, travail)` : le jeton vit exactement le temps du
  travail, aucun nom de variable ne peut le lâcher trop tôt.
* **Le décodage sur place** (`renderer/photo/decodage.rs`) : le décodeur écrit sa couleur
  d'origine dans la page du niveau natif, puis chaque pixel s'étale en RGBA prémultiplié **de la
  fin vers le début** — le pixel `i` se lit en `[k·i, k·i + k)` et s'écrit en `[4·i, 4·i + 4)`,
  et tout pixel `j < i` est lu avant `k·(j + 1) ≤ 4·i`. Les mêmes octets que le chemin d'avant,
  au bit près, pour le gris, le gris et alpha, le RGB, le RGBA, en PNG et en JPEG ; le seize bits
  garde le chemin d'avant.

**Mesuré** (`examples/bench_decodage`, une photo de 50 Mpx, chaque chemin dans son processus) :

| JPEG de 50 Mpx | chemin d'avant | sur place | pyramide |
|---|---|---|---|
| ordinaire (celui des appareils) | 445 Mo | **254 Mo** | 254 Mo |
| progressif 4:2:0 | 445 Mo | **292 Mo** | |
| progressif 4:4:4 | 445 Mo | **437 Mo** | |

**Ce que la mesure a démenti** : ma borne du travail du décodeur, tirée du papier (« chaque
coefficient sur seize bits, six octets par pixel »), comptait 300 Mo de trop pour cette photo —
plus que sa pyramide. Le pire mesuré est 3,8 octets par pixel ; la borne en compte quatre, et dit
d'où ils viennent.

## 6 bis. FANTOME-1 — le fantôme part toujours du curseur (P1)

Sa demande du 13/10 : *« la preview se met au milieu de la souris, alors que lorsqu'on trace le
rectangle, ça se met en haut à gauche — il faudrait absolument que ça se mette toujours en haut
à gauche de la souris ; de même pour la membrane »*.

* **La grille armée du Storyboard** se centrait sur le curseur (`disposition_au_clic`, héritée de
  Tauri) ; le tracé, lui, part du point d'appui : le fantôme sautait d'une demi-grille dès que la
  main glissait. Désormais **la première case a son coin haut gauche sous le curseur**, au survol
  comme au clic — le point même d'où un glisser tracerait. La loi du centrage (`Grille::coins`)
  n'était plus appelée que par sa propre épreuve : retirée.
* **La membrane et le dossier** avaient déjà leur coin au curseur ; l'aimant peut les tirer de
  quelques pixels vers un voisin, ce qui est voulu (PLACEMENT-1). Une épreuve le garde désormais
  pour les deux, par le vrai chemin du tracé.
* **Regardé** : l'aperçu hors écran (`apercu_storyboard`), la place du curseur marquée — le coin
  de la première case est dessous. **Éprouvé à la vraie souris** (le survol, puis le clic) ;
  **3 sabotages, 3 chutes** (`outils/sabotages/fantome-1.json`).

**Ce que P1 a révélé — GLISSER-2.** L'épreuve de STORYBOARD-2 « glisser puis lâcher une case se
défait **exactement** » posait sa grille en (0,3 ; 0,7) pour éprouver les flottants. Posée par son
coin, l'image revient **deux ulps** à côté après l'annulation. GLISSER-1 n'a jamais promis
l'annulation exacte : un glisser s'écrit en un **pas**, et défaire soustrait ce que faire a
ajouté — `(x + d) − d` n'est pas toujours `x`. Ce qu'il garantit tient (`Échap` rend les départs
exacts, le fichier rejoue l'écran au bit près). L'épreuve promettait plus, et passait par chance
des coordonnées. Elle dit désormais ce qui est exact — chaque case, à l'aller et au retour — et
ce que l'œil voit — l'image dans sa case —, sans tolérance choisie. **Une annulation exacte des
images** demanderait au journal leurs départs eux-mêmes : une édition neuve, donc un format neuf
(GLISSER-2, SUITE).

## 7. Ce qui reste — et ce qui déclare P0 fini

1. **Lire le téléphone par le câble** : `bash outils/android/lire_le_telephone.sh <dossier>` —
   la raison du redémarrage (`sys.boot.reason`), les morts de Glucose qu'Android a notées
   (`dumpsys activity exit-info`), la boîte des incidents, les journaux, la mémoire, le pilote,
   et un rapport de bogue complet. **Puis** ouvrir Glucose une fois, pour que la session du gel
   parte.
2. **Un APK neuf** sur son téléphone, puis une longue session : ses lignes `memoire` diront si
   l'empreinte monte, et ses `alerte_memoire` si Android a parlé.
3. Fini quand : une longue session sur son téléphone sans saccade qui monte ni gel, et ses lignes
   lues.
4. Plus tard, si le téléphone le désigne : le budget de la carte par `VK_EXT_memory_budget` hors
   de Windows (la sonde l'ignore encore), et décoder à la taille affichée.

## 8. Les erreurs de la session — ce que j'ai cru et que la vérification a démenti

* **La borne de papier** du travail du décodeur (six octets par pixel) : la mesure dit 0 pour un
  JPEG ordinaire, 3,8 au pire.
* **Une ligne redondante** dans `offrir_tout` (remettre chaque image à revoir) : `rentrer` le
  faisait déjà ; son sabotage ne faisait rien tomber. Retirée.
* **Une attente du disque non gardable** dans l'alerte : aucune épreuve ne pouvait la faire tomber
  sans dépendre de l'ordre des fils. Retirée — la ligne `arriere_plan`, elle, attend le disque.
* **Une négation dans une couche qu'aucune épreuve n'ouvre** (la présentation graphique, qui veut
  une fenêtre) : descendue dans la scène, éprouvée sur une vraie carte.
* **Des épreuves qui auraient bloqué** au lieu de tomber si la règle d'admission cassait :
  réécrites avec un fil et un délai.
* **Une épreuve qui promettait plus que le système** (fiche 65) : « défaire rend l'image
  exacte » ne tenait que par la chance des coordonnées — FANTOME-1 l'a montré (§ 6 bis).
* **Un `\n` mangé par Git Bash**, encore, dans un script Python passé par un *heredoc* :
  corrigé à l'outil d'édition.

## 9. Ce qui est prouvé

Les épreuves de MEMOIRE-1, 2 et 3 ; **35 sabotages, 35 chutes** : 17 pour MEMOIRE-1 et 2
(`outils/sabotages/memoire.json` — le dix-huitième visait la ligne redondante, retirée), 18 pour
MEMOIRE-3 et le `LISEZ-MOI` (`outils/sabotages/memoire-3.json`). Clippy strict sous Windows et
pour Android ; le serveur, ses sept épreuves et deux essais en ligne. **Ce qui n'est gardé que
par une mesure** : que le décodage sur place ne tienne rien de plus que la pyramide — c'est
`bench_decodage` qui le dit, pas une épreuve (un pic de mémoire ne se mesure pas au milieu de la
suite, où seize épreuves tournent ensemble). **Rien n'est jugé sur son téléphone.**
