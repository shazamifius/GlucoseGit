//! **Tes raccourcis** (P4, fiche 74) : la couche de l'utilisateur, au-dessus de celle de
//! Glucose — lue sur le disque au lancement, changée dans Glucose, écrite à chaque changement.
//!
//! # Comment on la change
//!
//! « Changer un raccourci… » (la palette, le menu) ouvre la palette pour **choisir** la commande ;
//! la commande choisie pose une question : *« Appuie sur le nouvel accord »*. La touche frappée,
//! lue par la loi même des raccourcis (ACCORD-1, les chiffres par leur touche physique), devient
//! **son seul** raccourci ; si l'accord menait ailleurs, le message le dit — comme VS Code marque
//! un accord lié deux fois. Les boutons de la question retirent son raccourci, ou rendent celui de
//! Glucose ; Échap renonce. Au doigt, sans clavier, les boutons suffisent.
//!
//! # Où elle vit, et ce que Glucose n'écrase jamais
//!
//! Un texte, `raccourcis.txt`, dans le dossier de l'application — le format même de la couche
//! de Glucose (`assets/raccourcis/glucose.txt`), qu'un inconnu lit et modifie. Il s'écrit à
//! chaque changement, comme les préférences de Blender, d'un bloc (SAVE-1). Mais si le fichier
//! existe et **ne se lit pas**, ou porte des lignes fausses écrites à la main, Glucose ne le
//! réécrit pas : le changement vaut pour la session, et un message dit pourquoi. Seul « Rétablir
//! tous les raccourcis de Glucose », qu'on confirme, le remplace.

use super::{clavier_de_glucose, Commande};
use crate::app::GlucoseApp;
use crate::ui::question::{Question, Reponse, Suite};
use glucose_core::raccourcis::{Accord, Clavier, Couche};
use winit::keyboard::{Key, NamedKey};

/// Le nom du fichier, dans le dossier de l'application.
pub const FICHIER: &str = "raccourcis.txt";

/// Le nom de la couche de l'utilisateur.
pub const COUCHE: &str = "utilisateur";

/// Ce que le fichier dit de lui-même, au-dessus de ses liaisons.
const EN_TETE: &str = "# Tes raccourcis, posés sur ceux de Glucose (fiche 74).\n\
# Une ligne : un accord, puis la commande — son identifiant, que la palette trouve aussi ;\n\
# plusieurs, séparées par |, la première qui s'applique l'emportant ; - retire l'accord.\n\
# Glucose réécrit ce fichier quand tu changes un raccourci dans Glucose : tes commentaires\n\
# n'y survivent pas, tes liaisons si. Une ligne fausse l'en empêche, jusqu'à ce qu'elle soit\n\
# corrigée.\n\n";

/// Les réponses de la question du raccourci.
const RETIRER: Reponse = Reponse::Choix(0);
const RETABLIR: Reponse = Reponse::Choix(1);

impl GlucoseApp {
    /// **Lit ses raccourcis** sur le disque, et les pose sur ceux de Glucose. Un fichier absent
    /// est une couche vide. Ce qui ne va pas se dit **en un seul message** — un second effacerait
    /// le premier — ; un fichier qui ne se lit pas, ou porte des lignes fausses, devient de plus
    /// intouchable jusqu'au lancement suivant.
    pub(crate) fn charger_les_raccourcis(&mut self) {
        let (texte, illisible) = match std::fs::read_to_string(&self.disque.raccourcis) {
            Ok(texte) => (texte, None),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => (String::new(), None),
            Err(e) => (
                String::new(),
                Some(format!("le fichier ne se lit pas ({e})")),
            ),
        };
        let (couche, erreurs) = Couche::lire(COUCHE, &texte);
        let fausses = erreurs.first().map(|e| {
            let autres = erreurs.len() - 1;
            let et = if autres > 0 {
                format!(" (et {autres} autre(s))")
            } else {
                String::new()
            };
            format!("ligne {} : {}{et}", e.ligne, e.raison)
        });
        self.disque.raccourcis_proteges = illisible.clone().or_else(|| fausses.clone());
        let inconnues: Vec<String> = couche
            .commandes()
            .filter(|id| Commande::de_l_id(id).is_none())
            .map(|id| format!("« {id} »"))
            .collect();
        let inconnues = (!inconnues.is_empty())
            .then(|| format!("Glucose ne connaît pas {}", inconnues.join(", ")));
        let dits: Vec<String> = [illisible, fausses, inconnues]
            .into_iter()
            .flatten()
            .collect();
        if !dits.is_empty() {
            self.ui
                .show_toast(format!("Tes raccourcis ({FICHIER}) : {}", dits.join(" ; ")));
        }
        let mut couches = clavier_de_glucose().0.couches().to_vec();
        if let Some(dessus) = couches.last_mut() {
            *dessus = couche;
        }
        self.ui.clavier = Clavier::de(couches);
    }

