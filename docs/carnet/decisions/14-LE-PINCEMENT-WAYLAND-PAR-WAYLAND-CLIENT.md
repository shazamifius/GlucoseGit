# 14 — Le pincement du pavé sous Wayland, par `wayland-client`

> 8/11/2026, fiche 93.

**Le besoin** : sous Linux, pincer le pavé ne zoomait pas du tout. `winit` 0.30 ne transmet le
pincement (`PinchGesture`) que sous macOS et iOS ; sous Wayland, le geste passe par le protocole
`zwp_pointer_gestures_v1`, que `winit` n'écoute pas. Le compositeur le propose (niri le donne en
version 3, comme GNOME, KDE et sway).

**Ce qui est pris** : `wayland-client` 0.31, `wayland-backend` 0.3 (`client_system`, pour se brancher
sur la connexion de la fenêtre) et `wayland-protocols` 0.32 (`client`, `unstable`, pour
`pointer-gestures`) — **sous Linux et les autres Unix seulement**.

**Pourquoi ce n'est pas une caisse de plus** : les trois sont déjà dans le programme, apportées par
`winit` (et `smithay-client-toolkit`, qui active déjà `unstable`). Aucune ne s'ajoute au `Cargo.lock` ;
Glucose s'en sert directement, là où `winit` s'arrête.

**Ce qui a été écarté** :

* **attendre `winit`** : sa version 0.30 ne le fait pas, et le geste manque aujourd'hui ;
* **écrire le protocole à la main** sur la prise de Wayland : il faudrait partager la connexion de
  `winit` (le pincement ne vise que les surfaces de la connexion qui les a créées), donc libwayland, que
  `wayland-backend` enveloppe déjà sans rien ajouter.

**Quand la retirer** : le jour où `winit` transmet le pincement sous Wayland, ce pont disparaît et la
même similitude ([`Mouvement::pincement`]) vient de `PinchGesture`.

**Un second usage, sans rien ajouter** (12/11/2026, fiche 112) : la sonde de PARU-1
(`plateforme/presentation_wayland.rs`) se branche de la même façon sur la connexion de la fenêtre
pour `wp_presentation` — un protocole **stable**, déjà dans `wayland-protocols` avec `client` ;
aucune fonctionnalité ni caisse de plus. Le jour où ce pont disparaît pour le pincement, il reste
pour elle, tant que `winit` ne dit pas quand une image a paru.

