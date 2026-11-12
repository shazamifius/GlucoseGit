# 110 — La carte signée, le journal des rectifications, le zoom qui compte sa fin

> Session du 12/11/2026, de 13 h 33 à la fin de l'après-midi. La passation (fiche 109) laissait :
> la carte de Wikipédia à reposer par la version neuve, le journal des rectifications conçu mais pas
> écrit, le zoom à finir. **Une faute grave au milieu** : une suite d'épreuves sans borne a figé sa
> machine deux heures et demie (§ 5). Envoyé : `7b20de6`, `5044d8b` (RECTIF-1), le zoom, les
> documents.

## 1. La carte de Wikipédia reposée, signée

`GLUCOSE_LIENS=1 GLUCOSE_SENS_EN_COULEURS=1 … essai_squelette.sh ~/.cache/glucose-bancs/squelette-110`
(les réponses de Wikipédia reprises de `squelette-liens/`) : **1 870 gestes, tous signés « agent le
terminal »**, 1 671 nœuds que l'IA a le droit de juger. Sa carte de `~/Documents` en compte 1 879 :
les mêmes, plus neuf petits déplacements (dont un de 596 px le 11/11 au soir). La première pose,
sans `GLUCOSE_SENS_EN_COULEURS`, sortait toute grise : sa carte porte aussi les teintes des régions
de la fiche 99, qu'un script pose.

* **L'ouverture** (`bench_ouverture`, release, trois fois) : 23-24 ms pour la carte signée, 18-19 ms
  pour l'ancienne, à taille égale (2,1 Mo) — la relecture des mains coûte environ 5 ms, une fois.
  Pas un coût qu'on sent : gardé tel quel. Ce qu'elle relisait pour rien est retiré (§ 2).
* **Regardé** (`outils/wikipedia/vues.sh`, neuf : il prend le document et le copie) : l'ensemble et
  le fond d'« Eau » comme avant ; la souris près du texte « Arts » : une poignée de flèches
  s'éclaire, le faisceau reste discret — PARENTE-4 sur la vraie carte, pour la première fois.
* `lire_histoire` connaît les auteurs (MAIN-1), les écrits et leurs sujets.

## 2. RECTIF-1 — le journal des rectifications (`desktop/parente/rectifications.rs`)

La première marche de la page « Le cercle de l'IA » (VISION § 8 bis), conçue à la fiche 109 § 3,
**remise en question avant d'être écrite** — trois changements :

* **Aucun seuil.** La conception notait une flèche que l'IA jugeait « d'habitude » (sûreté sous
  0,5). Couper là jette ce qu'une correction apprise (D2) saura peser : l'humain qui retire une
  flèche jugée sûre la contredit autant que celui qui suit une flèche d'habitude. Chaque geste sur
  ce qu'elle jugeait s'écrit, avec sa sûreté ; la constante disparaît.
* **Suivre ne retire pas la flèche du jugement.** Ce n'est pas un geste du document — il ne se
  défait pas, ce que la règle « rien à moitié » refuse —, et un clic est un signal faible (on suit
  aussi pour voir ; Joachims et al., 2005 : un clic dit une préférence relative, biaisée, pas une
  étiquette). L'écrit le garde ; ce qu'il vaut se décidera sur ce que le journal récolte. Une flèche
  **retouchée** sort du jugement, elle : MAIN-1 le fait déjà, par un geste qui se défait (D1).
* **Pas de « sorte » à l'écrit** : son auteur la dit déjà. NOTE gagne seulement ses **sujets** (les
  nœuds dont il parle), en queue — NOTE n'a jamais été publié ; un écrit d'avant se lit sans eux.

Ce qui s'écrit, signé **l'IA de Glucose** (`Auteur::de_glucose` : un identifiant hors de l'espace des
noms d'agents — un agent qui se nommerait comme elle se montre « agent l'IA de Glucose ») :

* l'humain donne sa couleur à un texte qu'elle rangeait dans une famille → au rang de ce geste :
  « L'IA de Glucose rangeait ce texte dans une famille, sûre à 90 % : il ressemble le plus à « … »,
  par ses mots « pluie », « nappes » et « eau ». L'humain lui a donné sa couleur, #e06c75. » ;
* il retouche ou retire une flèche d'agent qu'elle jugeait → « … sûre à 20 % que ses deux bouts
  parlent de la même chose. L'humain l'a retouchée : elle est désormais la sienne… » ;
