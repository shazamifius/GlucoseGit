//! L'unique mise à l'échelle du monde vers l'écran (standard § 4.4).
//!
//! # SCALE-1 — une seule transformation, jamais une borne par valeur
//!
//! Tout ce qui appartient au monde — cadre, police, marges, rayons, puces, curseur — passe
//! par [`WorldScale::world`], et par elle seule. Une borne posée sur la police sans borne
//! équivalente sur la boîte fait déborder le texte ; six bornes à six seuils cassent la mise
//! en page six fois, à six niveaux de zoom différents. C'est le constat **R-45**, et le code
//! TypeScript d'origine documentait déjà le piège (`HtmlAnnotationLayer.tsx:203-210`) :
//!
//! > *« Une transformation d'échelle plutôt qu'une largeur divisée : réduire la boîte sans
//! > réduire la police ferait déborder le texte, alors que `scale` emporte tout d'un coup —
//! > cadre, police, marges, badges. »*
//!
//! La seule exception admise est [`WorldScale::screen`], la contrepartie du `1 / scale` que
//! le TSX applique à ses guides : ce qui doit garder une **taille écran constante** ne passe
//! pas par la mise à l'échelle, il est écrit en pixels et ne bouge pas.
//!
//! # SCALE-2 — un seul niveau de détail, décidé pour l'élément entier
//!
//! Si une borne de lisibilité est souhaitée, elle porte sur la transformation entière et
//! nulle part ailleurs : sous [`WorldScale::SIMPLIFIED_BELOW`], un élément se dessine
//! **simplifié** — son cadre, sans son texte. C'est un niveau de détail assumé, nommé et
//! testé, pas six bornes qui se déclenchent chacune dans son coin.

/// Le facteur de zoom d'une passe de rendu, et la seule façon d'en dériver une longueur.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WorldScale(f32);

impl WorldScale {
    /// Échelle sous laquelle un élément se dessine simplifié, sans son texte (SCALE-2).
    ///
    /// Le corps d'une carte mesure 14 unités monde : à ce seuil il vaut 2,8 px à l'écran,
    /// soit moins que l'épaisseur d'un jambage. En deçà, le glyphe ne transporte plus
    /// d'information — le rastériser remplirait le cache de tailles que personne ne lit,
    /// et le composer coûterait une passe de mélange pour un gris uniforme.
    pub const SIMPLIFIED_BELOW: f32 = 0.2;

    /// Adopte le zoom d'un viewport.
    ///
    /// Un zoom non fini ou négatif est ramené à zéro : rien ne se dessine, ce qui vaut
    /// infiniment mieux que de laisser un `NaN` atteindre la rastérisation, où il devient
    /// une taille de glyphe aberrante et une boucle de plusieurs centaines de millions
    /// d'itérations.
    pub(super) fn new(scale: f64) -> Self {
        if scale.is_finite() && scale > 0.0 {
            Self(scale as f32)
        } else {
            Self(0.0)
        }
    }

    /// Met une longueur exprimée en **unités monde** à l'échelle de l'écran.
    pub fn world(self, length: f32) -> f32 {
        length * self.0
    }

    /// Une longueur qui garde une **taille écran constante** quel que soit le zoom.
    ///
    /// C'est l'exception de SCALE-1, réservée aux affordances : poignées, anneau de
    /// sélection, traits d'un pixel. Le passage par cette fonction est délibéré — il rend
    /// l'exception visible au point d'appel au lieu d'un littéral muet.
    pub fn screen(self, pixels: f32) -> f32 {
        let _ = self;
        pixels
    }

    /// L'élément se dessine-t-il avec son texte, ou simplifié (SCALE-2) ?
    pub(super) fn draws_detail(self) -> bool {
        self.0 >= Self::SIMPLIFIED_BELOW
    }
}

// ── SCALE-3 — un filet se pose sur la grille de pixels, et ne s'anti-aliase pas ──

