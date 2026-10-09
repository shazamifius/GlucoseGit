# 60 — Les dossiers, la carte au journal, le presse-papiers tenu

> Session du 09/10/2026, au soir, à la suite de la fiche 59. Ses essais de la fiche 59, un par
> un ; l'envoi avec la CI ; trois corrections ; et un recul trouvé en relisant le copier, qui
> retient la publication de la 2.0.2-beta.3. **§ 6 : ce qui attend sa parole.**

## 1. Ses retours, mot pour mot

**Au bureau** :

1. « c'est good, aucun panneau ouvert » ;
2. le copier vers Discord : « c'est absolument impressionnant… j'ai l'impression que ça marche
   pour genre TOUT… les textes, les post-it, les images, c'est magnifique !!! » — puis : « juste,
   tu peux vérifier les journaux envoyés depuis mon Linux, mais OUAAA il y a TELLEMENT de choses
   qui ne marchent pas — justement donc ce système de bloc, dans la version 2.0.2 beta 2 » ;
3. recoller dans Glucose : « parfait aussi sur Windows » ;
4. glisser vers Discord, Docs, le bureau : « parfait aussi sur Windows » ;
5. le défilement au bord : « c'est moche, c'est pas fluide, c'est pas très pratique, ça a du
   mal… c'est pas ouf, mais t'inquiète, te concentre pas là-dessus » ;
6. fermé : « fait ».

**Au téléphone** : l'APK installé ; la question au-dessus de tout, « fait » ; « Supprimer » par
l'appui long : « oui ça fonctionne, mais les dossiers ça ne fonctionne pas — tu sais, les
fichiers ?? dossiers ?? » ; les rayures : « pour l'instant rien, je surveillerai ».

**Ses paroles** : « mon téléphone est déjà branché, tu peux déjà y accéder à 100 % » ; « oui, tu
peux mettre à jour » (le serveur du journal technique) ; « j'aimerais bien que tu envoies tout
sur GitHub avec la CI, plus la publication de la 2.0.2 beta 3 ».

## 2. Ce qui est fait

* **L'envoi, avec la CI** : les six commits de la fiche 59, jusqu'à `3fbf383` (exécution
  37946507360) — **les dix tâches vertes**, Linux, les deux Mac et NixOS compris : le code du
  copier hors de Windows, qui n'avait été que relu, compile et passe ses épreuves.
* **DOSSIER-1** (`abae87b`, en local) — « les dossiers ça ne fonctionne pas ». L'appui long
  choisissait le dossier, mais le menu et la barre d'action ne comptaient que les images et les
  annotations : un dossier choisi recevait le menu du vide, sans « Supprimer », et aucune barre.
  Au bureau, la touche `Suppr` le prenait ; le téléphone n'en a pas. Le noyau compte désormais la
  sélection une fois, dossier compris (`Store::compte_de_la_selection`), et le menu, la barre et
  le clavier lisent ce compte. Un dossier seul n'offre que « Supprimer » : copier, dupliquer et
  l'ordre ne portent pas encore les dossiers (Glucose Tauri offrait « Couleur » et « Supprimer »
  dans sa barre de dossier). Une épreuve joue le geste entier au doigt : appui long, Supprimer,
  annuler rend le dossier. Quatre sabotages, quatre chutes.
