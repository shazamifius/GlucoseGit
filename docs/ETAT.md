# Où en est Glucose

> **Au 15/10/2026, le soir** (fiche 72). L'état **vérifié** du projet, en une lecture. Il se réécrit à chaque
> fin de session qui change quelque chose d'important ; l'histoire, session par session, vit dans
> le [carnet](carnet/00-INDEX.md) (fiches 51 à 72 pour la V2). La direction : [VISION](VISION.md).
> La suite : [SUITE](SUITE.md). Chaque fonction et son bouton : [INVENTAIRE](INVENTAIRE.md).

---

## 1. En une page

* **Glucose Rust est publié.** La **2.0.3-beta.3** est la release « latest » depuis le 15/10
  (publiée par lui) : installeur Windows, AppImage, `.deb`, `.rpm`, APK Android, et
  `nix run github:shazamifius/GlucoseGit`. Chaque Glucose Tauri installé (depuis la 1.0.1-beta.9)
  bascule de lui-même vers Glucose Rust ; ses documents s'ouvrent, ses données ne sont jamais
  touchées. **La 2.0.3-beta.4 est prête en brouillon** (fiche 72 : CI dix vertes sur `3e1ee42`,
  le brouillon sur `b6a8ea4`, qui n'ajoute que ses notes) : la télémétrie qui ne perd plus les
  longues sessions, la raison de mort d'Android, ce que la reprise rouvre, le texte net de près,
  le zoom ×256. Son APK, signé de sa clé, est zippé sur son Bureau
  (`Glucose-2.0.3-beta.4-telephone.zip`) : **son essai, puis son clic** « Publish release ».
* **Ses utilisateurs** : 5 de Glucose Tauri (Windows, Linux) ; une vingtaine de testeurs qui
  attendent Glucose Rust, dont 10 sous Android ; une personne qui compte énormément pour lui, sur
  iPad (Glucose dans Safari, plus tard). **La télémétrie** (avec l'accord de chacun, un serveur
  Cloudflare de son compte, `docs/TELEMETRIE.md`) : 59 sessions, 5 installations au 15/10, dont
  deux utilisateurs Windows inconnus de lui.
* **Le 15/10, sur deux téléphones, un tableau de plus de 6 000 images s'est fermé seul et n'est
  pas revenu** (« parfaitement fluide » jusque-là). **Cause inconnue** : aucune session de la
  beta 3 n'est arrivée du téléphone — le serveur refusait toute session de plus de 512 Kio, et le
  refus était retenu pour toujours (réparé, POIDS-1). Le soir, sa précision : l'application a
  « juste dépop » et a redémarré seule sur la carte de bienvenue — la reprise n'a rien rouvert.
  Hypothèses : le brouillon encore tenu par le processus qui mourait (défendu par LIBERE-1), ou
  le disque plein. La beta 4 dira la raison de mort qu'Android garde (SORTIE-1), ce que le
  lancement a rouvert ou sauté, et la place du disque (REPRISE-1). Rien dans le code n'efface un
  brouillon à la mort : **son tableau est peut-être encore dans le téléphone**.
* **Android** : Glucose tourne sur son **moto g54 5G** (une PowerVR BXM-8-256). **Le 12/10, une
  longue session a figé tout le téléphone**, bouton d'alimentation compris, après des saccades qui
  montaient, puis un redémarrage seul — *« c'est bien Glucose »*. Cause la plus probable : la
  mémoire (Glucose n'y bornait pas ses décodages, n'écoutait pas `onTrimMemory`, ne mesurait pas
  sa propre empreinte) — **corrigé côté code (fiche 66), pas prouvé** : il faut une longue session
  sur son téléphone. **Il a perdu son câble** ; le débogage sans fil est probablement impossible
  (son PC passe par le partage de connexion du téléphone) : l'APK passera par un zip (fiche 71 § 3).
  Sur son téléphone, une image coûtait **19,5 ms** en médiane — sous le plancher presque tout le
  temps.
* **Sa machine** (Windows 11, RTX 5070 Laptop et Intel Arc 140T, écran 2 560 × 1 600 à 240 Hz et
  150 %) : 3 à 7 ms par image en médiane ; **le pire monte à 11-24 ms dans les zooms chargés**
  (sa chronique du 14/10 : 345 images sur 13 886 au-dessus de 10 ms). **Le plancher de 100 images
  par seconde n'est pas tenu partout.**
* **La parité avec Glucose Tauri est loin d'être atteinte** (un quart au comptage du 24/09, bien
  plus aujourd'hui ; [INVENTAIRE](INVENTAIRE.md) § 3 dit ce qui manque) ; le cœur est solide.
* **Il ne peut dépenser aucun argent** : pas de certificat Windows, pas de compte Apple, pas de
  compte Play à 25 $. Android passe par un APK installé à la main (Play Protect : « Installer quand
  même » ; la vérification des développeurs n'arrive en France qu'en 2027).

## 2. Ce que fait Glucose Rust

| domaine | ce qui marche |
|---|---|
| **Naviguer** | canevas infini, **zoom jusqu'à ×256** (déduit de la précision du document, fiche 72) ; souris instantanée ; pavé par *Direct Manipulation* (la similitude entière des doigts — « tellement mieux qu'avant », 15/10) ; `F` cadre la sélection, `Alt+F` tout ; signets `Ctrl+1..9` ; minimap ; onglets ; le canevas défile au bord ; les doigts au téléphone |
| **Le document** | `.glucose` écrit geste après geste, images scellées, brouillons, reprise au lancement, Time Machine (`Ctrl+H`), documents de Tauri lus ; `Ctrl+N`, `Ctrl+O` ; au téléphone, les documents rangés et rouverts |
| **Images** | PNG, JPEG, WebP, GIF, BMP ; coller, déposer (Pinterest en pleine résolution sous Windows) ; rotation, recadrage non destructif, `Ctrl+B` ; la mémoire par étages (carte graphique, mémoire, disque, bornés par ce que la machine laisse) |
| **Texte** | Markdown, tableaux, liens, LaTeX fidèle à KaTeX, annulation mot par mot ; le clavier du téléphone ; **de près, les grandes lettres se tracent** (GRAND-1, fiche 72) |
| **L'interface** | **les raccourcis sont une donnée** (une couche de texte, `assets/raccourcis/glucose.txt`), **75 commandes** dans une table, le menu qui les exécute et dit leurs raccourcis, **la palette** (`Ctrl+Maj+P`) — la fondation des presets (P4, fiche 72) |
| **Flèches** | droites ou courbes, double sens, six relations, coudes, contournement, ancres de texte |
| **Copier, coller** | la sélection entière d'une fenêtre à l'autre ; un seul « Copier » qui donne aussi les images (Discord, navigateur) ; le glisser vers d'autres applications sous Windows ; sous Linux, la sélection X11 servie par Glucose |
| **Organiser** | membranes (tracées ou posées) ; dossiers ; domaines ; aimant ; **Ordonner** (cinq dispositions, huit tris dont trois par la couleur) ; **Mood board** (six gabarits de zones, « Par mood board ») ; **le Storyboard** : la grille tracée, retracée, ses cases glissées, décrites, plusieurs images qui remplissent les cases, un storyboard de la sélection, les « + », la lecture plein écran, durée, plan et champs par case, plusieurs storyboards par tableau, les planches en PDF |
| **Exporter** | Markdown ; **SVG complet** (le texte en tracés, lisible par tout logiciel) ; PNG de la sélection ou du tableau ; planches d'un storyboard ; au téléphone, par la feuille de partage |
| **Fenêtre** | le mode référence (`Ctrl+Maj+A`) : sans interface, sans cadre, au premier plan |
| **Mise à jour** | automatique et signée, sous Windows et Linux ; sous Android, écrite, à prouver par la prochaine version |

