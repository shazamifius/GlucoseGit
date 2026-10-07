# 53 — Pinterest qui devenait des liens, et le pavé par *Direct Manipulation*

> Session du 07/10/2026 au soir. Son message d'ouverture : *« j'ai eu de gros gros soucis avec
> Pinterest et les téléchargements qui n'ont pas du tout voulu se faire — ça a finalement créé
> des liens bleus moches au lieu des images »*, capture à l'appui, avec la sortie de sa session
> (`sortie-essais-52d.txt`). Puis les chantiers 1 bis (le pincement) et 1 ter (la sélection) de
> la suite. **Rien de ce qui suit n'a encore été vu à son écran** : le § 6 dit quoi regarder.

---

## 1. Pinterest : six épingles devenues six liens (DEPOT-WEB-6)

**Ce que disent ses traces** — toutes lues sur des **copies** : son document (`fuser.glucose`,
212 Mo, copié dans le bac de la session, lu par `lire_histoire`), sa boîte noire, sa chronique.

* Les six liens sont nés à **13 h 11 min 41 s** (trois dans la même seconde), puis 13 h 11 min 57,
  13 h 12 min 13, 13 h 12 min 29 : **seize secondes pile** d'écart, le rythme du délai de quinze
  secondes d'une étape de téléchargement, pas celui d'une main.
* La boîte noire montre 131 s « au repos » juste avant, pendant lesquelles Glucose a dessiné
  243 images : le marqueur « en chemin » qui attendait.
* Sa sortie ne contient **aucune** ligne « image rapatriée », et aucune raison : le détail
  d'un échec ne s'écrivait que sous l'instrument `GLUCOSE_DEPOT`.
* Les six mêmes épingles, essayées ici une heure plus tard par `essai_rapatriement`, se
  rapatrient toutes en pleine résolution. Ce n'était donc pas Pinterest qui avait changé.
* Windows n'a rien noté (journaux réseau, système) : la cause exacte de **ce moment-là** reste
  sans trace.

**La cause trouvée** : chronométrée, une page d'épingle met **17 à 53 s** à arriver nue
(1,2 Mo), pour un premier octet en moins d'une seconde — et l'image n'y est annoncée (`og:image`)
qu'à **1,07 Mo**, presque à la fin du flux. Compressée, la même page arrive en **une seconde**
(127 Ko). Tous les navigateurs demandent la compression ; **WinHTTP ne la demande que si on le
lui dit**, et Glucose ne le disait pas. Sur un réseau un peu lent, une pause du serveur suffisait
à épuiser le délai d'une étape, et le dépôt retombait sur son repli : le lien. **Probablement la
cause de sa soirée, à confirmer** : le réseau de 13 h 10 n'a laissé aucune trace.

**Ce qui est fait** :

