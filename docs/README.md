# La documentation de Glucose

> [!NOTE]
> **Vous voulez utiliser Glucose ?** Lisez le [guide d'utilisation](../GUIDE.md). Ce dossier est
> la documentation de l'**ingénierie**, en français.

## Quatre documents, à lire dans cet ordre

| | | ce qu'il dit |
|---|---|---|
| 1 | [**ÉTAT**](ETAT.md) | où en est Glucose : ce qui est publié, ce qui marche, ce qui manque, ses retours |
| 2 | [**SUITE**](SUITE.md) | ce qu'on fait ensuite, dans l'ordre, et ce qui attend la parole du propriétaire |
| 3 | [**ARCHITECTURE**](ARCHITECTURE.md) | comment le code est fait aujourd'hui, et où vit chaque chose |
| 4 | [**MÉTHODE**](METHODE.md) | les règles qui font foi, la façon de prouver, les leçons payées |

Ces quatre-là se **réécrivent** quand la réalité change : ils disent toujours le présent.

## Le reste

| | |
|---|---|
| [`carnet/`](carnet/00-INDEX.md) | le **carnet de bord** : cinquante fiches datées, du 10/09 au 30/09/2026. Elles ne se réécrivent pas, et le code les cite par leur numéro. Les fiches 06 à 10 y sont la **spécification de la cible** (Glucose Tauri relevé au pixel) ; [`carnet/decisions/`](carnet/decisions/), une note par dépendance |
| [`heritage/`](heritage/README.md) | la feuille de route et la spécification de **Glucose Tauri**, la première version : l'inventaire de ce que Glucose doit savoir faire, et la vision |
| [`../style.md`](../style.md) | le langage visuel : monochrome, brutaliste, la couleur appartient au contenu |
| [`../PLUGINS.md`](../PLUGINS.md) | la vision du système de plugins, pas encore construit |

## Glucose Tauri et Glucose Rust

| | Glucose Tauri | Glucose Rust |
|---|---|---|
| **Quoi** | la première version : React, TypeScript et PixiJS dans une fenêtre Tauri | le même logiciel, recréé de zéro en Rust natif |
| **Où** | la branche [`tauri-v1.0.1`](https://github.com/shazamifius/GlucoseGit/tree/tauri-v1.0.1), figée | la branche `main`, dossier `crates/` |
| **Versions** | jusqu'à 1.0.2-beta.1 | depuis 2.0.1-beta.1 (30/09/2026) |
| **Son rôle** | la **cible** : ce que Glucose Rust doit faire et montrer, à l'identique puis mieux | le projet |
