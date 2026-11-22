//! Les épreuves du tiroir DISCUSSION (fiche 129) : couper, éclaircir, cliquer — une mesure
//! d'essai (six pixels par caractère) à la place de la typographie.

use super::paint::visible;
use super::*;

fn six(t: &str) -> f32 {
    6.0 * t.chars().count() as f32
}

#[test]
fn test_un_texte_se_coupe_entre_les_mots_et_garde_ses_sauts() {
    let lignes = couper_le_texte("un deux trois quatre\ncinq", &six, 6.0 * 9.0);
    assert_eq!(lignes, ["un deux", "trois", "quatre", "cinq"]);
    // Chaque ligne tient.
    assert!(lignes.iter().all(|l| six(l) <= 54.0));
    // Une ligne blanche voulue reste une ligne vide : seule la coupe d'un mot n'en fait jamais.
    assert_eq!(couper_le_texte("a\n\nb", &six, 54.0), ["a", "", "b"]);
}

/// Un mot plus long que la ligne se coupe, sans boucler même si un seul caractère ne tient pas.
#[test]
fn test_un_mot_trop_long_se_coupe_et_la_coupe_avance_toujours() {
    let lignes = couper_le_texte("abcdéfghij", &six, 24.0);
    assert_eq!(lignes, ["abcd", "éfgh", "ij"]);
    let etroit = couper_le_texte("abc", &six, 1.0);
    assert_eq!(etroit, ["a", "b", "c"]);
}

#[test]
fn test_le_markdown_d_une_reponse_se_lit_en_clair() {
    assert_eq!(
        en_clair("## Titre\n* **gras** et `code`\n  - sous-point"),
        "Titre\n• gras et code\n  • sous-point"
    );
    assert_eq!(en_clair("3 * 4"), "3 * 4");
}

fn ui_de(n: usize) -> DiscussionUi {
    let ligne = Ligne {
        texte: "x".into(),
        genre: Genre::Ia,
    };
    DiscussionUi {
        lignes: vec![ligne; n],
        ..DiscussionUi::default()
    }
}

/// **Une seule géométrie** : la hauteur est celle de la mise en page, et chaque ligne de plus
/// l'allonge d'une ligne, à toute échelle.
#[test]
fn test_la_hauteur_suit_les_lignes_a_toute_echelle() {
    for s in [0.5, 1.0, 1.25, 3.0] {
        let h0 = hauteur_naturelle(&ui_de(0), LARGEUR * s, s);
        let h4 = hauteur_naturelle(&ui_de(4), LARGEUR * s, s);
        assert!((h4 - h0 - 4.0 * LIGNE * s).abs() < 1e-3, "{s}");
    }
}

#[test]
fn test_le_bouton_envoie_ou_arrete_et_le_champ_prend_le_clavier() {
    let mut ui = ui_de(3);
    let cadre = ScaledRect {
        x: 100.0,
        y: 50.0,
        w: LARGEUR,
        h: 400.0,
        scale: 1.0,
    };
    let l = layout_discussion_panel(cadre, &ui);
    let au = |r: WidgetRect| Pointer {
        x: r.x + r.w / 2.0,
        y: r.y + r.h / 2.0,
    };
    assert_eq!(
        click_discussion_panel(&ui, &l, au(l.bouton)),
        Some(DiscussionIntent::Envoyer)
    );
    assert_eq!(
        click_discussion_panel(&ui, &l, au(l.champ)),
        Some(DiscussionIntent::Saisir)
    );
    assert_eq!(click_discussion_panel(&ui, &l, au(l.fil)), None);
    ui.en_cours = Some("Gemini".into());
    assert_eq!(
        click_discussion_panel(&ui, &l, au(l.bouton)),
        Some(DiscussionIntent::Arreter)
    );
    // Le champ et le bouton ne se touchent pas, et le fil finit au-dessus d'eux.
    assert!(l.champ.x + l.champ.w < l.bouton.x);
    assert!(l.fil.y + l.fil.h < l.champ.y);
}

/// **Ce qui se voit du champ** : le curseur reste toujours visible, et rien ne déborde.
#[test]
fn test_le_champ_montre_le_curseur_et_ne_deborde_pas() {
    let tout = "abcdefghijklmnop";
    for curseur in 0..=tout.len() {
        let (avant, apres) = visible(&tout[..curseur], tout, &six, 30.0);
        assert!(tout[..curseur].ends_with(avant), "{curseur}");
        assert!(tout[curseur..].starts_with(apres), "{curseur}");
        assert!(six(avant) + six(apres) <= 30.0, "{curseur}");
    }
    assert_eq!(
        visible("abcdefghij", "abcdefghij", &six, 30.0),
        ("fghij", "")
    );
    assert_eq!(visible("ab", "abcdefghij", &six, 30.0), ("ab", "cde"));
}
