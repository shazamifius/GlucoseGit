# Glucose — pour une session de Claude

Glucose est un canevas infini (photos, textes, flèches, membranes) réécrit en Rust natif.
**Glucose Rust** est ce dépôt (`main`, `crates/`) ; **Glucose Tauri**, la première version, est la
cible figée (branche `tauri-v1.0.1`, lisible dans `C:\Users\Administrator\Documents\GlucoseTauri\`).

## Avant tout

1. Lire **toutes** les mémoires, une par une, en entier : elles portent la charte (100 images par
   seconde, aucune machine exclue, deux voies processeur et carte graphique, élégance
   mathématique) et la façon de travailler avec le propriétaire.
2. Lire `docs/ETAT.md`, `docs/SUITE.md`, `docs/METHODE.md`, puis `docs/ARCHITECTURE.md`.
3. Prendre le premier chantier non coché de `docs/SUITE.md`, et **tout remettre en question**,
   ses paroles comprises, avec des arguments et, avant de concevoir, une recherche sur internet.

Le code cite le carnet par numéro (« fiche 22 § 5 ») : c'est `docs/carnet/22-…`. Ne jamais
renuméroter une fiche. La prochaine porte le numéro 57.

## Avec lui

Il ne code pas, parle français, ne voit pas la console, et veut être contredit quand on a un
argument. Ce qu'il lance s'écrit dans un fichier (`cargo run --release > sortie-x.txt 2>&1`), qu'on
lit soi-même. **Jamais de fenêtre sur son écran** pour mesurer. **Ses données avant tout** :
diagnostic sur une copie, et `%LOCALAPPDATA%\Glucose` ne se touche jamais. Questions en prose,
jamais en menu.

## Les commandes

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace                 # ~2 110 épreuves, dont les cliquets
bash outils/suivre_ci.sh <empreinte complète> <sortie.txt>  # la vérification GitHub (dix tâches)
bash outils/android/construire.sh      # l'APK (SDK et NDK dans ~/Android/Sdk, Gradle 9.6+)
python outils/saboter.py <sabotages.json>   # exiger qu'une épreuve tombe quand on casse sa garde
```

## Les règles qui ne se discutent pas

* **Rien à moitié** : branché, visible, annulable, enregistré.
* **Les cliquets** (`crates/glucose-desktop/tests/cliquets_suite.rs`) ne se relèvent jamais :
  80 lignes par fonction, 600 par fichier. On extrait.
* **Le noyau (`glucose-core`) n'a aucune dépendance.** Une dépendance ailleurs exige sa note dans
  `docs/carnet/decisions/`.
* **Une épreuve qui ne tombe pas quand on casse ce qu'elle garde est aveugle** : saboter chaque garde.
* **« Réglé » seulement vérifié** : sur une copie, sur GitHub, ou à son écran.
* **Les messages de commit** s'écrivent dans un fichier du scratchpad, puis `git commit -F` : Git
  Bash mange les barres obliques inverses.
* **Jamais de jeton dans le dépôt.**

## En fin de session

Mettre à jour `docs/ETAT.md` et cocher `docs/SUITE.md` ; une fiche datée dans le carnet si le
travail le mérite ; lui donner une seule liste courte d'essais à l'écran.
