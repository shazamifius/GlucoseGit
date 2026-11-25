//! **Ce qu'un dépôt dit**, en une phrase pour tout le lot.

/// Ce qu'un lot déposé a donné, en une phrase.
///
/// Pure, donc testable sans fenêtre : la formulation d'un message est exactement le genre
/// de chose qui se casse sans qu'aucun test ne le voie.
pub(super) fn compte_rendu(poses: usize, rates: usize) -> String {
    let pluriel = if poses > 1 { "s" } else { "" };
    match (poses, rates) {
        (0, 0) => String::new(),
        (0, 1) => "Le fichier n'a pas pu être posé".to_string(),
        (0, n) => format!("Aucun des {n} fichiers n'a pu être posé"),
        (p, 0) => format!("{p} élément{pluriel} posé{pluriel}"),
        (p, n) => format!(
            "{p} élément{pluriel} posé{pluriel}, {n} illisible{}",
            if n > 1 { "s" } else { "" }
        ),
    }
}

/// **Ce que dit un dépôt qui s'est replié** (DEPOT-WEB-6) : un lien posé à la place d'une image
/// est un échec, et le dire « posé » le faisait passer pour une réussite. Le message nomme le
/// site et la raison, et le geste qui rattrape.
pub(super) fn avec_l_echec(rendu: String, echec: Option<&str>, des_liens: bool) -> String {
    match echec {
        None => rendu,
        Some(raison) if des_liens => format!(
            "Image introuvable ({raison}) : lien posé à sa place — clic droit, « Remplacer par l'image »"
        ),
        Some(raison) => format!("{rendu} — l'image entière est introuvable ({raison})"),
    }
}
