# 76 — Le tri de la demande, et la flèche qui pointe

> Session du 17/10/2026 (le soir), suite de la fiche 75. Sa parole, mot pour mot dans son texte du
> 17/10 : *« une bulle axée sur la demande utilisateur — est-ce que c'est une réponse qui nécessite
> d'être courte et peut être répondue comme ça ? a-t-elle besoin de plus d'outils ? a-t-elle besoin
> d'un workflow complet, et de quelle taille ? — une bulle recherche qui impactera la bulle demande
> si Protéo découvre finalement plus finement ce que voulait l'utilisateur »*, et *« créer des
> flèches entre ses références et le document […] le surlignage et le pointage de la flèche »*.

## 1. La recherche

* **L'estimation de complexité d'une tâche d'agent** (tianpan.co, avril 2026, « agentic task
  complexity estimation » ; l'étude des stratégies de routage, arXiv 2502.00409) : des paliers —
  une réponse directe, une tâche, une orchestration —, une décision de routage qui doit rester
  **légère**, et l'erreur qui coûte le plus est la **sous-estimation** : « une réponse confiante mais
  incomplète ». La surestimation ne coûte que du temps.
* **tldraw** (le kit d'agent, la conférence de juin 2026) : rien sur la provenance ni sur des flèches
  qui citent. **Les outils d'annotation** (LiquidText, MarginNote, le paquet LaTeX `smartnotes`) :
  le lien vers un extrait part **de l'extrait**, et `smartnotes` place ses notes du côté qui garde le
  trait « aussi court et direct que possible ».

## 2. Le tri (Protéo, `proteo-server/src/tri.rs`)

**Le programme pose deux questions fermées, le modèle répond vrai ou faux, le programme en tire le
tri** : *faut-il agir ou vérifier ?* et *plusieurs étapes ?* — plusieurs étapes, un **workflow** ;
une seule, des **outils** s'il faut agir, sinon une **réponse**. Au format imposé (schéma JSON,
température 0, 32 jetons au plus : le bilan du 16/10 avait vu 9 600 jetons décodés dans le vide).

**Mesuré sur ses 28 demandes réelles** (relevées dans `conversations/`, telles qu'il les a tapées,
plus six courtes), contre le vrai `qwen2.5:7b` — l'épreuve à la main `le_tri_sur_ses_demandes` :

| essai | justes | fautes graves (un travail trié « réponse ») |
|---|---|---|
| une question à trois valeurs | 24 / 28 | 1 |
| la même, deux phrases de plus dans ses définitions | 19 / 28 | **9** |
| deux questions vrai/faux | 22 / 28 | 1 |
| **deux questions, et la règle de l'année** | **23 / 28** | **0** |

La deuxième ligne a décidé la forme : deux phrases ajoutées, et le 7B triait « réponse » des scripts
à lancer. Le sens d'une valeur tenait tout entier dans des définitions qu'il pèse mal ; deux
questions sur un seul axe chacune se tiennent seules. La faute restante — « qui a gagné la coupe du
monde 2026 ? » trié « réponse » — n'est pas une affaire de consigne : le modèle croit savoir, et ce
qu'il sait s'arrête à son entraînement. **Le programme, lui, sait la date** : une année en cours ou à
venir nommée dans la demande désigne un fait que le modèle ne peut pas avoir appris (`trancher`).
La règle ne peut que faire agir. Les cinq écarts qui restent sont tous entre outils et workflow,
dans les deux sens : un serpent Blender trié « outils » garde `mission_ouvrir`, un script trié
« workflow » coûte un plan de trop.

**La route** (`agent/mod.rs`) :

* **une réponse** : aucun outil montré, une consigne courte (`consigne_de_reponse`), et pas de
  relance « tu as annoncé sans agir » — il n'y a rien à appeler ;
* **des outils** : la boucle d'avant, inchangée ;
* **un workflow** : **le programme ouvre la mission** et en fait fixer le plan
  (`ouvrir_par_le_programme`) — mesuré le 02/10, le 7B laissé libre annonçait un plan sans l'ouvrir ;
  le modèle garde la main sur les étapes, leurs preuves et la réévaluation ;
* **une mission en cours** est déjà un workflow : la demande qui la reprend ne se trie pas ;
* **un tri impossible** (Ollama absent, une réponse illisible) ne décide rien : la boucle garde tous
  ses outils, comme avant. Le contexte du tri est la dernière réponse de Protéo (« et pour Linux ? »).

**Ce que je contredis** : le commentaire laissé dans la boucle par une session d'avant disait
« Protéo est pleinement autonome : aucune interception automatique n'impose une mission ». C'est sa
demande même qui veut le tri, et le bilan du 16/10 qui dit pourquoi le programme doit le faire.

