# 36 — La route vers la V1

> **Rôle de ce document.** L'utilisateur, le 24/09 au soir : *« j'aimerais vraiment beaucoup
> avancer et commencer à pouvoir publier Glucose ; le problème c'est qu'il y a même pas 25 % des
> fonctionnalités faites »*. Il veut la mise à jour automatique — et que tous ceux qui ont Glucose
> Tauri passent à Glucose Rust par le popup habituel —, tout le système de Tauri refait en Rust,
> l'intégration complète au MCP et au co-working, et **tout** le système d'enregistrement. Il
> demande la liste de ce qui reste, dans l'ordre que je choisirais. **Aucun code dans cette
> fiche** : c'est la carte des sessions à venir.
>
> **Date** : 2026-09-24 · au commit `ac89af6`.

---

## 1. Où l'on en est, mesuré

### 1.1 Sa dernière session (`sortie-vol.txt`, 219 s)

* **Validé à l'écran** : le vol (`F`, signets) — *« tout ce qui est du dézoom et zoom c'est
  parfait, tout ce qui est timing c'est vraiment parfait »*.
* **« Ça lag beaucoup encore »**, et la chronique le confirme : 43 images par seconde entre deux
  images, tempo à 5-6 balayages, latence 33 ms au p99. Le plancher de la charte est 100.
  * **Nouveau suspect n° 1** : les **panneaux** (`docks`) coûtent 21 à 34 ms pendant un zoom,
    alors que la chronique ne leur connaît presque aucune raison de se redessiner — sept des
    douze pires images.
  * Le reste est un **coût de base** : une image ordinaire coûte 5 à 7 ms, répartis sur une
    dizaine de postes de 0,5 à 1 ms (`blit`, `soumettre`, `effacer`, `relever`, `present`,
    `textures`) — fiche 34 § 2.2.
* Le gel de 1,8 s : `Ctrl+S`, encore. La mémoire tient : 436 Mo.

### 1.2 La parité avec Glucose Tauri

La fiche [`14`](14-ETAT-ET-RESTE.md) (16/09) le mesurait : **≈ 34 % des gestes, ≈ 25 % du
logiciel** — son ressenti est juste. Depuis, dix-neuf sessions ont surtout porté sur le rendu,
la mémoire et les images (voies graphiques, pyramides, mémoire par étages, dépôt depuis un
navigateur, recadrage, `Ctrl+B`, vol) : la **qualité** a beaucoup bougé, la **parité** peu.
Neuf domaines sont toujours à zéro : rideaux, temporalité, storyboard, presets et zones,
export, miroirs, collaboration, plugins, divers.

### 1.3 Ce que la bascule depuis Tauri exige — vérifié le 24/09

| | ce qui existe | ce que ça implique |
|---|---|---|
| **Le canal de mise à jour** | Tauri lit `releases/latest/download/latest.json` **de ce dépôt**, vérifie une signature minisign, lance un installeur NSIS, relance (`UpdatePrompt.tsx`) | la bascule peut passer par le popup habituel ; **toute release marquée « latest » sur ce dépôt est lue par les applications Tauri installées** |
| **La clé de signature** | `~/.tauri/glucose_updater.key`, du 4 juillet | **à sauvegarder ailleurs dès maintenant** : perdue, plus aucune mise à jour ne peut atteindre un utilisateur de Tauri ; et il faudra son mot de passe |
| **Les plateformes** | la 1.0.2-beta.1 (09/09) existe pour **Windows, macOS, Linux** (AppImage, deb, rpm, NixOS) | Glucose Rust ne tourne que sous Windows : `latest.json` peut viser Rust pour Windows seulement, ou attendre macOS et Linux |
| **Les documents** | les `.glucose` de Tauri sont en Automerge ; Glucose Rust ne les ouvre pas (cinq refusés sur son disque) | **sans importeur, un utilisateur qui bascule perd l'accès à ses documents** |
| **Le MCP** | 11 outils (Node) qui lisent et écrivent le format **Tauri**, et un « pair IA » qui rejoint une session de co-working | à refaire sur le format Rust |
| **Le co-working** | Automerge + le serveur public `sync.automerge.org`, curseurs, identité, canal des images | à refaire, sur le même fondement que l'enregistrement |
| **La télémétrie** | performance seulement (images/s, gels, GPU, système), opt-in, sur son serveur NixOS | elle dit sur quels systèmes sont ses utilisateurs, pas quelles fonctionnalités ils emploient |

---

## 2. L'ordre que je choisirais, et pourquoi

