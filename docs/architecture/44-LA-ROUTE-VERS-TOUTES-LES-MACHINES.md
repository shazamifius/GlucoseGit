# 44 — La route vers toutes les machines

> **Rôle de ce document.** Le 29/09, il reprend la question de la fiche [`36`](36-LA-ROUTE-VERS-LA-V1.md) :
> *« qu'est-ce qui pourrait faire qu'on puisse enfin publier le projet et avancer dessus ? »* Il
> propose des journaux « en béton » sur tous les appareils, une télémétrie avec un bouton
> « accepter », la bascule de Glucose Tauri vers Glucose Rust par la mise à jour automatique —
> *« sans JAMAIS perdre une version, et TOUJOURS avec notre mise à jour »* —, puis la **2.0.1
> bêta**. Puis il précise trois choses, et demande s'il existe mieux que les journaux.
> **Aucun code dans cette fiche** : c'est la route des sessions à venir. Elle remplace les
> phases 3, 8 et 9 de la fiche 36, et tranche sa première question.
>
> **Date** : 2026-09-29 · au commit `47cf1e7`.

---

## 0. En une page

**Ses trois précisions**, qui décident de tout :

1. **Cinq utilisateurs** : on les fait passer **tous d'un coup**. La question « un moment ou
   deux » de la fiche 36 est tranchée : un seul.
2. **Toutes les machines**, pas seulement Windows : Windows, Mac à puce Apple et Mac Intel, Linux
   — **NixOS d'abord** —, Android, *« tout type d'OS »*.
3. **La vie privée** : *« ça m'importe énormément, je ne souhaite manquer de respect à
   personne »*.

**Sa question** — pourquoi des journaux ? *« Pour l'adaptativité du programme face au matériel
de l'utilisateur ; on n'a jamais testé le logiciel sur un autre matériel, parce que je n'en
possède pas ; et pour les erreurs de compatibilité. Si ça se trouve, il existe mieux. »*

**Oui, il existe mieux**, pour chacun de ses deux buts (§ 1) :

* **l'adaptativité** ne passe pas par des journaux envoyés : le programme **se mesure lui-même**,
  sur place, et décide seul — rien ne quitte la machine ;
* **la compatibilité** se voit **sans posséder le matériel** : des épreuves automatiques sur tous
  les systèmes, gratuites ; le programme qui **se vérifie** sur la machine ; des **rapports de
  plantage** exacts ; une **liste des pilotes** connus pour mal faire.

Les journaux restent, en dernier recours, et sur la machine. La vie privée en sort gagnante :
ce qui s'adapte sur place n'a rien à envoyer (§ 2).

**L'ordre** (§ 3) : voir sans posséder → porter → se mesurer et se vérifier → distribuer et se
mettre à jour → la télémétrie → **la 2.0.1 bêta, qui est la bascule** → Android.

---

## 1. Mieux que les journaux

### 1.1 L'adaptativité : que le programme se mesure lui-même

Un journal envoyé raconte **après coup** ce qui s'est passé chez quelqu'un ; il ne fait rien
pour lui **maintenant**. L'adaptativité que veut la charte — un modèle de coût qui **prédit**,
la cadence adaptée à la machine, la mémoire par étages — doit se décider **sur la machine**, à
l'instant, sans réseau.

C'est ce que font les moteurs de jeu. Unreal Engine, réglé sur « auto », lance au premier
démarrage un **banc du matériel** (`RunHardwareBenchmark`) : quelques instants de travail
synthétique, qui donnent un indice du processeur et un indice de la carte graphique, et les
réglages en découlent. Glucose a déjà des morceaux de cette idée — l'arbitre qui choisit la carte
graphique, la cadence, les étages de mémoire ; il lui manque **l'étalonnage** : au premier
lancement, hors écran, une fraction de seconde pour mesurer ce que coûtent, **sur cette
machine**, les gestes que le modèle de coût prédit (dessiner, envoyer une image à la carte
graphique, décoder). Le modèle prédit ensuite avec les chiffres de la machine, et non avec ceux
de la mienne.

La télémétrie ne sert alors qu'à **nous** : comparer les étalonnages de machines différentes
pour améliorer le modèle. Elle devient petite par construction (§ 2).

### 1.2 La compatibilité : voir sans posséder

