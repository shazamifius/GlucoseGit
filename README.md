<div align="center">

# Glucose

**Un canevas infini pour penser en images et en texte. Natif, écrit en Rust.**

*Pose. Relie. Zoome. Explore.*

[![Licence MIT](https://img.shields.io/badge/licence-MIT-blue?style=flat-square)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-natif-CE422B?style=flat-square&logo=rust)](https://www.rust-lang.org/)
[![Noyau sans dépendance](https://img.shields.io/badge/noyau-0%20d%C3%A9pendance-brightgreen?style=flat-square)](#mesuré-pas-affirmé)
[![Version](https://img.shields.io/github/v/release/shazamifius/GlucoseGit?include_prereleases&label=version&style=flat-square)](https://github.com/shazamifius/GlucoseGit/releases/latest)

[Télécharger](#télécharger) · [Guide d'utilisation](GUIDE.md) · [Discussions](https://github.com/shazamifius/GlucoseGit/discussions) · [Signaler un bug](https://github.com/shazamifius/GlucoseGit/issues/new/choose)

</div>

> [!IMPORTANT]
> **Glucose 2 est Glucose réécrit en Rust natif**, sans navigateur et sans JavaScript. Si Glucose
> (la version 1, en Tauri) est installé chez vous, la mise à jour vous propose la version 2
> d'elle-même : vos documents s'ouvrent dans la version 2 (`Ctrl+O`), sans être réécrits, et rien
> de vos données n'est touché. C'est une **bêta** :
> une partie des fonctionnalités de la version 1 manque encore ([liste](#ce-qui-manque-encore)).

## Ce qu'est Glucose

Une feuille noire, sans bord, sur laquelle on pose tout ce qui aide à penser : des photos, des
textes en Markdown, des formules, des notes, des flèches qui disent *pourquoi* deux choses sont
liées, des membranes qui regroupent, des dossiers qui s'ouvrent sur un autre canevas. On zoome pour
voir le détail, on dézoome pour voir la forme d'ensemble.

Glucose sert à monter un moodboard, préparer du concept art, organiser une campagne de jeu de rôle,
démêler un sujet d'étude. Son horizon : rendre navigable une carte immense, jusqu'à l'univers des
connaissances.

L'interface est presque absente : monochrome, plate. **La couleur appartient à ce que vous posez.**

## Télécharger

La dernière version est sur la page [**Releases**](https://github.com/shazamifius/GlucoseGit/releases/latest).

| système | fichier |
|---|---|
| **Windows 10 et 11** | `Glucose_…_x64-setup.exe` : installé pour vous seul, sans droits d'administrateur |
| **Linux**, toute distribution | `Glucose_…_amd64.AppImage` |
| **Debian, Ubuntu** | `Glucose_…_amd64.deb` |
| **Fedora** | `Glucose-…x86_64.rpm` |
| **NixOS** | `nix run github:shazamifius/GlucoseGit` |

Sous Windows, un avertissement peut paraître (« Windows a protégé votre ordinateur ») : Glucose
n'est pas signé par un certificat payant. « Informations complémentaires », puis « Exécuter quand
même ». Glucose se met ensuite à jour tout seul.

**Mac, Android et iPad** : pas encore. Android et l'iPad (par le navigateur) sont les prochains.

## Ce qui fonctionne

Tout ce qui modifie le document s'annule par `Ctrl+Z`. Le détail, touche par touche, est dans le
**[guide](GUIDE.md)**.

| | |
|---|---|
| **Naviguer** | canevas infini, zoom au curseur, pavé tactile, `F` cadre tout, signets de vue (`Ctrl+1`…`9`), minimap, onglets |
| **Le document** | enregistré **à chaque geste**, sans jamais figer ; il survit à un plantage, texte en cours compris ; la **Time Machine** (`Ctrl+H`) remonte le temps geste par geste ; les documents de Glucose 1 s'ouvrent |
| **Images** | PNG, JPEG, WebP, GIF, BMP ; collage, glisser-déposer, **depuis un navigateur** sous Windows (une épingle Pinterest arrive en pleine résolution) ; rotation, recadrage non destructif, `Ctrl+B` retire les bordures d'un lot |
| **Texte** | Markdown, tableaux, liens, **formules LaTeX** nettes à tout zoom |
| **Relier** | flèches droites ou courbes, qui **contournent** ce qu'elles croisent ; six relations ; une flèche peut partir d'une **phrase précise** d'un texte |
| **Organiser** | membranes qui emportent leur contenu, mode Focus, dossiers, domaines colorés, rangement automatique, alignement magnétique |

## Ce qui manque encore

Le storyboard, les presets, les rideaux, la réglette temporelle, la recherche, les exports PNG et
HTML, les vidéos, la collaboration, les plugins et l'IA locale. Les boutons qui ne font rien encore
le disent ; aucun ne fait semblant.

## Pourquoi tout réécrire en Rust

- **Toujours fluide** : au moins cent images par seconde, quoi qu'il se passe ; à l'arrêt, tout
  est net.
- **Aucune machine exclue** : un vieux PC, une machine sans carte graphique, un téléphone.
- **Deux moteurs, pas un compromis** : le processeur et la carte graphique ont chacun leur voie, et
  Glucose se place **là où il reste de la place**, pour ne jamais gêner Blender ou Photoshop ouverts
  à côté.
- **Des millions d'éléments** : le noyau est conçu pour dix millions de nœuds.

## Mesuré, pas affirmé

| | mesure | pour la refaire |
|---|---|---|
| **Dépendances du noyau** | **0** : `glucose-core` n'utilise que la bibliothèque standard | `cargo tree -p glucose-core` |
| **Dix millions de nœuds** | 424 Mo, chargés en 141 ms, une requête de vue en 0,21 µs *(14/09/2026)* | `cargo run --release -p glucose-core --example bench_arena` |
| **429 photos, écran 2560 × 1600** | processeur : 4,2 à 6,3 ms, pixelisé ; carte intégrée : **1,22 ms, net** ; carte dédiée : **0,26 ms, net** *(21/09/2026)* | `cargo run --release -p glucose-desktop --example bench_voie_gpu` |
| **Épreuves** | environ **1 970**, sur Windows, Linux, deux Mac, NixOS, Android et le web à chaque envoi *(30/09/2026)* | `cargo test --workspace` |

## Compiler soi-même

Il faut [Rust](https://rustup.rs) (stable), puis :

```bash
git clone https://github.com/shazamifius/GlucoseGit.git
cd GlucoseGit
cargo run -p glucose-desktop --release
```

Sous Linux, les dialogues de fichiers demandent GTK 3 (`libgtk-3-dev` sous Debian et Ubuntu).

## Sous le capot

```
crates/
├── glucose-core/      le noyau : modèle, géométrie, journal, format de fichier, texte — 0 dépendance
├── glucose-math/      les formules LaTeX en géométrie pure (katex-rs)
└── glucose-desktop/   l'application : fenêtre, deux voies de rendu (tiny-skia, wgpu), mise à jour
```

La documentation de l'ingénierie est dans [`docs/`](docs/README.md), en français.

## Glucose 1 (Tauri)

La première version (React et TypeScript dans une fenêtre Tauri) est figée sur la branche
[`tauri-v1.0.1`](https://github.com/shazamifius/GlucoseGit/tree/tauri-v1.0.1). Ses installeurs
restent dans les [Releases](https://github.com/shazamifius/GlucoseGit/releases), jusqu'à
`v1.0.2-beta.1`. Elle reste la référence de ce que Glucose 2 doit savoir faire.

## Contribuer, soutenir

Une idée, une question : les [Discussions](https://github.com/shazamifius/GlucoseGit/discussions).
Un bug : une [issue](https://github.com/shazamifius/GlucoseGit/issues/new/choose). Pour le code :
[`CONTRIBUTING.md`](.github/CONTRIBUTING.md). Pour aider à le faire avancer :
[GitHub Sponsors](https://github.com/sponsors/shazamifius) ou [Ko-fi](https://ko-fi.com/shazamifius).

## Licence

[MIT](LICENSE) : utilisation, modification et redistribution libres.