* il la suit → attaché au dernier geste du document (ce qui attend d'être écrit l'est d'abord).

Ne s'écrit pas : un texte sans famille (elle ne proposait rien), un texte déjà coloré, le geste d'un
agent, la flèche de l'humain, une flèche qu'elle ne juge pas. `suivre_les_mains` rend ce que le geste
de l'humain reprend aux agents (`Ecrit::repris`) ; l'avis lu est celui qu'elle montrait avant le
geste. La Time Machine, `glucose_histoire` (avec les sujets) et `lire_histoire` nomment l'auteur par
une seule loi, `Auteur::en_mots`. À l'ouverture, un geste de l'humain ne se relit pour la provenance
que s'il reste quelque chose à un agent : un document où seule l'IA de Glucose a écrit ne relit rien
de plus.

**Essayé dans le vrai Glucose** (le pilote, une copie de la carte signée, le fond d'« Eau ») : un clic
sur le texte « L'eau est le principal constituant du corps humain… », `Ctrl+Maj+P`, « rouge »,
Entrée. Son histoire porte alors, sous ce geste : « L'IA de Glucose rangeait ce texte dans une
famille, sûre à 87 % : il ressemble le plus à « eau minérale, eau de Seltz, eau de source, eau de mer,
eau d… », par ses mots « eau », « litres » et « environ ». L'humain lui a donné sa couleur,
#f87171. » La Time Machine, « en direct », ne montre un écrit que sur un geste du passé : son
interface est le chantier d'après. **Trouvé en chemin** : mon premier clic avait choisi une flèche, et
la palette proposait « Couleur : rouge », qui ne fait rien sur une flèche (sa couleur vient des halos
qu'elle relie) — un bouton qui ment, noté dans ÉTAT. Le pilote sait maintenant `ctrlmaj`.

**Le prix, à dire** : un document ordinaire où il colore un texte porte désormais une déclaration
d'auteur et un écrit — les versions publiées (son téléphone en beta 4, le PC de l'école en beta 9)
le refuseront avec un message net (PERSIST-5). La prochaine publication devient plus pressante.

## 3. B3 — l'IA qui dit pourquoi (`core/sens/pourquoi.rs`)

Le sens d'un texte est la somme des vecteurs de ses morceaux, ramenée à la longueur 1 : le cosinus de
deux textes se découpe **exactement** en la part de chaque mot (`Sens::pourquoi`), les parts
s'additionnent à la ressemblance (épreuve, à 10⁻⁴). Les mots se séparent comme le modèle les sépare —
aux blancs et à la ponctuation, « l'eau » donne « l », « ' », « eau » —, avec leur écriture d'origine ;
« Eau » et « eau » n'en font qu'un. **Regardé sur sa carte** : « eau » pèse 0,46 entre deux textes sur
les noms de l'eau, « énergie » 0,24, « mathématiques » 0,14 ; les mots vides presque rien (le modèle
les a appris courts) — une virgule partagée pèse parfois 0,04 à 0,07 : l'écrit ne montre que les mots
d'au moins deux lettres. La parenté garde le **plus proche parent** de chaque texte (celui d'où sa
sûreté se tire).

Pas encore : le pourquoi d'une flèche (un bout peut être une région), les flèches que l'humain tire
lui-même et que l'IA ne juge pas — son « comprendre leur importance ».

## 4. QUEUE-1 — la cascade compte la fin de l'image (`present/scene_gpu/cascade.rs`)

