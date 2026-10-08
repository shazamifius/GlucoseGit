# Où en est Glucose

> **Au 08/10/2026, dans la nuit**, après la session de ses retours sur la V2 (fiche 51), celle de la
> carte de l'écran (fiche 52), celle de Pinterest et du pavé (fiche 53), celle de la similitude
> des doigts, de la télémétrie et d'Android (fiches 54 et 55), celle du partage vers Glucose
> et des documents du téléphone (fiche 56), celle du clavier, de l'appui long et des
> documents qu'on renomme (fiche 57), et celle des questions dessinées partout, des noms du
> rail et des emojis (fiche 58). Ce document dit l'état **vérifié** du projet,
> en une lecture. Il se réécrit à chaque fin de session qui change quelque chose d'important ;
> l'historique, lui, vit dans le [carnet](carnet/00-INDEX.md).

---

## 1. En une page

* **Glucose Rust est publié.** La **2.0.1-beta.1** est la release « latest » du dépôt depuis le
  30/09/2026 : installeur Windows, AppImage, `.deb`, `.rpm`, et `nix run github:shazamifius/GlucoseGit`.
  **La 2.0.2-beta.1 attend en brouillon** (08/10, fiche 55 § 1), APK Android compris : le clic
  « Publish release » est son geste.
* **Glucose Tauri bascule de lui-même.** Chaque Glucose Tauri installé (depuis la 1.0.1-beta.9)
  trouve la 2.0.1 dans le même `latest.json`, par le popup habituel. Ses documents s'ouvrent dans
  Glucose Rust, et ses données ne sont jamais touchées. Cette bascule a été répétée de bout en bout
  sur GitHub : onze secondes de Tauri ouvert à Glucose Rust vivant.
