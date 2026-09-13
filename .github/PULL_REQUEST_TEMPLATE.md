<!--
Merci pour ta contribution ! Voici quelques infos pour faciliter la relecture.
-->

## Description

<!-- Que fait cette PR ? Quel problème elle résout ? Le *pourquoi* avant le *quoi*. -->

## Type de changement

- [ ] 🐛 Correction
- [ ] ✨ Fonctionnalité — branchée, visible, annulable, enregistrée (les quatre)
- [ ] ♻️ Refonte (pas de changement de comportement)
- [ ] 📚 Documentation
- [ ] 🔒 Sécurité
- [ ] ⚡ Performance — avec la mesure avant / après
- [ ] 🧪 Tests

## Chantier

<!-- À quel chantier de docs/architecture/12-PLAN-D-EXECUTION.md ce changement contribue ? -->

## Checklist

- [ ] `cargo fmt --all -- --check` passe
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` passe, sans `#[allow]` ajouté
- [ ] `cargo test --workspace` passe
- [ ] `glucose-core` n'a toujours aucune dépendance
- [ ] Aucun bouton n'annonce une action qui n'a pas lieu
- [ ] Le message de commit décrit ce que fait le code (vérifiable par `grep`)
- [ ] Documentation mise à jour si nécessaire (README, GUIDE, dossier d'architecture)