**Jugé à son écran** (fiches 51, 52, 58, 60, 64) : le copier-coller et le glisser entre fenêtres,
Discord ; `Ctrl+N` ; le mode référence et ses signets ; les poignées ; la molette ; le lag réglé
(ECRAN-1) ; les questions dessinées ; aucun panneau au lancement ; NETTETE-1 (« plus du tout de
flou ») ; l'export au téléphone par la feuille de partage ; le déplacement au pavé. **Tout ce qui
est venu depuis la fiche 62 attend son écran** : la liste unique est dans [SUITE](SUITE.md) § 2.

## 3. La qualité, mesurée

* **2 573 épreuves** sous Windows, toutes vertes (15/10, fiche 72) ; clippy strict à zéro sous
  Windows et pour Android ; la suite Linux tourne aussi sur ce PC (WSL `Glucose-Linux`, METHODE
  § 5), rejouée pour la dernière fois à la fiche 61.
* **Les sabotages** : chaque garde est cassée exprès et doit faire tomber une épreuve
  (`outils/sabotages/`, un fichier par chantier) — 30 sur 30 pour l'export SVG (fiche 71) ; à la
  fiche 72, 25 sur 25 pour le téléphone, 4 sur 4 pour LIBERE-1 (sous Linux), 10 sur 10 pour
  GRAND-1, 2 sur 2 pour le zoom, 20 sur 20 pour P4.
* **Sur GitHub, dix tâches** à chaque envoi avec la CI : Windows, Linux, les deux Mac, le noyau
  pour Android et le web, l'APK, NixOS, l'installeur et la mise à jour, la répétition de la
  bascule depuis un vrai Glucose Tauri, les paquets Linux. Dernières vertes : `82c774b` (13/10) ;
  **`58478d6` (15/10) : dix vertes** — la CI groupée de la beta 3 (exécution 38717823204) ;
  **`3e1ee42` (15/10) : dix vertes** — celle de la beta 4 (exécution 38749250295).
* **Aucun plantage ni gel** de Glucose noté par Windows depuis la publication. **Au repos**, Glucose
  redessine encore pour ses messages (1,6 image par seconde) : la charte dit zéro.

## 4. Points de vigilance

* **Sa clé de signature** (celle de chaque version de Tauri) et **la clé Android**
  (`Documents\Glucose-cle-android`) sont dans les secrets du dépôt, et sauvegardées hors de ce PC.
* `%LOCALAPPDATA%\Glucose` ne se touche jamais ; un diagnostic se fait sur une copie de ses
  documents.
* Une branche locale, `secours/avant-reattribution`, garde 83 commits d'avant la réécriture des
  auteurs : une sauvegarde, à garder. Sur GitHub : `main` et `tauri-v1.0.1`.
