//! **Ce que fait chaque commande** — un appel au geste qui existait déjà, et rien d'autre : la
//! table nomme, les modules de gestes font (§ 1.7). Un seul `match`, sans joker : une commande
//! ajoutée sans son geste ne compile pas.

use super::Commande;
use crate::app::GlucoseApp;
use glucose_core::store::StackMove;

impl GlucoseApp {
    /// **Exécute cette commande** ; rend `false` si elle ne s'applique pas — l'accord passe
    /// alors à la suivante de sa liste. Seules les relations des flèches peuvent ne pas
    /// s'appliquer : sans flèche choisie, un chiffre vole au signet.
    pub(crate) fn executer(&mut self, commande: Commande) -> bool {
        match commande {
            Commande::NouveauDocument => self.nouveau_document(),
            Commande::OuvrirUnDocument => self.choisir_un_document(),
            Commande::Enregistrer => self.enregistrer_depuis_le_menu(),
            Commande::EnregistrerSous => self.enregistrer_sous_depuis_le_menu(),
            Commande::Exporter => self.export_board(),
            Commande::ImporterDesImages => self.pick_and_import_images(),
            Commande::TimeMachine => self.basculer_la_machine(),
            Commande::Annuler => self.annuler(),
            Commande::Retablir => self.retablir(),
            Commande::Copier => self.copy_selection(false),
            Commande::Couper => self.copy_selection(true),
            Commande::Coller => self.paste_from_clipboard(),
            Commande::Dupliquer => self.duplicate_selection(),
            Commande::ToutChoisir => self.select_all(),
            Commande::Supprimer => self.delete_selection(),
            Commande::PremierPlan => self.restack(StackMove::Front),
            Commande::ArrierePlan => self.restack(StackMove::Back),
            Commande::RetirerLesBordures => self.retirer_les_bordures_de_la_selection(),
            Commande::Verrouiller => self.toggle_lock(),
            Commande::Pousser(sens, dix) => self.nudge(sens.pas(), dix),
            Commande::ToutCadrer => self.cadrer(false),
            Commande::CadrerLaSelection => self.cadrer(true),
            Commande::ModeReference => self.basculer_le_mode_reference(),
            Commande::Naviguer => self.enter_pan_mode(),
            Commande::Echap => self.escape_gesture(),
            Commande::PoserLeSignet(n) => {
                let message = self.poser_le_signet(chiffre(n));
                self.annoncer_le_signet(message);
            }
            Commande::AllerAuSignet(n) => {
                let message = self.voler_vers_le_signet(chiffre(n));
                self.annoncer_le_signet(message);
            }
            Commande::Outil(outil) => {
                self.ui.active_tool = outil;
                self.update_cursor();
            }
            Commande::EnregistrerLImage => self.enregistrer_l_image_sous(),
            Commande::RemplacerParLImage => self.remplacer_les_liens_par_leur_image(),
            Commande::StoryboardDeLaSelection => self.storyboard_de_la_selection(),
            Commande::LireLeStoryboard => {
                let depuis = self.ui.storyboard.choisie.clone();
                self.lire_le_storyboard(depuis.as_deref());
            }
            Commande::ExporterLesPlanches => self.exporter_les_planches(),
            Commande::Signets => self.montrer_les_signets(),
            Commande::NouvelOnglet => self.ajouter_un_onglet(),
            Commande::ImporterUnDocument => self.choisir_un_document_a_ajouter(),
            Commande::JournalTechnique => self.revoir_la_telemetrie(),
            Commande::VoirCeQuiPart => self.voir_ce_qui_part(),
            Commande::Relation(rang) => return self.qualifier_les_fleches(rang),
            Commande::Palette => self.ouvrir_la_palette(),
        }
        self.mark_dirty();
        true
    }
}

/// Le chiffre d'un signet, tel que ses gestes le nomment.
fn chiffre(n: u8) -> char {
    char::from_digit(u32::from(n), 10).unwrap_or('1')
}
