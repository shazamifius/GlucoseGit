//! **La vie de l'application sous Android** (VIE-1, fiche 56) : le système reprend la
//! surface quand Glucose passe derrière une autre application, et la rend au retour.
//!
//! C'est le chemin même du partage : on est dans Pinterest, on partage une image vers Glucose,
//! et Glucose revient au premier plan avec une surface neuve. La présentation garde la carte
//! et ses textures, et ne refait que la surface ([`crate::present::Presenter`]) ; entre les
//! deux, Glucose ne dessine rien — il n'y a personne pour voir.
//!
//! « En arrière-plan » n'est pas un état que l'application tient à côté : c'est « la
//! présentation n'a pas de surface ». Une seule vérité, qui ne peut pas diverger.
//!
//! Le document, lui, n'a rien à faire ici : il s'écrit déjà geste après geste, et un système
//! qui tue l'application en arrière-plan ne lui fait rien perdre.

use super::GlucoseApp;
use std::num::NonZeroU32;

impl GlucoseApp {
    /// **Glucose est-il derrière une autre application ?** La présentation le sait : elle n'a
    /// plus de surface.
    pub(crate) fn en_arriere_plan(&self) -> bool {
        self.presenter.as_ref().is_some_and(|p| !p.a_sa_surface())
    }

    /// **Le système reprend la surface.**
    pub(super) fn suspendre(&mut self) {
        if let Some(presenter) = self.presenter.as_mut() {
            presenter.lacher_la_surface();
        }
        println!("[Glucose] vie : en arriere-plan, la surface est rendue au systeme");
    }

    /// **Le système rend une surface** : la présentation s'y accorde, ou une neuve s'ouvre ;
    /// puis toute la vue se repeint — la surface neuve ne porte rien.
    pub(super) fn reprendre(&mut self) {
        if !self.en_arriere_plan() {
            return;
        }
        let Some(window) = self.window.clone() else {
            return;
        };
        let debut = std::time::Instant::now();
        let rendue = match self.presenter.as_mut() {
            Some(presenter) => presenter.retrouver_la_surface(&window),
            None => Err(crate::error::DesktopError::WindowError(
                "aucune presentation".into(),
            )),
        };
        if let Err(e) = rendue {
            eprintln!("[Glucose] vie : {e} -- une presentation neuve s'ouvre");
            let taille = window.inner_size();
            let w = NonZeroU32::new(taille.width).unwrap_or(NonZeroU32::MIN);
            let h = NonZeroU32::new(taille.height).unwrap_or(NonZeroU32::MIN);
            match self.presentation_neuve(&window, w, h) {
                Ok(neuve) => self.presenter = Some(neuve),
                Err(e) => eprintln!("[Glucose] vie : aucune presentation possible : {e}"),
            }
        }
        self.mark_dirty();
        println!(
            "[Glucose] vie : au premier plan, la surface revenue en {} ms",
            debut.elapsed().as_millis()
        );
    }
}

impl GlucoseApp {
    /// **Les marges que le système vient de dire** (BORD-1, fiche 57) — la barre d'état, la
    /// navigation, le clavier qui sort ou rentre : l'interface s'y cale, et la ligne qu'on
    /// écrit reste au-dessus du clavier.
    pub(crate) fn suivre_les_marges(&mut self) {
        if let Some(marges) = crate::plateforme::marges::nouvelles() {
            self.adopter_les_marges(marges);
        }
    }

    /// Ces marges deviennent celles de l'interface.
    pub(crate) fn adopter_les_marges(&mut self, marges: crate::plateforme::marges::Marges) {
        if self.ui.marges == marges {
            return;
        }
        // Le journal le dit : ce que chaque téléphone annonce ne se devine pas ailleurs.
        println!(
            "[Glucose] marges : haut {}, bas {}, gauche {}, droite {}, clavier {}",
            marges.haut, marges.bas, marges.gauche, marges.droite, marges.clavier
        );
        self.ui.marges = marges;
        self.mark_dirty();
        self.garder_la_ligne_en_vue();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::present::{Issue, Presenter};
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    /// Une présentation qui compte les surfaces qu'on lui fait lâcher, et dit si elle en tient.
    struct Temoin(Arc<AtomicUsize>, bool);

    impl Presenter for Temoin {
        fn resize(&mut self, _: NonZeroU32, _: NonZeroU32) -> crate::error::DesktopResult<()> {
            Ok(())
        }
        fn present(&mut self, _: &tiny_skia::Pixmap) -> crate::error::DesktopResult<Issue> {
            Ok(Issue::Presentee)
        }
        fn lacher_la_surface(&mut self) {
            self.0.fetch_add(1, Ordering::Relaxed);
            self.1 = false;
        }
        fn a_sa_surface(&self) -> bool {
            self.1
        }
        fn nom(&self) -> &'static str {
            "temoin"
        }
        fn rythme(&self) -> &'static str {
            "aucun"
        }
    }

    /// **VIE-1** : quand le système reprend la surface, la présentation la lâche — et
    /// l'application se sait en arrière-plan, et cesse de dessiner jusqu'au retour.
    #[test]
    fn test_suspended_releases_the_surface_and_stops_drawing() {
        let lachees = Arc::new(AtomicUsize::new(0));
        let mut app = GlucoseApp::new();
        app.presenter = Some(Box::new(Temoin(lachees.clone(), true)));
        assert!(!app.en_arriere_plan());
        app.suspendre();
        assert_eq!(
            lachees.load(Ordering::Relaxed),
            1,
            "la surface doit être lâchée"
        );
        assert!(
            app.en_arriere_plan(),
            "plus rien ne se dessine en arrière-plan"
        );
    }
}
