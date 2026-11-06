# 84 — L'échelle des moteurs : la machine que Glucose excluait

> Session du 6/11/2026, la première sur son **VivoBook sous NixOS** : le PC Windows de toutes les
> fiches précédentes est cassé. Sa demande : savoir où reprendre, regarder la télémétrie, et que
> *« les autres sessions Claude sans aucun contexte arrivent tout de même à travailler »*.

## 1. Ce que la télémétrie a montré

Douze jours sans relevé : 132 sessions, 7 installations, 29 fins anormales. **L'installation
`01ca394b`, Windows, beta 6, le 29/10 : quatre lancements en dix-huit secondes, aucune ligne
`carte`, aucune image, et trois fois `igvk64.dll` version 31.0.101.2140, code `0xC0000005`, au
même décalage (6 491 761).** Le pilote Vulkan d'Intel tue le processus à l'ouverture, sans qu'aucun
`Result` ne remonte : le repli vers le moteur suivant ne s'exécutait jamais. La personne a abandonné.
La charte dit *aucune machine exclue* ; celle-ci l'était, et personne ne le savait.

La recherche : des plantages de `igvk64.dll` à `vkCreateInstance` existent chez d'autres, et Intel
répond « réinstallez le pilote » ([forum d'Intel](https://community.intel.com/t5/Developing-Games-on-Intel/crash-CreateDevice-on-both-vulkan-and-DirectX12-UHD770/m-p/1579957)).
On ne peut pas demander ça à un utilisateur. `wgpu` n'offre aucun repli sur plantage. L'idée de
mémoriser le chemin qui a planté existe ailleurs (Warp, un terminal, a des chemins de reprise qui
changent de moteur après un plantage au démarrage).

## 2. MOTEUR-1 — observer, sans table de pilotes

`present/echelle.rs`. Avant d'ouvrir un moteur, Glucose **écrit son nom** dans `moteurs.txt` ; à la
première image présentée (ou à une fermeture normale), il l'efface. **Un nom encore écrit au
lancement suivant dit que ce moteur n'a pas survécu à son ouverture** — quel que soit le pilote, le
système, l'étape. Aucune liste de pilotes, aucun seuil, aucune durée.

* **Un moteur à la fois** (`moteurs::MOTEURS` : Vulkan, Metal, Direct3D 12, OpenGL) : ouvrir Vulkan
  et Direct3D 12 ensemble chargeait leurs deux pilotes, et on n'aurait pas su lequel écarter.
* **Une carte logicielle n'est prise qu'à défaut** : ouvrir ensemble laissait `wgpu` préférer une
  vraie carte ; sans cela, un Vulkan logiciel masquerait un Direct3D 12 matériel.
* **Le dernier barreau est le processeur**, qui n'a pas de pilote : l'échelle se termine toujours.
* **Une nouvelle version repose la question sans risque** : `--sonder-moteur <nom>`, un processus
  à part, en fond. S'il meurt, personne ne le voit (`SEM_NOGPFAULTERRORBOX` sous Windows) ; s'il
  vit, le moteur revient au lancement suivant. Sans cela, l'utilisateur replanterait à chaque
  mise à jour. Android n'a pas d'exécutable à relancer : il réessaie directement.
* **Pas de ligne de plus dans le journal** : le serveur n'accepte qu'une liste fixe de sortes, et
  `plantage` puis `carte` (le moteur qui a pris le relais) racontent déjà l'histoire.
* **Le cas de la personne bloquée** : après une mauvaise fin, la mise à jour se cherche **avant**
  d'ouvrir la carte (fiche 48). La beta 7 peut donc l'atteindre si elle relance Glucose.

**Ce que ça coûte** : une chute visible, une fois, sur la machine touchée. Une coupure de courant
pile pendant l'ouverture (quelques centaines de millisecondes) écarte un moteur à tort jusqu'à la
version suivante : accepté, c'est rare, et Glucose continue de marcher.

## 3. Trouvé en le jouant

Le vrai programme, de vrais `abort` (`GLUCOSE_TOMBER=vulkan`), une fenêtre sur son écran, chaque
image regardée :

* **OpenGL était mort sous Wayland** : `gl not compatible with provided surface`. L'instance
  naissait sans l'affichage de la fenêtre ; EGL ne s'ouvrait donc que hors écran. Sur toute machine
  Wayland sans Vulkan, Glucose tombait au processeur en ayant une carte. Corrigé :
  `moteurs::instance_pour_la_fenetre`. **Vu dessiner juste** sur Mesa, Intel UHD 620.
* **Le message passait inaperçu** : un toast de 2,4 s pour une phrase de cent lettres, parti avant
  la fenêtre. `Toast::a_lire` : le temps d'un toast plus celui de lire ses lettres.
* **ACCUEIL-2** : au premier lancement, le nœud de bienvenue naissait sous la barre d'outils, son
  titre caché. Le retrait suit la hauteur réelle de la bande. Un défaut vieux de toujours, que
  personne ne pouvait voir qu'en ouvrant Glucose pour la première fois.

**L'échelle entière, en réel, sous Linux** : Vulkan tombe → OpenGL dessine ; OpenGL tombe → le
processeur dessine ; une version neuve sonde les deux ; Vulkan revient. Aucun brouillon créé.

## 4. Ce qui n'est PAS prouvé

* **Direct3D 12 sur la machine touchée** : personne n'a ce pilote ici. La CI exige désormais le
  rendu par Direct3D 12 (WARP) sous Windows, ce qui prouve les nuanceurs, pas son pilote Intel.
* **Que ce plantage soit bien à l'ouverture de Vulkan** : le journal dit `igvk64.dll` et aucune
  ligne `carte`. C'est la lecture la plus probable, pas une certitude ; mais l'échelle ne dépend pas
  de l'étape exacte.
* **On ne sait pas qui est cette personne.** La prochaine ligne `plantage` ou `carte` de
  `01ca394b` dira si elle est revenue.

## 5. NOTES-1 — des notes qui ne mentent plus

La beta 6 est partie avec les notes de la beta 5 : `notes.md` s'écrit à la main, et rien ne le
vérifiait. `outils/publication/verifier_notes.py` arrête la publication **avant de construire** si
elles ne disent pas « depuis la <dernière version publiée> ».

## 6. Le changement de machine

Le code n'a rien perdu : `c6443cf` était sur GitHub. **Le reste, relevé :**

* **La clé Android** : dans `sauvegarde-documents-2026-10-31.7z`, sur le disque de stockage, testée
  sans erreur (733 fichiers vérifiés). Elle sert aussi de secret GitHub.
* **La clé de mise à jour** (`glucose_updater.key`, sur l'ancien PC) : **son seul exemplaire connu
  ici est le secret GitHub, qui ne se relit pas.** La session du 31/10 l'a mise dans une seconde
  archive, `sauvegarde-profil-2026-10-31.7z` (317 Mo, avec les clés de serveur et l'identité web3),
  qu'elle disait d'envoyer sur un cloud ; elle n'est ni sur ce PC, ni dans les RAR, ni sur le Google
  Drive connecté. Les publications continuent ; le jour où le secret disparaîtrait, plus aucune mise
  à jour n'atteindrait personne.
* **`%LOCALAPPDATA%\Glucose`** (brouillons, boîte noire) : dans cette même seconde archive d'après
  la session du 31/10, et nulle part ici.
* **Les outils** : Rust passe par `nix develop` ; ni SDK Android ni téléphone virtuel ; Cloudflare
  se reconnecte à la demande. Le mode d'emploi : METHODE § 5, en tête.

## 7. Les leçons

* **Un repli que le plantage court-circuite n'est pas un repli** : `Result` ne protège que de ce
  qui veut bien échouer poliment. Ce qui doit survivre à une mort s'écrit avant le risque.
* **Un barreau jamais emprunté est mort** (OpenGL sous Wayland) : seule la chute réelle l'a montré.
* **Lancer un Glucose « isolé » n'isole pas de la télémétrie** : sa question s'affiche, et un clic
  distrait de l'utilisateur a envoyé une session d'essai, effacée ensuite du serveur. D'où
  `outils/essai_isole.sh`, qui écrit « non » d'avance.
* **Une fenêtre d'essai prend le focus** et peut voler ce qu'il tape : brève, et annoncée.

## 8. COURSE-1 — le plantage des épreuves

Après MOTEUR-1, `cargo test --workspace` a fait tomber deux processus d'épreuve, sans dire
laquelle : `SIGSEGV`. **Sous `gdb`, 4 essais sur 4, la même pile** : `vkEnumerateInstanceExtensionProperties`
→ `loader_icd_scan` → `loader_scanned_icd_add` → l'adresse 0, dans `libvulkan` 1.4.357. Plusieurs
épreuves ouvrent leur moteur en parallèle, et le chargeur balaie ses pilotes (quatorze ici) en les
chargeant et les déchargeant. **Rien à voir avec l'échelle** : le chemin est le même avant elle.

* **La réparation** : un seul à la fois devant le chargeur, et une instance gardée ouverte pour que
  les pilotes restent chargés (`banc_gpu::instance_du_banc`). Le programme n'ouvre qu'un moteur à la
  fois ; seules les épreuves étaient exposées.
* **La preuve** : 8 passages sur 8 et 3 sur 3 sous `gdb` (`present::`), puis les 2 701 épreuves.
* **Probablement** le plantage de `vulkan-1.dll` de la fiche 49 § 4, au même octet : à confirmer sous
  Windows. **Pas** « l'épreuve qui tombe au hasard » de la fiche 35 § 4, qui est un écart de pixels,
  contrairement à ce que la fiche 49 supposait.
* **Une première épreuve était aveugle** : vingt fils qui ouvrent la carte, cinq fois, sans rien
  faire tomber une fois le tour retiré. Sabotée, elle est partie : une course ne se provoque pas à
  la demande. Elle est remplacée par une garde du code, qui dit ce qu'elle garde et pas plus.
* **Autre aveugle trouvé par le sabotage** : la durée de lecture du message n'avait pas d'épreuve.

## 9. La vérification

Sur NixOS, le 6/11 : format, clippy à zéro, **2 701 épreuves vertes, 7 ignorées** ; sabotages
`moteur-1.json`, tous tombés après correction des deux aveugles ci-dessus. La CI Windows dira le reste.
