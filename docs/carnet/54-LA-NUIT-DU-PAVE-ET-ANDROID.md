# 54 — La similitude des doigts, la boîte noire qui voyage, et Glucose qui compile pour Android

> Session du 07 au 08/10/2026. Son message d'ouverture : *« une version ultra stable
> avec tout le mode Ctrl+Maj+A […] compatible tablette, Linux, Windows et Mac ; un build complet
> pour tous les Linux, tous les Mac, et SURTOUT Android ; et une télémétrie de tous les
> utilisateurs »*. Puis, pendant le travail : *« des fois le pavé tactile ne veut plus du tout
> fonctionner, coupure de dix secondes — exactement présent sur Blender »* et *« lorsqu'on déplace
> un groupe, le carré de déplacement ne suit pas le groupe »*. Ses réponses : l'envoi **sans CI**,
> un **Xiaomi 9**, **Cloudflare Workers**.

---

## 1. L'envoi

`a32e074`, `efab62f`, `35511c8` et une note dans ETAT (`d17e838`, `[skip ci]`) sont partis à sa
demande, **sans CI** : vérifié sur GitHub, aucune exécution. Tout ce qui est parti depuis `20deb6f`
n'a donc toujours pas été vu sous Linux ni sous Mac.

## 2. Le cadre du groupe qui restait sur place (GROUPE-1)

Le cadre du groupe était gardé tant que la **version** du document ne changeait pas — or elle
n'avance qu'au relâchement d'un geste. Il se garde désormais aussi sur le geste en cours et ce
qu'il a déjà écrit (le numéro du geste et le nombre de ses éditions, la convention de GESTE-1) :
il suit un glisser, une mise à l'échelle, une rotation. **Trouvé par le sabotage** : le premier
essai de l'épreuve ne demandait le cadre qu'à la fin du geste, et un sabotage passait ; elle le
demande désormais à chaque pas, comme le dessin à chaque image.

## 3. Le déplacement à deux doigts : la similitude entière

**Sa chronique** du 07/10 au soir : dix-neuf déplacements pris pour des pincements, le plus petit
sur un écart d'échelle de **0,13 %**. La lecture de la fiche 53 était celle de Chromium (vérifiée
dans son code : un geste dont l'échelle bouge devient un pincement jusqu'à la fin, translation
jetée — et sa tolérance, un cent-millième relatif, est cent fois plus large que celle qu'avait
Glucose) et de Blender. Elle bloquait le déplacement jusqu'à ce que les doigts se lèvent.

**La recherche** : Flutter (`direct_manipulation.cc`) transmet le déplacement et l'échelle
**ensemble**, à chaque mise à jour, et cale son *viewport* sur la taille de la fenêtre.

**Ce qui est fait** : d'une image à la suivante, le système fait subir au contenu une
similitude, `q ↦ r · q + b` ; Glucose l'applique entière, dite comme un déplacement
`p = (b − (1 − r) · a) / r` puis un zoom `r` autour du curseur `a` — **exacte quelle que soit
l'ancre**. Plus de classement, plus de blocage, plus de seuil, plus de bascule. Le *viewport*
prend la taille de la fenêtre et la suit (la constante 1000 × 1000 disparaît). Le « décalage
absurde » de Blender s'explique : c'est le déplacement qui garde fixe le point du zoom, compté
deux fois par qui zoome aussi autour du curseur.

**Vérifié à son écran** (session de 900 s) : **le point que Windows garde fixe est à 2 px du
curseur en médiane** (10 au p90, sur 1 719 images) — Windows zoome bien sous le curseur, comme le
calcul le suppose. **Son ressenti sur le biais n'est pas encore rapporté.**

## 4. La coupure de 7 à 10 secondes

