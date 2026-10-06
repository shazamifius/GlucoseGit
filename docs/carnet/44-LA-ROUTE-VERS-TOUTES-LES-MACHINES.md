# 44 — La route vers toutes les machines

> **Rôle de ce document.** Le 29/09, il reprend la question de la fiche [`36`](36-LA-ROUTE-VERS-LA-V1.md) :
> *« qu'est-ce qui pourrait faire qu'on puisse enfin publier le projet et avancer dessus ? »* Il
> propose des journaux « en béton » sur tous les appareils, une télémétrie avec un bouton
> « accepter », la bascule de Glucose Tauri vers Glucose Rust par la mise à jour automatique —
> *« sans JAMAIS perdre une version, et TOUJOURS avec notre mise à jour »* —, puis la **2.0.1
> bêta**. Ses précisions, en deux temps, ont refait l'ordre (§ 0). **Aucun code dans cette
> fiche** : c'est la route des sessions à venir. Elle remplace les phases 3, 8 et 9 de la fiche
> 36, et tranche sa première question.
>
> **Date** : 2026-09-29 · au commit `104154a`, révisée le même jour après sa réponse.

---

## 0. En une page

**Ce qu'il a précisé** :

1. **Cinq utilisateurs de Tauri, sous Windows et Linux** : on les fait passer **tous d'un coup**.
   La question « un moment ou deux » de la fiche 36 est tranchée : un seul.
2. **Une vingtaine de personnes attendent** la version Rust, **dont dix sous Android**, téléphones
   et tablettes — certains très anciens.
3. **Toutes les machines** : Windows, Mac à puce Apple et Mac Intel, Linux — **NixOS d'abord** —,
   Android, *« tout type d'OS »*.
4. **La vie privée** : *« ça m'importe énormément, je ne souhaite manquer de respect à
   personne »*.
5. **Le journal et la télémétrie restent un pilier** : *« ce serait réellement utile pour tous
   les plantages, les bugs, les problèmes, et juste voir où en est Glucose sur les très anciens
   téléphones, lorsqu'il commence à chauffer, lorsqu'il s'éteint à cause de la batterie […] un
   système de journal, pour un projet de R&D et un début d'application, c'est obligatoire »*.

**Il a raison sur le point 5**, et ma première version avait tort de ranger le journal « en
dernier recours » : pour une application de recherche qui démarre, avec vingt testeurs sur des
appareils que personne ici ne possède, **la boîte noire** — un journal complet, et la télémétrie
qui le rapporte avec leur accord — est la première source de vérité (§ 1). Les autres outils la
complètent : les épreuves sur tous les systèmes (voir **avant** de publier), le programme qui se
vérifie lui-même, et l'étalonnage — que la boîte noire nourrira.

**L'ordre** (§ 3) : voir sans posséder → la boîte noire → Linux et NixOS → distribuer et se
mettre à jour → **la 2.0.1 bêta, qui est la bascule** → Android → l'étalonnage appris des vraies
machines → Mac.

**Où en est la route** : la phase 1 et la première moitié de la phase 2 sont faites le 29/09 —
fiche [`45`](45-VOIR-SANS-POSSEDER-ET-LA-BOITE-NOIRE.md).

---

## 1. La boîte noire, et ce qui la complète

### 1.1 La boîte noire : ce qu'elle enregistre

Un enregistreur, comme celui d'un avion : il écrit **en continu**, sur le disque, ce qui se
passe — et **survit à la fin brutale** qu'il doit expliquer.

* **Ce qui change, quand ça change** : l'état thermique (Android le dit depuis Android 10, et la
  marge avant bridage depuis Android 11 ; ailleurs, ce que le système donne) ; la batterie — le
  niveau, la charge, la température, lisible sur **tous** les Android, même très anciens ; la
  mémoire que le système réclame ; la voie graphique choisie ; un gel, une erreur.
* **Le coût des images, résumé en continu** — ce que la chronique calcule déjà, mais au fil du
  temps, et non plus seulement en fin de session : c'est là que se voit **le bridage thermique**,
  le même travail qui coûte de plus en plus cher. La charte le dit : sur mobile, le bridage est le
  régime normal ; il faut donc le voir.