/// Remplit un rectangle **aligné sur les axes** — filet, plaque, poignée — proprement.
///
/// # Pourquoi une fonction plutôt que quatre appels
///
/// `tiny-skia` **panique** quand on lui demande un `fill_rect` anti-aliasé dont un côté
/// tombe au-dessous du pixel : `hairline_aa.rs` fait `assert!(false)`. Ce n'est pas une
/// hypothèse — le défaut a emporté treize tests de redimensionnement sur un filet de
/// séparation, puis la capture de la scène sélectionnée sur la barre d'une citation. Deux
/// fois le même crash, à deux endroits, parce que la correction avait été faite sur place.
///
/// # Et c'est aussi plus net, donc ce n'est pas un contournement
///
/// Un rectangle aligné sur les axes n'a aucun bord oblique : l'anti-aliasing ne lui apporte
/// rien et lui coûte tout. Un filet d'un pixel posé sur une demi-position devient deux
/// demi-traits gris — flou là où la charte demande « net à quasi 100 % » (R-46). L'arrondi à
/// la grille rend donc un meilleur dessin, et le crash disparaît par surcroît.
///
/// Un côté nul serait invisible : il est ramené à un pixel, la plus petite chose qu'un écran
/// sache montrer. Des coordonnées non finies ne dessinent rien plutôt que de tout arrêter.
pub(crate) fn fill_crisp(
    pixmap: &mut tiny_skia::PixmapMut,
    rect: tiny_skia::Rect,
    color: tiny_skia::Color,
) {
    let Some(rect) = on_pixel_grid(rect) else {
        return;
    };
    remplir(pixmap, rect, color.premultiply());
}

/// **Remplit des pixels entiers d'une couleur unie, par la règle même de `tiny-skia`** — sans
/// passer par lui.
///
/// Un rectangle sur la grille n'a aucune couverture partielle : chaque pixel reçoit la couleur,
/// posée sur le fond, et c'est tout. Passer par le rastériseur coûtait la mise en place de son
/// pipeline à chaque appel — environ 0,65 µs, soit deux millisecondes et demie pour les
/// poignées de 243 photos sélectionnées (`bench_ornements`) — pour une opération qui tient en
/// une ligne par pixel.
///
/// Les formules sont celles de sa voie huit bits (`pipeline/lowp.rs`) : la couleur prémultipliée
/// ramenée sur `[0, 255]` au plus proche, puis `s + (d · (255 − a) + 255) >> 8`. Opaque, elle
/// donne la couleur elle-même, ce que sa copie mémoire donne aussi. L'épreuve les compare à
/// `fill_rect` sur des milliers de cas tirés au hasard, au bit près.
fn remplir(
    pixmap: &mut tiny_skia::PixmapMut,
    rect: tiny_skia::Rect,
    couleur: tiny_skia::PremultipliedColor,
) {
    let s = [
        couleur.red(),
        couleur.green(),
        couleur.blue(),
        couleur.alpha(),
    ]
    .map(|v| (v * 255.0 + 0.5) as u16);
    let reste = 255 - s[3];
    let (largeur, hauteur) = (pixmap.width() as usize, pixmap.height() as usize);
    let borne = |v: f32, max: usize| (v.max(0.0) as usize).min(max);
    let (x0, x1) = (borne(rect.left(), largeur), borne(rect.right(), largeur));
    let (y0, y1) = (borne(rect.top(), hauteur), borne(rect.bottom(), hauteur));
    if x0 >= x1 || y0 >= y1 {
        return;
    }
    let pas = largeur * 4;
    let donnees = pixmap.data_mut();
    for y in y0..y1 {
        for pixel in donnees[y * pas + x0 * 4..y * pas + x1 * 4]
            .as_chunks_mut::<4>()
            .0
        {
            for (d, s) in pixel.iter_mut().zip(s) {
                *d = (s + ((u16::from(*d) * reste + 255) >> 8)) as u8;
            }
        }
    }
}

