//! Ce que la mémoire des mises en page doit garantir (P8 bis, fiche 73 § 7).

use super::*;

fn outils() -> (Typography, MathRenderer) {
    (Typography::new(), MathRenderer::new())
}

/// **Une mise en page retenue est celle qu'on aurait calculée** — et, demandée deux fois, la
/// même : aucun calcul de plus.
#[test]
fn test_une_mise_en_page_retenue_est_celle_qu_on_aurait_calculee() {
    let (typo, math) = outils();
    let memoire = MisesEnPage::default();
    let texte = "# Un titre\n\nUn texte **gras**, une formule $a^2$, et la suite.";
    let a = memoire.de((&typo, &math), texte, 240.0, TextMode::Rendered);
    let b = memoire.de((&typo, &math), texte, 240.0, TextMode::Rendered);
    assert!(Rc::ptr_eq(&a, &b), "demandée deux fois, elle est refaite");
    assert_eq!(
        *a,
        card_text_layout(&typo, &math, texte, 240.0, TextMode::Rendered)
    );
    assert_eq!(memoire.combien(), 1);
}

/// **Le texte, la largeur et le mode font la clé** : changer l'un des trois donne la mise en
/// page de ce qu'on demande, jamais celle d'à côté.
#[test]
fn test_le_texte_la_largeur_et_le_mode_font_la_cle() {
    let (typo, math) = outils();
    let memoire = MisesEnPage::default();
    let texte = "Un **texte** assez long pour se couper en plusieurs lignes, ou pas.";
    let demandes = [
        (texte, 240.0, TextMode::Rendered),
        (texte, 120.0, TextMode::Rendered),
        (texte, 240.0, TextMode::Source),
        (
            "Un **texte** assez long pour se couper en plusieurs lignes, ou non.",
            240.0,
            TextMode::Rendered,
        ),
    ];
    for (t, l, m) in demandes {
        memoire.de((&typo, &math), t, l, m);
    }
    for (t, l, m) in demandes {
        let retenue = memoire.de((&typo, &math), t, l, m);
        assert_eq!(
            *retenue,
            card_text_layout(&typo, &math, t, l, m),
            "{l} {m:?}"
        );
    }
    assert_eq!(memoire.combien(), demandes.len());
}

/// **Ce que l'image d'avant n'a pas demandé s'oublie** ; ce qu'elle a demandé passe à la
/// suivante — et y reste tant qu'on le demande.
#[test]
fn test_ce_que_l_image_d_avant_n_a_pas_demande_s_oublie() {
    let (typo, math) = outils();
    let memoire = MisesEnPage::default();
    let de = |t: &str| memoire.de((&typo, &math), t, 240.0, TextMode::Rendered);
    let a = de("a");
    de("b");
    memoire.nouvelle_image();
    // Les deux ont servi à l'image d'avant : elles sont là.
    assert_eq!(memoire.combien(), 2);
    assert!(
        Rc::ptr_eq(&a, &de("a")),
        "a s'est perdu d'une image à l'autre"
    );
    memoire.nouvelle_image();
    // Seul « a » a servi : « b » s'oublie.
    assert_eq!(memoire.combien(), 1);
    assert!(Rc::ptr_eq(&a, &de("a")));
    memoire.nouvelle_image();
    memoire.nouvelle_image();
    assert_eq!(memoire.combien(), 0, "rien n'a servi : rien ne reste");
}

/// **Le rendu oublie ce qui a quitté l'écran** — par de vraies images : un nœud dessiné, puis un
/// tableau où il n'est plus ; deux images plus tard, sa mise en page est partie. Sans
/// `nouvelle_image` à chaque image, la mémoire grossirait de tout ce qu'on a jamais vu.
#[test]
fn test_le_rendu_oublie_ce_qui_a_quitte_l_ecran() {
    use glucose_core::store::Store;
    use glucose_core::types::Annotation;
    let tableau = |texte: &str| {
        let mut store = Store::new("memoire");
        let board = store.project.active_board_id.clone();
        store.add_annotation(&board, Annotation::text("c", 20.0, 20.0, texte));
        store.clear_selection();
        store
    };
    let (avant, apres) = (tableau("le premier texte"), tableau("le second texte"));
    let mut renderer = crate::renderer::Renderer::new();
    let mut ui = crate::ui::UiState::new();
    crate::bench::render_frame(&mut renderer, &mut ui, &avant, 400, 300);
    assert!(
        renderer.mises_en_page.retient("le premier texte"),
        "l'épreuve doit voir la mémoire servir"
    );
    for _ in 0..2 {
        crate::bench::render_frame(&mut renderer, &mut ui, &apres, 400, 300);
    }
    assert!(renderer.mises_en_page.retient("le second texte"));
    assert!(
        !renderer.mises_en_page.retient("le premier texte"),
        "ce qui a quitté l'écran reste en mémoire"
    );
}
