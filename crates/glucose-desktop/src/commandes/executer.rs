//! **Ce que fait chaque commande** — un appel au geste qui existait déjà, et rien d'autre : la
//! table nomme, les modules de gestes font (§ 1.7). Un seul `match`, sans joker : une commande
//! ajoutée sans son geste ne compile pas.

use super::Commande;
use crate::app::GlucoseApp;
use glucose_core::store::StackMove;

impl GlucoseApp {
    /// **Exécute cette commande** ; rend `false` si elle ne s'applique pas — l'accord passe
    /// alors à la suivante de sa liste. Seules les relations des flèches, les opérateurs des
    /// post-it, les couleurs et le groupe peuvent ne pas s'appliquer : sans flèche choisie, un chiffre vole au signet.
    pub(crate) fn executer(&mut self, commande: Commande) -> bool {
        // La fenêtre des liens est passagère : ce qui ne la regarde pas la ferme d'abord, et ne
        // s'exécute que sur le document (LIEN-3, fiche 102).
        if !commande.regarde_la_fenetre() {
            self.fermer_la_fenetre_des_liens();
        }
        self.faire(commande)
    }

    /// **Le geste de chaque commande** : un seul `match`, sans joker.
    fn faire(&mut self, commande: Commande) -> bool {
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
            Commande::Grouper => return self.grouper_la_selection(),
            Commande::ToutChoisir => self.select_all(),
            Commande::Supprimer => self.delete_selection(),
            Commande::PremierPlan => self.restack(StackMove::Front),
            Commande::ArrierePlan => self.restack(StackMove::Back),
            Commande::RetirerLesBordures => self.retirer_les_bordures_de_la_selection(),
            Commande::Verrouiller => self.toggle_lock(),
            Commande::Pousser(sens, dix) => self.nudge(sens.pas(), dix),
            Commande::ToutCadrer => self.cadrer(false),
            Commande::VuePrecedente => return self.revenir(true),
            Commande::VueSuivante => return self.revenir(false),
            Commande::CadrerLaSelection => self.cadrer(true),
            Commande::SuivreLeLien => return self.suivre_le_lien_choisi(),
            Commande::MontrerLesLiens => return self.montrer_les_liens(),
            Commande::AllerAuNoeud => return self.aller_au_noeud_montre(),
            Commande::DisposerEnColonnes => self.disposer_la_selection_en_colonnes(),
            Commande::EntrerDansLaMembrane => self.entrer_dans_la_membrane(),
            Commande::SortirDeLaMembrane => {
                self.sortir_de_la_membrane();
            }
            Commande::ModeReference => self.basculer_le_mode_reference(),
            Commande::Naviguer => self.enter_pan_mode(),
            Commande::Echap => self.escape_gesture(),
            Commande::PoserLeSignet(n) => self.geste_du_signet(n, true),
            Commande::AllerAuSignet(n) => self.geste_du_signet(n, false),
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
            Commande::Operateur(voulu) => return self.poser_l_operateur(voulu),
            Commande::Couleur(rang) => return self.colorer_la_selection(rang),
            Commande::RoueDeCouleur => return self.ouvrir_la_roue(),
            Commande::Chercher => self.ouvrir_la_palette(super::palette::But::Chercher),
            Commande::ResultatSuivant => self.resultat_voisin(1),
            Commande::ResultatPrecedent => self.resultat_voisin(-1),
            Commande::Palette => self.ouvrir_la_palette(super::palette::But::Lancer),
            Commande::ChangerUnRaccourci => {
                self.ouvrir_la_palette(super::palette::But::ChangerLeRaccourci);
            }
            Commande::RetablirLesRaccourcis => self.demander_a_retablir_les_raccourcis(),
            Commande::OublierLesAgents => self.demander_a_oublier_les_agents(),
        }
        self.mark_dirty();
        true
    }

    /// Poser un signet, ou y voler, et dire ce qui a eu lieu.
    fn geste_du_signet(&mut self, n: u8, poser: bool) {
        let message = if poser {
            self.poser_le_signet(chiffre(n))
        } else {
            self.voler_vers_le_signet(chiffre(n))
        };
        self.annoncer_le_signet(message);
    }
}

/// Le chiffre d'un signet, tel que ses gestes le nomment.
fn chiffre(n: u8) -> char {
    char::from_digit(u32::from(n), 10).unwrap_or('1')
}
