//! Les épreuves des équerres de la sélection (Q12, fiche 137).

use super::*;
use crate::params::{Pointer, SceneOverlay};
use crate::renderer::{Regard, Renderer};
use crate::ui::UiState;
use glucose_core::smart_align::SnapGuides;
use glucose_core::types::{BoardImage, CanvasFolder, Viewport};
use tiny_skia::Pixmap;

const LARGEUR: u32 = 900;
const HAUTEUR: u32 = 700;

/// La boîte écran du nœud éprouvé : loin du bandeau, la vue au zoom 1 sur l'origine.
const BOITE: (f32, f32, f32, f32) = (300.0, 260.0, 240.0, 160.0);

/// Le tableau du nœud éprouvé `n`, et un post-it loin de lui : choisis ensemble, leurs poignées
/// sont celles du groupe — aucune poignée de nœud ne se pose au milieu d'un bord.
fn tableau(n: &str) -> Store {
    let mut store = Store::new("equerres");
    let (x, y, w, h) = BOITE;
    let (x, y, w, h) = (f64::from(x), f64::from(y), f64::from(w), f64::from(h));
    let board = store.project.boards.first_mut().expect("un tableau");
    board.viewport = Viewport {
        x: 0.0,
        y: 0.0,
        scale: 1.0,
    };
    board
        .annotations
        .push(Annotation::sticky("loin", 700.0, 560.0, "loin"));
    match n {
        "texte" => {
            let mut t = Annotation::text("n", x, y, "Le col du Lautaret");
            if let Annotation::Text { width, height, .. } = &mut t {
                (*width, *height) = (Some(w), Some(h));
            }
            board.annotations.push(t);
        }
        "post-it" => {
            let mut s = Annotation::sticky("n", x, y, "Réserver le refuge");
            if let Annotation::Sticky { width, height, .. } = &mut s {
                (*width, *height) = (Some(w), Some(h));
            }
            board.annotations.push(s);
        }
        "membrane" => board
            .annotations
            .push(Annotation::membrane("n", x, y, w, h)),
        "dossier" => {
            let mut f = CanvasFolder::new("n", "Recherches", "board-enfant");
            (f.x, f.y, f.width, f.height) = (x, y, w, h);
            board.folders.push(f);
        }
        "photo" => board
            .images
            .push(BoardImage::new("n", x + w / 2.0, y + h / 2.0, w, h)),
        autre => panic!("sorte inconnue : {autre}"),
    }
    store
}

/// Le tableau rendu par le vrai rendu, interface comprise.
fn rendre(store: &Store, editing: Option<&TextEditSession>) -> Pixmap {
    let mut pixmap = Pixmap::new(LARGEUR, HAUTEUR).expect("un pixmap");
    let mut renderer = Renderer::new();
    let mut ui = UiState::new();
    let guides = SnapGuides::default();
    let mut overlay = SceneOverlay::sans_rien(&guides);
    overlay.editing = editing;
    let pointeur = Pointer { x: 0.0, y: 0.0 };
    let regard = Regard::immobile();
    renderer.render(
        &mut pixmap.as_mut(),
        store,
        &mut ui,
        overlay,
        pointeur,
        regard,
    );
    pixmap
}

fn choisir(store: &mut Store, ids: &[&str]) {
    for id in ids {
        let id = (*id).to_string();
        if store
            .active_board()
            .is_some_and(|b| b.images.iter().any(|i| i.id == id))
        {
            store.selected_image_ids.push(id);
        } else if store
            .active_board()
            .is_some_and(|b| b.folders.iter().any(|f| f.id == id))
        {
            store.selected_folder_ids.push(id);
        } else {
            store.selected_annotation_ids.push(id);
        }
    }
}

fn pixel(p: &Pixmap, (x, y): (i32, i32)) -> [u8; 4] {
    let c = p.pixel(x as u32, y as u32).expect("dans le pixmap");
    [c.red(), c.green(), c.blue(), c.alpha()]
}

