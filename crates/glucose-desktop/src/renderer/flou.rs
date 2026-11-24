//! **Le flou du reste, autour de ce qu'on a choisi** (Q12, fiche 137) — ce que le bureau en fait :
//! ce qui reste net (le masque), et le flou sur la voie du processeur. La loi est celle du noyau
//! ([`glucose_core::flou`]) ; la carte graphique la transcrit (`present/flou_gpu`).
//!
//! # Combien
//!
//! Sa réponse, sous l'image D : *« au moins 60 % moins que ce qu'il y a actuellement »*. L'image D
//! floutait à σ ≈ 14 pixels : réduite au quart (variance 1,25), trois flous de boîte de rayon 3 sur
//! la petite grille (3 × 4 = 12, soit 192 sur la grande), agrandie au quart en bilinéaire
//! (16 / 6 ≈ 2,67) — √195,9 ≈ 14,0. Au moins 60 % de moins : **σ = 40 % × 14 = 5,6 pixels
//! logiques**, [`SIGMA`]. Un nombre de ressenti, le sien.
//!
//! # Ce qui reste net
//!
//! Chaque nœud que ses équerres visent ([`super::equerres::les_vises`]), et chaque flèche choisie,
//! son tracé épais de son halo et de ses disques — élargis de la [`marge`] : l'écart des équerres,
//! plus une maille et demie de la grille du masque. La lecture bilinéaire d'une grille grossière ne
//! vaut entièrement 1 qu'à une maille et demie de son bord : c'est ce qui garde les équerres, et les
//! poignées logées dans leurs angles, entièrement nettes.

use super::equerres::{les_vises, ECART};
use super::scale::WorldScale;
use super::PaintKit;
use crate::params::ViewPass;
use crate::renderer::TextEditSession;
use glucose_core::flou::{self as loi, Masque, Plan};
use glucose_core::store::Store;
use tiny_skia::{FillRule, LineCap, LineJoin, PathBuilder, PixmapMut, Stroke, Transform};

/// **L'écart-type du flou**, en pixels logiques : 40 % de celui de l'image D (voir l'en-tête).
pub(crate) const SIGMA: f32 = 5.6;

/// **Ce qui reste net**, à l'écran, en pixels du tampon.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Forme {
    /// Un nœud visé : ses quatre coins, dans l'ordre du tour.
    Quad([(f32, f32); 4]),
    /// Une flèche choisie : son tracé aplati, et sa demi-épaisseur.
    Trait { points: Vec<(f32, f32)>, demi: f32 },
}

impl Forme {
    /// La même forme vue `facteur` fois plus grande — d'une scène rendue réduite à l'écran.
    pub(crate) fn agrandie(&self, facteur: f32) -> Self {
        let k = |(x, y): (f32, f32)| (x * facteur, y * facteur);
        match self {
            Forme::Quad(c) => Forme::Quad(c.map(k)),
            Forme::Trait { points, demi } => Forme::Trait {
                points: points.iter().copied().map(k).collect(),
                demi: demi * facteur,
            },
        }
    }
}

/// **Le flou d'une image** : son plan, à la densité de l'écran, la grille de ce qui reste net, et
/// la ligne au-dessus de laquelle rien ne change — la bande de l'interface, qui ne se floute pas.
#[derive(Clone, Debug, PartialEq)]
pub struct Flou {
    pub plan: Plan,
    pub masque: Masque,
    pub haut: f32,
}

