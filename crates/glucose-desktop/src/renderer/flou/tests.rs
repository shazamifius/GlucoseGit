//! Les épreuves du flou de la sélection, côté bureau (Q12, fiche 137).

use super::*;
use crate::params::{Pointer, SceneOverlay};
use crate::renderer::{Regard, Renderer};
use crate::ui::UiState;
use glucose_core::smart_align::SnapGuides;
use glucose_core::types::{Annotation, Viewport};
use tiny_skia::Pixmap;

const LARGEUR: u32 = 900;
const HAUTEUR: u32 = 700;

/// Un texte choisi en (300, 260), un texte témoin jamais choisi en (80, 470), une flèche choisie
/// sous eux.
fn tableau() -> Store {
    let mut store = Store::new("flou");
    let board = store.project.boards.first_mut().expect("un tableau");
    board.viewport = Viewport {
        x: 0.0,
        y: 0.0,
        scale: 1.0,
    };
    let mut choisi = Annotation::text("n", 300.0, 260.0, "Le col du Lautaret");
    let mut temoin = Annotation::text("t", 80.0, 470.0, "Un témoin jamais choisi, flouté");
    for (t, w, h) in [(&mut choisi, 240.0, 120.0), (&mut temoin, 300.0, 120.0)] {
        if let Annotation::Text { width, height, .. } = t {
            (*width, *height) = (Some(w), Some(h));
        }
    }
    board.annotations.extend([
        choisi,
        temoin,
        Annotation::arrow("f", 520.0, 520.0, 820.0, 640.0),
    ]);
    store.selected_annotation_ids.push("n".into());
    store.selected_annotation_ids.push("f".into());
    store
}

fn rendre(store: &Store, flou: bool) -> Pixmap {
    let mut pixmap = Pixmap::new(LARGEUR, HAUTEUR).expect("un pixmap");
    let mut renderer = Renderer::new();
    let mut ui = UiState::new();
    let guides = SnapGuides::default();
    let mut overlay = SceneOverlay::sans_rien(&guides);
    overlay.flou = flou;
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

/// L'écart le plus fort entre deux images sur une boîte `(x0, y0, x1, y1)`, et leur écart total.
fn ecart(a: &Pixmap, b: &Pixmap, (x0, y0, x1, y1): (u32, u32, u32, u32)) -> (u8, u64) {
    let (mut fort, mut total) = (0u8, 0u64);
    for y in y0..y1 {
        for x in x0..x1 {
            let (p, q) = (
                a.pixel(x, y).expect("dedans"),
                b.pixel(x, y).expect("dedans"),
            );
            let d = [
                p.red().abs_diff(q.red()),
                p.green().abs_diff(q.green()),
                p.blue().abs_diff(q.blue()),
            ];
            fort = fort.max(d.into_iter().max().unwrap_or(0));
            total += d.iter().map(|&v| u64::from(v)).sum::<u64>();
        }
    }
    (fort, total)
}

/// **Q12 — ce qu'on a choisi reste net, le reste se floute** : le texte choisi, ses équerres et la
/// flèche choisie ne changent pas d'un pixel ; le texte témoin, si ; la bande, jamais.
#[test]
fn test_q12_ce_qu_on_a_choisi_reste_net_le_reste_se_floute() {
    let store = tableau();
    let (net, flou) = (rendre(&store, false), rendre(&store, true));
    // Le texte choisi et ses équerres, à l'écart des équerres près (5 pixels).
    assert_eq!(
        ecart(&net, &flou, (295, 255, 545, 385)).0,
        0,
        "le choisi reste net"
    );
    // La flèche choisie, son trait et son halo : six pixels de part et d'autre.
    for x in 560..780 {
        let y = 520 + (x - 520) * 2 / 5;
        let (fort, _) = ecart(&net, &flou, (x, y - 6, x + 1, y + 7));
        assert_eq!(fort, 0, "la flèche reste nette en x = {x}");
    }
    // Les poignées du groupe (le texte et la flèche choisis ensemble) : celle du coin haut-droit
    // de l'emprise, en (820, 260), ne tombe sur aucun nœud.
    assert_eq!(
        ecart(&net, &flou, (812, 252, 829, 269)).0,
        0,
        "la poignée du groupe reste nette"
    );
    // La bande de l'interface.
    let bande = UiState::new().header_height() as u32;
    assert_eq!(
        ecart(&net, &flou, (0, 0, LARGEUR, bande)).0,
        0,
        "la bande ne se floute pas"
    );
    // Le témoin : ses lettres s'étalent.
    let (fort, total) = ecart(&net, &flou, (80, 470, 380, 590));
    assert!(
        fort > 40 && total > 20_000,
        "le témoin se floute : {fort}, {total}"
    );
}

/// Le plus fort écart d'un pixel à son voisin de droite sur une boîte : les lettres nettes d'un
/// texte en ont un grand, que le flou efface.
fn contraste(p: &Pixmap, (x0, y0, x1, y1): (u32, u32, u32, u32)) -> u8 {
    let mut fort = 0;
    for y in y0..y1 {
        for x in x0..x1 {
            let (a, b) = (
                p.pixel(x, y).expect("dedans"),
                p.pixel(x + 1, y).expect("dedans"),
            );
            fort = fort.max(a.red().abs_diff(b.red()));
        }
    }
    fort
}

/// **Rien de visé, rien de flou** : un écran entièrement flou ne dirait rien de la sélection. Le
/// texte témoin garde le contraste de ses lettres — un témoin indépendant : deux rendus qui
/// passeraient tous deux par le flou seraient égaux entre eux.
#[test]
fn test_q12_rien_de_vise_rien_ne_se_floute() {
    let mut store = tableau();
    store.selected_annotation_ids.clear();
    let rendu = rendre(&store, true);
    assert_eq!(rendu.data(), rendre(&store, false).data());
    let lettres = (90, 480, 370, 520);
    assert!(contraste(&rendu, lettres) > 80, "le témoin reste net");
    // À l'envers : choisi ailleurs, le flou efface ce contraste.
    let floute = rendre(&tableau(), true);
    assert!(
        contraste(&floute, lettres) < 40,
        "flouté, le témoin perd ses lettres"
    );
}

/// **Le flou vaut 40 % de celui de l'image D** — sa réponse : « au moins 60 % moins ». Celui de
/// l'image D se recompte depuis le prototype : réduite au quart (variance (4² − 1)/12), trois
/// flous de boîte de rayon 3 sur la petite grille (3 × 3 × 4 / 3), agrandie au quart (4²/6).
#[test]
fn test_q12_le_flou_vaut_quarante_pour_cent_de_celui_de_l_image_d() {
    let reduire: f64 = (16.0 - 1.0) / 12.0;
    let boites = 16.0 * 3.0 * (3.0 * 4.0) / 3.0;
    let agrandir = 16.0 / 6.0;
    let sigma_d: f64 = (reduire + boites + agrandir).sqrt();
    assert!((sigma_d - 14.0).abs() < 0.05, "σ de l'image D : {sigma_d}");
    assert!(
        (f64::from(SIGMA) - 0.4 * sigma_d).abs() < 0.05,
        "40 % : {SIGMA}"
    );
}
