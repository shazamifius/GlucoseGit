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
