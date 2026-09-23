# Contribuer à Glucose

Merci de l'intérêt que vous portez à Glucose. Les contributions sont bienvenues sous toutes
leurs formes, et **les retours d'usage comptent autant que le code** : dire qu'un geste est
pénible, qu'une chose manque ou qu'une autre est belle aide à décider de la suite.

## Quelle version ?

Ce dépôt porte **Glucose Rust**, sur la branche `main`. L'ancienne version, **Glucose Tauri**
(branche `tauri-v1.0.1`), est figée : elle ne reçoit plus de correctifs, et une contribution
qui la vise ne sera pas intégrée. Un défaut constaté dans Glucose Tauri reste intéressant s'il
existe aussi dans Glucose Rust.

## Donner un avis, signaler un problème

- **Une idée, une question, un avis** : les
  [Discussions](https://github.com/shazamifius/GlucoseGit/discussions). C'est l'endroit pour en
  parler avant d'écrire du code.
- **Un bug** : une [issue](https://github.com/shazamifius/GlucoseGit/issues/new/choose). Dites
  ce que vous faisiez, ce que vous attendiez, ce qui s'est passé, et sur quelle machine
  (système, carte graphique, taille d'écran).
- **Une faille de sécurité** : jamais en public. Suivez [`SECURITY.md`](SECURITY.md).

## Compiler et lancer

Il faut [Rust](https://rustup.rs) (version stable), et rien d'autre : ni Node.js, ni outil
tiers. Sous Linux, les dialogues de fichiers demandent GTK 3 (`libgtk-3-dev`).

```bash
cargo run -p glucose-desktop --release
```

## Avant de proposer du code

Ces trois commandes doivent passer, sans exception :

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Et ces règles tiennent tout le projet. Elles sont détaillées dans les
[standards de code](../docs/architecture/05-STANDARDS-DE-CODE.md).

- **Le noyau n'a aucune dépendance.** `crates/glucose-core` garde un `[dependencies]` vide :
  modèle, géométrie, hachage, format de fichier, tout est écrit sur la bibliothèque standard.
  Ailleurs, une dépendance se défend par une impossibilité de faire sans, pas par le confort.
- **Rien à moitié.** Une fonctionnalité n'est finie que si elle est branchée, visible,
  annulable et enregistrée. Un bouton dont la fonction n'existe pas le dit ; il ne fait
  jamais semblant.
- **Glucose Tauri dit *quoi*, jamais *comment*.** Il est la référence de ce que Glucose doit
  faire et montrer. Son code n'est pas un modèle : on ne traduit pas le TypeScript, on
  recrée.
- **La fluidité ne se négocie pas.** Un travail lourd se découpe pour tenir dans le temps libre
  de chaque image : le rendu n'attend jamais. Une optimisation arrive avec sa mesure, avant et
  après.
- **Un changement, un sujet.** Expliquez le *pourquoi*, pas seulement le *quoi*.
- **Un comportement nouveau arrive avec son test**, et un module du noyau avec son appelant.

Pour comprendre où en est le projet et pourquoi, le [carnet de bord](../docs/README.md) raconte
tout, décision par décision.

## Code de conduite

Ce projet suit un [code de conduite](CODE_OF_CONDUCT.md). En participant, vous vous engagez à
le respecter.
