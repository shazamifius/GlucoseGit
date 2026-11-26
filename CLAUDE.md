# Glucose — pour une session de Claude

Glucose est un canevas infini (photos, textes, flèches, membranes) réécrit en Rust natif.
**Glucose Rust** est ce dépôt (`main`, `crates/`) ; **Glucose Tauri**, la première version, est la
référence figée (branche `tauri-v1.0.1` ; pour la lire : `git worktree add --detach <scratchpad>/tauri
origin/tauri-v1.0.1`, puis `git worktree remove` en sortant).

**On travaille sur son poste fixe sous NixOS** (Ryzen 5 7600X, 30 Go, RTX 4070 SUPER ; Wayland, niri),
**définitivement depuis le 22/11/2026** ; sa carte : `~/Projets/LISEZ-MOI.md`. Les fiches du 6 au 22/11
viennent de son VivoBook (i7-8550U, 7,6 Go, Intel UHD 620) : leurs mesures et leurs bornes de mémoire
sont celles de cette petite machine, plus la nôtre. Celles d'avant le 6/11 parlent de son PC Windows,
cassé : ce qui y parle de `C:\`, de PowerShell, de WSL ou de `.exe` ne vaut plus ici.

**Un relais autonome travaille sur Glucose** (22/11) : des sessions de Claude qui continuent seules, d'un
compte à l'autre. Son mode d'emploi : [`outils/relais/MODE-EMPLOI.md`](outils/relais/MODE-EMPLOI.md), à lire
avant de toucher au dépôt. **Une seule session à la fois** (sa parole du 23/11) : ton premier message prend
la main, et une session du relais qui vivait écrit sa passation et s'arrête — s'il en vivait une, ton contexte
le dit (« [Relais] … ») : attends-la. **En partant, rends-lui la main** : `python3 -I outils/relais/relais.py
rendre`. Tout ce qu'on a à dire au relais s'écrit dans son Glucose ; il ne lit pas les conversations.

## Lire, dans cet ordre — environ 10 000 mots

**Une session du relais lit moins** (les jetons, sa priorité du 24/11 ; fiche 142) : sa consigne, la
passation et ses notes d'abord, puis seulement la section dont son travail a besoin ; son premier
message le dit. Une session à lui lit tout, dans cet ordre :

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

Puis : prendre le premier chantier ouvert du plan — **son ordre vit dans son Glucose, le nœud
`relais-plan`** (sa demande du 24/11) ; le détail de chaque chantier, dans SUITE § 2 —, **tout remettre en question** — ses paroles
comprises, avec des arguments —, et **chercher sur internet avant de concevoir**, sources rapportées.

Le code cite le carnet par numéro (« fiche 22 § 5 ») : c'est `docs/carnet/22-…`. Ne jamais renuméroter une
fiche. **La prochaine porte le numéro 174.**

## Avec lui

Il ne code pas, parle français, ne voit pas la console, et veut être contredit quand on a un argument. Il
porte la vision et le ressenti ; la technique se décide et s'exécute seule. Ce qu'il lance s'écrit dans un
fichier : la commande `glucose` y envoie elle-même sa sortie (`~/sortie-glucose.txt`), qu'on lit soi-même. **C'est à nous de tester et de
juger** (sa parole du 18/10) : une fenêtre sur son écran est permise pour un essai, puis fermée (`bash
outils/essai_isole.sh <secondes> [capture.png]`) ; les bancs restent hors écran. **Ses données avant
tout** : diagnostic sur une copie ; son dossier Glucose (`~/.local/state/glucose`) ne se touche jamais.
Questions en prose, jamais en menu — **et dans son Glucose, plus en pages à part** (sa parole du 19/11 : on lit ses réponses et on y répond par sa porte — `glucose lire`, ou les outils
`mcp__glucose__*` que `glucose --mcp` donne à chaque session depuis le 22/11 ; son document de travail :
`GLUCOSE PROJET.glucose`, à la racine du dépôt, exclu de Git, jamais publié). Dire « nœud » (jamais « carte » pour un élément du tableau) et « carte
graphique » en entier. Un point en français simple toutes les 30 à 45 minutes, l'heure lue (`date`).

## Les commandes

Rust n'existe ici que dans `nix develop` : préfixer par `nix develop -c`. **Une seule grosse compilation
à la fois, et toujours bornée** : toute compilation, suite ou mesure passe par `bash outils/borne.sh
<commande>`, qui calcule sa part de la mémoire (sur ce poste : ~18 Go, sept compilations ; tout vérifier
`cargo check` prend 2 min 30 depuis zéro, la version optimisée 2 minutes).
Le 12/11, un `cargo test --workspace` nu a lancé huit compilations à la fois et figé sa machine deux
heures et demie (le swap compressé ne tue rien) ; bornée, une commande qui déborde meurt seule.

```bash
cargo fmt --all -- --check
bash outils/borne.sh nix develop -c cargo clippy --workspace --all-targets -- -D warnings
bash outils/borne.sh nix develop -c cargo test --workspace --no-fail-fast  # ~3 160 épreuves, dont les cliquets
GH_TOKEN=… bash outils/suivre_ci.sh <empreinte complète> <sortie.txt>  # la vérification GitHub (dix tâches)
python3 outils/saboter.py <sabotages.json>  # exiger qu'une épreuve tombe quand on casse sa garde
bash outils/verifier_cibles.sh [windows|mac|android]  # le clippy strict pour Windows, le Mac et Android, ici (fiche 156)
bash outils/pilote/piloter.sh demarrer <maison> <sortie>  # le vrai Glucose hors écran : clic, molette, taper, capture
node outils/telemetrie/tableau.mjs <dossier>  # la télémétrie, plantages compris : à lire en début de session
glucose lire niveau=plan  # son Glucose ouvert, par sa porte (puis dans=…, ids=…, niveau=complet) ; `glucose aide` liste tout
bash outils/borne.sh nix develop -c cargo build --release -p glucose-desktop --bin glucose-desktop  # SON Glucose (la commande `glucose` lance ce binaire), puis : python3 -I outils/relais/relancer_glucose.py "<raison>"
```

Le relais de la porte publique (`outils/porte`) vit sur le même compte : `npx wrangler deploy`, son essai
réel par `outils/porte/essai/` (MÉTHODE § 5).

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
* **Jamais de jeton dans le dépôt, ni dans son Glucose.** Il vit dans `~/Projets/github.md` (il le
  renouvelle tous les 3 jours ; aucune remarque de notre part) ; le lire au moment d'envoyer, sans
  l'écrire nulle part : `git push "https://x-access-token:$JETON@github.com/shazamifius/GlucoseGit.git"
  main:main`. Git n'a pas d'identité globale sur cette machine : celle du dépôt est réglée.
* **`docs/` est un dépôt à part, privé** (`shazamifius/GlucoseGit-docs`, sa réponse du 25/11 dans
  `relais-rep-orga`), ignoré du dépôt public : ce qui y change se commite et s'envoie **là**
  (`git -C docs …`, le même jeton) ; le code, ici. Publics : `README.md`, `GUIDE.md`, `TELEMETRIE.md`.
* **Deux branches, pas une de plus** : `main` et `tauri-v1.0.1`. Une branche de travail se retire, en local
  et sur GitHub, dès qu'elle est fusionnée.
* **Publier une version reste son geste** : le brouillon se prépare, le clic « Publish release » est le sien.

## En fin de session

[MÉTHODE § 6](docs/METHODE.md) : ÉTAT réécrit au présent, SUITE sans ce qui est fini, une fiche datée et
sa ligne à l'index, ses paroles nouvelles dans VISION, une seule liste courte d'essais à son écran. Puis rendre la main
au relais (`python3 -I outils/relais/relais.py rendre`).