* **Comment la session précédente a fini**, au lancement suivant : proprement, par un plantage,
  par un gel, tuée par le système faute de mémoire, ou **l'appareil éteint** (la batterie). Sur
  Android 11 et après, le système donne lui-même la raison (`ApplicationExitInfo`) ; ailleurs,
  une session sans fin propre et un appareil qui a redémarré le disent — avec le dernier niveau de
  batterie et la dernière température que la boîte noire a écrits.
* **Les plantages** : un petit processus **témoin**, à côté, écrit l'état exact du programme qui
  tombe — où, dans quel pilote, quelle version —, même quand c'est le pilote de la carte graphique
  qui plante. Les outils existent en Rust et servent en production : `crash-handler` et
  `minidumper` (Embark), sous Windows, Linux, Mac et **Android sur ARM** — les téléphones ;
  l'analyse, par `rust-minidump` (Mozilla, celle des plantages de Firefox), se fait chez lui.
* **Le profil de la machine**, une fois par session : système et version, processeur, mémoire,
  cartes graphiques et pilotes, écrans.

Le journal **ne ralentit jamais l'image** (écrit par un fil à part) et **ne remplit jamais le
disque** (il tourne, et ses pertes se comptent). Rien de tout cela n'est un nombre choisi : on
écrit ce qui change, quand ça change.

**Ce qui part** — avec l'accord de la personne (§ 2) : la boîte noire, en petits lots, en
continu ; **tout ce qui a mal fini remonte avec ses dernières minutes**. Le testeur peut ouvrir sa
propre boîte noire dans Glucose, et voir ce qui part.

### 1.2 Ce qui la complète

1. **Voir avant de publier : des épreuves sur tous les systèmes, à chaque envoi.** GitHub prête,
   pour un dépôt public, des machines Windows (x64 et ARM), Linux (x64 et ARM), Mac Intel et Mac à
   puce Apple ; sans carte graphique, mais avec des **rendus logiciels** — WARP sous Windows, Mesa
   sous Linux, Metal sur Mac — : c'est ainsi que **wgpu**, la bibliothèque graphique de Glucose,
   éprouve ses propres rendus. **Vérifié le 29/09** : le dépôt est public, et ces machines ne lui
   ont rien coûté — en septembre, 4,94 $ bruts, entièrement remisés, **0 $ net**. L'intégration
   continue coupée le 15/09 « parce qu'elle facturait » reposait sur un malentendu ; et elle
   montrait déjà que **Glucose Rust compile et s'éprouve sous Linux** : 389 épreuves passaient, une
   seule tombait (le presse-papiers, faute d'écran). Pour Android : des émulateurs, et de **vrais
   téléphones** par Firebase Test Lab — cinq essais par jour, gratuitement.
2. **Le programme se vérifie sur la machine.** Au premier lancement, hors écran, il dessine une
   scène connue par la carte graphique et par le processeur, et compare au pixel près. Un pilote
   qui dessine faux est su **avant** de montrer quoi que ce soit, et le programme choisit l'autre
   voie pour ce qui cloche — un **choix délibéré**, pas une bascule par échec, que la charte
   interdit. C'est sur les vieux téléphones que les pilotes sont les plus fragiles.
3. **La liste des pilotes connus**, comme celle de Chrome (`gpu_driver_bug_list.json`) : par
   fabricant, modèle et version, le contournement de chacun ; remplie par la boîte noire et par
   l'auto-vérification, lue au démarrage.
4. **L'étalonnage** : Unreal Engine, réglé sur « auto », fait tourner un banc du matériel et
   règle tout d'après (`RunHardwareBenchmark`). Glucose mesurera sur place ce que coûtent ses
   gestes, pour que son modèle de coût **prédise avec les chiffres de la machine**. La boîte noire
   dira d'abord, sur les vraies machines, où le modèle se trompe : l'étalonnage vient après elle.

---

## 2. La vie privée : les règles

Ce ne sont pas des options : c'est le cahier des charges. Elles tiennent **avec** une boîte noire
complète.

1. **Rien ne part sans accord.** Éteint par défaut ; un écran clair, en français simple, qui dit
   que c'est une bêta de recherche, quoi est recueilli, pourquoi, pour combien de temps ; Glucose
   marche **exactement pareil** sans ; on peut retirer son accord à tout moment.
2. **Voir ce qui part** : le testeur peut lire sa boîte noire et le contenu exact d'un envoi.
3. **Jamais** : le contenu d'un document, un nom de fichier, un chemin, le nom d'utilisateur. Le
   serveur ne garde pas l'adresse IP.
4. **Un rapport de plantage se demande** — un plantage peut contenir des fragments de mémoire :
   la première fois, avec le choix « toujours envoyer ».
5. **Chiffré, et à une adresse qui dure** : HTTPS, sous un nom de domaine. Une conservation
   bornée ; l'effacement sur demande (un identifiant tiré au hasard, que la personne peut
   renouveler) ; une page publique dans le dépôt qui dit exactement ce qui est recueilli — et le
   code étant public, chacun peut le vérifier.

