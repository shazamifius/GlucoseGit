//! Les lois de la mise en page mathématique. Aucune ne demande d'écran : une formule mise en
//! page est une liste de nombres, et c'est tout l'intérêt d'avoir séparé la géométrie du dessin.

use crate::layout::MathItem;
use crate::{layout, Family, MathError, Mode, Style};

/// Le premier glyphe dont le texte est `quoi`, s'il existe.
fn glyphe<'a>(items: &'a [MathItem], quoi: &str) -> Option<&'a MathItem> {
    items.iter().find(|i| matches!(i, MathItem::Glyph { text, .. } if text == quoi))
}

/// Les coordonnées, la taille et la fonte d'un glyphe nommé.
fn place(items: &[MathItem], quoi: &str) -> (f64, f64, f64, Family, Style) {
    match glyphe(items, quoi).unwrap_or_else(|| panic!("le glyphe {quoi:?} est absent")) {
        MathItem::Glyph { x, y, size, family, style, .. } => (*x, *y, *size, *family, *style),
        autre => unreachable!("{autre:?} n'est pas un glyphe"),
    }
}

/// **Une fraction empile vraiment.** Le numérateur est au-dessus de la ligne de base, le
/// dénominateur en dessous, et le filet entre les deux. C'est le cas qui échouait chez la
/// bibliothèque écartée au § 7.1 de la fiche 12.
#[test]
fn test_une_fraction_empile_numerateur_filet_denominateur() {
    let f = layout(r"\frac{a}{b}", Mode::Inline).expect("une fraction valide");
    let (_, ya, ta, _, _) = place(&f.items, "a");
    let (_, yb, tb, _, _) = place(&f.items, "b");

    assert!(ya > 0.0, "le numérateur monte : {ya}");
    assert!(yb < 0.0, "le dénominateur descend : {yb}");
    assert!(ta < 1.0 && tb < 1.0, "les deux se réduisent : {ta} et {tb}");

    let filet = f
        .items
        .iter()
        .find_map(|i| match i {
            MathItem::Rule { x, y, width, height } => Some((*x, *y, *width, *height)),
            _ => None,
        })
        .expect("une barre de fraction");
    assert!(filet.1 < ya && filet.1 > yb, "le filet est entre les deux : {}", filet.1);
    assert!(filet.2 > 0.0, "le filet a une longueur : {}", filet.2);
    assert!(filet.3 > 0.0, "et une épaisseur : {}", filet.3);
    assert!(
        (filet.2 - f.width).abs() < 1e-9,
        "le filet fait toute la largeur : {} contre {}",
        filet.2,
        f.width
    );
}

/// Un exposant monte et rapetisse ; un indice descend et rapetisse. Et les deux gardent la
/// **même** réduction : c'est le style « script » de TeX, pas deux réglages indépendants.
#[test]
fn test_l_exposant_monte_et_l_indice_descend() {
    let f = layout(r"x^2 + y_1", Mode::Inline).expect("valide");
    let (_, yx, tx, _, _) = place(&f.items, "x");
    let (_, y2, t2, _, _) = place(&f.items, "2");
    let (_, y1, t1, _, _) = place(&f.items, "1");

    assert_eq!(yx, 0.0, "le corps est sur la ligne de base");
    assert!(y2 > 0.0, "l'exposant monte : {y2}");
    assert!(y1 < 0.0, "l'indice descend : {y1}");
    assert!(t2 < tx && t1 < tx, "les deux rapetissent");
    assert!((t2 - t1).abs() < 1e-9, "et de la même façon : {t2} contre {t1}");
}

/// **Les bornes d'une somme sont centrées sur elle.**
///
/// Ce test a trouvé un vrai défaut le jour où il a été écrit : l'interprète posait chaque étage
/// d'un empilement contre son bord gauche, et les bornes se collaient au flanc du `∑` au lieu
/// d'être au milieu. La règle est dans le CSS de KaTeX (`text-align: center` sur `op-limits`) ;
/// elle n'était pas lue.
#[test]
fn test_les_bornes_d_une_somme_sont_centrees_sur_elle() {
    let f = layout(r"\sum_{i=1}^{n} k", Mode::Display).expect("valide");
    let (x_somme, _, _, famille, _) = place(&f.items, "∑");
    let (x_n, y_n, _, _, _) = place(&f.items, "n");
    let (x_i, y_i, _, _, _) = place(&f.items, "i");

    assert_eq!(famille, Family::Size2, "un grand opérateur en display vient de Size2");
    assert!(y_n > 0.0, "la borne haute est au-dessus : {y_n}");
    assert!(y_i < 0.0, "la borne basse est en dessous : {y_i}");
    assert!(
        x_n > x_somme && x_i > x_somme,
        "les deux bornes sont décalées vers l'intérieur : {x_n} et {x_i} contre {x_somme}"
    );
}

