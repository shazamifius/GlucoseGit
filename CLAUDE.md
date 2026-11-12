# Glucose — pour une session de Claude

Glucose est un canevas infini (photos, textes, flèches, membranes) réécrit en Rust natif.
**Glucose Rust** est ce dépôt (`main`, `crates/`) ; **Glucose Tauri**, la première version, est la
référence figée (branche `tauri-v1.0.1` ; pour la lire : `git worktree add --detach <scratchpad>/tauri
origin/tauri-v1.0.1`, puis `git worktree remove` en sortant).

**On travaille sur son VivoBook sous NixOS** (i7-8550U, 7,6 Go, Intel UHD 620 et MX150, écran 60 Hz,
Wayland, niri), depuis le 6/11/2026. Les fiches d'avant parlent de son PC Windows, cassé : ce qui y parle
de `C:\`, de PowerShell, de WSL ou de `.exe` ne vaut plus ici. La carte de la machine et de ses projets :
`~/Projets/LISEZ-MOI.md`.

## Lire, dans cet ordre — environ 10 000 mots

1. **[`docs/VISION.md`](docs/VISION.md), en entier** : ses paroles, datées, et **la charte** (100 images
   par seconde, aucune machine exclue, deux voies, l'élégance, tout a une signification). C'est le seul
   endroit où elles vivent.
2. **[`docs/ETAT.md`](docs/ETAT.md)** : le présent, en une page.
3. **[`docs/SUITE.md`](docs/SUITE.md)** : § 2 (ce qu'on fait maintenant, dans l'ordre) et § 4 (ce qui
   attend sa parole) ; le reste quand on en a besoin.
4. **[`docs/METHODE.md`](docs/METHODE.md)** : § 1 à 4 en entier (les règles, prouver, l'ordre du travail,
   les principes) ; § 5 (les gestes de cette machine) au moment de s'en servir.
5. **La fiche du chantier qu'on prend** (le carnet, [`docs/carnet/00-INDEX.md`](docs/carnet/00-INDEX.md)),
   et celles qu'elle cite ; **[`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md)** à la section du code qu'on
   touche.

**Un fait, un seul endroit** : ses paroles dans VISION, le présent dans ÉTAT, le plan dans SUITE, la façon
de travailler dans MÉTHODE, le détail d'un chantier dans sa fiche. Ce qui se répète ailleurs finit par se
contredire (sa parole du 11/11). Les mémoires de Claude ne gardent que des préférences de travail.

Puis : prendre le premier chantier ouvert de SUITE § 2, **tout remettre en question** — ses paroles
comprises, avec des arguments —, et **chercher sur internet avant de concevoir**, sources rapportées.

Le code cite le carnet par numéro (« fiche 22 § 5 ») : c'est `docs/carnet/22-…`. Ne jamais renuméroter une
fiche. **La prochaine porte le numéro 112.**

## Avec lui

Il ne code pas, parle français, ne voit pas la console, et veut être contredit quand on a un argument. Il
porte la vision et le ressenti ; la technique se décide et s'exécute seule. Ce qu'il lance s'écrit dans un
fichier (`glucose > ~/sortie-glucose.txt 2>&1`), qu'on lit soi-même. **C'est à nous de tester et de
juger** (sa parole du 18/10) : une fenêtre sur son écran est permise pour un essai, puis fermée (`bash
outils/essai_isole.sh <secondes> [capture.png]`) ; les bancs restent hors écran. **Ses données avant
tout** : diagnostic sur une copie ; son dossier Glucose (`~/.local/state/glucose`) ne se touche jamais.
Questions en prose, jamais en menu. Dire « nœud » (jamais « carte » pour un élément du tableau) et « carte
graphique » en entier. Un point en français simple toutes les 30 à 45 minutes, l'heure lue (`date`).

## Les commandes

Rust n'existe ici que dans `nix develop` : préfixer par `nix develop -c`. **Une seule grosse compilation
à la fois** (7,6 Go) : compter 5 à 8 minutes pour une compilation d'épreuves. **Et toujours bornée** :
toute compilation, suite, mesure ou `nix profile upgrade` passe par `bash outils/borne.sh <commande>`.
Le 12/11, un `cargo test --workspace` nu a lancé huit compilations à la fois et figé sa machine deux
heures et demie (le swap compressé ne tue rien) ; bornée, une commande qui déborde meurt seule.

```bash
cargo fmt --all -- --check
bash outils/borne.sh nix develop -c cargo clippy --workspace --all-targets -- -D warnings
bash outils/borne.sh nix develop -c cargo test --workspace --no-fail-fast  # ~3 160 épreuves, dont les cliquets
GH_TOKEN=… bash outils/suivre_ci.sh <empreinte complète> <sortie.txt>  # la vérification GitHub (dix tâches)
python3 outils/saboter.py <sabotages.json>  # exiger qu'une épreuve tombe quand on casse sa garde
bash outils/pilote/piloter.sh demarrer <maison> <sortie>  # le vrai Glucose hors écran : clic, molette, taper, capture
node outils/telemetrie/tableau.mjs <dossier>  # la télémétrie, plantages compris : à lire en début de session
bash outils/mettre_a_jour.sh  # son Glucose installé, après un envoi qui change le programme (sous chien de garde)
```

Cloudflare, si `tableau.mjs` dit qu'on n'est pas connecté — demander son clic, et lui donner le lien :
`cd outils/telemetrie && npx wrangler login --device --browser=false --scopes account:read user:read d1:write`
(ajouter `workers:write workers_scripts:write` pour déployer le serveur). **Le serveur refuse toute ligne
qu'il ne connaît pas : il se déploie AVANT la version qui l'écrit** ; `outils/publication/verifier_serveur.py`
arrête une publication qui l'oublierait.

## Les règles qui ne se discutent pas

* **Rien à moitié** : branché, visible, annulable, enregistré.
* **Les cliquets** (`crates/glucose-desktop/tests/cliquets_suite.rs`) ne se relèvent jamais : 80 lignes par
  fonction, 600 par fichier. On extrait.
* **Le noyau (`glucose-core`) n'a aucune dépendance.** Une dépendance ailleurs exige sa note dans
  `docs/carnet/decisions/`.
* **Une épreuve qui ne tombe pas quand on casse ce qu'elle garde est aveugle** : saboter chaque garde.
* **« Réglé » seulement vérifié** : sur une copie, sur GitHub, ou à son écran.
* **Les messages de commit** s'écrivent dans un fichier du scratchpad, puis `git commit -F` ; le dernier
  d'un envoi porte `[skip ci]` (la CI seulement avant une publication).
* **Jamais de jeton dans le dépôt.** Il le donne dans la conversation, sans remarque de notre part ; envoyer
  sans l'écrire nulle part : `git push "https://x-access-token:$JETON@github.com/shazamifius/GlucoseGit.git"
  main:main`. Git n'a pas d'identité globale sur cette machine : celle du dépôt est réglée.
* **Deux branches, pas une de plus** : `main` et `tauri-v1.0.1`. Une branche de travail se retire, en local
  et sur GitHub, dès qu'elle est fusionnée.
* **Publier une version reste son geste** : le brouillon se prépare, le clic « Publish release » est le sien.

## En fin de session

[MÉTHODE § 6](docs/METHODE.md) : ÉTAT réécrit au présent, SUITE sans ce qui est fini, une fiche datée et
sa ligne à l'index, ses paroles nouvelles dans VISION, une seule liste courte d'essais à son écran.
