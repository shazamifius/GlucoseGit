//! FORMULE-1 : chaque règle de la feuille de KaTeX que le pont applique, vérifiée par ce
//! qu'elle change. Les nombres attendus viennent des métriques et de la feuille de KaTeX, pas
//! du pont : `0.12` est `.nulldelimiter`, `0.5` l'`arraycolsep` de KaTeX.

use crate::layout::{layout, layout_avec, MathItem};
use crate::{Family, Mode};

const E: f64 = 1e-9;

/// La largeur d'un caractère selon les métriques de KaTeX, en `em`.
fn largeur(c: char, fonte: &str) -> f64 {
    let ctx = katex::KatexContext::default();
    katex::get_character_metrics(&ctx, c, fonte, katex::symbols::Mode::Math)
        .expect("métriques lisibles")
        .expect("caractère connu")
        .width
}

fn glyphes(items: &[MathItem]) -> Vec<(String, f64, f64, f64, Family)> {
    items
        .iter()
        .filter_map(|i| match i {
            MathItem::Glyph {
                text,
                x,
                y,
                size,
                family,
                ..
            } => Some((text.clone(), *x, *y, *size, *family)),
            _ => None,
        })
        .collect()
}

fn filets(items: &[MathItem]) -> Vec<(f64, f64, f64, f64)> {
    items
        .iter()
        .filter_map(|i| match i {
            MathItem::Rule {
                x,
                y,
                width,
                height,
            } => Some((*x, *y, *width, *height)),
            _ => None,
        })
        .collect()
}

fn x_de(items: &[MathItem], texte: &str) -> f64 {
    glyphes(items)
        .into_iter()
        .find(|g| g.0 == texte)
        .unwrap_or_else(|| panic!("pas de {texte}"))
        .1
}

/// **Chaque lettre d'un mot avance de sa propre largeur.** KaTeX fusionne « or » en un symbole
/// et n'en garde que la largeur du « o » : le pont posait le « d » de « fjord » sur le « r ».
#[test]
fn test_chaque_lettre_fusionnee_avance_de_sa_largeur() {
    let f = layout("fjord", Mode::Display).expect("valide");
    let (xo, xr, xd) = (
        x_de(&f.items, "o"),
        x_de(&f.items, "r"),
        x_de(&f.items, "d"),
    );
    assert!(
        (xr - xo - largeur('o', "Math-Italic")).abs() < E,
        "le r suit le o de la largeur du o : {}",
        xr - xo
    );
    // Le « r » porte une correction d'italique : elle s'ajoute après lui, pas avant.
    assert!(
        xd - xr >= largeur('r', "Math-Italic") - E,
        "le d ne mord pas sur le r : {}",
        xd - xr
    );
}

/// **La correction d'italique d'un symbole est une marge** : l'exposant d'une intégrale se pose
/// après elle, l'indice recule d'autant. Sans elle, les deux bornes se posaient sur le signe.
#[test]
fn test_les_bornes_d_une_integrale_s_ecartent_du_signe() {
    let f = layout(r"\int_0^\infty", Mode::Display).expect("valide");
    let signe = largeur('∫', "Size2-Regular");
    let (x_haut, x_bas) = (x_de(&f.items, "∞"), x_de(&f.items, "0"));
    assert!(
        (x_haut - 1.0).abs() < 1e-4,
        "l'exposant après la correction (0,556 + 0,444) : {x_haut}"
    );
    assert!(
        (x_bas - signe).abs() < 1e-4,
        "l'indice au pied du signe : {x_bas}"
    );
}

