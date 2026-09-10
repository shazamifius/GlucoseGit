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
- [Rust](https://www.rust-lang.org/tools/install) (toolchain stable 2021+)
- Aucun Node.js, aucun framework JS lourd requis pour le moteur natif.

**Lancer en local :**

```bash
# Compiler et lancer l'application PureRef native
cargo run -p glucose-desktop --release
```

## ✅ Avant d'ouvrir une Pull Request

Avant d'ouvrir une PR, assure-toi que la suite de tests et les vérifications passent localement :

```bash
# Vérifier la compilation sans erreur ni warning
cargo check --workspace

# Lancer l'intégralité des 202 tests (100% verts)
cargo test --workspace

# Vérifier le linter Clippy
cargo clippy --workspace --all-targets -- -D warnings
```

**Règles d'or d'architecture :**
- **ZÉRO DÉPENDANCE DANS LE CORE** : `crates/glucose-core` doit impérativement conserver un `[dependencies]` strictement vide. Toute logique de données, géométrie, hachage, sérialisation ou algorithme doit être réalisée en pur Rust `std`.
- Une PR = un sujet. Décris le *pourquoi*, pas seulement le *quoi*.
- Tout nouveau comportement non trivial doit être **couvert par un test d'intégration**.
- Respecte les 6 invariants architecturaux (détaillés dans [HANDOFF.md](HANDOFF.md)).

## 📜 Code de conduite

Ce projet suit un [Code de conduite](CODE_OF_CONDUCT.md). En participant, tu
t'engages à le respecter.

