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

## 6. Les décisions prises en lisant (25/11 vers 3 h 45, pour GIF-1 et 2)

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