Par ordre de valeur :

1. **Des épreuves automatiques sur tous les systèmes, à chaque envoi.** GitHub prête
   gratuitement, sans limite pour un dépôt public, des machines Windows (x64 et ARM), Linux (x64
   et ARM), Mac Intel et Mac à puce Apple. Elles n'ont pas de carte graphique, mais des **rendus
   logiciels** existent — WARP sous Windows, Mesa sous Linux, Metal sur Mac — : c'est ainsi que
   **wgpu**, la bibliothèque graphique de Glucose, éprouve ses propres rendus. **Glucose Rust n'a
   jamais été compilé ailleurs que sous Windows** : des replis existent dans `plateforme/`, mais
   seule une compilation dira ce qui casse. NixOS s'y ajoute par sa propre construction (Nix, dans
   la même intégration continue). Pour Android : des émulateurs, et de **vrais téléphones** par
   Firebase Test Lab — cinq essais par jour sur de vrais appareils, dix sur des appareils
   virtuels, gratuitement.
2. **Le programme se vérifie sur la machine.** Au premier lancement, hors écran, il dessine une
   scène connue par la carte graphique et par le processeur, et compare les deux au pixel près.
   Un pilote qui dessine faux est su **avant** de montrer quoi que ce soit, et le programme
   choisit l'autre voie pour ce qui cloche — un **choix délibéré**, pas une bascule par échec, ce
   que la charte interdit.
3. **Des rapports de plantage exacts.** Quand un programme plante — même dans le pilote de la
   carte graphique —, un petit processus **témoin**, à côté, écrit l'état exact : où, dans quel
   pilote, quelle version. C'est bien plus précis qu'un journal, qui s'arrête avec le programme.
   Les outils existent en Rust et servent en production : `crash-handler` et `minidumper`
   (Embark), et l'analyse de Mozilla (`rust-minidump`), celle des plantages de Firefox.
4. **La liste des pilotes connus.** Chrome tient une liste de pilotes défectueux, par fabricant,
   modèle et version, avec le contournement de chacun (`gpu_driver_bug_list.json`). Glucose
   tiendra la sienne, remplie par les points 2 et 3, et la lira au démarrage.
5. **Le journal**, sur la machine : structuré, tournant (il ne remplit jamais le disque), qui ne
   ralentit jamais l'image, avec le profil complet de la machine. Pour ce que rien d'autre
   n'explique — et envoyé **seulement si la personne le décide**.

### 1.3 Ce qu'on ne peut pas voir ainsi

La **vraie** carte graphique de quelqu'un : ses pilotes, sa chaleur, sa batterie. Les machines
prêtées n'en ont pas ; l'étalonnage et l'auto-vérification la mesurent chez lui, et la
télémétrie, s'il l'accepte, nous le dit. Une machine Mac louée à l'heure reste possible pour un
essai précis.

---

## 2. La vie privée : les règles

Ce ne sont pas des options : c'est le cahier des charges.

1. **Rien ne part sans accord.** Éteint par défaut ; un écran clair, en français simple, qui dit
   quoi, pourquoi, pour combien de temps ; Glucose marche **exactement pareil** sans ; on peut
   retirer son accord à tout moment.
2. **Voir ce qui part.** La personne peut lire le contenu exact d'un envoi avant qu'il parte.
3. **Jamais** : le contenu d'un document, un nom de fichier, un chemin, le nom d'utilisateur. Le
   serveur ne garde pas l'adresse IP.
4. **Peu, et agrégé.** L'adaptation se faisant sur place (§ 1.1), la télémétrie se réduit à la
   classe de la machine et à des mesures résumées — ce que calcule déjà la chronique.
5. **Un plantage se demande à chaque fois** : *« Glucose s'est fermé. Envoyer le rapport ?
   Voir le contenu. »* — un rapport de plantage peut contenir des fragments de mémoire.
6. **Chiffré, et à une adresse qui dure** : HTTPS, sous un nom de domaine. Une durée de
   conservation bornée ; l'effacement sur demande (un identifiant tiré au hasard, que la
   personne peut renouveler) ; une page publique dans le dépôt qui dit exactement ce qui est
   recueilli — et le code étant public, chacun peut le vérifier.

**Ce que je dois dire franchement.**

