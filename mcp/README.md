# glucose-mcp — brancher Claude sur tes projets Glucose

Un petit serveur **MCP** (Model Context Protocol) qui donne à Claude la capacité
de **lire et explorer tes fichiers `.glucose`** — pour t'aider à organiser tes
projets — sans qu'il ait à comprendre le format binaire.

- **100 % local** : rien ne sort sur le réseau. Le serveur lit/écrit tes fichiers
  sur disque, point.
- **Lecture + écriture** : il liste, lit, cherche, **et** peut créer un `.glucose`
  ou y ajouter des notes/flèches. Les fichiers produits passent le vrai schéma de
  validation de l'app (vérifié) → ils s'ouvrent dans Glucose sans erreur.
- **Zéro dépendance** en dehors de l'Automerge déjà installé dans ce repo. Le
  protocole MCP (JSON-RPC sur stdio) est implémenté à la main dans un seul
  fichier : [`glucose-mcp.mjs`](glucose-mcp.mjs).

## Pourquoi c'est utile

Un `.glucose` est un document [Automerge](https://automerge.org/) (CRDT) binaire.
Ni toi ni Claude ne pouvez le lire à l'œil. Ce serveur le **décode** et en sort
un digest lisible : le texte des notes dans l'ordre de lecture, les **flèches
sémantiques** (`source —prédicat→ cible`), les zones (membranes), les domaines.

Résultat : tu peux demander à Claude « résume-moi ce cours », « où ai-je parlé de
X à travers tous mes projets ? », « quels projets sont vides / doublons ? », etc.

## Les outils exposés

**Lecture**

| Outil | Ce qu'il fait |
|-------|---------------|
| `list_glucose_projects` | Scanne un dossier (défaut `~/Documents`) et liste tous les `.glucose` avec un aperçu décodé (nom, nb de boards/notes/flèches, date). |
| `read_glucose` | Décode un `.glucose` → digest Markdown complet (notes + relations + zones). Option `includeIds` pour afficher les ids (utile avant `connect_notes`). |
| `analyze_architecture` | Reconstruit la **structure** sans vision : inclusion géométrique zones↔notes, regroupements spatiaux, forme du graphe (chaîne/étoile/réseau), hubs, hiérarchie des boards, dossiers-miroirs. Base pour comprendre puis proposer une organisation. |
| `lint_layout` | **QA visuel sans vision** : calcule les défauts qu'on verrait à l'écran — flèches qui traversent une note, croisements, notes qui se chevauchent, flèches trop longues. Score de désordre déterministe, 0 token. À lancer après une réorganisation. |
| `search_glucose` | Cherche une expression dans le texte de **tous** les `.glucose` d'un dossier. |

**Écriture** (modifie le disque)

| Outil | Ce qu'il fait |
|-------|---------------|
| `create_glucose_project` | Crée un nouveau `.glucose` à partir d'un titre + une liste de notes, disposées en colonnes lisibles. Refuse d'écraser sauf `overwrite=true`. |
| `add_note` | Ajoute une note (`text`/`sticky`) à un projet existant (position auto sous le contenu). |
| `connect_notes` | Relie deux notes par une flèche sémantique (par id **ou** par sous-chaîne de texte), prédicat optionnel. |
| `apply_layout` | **Applique une réorganisation entière en un coup, atomiquement** : retire des annotations (`removeIds`), déplace des notes (`moves`), crée des zones (`zones`), ajoute des flèches (`arrows`). Écrit de préférence dans `outPath` (une copie). C'est l'outil qui matérialise une proposition d'organisation. |

Les fichiers d'un ancien format (pré-Automerge) sont **signalés** « format
illisible » au lieu de faire planter le scan.

### ⚠️ Précautions écriture

- **Ferme le projet dans Glucose avant d'écrire** : l'app ne surveille pas le
  fichier sur disque. Écris pendant qu'il est fermé, puis **rouvre-le** pour voir
  le résultat. (Écrire pendant qu'il est ouvert dans Glucose fera écraser tes
  modifs non enregistrées au prochain Ctrl+S de l'app.)
- Toute réécriture crée d'abord une sauvegarde `<fichier>.glucose.bak`.
- Les prédicats de flèche valides : `est_precurseur`, `contredit`, `herite_de`,
  `inspire`, `depend_de`, `illustre`.

## Brancher sur **Claude Code** (ce terminal / l'extension)

C'est déjà câblé : un fichier [`.mcp.json`](../.mcp.json) à la racine du repo
déclare le serveur `glucose`. Au prochain lancement de Claude Code dans ce projet,
il te **demandera d'approuver** ce serveur MCP (sécurité des serveurs de projet).
Accepte, puis vérifie avec `/mcp` que `glucose` est bien connecté.

> Le chemin dans `.mcp.json` est absolu (machine actuelle). Si tu déplaces le
> repo, mets-le à jour, ou remplace-le par un chemin relatif `mcp/glucose-mcp.mjs`.

## Brancher sur **Claude Desktop**

Ajoute ceci à ton `claude_desktop_config.json`
(`%APPDATA%\Claude\claude_desktop_config.json` sur Windows), puis redémarre
Claude Desktop :

```json
{
  "mcpServers": {
    "glucose": {
      "command": "node",
      "args": ["C:\\Users\\Administrator\\Documents\\GlucoseGit-main\\mcp\\glucose-mcp.mjs"]
    }
  }
}
```

## Tester à la main (sans Claude)

```sh
# depuis la racine du repo (pour que Node résolve @automerge/automerge)
printf '%s\n' \
  '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{}}}' \
  '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"list_glucose_projects","arguments":{}}}' \
  | node mcp/glucose-mcp.mjs
```

## Limite technique importante

Le serveur **doit vivre dans ce repo** : il réutilise le paquet
`@automerge/automerge` de `node_modules/`, que Node résout depuis le dossier du
script. Sorti du repo, il ne trouverait plus Automerge (sauf à l'installer à côté).