/// Une variable est en italique mathématique, un chiffre en romain — c'est la convention de
/// TeX, et elle se lit sur la famille de fonte, pas sur un réglage.
#[test]
fn test_les_variables_sont_en_italique_et_les_chiffres_en_romain() {
    let f = layout(r"2x", Mode::Inline).expect("valide");
    let (_, _, _, f_deux, s_deux) = place(&f.items, "2");
    let (_, _, _, f_x, s_x) = place(&f.items, "x");

    assert_eq!((f_deux, s_deux), (Family::Main, Style::ROMAN), "le chiffre");
    assert_eq!((f_x, s_x), (Family::Math, Style::ITALIC), "la variable");
}

/// Le mode `display` n'est pas une décoration : il change la mise en page. Une somme y prend
/// une fonte plus grande et met ses bornes au-dessus et en dessous, au lieu de les poser à côté.
#[test]
fn test_display_et_inline_ne_donnent_pas_la_meme_chose() {
    let inline = layout(r"\sum_{i=1}^{n} k", Mode::Inline).expect("valide");
    let display = layout(r"\sum_{i=1}^{n} k", Mode::Display).expect("valide");

    assert!(
        display.total_height() > inline.total_height(),
        "display monte plus haut : {} contre {}",
        display.total_height(),
        inline.total_height()
    );
    let (_, _, _, fam_inline, _) = place(&inline.items, "∑");
    let (_, _, _, fam_display, _) = place(&display.items, "∑");
    assert_eq!(fam_inline, Family::Size1, "en ligne, l'opérateur reste petit");
    assert_eq!(fam_display, Family::Size2, "isolé, il grandit");
}

/// Une racine porte son trait, et son indice est plus petit que son contenu.
#[test]
fn test_une_racine_a_son_trait() {
    let f = layout(r"\sqrt[3]{x}", Mode::Inline).expect("valide");
    let radical = f
        .items
        .iter()
        .find_map(|i| match i {
            MathItem::Path { name, width, height, .. } => Some((name.clone(), *width, *height)),
            _ => None,
        })
        .expect("le radical de la racine");
    assert!(radical.0.starts_with("sqrt"), "un chemin de radical : {}", radical.0);
    assert!(radical.1 > 0.0 && radical.2 > 0.0, "et une boîte : {} × {}", radical.1, radical.2);
    let (_, y3, t3, _, _) = place(&f.items, "3");
    let (_, _, tx, _, _) = place(&f.items, "x");
    assert!(t3 < tx, "l'indice de racine est plus petit : {t3} contre {tx}");
    assert!(y3 > 0.0, "et il est en haut à gauche : {y3}");
}

/// **Une matrice a plusieurs lignes.** C'est le cas qui a disqualifié la bibliothèque écartée,
/// qui rendait `(a bc d)` sur une seule ligne.
#[test]
fn test_une_matrice_a_vraiment_deux_lignes() {
    let f = layout(r"\begin{pmatrix} a & b \\ c & d \end{pmatrix}", Mode::Display)
        .expect("une matrice valide");

    let (_, ya, _, _, _) = place(&f.items, "a");
    let (_, yc, _, _, _) = place(&f.items, "c");
    assert!(ya > yc, "`a` est au-dessus de `c` : {ya} contre {yc}");

    let (xa, _, _, _, _) = place(&f.items, "a");
    let (xb, _, _, _, _) = place(&f.items, "b");
    assert!(xb > xa, "`b` est à droite de `a` : {xb} contre {xa}");

    // Et les deux lignes sont vraiment alignées en colonnes.
    let (xc, _, _, _, _) = place(&f.items, "c");
    let (xd, _, _, _, _) = place(&f.items, "d");
    assert!((xa - xc).abs() < 0.05, "colonne gauche alignée : {xa} contre {xc}");
    assert!((xb - xd).abs() < 0.05, "colonne droite alignée : {xb} contre {xd}");
}

/// Les lettres grecques et les symboles viennent de leurs fontes, sans repli silencieux.
#[test]
fn test_les_symboles_viennent_de_leurs_fontes() {
    let f = layout(r"\alpha + \infty", Mode::Inline).expect("valide");
    let (_, _, _, fam_alpha, style_alpha) = place(&f.items, "α");
    assert_eq!(fam_alpha, Family::Math, "une lettre grecque minuscule est une variable");
    assert!(style_alpha.italic);
    let (_, _, _, fam_infini, _) = place(&f.items, "∞");
    assert_eq!(fam_infini, Family::Main);
}

/// **Une formule fausse rend une erreur avec un message lisible**, et ne panique pas. C'est ce
/// message qui colorera une formule en rouge pendant la frappe.
#[test]
fn test_une_formule_fausse_rend_un_message_et_ne_panique_pas() {
    for mauvaise in [r"\frac{", r"\begin{pmatrix} a", r"\unecommandequinexistepas{x}"] {
        match layout(mauvaise, Mode::Inline) {
            Err(MathError::Parse(m)) => assert!(!m.is_empty(), "un message vide pour {mauvaise:?}"),
            Ok(_) => panic!("{mauvaise:?} aurait dû échouer"),
        }
    }
}

