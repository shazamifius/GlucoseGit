//! Les lois du document : les couleurs, et la cohérence des tables avec l'arène.

use super::*;

fn px(v: i32) -> Fx {
    Fx::from_px(v)
}

fn boite(x: i32, y: i32, w: i32, h: i32) -> Box2 {
    Box2::new(px(x), px(y), px(w), px(h))
}

/// Les quatre formes hexadécimales sont lues, et la forme canonique ressort.
#[test]
fn test_les_quatre_formes_hexadecimales_sont_lues() {
    for (ecrit, canonique) in [
        ("#abc", "#aabbcc"),
        ("#ABC", "#aabbcc"),
        ("#aabbcc", "#aabbcc"),
        ("#AABBCC", "#aabbcc"),
        ("#0d0d0d", "#0d0d0d"),
        ("#f5c542", "#f5c542"),
        ("#12345678", "#12345678"),
        ("#abcd", "#aabbccdd"),
    ] {
        let c = Rgba::parse(ecrit).unwrap_or_else(|| panic!("{ecrit} devrait être lisible"));
        assert_eq!(c.to_hex(), canonique, "{ecrit}");
        // Et la forme canonique se relit en la même couleur : la normalisation est stable.
        assert_eq!(Rgba::parse(&c.to_hex()), Some(c));
    }
}

/// Ce qui n'est pas un hexadécimal n'est pas lu — et c'est voulu : le pont conservera la
/// chaîne telle quelle plutôt que d'inventer une couleur.
#[test]
fn test_ce_qui_n_est_pas_hexadecimal_n_est_pas_lu() {
    for s in [
        "rebeccapurple",
        "rgb(1, 2, 3)",
        "",
        "#",
        "#ab",
        "#abcde",
        "#gggggg",
        "0d0d0d",
        "#0d0d0d ",
    ] {
        assert_eq!(Rgba::parse(s), None, "{s:?} ne devrait pas être lu");
    }
}

/// Une couleur reconnue coûte quatre octets ; une couleur illisible est conservée à la lettre.
/// Les deux tables s'excluent, et poser l'une retire l'autre.
#[test]
fn test_les_deux_formes_de_couleur_s_excluent() {
    let mut d = Doc::new();
    let id = d.spawn(Kind::Text, boite(0, 0, 10, 10), NodeId::NONE);

    d.set_color(id, "#f5c542");
    assert_eq!(d.color_of(id).as_deref(), Some("#f5c542"));
    assert!(d.color.has(id) && !d.color_literal.has(id));
    d.check().unwrap();

    d.set_color(id, "rebeccapurple");
    assert_eq!(d.color_of(id).as_deref(), Some("rebeccapurple"));
    assert!(!d.color.has(id) && d.color_literal.has(id));
    d.check().unwrap();

    d.set_color(id, "#abc");
    assert_eq!(d.color_of(id).as_deref(), Some("#aabbcc"));
    assert!(d.color.has(id) && !d.color_literal.has(id));
    d.check().unwrap();

    assert_eq!(std::mem::size_of::<Rgba>(), 4, "contre 24 pour un String, plus son tas");
}

/// La couleur de fond a sa propre paire de tables : sans cela, un fond illisible écraserait la
/// couleur de trait du même nœud.
#[test]
fn test_le_fond_et_le_trait_ne_se_marchent_pas_dessus() {
    let mut d = Doc::new();
    let id = d.spawn(Kind::Sticky, boite(0, 0, 10, 10), NodeId::NONE);
    d.set_color(id, "une couleur qui n'en est pas une");
    d.set_background(id, "un fond qui n'en est pas un");
    assert_eq!(d.color_of(id).as_deref(), Some("une couleur qui n'en est pas une"));
    assert_eq!(d.background_of(id).as_deref(), Some("un fond qui n'en est pas un"));
    d.check().unwrap();
}

/// Supprimer un nœud conserve ses attributs : annuler doit rendre le nœud entier, pas sa
/// carcasse.
#[test]
fn test_supprimer_conserve_les_attributs() {
    let mut d = Doc::new();
    let id = d.spawn(Kind::Text, boite(0, 0, 10, 10), NodeId::NONE);
    d.text.set(id, "à retrouver");
    d.set_color(id, "#f5c542");
    d.font_size.set(id, 14.0);

    assert!(d.kill(id));
    assert_eq!(d.len(), 0);
    // Les attributs sont toujours là, rattachés au même identifiant.
    assert_eq!(d.text.get(id), "à retrouver");
    assert_eq!(d.color_of(id).as_deref(), Some("#f5c542"));
    d.check().unwrap();

    assert!(d.nodes.revive(id));
    assert_eq!(d.len(), 1);
    assert_eq!(d.font_size.get(id), Some(&14.0));
    d.check().unwrap();
}

/// La vérification attrape une table qui désigne un nœud inexistant — la faute qui, sans elle,
/// ne se verrait qu'au premier clic.
#[test]
fn test_la_verification_attrape_une_table_qui_pointe_dans_le_vide() {
    let mut d = Doc::new();
    let a = d.spawn(Kind::Text, boite(0, 0, 10, 10), NodeId::NONE);
    let b = d.spawn(Kind::Text, boite(20, 0, 10, 10), NodeId::NONE);
    d.check().unwrap();

    let fantome = NodeId::from_index(999);
    d.font_size.set(fantome, 12.0);
    assert!(d.check().unwrap_err().contains("font_size"));
    d.font_size.remove(fantome);
    d.check().unwrap();

    d.mirror_of.set(a, fantome);
    assert!(d.check().unwrap_err().contains("miroir"));
    d.mirror_of.set(a, b);
    d.check().unwrap();

    let f = d.spawn(Kind::Arrow, boite(0, 0, 10, 10), NodeId::NONE);
    d.arrow.set(f, ArrowTrait { source: fantome, ..Default::default() });
    assert!(d.check().unwrap_err().contains("source"));
    d.arrow.set(f, ArrowTrait { source: a, target: fantome, ..Default::default() });
    assert!(d.check().unwrap_err().contains("cible"));
    d.arrow.set(f, ArrowTrait { source: a, target: b, ..Default::default() });
    d.check().unwrap();
}

/// Le document dit ce qu'il pèse, et ce poids suit le contenu.
#[test]
fn test_le_document_dit_ce_qu_il_pese() {
    let mut d = Doc::with_capacity(100);
    assert_eq!(d.bytes(), 0);
    for i in 0..100 {
        let id = d.spawn(Kind::Text, boite(i * 10, 0, 10, 10), NodeId::NONE);
        d.text.set(id, "dix chars.");
    }
    // Tronc 22 + intervalle 8 + dix octets de texte = 40 par nœud.
    assert_eq!(d.bytes(), 100 * 40);
}
