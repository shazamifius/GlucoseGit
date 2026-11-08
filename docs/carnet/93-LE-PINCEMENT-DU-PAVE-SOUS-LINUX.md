# 93 — Le pincement du pavé sous Linux (et sous macOS)

> Session du 8/11/2026, après la fiche 92. Sa demande : *« sur Linux, le pavé tactile pour zoomer et
> dézoomer ne fonctionne pas du tout »*.

## 1. La cause, lue dans le code de `winit`

`winit` 0.30.13 ne transmet le pincement (`WindowEvent::PinchGesture`) que **sous macOS et iOS** : son
code Wayland n'écoute pas le protocole des gestes, `zwp_pointer_gestures_v1`. Le compositeur, lui, le
donne — **niri le propose en version 3** (`wayland-info` sur sa session), comme GNOME, KDE et sway.
Le pincement n'arrivait donc jamais à Glucose. Le déplacement à deux doigts, lui, arrive en défilement
fin, et marchait.

**Au passage, un second défaut du même geste** : Glucose n'écoutait pas non plus `PinchGesture` —
**sous macOS, pincer ne zoomait pas davantage**. Personne ne l'avait vu, faute de Mac.

## 2. La réparation : un second pavé natif

Glucose avait déjà le contrat d'un pavé natif (le trait `Pave` : *ce que les doigts ont fait depuis
l'image précédente*, une similitude), rempli sous Windows par *Direct Manipulation* (fiches 53, 54 :
« tellement mieux qu'avant »). **Wayland le remplit désormais aussi** (`plateforme/pave_wayland.rs`) :

* il se branche sur **la connexion même de la fenêtre** (`Backend::from_foreign_display`) — le
  pincement ne vise que les surfaces de la connexion qui les a créées —, sur une file à lui, lue par un
  fil à lui ; libwayland laisse plusieurs lecteurs se partager la connexion ;
* il demande au siège un pointeur, puis le pincement de ce pointeur ;
* **PINCE-1** : chaque pas du geste dit l'échelle depuis le début du geste et le glissement du centre
  des doigts ; le pas est la similitude `q → r · (q − c) + c + d` (`Mouvement::pincement`), `r` le
  rapport à l'échelle du pas d'avant, `c` le centre des doigts (le pointeur au début, puis ses
  glissements), au facteur d'échelle de l'écran. Versée dans le chemin de Windows, la vue la montre en
  entier à l'image suivante : rien n'est classé, aucun seuil, le zoom et le déplacement ensemble ;
* sous macOS, `PinchGesture` donne la même similitude, autour du curseur.

**Aucune bibliothèque nouvelle** (`decisions/14`) : `wayland-client`, `wayland-backend` et
`wayland-protocols` sont déjà dans le programme, apportées par `winit` ; le `Cargo.lock` n'a gagné que
trois noms dans la liste de ce que Glucose emploie.

## 3. Prouver

* **Les épreuves** : au fil des pas, le point sous les doigts au début est sous les doigts à la fin, et
  tout autre point a grandi de l'échelle du geste autour de lui, au facteur d'écran 1, 1,25 et 2 ; une
  échelle absurde ne bouge rien ; le pincement de macOS zoome de son gain, le point sous le curseur y
  reste — par la vraie boucle (`bouger_la_camera`).
* **Sabotages** (`outils/sabotages/pave-93.json`) : **5 tombés sur 5**.
* **Sur sa vraie session niri** (une fenêtre de huit secondes, `essai_isole.sh`) : « pave : le
  pincement est pris au système ». **Le geste lui-même attend son doigt** : aucun pavé ne se simule ici
  sans les droits de `/dev/uinput`, et injecter des gestes dans sa session n'aurait pas été à faire. La
  boîte noire compte chaque pincement du pavé : sa télémétrie le dira.

**Vu à son écran le 8/11 au soir** (Glucose installé dans son profil, `1282358`) : *« ça fonctionne
parfaitement »*.

## 4. Ce que l'essai m'a appris

**Le premier essai à l'écran a lancé l'ancien programme** : les épreuves et clippy recompilent la
bibliothèque, pas `target/debug/glucose-desktop`. Aucune ligne du pavé, et j'ai cru d'abord à un échec du
branchement. Reconstruire le programme avant tout essai réel ; et faire dire au pavé **pourquoi** il ne
s'installe (`hors de Wayland…`, `le compositeur ne propose pas pointer-gestures…`) — utile aussi chez un
utilisateur dont le compositeur ne le donne pas.

## 5. Les limites, dites

* **Sous X11** (une session Xorg), le pincement reste ce que `winit` en dit : rien. XInput 2.4 le donne
  (Xorg 21.1 et après), par `x11rb`, déjà là : à faire si un utilisateur sous X11 le demande.
* **La rotation** des doigts est ignorée : la vue de Glucose ne tourne pas.
* **macOS** : écrit sans essai réel (aucun Mac ici) ; la CI le compile.