/// Une source vide est valide et ne produit rien — elle ne doit pas être une erreur : pendant
/// la frappe, `$$` existe une frappe avant `$x$`.
#[test]
fn test_une_source_vide_est_valide_et_vide() {
    let f = layout("", Mode::Inline).expect("le vide est valide");
    assert!(f.is_empty());
    assert!(f.ink_bounds().is_none());
}

/// **La largeur annoncée contient vraiment tout.** Un glyphe posé au-delà voudrait dire qu'une
/// carte le couperait — c'est l'autre défaut de la bibliothèque écartée.
#[test]
fn test_la_largeur_annoncee_contient_tout_ce_qui_est_pose() {
    for source in [
        r"\frac{a}{b}",
        r"x^2 + y_1",
        r"\sum_{i=1}^{n} k",
        r"\int_0^\infty e^{-x^2} dx",
        r"\begin{pmatrix} a & b \\ c & d \end{pmatrix}",
        r"\sqrt[3]{x^2 + y^2}",
    ] {
        let f = layout(source, Mode::Display).expect(source);
        let (x0, _, x1, _) = f.ink_bounds().expect("des éléments");
        assert!(x0 >= -1e-9, "{source} : un élément déborde à gauche ({x0})");
        assert!(
            x1 <= f.width + 1e-6,
            "{source} : un élément déborde à droite ({x1} contre une largeur de {})",
            f.width
        );
    }
}

/// La hauteur et la profondeur annoncées contiennent ce qui est posé.
#[test]
fn test_la_hauteur_annoncee_contient_ce_qui_est_pose() {
    for source in [r"\frac{a}{b}", r"\sum_{i=1}^{n} k", r"\int_0^\infty x\,dx"] {
        let f = layout(source, Mode::Display).expect(source);
        let (_, y0, _, y1) = f.ink_bounds().expect("des éléments");
        assert!(y1 <= f.height + 1e-6, "{source} : ça dépasse en haut ({y1} > {})", f.height);
        assert!(y0 >= -f.depth - 1e-6, "{source} : ça dépasse en bas ({y0} < {})", -f.depth);
    }
}

/// La mise en page est **reproductible** : même source, même résultat. Une formule sera mise en
/// cache par sa source, ce qui n'aurait aucun sens autrement.
#[test]
fn test_la_mise_en_page_est_reproductible() {
    let a = layout(r"\int_0^\infty e^{-x^2}\,dx = \frac{\sqrt{\pi}}{2}", Mode::Display).expect("valide");
    let b = layout(r"\int_0^\infty e^{-x^2}\,dx = \frac{\sqrt{\pi}}{2}", Mode::Display).expect("valide");
    assert_eq!(a, b);
}

/// Rien ne sort avec une coordonnée qui n'est pas un nombre : un `NaN` contaminerait la scène
/// entière, et se verrait comme un glyphe disparu plutôt que comme une erreur.
#[test]
fn test_aucune_coordonnee_n_est_un_non_nombre() {
    for source in [
        r"\frac{\frac{a}{b}}{\frac{c}{d}}",
        r"\left( \frac{\sum_{k=0}^{n} \binom{n}{k}}{\sqrt[4]{1+x^2}} \right)^{2}",
        r"\begin{pmatrix} \alpha & \beta \\ \gamma & \delta \end{pmatrix}",
    ] {
        let f = layout(source, Mode::Display).expect(source);
        assert!(f.width.is_finite() && f.height.is_finite() && f.depth.is_finite(), "{source}");
        for item in &f.items {
            assert!(item.x().is_finite(), "{source} : abscisse non finie");
            assert!(item.y().is_finite(), "{source} : ordonnée non finie");
            if let MathItem::Glyph { size, .. } = item {
                assert!(size.is_finite() && *size > 0.0, "{source} : taille {size}");
            }
        }
    }
}

/// Chaque famille sait nommer son fichier de fonte, et deux familles n'en nomment jamais un
/// seul — c'est ce qui garantit qu'un glyphe ne se dessine pas avec la mauvaise.
#[test]
fn test_chaque_famille_nomme_une_fonte_distincte() {
    let familles = [
        Family::Main,
        Family::Math,
        Family::Ams,
        Family::Caligraphic,
        Family::Fraktur,
        Family::SansSerif,
        Family::Script,
        Family::Typewriter,
        Family::Size1,
        Family::Size2,
        Family::Size3,
        Family::Size4,
    ];
    let noms: std::collections::HashSet<_> = familles.iter().map(|f| f.font_name()).collect();
    assert_eq!(noms.len(), familles.len(), "deux familles partagent un nom de fonte");
    for f in familles {
        assert!(f.font_name().starts_with("KaTeX_"), "{:?}", f.font_name());
    }
    assert_eq!(Style::ROMAN.suffix(), "Regular");
    assert_eq!(Style { bold: true, italic: true }.suffix(), "BoldItalic");
}
