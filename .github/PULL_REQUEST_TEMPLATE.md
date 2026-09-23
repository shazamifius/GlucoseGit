<!-- Merci pour cette contribution. Quelques repères pour la relecture. -->

## Pourquoi

<!-- Le problème que ce changement résout, avant ce qu'il fait. -->

## Ce qui change

<!-- Ce que l'utilisateur verra de différent, ou ce qui change sous le capot. -->

## Genre de changement

- [ ] Correction
- [ ] Fonctionnalité : branchée, visible, annulable et enregistrée (les quatre)
- [ ] Refonte, sans changement de comportement
- [ ] Performance, avec la mesure avant et après
- [ ] Documentation
- [ ] Tests

## Vérifications

- [ ] `cargo fmt --all -- --check` passe
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` passe, sans `#[allow]` ajouté
- [ ] `cargo test --workspace` passe
- [ ] `glucose-core` n'a toujours aucune dépendance
- [ ] Aucun bouton n'annonce une action qui n'a pas lieu
- [ ] Essayé dans l'application, sur : <!-- système, carte graphique -->
- [ ] Le guide, le README ou le carnet de bord sont à jour si le changement les touche