/// **Celui qui dessine mesure** : une avance donnée par la fonte remplace celle des
/// métriques — `∬` est large de 1,084 em dans sa fonte, de 0,556 dans les métriques de KaTeX,
/// et l'indice de `\iint_D` se posait sur le signe.
#[test]
fn test_l_avance_de_la_fonte_prime_sur_les_metriques() {
    let par_katex = layout(r"\iint_D", Mode::Display).expect("valide");
    let fonte = |c: char, _: Family, _: crate::Style| (c == '∬').then_some(1.084);
    let par_la_fonte = layout_avec(r"\iint_D", Mode::Display, &fonte).expect("valide");
    let ecart = x_de(&par_la_fonte.items, "D") - x_de(&par_katex.items, "D");
    assert!(
        (ecart - (1.084 - largeur('∬', "Size2-Regular"))).abs() < 1e-9,
        "l'indice suit le signe réel : décalé de {ecart}"
    );
    assert!(
        par_la_fonte.width > par_katex.width,
        "et la formule s'élargit"
    );
}

/// **Une fraction centre ses deux étages sur sa barre**, et la barre s'arrête aux vides de
/// 0,12 em qui bordent la fraction.
#[test]
fn test_une_fraction_centre_ses_etages_sur_sa_barre() {
    let f = layout(r"\frac{a}{bbb}", Mode::Display).expect("valide");
    let barre = filets(&f.items)[0];
    let milieu = barre.0 + barre.2 / 2.0;
    let haut = x_de(&f.items, "a") + largeur('a', "Math-Italic") / 2.0;
    let b: Vec<f64> = glyphes(&f.items)
        .iter()
        .filter(|g| g.0 == "b")
        .map(|g| g.1)
        .collect();
    let bas = (b[0] + b[2] + largeur('b', "Math-Italic")) / 2.0;
    assert!(
        (haut - milieu).abs() < E,
        "le numérateur au milieu : {haut} contre {milieu}"
    );
    assert!(
        (bas - milieu).abs() < E,
        "le dénominateur au milieu : {bas} contre {milieu}"
    );
    assert!(
        (barre.0 - 0.12).abs() < E,
        "la barre après le vide : {}",
        barre.0
    );
    assert!(
        (f.width - (barre.0 + barre.2) - 0.12).abs() < E,
        "et le vide de droite"
    );
}

/// **Les colonnes d'une matrice se séparent** de deux fois `arraycolsep` (0,5 em) : la largeur
/// que KaTeX écrit dans le style. Sans elle, « a b » sortait « ab ».
#[test]
fn test_les_colonnes_d_une_matrice_se_separent() {
    let f = layout(r"\begin{pmatrix}a&b\end{pmatrix}", Mode::Display).expect("valide");
    let blanc = x_de(&f.items, "b") - (x_de(&f.items, "a") + largeur('a', "Math-Italic"));
    assert!(
        (blanc - 1.0).abs() < E,
        "un em entre les colonnes : {blanc}"
    );
}

/// **Chaque classe de fonte de la feuille**, et chacune ne fixe que ce qu'elle dit : `\mathbb`
/// et `\mathfrak` sortaient en lettres droites ordinaires.
#[test]
fn test_les_familles_de_la_feuille() {
    let f = layout(
        r"\mathbb{R}\mathfrak{g}\mathscr{F}\mathbf{v}",
        Mode::Display,
    )
    .expect("valide");
    let familles: Vec<(String, Family)> =
        glyphes(&f.items).into_iter().map(|g| (g.0, g.4)).collect();
    assert_eq!(
        familles,
        vec![
            ("R".into(), Family::Ams),
            ("g".into(), Family::Fraktur),
            ("F".into(), Family::Script),
            ("v".into(), Family::Main),
        ]
    );
    let gras = f.items.iter().any(|i| {
        matches!(i, MathItem::Glyph { text, style, .. } if text == "v" && style.bold && !style.italic)
    });
    assert!(gras, "\\mathbf est gras et droit");
}

