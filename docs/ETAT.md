# Où en est Glucose

> **Au 06/10/2026**, commit `2089375` et ce rangement. Ce document dit l'état **vérifié** du projet,
> en une lecture. Il se réécrit à chaque fin de session qui change quelque chose d'important ;
> l'historique, lui, vit dans le [carnet](carnet/00-INDEX.md).

---

## 1. En une page

* **Glucose Rust est publié.** La **2.0.1-beta.1** est la release « latest » du dépôt depuis le
  30/09/2026 : installeur Windows, AppImage, `.deb`, `.rpm`, et `nix run github:shazamifius/GlucoseGit`.
* **Glucose Tauri bascule de lui-même.** Chaque Glucose Tauri installé (depuis la 1.0.1-beta.9)
  trouve la 2.0.1 dans le même `latest.json`, par le popup habituel. Ses documents s'ouvrent dans
  Glucose Rust, et ses données ne sont jamais touchées. Cette bascule a été répétée de bout en bout
  sur GitHub : onze secondes de Tauri ouvert à Glucose Rust vivant.
* **Ce qu'on sait des utilisateurs : presque rien.** Au 06/10, l'installeur Windows a été téléchargé
  **2 fois**, les paquets Linux **0 fois**, et `latest.json` a été lu 14 fois, dont une bonne part par
  sa propre machine. La boîte noire existe sur chaque machine mais **n'envoie rien** tant que le
  serveur n'est pas décidé (voir [SUITE](SUITE.md), chantier 2).
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
| **Naviguer** | canevas infini, molette, pavé tactile (deux doigts, pincement), élan, `F` (vol qui cadre tout), signets `Ctrl+1..9` / `1..9`, minimap qu'on tient, onglets (renommer, ranger, supprimer, importer un document dedans) |
| **Le document** | `.glucose` qui s'écrit **geste après geste** (plus de gel à l'enregistrement), images scellées dans le fichier, brouillons, texte en cours de frappe qui survit à un plantage, reprise au lancement (le dernier document, le curseur), Time Machine (`Ctrl+H` : regarder, restaurer, jalons datés), documents Tauri lus par un lecteur écrit ici |
| **Images** | PNG, JPEG, WebP, GIF, BMP ; collage, dépôt de fichiers, **dépôt depuis un navigateur** sous Windows (Pinterest en pleine résolution) ; rotation, recadrage non destructif, `Ctrl+B` (bordures) ; mémoire par étages |
| **Texte** | Markdown, tableaux, liens, **LaTeX** fidèle à KaTeX, annulation mot par mot |
| **Flèches** | l'aspect de Tauri, droite ou courbe, double sens, épaisseur, six relations, coudes, **contournement** des obstacles, **ancres de texte** (une flèche part d'une phrase précise) |
| **Organiser** | membranes qui **possèdent** ce qu'on y dépose, mode Focus, dossiers, domaines, Ordonner, aimant aux voisines, placement aimanté dès le premier clic |
| **Fenêtre** | `Alt+T` met la fenêtre au premier plan (caché, et oublié à la relance) |
| **Mise à jour** | automatique, signée, au format de Tauri, sous Windows et Linux |

## 4. La qualité, mesurée

* **Épreuves** : environ 1 970 sous Windows, clippy strict à zéro. **Sur GitHub, à chaque envoi,
  neuf tâches** : Windows, Linux, Mac à puce Apple, Mac Intel, le noyau pour Android et le web, NixOS,
  l'installeur et la mise à jour, la répétition de la bascule, les paquets Linux. Toutes vertes
  au dernier envoi.
* **Sa machine** (Windows 11, RTX 5070 Laptop et Intel Arc 140T, écran 240 Hz à 150 %), session du
  05/10 : une image coûte **5 à 7 ms** en médiane. Le pire monte à **11-21 ms pendant les zooms
  chargés** et à 31 ms au décodage de l'ouverture : le plancher de 100 images par seconde n'est
  **pas encore tenu partout**.
* **Aucun plantage ni gel** de Glucose noté par Windows depuis la publication.
* **Au repos**, Glucose dessine encore 1 à 2,5 images par seconde. La seule bonne réponse est zéro.

## 5. Ce qui manque encore (de Tauri)

Rideaux · temporalité (réglette de −10 000 à 2 100) · storyboard · presets et zones · exports PNG et
HTML · miroirs et dossiers miroirs du disque · recherche (`Ctrl+F`) · couleur des domaines sur le
canevas · vidéos · provenance des images (SauceNAO) · sélecteur de couleur · collaboration · MCP ·
plugins et IA locale · Mac · Android · iPad. Les membranes « étirées » et « minimisées » (l'idée de
Mary) **attendent sa parole**, comme les rideaux et Trans-domaines.

## 6. Ses retours sur la V2 (06/10)

1. **La souris est insupportable** : le lissage du pavé tactile s'applique aussi à la molette et au
   glisser. *À la souris : rapidité, instantanéité. Au doigt et sur tablette : du lisse.*
2. **Aucun moyen de créer un document vierge** : pas de `Ctrl+N`, pas de bouton.
3. **Un mode référence à la PureRef** : plus aucune interface, pas de cadre de fenêtre, toujours au
   premier plan, pour garder sa référence à côté de Blender.

Ces trois retours ouvrent la [suite](SUITE.md).

## 7. Points de vigilance

* **Sa clé de signature** est dans les secrets du dépôt (elle a signé chaque version de Tauri). Il
  manque **une copie hors de ce PC** : perdue, plus aucune mise à jour n'atteint personne.
* **La version de travail** porte encore `2.0.0-dev`, sous la publiée : à passer en `2.0.2-dev`.
* **`%TEMP%\glucose_pasted`** garde 2,9 Go d'anciennes images collées. Aucun document n'en dépend
  plus (fiche 47), mais c'est à lui de décider de les effacer.
* Sur GitHub, quatre **brouillons de release** de juillet (Tauri 1.0.1-beta.1, .7, .16, .19) et la
  branche `feat/mcp-glucose-bridge` (entièrement contenue dans `main`) encombrent. À retirer s'il
  le veut. Une branche locale, `secours/avant-reattribution`, garde 83 commits d'avant la
  réécriture des auteurs : une sauvegarde, à garder.
