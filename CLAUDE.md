# Glucose — pour une session de Claude

Glucose est un canevas infini (photos, textes, flèches, membranes) réécrit en Rust natif.
**Glucose Rust** est ce dépôt (`main`, `crates/`) ; **Glucose Tauri**, la première version, est la
cible figée (branche `tauri-v1.0.1` ; pour la lire : `git worktree add --detach <scratchpad>/tauri
origin/tauri-v1.0.1`, puis `git worktree remove` en sortant).

**Depuis le 6/11/2026, on travaille sur son VivoBook sous NixOS** (i7-8550U, 7,6 Go, Intel UHD 620 et
MX150, écran 60 Hz, Wayland, niri), et plus sur le PC Windows que décrivent les anciennes fiches : tout
ce qui parle de `C:\`, de PowerShell, de WSL ou de `.exe` se revérifie. La carte de la machine et de
tous ses projets : `~/Projets/LISEZ-MOI.md`.

## Avant tout

1. Lire **toutes** les mémoires, une par une, en entier : elles portent la charte (100 images par
   seconde, aucune machine exclue, deux voies processeur et carte graphique, élégance
   mathématique) et la façon de travailler avec le propriétaire.
2. Lire `docs/VISION.md` (ce que Glucose veut devenir, sa parole), puis `docs/ETAT.md`,
   `docs/SUITE.md`, `docs/METHODE.md` et `docs/ARCHITECTURE.md`.
3. Prendre le premier chantier non coché de `docs/SUITE.md`, et **tout remettre en question**,
   ses paroles comprises, avec des arguments et, avant de concevoir, une recherche sur internet.

Le code cite le carnet par numéro (« fiche 22 § 5 ») : c'est `docs/carnet/22-…`. Ne jamais
renuméroter une fiche. La prochaine porte le numéro 90.

## Avec lui

Il ne code pas, parle français, ne voit pas la console, et veut être contredit quand on a un
argument. Ce qu'il lance s'écrit dans un fichier (`cargo run --release > sortie-x.txt 2>&1`), qu'on
lit soi-même. **C'est à nous de tester et de juger** (sa parole du 18/10) : une fenêtre sur son
écran est permise pour un essai, puis fermée (`bash outils/essai_isole.sh <secondes> [capture.png]`,
qui isole ses données et coupe la télémétrie d'avance) ; les bancs restent hors écran. **Ses données
avant tout** : diagnostic sur une copie, et son dossier Glucose (`~/.local/state/glucose`,
`%LOCALAPPDATA%\Glucose` sous Windows) ne se touche jamais. Questions en prose, jamais en menu.

## Les commandes

Rust n'existe ici que dans `nix develop` : préfixer par `nix develop -c`. **Une seule grosse
compilation à la fois** (7,6 Go) : compter 5 à 8 minutes pour une compilation d'épreuves.

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --no-fail-fast  # ~2 700 épreuves, dont les cliquets
GH_TOKEN=… bash outils/suivre_ci.sh <empreinte complète> <sortie.txt>  # la vérification GitHub (dix tâches)
bash outils/android/construire.sh      # l'APK (SDK et NDK dans ~/Android/Sdk, ABSENTS ici : la CI le construit)
python3 outils/saboter.py <sabotages.json>  # exiger qu'une épreuve tombe quand on casse sa garde
node outils/telemetrie/tableau.mjs <dossier>  # la télémétrie, plantages compris : à lire en début de session
```

Cloudflare, si `tableau.mjs` dit qu'on n'est pas connecté — demander son clic, et lui donner le lien :
`cd outils/telemetrie && npx wrangler login --device --browser=false --scopes account:read user:read d1:write`
(ajouter `workers:write workers_scripts:write` pour déployer le serveur). **Le serveur refuse toute ligne
qu'il ne connaît pas : il se déploie AVANT la version qui l'écrit** ; `outils/publication/verifier_serveur.py`
arrête une publication qui l'oublierait.

## Les règles qui ne se discutent pas

* **Rien à moitié** : branché, visible, annulable, enregistré.
* **Les cliquets** (`crates/glucose-desktop/tests/cliquets_suite.rs`) ne se relèvent jamais :
  80 lignes par fonction, 600 par fichier. On extrait.
* **Le noyau (`glucose-core`) n'a aucune dépendance.** Une dépendance ailleurs exige sa note dans
  `docs/carnet/decisions/`.
* **Une épreuve qui ne tombe pas quand on casse ce qu'elle garde est aveugle** : saboter chaque garde.
* **« Réglé » seulement vérifié** : sur une copie, sur GitHub, ou à son écran.
* **Les messages de commit** s'écrivent dans un fichier du scratchpad, puis `git commit -F`.
* **Jamais de jeton dans le dépôt.** Envoyer sans l'écrire nulle part :
  `git push "https://x-access-token:$JETON@github.com/shazamifius/GlucoseGit.git" main:main`. Git n'a pas
  d'identité globale sur cette machine : celle du dépôt est réglée (`git config user.name`).
* **Deux branches, pas une de plus** : `main` et `tauri-v1.0.1`. On travaille sur `main` ; une
  branche de travail se retire, en local et sur GitHub, dès qu'elle est fusionnée.

## En fin de session

Mettre à jour `docs/ETAT.md` et cocher `docs/SUITE.md` ; une fiche datée dans le carnet si le
travail le mérite ; lui donner une seule liste courte d'essais à l'écran.