/// **Ce qui reste net dans cette image** : les nœuds visés, les flèches choisies — rien quand le
/// flou ne vit pas.
pub(crate) fn formes(
    vit: bool,
    kit: PaintKit<'_>,
    store: &Store,
    (pass, editing): (&ViewPass<'_>, Option<&TextEditSession>),
) -> Vec<Forme> {
    if !vit {
        return Vec::new();
    }
    let mut formes: Vec<Forme> = les_vises(kit, store, pass, editing)
        .into_iter()
        .map(|v| Forme::Quad(v.coins))
        .collect();
    formes.extend(
        super::arrow::traces_des_choisies(store, *pass, (kit.typography, kit.math))
            .into_iter()
            .map(|(points, demi)| Forme::Trait { points, demi }),
    );
    formes
}

/// **La marge autour de ce qui reste net**, en pixels du tampon : l'écart des équerres, plus une
/// maille et demie de la grille du masque (voir l'en-tête).
pub(crate) fn marge(scale: WorldScale, plan: &Plan) -> f32 {
    scale.screen(ECART) + 1.5 * plan.maille() as f32
}

/// **Le flou de cette image**, ou rien quand rien n'est visé — un écran entièrement flou, sans
/// rien de net, ne dirait rien de la sélection.
pub(crate) fn flou(
    formes: &[Forme],
    (largeur, hauteur): (u32, u32),
    (scale, haut): (WorldScale, f32),
) -> Option<Flou> {
    if formes.is_empty() || largeur == 0 || hauteur == 0 {
        return None;
    }
    let plan = Plan::de(f64::from(scale.screen(SIGMA)));
    let (gl, gh) = loi::grille(&plan, (largeur as usize, hauteur as usize));
    let mut grille = tiny_skia::Mask::new(gl as u32, gh as u32)?;
    let maille = plan.maille() as f32;
    let sur_la_grille = Transform::from_scale(1.0 / maille, 1.0 / maille);
    let m = marge(scale, &plan);
    let mut pb = PathBuilder::new();
    for f in formes {
        match f {
            Forme::Quad(coins) => elargir(&mut pb, coins, m),
            Forme::Trait { points, demi } => {
                if let Some(trait_) = trait_elargi(points, demi + m) {
                    grille.fill_path(&trait_, FillRule::Winding, true, sur_la_grille);
                }
            }
        }
    }
    if let Some(quads) = pb.finish() {
        grille.fill_path(&quads, FillRule::Winding, true, sur_la_grille);
    }
    Some(Flou {
        plan,
        masque: Masque {
            largeur: gl,
            hauteur: gh,
            octets: grille.data().to_vec(),
        },
        haut,
    })
}

/// Un quadrilatère élargi de `m` de chaque côté : chaque coin écarté le long de ses deux côtés.
fn elargir(pb: &mut PathBuilder, coins: &[(f32, f32); 4], m: f32) {
    for i in 0..4 {
        let p = coins[i];
        let unitaire = |q: (f32, f32)| {
            let (dx, dy) = (q.0 - p.0, q.1 - p.1);
            let n = dx.hypot(dy).max(f32::EPSILON);
            (dx / n, dy / n)
        };
        let (u, w) = (unitaire(coins[(i + 1) % 4]), unitaire(coins[(i + 3) % 4]));
        let (x, y) = (p.0 - m * (u.0 + w.0), p.1 - m * (u.1 + w.1));
        if i == 0 {
            pb.move_to(x, y);
        } else {
            pb.line_to(x, y);
        }
    }
    pb.close();
}

/// Le contour plein d'un tracé épais de `2 × demi`, aux bouts et aux coudes ronds.
fn trait_elargi(points: &[(f32, f32)], demi: f32) -> Option<tiny_skia::Path> {
    let (premier, reste) = points.split_first()?;
    let mut pb = PathBuilder::new();
    pb.move_to(premier.0, premier.1);
    for p in reste {
        pb.line_to(p.0, p.1);
    }
    if reste.is_empty() {
        pb.line_to(premier.0, premier.1);
    }
    let stroke = Stroke {
        width: 2.0 * demi,
        line_cap: LineCap::Round,
        line_join: LineJoin::Round,
        ..Stroke::default()
    };
    pb.finish()?.stroke(&stroke, 1.0)
}

/// **Floute le reste, sur la voie du processeur** : la scène de ce tampon, sous la bande, mêlée à
/// son flou par le masque.
pub(crate) fn flouter_le_reste(pixmap: &mut PixmapMut, flou: &Flou) {
    let (l, h) = (pixmap.width() as usize, pixmap.height() as usize);
    let demi = loi::flouter(pixmap.data_mut(), (l, h), &flou.plan);
    // La carte compare la ligne au centre de chaque pixel : `y + ½ < haut`, soit `y < ⌈haut − ½⌉`.
    let haut = (flou.haut - 0.5).max(0.0).ceil() as usize;
    loi::meler(
        pixmap.data_mut(),
        (l, h, haut),
        (&demi, &flou.plan),
        &flou.masque,
    );
    // Aucune marque de mesure à lui : le serveur de télémétrie ne la connaît pas (il se déploie
    // avant la version qui l'écrit) ; `img_scene_us` compte la scène, son flou compris.
}

impl super::Renderer {
    /// **Le flou de cette image**, à la taille de ce tampon : ce que la couche du dessus a relevé
    /// de net, vu `facteur` fois plus grand quand la scène s'est rendue réduite, à la densité de
    /// l'écran (Q12, fiche 137).
    pub(crate) fn flou_de_l_image(
        &self,
        taille: (u32, u32),
        facteur: u32,
        haut: f32,
    ) -> Option<Flou> {
        let k = facteur.max(1) as f32;
        let formes: Vec<Forme> = self.formes_du_flou.iter().map(|f| f.agrandie(k)).collect();
        let scale = WorldScale::new(1.0, self.densite);
        flou(&formes, taille, (scale, haut))
    }

    /// **Floute le reste, sur la voie du processeur**, si le flou vit dans cette image.
    pub(super) fn flouter_le_reste(&self, pixmap: &mut PixmapMut, facteur: u32, haut: f32) {
        let taille = (pixmap.width(), pixmap.height());
        if let Some(f) = self.flou_de_l_image(taille, facteur, haut) {
            flouter_le_reste(pixmap, &f);
        }
    }
}

#[cfg(test)]
mod tests;