/// **Q12 — chaque sorte de nœud choisie porte ses quatre équerres, et rien d'autre ne change** :
/// ni le long de ses bords — où l'anneau du texte et du post-it, le cadre décollé de la photo, le
/// trait plein de la membrane et le halo du dossier posaient leur encre —, ni en son centre, où
/// le dossier choisi épaississait son voile. Le nœud reste tel quel (l'image B, qu'il a choisie).
#[test]
fn test_q12_chaque_sorte_porte_ses_equerres_et_rien_d_autre_ne_change() {
    let (x, y, w, h) = BOITE;
    let (x0, y0) = ((x - ECART) as i32, (y - ECART) as i32);
    let (x1, y1) = ((x + w + ECART) as i32 - 1, (y + h + ECART) as i32 - 1);
    let (cx, cy) = ((x + w / 2.0) as i32, (y + h / 2.0) as i32);
    for sorte in ["texte", "post-it", "membrane", "dossier", "photo"] {
        let repos = rendre(&tableau(sorte), None);
        let mut store = tableau(sorte);
        choisir(&mut store, &["n", "loin"]);
        let choisi = rendre(&store, None);
        // Sur chaque bras horizontal, au-delà de la poignée de coin qui se loge dans l'angle.
        for bras in [(x0 + 12, y0), (x1 - 12, y0), (x1 - 12, y1), (x0 + 12, y1)] {
            let p = pixel(&choisi, bras);
            assert!(
                p[..3].iter().all(|&c| c >= 150),
                "{sorte} : l'équerre passe en {bras:?}, blanche ({p:?})"
            );
            assert_ne!(p, pixel(&repos, bras), "{sorte} : au repos, pas d'équerre");
        }
        // Le long des bords, de six pixels dehors jusqu'au bord, et au centre : identique.
        let mut temoins = vec![(cx, cy)];
        for d in 0..=6 {
            temoins.extend([(cx, y as i32 - d), (x as i32 - d, cy)]);
            temoins.extend([(cx, (y + h) as i32 - 1 + d), ((x + w) as i32 - 1 + d, cy)]);
        }
        for t in temoins {
            assert_eq!(
                pixel(&choisi, t),
                pixel(&repos, t),
                "{sorte} : choisi, le nœud change en {t:?} — rien ne doit courir le long des bords"
            );
        }
    }
}

/// **La poignée d'un coin se loge dans l'angle de son équerre** : un nœud choisi seul porte ses
/// poignées, et l'équerre passe après elles — posée avant, la poignée la recouvrait presque
/// entière (sur la photo, il n'en restait que des moignons). L'angle extérieur de l'équerre, que
/// la poignée atteint, est blanc.
#[test]
fn test_q12_la_poignee_d_un_coin_se_loge_dans_l_angle_de_son_equerre() {
    let (x, y, _, _) = BOITE;
    let angle = ((x - ECART) as i32, (y - ECART) as i32);
    for sorte in ["texte", "photo"] {
        let mut store = tableau(sorte);
        choisir(&mut store, &["n"]);
        let p = pixel(&rendre(&store, None), angle);
        assert!(
            p[..3].iter().all(|&c| c >= 150),
            "{sorte} : l'angle de l'équerre en {angle:?} ({p:?})"
        );
    }
}

/// **Une flèche ne porte pas d'équerres** : ses deux bouts sont déjà ses prises. Celle-ci descend
/// à peine, de (320, 500) à (620, 520) : les coins de sa boîte opposés à son trait — en bas à
/// gauche, en haut à droite — sont loin de lui ; choisie, rien n'y change. Une équerre, même
/// réduite par la minceur de la boîte (POIGNEE-1), y poserait de l'encre.
#[test]
fn test_q12_une_fleche_n_est_pas_visee() {
    let avec_fleche = |choisie: bool| {
        let mut store = tableau("texte");
        store
            .project
            .boards
            .first_mut()
            .expect("un tableau")
            .annotations
            .push(Annotation::arrow("f", 320.0, 500.0, 620.0, 520.0));
        if choisie {
            choisir(&mut store, &["f"]);
        }
        rendre(&store, None)
    };
    let (choisie, repos) = (avec_fleche(true), avec_fleche(false));
    for (cx, cy, sx, sy) in [(320, 520, 1, -1), (620, 500, -1, 1)] {
        for d in 0..12 {
            for e in 0..12 {
                // Du coin vers le dehors de la boîte, d'abord en x puis en y.
                let p = (cx - sx * d, cy - sy * e);
                assert_eq!(
                    pixel(&choisie, p),
                    pixel(&repos, p),
                    "choisie, la flèche pose de l'encre au coin de sa boîte, en {p:?}"
                );
            }
        }
    }
}

/// **Le texte qu'on écrit est visé**, choisi ou non : son anneau le montrait.
#[test]
fn test_q12_le_texte_qu_on_ecrit_est_vise() {
    let (x, y, _, _) = BOITE;
    let coin = ((x - ECART) as i32, (y - ECART) as i32);
    let store = tableau("texte");
    let session = TextEditSession {
        ann_id: "n".into(),
        buffer: "Le col du Lautaret".into(),
        selection: glucose_core::text::Selection::at(3),
        goal_x: None,
        blink_timer: std::time::Instant::now(),
        curseur_visible: false,
    };
    assert!(pixel(&rendre(&store, None), coin)[..3]
        .iter()
        .any(|&c| c < 150));
    let p = pixel(&rendre(&store, Some(&session)), coin);
    assert!(p[..3].iter().all(|&c| c >= 150), "en écriture : {p:?}");
}

