//! Ce qu'un fichier déposé devient : classement, troncature, mise en Markdown.

use super::*;

/// La table est triée et sans doublon — sinon la dichotomie rate des extensions au hasard.
///
/// Ce test ne vérifie pas une optimisation : il vérifie qu'une faute de frappe ou un
/// doublon glissé dans les soixante-trois lignes de la table ne peut pas passer. Une
/// recherche linéaire n'aurait pas eu ce filet.
#[test]
fn test_the_table_is_sorted_and_has_no_duplicate() {
    for pair in READABLE.windows(2) {
        assert!(
            pair[0].0 < pair[1].0,
            "la table n'est pas triée : {:?} vient avant {:?}",
            pair[0].0,
            pair[1].0
        );
    }
}

/// L'extension se lit sans se soucier de la casse, du chemin, ni du séparateur.
#[test]
fn test_the_extension_ignores_case_path_and_separator() {
    assert_eq!(extension("notes.MD"), "md");
    assert_eq!(extension("C:\\dossier\\module.RS"), "rs");
    assert_eq!(extension("/home/u/projet/main.rs"), "rs");
    assert_eq!(extension("Makefile"), "", "aucun point : aucune extension");
    assert_eq!(extension("archive.tar.gz"), "gz", "le dernier point gagne");
}

/// Un nom qui n'est **qu'**une extension se lit quand même : `.gitignore` est du texte.
///
/// C'est le cas que « la partie après le dernier point » gère naturellement, et qu'une
/// règle « il faut un nom avant le point » aurait manqué.
#[test]
fn test_a_dotfile_is_read_by_what_follows_its_dot() {
    assert_eq!(extension(".gitignore"), "gitignore");
    assert_eq!(readable(".gitignore"), Some(Readable::Fenced("text")));
    assert_eq!(readable(".gitattributes"), Some(Readable::Fenced("text")));
}

/// Du Markdown se rend tel quel ; le reste se pose dans un bloc marqué de son langage.
#[test]
fn test_markdown_stays_raw_and_the_rest_is_fenced() {
    assert_eq!(readable("notes.md"), Some(Readable::Markdown));
    assert_eq!(readable("README.markdown"), Some(Readable::Markdown));
    assert_eq!(readable("main.rs"), Some(Readable::Fenced("rust")));
    assert_eq!(readable("script.ps1"), Some(Readable::Fenced("powershell")));
    assert_eq!(readable("en-tete.h"), Some(Readable::Fenced("c")));
}

/// Ce qui n'est pas du texte n'est pas classé : le décodeur tranchera, pas une liste.
#[test]
fn test_what_is_not_text_is_left_to_the_decoder() {
    for nom in [
        "photo.png",
        "film.mkv",
        "archive.zip",
        "scene.blend",
        "sans",
    ] {
        assert_eq!(readable(nom), None, "{nom} n'a rien à faire dans la table");
    }
}

/// La troncature coupe sur un caractère entier, jamais au milieu d'un accent.
///
/// C'est le cas qui arrive dès qu'un journal français dépasse la limite : trancher au
/// milieu des deux octets d'un « é » rendrait la chaîne invalide, et `&text[..limit]`
/// paniquerait.
#[test]
fn test_truncation_never_splits_a_character() {
    let texte = "aéiou"; // « é » occupe deux octets : les frontières sont 0,1,3,4,5,6
    assert_eq!(truncate_on_char_boundary(texte, 6), "aéiou");
    assert_eq!(truncate_on_char_boundary(texte, 100), "aéiou");
    assert_eq!(
        truncate_on_char_boundary(texte, 2),
        "a",
        "recule d'un octet"
    );
    assert_eq!(truncate_on_char_boundary(texte, 3), "aé");
    assert_eq!(truncate_on_char_boundary("é", 1), "");
    assert_eq!(truncate_on_char_boundary("", 0), "");
}

/// Le Markdown porte le nom du fichier en titre et le contenu dans son bloc.
#[test]
fn test_the_markdown_carries_the_name_and_the_body() {
    let rendu = as_markdown("main.rs", "fn main() {}", false);
    assert_eq!(rendu, "### main.rs\n\n```rust\nfn main() {}\n```");

    let brut = as_markdown("notes.md", "# Titre\n\ndu texte", false);
    assert_eq!(
        brut, "### notes.md\n\n# Titre\n\ndu texte",
        "un Markdown ne s'enrobe pas dans un bloc de code"
    );
}

/// Une carte tronquée le dit : elle ne se fait jamais passer pour le fichier entier.
#[test]
fn test_a_truncated_card_says_so() {
    let rendu = as_markdown("journal.log", "des lignes", true);
    assert!(
        rendu.contains("tronqué"),
        "rien ne signale la coupure : {rendu}"
    );
    assert!(rendu.contains(&INLINE_MAX_BYTES.to_string()));
    assert!(
        !as_markdown("journal.log", "des lignes", false).contains("tronqué"),
        "un fichier entier ne doit pas s'annoncer tronqué"
    );
}

/// Un fichier inconnu passé à `as_markdown` garde son contenu brut plutôt que de le perdre.
///
/// Le cas n'arrive pas par le glisser-déposer — qui ne l'appelle que sur du lisible — mais
/// une fonction qui perdrait silencieusement son entrée serait un piège pour le prochain
/// appelant.
#[test]
fn test_an_unknown_file_keeps_its_content() {
    let rendu = as_markdown("chose.inconnue", "du contenu", false);
    assert!(rendu.contains("du contenu"));
}
