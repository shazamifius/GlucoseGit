# 71 — Sa direction, l'inventaire, et le téléphone sans câble

> Session du 15/10/2026, après la fiche 70. Elle commence, à sa demande, par **un inventaire**
> (les questions qui attendaient sa parole, ce qui reste à faire) — on n'avait jamais eu le temps
> de parler. Ses réponses donnent, pour la première fois par écrit, **la direction de Glucose** :
> [`VISION.md`](../VISION.md).

## 1. Ce qui est vérifié en ouvrant

* Le téléphone n'est pas branché — et **il a perdu son câble**.
* **Il a lancé Glucose le 14/10 de 23 h 00 à 23 h 18** (1 059 s), sur une build construite à
  23 h 00 — après `5bb82ec` (schéma 4), avant `4827118` (schéma 5) : très probablement le schéma 4,
  à vérifier sur une copie de ses documents avant de toucher au format. Aucune fiche ne la dit. Sa
  chronique : 345 images sur 13 886 au-dessus de 10 ms (pire 24,1), un tressaut à 91 px au p99
  quand la durée d'une image change (×11,4), une latence de 21,9 ms au p99.
* `renderer/math.rs` : LF dans le dépôt (`i/lf`), CRLF seulement dans la copie de travail —
  sans conséquence.

## 2. Ses réponses, mot pour mot ou presque

* **Le téléphone** : *« j'ai perdu complètement mon câble »* ; un APK envoyé par plusieurs moyens et
  téléchargé au téléphone, même à la fibre, reste **en téléchargement infini**. Sa question :
  pourquoi, et comment le faire accepter gratuitement par Play Protect. Son plan : l'APK dans un
  zip, envoyé par Drive ou Discord, essayé, *« et prier pour que tout se retrouve sur
  Cloudflare »*.
* **Les essais** : *« si j'ai absolument tout testé, normalement 100 % des fonctionnalités sont
  validées »*.
* **« Créer un preset custom »** : *« on n'est pas du tout d'accord »* — un preset est l'interface
  entière (disposition, comportement, couleurs, raccourcis, fenêtres) : un preset de Glucose
  jamais vu, un PureRef, un Figma, un Miro. Puis toute sa vision : `VISION.md`.
* **Trans-domaines** : de la recherche, son raisonnement sur « domaine » n'est pas fini.
* **L'export PNG, et tout ce qui a deux réponses légitimes** : *« d'accord avec le système qui
  permettrait à l'utilisateur de TOUJOURS CHOISIR »*.
* **Le SVG** : *« super utile »*.
* **Les planches** : claires ou sombres, au choix de l'utilisateur ; **les champs** (dialogue,
  action, notes) sous chaque case : oui ; **« 2.3 »** : non, mais pouvoir choisir ; **le numéro**
  dans la bande ou dans le coin : les deux, au choix ; **le panneau Storyboard** pleine hauteur :
  ne gêne pas.
* **Le dossier** : *« double-clic + zoom, comme Tauri »* ; **le design des dossiers est à
  retravailler**.
* **Le dessin de la sélection** : E pour l'instant ; l'ergonomie, une grosse session quand toutes
  les fonctionnalités seront là. **Le défilement au bord** : plus tard.
* **Le pavé** (le déplacement à deux doigts) : *« franchement, c'est tellement mieux qu'avant »*.
* **Le mode référence** (opacité, clics qui traversent) : pas pour l'instant.
* La liste des capacités de PureRef à refaire : à venir.

## 3. Le téléphone sans câble

