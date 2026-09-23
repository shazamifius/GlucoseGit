//! Où une photo se pose à l'écran : la boîte, l'angle, l'opacité — et la fenêtre de la source
//! qu'elle montre (RECADRAGE-1).
//!
//! Extrait de [`super`], qui portait la pose ET ce que la carte détient : deux raisons de
//! changer, et six cent onze lignes là où la fiche 05 en admet six cents. La coupure tombe là
//! où la nature du sujet change — ici la géométrie d'un quad, là-bas la mémoire de la carte.

/// Où une photo se pose à l'écran, et avec quelle opacité.
#[derive(Debug, Clone, Copy)]
pub struct Pose {
    /// Le coin haut-gauche, en pixels d'écran.
    pub x: f32,
    pub y: f32,
    pub largeur: f32,
    pub hauteur: f32,
    pub opacite: f32,
    /// L'angle, en radians, autour du **centre** de la photo.
    pub angle: f32,
    /// **La fenêtre de la source que ce quad montre**, en fractions de la texture : le coin
    /// haut-gauche puis la taille (RECADRAGE-1). [`Pose::TOUT`] est la texture entière.
    ///
    /// C'est ce qui rend le recadrage gratuit sur cette voie : la boîte ne change pas, le
    /// nuanceur lit un sous-rectangle, et pas un pixel n'est écrit de plus ni de moins.
    pub fenetre: [f32; 4],
    /// **Les coordonnées de texture que le nuanceur a le droit de lire** : `u` et `v`
    /// minimaux, puis maximaux (BORDURES-4). [`Pose::PARTOUT`] ne borne rien de plus que la
    /// texture elle-même.
    ///
    /// La fenêtre dit **quoi montrer** ; les bornes disent **quoi lire**. Elles diffèrent d'un
    /// demi-texel, et ce demi-texel est tout le liseré : au bord d'une fenêtre, le filtre
    /// lisse lirait le texel voisin, c'est-à-dire ce que le recadrage a retiré.
    pub bornes: [f32; 4],
}

/// Ce qu'une pose occupe dans le tampon : quatre `vec4`, l'alignement d'un élément de tableau.
pub(super) const OCTETS_POSE: usize = 64;

impl Pose {
    /// La texture entière : aucun recadrage.
    pub const TOUT: [f32; 4] = [0.0, 0.0, 1.0, 1.0];

    /// Aucune borne de plus que la texture : ce qu'un composant, jamais recadré, emploie.
    pub const PARTOUT: [f32; 4] = [0.0, 0.0, 1.0, 1.0];

    /// **Les bornes qu'un recadrage donne à un niveau** `niveau` d'une image `native`, réduit
    /// `facteur` fois : le centre du premier et du dernier texel lisibles, selon la règle du
    /// noyau ([`glucose_core::types::Recadrage::texels_lisibles`]) — la même que la voie
    /// processeur suit, écrite une fois. Le filtre, ramené dans ces bornes, ne lit aucun autre
    /// texel, et un texel réduit qui mêlerait ce qu'on garde à ce qu'on a coupé n'est pas lu.
    pub fn bornes_de(
        crop: glucose_core::types::Recadrage,
        native: (u32, u32),
        (facteur, niveau): (u32, (u32, u32)),
    ) -> [f32; 4] {
        let [g, haut, d, bas] = crop.texels_lisibles(native, facteur);
        let (l, h) = (niveau.0 as f32, niveau.1 as f32);
        [
            (g as f32 + 0.5) / l,
            (haut as f32 + 0.5) / h,
            (d as f32 + 0.5) / l,
            (bas as f32 + 0.5) / h,
        ]
    }

    /// La fenêtre qu'un recadrage du modèle donne à ce quad.
    pub fn fenetre_de(crop: glucose_core::types::Recadrage) -> [f32; 4] {
        let (g, h, _, _) = crop.marges();
        [
            g as f32,
            h as f32,
            crop.largeur_visible() as f32,
            crop.hauteur_visible() as f32,
        ]
    }

    pub(super) fn ecrire(&self, dans: &mut Vec<u8>) {
        for v in [
            self.x,
            self.y,
            self.largeur,
            self.hauteur,
            self.opacite,
            self.angle,
            0.0,
            0.0,
            self.fenetre[0],
            self.fenetre[1],
            self.fenetre[2],
            self.fenetre[3],
            self.bornes[0],
            self.bornes[1],
            self.bornes[2],
            self.bornes[3],
        ] {
            dans.extend_from_slice(&v.to_le_bytes());
        }
    }
}
