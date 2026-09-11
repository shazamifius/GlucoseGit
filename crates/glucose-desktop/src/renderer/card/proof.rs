//! Preuve visuelle et numérique de R-45 : la même carte, cinq zooms, un seul rapport.
//!
//! On ne mesure pas les nombres qui ont servi à poser le texte — ils sont justement ce que
//! R-45 met en doute — mais **l'encre réellement écrite sur le pixmap**, obtenue en
//! dessinant la même carte avec et sans son texte puis en comparant les deux images. La
//! mesure est donc indépendante de l'implémentation : elle resterait valable si le texte
//! changeait de moteur de rendu.

use super::*;
use crate::params::ViewPass;
use std::collections::HashSet;
use tiny_skia::Pixmap;

/// Largeur et hauteur de la carte de la preuve visuelle, en unités monde.
const PROOF_W: f64 = 260.0;
const PROOF_H: f64 = 120.0;
/// Origine écran de la carte dans les captures.
const PROOF_ORIGIN: f64 = 40.0;
const PROOF_TEXT: &str =
    "# Fidelite du zoom\n- une seule transformation\n- aucune borne par valeur\nR-45";

fn proof_store(text: &str) -> Store {
    let mut store = Store::new("R-45");
    let board = store.project.active_board_id.clone();
    store.add_annotation(
        &board,
        Annotation::Text {
            id: "fidelite".into(),
            x: 0.0,
            y: 0.0,
            width: Some(PROOF_W),
            height: Some(PROOF_H),
            text: text.into(),
            font_size: Some(14.0),
            color: Some("#38bdf8".into()),
            cursor_pos: None,
            source_file: None,
            membrane_id: None,
            domains: Vec::new(),
            mirror_of: None,
            temporal_anchor: None,
        },
    );
    store
}

fn render_proof(typo: &Typography, text: &str, scale: f64) -> Pixmap {
    let store = proof_store(text);
    let mut pixmap = Pixmap::new(1200, 800).expect("pixmap 1200x800");
    pixmap.fill(Color::from_rgba8(13, 14, 18, 255));
    let ids: HashSet<&str> = std::iter::once("fidelite").collect();
    let pass = ViewPass {
        vp: Viewport { x: PROOF_ORIGIN, y: PROOF_ORIGIN, scale },
        visible_ids: &ids,
        header_h: 0.0,
    };
    let mut hue = SymbioticHueCache::new();
    let mut tints = crate::renderer::domain::DomainTints::new();
    tints.refresh(&store, &crate::theme::Theme::dark());
    let mut view = pixmap.as_mut();
    draw_annotations(&mut hue, typo, &tints, &mut view, &store, None, pass);
    pixmap
}

/// Boîte englobante des pixels qui diffèrent entre la carte pleine et la carte vide :
/// l'encre du texte, mesurée sans rien savoir de la façon dont elle a été posée.
fn ink_bbox(inked: &Pixmap, empty: &Pixmap) -> Option<(u32, u32, u32, u32)> {
    let (mut x0, mut y0, mut x1, mut y1) = (u32::MAX, u32::MAX, 0u32, 0u32);
    let w = inked.width();
    let (left, _) = inked.data().as_chunks::<4>();
    let (right, _) = empty.data().as_chunks::<4>();
    for (i, (a, b)) in left.iter().zip(right).enumerate() {
        if a != b {
            let (x, y) = (i as u32 % w, i as u32 / w);
            x0 = x0.min(x);
            y0 = y0.min(y);
            x1 = x1.max(x + 1);
            y1 = y1.max(y + 1);
        }
    }
    (x0 != u32::MAX).then_some((x0, y0, x1, y1))
}

#[test]
fn test_r45_the_drawn_text_keeps_the_same_share_of_the_card() {
    // La preuve que R-45 est clos : on mesure l'encre réellement posée sur le pixmap, pas
    // les nombres qui ont servi à la poser. Avant le correctif, la même mesure ne trouvait
    // AUCUN texte à 0,25 ni à 0,5 (la police bornée à 8 px passait sous le seuil de rendu,
    // et la boîte s'étirait à 51 px au lieu de 30), puis 0,60 à 1, 0,49 à 2 et 0,25 à 4.
    let dir = std::path::Path::new("target/r45-after");
    std::fs::create_dir_all(dir).expect("dossier de capture");
    let typo = Typography::new();
    let mut report = String::new();
    let mut measured: Vec<Measured> = Vec::new();

    for zoom in [0.25_f64, 0.5, 1.0, 2.0, 4.0] {
        let inked = render_proof(&typo, PROOF_TEXT, zoom);
        let empty = render_proof(&typo, "", zoom);
        let (box_w, box_h) = (PROOF_W * zoom, PROOF_H * zoom);
        let (x0, y0, x1, y1) = ink_bbox(&inked, &empty).expect("la carte doit porter du texte");
        let (w, h) = ((x1 - x0) as f64, (y1 - y0) as f64);
        let (rw, rh, left) = (w / box_w, h / box_h, (x0 as f64 - PROOF_ORIGIN) / box_w);
        report.push_str(&format!(
            "scale={zoom:.2} boite={box_w:.1}x{box_h:.1} encre={w:.1}x{h:.1} \
             ratio_l={rw:.4} ratio_h={rh:.4} marge_g={left:.4}\n"
        ));
        measured.push(Measured { zoom, box_w, box_h, rw, rh, left });
        inked.save_png(dir.join(format!("card-x{zoom:.2}.png"))).expect("ecriture png");
    }
    std::fs::write(dir.join("ratios.txt"), &report).expect("rapport");
    println!("{report}");

    // La référence est le zoom 1 : c'est le seul où, même avant le correctif, aucune borne
    // ne se déclenchait. Si les cinq zooms s'y ramènent, ils dessinent la même carte.
    let reference = measured[2];
    for m in &measured {
        m.agrees_with(reference, &report);
    }
}

