# 90 — Les commandes du terminal : une seule table, deux portes

> Session du 8/11/2026, après la fiche 89. Sa demande du 7/11 : *« des commandes Linux directement pour
> faire des trucs avec Glucose, plein plein de commandes, dont l'ouvrir »*. Et la règle la plus forte de
> VISION : *« un Glucose doit pouvoir être lu par un humain ET par une IA, de quasi la même façon »*.

## 1. Avant : ce que la fiche 89 laissait

* **`Ctrl+F` n'était pas enregistré** : écrit, éprouvé en partie, ni commité ni saboté. En le relisant,
  un défaut : **une flèche trouvée par ce qu'elle dit ne faisait pas voler la vue**. Entrée la
  choisissait, puis rien — pas même une image redessinée. La cause : « voler jusqu'à un nœud » était
  écrit **trois fois** (la recherche, le panneau Domaines, l'éditeur d'ancres), chacune par
  `node_rect`, la boîte où une flèche s'accroche — et une flèche n'en a pas. **VOL-1** : une seule
  fonction (`GlucoseApp::voler_vers_le_noeud`), qui prend l'**emprise** du nœud
  (`glucose_core::arrow::emprise` : sa boîte, ou l'enveloppe de l'encre d'une flèche — ce que `F`
  cadre déjà). Les trois copies l'appellent ; le panneau Domaines et l'éditeur d'ancres y gagnent les
  flèches au passage.

## 2. La recherche (consultée le 8/11/2026)