* Avec cinq utilisateurs qu'il connaît, **l'anonymat n'existe pas** : il reconnaîtrait chaque
  machine. C'est pourquoi l'accord et la transparence comptent plus que l'anonymat.
* **La télémétrie de Tauri envoie en clair** (HTTP, sans chiffrement), **vers l'adresse IP brute
  de sa box**, écrite dans chaque application installée et dans le dépôt public. Si l'adresse
  change, ces applications parlent dans le vide ; et elle expose sa box. Glucose Rust ne reprendra
  ni l'un ni l'autre : un nom de domaine, du HTTPS — par exemple un tunnel Cloudflare, qui évite
  d'ouvrir un port chez lui ; le seul coût est le domaine, une dizaine d'euros par an. La bascule
  mettra fin à l'ancienne.
* La CNIL le demande : l'accord préalable avant de lire ou d'écrire sur l'appareil, sauf ce qui est
  strictement nécessaire au service demandé ; son guide pour les développeurs en fait la méthode.

---

## 3. L'ordre, et pourquoi

Poids en sessions comme celles de septembre — des ordres de grandeur, pas des promesses.

| # | Phase | Sessions | Ce qu'elle donne | Sa part |
|---|---|:--:|---|---|
| 0 | **La clé** de signature et son mot de passe, sauvegardés **hors de ce PC** (fiche 36 § 3.4) | — | perdue, plus aucun utilisateur de Tauri ne peut être mis à jour, jamais | **tout** : je ne lis jamais cette clé |
| 1 | **Voir sans posséder** : l'intégration continue sur Windows, Mac (puce Apple et Intel), Linux, et la construction Nix | 1-2 | à chaque envoi, compiler et éprouver partout ; savoir enfin ce qui casse hors de Windows | — |
| 2 | **Porter** Mac, Linux, NixOS : les services de `plateforme/` (le dépôt, le téléchargement, l'offre de mémoire, le budget de la carte graphique, l'heure, la priorité, le verrou, le pincement) | 2-5 | Glucose tourne sur les systèmes de bureau ; l'incertitude tombe à la phase 1 | essayer sur une machine de ses utilisateurs, s'il le peut |
| 3 | **Se mesurer et se vérifier** : l'étalonnage au premier lancement, l'auto-vérification au pixel, la liste des pilotes, le journal local, les rapports de plantage par le témoin | 2-3 | l'adaptativité sur place ; rien ne quitte la machine | — |
| 4 | **Distribuer et se mettre à jour** : les installeurs (Windows, Mac, AppImage, et le paquet Nix), la mise à jour de Rust au **format de Tauri avec la même clé**, l'épreuve N → N+1 sur chaque système | 3-4 | la chaîne des versions prouvée avant qu'un seul utilisateur la reçoive | signer (sa clé) |
| 5 | **La télémétrie respectueuse** : l'écran d'accord, « voir ce qui part », les mesures résumées, les plantages demandés un à un, la page de confidentialité | 1-2 | ce que ses cinq machines disent, avec leur accord | le nom de domaine, le HTTPS devant sa box |
| 6 | **La 2.0.1 bêta, qui est la bascule** : les cinq, d'un coup, par le popup habituel | 1 | Glucose Rust chez tous ses utilisateurs | signer, prévenir ses cinq utilisateurs |
| 7 | **Android** : le toucher, le clavier virtuel, la vie d'une application qu'on suspend, le stockage, l'installation | 5-10 | Glucose sur téléphone et tablette ; éprouvé sur de vrais téléphones | — |

**Pourquoi cet ordre.**

* **Voir avant de porter** : la phase 1 transforme « on n'a jamais testé ailleurs » en une liste
  exacte de ce qui casse. Elle coûte peu et rend chaque phase suivante vérifiable.
* **Porter avant d'étalonner** : l'étalonnage et l'auto-vérification doivent tourner sur chaque
  système, donc après qu'il y tourne.
* **La mise à jour avant quiconque** : la seule erreur qu'on ne rattrape pas, c'est une version
  livrée qui ne sait plus se mettre à jour.
* **La télémétrie après l'adaptation locale** : ce qui se décide sur place n'a pas à voyager ;
  elle en sort petite.
* **Android après la bascule** : Glucose Tauri n'a jamais eu de version Android — **aucun** de
  ses utilisateurs n'y est, et la bascule n'a pas à l'attendre. C'est le chantier le plus long :
  il ne doit retarder personne.

