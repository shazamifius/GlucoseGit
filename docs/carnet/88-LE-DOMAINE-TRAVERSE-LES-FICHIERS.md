# 88 — Le domaine traverse les fichiers, et une IA le lit comme l'humain

> Session du 7/11/2026, après la fiche 87. Le chantier que **VISION** désigne : *« pour l'instant, on se
> concentre sur le domaine »*. Sa définition, du 22/10 : « l'eau, ça mouille », « eau = H₂O », « l'eau
> dans un jeu vidéo » sont trois blocs séparés, peut-être dans des fichiers différents, **et le domaine
> les traverse tous**. Sa règle, du 7/11, dite pendant cette session : *« on ne peut pas inscrire deux
> domaines avec le même nom ; ils doivent impérativement demander à l'IA ou à l'humain la spécification
> ou la spécialisation du mot — ça donnerait python code et python serpent »*.

## 1. Avant le domaine : ce que la fiche 87 laissait

* **Windows rouge sur GitHub, une deuxième fois.** Après `bench_memoire`, c'était l'épreuve de l'atlas :
  « un écart de 2 sur 255 ». Sa borne de 1 était une supposition. Une carte lit à 1/2⁸ de texel près
  sur chaque axe (`D3D12_SUBTEXEL_FRACTIONAL_BIT_COUNT` = 8 ; Vulkan en fait un palier de sa feuille de
  route 2022), et la page arrondit la même place autrement que la texture seule : sur du bruit, au plus
  255 × 2 / 2⁸, moins de deux unités. **La borne se déduit désormais**, et ses deux sabotages tombent
  encore : une lecture du demi-texel voisin coûte des dizaines d'unités. **`39894bf` : dix vertes**
  (exécution 42402082754, 2 878 épreuves sous Windows).
* **Les 20 sabotages restants de `huit-mille-2.json` : 16 tombés, 4 aveugles**, chacun corrigé par une
  meilleure épreuve et rejoué : une marque comparée à une valeur déjà périmée (CHOIX-1) ; le cas « sans
  serveur » n'avait jamais d'accord, il ne pouvait rien prouver (JOURNAL-1) ; une question hors image
  posée une seule fois (PHOTO-1) ; un numéro d'image que l'épreuve ne lisait pas (LOT-1). **Les 75 gardes
  sont jouées.**
* **Le brouillon de la beta 9** (`v2.0.3-beta.9`, sur `b04ccef`, onze fichiers) attend son clic. Il ne
  porte pas ce qui suit.

## 2. Ce qui manquait

Le noyau avait la bonne forme pour **un** document : un catalogue (`Project.domains`), et chaque nœud
porte ses domaines avec un poids de 0 à 1. Il manquait trois choses :

1. **Traverser les fichiers** : chaque document avait ses identifiants (`domain-3`), qui ne voulaient
   rien dire ailleurs.
2. **Être lu par une IA** : `glucose_lire` ne disait pas les domaines. Contraire à la règle la plus forte
   de la vision : *« un Glucose doit pouvoir être lu par un humain ET par une IA, de quasi la même
   façon »*.
3. **Sa règle du nom** : rien n'empêchait deux « Python ».

## 3. La recherche (consultée le 7/11/2026)

