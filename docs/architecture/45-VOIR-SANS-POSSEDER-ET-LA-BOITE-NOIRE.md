# 45 — Voir sans posséder, et la boîte noire

> **Rôle de ce document.** Les deux premières phases de la route de la fiche
> [`44`](44-LA-ROUTE-VERS-TOUTES-LES-MACHINES.md), faites le 29/09 : **voir** Glucose sur les
> machines qu'on ne possède pas, et **la boîte noire** — la première moitié, celle qui écrit sur
> la machine ; ce qui voyage vers lui (l'accord, le serveur) est la suivante. **Écrite au fil
> de l'eau.**
>
> **Date** : 2026-09-29 · commits `2c8c7ea` à ceux de cette fiche.
> **État à la fin** : sous Windows, **1 920 épreuves vertes**, clippy strict à zéro, l'application
> se construit, `%LOCALAPPDATA%\Glucose` n'a rien reçu. Sur les machines de GitHub : § 1.

---

## 0. En une page

| | ce qui change pour lui | commit |
|---|---|---|
| **Voir** | à chaque envoi, Glucose se compile et s'éprouve sur Windows, Linux, Mac à puce Apple, Mac Intel, et son noyau pour les téléphones Android — sans rien coûter | `2c8c7ea` |
| **Portable** | trois morceaux propres à Windows n'existent plus ailleurs ; cinq épreuves de l'offre de mémoire sont dites propres à Windows, avec leur raison | `e4fd2ea`, `86cc1fd` |
| **Honnête** | là où une carte graphique est promise, une épreuve graphique ne peut plus se sauter en silence | `86cc1fd` |
| **La boîte noire** | chaque session a son fichier, écrit au fil de l'eau et synchronisé ; au lancement suivant, Glucose dit comment la session d'avant a fini | `a884e6b` |
| **Son presse-papiers** | les épreuves ne touchent plus au presse-papiers de sa machine : chaque `cargo test` remplaçait ce qu'il venait de copier | ce lot |

---

## 1. Ce que la vérification a montré, dès sa première exécution

Coupée le 15/09 « parce qu'elle facturait » : le dépôt est public, et GitHub Actions n'a rien
coûté — 4,94 $ bruts en septembre, **entièrement remisés**, 0 $ net (vérifié par l'API de
facturation). Rallumée sur quatre systèmes, et le noyau pour Android :

* **Android** : le noyau et les maths passent clippy strict pour **ARM 64 bits et ARM 32 bits** —
  les téléphones d'aujourd'hui et les très anciens. Le cœur de Glucose est prêt pour eux.
* **Linux, Mac à puce Apple, Mac Intel** : clippy strict ne tombait que sur **trois** morceaux de
  code sans appelant hors de Windows — le pont du pincement, et une adresse qui ne servait qu'à
  une épreuve propre à Windows. Ils n'existent plus que sous Windows.
* **Les épreuves** : Windows entièrement vert **sur la machine de GitHub** (le même compte que
  chez lui) ; sur Linux et les deux Mac, **1 030 réussies, 5 tombées** — toutes celles qui
  supposent qu'une image puisse être **offerte au système** quand l'écran ne la montre plus.
  Cette offre n'existe encore que sous Windows : ailleurs, le code la refuse, volontairement.
  Ces cinq épreuves sont **ignorées hors de Windows, avec leur raison écrite** — elles paraissent
  « ignorées » dans chaque rapport, et non vertes en silence. L'offre viendra avec les phases 3
  (Linux : `madvise`) et 8 (Mac).
* **Une panne de GitHub, pas de Glucose** : une fois, le Mac Intel n'a pas pu joindre le
  registre des bibliothèques Rust. Le lot suivant l'a relancé.

**Une épreuve graphique ne ment plus en vert.** Sur une machine sans carte, elles se sautaient —
sans rien dire, et le lanceur d'épreuves cache la sortie de celles qui passent : une chaîne verte
ne prouvait pas qu'elles avaient tourné. Sur GitHub, une carte logicielle est promise partout
(WARP, Mesa, Metal) et `GLUCOSE_EXIGER_UNE_CARTE` le dit : là, une épreuve sans carte **tombe**.
Un seul endroit décide ([`banc_gpu::sans_carte`](../../crates/glucose-desktop/src/present/banc_gpu.rs)) ;
la copie du banc dans les épreuves de la scène a disparu.

**Vérifié sur GitHub** (`a884e6b`, la carte exigée) : **Windows, Linux, Mac à puce Apple et
Android passent entièrement** — les épreuves graphiques ont donc vraiment tourné sur WARP, sur
Mesa (lavapipe) et sur Metal. Le Mac Intel n'a pu être jugé : la panne de réseau de GitHub.

---

## 2. La boîte noire (phase 2, sa première moitié)

### 2.1 Pourquoi, en plus de la chronique

*« Un système de journal, pour un projet de R&D et un début d'application, c'est
obligatoire »* — pour les plantages, les bugs, et *« voir où en est Glucose sur les très anciens
téléphones, lorsqu'il commence à chauffer, lorsqu'il s'éteint à cause de la batterie »*.

La chronique agrégeait une session dans **un seul** fichier, que la session suivante écrasait : la
trace d'une session qui avait mal fini se perdait, sauf à copier son fichier avant de relancer (la
fiche 43 le lui demandait). Et une agrégation ne montre pas le temps : un téléphone qui chauffe
coûte de plus en plus cher pour le même travail, ce qu'aucun centile de fin de session ne dit.

