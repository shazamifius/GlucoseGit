# La documentation de Glucose

> [!NOTE]
> **Vous voulez utiliser Glucose ?** Lisez le [guide d'utilisation](../GUIDE.md). Ce dossier est
> la documentation de l'**ingénierie**, en français.

## Cinq documents, et une règle : un fait, un seul endroit

| | | ce qu'il dit |
|---|---|---|
| 1 | [**VISION**](VISION.md) | ce que Glucose veut devenir : les paroles de son propriétaire, datées, et la charte |
| 2 | [**ÉTAT**](ETAT.md) | le présent, en une page : ce qui est publié, ce qui marche, ce qui ne va pas |
| 3 | [**SUITE**](SUITE.md) | ce qu'on fait ensuite, dans l'ordre, et ce qui attend sa parole |
| 4 | [**MÉTHODE**](METHODE.md) | les règles du code, la façon de prouver, les principes payés, les gestes de la machine |
| 5 | [**ARCHITECTURE**](ARCHITECTURE.md) | comment le code est fait aujourd'hui, et où vit chaque chose |

Ils disent **toujours le présent**, et se réécrivent quand la réalité change. L'histoire vit dans le
carnet.

## Le reste

| | |
|---|---|
| [`carnet/`](carnet/00-INDEX.md) | le **carnet de bord** : une fiche datée par session (104 au 11/11/2026). Elles ne se réécrivent pas, et le code les cite par leur numéro. Les fiches 06 à 10 sont la **spécification de la référence** (Glucose Tauri relevé au pixel) ; [`carnet/decisions/`](carnet/decisions/), une note par dépendance ; deux archives, les leçons une par une et le PC Windows |
| [`INVENTAIRE`](INVENTAIRE.md) | chaque fonction, et par où la main la trouve, au PC et au téléphone |
| [`PUREREF`](PUREREF.md), [`TELEMETRIE`](TELEMETRIE.md) | la cible PureRef ; ce que la télémétrie envoie, et comment on la lit |
| [`heritage/`](heritage/README.md) | la feuille de route et la spécification de **Glucose Tauri**, la première version |
| [`../style.md`](../style.md) | le langage visuel : monochrome, brutaliste, la couleur appartient au contenu |
| [`../PLUGINS.md`](../PLUGINS.md) | la vision du système de plugins, pas encore construit |

## Glucose Tauri et Glucose Rust

| | Glucose Tauri | Glucose Rust |
|---|---|---|
| **Quoi** | la première version : React, TypeScript et PixiJS dans une fenêtre Tauri | le même logiciel, recréé de zéro en Rust natif |
| **Où** | la branche [`tauri-v1.0.1`](https://github.com/shazamifius/GlucoseGit/tree/tauri-v1.0.1), figée | la branche `main`, dossier `crates/` |
| **Versions** | jusqu'à 1.0.2-beta.1 | depuis 2.0.1-beta.1 (30/09/2026) |
| **Son rôle** | la **référence** : ce que Glucose Rust doit savoir faire et montrer, puis mieux | le projet |