/// L'encre la plus forte que les équerres posent sur la rangée `y`, en rouge sur fond noir.
fn encre_des_equerres(v: Vise, y: u32) -> u8 {
    let mut pixmap = Pixmap::new(400, 400).expect("pixmap");
    pixmap.fill(tiny_skia::Color::BLACK);
    let scale = crate::renderer::scale::WorldScale::new(1.0, 1.0);
    poser(&mut pixmap.as_mut(), &Theme::dark(), scale, &[v]);
    (0..400)
        .map(|x| pixmap.data()[((y * 400 + x) * 4) as usize])
        .max()
        .unwrap_or(0)
}

/// **POIGNEE-1 — les équerres suivent la place du nœud** : entières autour d'une photo, à peine
/// une trace autour d'une vignette de huit pixels — la loi du cadre des photos qu'elles
/// remplacent, qui dessinait sans elle la grille blanche de son retour du 07/10.
#[test]
fn test_poignee_1_les_equerres_d_une_vignette_s_estompent() {
    // La rangée du bord haut des équerres : l'écart au-dessus de la boîte.
    let grande = encre_des_equerres(Vise::droit((100.0, 100.0, 200.0, 200.0), false), 95);
    let vignette = (90..100)
        .map(|y| encre_des_equerres(Vise::droit((100.0, 100.0, 8.0, 8.0), false), y))
        .max()
        .unwrap_or(0);
    assert!(grande >= 200, "les équerres d'une grande photo : {grande}");
    assert!(
        vignette < grande / 4,
        "les équerres d'une vignette de 8 px gardent {vignette} d'encre contre {grande}"
    );
}

/// **Une photo verrouillée porte ses équerres à l'encre de l'alerte** (fiche 06 § 4.3).
#[test]
fn test_q12_une_photo_verrouillee_porte_l_alerte() {
    let mut pixmap = Pixmap::new(400, 400).expect("pixmap");
    pixmap.fill(tiny_skia::Color::BLACK);
    let scale = crate::renderer::scale::WorldScale::new(1.0, 1.0);
    let theme = Theme::dark();
    poser(
        &mut pixmap.as_mut(),
        &theme,
        scale,
        &[Vise::droit((100.0, 100.0, 200.0, 200.0), true)],
    );
    let p = pixel(&pixmap, (95, 95));
    let alerte = theme.alert.to_color_u8();
    assert_eq!(
        (p[0], p[1], p[2]),
        (alerte.red(), alerte.green(), alerte.blue())
    );
}

/// **Une photo penchée tourne ses équerres avec elle** : un carré tourné d'un quart de tour
/// porte son équerre au-dessus de son sommet, et rien au coin qu'aurait le carré droit.
#[test]
fn test_q12_une_photo_penchee_tourne_ses_equerres() {
    let mut img = BoardImage::new("p", 200.0, 200.0, 141.42, 141.42);
    img.rotation = std::f64::consts::FRAC_PI_4;
    let index = glucose_core::quadtree::SpatialHash::new(1000.0);
    let rangs: Vec<u32> = Vec::new();
    let pass = ViewPass {
        vp: Viewport {
            x: 0.0,
            y: 0.0,
            scale: 1.0,
        },
        visibles: &rangs,
        index: &index,
        contournement: None,
        header_h: 0.0,
        densite: 1.0,
    };
    let v = vise_d_une_photo(&img, &pass);
    assert!(!v.droit);
    let mut pixmap = Pixmap::new(400, 400).expect("pixmap");
    pixmap.fill(tiny_skia::Color::BLACK);
    let scale = crate::renderer::scale::WorldScale::new(1.0, 1.0);
    poser(&mut pixmap.as_mut(), &Theme::dark(), scale, &[v]);
    // Le sommet haut du losange est en (200, 100) ; son équerre, au-dessus.
    let au_dessus = (90..100)
        .map(|y| pixel(&pixmap, (200, y))[0])
        .max()
        .unwrap_or(0);
    assert!(au_dessus >= 150, "l'équerre du sommet : {au_dessus}");
    // Le coin haut-gauche du carré droit (129, 129), écarté : dehors, rien.
    assert_eq!(pixel(&pixmap, (124, 124))[0], 0);
}
