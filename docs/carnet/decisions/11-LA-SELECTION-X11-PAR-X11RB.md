# 11 — La sélection de X11, servie par Glucose avec `x11rb`

> Son choix du 10/10 (la voie B de la fiche 60 § 3) : sous Linux, un seul « Copier » qui pose à
> la fois l'image, les fichiers et le bloc, comme sous Windows depuis la fiche 59. Cette note dit
> la caisse prise **en direct**, `x11rb`, qui était **déjà dans l'arbre** par `arboard` — même
> version, aucune caisse nouvelle à compiler.
>
> **Date** : 2026-10-10 · **Portée** : `plateforme/selection_x11*`, sous Linux et les autres
> Unix de X11 · **Fiche** : [`61`](../61-LE-COPIER-UNIQUE-SOUS-LINUX.md).

---

## Ce qu'il faut

Sous X11, copier, c'est **devenir le propriétaire** de la sélection `CLIPBOARD` et répondre à
chaque application qui colle, dans la forme qu'elle demande (ICCCM, chapitre 2). Pour poser le
lot, le texte, l'image et les fichiers **ensemble**, Glucose doit tenir ce propriétaire
lui-même :

* **`arboard`** (3.6.1, lu à la source, `platform/linux/x11.rs`) sait en servir plusieurs en
  interne, mais son interface n'en pose **qu'une** à la fois (`set_text`, `set_html`,
  `set_image`, `set_file_list` : chacune remplace la précédente) ; et il écrit chaque forme d'un
  bloc, **jamais par `INCR`** — une image plus grande que la plus grande requête du serveur ne
  partirait pas ;
* **`clipboard-rs`** pose plusieurs formes d'un coup, mais n'écrit jamais par `INCR` non plus, et
  l'erreur d'un envoi trop gros tue sa boucle de service (lu à la source, fiche 60 § 3).

## Pourquoi `x11rb`

* **Déjà là** : `arboard` la tire sous Linux (0.13.2) ; la prendre en direct ne compile rien de
  plus, et ne grossit pas le programme.
* **En Rust pur** (`RustConnection`) : ni `libxcb`, ni `libX11`, ni compilateur C — rien à
  installer chez l'utilisateur.
* **Le protocole tel qu'il est**, sans couche : la propriété, `TARGETS`, `TIMESTAMP`, `MULTIPLE`,
  `INCR`, la remise au gestionnaire (`SAVE_TARGETS`) s'écrivent comme Chromium les écrit
  (`ui/base/x/selection_owner.cc`).

## Pourquoi pas autre chose

| | ce que ça donne | pourquoi non |
|---|---|---|
| **`arboard` seul** | rien à écrire | une forme à la fois : le recul de COPIER-1 sous Linux (fiche 60 § 3) |
| **`clipboard-rs`** | plusieurs formes | pas d'`INCR`, une boucle qui meurt d'une grosse image |
| **`x11-clipboard`** | un propriétaire simple | une forme par appel, lui aussi ; et une caisse de plus |
| **`libxcb` par `xcb`** | la bibliothèque du système | une bibliothèque C à lier, que `x11rb` en Rust pur évite |
| **Wayland directement** (`wl_data_device`) | sans XWayland | il faut la connexion de `winit` et le numéro du dernier geste ; un autre chantier, le jour où un bureau sans XWayland se présentera |

## Ce qui reste vrai

* **Wayland passe par XWayland**, comme avant avec `arboard` : GNOME relaie les deux sens depuis
  2015, KDE sans exiger le focus depuis Plasma 6.0.
* `libc` est prise sous le même `cfg` : l'attente bornée d'une lecture (`poll`) en a besoin,
  et elle n'était déclarée que pour Linux et Android.

## Sources

* ICCCM, chapitre 2 — [*Peer-to-Peer Communication by Means of Selections*](https://tronche.com/gui/x/icccm/sec-2.html).
* freedesktop, [*Clipboard Manager Specification*](https://wiki.freedesktop.org/www/ClipboardManager/).
* Chromium, `ui/base/x/selection_owner.cc`, `ui/base/clipboard/clipboard_ozone.cc`,
  `ui/base/clipboard/file_info.cc` (lus à la source, `chromium.googlesource.com`).
* `arboard` 3.6.1, `src/platform/linux/x11.rs` ; `x11rb` 0.13.2.
