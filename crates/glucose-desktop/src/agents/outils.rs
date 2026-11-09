//! **Les outils que Glucose offre aux agents** : leur nom, ce qu'ils font, et la forme de leurs
//! arguments — ce que `tools/list` rend, et ce que [`super::executer`] sait faire. Une seule
//! liste : un outil annoncé et inconnu de l'exécution ne compile pas sans son épreuve
//! (`chaque_outil_annonce_s_execute`).
//!
//! Les descriptions sont écrites pour un modèle de langage qui ne connaît pas Glucose : ce qu'il
//! lit ici est tout ce qu'il saura.

use super::json::Json;

/// Ce que Glucose dit de lui-même à un agent, une fois, à la découverte.
pub const INSTRUCTIONS: &str = "Glucose est un canevas infini : des textes (Markdown, LaTeX), \
des post-it, des photos, des membranes (des bulles qui contiennent d'autres nœuds) et des \
flèches qui relient les nœuds. Ce que tu poses apparaît tout de suite sous les yeux de \
l'utilisateur, sur le tableau qu'il regarde ; chaque appel est un geste qu'il peut annuler \
(Ctrl+Z). Commence par glucose_lire : il rend le sommaire du tableau (chaque nœud, sa première \
phrase, sa taille) — sur un grand tableau, son plan d'abord (niveau plan : les membranes seules) ; lis ensuite en entier seulement les nœuds dont tu as besoin (ids), ou \
cherche une phrase dans tous (cherche). « selection » dit les nœuds que l'utilisateur a \
sélectionnés : quand il parle de « ça », c'est eux. L'empreinte d'un nœud change quand son texte \
change : garde-la, et à la lecture suivante, relis seulement ceux dont l'empreinte a changé — \
l'utilisateur les a récrits. Les positions sont en unités du canevas, l'origine en haut \
à gauche, y vers le bas. Si tu ne donnes pas de position, le nœud se pose sous le contenu de sa \
membrane (dans), ou à droite de tout le tableau : range ton travail dans des membranes. Donne \
tes propres identifiants (« proteo-rapport ») : poser deux fois le même met à jour au lieu de \
dupliquer. Une flèche peut partir d'une phrase exacte d'un texte et arriver à une phrase exacte \
d'un autre (passage_source, passage_cible) : ces phrases sont surlignées, et doivent s'y trouver \
mot pour mot. Un domaine est un sujet qui traverse les membranes et les fichiers (« eau » : un texte \
sur H2O, un autre sur l'eau dans les jeux vidéo) ; un nœud y appartient avec un poids de 0 à 1, sans \
rapport avec ses autres domaines ; un domaine posé sur une membrane vaut pour tout ce qu'elle contient \
— pose-le sur la membrane d'un article, pas sur chacun de ses paragraphes. Quand tu ne sais pas ce que tu cherches, commence par \
glucose_domaines : c'est la table des matières de tout ce que Glucose connaît ; quand tu sais les \
mots, glucose_chercher les trouve dans tout le document, comme Ctrl+F pour l'utilisateur. Deux domaines ne portent \
jamais le même nom : si « python » existe et que tu parles d'autre chose, précise le mot (« python \
serpent »). Le sens d'un tableau ne tient pas dans les positions : écris ce qui compte en membrane, \
en flèche ou en domaine.";

