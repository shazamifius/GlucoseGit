# 31 — La longue session lue, et la mémoire par étages

> **Rôle de ce document.** L'utilisateur a joué la longue session que la fiche 30 § 9 demandait
> (332 s, 8 765 images, 243 nœuds dont 45 épingles rapatriées) et a donné une orientation
> nouvelle : *« utiliser et la VRAM et la RAM et le SSD, et dès qu'une application utilise trop
> la RAM ou la VRAM, on switch ; toujours prio la VRAM, ensuite la RAM, ensuite le SSD ; et que
> le logiciel s'adapte en temps réel tout le temps »*. Cette fiche dit ce que sa chronique a
> montré, ce qui en a été corrigé, la première brique de la mémoire par étages, et la suite.
>
> **Date** : 2026-09-23, nuit · commits `42abdca` à `cbf74d4`, et celui de cette fiche.
> **État vérifié** : `cargo test --workspace` exit 0, **1 523 tests verts**, clippy strict à
> zéro, douze cliquets — le plafond des `fill_rect` bruts rabaissé de 10 à 9.
>
> Sa chronique est copiée (`chronique-longue-1.txt` dans le scratchpad de la session). Il n'a
> pas encore dit ce qu'il a **vu** : le texte de près, les vagues, le repos restent à juger.

---

## 1. Ce que sa chronique a dit

**La provenance marche, pour la première fois** (fiche 30 § 4). D'où viennent ses 8 765 images :
un message à l'écran 56,9 %, la vue qui glisse encore 47,7 %, la main 37,3 %, un décodage
2,4 %, un dépôt 0,9 %, **le système 0,4 %** — une image peut en porter plusieurs. Les
« 14,7 images par seconde au repos » sont donc des **animations** — les messages de ses 45 dépôts
qui s'effacent, l'élan après un pincement —, et ce que personne n'a demandé tient en une
trentaine d'images. La ligne de la veille (« zéro est la seule bonne réponse ») compte une
animation visible comme un gaspillage : elle est trop sévère, et la prochaine retouche de
l'instrument est de ventiler le repos par provenance.

