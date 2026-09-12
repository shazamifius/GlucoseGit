# 09 — Spécification des Systèmes Métier, Persistance & Réseau

> **Rôle de ce document** : définir l'architecture des données, les moteurs d'historique (Undo/Redo), les formats de fichiers sur disque, la gestion des assets binaires dédupliqués, la collaboration réseau et l'écosystème de plugins.
> Il détaille la logique interne non-visuelle qui fait de Glucose une plateforme robuste, résiliente aux pannes et pérenne.

---

## 1. Schéma de Données Universel

Le projet Glucose est une arborescence sérialisable de tableaux et de nœuds.

```
Project
 ├── metadata (version, name, createdAt, updatedAt)
 ├── domains: Vec<Domain> (partagés entre tous les boards)
 ├── presets: Vec<Preset> (gabarits de disposition)
 ├── blobs: HashMap<SHA256, Vec<u8>> (assets binaires embarqués)
 └── boards: Vec<Board>
      ├── viewport (x, y, scale)
      ├── images: Vec<BoardImage>
      ├── annotations: Vec<Annotation> (Text, Sticky, Arrow, Membrane)
      ├── folders: Vec<CanvasFolder> (pointent vers d'autres boards)
      ├── panels: Vec<StoryboardPanel>
      └── bookmarks: HashMap<"1".."9", Viewport>
```

### Invariants du Modèle
1. **Identifiants Déterministes (Nanoid / UUID)** :
   Chaque entité (image, annotation, dossier, tableau, domaine) possède un identifiant textuel unique de 16 à 21 caractères généré en $O(1)$.
2. **Bornes Numériques Spatiales (Anti-Crash)** :
   Pour prévenir les débordements de flottants (IEEE 754) et les instabilités dans les moteurs de rastérisation :
   * Coordonnées $(x, y)$ bornées à $\pm 1\,000\,000\text{ px}$ (`COORD_LIMIT = 1_000_000`).
   * Dimensions $(w, h)$ bornées entre $1\text{ px}$ et $200\,000\text{ px}$ (`SIZE_LIMIT = 200_000`).
   * Échelle de zoom bornée entre $0.005$ ($\times 200$ dézoomé) et $50.0$ ($\times 50$ zoomé).

---

## 2. Moteur d'Undo / Redo & Transactions Atomiques

L'annulation/rétablissement est le garant de la liberté créative de l'utilisateur.

### 2.1 Pile Bornée d'Historique
* Profondeur maximale fixée à **200 niveaux d'annulation** (`LIMITS.UNDO_DEPTH = 200`).
* Au-delà de 200 gestes, les modifications les plus anciennes sortent de la pile, garantissant une consommation mémoire bornée. Pour remonter plus loin dans le passé, l'application utilise les **jalons durables** sur disque.

### 2.2 Sessions d'Édition Continue (`beginLiveEdit` / `endLiveEdit`)
Certains gestes utilisateurs émettent des centaines d'évènements par seconde (déplacer une image à la souris, étirer un bord, taper un paragraphe de texte).
* **Le problème** : Enregistrer un snapshot à chaque pixel déplacé saturerait la mémoire et forcerait l'utilisateur à presser Ctrl+Z 300 fois pour annuler un seul glisser-déposer.
* **La solution Glucose** :
  1. Au premier pixel bougé ou à la première lettre tapée : ouverture d'une transaction via `beginLiveEdit()`.
  2. Pendant toute la durée du geste : les modifications s'appliquent en direct sur l'écran sans créer de nouvelle entrée dans la pile d'undo.
  3. Au relâchement de la souris (`pointerup`) ou à la fermeture de l'éditeur de texte : fermeture via `endLiveEdit()`.
  * **Résultat** : Un glisser de 5 secondes ou la rédaction d'un long mémo est annulé **en un seul coup de Ctrl+Z**.

---

## 3. Persistance & Formats de Sauvegarde sur Disque

Glucose refuse d'enfermer l'utilisateur dans un format opaque ou un cloud propriétaire.

### 3.1 Format Binaire `.glucose` v2 (Format Principal)
* Fichier binaire local hautement compressé.
* Structure interne : En-tête de fichier magique + flux binaire compacté Automerge.
* **Sauvegarde Incrémentale Instantanée (`append_glucose_binary`)** :
  Lors des sauvegardes rapides (Ctrl+S ou autosave), le logiciel ne réécrit pas les 50 Mo du fichier complet : il se contente d'ajouter à la fin du fichier les octets du delta incrémental depuis le dernier enregistrement ($< 1\text{ Ko}$, exécuté en $< 1\text{ ms}$).

