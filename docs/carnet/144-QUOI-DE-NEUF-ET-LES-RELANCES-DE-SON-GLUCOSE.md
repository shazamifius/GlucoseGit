# 144 — Quoi de neuf ; les relances de son Glucose et sa synchronisation coupée (24/11, le relais, compte b)

> Le point 2 de sa liste du MCP (`relais-rep-mcp`), et sa note du 24/11 dans la bulle 🐞 4 (`text-u33i-2328`) :
> *« j'ai encore été déconnecté de la sync le 24/11 à 12 h 10 je crois bien »*.

## 1. MCP-NEUF — `glucose_quoi_de_neuf`

**Sa demande** (`relais-rep-mcp`, point 2) : *« ce qui a changé depuis ma dernière lecture, qui l'a fait
(toi, ton autre appareil, une IA), où et quand »*. `glucose_histoire` rendait des **gestes** : une IA qui
revient veut des **nœuds**, et ses jetons sont sa priorité (fiche 142).

**Remis en question.** Un argument `depuis` sur `glucose_histoire` aurait évité un outil de plus ; mais
l'histoire répond « comment le document s'est fait », et quoi de neuf « qu'est-ce qui est différent » : deux
questions, deux formes. « Depuis ma dernière lecture » demanderait à Glucose de retenir les lectures de
chaque IA (un état caché, sous un nom déclaré, jamais vérifié) ; **depuis son dernier geste** se lit dans
l'histoire, sans rien retenir : ce que les autres ont fait depuis qu'elle a agi. Et `jusqu_a`, rendu à chaque
lecture, est le `depuis` de la suivante — la mémoire reste chez qui la demande.

**La forme** : chaque nœud touché **une fois** — son premier « avant » et son dernier « après » disent
s'il est **posé**, **changé** ou **retiré** ; venu et reparti entre deux lectures, il n'est pas dit. Par
qui (« humain », le nom d'un autre appareil, « agent » et son nom), quand (son dernier geste), son tableau,
sa membrane et sa première phrase s'il est encore là ; les plus récents d'abord ; le reste en mots, compté
(« déplace des nœuds (×3) »). Le noyau dit ce qu'un geste fait aux nœuds (`Transaction::effets`,
`store/journal/effets.rs`) ; l'exécuteur relit l'histoire (`agents/executer/neuf.rs`). C'est un lecteur :
il passe pendant un geste de l'humain, et sous « Lire seulement ».

**Limite connue** : un déplacement garde son vecteur et les **rangs** des nœuds (`Edit::Translation`), pas
leurs noms : il est dit dans « le reste », pas sur le nœud. Le dire par nœud demanderait de rejouer
l'histoire jusqu'à ce geste ; un nœud déplacé n'a pas changé de sens.

**Épreuves** : 1 au noyau, 2 à l'exécuteur (`tests/neuf.rs` : depuis son dernier geste, `jusqu_a`, depuis
0, la membrane). **Sabotages** `outils/sabotages/neuf-144.json` : **12 sur 12** tombent.

## 2. Sa synchronisation coupée : les relances de son Glucose

**Ce qui coïncide** : son Glucose de ce PC est un service (`glucose.service`, `Restart=always`), relancé
aujourd'hui à 7 h 57, 10 h 26, **12 h 18**, 12 h 25, 12 h 38 (le relais, pour lui donner les réparations du
jour), puis 13 h 30 et 13 h 43 (cette session). Sa coupure « vers 12 h 10 » tombe probablement sur 12 h 18.
À chaque relance, ce PC quitte sa salle et la rejoint au numéro que son fichier contient (PARTAGE-SURVIT,
fiche 131) — à 13 h 43, **repris au geste n° 425** (sa sortie). Chez lui, de loin, quelque chose casse
pourtant : quoi, sa réponse le dira (`relais-rep-sync-1210` : quel appareil, ce qu'il a vu, revenu seul
ou non). Le chantier des codes (bulle 4 : deux codes au plus, jamais de doublon, un lien qui ne meurt pas)
le reprendra.

**Mon erreur de 13 h 30** : j'ai fermé son Glucose (`SIGTERM`, fermé proprement en 2 s) puis relancé à la
main ; son service en a ouvert un second dix secondes après. Deux fenêtres pendant treize minutes ; la
seconde **n'a rouvert aucun document** (son journal technique : `"rouvert":"rien"`), et la porte des agents
était à la première : son document n'a rien eu. Fermé à 13 h 43, le service relancé seul.

**Fait** : `outils/relais/relancer_glucose.py "<raison>"` — le seul geste pour relancer son Glucose : par
systemd, puis il lit dans la sortie ce qu'a fait le partage, et l'écrit dans son Glucose, la plus récente en
haut (`relais-relances`, douze lignes ; l'histoire garde les autres). Le mode d'emploi du relais le dit
(les pièges payés), et `CLAUDE.md` à la ligne de la compilation : **une relance pour plusieurs chantiers**.

### La fermeture attend le retour de ses gestes (RETOUR-1)

**Une cause possible, lue dans le code** : à la fermeture, le numéro de la salle n'est retenu que si
aucun geste de ce PC n'attend le sien (`souvenir_du_canevas`) ; sinon le lancement suivant s'arrête « net »
et **ne rejoint pas** (fiche 131). Or le relais écrit sans cesse dans son Glucose (le ❤️ chaque minute, les
réponses des sessions), et la fermeture elle-même écrit ce qui restait (`consigner`) : un geste parti dans
la dernière fraction de seconde suffisait. Ce PC hors de la salle, ce que le relais écrit n'arrive plus chez
lui, de loin — *« j'ai bien le fichier, mais seul, sans mise à jour »* (sa note `text-c3kr-2251`). **Probable,
pas prouvé** : aucun journal ne garde ce qu'a fait le partage à 12 h 18 (la télémétrie n'en dit rien) ; sa
réponse et `relais-relances` le diront.