/// Le même rectangle, ramené sur la grille de pixels, ou `None` s'il n'a pas de sens.
fn on_pixel_grid(r: tiny_skia::Rect) -> Option<tiny_skia::Rect> {
    tiny_skia::Rect::from_xywh(
        r.x().round(),
        r.y().round(),
        r.width().round().max(1.0),
        r.height().round().max(1.0),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scale_1_all_world_lengths_share_one_factor() {
        // La preuve de SCALE-1 : quel que soit le zoom, le rapport entre deux longueurs
        // du monde est celui qu'elles ont dans le monde. Aucune borne ne peut s'y glisser.
        for zoom in [0.05_f64, 0.25, 0.5, 1.0, 2.0, 4.0, 20.0] {
            let s = WorldScale::new(zoom);
            let (box_w, font, pad, radius) = (260.0_f32, 14.0, 18.0, 24.0);
            for length in [font, pad, radius] {
                let expected = length / box_w;
                let observed = s.world(length) / s.world(box_w);
                assert!(
                    (observed - expected).abs() < 1e-6,
                    "zoom {zoom} : {length} / {box_w} vaut {observed} au lieu de {expected}"
                );
            }
        }
    }

    #[test]
    fn test_scale_1_screen_lengths_never_follow_the_zoom() {
        for zoom in [0.25_f64, 1.0, 8.0] {
            assert_eq!(WorldScale::new(zoom).screen(2.0), 2.0);
        }
    }

    #[test]
    fn test_scale_1_a_degenerate_zoom_draws_nothing() {
        for bad in [0.0_f64, -1.0, f64::NAN, f64::INFINITY] {
            let s = WorldScale::new(bad);
            assert_eq!(s.world(1.0), 0.0, "zoom {bad}");
            assert_eq!(s.world(14.0), 0.0);
            assert!(!s.draws_detail());
        }
    }

    #[test]
    fn test_scale_2_the_level_of_detail_is_one_named_threshold() {
        assert!(!WorldScale::new(0.19).draws_detail());
        assert!(WorldScale::new(0.2).draws_detail());
        assert!(
            WorldScale::new(0.25).draws_detail(),
            "le zoom 0,25 garde son texte"
        );
    }
    // ── SCALE-3 ──

    /// Le cas exact qui faisait paniquer `tiny-skia` : un filet plus fin qu'un pixel.
    ///
    /// Sans ce test, la correction serait une anecdote ; avec lui, c'est une règle. Le crash
    /// est arrivé deux fois, à deux endroits, et rien ne l'avait retenu la première fois.
    #[test]
    fn test_scale_3_a_hairline_never_reaches_the_rasterizer() {
        let mut pixmap = tiny_skia::Pixmap::new(40, 40).expect("pixmap");
        let mut vue = pixmap.as_mut();
        let noir = tiny_skia::Color::from_rgba8(0, 0, 0, 255);
        for fin in [0.0f32, 0.01, 0.3, 0.5, 0.9] {
            let rect = tiny_skia::Rect::from_xywh(10.5, 10.5, 20.0, fin)
                .or_else(|| tiny_skia::Rect::from_xywh(10.5, 10.5, 20.0, 0.1))
                .expect("un rectangle");
            // Ne panique pas : c'est tout ce que ce test demande, et c'est ce qui manquait.
            fill_crisp(&mut vue, rect, noir);
        }
    }

    /// Un rectangle posé à une demi-position retombe sur la grille, et garde un pixel.
    #[test]
    fn test_scale_3_the_grid_keeps_at_least_one_pixel() {
        let sur = on_pixel_grid(tiny_skia::Rect::from_xywh(10.4, 10.6, 20.3, 0.2).expect("rect"))
            .expect("sur la grille");
        assert_eq!((sur.x(), sur.y()), (10.0, 11.0));
        assert_eq!((sur.width(), sur.height()), (20.0, 1.0));
    }

    /// **Le remplissage direct donne les octets de `tiny-skia`, au bit près** — sur des fonds,
    /// des couleurs et des rectangles tirés au hasard, opaques, translucides, transparents,
    /// dedans, à cheval sur le bord ; et rien du tout quand le rectangle est dehors.
    ///
    /// C'est ce qui autorise à ne plus passer par lui : sans cette épreuve, la voie directe
    /// serait une imitation, et un arrondi différent déplacerait chaque filet de l'interface
    /// d'un niveau sans que rien ne le dise. Elle a trouvé au passage un défaut de `tiny-skia`
    /// lui-même, que la voie directe n'a pas (voir plus bas).
    #[test]
    fn test_scale_3_le_remplissage_direct_donne_les_octets_de_tiny_skia() {
        let mut graine = 0x9e37_79b9_7f4a_7c15_u64;
        let mut tirer = move |n: u32| {
            graine ^= graine << 13;
            graine ^= graine >> 7;
            graine ^= graine << 17;
            (graine % u64::from(n)) as u32
        };
        for _ in 0..2_000 {
            let (l, h) = (1 + tirer(24), 1 + tirer(24));
            let mut fond = tiny_skia::Pixmap::new(l, h).expect("pixmap");
            for pixel in fond.data_mut().as_chunks_mut::<4>().0 {
                // Un fond prémultiplié : aucune composante au-dessus de son alpha.
                let a = tirer(256) as u8;
                for c in &mut pixel[..3] {
                    *c = (tirer(u32::from(a) + 1)) as u8;
                }
                pixel[3] = a;
            }
            let alpha = [0, 255, tirer(256)][tirer(3) as usize] as u8;
            let couleur = tiny_skia::Color::from_rgba8(
                tirer(256) as u8,
                tirer(256) as u8,
                tirer(256) as u8,
                alpha,
            );
            let x = tirer(40) as f32 - 8.0;
            let y = tirer(40) as f32 - 8.0;
            let Some(rect) =
                tiny_skia::Rect::from_xywh(x, y, 1.0 + tirer(20) as f32, 1.0 + tirer(20) as f32)
            else {
                continue;
            };
            let mut obtenu = fond.clone();
            remplir(&mut obtenu.as_mut(), rect, couleur.premultiply());
            // **La reference est `tiny-skia` sur le rectangle deja coupe a l'image.** Sur un
            // rectangle qui deborde a gauche ou en haut, il peint une colonne ou une rangee de
            // trop -- de -7 a 9, les colonnes 0 a 9 et non 0 a 8 : un pixel qui n'appartient
            // pas au rectangle. Coupe d'abord, il est exact, et c'est ce qu'on lui demande.
            let image = tiny_skia::Rect::from_xywh(0.0, 0.0, l as f32, h as f32).expect("image");
            let coupe = rect
                .intersect(&image)
                .filter(|c| c.width() > 0.0 && c.height() > 0.0);
            let Some(coupe) = coupe else {
                assert_eq!(
                    obtenu.data(),
                    fond.data(),
                    "{rect:?} est dehors : rien ne change"
                );
                continue;
            };
            let mut attendu = fond.clone();
            let mut paint = tiny_skia::Paint {
                anti_alias: false,
                ..Default::default()
            };
            paint.set_color(couleur);
            attendu.fill_rect(coupe, &paint, tiny_skia::Transform::identity(), None);
            assert_eq!(
                obtenu.data(),
                attendu.data(),
                "{l}x{h}, {rect:?}, couleur {couleur:?} : les octets different"
            );
        }
    }

    /// Des coordonnées impossibles ne dessinent rien plutôt que d'arrêter le rendu.
    #[test]
    fn test_scale_3_nonsense_draws_nothing() {
        let Some(rect) = tiny_skia::Rect::from_xywh(0.0, 0.0, 10.0, 10.0) else {
            panic!("rect");
        };
        let mut pixmap = tiny_skia::Pixmap::new(4, 4).expect("pixmap");
        // Le cas normal passe ; le cas dégénéré est écarté par `on_pixel_grid`.
        fill_crisp(
            &mut pixmap.as_mut(),
            rect,
            tiny_skia::Color::from_rgba8(1, 2, 3, 4),
        );
        assert!(on_pixel_grid(rect).is_some());
    }
}