### 3.2 Format Bundle Portable (Dossier Autonome)
Conçu pour archiver ou transférer un projet gigantesque contenant des gigaoctets de photos et vidéos sans saturer le fichier de projet principal :
```
MonProjet.glucose_bundle/
 ├── project.glucose       <-- Document structurel (léger, quelques Ko)
 └── objects/              <-- Dossier des assets lourds
      ├── 4f8a9b...png     <-- Fichier nommé par son hash SHA-256
      ├── e2c10d...mp4
      └── 9a3b7c...jpg
```
* **Déduplication Native** : Deux images identiques importées sur le canvas partagent le même fichier dans `objects/`. Copier le dossier emporte 100% des médias sans aucune perte de chemin.

### 3.3 Filet de Sauvegarde Automatique & Résilience aux Pannes
1. **Autosave Débouncé (2000 ms)** :
   Toute modification déclenche une minuterie de 2 secondes. Si aucune nouvelle action n'intervient, le projet est écrit silencieusement sur le disque.
2. **Écriture Atomique Sécurisée** :
   Le fichier est d'abord écrit sous une extension temporaire (`.glucose.tmp`). Une fois l'écriture réussie et vérifiée, il remplace le fichier cible par renommage atomique OS (`std::fs::rename`). Si l'ordinateur s'éteint brutalement au milieu de l'écriture, le fichier original n'est jamais corrompu.
3. **Restauration depuis un Jalon Sain (`loadLatestHealthyVersion`)** :
   Si un fichier `.glucose` est altéré (coupure de courant, disque défectueux) :
   * Le chargeur détecte la corruption à la lecture.
   * L'application interroge le dossier d'historique durable `.glucose.versions/` associé au document.
   * Une invite propose immédiatement : *"Ce document est abîmé. Restaurer le dernier jalon sain du 12/09/2026 à 14h20 « Auto-save » ?"*.
   * L'utilisateur récupère son travail en un clic (`Toast: "Restauré depuis le jalon 🛟"`).

---

## 4. Pipeline d'Assets Binaires Adressés par le Contenu

Pour manipuler des milliers d'images et vidéos avec une intégrité parfaite :

### 4.1 Référence Universelle (`AssetRef`)
Toute ressource multimédia est typée par une union discriminée stricte :
* **Mode `embed`** : Les octets bruts vivent directement dans le projet, indexés par leur condensat cryptographique SHA-256 (`project.blobs[sha256]`).
* **Mode `link`** : Référence externe via un chemin relatif ou une URL web (`href: "images/photo.jpg"` ou `https://...`). Utilisé pour les fichiers vidéo volumineux ou les miroirs de dossiers disques.

### 4.2 Déduplication Cryptographique
À l'import d'une image :
1. Calcul du hash SHA-256 des octets.
2. Vérification dans la table des blobs existants :
   * Si le hash est déjà présent : l'image réutilise le blob existant sans allouer un seul octet supplémentaire sur le disque ni en mémoire.
   * Si le hash est nouveau : les octets sont stockés dans la table.

---

## 5. Moteur d'Export Multi-Formats

Glucose permet d'exporter ses tableaux vers le monde extérieur :

| Format d'export | Caractéristiques techniques |
|---|---|
| **HTML Interactif Autonome** | Produit un fichier `.html` unique avec JavaScript embarqué. N'importe quel navigateur peut ouvrir le tableau, naviguer au pan/zoom et consulter les notes sans avoir Glucose installé. |
| **PNG Haute Définition** | Rendu matriciel complet de la scène avec facteur d'échelle réglable ($\times 1$, $\times 2$, $\times 4$) pour impression ou projection. |
| **SVG Vectoriel** | Fichier vectoriel pur avec calques distincts pour les cartes, flèches, membranes et textes, directement éditable dans Illustrator ou Inkscape. |
| **Markdown Structuré** | Extrait l'ensemble des blocs de texte, notes adhésives et descriptions de flèches dans un document Markdown linéaire hiérarchisé selon la disposition spatiale des cartes. |

---

## 6. Domaines Sémantiques & Calcul Chromatique des Membranes

Les domaines catégorisent la connaissance (ex: *Sciences*, *Histoire*, *Design*, *Mythologie*).

### 6.1 Structure d'un Domaine
```rust
pub struct Domain {
    pub id: String,
    pub name: String,
    pub color: String, // Code couleur HSL ou Hex (ex: "#3b82f6")
    pub icon: String,  // Symbole ou émoji affiché dans les badges
    pub created_at: u64,
}
```

### 6.2 Signature Poly-Sémantique d'un Nœud
Un élément peut appartenir à plusieurs domaines avec des poids distincts :
$$\text{domains} = [(\text{Science}, 0.7), (\text{Histoire}, 0.3)]$$