/// Une mesure d'encre à un zoom donné.
#[derive(Clone, Copy)]
struct Measured {
    zoom: f64,
    box_w: f64,
    box_h: f64,
    rw: f64,
    rh: f64,
    left: f64,
}

impl Measured {
    /// Les trois rapports coïncident-ils avec ceux de la référence ?
    ///
    /// La comparaison se fait **en pixels**, pas en rapports : une boîte de 30 px et une
    /// boîte de 480 px ne peuvent pas porter le même rapport au millième, parce que l'encre
    /// est discrétisée au pixel et que `fontdue` arrondit à l'entier la hauteur d'un glyphe.
    /// La tolérance couvre exactement ces deux arrondis — **2 px** pour la boîte englobante,
    /// **3 %** pour la quantification des métriques — et rien d'autre.
    fn agrees_with(self, reference: Self, report: &str) {
        for (name, observed, expected) in [
            ("largeur", self.rw * self.box_w, reference.rw * self.box_w),
            ("hauteur", self.rh * self.box_h, reference.rh * self.box_h),
            ("marge gauche", self.left * self.box_w, reference.left * self.box_w),
        ] {
            let allowance = 2.0 + 0.03 * expected;
            assert!(
                (observed - expected).abs() <= allowance,
                "zoom {} : {name} de {observed:.1} px au lieu de {expected:.1} px (tolérance {allowance:.1} px)\n{report}",
                self.zoom
            );
        }
    }
}

/// Le texte de la preuve visuelle de R-51 : chaque accent du français courant, la ligature
/// et le tiret cadratin, dans une phrase qu'un moodboard pourrait vraiment porter.
const ACCENTED_TEXT: &str = "Éditer — déjà prêt, à bientôt, cœur";
/// La même phrase amputée de ses accents : ce que l'ancienne police en faisait.
const STRIPPED_TEXT: &str = "Editer - deja pret, a bientot, coeur";

#[test]
fn test_r51_accented_text_is_drawn_on_the_card() {
    // Avant Inter, la police n'avait aucun accent : « Éditer » se dessinait « Editer » par
    // repli silencieux, et « cœur » devenait « cour ». On ne mesure pas la police, on mesure
    // l'encre : la carte accentuée doit différer de la carte amputée, et déborder au-dessus
    // de la capitale là où l'accent aigu de « É » se pose.
    let dir = std::path::Path::new("target/r51-font");
    std::fs::create_dir_all(dir).expect("dossier de capture");
    let typo = Typography::new();
    for zoom in [1.0_f64, 2.0] {
        let accented = render_proof(&typo, ACCENTED_TEXT, zoom);
        let stripped = render_proof(&typo, STRIPPED_TEXT, zoom);
        let empty = render_proof(&typo, "", zoom);
        let (_, top_accented, _, _) = ink_bbox(&accented, &empty).expect("la carte porte du texte");
        let (_, top_stripped, _, _) = ink_bbox(&stripped, &empty).expect("la carte porte du texte");
        assert_ne!(accented.data(), stripped.data(), "zoom {zoom} : les accents ne laissent aucune encre");
        assert!(
            top_accented < top_stripped,
            "zoom {zoom} : l'accent de « É » devrait dépasser la capitale ({top_accented} >= {top_stripped})"
        );
        accented.save_png(dir.join(format!("card-x{zoom:.2}.png"))).expect("ecriture png");
    }
}

#[test]
fn test_scale_2_below_the_threshold_the_card_keeps_its_frame_and_drops_its_text() {
    let typo = Typography::new();
    let below = WorldScale::SIMPLIFIED_BELOW as f64 / 2.0;
    let inked = render_proof(&typo, PROOF_TEXT, below);
    let empty = render_proof(&typo, "", below);
    assert!(
        ink_bbox(&inked, &empty).is_none(),
        "sous le seuil, la carte ne doit plus rastériser de glyphe"
    );
    // Le cadre, lui, reste : la carte est simplifiée, pas absente.
    let mut background = Pixmap::new(1200, 800).expect("pixmap");
    background.fill(Color::from_rgba8(13, 14, 18, 255));
    assert_ne!(inked.data(), background.data(), "la carte simplifiee doit rester visible");
}
