# 61 — Le copier unique sous Linux

> Session du 10/10/2026. Ses réponses aux questions de la fiche 60, puis la voie B qu'il a
> choisie : sous Linux, un seul « Copier » qui pose à la fois l'image, les fichiers et le bloc,
> comme sous Windows depuis la fiche 59 — servi par Glucose lui-même, et **éprouvé contre un vrai
> serveur X**. **§ 9 : ce qui attend sa parole.**

## 1. Ses réponses, mot pour mot ou presque

1. **La beta 3** : « je propose la voie B » — puis, en clair : « d'abord la vraie voie : écrire
   pour Linux et Mac ce que Windows a déjà, un seul Copier qui pose l'image, les fichiers et le
   bloc à la fois ; la beta 3 viendrait après ».
2. **Son Linux** : « il est branché en SSH, que tu peux accéder je crois, mais ce n'est
   absolument pas NixOS-Proteo, ça c'est un autre projet ; mon NixOS est une machine à part
   entière sur le réseau ».
3. **Le téléphone** : « je n'ai pas encore testé sur Redmi 9 ; j'ai juste testé sur Motorola
   g54, c'est lui qui avait les rayures ».
4. L'envoi : pas de réponse encore (§ 9).

## 2. Ce que la lecture a établi avant d'écrire

* **Son NixOS** est connu du routeur sous le nom `nixos.home` (192.168.1.12), mais il était
  **éteint ou en veille** pendant toute la session (« hôte injoignable » ; aucun port ouvert, 22
  compris). Ce PC n'a aucune clé ni configuration SSH vers lui. Rien n'a donc tourné sur son
  Linux.
* **`arboard` 3.6.1** (lu à la source, `platform/linux/x11.rs`) : son serveur garde une liste de
  formes, mais son interface n'en pose qu'**une** à la fois, et il écrit chacune d'un bloc —
  **jamais par `INCR`** ; une image plus grosse que la plus grande requête du serveur ne
  partirait pas.
* **Chromium sous Linux** (lu à la source, `chromium.googlesource.com`) — donc Discord, et tout
  navigateur :
  * l'image se lit en `image/png` (`ClipboardOzone::ReadPng`), les fichiers en `text/uri-list`
    (`ReadFilenames` → `URIListToFileInfos`) ;
  * **la liste n'est coupée que sur `\r\n`** (`file_info.cc`, `kURIListSeparator`) : une liste
    séparée par `\n` seul ne donnerait qu'un chemin faux ;
  * Blink fait une pièce jointe de chaque fichier **et** une de l'image présente
    (`data_object.cc`, `CreateFromClipboard`), sur tous les systèmes : **la règle de Windows vaut
    donc sous Linux** — une image seule en PNG, plusieurs en fichiers, jamais les deux ;
  * Chromium sert `TIMESTAMP`, `TARGETS`, `MULTIPLE`, et passe par `INCR` au-delà de la
    plus grande requête (`selection_owner.cc`) ; il **ne remet rien** au gestionnaire du
    presse-papiers en quittant — Qt et GTK le font (`SAVE_TARGETS`).