**Ce que je dois dire franchement.**

* Avec une vingtaine de testeurs qu'il connaît, **l'anonymat n'existe pas** : il reconnaîtrait
  chaque machine. C'est pourquoi l'accord et la transparence comptent plus que l'anonymat.
* **La télémétrie de Tauri envoie en clair** (HTTP, sans chiffrement), **vers l'adresse IP brute
  de sa box**, écrite dans chaque application installée et dans le dépôt public. Si l'adresse
  change, ces applications parlent dans le vide ; et elle expose sa box. Glucose Rust ne reprendra
  ni l'un ni l'autre : un nom de domaine et du HTTPS — par exemple un tunnel Cloudflare, qui évite
  d'ouvrir un port chez lui ; le seul coût est le domaine, une dizaine d'euros par an. La bascule
  mettra fin à l'ancienne.
* La CNIL le demande : l'accord préalable avant de lire ou d'écrire sur l'appareil, sauf ce qui est
  strictement nécessaire au service demandé ; son guide pour les développeurs en fait la méthode.

---

## 3. L'ordre, et pourquoi

Poids en sessions comme celles de septembre — des ordres de grandeur, pas des promesses.

| # | Phase | Sessions | Ce qu'elle donne | Sa part |
|---|---|:--:|---|---|
| 0 | **La clé** de signature de Tauri et son mot de passe, sauvegardés **hors de ce PC** (fiche 36 § 3.4) | — | perdue, plus aucun utilisateur de Tauri ne peut être mis à jour, jamais | **tout** : je ne lis jamais cette clé |
| 1 | **Voir sans posséder** : l'intégration continue rallumée — Windows, Linux, Mac (puce Apple et Intel) ; le noyau compilé pour Android | 1-2 | à chaque envoi, compiler et éprouver partout | — |
| 2 | **La boîte noire** (§ 1.1) : le journal, les sondes de chaleur et de batterie, la fin de la session d'avant, le témoin des plantages ; l'écran d'accord, « voir ce qui part », la page de confidentialité ; le serveur, chiffré | 2-3 | chaque session de chaque testeur ; ce qui a mal fini, et pourquoi | le nom de domaine, le HTTPS devant sa box |
| 3 | **Linux et NixOS** : ce qui reste des services de `plateforme/`, et le paquet Nix | 1-3 | Glucose sur les systèmes de ses cinq utilisateurs | — |
| 4 | **Distribuer et se mettre à jour** : l'installeur Windows, l'AppImage, le paquet Nix ; la mise à jour de Rust au **format de Tauri avec la même clé** ; l'épreuve N → N+1 ; la répétition générale de la bascule (§ 4) | 2-3 | la chaîne des versions prouvée avant qu'un seul utilisateur la reçoive | signer (sa clé) |
| 5 | **La 2.0.1 bêta, qui est la bascule** : ses cinq utilisateurs d'un coup, par le popup habituel ; publiée pour les testeurs Windows et Linux qui attendent | 1 | Glucose Rust publié | signer, prévenir |
| 6 | **Android** : le toucher, le clavier virtuel, la vie d'une application qu'on suspend, le stockage, l'APK et sa mise à jour ; la boîte noire sur téléphone ; l'auto-vérification des pilotes ; de vrais téléphones anciens par Firebase | 5-10 | Glucose chez ses **dix** testeurs Android | un compte développeur Google, sa clé Android sauvegardée |
| 7 | **L'étalonnage**, appris de ce que la boîte noire aura vu sur les vraies machines | 2-3 | l'adaptativité qui prédit juste, même sur un téléphone qui chauffe | — |
| 8 | **Mac**, quand quelqu'un l'attend : il compile dès la phase 1 | 1-3 | — | la signature Apple, plus tard |