/// Chaque outil : son nom, sa description, le schéma de ses arguments.
const OUTILS: &[(&str, &str, &str)] = &[
    (
        "glucose_lire",
        "Lit le tableau que l'utilisateur regarde, par niveaux : par défaut son sommaire — chaque \
         nœud (id, sorte, boîte, membrane, bouts et relation d'une flèche, adresse d'une photo), \
         la première phrase de son texte (debut), sa taille et son empreinte (qui change quand \
         le texte change), sans le texte entier. « ids » rend le texte entier de ces nœuds ; \
         « cherche » rend la phrase (passage) de chaque nœud qui contient ces mots ; « dans » ne \
         garde qu'une membrane ; « domaine » ne garde que les nœuds qui le portent ; niveau \
         « complet » rend tous les textes ; niveau « plan » rend seulement les membranes et ce que \
         chacune contient — la carte d'un grand tableau, d'où descendre ensuite par « dans ». Chaque nœud dit ses domaines et leur poids. Rend aussi ce \
         que l'utilisateur a sélectionné (selection), la partie visible à l'écran, la liste des \
         tableaux et les domaines de ce document (domaines).",
        r##"{"type":"object","properties":{"tableau":{"type":"string","description":"identifiant d'un tableau ; par défaut celui qu'il regarde"},"dans":{"type":"string","description":"identifiant d'une membrane"},"ids":{"type":"array","items":{"type":"string"},"description":"les nœuds à lire en entier"},"cherche":{"type":"string","description":"des mots à trouver dans les textes"},"domaine":{"type":"string","description":"le nom d'un domaine de ce document"},"niveau":{"type":"string","enum":["sommaire","complet","plan"]}}}"##,
    ),
    (
        "glucose_poser_texte",
        "Pose un texte (Markdown, LaTeX entre $…$) ou un post-it — qui peut être un opérateur \
         (ET, OU, MAIS, PARCE QUE) —, ou met à jour celui qui porte déjà cet identifiant : seul \
         ce que l'appel nomme change. Rend son identifiant et sa boîte.",
        r##"{"type":"object","properties":{"id":{"type":"string"},"texte":{"type":"string"},"post_it":{"type":"boolean"},"operateur":{"type":"string","enum":["et","ou","mais","parce-que","aucun"],"description":"un post-it devenu opérateur logique entre deux idées (une pilule ET, OU, MAIS, PARCE QUE) ; aucun le rend papier"},"dans":{"type":"string","description":"identifiant de la membrane où le poser ; elle grandit pour le contenir"},"x":{"type":"number"},"y":{"type":"number"},"largeur":{"type":"number"},"couleur":{"type":"string","description":"#rrggbb : le papier d'un post-it, la teinte d'un texte — celle de la barre de couleur de l'humain ; glucose_lire la dit"},"domaines":{"type":"array","description":"les domaines de ce nœud, posés dans le même geste : [{\"domaine\":\"eau\",\"poids\":0.8}] s'il existe, [{\"nouveau\":\"eau chaude\"}] pour le créer","items":{}},"tableau":{"type":"string"}},"required":["texte"]}"##,
    ),
    (
        "glucose_poser_membrane",
        "Pose une membrane — une bulle qui regroupe des nœuds, avec un titre — ou met à jour celle \
         qui porte cet identifiant : seul ce que l'appel nomme change.",
        r##"{"type":"object","properties":{"id":{"type":"string"},"titre":{"type":"string"},"dans":{"type":"string"},"x":{"type":"number"},"y":{"type":"number"},"largeur":{"type":"number"},"hauteur":{"type":"number"},"couleur":{"type":"string","description":"#rrggbb"},"domaines":{"type":"array","description":"les domaines de ce nœud, posés dans le même geste : [{\"domaine\":\"eau\",\"poids\":0.8}] s'il existe, [{\"nouveau\":\"eau chaude\"}] pour le créer","items":{}},"tableau":{"type":"string"}}}"##,
    ),
    (
        "glucose_ecrire",
        "Écrit le texte d'un nœud : le remplace, ou l'allonge (ajouter : true) — pour un document \
         qu'on remplit au fil du travail. Les flèches ancrées à une phrase la suivent.",
        r##"{"type":"object","properties":{"id":{"type":"string"},"texte":{"type":"string"},"ajouter":{"type":"boolean"},"tableau":{"type":"string"}},"required":["id","texte"]}"##,
    ),
    (
        "glucose_relier",
        "Tend une flèche d'un nœud à un autre. passage_source et passage_cible, s'ils sont donnés, \
         sont des phrases exactes des deux textes : la flèche en part et y arrive, et elles sont \
         surlignées. relation : est_precurseur, contredit, herite_de, inspire, depend_de, illustre.",
        r##"{"type":"object","properties":{"id":{"type":"string"},"source":{"type":"string"},"cible":{"type":"string"},"passage_source":{"type":"string"},"passage_cible":{"type":"string"},"relation":{"type":"string","enum":["est_precurseur","contredit","herite_de","inspire","depend_de","illustre"]},"texte":{"type":"string","description":"ce que la flèche dit, écrit sur elle"},"tableau":{"type":"string"}},"required":["source","cible"]}"##,
    ),
    (
        "glucose_poser_page",
        "Pose une page web : sa capture d'écran en miniature (un fichier image local), qui ouvre \
         la page dans le navigateur au double-clic, et sous elle un texte avec son titre et son \
         adresse, où écrire ensuite les citations qu'on y a lues. Rend l'identifiant de la \
         photo et celui du texte.",
        r##"{"type":"object","properties":{"id":{"type":"string"},"adresse":{"type":"string","description":"http:// ou https://"},"titre":{"type":"string"},"capture":{"type":"string","description":"chemin d'un fichier PNG, JPEG ou WebP"},"texte":{"type":"string","description":"ce qui s'écrit sous le titre — les citations lues"},"dans":{"type":"string"},"x":{"type":"number"},"y":{"type":"number"},"largeur":{"type":"number"},"domaines":{"type":"array","description":"les domaines de ce nœud, posés dans le même geste : [{\"domaine\":\"eau\",\"poids\":0.8}] s'il existe, [{\"nouveau\":\"eau chaude\"}] pour le créer","items":{}},"tableau":{"type":"string"}},"required":["adresse"]}"##,
    ),
    (
        "glucose_deplacer",
        "Déplace un nœud : le coin haut gauche de sa boîte va en (x, y). Il appartient ensuite à la \
         membrane où il tombe.",
        r##"{"type":"object","properties":{"id":{"type":"string"},"x":{"type":"number"},"y":{"type":"number"},"tableau":{"type":"string"}},"required":["id","x","y"]}"##,
    ),
    (
        "glucose_ouvrir_tableau",
        "Rend l'identifiant de l'onglet qui porte ce nom, et le crée s'il n'existe pas — un \
         tableau à toi (un carnet, des archives), où poser ensuite avec « tableau ». L'onglet que \
         l'utilisateur regarde ne change pas. Le même nom rend toujours le même onglet.",
        r##"{"type":"object","properties":{"nom":{"type":"string"}},"required":["nom"]}"##,
    ),
    (
        "glucose_poser_domaine",
        "Pose un domaine sur des nœuds, en un geste. « domaine » prend un domaine qui existe, ici ou \
         dans un autre fichier ; « nouveau » en crée un. Un nom déjà pris est refusé et la réponse dit \
         ce qu'il désigne : même sujet, redonne-le dans « domaine » ; sinon, précise le mot. poids : de \
         0 à 1, sans rapport avec ses autres domaines ; 0 le retire. « renommer » change son nom ; \
         « meme_sujet » : true réunit deux domaines créés séparément sous le même nom.",
        r##"{"type":"object","properties":{"domaine":{"type":"string"},"nouveau":{"type":"string"},"ids":{"type":"array","items":{"type":"string"}},"poids":{"type":"number"},"couleur":{"type":"string","description":"#rrggbb"},"renommer":{"type":"string"},"meme_sujet":{"type":"boolean"},"tableau":{"type":"string"}}}"##,
    ),
    (
        "glucose_domaines",
        "La table des matières de tout ce que Glucose connaît : chaque domaine, à travers tous les \
         fichiers déjà ouverts et leurs voisins, combien de nœuds le portent et où. Rien ne s'ouvre à \
         l'écran. « domaine » rend ses nœuds, du plus lourd au plus léger, dans tous les fichiers : \
         leur première phrase et leur taille ; « entier » : true leur texte ; « cherche » la phrase qui \
         contient ces mots ; herite_de dit la membrane dont un nœud tient le domaine sans le porter. a_preciser dit qu'un nom désigne plusieurs sujets : demande lequel. \
         a_jour : false dit qu'une lecture du disque est en route : rappelle pour la suite. fichier \
         est le chemin d'un fichier ; fichier_ouvert celui que l'utilisateur regarde.",
        r##"{"type":"object","properties":{"domaine":{"type":"string"},"entier":{"type":"boolean"},"cherche":{"type":"string"}}}"##,
    ),
    (
        "glucose_chercher",
        "Cherche dans tout le document, comme Ctrl+F pour l'utilisateur : les mêmes résultats, dans \
         le même ordre, la même page de dix. Sans majuscules ni accents ; chaque mot doit être dans \
         le texte, ceux qui commencent un mot passent devant. Cherche dans les textes, les post-it, \
         les titres de membranes, ce que disent les flèches, les étiquettes et l'adresse des photos, \
         les noms des tableaux et des dossiers. Rend total, et pour chaque résultat son tableau, son \
         onglet, son id, sa sorte et la phrase trouvée ; « depuis » donne la page suivante. Lis \
         ensuite un nœud en entier par glucose_lire (tableau, ids).",
        r##"{"type":"object","properties":{"cherche":{"type":"string","description":"les mots à trouver"},"depuis":{"type":"number","description":"le rang du premier résultat rendu ; 0 par défaut"},"combien":{"type":"number","description":"le nombre de résultats rendus ; dix par défaut, comme la palette"}},"required":["cherche"]}"##,
    ),
    (
        "glucose_grouper",
        "Entoure ces nœuds d'une membrane neuve — une région —, née dans la plus proche membrane \
         qui les contient tous ; ce qu'une membrane nommée contient la suit sans changer de place. \
         Le geste « Grouper dans une membrane » (Ctrl+G) de l'humain, en un geste qu'il défait par \
         Ctrl+Z. Rend l'identifiant de la membrane et sa boîte.",
        r##"{"type":"object","properties":{"ids":{"type":"array","items":{"type":"string"},"description":"les nœuds à entourer"},"titre":{"type":"string"},"couleur":{"type":"string","description":"#rrggbb"},"tableau":{"type":"string"}},"required":["ids"]}"##,
    ),
    (
        "glucose_disposer",
        "Range en colonnes de journal ce que porte une membrane, ses sous-membranes comprises : \
         dans l'ordre de lecture (de haut en bas, puis de gauche à droite), chaque membrane prenant \
         la proportion de l'écran ; son coin haut-gauche ne bouge pas, chaque nœud reste dans sa \
         membrane. Un seul geste, que l'utilisateur défait par Ctrl+Z. À faire après avoir posé un \
         long contenu (un article, un rapport) : posé, il s'empile en une seule colonne. Rend le \
         nombre de nœuds rangés et la boîte de la membrane.",
        r##"{"type":"object","properties":{"dans":{"type":"string","description":"la membrane à ranger"},"proportion":{"type":"number","description":"largeur sur hauteur visée ; par défaut celle de l'écran"},"tableau":{"type":"string"}},"required":["dans"]}"##,
    ),
    (
        "glucose_retirer",
        "Retire des nœuds, et les flèches qui n'ont plus de bout.",
        r##"{"type":"object","properties":{"ids":{"type":"array","items":{"type":"string"}},"tableau":{"type":"string"}},"required":["ids"]}"##,
    ),
];

