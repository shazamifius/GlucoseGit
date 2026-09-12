# 09 — Spécification des Systèmes Métier, Persistance & Réseau

> **Rôle de ce document** : définir l'architecture des données, les moteurs d'historique (Undo/Redo), les formats de fichiers sur disque, la gestion des assets binaires dédupliqués, la collaboration réseau et l'écosystème de plugins.
> Il détaille la logique interne non-visuelle qui fait de Glucose une plateforme robuste, résiliente aux pannes et pérenne.

---

## 1. Schéma de Données Universel

### Invariants du Modèle
* **À FAIRE** — **Bornes numériques spatiales (anti-crash)** : coordonnées $(x, y)$ bornées à $\pm 1\,000\,000\text{ px}$, dimensions $(w, h)$ entre $1$ et $200\,000\text{ px}$. Rien ne les borne aujourd'hui. À obtenir par la **représentation** — entiers à virgule fixe, $\pm 8{,}4\text{ M px}$ par construction, plan de marche RQ-1 — et non par un clamp en `f64`, qui serait du travail jeté.

---

## 2. Moteur d'Undo / Redo & Transactions Atomiques

### 2.1 Pile Bornée d'Historique
* **À FAIRE** — Pour remonter plus loin dans le passé que les 200 niveaux de la pile, l'application utilise les **jalons durables** sur disque (`.glucose.versions/`, voir § 3.3). Aucune trace dans `crates/` : ni dossier de versions, ni chargeur de jalon.

---

## 3. Persistance & Formats de Sauvegarde sur Disque

### 3.1 Format Binaire `.glucose` v2 (Format Principal)
* **À DÉCIDER** — « Fichier hautement compressé ». Le conteneur n'est pas compressé. La compression est en tension avec la lecture par `mmap` que la cible 10⁷ appelle (plan de marche RQ-1) : un fichier compressé ne se projette pas en mémoire. À trancher avec la persistance à grande échelle, pas avant.
* **À DÉCIDER** — « Flux binaire compacté Automerge ». Le conteneur est un format propre : sections typées, `sha256` par section (`persist/container.rs`). Automerge ne se justifie que par la collaboration CRDT (§ 9), entièrement absente ; sans elle, c'est une dépendance lourde pour rien. Ce qui reste vrai : les fichiers de Glucose Tauri **sont** Automerge, et le conteneur prévoit un importeur v1 qu'il nomme dans son message de refus — cet importeur n'existe pas.
* **À FAIRE** — **Sauvegarde incrémentale instantanée (`append_glucose_binary`)** : ajouter en fin de fichier le delta depuis le dernier enregistrement ($< 1\text{ Ko}$, $< 1\text{ ms}$) au lieu de réécrire le fichier. La nature de section `KIND_JOURNAL` est réservée et sautée à la lecture (testé), rien ne l'écrit.

### 3.2 Format Bundle Portable (Dossier Autonome)
* **À DÉCIDER** — Le dossier `MonProjet.glucose_bundle/` (`project.glucose` + `objects/` nommés par SHA-256) n'existe pas. Sa raison d'être première — déduplication et portabilité des médias — est déjà remplie par le fichier unique : le conteneur v2 embarque les actifs, un seul contenu par `sha256` (testé). Ne lui reste que la séparation document léger / objets lourds pour les projets à gigaoctets, qui rejoint la persistance à grande échelle. Le module `bundle.rs` qui portait des briques sans appelant a été retiré ; `git` le garde.

### 3.3 Filet de Sauvegarde Automatique & Résilience aux Pannes
* **À FAIRE** — **Autosave débouncé (2000 ms)** : toute modification arme une minuterie de 2 s ; sans nouvelle action, le projet est écrit silencieusement. Rien n'existe.
* **À FAIRE** — **Restauration depuis un jalon sain (`loadLatestHealthyVersion`)** : dossier d'historique durable `.glucose.versions/`, invite *« Ce document est abîmé. Restaurer le dernier jalon sain du … ? »*, toast *« Restauré depuis le jalon 🛟 »*. La détection de corruption à la lecture, elle, existe et est testée (troncature, bit inversé, somme forgée, document courant laissé intact) ; ni le dossier de jalons, ni l'invite.

---

## 4. Pipeline d'Assets Binaires Adressés par le Contenu

> **État mesuré le 12/09/2026.** Ce qui existe : le type `AssetRef` (`Embed { sha256, mime }` / `Link { href }`), sérialisé dans le fichier ; un magasin `AssetStore` ; et, dans le conteneur `.glucose`, un seul blob par `sha256` (testé : `test_a_duplicated_asset_writes_a_single_blob`). Ce qui manque est **un chantier d'architecture, pas un correctif** — il touche l'import, le magasin et le rendu ensemble, et se conçoit avec le rendu GPU (plan de marche RQ-2) :
>
> * `AssetRef` n'est **construit par aucun code de production**. L'identité d'une image reste `src`, un chemin de fichier sur la machine d'import ; le magasin en mémoire est indexé par ce chemin ; le cache du renderer aussi, et il décode dans la boucle de rendu.
> * L'empreinte n'est pas calculée à l'import mais **à chaque enregistrement**, sur chaque octet de chaque actif, après avoir **relu chaque fichier depuis le disque** (`persist/assets.rs::collect`, reconstruit à chaque Ctrl+S). Mesuré au banc (`bench_store`, section « Sauvegarde ») sur 1 000 nœuds : 0 Mo d'actifs → 1 ms ; 16 Mo → 205 ms ; 64 Mo → 1 075 ms ; 256 Mo → **3 928 ms**, hors lecture et écriture disque, avec une copie intégrale des actifs allouée pour construire le fichier. La sauvegarde coûte le volume des actifs, pas la modification : c'est la loi L3 violée, et la sauvegarde incrémentale du § 3.1 est impossible tant que c'est le cas.

### 4.1 Référence Universelle (`AssetRef`)
* **À FAIRE** — Toute ressource multimédia est typée par `AssetRef` : **`embed`**, octets dans le magasin, indexés par leur SHA-256 ; **`link`**, chemin relatif ou URL (`href: "images/photo.jpg"`, `https://...`) pour les vidéos volumineuses et les miroirs de dossiers. Le `src` d'origine devient une information de provenance, jamais une identité.

### 4.2 Déduplication Cryptographique
* **À FAIRE** — À l'import : SHA-256 des octets **une fois** ; si l'empreinte est déjà dans le magasin, l'image réutilise le blob sans allouer un octet, ni en mémoire ni sur le disque ; sinon les octets entrent dans le magasin sous leur empreinte. L'enregistrement n'a plus rien à hacher ni à relire : il écrit ce qu'il a, le rendu décode depuis le magasin, hors frame.

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