**Fait** : `fermer_le_document` attend le retour de ses gestes (`attendre_le_retour_de_mes_gestes`) — tant
que la salle est reliée (sinon rien ne reviendra), au présent, et au plus `SILENCE` (le temps au bout
duquel un pair silencieux passe pour parti, déjà là : aucune durée neuve) ; puis elle écrit ce qui est
arrivé pendant l'attente, **avant** de lire le numéro. Le lancement qui ne reprend pas le dit désormais
dans sa sortie (« pas repris, arrêté net ») : `relancer_glucose.py` l'écrit chez lui.

**Épreuves** : un lien d'épreuve relié (`salle::lien::d_epreuve`) rend, pendant la fermeture, un geste de
Bob puis le mien : numéro 3 retenu, le geste de Bob dans le fichier relancé ; et la loi de l'attente.
**Sabotages** `outils/sabotages/retour-144.json` : **7 sur 7**.

**Ce qui reste** (le chantier des codes) : un arrêt vraiment net (coupure de courant, `SIGKILL`) ne
reprend toujours pas ; et « Partager » fait **toujours une salle neuve** (`partager_ce_canevas` : un code
neuf à chaque fois) — d'où ses doublons : un document repartagé laisse ses autres appareils dans
l'ancienne salle. La forme juste, à concevoir : une salle par document, gardée ; le numéro exact écrit
dans l'histoire du document elle-même, plutôt qu'à la fermeture.

### Une seconde fenêtre à chaque relance : le relais MCP la lançait (SERVICE-1)

À 14 h 02, `relancer_glucose.py` relance le service… et **deux** Glucose s'ouvrent à la même seconde :
celui du service, et un second dans la portée tmux de ma session — **le relais MCP** (`glucose --mcp`),
qui, sa porte tue pendant la relance, rouvrait Glucose lui-même (`agents/relais.rs`, `lancer_glucose`) ;
la commande `glucose lire` fait pareil, et le veilleur l'appelle chaque minute. Les deux fenêtres
avaient rouvert le document et rejoint la salle. **Son document n'a rien eu** : le verrou d'écriture
(`persist/verrou.rs`, `flock`) n'en laisse écrire qu'une ; celle du relais MCP l'écrivait, et son
histoire se relit entière jusqu'au dernier geste (1571, 14 h 02, relu du fichier). La fenêtre du
service a été coupée net (`SIGKILL`, sans écriture de fermeture), le service arrêté.

**Fait** : sous Linux, quand un service utilisateur `glucose.service` est activé, `lancer_glucose`
**le demande au service** (`systemctl --user start`) au lieu de lancer une fenêtre à côté : `start` ne
fait rien sur un service qui tourne, et rejoint celui qui redémarre — jamais deux. Sans service, rien ne
change (les autres machines, la CI). **Vérifié pour de vrai** : la fenêtre hors service fermée
proprement, `glucose lire` a rouvert Glucose **par son service** (sa portée : `glucose.service`), une
seule fenêtre, partage repris au geste n° 435. Pas d'épreuve automatique : systemd n'existe pas dans
la CI ; l'essai réel en tient lieu.

### Seule la fenêtre qui écrit le document reprend son partage (SCRIBE-1)

Vu pendant SERVICE-1 : une fenêtre qui n'a pas le document en écriture (le verrou tenu ailleurs)
rejoignait quand même la salle et, fermée proprement, pouvait retenir un numéro que le fichier ne
contient pas. **Fait** (`11b108e`) : `document_du_canevas` (`app/collaborer/reprise.rs`) ne rend le
document que si cette fenêtre tient son verrou d'écriture (`verrou::meme_fichier`) ; une seconde
fenêtre ne reprend pas la salle et ne retient rien. Épreuve
`partage_survit_une_seconde_fenetre_ne_reprend_pas_la_salle` ; sabotage `retour-144.json` (8 sur 8).
**Sa première version était aveugle** (le sabotage passait) : sa seconde fenêtre n'ouvrait aucun
document, car un document qu'une autre fenêtre écrit ne s'ouvre pas ici (`DejaOuvertAilleurs`). La
garde ne sert donc qu'aux fenêtres qui ont le document sans l'écrire : la course de deux lancements à
la même seconde (14 h 02 : le verrou pris entre la vérification et l'ouverture), ou un fichier en
lecture seule — ce que l'épreuve fait désormais, la prémisse affirmée (document ouvert, pas
d'écriture). Payé en chemin : un cache incrémental de rustc corrompu (« undefined reference to
`anon.…llvm…` » à la liaison, puis une panique de rustc) — `rm -rf target/debug/incremental/glucose_desktop-*`.

## 3. MCP-REPONDRE, vérifié chez lui

Son Glucose relancé sur `0f9ec0b` : la première réponse posée par `glucose repondre` (`relais-rep-mcp-4`)
est à droite de sa note (x = 3 242 + 700 + 30), même hauteur, dans la membrane « relais », avec sa flèche.
