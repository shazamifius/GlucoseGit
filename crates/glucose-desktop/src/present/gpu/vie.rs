//! **La surface qu'Android reprend et rend** (VIE-1, fiche 56).
//!
//! # Ce qui se passait
//!
//! Sous Android, quand Glucose passe derrière une autre application — Pinterest, d'où l'on va
//! justement partager une image vers lui —, le système **détruit** la surface de la fenêtre,
//! et en donne une neuve au retour. `winit` le dit (`Suspended`, `Resumed`) ; Glucose ne
//! l'écoutait pas, et présentait au retour dans une surface qui n'existait plus.
//!
//! # Ce qui part, et ce qui reste
//!
//! Seule la **surface** part. Le périphérique, et tout ce qu'il tient — les textures des
//! photos, les nuanceurs —, reste : refaire tout cela au retour coûterait des secondes sur un
//! téléphone, pendant lesquelles les photos reviendraient floues. C'est ce que font les
//! exemples de `wgpu` et Bevy : une surface par séjour au premier plan, un périphérique pour
//! la vie de l'application.
//!
//! La surface neuve se configure **comme l'ancienne**, à la taille de la fenêtre : les
//! nuanceurs sont bâtis pour son format. Si elle ne l'accepte plus — rien ne l'interdit en
//! principe —, la présentation le dit, et l'application en ouvre une neuve.

use super::GpuPresenter;
use crate::error::{DesktopError, DesktopResult};
use std::sync::Arc;
use winit::window::Window;

impl GpuPresenter {
    /// La surface part ; le périphérique et ses textures restent.
    pub(super) fn lacher(&mut self) {
        self.surface = None;
        self.a_reaccorder = false;
    }

    /// Une surface neuve sur cette fenêtre, sur **le même** périphérique.
    pub(super) fn retrouver(&mut self, fenetre: &Arc<Window>) -> DesktopResult<()> {
        let echec = |quoi: String| DesktopError::WindowError(format!("surface rendue : {quoi}"));
        let surface = self
            .instance
            .create_surface(fenetre.clone())
            .map_err(|e| echec(e.to_string()))?;
        let formats = surface.get_capabilities(&self.adapter).formats;
        if !formats.contains(&self.config.format) {
            return Err(echec(format!(
                "elle refuse le format {:?}",
                self.config.format
            )));
        }
        let taille = fenetre.inner_size();
        self.config.width = taille.width.max(1);
        self.config.height = taille.height.max(1);
        surface.configure(&self.device, &self.config);
        self.surface = Some(surface);
        self.a_reaccorder = false;
        Ok(())
    }
}
