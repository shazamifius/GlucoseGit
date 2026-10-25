# 82 — La fin que le système demande, et la fenêtre sourde d'OpenGL

> Session du 25/10/2026. Sa parole d'ouverture : *« j'aimerais mettre une pause à Protéo et
> essayer de définitivement finir la partie 1 de Glucose : le logiciel complet, complet, en
> Rust, ultra optimisé, avec des utilisateurs et une ergonomie parfaite »*. SUITE § 0 en tire la
> ligne d'arrivée (la 2.1.0, six conditions) et cinq étapes ; voici le premier chantier de
> l'étape 1, « ne rien perdre ».

## 1. Ce que la télémétrie a montré

Relevée au début de la session (`node outils/telemetrie/tableau.mjs`, après `npx wrangler whoami`
pour l'erreur 7403) : 125 sessions, 6 installations, 25 fins anormales. Parmi elles, chez un
utilisateur Windows qu'il ne connaît pas (une GTX 1660, 12 sessions du 20 au 23/10), **deux
sessions de 3 h 37 et de 7 h « interrompues », chacune suivie du démarrage du PC une minute
plus tard**. Son document (373 images) est revenu à chaque fois. Lecture : Windows éteint ou
redémarré avec Glucose ouvert — pas un plantage. Mais une extinction devrait être une fin
**propre**, et un journal qui confond les deux cache les vrais plantages.

## 2. Ce que le code faisait

* Le travail ne se perdait pas : le scribe synchronise le disque après chaque réveil (chaque
  geste, sur un disque rapide), et le texte en cours de frappe a son fichier.
* Mais **`winit` 0.30.13 ne traite pas `WM_ENDSESSION`** (lu dans sa source : aucune
  occurrence) : `exiting` n'était jamais appelé à l'extinction, et son commentaire affirmait
  couvrir « l'arrêt demandé par le système ». Sous Linux, personne n'écoutait `SIGTERM`. Sur
  Mac, AppKit annonce la fin (`applicationWillTerminate:`) et `winit` la transmet.
* `exiting` ne fermait pas le document : chaque chemin devait l'avoir fait avant lui.

La recherche (Microsoft) : répondre vite à `WM_QUERYENDSESSION`, ne jamais retenir l'extinction,
faire le ménage dans `WM_ENDSESSION` — le processus peut être tué dès que toutes les applications
ont rendu la main sur ce message ; `ENDSESSION_CLOSEAPP` y dit le Restart Manager (un installeur,
Windows Update), et l'application doit alors se fermer en sauvant sans rien demander.

## 3. Ce qui est fait

* **FIN-1** (`app/fin.rs`) : la fin, quelle qu'elle soit, s'écrit une fois — `finir` : le
  document écrit et synchronisé sans question, sauf si une réponse a déjà réglé son sort (« Ne pas
  enregistrer » ne doit pas faire renaître un brouillon) ; puis le presse-papiers, la chronique,
  la boîte noire. `exiting` l'appelle. Chaque étape ne travaille qu'une fois par elle-même (le
  document rend son écriture, la boîte noire se rend) : pas de drapeau, qui aurait été une garde
  aveugle.
* **FIN-SESSION-1, Windows** (`plateforme/fin/session_windows.rs`) : une procédure glissée devant
  celle de `winit` (la forme du pavé) **marque la fenêtre** (une propriété) puis passe un
  `WM_CLOSE` à la procédure d'avant ; `winit` appelle l'application avant de rendre la main, et
  `CloseRequested` sur une fenêtre marquée finit sans question (`finir_avec_la_session`). La
  marque part avec la fenêtre.
* **FIN-SESSION-1, Linux** (`plateforme/fin/signaux.rs`) : `SIGTERM`, `SIGINT`, `SIGHUP` — un
  gestionnaire qui n'écrit qu'un octet dans un tuyau (la seule chose permise), un fil qui le lit,
  réveille la boucle, qui part ; `exiting` finit. **Un second signal fait partir sur-le-champ** :
  un Glucose figé reste tuable, sans délai choisi. Pas de masque de signaux : il passerait aux
  programmes que Glucose lance.

## 4. L'essai réel a échoué — la fenêtre sourde

Huit épreuves vertes, onze sabotages tombés. Puis l'essai réel : un vrai Glucose sur un dossier
jetable (`LOCALAPPDATA`, `TEMP`, `TMP` redirigés), fermé par le **Restart Manager** de Windows
(`RmShutdown`, les mêmes messages qu'une extinction) — **351, Glucose ne part pas, aucune ligne de
fin**. Une trace dans la procédure : aucun message de fin de session ne lui parvenait.

Les fenêtres de premier niveau du processus, énumérées et interrogées une à une
(`SendMessageTimeout`) : toutes répondent, **sauf `wgpu Device Class`** et sa fenêtre de saisie.
Dans `wgpu-hal` 30.0.1 (`gles/wgl.rs`), le moteur OpenGL de Windows lance un fil qui crée une
fenêtre cachée de premier niveau, puis attend la fin de l'instance (`drop_rx.recv()`) **sans
jamais lire un message**. Glucose ouvrait tous les moteurs (`new_without_display_handle()`), alors
que `wgpu` choisit Vulkan dès qu'il existe : OpenGL était ouvert pour rien, et sa fenêtre sourde
bloquait le Restart Manager — et, selon toute vraisemblance, faisait paraître l'écran « cette
application empêche le redémarrage » à chaque extinction (à confirmer à son écran).

**FENETRES-1** (`present/moteurs.rs`) : les moteurs dans l'ordre — Vulkan, Metal, Direct3D 12 ;
OpenGL seulement si aucun n'offre de carte (les très vieilles machines, que la charte interdit
d'exclure ; leur fenêtre sourde reste, c'est le défaut de `wgpu`). Une seule définition, que la
présentation et le banc appellent.

## 5. Prouvé

* **En réel, sous Windows** : le Glucose corrigé reçoit `WM_QUERYENDSESSION` puis `WM_ENDSESSION`
  (`ENDSESSION_CLOSEAPP`), finit, sa boîte noire écrit `{"type":"fin"}`, le processus part en
  **287 ms**, et `RmShutdown` rend 0. **À l'envers**, la beta 6 installée, dans les mêmes
  conditions : 351, toujours là après 20 s, aucune fin.
* **Les épreuves** : FIN-1 (le travail écrit sans question, la boîte noire close ; un brouillon
  abandonné ne renaît pas) ; FIN-SESSION-1 (la marque précède la fermeture que reçoit `winit`,
  une extinction annulée ne ferme rien, la marque part avec la fenêtre, le vrai chemin finit sans
  question et sa preuve à l'envers où la croix demande ; le premier signal finit, le suivant fait
  partir ; un vrai `SIGTERM` réveille sans tuer, sous Linux) ; FENETRES-1, dans un processus à
  elle (aucune fenêtre sourde par les moteurs de Glucose — et, à l'envers, OpenGL en laisse deux).
* **Les sabotages** : 11 sur 11 (`outils/sabotages/fin-session-1.json`), 2 sur 2
  (`fenetres-1.json`).
* **En réel, sous Linux** (la distribution `Glucose-Linux`, Xvfb, un dossier jetable par
  `XDG_STATE_HOME` et `TMPDIR`) : un vrai Glucose reçoit `SIGTERM`, part de lui-même avec le code
  0, et sa boîte noire écrit `{"type":"fin"}` ; **à l'envers**, la même copie sans l'écoute des
  signaux est tuée (code 143), sans fin. Ce chemin passe par `exiting` : le passage d'`exiting` à
  `finir` est prouvé là. Clippy strict et les épreuves de la fin, vrai `SIGTERM` compris, sous
  Linux. (Deux pièges payés : la distribution n'avait pas `libxkbcommon-x11`, que `winit` demande
  sous X11 — installée, avec Mesa ; et `rsync` garde les dates de Windows, si bien que cargo a cru
  à jour le binaire de l'essai précédent — `touch` avant de construire.)

## 6. Ce qui reste

* **À son écran** : éteindre le PC avec Glucose ouvert — rien ne doit attendre, et la session
  suivante dit « fermée proprement ».
* **Le passage d'`exiting` à `finir`** ne s'éprouve pas sans boucle réelle (`ActiveEventLoop`) :
  prouvé en réel par le chemin de Linux, non saboté.
* **Dire le défaut à `wgpu`** (un ticket) : à sa parole — c'est publier.
* **`RegisterApplicationRestart`** : Windows rouvrirait Glucose après une mise à jour du système
  (si l'utilisateur l'a demandé dans ses réglages). Glucose rouvre déjà son dernier travail au
  lancement ; une idée, pas un manque.

## 7. Les leçons

* **Huit épreuves vertes et onze sabotages tombés ne disent pas que ça marche** : la fenêtre
  témoin tenait la place de `winit`, et rien dans les épreuves ne voyait qu'une autre fenêtre du
  processus empêchait le système d'arriver jusqu'à la nôtre. L'essai réel l'a dit.
* **Une bibliothèque peut poser une fenêtre de premier niveau qui ne lit jamais ses messages** :
  énumérer les fenêtres du processus et parler à chacune (`SendMessageTimeout`) — FENETRES-1.
* **Ouvrir un moteur qu'on n'emploie pas n'est pas gratuit** : n'ouvrir que ce qui sert, dans
  l'ordre où ça sert.
* **Un document peut affirmer un défaut corrigé** : SUITE et INVENTAIRE disaient Trans-domaines
  muet, alors qu'il dit « bientôt » depuis la fiche 71 — lire le code avant de prendre un
  chantier.