---

## 4. La mise à jour : ce qui garantit qu'on ne perd jamais une version

1. **Le format et la clé de Tauri.** La mise à jour de Rust lit le même genre de fichier signé
   (minisign), vérifié par la même clé publique : la bascule est sans couture, et il n'y a qu'une
   clé à garder. Une petite bibliothèque des auteurs de Tauri fait exactement ce travail pour une
   application Rust (`cargo-packager-updater`, qui connaît aussi les installeurs NSIS).
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
   chemin de retour, que ses cinq utilisateurs peuvent prendre avec lui.
6. **Ce que la bascule garde** : Glucose Rust lit déjà les documents de Tauri, sans jamais les
   réécrire — les originaux restent intacts.

**Par système** :

* **Windows** : l'installeur NSIS, par utilisateur, comme Tauri.
* **Mac** : Apple demande une signature « Developer ID » et une **notarisation** pour qu'une
  application téléchargée s'ouvre sans avertissement (le programme des développeurs coûte 99 $
  par an). Sans elle, depuis macOS Sequoia, il faut l'autoriser dans Réglages → Confidentialité
  et sécurité → « Ouvrir quand même ». Pour cinq personnes prévenues, c'est tenable.
* **NixOS** : la mise à jour passe par Nix — c'est ainsi que NixOS fonctionne, tout y est
  immuable. Glucose dit qu'une version existe ; Nix l'installe.
* **Linux ailleurs** : l'AppImage, que la mise à jour de Tauri sait remplacer.
* **Android** (phase 7) : l'installation directe d'un APK, avec l'accord de la personne, puis un
  magasin d'applications, plus tard.

---

## 5. Ce qui attend sa parole

1. **Les systèmes de ses cinq utilisateurs** : ce qui décide quand la bascule peut avoir lieu. Sa
   télémétrie de Tauri le sait peut-être ; eux le savent sûrement.
2. **La clé** : est-elle déjà sauvegardée hors de ce PC ? Sinon, c'est son premier geste.
3. **Le serveur** : sa box derrière un nom de domaine et du HTTPS, ou une petite machine louée
   en Europe (quelques euros par mois).
4. **Plus tard**, les signatures payantes (Apple, Windows) : inutiles pour cinq personnes
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
* Epic Games, [`RunHardwareBenchmark`](https://dev.epicgames.com/documentation/en-us/unreal-engine/API/Runtime/Engine/GameFramework/UGameUserSettings/RunHardwareBenchmark)
  et T. Looman, [*Determine Optimal Scalability Settings for Players Hardware*](https://tomlooman.com/unreal-engine-optimal-graphics-settings/)
  — l'étalonnage au premier lancement.
* Chromium, [`gpu_driver_bug_list.json`](https://github.com/chromium/chromium/blob/main/gpu/config/gpu_driver_bug_list.json)
  — les pilotes défectueux, par fabricant, modèle et version, et leur contournement.
* Embark Studios, [`crash-handling`](https://github.com/EmbarkStudios/crash-handling) — le
  rapport de plantage écrit par un processus témoin ; Mozilla,
  [`rust-minidump`](https://github.com/rust-minidump/rust-minidump) — son analyse.
* Tauri, [*Updater*](https://v2.tauri.app/plugin/updater/) — la version doit être plus grande que
  la version installée ; les modes d'installation ; CrabNebula,
  [`cargo-packager-updater`](https://docs.rs/cargo-packager-updater) — la même signature minisign
  pour une application Rust.
* Apple, [*Safely open apps on your Mac*](https://support.apple.com/en-us/102445) et
  [*Updates to runtime protection in macOS Sequoia*](https://developer.apple.com/news/?id=saqachfa)
  — la notarisation, et « Ouvrir quand même ».
* CNIL, [*Guide RGPD de l'équipe de développement*](https://www.cnil.fr/fr/guide-rgpd-du-developpeur)
  et [*Recommandation relative aux applications mobiles*](https://www.cnil.fr/sites/cnil/files/2024-09/recommandation-applications-mobiles.pdf)
  (2024) — l'accord préalable, la minimisation.

---

**Retour** : [`00-INDEX.md`](00-INDEX.md)
