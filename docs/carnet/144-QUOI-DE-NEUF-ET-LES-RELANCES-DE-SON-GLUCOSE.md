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

## 3. MCP-REPONDRE, vérifié chez lui

Son Glucose relancé sur `0f9ec0b` : la première réponse posée par `glucose repondre` (`relais-rep-mcp-4`)
est à droite de sa note (x = 3 242 + 700 + 30), même hauteur, dans la membrane « relais », avec sa flèche.
