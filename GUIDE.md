# 📖 Guide d'utilisation Glucose

> **Glucose** est un canvas infini pour poser tes idées à plat. Une seule interface, pas de modes : pose, relie, zoome, explore.

**Version :** 1.0.0-beta.1

---

## 1. Démarrage rapide

À l'ouverture, Glucose te donne un canvas vide. **Cinq gestes pour t'y retrouver** :

1. **Glisser** une image depuis ton navigateur ou ton explorateur de fichiers → elle se pose sur le canvas
2. Appuyer **`N`** puis cliquer → une note jaune (sticky)
3. Appuyer **`A`** puis tirer une ligne entre deux blocs → flèche typée
4. **Molette** = zoom · **Espace** maintenu = pan · **Ctrl+Shift+F** = tout cadrer
5. **Ctrl+S** pour sauvegarder ton projet en `.glucose`

C'est tout. Le reste est de la spécialisation.

---

## 2. Vocabulaire

| Terme | Sens |
|---|---|
| **Board** | Une zone de canvas indépendante (onglet en haut). Chaque board a ses images, annotations, dossiers, viewport. |
| **Annotation** | Tout ce qui n'est pas une image : texte, sticky, flèche, membrane. |
| **Sticky** | Note jaune (ou autre couleur). Idéal pour les commentaires courts. |
| **Texte** | Bloc Markdown avec rendu prose : titres, listes, LaTeX, gras, italique. |
| **Flèche** | Lien orienté entre deux nœuds. Peut porter un **prédicat** sémantique. |
| **Membrane fixe** | Zone colorée dessinée à la main avec l'outil `M`. |
| **Membrane implicite** | Halo coloré généré automatiquement autour d'un cluster d'images (Union-Find). |
| **Domaine** | Catégorie sémantique (Science, Art…) avec couleur et icône. |
| **Dossier** | Sous-canvas zoomable. Zoomer dessus = y entrer. |
| **Miroir** ↻ | Copie vivante d'un nœud ou dossier. Modifier l'original = tous les miroirs changent. |
| **Jalon** 📌 | Point de repère nommé dans la Time Machine. |
| **Prédicat** | Type sémantique d'une flèche : `inspire`, `contredit`, `hérite_de`, `est_précurseur`, `dépend_de`, `illustre`. |
| **Trans-domaine** | Flèche dont source et cible n'ont aucun domaine en commun (rendue en pointillés). |

---

## 3. Les outils (toolbar)

Chaque outil s'active par sa lettre. Sélectionne, fais ton geste, retombe en `V`.

| Touche | Outil | Geste |
|---|---|---|
| `V` | **Sélection** (par défaut) | Click pour sélectionner, drag pour déplacer, drag-rectangle pour multi-sélectionner |
| `T` | **Texte** | Click pour poser, taper, Échap pour valider. Markdown supporté. |
| `N` | **Sticky** | Click pour poser une note. Couleurs disponibles dans la barre contextuelle. |
| `A` | **Flèche** | Click sur le bloc source, puis sur le bloc cible. Sélectionne du texte avant le 2e click pour pointer un paragraphe précis. |
| `F` | **Dossier** | Drag pour dessiner une zone. **Tout ce qui est dedans est capturé** automatiquement. |
| `M` | **Membrane** | Drag pour dessiner une zone colorée fixe (organisationnelle). |
| `Espace` (maintenu) | **Pan** | Drag = déplacer le viewport |

---

## 4. Concepts avancés

### 🌈 Domaines

Les domaines sont des catégories sémantiques que tu crées toi-même : *Science*, *Art*, *Histoire*, *Game design*…

**Ouvrir le panel Domaines** → bouton dans la toolbar. Crée un domaine (couleur + emoji), puis sélectionne des nœuds et fais glisser un curseur 0–100 % pour les associer.

Effets visuels :
- 🟣 **Membranes** : leur couleur dérive du domaine dominant des images qu'elles entourent
- 🏷️ **Badges** : un nœud avec poids > 40 % dans un domaine porte le badge correspondant (coin haut-droit)
- ⚡ **Flèches trans-domaines** : si une flèche relie deux nœuds sans domaine commun, elle apparaît en **pointillés** — rappel visuel d'un lien interdisciplinaire

### 📅 Réglette temporelle

