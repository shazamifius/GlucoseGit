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

> **État le 12/09/2026** : aucun export n'est atteignable par l'utilisateur — pas de commande, pas de bouton (`[Export]` de la barre d'outils, fiche 10). Le noyau porte `export.rs` (SVG et Markdown, purs, 8 tests) sans aucun appelant de production.

| Format d'export | État | Caractéristiques techniques |
|---|---|---|
| **HTML Interactif Autonome** | **À FAIRE** — rien | Produit un fichier `.html` unique avec JavaScript embarqué. N'importe quel navigateur peut ouvrir le tableau, naviguer au pan/zoom et consulter les notes sans avoir Glucose installé. |
| **PNG Haute Définition** | **À FAIRE** — rien | Rendu matriciel complet de la scène avec facteur d'échelle réglable ($\times 1$, $\times 2$, $\times 4$) pour impression ou projection. |
| **SVG Vectoriel** | **À FAIRE** — noyau écrit (`scene_to_svg`), non branché ; un seul calque (`<g class="card">`), pas de calques distincts | Fichier vectoriel pur avec calques distincts pour les cartes, flèches, membranes et textes, directement éditable dans Illustrator ou Inkscape. |
| **Markdown Structuré** | **À FAIRE** — noyau écrit (`project_to_markdown`, zones → cartes → liens), non branché | Extrait l'ensemble des blocs de texte, notes adhésives et descriptions de flèches dans un document Markdown linéaire hiérarchisé selon la disposition spatiale des cartes. |

---

## 6. Domaines Sémantiques & Calcul Chromatique des Membranes

### 6.3 Dérivation Chromatique des Membranes
* **À FAIRE** — La couleur d'une membrane est **dérivée de la moyenne pondérée des domaines de son contenu** : une membrane surtout *Sciences* (bleu) avec un peu d'*Art* (jaune) se teinte d'un vert bleuté. Rien n'existe : `DomainTints` teinte les badges domaine par domaine, la teinte symbiotique est positionnelle, et aucune des deux ne lit les domaines du contenu d'une membrane.
* **Formule à corriger avant de l'implémenter.** La fiche donnait $H = \sum H_i\,w_i \big/ \sum w_i$ : une moyenne *arithmétique* d'angles, fausse dès que les teintes chevauchent 0° — deux rouges à $350°$ et $10°$ moyennent à $180°$, un cyan. La moyenne juste est **vectorielle circulaire**, celle que `symbiotic_hue.rs` applique déjà aux voisines : $H = \operatorname{atan2}\big(\sum w_i \sin H_i,\ \sum w_i \cos H_i\big)$. L'exemple bleu + jaune donne un vert dans les deux cas, ce qui a masqué l'erreur.

---

## 7. Temporalité & Time Machine (Ancrage Historique)

### 7.2 Réglette Temporelle (Shift+R) & Filtrage Dynamique
* **À FAIRE** — Ouvre une réglette interactive en bas de l'écran couvrant de $-10\,000$ à $+2100$. Rien côté interface : ni raccourci, ni réglette. Le noyau porte déjà ce qu'il faut pour la graduer (`timeline::tick_step`, ères nommées `DEFAULT_ERAS`, `format_year` du géologique au calendaire).
* **À FAIRE** — En déplaçant le curseur temporel, les nœuds dont la période ne correspond pas à l'époque observée subissent une atténuation d'opacité vers $0.10$. Le noyau sait décider (`node_matches_temporal_filter`, testé) et le store porte `temporal_filter` avec `set_temporal_filter` — qu'aucun code de production n'appelle ; le rendu ne lit pas le filtre.

---

## 8. Mode Storyboard & Mise en Scène Séquentielle

> **État le 12/09/2026** : le panneau STORYBOARD existe (format, largeur, colonnes, espacement, grille de cellules, bouton Activer) et **ne touche pas au canevas** — il était une façade dont le bouton s'allumait sans effet ; il dit désormais « pas encore disponible » (testé : `test_the_storyboard_activate_button_does_not_stay_lit`). Le modèle porte `StoryboardPanel { order, description, x, y, width, height }` et le store `add_panel` / `update_panel`, journalisés, sans appelant.

* **À FAIRE** — **Ratios de cadrage standardisés** : `16:9` (Cinéma/TV), `4:3` (Classique), `2.35:1` (Cinémascope anamorphique), `1:1` (Carré), `9:16` (Vertical/Mobile). Les cinq libellés sont dans le panneau, sans effet.
* **À FAIRE** — **Grille de mise en page automatique** : alignement des vignettes en colonnes et rangées configurables avec espacement régulier.
* **À FAIRE** — **Légendes de production** : chaque panneau associe une image à une description narrative, un numéro d'ordre et un temps de plan. Le modèle n'a ni lien vers l'image, ni temps de plan.

---

## 9. Collaboration Multi-Utilisateur en Temps Réel (CRDT & Réseau)

> **État le 12/09/2026** : rien. Ni réseau, ni CRDT, ni pairs. `Project` porte `collab_url` et `asset_channel_url`, persistés et jamais lus. Le bouton **[Collab]** disait « Collaboration connectée » ; il dit désormais « pas encore disponible » et ne s'allume pas (testé). **À décider avant tout le reste** : Automerge (§ 3.1) n'a de sens qu'avec ce paragraphe, et ce paragraphe engage une dépendance lourde — voir la charte, exigence 1.

### 9.1 Synchronisation par CRDT Automerge
* **À FAIRE** — Les modifications concurrentes de deux collaborateurs sont fusionnées sans conflit d'écrasement (CRDT). Deux utilisateurs déplaçant deux cartes différentes en même temps voient les deux bouger sur les deux écrans, sans verrouillage de fichier.

### 9.2 Canal Séparé pour les Octets Lourds (`assetChannelUrl`)
* **À FAIRE** — Le document principal ne contient que les métadonnées légères ($< 500\text{ Ko}$) ; un canal d'assets secondaire transfère les octets d'images de pair à pair en arrière-plan. (Le conteneur `.glucose` sépare déjà document et actifs en sections distinctes : c'est la base de cette séparation.)

### 9.3 Curseurs Distants Fluides
* **À FAIRE** — Les mouvements de souris des pairs sont diffusés en coordonnées monde avec horodatage ; le moteur local interpole leur trajectoire (spline) et affiche une flèche à la couleur du collaborateur, coiffée de son nom.

---

## 10. Extensibilité : Système de Plugins & IA Locale (Ollama)

> **État le 12/09/2026** : rien. Le panneau PLUGINS reproduit la maquette (IA LOCALE, Cours magistral, densité, disposition) — mais il affichait une pastille verte **« Ollama actif »** et **« Ce PC : 32 Go RAM · 12 cœurs · GPU 6 Go »** écrits en dur, sans avoir jamais interrogé quoi que ce soit. Il affiche désormais l'état réel (« Ollama : non détecté », pastille grise), et le bouton Télécharger dit qu'il ne télécharge pas (testé : `test_the_download_model_button_does_not_pretend_to_download`).

### 10.1 Manifeste de Plugin (`plugin.json`)
* **À FAIRE** — Chaque plugin est un dossier autonome décrivant ses capacités, ses déclencheurs et ses paramètres d'interface.

### 10.2 Pont IPC Sécurisé
* **À FAIRE** — Les plugins s'exécutent en sandbox et communiquent avec le noyau par JSON sur stdin/stdout. Ils peuvent lire le board courant, proposer des dispositions ou injecter des nœuds.

### 10.3 Moteur IA Local (Ollama Bridge)
* **À FAIRE** — Détection automatique d'Ollama (`http://localhost:11434`).
* **À FAIRE** — Téléchargement de modèles légers (Llama 3, Mistral, Gemma) en un clic.
* **À FAIRE** — **Génération spatiale de savoir** : l'utilisateur colle un texte ; le LLM local en extrait les concepts-clés et génère un réseau de cartes et de flèches spatialisées sur le canvas. C'est le cas d'usage qui fixe l'échelle du projet (charte : le canva de Wikipédia).
