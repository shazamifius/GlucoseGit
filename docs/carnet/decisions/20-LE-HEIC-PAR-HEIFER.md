# 20 — Le HEIC par `heifer` (HEIC-1, 25/11)

**Le besoin** : `image` 0.25 ne lit pas le HEIC, le format des photos d'iPhone depuis 2017 (et de
beaucoup d'Android) ; une photo d'iPhone glissée dans Glucose est refusée (fiche 161).

**Le choix** : [`heifer`](https://crates.io/crates/heifer) 0.5, **sans sa fonction `encoder`**
(Glucose ne fait que lire) — HEIF et HEVC écrits en Rust, MIT ou Apache-2.0, validé sur les
fichiers de conformité officiels. Il tire trois caisses : les siennes (`heifer-isobmff`,
`heifer-hevc-dec`) et `thiserror`. **Mesuré** (fiche 161 § 5) : 627 Ko au binaire ; une photo
d'iPhone de 12 Mpx en 423-440 ms sur un fil ; sa vignette en 2 ms ; 3,8 octets par pixel de travail,
sous la borne de l'admission ; 31 fichiers d'épreuve sur 31 ; 60 photos abîmées sans une panique.
Une seule voie pour toutes les machines, Android compris : rien en C à compiler.

**Écartés** : `heic` d'imazen (sous AGPL-3.0 : Glucose est sous MIT, et partiel) ; `libheif` en C
(`heif-rs` : du C pour quatre systèmes et Android) ; les décodeurs des systèmes (Windows sans
l'extension payante du Store et Linux ne lisent rien : deux machines exclues, contre la charte).

**Le brevet** : le HEVC est couvert par des brevets (les groupes Access Advance et Via LA). Un
décodeur logiciel dans une application libre et gratuite est l'usage courant (VLC, libde265,
GIMP) ; à dire, pas à taire — à lui de le savoir avant la publication qui le porte.

**Si `heifer` s'arrêtait** : `heic-decoder` (ente, Rust pur aussi) est la voie de repli ; la porte
de Glucose (la signature, l'atelier) ne change pas.
