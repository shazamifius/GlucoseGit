//! **Une image qui arrive** : le marqueur d'un dépôt web, entre le lâcher et la livraison.
//!
//! # Ce qu'il dit, et ce qu'il ne dit pas
//!
//! Une épingle de Pinterest met 0,8 à 1,4 s à arriver — mesuré sur trois épingles réelles —,
//! et l'essentiel de cette attente se passe **avant** le premier octet de l'image : ouvrir la
//! connexion, lire la page, trouver l'original. Pendant ce temps, rien n'est connu de l'image :
//! ni sa taille, ni sa forme, ni combien d'octets il reste.
//!
//! Le marqueur ne dit donc que ce qui est vrai : **le dépôt est reçu, l'image vient de tel
//! site, et elle se posera ici.** Pas de barre de progression — elle resterait vide les quatre
//! cinquièmes du temps puis se remplirait d'un coup —, et pas de cadre à la forme d'une image
//! qu'on ne connaît pas encore : il faudrait le redimensionner à l'arrivée, ce qui serait
//! avouer qu'il mentait. C'est la limite d'un dixième de seconde de Nielsen qu'il tient : un
//! geste direct doit recevoir sa réponse tout de suite, même quand son résultat tarde.
//!
//! Il a la forme et les couleurs d'un toast, parce qu'il en est un : un message de
//! l'interface, simplement posé là où l'image arrivera plutôt qu'au bas de l'écran.

use super::PaintKit;
use crate::canvas::world_to_screen;
use crate::params::Arrivage;
use crate::typography::{Face, TextStyle};
use glucose_core::types::Viewport;
use tiny_skia::{Paint, PixmapMut, Stroke, Transform};

/// Pose le marqueur de chaque image qui arrive, centré sur le point où elle se posera.
pub(super) fn dessiner_les_arrivages(
    kit: PaintKit<'_>,
    pixmap: &mut PixmapMut,
    arrivages: &[Arrivage],
    (vp, echelle_ui): (&Viewport, f32),
) {
    let s = crate::theme::clamp_ui_scale(echelle_ui);
    for arrivage in arrivages {
        let texte = legende(&arrivage.hote);
        let (tw, _) = kit.typography.measure_text(&texte, 13.0 * s, Face::Regular);
        let (w, h) = (tw + 40.0 * s, 36.0 * s);
        let (cx, cy) = world_to_screen(arrivage.monde.0, arrivage.monde.1, vp);
        let (x, y) = (cx as f32 - w / 2.0, cy as f32 - h / 2.0);
        if let Some(pilule) = crate::ui::toast::pilule(x, y, w, h, h / 2.0) {
            let mut fond = Paint::default();
            fond.set_color(kit.theme.toast_bg);
            pixmap.fill_path(
                &pilule,
                &fond,
                tiny_skia::FillRule::Winding,
                Transform::identity(),
                None,
            );
            let mut bord = Paint::default();
            bord.set_color(kit.theme.toast_border);
            let trait_ = Stroke {
                width: s,
                ..Default::default()
            };
            pixmap.stroke_path(&pilule, &bord, &trait_, Transform::identity(), None);
        }
        kit.typography.draw_text(
            pixmap,
            &texte,
            x + 20.0 * s,
            y + (h - 13.0 * s) / 2.0,
            TextStyle {
                size: 13.0 * s,
                color: kit.theme.toast_text,
                face: Face::Regular,
            },
        );
    }
}

/// Ce que le marqueur dit : d'où l'image vient, s'il le sait.
fn legende(hote: &str) -> String {
    if hote.is_empty() {
        "Téléchargement…".to_string()
    } else {
        format!("Téléchargement depuis {hote}…")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Le marqueur nomme le site quand il le connaît, et ne l'invente pas sinon.
    #[test]
    fn test_le_marqueur_ne_dit_que_ce_qu_il_sait() {
        assert_eq!(
            legende("fr.pinterest.com"),
            "Téléchargement depuis fr.pinterest.com…"
        );
        assert_eq!(legende(""), "Téléchargement…");
    }

    /// Rend un marqueur sur un fond uni, au point du monde `(100, 50)`.
    fn rendu(vp: Viewport) -> tiny_skia::Pixmap {
        let mut pixmap = tiny_skia::Pixmap::new(400, 200).expect("pixmap");
        pixmap.fill(tiny_skia::Color::from_rgba8(13, 14, 18, 255));
        let typography = crate::typography::Typography::new();
        let math = crate::renderer::math::MathRenderer::new();
        let tints = crate::renderer::DomainTints::default();
        let theme = crate::theme::Theme::dark();
        let kit = PaintKit {
            typography: &typography,
            math: &math,
            tints: &tints,
            theme: &theme,
        };
        let arrivages = [Arrivage {
            numero: 1,
            monde: (100.0, 50.0),
            hote: "fr.pinterest.com".into(),
        }];
        dessiner_les_arrivages(kit, &mut pixmap.as_mut(), &arrivages, (&vp, 1.0));
        pixmap
    }

    /// **Le marqueur se centre sur le point du monde où l'image se posera**, et suit la vue :
    /// déplacée de cinquante pixels, la vue le déplace d'autant.
    #[test]
    fn test_le_marqueur_se_pose_ou_l_image_arrivera() {
        let fond =
            |p: tiny_skia::PremultipliedColorU8| (p.red(), p.green(), p.blue()) == (13, 14, 18);
        let ici = Viewport {
            x: 100.0,
            y: 50.0,
            scale: 1.0,
        };
        let pixmap = rendu(ici);
        // Le point (100, 50) du monde tombe en (200, 100) de l'écran.
        assert!(
            !fond(pixmap.pixel(200, 100).expect("pixel")),
            "de l'encre au point de dépôt"
        );
        assert!(
            fond(pixmap.pixel(200, 10).expect("pixel")),
            "rien loin au-dessus"
        );
        let decalee = rendu(Viewport {
            x: 100.0,
            y: 0.0,
            scale: 1.0,
        });
        assert!(
            !fond(decalee.pixel(200, 50).expect("pixel")),
            "la vue l'emporte avec elle"
        );
    }
}
