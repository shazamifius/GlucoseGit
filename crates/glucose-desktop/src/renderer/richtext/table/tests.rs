//! TABLE-3 — un tableau tient dans la largeur de son nœud (fiche 123).

use super::repartir;
use crate::renderer::card::text_box;
use crate::renderer::math::MathRenderer;
use crate::renderer::richtext::hit::{line_of_offset, offset_to_x};
use crate::renderer::richtext::{font_of, layout_rich_text, TextBox, TextLayout, TextMode, NO_TAB};
use crate::typography::{Face, Typography};
use glucose_core::text::SpanRole;
use std::collections::BTreeMap;

/// **Ses deux tableaux du 19/11**, tels que Claude.ai les a posés par la porte dans « Textes
/// optimisés » — ceux qui sortaient de leur nœud de 900 sur sa capture.
const SES_TABLEAUX: [&str; 2] = [
    "| | Comportement |\n|---|---|\n| 🟠 **Aujourd'hui** | La Time Machine (bouton à gauche) s'ouvre à droite ; Domaines (bouton à droite) s'ouvre à gauche. **Les deux se croisent.** Ce sont des bulles à coins arrondis avec une ombre (Domaines en haut à gauche, Time Machine en haut à droite). |\n| 🟢 **Avec la loi** | Chacun s'ouvre **du côté de son bouton**. Prototype : Time Machine collée au bord gauche, Domaines collé au bord droit, plats, sans ombre. |",
    "| # | Choix | Pourquoi / Conséquence |\n|---|---|---|\n| 1️⃣ | **F11** devient le plein écran ; le **mode zen** passe à **Maj+F11** | Le zen était à F11 depuis Tauri, mais F11 est le plein écran de tous les logiciels de bureau. Les raccourcis se changent dans Glucose si tu veux l'inverse. |\n| 2️⃣ | **À 100 %, les icônes ont des traits d'un pixel** | Plus nettes qu'avant, mais plus légères. Trop fines sur ton écran ? Je les passe à deux pixels. |\n| 3️⃣ | **Aucune correction de contraste du texte** | Le texte clair sur fond sombre paraît un peu plus maigre ; Chrome et Edge le renforcent sous Windows. J'ai jugé que ce n'était pas la cause du « pixelisé ». Dis-moi si le texte te paraît trop fin dans la beta.3. |",
];

fn pose(source: &str, bx: TextBox, mode: TextMode) -> TextLayout {
    layout_rich_text(&Typography::new(), &MathRenderer::new(), source, bx, mode)
}

fn boite(usable: f32) -> TextBox {
    TextBox {
        usable,
        ..text_box(900.0)
    }
}

/// Le bord droit de chaque fragment, depuis la marge : là où le dessin le fait finir.
fn bords_droits(typo: &Typography, layout: &TextLayout, source: &str) -> Vec<(f32, String)> {
    let mut out = Vec::new();
    for line in &layout.lines {
        let corps = font_of(line.kind, 14.0);
        for f in layout.fragments_of(line) {
            let x = offset_to_x(typo, layout, line, source, f.end, corps);
            out.push((x, source[f.start..f.end].to_string()));
        }
    }
    out
}

/// Pour chaque octet dessiné, la ligne qui le porte — et combien de fois il l'est.
fn lignes_des_octets(layout: &TextLayout) -> BTreeMap<usize, Vec<usize>> {
    let mut out: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
    for (rang, line) in layout.lines.iter().enumerate() {
        for f in layout.fragments_of(line) {
            for at in f.start..f.end {
                out.entry(at).or_default().push(rang);
            }
        }
    }
    out
}

