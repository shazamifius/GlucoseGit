//! **REVENIR-2 — un document revient dans sa salle** (fiche 148 ; le noyau : fiche 147 § 6).
//!
//! Le fil part de zéro : la salle donne sa base, chacun de ses gestes, puis « à jour ». On les
//! garde ; à jour, `Store::revenir_dans_la_salle` la rebâtit à la borne (l'ancêtre commun) et
//! maintenant, la fusionne avec le document d'ici, et l'on participe. Ce qui n'a changé qu'ici part
//! comme mon geste ; ce qui a changé des deux côtés, la salle l'emporte — et on le dit. Le lancement
//! d'un document resté dedans, et « Partager » un document qui a sa salle, passent par là.

use super::{identite, nouvelle_marque, reprise, CanevasPartage, Entree};
use super::{PAS_DE_RESEAU, SALLE_VIDE};
use crate::app::GlucoseApp;
use glucose_core::salle::code::Code;
use glucose_core::salle::lettre::Corps;
use glucose_core::store::journal::partage::GestePartage;
use glucose_core::store::CeQueLaSalleGarde;
use glucose_core::types::Project;

/// **Ce que la salle a donné** depuis sa base, et la borne d'où l'on revient.
pub struct Retour {
    retenu: u64,
    base: Option<Project>,
    gestes: Vec<(u64, GestePartage)>,
}

impl GlucoseApp {
    /// **Revient dans la salle de ce code** : le fichier en contient tout jusqu'au numéro `retenu`,
    /// au moins. Le fil part de zéro. Rend `false` sans réseau.
    pub(crate) fn revenir_par_le_code(&mut self, code: Code, retenu: u64) -> bool {
        let Ok(marque) = nouvelle_marque() else {
            return false;
        };
        let Some(lien) = self.relier(code.clone(), 0) else {
            return false;
        };
        let moi = identite::Identite::la_sienne(&self.disque.identite);
        let qui = (marque, moi.nom, moi.couleur as u8);
        let objets = std::sync::Arc::clone(&self.disque.objets);
        let mut cp = CanevasPartage::neuf(lien, code, qui, Entree::Revient, objets);
        cp.borne = reprise::Borne::depuis(retenu);
        cp.retour = Some(Box::new(Retour {
            retenu,
            base: None,
            gestes: Vec::new(),
        }));
        self.lancement.agents.partage = Some(cp);
        self.ui.porte_ouverte = true;
        self.retenir_le_canevas_en_cours();
        crate::dire!("[Glucose] canevas partagé : revient dans sa salle (au moins au n° {retenu})");
        true
    }

    /// **« Partager » un document qui a sa salle** (fiche 146 § 4, le 1) : il y revient, sous les
    /// mêmes codes, au lieu d'en ouvrir une seconde. Rend `false` s'il n'en a pas.
    pub(super) fn revenir_dans_sa_salle(&mut self) -> bool {
        let souvenir = (self.document_du_canevas())
            .and_then(|d| reprise::lire(&self.disque.canevas, &d))
            .and_then(|s| Some((Code::lire_le_texte(&s.code)?, s.retenu)));
        let Some((code, retenu)) = souvenir else {
            return false;
        };
        let note = if self.revenir_par_le_code(code, retenu) {
            REVENU
        } else {
            PAS_DE_RESEAU
        };
        self.montrer_collaborer_avec(Some(note.into()));
        true
    }

    /// **Une lettre numérotée, pendant qu'on revient** : la base, puis les gestes. Une base qui
    /// suit la première se confirme comme un geste vide, ainsi qu'à l'intérieur.
    pub(super) fn recueillir(&mut self, n: u64, corps: Corps) {
        let cp = self.lancement.agents.partage.as_mut();
        let Some(r) = cp.and_then(|cp| cp.retour.as_mut()) else {
            return;
        };
        match corps {
            Corps::Base(base) if r.base.is_none() => r.base = Some(*base),
            Corps::Base(_) => r.gestes.push((n, GestePartage::default())),
            Corps::Geste(g) => r.gestes.push((n, g)),
            Corps::Curseur { .. } | Corps::Objet { .. } | Corps::Demande { .. } => {}
        }
    }

    /// **La salle a tout donné** : la fusion, et l'on participe. Sans base, ce code ne mène encore à
    /// aucun canevas : on le dit, et l'on attend encore.
    pub(super) fn revenir_quand_la_salle_est_la(&mut self, n: u64) {
        let Some(cp) = self.lancement.agents.partage.as_mut() else {
            return;
        };
        let (marque, ecrivain) = (cp.marque.clone(), cp.code.peut_ecrire());
        let Some(retour) = cp.retour.take() else {
            return;
        };
        let Some(base) = &retour.base else {
            cp.retour = Some(retour);
            return self.dire_ce_qui_arrive_au_canevas(SALLE_VIDE.into());
        };
        // Jamais pendant un geste ouvert : la fusion se ferait sur un état qui bouge encore.
        self.terminer_les_gestes_en_cours();
        self.consigner();
        let salle = CeQueLaSalleGarde {
            base,
            gestes: &retour.gestes,
            n,
        };
        let rendu = (self.store).revenir_dans_la_salle(salle, retour.retenu, &marque, ecrivain);
        self.mark_dirty();
        let Some(conflits) = rendu else {
            self.lancement.agents.partage = None;
            self.ui.porte_ouverte = self.lancement.agents.publique.is_some();
            return self.dire_ce_qui_arrive_au_canevas(PAS_REVENU.into());
        };
        if let Some(cp) = self.lancement.agents.partage.as_mut() {
            cp.entree = Entree::Dedans;
        }
        crate::dire!(
            "[Glucose] canevas partagé : revenu au n° {n}, {} conflit(s)",
            conflits.len()
        );
        if !conflits.is_empty() {
            self.dire_ce_qui_arrive_au_canevas(format!(
                "De retour dans la salle : {} élément(s) changé(s) ici et là-bas — la salle l'a emporté",
                conflits.len()
            ));
        }
    }
}

const REVENU: &str =
    "Ce canevas revient dans sa salle, sous les mêmes codes : ce qui a changé ici \
                      et là-bas se rejoint.";
const PAS_REVENU: &str = "Ce canevas n'a pas pu revenir dans sa salle : il reste ici, à toi seul.";
