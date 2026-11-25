# 160 — Les vidéos : l'étude (25/11 vers 3 h 25, le relais, compte b)

Le plan du relais, point 12 : **les vidéos** (INVENTAIRE § 3 ; Tauri les jouait par `<video>`,
VID-1, `GlucoseCanvas.tsx` l. 180, et sa liste d'extensions, `assetRef.ts` : mp4, webm, mov, mkv).
Une étude : rien n'est codé, et trois choses attendent sa parole (`relais-q-videos`).

## 1. Ce qu'une vidéo demande que les GIF n'ont pas

* **Un décodeur** : H.264 et HEVC (ce que filment les téléphones) sont brevetés ; un décodeur livré
  avec Glucose engage sa licence. Ceux du système l'ont déjà payée.
* **Le son** : une sortie audio, qui n'existe pas dans Glucose.
* **Le poids** : une vidéo de téléphone pèse des centaines de mégaoctets ; le document scelle
  aujourd'hui les octets de chaque photo. Une vidéo scellée ferait des documents énormes et des
  partages lents ; une vidéo liée (son chemin sur le disque) casse si le fichier bouge.

## 2. Les voies

| voie | poids livré | machines | licence des codecs | travail |
|---|---|---|---|---|
| **A. Les décodeurs du système** : Media Foundation (Windows), MediaCodec (Android), AVFoundation (Mac), GStreamer (Linux) | rien | toutes, si le système les a (sous Linux, les greffons de GStreamer) | payée par le système | quatre portes à écrire ; décodage par la carte graphique partout |
| **B. FFmpeg dans Glucose** (LGPL, lié dynamiquement) | des dizaines de Mo, non mesuré | toutes, à compiler pour chacune (Android, Mac) | à notre charge pour H.264 et HEVC | une porte ; la CI s'alourdit |
| **C. Les codecs libres seuls** (AV1, VP9 : dav1d, libvpx) | quelques Mo | toutes | aucune | une porte ; **ne lit pas ce que filme un téléphone** (H.264, HEVC) : écartée |

Ce qui existe en Rust : rien de mûr. `video-sys` (0.1, janvier 2026, MPL-2.0) fait la voie A avec
MediaCodec, GStreamer et `windows` ([lib.rs](https://lib.rs/crates/video-sys)) ; `mediaway-decoder`
(Media Foundation, avant-première) ; les enveloppes de FFmpeg (`playa-ffmpeg`, `ff-decode`)
([docs.rs](https://docs.rs/crate/playa-ffmpeg/8.0.3)). La taille d'un FFmpeg réduit à H.264 n'est
publiée nulle part (la recette : `--disable-all --enable-decoder=h264`, [libav-user, 2018](https://lists.ffmpeg.org/pipermail/libav-user/2018-March/010991.html)).

**Proposé : la voie A**, porte par porte, en commençant par Linux (sa machine) — aucun poids, aucune
licence à porter, le décodage par la carte graphique (les 100 images par seconde tiennent). Une note
dans `decisions/` le jour où elle se code. Le cycle de vie suit celui des GIF (fiche 159) : la
première image est l'image fixe (pyramide, aperçu, export) ; un lecteur par vidéo visible.

## 3. À sa parole (`relais-q-videos`)

1. **Scellée ou liée** : la vidéo dans le document (tout voyage avec lui, documents lourds) ou son
   chemin (léger, mais casse si le fichier bouge). Proposé : **scellée sous une taille** (celle d'une
   grande photo), **liée au-delà**, et dite comme telle sur le nœud.
2. **Le son** : joué au clic seulement, jamais au chargement (proposé), ou jamais.
3. **Lecture** : immobile avec ▶ jusqu'au clic (proposé : une vidéo qui bouge seule distrait sur un
   tableau de travail), ou en boucle muette comme un GIF.