/// Les noms des outils, dans l'ordre de la liste.
pub fn noms() -> impl Iterator<Item = &'static str> {
    OUTILS.iter().map(|(nom, _, _)| *nom)
}

/// Ce que fait cet outil, tel qu'un agent le lit — et `glucose aide` (fiche 90).
pub fn description(nom: &str) -> Option<&'static str> {
    OUTILS
        .iter()
        .find(|(n, _, _)| *n == nom)
        .map(|(_, d, _)| *d)
}

/// La forme des arguments de cet outil — ce que lit `tools/list`, et la ligne de commande (fiche 90).
pub fn schema(nom: &str) -> Option<Json> {
    OUTILS
        .iter()
        .find(|(n, _, _)| *n == nom)
        .and_then(|(_, _, s)| Json::lire(s).ok())
}

/// Ce que `tools/list` rend.
pub fn liste() -> Json {
    let outils = OUTILS
        .iter()
        .map(|(nom, description, _)| {
            Json::objet([
                ("name", Json::from(*nom)),
                ("description", Json::from(*description)),
                // Écrits à la main ci-dessus et éprouvés : ils se lisent toujours.
                ("inputSchema", schema(nom).unwrap_or(Json::Nul)),
            ])
        })
        .collect();
    Json::objet([("tools", Json::Liste(outils))])
}
