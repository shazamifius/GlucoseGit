//! **Les passes que la carte pose autour des photos** — le fond, les lueurs, les zones, le
//! rectangle de la vue sur la minimap, les membranes, les flèches, le liseré —, nées,
//! préparées et rangées dans les cinq temps **en un seul endroit**.
//!
//! # Pourquoi elles vivent ensemble
//!
//! La présentation (`gpu::cinq_temps`) et le banc des épreuves (`banc_gpu`) les faisaient
//! naître et les préparaient chacun de son côté : deux copies de la même liste. Une passe
//! oubliée dans la présentation ne faisait donc tomber aucune épreuve — elles composaient par
//! le banc, qui l'avait. C'est la forme exacte des régressions de la fiche 21, *une passe qui
//! cesse d'être appelée*, et le cadre de la minimap (CHROME-1) l'a montrée en s'ajoutant aux
//! deux copies. Une seule définition : ce que les épreuves composent est ce que l'écran reçoit.

use super::{
    couches, fleches_gpu, fond_gpu, lisere_gpu, lueurs_gpu, membranes_gpu, scene_gpu, zones_gpu,
};
use crate::renderer::Confie;

/// Les passes de la carte, hors photos et couches du processeur.
pub struct Passes {
    fond: fond_gpu::FondGpu,
    lueurs: lueurs_gpu::Lueurs,
    /// Le corps des zones d'un gabarit (PRESET-1).
    zones: zones_gpu::Zones,
    /// Le rectangle de la vue sur la minimap, en filets (CHROME-1) : les mêmes rectangles unis
    /// que les zones, posés à un autre rang.
    cadre: zones_gpu::Zones,
    membranes: membranes_gpu::Membranes,
    fleches: fleches_gpu::FlechesGpu,
    lisere: lisere_gpu::LisereGpu,
}

impl Passes {
    pub fn nouvelles(peripherique: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        Self {
            fond: fond_gpu::FondGpu::nouveau(peripherique, format),
            lueurs: lueurs_gpu::Lueurs::nouvelles(peripherique, format),
            zones: zones_gpu::Zones::nouvelles(peripherique, format),
            cadre: zones_gpu::Zones::nouvelles(peripherique, format),
            membranes: membranes_gpu::Membranes::nouvelles(peripherique, format),
            fleches: fleches_gpu::FlechesGpu::nouvelles(peripherique, format),
            lisere: lisere_gpu::LisereGpu::nouveau(peripherique, format),
        }
    }

    /// **Écrit ce que cette image leur confie.** À faire avant d'ouvrir la passe.
    pub fn preparer(
        &mut self,
        (peripherique, file): (&wgpu::Device, &wgpu::Queue),
        ecran: (f32, f32),
        confie: &Confie,
    ) {
        self.fond.preparer(file, ecran, confie.fond);
        self.lueurs
            .preparer(peripherique, file, ecran, &confie.lueurs);
        self.zones
            .preparer(peripherique, file, ecran, &confie.zones);
        self.cadre
            .preparer(peripherique, file, ecran, confie.chrome.cadre());
        self.membranes
            .preparer(peripherique, file, ecran, &confie.membranes);
        self.fleches
            .preparer(peripherique, file, ecran, &confie.fleches);
        self.lisere.preparer(file, ecran, confie.lisere);
    }

    /// **Les cinq temps de cette image**, avec les photos et les couches du processeur, et les
    /// rangs où la pose des textures se coupe — tous lus sur ce qui a été confié.
    pub fn temps<'a>(
        &'a self,
        (couches, scene, retenues): (
            &'a couches::Couches,
            &'a scene_gpu::SceneGpu,
            &'a scene_gpu::Retenues,
        ),
        confie: &Confie,
    ) -> couches::Temps<'a> {
        couches::Temps {
            fond: &self.fond,
            lueurs: &self.lueurs,
            zones: &self.zones,
            cadre: &self.cadre,
            membranes: &self.membranes,
            fleches: &self.fleches,
            lisere: &self.lisere,
            couches,
            scene,
            retenues,
            photos: confie.photos.len(),
            sous_le_dessus: confie.sous_le_dessus(),
            sous_le_cadre: confie.sous_le_cadre(),
        }
    }
}
