# Contribuer à Glucose

Merci de l'intérêt que tu portes à Glucose ! Les contributions — code, idées,
rapports de bugs, retours — sont les bienvenues.

## 🐛 Signaler un bug / proposer une idée

- Un **bug** ? Ouvre une [issue](../../issues) avec : ce que tu faisais, ce qui
  était attendu, ce qui s'est passé, et ta plateforme (OS + version).
- Une **idée** ou une question ouverte ? Lance une
  [Discussion](../../discussions) — c'est l'endroit pour débattre avant de coder.

## 🛠️ Mettre en place l'environnement de dev

**Prérequis :**
- [Rust](https://www.rust-lang.org/tools/install) (toolchain stable ; le projet est vérifié avec la 1.95)
- Aucun Node.js, aucun outil tiers : le dossier `src/` est l'ancienne version TypeScript, conservée comme inventaire, et ne se compile plus ici.

**Lancer en local :**

```bash
# Compiler et lancer l'application
cargo run -p glucose-desktop --release
```

## ✅ Avant d'ouvrir une Pull Request

Avant d'ouvrir une PR, assure-toi que la suite de tests et les vérifications passent localement :

```bash
# Le formatage, le linter strict et tous les tests — c'est exactement ce que la CI exécute
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

**Règles d'or d'architecture :**
- **ZÉRO DÉPENDANCE DANS LE NOYAU** : `crates/glucose-core` garde un `[dependencies]` strictement vide. Modèle, géométrie, hachage, sérialisation, algorithmes : tout est écrit sur `std`.
- **Rien à moitié** : une fonctionnalité n'est finie que si elle est branchée, visible, annulable et enregistrée. Un bouton dont la fonction n'existe pas le dit — il ne simule jamais.
- **L'ancien code TypeScript n'est pas un modèle** : il sert d'inventaire de ce qui existe, jamais de justification d'une implémentation.
- Une PR = un sujet. Décris le *pourquoi*, pas seulement le *quoi*.
- Tout nouveau comportement non trivial est **couvert par un test**, et tout module du noyau arrive **avec son appelant**.
- Les règles complètes sont dans [`docs/architecture/05-STANDARDS-DE-CODE.md`](docs/architecture/05-STANDARDS-DE-CODE.md) ; l'état du projet dans [`docs/architecture/00-INDEX.md`](docs/architecture/00-INDEX.md).

## 📜 Code de conduite

Ce projet suit un [Code de conduite](CODE_OF_CONDUCT.md). En participant, tu
t'engages à le respecter.

