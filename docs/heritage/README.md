# Héritage — les documents de Glucose Tauri

Ces documents décrivent **la version TypeScript / Tauri** de Glucose, celle que la réécriture
en Rust prend pour cible. Ils sont conservés parce qu'ils servent encore :

- **comme inventaire** de ce qui existait — chaque fonctionnalité qu'ils décrivent est une
  fonctionnalité à retrouver dans la version Rust, à l'identique visuellement, mieux dans le code ;
- **comme vision** — le pont humain ↔ IA, Wikipédia dans Glucose, les plugins comme bus : ce cap
  n'a pas changé.

**Ce qu'ils déclarent « fait », « livré » ou « stable » l'est dans le code TypeScript du dossier
`src/`, pas dans les crates Rust.** L'état réel de la version Rust est tenu, fonctionnalité par
fonctionnalité, dans [`docs/architecture/`](../architecture/00-INDEX.md), et résumé dans le
[README](../../README.md).

| Document | Ce que c'est | Date |
|---|---|---|
| [`ROADMAP-TAURI.md`](ROADMAP-TAURI.md) | La roadmap de Glucose Tauri : vision, décisions cadres, acquis des phases 1 à 7.6, chantiers P1 à P7 | 2026-06-10 (en-tête retouché le 2026-09-10) |
| [`SPECIFICATION-TAURI.md`](SPECIFICATION-TAURI.md) | L'analyse script par script du code TypeScript et du backend Tauri — le « quoi » de chaque module, pour guider la réécriture | 2026-09 |

Règle de lecture, celle de la fiche [05](../architecture/05-STANDARDS-DE-CODE.md) (R2) :
l'ancien code n'est pas un modèle d'implémentation. Ces documents disent *ce que* Glucose fait,
jamais *comment* le Rust doit le faire.
