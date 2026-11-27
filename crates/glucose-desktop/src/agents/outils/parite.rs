//! **Les deux voies : chaque outil des IA, avec son geste en face** (sa note 19 du 24/11, dans
//! `relais-bugs-liste` ; fiche 150). Ce qu'une IA fait par un outil, sa main le fait aussi — par
//! une commande de Glucose (son identifiant, `commandes.rs` : sa touche, la palette), par un geste
//! qu'aucune commande ne nomme, ou pas encore : ce qui manque se dit. Un outil neuf sans sa ligne
//! ne passe pas ; une commande renommée sans la table non plus.

use crate::commandes::Commande;

/// Ce que sa main fait, en face d'un outil.
enum Main {
    /// Ces commandes (leurs identifiants), et ce que la main y ajoute.
    Par(&'static [&'static str], &'static str),
    /// Pas encore à sa main : ce qui manque.
    Manque(&'static str),
}

const EN_FACE: &[(&str, Main)] = &[
    (
        "glucose_lire",
        Main::Par(&["tout-cadrer"], "regarder le tableau"),
    ),
    (
        "glucose_poser_texte",
        Main::Par(&["outil.texte", "outil.post-it"], "un clic, puis écrire"),
    ),
    (
        "glucose_poser_membrane",
        Main::Par(&["outil.membrane"], "tracer la bulle"),
    ),
    (
        "glucose_ecrire",
        Main::Par(&[], "double-clic sur le nœud, puis écrire"),
    ),
    (
        "glucose_relier",
        Main::Par(
            &["outil.fleche"],
            "glisser d'un nœud à l'autre ; d'une phrase à une autre : « Éditer le texte lié »",
        ),
    ),
    (
        "glucose_poser_page",
        Main::Manque("coller ou déposer une adresse ne pose pas sa page : seule une IA la capture"),
    ),
    (
        "glucose_deplacer",
        Main::Par(&["outil.selection", "pousser.droite"], "glisser le nœud"),
    ),
    (
        "glucose_ouvrir_tableau",
        Main::Par(&["nouvel-onglet"], "un clic sur son onglet"),
    ),
    (
        "glucose_repondre",
        Main::Par(
            &["outil.texte", "outil.fleche"],
            "deux gestes : son texte à côté, puis la flèche ; en un seul, IA seulement",
        ),
    ),
    (
        "glucose_renommer",
        Main::Par(&["renommer-le-dossier"], "un onglet : double-clic dessus"),
    ),
    (
        "glucose_ranger",
        Main::Par(&[], "glisser les nœuds choisis sur le dossier"),
    ),
    (
        "glucose_poser_domaine",
        Main::Par(
            &[],
            "le bouton Domaines : les nœuds choisis, puis le domaine",
        ),
    ),
    ("glucose_domaines", Main::Par(&[], "le bouton Domaines")),
    ("glucose_chercher", Main::Par(&["chercher"], "")),
    ("glucose_grouper", Main::Par(&["grouper"], "")),
    (
        "glucose_disposer",
        Main::Par(&["disposer-en-colonnes"], "une membrane choisie"),
    ),
    ("glucose_retirer", Main::Par(&["supprimer"], "")),
    ("glucose_dater", Main::Par(&["dater"], "")),
    (
        "glucose_histoire",
        Main::Par(&["time-machine", "frise-chronologique"], ""),
    ),
    (
        "glucose_quoi_de_neuf",
        Main::Manque("la Time Machine montre l'histoire, sans marquer où il l'avait laissée"),
    ),
    (
        "glucose_voir",
        Main::Par(&["tout-cadrer"], "regarder la photo"),
    ),
    (
        "glucose_noter",
        Main::Par(
            &["time-machine"],
            "un jalon nomme un moment ; une note sur un geste, IA seulement",
        ),
    ),
    (
        "glucose_verifier",
        Main::Par(
            &["tout-cadrer"],
            "regarder le tableau : l'œil voit ce qui se chevauche",
        ),
    ),
];

/// **Chaque outil annoncé a sa ligne, une seule ; aucune ligne pour un outil qui n'existe pas.**
#[test]
fn parite_19_chaque_outil_a_son_geste_en_face() {
    let outils: Vec<&str> = super::noms().collect();
    for nom in &outils {
        let lignes = EN_FACE.iter().filter(|(o, _)| o == nom).count();
        assert_eq!(lignes, 1, "« {nom} » : une ligne en face, pas {lignes}");
    }
    for (outil, _) in EN_FACE {
        assert!(outils.contains(outil), "« {outil} » n'est plus un outil");
    }
}

/// **Chaque commande citée existe**, et ce qui manque dit ce qui manque.
#[test]
fn parite_19_chaque_commande_citee_existe() {
    for (outil, main) in EN_FACE {
        match main {
            Main::Par(commandes, geste) => {
                assert!(
                    !commandes.is_empty() || !geste.is_empty(),
                    "« {outil} » : ni commande ni geste"
                );
                for id in *commandes {
                    assert!(
                        Commande::de_l_id(id).is_some(),
                        "« {outil} » cite « {id} », qui n'est pas une commande"
                    );
                }
            }
            Main::Manque(quoi) => assert!(!quoi.is_empty(), "« {outil} » : que manque-t-il ?"),
        }
    }
}