* **Wayland** : Glucose copie par XWayland (comme `arboard`, pris sans sa voie Wayland). GNOME
  relaie le presse-papiers entre X11 et Wayland dans les deux sens depuis 2015 ([Mutter
  3.17.2](https://www.phoronix.com/news/GNOME-3.17.2-Mutter-X11)) ; KDE laisse une application
  X11 poser le presse-papiers sans le focus depuis Plasma 6.0 ([KWin MR
  5510](https://invent.kde.org/plasma/kwin/-/merge_requests/5510)), avec une panne en 6.5.0 et
  6.5.1, corrigée en 6.5.2 ([le fil de CachyOS](https://discuss.cachyos.org/t/clipboard-between-wine-xwayland-and-wayland-broken/17802)).
* **Aucune version Mac n'est publiée** (ETAT § 1) : le recul de COPIER-1 sous Mac ne touche
  aujourd'hui personne. **Linux d'abord**, vérifié pour de vrai ; le Mac ensuite (§ 7).

## 3. Ce qui est fait

* **La règle des formes, écrite une fois** (`Formes::offertes`, `interactions/clipboard/formes.rs`) :
  toujours le lot, le texte s'il y en a ; au presse-papiers, une image seule en image et jamais
  aussi en fichier ; sinon les fichiers dès une image. Windows la lit désormais
  (`selection_windows`, qui la dit dans ses formats : `Rendu`) ; Linux aussi.
* **Les fichiers d'une copie** (`plateforme/copies.rs`) sortent de `selection_windows` : écrits au
  premier collage qui les veut, une fois ; le dossier d'un lot vit jusqu'au suivant. Partagés par
  Windows et Linux.
* **Le propriétaire X11 de Glucose** (`plateforme/selection_x11*`, `decisions/11`) : une connexion
  et une fenêtre à soi, servies par un fil à soi.
  * Ce qui part : `application/x-glucose-lot` (le lot, tel quel), `UTF8_STRING`,
    `text/plain;charset=utf-8`, `text/plain` (le texte), `image/png` (l'image seule), ou
    `text/uri-list` et `x-special/gnome-copied-files` (les fichiers, pour Chromium, KDE et les
    gestionnaires de fichiers de GNOME). **Plus de HTML** : il portait le lot en base64 dans
    tout ce qui colle du HTML.
  * Le protocole : `TARGETS`, `TIMESTAMP`, `MULTIPLE` ; `INCR` dès que la forme dépasse la plus
    grande requête que **ce** serveur accepte, sans autre seuil ; un envoi par tranches finit
    quand son demandeur disparaît (`DestroyNotify`) — l'évènement, pas une horloge.
  * **L'heure du serveur** : la propriété se prend à l'heure d'une écriture vide sur notre
    fenêtre — qui réveille aussi le fil qui sert. Une demande d'avant la prise est refusée
    (ICCCM 2.2), l'horloge qui boucle tous les 49 jours comprise.
  * **À la fermeture**, la sélection va au gestionnaire du presse-papiers (`SAVE_TARGETS`, avec
    la liste des cibles à garder) ; la remise finit **quand il la prend** — le seul signe qui
    vaut chez tous, KDE compris (il répond avant même de demander les formes). Un collage après
    coup la trouve encore.
* **Coller le bloc d'un autre Glucose** : la fenêtre de sélection porte une marque
  (`_GLUCOSE_SELECTION`). Un `Ctrl+V` ne demande qu'au **serveur** qui tient la sélection et si
  elle porte la marque : la nôtre se lit sur place ; celle d'un autre Glucose, **hors du fil qui
  dessine** (un `LotALire`, lu sur le fil du collage) ; une autre application n'est jamais
  sollicitée — elle pourrait ne jamais répondre.
* **La CI** : la tâche Linux installe Xvfb et `xclip`, lance un serveur X d'épreuve
  (`-noreset`, § 5), et ses épreuves **tombent** s'il manque.

## 4. Comment c'est prouvé

* **Une distribution Linux sur ce PC, à part** : `Glucose-Linux` (WSL, Ubuntu 24.04, image
  officielle vérifiée par son empreinte), sans toucher NixOS-Proteo. Rust 1.99 comme ici et sur
  GitHub ; un utilisateur ordinaire (les épreuves des droits mentent en `root`) ; un serveur X
  **invisible** (Xvfb `:99`) — rien ne s'affiche sur son écran. Mode d'emploi : `docs/METHODE.md`
  § 5.
* **Le code de Linux compile et passe clippy strict sous Linux**, pour la première fois depuis ce
  PC.
* **Neuf épreuves**, dont sept contre le vrai serveur X, **avec `xclip`** comme demandeur — une
  implémentation qui n'est pas la nôtre : l'image seule, les fichiers écrits à la demande, un lot
  de 33 Mo par tranches (relu entier par `xclip` et par un autre Glucose), `MULTIPLE`, le refus
  d'une demande d'avant la prise, la remise à un gestionnaire (en 1 à 7 ms, pas à l'échéance), et
  le lot lu chez soi, chez un autre Glucose, jamais chez une application muette.
* **Quinze sabotages, quinze chutes** : la liste coupée sur `\n`, l'espace non codé, l'heure
  ignorée (deux fois), `MULTIPLE` qui ne refuse rien, jamais d'`INCR`, une tranche qui saute un
  octet, l'image seule en fichier aussi, le texte sans `charset=utf-8`, le collage qui sollicite
  tout le monde, la remise qui n'attend pas, qui ne finit pas, qui ne liste rien, le lecteur qui
  prend l'en-tête `INCR` pour le lot, les fichiers réécrits à chaque collage.
* **Vingt tours de suite, vingt verts.**
* Les épreuves ne visent **jamais** l'écran de quelqu'un : seulement le serveur qu'on leur nomme
  (`GLUCOSE_ECRAN_X_D_EPREUVE`) — sur une vraie session Linux, elles prendraient sinon le vrai
  presse-papiers de la personne qui les lance.

## 5. Ce que la session a payé

* **Des courses dans les épreuves, pas dans Glucose.** Deux blocages : un gestionnaire d'épreuve
  qui fermait sa connexion juste après sa dernière réponse — le serveur peut jeter les dernières
  requêtes d'un client qui part, et `xclip` restait sans réponse une fois sur deux ; et l'ordre
  entre deux clients X, que rien ne garantit sans un aller-retour. Glucose, lui, ne peut pas
  perdre ainsi sa dernière réponse : son fil ne s'arrête qu'en recevant `DestroyNotify`, que le
  serveur n'envoie qu'après avoir traité tout ce qui précède.
* **Un serveur X d'épreuve se réinitialise** chaque fois que son dernier client part, et coupe un
  moment les connexions : 288 réinitialisations dans son journal, une épreuve sur vingt tombait.
  `-noreset`, ici et sur GitHub.
* **Une garde redondante dans le code neuf** : la remise finissait sur la réponse du gestionnaire
  **ou** sur la sélection prise — chacune, retirée seule, n'aurait rien fait tomber. La seconde
  suffit chez tous ; la première est partie.
* **`root` ment sur les droits** : une épreuve d'un fichier en lecture seule tombait en `root`,
  et des dossiers d'épreuve laissés par `root` ont fait tomber quatre épreuves d'images ensuite.

## 6. Le téléphone

* **Son téléphone est le moto g54 5G** (MediaTek Dimensity 7020, écran 1080 × 2400 à 120 Hz,
  Android 13 d'origine) — **pas** le Redmi 9 que les documents nommaient. Sa puce graphique est
  une **PowerVR, l'IMG BXM-8-256** — pas une Mali : la recherche de la fiche 59 sur les rayures
  (Mali-G52) visait la mauvaise puce. Corrigé dans ETAT et la skill du téléphone.
* **La piste, si les rayures reviennent** : Chromium (Dawn, l'implémentation de WebGPU de
  Chrome) **désactive un chemin Vulkan sur les puces Imagination** pour un défaut connu du
  pilote ([Dawn](https://dawn.googlesource.com/dawn/+/refs/heads/chromium/7922_43)) ; sur la même
  puce (un moto g73), Chrome affichait des pages corrompues avec le pilote GLES 1.15@6133109,
  corrigé dans un pilote plus récent que seul le fabricant distribue ([le forum
  d'Imagination](https://forums.imgtec.com/t/bxm-8-256-long-list-of-driver-issues/3891)). La
  première question sera donc la **version du pilote**, que son journal par le câble dit au
  lancement (fiche 59 § 7). Rien n'est conclu : « pour l'instant rien », a-t-il dit.

## 7. Ce qui n'est pas fait, et pourquoi

* **Le Mac** : `NSPasteboard` à plusieurs types, les fichiers fournis à la demande (un
  `NSPasteboardItemDataProvider`). Aucune version Mac n'est publiée : personne ne recule
  aujourd'hui ; et ce PC ne peut ni le compiler ni le lancer — seule la CI le compile, et aucun
  Mac ne l'essaierait. **Le prochain chantier du copier**, après la beta 3.
* **Le glisser hors de la fenêtre sous Linux** (XDND) : Windows l'a depuis la fiche 59 ; sous
  Linux, un nœud qui sort de la fenêtre revient à sa place, comme avant. Un protocole à part.
* **Un bureau Wayland sans XWayland** : ni ce copier, ni `arboard`. Il faudrait parler
  `wl_data_device` sur la connexion de `winit`, avec le numéro du dernier geste.
* **Les applications en bac à sable** (Flatpak) : elles ne voient pas `/tmp` de l'hôte. Chromium
  offre pour elles le portail des fichiers (`application/vnd.portal.filetransfer`) ; une image
  seule, elle, passe toujours (ses octets voyagent dans la sélection).

## 8. Ce qui n'est pas prouvé

* **Rien n'a tourné sur son Linux** : ni le copier vers Discord, ni le recollage entre deux
  Glucose, ni TENU-1, ni la remise à son gestionnaire. Le serveur X d'épreuve et `xclip` disent
  que le protocole est juste ; son bureau (GNOME ou KDE, X11 ou Wayland) et son Discord diront le
  reste.
* Le relais de XWayland vers les applications Wayland : lu, pas vu.

## 9. Ce qui attend sa parole

* **L'envoi**, avec ou sans la CI : les cinq commits de la fiche 60 et ceux de cette session.
  Avec la CI, la tâche Linux éprouvera la sélection contre son Xvfb ; puis **la publication de la
  2.0.2-beta.3** (Actions → Publier, en brouillon) — son clic « Publish release ».
* **Son NixOS allumé**, et l'accès SSH s'il le permet : la liste de ce qui ne marche pas s'y
  lirait, et le copier s'y essaierait.
* Sa clé SauceNAO, quand on attaquera la provenance.

---

**Retour** : [`00-INDEX.md`](00-INDEX.md)