Quatre principes : **ne jamais perdre un document**, **tenir la promesse** (la fluidité) avant
d'attirer du monde, **fonder une fois** ce que plusieurs chantiers partagent, et **ce qui est
écrit et éteint avant ce qui n'existe pas**.

Poids : ○ un geste · ◐ un chantier · ● un sous-système · ●● une vague.

### Phase 1 — Le document ●● *(la fondation : tout le reste s'appuie dessus)*

| # | Chantier | Poids | Pourquoi ici |
|---|---|:--:|---|
| 1.1 | **Une conception unique : le journal.** Enregistrement, retour dans le temps, co-working et MCP écrivent et lisent la même chose — un geste. Choisir, par un banc sur ses documents : le journal actuel du noyau (JRN-1) et une synchronisation écrite ici, ou **Loro** (co-édition et voyage dans le temps intégrés, le plus rapide des bancs), ou **Automerge** (le format de Tauri, qui lirait aussi ses anciens fichiers) | ◐ | concevoir l'enregistrement sans le co-working, c'est le refaire deux fois |
| 1.2 | **L'importeur des fichiers Tauri** | ◐ | condition de toute bascule — et **aucun vrai document n'a jamais été ouvert** dans Glucose Rust (fiche 14 § 5) |
| 1.3 | **Le magasin d'images par empreinte**, hors du dossier temporaire | ◐ | supprime le risque de perte (fiche 33 § 5.1) |
| 1.4 | **Journal en ajout seul** : enregistrement instantané et automatique, récupération après plantage | ● | supprime le gel de `Ctrl+S` |
| 1.5 | **Le retour dans le temps** : instantanés, jalons nommés, Time Machine (réglette, aperçu ambré, restaurer) | ● | ce qu'il a demandé, et que Tauri avait |

Décision à lui : **où vit l'histoire** — dans le fichier (recommandé) ou dans un dossier à côté.

### Phase 2 — La fluidité ● *(« ça lag »)*

| # | Chantier | Poids |
|---|---|:--:|
| 2.1 | **Les panneaux** à 21-34 ms pendant un zoom | ◐ |
| 2.2 | **Le coût de base d'une image** : viser 8,3 ms — deux balayages à 240 Hz, 120 images par seconde | ● |
| 2.3 | L'épreuve de la carte qui tombe au hasard (fiche 35 § 4) | ○ |

Avant la parité, parce qu'un utilisateur de Tauri qui passe à une version saccadée en garde la
première impression — et que c'est **la** raison d'être de Glucose Rust.

### Phase 3 — La distribution ◐◐ *(peut avancer en parallèle dès maintenant)*

| # | Chantier | Poids | Note |
|---|---|:--:|---|
| 3.1 | **L'installeur Windows** de Glucose Rust | ◐ | le jour de la bascule, il remplace proprement Tauri et garde les données |
| 3.2 | **La mise à jour automatique** de Glucose Rust, avec son popup | ◐ | recommandation : le **même format** que Tauri (`latest.json` signé minisign — la vérification tient en une petite caisse) : continuité, clé unique, presque rien de plus ; **Velopack** si l'on veut des mises à jour par différences |
| 3.3 | La construction et la publication **automatiques** (GitHub Actions) | ◐ | l'intégration continue de Tauri est désactivée : ses versions ont été signées à la main |
| 3.4 | Un **canal bêta** : Glucose Rust installable **à côté** de Tauri, sans bascule | ○ | des retours avant de forcer quiconque — **sans jamais marquer « latest »** une release qui porterait un `latest.json` |
| 3.5 | La signature de code Windows (SmartScreen) | ◐ | un certificat payant — **décision à lui** |

### Phase 4 — La parité fonctionnelle ●●● *(le gros morceau : trois quarts du logiciel)*

Dans cet ordre, repris de la fiche 14 § 4 et mis à jour :