Sa question : pourquoi le téléchargement de l'APK reste infini, et comment passer Play Protect
gratuitement. Ce qui est établi (sources : le blog des développeurs Android, mars 2026 ; Engadget ;
Android Authority ; la page d'adb) :

* **Le débogage sans fil** (Android 11 et après) remplace le câble : `adb pair <adresse:port>`
  avec le code à six chiffres, puis `adb connect <adresse:port>` (un autre port). Le script
  `outils/android/lire_le_telephone.sh` marche tel quel : adb ne distingue pas le câble du Wi-Fi.
  Il permet de lire le téléphone **avant** d'ouvrir Glucose, d'installer l'APK sans rien
  télécharger, et de relever `dumpsys meminfo` en direct — ce que son plan du zip ne permet pas.
* **Le téléchargement infini** : trois causes probables, aucune prouvée sans le téléphone —
  Discord refuse plus de 10 Mo sans abonnement (l'APK en pèse 18) ; l'application Drive intercepte
  les APK ; Chrome fait analyser chaque APK et l'analyse peut rester en attente.
* **Play Protect** : en France, rien ne bloque encore — la vérification des développeurs commence
  le 30/09/2026 au Brésil, en Indonésie, à Singapour et en Thaïlande, le reste du monde en 2027 ;
  aujourd'hui, « Plus de détails » puis « Installer quand même ». Pour plus tard : un compte
  **gratuit** « distribution limitée » (sans frais ni pièce d'identité, un profil de paiement à son
  nom, **20 appareils**) ; le compte ordinaire coûte 25 $ ; `adb` reste permis.

## 4. La suite entière, rejouée — et trois témoins

Pour la première fois depuis la fiche 70 : **2 526 épreuves passent, 3 tombent** — les trois
images témoins (la scène, la sélection, le menu). Produites avant (`9c6a742`, un arbre de travail
à part) et après, comparées pixel à pixel, regardées agrandies : **tout vient du dossier corrigé**
(le titre à sa ligne de base, le chiffre du badge dans sa pastille, « Double-clic pour ouvrir »
dans un dossier vide) — 1 633, 1 608 et 1 324 pixels, tous dans le bandeau du dossier. Le
changement est voulu : les empreintes sont mises à jour, chacune avec sa ligne d'histoire.

## 5. Les 80 secondes — mesurées, et parties

Chronométrée étape par étape, l'épreuve de superposition passait **28,3 s sur 28,9 dans
`resvg`** ; la comparaison, que la fiche 70 avait accélérée sans effet, prend 0,1 s. La cause
était dans **le SVG lui-même** : chaque lueur déclarait une région de filtre de ±100 000 unités
(et son commentaire annonçait « trois écarts-types »), et tout lecteur floutait un carré de
200 000 de côté par lueur — Inkscape ou un navigateur aussi.

La région est désormais **la boîte de la lueur élargie de sa portée** (`portee_du_flou`, HALO-2) :
la distance au-delà de laquelle elle ne change plus un niveau, déjà calculée pour le canevas.
Aucune constante. **28,3 s → 62 ms**, et la superposition est identique au centième (forme 26,62,
teinte 6,31, avec et sans la borne). Une épreuve neuve la garde — remettre ±100 000 laissait
toutes les autres vertes, seulement plus lentes : `test_p5_chaque_flou_tient_dans_le_cadre_de_l_export`.

## 6. Les sabotages de P5 — et ce que la superposition ne voyait pas

`outils/sabotages/p5.json` : chaque appel du SVG retiré, l'ordre des lueurs, le fond, le premier
rang, les membranes, les textes, les dossiers, l'échelle et l'avance d'un glyphe, la ligne de
base, le détourage, l'opacité d'un filet, `rayon_tenu`, le rectangle net, la région des flous
(immense ou sans portée), BORNES-2, le badge et le texte d'un dossier vide, le titre et le
chiffre du dossier. Premier passage : **20 chutes, 9 aveugles**. Deux venaient de **ma commande**
(BORNES-2 : son épreuve vit dans `tests/store_suite.rs`, je ne lançais que `--lib`). Les sept
autres disaient une vraie limite : la superposition borne le gros, et dilue le petit ou le peu
contrasté. Sept épreuves ciblées, jugées par un autre que nous quand c'est possible :

* les textes peu contrastés (le nom d'une zone, bleu sur son bandeau bleu ; sa phrase grise ; la
  description d'une case ; « Double-clic pour ouvrir ») et l'absence de badge d'un dossier vide ;
* la taille d'un glyphe, mesurée contre la boîte que **`fontdue`** rastérise pour le canevas ;
* le filet d'une fraction à l'encre de sa formule ; un coin borné à la moitié du côté ;
* **le rectangle net, jugé par `resvg` au pixel près — et il tombait sur le code intact** : le
  canevas cale un rectangle en *arrondissant* (`on_pixel_grid`), le SVG *tronquait* ses bords
  (son commentaire promettait « les pixels mêmes que `fill_crisp` »). Chaque zone et chaque case
  du SVG étaient décalées d'un pixel. Le SVG appelle désormais la règle du canevas.

Rejoués : **30 sabotages, 30 chutes**.

## 7. Le SVG d'un vrai document

Une copie de son document « UwU » (`Desktop\projet horreur`, 5 Mo ; l'original n'est pas touché),
exportée par le chemin même du bouton : un SVG de 6,9 Mo (les photos y portent leurs octets
d'origine) que `resvg` rend en **0,5 s**, superposé au PNG de 7 710 × 2 260 pixels : 0,4 % des
pixels s'écartent de plus de 32 niveaux, 1 461 de plus de 96 — tous dans le détail fin des photos
(deux rééchantillonnages différents), invisibles côte à côte. Le texte, les lueurs, les flèches,
les nœuds se superposent.

## 8. Mood board, Trans-domaines

* **Le panneau « Presets » devient « Mood board »** (sa parole) : la bande, le rail, le titre, les
  légendes, « + Créer un mood board » (grisé), « Par mood board » dans Ordonner. La bande mesure
  sa densité : elle garde tous ses libellés dès **1 626 pixels** (1 594 avant) ; son écran (2 560 à
  150 %) en a 1 707 — l'épreuve de la bande porte désormais sur lui. Les trois témoins : la bande
  seule, 2 764 pixels, regardés.
* **Trans-domaines dit « bientôt — en recherche »**, comme Collaborer. Sa parole du 23/09 disait
  « laisse le bouton ne rien faire » ; je lui ai annoncé le changement le 15/10 (un bouton ne fait
  jamais semblant, R1) sans objection — **à lui dire, réversible**.
* 2 sabotages, 2 chutes (`outils/sabotages/mood-board.json`).

## 9. La remise en forme des documents

* **[VISION](../VISION.md)** (neuf) : sa direction. **[PUREREF](../PUREREF.md)** (neuf) : sa liste des
  fonctions de PureRef, chacune en face de ce que Glucose fait — la cible du preset PureRef. En
  l'écrivant, trouvé : le tri par nom et par date, la ligne et la colonne existent dans le noyau
  et **aucun bouton ne les montre**.
* **[ÉTAT](../ETAT.md)** tient en une page (l'histoire reste au carnet) ; **[SUITE](../SUITE.md)** dit
  le programme, puis **une seule liste de ce qui attend son écran** — pour sa séance d'essais.
* `decisions/13` (ttf-parser, resvg) ; METHODE : cinq leçons, et le téléphone sans câble.