* **CARTE-1** (`e869805`, en local) — la carte graphique dans le journal technique, **par ses
  numéros** : le moteur (`vulkan`, `gl`, `dx12`, `metal`, `webgpu`, `aucun`), le genre (`integree`,
  `dediee`, `virtuelle`, `processeur`, `autre`), le fabricant et le modèle (PCI, ou Khronos sur
  un téléphone), le plafond des textures. **Jamais le nom que donne le pilote** : le type de la
  boîte noire ne porte que des nombres et des mots fixés à la compilation, et la promesse de
  `docs/TELEMETRIE.md` reste entière (elle dit désormais ce qui part en plus, comme le
  `LISEZ-MOI.txt`). Une seule source : la présentation rend son nom (pour l'écran) et sa carte
  (pour le journal) ; l'identité d'ECRAN-1 se lit dans la carte. **Le serveur d'abord** : déployé
  à sa parole (version `0bf7998f`), vérifié en ligne — une session portant la ligne entre (201),
  la même avec un nom glissé dans le moteur est refusée (422), l'essai effacé de la base. Huit
  sabotages, huit chutes (cinq de Glucose, trois du serveur).
* **TENU-1** (`b72965a`, en local) — **une cause probable de « tant de choses qui ne marchent pas »
  sous Linux.** Lu à la source d'arboard 3.6.1 (`platform/linux/x11.rs`) et dans sa documentation
  ([docs.rs](https://docs.rs/crate/arboard/latest)) : sous Linux, c'est le programme qui a copié
  qui sert chaque collage, tant que l'objet d'arboard vit — « ne le lâchez pas aussitôt ».
  Glucose en ouvrait un par geste et le lâchait sur-le-champ : arboard tentait de confier la
  copie au gestionnaire du système (jusqu'à 100 ms d'attente), et **là où il n'y en a pas, ce
  qu'on venait de copier disparaissait** — un bloc d'un Glucose à l'autre compris. Un objet est
  désormais tenu toute la vie de l'application (`plateforme::presse_papiers::tenir`) ; ceux de
  chaque geste partagent son serveur (`Clipboard::new` rend le global tant qu'il vit) ; il se
  lâche à la fermeture, l'instant où arboard confie la copie au gestionnaire. Deux sabotages,
  deux chutes. **Pas prouvé sous Linux** : l'épreuve tient le mécanisme, son Linux dira le reste.
* En passant : deux messages abîmés par une barre oblique que Git Bash avait mangée (une suite
  d'espaces au milieu d'une phrase, `ouverture.rs` et `bench_salissure.rs`).

## 3. Ce que la relecture a trouvé — et qui retient la beta 3

**COPIER-1 fait reculer Linux et Mac.** La 2.0.2-beta.2 avait, hors de Windows, un « Copier
l'image » par `arboard::set_image` : l'image partait vers Discord. COPIER-1 l'a retiré du menu, et
hors de Windows le copier unique ne pose que le lot dans du HTML — `arboard` ne pose **qu'une
forme à la fois** (son API publique ; en interne, son serveur X11 sait en servir plusieurs).
Publiée telle quelle, la beta 3 enlèverait aux utilisateurs de Linux et de Mac le moyen d'envoyer
une image à une autre application. Les deux voies, chiffrées et proposées à lui :

* **A — ce soir** : remettre « Copier l'image » sous Linux et Mac seulement, tel qu'il était dans
  la beta 2 (le code est dans `git show 5fad7bc`, et il compilait) ; environ une heure, CI
  comprise. Windows a le copier unique ; Linux et Mac gardent deux gestes en attendant.
* **B — d'abord la vraie voie** : écrire hors de Windows ce que Windows a déjà (un objet qui sert
  plusieurs formes, les fichiers fabriqués à la demande). Sous X11, un propriétaire de la
  sélection à soi, par `x11rb` (déjà dans l'arbre par arboard) : `TARGETS`, le lot sous un type à
  soi, `image/png`, `text/uri-list` servi à la demande, `INCR` pour les grosses données, et la
  remise au gestionnaire à la fermeture (`SAVE_TARGETS`) ; sous macOS, `NSPasteboard` à plusieurs
  types (`objc2-app-kit`, déjà dans l'arbre). Plusieurs heures ; ce PC ne compile pas pour Linux
  (la caisse `ring` veut un compilateur C de Linux) — sauf par sa NixOS de WSL, s'il le permet.
  `clipboard-rs` a été lu et écarté : il pose plusieurs formes d'un coup, mais n'écrit **jamais**
  par `INCR` (une image de quelques Mo part d'un bloc, et l'erreur tue sa boucle de service).

Mon avis, donné : **A ce soir, puis B en chantier**. Sa réponse n'est pas venue avant la fin de
la session.

## 4. Ce que disent les journaux

* **Son Linux** : une seule session, aujourd'hui, la 2.0.2-beta.2 (installation `13cff848…`),
  293 s, **fermée proprement** ; des gels jusqu'à 142 ms en déplaçant la vue vers la 180ᵉ seconde,
  et 107 ms au décodage des images. Le journal technique ne dit que des durées : il **ne peut pas**
  dire ce qui ne marche pas — c'est à lui de le lister. Ce PC porte une NixOS sous WSL
  (« NixOS-Proteo ») : **son Linux est peut-être celui-là** (WSLg), à lui demander — et s'il
  permet qu'on s'en serve pour compiler et essayer Glucose sous Linux.
* **Son téléphone** : branché, Windows le voit comme un **moto g54 5G**, en transfert de
  fichiers (WPD), **sans débogage USB** : `adb devices` ne le voit pas. Les documents disent un
  Redmi 9 (Helio G80, Mali-G52). Lequel avait les rayures ? À lui demander, avec le chemin du
  débogage USB (Paramètres → À propos du téléphone → sept touchers sur « Numéro de build », puis
  Système → Options pour les développeurs → Débogage USB, et accepter la question sur le
  téléphone). La télémétrie ne connaît qu'**une** installation Android (`1c287c59…`).
* **Sa chronique au PC** (`sortie-essais-59.txt`, 391 s, RTX 5070, 240 Hz) : 6 % des images
  au-dessus de 10 ms ; **au repos, 371 images redessinées pour « un message à l'écran »** (1,6
  image par seconde quand rien ne bouge — la charte dit zéro) et des images de repos à 16 ms pour
  55 nœuds ; un tressaut de 64 px au p99 ; le bord qui fait défiler, 16 % des images — à relire
  avec son « c'est pas fluide ».

* **Les publications** : la **2.0.2-beta.2 est la release « latest »** depuis le 09/10 (13 h 44
  UTC), construite sur `06f4217` (la fiche 58), APK Android compris — **aucun document ne le
  disait** (`ETAT.md` parlait encore de la 2.0.1 et d'un brouillon) ; corrigé. La beta 3 sera
  donc la première version à pouvoir arriver d'elle-même à un téléphone qui a la beta 2
  (MAJ-ANDROID-1).

## 5. Les leçons

* **Relire ce qu'un chantier retire, plateforme par plateforme.** COPIER-1 a ajouté sous Windows
  et retiré partout ; la CI verte ne voit pas un recul de fonction.
* **Une bibliothèque se lit jusqu'à sa durée de vie.** « Ouvrir, écrire, refermer » est juste sous
  Windows et faux sous X11, où le copieur sert le collage — arboard l'écrit en toutes lettres.
* **Le journal technique ne dit pas ce qui ne marche pas** : il dit des durées. Une panne de
  fonction se demande à lui, ou se lit dans un journal à soi.
* **Vérifier quel appareil est branché** avant de bâtir un diagnostic sur son nom
  (`Get-PnpDevice`) : le téléphone de la fiche 59 n'est peut-être pas celui des documents.
* **La suite complète avant chaque commit, pas les épreuves ciblées.** CARTE-1 a été commité
  après ses seules épreuves ; la suite entière, jouée pour la passation, a fait tomber le
  cliquet des 80 lignes (`champs` à 86, `avec_cadence` à 82). Corrigé dans un commit à part, en
  extrayant : la carte se décrit elle-même (`Carte::champs`), et l'ouverture rend la carte
  qu'elle lit déjà — l'information de l'adaptateur ne se relit plus deux fois.
* `node --test outils/telemetrie/test` (un dossier) échoue sur ce Node : nommer le fichier.
  Cloudflare refuse un client sans en-tête d'identité (erreur 1010) : en donner un aux essais.

## 6. Ce qui attend sa parole

* **A ou B** pour la beta 3 (§ 3) ;
* **son Linux** : ce qui ne marche pas, en vrac ; si c'est la NixOS de WSL, et s'il permet qu'on
  s'en serve ;
* **le téléphone** : lequel avait les rayures, et le débogage USB s'il veut qu'on lise son
  journal (utile seulement si elles reviennent) ;
* **l'envoi** des quatre commits (`abae87b`, `e869805`, `b72965a`, `a092eb7`) et celui de la documentation,
  avec la CI ; puis **la publication** de la 2.0.2-beta.3 (Actions → Publier, en brouillon) ; le
  clic « Publish release » reste le sien ;
* sa clé SauceNAO, quand on attaquera la provenance.

## 7. Ce qui n'est pas prouvé

* DOSSIER-1 à son téléphone : **l'APK de son Bureau (09/10, 16 h 07) ne le porte pas** — à
  reconstruire avant de lui demander l'essai.
* CARTE-1 : aucune vraie session n'a encore écrit la ligne ; la première après une
  construction le fera, et la base la montrera.
* TENU-1 : le comportement sous Linux.
* Rien de cette fiche n'est envoyé.

---

**Retour** : [`00-INDEX.md`](00-INDEX.md)
