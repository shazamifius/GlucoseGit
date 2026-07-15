# glucose-mcp — brancher Claude sur tes cartes Glucose

Un serveur **MCP** qui donne à Claude la capacité de **lire, comprendre et
réorganiser** tes fichiers `.glucose`.

Pas de capture d'écran, pas de modèle de vision : Claude comprend l'architecture
de ton projet **par sa structure** — la géométrie, les zones, le graphe des
flèches. C'est déterministe, ça ne dépend d'aucun modèle installé chez toi, et
ça ne coûte rien en tokens.

**100 % local.** Rien ne sort sur le réseau. Le serveur lit et écrit tes fichiers
sur disque, point.

---

## Installation

### La façon la plus simple

```bash
claude mcp add glucose -- npx -y glucose-mcp
```

C'est tout. `npx` télécharge le pont au premier lancement. Pas de clone, pas de
compte, rien à compiler. Il te faut juste [Node.js](https://nodejs.org) ≥ 18.

Vérifie que ça a marché :

```bash
claude
> /mcp
```

Tu dois voir `glucose` dans la liste, avec ses outils.

### Si tes projets ne sont pas dans `~/Documents`

Le pont cherche tes `.glucose` dans `~/Documents` par défaut. Si tu les ranges
ailleurs — fréquent sur Linux, où `~/Documents` n'existe parfois même pas :

```bash
claude mcp add glucose --env GLUCOSE_ROOT=/chemin/vers/tes/projets -- npx -y glucose-mcp
```

Sans ça, `list_glucose_projects` répondrait « aucun projet » sur un disque qui en
est plein — un échec silencieux, le pire genre.

> `GLUCOSE_ROOT` ne fixe que le point de départ des recherches : tu peux toujours
> passer un chemin absolu à n'importe quel outil.

### Depuis le dépôt (pour développer)

```bash
git clone https://github.com/shazamifius/GlucoseGit.git
cd GlucoseGit && npm install
claude          # .mcp.json est lu automatiquement
```

⚠️ Prévois **442 Mo** de `node_modules` : tu installes toute l'application
Glucose, pas seulement le pont. Pour un simple usage, `npx` (35 Mo) est
12× plus léger.

### ⚠️ Après toute mise à jour : reconnecte

Le serveur démarre **une fois**, au lancement de ta session. Si tu mets le pont à
jour, la session en cours continue de faire tourner l'ancien code — et tu
déboguerais des bugs déjà corrigés. Fais `/mcp` pour reconnecter.

---

## L'idée, en une phrase

> **Toi** tu lis un dessin en 2D. **Claude** lit une structure.
> Le pont traduit l'un vers l'autre — sans jamais laisser la préférence linéaire
> d'une IA dicter ta mise en page.

---

## Le parcours normal

Les outils sont faits pour s'enchaîner :

```
1. list_glucose_projects   →  qu'est-ce que j'ai ?
2. read_glucose            →  qu'est-ce que ça raconte ?
3. analyze_architecture    →  comment c'est bâti ?
4. detect_organization     →  quelle logique j'ai suivie ?
5. optimize_layout         →  réorganiser SANS trahir cette logique
6. lint_layout             →  qu'est-ce que ça donne à l'écran ?
```

Une vraie conversation ressemble à ça :

> **Toi :** Regarde `~/Documents/cours.glucose` et dis-moi comment c'est organisé.
>
> **Claude** enchaîne `read_glucose`, `analyze_architecture` et
> `detect_organization` — et te répond que c'est une frise chronologique à
> 5 époques datées, que le nœud central est « Neurobiologie musicale » (6 liens),
> et que 3 notes ne sont reliées à rien.
>
> **Toi :** Range-le mieux, mais garde la chronologie.
>
> **Claude** appelle `optimize_layout` en mode chronologique **sur une copie**,
> puis `lint_layout` pour vérifier le rendu.

---

## Les 11 outils

### 📖 Lire — ils ne touchent jamais à tes fichiers

#### `list_glucose_projects`
Liste tous les `.glucose` sous un dossier, avec un aperçu décodé de chacun : nom,
nombre de boards, de notes, de flèches. **Le point d'entrée.**

> « Quels projets Glucose j'ai ? »

#### `read_glucose`
Sort un digest lisible en Markdown : les notes dans l'ordre de lecture, les
relations (`source → prédicat → cible`), les zones.

- `includeText: false` → structure seule, sans le texte (pour les gros projets)
- `includeIds: true` → préfixe chaque note de son `id`. **Indispensable avant
  `connect_notes` ou `apply_layout`**, qui désignent les notes par id.

> « Résume-moi ce projet. »

#### `search_glucose`
Cherche une expression à travers **tous** tes projets d'un coup, et te dit
lesquels la contiennent, avec le passage.

> « Dans quel projet j'avais parlé de Zatorre ? »

#### `analyze_architecture`
**Le cœur du pont.** Comprend comment le projet est *bâti*, sans le voir :

- quelles notes tombent dans quelle zone (inclusion géométrique)
- les regroupements spatiaux hors zone
- la forme du graphe : chaîne, étoile, réseau
- les **hubs** (nœuds très connectés), les **racines**, les **feuilles**, les
  **isolées**
- la hiérarchie des boards, et les dossiers-miroirs

> « Comment ce projet est-il structuré ? »

#### `detect_organization`
Détecte la logique que **tu** as déjà suivie : chronologie, thématique, hub,
chaîne, ou non structuré — d'après les dates, l'axe temporel, la forme du graphe.

Sert à **respecter ton intention** au lieu de plaquer une recette. C'est ce qui
empêche une IA de transformer ta frise en liste sous prétexte qu'elle préfère les
listes.

> « Quelle est la logique de ce board ? »

#### `lint_layout`
**Le contrôle qualité visuel, sans vision.** Calcule les défauts qu'on *verrait* à
l'écran : notes qui se chevauchent, flèches qui masquent une note, croisements.

Sa particularité : **il juge selon l'intention** (via `detect_organization`). Dans
une frise, une flèche longue *le long du temps* est le message — pas un défaut.
Seul son écart *en travers* est facturé.

Le rapport annonce toujours le **mode retenu** et l'**étalon** qui ont servi à
juger, avec une empreinte : deux scores ne sont comparables que si l'empreinte est
identique. Deux modes = deux monnaies.

> « Est-ce que ça se lit bien ? »

---

### ✍️ Écrire — ils sauvegardent toujours avant

> **Les 3 règles de sécurité**, appliquées par tous les outils d'écriture :
>
> 1. `<fichier>.orig.bak` est écrit **une seule fois**, jamais réécrit : ton état
>    d'avant la toute première écriture reste récupérable pour toujours.
> 2. `<fichier>.<horodatage>Z.bak` garde les 3 derniers états.
> 3. `outPath` te laisse écrire dans une **copie** sans jamais toucher la source.
>    **Utilise-le.**

#### `create_glucose_project`
Crée un `.glucose` neuf à partir d'un titre et d'une liste de notes, disposées en
colonnes lisibles. Refuse d'écraser un fichier existant sauf `overwrite: true`.

> « Fais-moi un Glucose avec ces 12 idées. »

#### `add_note`
Ajoute une note (`text` ou `sticky`). Position automatique sous le contenu
existant si tu ne précises rien.

> « Ajoute une note "à creuser : l'octave chez le bébé". »

#### `connect_notes`
Relie deux notes par une flèche. Tu désignes les extrémités par `id` (fiable) ou
par un bout de texte (pratique).

Ce qui le rend spécial : **`sourceSel` / `targetSel`**. La flèche part d'une
**phrase précise** dans la note, pas du bloc entier — le texte est souligné et la
flèche le pointe.

Prédicats disponibles : `est_precurseur`, `contredit`, `herite_de`, `inspire`,
`depend_de`, `illustre`.

> « Relie la phrase "prédiction" de la note A à la note B, prédicat
> `est_precurseur`. »

#### `apply_layout`
Applique une réorganisation **en un seul coup, atomiquement** : retire des
annotations, déplace des notes, crée des zones, ajoute des flèches, patche des
propriétés. C'est l'outil qui matérialise une proposition.

Désigne le board par **`boardId`**, pas par son nom : les noms de boards ne sont
pas uniques.

#### `optimize_layout`
Réorganise automatiquement, **en respectant l'intention détectée** :

| mode | ce qu'il fait |
|---|---|
| `auto` | détecte l'intention et applique le layout qui la sert *(défaut)* |
| `chronological` | frise : époques ordonnées gauche→droite par année |
| `thematic` | territoires 2D par sujet (force + clusters) |
| `hub` | radial : le nœud central au milieu, satellites autour |
| `linear` | colonne / parcours de lecture |

Toute la géométrie est **déterministe** : intersection segment-segment, union-find,
force dirigée à graine fixe, extension harmonique sur le graphe. Même entrée =
même sortie, toujours.

> « Réorganise ça en frise, dans une copie. »

---

## Les pièges qui font perdre du temps

**Le board « actif » est souvent le mauvais.** Dans les vrais projets,
`activeBoardId` pointe fréquemment sur un board vide — le dernier consulté. Tous
les outils choisissent donc le board **le plus structuré** (le plus de flèches),
pas le board actif. Passe `boardId` si tu veux être certain.

**Ferme Glucose avant d'écrire.** L'app enregistre automatiquement ~1,5 s après
une modification. Si elle est ouverte sur le fichier que le pont modifie, elle
écrasera l'écriture du pont.

**Les vieux `.glucose` (JSON v1) fonctionnent.** Le pont les lit et les convertit
en v2 binaire à la première écriture, exactement comme le fait l'app. L'original
reste dans `.orig.bak`.

---

## Ce que le pont ne sait pas faire (encore)

Une liste honnête vaut mieux qu'une mauvaise surprise :

- **Les images et vidéos sont comptées, jamais placées.** `optimize_layout` ne
  calcule la géométrie que sur les annotations : sur un board illustré, il peut
  empiler des notes sur tes médias — et `lint_layout` ne le verra pas.
- **Les membranes sont recréées, pas déplacées** : les champs `domains`,
  `temporalAnchor` et `mirrorOf` d'une zone sont perdus lors d'un
  `optimize_layout`.
- **Le lint n'est pas calibré sur un humain.** Ses poids sont posés à la main. Il
  lui arrive de préférer une mise en page qu'un humain trouve illisible.
- **Aucun test automatisé ne couvre le pont** pour l'instant.

---

## Le pair vivant (expérimental)

`glucose-peer.mjs` rejoint une session **collaborative** Glucose en direct, via le
même serveur de synchronisation que l'app. Claude devient un participant : ce
qu'il écrit apparaît sur ton canvas en temps réel.

```bash
node glucose-peer.mjs <automerge:url>              # lire le doc vivant
node glucose-peer.mjs <automerge:url> --say "..."  # écrire une note, en direct
node glucose-peer.mjs                              # auto-test de connectivité
```

Non inclus dans le paquet npm : c'est un chantier en cours, disponible dans le
dépôt.

---

## Tester à la main, sans Claude

Le serveur parle JSON-RPC 2.0 sur stdio, une requête par ligne :

```sh
printf '%s\n' \
  '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{}}}' \
  '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"list_glucose_projects","arguments":{}}}' \
  | npx -y glucose-mcp
```

---

## Sous le capot

Un `.glucose` est un document [Automerge](https://automerge.org/) (CRDT) binaire —
illisible à l'œil. Le pont le décode avec le même Automerge que l'app et en sort
une structure.

Le protocole MCP est **implémenté à la main** dans un seul fichier, sans aucune
dépendance en dehors d'Automerge. Tu peux le lire en entier :
[`glucose-mcp.mjs`](glucose-mcp.mjs).

## Licence

MIT
