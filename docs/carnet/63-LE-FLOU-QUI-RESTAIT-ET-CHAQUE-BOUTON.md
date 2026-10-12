# 63 — Le flou qui restait, et un bouton pour chaque fonction

> Session du 12/10/2026 au soir. Il a dit : *« on a certains problèmes avec les bulles de texte
> et les membranes qui deviennent floues […] il y a des niveaux de zoom qui des fois buguent et
> restent ultra floutés, même pour du texte — ça crée des carrés qui ne souhaitent pas du tout se
> déflouter »* ; *« tester s'il y a bien un bouton pour absolument chaque fonctionnalité, sur PC
> ou téléphone »* ; *« regarde les performances »* ; et l'idée de publier la **2.0.3-beta.1**
> quand *« absolument TOUT sera parfait »*.

## 1. Ses réponses, mot pour mot ou presque

* **Les envois** : *« je valide tous les push publics sur le GitHub, mais sans activer la CI »* ;
  *« je t'autorise à utiliser la CI, mais de façon raisonnable, pas pour chaque push »* — la
  mémoire `jamais-de-push-sans-son-accord` le dit désormais (elle était citée par l'index et
  n'existait pas).
* **Liberté** : *« tu es absolument libre ; tu as mon PC portable à ta disposition, et mon
  GitHub »*.

## 2. NETTETE-1 — le flou qui ne partait pas (`c94cc60`)

**Le symptôme** se lisait dans le code avant toute mesure : trois chemins s'endormaient avant
d'être nets, et **aucune raison de réveil ne les rappelait**.

1. **La dernière image d'un geste est une image de mouvement.** À la molette, l'élan s'éteint
   dans l'image même qui montre le cran (NAV-4). Cette image rend les textes au palier dyadique le
   plus proche, que la carte graphique agrandit jusqu'à √2 : **à tous les zooms éloignés d'une
   puissance de deux, le texte restait flou** jusqu'au geste suivant — « des niveaux de zoom ».
2. **La voie processeur remonte sa finesse par moitiés** à l'arrêt (8 → 4 → 2 → 1), une image par
   moitié — et rien ne demandait ces images : la scène restait agrandie au plus proche, en blocs
   de deux à huit pixels. Les membranes et les textes de cette voie en étaient.
3. **La cascade reporte** les textures que le budget ne permet pas (CASCADE-2, 3) — les tuiles
   d'un texte vu de près (DE-PRES-1), les paliers neufs — et pose leur repli : le texte entier à
   un palier bien plus bas. Personne ne la rappelait : **des carrés flous restaient posés**.

**La correction** : une raison de réveil, *la netteté revient* (`Raison::Nettete`) — l'image
d'arrêt (une seule), la finesse jusqu'à un (une image par moitié), les textures reportées (sans
rien repeindre : `demander_une_image`, les couches du processeur sont justes). Chacune finit
d'elle-même ; **le repos reste à zéro image**. Un repli qui attend ne compte pas : il ne force
jamais une image, et sur une machine dont chaque image dépasse le plancher (le téléphone), il
garderait la boucle éveillée sans fin — c'est ma première version qui le faisait, je l'ai vu en
écrivant l'épreuve.

**La recherche confirme la forme** — Chromium fait de même : pendant un pincement, l'échelle de
rastérisation change **par puissances de deux** et le texte est étiré jusqu'à ×2
(`kMaxScaleRatioDuringPinch`), puis à la fin du geste il rastérise à l'échelle exacte, en
arrière-plan, et pose les tuiles au fur et à mesure ([notes sur le code de Chromium,
résolution pendant l'interaction](https://canary.grida.co/docs/wg/research/chromium/resolution-scaling-during-interaction),
[tuiles et rastérisation](https://canary.grida.co/docs/wg/research/chromium/tiling-and-rasterization),
[impl-side painting](https://www.chromium.org/developers/design-documents/impl-side-painting/)).
Le navigateur de Jolla a eu notre défaut exact : *« le texte reste pixelisé longtemps après un
pincement, jusqu'à ce qu'on fasse défiler »*
([forum Jolla](https://together.jolla.com/question/51999/browser-sometimes-takes-long-to-update-pixelated-text-after-zoom)).
**La seule différence qui reste** : Chromium rastérise ses tuiles sur un groupe de fils, Glucose
sur le fil qui dessine (§ 7).

Quatre épreuves, sept sabotages, sept chutes. **À son écran** : zoomer à la molette sur un texte,
s'arrêter entre deux crans — il doit devenir net aussitôt.

## 3. ACCES-1 — un bouton pour chaque fonction

**L'inventaire** (`docs/INVENTAIRE.md`, neuf, tenu à jour) a relu la bande, le rail, les trois
menus, la barre d'action, les panneaux et les raccourcis, au PC et au téléphone, contre les 279
fonctions de Tauri (fiche 03). **Ce qu'il a trouvé** :

* **au clavier seulement** — donc jamais au téléphone, ni à la souris seule : enregistrer,
  enregistrer sous, annuler et rétablir au bureau, tout cadrer, cadrer la sélection, **les
  signets** (aucun accès du tout au téléphone) ;
* **des boutons qui ne faisaient rien au téléphone** : Exporter, « Enregistrer l'image sous… »,
  « Importer un document… » — tous trois passaient par un sélecteur de fichiers qui n'existe pas
  sous Android, et rien ne s'ouvrait, en silence (§ 4) ;
* **Trans-domaines** ne fait rien et ne le dit pas — sa parole du 23/09 (« laisse le bouton ne
  rien faire ») : une question pour lui ;
* la carte d'accueil naît **choisie** : le premier clic droit d'un lancement ouvre le menu d'une
  sélection — à vérifier contre Tauri ;
* les **opérateurs des post-it** (ET, OU, MAIS, PARCE QUE) se dessinent, aucun geste ne les pose.

**Ce qui est fait** (PureRef range toute son interface dans son clic droit, chaque commande avec
son raccourci — Glucose en prend la règle) :

* **Annuler, Rétablir** en tête de tout menu du canevas, quand ils ont un objet ;
* le menu du vide gagne **Tout cadrer, Signets…, Enregistrer, Enregistrer sous…, Exporter…**
  (et « Ajouter des images… » son raccourci, `Ctrl+I`) ; celui d'une sélection, **Cadrer la
  sélection** ;
* **« Signets… »** (`interactions/signets.rs`, les signets y déménagent de `shortcuts.rs`) : une
  question dessinée — y voler d'un toucher, en poser un ici, et, les neuf pris, choisir lequel
  remplacer ;
* **au téléphone**, « Enregistrer » range un travail sans nom, « Enregistrer sous… » lui donne
  un autre nom (sans dossiers, c'est ce que le mot veut dire), « Importer un document… » liste
  les autres documents rangés ;
* **le menu se range en colonnes** quand l'écran est trop bas — la règle du rail : un groupe
  reste entier tant qu'il tient dans une colonne, un filet ne commence ni ne finit une colonne.
  Au doigt, un téléphone de cinq pouces ne tenait qu'une douzaine d'entrées ; le bas sortait de
  l'écran. Sur un écran qui le tient, le menu est identique au pixel près à celui d'avant.

Les entrées du menu sortent dans `ui/context_menu/entrees.rs` (le fichier passait 600 lignes :
le cliquet l'a dit). L'image témoin du menu a changé d'une entrée, **regardée** avant d'en
accepter l'empreinte. Dix épreuves, quinze sabotages — **un aveugle d'abord** : la hauteur
d'écran choisie faisait tomber la coupure entre deux groupes par hasard ; l'épreuve prend
désormais une hauteur où elle tomberait au milieu d'un groupe.

## 4. PARTAGE-2 — exporter au téléphone, par la feuille de partage

Au téléphone, il n'y a pas de sélecteur à qui demander « où ». Le téléphone fait comme Procreate
et Concepts : le fichier s'écrit dans `partages/` (le dossier privé de Glucose), et la **feuille
de partage d'Android** l'offre à l'application qu'on choisit — Fichiers, la galerie, Discord,
Drive. Côté Java, `MainActivity.partagerUnFichier` et un `FileProvider` (déjà dans
`androidx.core`, aucune dépendance nouvelle) qui n'ouvre **que** `glucose/partages/` ; côté Rust,
une porte (`plateforme::partager`) que `glucose-android` branche au lancement.

**Le lieu décide, et rien d'autre** : tout fichier qui vient de s'écrire dans `partages/` —
l'image enregistrée, l'export en texte, l'export en image **à la fin de son fil** — s'offre ; le
dossier se vide à chaque nouveau partage. Exporter demande d'abord le format (aucun nom de
fichier ne le dit par son extension). Quatre épreuves ; **jamais essayé sur un téléphone** : la
feuille de partage ne se prouve qu'à son écran.

## 5. Ce que la télémétrie a dit — et ce qu'elle ne savait pas dire

Les 34 sessions reçues (lues dans sa base D1, sans rien en retirer) :

| machine | geste | images | médiane | p99 le pire | pire |
|---|---|---:|---:|---:|---:|
| **son téléphone** (moto g54, 14 sessions) | déplacer la vue | 7 999 | **19,5 ms** | 75,7 | 75,7 |
| | zoomer | 4 650 | **15,7 ms** | 104,0 | 104,0 |
| | glisser un nœud | 5 021 | 14,6 ms | 836,7 | **885,1** |
| | repos (toasts, décodages) | 4 830 | 13,9 ms | 79,5 | 79,5 |
| **son PC** (RTX 5070, beta 2) | zoomer | 26 600 | 3,25 ms | 93,8 | 93,8 |
| | déplacer la vue | 26 422 | 3,03 ms | 12,5 | 12,5 |
| **un autre utilisateur** (Windows, `5280efb2`, né le 12/10) | déplacer la vue | 2 670 | 2,0 ms | 17,8 | 17,8 |
| | zoomer | 376 | 4,7 ms | 28,4 | 28,4 |

* **Le téléphone est cinq à six fois plus lent par image**, et sous le plancher de la charte
  presque tout le temps (93 % des images en déplaçant la vue au-dessus de 10 ms).
* **Un nouvel utilisateur** : une installation Windows inconnue, montée de la 2.0.1-beta.1 à la
  2.0.2-beta.2 **par la mise à jour automatique** le 12/10, des sessions de deux heures, fins
  propres, et une machine rapide. La bascule marche chez quelqu'un d'autre que lui.
* **Ce qu'elle ne savait pas dire** : *où* part le temps au téléphone. Une durée d'épisode ne
  désigne aucun coupable — d'où POSTES-1.

## 6. POSTES-1 — la boîte noire dit où part le temps

À la clôture d'un épisode **plus lent que le plancher** (10 ms), une ligne `poste` par étape du
dessin, pour **le plus petit ensemble de postes sans lesquels l'image ordinaire y tiendrait** :
du plus cher au moins cher, jusqu'à couvrir l'excès. Aucun nombre choisi ; deux pièges évités —
le serveur refuse une session de plus de 512 Ko (les plus lourdes en font 450), et **au
téléphone une session finit toujours tuée par Android** : un résumé écrit à la fermeture n'y
arriverait jamais, une ligne par épisode, si.

Ma première règle — couvrir la médiane entière — les prenait **presque tous** : les marques se
ferment l'une l'autre, donc les postes d'une image se somment à sa durée. Vu en écrivant
l'épreuve, avec mes propres nombres.

Les noms sont les 44 marques de mesure du code (`perf::stage`), une liste fermée du serveur
(`NOMS.poste`) qu'une épreuve lie au code : une marque ajoutée sans l'être là-bas ferait refuser
en silence chaque session lente. **Le serveur est redéployé et éprouvé en ligne** (une session
avec un poste reçue, un nom inventé refusé, l'essai effacé, la base relue : rien ne reste). La
page publique (`docs/TELEMETRIE.md`) et le `LISEZ-MOI.txt` du journal le disent. `chronique.rs`
passait ses 600 lignes : ce qu'on sait d'un geste sort dans `chronique/par_geste.rs`.

## 7. La performance du téléphone : ce qui est mesuré, ce qui suit

**Mesuré ici, hors écran, à la taille de son téléphone** (1080 × 2400, interface à 275 %) : la
couche du dessus — la chrome que le processeur peint et que la carte reçoit — touche **556
lignes sur 2 400** rail replié (2,4 Mo envoyés par image), **1 865** rail déplié (8 Mo), contre
297 sur 1 600 au PC. Elle repart **à chaque image**, même quand la chrome ne change pas. C'est un
gaspillage réel ; **ce n'est probablement pas tout le lag** — pendant un simple déplacement,
aucun texte ne se refait, et l'image coûte pourtant 19,5 ms.

**Les deux pistes, dans l'ordre où la mesure les départagera** (POSTES-1 dira la part de
chacune dès sa prochaine session au téléphone, par les postes `blit`, `effacer`, `relever`,
`textures`, `present`…) :

1. **CHROME-1 — la chrome posée par la carte**, comme les panneaux (PANNEAUX-1) : la bande, le
   rail, la minimap en textures refaites à leur changement seulement — plus rien à effacer, à
   relever ni à envoyer à chaque image.
2. **RASTER-1 — les textes rendus sur plusieurs fils**, comme Chromium (un groupe de fils de
   rastérisation, l'image n'attend jamais) : aujourd'hui une texture de texte se rend sur le fil
   qui dessine, dans le temps libre de l'image — au téléphone, « au moins une par image », et
   une carte vue de près met des dizaines d'images à redevenir nette. Il faut à chaque fil sa
   typographie (son cache de glyphes est un `RefCell`).

## 8. Les erreurs de la session — ce que j'ai cru et que la vérification a démenti

* **Un repli compté comme travail à finir** aurait gardé le téléphone éveillé sans fin au repos
  — vu en écrivant l'épreuve de la cascade.
* **« Couvrir la médiane »** (POSTES-1) les prenait presque tous — vu avec mes propres nombres.
* **Un sabotage aveugle** : la hauteur d'écran de l'épreuve des colonnes faisait tomber la
  coupure entre deux groupes, par hasard.
* **Mon menu élargi de 8 pixels** : la première mise en page en colonnes changeait le dessin
  d'une seule colonne ; repris avant de compiler, puis l'image témoin regardée.
* **Deux suites d'épreuves en même temps** sur le même exécutable : l'éditeur de liens a refusé
  (`LNK1104`), et j'ai d'abord cru une épreuve bloquée.
* **Le cliquet des 600 lignes après `cargo fmt`** — deux fois (`context_menu.rs`,
  `chronique.rs`) : on a extrait.
* **L'envoi « urgent »** (`58ad6ea`) a été pris pendant que le saboteur travaillait : vérifié,
  aucun fichier saboté n'y est entré ; mais il lui manquait de quoi compiler les épreuves, et sa
  CI l'a dit. Complété dans l'heure (`aabd7da`).
* **La répétition de la bascule surveillait `apercus/v2`**, que la version 3 des aperçus retire
  (fiche 62) : rouge sur GitHub seulement. Une épreuve lie désormais les scripts à la version.

## 9. Ce qui attend sa parole

1. **Trans-domaines** ne fait rien et ne le dit pas (sa parole du 23/09). Mon avis : qu'il dise
   « bientôt », comme Collaborer — sinon c'est le seul bouton qui ment encore.
2. **`Ctrl+F`** cadre tout ; dans Tauri, c'était la **recherche**. Quand la recherche viendra,
   lequel des deux garde `Ctrl+F` ?
3. **L'export PNG** : à 100 %, ou à la pleine résolution des originaux ? (fiche 62, toujours
   ouverte.)
4. **La 2.0.3-beta.1** : tout est prêt jusqu'au brouillon (§ 10) — son essai à l'écran d'abord,
   puis son clic « Publish release ».

## 10. À essayer à l'écran

`cargo run --release > sortie-63.txt 2>&1` au PC, et l'APK de son Bureau au téléphone :

* **Zoomer à la molette sur un texte**, s'arrêter entre deux crans : net aussitôt. De très près
  sur un long texte : les carrés flous disparaissent en un instant.
* **Le clic droit sur le vide** : Annuler, Rétablir, Tout cadrer, **Signets…** (en poser un,
  s'éloigner, y revenir), Enregistrer, Exporter.
* **Au téléphone** : deux touchers sur le vide, puis Exporter → un format → la feuille de
  partage (l'envoyer à soi-même, ou dans Fichiers) ; une image, appui long, « Enregistrer
  l'image sous… » → la galerie ou Fichiers.
* Ordonner (les tris par la couleur, les champs), les Presets, l'export PNG (fiche 62).

## 11. Le rangement, et les scripts sur toutes les machines

* **La racine du dépôt** ne garde que ses vrais fichiers : les dix-neuf `sortie-*.txt` de ses
  essais du 7 au 9/10 (déjà lus et consignés, fiches 52 à 59) sont retirés. Les captures que les
  épreuves de preuve écrivent dans `crates/glucose-desktop/target/` sont à leur place (Cargo, et
  ignorées par git).
* **Les scripts** : Windows (l'installeur NSIS, la répétition), Linux (AppImage, `.deb`, `.rpm`,
  NixOS), Android (`construire.sh`) — chacun éprouvé par la CI. Le script d'Android se
  contredisait sur la version de Gradle : corrigé. Sous Windows, Gradle veut `JAVA_HOME` et
  `ANDROID_HOME` **en chemins Windows** (`C:/…`), pas ceux de Git Bash (MÉTHODE § 5).
* **Le trou** : **aucune version Mac n'est publiée** — la CI compile les deux Mac, rien ne les
  empaquette, et la mise à jour ne sait pas encore remplacer une application Mac. C'est le seul
  système de sa liste qui manque ; un chantier à part (SUITE § 3).
* **La version de travail passe en 2.0.3-dev** (au-dessus de la 2.0.3-beta.1 qu'on prépare), et
  les notes de la publication sont réécrites (`outils/publication/notes.md`).
* **La mémoire** `jamais-de-push-sans-son-accord`, citée par l'index et absente du dossier, est
  écrite — de sa parole du jour.
