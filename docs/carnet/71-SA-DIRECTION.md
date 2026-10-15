# 71 — Sa direction, l'inventaire, et le téléphone sans câble

> Session du 15/10/2026, après la fiche 70. Elle commence, à sa demande, par **un inventaire**
> (les questions qui attendaient sa parole, ce qui reste à faire) — on n'avait jamais eu le temps
> de parler. Ses réponses donnent, pour la première fois par écrit, **la direction de Glucose** :
> [`VISION.md`](../VISION.md).

## 1. Ce qui est vérifié en ouvrant

* Le téléphone n'est pas branché — et **il a perdu son câble**.
* **Il a lancé Glucose le 14/10 de 23 h 00 à 23 h 18** (1 059 s), sur une build construite à
  23 h 00 — après `5bb82ec` (schéma 4), avant `4827118` (schéma 5) : très probablement le schéma 4,
  à vérifier sur une copie de ses documents avant de toucher au format. Aucune fiche ne la dit. Sa
  chronique : 345 images sur 13 886 au-dessus de 10 ms (pire 24,1), un tressaut à 91 px au p99
  quand la durée d'une image change (×11,4), une latence de 21,9 ms au p99.
* `renderer/math.rs` : LF dans le dépôt (`i/lf`), CRLF seulement dans la copie de travail —
  sans conséquence.

## 2. Ses réponses, mot pour mot ou presque

* **Le téléphone** : *« j'ai perdu complètement mon câble »* ; un APK envoyé par plusieurs moyens et
  téléchargé au téléphone, même à la fibre, reste **en téléchargement infini**. Sa question :
  pourquoi, et comment le faire accepter gratuitement par Play Protect. Son plan : l'APK dans un
  zip, envoyé par Drive ou Discord, essayé, *« et prier pour que tout se retrouve sur
  Cloudflare »*.
* **Les essais** : *« si j'ai absolument tout testé, normalement 100 % des fonctionnalités sont
  validées »*.
* **« Créer un preset custom »** : *« on n'est pas du tout d'accord »* — un preset est l'interface
  entière (disposition, comportement, couleurs, raccourcis, fenêtres) : un preset de Glucose
  jamais vu, un PureRef, un Figma, un Miro. Puis toute sa vision : `VISION.md`.
* **Trans-domaines** : de la recherche, son raisonnement sur « domaine » n'est pas fini.
* **L'export PNG, et tout ce qui a deux réponses légitimes** : *« d'accord avec le système qui
  permettrait à l'utilisateur de TOUJOURS CHOISIR »*.
* **Le SVG** : *« super utile »*.
* **Les planches** : claires ou sombres, au choix de l'utilisateur ; **les champs** (dialogue,
  action, notes) sous chaque case : oui ; **« 2.3 »** : non, mais pouvoir choisir ; **le numéro**
  dans la bande ou dans le coin : les deux, au choix ; **le panneau Storyboard** pleine hauteur :
  ne gêne pas.
* **Le dossier** : *« double-clic + zoom, comme Tauri »* ; **le design des dossiers est à
  retravailler**.
* **Le dessin de la sélection** : E pour l'instant ; l'ergonomie, une grosse session quand toutes
  les fonctionnalités seront là. **Le défilement au bord** : plus tard.
* **Le pavé** (le déplacement à deux doigts) : *« franchement, c'est tellement mieux qu'avant »*.
* **Le mode référence** (opacité, clics qui traversent) : pas pour l'instant.
* La liste des capacités de PureRef à refaire : à venir.