## 3. La toile repensée (`toile.rs`, `toile/dessin.rs`)

**Une toile par demande**, dont la clé est le tour (une demande refaite demain est une toile neuve ;
un appel retenté met à jour, AGENT-4). La bulle **Demande** porte la demande, **son tri** — et, pour un
workflow, **sa taille mesurée par le plan**, puis chaque étape cochée ou écartée, tenue à jour à
chaque sauvegarde de la mission —, ce que chaque recherche en comprend, puis **la réponse de
Protéo**. Un tri que le travail dément le dit (« des outils, en une tâche — devenu un workflow
complet, 3 étapes ») : c'est le travail qui précise la demande. **N recherches** par demande, une
bulle chacune (une mission à deux axes posait deux bulles Demande identiques). Le banc d'essai ne
pose plus rien sur son tableau (`Cadre::toile`).

**Un trou de diagnostic fermé** : le 17/10 à 20 h 11, il a lancé l'essai ; le journal de Protéo n'a
rien dit de la toile — une toile posée et une toile qui attend son accord laissaient le même journal,
vide. Chaque événement posé s'écrit désormais (`toile.pose`, ses gestes et ses refus).

## 4. FLECHE-6 — une flèche ancrée à un passage le pointe (Glucose, `arrow_anchor.rs`)

La fiche 75 le laissait à son jugement ; je l'ai tranché, parce que son texte le demande (*« le
pointage de la flèche »*) et qu'une ancre qui ne pointe pas son passage ne sert à rien. Le bout
visait le milieu du nœud à la hauteur du passage et s'arrêtait au bord sur la droite qui y mène — la
loi de Tauri : venue de loin, elle touchait le bord du haut, la citation quatre cents unités plus
bas. **Désormais, quand ce qu'elle relie est à côté du nœud (hors de sa largeur), elle sort par le
côté qui fait face, à la hauteur exacte du passage** ; juste au-dessus ou au-dessous, la loi d'avant.
Aucune constante : la frontière est le bord du nœud. La droite qui part d'un point du côté gauche
vers un point plus à gauche ne traverse jamais le nœud. Une seule loi, que le dessin, le clic et les
deux voies lisent (ARROW-1). L'image l'a montré sur une recherche neuve : les trois flèches des
citations [1] à [3] arrivent chacune sur le flanc de la page, à côté de sa citation.

## 5. Prouver

* **Protéo** : 131 + 99 épreuves vertes (215 avant) ; 14 neuves pour la toile et le tri, 6 pour la
  route dans la boucle (un faux Ollama qui répond au tri hors script) ; **24 sabotages sur 24**
  (`PROTEO/outils/sabotages-tri-et-toile.json`, joué par `outils/saboter.py` de Glucose). Une épreuve
  **aveugle** trouvée en route : la mesure réelle passait « ok » avec 0 sur 28 — Ollama était mort avec
  Protéo, chaque réponse était illisible, et l'épreuve ne comptait que les fautes graves. Elle exige
  désormais zéro réponse illisible.
* **Glucose** : FLECHE-6, 3 épreuves (la loi, sa preuve à l'envers ; le vrai chemin `path_with` avec un
  nœud qui mesure son passage) ; **5 sabotages sur 5** (`outils/sabotages/fleche-6.json`).
* **L'image** : un Glucose sans fenêtre, le vrai Protéo, le vrai web — « Comment les abeilles
  communiquent-elles entre elles ? » : 23 pages lues, 8 citations, 57 gestes, aucun refusé.

## 6. Ce qui reste

* **Son essai** : ce qu'il a vu le 17/10 à 20 h 11 (la recherche sur Rennes) — on ne le sait pas.
* **Le flou** (sa parole) : *« au cours de ses recherches il peut se rendre compte qu'il ne connaît
  pas assez […] il doit pouvoir de lui-même organiser des recherches beaucoup plus intensives, en
  parallèle »* — le bilan l'appelle FLOU_DETECTE ; pas fait.
* **Les flèches qui traversent** les colonnes voisines sur de longues lignes droites : l'itinéraire
  évite les nœuds, pas les membranes. À regarder avec lui.
* **Protéo n'est pas enregistré** dans git : 1 161 lignes d'une session d'avant mêlées aux nôtres —
  à sa parole.
* **Le relais de Claude Code** (`Documents\Glucose-relais\glucose.exe`, une copie : Claude Code garde
  le relais vivant toute sa session, et un exécutable tenu ne se remplace pas). Quand une version
  publiée aura la porte, viser l'installée posera une question : l'installeur attend que Glucose
  soit fermé — un relais tenu par Claude Code le ferait attendre.
