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
}

/// Ce qu'une pose occupe dans le tampon : trois `vec4`, l'alignement d'un élément de tableau.
pub(super) const OCTETS_POSE: usize = 48;

impl Pose {
    /// La texture entière : aucun recadrage.
    pub const TOUT: [f32; 4] = [0.0, 0.0, 1.0, 1.0];

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
        ] {
            dans.extend_from_slice(&v.to_le_bytes());
        }
    }
}
