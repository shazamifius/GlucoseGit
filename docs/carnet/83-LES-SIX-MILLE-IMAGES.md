# 83 — Les six mille images : la mort du téléphone, reproduite et réparée

> Session du 25/10/2026, suite de la fiche 82 (partie 1, étape 1 : « ne rien perdre »). Le 15/10,
> sur deux téléphones, un tableau de plus de 6 000 images s'est fermé seul ; sa cause était
> inconnue depuis dix jours — son téléphone, encore en beta 3, n'a rien envoyé depuis, et il a
> perdu son câble. La session s'est donné un téléphone à elle.

## 1. Un téléphone virtuel, sur ce PC

* L'émulateur d'Android (`emulator` 37.2) et une image **Android 14** x86_64 (celle de son moto
  g54), installés dans le SDK ; un appareil de **4 Go**, 1080 × 2400 ; l'accélération de Windows
  (WHPX) est disponible : il tourne à vitesse normale, **sans fenêtre** (`-no-window`), piloté
  par `adb` (`input tap`, `screencap`, `logcat`, `dumpsys meminfo`, `run-as`). Vulkan y passe par
  la carte graphique du PC (gfxstream).
* **L'APK du téléphone virtuel** : `GLUCOSE_PROCESSEURS=x86_64 bash outils/android/construire.sh`
  (la variable est neuve ; par défaut, rien ne change, et le script retire de `jniLibs` les
  processeurs qu'on ne demande pas — une bibliothèque x86_64 ne partira jamais dans l'APK
  publié).
* **Le grand tableau** : `examples/fabriquer_un_grand_tableau` — des JPEG fabriqués (du bruit
  dedans, pour peser comme des photos), posés par **le vrai dépôt** par lots de cent, écrits par
  **le vrai scribe**. 6 000 images de 600 × 900 : un document de 1,7 Go en deux minutes.
  Déposé comme **brouillon** dans le Glucose du téléphone virtuel (`run-as … cp`), il s'est
  rouvert de lui-même au lancement — la reprise marche.
* La question du journal technique a reçu **Non** : le téléphone virtuel n'envoie rien, la
  télémétrie reste la sienne.

## 2. La mort, reproduite

Le tableau ouvert, Glucose tenait à 276 Mo. **« Tout cadrer »** (le menu au doigt) : en cinq
secondes, le processus monte de 0,9 à 3,1 Go ; le tueur d'Android (`lowmemorykiller`) ferme le
clavier, le lanceur, les services de Google, puis **Glucose, au premier plan** : *« app died, no
saved state »*. Le relancement a rouvert le brouillon, et la boîte noire a dit ce que son
téléphone aurait dit en beta 4 : *« tuée par Android faute de mémoire (1 440 Mo occupés) »*
(SORTIE-1).

## 3. L'autopsie

* **Le journal de la session tuée** : la mémoire du processus grimpe (891 → 2 936 Mo), et
  **aucun** compte de Glucose ne bouge — images tenues 0 à 3 Mo, décodages en vol 5 à 15 Mo,
  carte 83 Mo. 2,7 Go comptés nulle part.
* **`dumpsys meminfo`, toutes les demi-secondes** : le tas natif reste vers 150 Mo ; tout ce qui
  monte est **« Unknown »** — des projections anonymes, privées et sales. Glucose n'en fait qu'à
  un endroit : les pages des pyramides (`plateforme::offre`, pour pouvoir les offrir au système).
* **Le code** : un niveau que l'écran ne demande plus **part à l'offre** — ses octets sortent de
  la pyramide (`sortir`, l'état « en chemin ») et entrent dans la file des ouvriers ; l'écriture
  d'un aperçu y emporte aussi ses pixels. Or l'ordre des files était promesses → reprises →
  **décodages → offres** : derrière six mille décodages, aucune offre ne passait, et chaque image
  décodée laissait ses trois mégaoctets dans la file. Et le relevé ne comptait que les niveaux
  **tenus** : ce qui était « en chemin » ne se voyait pas.

## 4. La réparation — MEMOIRE-4

* **Ce qui rend la mémoire passe devant ce qui en prend** (`atelier::Files::prochain`) :
  promesses, reprises (l'écran attend), **offres et aperçus**, décodages. L'ancien commentaire
  disait de l'offre « rien ne l'attend » : la mémoire l'attendait. Ce qui attend d'être rendu est
  désormais borné par le nombre d'ouvriers, quelle que soit la taille du tableau.
* **Le compte voit ce qui est en chemin** (`Magasin::octets_en_memoire`, lu par `images_mo`) :
  un niveau chez un ouvrier est encore en mémoire. Le `LISEZ-MOI` du journal le dit.

## 5. Prouvé

* **En réel, sur le téléphone virtuel** : le même brouillon, le même « Tout cadrer ». La mémoire
  « sale » reste vers 20 à 50 Mo ; ce qui monte est de la mémoire **propre** — les niveaux
  offerts, que le noyau jette sans rien écrire — et Android la compte disponible (2,4 Go libres
  sur 4). Glucose **survit**, deux minutes d'observation, aucun processus tué ; les **6 000
  images sont dessinées** (capture regardée : les soixante cascades, en couleurs). **À l'envers**,
  la version d'avant, mêmes gestes : morte à 3,1 Go.
* **Les épreuves** : l'ordre des files (une reprise, puis une offre et un aperçu, avant trente
  décodages) ; un niveau en chemin reste compté. **Sabotages 2 sur 2** (`memoire-4.json`).

## 6. Ce qui reste

* **Son téléphone** : la beta 6 n'a pas MEMOIRE-4 — la prochaine version l'aura. P0 se ferme
  quand son téléphone l'aura vécu (un grand tableau, « Tout cadrer »), et que ses lignes `sortie`
  ne disent plus de mort par la mémoire.
* **« Non rouvert »** : sur le téléphone virtuel, la reprise a rouvert le brouillon après chaque
  mort. Sur le sien, en beta 3, non — LIBERE-1 (fiche 72, beta 4) en défend la cause la plus
  probable (l'activité recréée dans le même processus, l'ancien Glucose tenant encore le verrou).
* **Plus loin (étape 3)** : décoder à la taille montrée — un JPEG se décode au demi, au quart, au
  huitième pour presque rien — rendrait « Tout cadrer » sur six mille images bien plus rapide et
  léger que décoder six mille originaux pour en garder la vignette. Les aperçus le font déjà à la
  seconde ouverture.
* **Vu en passant** : le journal du téléphone dit « porte des agents : un autre Glucose la tient,
  ou le système la refuse », alors qu'il n'y a pas de porte au téléphone — à regarder.

## 7. Les leçons

* **Un travail qui rend une ressource ne doit jamais attendre derrière celui qui la consomme** :
  la priorité « rien ne l'attend » était fausse, la mémoire l'attendait.
* **Un état intermédiaire que personne ne compte est un trou dans l'instrument** : « en chemin »
  n'était ni tenu ni offert, et le journal disait 2,7 Go « inconnus ».
* **Un téléphone qu'on ne possède pas se remplace par un téléphone virtuel** : sans câble, la
  cause dormait depuis dix jours ; l'émulateur l'a montrée en une heure. `MSYS_NO_PATHCONV=1`
  devant `adb` sous Git Bash, sinon `/data/local/tmp` devient `C:/Program Files/Git/data/…`.