Rien ne la trace : la boîte noire ne savait pas distinguer « il n'a pas touché le pavé » de « le
pavé n'a rien transmis ». Qu'elle touche **aussi Blender**, qui prend le pavé de la même façon,
désigne Windows ou le pilote autant que la méthode. Le seul rapport de Blender au symptôme
semblable (#112173) concerne le Mac et n'a jamais été résolu. **La boîte noire note désormais
chaque contact confié au système, chaque changement d'état, chaque début et fin de geste**
(`{"type":"pave", "quoi": …}`), à l'heure où ils ont eu lieu. Sa session suivante : 112 contacts,
112 gestes ouverts, 112 fermés, aucun refus — et pas de coupure. **La prochaine dira à quel
étage elle s'arrête.**

## 5. Sa session sur batterie : une première

La session de 900 s tournait **débranchée** (47 % → 33 %) ; toutes les précédentes étaient sur
secteur. Le tempo est monté à 7 ou 8 balayages (30 images par seconde) sur 11 % des images en
mouvement, le poste « effacer » jusqu'à 27 ms. Un déplacement pur fait exactement le même calcul
qu'avant : **probablement le bridage de la RTX sur batterie, à confirmer** par une session
branchée et une débranchée. La charte ne l'excuse pas : sur batterie aussi, 100 images par
seconde ou la pixelisation. C'est la première mesure de Glucose bridé — chantier 6.

## 6. La boîte noire qui voyage (chantier 4)

**Sa décision** : Cloudflare Workers. GitHub ne convenait pas (un jeton glissé dans l'application
serait lisible par tous, et GitHub révoque un jeton publié).

* **Le serveur** (`outils/telemetrie/`) : un Worker et sa base D1, gratuits, dans son compte. Deux
  portes — déposer une session, effacer une installation — et **aucune pour lire** (on lit par
  `wrangler d1 execute`, depuis son compte). Aucune adresse IP : le Worker n'en lit aucune, la base
  n'a pas de colonne pour elle, ses journaux de requêtes sont éteints. Il n'accepte **que** les
  lignes de la boîte noire, chaque nom pris dans une **liste fermée** — un motif aurait laissé
  passer « mon document secret » (je l'avais d'abord écrit ainsi, et l'ai vu en relisant). Une
  épreuve de Glucose tient les deux listes égales.
* **Glucose** (`telemetrie.rs`) : la question une fois, en français simple ; l'accord et un
  identifiant tiré au hasard ; au lancement, sur un fil à part, chaque session close part une
  fois, ses seules lignes entières, sous une empreinte qui ne dit rien de son nom ; au menu,
  « Journal technique… » et « Voir ce qui part » ; « non » après « oui » efface tout sur le
  serveur et change d'identifiant. La porte réseau sait envoyer (`POST`, `DELETE`), éprouvée
  contre un serveur local.
* **La page publique** : `docs/TELEMETRIE.md`, pour ses testeurs.
* **Attend** : son compte Cloudflare, puis l'adresse (`telemetrie::ADRESSE`) — tant qu'elle
  manque, rien ne se demande ni ne part, et le menu ne montre rien.

## 7. Android : Glucose compile pour les téléphones

**Installé dans son dossier** (`~/Android/Sdk`, sans droits d'administrateur) : les outils, la
plateforme 36, le NDK 29 (empreinte vérifiée), `cargo-ndk`, les cibles Rust. Le premier
téléchargement du NDK s'est figé pendant une coupure du réseau ; repris par `curl`.

**Mesuré d'abord** : sur 152 caisses, **deux seules** refusaient Android — `rfd` et `arboard`.

* **DIAL-4** : `rfd` ne vit plus que dans `dialogue.rs` (un sélecteur `Fichier` à Glucose,
  `oui_non_ou_annuler`), `arboard` que dans le presse-papiers (un type `Pixels` à Glucose). Le
  **cliquet 12** tient les deux portes. Sous Android : aucun dialogue encore, le presse-papiers de
  Glucose seul.
* **`demarrage::lancer`** : toute la séquence de lancement, sortie de `main.rs`.
* **`crates/glucose-android`** : `android_main`, le dossier privé de l'application, et la sortie de
  Glucose vers le journal d'Android (`liblog`, par un tube).
* **`android/`** : Gradle 9.8.1 et le plugin Android **9.4.1** (le résumé d'un moteur de recherche
  disait 8.13 : vérifié au dépôt Maven de Google, il avait tort), `games-activity` **4.4.0**
  exactement. **Plancher : Android 5.0 (API 21)**, celui de Rust — vérifié, les deux processeurs
  se construisent. Note `decisions/08`.
* **Le toucher** (`interactions::toucher`) : Glucose ignorait les doigts. Un doigt agit comme la
  souris, sauf sur le vide où il déplace le canevas avec son élan ; deux doigts font la similitude
  qu'ils décrivent, par le code du pavé. **Trouvé par l'épreuve** : deux doigts qui bougent dans
  la même image donnaient deux similitudes que l'élan additionnait — le point entre les doigts
  dérivait de 12,5 unités sur un pincement du double, exactement l'écart que le calcul prédit.
  Toutes les similitudes d'une image se composent désormais exactement en une seule
  (`r = r₂r₁`, `b = r₂b₁ + b₂`), pavé compris.
* **Sous Windows**, les doigts ne passent pas encore par là : `winit` les livre **et** Windows
  simule la souris du premier — tout arriverait deux fois. « Compatible tablette » sous Windows
  demande d'abord d'écarter la souris simulée.
* **La CI** construit l'APK et le dépose (tâche `apk`, la dixième) : le Gradle d'ici arrivait à
  5 Ko/s.

## 8. Prouvé, et pas prouvé

* **Prouvé** : 2 108 épreuves, clippy strict sous Windows **et pour Android** ; dans cette session,
  quarante-cinq gardes sabotées, quarante-cinq chutes (trois passaient d'abord : le cadre du
  groupe, la fin de geste du pavé, la traduction de la fermeture — toutes gardées depuis) ; le point fixe du
  pavé à son écran ; l'envoi par la vraie porte réseau contre un serveur local.
* **Pas prouvé** : son ressenti sur le déplacement en biais et le cadre du groupe ; la cause de la
  coupure ; la cause du ralentissement sur batterie ; **rien n'a tourné sur un téléphone** ; le
  serveur n'est pas déployé ; Linux et Mac (la CI, toujours pas lancée depuis `20deb6f`).

## 9. Ce qui attend sa parole

1. **L'envoi, et cette fois la CI** : c'est elle qui construira l'APK.
2. **Son compte Cloudflare** (une adresse et un mot de passe) ; je déploie le reste.
3. Le **Xiaomi 9** : Mi 9 (2019, Snapdragon) ou Redmi 9 (2020, MediaTek) ? Et les autres
   téléphones de ses testeurs.
4. Une **clé de signature** Android à garder pour toujours (un APK ne se met à jour que signé de
   la même clé) — et sa copie hors du PC, comme celle de Tauri.

---

**Retour** : [`00-INDEX.md`](00-INDEX.md)