### 2.2 Ce qu'elle écrit

Dans `%LOCALAPPDATA%\Glucose\boite-noire\` (ailleurs, le dossier de l'application), un fichier
par session, `session-<début>-<processus>.jsonl`, une ligne JSON par enregistrement :

* **le début** : la version, le système, l'architecture, l'heure, et le démarrage de l'appareil ;
* **l'épisode** : une suite d'images d'un même geste — combien, la médiane, le p99, la pire. Il
  s'écrit **quand le geste change** : aucune période n'est choisie ;
* **le record** : une image plus lente que toutes les précédentes de la session — l'instant où la
  session a quelque chose de plus à dire (la règle de `Chronique::du_neuf`) ;
* **la machine** : la batterie et le secteur, **seulement quand ils changent** ;
* **la panique** : où, dans le code de Glucose, et sur quel fil — **jamais son message**, qui peut
  citer une donnée ;
* **la fin propre**, à la fermeture ;
* **la précédente** : comment la session d'avant a fini, pour que ce bilan voyage avec celle-ci.

### 2.3 Survivre à la fin qu'elle doit expliquer

Le fil d'écriture suit la règle du scribe ([`persist::scribe`](../../crates/glucose-desktop/src/persist/scribe.rs)) :
il dort tant que rien n'arrive, écrit tout ce qui attend, **synchronise le disque**, se rendort.
La fréquence des synchronisations suit la vitesse du disque, sans constante. Ce qui a été
synchronisé survit à un plantage comme à un appareil qui s'éteint. Les sondes de la machine se
lisent sur ce fil, jamais sur celui qui dessine.

### 2.4 La confidentialité se lit dans le type

Un enregistrement ne porte que des nombres, des booléens et des noms **fixés à la compilation** :
le type ne peut pas transporter un mot de l'utilisateur, un nom de fichier, un chemin. **Une seule
exception, dite** : la ligne d'une panique porte le chemin du fichier source de Glucose où elle a
eu lieu — écrit par le compilateur, jamais une donnée — ; c'est pourquoi elle n'est pas un
enregistrement comme les autres.

### 2.5 Comment la session d'avant a fini — ce qu'on peut dire, et rien de plus

* **fermée proprement** : sa dernière ligne le dit ;
* **tombée sur une panique** : du fil principal — un autre fil peut tomber sans emporter
  Glucose, et la session continuer jusqu'à sa fermeture ;
* **arrêtée sans rien dire** : tuée, plantée hors de Rust (un pilote), gelée puis fermée de force
  — ou l'appareil s'est éteint. **Ce dernier cas se reconnaît à une chose sûre : sa dernière trace
  précède le démarrage de l'appareil présent.** On le dit tel quel, *« l'appareil a redémarré
  depuis »*, avec la dernière batterie écrite ; conclure « la batterie » serait deviner.

Au lancement, la console le dit : `[Glucose] la session précédente : arrêtée sans rien dire,
après 12 min 3 s ; l'appareil a redémarré depuis — dernière batterie : 3 %, sur batterie`.

### 2.6 Ce qui reste sur le disque — sans aucun nombre choisi

La session d'avant, quelle que soit sa fin ; **toute session qui a mal fini** ; une session encore
tenue par un autre Glucose. Une session plus ancienne qui a fini proprement s'efface : elle a été
racontée au lancement qui la suivait. **Deux Glucose ouverts à la fois** : chaque session est
tenue sous le verrou des documents ([`persist::verrou`](../../crates/glucose-desktop/src/persist/verrou.rs)) ;
celle de l'autre fenêtre n'est jamais prise pour une session interrompue.

