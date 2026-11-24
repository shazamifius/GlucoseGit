# 149 — Ctrl emporte ; la flèche en double ; le flou (24/11 au soir, le relais, compte b)

## 1. Q10 — `Ctrl` + glisser emporte hors de Glucose

Sa validation du 24/11 (`relais-rep-q10`) : **`Ctrl` + glisser un nœud l'emporte vers une autre
fenêtre**, même posée par-dessus Glucose — Discord, un navigateur — qu'on ne rejoint pas en sortant
de la sienne. Aucun bureau ne prend `Ctrl` + glisser (Alt et la touche du logo déplacent les
fenêtres) ; c'est le « copier » de tous les explorateurs.

**Fait** : `interactions/mouse.rs`, `part_ailleurs` — les nœuds tenus partent au bord de la fenêtre
(fiche 51 § 2), ou dès que la main bouge avec `Ctrl`, par le même `emporter_hors_de_la_fenetre`.
Seulement là où un glisser sait partir (Windows, Wayland par son pont, fiche 100). **Pas de seuil** :
un `Ctrl` + clic qui tremble part et retombe sur Glucose, qui ne pose pas ce qui vient de lui.
Épreuve : `Ctrl` + appui prend le nœud (`selection/tests.rs`).

**Essayé au pilote** (sway sans écran, la version optimisée) : `Ctrl` tenu par le clavier virtuel,
appui sur le texte d'accueil, la main monte de 200 px. Le journal du compositeur (`PILOTE_DETAIL=1`)
montre `start_drag`, les cinq formes offertes, l'action « copier », le glisser qui survole Glucose,
puis `cancelled` au lâcher ; la capture : le nœud choisi, à sa place, rien de doublé. Le témoin sans
`Ctrl` : le nœud suit la main.

**Piège payé** : le premier clic après que la fenêtre a pris le clavier, sans mouvement de souris
entre les deux, est ignoré exprès (`clic_de_reveil`) — et le clavier virtuel qui tient `Ctrl` lui
donne le clavier. Au pilote : bouger la souris après avoir pris `Ctrl`, avant d'appuyer.

**Reste** : l'ombre du nœud sous la main (une icône de glisser : `start_drag(…, icon)` sous Wayland,
`IDragSourceHelper` sous Windows) — aujourd'hui le curseur seul dit le glisser ; au téléphone, l'appui
long ; sous macOS, `Ctrl` + clic est le clic droit : ce sera `⌥` (le « copier » du Finder) le jour où
le Mac saura glisser dehors. Son essai à l'écran (vers Discord ou un navigateur posé par-dessus).

## 2. La flèche en double — sa note du 24/11 (`text-pxtg-2317`)

*« On peut créer 2 flèches qui vont au même endroit […] il faudrait indiquer à l'utilisateur et savoir
s'il veut réellement une duplication. »* Deux flèches aux mêmes bouts suivent le même itinéraire : la
seconde se cache sous la première. **Proposé** (`relais-rep-fleche-double`) : au lâcher d'une flèche
dont les deux bouts (nœud et passage) sont déjà reliés, dans un sens ou dans l'autre, la question
« Une flèche relie déjà ces deux nœuds. En tracer une seconde ? » — non : l'ancienne est choisie ; oui :
la seconde passe à côté. Une IA qui relie deux nœuds déjà reliés met à jour la flèche existante.

**Fait** (le même soir) :
* **le noyau** (`arrow/double.rs`) : `qui_relie_deja` — mêmes nœuds, mêmes passages (bloc et
  sélection), dans un sens ou dans l'autre ; un bout libre n'en double aucune. `a_reprendre`, pour
  les IA : la flèche de même relation d'abord, sinon une sans relation, ou n'importe laquelle si
  l'IA n'en donne pas ; **deux relations différentes sont deux affirmations** (« inspire » et
  « contredit ») et gardent chacune leur flèche. `Store::fleche_qui_relie_deja` (`store/double.rs`).
* **sa main** (`interactions/fleche_en_double.rs`) : au lâcher (`finish_draw`), si la flèche tracée
  en double une autre, son geste s'abandonne (`cancel_live_edit` : rien dans l'histoire) et la
  question se pose (`Suite::FlecheEnDouble`). « Tracer la seconde » la pose d'un geste qu'un Ctrl+Z
  défait ; « Annuler » ou Échap choisit l'ancienne.
* **les IA** : `relier_pour_un_agent` reprend la flèche qui dit la même chose — ce que l'IA dit de
  neuf (relation, texte) s'y écrit, rien d'autre n'est effacé — et rend son identifiant, que
  `glucose_relier` renvoie.
* 8 épreuves (noyau 4, application 5 — dont « un seul Ctrl+Z »), 7 sabotages sur 7
  (`outils/sabotages/double-149.json`).

**Reste — la seconde à côté** : aujourd'hui, « Tracer la seconde » la pose encore sous la première.
Il faut que l'itinéraire écarte les flèches parallèles (mêmes bouts) l'une de l'autre, comme tout
dessinateur de graphes : un rang parmi ses parallèles, demandé par `Noeuds`, et un écart d'une
pastille (on doit pouvoir lire les deux étiquettes) ; tous les lecteurs du tracé (dessin, clic,
poignées, étiquette) passent par `morceaux_with` et suivraient d'eux-mêmes. Cela vaut aussi pour les
doubles qui existent déjà dans son document.

## 3. Le flou sous NVIDIA — sa note du 24/11 (`text-pxtg-2313`)

*« Avec NVIDIA […] le pilote applique ses préréglages au logiciel […] et rend tout extrêmement
flou »* — d'un autre appareil du canevas partagé (« Hibou Indigo »), une capture de 364 × 353 trop
petite pour trancher. Trois suspects : le repli de Glucose (`scene_gpu` pose l'ancienne image, un peu
floue, quand la nouvelle coûte trop), Windows qui étire une fenêtre dont il croit l'échelle mal
comprise (125 %, 150 %), les réglages du pilote NVIDIA (profil de base appliqué aux programmes
inconnus). **Question posée** (`relais-rep-nvidia-flou`) : une capture de tout l'écran — la barre
d'outils nette et le canevas flou désigne Glucose, tout flou désigne Windows ou le pilote.