/// **Un cadre est quatre filets qui ne se recouvrent pas** : le haut et le bas de bord à bord,
/// les côtés entre eux.
#[test]
fn test_un_cadre_ferme_sans_recouvrement() {
    let f = layout(r"\boxed{x}", Mode::Display).expect("valide");
    let mut c = filets(&f.items);
    assert_eq!(c.len(), 4, "quatre côtés : {c:?}");
    c.sort_by(|a, b| a.1.total_cmp(&b.1).then(a.0.total_cmp(&b.0)));
    let (bas, gauche, droite, haut) = (c[0], c[1], c[2], c[3]);
    assert!((bas.2 - f.width).abs() < E && (haut.2 - f.width).abs() < E);
    assert!(
        (gauche.1 - (bas.1 + bas.3)).abs() < E,
        "le côté part du haut du bas"
    );
    assert!(
        (gauche.1 + gauche.3 - haut.1).abs() < E,
        "et finit au bas du haut"
    );
    assert!(
        (droite.0 + droite.2 - f.width).abs() < E,
        "le côté droit ferme"
    );
    assert!(gauche.0.abs() < E && (gauche.2 - 0.04).abs() < 1e-3);
}

/// `\rule` n'est que bordure ; `bottom` le lève.
#[test]
fn test_un_trait_rule_et_sa_levee() {
    let f = layout(r"\rule{1em}{0.5em}\rule[0.5ex]{1em}{1pt}", Mode::Display).expect("valide");
    let c = filets(&f.items);
    assert_eq!(c.len(), 2, "un filet chacun, pas de double : {c:?}");
    assert!((c[0].2 - 1.0).abs() < E && (c[0].3 - 0.5).abs() < E);
    assert!(c[0].1.abs() < E, "posé sur la ligne de base");
    assert!(c[1].1 > 0.2, "levé d'un demi-ex : {}", c[1].1);
    assert!((f.width - 2.0).abs() < E, "chacun avance de sa largeur");
}

/// Le filet vertical d'un tableau : aussi haut que le tableau, centré sur la frontière des
/// colonnes, et sans largeur propre (`margin: 0 -t/2`).
#[test]
fn test_le_filet_vertical_d_un_tableau() {
    let f = layout(r"\begin{array}{c|c}a&b\end{array}", Mode::Display).expect("valide");
    let c = filets(&f.items);
    assert_eq!(c.len(), 1, "{c:?}");
    let (xa, xb) = (x_de(&f.items, "a"), x_de(&f.items, "b"));
    let frontiere = xa + largeur('a', "Math-Italic") + 0.5;
    assert!(
        (c[0].0 + c[0].2 / 2.0 - frontiere).abs() < E,
        "centré sur la frontière"
    );
    assert!(
        (xb - frontiere - 0.5).abs() < E,
        "sans rien prendre à la colonne suivante"
    );
    assert!(c[0].3 > 1.0, "de toute la hauteur : {}", c[0].3);
}

/// Les ratures de `\cancel` : un trait, pas un remplissage, de toute la boîte.
#[test]
fn test_la_rature_est_un_trait() {
    let f = layout(r"\cancel{x}", Mode::Display).expect("valide");
    let rature = f.items.iter().find_map(|i| match i {
        MathItem::Path { width, forme, .. } => Some((*width, forme.clone())),
        _ => None,
    });
    let (w, forme) = rature.expect("une rature");
    let forme = forme.expect("lisible");
    assert!((forme.epaisseur.expect("un trait") - 0.046).abs() < E);
    assert!(
        (w - largeur('x', "Math-Italic")).abs() < E,
        "de toute la largeur : {w}"
    );
}

/// **Le chapeau d'une lettre penchée se décale** de deux fois son inclinaison : l'étage de
/// l'accent porte `margin-left` et `width: calc(100% − …)`, et c'est dans cette boîte que la
/// forme se pose.
#[test]
fn test_l_accent_large_suit_l_inclinaison() {
    let f = layout(r"\widehat{A}", Mode::Display).expect("valide");
    let ctx = katex::KatexContext::default();
    let penche = katex::get_character_metrics(&ctx, 'A', "Math-Italic", katex::symbols::Mode::Math)
        .expect("lisible")
        .expect("connu")
        .skew;
    let (x, w) = f
        .items
        .iter()
        .find_map(|i| match i {
            MathItem::Path { x, width, .. } => Some((*x, *width)),
            _ => None,
        })
        .expect("un chapeau");
    assert!(
        (x - 2.0 * penche).abs() < 1e-4,
        "décalé de 2 × {penche} : {x}"
    );
    assert!((x + w - f.width).abs() < 1e-4, "jusqu'au bord droit");
}