/// **TABLE-3 — ses tableaux tiennent dans leur nœud**, à sa largeur (900, donc 852 offerts au
/// texte) et en dessous, jusqu'à 60 unités où même les mots se coupent : aucun fragment ne
/// finit au-delà de la place, et chaque lettre d'une cellule est dessinée une fois et une
/// seule. Preuve à l'envers : à largeur naturelle (l'ancienne loi), ils dépassaient 852.
#[test]
fn test_table_3_ses_tableaux_tiennent_dans_leur_noeud() {
    let typo = Typography::new();
    for source in SES_TABLEAUX {
        let naturel = pose(source, boite(1.0e6), TextMode::Rendered);
        let large = naturel.lines.iter().map(|l| l.largeur).fold(0.0, f32::max);
        assert!(large > 852.0, "l'ancienne loi tenait déjà : {large}");

        for usable in [852.0, 600.0, 400.0, 200.0, 60.0] {
            let layout = pose(source, boite(usable), TextMode::Rendered);
            let mut fautes = Vec::new();
            for (x, morceau) in bords_droits(&typo, &layout, source) {
                if x > usable + 1e-3 {
                    fautes.push(format!("{morceau:?} finit à {x}"));
                }
            }
            for line in &layout.lines {
                if line.largeur > usable + 1e-3 {
                    fautes.push(format!("une ligne grise de {} de large", line.largeur));
                }
            }
            let octets = lignes_des_octets(&layout);
            for (rang, rangee) in source.lines().enumerate() {
                let debut = source
                    .lines()
                    .take(rang)
                    .map(|l| l.len() + 1)
                    .sum::<usize>();
                for cellule in glucose_core::text::block::cells(rangee) {
                    if rangee.contains("---") {
                        break;
                    }
                    let texte = &rangee[cellule.clone()];
                    for (i, ch) in texte.char_indices() {
                        let n = octets.get(&(debut + cellule.start + i)).map_or(0, Vec::len);
                        let signe = ch == '*' || ch == ' ';
                        if n > 1 || (n == 0 && !signe) {
                            fautes.push(format!("{ch:?} de {texte:?} dessiné {n} fois"));
                        }
                    }
                }
            }
            assert!(fautes.is_empty(), "à {usable} : {fautes:#?}");
        }
    }
}

/// **TABLE-3 — tant que les mots tiennent, aucun ne se coupe** : chaque mot d'une cellule
/// est sur une seule ligne. À 852 et à 400, ses tableaux sont entre leurs largeurs minimales
/// et naturelles (la combinaison linéaire) — on le vérifie avant de juger.
#[test]
fn test_table_3_aucun_mot_coupe_tant_que_les_mots_tiennent() {
    for source in SES_TABLEAUX {
        for usable in [852.0, 400.0] {
            let layout = pose(source, boite(usable), TextMode::Rendered);
            let hauteur_naturelle = pose(source, boite(1.0e6), TextMode::Rendered).line_count();
            assert!(
                layout.line_count() > hauteur_naturelle,
                "à {usable}, rien ne s'est coupé : le cas n'est pas exercé"
            );
            let octets = lignes_des_octets(&layout);
            let mut debut = 0;
            for rangee in source.lines() {
                for mot in rangee.split(' ') {
                    let at = debut + (mot.as_ptr() as usize - rangee.as_ptr() as usize);
                    let lignes: std::collections::BTreeSet<usize> = (at..at + mot.len())
                        .filter_map(|o| octets.get(&o))
                        .flatten()
                        .copied()
                        .collect();
                    assert!(
                        lignes.len() <= 1,
                        "à {usable}, {mot:?} est coupé : {lignes:?}"
                    );
                }
                debut += rangee.len() + 1;
            }
        }
    }
}

/// **TABLE-1 tient toujours** : coupées en lignes, les colonnes restent en face — un tableau
/// de trois colonnes n'a que trois taquets, d'une rangée à l'autre et d'une ligne à l'autre.
#[test]
fn test_table_3_les_colonnes_restent_en_face() {
    let layout = pose(SES_TABLEAUX[1], boite(400.0), TextMode::Rendered);
    let mut taquets: Vec<f32> = layout
        .fragments
        .iter()
        .filter(|f| f.tab != NO_TAB)
        .map(|f| f.tab)
        .collect();
    taquets.sort_by(f32::total_cmp);
    taquets.dedup_by(|a, b| (*a - *b).abs() < 1e-6);
    assert_eq!(
        taquets.len(),
        3,
        "des colonnes qui ne sont plus en face : {taquets:?}"
    );
}

