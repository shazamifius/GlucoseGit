//! **BARRE-3 — la barre de Windows, au choix** (fiche 120).
//!
//! Depuis BARRE-1 (fiche 85), sous Windows, la bande de Glucose remplace la barre blanche du
//! système : elle déplace la fenêtre et porte réduire, agrandir, fermer — comme Chrome, Figma ou
//! VS Code. Sa capture du 17/11 : *« plus le fait d'avoir retiré de manière pas super propre la
//! barre blanche Windows […] le combo des deux fait qu'aucun bouton n'est accessible »* — le défaut
//! était le clic décalé (BARRE-2, corrigé). Deux réponses restent légitimes : la bande, qui rend au
//! canevas la hauteur d'une barre, et la barre du système, que d'autres préfèrent. **La bande par
//! défaut, et le choix** (sa parole du 15/10 : *« l'utilisateur doit TOUJOURS CHOISIR »*), retenu
//! d'une session à l'autre et basculé sans relancer.

use crate::app::GlucoseApp;

/// Le fichier qui retient le choix de la barre du système, dans le dossier où l'application habite.
const SOUVENIR: &str = "barre-du-systeme";

impl GlucoseApp {
    /// **La barre du système était-elle choisie** à la fin de la session précédente ?
    pub(crate) fn barre_du_systeme_retenue(&self) -> bool {
        self.souvenir_de_la_barre().exists()
    }

    /// **« Barre de Windows »** (la palette, le menu du vide) : la barre du système revient, ou
    /// repart au profit de la bande — et le choix se retient.
    pub(crate) fn basculer_la_barre_du_systeme(&mut self) {
        let maison = !self.ui.cadre_maison;
        if let Some(fenetre) = &self.window {
            self.ui.cadre_maison = crate::plateforme::cadre::rhabiller(fenetre, maison);
        }
        let souvenir = self.souvenir_de_la_barre();
        // Un souvenir qui ne s'écrit pas coûte la relance, jamais le choix : il ne se dit pas.
        if maison {
            let _ = std::fs::remove_file(&souvenir);
        } else {
            let _ = crate::persist::atomic::write_atomic(&souvenir, b"1");
        }
        // La bande gagne ou perd la place des trois boutons : toute la chrome change.
        self.mark_dirty();
    }

    fn souvenir_de_la_barre(&self) -> std::path::PathBuf {
        self.disque
            .brouillons
            .parent()
            .unwrap_or(std::path::Path::new("."))
            .join(SOUVENIR)
    }
}

#[cfg(test)]
mod tests {
    use crate::persist::disque::tests::{application, dossier};

    /// **Le choix se retient, et le même geste le défait** : sans fenêtre, la bande ne change pas,
    /// mais le souvenir suit le geste.
    #[test]
    fn test_barre_3_le_choix_se_retient_et_se_defait() {
        let d = dossier("barre-du-systeme");
        let mut app = application(&d);
        app.ui.cadre_maison = true;
        assert!(!app.barre_du_systeme_retenue());
        app.basculer_la_barre_du_systeme();
        assert!(
            application(&d).barre_du_systeme_retenue(),
            "retenue pour la relance"
        );
        app.ui.cadre_maison = false;
        app.basculer_la_barre_du_systeme();
        assert!(!application(&d).barre_du_systeme_retenue(), "et oubliée");
    }
}
