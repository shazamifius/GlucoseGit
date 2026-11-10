//! **Les petites textures rangées ensemble** (ATLAS-1, fiches 86 et 87).
//!
//! # Ce que ça répare
//!
//! À la vue d'ensemble, chaque photo fait quelques dizaines de pixels, et chacune avait sa
//! texture, sa liaison et son dessin. Huit mille dessins par image : `soumettre` passait de 1,7 à
//! 23 ms sans cet atlas, sur son tableau de 8 000 images, trois paires entrelacées (fiche 87).
//! C'est ce que `scene_gpu` annonçait : « la complexité d'un atlas ne se prend que si un banc la
//! réclame ».
//!
//! # La forme, sans nuanceur de plus
//!
//! Une **page** est une texture ordinaire. Le nuanceur sait déjà lire un sous-rectangle (le
//! recadrage, RECADRAGE-1) et se borner à une zone (BORDURES-4) : une petite texture n'est donc
//! qu'une fenêtre de plus. Les poses consécutives d'une même page partent en **un seul dessin**.
//! Aucun tableau de textures ni `binding_array` : rien de plus que ce que toutes les cartes savent
//! déjà faire, OpenGL compris.
//!
//! **Une cellule est une puissance de deux**, la plus petite qui contient la texture, et **une
//! page ne porte qu'un seul côté de cellule** : aucun découpage à calculer, et une texture ne
//! perd jamais plus de trois quarts de sa cellule. La page en porte toujours
//! [`PAR_COTE`]² : sa taille suit donc ce qu'elle range, et une photo réduite à quatre pixels ne
//! réserve pas seize mégaoctets. Une cellule de 64 pixels donne la page de 2 048, ce que toute
//! carte accepte (le minimum de `wgpu`, OpenGL ES compris) : c'est ce qui fixe la plus grande.
//!
//! **Le bord ne bave pas** : les bornes s'arrêtent au demi-texel de la texture, exactement là où
//! une texture à elle s'arrêterait, sans marge entre les cellules.
//!
//! **Une page vide se rend entière** ; une page partiellement vide occupe la carte tout entière
//! et le dit ([`SceneGpu::octets`](super::SceneGpu::octets)).

use super::{Cle, Pose, SceneGpu, Televersee};
use std::collections::BTreeMap;

/// Combien de cellules une page porte sur chaque côté.
const PAR_COTE: u32 = 32;
/// Le plus grand côté de cellule : une texture plus grande garde la sienne. Elle se voit de près,
/// et elle est de toute façon peu nombreuse à l'écran.
const COTE_MAX: u32 = 64;

/// **Le côté de la cellule qui range cette texture**, ou rien si elle est trop grande pour une
/// page.
fn cote_de_cellule(largeur: u32, hauteur: u32) -> Option<u32> {
    let cote = largeur.max(hauteur).next_power_of_two();
    (cote <= COTE_MAX).then_some(cote)
}

/// **Ce qu'une texture de cette taille occupe sur la carte**, cellule comprise : la règle de la
/// comptabilité, écrite une fois. Le cran des photos la lit aussi : il ne doit pas croire qu'une
/// photo réduite ne coûte rien alors que sa cellule, elle, reste.
pub fn octets_d_une_texture(largeur: u32, hauteur: u32) -> u64 {
    let (l, h) = cote_de_cellule(largeur, hauteur).map_or((largeur, hauteur), |c| (c, c));
    u64::from(l) * u64::from(h) * 4
}

/// Une place dans une page.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Cellule {
    pub lot: u64,
    pub rang: u32,
    /// Le côté des cellules de cette page.
    pub cote: u32,
    pub largeur: u32,
    pub hauteur: u32,
}

impl Cellule {
    /// **La pose qui lit cette cellule** : la même, dont la fenêtre et les bornes se lisent dans la
    /// page. Les bornes s'arrêtent au demi-texel de la texture : c'est là qu'une texture à elle
    /// s'arrêterait aussi, et le filtre ne lit jamais la voisine.
    pub fn dans_la_page(self, pose: Pose) -> Pose {
        let (cx, cy) = coin_de(self.rang, self.cote);
        let page = (self.cote * PAR_COTE) as f32;
        let (l, h) = (self.largeur as f32, self.hauteur as f32);
        let u = |x: f32| (cx as f32 + x * l) / page;
        let v = |y: f32| (cy as f32 + y * h) / page;
        let [g, t, fl, fh] = pose.fenetre;
        let [umin, vmin, umax, vmax] = pose.bornes;
        Pose {
            fenetre: [u(g), v(t), fl * l / page, fh * h / page],
            bornes: [
                u(umin.max(0.5 / l)),
                v(vmin.max(0.5 / h)),
                u(umax.min(1.0 - 0.5 / l)),
                v(vmax.min(1.0 - 0.5 / h)),
            ],
            ..pose
        }
    }
}

/// Le coin d'une cellule dans sa page, en pixels.
fn coin_de(rang: u32, cote: u32) -> (u32, u32) {
    ((rang % PAR_COTE) * cote, (rang / PAR_COTE) * cote)
}

