<div align="center">

# Glucose

**Un canevas infini pour penser en images et en texte. Natif, écrit en Rust.**

*Pose. Relie. Zoome. Explore.*

[![Licence MIT](https://img.shields.io/badge/licence-MIT-blue?style=flat-square)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-natif-CE422B?style=flat-square&logo=rust)](https://www.rust-lang.org/)
[![Noyau sans dépendance](https://img.shields.io/badge/noyau-0%20d%C3%A9pendance-brightgreen?style=flat-square)](#mesuré-pas-affirmé)
[![Windows](https://img.shields.io/badge/v%C3%A9rifi%C3%A9%20sous-Windows-0078D6?style=flat-square)](#essayer-glucose-rust)
[![État](https://img.shields.io/badge/%C3%A9tat-en%20d%C3%A9veloppement-orange?style=flat-square)](#ce-qui-manque-encore)

[Guide d'utilisation](GUIDE.md) · [Discussions](https://github.com/shazamifius/GlucoseGit/discussions) · [Signaler un bug](https://github.com/shazamifius/GlucoseGit/issues/new/choose) · [L'ancienne version](#glucose-tauri-lancienne-version)

</div>

> [!IMPORTANT]
> **Glucose a changé de moteur.** La première version, **Glucose Tauri**, était une
> application web (React et TypeScript) enfermée dans une fenêtre Tauri. Elle est **figée**.
> Ce dépôt est désormais **Glucose Rust** : le même logiciel, recréé de zéro en Rust natif,
> sans navigateur et sans JavaScript.
>
> - La branche `main` ne contient **que** Glucose Rust. Pas une ligne de TypeScript.
> - Glucose Rust **n'a pas encore de version téléchargeable** : pour l'essayer, il faut le
>   compiler ([deux commandes](#essayer-glucose-rust)).
> - Les versions publiées dans l'onglet *Releases*, jusqu'à `v1.0.2-beta.1`, sont toutes
>   **Glucose Tauri**.
> - Les fichiers enregistrés par Glucose Tauri **ne s'ouvrent pas encore** dans Glucose Rust.

<!-- CAPTURE : le canevas principal, avec des photos, des cartes, une membrane et des flèches -->

## Ce qu'est Glucose

Une feuille noire, sans bord, sur laquelle on pose tout ce qui aide à penser : des photos,
des cartes de texte en Markdown, des formules, des notes, des flèches qui disent *pourquoi*
deux choses sont liées, des membranes qui regroupent, des dossiers qui s'ouvrent sur un autre
canevas. On zoome pour voir le détail, on dézoome pour voir la forme d'ensemble.

Glucose sert à monter un moodboard, préparer du concept art, organiser une campagne de jeu de
rôle, démêler un sujet d'étude. Son horizon est plus large : rendre navigable une carte
immense, jusqu'à **l'univers entier des connaissances**, des millions de textes, de liens et
de domaines, sur lequel on se déplace sans effort.

L'interface est volontairement presque absente : monochrome, plate, sans décor. **La couleur
appartient à ce que vous posez**, pas au logiciel.

## Pourquoi tout réécrire en Rust

Glucose Tauri tenait beaucoup d'images, mais il portait un navigateur entier. Passer au natif
n'a de sens que pour aller plus loin, sur chacun de ces points à la fois :

- **Toujours fluide.** Au moins cent images par seconde, quoi qu'il se passe. Quand la
  machine ne suit plus, ce qui cède est la finesse des images *pendant* le mouvement, jamais
  la cadence. À l'arrêt, tout est net.
- **Aucune machine exclue.** Un vieux PC, une machine sans carte graphique, un bureau
  distant : Glucose doit s'ouvrir partout, et utiliser tout ce que chaque machine offre.
- **Deux moteurs, pas un compromis.** Le processeur et la carte graphique ont chacun leur
  voie, écrite pour ce qu'ils font le mieux. Glucose choisit sa carte graphique en la
  regardant travailler, et cherche à se placer **là où il reste de la place** : il ne doit
  jamais disputer les ressources à Blender ou Photoshop ouverts à côté.
- **Des millions d'éléments.** Le noyau est conçu pour dix millions de nœuds.
- **Presque rien sous le capot.** Le noyau n'a aucune dépendance. Chaque dépendance de
  l'application se justifie par une impossibilité de faire sans, pas par le confort.

## Ce qui fonctionne aujourd'hui

Tout ce qui modifie le document s'annule par `Ctrl+Z` et s'enregistre avec lui. Le détail,
touche par touche, est dans le **[guide](GUIDE.md)**.

| | |
|---|---|
| **Naviguer** | canevas infini, zoom au curseur, pavé tactile (deux doigts pour se déplacer, pincer pour zoomer), élan fluide, `F` cadre tout le contenu, **signets de vue** (`Ctrl+1` à `9` pour poser, `1` à `9` pour y voler), minimap qu'on tient pour voyager, plusieurs tableaux en onglets |
| **Images** | PNG, JPEG, WebP, GIF, BMP ; import, collage, glisser-déposer de plusieurs fichiers à la fois ; **depuis un navigateur** sous Windows : Glucose télécharge lui-même l'image, une épingle Pinterest arrive en pleine résolution ; rotation, **recadrage non destructif**, `Ctrl+B` retire les bordures unies d'un lot d'images, verrouillage |
| **Texte** | cartes Markdown (titres, listes, citations, blocs de code, tableaux, liens), gras, italique, barré, **formules LaTeX** rendues nativement et vectorielles, nettes à tout zoom ; édition complète au clavier et à la souris |
| **Relier** | flèches tracées au glisser, qui s'accrochent aux éléments ; coudes, étiquettes, et **six relations** qu'on pose d'une touche : *est précurseur de*, *contredit*, *hérite de*, *inspire*, *dépend de*, *illustre* |
| **Organiser** | membranes, dossiers qui s'ouvrent sur un sous-canevas avec une plongée animée, domaines colorés assignés avec un poids, panneau Ordonner, alignement magnétique avec guides |
| **Fichiers** | format `.glucose` binaire, écriture atomique, somme de contrôle par section, chaque image stockée une seule fois ; alerte à la fermeture ; export Markdown et SVG ; un fichier texte déposé devient une carte, tout autre fichier une tuile qui y mène |

## Ce qui manque encore

Glucose Tauri reste la cible, fonctionnalité par fonctionnalité. Il manque notamment :

- la **collaboration**, les **plugins** et l'**IA locale** : leurs panneaux existent et disent
  honnêtement qu'ils ne font rien encore ;
- le **Storyboard** et l'application des **Presets** ;
- la recherche, la Time Machine, la réglette temporelle ;
- la sauvegarde automatique et la récupération après un plantage ;
- l'ouverture des fichiers de Glucose Tauri ;
- l'export en PNG et en HTML, et les vidéos ;
- **macOS, Linux et Android** : le code est écrit pour y tourner, mais seul Windows a été
  vérifié. Le glisser-déposer depuis un navigateur n'existe que sous Windows.

## Essayer Glucose Rust

Il n'y a pas encore d'installateur. Il faut [Rust](https://rustup.rs) (version stable), puis :

```bash
git clone https://github.com/shazamifius/GlucoseGit.git
cd GlucoseGit
cargo run -p glucose-desktop --release
```

La première compilation prend quelques minutes. L'exécutable se trouve ensuite dans
`target/release/`.

Aucun Node.js, aucun navigateur, aucune carte graphique n'est requis. Sous Linux, les
dialogues de fichiers demandent GTK 3 (`libgtk-3-dev` sous Debian et Ubuntu) ; cette
plateforme n'est pas encore vérifiée, et un retour y est précieux.

## Glucose Tauri, l'ancienne version

| | |
|---|---|
| **Ce que c'est** | la première version de Glucose : React et TypeScript dans une fenêtre Tauri |
| **Son code** | la branche [`tauri-v1.0.1`](https://github.com/shazamifius/GlucoseGit/tree/tauri-v1.0.1), figée |
| **La télécharger** | l'onglet [Releases](https://github.com/shazamifius/GlucoseGit/releases), de `v0.2.0` à `v1.0.2-beta.1` (Windows, macOS, Linux) |
| **Son avenir** | aucun correctif. Elle reste la **référence** visuelle et fonctionnelle de Glucose Rust |

## Mesuré, pas affirmé

Chaque chiffre se refait par une commande, sur des documents synthétiques déterministes. Ils
sont datés : le projet avance vite, et un chiffre ancien ne doit pas passer pour actuel.

| | Mesure | Pour la refaire |
|---|---|---|
| **Dépendances du noyau** | **0** : `glucose-core` n'utilise que la bibliothèque standard | `cargo tree -p glucose-core` |
| **Dépendances de l'application** | **12** directes, **132** dans l'arbre complet sous Windows, dont l'essentiel pour la couche graphique portable *(23/09/2026)* | `cargo tree --workspace` |
| **Dix millions de nœuds** | **424 Mo** en mémoire, chargés en **141 ms**, une requête de vue en **0,21 µs** *(14/09/2026)* | `cargo run --release -p glucose-core --example bench_arena` |
| **429 photos, écran 2560 × 1600** | processeur : 4,2 à 6,3 ms par image, **pixelisée** ; carte graphique intégrée : **1,22 ms, nette** ; carte dédiée : **0,26 ms, nette** *(21/09/2026)* | `cargo run --release -p glucose-desktop --example bench_voie_gpu` |
| **Tests** | environ **1 500** *(23/09/2026)* | `cargo test --workspace` |

La troisième ligne dit pourquoi Glucose a deux moteurs : la carte graphique filtre les images
dans son silicium, et rend net pour un cinquième du prix ce que le processeur ne tient qu'en
abîmant. Le processeur, lui, reste une voie de plein droit, pas un secours.

## Sous le capot

```
crates/
├── glucose-core/      le noyau : modèle, géométrie, index spatial, annulation,
│                      format de fichier, texte, SHA-256. Aucune dépendance, testé sans écran.
├── glucose-math/      les formules LaTeX en géométrie pure (katex-rs, le portage Rust de KaTeX)
└── glucose-desktop/   l'application : la fenêtre (winit), les deux voies de rendu
                       (tiny-skia au processeur, wgpu pour la carte graphique), les glyphes
                       (fontdue), le décodage d'images, les dialogues, le presse-papiers
```

`wgpu` est la plus grosse dépendance, et elle est assumée : c'est elle qui atteint Vulkan,
Metal, Direct3D et OpenGL ES derrière une seule interface. Écrire quatre pilotes à la main
couvrirait moins de machines, pas plus.

## Documentation

| | |
|---|---|
| [`GUIDE.md`](GUIDE.md) | **utiliser Glucose** : chaque geste, chaque touche |
| [`docs/`](docs/README.md) | le carnet de bord de l'ingénierie : plans, mesures, décisions. En français, écrit au fil des sessions |
| [`style.md`](style.md) | le langage visuel : pourquoi l'interface est monochrome et brutaliste |
| [`PLUGINS.md`](PLUGINS.md) | la vision du système de plugins, pas encore construit |

## Contribuer

Les retours d'usage valent autant que le code. Une idée, une question, un avis : les
[Discussions](https://github.com/shazamifius/GlucoseGit/discussions). Un bug : une
[issue](https://github.com/shazamifius/GlucoseGit/issues/new/choose). Pour le code, lire
d'abord [`CONTRIBUTING.md`](.github/CONTRIBUTING.md).

## Soutenir le projet

Glucose est libre et le restera. Pour aider à le faire avancer :
[GitHub Sponsors](https://github.com/sponsors/shazamifius) ou
[Ko-fi](https://ko-fi.com/shazamifius).

## Licence

[MIT](LICENSE) : utilisation, modification et redistribution libres.