Pour ancrer un nœud à une **date du contenu décrit** (pas la date d'édition).

1. Sélectionne un ou plusieurs nœuds
2. **`Shift+T`** ouvre le modal d'ancrage
3. Tape une date dans n'importe quel format :
   - `1789` ou `1789-1799`
   - `-3000` (av. J.-C.)
   - `Renaissance` (autocomplétion sur 30 époques nommées)
   - `10 ka` (10 000 ans avant maintenant)
   - `1,5 Ma` (1,5 millions d'années)

Ouvre la **réglette zoomable** avec **`Shift+R`** : drag les deux poignées jaunes pour filtrer le canvas par fenêtre temporelle. Les nœuds **non ancrés** restent toujours visibles. Molette sur la réglette = zoom de l'échelle (de la milliseconde au géologique).

Un nœud ancré porte un badge 📅 visible au coin bas-droit.

### 🪞 Miroirs (alias)

Un **miroir** est une copie vivante : modifier l'original ou le miroir change les deux. Idéal pour :
- Référencer une même idée à plusieurs endroits du canvas
- Construire un index visuel d'éléments éparpillés
- Faire une vue alternative d'un dossier sans dupliquer son contenu

**Créer un miroir** : sélectionne un nœud (ou un dossier) → `Ctrl+Shift+M`. Le miroir apparaît avec un offset léger et un badge ↻ bleu.

**Cliquer le badge ↻** = téléportation animée vers l'original (peut traverser les boards).

**Anti-Inception** : Glucose refuse de créer un miroir qui produirait un cycle (A contient B contient A). Un message console l'indique.

### 🗂️ Dossiers zoomables

Un dossier est un **sous-canvas complet** avec ses propres boards/images/annotations.

**Créer** : outil `F`, drag-rectangle. Tout ce dont le centre tombe dans la zone est **capturé** dans le dossier (les coords sont ajustées en relatif).

**Naviguer** :
- **Zoom** (molette) sur un dossier → tu y entres automatiquement
- **Dézoom** dans un dossier → tu en sors automatiquement
- Une **bordure colorée** apparaît autour de l'écran quand tu es dans un dossier (couleur du dossier)
- Quand tu approches du seuil de sortie, un bandeau te prévient `⤴ continue à dézoomer pour sortir`

**Breadcrumb** en haut à gauche : façon VSCode, avec dropdown sur hover pour sauter directement entre dossiers frères.

### ⏳ Time Machine

Glucose enregistre **chaque action** comme un commit Automerge. Tu disposes d'un undo/redo **infini** + d'une vraie machine à voyager dans le temps de ton projet.

**Ouvrir** : `Ctrl+H` → un slider apparaît en bas du canvas.

**Drag du curseur** sur le slider = **aperçu live** d'un état passé. Le canvas redessine en temps réel. Une **bordure jaune pleine-écran** signale le mode preview.

En mode preview, **les modifications sont bloquées** (warning console). Trois options :
- **« ← Maintenant »** : retour au présent
- **« ⏪ Restaurer cet état »** : applique l'état preview comme un nouveau commit. L'historique antérieur est conservé — tu peux toujours `Ctrl+Z` pour annuler la restauration.
- **« + Marquer un jalon »** (en mode normal uniquement) : commit nommé qui apparaît en 📌 jaune sur la piste

Les jalons cliquables apparaissent sous la piste pour navigation rapide (« avant la refonte », « v1 stable », etc.).

`Ctrl+Z` / `Ctrl+Y` = undo/redo classiques (équivalent à reculer/avancer d'un commit).

### 🛰️ Multijoueur LAN

Édite un même projet à plusieurs sur le même réseau local — sync **temps réel**, fusion CRDT automatique, **zéro conflit**.

**Activer** : `Ctrl+Shift+L` → panel multijoueur → **« ▶ Activer le multijoueur »**.

Glucose annonce ton instance via mDNS sur le réseau et écoute les autres. Toute autre machine du LAN qui active le multijoueur apparaît dans **« Instances sur le LAN »**. Click sur un peer = connexion établie (LED verte).

À partir de là, **chaque modification de l'un est répliquée chez les autres en temps réel**. Tu peux travailler en parallèle sur différentes parties du canvas — Automerge merge tout automatiquement, même en cas de modifications simultanées.

**Limites MVP** :
- Pas de curseurs flottants temps réel encore (Phase 7.5bis polish à venir)
- Pas de chiffrement (LAN non-fiable → utilise un VPN)
- Si la découverte mDNS échoue (firewall/routeur restrictif), entre l'IP manuellement (`192.168.x.x:7777`)

---

## 5. Multimédia & App Bridge

### Images

- **Drag-drop** une image depuis n'importe où (browser, explorer, copier-coller).
- **URL d'image web** : drag-drop l'URL → Glucose télécharge en meilleure qualité automatiquement (Pinterest, Twitter, Instagram, Reddit, Imgur, Tumblr, Wallhaven, ArtStation, DeviantArt sont reconnus avec upgrade auto vers la résolution originale).
- Les images sont stockées **externalisées** dans `assets/<hash>.<ext>` côté disque (pas en base64 dans le `.glucose`) — fichier projet compact, dédup automatique.

### Vidéos

- **Drag-drop** un fichier `.mp4`/`.mov`/`.mkv` local → vidéo intégrée
- **URL YouTube / TikTok / Instagram / Vimeo** → yt-dlp embarqué télécharge automatiquement (la première fois, yt-dlp se télécharge ; ensuite c'est immédiat)
- Vidéos jouent en boucle muette dans le canvas

### App Bridge (fichiers créatifs)

Drag d'un fichier non-image (`.blend`, `.psd`, `.kra`, `.ai`, `.fbx`, `.obj`, `.c4d`, `.fig`, etc.) → crée un **sticky source** avec icône logiciel.

**Double-click** sur le sticky source = **ouvre le fichier dans son app native** (Blender, Photoshop, Krita…). Glucose vérifie une whitelist d'extensions sûres avant d'ouvrir (les `.exe`, `.bat`, etc. sont refusés).

---

## 6. Sauvegarde et format `.glucose`

| Action | Raccourci |
|---|---|
| Enregistrer (rapide, path courant) | `Ctrl+S` |
| Enregistrer sous… (nouveau path) | `Ctrl+Shift+S` |
| Ouvrir un projet existant | `Ctrl+O` |

Le fichier `.glucose` est un **binaire Automerge** compact qui contient :
- Toutes les données du projet (boards, annotations, images, dossiers, domaines, presets)
- L'**historique complet** de tes modifications (Time Machine s'en sert)

À l'ouverture d'un ancien `.glucose` v1 (format JSON), Glucose détecte automatiquement et migre. Au prochain `Ctrl+S` il est ré-écrit en v2 binaire.

**Note** : les images / vidéos téléchargées ne sont **pas bundlées** dans le `.glucose` — elles vivent dans `app_data_dir/assets/` et `app_data_dir/videos/`. Si tu déplaces un `.glucose` sur une autre machine, les chemins ne suivront pas (un format archive zip est prévu plus tard).

---

## 7. Tableau complet des raccourcis

### Outils
| Touche | Action |
|---|---|
| `V` | Sélection |
| `T` | Texte |
| `N` | Sticky |
| `A` | Flèche |
| `F` | Dossier |
| `M` | Membrane |
| `Espace` (maintenu) | Pan |

### Édition
| Touche | Action |
|---|---|
| `Ctrl+Z` / `Ctrl+Y` | Undo / Redo (infini) |
| `Ctrl+A` | Sélectionner tout |
| `Ctrl+C` / `Ctrl+X` / `Ctrl+V` | Copier / Couper / Coller |
| `Ctrl+D` | Dupliquer la sélection |
| `Suppr` / `Backspace` | Supprimer |
| `Ctrl+Shift+M` | Créer miroir(s) de la sélection |
| `Alt+1..4` | Sticky → opérateur logique (AND/OR/BUT/BECAUSE) |
| `Alt+0` | Sticky → retire l'opérateur |

### Navigation
| Touche | Action |
|---|---|
| Molette | Zoom |
| `Ctrl+Shift+F` | Zoom-to-fit (tout cadrer) |
| `Ctrl+F` | Recherche globale |
| `L` | Verrouiller / déverrouiller la sélection |
| `G` | Toggle alignement intelligent |

### Fichier
| Touche | Action |
|---|---|
| `Ctrl+S` | Enregistrer |
| `Ctrl+Shift+S` | Enregistrer sous… |
| `Ctrl+O` | Ouvrir |
| `F11` | Mode Zen (cache toute l'UI) |

### Phases avancées
| Touche | Action |
|---|---|
| `Shift+R` | Réglette temporelle |
| `Shift+T` | Ancrer une date à la sélection |
| `Ctrl+H` | Time Machine |
| `Ctrl+Shift+L` | Multijoueur LAN |

---

## 8. Workflows types

### 🎨 Design d'un personnage

1. Crée un dossier `MonPerso` (outil `F`, drag)
2. Zoome dedans (entre automatiquement)
3. Drag-drop tes références d'inspiration (Pinterest, ArtStation…)
4. Outil `M` → dessine 3 zones colorées : *Refs*, *Sketches*, *Final*
5. Place tes croquis dans la bonne zone
6. Outil `A` → trace des flèches `inspire` entre tes refs et tes sketches
7. Sticky note (`N`) à côté du final : description, intentions
8. **`Ctrl+Shift+M`** sur le sketch final pour le **miroir** dans le board principal — il y reste lié

### 📚 Recherche académique

1. Domaines : *Auteurs*, *Concepts*, *Sources*
2. Crée un sticky par auteur, attribue le domaine *Auteurs* à 100 %
3. Crée un sticky par concept, attribue *Concepts*
4. Flèches `inspire` / `contredit` / `dépend_de` entre concepts
5. Pour chaque concept, ancre une **date temporelle** (`Shift+T`) — `1859` pour Darwin par exemple
6. Active la réglette (`Shift+R`) → drag pour ne voir que l'époque concernée
7. **Time Machine** : marque un jalon `📌 état initial de ma thèse` avant chaque grande modif

### 🌍 Worldbuilding (univers fictif)

1. Boards : *Géographie*, *Personnages*, *Histoire*, *Magie* — un par grand domaine
2. Dans *Histoire* : ancre chaque événement (`Shift+T`) → réglette temporelle = chronologie visuelle automatique
3. Dans *Personnages* : un dossier par perso, chaque dossier contient refs + bio + relations
4. Flèches inter-dossiers (le `targetBoardId` des flèches portail) → click = téléportation vers le board cible
5. Multi-utilisateur LAN (`Ctrl+Shift+L`) si tu travailles en duo avec un coauteur sur le même réseau

---

## 9. Astuces

- **Mode Zen** (`F11`) : cache toolbar / breadcrumb / tabs. Utile pour les sessions longues sans distraction.
- **Pomodoro** intégré : panel Pomodoro 25/5/15. Notification système à la fin.
- **Recherche globale** (`Ctrl+F`) : cherche dans les textes, sticky, tags d'image, noms de dossiers, sur tous les boards.
- **Trans-domaines toggle** dans la toolbar : masque/affiche les flèches en pointillés trans-domaines.
- **Tags d'images** : sélectionne une image → champ tags dans la barre contextuelle. Searchables via Ctrl+F.

---

## 10. Où sont mes données ?

| OS | Dossier |
|---|---|
| Windows | `%APPDATA%\Glucose\` |
| macOS | `~/Library/Application Support/Glucose/` |
| Linux | `~/.config/Glucose/` |

Sous-dossiers :
- `assets/` — images externalisées (dédup SHA-256)
- `videos/` — vidéos téléchargées via yt-dlp
- `yt-dlp.exe` — binaire pinné pour l'import vidéo

Tes fichiers `.glucose` sont sauvegardés là où tu choisis (par défaut dans ton home).

---

## 11. Que faire si…

**…le drag-drop d'image ne marche pas** : ouvre la DevTools (`Ctrl+Shift+I`), onglet Console. Regarde les logs `[drop]` et `[handleDrop]` — ils te disent exactement ce qui a été reçu et tenté. Pinterest/Insta/etc. sont gérés via fallback automatique.

**…un projet ne s'ouvre pas** : Glucose valide la structure via Zod. Si le `.glucose` est corrompu, un message clair indique le champ fautif. Le projet courant n'est pas écrasé.

**…le multijoueur LAN ne se découvre pas** : firewall ou routeur qui filtre mDNS. Solution : entre l'IP manuellement (`192.168.x.x:7777`) dans le panel. Vérifie aussi que les deux machines sont sur le même sous-réseau.

**…un sticky source n'ouvre pas son fichier natif** : vérifie que le fichier existe encore au chemin enregistré, et que son extension est dans la whitelist (`.blend`, `.psd`, `.kra`, etc. — pas `.exe` ou similaires).

---

<div align="center">

**Glucose, c'est juste poser, relier, zoomer, explorer.**

[← Retour au README](README.md) · [Roadmap](ROADMAP.md) · [Issues](../../issues)

</div>