### 2.7 Ce qui la tient

Onze épreuves : une session fermée se relit propre, avec ses épisodes ; une session copiée en
plein vol — ce qu'un plantage laisserait — se relit interrompue ; l'appareil redémarré se dit
d'après la dernière trace, et jamais pour une session fermée ; une ligne coupée ne cache pas ce
qui la précède ; seule une panique du fil principal finit la session ; le rangement ; chaque ligne
est du JSON ; seuls les records s'écrivent ; la machine ne s'écrit que si elle change ; les
sondes répondent ; **le témoin note une vraie panique** — sans son message.

**Seize sabotages tombent — après correction de deux épreuves aveugles**, trouvées ainsi : une
fin propre écrite *après* la panique d'un autre fil masquait la différence ; la ligne de fin
décalait la dernière trace *après* le démarrage choisi.

### 2.8 Ce qui manque encore — dit, pas caché

* **Ce qui voyage** (phase 2, seconde moitié) : l'écran d'accord, « voir ce qui part », l'envoi
  chiffré, le serveur — sa part : un nom de domaine, le HTTPS devant sa box.
* **Les plantages hors de Rust** — un pilote de carte graphique qui tombe : le processus témoin
  (`crash-handler`, `minidumper`) ; aujourd'hui, ils se lisent « arrêtée sans rien dire ».
* **La chaleur** : Android la dit (phase 6) ; Windows n'a pas d'interface simple pour elle.
* **Les sondes du Mac** (phase 8) : d'ici là, elles disent qu'elles ne savent pas.
* **Un très long épisode** — une heure d'écriture sans changer de geste — ne relève la machine
  qu'à ses bords. Android écrira ses changements par ses propres événements.
* **Le coût de la synchronisation sur un téléphone** n'est pas mesuré.

---

## 3. Son presse-papiers, que les épreuves remplaçaient

Trouvé en passant, en éprouvant sous Linux : l'épreuve de `Ctrl+C` et `Ctrl+X` passait par le
**vrai presse-papiers de Windows** — à chaque `cargo test` sur sa machine, ce qu'il venait de
copier était remplacé par le texte de l'épreuve ; et un collage lisait ce qu'il y avait mis. La
même faute que les fenêtres ouvertes sur son écran, en plus discrète.

Le presse-papiers se choisit désormais comme son dossier : **l'application prend celui du
système au lancement** (`main.rs`) ; sans cela, chaque fil a le sien, en mémoire
([`interactions::presse_papiers`](../../crates/glucose-desktop/src/interactions/presse_papiers.rs)).
Le collage d'un texte, qui n'avait aucune épreuve, en a une. Trois sabotages tombent — un
quatrième ne tombait pas parce que **mon** sabotage était mal construit, non l'épreuve. Linux
n'a plus besoin d'écran virtuel.

---

## 4. Ce qu'il faut regarder à l'écran

```text
cargo run --release > sortie-boite-noire.txt 2>&1
```

1. **Lancer, utiliser, fermer par la croix, relancer** : la console dit *« la session
   précédente : fermée proprement »*. Le dossier `%LOCALAPPDATA%\Glucose\boite-noire\` contient
   un fichier par session.
2. **Fermer Glucose par le Gestionnaire des tâches**, puis relancer : *« arrêtée sans rien
   dire »*.
3. **Copier un texte ailleurs**, puis me laisser lancer les épreuves : il est toujours dans ton
   presse-papiers après.

---

## 5. Les sources

Celles de la fiche 44 § 6 — GitHub, wgpu, `crash-handling`, Android — ; et :

* Rust, [`File::sync_data`](https://doc.rust-lang.org/std/fs/struct.File.html#method.sync_data)
  — ce que la synchronisation garantit.
* Microsoft, [`GetSystemPowerStatus`](https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-getsystempowerstatus)
  et [`GetTickCount64`](https://learn.microsoft.com/en-us/windows/win32/api/sysinfoapi/nf-sysinfoapi-gettickcount64)
  — la batterie, et le temps depuis le démarrage, sommeil compris ; Linux,
  [`/sys/class/power_supply`](https://www.kernel.org/doc/html/latest/power/power_supply_class.html)
  et `proc(5)` pour `/proc/uptime`.

---

**Retour** : [`00-INDEX.md`](00-INDEX.md)