/// **`\phantom` réserve sa place sans se montrer** — rien de ce qu'il contient ne se dessine,
/// ni glyphe ni barre, et ce qui suit se pose comme s'il était là.
#[test]
fn test_un_fantome_occupe_sa_place_sans_se_dessiner() {
    let visible = layout(r"x\frac{y}{y}z", Mode::Display).expect("valide");
    let fantome = layout(r"x\phantom{\frac{y}{y}}z", Mode::Display).expect("valide");
    assert!(
        (x_de(&fantome.items, "z") - x_de(&visible.items, "z")).abs() < E,
        "le z à la même place"
    );
    assert_eq!(
        glyphes(&fantome.items).len(),
        2,
        "seuls x et z se dessinent"
    );
    assert!(filets(&fantome.items).is_empty(), "ni la barre");
}

/// **L'indice d'une racine se pose dans l'ouverture du radical** : la feuille l'écarte de
/// 0,2778 em à gauche et le fait reculer de 0,5556 em à droite (`.sqrt > .root`) — le radical
/// commence donc **avant** lui, à 0,2778 + largeur − 0,5556.
#[test]
fn test_l_indice_d_une_racine_recule_dans_le_radical() {
    let f = layout(r"\sqrt[3]{x}", Mode::Display).expect("valide");
    let indice = glyphes(&f.items)
        .into_iter()
        .find(|g| g.0 == "3")
        .expect("un indice");
    let radical = f
        .items
        .iter()
        .find_map(|i| match i {
            MathItem::Path { x, .. } => Some(*x),
            _ => None,
        })
        .expect("un radical");
    let (gauche, recul) = (0.277_777_777_8, 0.555_555_555_6);
    assert!(
        (indice.1 - gauche).abs() < E,
        "l'indice écarté : {}",
        indice.1
    );
    let attendu = gauche + largeur('3', "Main-Regular") * indice.3 - recul;
    assert!(
        (radical - attendu).abs() < E,
        "le radical sous l'indice : {radical} contre {attendu}"
    );
}

/// **Aucune forme de KaTeX ne reste sans tracé**, et chacune a une boîte : racines de toutes
/// tailles, grands délimiteurs empilés (dont les morceaux ont une largeur écrite), flèches,
/// accolades, chapeaux.
#[test]
fn test_toutes_les_formes_ont_un_trace_et_une_boite() {
    for source in [
        r"\sqrt{x}",
        r"\sqrt{\frac{a}{b}}",
        r"\sqrt{\frac{\frac{a}{b}}{\frac{c}{d}}}",
        r"\sqrt[3]{x}",
        r"\left(\begin{array}{c}1\\2\\3\\4\\5\end{array}\right)",
        r"\left\{\begin{array}{c}1\\2\\3\\4\\5\end{array}\right.",
        // Une barre haute est un seul SVG, de largeur écrite : sans elle, il n'aurait aucune
        // largeur — aucun étage voisin ne la lui prêterait.
        r"\left|\begin{array}{c}1\\2\\3\\4\\5\end{array}\right\|",
        r"\overrightarrow{AB}\xleftarrow[g]{h}",
        r"\overbrace{a+b}^n\underbrace{c+d}_m",
        r"\widehat{xyz}\widetilde{abc}\vec{v}",
    ] {
        let f = layout(source, Mode::Display).expect(source);
        let mut formes = 0;
        for item in &f.items {
            if let MathItem::Path {
                name,
                width,
                height,
                forme,
                ..
            } = item
            {
                formes += 1;
                assert!(forme.is_some(), "{source} : {name} sans tracé");
                assert!(
                    *width > 0.0 && *height > 0.0,
                    "{source} : {name} sans boîte"
                );
            }
        }
        assert!(formes > 0, "{source} dessine une forme");
    }
}