La cause de la fiche 109 § 2, relue image par image : au premier cran, la cascade remplit son budget
(ce qui reste de la période de l'écran, 12,8 ms pour 275 textures), puis viennent les poses, l'envoi
à la carte graphique et la présentation — 2,6 ms de plus : 20,4 ms, au-delà des 16,7 de l'écran.
**La piste A** (prévoir la queue) plutôt que la B (ne donner que le plancher au second tour) : la B
change la règle, la A corrige une fausse hypothèse — que l'image finit avec la cascade.

* La fin de l'image **s'apprend** : une droite, un prix fixe plus un prix par texture faite (le
  nombre de textures explique l'envoi bien mieux que leurs pixels : 275 textures de 354 kpx, 2,6 ms ;
  41 de 320 kpx, 1,3 ms). C'est la droite de CASCADE-4 : `Debit` devient `Droite`, de variable `x`
  — un champ `pixels` qui compterait des textures mentirait. CASCADE-4 trouve là son usage.
* Une texture ne s'entame que si **l'image entière** tient : `écoulé + prévu + fin(faites + 1) <
  budget`.
* **La première version comptait l'attente de l'écran** (`acquerir`, jusqu'à 20 ms quand la souris
  bouge) : elle tombe sur les images sans texture, et la droite prévoyait 4 ms de fin pour une
  texture, 0 pour 280 — la pente à l'envers. Le compteur `fin_prevue_us` l'a montré à la première
  mesure. Attendre n'est pas travailler : l'attente est retirée (une épreuve l'exige).
* Le commentaire du budget disait « le plancher » quand le code donne la période : corrigé.

**Mesuré** (sa carte, zoom de loin, sway sans écran, en paires contre son Glucose installé
`e44f3c7`, `~/.cache/glucose-bancs/lag-107/m110b-*`, lecture `zoom.py`) :

| zoom de loin | installé | neuf |
|---|---|---|
| images qui ratent un battement (> 16,7 ms) | 8 et 10 | **3 et 3** |
| médiane | 16,1 / 16,9 ms | 13,5 / 14,5 ms |
| p90 | 24,4 / 22,9 ms | **17,0 / 17,0 ms** |
| pire | 25,4 / 23,2 ms | 22,0 / 18,5 ms |

**Ce qui reste**, regardé image par image : la fin d'une image dépasse parfois sa prévision (5,2 ms
contre 3,5 — l'envoi varie), et au tout premier cran une texture coûte plus que sa droite ne le dit
(14 ms de cascade). Et **l'objectif de SUITE n'est pas atteint** : « sous 10 ms au 99ᵉ centile, zoom
compris » ne peut pas l'être tant que la cascade a droit à la période de l'écran — c'est la règle de
`tranche_de_fond` (remplir la période ne se voit pas à 60 Hz). Le remettre en question : le critère
qui se voit est « aucune image ne rate un battement » ; les 10 ms de la charte pèsent sur la latence
d'un geste qui arrive pendant une image de cascade. À trancher sur une mesure de latence, pas sur le
papier.

**Prouvé** : une épreuve neuve (`test_queue_1_…` : la seconde texture attend quand l'image entière ne
tiendrait pas, la preuve à l'envers sans la fin apprise, l'attente de l'écran retirée), la suite
entière ; sabotages
`outils/sabotages/queue-110.json` (§ 6). L'appel qui apprend la fin, dans la vraie présentation, n'a
pas d'épreuve (il y faut une fenêtre) : c'est la mesure qui le prouve vivant (`fin_prevue_us`).

## 5. La machine figée deux heures et demie — ma faute

À 14 h 36, j'ai relancé `cargo test --workspace` après deux extractions. Une seule commande, mais
`cargo` compile un morceau par cœur : huit `rustc` à la fois, jusqu'à un gigaoctet chacun. Ses 7,6 Go
ont débordé dans le swap compressé (zram, `vm.swappiness = 180`, `materiel-vivobook.nix`) — où Linux
ne tue rien tant que le swap répond. La machine s'est étouffée jusqu'à 17 h 05 min 56 s, quand le
noyau a tué un `rustc` de 1,08 Go (`journalctl -k`). La même suite était passée une heure plus tôt :
par chance. La règle « une seule grosse compilation à la fois » était respectée en apparence.

**Ce qui le rend impossible** : `outils/borne.sh` — toute commande lourde dans une part de mémoire
bornée par systemd (60 % de la mémoire, sans swap, `nice`), une compilation par 1,5 Go de borne
(trois ici). Prouvé : 400 Mo demandés dans une borne de 200 Mo, tués en 0,6 s, rien d'autre touché.
Bornée, la suite entière a pris 13 minutes, 2 Go toujours libres (un moniteur le relevait toutes
les trois minutes). La règle est dans `CLAUDE.md`, MÉTHODE § 5 et la carte de la machine
(`~/Projets/LISEZ-MOI.md`, pour tous ses projets).

**Et toute la machine** (sa parole : *« yep je veux bien que tu mettes earlyoom »*) : `earlyoom` veille
depuis 18 h 32 (`/etc/nixos/materiel-vivobook.nix`, commit `f3150c3` de sa configuration) — les seuils
de Fedora, 4 % de mémoire disponible et 10 % de swap libre pour demander (SIGTERM), la moitié pour
tuer ; il vise d'abord un compilateur (`rustc`, `cc1`, `ld`), jamais le bureau (niri, Xwayland, la
barre, kitty) ; une notification dit qui il arrête. Vérifié : la configuration évaluée sur une copie,
la reconstruction réduite à 26 Ko, le service lancé avec ces seuils. **Pas éprouvé en vrai** : il
faudrait remplir sa mémoire exprès. Les sources : la page de manuel (<https://www.mankier.com/1/earlyoom>),
la discussion de Fedora sur earlyoom et zram (<https://lists.fedorahosted.org/archives/list/devel@lists.fedoraproject.org/message/6YDJMPXQB3YC5S7CPI7MLUSLAILU7CB4>).
Deux essais pour le mot de passe : un terminal ouvert par niri est resté hors de sa vue (`sudo` a
abandonné au bout de cinq minutes) ; puis `pkexec`, dont l'environnement vide a fait tomber mon script
(`set -u` avant `/etc/set-environment`) — simulé ensuite dans un environnement vide avant le
troisième. `nix profile upgrade`, lui, compile hors de toute borne : il s'est fait sous un chien de
garde qui l'aurait coupé sous 0,8 Go libres.

## 6. Les sabotages

* `outils/sabotages/rectif-110.json` : **22 sur 22** — le pourquoi (la longueur du texte, « Eau » et
  « eau », la ponctuation, un contrôle qui sépare, le parent le plus lointain), le format (les
  sujets, un écrit d'avant eux, l'identifiant et le nom de l'IA de Glucose), la provenance (un agent
  qui retire, l'humain qui reprend, la relecture à l'ouverture), les rectifications (un agent, un
  texte sans famille, un texte déjà coloré, la flèche de l'humain, retirée ou retouchée, le suivi, la
  déclaration, le rang). **Une épreuve aveugle trouvée** : le texte déjà coloré était recoloré de la
  même couleur — un geste qui ne change rien ne s'écrit pas ; elle le recolore d'une autre.
* **Un sabotage de QUEUE-1 « passait »** : rejoué à la main, l'épreuve tombait. Une épreuve
  graphique qui n'obtient pas la carte se saute et compte comme réussie — en parallèle, plusieurs se
  la disputent. `outils/saboter.py` rejoue désormais seul, un fil à la fois, un sabotage qui semble
  passer, avant de dire l'épreuve aveugle. **QUEUE-1 : 4 sur 4.**
* L'extraction des mains (`store/journal/mains.rs`, cliquet des 600 lignes) a déplacé deux gardes
  de `provenance-108.json` ; le renommage de la droite, deux de `zoom-109.json` : mis à jour.
  **Une soixantaine de sabotages anciens** (fiches 72 à 100) visent du code qui a bougé depuis leur
  chantier : ils ne servent plus qu'à le relire.

## 7. La recherche (consultée le 12/11/2026)

* Möller, Nikolaev et Padó, *An Attribution Method for Siamese Encoders* (EMNLP 2023) et
  *Approximate Attributions for Off-the-Shelf Siamese Transformers* (EACL 2024,
  <https://aclanthology.org/2024.eacl-long.125.pdf>, <https://arxiv.org/abs/2402.02883>) : une
  attribution **exacte** — la somme des parts est la ressemblance — n'existe pour un encodeur à
  attention qu'au prix d'un réentraînement ; le cosinus y fait obstacle. Pour un modèle statique,
  elle est exacte par linéarité, sans rien toucher.
* Opitz et al., *Interpretable Text Embeddings and Text Similarity Explanation: A Survey*
  (EMNLP 2025, <https://arxiv.org/abs/2502.14862>).
* Joachims, Granka, Pan, Hembrooke et Gay, *Accurately Interpreting Clickthrough Data as Implicit
  Feedback* (SIGIR 2005, <https://www.sigir.org/wp-content/uploads/2017/06/p003.pdf>) : un clic est
  informatif mais biaisé ; il dit une préférence relative, pas une étiquette — d'où : suivre une
  flèche s'écrit, ne décide rien.

## 8. Les leçons

* **Une commande peut être plusieurs grosses compilations** : `cargo` en lance une par cœur. Borner
  la mémoire de la commande, pas compter les commandes. Et le swap compressé ne protège pas : il
  retarde la mort jusqu'à l'étouffement.
* **Un compteur dans la mesure trouve le défaut que l'épreuve ne voit pas** : `fin_prevue_us` a
  montré la droite à l'envers dès la première mesure ; sans lui, le gain modeste se serait lu comme
  « fini ».
* **Un geste qui ne change rien n'existe pas** : une épreuve qui « recolore » de la même couleur
  n'exerce rien (le sabotage l'a dit).
* **Remettre la conception en question au moment d'écrire** : trois choix de la fiche 109 sont
  tombés (le seuil, le suivi qui sort du jugement, la sorte de l'écrit), chacun sur un argument.
