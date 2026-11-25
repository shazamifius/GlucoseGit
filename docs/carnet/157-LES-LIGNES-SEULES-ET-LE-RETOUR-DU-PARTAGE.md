# 157 — Les lignes seules, et le retour du partage (25/11 vers 3 h, le relais, compte b)

## 1. La revue d'OMBRE-1 et de PORTES-1 (le plan, point 11)

Rien à reprendre.

* **OMBRE-1** (fiche 155) : `copie_a_exporter` clone `assets`, ce qui ferait copier toutes les photos
  à chaque glisser ; mais le `Store` de l'application ne les porte pas (elles vivent ailleurs ; seul
  `bench_actions` y écrit) : la copie est vide. Le cycle de vie de l'icône Wayland tient : une
  annulation qui précéderait `montrer` rend des requêtes sur une surface morte, que `wayland-client`
  refuse chez nous, sans faute de protocole. Limite connue : la marge de l'export n'est pas exactement
  proportionnelle à l'échelle, et l'ombre au plafond peut perdre un ou deux pixels au bord.
* **PORTES-1** (fiche 154) : les codes de réponse de Collaborer (0 à 17) ne se chevauchent pas ;
  « Retour » remonte d'un niveau partout.

## 2. LIEN-JOURNAL : la sortie dit si le partage revient

Sa sortie du 25/11 finissait sur « canevas partagé : le relais a fermé la connexion » (1 h 24), rien
après : on ne pouvait pas savoir si le lien était revenu — ce que sa note « déconnecté de la sync »
(24/11) demande de savoir. `salle/lien.rs` dit désormais la raison du **premier** échec d'une série
(« pas relié (…) ; il réessaie ») et le retour (« relié de nouveau, après N s ») ; les échecs suivants
se taisent (la pause double jusqu'à `PAUSE_MAX`). Une ligne de journal, sans épreuve : rien ne la
garde.

## 3. Les lignes seules (la fiche 127, close)

La fiche 127 proposait de mesurer `debug = "line-tables-only"` contre l'information de débogage
complète. Mesuré ici (`/usr/bin/time -v`, `CARGO_INCREMENTAL=0`, `cargo test -p glucose-desktop
--no-run` après un `touch`, en alternance) :

| profil | durées | plus gros `rustc` |
|---|---|---|
| complet (avant) | 1 min 46, 2 min 13, 2 min 49 | 4,28 Go, 4,05 Go, 3,45 Go |
| lignes seules | 1 min 28, 2 min 10 | 3,09 Go, 2,91 Go |

Les durées dérivent d'une mesure à l'autre (la machine n'était pas au repos) ; prises deux à deux,
les lignes seules gagnent à chaque fois, ~20 % du temps et ~25 % de la pointe. Personne ne lance de
débogueur ici, et une trace de plantage n'a besoin que des fichiers et des lignes : **`[profile.dev]
debug = "line-tables-only"`** (Cargo.toml). La version optimisée (la sienne) n'y est pour rien. Les
dépendances se recompilent une fois : **la suite entière, 3 697 épreuves vertes**, en 4 min 43, cette
recompilation comprise (le plus gros `rustc` de toute la suite, dépendances en `opt-level 3` comprises :
4,0 Go).

**La borne** (`une compilation par 2,5 Go`) : son commentaire mesurait 2,4 Go le 19/11 ; le bureau a
grandi (4,3 Go en complet). Sur ce poste (18,7 Go de borne, sept compilations, les deux grosses
seules à dépasser 2,5 Go), elle tient : inchangée.

## 4. Piège

`clippy` a planté sur le cache incrémental (`dep_graph/graph.rs`) **sans aucun sabotage avant** :
le piège de MÉTHODE § 5 n'est pas propre aux sabotages. Le même remède :
`rm -rf target/debug/incremental/glucose_desktop-*`.

## 5. LIEN-JOURNAL-2 : relu le 25/11 (compte a), un trou dans le cas même de sa note

À la relecture : la raison du premier échec ne se disait **qu'au démarrage**. Après une connexion
perdue (« le relais a fermé la connexion », sa sortie du 25/11 à 1 h 24), `perdue` était déjà posé :
les essais manqués qui suivaient se taisaient tous, et une salle qui ne revenait plus ne disait pas
pourquoi — exactement le cas de « déconnecté de la sync ». La coupure vit maintenant dans une petite
structure, `Coupure` (`salle/lien.rs`) : la perte ouvre une série, le premier échec de **chaque** série
dit sa raison, le retour la referme. Une épreuve
(`la_sortie_dit_le_premier_echec_de_chaque_serie_puis_le_retour`), **4 sabotages sur 4**
(`outils/sabotages/lien-journal-157.json`) ; clippy du bureau et cliquets verts. Les trois appels de la
boucle (`tourner`) restent sans épreuve : elle ouvre un vrai réseau.

Relu sans le reprendre : une salle qui accepterait la connexion puis la fermerait aussitôt ferait
reprendre la pause à une seconde (doublée à deux) et dirait deux lignes à chaque fois. Le serveur ne le
fait pas : un secret refusé l'est à l'ouverture (403, `salle.js`), donc par l'échec, qui se tait.

**Piège payé** : après `saboter.py` sur une épreuve Rust, `rustc` a paniqué deux fois de suite sur son
cache incrémental (`dep_graph/graph.rs`) ; vider `target/debug/incremental/glucose_desktop-*` l'a
réparé.