struct Page {
    texture: wgpu::Texture,
    liaison: wgpu::BindGroup,
    cote: u32,
    /// Les rangs qui restent, le plus petit dernier : les cellules se remplissent dans l'ordre, et
    /// des photos voisines dans le document restent voisines dans la page, donc dans un dessin.
    libres: Vec<u32>,
    occupees: u32,
}

/// Les pages, et le numéro qui distingue chaque liaison, page ou texture seule.
#[derive(Default)]
pub(super) struct Atlas {
    /// Rangées par lot, et non dans une table à l'ordre tiré au sort : la page la plus ancienne se
    /// remplit d'abord, ce qui garde les suites de dessin longues et l'épreuve répétable.
    pages: BTreeMap<u64, Page>,
    prochain_lot: u64,
}

impl Atlas {
    /// Un numéro de lot neuf : deux liaisons différentes n'en partagent jamais.
    pub fn lot_neuf(&mut self) -> u64 {
        self.prochain_lot += 1;
        self.prochain_lot
    }

    /// **Rend une cellule**, et la page entière si elle était la dernière.
    pub fn liberer(&mut self, cellule: Cellule) {
        let Some(page) = self.pages.get_mut(&cellule.lot) else {
            return;
        };
        page.libres.push(cellule.rang);
        page.occupees -= 1;
        if page.occupees == 0 {
            self.pages.remove(&cellule.lot);
        }
    }

    /// Une page de ce côté de cellule qui a de la place, s'il y en a une.
    fn une_place(&mut self, cote: u32) -> Option<(u64, u32)> {
        self.pages
            .iter_mut()
            .filter(|(_, page)| page.cote == cote)
            .find_map(|(lot, page)| {
                let rang = page.libres.pop()?;
                page.occupees += 1;
                Some((*lot, rang))
            })
    }

    /// **Ce que les pages occupent sans que personne s'en serve** : les cellules libres. Une page
    /// ne se rend qu'entière, et cette place-là, la carte la tient bel et bien.
    pub fn octets_vides(&self) -> u64 {
        self.pages
            .values()
            .map(|p| p.libres.len() as u64 * u64::from(p.cote) * u64::from(p.cote) * 4)
            .sum()
    }

    /// Tout part : la carte rend sa mémoire (MEMOIRE-2).
    pub fn vider(&mut self) {
        self.pages.clear();
    }
}

impl SceneGpu {
    /// **Range cette texture dans une page, si elle y tient.** Rend `false` si elle est trop
    /// grande : elle garde alors sa texture à elle.
    pub(super) fn televerser_dans_l_atlas(
        &mut self,
        peripherique: &wgpu::Device,
        envoi: &mut crate::present::envoi::Envoi,
        (identite, cle): (&Cle, &Cle),
        source: tiny_skia::PixmapRef<'_>,
    ) -> bool {
        let (l, h) = (source.width(), source.height());
        let Some(cote) = cote_de_cellule(l, h).filter(|_| !self.sans_atlas) else {
            return false;
        };
        // L'identité rend sa cellule d'abord : un changement de palier la reprend aussitôt.
        self.oublier_la_texture(identite);
        let (lot, rang) = match self.atlas.une_place(cote) {
            Some(place) => place,
            None => {
                let lot = self.atlas.lot_neuf();
                let page = self.page_neuve(peripherique, cote);
                self.atlas.pages.insert(lot, page);
                let Some(place) = self.atlas.une_place(cote) else {
                    return false;
                };
                place
            }
        };
        let Some(page) = self.atlas.pages.get(&lot) else {
            return false;
        };
        let liaison = page.liaison.clone();
        envoi.texture(
            &page.texture,
            coin_de(rang, cote),
            (l, h),
            (source.data(), l as usize * 4),
        );
        let vue = self.image;
        self.photos.insert(
            identite.clone(),
            Televersee {
                liaison,
                lot,
                cellule: Some(Cellule {
                    lot,
                    rang,
                    cote,
                    largeur: l,
                    hauteur: h,
                }),
                cle: cle.clone(),
                vue,
                octets: octets_d_une_texture(l, h),
                photo: false,
            },
        );
        true
    }