### 6.3 Dérivation Chromatique des Membranes
La couleur d'une membrane n'est pas choisie au hasard par l'application : elle est **dérivée de la moyenne pondérée des domaines de son contenu** :
$$H_{\text{membrane}} = \sum (H_i \times \text{poids}_i) \Big/ \sum \text{poids}_i$$
Une membrane contenant majoritairement des cartes liées au domaine *Sciences* (bleu) et un peu d'*Art* (jaune) se teintera automatiquement d'un vert bleuté harmonieux.

---

## 7. Temporalité & Time Machine (Ancrage Historique)

Glucose permet de cartographier des évènements à travers le temps géologique et humain.

### 7.1 L'Ancre Temporelle (`TemporalAnchor`)
* Représente la date du **sujet décrit** par la carte (pas la date de modification du fichier).
* Années calendaires sous forme d'entiers signés :
  * Valeur positive : ère commune (ex: `1789` = Révolution française).
  * Valeur négative : avant J.-C. (ex: `-3000` = Invention de l'écriture en Mésopotamie).
* Plages continues : intervalle `[start, end]` (ex: Renaissance `[1400, 1600]`).

### 7.2 Réglette Temporelle (Shift+R) & Filtrage Dynamique
* Ouvre une réglette interactive en bas de l'écran couvrant de $-10\,000$ à $+2100$.
* En déplaçant le curseur temporel, les nœuds dont la période ne correspond pas à l'époque observée subissent une atténuation d'opacité vers $0.10$, mettant en relief la contemporanéité des idées.

---

## 8. Mode Storyboard & Mise en Scène Séquentielle

Transforme un ensemble d'images libres en une séquence narrative de plans (pour le cinéma, l'animation, la BD ou le jeu vidéo).

* **Ratios de cadrage standardisés** : `16:9` (Cinéma/TV), `4:3` (Classique), `2.35:1` (Cinémascope anamorphique), `1:1` (Carré), `9:16` (Vertical/Mobile).
* **Grille de mise en page automatique** : Alignement automatique des vignettes en colonnes et rangées configurables avec espacement régulier.
* **Légendes de production** : Chaque panneau associe une image à une description narrative, un numéro d'ordre et un temps de plan.

---

## 9. Collaboration Multi-Utilisateur en Temps Réel (CRDT & Réseau)

Conçu pour fonctionner en réseau local (LAN) ou distant sans serveur central complexe.

### 9.1 Synchronisation par CRDT Automerge
* Les modifications concurrentes de deux collaborateurs sont fusionnées sans aucun conflit d'écrasement grâce au formalisme mathématique des CRDT (Conflict-free Replicated Data Types).
* Si deux utilisateurs déplacent deux cartes différentes en même temps, les deux cartes bougent sur les deux écrans sans aucun verrouillage de fichier.

### 9.2 Canal Séparé pour les Octets Lourds (`assetChannelUrl`)
* **Le piège classique des CRDT** : Mettre des octets d'images dans le document CRDT fait exploser la taille de l'historique et gèle les calculs de fusion.
* **L'architecture Glucose** :
  * Le document principal ne contient que les métadonnées légères ($< 500\text{ Ko}$).
  * Un **canal d'assets secondaire dédié** transfère les flux binaires d'images de pair à pair en arrière-plan.

### 9.3 Curseurs Distants Fluides (`PeerCursorsLayer.tsx`)
* Les mouvements de souris des pairs sont diffusés en coordonnées monde avec horodatage.
* Le moteur local interpole la trajectoire des curseurs distants par lissage polynomial (spline), affichant une flèche fluide à la couleur du collaborateur coiffée de son nom d'utilisateur.

---

## 10. Extensibilité : Système de Plugins & IA Locale (Ollama)

Glucose s'interface avec l'écosystème machine local via des processus sidecars légers.

### 10.1 Manifeste de Plugin (`plugin.json`)
Chaque plugin est un dossier autonome décrivant ses capacités, ses déclencheurs et ses paramètres d'interface.

### 10.2 Pont IPC Sécurisé
Les plugins s'exécutent en sandbox et communiquent avec le noyau via des canaux IPC standards (JSON stdin/stdout). Ils peuvent lire le board courant, proposer de nouvelles dispositions géométriques ou injecter des nœuds.

### 10.3 Moteur IA Local (Ollama Bridge)
* Détection automatique d'Ollama sur la machine (`http://localhost:11434`).
* Possibilité de télécharger des modèles légers (Llama 3, Mistral, Gemma) en un clic.
* **Génération Spatiale de Savoir** : L'utilisateur colle un texte brut ou un article de recherche ; le plugin interroge le LLM local qui analyse les concepts-clés et génère automatiquement un réseau complet de cartes et de flèches logiques spatialisées sur le canvas.