/// **TABLE-3 — la répartition**, sur des nombres : la naturelle quand la place suffit ; entre
/// les deux, la combinaison linéaire qui remplit la place, chaque colonne entre sa minimale et
/// sa naturelle ; en deçà, les minimales en proportion. Continue aux deux frontières.
#[test]
fn test_table_3_la_repartition_remplit_la_place() {
    let (minimales, naturelles) = ([10.0f32, 50.0, 20.0], [100.0f32, 60.0, 400.0]);
    let somme = |v: &[f32]| v.iter().sum::<f32>();
    assert_eq!(
        repartir(&minimales, &naturelles, 1000.0),
        naturelles.to_vec()
    );
    for place in [80.0f32, 81.0, 200.0, 559.0] {
        let l = repartir(&minimales, &naturelles, place);
        assert!((somme(&l) - place).abs() < 1e-3, "{place} : {l:?}");
        for ((largeur, minimale), naturelle) in l.iter().zip(&minimales).zip(&naturelles) {
            assert!(
                *largeur >= minimale - 1e-4 && *largeur <= naturelle + 1e-4,
                "{l:?}"
            );
        }
    }
    let sous = repartir(&minimales, &naturelles, 40.0);
    assert!((somme(&sous) - 40.0).abs() < 1e-4, "{sous:?}");
    assert!(
        (sous[1] / sous[0] - 5.0).abs() < 1e-4,
        "pas en proportion : {sous:?}"
    );
    let pres = |a: &[f32], b: &[f32]| a.iter().zip(b).all(|(x, y)| (x - y).abs() < 1e-2);
    assert!(pres(&repartir(&minimales, &naturelles, 80.0), &minimales));
    assert!(pres(&repartir(&minimales, &naturelles, 79.999), &minimales));
    assert!(pres(
        &repartir(&minimales, &naturelles, 559.999),
        &naturelles
    ));
}

/// **L'en-tête se mesure comme il se dessine : en gras.** Mesuré en romain, une colonne dont
/// l'en-tête est la plus large cellule était plus étroite que lui, et son gras mordait sur la
/// gouttière de la suivante.
#[test]
fn test_table_3_l_en_tete_se_mesure_en_gras() {
    let typo = Typography::new();
    let source = "| WWWWWWWW | b |\n|---|---|\n| a | c |";
    let layout = pose(source, boite(852.0), TextMode::Rendered);
    let entete = layout.fragments_of(&layout.lines[0]);
    let seconde = entete
        .iter()
        .find(|f| f.tab > 0.0)
        .expect("la seconde colonne");
    let (gras, _) = typo.measure_text("WWWWWWWW", 14.0, Face::Bold);
    let gouttiere = typo.advance(' ', 14.0, Face::Regular) * 2.0;
    assert!(
        seconde.tab * 14.0 >= gras + gouttiere - 1e-3,
        "la seconde colonne à {}, l'en-tête gras finit à {gras}",
        seconde.tab * 14.0
    );
}

/// **En édition, une rangée est sa source** (MODE-1) : ses barres reviennent, en signes gris,
/// elle se coupe comme un paragraphe sans taquet, ses lignes se suivent sans trou — le curseur
/// trouve chaque octet sur la ligne qui le porte — et rien ne déborde.
#[test]
fn test_table_3_en_edition_une_rangee_est_sa_source() {
    let typo = Typography::new();
    let source = SES_TABLEAUX[0];
    let layout = pose(source, boite(400.0), TextMode::Source);
    assert!(
        layout.fragments.iter().all(|f| f.tab == NO_TAB),
        "un taquet en édition"
    );
    let barres: Vec<_> = layout
        .fragments
        .iter()
        .filter(|f| source[f.start..f.end].contains('|'))
        .collect();
    assert!(!barres.is_empty(), "les barres ne reviennent pas");
    assert!(
        barres.iter().all(|f| f.role == SpanRole::Marker),
        "une barre n'est pas un signe"
    );
    for at in 0..=source.len() {
        let l = &layout.lines[line_of_offset(&layout, at)];
        let separation = l.kind == glucose_core::text::BlockKind::TableRule;
        assert!(
            separation || (l.start <= at && at <= l.end),
            "l'octet {at} tombe sur la ligne {}..{}",
            l.start,
            l.end
        );
    }
    for (x, morceau) in bords_droits(&typo, &layout, source) {
        assert!(x <= 400.0 + 1e-3, "{morceau:?} finit à {x}");
    }
}
