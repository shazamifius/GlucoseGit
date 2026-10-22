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
phrase, sa taille) ; lis ensuite en entier seulement les nœuds dont tu as besoin (ids), ou \
cherche une phrase dans tous (cherche). « selection » dit les nœuds que l'utilisateur a \
sélectionnés : quand il parle de « ça », c'est eux. L'empreinte d'un nœud change quand son texte \
change : garde-la, et à la lecture suivante, relis seulement ceux dont l'empreinte a changé — \
l'utilisateur les a récrits. Les positions sont en unités du canevas, l'origine en haut \
à gauche, y vers le bas. Si tu ne donnes pas de position, le nœud se pose sous le contenu de sa \
membrane (dans), ou à droite de tout le tableau : range ton travail dans des membranes. Donne \
tes propres identifiants (« proteo-rapport ») : poser deux fois le même met à jour au lieu de \
dupliquer. Une flèche peut partir d'une phrase exacte d'un texte et arriver à une phrase exacte \
d'un autre (passage_source, passage_cible) : ces phrases sont surlignées, et doivent s'y trouver \
mot pour mot.";

/// Chaque outil : son nom, sa description, le schéma de ses arguments.
const OUTILS: &[(&str, &str, &str)] = &[
    (
        "glucose_lire",
        "Lit le tableau que l'utilisateur regarde, par niveaux : par défaut son sommaire — chaque \
         nœud (id, sorte, boîte, membrane, bouts et relation d'une flèche, adresse d'une photo), \
         la première phrase de son texte (debut), sa taille et son empreinte (qui change quand \
         le texte change), sans le texte entier. « ids » rend le texte entier de ces nœuds ; \
         « cherche » rend la phrase (passage) de chaque nœud qui contient ces mots ; « dans » ne \
         garde qu'une membrane ; niveau « complet » rend tous les textes. Rend aussi ce que \
         l'utilisateur a sélectionné (selection), la partie visible à l'écran et la liste des \
         tableaux.",
        r##"{"type":"object","properties":{"tableau":{"type":"string","description":"identifiant d'un tableau ; par défaut celui qu'il regarde"},"dans":{"type":"string","description":"identifiant d'une membrane"},"ids":{"type":"array","items":{"type":"string"},"description":"les nœuds à lire en entier"},"cherche":{"type":"string","description":"des mots à trouver dans les textes"},"niveau":{"type":"string","enum":["sommaire","complet"]}}}"##,
    ),
    (
        "glucose_poser_texte",
        "Pose un texte (Markdown, LaTeX entre $…$) ou un post-it, ou met à jour celui qui porte \
         déjà cet identifiant. Rend son identifiant et sa boîte.",
        r##"{"type":"object","properties":{"id":{"type":"string"},"texte":{"type":"string"},"post_it":{"type":"boolean"},"dans":{"type":"string","description":"identifiant de la membrane où le poser ; elle grandit pour le contenir"},"x":{"type":"number"},"y":{"type":"number"},"largeur":{"type":"number"},"couleur":{"type":"string","description":"#rrggbb"},"tableau":{"type":"string"}},"required":["texte"]}"##,
    ),
    (
        "glucose_poser_membrane",
        "Pose une membrane — une bulle qui regroupe des nœuds, avec un titre — ou met à jour celle \
         qui porte cet identifiant.",
        r##"{"type":"object","properties":{"id":{"type":"string"},"titre":{"type":"string"},"dans":{"type":"string"},"x":{"type":"number"},"y":{"type":"number"},"largeur":{"type":"number"},"hauteur":{"type":"number"},"couleur":{"type":"string","description":"#rrggbb"},"tableau":{"type":"string"}}}"##,
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
        r##"{"type":"object","properties":{"id":{"type":"string"},"adresse":{"type":"string","description":"http:// ou https://"},"titre":{"type":"string"},"capture":{"type":"string","description":"chemin d'un fichier PNG, JPEG ou WebP"},"texte":{"type":"string","description":"ce qui s'écrit sous le titre — les citations lues"},"dans":{"type":"string"},"x":{"type":"number"},"y":{"type":"number"},"largeur":{"type":"number"},"tableau":{"type":"string"}},"required":["adresse"]}"##,
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
        "glucose_retirer",
        "Retire des nœuds, et les flèches qui n'ont plus de bout.",
        r##"{"type":"object","properties":{"ids":{"type":"array","items":{"type":"string"}},"tableau":{"type":"string"}},"required":["ids"]}"##,
    ),
];

/// Les noms des outils, dans l'ordre de la liste.
pub fn noms() -> impl Iterator<Item = &'static str> {
    OUTILS.iter().map(|(nom, _, _)| *nom)
}

/// Ce que `tools/list` rend.
pub fn liste() -> Json {
    let outils = OUTILS
        .iter()
        .map(|(nom, description, schema)| {
            Json::objet([
                ("name", Json::from(*nom)),
                ("description", Json::from(*description)),
                // Écrits à la main ci-dessus et éprouvés : ils se lisent toujours.
                ("inputSchema", Json::lire(schema).unwrap_or(Json::Nul)),
            ])
        })
        .collect();
    Json::objet([("tools", Json::Liste(outils))])
}
