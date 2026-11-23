//! OPACITE-1 — la couleur d'un dossier porte son opacité (fiche 133).

use super::*;

#[test]
fn opacite_1_un_dossier_ne_de_rien_garde_six_chiffres() {
    let f = CanvasFolder::new("d", "Dossier", "b");
    assert_eq!(f.teinte(), COULEUR_DE_NAISSANCE);
    assert_eq!(f.opacite(), OPACITE_DE_NAISSANCE);
    // À l'opacité de la naissance, la couleur reste ce que toutes les versions lisent.
    assert_eq!(
        couleur_de_dossier("#2a8cf9", OPACITE_DE_NAISSANCE),
        "#2a8cf9"
    );
}

#[test]
fn opacite_1_huit_chiffres_se_separent_en_teinte_et_opacite() {
    let mut f = CanvasFolder::new("d", "Dossier", "b");
    f.color = couleur_de_dossier("#2a8cf9", 0x80);
    assert_eq!(f.color, "#2a8cf980");
    assert_eq!(f.teinte(), "#2a8cf9");
    assert_eq!(f.opacite(), 0x80);
    // Changer de teinte garde l'opacité, et l'inverse — chacune par sa fonction.
    assert_eq!(couleur_de_dossier(&f.color, 0xff), "#2a8cf9ff");
    assert_eq!(couleur_de_dossier("#00ff00", f.opacite()), "#00ff0080");
    // Une opacité nulle est une opacité choisie : elle s'écrit.
    assert_eq!(couleur_de_dossier("#00ff00", 0), "#00ff0000");
}

#[test]
fn opacite_1_ce_qui_n_est_pas_un_rrggbb_ne_recoit_rien() {
    // Un `#rgb` (Glucose Tauri), un texte faux : aucun chiffre collé qu'on ne saurait séparer.
    assert_eq!(couleur_de_dossier("#abc", 0x80), "#abc");
    assert_eq!(couleur_de_dossier("bleu", 0x80), "bleu");
    assert_eq!(teinte_de("#2a8cf9zz"), "#2a8cf9zz");
    assert_eq!(opacite_de("#2a8cf9zz"), OPACITE_DE_NAISSANCE);
    assert_eq!(opacite_de("#abcd"), OPACITE_DE_NAISSANCE);
}