**Pourquoi cet ordre.**

* **Voir d'abord** : la phase 1 coûte peu, ne coûte rien, et rend chaque phase suivante
  vérifiable sur tous les systèmes.
* **La boîte noire avant la première publication** : chaque testeur doit arriver dans une version
  qui sait déjà dire ce qui lui arrive — c'est son argument, et il est juste.
* **Le bureau avant Android** : Windows tourne, Linux compilait déjà le 15/09 ; la bascule de ses
  cinq utilisateurs et les testeurs de bureau sont à quelques sessions. Android est un vrai
  portage — le toucher, le clavier, la vie d'une application qu'on suspend — : le plus long
  chantier ne doit retarder personne, et il arrivera avec une boîte noire et une chaîne de mises à
  jour déjà prouvées.
* **L'étalonnage après la boîte noire** : on n'étalonne bien que ce qu'on a vu se tromper.

---

## 4. La mise à jour : ce qui garantit qu'on ne perd jamais une version

1. **Le format et la clé de Tauri.** La mise à jour de Rust lit le même genre de fichier signé
   (minisign), vérifié par la même clé publique : la bascule est sans couture, et il n'y a qu'une
   clé à garder. Une petite bibliothèque des auteurs de Tauri fait exactement ce travail pour une
   application Rust (`cargo-packager-updater`, qui connaît les installeurs NSIS).
2. **Chercher la mise à jour avant tout ce qui peut planter** — la carte graphique, le document.
   Une version qui plante au démarrage sur une machine peut ainsi recevoir sa correction : c'est la
   façon la plus courante de perdre un utilisateur pour toujours.
3. **Chaque publication est éprouvée** sur les machines de GitHub, jamais sur son écran :
   installer la version N, publier N+1 dans un canal d'essai, vérifier que N devient N+1 seule.
   **Rien ne sort sans cette épreuve verte**, sur chaque système.
4. **La répétition générale de la bascule** : sur ces mêmes machines, une application Tauri
   construite avec une **clé d'essai** passe à Glucose Rust par un fichier d'essai. La vraie
   bascule ne fait que rejouer ce qui a déjà réussi.
5. **On ne supprime jamais une publication**, et les numéros ne font que monter : Tauri ne passe à
   une version que si elle est **strictement plus grande** (2.0.1 après 1.0.x). Une mauvaise
   version se corrige par la suivante. Les installeurs de Tauri restent téléchargeables : un
   chemin de retour, que ses utilisateurs peuvent prendre avec lui.
6. **Ce que la bascule garde** : Glucose Rust lit déjà les documents de Tauri, sans jamais les
   réécrire — les originaux restent intacts.

**Par système** :

* **Windows** : l'installeur NSIS, par utilisateur, comme Tauri.
* **Linux** : l'AppImage, que la mise à jour de Tauri sait remplacer.
* **NixOS** : la mise à jour passe par Nix — c'est ainsi que NixOS fonctionne, tout y est
  immuable. Glucose dit qu'une version existe ; Nix l'installe.
* **Android** : un APK signé d'une clé **qui ne doit jamais se perdre non plus** — Android
  n'installe une mise à jour que signée de la même clé ; perdue, chaque testeur devrait
  désinstaller, et perdre ce que l'application gardait. Google exige désormais que les
  applications installées hors du Play Store viennent d'un **développeur vérifié** : dans quatre
  pays dès le 30/09/2026, **partout en 2027**. Pour les amateurs, un compte **gratuit, sans pièce
  d'identité, limité à vingt appareils** — son échelle ; au-delà, un compte vérifié (25 $, une
  fois).
* **Mac** (phase 8) : Apple demande une signature « Developer ID » et une **notarisation** pour
  qu'une application téléchargée s'ouvre sans avertissement (99 $ par an) ; sans elle, depuis
  macOS Sequoia, il faut l'autoriser dans Réglages → Confidentialité et sécurité → « Ouvrir quand
  même ».

**Le plancher des très anciens téléphones** : Rust sur Android suit ce que le kit de Google
(le NDK) prend en charge — Android 5.0 et après, aujourd'hui. La carte graphique doit parler
OpenGL ES 3 ou Vulkan ; en deçà, reste la voie du processeur, ce que la charte veut (aucune
exclusion matérielle) — **à vérifier** en phase 6, sur les appareils de ses testeurs.

