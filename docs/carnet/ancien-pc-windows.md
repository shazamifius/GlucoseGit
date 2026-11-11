# Les pièges du PC Windows — archive

> **Archive.** Écrit sur son PC Windows (septembre à début novembre 2026), cassé depuis ; on travaille
> depuis le 6/11 sur son VivoBook sous NixOS ([MÉTHODE § 5](../METHODE.md)). Ce qui vaut sur toute
> machine a été repris dans MÉTHODE le 11/11 (fiche 104) ; le reste — Git Bash, PowerShell, WSL, les
> chemins `C:\`, l'APK et le téléphone virtuel construits depuis Windows — attend ici qu'une machine
> Windows revienne.

* **Git Bash mange les barres obliques inverses** dans un heredoc : écrire un script ou un message
  de commit avec l'outil d'écriture, dans le scratchpad, puis `git commit -F fichier`.
* **Git Bash refuse aussi certains heredocs** (apostrophes, guillemets français) avec « unexpected
  EOF » : tout script Python long s'écrit dans un fichier du scratchpad, puis `python -I fichier`.
  Et Python sous Windows écrit en CRLF : ouvrir avec `newline='\n'` (le dépôt est en LF).
* **Même entre apostrophes (`<<'EOF'`), Git Bash change `\\` en `\`** : trois scripts de la fiche 73
  en ont perdu leurs `\\n` et leurs `\\sum`. La règle qui suit vaut sans exception.
* **Git Bash mange aussi les barres obliques inverses d'un `python - <<'EOF'`** (fiche 56 : des
  `\n` devenus de vrais sauts de ligne dans une chaîne Rust, et un script qui ne s'est jamais
  écrit). Tout script qui en porte s'écrit avec l'outil d'écriture, puis `python -I fichier`.
* **Un fichier Rust qu'aucun `mod` ne nomme n'est pas compilé — mais les cliquets le lisent** :
  on peut préparer un module pendant qu'un sabotage tourne ; il doit déjà respecter les
  cliquets (fiche 56 : deux `fill_rect`).
* **Ne jamais éditer un fichier pendant qu'un sabotage le touche** : le saboteur restaure ce
  qu'il a lu au départ, et effacerait la modification (fiche 54 : elle a survécu, de peu).
* **Git Bash exécute ce qu'on met entre accents graves**, même dans une chaîne destinée à
  Python : `` `docs/INVENTAIRE.md` `` dans un `python -c "…"` a été lancé comme un script
  (fiche 63, sans dégât). Tout texte qui en porte s'écrit par l'outil d'écriture ou d'édition.
* **Les fichiers de sabotage se rangent dans `outils/sabotages/`** (`chrome-1.json`, `p5.json`, `mood-board.json`,
  `storyboard-1.json`, `storyboard-2.json`, `memoire.json`, `memoire-3.json`, `p2-4.json`, et
  depuis la fiche 72 `telephone-72.json`, `grand-1.json`, `zoom-72.json`, `p4.json`, la fiche 73 `p8.json` et `p8-bis.json`, la fiche 74 `p4-utilisateur.json`, la fiche 75 `agents.json` (36 gardes), la fiche 76 `fleche-6.json` (et, pour Protéo, `PROTEO/outils/sabotages-tri-et-toile.json`), la fiche 82 `fin-session-1.json` et `fenetres-1.json`, la fiche 83 `memoire-4.json`, et
  `libere-1.json`, dont la commande passe par `wsl … glucose-lancer` : ce qui ne se joue que
  sous Linux se sabote sous Linux) : le scratchpad disparaît avec la session, et la suivante doit pouvoir les
  rejouer. Un script de remplacement s'écrit **après** `cargo fmt` (sinon il ne trouve plus son
  texte).
* **L'APK signé, depuis ce PC** : `outils/android/construire.sh` avec, **en chemins Windows**,
  `JAVA_HOME=C:/Program Files/Java/jdk-21.0.12`, `ANDROID_HOME=C:/Users/Administrator/Android/Sdk`,
  `GRADLE=C:/Users/Administrator/Android/gradle/gradle-9.8.1/bin/gradle.bat`, et
  `GLUCOSE_KEYSTORE` / `GLUCOSE_KEYSTORE_PASSWORD` lus dans `Documents\Glucose-cle-android` ;
  vérifier l'empreinte (`apksigner verify --print-certs` : `3f2af005…`) avant de le poser sur
  son Bureau.
* **Android** : les objets de construction dans `target/android` (`CARGO_TARGET_DIR`) pour ne pas
  recompiler le bureau ; `cargo ndk -t arm64-v8a -P 21 clippy -p glucose-android -p
  glucose-desktop -- -D warnings` voit ce que Windows ne compile pas (les doigts, `ureq`).
* **`SetWindowSubclass` empêche le programme de démarrer** sans manifeste des contrôles communs v6
  (`STATUS_ENTRYPOINT_NOT_FOUND`) : le sous-classement passe par `SetWindowLongPtrW` (fiche 53 § 2).
* **PowerShell ne distingue pas les majuscules** dans les noms de variables (`$tauri` est
  `$Tauri`) : un script d'épreuve s'y est trompé.
* **Les épreuves tournent en parallèle** : deux qui écrivent le même fichier temporaire tombent
  au hasard (nommer par l'empreinte du contenu).
* **Une épreuve graphique se saute sans carte** ; sur GitHub, `GLUCOSE_EXIGER_UNE_CARTE` la fait
  tomber au lieu de mentir en vert.
* **Une épreuve de temps** qui tombe sur une machine de GitHub dit que la manière doit changer,
  jamais que la borne doit monter.
* **Les épreuves de la mémoire offerte** (`plateforme::offre`, le `Ctrl+B` qui attend ses
  originaux) tombent sous une forte pression mémoire : le système jette alors les pages offertes.
  Les rejouer seules avant de conclure (fiche 59).
* **Linux, sur ce PC : la distribution WSL `Glucose-Linux`** (Ubuntu 24.04, à part de
  NixOS-Proteo, qui est un autre projet). Un utilisateur ordinaire `glucose`, Rust stable, et un
  serveur X **invisible** (Xvfb `:99 -noreset`) — rien ne s'affiche sur son écran ; WSLg est
  écarté (`DISPLAY`, `WAYLAND_DISPLAY`). Deux commandes, en `root` :
  `glucose-lancer '<commande>'` recopie le dépôt de Windows (`rsync`, sans `target`) puis lance
  la commande dans la copie ; `glucose-faire <sortie.txt> <commande>` fait de même et écrit tout
  dans le fichier, fini par `CODE <n>`. Les épreuves de la sélection X11 veulent
  `GLUCOSE_ECRAN_X_D_EPREUVE=:99` (et `GLUCOSE_EXIGER_UN_SERVEUR_X=1` pour tomber sans lui).
  Exemple : `wsl -d Glucose-Linux -u root -- glucose-faire /mnt/c/…/sortie.txt cargo clippy
  --workspace --all-targets -- -D warnings`. `cargo check --target …-linux-gnu` depuis Windows ne
  passe toujours pas (`ring`).
* **La distribution grossit** : un dossier `target` y pèse près de 30 Go. Pour rendre la place à
  Windows : `fstrim -av` dedans, `wsl --shutdown`, puis `diskpart` (`select vdisk file=…ext4.vhdx`,
  `attach vdisk readonly`, `compact vdisk`, `detach vdisk`) — le disque en mode normal, pas
  « clairsemé », que `diskpart` refuse (fiche 61 : 58 Go ramenés à 33).
* **Le Mac ne se compile pas ici** : seule la CI le fait.
* **Compiler pendant qu'il essaie** (fiche 76) : un exécutable ouvert ne se remplace pas sous
  Windows, mais **il se renomme** — `target/release/glucose-desktop.exe` **et** son lien dur
  `deps/glucose_desktop.exe` (`ls -i`, `find -inum`), puis `cargo build --release` ; son Glucose
  continue, le suivant sera le neuf. Pour Protéo, `proteo.exe` est un lien dur vers
  `target/release/proteo.exe` : compiler ailleurs (`CARGO_TARGET_DIR` hors du dépôt — un dossier
  dans le dépôt apparaît à git). Jamais tuer ce qu'il a ouvert.
* **Regarder ce qu'un agent pose, sans fenêtre** (fiche 75) : `cargo build -p glucose-desktop --bin
  glucose-desktop --example porte_sans_fenetre`, puis, **par PowerShell** (Git Bash déforme
  `\\.\pipe\…`), `porte_sans_fenetre.exe \\.\pipe\<nom> <arrêt> <dossier>` ; l'agent se branche
  par `glucose-desktop.exe --mcp` avec `GLUCOSE_PORTE=\\.\pipe\<nom>`. Créer le fichier `<arrêt>`
  rend le tableau, chaque bulle et une flèche survolée en PNG. Protéo : `PROTEO_GLUCOSE=<exe>`, et
  ses épreuves à la main `rejouer_une_vraie_recherche_sur_la_toile` (`PROTEO_RAPPORT=<preuve>`) et
  `recherche_neuve_posee_en_direct` (`PROTEO_QUESTION=…`).
* **Un témoin de format s'écrit par la build d'avant** (fiche 65, `riche-v2.glucose`) : un
  arbre de travail détaché au dernier commit qui produit ce schéma (`git worktree add --detach
  <scratchpad>/vN HEAD`), une épreuve jetable qui y écrit la fixture, sa propre cible
  (`CARGO_TARGET_DIR`), puis `git worktree remove --force`. Un témoin écrit par la build du
  jour ne prouverait que sa propre idée de l'ancien schéma.
* **Lire le téléphone par le câble** (fiche 66) : `bash outils/android/lire_le_telephone.sh
  <dossier>` — la raison du dernier redémarrage, les morts de Glucose qu'Android a notées
  (`exit-info`), la boîte des incidents, les journaux, la mémoire, le pilote, et un rapport de
  bogue complet. Rien n'est écrit sur le téléphone, et Glucose n'y est pas ouvert.
* **Sans câble** (il a perdu le sien, fiche 71) : le **débogage sans fil** d'Android 11 et après
  (`adb pair <adresse:port>` et son code à six chiffres, puis `adb connect <adresse:port>`, un
  autre port) — le même script marche. Il demande que téléphone et PC soient sur **un même
  Wi-Fi** : quand le PC passe par le partage de connexion du téléphone, c'est probablement
  refusé. Le repli : l'APK dans un zip, envoyé par Drive (Discord refuse plus de 10 Mo sans
  abonnement), extrait par l'application Fichiers ; la télémétrie dit le reste.
* **Le téléphone virtuel de ce PC** (fiche 83) : l'appareil `glucose-telephone` (Android 14
  x86_64, 4 Go, 1080 × 2400), lancé sans fenêtre — `~/Android/Sdk/emulator/emulator.exe -avd
  glucose-telephone -no-window -no-audio -no-boot-anim -no-snapshot -gpu host` en fond, prêt
  quand `adb shell getprop sys.boot_completed` rend 1 (son processus s'appelle
  `qemu-system-x86_64-headless`). L'APK : `GLUCOSE_PROCESSEURS=x86_64 bash
  outils/android/construire.sh`, puis `adb install -r`. Sous Git Bash, **`MSYS_NO_PATHCONV=1`
  devant `adb`**. Les données de Glucose : `adb shell run-as com.glucose.app …` dans
  `files/glucose/` ; un document se dépose par `adb push` dans `/data/local/tmp`, puis `run-as …
  cp` (un brouillon : `files/glucose/brouillons/<nombre>-1.glucose`). Le grand tableau :
  `cargo run --release -p glucose-desktop --example fabriquer_un_grand_tableau -- <sortie> 6000`.
  Répondre **Non** à la question du journal technique : le téléphone virtuel n'envoie rien.
* **Le pilote Vulkan du téléphone** : `adb shell cmd gpu vkjson` (dans `lire_le_telephone.sh`,
  `vulkan.json`) dit ses extensions — `VK_EXT_memory_budget` y est-il ? vulkan.gpuinfo.org ne
  répond pas hors de son navigateur (fiche 67).
* **Le jeton de `wrangler` expire** : une erreur 7403 (« account not authorized ») sur une
  requête à la base se règle par `npx wrangler whoami`, qui le rafraîchit.
* **Le serveur avant le client** : une ligne neuve de la boîte noire se déploie d'abord
  (`npx wrangler deploy` dans `outils/telemetrie`), se prouve en ligne une minute plus tard (une
  session jetable, puis son effacement, puis la base relue — le script de la fiche 72), et
  seulement ensuite part dans une version : un client neuf devant un serveur ancien voit chacune de
  ses sessions refusée.
* **Les épreuves du serveur** : `node --test outils/telemetrie/test/valider.test.js` — nommer le
  fichier, le dossier seul échoue sur ce Node. **Cloudflare refuse un client sans en-tête
  d'identité** (erreur 1010) : un essai en ligne donne un `user-agent`, s'efface ensuite
  (`DELETE /v1/installations/<id>`), et la base se relit pour le prouver (fiche 60).
* **`cargo test --workspace` compile beaucoup** : `target/` a atteint 94 Go avant le rangement du
  06/10. `cargo clean` de temps en temps ne coûte qu'une recompilation.