**Un gel de 2,46 s à la 256ᵉ seconde**, entièrement dans « écouter la main » : **un seul**
événement de la main a bloqué la boucle. `Ctrl+B` est innocent — mesuré sur ses 45 originaux,
la détection coûte 18 ms en tout. Le suspect est une fenêtre de Windows ouverte depuis Glucose
(choix d'images, enregistrer sous) : elles bloquent la boucle tant qu'elles sont ouvertes.
**Question posée à l'utilisateur** ; si c'est cela, la suite est de le dire dans la chronique
(« dans une fenêtre de Windows ») au lieu d'un gel, et d'ouvrir ces fenêtres sans bloquer.

**Le tempo tombait à cinq balayages** — 48 images par seconde — sur 20 % des images en
mouvement, et **toutes** les images les plus lentes (15 à 28 ms) accusaient le poste
`ornements`, avec 243 nœuds et une grande sélection. § 2.

La mémoire de travail : 1 550 Mo à la fin. La mémoire graphique, personne ne la mesurait. § 4.

---

## 2. ORNEMENTS-2 — 243 photos sélectionnées : 21,6 ms → 0,48 ms

`bench_ornements` reproduit le terrain sur son écran (2160 × 1350) : **21,55 ms** pour 243
photos sélectionnées, 90 µs par photo — dix-sept appels au rastériseur, un cadre et huit
poignées remplies puis bordées. Quatre marches, chacune mesurée :

| étape | 243 photos |
|---|---:|
| au départ | 21,55 ms |
| cadres réunis en un tracé, poignées nettes | 9,62 ms |
| un cadre droit en quatre filets sur la grille | 3,15 ms |
| `fill_crisp` écrit ses pixels lui-même | **0,48 ms** |

* **Les poignées** se posent sur la grille (SCALE-3, dont la doc les nommait déjà) : un carré de
  liseré, un carré de fond dedans.
* **Un cadre droit** est quatre filets, de l'épaisseur entière la plus proche de 1,25 px — un
  pixel, net. Un cadre penché garde le contour anti-crénelé, réuni avec les autres en un tracé.
* **`fill_crisp`** n'appelle plus `tiny-skia` : il écrit par la règle de sa voie huit bits,
  `s + (d·(255 − a) + 255) >> 8`. L'épreuve le compare à `fill_rect` sur 2 000 cas tirés au
  hasard, au bit près — et a trouvé **un défaut de `tiny-skia`** : un rectangle qui déborde à
  gauche y peint une colonne de trop (de −7 à 9, les colonnes 0 à 9). La voie directe ne l'a pas.
* Toute l'interface passe par `fill_crisp` : la scène témoin sans sélection **n'a pas bougé d'un
  pixel**. Les deux scènes sélectionnées changent de 4 491 pixels, regardés au zoom.

---

## 3. Une épreuve qui tombait au hasard

`test_pendant_le_vol_on_est_encore_dans_le_tableau_parent` demandait une image « au tout début
du vol » par une horloge réelle, et exigeait que la caméra ait bougé : sur une machine rapide,
aucune microseconde ne s'était écoulée. `Animator::tick` lit l'horloge et appelle
`avancer(store, écoulé)` ; l'épreuve appelle `avancer` à une milliseconde.

---

## 4. VRAM-1 — la première brique de la mémoire par étages

* **La sonde** (`plateforme::graphique::Sonde`) lit, sous Windows, le **budget** que le
  gestionnaire de mémoire vidéo accorde à ce processus et ce qu'il y occupe
  (`QueryVideoMemoryInfo`). Il vaut pour Vulkan comme pour le reste — c'est le système qui le
  tient —, et il **rétrécit** quand une autre application réclame la carte : le signal de
  l'adaptation, observé et non deviné. La carte se retrouve par ses identifiants matériels.
  Ailleurs, la sonde ne sait rien et le dit ; le cache garde alors ce que l'écran demande.
* **Le cache de la carte** : ce qui quitte l'écran n'est plus oublié à la fin de l'image. Les
  textures les plus récemment vues se gardent tant qu'elles tiennent dans **la moitié de ce que
  le budget laisserait sans le cache** — la règle de la mémoire vive (ADAPT-1). Revenir sur une
  photo gardée ne la téléverse pas. Quand le budget baisse, le cache rend les plus anciennes.
* **La chronique** dit la mémoire graphique : à la fin, au pire, ce que le cache a gardé, et le
  budget accordé au plus bas.

Non vérifié à l'écran. Et une propriété à surveiller : sur une carte **intégrée**, la VRAM est de
la RAM — le cache de textures et le magasin d'images prennent alors chacun la moitié de ce qui
reste, et convergent ; c'est la même règle des deux côtés, mais personne ne l'a observé.

---

## 5. La mémoire par étages — ce qui existe, et la suite

| étage | ce qu'il porte | sa borne | état |
|---|---|---|---|
| **VRAM** | les textures posées, et celles vues récemment | la moitié de ce que le budget du système laisse | **VRAM-1** |
| **RAM** | les pyramides décodées | la moitié de la mémoire disponible (ADAPT-1) | existait |
| **SSD** | les fichiers sources ; l'atelier les décode en parallèle | le disque | existait |

Ce que la mesure a déjà démenti : **décoder un JPEG directement réduit** (mise à l'échelle DCT,
`jpeg-decoder`) ne coûte que 20 % de moins sur ses 44 JPEG — 494 ms au huitième contre 616 ms
en entier. C'est le déchiffrement du fichier qui domine ; ce n'est pas le levier de l'étage SSD,
et il ne justifie pas une dépendance.

La suite, dans l'ordre :

1. **La priorité VRAM dans l'éviction de la RAM** : quand la RAM se tend, rendre d'abord les
   pyramides dont la carte détient encore la texture — elles restent affichables.
2. **Quand le budget VRAM ne suffit plus à l'écran** : envoyer des niveaux plus petits
   (pixelisation assumée, invisible si elle est bien faite), puis — ce que la charte demande
   depuis le 17/09 — basculer la pose sur la voie processeur si la carte est prise.
3. **La chronique par étage** : combien d'images vivent dans chaque étage, et combien ont
   descendu ou remonté pendant la session.
4. **Un cache de niveaux décodés sur le SSD** — à mesurer avant : la lecture brute d'un niveau
   contre le décodage de son JPEG, et la place disque que la charte n'autorise pas à gaspiller.

---

## 6. Ce qu'il faut lui demander

* **Ce qu'il a vu** : le texte de près, les images qui arrivent, le repos.
* **Ce qu'il faisait vers 4 min 16 s** (le gel de 2,46 s).
* **Le fichier de la gravure** (fiche 30 § 6).

---

**Retour** : [`00-INDEX.md`](00-INDEX.md)
