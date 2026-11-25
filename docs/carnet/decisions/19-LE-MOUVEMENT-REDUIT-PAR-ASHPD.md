# 19 — Le mouvement réduit par `ashpd` (GIF-3, 25/11)

**Le besoin** : sous Linux, le réglage « moins de mouvement » vit dans le portail du bureau
(`org.freedesktop.portal.Settings`, espace `org.freedesktop.appearance`, clé `reduced-motion`,
xdg-desktop-portal 1.21) ; sinon l'interrupteur de GNOME (`org.gnome.desktop.interface`,
`enable-animations`). Il faut parler D-Bus (fiche 159 § 8).

**Le choix** : `ashpd` 0.11 et `pollster` 0.4, **déjà compilés par `rfd`** (ses dialogues de
fichiers passent par le même portail) — mêmes versions, des fonctions déjà activées : la ligne de
`Cargo.toml` ne compile rien de neuf. Sous Linux seulement.

**Écartés** : `zbus` en direct (son API bloquante, `blocking-api`, n'est pas activée : il aurait
fallu la recompiler, et `ashpd` écrit déjà l'appel) ; lancer `gdbus` (un programme qui n'est pas
toujours là, et un processus par lecture) ; ne rien lire (la charte d'accessibilité des
navigateurs fait de ce réglage un devoir : `prefers-reduced-motion`).

**Si `rfd` quittait `ashpd`** : la lecture tient en un appel D-Bus ; `zbus` seul suffirait.
