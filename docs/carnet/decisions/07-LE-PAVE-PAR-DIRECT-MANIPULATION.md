# 07 — Le pavé de précision par *Direct Manipulation*

> Le pincement au pavé arrivait à Glucose en `Ctrl` + molette, par paquets : trois réglages de
> gain et de lissage n'ont pas rendu la continuité perdue en route (fiche 52 § 8 à 10). Cette
> note prend le pavé comme Chromium et Blender le prennent. Elle n'ajoute **aucune caisse** :
> trois recoins de `windows`, déjà présente.
>
> **Date** : 2026-10-07 · **Portée** : `crates/glucose-desktop`, `interactions::pave`,
> `plateforme::pave_windows` · **Fiche** : [`53`](../53-PINTEREST-ET-LE-PAVE.md).

---

## Ce qu'il faut

* **`Win32_Graphics_DirectManipulation`** : le gestionnaire, le *viewport* fictif, l'écouteur
  de sa transformation. C'est l'API que Windows destine exactement à cela : livrer le geste
  d'un pavé de précision avec son échelle, son déplacement et son inertie, au rythme du pavé.
* **`Win32_UI_Input_Pointer`** : `GetPointerType`, pour ne prendre que les contacts du **pavé**
  — un doigt sur un écran tactile garde son chemin.
* **`Win32_UI_WindowsAndMessaging`** : `DM_POINTERHITTEST` arrive à la procédure de la fenêtre,
  que `winit` possède ; on glisse la nôtre devant (`SetWindowLongPtrW`) et on lui rend tout le
  reste. Cette fonctionnalité n'était tirée que par les épreuves.

## Pourquoi pas autre chose

| | ce que ça donne | pourquoi non |
|---|---|---|
| régler la molette encore | trois essais le 07/10 | les paquets ont déjà perdu le geste |
| `WM_POINTER` à la main | les contacts bruts du pavé | réécrire la reconnaissance des gestes et l'inertie de Windows ; et un pavé ne livre ses contacts qu'à qui prend *Direct Manipulation* |
| `SetWindowSubclass` | le sous-classement conseillé | vit dans les contrôles communs **v6** : sans manifeste qui les demande, le programme **ne démarre pas** (vu en épreuve, `STATUS_ENTRYPOINT_NOT_FOUND`) |

## Ce qui reste vrai

* **Rien ne tourne au repos** : le système n'avance qu'entre le début d'un geste et la fin de son
  inertie (`OnInteraction`, `RUNNING`, `INERTIA`).
* **Un refus ne casse rien** : si Windows refuse, le pavé reste des défilements de molette.
* **Hors de Windows**, rien ne change : `winit` y livre déjà le pavé en pixels.

## Sources

* Chromium, `content/browser/renderer_host/direct_manipulation_helper_win.cc` et
  `direct_manipulation_event_handler_win.cc` — le viewport de 1000 × 1000, la mise à jour
  manuelle, la remise à l'identité au `READY`.
* Blender, `intern/ghost/intern/GHOST_TrackpadWin32.cc` — le pincement qui n'est pas reconnu
  tout de suite, la translation « absurde » pendant un pincement.