---

## 5. Ce qui attend sa parole

1. **Les appareils de ses dix testeurs Android** — la marque et le modèle, ou au moins la version
   d'Android du plus ancien : c'est ce qui fixe le plancher, et les téléphones à éprouver.
2. **La clé de Tauri** : est-elle déjà sauvegardée hors de ce PC ? Sinon, c'est son premier geste.
3. **Le serveur** : sa box derrière un nom de domaine et du HTTPS, ou une petite machine louée en
   Europe (quelques euros par mois).
4. **Plus tard**, les signatures payantes (Apple, Windows) : inutiles pour vingt personnes
   prévenues, nécessaires pour publier au grand jour.

---

## 6. Les sources

* GitHub, [*GitHub-hosted runners*](https://docs.github.com/en/actions/reference/runners/github-hosted-runners)
  — *« Use of the standard GitHub-hosted runners is free and unlimited on public
  repositories »* ; Windows x64 et ARM, Linux x64 et ARM, macOS Intel (`macos-15-intel`) et puce
  Apple (`macos-14`, `macos-15`).
* wgpu, [son intégration continue](https://github.com/gfx-rs/wgpu/blob/trunk/.github/workflows/ci.yml)
  — ses épreuves graphiques sur Windows (WARP, Mesa), Mac à puce Apple (Metal) et Linux (Mesa).
* Firebase, [*Test Lab — quotas*](https://firebase.google.com/docs/test-lab/usage-quotas-pricing)
  — cinq essais par jour sur de vrais appareils, dix sur des appareils virtuels, sans frais.
* Android, [*Thermal API*](https://developer.android.com/games/optimize/adpf/thermal) — l'état
  thermique et la marge avant bridage ; [`ApplicationExitInfo`](https://developer.android.com/reference/android/app/ApplicationExitInfo)
  — la raison de la fin d'un processus ; [*Android developer verification*](https://developer.android.com/developer-verification)
  et [l'annonce de mars 2026](https://android-developers.googleblog.com/2026/03/android-developer-verification.html)
  — les développeurs vérifiés, le compte des amateurs limité à vingt appareils.
* Rust, [*Android platform support*](https://doc.rust-lang.org/nightly/rustc/platform-support/android.html)
  — les niveaux d'API que suit le NDK.
* Embark Studios, [`crash-handling`](https://github.com/EmbarkStudios/crash-handling) — le
  rapport de plantage écrit par un processus témoin, et ses plateformes ; Mozilla,
  [`rust-minidump`](https://github.com/rust-minidump/rust-minidump) — son analyse.
* Epic Games, [`RunHardwareBenchmark`](https://dev.epicgames.com/documentation/en-us/unreal-engine/API/Runtime/Engine/GameFramework/UGameUserSettings/RunHardwareBenchmark)
  et T. Looman, [*Determine Optimal Scalability Settings for Players Hardware*](https://tomlooman.com/unreal-engine-optimal-graphics-settings/)
  — l'étalonnage.
* Chromium, [`gpu_driver_bug_list.json`](https://github.com/chromium/chromium/blob/main/gpu/config/gpu_driver_bug_list.json)
  — les pilotes défectueux, par fabricant, modèle et version, et leur contournement.
* Tauri, [*Updater*](https://v2.tauri.app/plugin/updater/) — la version doit être plus grande que
  la version installée ; CrabNebula, [`cargo-packager-updater`](https://docs.rs/cargo-packager-updater)
  — la même signature minisign pour une application Rust.
* Apple, [*Safely open apps on your Mac*](https://support.apple.com/en-us/102445) et
  [*Updates to runtime protection in macOS Sequoia*](https://developer.apple.com/news/?id=saqachfa).
* CNIL, [*Guide RGPD de l'équipe de développement*](https://www.cnil.fr/fr/guide-rgpd-du-developpeur)
  et [*Recommandation relative aux applications mobiles*](https://www.cnil.fr/sites/cnil/files/2024-09/recommandation-applications-mobiles.pdf)
  (2024) — l'accord préalable, la minimisation.

---

**Retour** : [`00-INDEX.md`](00-INDEX.md)
