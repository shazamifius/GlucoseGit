# 46 — La suite, dans l'ordre

> **Rôle de ce document.** La passation du 29/09 au soir : **où en est Glucose Rust, vérifié**,
> ce qu'il a décidé, sa contrainte d'argent et ce qu'elle change, **le plan complet dans l'ordre**
> — qu'il a approuvé en entier (*« je suis d'accord avec TOUT ton plan »*) —, les pièges connus
> et les outils. Une session neuve commence ici, après les mémoires et les fiches 44 et 45.
> **Tout y est à remettre en question** — le plan compris.
>
> **Date** : 2026-09-29 · au commit de cette fiche, après `70cd50a`.

---

## 0. En une page

* **Vérifié** : sous Windows, **1 929 épreuves vertes**, clippy strict à zéro ; sur GitHub, à
  chaque envoi, **Windows, Linux, Mac à puce Apple, Mac Intel, NixOS et le noyau pour Android
  passent tous** (`70cd50a`, la première fois que le Mac Intel est jugé jusqu'au bout).
* **Ses utilisateurs** : cinq sur Glucose Tauri (Windows et Linux) — basculés **d'un coup** ;
  une vingtaine qui attendent Glucose Rust, **dont dix sous Android** ; et **une personne qui
  compte énormément pour lui, sur iPad**.
* **Sa contrainte** : **il ne peut dépenser aucun argent** (§ 2).
* **Sa règle, après la peur du 29/09** : *« il faut que l'app soit parfaitement clean et
  fonctionnelle de bout en bout »* — une perte de données passe avant tout (mémoire
  `ses-donnees-avant-tout`).
* **L'ordre** (§ 3) : **A.** la sûreté de ses données → **B.** la mise à jour, l'installeur et la
  bascule de ses cinq utilisateurs (la 2.0.1 bêta) → **C.** la boîte noire qui voyage →
  **D.** le toucher : **Android et l'iPad par le web** → **E.** ce qui manque de Tauri → **F.** la
  fluidité.

---

## 1. Où on en est — vérifié le 29/09

| | fait | fiche |
|---|---|---|
| **Voir sans posséder** | la vérification automatique sur six machines, à chaque envoi, gratuite (dépôt public : 0 $ vérifié par l'API de facturation) ; les épreuves graphiques exigent une carte logicielle, et ne peuvent plus se sauter en silence | 45 § 1 |
| **La boîte noire (locale)** | un fichier par session dans `…\Glucose\boite-noire\`, écrit au fil de l'eau et synchronisé ; épisodes, records, batterie, paniques sans message ; au lancement, comment la session d'avant a fini. **N'envoie rien encore** | 45 § 2 |
| **Son presse-papiers** | les épreuves ont le leur ; l'application prend celui du système au lancement | 45 § 3 |
| **NixOS** | `flake.nix` + `nix/glucose.nix`, construit à chaque envoi, verrou versé | 45 § 4 |
| **Linux, la mémoire** | l'offre au système par `madvise(MADV_FREE)` et un témoin par page | 45 § 4 |
| **La mise à jour, son cœur** | `mise_a_jour` : le `latest.json` et la clé de Tauri, semver, minisign — **rien de branché** | 45 § 5 |
| **COLLER-2** | trois images collées le 28/09 s'étaient scellées **vides** dans `proteo.glucose` ; réparées à l'ouverture — **il a confirmé : « on a retrouvé l'image »** | 45 § 6 |
| **L'aimant (SNAP-4)** | les voisines, parmi ce que l'écran montre — **il a dit : « 100 % fonctionne »** | 43 § 9 |

---

## 2. Aucun argent : ce que cela change

| besoin | avec argent | **sans argent — la voie à prendre** |
|---|---|---|
| **iPad** | TestFlight (99 $/an) | **Glucose dans Safari** : WebAssembly + WebGPU, activé par défaut dans Safari 26 sur iPadOS 26 ; repli WebGL2 (wgpu le sait) pour un iPad plus ancien. Hébergement gratuit (§ 3 D) |
| **Android** | compte Google vérifié (25 $) | **l'APK installé directement** ; pour la vérification des développeurs que Google exige partout en 2027, **le compte gratuit « amateur », limité à vingt appareils** — son échelle |
| **Windows** | certificat de signature | aucun : Windows avertit au téléchargement ; ses testeurs cliquent « Informations complémentaires » → « Exécuter quand même » — **à leur dire d'avance** |
| **Mac** | notarisation (99 $/an) | « Ouvrir quand même » dans Réglages → Confidentialité et sécurité — plus tard (phase Mac) |
| **La boîte noire qui voyage** | un nom de domaine (≈ 10 €/an) | **un nom gratuit** (DuckDNS, ou deSEC — associatif, européen) devant **sa box NixOS**, et un certificat Let's Encrypt (gratuit) par Caddy ; ou un hébergement gratuit sans domaine (Cloudflare Workers) — **à comparer**, la vie privée d'abord |
| **La signature des mises à jour** | — | **gratuite** : sa clé minisign, déposée par lui dans les secrets du dépôt GitHub — ou il signe lui-même chaque version |
| **La vérification automatique** | — | **gratuite** : dépôt public |

---

## 3. Le plan, dans l'ordre

### A. La sûreté de ses données — d'abord

1. **Passer au crible de COLLER-2 chaque chemin qui écrit ses données** : déposer, importer,
   coller, enregistrer, « enregistrer sous », les brouillons, l'ajout d'un document dans un
   onglet, l'import d'un document Tauri. Pour chacun : une épreuve de bout en bout, et une
   réparation si un état abîmé a pu exister chez lui. Diagnostiquer **sur une copie**
   (`examples/verifier_images`, `examples/lire_histoire`).
2. **Les images collées vivent dans `%TEMP%\glucose_pasted`** jusqu'à leur scellement — et la
   réparation de COLLER-2 relit ce dossier. Le système peut le vider. Les ranger là où il ne fait
   jamais le ménage.
3. **Découvert le 29/09 — à vérifier avant tout installeur** : Glucose Tauri s'est installé dans
   **`%LOCALAPPDATA%\Glucose`** (`glucose.exe`, `uninstall.exe`), **le dossier même où Glucose
   Rust range ses données** — la boîte noire, les aperçus, les **brouillons** (des documents non
   enregistrés !), et `dernier-document.txt`. Lire ce que fait le désinstalleur NSIS de Tauri
   2 ; au besoin, **déplacer les données de Rust ailleurs**, avec une migration éprouvée qui
   ne perd rien.
4. Sa clé de signature de Tauri, sauvegardée **hors de ce PC** (fiche 36 § 3.4) — **lui seul**.

### B. La mise à jour, l'installeur, la bascule — la 2.0.1 bêta

Ce qui existe : le cœur (`mise_a_jour`). Ce qui manque, dans l'ordre :

1. **Une seule bibliothèque réseau** pour tout — la mise à jour, l'envoi de la boîte noire, le
   téléchargement d'une image depuis un lien hors de Windows (aujourd'hui : WinHTTP seulement).
   Le choix est à faire, argumenté (`ureq` + `rustls` ?) — dépendances minimales mais assumées.
2. **Chercher la mise à jour avant tout ce qui peut planter** (la carte graphique, le document) ;
   télécharger, **vérifier la signature**, lancer l'installeur, quitter.
3. **L'installeur** Windows (Tauri : NSIS, par utilisateur), l'AppImage ; NixOS par le flake.
4. **La construction, la signature et la publication automatiques** — en pré-version ou en
   brouillon, **jamais « latest » avec un `latest.json`** tant que la bascule n'est pas décidée.
5. **L'épreuve « la version N devient N+1 toute seule »** sur les machines de GitHub.
6. **La répétition générale de la bascule** : une application Tauri construite avec une **clé
   d'essai** passe à Glucose Rust. Puis, et seulement alors, la vraie : ses cinq utilisateurs,
   d'un coup, par le popup habituel — **prévenus**, les installeurs de Tauri toujours
   téléchargeables (un chemin de retour).

### C. La boîte noire qui voyage

L'écran d'accord (éteint par défaut, « voir ce qui part ») ; l'envoi chiffré, en petits lots ; le
serveur (§ 2 — sa box, un nom gratuit, HTTPS) ; les plantages hors de Rust (un processus témoin :
`crash-handler`, `minidumper`) ; la page de confidentialité. Règles : fiche 44 § 2. **La
télémétrie de Tauri part en clair vers l'adresse IP brute de sa box** : Glucose Rust ne la
reprend pas.

### D. Le toucher : Android, et l'iPad par le web

**Le toucher est commun** : les doigts, le pincement, pas de survol — puis le crayon sur iPad.

* **Android** (dix testeurs) : le portage (`android-activity`, le cycle de vie, le clavier, le
  stockage), l'APK et sa mise à jour, sa clé à ne jamais perdre, les sondes de chaleur et de
  batterie, l'auto-vérification des pilotes, de vrais téléphones anciens par Firebase Test Lab
  (gratuit).
* **L'iPad, par le web** : Glucose en WebAssembly, rendu par WebGPU (WebGL2 en repli), dans
  Safari. **À rechercher avant d'écrire** : ce qu'un navigateur retire — les fils (il faut des
  en-têtes d'isolation, que GitHub Pages ne sait pas poser ; Cloudflare Pages, gratuit, le
  sait), le disque (OPFS ; Safari n'a pas le sélecteur de fichiers moderne), la mémoire que
  Safari laisse à une page sur iPad — et **quels iPad reçoivent iPadOS 26**. Il veut que ce
  soit **parfaitement propre** : une version web qui perd un document serait pire que rien.
  Demander le modèle de son iPad.

### E. Ce qui manque de Tauri, et ce qui attend sa parole

La parité (fiche 36 § 2, registre fiche 39) ; **les membranes, les rideaux, Trans-domaines** —
rien sans sa parole ; la Time Machine « optimisée » (fiche 43 § 11).

### F. La fluidité

Les panneaux qui coûtent quand la Time Machine est ouverte ; **2,5 images par seconde dessinées
au repos** (zéro est la seule bonne réponse) ; les gels sur la carte Intel Arc (fiche 43 § 8) ;
puis l'étalonnage et l'auto-vérification des pilotes (fiche 44 § 1).

---

## 4. Les pièges connus

**Technique**

* **Git Bash mange les barres obliques inverses** dans un heredoc (`\\` → `\`, `\n` → un vrai
  saut de ligne). Tout script qui en contient s'écrit avec l'outil d'écriture, puis se lance.
* **Les cliquets** : une fonction ≤ 80 lignes, un fichier ≤ 600 (`app.rs` y est **déjà** ;
  `chronique.rs` à 595), le couplage ≤ 38. On extrait ; on ne relève jamais un plafond.
* **Les épreuves ne touchent jamais sa machine** : ni fenêtre sur son écran, ni son
  presse-papiers, ni `%LOCALAPPDATA%\Glucose` — qui contient désormais **sa** boîte noire et
  **ses** brouillons : on vérifie qu'aucune épreuve n'y a écrit, pas qu'il est vide.
* **La vérification GitHub** : un envoi plus récent **annule** la précédente — le Mac Intel,
  lent et rare, n'a longtemps jamais fini. **Grouper les envois.** La suivre par
  `outils/suivre_ci.sh`, avec l'empreinte **complète**.
* **Une épreuve de temps** (la relève de la mémoire, 25 µs) peut tomber sur une machine de
  GitHub : elle dit que la manière doit changer — jamais relever la borne.
* **Une épreuve graphique se saute sans carte** (`banc_gpu::sans_carte`) — sauf sur GitHub, où
  `GLUCOSE_EXIGER_UNE_CARTE` la fait tomber.

**Méthode**

* **Saboter chaque garde** (`outils/saboter.py`) : une épreuve qui ne tombe pas quand on casse ce
  qu'elle garde est aveugle. Le 29/09, quatre épreuves aveugles trouvées ainsi — et une fois,
  c'était **mon sabotage** qui était mal construit.
* **Jamais de victoire annoncée trop tôt** : « réglé » seulement vérifié en conditions réelles —
  sur une copie de ses documents, sur les machines de GitHub, ou à son écran.

---

## 5. Les outils

| outil | ce qu'il fait |
|---|---|
| `outils/saboter.py` | remplace un passage, lance les épreuves, exige qu'elles tombent, restaure — `[{nom, fichier, avant, apres, cmd}]` |
| `outils/suivre_ci.sh` | suit la vérification GitHub d'un commit jusqu'à sa fin ; lit le jeton dans l'adresse du dépôt, ne le montre jamais |
| `examples/verifier_images` | où vivent les octets de chaque image d'un document, s'ils se décodent — lecture seule |
| `examples/lire_histoire` | les derniers gestes d'un document, ses jalons — lecture seule |
| `examples/bench_*` | les bancs, **hors écran** |

---

## 6. Ce qui attend sa parole

1. **Le modèle de l'iPad** de la personne qui compte tant pour lui (ou sa version d'iPadOS).
2. **Les appareils de ses dix testeurs Android** — le plus ancien surtout.
3. **Sa clé de signature** : sauvegardée hors de ce PC ? Et la déposer dans les secrets GitHub,
   ou signer lui-même chaque version ?
4. **Le serveur** de la boîte noire : sa box derrière un nom gratuit, ou un hébergement gratuit.

---

## 7. Les sources

Celles des fiches 44 § 6 et 45 § 8 ; et :

* WebKit, [*WebKit Features in Safari 26.0*](https://webkit.org/blog/17333/webkit-features-in-safari-26-0/)
  — WebGPU activé par défaut sur iOS et iPadOS 26.
* Apple, [TestFlight](https://developer.apple.com/testflight/) — la voie payante, écartée.
* Android, [*developer verification*](https://developer.android.com/developer-verification) — le
  compte amateur gratuit, vingt appareils.

---

**Retour** : [`00-INDEX.md`](00-INDEX.md)