* **Le wiki fédéré** de Ward Cunningham ([Jon Udell](https://blog.jonudell.net/2015/01/02/how-federated-wiki-neighborhoods-grow-and-change/)) :
  chacun a ses pages, et un **voisinage** se forme des sites qu'on a rencontrés ; les pages s'y
  désignent par leur nom, et l'on forke. Le précédent le plus proche : pas de serveur central, une vue
  faite de ce qu'on a croisé. Je n'ai pas trouvé sa règle exacte de correspondance des noms.
* **[SKOS](https://www.w3.org/TR/skos-primer/)** : un concept est identifié **indépendamment de ses
  étiquettes** ; `exactMatch` relie deux concepts de deux schémas sans les fondre. C'est DOM-5 : une
  identité fixe, un nom qui change, et la réunion de deux domaines comme un geste explicite.
* **Les étiquettes d'Obsidian** ([Tag Wrangler](https://forum.obsidian.md/t/tag-mass-action-add-rename-and-delete-a-tag-in-multiple-files-tag-wrangler/567/190),
  [les identifiants stables](https://forum.obsidian.md/t/uuid-for-note-links-in-url-scheme/14617)) : une
  étiquette n'est que du texte ; la renommer réécrit tous les fichiers, avec sauvegarde conseillée.
  Ce qu'on évite : renommer ici n'écrit dans aucun fichier fermé. Et [un essai](https://dev.to/synthaicode_commander/stable-ids-are-a-semantic-decision-not-a-file-trick-2oom)
  le dit bien : un identifiant stable est une décision **de sens**, pas un tour de fichiers — d'où la
  question qu'on pose au lieu de deviner.
* **[Folksonomie contre ontologie](https://www.cl.cam.ac.uk/~aac10/R207/ontology_vs_folksonomy.pdf)** :
  les étiquettes libres confondent les homonymes et séparent les synonymes. Sa règle règle les
  homonymes ; les synonymes (« eau », « water ») restent deux domaines (§ 8).
* **[Le logiciel local d'abord](https://www.inkandswitch.com/essay/local-first/)** : les données restent
  dans des fichiers à soi. D'où l'absence de catalogue central.
* **[L'appartenance possibiliste](https://rdrr.io/cran/ppclust/man/pcm.html)** (Krishnapuram et Keller) :
  des poids qui ne se somment pas à un, chacun une « typicalité ». C'est ce que Glucose fait déjà : un
  nœud peut être à 1 dans « eau » et à 1 dans « chimie ».
* **[Écrire des outils pour des agents](https://www.anthropic.com/engineering/writing-tools-for-agents)** :
  peu d'outils, des noms lisibles plutôt que des identifiants opaques, des erreurs qui disent quoi
  faire. D'où : un agent désigne un domaine **par son nom**, et chaque refus donne la correction.
* **Les agents qui ne savent pas ce qu'ils cherchent** : [CoG](https://arxiv.org/abs/2609.12791) (EMNLP
  2026) fait explorer un agent par cycles de plan, de recherche et de réflexion ; sa page ne donne
  aucun chiffre. [ACE-GraphRAG](https://arxiv.org/abs/2608.01269) a été retiré par son auteur : écarté.
  **Aucune preuve solide trouvée** qu'une table des matières aide un agent : je mesure ce qu'elle
  coûte à lire (§ 7), pas encore ce qu'elle fait trouver.

## 4. La conception

### Pas de catalogue central

Chaque fichier garde les domaines qu'il porte, et les emporte quand on le déplace ou qu'on l'envoie.
Un catalogue à part serait une seconde vérité : perdu, il viderait les fichiers de leur sens ; envoyé
sans lui, un fichier arriverait muet. **Le format du fichier ne change pas** (schéma 5) : le champ
`created_at` d'un domaine s'appelle désormais `defini_a`, et se lit de la même façon.

### DOM-4 — un nom, un domaine (sa règle)

Deux domaines ne portent jamais le même nom. La casse et les blancs en trop ne font pas un autre nom ;
les accents, si (« pêche », « péché »). Dans un document, le noyau refuse (`try_add_domain`,
`try_update_domain`) ; face aux autres fichiers, l'application et la porte des agents refusent. **Le
refus est la question** : « « Python » existe déjà : 1 nœud dans 1 fichier, par exemple « Python est un
langage de programmation. ». Si c'est le même sujet, prends ce domaine ; sinon, précise le mot. »

### DOM-5 — l'identité naît une fois, le nom peut changer

Un domaine a pour identifiant son premier nom et l'instant de sa naissance : `eau@1794080151933`. Le
nom se change ensuite sans toucher à l'identifiant, ni aux nœuds, ni aux autres fichiers, qui
apprennent le nom neuf **par la date la plus récente** (`defini_a`). Deux naissances du même nom dans la
même milliseconde (le nom d'attente « Domaine 1 », repris aussitôt) : la seconde naît à la suivante.
Les domaines d'avant (`domain-3`) gardent leur identifiant — l'histoire d'un fichier se rejoue
strictement, et le réécrire hors du journal arrêterait ce rejeu — ; leur identité se lit dans leur
nom et leur date.

**Deux créations séparées du même nom ont donc deux identités.** Quand leurs fichiers se rencontrent,
rien ne les réunit en silence : c'est une question, et deux réponses — **même sujet** (`reunir_un_domaine`,
un geste : tout ce qui le porte suit) ou **préciser le mot**.

### RECENSE-1 et 2 — ce que Glucose connaît, sans jamais écrire

`glucose_core::recensement` (pur) met côte à côte ce que les fichiers disent ; `glucose-desktop/src/recensement.rs`
les lit sur un fil à lui : les documents déjà ouverts ou enregistrés (`documents-connus.txt`, une ligne
par chemin), **leurs voisins de dossier**, les brouillons et les documents rangés. Rien d'autre : Glucose
ne fouille pas le disque. **Il n'écrit jamais dans un fichier qu'il lit.** Une passe ne relit que ce dont
la date ou la taille a changé, et **le fichier ouvert ne se lit jamais** : il est vivant dans Glucose.

### Pour un agent : AGENT-5 et AGENT-6, deux outils de plus (onze en tout)

* `glucose_lire` dit les domaines de chaque nœud (`[{"domaine":"Eau","poids":0.8}]`), le catalogue du
  document (`domaines`, avec `a_preciser` quand un nom désigne deux sujets), filtre par `domaine`, et dit
  le **fichier** ouvert.
* `glucose_poser_texte`, `_membrane`, `_page` posent les domaines du nœud **dans le même geste**.
* **`glucose_poser_domaine`** : `domaine` prend un domaine qui existe, ici ou ailleurs (sa définition
  entre alors dans ce fichier, pour qu'il reste entier) ; `nouveau` en crée un ; **AGENT-5** : un agent
  dit toujours lequel des deux il veut — sinon « poser python » mélangerait le serpent et le langage sans
  qu'il le sache. Un poids nul retire (**AGENT-6**), et ne fait jamais entrer un domaine. `renommer`,
  `meme_sujet` : les deux réponses à la question.
* **`glucose_domaines`** : la table des matières de tout ce que Glucose connaît, puis les nœuds d'un
  domaine dans tous les fichiers, du plus lourd au plus léger — leur première phrase, ou leur texte
  entier, ou la phrase qui contient des mots —, **sans rien ouvrir à l'écran**. `a_jour` dit si une
  lecture du disque est encore en route.
* **Reposer un nœud garde ses domaines** : un agent qui récrivait un texte remplaçait le nœud entier, et
  les domaines que l'humain y avait mis disparaissaient sans un mot. Trouvé en écrivant ce chantier.

### Pour l'humain : le panneau Domaines (DOM-UI-4)

Sous les domaines du document, ceux qu'on connaît d'ailleurs, avec leur nombre de nœuds et de
fichiers ; un palier les pose sur la sélection (ils entrent dans ce fichier, un seul `Ctrl+Z`). Un
homonyme remplace les paliers par **« Même sujet ? Oui / Non »** : rien ne se pose tant qu'on n'a pas
répondu. **« où »** liste chaque place où vit un domaine, tous fichiers confondus ; un clic y va — dans
un autre fichier, on quitte le document comme d'habitude (rien ne se perd), il s'ouvre, et la caméra vole
au nœud. La molette fait défiler les rangées. **La couleur sur le canevas et le filtre restent pour
l'ergonomie** (sa parole du 22/10).

**Au téléphone, le nom d'un domaine ne se tape pas encore** (INVENTAIRE) : la création garde donc un nom
d'attente, le premier « Domaine N » libre partout, plutôt que d'attendre un nom qu'on ne pourrait pas
écrire. Une première version créait le domaine seulement une fois nommé : elle aurait rendu la création
impossible au téléphone. Vu en relisant, avant tout essai.

## 5. L'essai réel

Le vrai Glucose sans fenêtre (`porte_sans_fenetre`, `GLUCOSE_DOSSIER_ESSAI`), de vrais fichiers sur le
disque, un client d'agent par la vraie porte, et la main d'épreuve qui enregistre, ouvre et crée des
documents. Le script est `outils/essai_domaine.py`. **Tout passe** :

1. « Eau » posé sur deux blocs de `physique/eau.glucose` ; un document neuf, `jeu/moteur.glucose`, connaît
   « Eau » sans l'ouvrir ; recréer « eau » est refusé, avec un exemple ; le prendre fait **trois blocs, deux
   fichiers, un seul domaine** ; l'agent lit le texte entier d'un nœud du fichier fermé, et l'écran n'a
   pas bougé.
2. Fermé, relancé : Glucose sait encore.
3. Le fichier déplacé loin : il sort de la vue ; rouvert d'où il est, ses domaines étaient en lui.
4. Deux « python » créés dans deux Glucose qui ne se connaissent pas, puis un fichier copié à côté de
   l'autre : **les deux sont marqués, rien n'est réuni**, la pose est refusée avec les deux sens (« un grand
   serpent constricteur », « un langage de programmation ») ; l'agent renomme en « Python code » ; plus de
   question.

**L'essai a trouvé ce que 137 épreuves ne voyaient pas** : le document ouvert se comptait **deux fois** —
sous son identité, qu'enregistrer ne change pas (fiche 80), et sous son fichier, que le fil lisait aussi
(quatre nœuds pour deux). Il se désigne désormais par le fichier où il s'écrit. Et le banc ne fermait pas
son document : les derniers gestes ne s'écrivaient pas (le vrai Glucose, lui, finit proprement, FIN-1).

## 6. Prouver

* **Noyau : 972 épreuves**, dont `nom`, `recensement`, `agents::domaines`. **L'application** : les domaines,
  la porte, le fil (de vrais fichiers, son réveil, ce qu'il relit), le panneau par ses intentions, l'aller
  dans un autre fichier de bout en bout. Clippy strict à zéro.
* **Sabotages** (`outils/sabotages/domaine-88.json`, 33 gardes) : voir ÉTAT pour le résultat.

## 7. Le chiffre

* **Lire le tableau de 8 000 images (2,3 Go)** en arrière-plan : **2,6 s, 93 Mo au plus**. Aucun cache : il
  ne se relit que s'il change, et jamais quand il est ouvert.
* **Ce qu'un agent lit pour trouver un sujet**, sur un tableau fabriqué (300 textes de six phrases, 30
  portent deux domaines) : tout lire, 169 737 caractères ; le sommaire, 51 044 ; **la table des domaines
  puis les 12 nœuds d'un domaine, 4 605 — 9 % du sommaire**. Ce chiffre mesure **le coût de lecture**, pas
  ce qu'une IA trouve mieux, et il suppose que les domaines ont été posés.

## 8. Les limites, dites

* **Un fichier jamais ouvert**, rangé loin des dossiers connus, reste invisible jusqu'à sa première
  ouverture.
* **Supprimer un domaine ne vaut que pour le fichier ouvert** ; un nom neuf se lit partout, mais ne s'écrit
  dans un autre fichier qu'à son ouverture (un geste annulable, que le message dit).
* **Les synonymes** (« eau », « water ») restent deux domaines. `exactMatch` à la SKOS viendra plus tard.
* **Un « é » écrit en deux caractères** reste différent d'un « é » écrit en un seul : le noyau n'a pas de
  table Unicode.
* **Le panneau n'a pas été vu à l'écran** : son dessin a été regardé en image (§ 9), pas sa vie dans une
  vraie fenêtre.

## 9. Les leçons

* **Une identité d'emprunt** : le document se désignait par l'identité que la fiche 80 avait faite pour les
  agents, qu'enregistrer ne change pas ; le fil, lui, le nommait par son fichier. Une chose, deux noms : il
  comptait double. *Ce qui se compare doit se nommer pareil des deux côtés.*
* **Un essai réel avant « fini »**, encore : 137 épreuves vertes, et deux défauts trouvés en vingt
  minutes de vrais fichiers.
* **Un chantier qui change un geste se relit plateforme par plateforme** (la leçon de la fiche 60, rejouée) :
  un nom d'attente retiré aurait fermé la création au téléphone.