    /// **Écrit ses raccourcis** — sauf si le fichier est à protéger : le changement vaut alors
    /// pour la session, et le message le dit.
    fn ecrire_les_raccourcis(&mut self, message: String) {
        // Protégé, écrit, ou refusé par le disque : la même phrase, dite au même endroit (le
        // cliquet 2 des messages).
        let dit = if let Some(raison) = &self.disque.raccourcis_proteges {
            format!("{message} — pour cette session : {raison}")
        } else {
            let couche = self.ui.clavier.couches().last().map(Couche::ecrire);
            let texte = format!("{EN_TETE}{}", couche.unwrap_or_default());
            let chemin = &self.disque.raccourcis;
            let ecrit = chemin
                .parent()
                .map_or(Ok(()), std::fs::create_dir_all)
                .and_then(|()| crate::persist::atomic::ecrire_d_un_bloc(chemin, texte.as_bytes()));
            match ecrit {
                Ok(()) => message,
                Err(e) => format!("{message} — mais il n'a pas pu s'enregistrer : {e}"),
            }
        };
        self.ui.show_toast(dit);
    }

    /// **La question du raccourci de cette commande** : la touche qu'on frappe le donne.
    pub(crate) fn demander_le_raccourci(&mut self, commande: Commande) {
        let actuels = self.ui.clavier.accords_de(&commande.id());
        let aujourd_hui = if actuels.is_empty() {
            "elle n'en a pas".to_string()
        } else {
            actuels
                .iter()
                .map(Accord::dire)
                .collect::<Vec<_>>()
                .join(", ")
        };
        let question = Question {
            titre: format!("Raccourci de « {} »", commande.nom()),
            texte: format!(
                "Appuie sur le nouvel accord. Aujourd'hui : {aujourd_hui}. Échap renonce."
            ),
            choix: vec![
                ("Retirer ce raccourci".into(), RETIRER),
                ("Rétablir celui de Glucose".into(), RETABLIR),
                ("Annuler".into(), Reponse::Annuler),
            ],
            focus: 2,
            ..Default::default()
        };
        self.demander(question, Suite::Raccourci(commande));
    }

    /// **La touche frappée pendant la question du raccourci** devient l'accord de sa commande.
    /// Un modificateur seul attend la touche ; Échap seul renonce. Rend `true` si la question
    /// du raccourci était posée : elle prend alors toute touche.
    pub(crate) fn prendre_l_accord(&mut self, touche: &Key) -> bool {
        let Some((_, Suite::Raccourci(commande))) = &self.ui.question else {
            return false;
        };
        let commande = *commande;
        let m = self.modifiers;
        let (ctrl, alt, maj) = (m.control_key(), m.alt_key(), m.shift_key());
        if matches!(touche, Key::Named(NamedKey::Escape)) && !(ctrl || alt || maj) {
            self.repondre_a_la_question(Some(Reponse::Annuler));
            return true;
        }
        let Some(t) = crate::interactions::shortcuts::touche_de(touche) else {
            return true;
        };
        let accord = Accord::de(ctrl, alt, maj, t);
        self.ui.question = None;
        let remplacees = self.ui.clavier.lier_seul(&commande.id(), accord);
        let mut message = format!("« {} » : {}", commande.nom(), accord.dire());
        if !remplacees.is_empty() {
            let noms: Vec<String> = remplacees
                .iter()
                .map(|id| Commande::de_l_id(id).map_or(id.clone(), Commande::nom))
                .map(|nom| format!("« {nom} »"))
                .collect();
            message.push_str(&format!(" — à la place de {}", noms.join(", ")));
        }
        self.ecrire_les_raccourcis(message);
        self.mark_dirty();
        true
    }

    /// **Les boutons de la question du raccourci** : retirer, ou rendre celui de Glucose.
    pub(crate) fn suivre_le_raccourci(&mut self, commande: Commande, reponse: Reponse) {
        let id = commande.id();
        if reponse == RETIRER {
            self.ui.clavier.delier(&id);
            let message = format!("« {} » n'a plus de raccourci", commande.nom());
            self.ecrire_les_raccourcis(message);
        } else if reponse == RETABLIR {
            self.ui.clavier.retablir(&id);
            let rendus = self.ui.clavier.accords_de(&id);
            let dits = if rendus.is_empty() {
                "sans raccourci".to_string()
            } else {
                rendus
                    .iter()
                    .map(Accord::dire)
                    .collect::<Vec<_>>()
                    .join(", ")
            };
            let message = format!("« {} » : {dits}, comme Glucose", commande.nom());
            self.ecrire_les_raccourcis(message);
        }
        self.mark_dirty();
    }

    /// **« Rétablir tous les raccourcis de Glucose » demande d'abord** : ses raccourcis seront
    /// oubliés.
    pub(crate) fn demander_a_retablir_les_raccourcis(&mut self) {
        let question = Question {
            titre: "Rétablir tous les raccourcis de Glucose ?".into(),
            texte: "Tes raccourcis seront oubliés, et ton fichier remplacé.".into(),
            choix: vec![
                ("Rétablir".into(), Reponse::Oui),
                ("Annuler".into(), Reponse::Annuler),
            ],
            focus: 1,
            ..Default::default()
        };
        self.demander(question, Suite::RetablirLesRaccourcis);
    }

    /// **Tous les raccourcis de Glucose, rendus** : la couche de l'utilisateur s'oublie, et son
    /// fichier — même protégé : on l'a confirmé — se réécrit vide.
    pub(crate) fn retablir_les_raccourcis(&mut self) {
        self.ui.clavier = clavier_de_glucose().0;
        self.disque.raccourcis_proteges = None;
        self.ecrire_les_raccourcis("Les raccourcis de Glucose sont rétablis".into());
        self.mark_dirty();
    }
}

#[cfg(test)]
mod tests;
