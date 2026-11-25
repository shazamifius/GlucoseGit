# 159 — Les GIF qui bougent : l'étude (25/11 vers 3 h 20, le relais, compte b)

Le plan du relais, point 12, après les miroirs (fiche 158) : **les GIF animés** (INVENTAIRE § 3).
Une étude : rien n'est codé. Contrairement aux miroirs, **le format ne change pas** : le document
garde les octets posés tels quels (`menu_image.rs`), toutes leurs images comprises. Le chantier se
code dès maintenant.

## 1. Où on en est

Un GIF posé montre sa première image, partout : le canevas, l'aperçu, la minimap, l'export.
Glucose Tauri les animait sans rien faire (le navigateur), et jouait les vidéos par `<video>`
(VID-1, `GlucoseCanvas.tsx`).

## 2. Ce que font les navigateurs

Chromium et Firefox jouent **à 100 ms une image dont le délai vaut 10 ms ou moins** : la règle
d'Internet Explorer, reprise parce que les auteurs écrivaient 0 ou 10 en voulant 100, et pour le
processeur ([IEInternals, *Animated GIF Timing*](https://learn.microsoft.com/en-us/archive/blogs/ieinternals/trivia-animated-gif-timing) ;
[Mozilla 232822](https://bugzilla.mozilla.org/show_bug.cgi?id=232822) ; [WebKit 14413](https://bugs.webkit.org/show_bug.cgi?id=14413) ;
la mesure de 2022 : [GIGAZINE](https://gigazine.net/gsc_news/en/20220222-fastest-gif-animation)). Les
GIF du monde sont faits pour cette règle : la suivre, c'est les jouer comme leurs auteurs les ont
vus.

## 3. La proposition

* **La première image reste l'image fixe** : la pyramide, l'aperçu sur le disque, la minimap,
  l'export PNG ne changent pas. Les exports SVG et HTML posent déjà les octets d'origine
  (`scene/image/svg.rs`, `export/html.rs`) : un navigateur les anime.
* **Un lecteur par GIF visible** (le tri de ce qui est à l'écran le dit) : il décode, sur l'atelier
  et par son admission (`pendant(octets, travail)`), **l'image suivante seulement**, à la taille du
  niveau que la carte graphique montre ; il ne garde que la toile composée (les GIF se composent
  par-dessus l'image d'avant). Pas de cache de toutes les images : la mémoire d'un GIF est celle
  d'une image, quelle que soit sa longueur. Hors de l'écran, il n'existe pas.
* **Le temps** : le délai de chaque image, 10 ms ou moins valant 100 ms (§ 2), compté sur
  l'horloge de ce que l'écran montre (`horloge.rs`) ; **une raison de réveil à elle** (`reveil.rs`)
  à l'échéance de l'image suivante — un GIF à l'écran réveille Glucose à son rythme (10 images par
  seconde d'ordinaire), sinon Glucose dort comme aujourd'hui. Le nombre de tours que le GIF
  demande (l'extension NETSCAPE2.0) se respecte.
* **Le coût par image** : un décodage hors du fil qui dessine, une écriture dans une texture qui
  existe déjà (≈ 1 Mo pour 500 × 500) : rien qui menace les 100 images par seconde.
* **Réduire les animations** (le réglage du système : GNOME `enable-animations`, Windows
  `SPI_GETCLIENTAREAANIMATION`, Android l'échelle des animations à zéro) : les GIF restent fixes.
* **WebP animé et APNG** : le même lecteur, si la caisse `image` les décode image par image (elle
  en a les décodeurs ; à vérifier au moment de coder).

## 4. Les étapes

1. **GIF-1, le lecteur** : des octets à la suite des toiles et de leurs délais (la règle des 10 ms,
   les tours), éprouvé sur des GIF construits dans l'épreuve ; aucune dépendance nouvelle (`image`
   a déjà `gif`).
2. **GIF-2, l'écran** : le lecteur branché au tri de ce qui est visible, la texture réécrite, le
   réveil ; regardé au pilote.
3. **GIF-3** : le réglage du système qui réduit les animations.

**Les vidéos** ne suivent pas ce chemin : il leur faut un décodeur (celui du système sur chaque
machine, ou une bibliothèque) — une dépendance, donc sa note dans `decisions/`, et une étude à part.

## 5. Le chemin dans le code (lu le 25/11, pour la session qui codera)

* **Ce que la carte montre d'une photo** se décide dans `renderer/voies/tenue.rs` (`pose_tenue` →
  `PhotoTenue { numero, facteur, pose, … }`).
* **Une texture a une identité et un contenu** (`present/scene_gpu/cle.rs`) : `Cle::Photo(numero)`
  est l'identité, `Cle::Niveau(numero, facteur)` ce qu'elle montre ; `SceneGpu::connait(identite,
  cle)` (`scene_gpu.rs` l. 194) compare les deux, et la cascade (`scene_gpu/cascade.rs`,
  `assurer`) réécrit l'identité dont le contenu a changé, sur son budget, l'ancien posé pendant ce
  temps. **Une image de GIF est donc un contenu de plus sous la même identité** — une variante de
  clé qui porte aussi le rang de l'image —, et la cascade fait déjà le reste. À vérifier : qu'une
  identité réécrite à la même taille réutilise sa texture au lieu d'en créer une.
* **Les pixels** viennent de `Source` (`scene_gpu.rs` l. 55 : une clé → des pixels prêtés) : la
  source doit savoir prêter la toile courante du lecteur.
* **La voie processeur** lit les niveaux de la pyramide (`renderer/magasin.rs`, `photo.rs` :
  `niveau_pour`) : la toile courante doit s'y substituer aussi, sinon les deux voies divergent.
* **Le réveil** : `app/reveil.rs`, `enum Raison` (`Animation` y sert aux animations des nœuds) —
  une raison à lui, pour que la chronique dise pourquoi Glucose s'est réveillé.
* **Le décodeur** : `image::codecs::gif::GifDecoder` et `AnimationDecoder::into_frames()` rendent des
  toiles déjà composées, avec leur délai (`Frame::delay`). Le nombre de tours n'y est pas exposé
  (à vérifier) ; la caisse `gif`, déjà dans l'arbre par `image`, le donne (`Decoder::repeat`) — une
  dépendance directe, donc sa note dans `decisions/`.
* **Où l'image suivante entre** : `Magasin::recolter` (au début de chaque image, et là seulement :
  le rendu voit un cache immobile) verse ce que l'atelier a fini (`Fait::Decodee`, `Fait::Deplace`) ;
  `accueillir` y remplace une pyramide par une neuve pour la même source (le numéro reste). Une
  image de GIF peut y entrer de la même façon — un `Fait` de plus, la pyramide de la toile — **à
  condition que la clé de la carte change avec elle** : rien du côté de la carte ne porte de
  génération (vérifié), une pyramide remplacée sous la même clé garderait l'ancienne texture.
  `accueillir` ne doit pas réécrire l'aperçu du disque pour une image de GIF (l'aperçu reste la
  première), et `changements` doit croître, sans quoi LOT-1 reconduit la récolte d'avant.

## 6. Les décisions prises en lisant (25/11 vers 3 h 25, pour GIF-1 et 2)

* **Fait d'abord : OUVRIER-1** (`87611c8`) — `atelier.rs` touchait son plafond (594 lignes sur
  600) : la vie d'un ouvrier (`ouvrier`, `attendre`, `decoder`, `encoder_en_png`) est passée dans
  `atelier/ouvrier.rs`, un déplacement pur ; `atelier.rs` a 476 lignes.
* **Le lecteur, sans dépendance nouvelle** : `image` 0.25.10 donne tout — `GifDecoder` puis
  `AnimationDecoder::into_frames()` (des toiles déjà composées, `Frame::delay()` en millisecondes),
  et `ImageDecoder::loop_count()` (`LoopCount::Finite(n)` ou `Infinite`, `codecs/gif.rs` l. 427) ;
  `GifEncoder::set_repeat` fabrique les GIF des épreuves. La caisse `gif` n'est pas nécessaire.
* **Savoir qu'une image est un GIF** : ni la clé (une clé du document n'a pas toujours
  d'extension) ni la pyramide ne le disent, et une image rouverte vient de son **aperçu** sans
  jamais passer par le décodeur. Proposé : le travail `Decoder` lit la signature (`GIF8`, six
  octets) sur ses deux chemins — une lecture partielle de la source (`Objets::source` : fichier ou
  tranche du document), sans l'empreinte, qui ne sert qu'à choisir ; seule la lecture entière, par
  le joueur, la vérifie — et `Fait::Decodee` porte ce drapeau jusqu'à l'entrée du magasin.
* **Le joueur** : un fil par GIF visible, qui cède au fil qui dessine
  (`plateforme::priorite`), lit les octets par `Objets::lire` (vérifiés), construit la pyramide de
  chaque toile sous l'admission, et les tend par un `sync_channel(1)` (une image d'avance, deux en
  mémoire au plus). Le magasin le lâche quand l'image n'est plus vue : le récepteur tombe, le fil
  s'arrête à son envoi suivant. Un GIF d'une seule image s'arrête à la deuxième demande, et
  l'entrée le retient (plus de joueur pour elle).
* **La clé de la carte** : `Cle::Niveau(numero, facteur)` porte aussi la **génération** de l'entrée
  (`Entree::generation`, qu'`accueillir` fait déjà croître) — la même règle sert alors à toute
  pyramide remplacée, pas seulement aux GIF.

## 7. GIF-1 et GIF-2 faits (`c947e97`, 25/11 vers 4 h 30, compte b)

Le lecteur, le joueur, le magasin et la clé de la carte suivent le § 6. **Trois défauts trouvés en
finissant**, chacun gardé par une épreuve et un sabotage (`outils/sabotages/gif-159.json` : **9 sur 9 tombent** ; l'oubli de la vignette par le magasin était aveugle au premier passage, gardé depuis par l'épreuve du magasin) :

* **Le réveil redessinait à chaque image** tant qu'un GIF était à l'écran : `attente_des_gifs`
  salissait la vue à chaque calcul du délai, donc à chaque tour de boucle (`ControlFlow` relancé
  par la demande d'image elle-même). Il ne demande plus une image qu'à l'heure d'une toile ; une
  toile en préparation n'attend rien de lui : **le fil du joueur réveille la boucle** en la
  tendant (`Magasin::brancher_le_reveil`, branché avec les autres dans `demarrage.rs`). Un GIF à
  dix toiles par seconde coûte dix images par seconde, plus aucune autre.
* **Une toile posée comptait comme une vue** (`Entree::nouvelle` date l'entrée de l'image en
  cours) : un GIF sorti de l'écran se gardait en vie de toile en toile. `accueillir(…, toile)`
  garde la vue d'avant.
* **La voie processeur reposait la vignette de la première toile** : une vignette ne connaît que
  son nœud, sa forme et son fichier. `Vignettes::oublier_la_source` l'oublie à chaque toile (la
  vignette faite, celle à demi construite, et la forme demandée : un GIF qui joue n'en fait
  construire aucune ; arrêté, il retrouve la sienne).

**Écarts au § 6, et doutes** :
* Le joueur construit ses pyramides **hors de l'admission** de l'atelier : trois toiles au plus
  par GIF à l'écran (la prête, celle du canal, celle qu'il fait), jamais comptées dans la borne de
  mémoire. Un GIF de 2 000 × 2 000 pèse 16 Mo la toile : à surveiller, pas encore mesuré.
* La première toile se repose une fois quand le GIF paraît (la même que celle du décodage) : un
  envoi de texture de trop, accepté — un GIF qui revient à l'écran doit, lui, repartir du début.
* **NETTETE-2, hors des GIF** (`magasin.rs`, `accueillir`) : une vignette faite sur l'**aperçu**
  d'une photo survivait au décodage entier qui le remplace — la voie processeur reposait l'aperçu
  agrandi, flou, tant que la vue ne bougeait pas (la charte : « à l'arrêt, net »). Prouvé par une
  épreuve qui tombait avant le geste (`nette_une_pyramide_remplacee_oublie_ses_vignettes`) ; jamais
  vu à l'écran. **Le geste** : toute pyramide qui en remplace une autre oublie les vignettes de son
  fichier, pas seulement une toile — un tour des vignettes par remplacement (pas par arrivée),
  quelques microsecondes pour les centaines de nœuds dessinés. Saboté (le même sabotage que la
  vignette du GIF, élargi).

**Regardé au pilote** (25/11 vers 4 h 15, le binaire de `ae960b5` construit à part : le sien,
`target/release`, remis en place aussitôt) : un GIF de quatre toiles unies de 600 ms (fabriqué par
un script, 400 × 300), posé par la porte du Glucose piloté (`poser-page … capture=<gif>` — le nom
du reçu dit `.png` : c'est bien la signature qui décide). Six captures à 300 ms d'écart, les
couleurs comptées : **par la carte graphique**, vert, vert, bleu, bleu, jaune, jaune ; **par le
processeur** (`moteurs.txt`), vert, bleu, bleu, jaune, jaune, rouge — la vignette ne fige plus
rien. **Le coût au repos**, GIF à l'écran : 0,8 % d'un cœur (carte graphique), 1,2 % (processeur),
sur 10 s ; **le GIF sorti de l'écran** (la vue déplacée au clic molette) : son fil s'arrête (24 → 23
fils), 0,3 %.

**Reste** : GIF-3 (réduire les animations).

**Pour GIF-3, lu le 25/11** : sous Linux, le réglage vit dans le portail du bureau, `Settings`,
espace `org.freedesktop.appearance`, clé **`reduced-motion`** (xdg-desktop-portal 1.21, janvier
2026 ; GNOME et KDE y branchent leur interrupteur) ; repli sur `org.gnome.desktop.interface
enable-animations` quand la clé manque (c'est ce que fait la caisse `mundy`). **Aucune lecture des
réglages du système n'existe dans Glucose** ; `ashpd` 0.11 (et `zbus` 5) sont déjà compilés, par
`rfd` : s'en servir directement coûte une ligne de `Cargo.toml` et sa note dans `decisions/`, pas
une compilation de plus — mais `ashpd` est asynchrone : la lecture se fait sur un fil, à
l'ouverture, et le signal `SettingChanged` suit un changement en cours de route. Sources :
[Phoronix, xdg-desktop-portal 1.21](https://phoronix.com/news/xdg-desktop-portal-1.21),
[gpui-base, reduce_motion.rs](https://docs.rs/crate/gpui-base/0.7.0/source/src/reduce_motion.rs),
[mundy, freedesktop](https://docs.rs/crate/mundy/0.2.3/source/src/freedesktop/mod.rs),
[GNOME, « reduced motion »](https://gitlab.gnome.org/GNOME/gnome-control-center/-/merge_requests/3253).

## 8. GIF-3, moins de mouvement (25/11 vers 4 h 30, compte b)

* **La lecture** (`plateforme/mouvement.rs`) : au lancement, puis **chaque fois que la fenêtre
  revient au premier plan** — c'est de là qu'on revient d'avoir changé le réglage ; sur un fil, la
  boucle réveillée seulement si la réponse change. Linux : le portail, `reduced-motion`, sinon
  l'interrupteur de GNOME (`ashpd` et `pollster`, déjà là par `rfd` : `decisions/19`, aucune
  compilation de plus). **Le portail de son poste est en version 2, sans la clé** : l'interrupteur
  de GNOME répond (`Some(false)`, lu par le code le 25/11 — une épreuve ignorée, qu'on lance à la
  main : `cargo test -- --ignored gif_3_le_portail`). Windows : `SPI_GETCLIENTAREAANIMATION` ; le
  Mac : `NSWorkspace.accessibilityDisplayShouldReduceMotion` (`objc2-app-kit`, déjà là ; deux
  fonctions de plus). Vérifiés par `verifier_cibles.sh`, jamais lancés sous Windows ni sur un Mac.
* **L'effet** : l'application porte le réglage au magasin (`suivre_le_systeme`, avec les marges et
  le clavier) ; moins de mouvement, chaque GIF reste sur la toile qu'il montre et plus rien ne
  réveille ; le réglage levé, il repart. Deux épreuves, deux sabotages (`gif-159.json`, 11 sur 11).
* **Pas suivi en direct** : un réglage changé pendant que Glucose garde le premier plan ne vaut
  qu'au retour de la fenêtre. Le portail sait l'annoncer (`SettingChanged`) : un fil à l'écoute
  pour toujours, et une caisse de flux de plus — écarté tant que personne ne le demande.
* **Android** : `ValueAnimator.areAnimatorsEnabled` (« Supprimer les animations », ou l'échelle
  des animations à zéro), par le pont Java du téléphone (`glucose-android`, qui installe son
  lecteur comme celui du doigt) — Android 8 et plus ; avant, la méthode manque, son exception est
  rendue et rien ne se lit (son Redmi 9 est sous Android 10). Vérifié par `verifier_cibles.sh
  android`, jamais lancé sur un téléphone. **Les quatre systèmes le lisent.**

## 9. Les WebP et PNG animés (GIF-4, 25/11 vers 4 h 45, compte b)

* **Le même lecteur** : `image` 0.25 compose aussi les toiles d'un WebP animé (`WebPDecoder`,
  `has_animation`) et d'un PNG animé (`PngDecoder::apng`, `is_apng`) ; `ouvrir` choisit le
  décodeur par la signature, `tour` est commun. La règle des 10 ms vaut pour les trois (Chromium
  l'applique à toute image animée, à ce qu'on en sait ; non vérifié pour Firefox).
* **La signature** (`gif/signature.rs`, une fonction pure) : `GIF8` ; un WebP dont le morceau
  `VP8X` porte le drapeau de l'animation (octet 20, bit 1) ; un PNG dont `acTL` vient avant le
  premier `IDAT`. **Lue aussi loin qu'il faut** (`ouvrier::est_anime`) : 64 octets d'abord, puis
  ce que les morceaux disent — un profil de couleur de 3 Ko devant `acTL` est franchi en une
  lecture de plus ; aucun plafond : un fichier qui finit avant ne dit rien, et la lecture s'arrête.
* **Épreuves** : un APNG écrit par la caisse `png`, un WebP animé assemblé à la main (`image`
  n'en écrit pas ; `image-webp` compose chaque toile par un mélange qui arrondit d'un cran : 254
  pour 255, toléré), les signatures, la lecture qui s'allonge. Quatre sabotages de plus
  (`gif-159.json`, 15 sur 15). Pas regardé au pilote : le chemin d'après la signature est celui
  des GIF, qui l'a été.