| # | Lot | Poids | Pourquoi dans ce rang |
|---|---|:--:|---|
| 4.1 | **Ce qui est écrit et éteint** : exports SVG et Markdown sur le disque, `mirror_graph`, `timeline`, rideaux (426 l.), membranes (1 442 l.) | ◐ | le meilleur rendement du dépôt |
| 4.2 | **Les membranes possèdent enfin** (`membrane_id`), grandissent, mode focus, couleur dérivée des domaines | ● | débloque un domaine entier à 12 % |
| 4.3 | **Le texte** : sélection souris et clavier, ancres (une flèche depuis une phrase), saisie IME | ● | débloque les flèches vers une phrase |
| 4.4 | **Naviguer dans l'immense** : `Ctrl+F`, la couleur des domaines | ◐ | ce que la charte nomme |
| 4.5 | **Dossiers et miroirs**, dont le miroir d'un dossier du disque | ● | |
| 4.6 | **Rideaux** (la fonctionnalité la plus originale), **temporalité** (réglette de −10 000 à 2 100), **storyboard**, **presets et zones** | ●● | quatre domaines à zéro |
| 4.7 | **L'interface** : sélecteur de couleur, panneau Ordonner complet, boards (renommer, fermer, réordonner), infobulles, barre d'état | ● | |
| 4.8 | **Images** : vidéos, provenance (SauceNAO), tags | ● | |
| 4.9 | **Flèches** : portails, courbes, épaisseur et couleur | ◐ | |
| 4.10 | Export HTML et PNG | ◐ | |

### Phase 5 — Le co-working ●● *(sur le journal de la phase 1)*

Session partagée, curseurs, identité, canal des images, droits des rideaux (privé, partagé),
et le **serveur** : Tauri passait par le relais public d'Automerge ; Rust pourra passer par un
relais à lui, sur la même machine NixOS que la télémétrie.

### Phase 6 — Le MCP ●

Réécrit **en Rust sur le noyau**, avec le kit officiel `rmcp` : les 11 outils sur le format
Rust — possible dès la phase 1, puisqu'ils ne demandent que le format —, puis le **pair IA en
direct** par le co-working (après la phase 5). Un seul exécutable, plus besoin de Node.

### Phase 7 — Plugins, App Bridge, IA locale (Ollama), divers ●

### Phase 8 — La bascule ◐

`latest.json` publié, signé de la clé de Tauri, qui fait passer ses utilisateurs à Glucose Rust
par le popup habituel — **par plateforme** : Windows d'abord, macOS et Linux quand Rust y
tournera (phase 9), ou tout ensemble si l'on attend.

### Phase 9 — macOS et Linux ●●, puis Android ●●

La couche graphique couvre déjà Metal et Vulkan ; ce qui est propre à Windows est à écrire
pour chacun : la cible de dépôt, l'offre de mémoire, le téléchargement, le budget de la carte.

### Au-delà de la V1

La fondation des 10⁷ nœuds (l'arène), et le but du projet : le canva Wikipédia de
`glucose-brain`.

---

## 3. Ce que je dois lui dire franchement

1. **Le volume.** Trois quarts du logiciel restent : un ordre de grandeur de **plusieurs
   dizaines de sessions** comme celle du 24/09, pas quelques-unes. Deux accélérateurs réels :
   * **des sessions en parallèle** sur des domaines indépendants (chacune dans son arbre de
     travail) une fois la phase 1 posée — les domaines de la parité se touchent peu ;
   * **prioriser par l'usage réel** : ce que ses utilisateurs emploient vraiment passe avant le
     reste. La télémétrie ne le mesure pas ; lui le sait peut-être.
2. **Faire basculer tout le monde avant la parité**, c'est retirer à chacun les fonctionnalités
   qui manquent encore — une régression imposée. D'où la recommandation de **deux moments
   distincts** : une V1 publique de Glucose Rust (bêta, Windows, à côté de Tauri), puis la
   bascule automatique quand la parité est atteinte.
3. **Un accident possible dès aujourd'hui** : une release « latest » de ce dépôt avec un
   `latest.json` ferait basculer les utilisateurs de Tauri — même par erreur.
4. **La clé** de `~/.tauri/` doit être sauvegardée hors de cette machine.

---

## 4. Ce qui attend sa parole

* **V1 et bascule : un seul moment, ou deux** (§ 3.2) — la question qui décide de l'ordre.
* Où vit l'histoire du projet (§ 2, phase 1).
* La signature de code Windows (phase 3.5).
* Ce que ses utilisateurs emploient le plus (§ 3.1).

---

## 5. Les sources

* [Velopack](https://github.com/velopack/velopack) — installeur et mises à jour par
  différences, écrit en Rust, multiplateforme.
* [Loro](https://loro.dev/) — co-édition en Rust, voyage dans le temps intégré ; comparatif
  [Yjs, Automerge, Loro](https://www.pkgpulse.com/guides/yjs-vs-automerge-vs-loro-crdt-libraries-2026).
* [`rmcp`](https://github.com/modelcontextprotocol/rust-sdk) — le kit officiel du MCP en Rust.

---

**Retour** : [`00-INDEX.md`](00-INDEX.md)