1. `WINHTTP_OPTION_DECOMPRESSION` : la page arrive compressée (vérifié : `accept-encoding: gzip`
   part, par un écho d'en-têtes).
2. **Un repli le dit** : la moisson porte sa raison (`Moisson::echec`, le site et « délai
   dépassé », « nom introuvable »…) jusqu'au message — qui disait « 1 élément posé », un échec
   annoncé comme une réussite — et jusqu'à la sortie, toujours.
3. **Un lien se rattrape** : au clic droit sur un nœud qui n'est **qu'un** lien, « Remplacer par
   l'image » relance la recherche, marqueur au coin du lien ; l'image prend sa place **en un seul
   geste** (`Ctrl+Z` rend le lien) et se tait comme un collage. Une note n'est jamais prise pour
   un lien (`adresse_du_lien`, l'inverse exact de la pose). Ses six liens se réparent ainsi.
4. **Une livraison attend la fin du geste de la main** : posée pendant un glisser, elle
   refermait ce glisser-là (son `end_live_edit` fermait la transaction de l'utilisateur). Le
   défaut existait déjà pour tout dépôt rapatrié.

**La recherche** : tldraw fait comme Glucose (les métadonnées Open Graph de la page) ; Eagle
passe par une extension de navigateur ; PureRef ne documente rien. La manière était juste.

**Épreuves** : six par l'entrée réelle (le clic sur l'entrée du menu, le relevé de la boucle),
la recherche fournie par l'épreuve — aucune n'atteint le réseau ; la lecture d'un lien ; les
raisons de WinHTTP. Onze sabotages, onze chutes. Le banc `essai_rapatriement` dit désormais le
nom, le poids, la taille de l'image et le temps de la page seule.

**Pas fait** : hors de Windows, `ureq` demande toujours la page nue (sa fonctionnalité `gzip`
tirerait `flate2` : une dépendance à défendre dans `decisions/`, et un code que seule la CI
jugerait). Et le message ajoute de fait **un site de message** de plus, par `dire_le_depot` : le
cliquet compte les appels à `show_toast`, pas les intentions — c'est dit ici plutôt que caché.

## 2. Le pincement par *Direct Manipulation* (chantier 1 bis)

**Son jugement**, après trois réglages : *« légèrement trop lent, et pas fluide, comme s'il
sautait »*, quand PureRef et un navigateur pincent *« instantanément, de manière fluide »*. La
cause : le pincement arrivait en `Ctrl` + molette, par paquets — continuité perdue avant d'arriver.

**La recherche** :

* Chromium, `direct_manipulation_helper_win.cc` et `direct_manipulation_event_handler_win.cc` :
  un *viewport* **fictif** de 1000 × 1000, `MANUALUPDATE`, `SetContact` à chaque
  `DM_POINTERHITTEST` d'un pavé (`PT_TOUCHPAD` seulement), `Update` à chaque image entre
  `INTERACTION_BEGIN` et `END`, la transformation lue (`xform[0]`, `[4]`, `[5]`), et au `READY`
  la remise à l'identité (`ZoomToRect`) seulement si elle n'y est pas déjà.
* Blender, `GHOST_TrackpadWin32.cc` : le pincement n'est pas toujours reconnu au premier instant
  (permettre de passer du déplacement au pincement) ; pendant un pincement, la translation est
  « absurde » ; l'état `RUNNING`/`INERTIA` à suivre.

**Ce qui est fait** :

* `interactions::pave` (pur) : `Lecteur` tire de chaque transformation un `Deplacer` ou un
  `Zoomer`. Un pincement le reste jusqu'à la fin du geste ; les octaves sont une **différence de
  logarithmes en `f64`** — la somme d'un geste vaut exactement son échelle (un rapport en `f32`
  perdait un demi-millionième d'octave, l'épreuve l'a vu) ; deux échelles égales à la précision
  d'un `f32` ne sont pas un pincement — aucun seuil choisi. Ce qui bouge passe par la **porte de
  la souris** (montré à l'image suivante) : le lissage et l'inertie sont ceux du système, ceux
  d'Edge et de Chrome. Le déplacement à deux doigts garde son élan parce que le système le donne ;
  le pincement n'en a pas.
* `plateforme::pave_windows` (le COM). **Sans les « rails » de Chromium** : sur un canevas, un
  déplacement en biais reste en biais. **Sans inertie d'échelle** : le pincement s'arrête avec
  les doigts.
* Une raison de réveil à lui (« le pavé tactile ») : hors geste, rien ne réveille la boucle.
* `GlucoseApp::new` dépassait 80 lignes : les deux tampons auxiliaires sont devenus `Tampons`.

**Deux fautes évitées par les épreuves** :

1. `SetWindowSubclass` vit dans les contrôles communs **v6** : sans manifeste qui les demande, le
   programme **ne démarre pas** (`STATUS_ENTRYPOINT_NOT_FOUND`, vu au lancement de l'épreuve).
   Glucose aurait refusé de s'ouvrir chez lui. Le sous-classement de `user32` le remplace.
2. Au `READY`, la remise du viewport à l'identité se serait lue comme un **dézoom défaisant tout
   le geste** si la lecture n'était pas remise **avant** `ZoomToRect`. L'épreuve le montre sur le
   vrai système.

**Épreuves** : la **chaîne COM réelle** sur une fenêtre à −32 000 jamais montrée — le viewport,
zoomé par programme d'un facteur deux, rend exactement une octave par le vrai écouteur, puis
revient à l'identité sans que rien ne soit défait ; l'installation et le retrait (la procédure de
`winit` reprend sa place) ; la lecture ; l'application par une source factice à travers la vraie
boucle (`suivre_le_pave`, `prochain_reveil`). Treize sabotages, treize chutes. Note
`decisions/07` : trois recoins de `windows`, aucune caisse.

**Pas prouvé** : le geste réel. Windows ne sait injecter que des doigts d'écran tactile et des
stylets, pas un pavé : **seul son écran le dira**. Le sens du déplacement est celui que Chromium
déduit (la transformation dit où va le contenu, sens choisi par l'utilisateur compris) ; s'il
était inversé, c'est un signe à changer.

## 3. Le dessin de la sélection (chantier 1 ter)

La planche des quatre directions est devenue **cinq** (`screens/53-selection-cinq-directions.png`),
**envoyée à son écran**. La recherche : PureRef, Figma et tldraw dessinent **un seul cadre pour le
groupe**, avec ses poignées, et chaque élément choisi ne reçoit qu'un **fil fin** — d'où E,
« un cadre, et un fil par image ». **Mon avis : E** — le groupe se lit d'un coup d'œil, chaque
image reste désignée sans être encadrée de carrés, et c'est la forme que ses outils lui ont
apprise. **Son choix** attend.

## 4. La CI des commits hors Windows

Relus, les blocs `cfg(not(windows))` ajoutés depuis `20deb6f` (`plateforme::ecran`,
`glisser_un_lot`, `installer_le_pave`) rendent `None` ou `Ok(false)` sans rien employer de
Windows, et les modules sont publics (pas d'avertissement d'inutilité sous Linux). La compilation
croisée reste impossible ici (pas de compilateur C pour Linux, `ring`), et le moteur Docker ne
tourne pas — le lancer ouvrirait sa fenêtre. **Seule la CI le prouvera** : à lui proposer.

## 5. Prouvé, et pas prouvé

* **Prouvé** : 2 051 épreuves, clippy strict à zéro ; vingt-quatre sabotages, vingt-quatre chutes ;
  la compression demandée (écho d'en-têtes) ; les six épingles de sa capture rapatriées ici ; la
  chaîne *Direct Manipulation* réelle, hors écran.
* **Pas prouvé** : la cause exacte de 13 h 10 ; le pincement et le déplacement au pavé à son
  écran ; Linux et Mac (la CI).

## 6. Ce que seul son écran dira

1. **Le pincement au pavé** : « instantané et fluide », comme PureRef ? Le déplacement à deux
   doigts : son élan, et **dans le bon sens** ? En biais, reste-t-il en biais ? La sortie doit
   dire `pave : pris par Direct Manipulation`.
2. **La molette de la souris** : inchangée (un tiers d'octave par cran).
3. **Pinterest** : glisser plusieurs épingles de suite ; puis, sur ses six liens, les choisir,
   clic droit → « Remplacer par l'image ».
4. **Le mode référence** : `Alt` + pincer redimensionne toujours la fenêtre.
5. **Son choix** sur la planche de la sélection.

## 7. Son écran, le 07/10 à 15 h 40 — et ce qui en a suivi

**Ses verdicts** : le pincement — « dans l'idée c'est bien, le zoom maintenant » ; la molette —
« parfait » ; Pinterest — « ça fonctionne, mais plutôt long » ; le mode référence — `Alt` +
pincer marche, et « c'est dommage que tu les aies supprimés, c'était vraiment pratique » (à
éclaircir avec lui : sans doute les panneaux retirés du mode référence, fiche 52 § 10). **Le
déplacement à deux doigts** : « la translation, la multi-direction pose problème » — en
horizontal et en biais.

**Le déplacement, pas encore compris** : sa chronique dit les 494 pincements passés par
*Direct Manipulation* (aucun message de molette marqué), mais ne disait pas par où passaient les
1 297 déplacements. Deux hypothèses écartées par l'épreuve sur le vrai système : le viewport
fictif **n'a pas de bords** (son contenu n'a jamais reçu de limites, `0x802A0005`), et un
déplacement programmé dans les quatre directions est rendu exactement. Restent : des
déplacements qui arriveraient **encore en molette** (par paquets, avec l'élan de Glucose), ou un
pincement reconnu à tort au milieu d'un déplacement en biais, qui bloque le déplacement jusqu'à
la fin du geste. La chronique sépare désormais les deux voies (« dont par Direct Manipulation »),
et compte le pavé comme la main. **Sa prochaine session tranchera, avant tout réglage.**

**Pinterest, plus vite** : une session WinHTTP pour tout Glucose (les connexions restent
ouvertes, plus de poignée de main chiffrée à chaque requête) ; les variantes d'une image courent
ensemble (`rapatrier::course` : la meilleure qui répond gagne dès que toutes celles qui la
précèdent ont échoué). Six épingles à la suite : **0,7 à 3 s, 1,3 s en général**, contre 2,5 à
3 s. Le reste est le temps que le serveur de Pinterest met à préparer sa page. Pour
l'instantané, la piste est de **poser la vignette que le navigateur glisse**, puis de la
remplacer par l'original : chaque dépôt dit désormais en une ligne s'il en portait une.

**`F` et `Ctrl+F`** (sa demande) : `F` cadre la sélection — images, textes, flèches, dossier —,
ou tout si rien n'est choisi ; `Ctrl+F` cadre tout. C'est la convention de Maya et d'Unity ;
Figma, tldraw et Excalidraw ont `Maj+1` et `Maj+2`, Blender « vue sur la sélection » et
`Origine`. **Une collision à lui dire** : `Ctrl+F` est la recherche dans Glucose Tauri et dans
tous les logiciels ; la recherche (ETAT § 5) devra prendre une autre touche, ou la reprendre.

**Ce qu'il attend ensuite**, mot pour mot : *« une version ultra stable avec tout le mode Ctrl+Maj+A,
compatible tablette, Linux et Windows — la version Windows fonctionne excellemment bien — et Mac ;
un build complet pour tous les Linux, tous les Mac, et SURTOUT ce que j'attends le PLUS, c'est
vraiment pour Android ; et un système de télémétrie de tous les utilisateurs, quel que soit leur
appareil, afin de travailler sur l'amélioration de Glucose »*. La suite est réordonnée ainsi.

**Envoi** : il a validé l'envoi des commits **sans CI**. Le dernier commit envoyé porte
`[skip ci]`.

## 8. Son écran, le 07/10 à 16 h 30 — le déplacement, deux pistes

**Ses mots** : le zoom est bien ; « la translation, la multi-direction pose problème ».
**Sa chronique** (session 54) : les 3 026 déplacements et les 1 576 pincements sont **tous**
passés par *Direct Manipulation* — la molette est hors de cause. Deux faits :

* **un tressaut ×5,8**, le pire de ses sessions (×1,9 à ×3,3 d'habitude) : le système avançait
  au réveil de la boucle, qui ne bat pas avec les images — une image recevait deux pas, la
  suivante aucun. Il avance désormais **dans l'image, juste avant que la caméra bouge**
  (`GlucoseApp::bouger_la_camera`), comme Chromium à chaque pas d'animation. **Probablement la
  cause, à confirmer à son écran** ;
* **57 s d'épisodes « zoomer »** dans sa boîte noire, là où il devait se déplacer : peut-être un
  déplacement en biais que le système prend pour un pincement, et que la lecture bloque jusqu'à
  la fin du geste (Blender et Chromium ont la même règle et le même seuil infime). La chronique
  compte désormais ces **bascules** et leur plus petit écart d'échelle : sa prochaine session
  tranchera.

## 9. Pinterest, la copie d'abord

Sa sortie de la session 54 : ses glisser **portaient l'adresse de l'image** (aucune page lue,
aucune vignette glissée), et l'original a mis de 0,8 à **12,4 s** — un PNG de 2 Mo. La première
copie qui arrive se pose désormais **tout de suite**, à la taille qu'aura l'original
(`taille_d_un_apercu`), et l'original **prend sa place** : même nœud, même endroit, même taille
(sauf si l'utilisateur l'a changée) ; une copie effacée ne revient pas
(`interactions/depot_web/apercu.rs`). La copie part **seule d'abord** : en même temps que
l'original, elle mettait 1,9 à 3,6 s (la bande passante disputée), seule 0,26 à 0,93 s. La
décision de la course est pure (`rapatrier::Course`). Le remplacement est un geste du journal :
un `Ctrl+Z` juste après rend la copie.

## 10. Transformer une sélection entière — son « Blender »

**Sa demande** : *« un système comme Blender : une origine combinée pour scale et rotate, et un
autre bouton, origines individuelles »*. **Sur la planche** : C et B ses préférées, mais aucune
ne le satisfait ; il penche pour **E** (un cadre et un fil), dont le dessin ne lui convient pas
encore.

**La recherche** : Blender, le *point de pivot* (« Bounding Box Center », « Individual
Origins ») ; PureRef transforme le groupe, et chaque image autour de son centre sous `Maj+Alt`.

**Ce qui est fait** : `glucose_core::groupe` (pur) — mise à l'échelle par un coin (coin opposé
immobile, rapport gardé), rotation autour du centre du groupe ; origine commune ou individuelle,
la même transformation autour du groupe ou de chaque centre. Le magasin relève la pose de
départ de ce qu'un déplacement emporte et réécrit tout depuis elle, en un geste ; un bout de
flèche accroché suit son nœud ; `Store::emprise_du_groupe` garde le cadre tant que ni le
document ni la sélection ne changent. Deux nœuds choisis ou plus n'ont que **les quatre
poignées de leur groupe** ; `Alt` + coin le fait tourner. Le bouton « Origine commune /
Origines individuelles » vit dans la barre d'action. Le dessin E est **provisoire** : cadre du
groupe et ses poignées, chaque nœud garde son cadre sans poignée.

**Trouvé par l'épreuve à la vraie souris** : la collecte du clic par l'index ne voit que les
nœuds proches du curseur — elle ne voyait donc jamais de groupe. Le cadre se lit désormais sur
toute la sélection.

**Ce qui n'est pas fait** : une carte de texte grandit sans que sa police suive (sa hauteur
remonte à son texte) ; une carte, un pense-bête, une membrane ne tournent pas (le modèle ne
leur donne pas d'angle — leur centre tourne avec le groupe) ; pas de raccourci clavier pour
l'origine (sans rien qui le montre, il basculerait en silence) ; le dessin E à concevoir
**avec lui**.

**Prouvé** : 2 082 épreuves, clippy strict ; dans cette partie, vingt-neuf sabotages, vingt-neuf
chutes ; témoins regardés. **Pas prouvé** : tout ce qui précède à son écran. Les commits de ces
§ 8 à 10 (`a32e074`, `efab62f` et la documentation) **ne sont pas envoyés**.