    fn page_neuve(&self, peripherique: &wgpu::Device, cote: u32) -> Page {
        let texture = peripherique.create_texture(&wgpu::TextureDescriptor {
            label: Some("page de l'atlas"),
            size: wgpu::Extent3d {
                width: cote * PAR_COTE,
                height: cote * PAR_COTE,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let vue = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let liaison = peripherique.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("page de l'atlas"),
            layout: &self.disposition_photo,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&vue),
            }],
        });
        Page {
            texture,
            liaison,
            cote,
            libres: (0..PAR_COTE * PAR_COTE).rev().collect(),
            occupees: 0,
        }
    }

    /// **Oublie la texture de cette identité**, et rend sa cellule si elle en avait une. Seul
    /// chemin qui retire : une cellule perdue serait de la mémoire qu'aucun compte ne voit.
    pub(super) fn oublier_la_texture(&mut self, identite: &Cle) {
        let Some(partie) = self.photos.remove(identite) else {
            return;
        };
        // Une texture du lot n'est plus là : ses poses ne valent plus rien (LOT-1). Le lot dit
        // lesquelles sont à lui (LOT-2) : la minimap qui change pendant un déplacement n'en est pas.
        if self.lot.as_ref().is_some_and(|lot| lot.pose(identite)) {
            self.lot = None;
        }
        if let Some(cellule) = partie.cellule {
            self.atlas.liberer(cellule);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pose() -> Pose {
        Pose {
            x: 0.0,
            y: 0.0,
            largeur: 10.0,
            hauteur: 10.0,
            opacite: 1.0,
            angle: 0.0,
            fenetre: Pose::TOUT,
            bornes: Pose::PARTOUT,
        }
    }

    #[test]
    fn test_atlas_1_les_cellules_ne_se_chevauchent_pas_et_tiennent_dans_la_page() {
        for cote in [1, 2, 4, 8, 16, 32, 64] {
            let mut vus = std::collections::HashSet::new();
            for rang in 0..PAR_COTE * PAR_COTE {
                let (x, y) = coin_de(rang, cote);
                assert!(x + cote <= cote * PAR_COTE && y + cote <= cote * PAR_COTE);
                assert!(vus.insert((x, y)), "deux cellules au même endroit");
            }
        }
    }

    /// **La plus grande page tient sur toute carte** : le minimum que `wgpu` garantit.
    #[test]
    fn test_atlas_1_la_plus_grande_page_est_acceptee_partout() {
        assert!(
            COTE_MAX * PAR_COTE
                <= wgpu::Limits::downlevel_webgl2_defaults().max_texture_dimension_2d
        );
    }

    /// **Une cellule est la plus petite puissance de deux qui contient la texture**, et ce qu'elle
    /// occupe est ce que le compte dit : une photo réduite ne se croit pas gratuite.
    #[test]
    fn test_atlas_1_une_texture_compte_sa_cellule() {
        assert_eq!(octets_d_une_texture(4, 4), 4 * 4 * 4);
        assert_eq!(
            octets_d_une_texture(3, 1),
            4 * 4 * 4,
            "le plus grand côté décide"
        );
        assert_eq!(octets_d_une_texture(40, 60), 64 * 64 * 4);
        assert_eq!(octets_d_une_texture(1, 1), 4);
        assert_eq!(
            octets_d_une_texture(65, 10),
            65 * 10 * 4,
            "trop grande pour une page : sa texture à elle"
        );
    }

    /// **Le filtre ne lit jamais la voisine** : les bornes s'arrêtent au demi-texel de la texture
    /// posée, même si elle ne remplit pas sa cellule.
    #[test]
    fn test_atlas_1_les_bornes_restent_dans_la_texture_posee() {
        let cote = 64;
        let c = Cellule {
            lot: 1,
            rang: PAR_COTE + 1,
            cote,
            largeur: 20,
            hauteur: 10,
        };
        let p = c.dans_la_page(pose());
        let page = (cote * PAR_COTE) as f32;
        let (cx, cy) = (cote as f32, cote as f32);
        assert!((p.fenetre[0] - cx / page).abs() < 1e-6);
        assert!((p.fenetre[1] - cy / page).abs() < 1e-6);
        assert!((p.fenetre[2] - 20.0 / page).abs() < 1e-6);
        assert!((p.fenetre[3] - 10.0 / page).abs() < 1e-6);
        assert!((p.bornes[0] - (cx + 0.5) / page).abs() < 1e-6);
        assert!((p.bornes[1] - (cy + 0.5) / page).abs() < 1e-6);
        assert!((p.bornes[2] - (cx + 19.5) / page).abs() < 1e-6);
        assert!((p.bornes[3] - (cy + 9.5) / page).abs() < 1e-6);
    }

    /// **Un recadrage garde son sens** : sa fenêtre se lit dans la cellule, et ses bornes aussi.
    #[test]
    fn test_atlas_1_un_recadrage_se_lit_dans_la_cellule() {
        let cote = 64;
        let c = Cellule {
            lot: 1,
            rang: 0,
            cote,
            largeur: 40,
            hauteur: 40,
        };
        let p = c.dans_la_page(Pose {
            fenetre: [0.5, 0.25, 0.5, 0.5],
            bornes: [0.5, 0.25, 1.0, 0.75],
            ..pose()
        });
        let page = (cote * PAR_COTE) as f32;
        assert!((p.fenetre[0] - 20.0 / page).abs() < 1e-6);
        assert!((p.fenetre[1] - 10.0 / page).abs() < 1e-6);
        assert!((p.fenetre[2] - 20.0 / page).abs() < 1e-6);
        assert!(
            (p.bornes[2] - 39.5 / page).abs() < 1e-6,
            "le bord de la texture borne encore"
        );
        assert!(
            (p.bornes[3] - 30.0 / page).abs() < 1e-6,
            "le recadrage borne aussi"
        );
    }
}