* **Ce qu'on sait des utilisateurs : presque rien.** Au 06/10, l'installeur Windows a été téléchargé
  **2 fois**, les paquets Linux **0 fois**, et `latest.json` a été lu 14 fois, dont une bonne part par
  sa propre machine. **La télémétrie est en ligne** (fiche 54 § 6) : le serveur tourne dans son
  compte Cloudflare (`glucose-boite-noire.ferme-nilslamber.workers.dev`, base D1 en Europe de
  l'Ouest), éprouvé de bout en bout sur Internet ; Glucose y envoie avec l'accord de chacun,
  demandé au premier lancement. **Le 08/10 au soir, huit sessions y sont arrivées**, dont quatre
  de son téléphone (fiche 57 § 5) : la question dessinée a donc marché chez lui.
* **Android : Glucose tourne sur son Redmi 9** (« oui, ça fonctionne sur téléphone », 08/10) —
  ARM 64 et 32 bits, Android 5.0 et après, avec les doigts. L'APK est signé de **la clé de
  Glucose pour Android** (créée le 08/10, `Documents\Glucose-cle-android`, et dans les secrets
  du dépôt) : la publication le joint à la release.
* **Le téléphone, fiche 56 — écrit, éprouvé hors écran, jamais essayé sur un téléphone** :
  Glucose dans « Partager » (images, et liens d'épingles rapatriés), la surface qu'Android
  reprend et rend, deux touchers sur le vide pour le menu, « Ajouter des images… » par le
  sélecteur de photos, la question dessinée (le journal technique, refusé à sa place par
  l'ancien Glucose, se reposera), les documents rangés sous « Canevas N » et rouverts par une
  liste.
* **Le téléphone, fiche 57 — écrit, éprouvé hors écran, jamais essayé sur un téléphone** :
  **le clavier** (le miroir de la saisie : on écrit enfin au téléphone), **l'appui long** (le
  clic droit du doigt, au délai que règle l'accessibilité d'Android), **renommer, dupliquer,
  supprimer** un document par un appui long dans la liste, et **le bord à bord** (l'interface
  évite la barre d'état, la navigation et le clavier, que Android 15 et 16 imposent).
  Un APK de cette version, signé de sa clé, est sur son Bureau (`Glucose.apk`).
* **Fiche 58 — « tout fonctionne » (09/10)** : **toutes les questions se
  dessinent dans Glucose**, au bureau aussi (POPUP-1 : ses écrans noirs venaient d'une boîte du
  système posée avant la première image) — Entrée, Tab, Échap ; **le rail écrit le nom de chaque
  icône** ; **les emojis se dessinent** (Noto Emoji monochrome, en repli d'Inter). Puis, à sa
  demande : **le journal technique lisible** (`journal-technique`, un `LISEZ-MOI.txt` en
  colonnes) et **les gestes à plusieurs doigts** (annuler, rétablir, choisir plusieurs nœuds).
* **La parité avec Glucose Tauri est loin d'être atteinte.** Environ un quart du logiciel au dernier
  comptage (24/09), mais le cœur est solide : document, rendu, images, flèches, texte.
* **Il ne peut dépenser aucun argent.** Pas de certificat Windows, pas de compte Apple : l'iPad passera
  par le web, Android par un APK installé à la main.

## 2. Ses utilisateurs

| qui | combien | sur quoi | ce qui les attend |
|---|---|---|---|
| utilisateurs de Glucose Tauri | 5 | Windows et Linux | la bascule automatique (faite) |
| testeurs qui attendent Glucose Rust | une vingtaine | dont **10 sous Android** | le portage Android |
| une personne qui compte énormément pour lui | 1 | **iPad** | Glucose dans Safari |

## 3. Ce que fait Glucose Rust aujourd'hui

| domaine | ce qui marche |
|---|---|
| **Naviguer** | canevas infini ; **à la souris, instantané** (molette, glisser au bouton du milieu : rien ne glisse) ; pavé tactile **par *Direct Manipulation*** : la similitude entière des doigts, sans bascule (fiche 54 § 3 — Windows zoome bien sous le curseur, **vérifié** ; le biais **à juger à son écran**), `F` (vol qui cadre la sélection, ou tout), `Ctrl+F` (tout), signets `Ctrl+1..9` / `1..9`, minimap qu'on tient, onglets (renommer, ranger, supprimer, importer un document dedans) ; **au doigt, sur un écran** hors Windows (fiche 54 § 7) |
| **Le document** | **`Ctrl+N`** (un document vierge) ; `.glucose` qui s'écrit **geste après geste** (plus de gel à l'enregistrement), images scellées dans le fichier, brouillons, texte en cours de frappe qui survit à un plantage, reprise au lancement (le dernier document, le curseur), Time Machine (`Ctrl+H` : regarder, restaurer, jalons datés), documents Tauri lus par un lecteur écrit ici |
| **Images** | PNG, JPEG, WebP, GIF, BMP ; collage, dépôt de fichiers, **dépôt depuis un navigateur** sous Windows (Pinterest en pleine résolution, la page demandée **compressée** ; un repli en lien **dit pourquoi**, et se rattrape au clic droit : « Remplacer par l'image ») ; rotation, recadrage non destructif, `Ctrl+B` (bordures) ; mémoire par étages |
| **Texte** | Markdown, tableaux, liens, **LaTeX** fidèle à KaTeX, annulation mot par mot |
| **Flèches** | l'aspect de Tauri, droite ou courbe, double sens, épaisseur, six relations, coudes, **contournement** des obstacles, **ancres de texte** (une flèche part d'une phrase précise) |
| **Copier, coller** | **la sélection entière** (textes, images, flèches, membranes) par `Ctrl+C`/`Ctrl+X`/`Ctrl+V`, d'une fenêtre de Glucose à l'autre, et **glissée** de l'une à l'autre ; au clic droit sur une image : **Copier l'image** (Discord, un navigateur), **Enregistrer l'image sous…** (les octets d'origine) |
| **Organiser** | membranes qui **possèdent** ce qu'on y dépose, mode Focus, dossiers, domaines, Ordonner, aimant aux voisines, placement aimanté dès le premier clic |
| **Fenêtre** | **le mode référence**, à la PureRef : `Ctrl+Maj+A` — sans interface, sans cadre, au premier plan, retenu à la relance ; déplacée au bouton droit, redimensionnée par un bord |
| **Mise à jour** | automatique, signée, au format de Tauri, sous Windows et Linux ; **sous Android, écrite** (fiche 58 § 6 : l'APK confié à l'installeur du système) — pas encore prouvée, aucune release ne la porte |

## 4. La qualité, mesurée

* **Épreuves** : 2 206 sous Windows, clippy strict à zéro, **et pour Android** ; la fiche 58 a
  saboté trente et une gardes, trente et une chutes (une redondante, retirée). **Sur GitHub, à
  chaque envoi, dix tâches** : Windows, Linux, Mac à puce Apple, Mac Intel, le noyau pour Android
  et le web, **l'application Android et son APK** (nouvelle, jamais lancée), NixOS, l'installeur et
  la mise à jour, la répétition de la bascule, les paquets Linux. **Les dix vertes sur
  `89510e2`** (08/10 ; la tâche des paquets Linux, figée une heure et demie à installer ses
  dépendances chez GitHub, relancée) ; **les commits suivants du 07/10 ont été envoyés sans CI**, à sa demande
  (`[skip ci]`), et ceux de la fiche 53 aussi, à sa demande (« je ne veux absolument pas que tu
  déclenches la CI », 07/10 au soir) : Linux et Mac **à vérifier avant la publication** (relus,
  fiche 53 § 4 ; seule la CI le prouvera). `a32e074`, `efab62f` et la documentation qui suit
  sont envoyés **sans CI** eux aussi, à sa demande (« tu peux parfaitement les envoyer sans
  déclencher la CI », 07/10 au soir), jusqu'à `d17e838` — vérifié : aucune exécution. Ceux de
  la fiche 54 sont partis **avec** la CI (`89510e2`, dix vertes) ; la télémétrie branchée et
  l'APK de la publication, **sans** (« fais pas marcher la CI », 08/10), jusqu'à `df1b854`
  (`[skip ci]`, vérifié : aucune exécution). Le rail, les fiches 55, 56 et 57 — douze
  commits, jusqu'à `57060c8` — sont partis **avec** la CI, à sa demande (08/10 au soir) : **les
  dix tâches vertes**, Linux et Mac compris (exécution 37801814665). **Les commits de la fiche 58 attendent en
  local** (POPUP-1, le rail, les emojis, la mise à jour d'Android, la documentation) : rien
  n'est envoyé sans son accord.
* **Sa machine** (Windows 11, RTX 5070 Laptop et Intel Arc 140T, écran 240 Hz à 150 %), session du
  05/10 : une image coûte **5 à 7 ms** en médiane. Le pire monte à **11-21 ms pendant les zooms
  chargés** et à 31 ms au décodage de l'ouverture : le plancher de 100 images par seconde n'est
  **pas encore tenu partout**.
* **Le lag du 07/10 au matin** (gels de 300 à 500 ms dans `present`, pour des images de 3 ms) :
  son portable a l'écran branché sur la RTX, et Glucose dessinait sur l'Intel, qui n'affiche
  rien et s'endort au repos. L'arbitre alternait d'une session à l'autre. Depuis ECRAN-1
  (fiche 52), Glucose dessine sur la carte qui tient l'écran — **vérifié** sur sa session
  suivante : la RTX, `mailbox`, pire image 48 ms au lieu de 505.
  Les gels de l'Arc des fiches 43 et 21 avaient très probablement la même cause.
* **Aucun plantage ni gel** de Glucose noté par Windows depuis la publication.
* **Au repos**, aucun réveil spontané : la chronique du 07/10 le dit raison par raison — seuls
  dessinent les messages qui s'effacent et les glissades qui finissent.

## 5. Ce qui manque encore (de Tauri)

Rideaux · temporalité (réglette de −10 000 à 2 100) · storyboard · presets et zones · exports PNG et
HTML · miroirs et dossiers miroirs du disque · recherche (`Ctrl+F`) · couleur des domaines sur le
canevas · vidéos · provenance des images (SauceNAO) · sélecteur de couleur · collaboration · MCP ·
plugins et IA locale · Mac · iPad. **Android** : tourne sur son téléphone ; les questions s'y
dessinent (fiche 56) ; le clavier, l'appui long et la gestion des documents sont écrits (fiche
57), le bord à bord aussi ; **pas encore** de presse-papiers du système, de mise à jour
automatique, ni de batterie dans la télémétrie ; les emojis se dessinent (fiche 58), sans les
familles ni les drapeaux composés. **Les tablettes Windows** : les doigts n'y passent
pas encore (la souris simulée à écarter d'abord). Les membranes « étirées » et « minimisées » (l'idée de
Mary) **attendent sa parole**, comme les rideaux et Trans-domaines.

## 6. Ses retours sur la V2 — jugés à son écran le 07/10

Fiches 51 et 52. **Tout ce qui suit est vu et approuvé à son écran** (« tout est absolument
parfait », 07/10 au soir), sauf où c'est dit :

* le copier-coller de nœuds, entre deux fenêtres et glissé de l'une à l'autre ; « Copier
  l'image » dans Discord et « Enregistrer l'image sous… » ;
* `Ctrl+N`, qui **demande toujours** (NOUVEAU-1) ;
* le mode référence (`Ctrl+Maj+A`) : sans interface ni panneaux, au premier plan ; au pavé,
  `Alt` + glisser déplace la fenêtre et `Alt` + pincer la redimensionne (une fois par image,
  REFERENCE-3) ; les signets retiennent le **centre** de la vue (SIGNET-2) ;
* les poignées, le cadre et l'anneau de sélection qui suivent la taille des nœuds (POIGNEE-1) ;
* la molette : trois crans doublent ;
* **le lag réglé** (ECRAN-1, la carte qui tient l'écran).

**Ouvert** :

* **le pincement au pavé**, par *Direct Manipulation* (fiche 53 § 2) : « c'est bien, le zoom
  maintenant » (07/10, 15 h 40) ;
* **le déplacement à deux doigts** : « la translation, la multi-direction pose problème ». Cause
  trouvée dans sa chronique — dix-neuf déplacements pris pour des pincements, bloqués jusqu'à la
  fin du geste ; corrigé par la similitude entière (fiche 54 § 3) — **son ressenti attendu** ;
* **la coupure du pavé de 7 à 10 secondes**, aussi dans Blender : cause inconnue ; la boîte noire
  en note désormais chaque étage (fiche 54 § 4) — **la prochaine coupure dira où** ;
* **sur batterie**, la cadence est tombée à 30 images par seconde par moments (fiche 54 § 5) :
  probablement le bridage de la RTX, à confirmer ; la charte demande de pixeliser plutôt ;
* **le cadre du groupe qui restait sur place** pendant un glisser : corrigé (GROUPE-1) — à son
  écran ;
* **transformer une sélection entière** (sa demande « comme Blender ») : poignées du groupe,
  `Alt` + coin, bouton « Origine commune / individuelles » (fiche 53 § 10) — **à son écran** ;
* **Pinterest** : six épingles devenues six liens le 07/10 (fiche 53 § 1) — la page arrivait
  nue, 1,2 Mo au lieu de 127 Ko compressée. Corrigé : « ça fonctionne, mais plutôt long » → la
  copie se pose désormais en 0,3 à 0,9 s, et l'original la remplace (fiche 53 § 9) ;
* **le dessin de la sélection** : « pas esthétique du tout ». Cinq directions **envoyées à son
  écran** (`docs/carnet/screens/53-selection-cinq-directions.png`) ; mon avis : E, un cadre
  pour le groupe et un fil fin par image (PureRef, Figma, tldraw) — **son choix** ;
* le gel qui a « failli planter » son PC : cause probable, la rafale de redimensionnements de
  `Alt` + pincer, corrigée (REFERENCE-3) — pas revu depuis ;
* l'opacité et les clics qui traversent du mode référence, s'il les veut ; et ce qu'il regrette
  d'avoir perdu en mode référence (« dommage que tu les aies supprimés, c'était vraiment
  pratique ») — à lui demander.

**Il navigue surtout au pavé tactile**, et a une souris, essayée le 07/10.

## 7. Points de vigilance

* **Sa clé de signature** est dans les secrets du dépôt (elle a signé chaque version de Tauri). Il
  manque **une copie hors de ce PC** : perdue, plus aucune mise à jour n'atteint personne.
* **`%TEMP%\glucose_pasted`** garde 2,9 Go d'anciennes images collées. Aucun document n'en dépend
  plus (fiche 47), mais c'est à lui de décider de les effacer.
* Sur GitHub, quatre **brouillons de release** de juillet (Tauri 1.0.1-beta.1, .7, .16, .19) et la
  branche `feat/mcp-glucose-bridge` (entièrement contenue dans `main`) encombrent. À retirer s'il
  le veut. Une branche locale, `secours/avant-reattribution`, garde 83 commits d'avant la
  réécriture des auteurs : une sauvegarde, à garder.