* **[L'interface en ligne de commande d'Obsidian](https://www.dsebastien.net/the-complete-guide-to-the-obsidian-cli-everything-you-can-do-from-the-terminal/)**
  (février 2026, [son journal](https://obsidian.md/changelog/2026-02-27-desktop-v1.12.4/)) : le
  précédent le plus proche. Une télécommande de l'application ouverte, qu'elle lance au besoin ; des
  arguments `clé=valeur` ; plus de cent commandes. **Ses défauts, relevés par son guide** : elle rend 0
  même quand une commande échoue, et ne marche pas sans fenêtre.
* **[Les règles des interfaces en ligne de commande](https://clig.dev/)** : l'humain d'abord, la machine
  ensuite — une sortie lisible dans un terminal, stable dans un tuyau ; un code de sortie qui dit
  l'échec.
* **MCP ou ligne de commande, pour un agent ?** ([Manveer](https://manveerc.substack.com/p/mcp-vs-cli-ai-agents),
  [Firecrawl](https://firecrawl.dev/blog/mcp-vs-cli), [Descope](https://www.descope.com/blog/post/mcp-vs-cli)) :
  la ligne de commande s'enchaîne et coûte peu de contexte ; MCP porte l'accord et le suivi. Les
  chiffres cités sont presque tous anecdotiques. Leur conclusion commune : **les deux, sur la même
  chose** — c'est TERMINAL-1.
* **Comment un agent cherche** : Claude Code a quitté l'index de vecteurs pour une recherche par mots
  exacts que l'agent relance et affine ([un résumé](https://www.mindstudio.ai/blog/is-rag-dead-what-ai-coding-agents-use-instead),
  [« Is Grep All You Need? »](https://arxiv.org/abs/2605.15184)) — contesté, mais c'est la forme de
  `glucose_chercher` : exact, rapide, à relancer. Quand on ne sait pas les mots : les domaines (fiche 88).
* **Ouvrir un fichier par double-clic sous Linux** : un type déclaré (shared-mime-info,
  [Red Hat](https://docs.redhat.com/en/documentation/red_hat_enterprise_linux/10/html/using_the_gnome_desktop_environment/configuring-file-associations))
  et la ligne `MimeType` du fichier de bureau, avec `%f` dans `Exec`.

## 3. La conception

### TERMINAL-1 — une seule table, deux portes

**Chaque commande est un outil de la porte des agents** : son nom sans `glucose_` (`poser-texte`), ses
arguments lus par le schéma de l'outil, son aide tirée de sa description. Rien n'est écrit deux fois :
un outil neuf a sa commande le jour où il naît, et une commande ne peut rien qu'un agent ne puisse.

* `clé=valeur`, la valeur au type du schéma ; une clé répétée fait une liste (`ids=a ids=b`) ; du JSON
  pour une liste d'objets.
* **les mots sans `=`** vont à l'argument que l'outil exige, s'il n'en exige qu'un :
  `glucose chercher eau chaude`, `glucose poser-texte "Bonjour"`, `glucose retirer n1 n2`.
* **`-`** lit l'entrée standard : `cat notes.md | glucose poser-texte -`.
* une clé inconnue est refusée avec la liste des bonnes ; un mot qui n'est ni une commande ni un fichier
  aussi — ouvrir Glucose sur une faute de frappe la cacherait.

### TERMINAL-2 — la même porte, le même accord

Une commande parle au Glucose ouvert par sa porte — **une seule ligne** du protocole moderne, sans
poignée de main — et l'ouvre s'il ne l'est pas, comme `glucose --mcp`. Elle se présente sous le nom
**« le terminal »** et passe par l'accord (AGENTS-1) : un agent sait taper une commande, et le terminal
ne doit pas être une porte dérobée. Chaque geste s'annule par `Ctrl+Z`.

**La sortie** : le JSON de l'outil — ce que lit une IA est ce que lit l'humain —, en retrait dans un
terminal, sur une ligne dans un tuyau (`jq`). **Le code** : 0 ; 1 si Glucose a refusé, sa phrase sur la
sortie d'erreur ; 2 si la commande est mal écrite.

### AGENT-7 — l'agent cherche comme l'humain

`Ctrl+F` cherchait dans tout le document ; un agent, seulement dans le tableau qu'on regarde
(`glucose_lire`, `cherche`). **`glucose_chercher`** lui donne la même recherche : le même index, les
mêmes résultats dans le même ordre, la même page de dix rangées que la palette, `depuis` pour la suite.

### OUVRIR-1 — `glucose document.glucose`, et le double-clic

Le document demandé passe devant ce que le lancement aurait repris ; un brouillon laissé par un
plantage attend son tour, et le compte rendu le dit. Un document déjà ouvert dans une autre fenêtre de
Glucose s'y trouve, et la nouvelle le dit (comme `Ctrl+O`). Le double-clic : le type
`application/x-glucose` (`*.glucose`) et `Exec=glucose %f`, dans **un seul** fichier de bureau, celui des
paquets, que le paquet Nix prend désormais aussi (il en écrivait un second).

## 4. Prouver

* **Les épreuves** : TERMINAL-1 (chaque outil a sa commande et l'inverse, aucune ne tombe sur l'aide ni
  sur une option ; la ligne de commande s'aiguille ; les arguments suivent le schéma — types, clé
  répétée, mots nus, `-` lu une fois ; l'aide dit chaque argument) ; TERMINAL-2 (la requête est un
  appel moderne au nom du terminal ; la réponse se lit, refus compris ; **de bout en bout par une vraie
  porte** : la question d'accord au nom du terminal, le texte posé, trouvé par `chercher`, défait par
  `Ctrl+Z`) ; AGENT-7 (l'agent rend les ids de la palette, dans son ordre, la page de dix puis la
  suite, sans rien écrire) ; OUVRIR-1 (le document demandé passe devant un brouillon, qui le dit et
  reparaît au lancement suivant) ; l'écriture en retrait se relit pareil.
* **L'essai réel** (`outils/essai_terminal.sh`) : le vrai Glucose sans fenêtre, le vrai programme
  `glucose` contre sa porte — l'aide, poser, poser depuis l'entrée standard, chercher, une ligne dans un
  tuyau et quatorze en retrait dans un terminal, lire, retirer ; les cinq échecs et leurs codes (une clé
  inconnue, une faute de frappe, un refus de Glucose, un document introuvable, une porte muette) ; puis
  un document enregistré, **ouvert par son nom** dans un compositeur sans écran : la capture montre ses
  nœuds. **Tout passe.** L'essai a trouvé un faux vert dans l'essai lui-même : la porte « muette »
  échouait parce que son chemin dépassait les 108 caractères d'une prise Unix ; un chemin court, et le
  message attendu, désormais.
* **Sabotages** (`outils/sabotages/terminal-90.json`, avec ceux de `chercher-89.json`) : voir ÉTAT.

## 5. Les limites, dites

* **Sous Windows**, Glucose est une application fenêtrée : ce qu'une commande écrit ne paraît pas dans
  la console qui l'a lancée, seulement redirigé (`glucose lire > sortie.txt`, `| more`). La réparation
  propre est un second petit programme en mode console, comme `code.cmd` pour VS Code ; rien de Windows
  ne s'essaie ici, et sa demande portait sur Linux : **à faire, avec la CI pour juge**.
* **Une commande ouvre Glucose** s'il ne l'est pas (comme `glucose --mcp`), et parle au Glucose qui tient
  la porte — le premier ouvert. Aucune lecture d'un fichier fermé sans fenêtre (ce qu'on reproche à
  Obsidian) ; `glucose domaines` lit déjà les fichiers fermés, par le Glucose ouvert.
* **La première commande pose la question d'accord** dans la fenêtre de Glucose, au nom du terminal, et
  l'accord vaut jusqu'à la fermeture (AGENTS-1). Plus lourd pour l'humain qui tape lui-même ; c'est le
  prix de ne pas ouvrir une porte dérobée aux agents qui tapent des commandes.
* **Le double-clic sous NixOS** : le paquet porte le type `.glucose`, mais NixOS ne recalcule la liste
  des types que pour les paquets du système (`/etc/nixos`, son mot de passe). Installé dans son profil,
  le type se déclare dans son dossier (`~/.local/share/mime`), une fois.
* **La sortie est du JSON**, le même pour l'IA et pour l'humain (sa règle « lu pareil ») ; lisible en
  retrait, mais pas une phrase. Une mise en forme par commande serait une seconde table : pas avant
  qu'un usage la demande.

## 6. Les leçons

* **Relire le travail d'une autre session avant de l'enregistrer** : Ctrl+F était « fait », ses
  épreuves vertes ; la flèche ne volait pas. Le défaut était dans une fonction écrite trois fois.
* **Un essai réel peut mentir aussi** : un code de sortie juste pour une mauvaise raison. Vérifier le
  message, pas seulement le code.
* **Préparer hors du dépôt pendant que la suite tourne** : les cliquets lisent tous les fichiers, même
  ceux qu'aucun `mod` ne nomme ; un module en préparation les aurait fait tomber. Les fichiers à changer
  se copient au scratchpad, s'y éditent, et reviennent d'un coup.
* **Nix ne voit que ce que git connaît** : un fichier neuf que le paquet nomme (`glucose-mime.xml`)
  empêche `nix develop` de s'ouvrir tant qu'il n'est pas ajouté.
