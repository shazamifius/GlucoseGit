# La documentation de Glucose

> [!NOTE]
> **Vous voulez utiliser Glucose ?** Ce dossier n'est pas pour vous : lisez le
> [guide d'utilisation](../GUIDE.md). Ici vit le **carnet de bord de l'ingénierie** : les plans,
> les mesures et les décisions, écrits en français au fil des sessions de travail.

## D'abord : Glucose Tauri, puis Glucose Rust

Glucose a connu deux logiciels, et ce dossier parle des deux. Il faut savoir lequel on lit.

| | Glucose Tauri | Glucose Rust |
|---|---|---|
| **Quoi** | la première version : React, TypeScript et PixiJS dans une fenêtre Tauri | le même logiciel, recréé de zéro en Rust natif |
| **Où** | la branche [`tauri-v1.0.1`](https://github.com/shazamifius/GlucoseGit/tree/tauri-v1.0.1) | la branche `main`, dossier `crates/` |
| **État** | **figée**. Plus aucun correctif | en développement actif |
| **Son rôle aujourd'hui** | la **cible** : ce que Glucose Rust doit faire et montrer, à l'identique puis mieux | le projet |

Quand un document dit « le code TypeScript », « `src/` » ou « la référence », il parle de
Glucose Tauri. Ce code n'est plus sur `main` : il se lit sur la branche `tauri-v1.0.1`.

## Ce que contient ce dossier

### [`architecture/`](architecture/00-INDEX.md) : le carnet de bord de Glucose Rust

Les fiches sont **numérotées dans l'ordre où elles ont été écrites**, et datées. Quand une
mesure dément ce qu'une fiche affirmait, la correction s'ajoute et se date, plutôt que
d'effacer ce qui a été cru. **La plus récente sur un sujet fait donc foi.** L'[index](architecture/00-INDEX.md) les liste
toutes, avec pour chacune quand la lire.

| Fiches | Ce qu'elles sont | Actuelles ? |
|---|---|---|
| **01 à 04** | l'état des lieux du 10 septembre 2026 : audit du code, architecture visée, inventaire des fonctionnalités, première feuille de route | historiques : les chiffres ont beaucoup changé depuis |
| **05** | les standards de code | **en vigueur** |
| **06 à 10** | la **spécification de la cible**, relevée sur Glucose Tauri : couleurs, dimensions, animations, comportements, panneaux | **en vigueur** : elles disent *quoi* faire |
| **11 à 16, 18, 21** | les plans : ordre de marche, performance, adaptation au matériel, les deux moteurs (processeur et carte graphique) | le plus récent d'abord : la [fiche 21](architecture/21-LES-VOIES.md) |
| **17, 19, 20, 22 à 26** | les comptes rendus de session : ce que la mesure a trouvé, et ce qu'elle a démenti | des journaux, datés |
| **27** | le chantier en cours : la provenance d'une image, sa meilleure qualité, son auteur | **en cours** |
| [`decisions/`](architecture/decisions/) | une note par décision structurante | en vigueur |

Les captures du dossier [`architecture/screens/`](architecture/screens/) sont des captures de
**Glucose Tauri**. Elles servent de référence visuelle aux fiches 06 à 10 et ne montrent pas
Glucose Rust.

### [`heritage/`](heritage/README.md) : les archives de Glucose Tauri

La feuille de route et la spécification de la première version. Elles restent utiles pour
deux raisons : elles inventorient ce que Glucose doit savoir faire, et elles portent la vision,
qui n'a pas changé. Ce qu'elles marquent « fait » l'est dans Glucose Tauri, **pas** dans
Glucose Rust.

## Ailleurs dans le dépôt

| | |
|---|---|
| [`GUIDE.md`](../GUIDE.md) | utiliser Glucose Rust, geste par geste |
| [`style.md`](../style.md) | le langage visuel de Glucose, commun aux deux versions |
| [`PLUGINS.md`](../PLUGINS.md) | la vision du système de plugins, qui n'existe encore dans aucune des deux versions sous cette forme |
| [`CONTRIBUTING.md`](../.github/CONTRIBUTING.md) | les règles pour proposer du code |
