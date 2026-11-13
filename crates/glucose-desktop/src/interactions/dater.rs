//! **DATE-1 — dater ce qui est choisi** (fiche 113) : `Maj+T`, le menu d'une sélection, la
//! palette. La question de Glucose Tauri (`TemporalAnchorPrompt.tsx`) : un champ, la date du
//! premier nœud daté déjà écrite dedans, les ères qui répondent à ce qu'on tape, « Retirer la
//! date ». Le geste est au noyau ([`glucose_core::store::Store::ancrer_dans_le_temps`]) : un seul
//! `Ctrl+Z` le défait.

use crate::app::GlucoseApp;
use crate::ui::question::{Champ, Question, Reponse, Suite};
use glucose_core::recherche::pli;
use glucose_core::text::Selection;
use glucose_core::timeline::{format_anchor, parse_anchor, NamedEra, DEFAULT_ERAS};

/// **Combien d'ères la question propose** : six quand rien n'est écrit, huit au plus sinon —
/// les nombres de Tauri, des nombres de ressenti (une rangée de puces, une question qui tient à
/// l'écran d'un téléphone).
const ERES_SANS_RIEN_D_ECRIT: usize = 6;
const ERES_AU_PLUS: usize = 8;

/// Ce que la question dit sous son titre : les formes qu'une date peut prendre.
pub(crate) const AIDE: &str = "Une année, une plage ou une ère : « 1789 », « 1789-1799 », \
     « 500 av. J.-C. », « Renaissance », « 10 ka », « 1,5 Ma ».";

impl GlucoseApp {
    /// **Demande la date de ce qui est choisi** ; le champ porte celle du premier nœud daté, tout
    /// sélectionnée — la première lettre tapée la remplace.
    pub(crate) fn demander_la_date(&mut self) {
        let ids: Vec<String> = self
            .store
            .selected_annotation_ids
            .iter()
            .chain(&self.store.selected_image_ids)
            .cloned()
            .collect();
        if ids.is_empty() {
            return;
        }
        let ecrit = self
            .store
            .dates_choisies()
            .into_iter()
            .flatten()
            .next()
            .map(format_anchor)
            .unwrap_or_default();
        let board = self.store.project.active_board_id.clone();
        self.poser_la_date(board, ids, ecrit, AIDE.to_string());
    }

    /// La question, avec `texte` sous son titre — l'aide, ou ce qui n'allait pas.
    fn poser_la_date(&mut self, board: String, ids: Vec<String>, ecrit: String, texte: String) {
        let datee = self.store.dates_choisies().iter().any(Option::is_some);
        let n = ids.len();
        let question = Question {
            titre: if n == 1 {
                "Dater ce nœud".to_string()
            } else {
                format!("Dater ces {n} nœuds")
            },
            texte,
            champ: Some(Champ {
                selection: Selection::all(&ecrit),
                texte: ecrit.clone(),
            }),
            choix: choix(&ecrit, datee),
            ..Default::default()
        };
        self.demander(question, Suite::Dater { board, ids, ecrit });
    }

    /// **La question suit son champ** (chaque image, comme la palette) : les ères proposées
    /// répondent à ce qu'on tape, l'évidence revient sur « Dater ».
    pub(crate) fn suivre_la_date(&mut self) {
        let Some((q, Suite::Dater { ecrit, .. })) = &mut self.ui.question else {
            return;
        };
        let texte = q
            .champ
            .as_ref()
            .map(|c| c.texte.clone())
            .unwrap_or_default();
        if texte == *ecrit {
            return;
        }
        let datee = q.choix.iter().any(|(_, r)| *r == Reponse::Non);
        q.choix = choix(&texte, datee);
        q.focus = 0;
        *ecrit = texte;
        self.mark_dirty();
    }

    /// **La réponse** : la date écrite (« Dater », Entrée), une ère proposée, ou la retirer. Une
    /// date que Glucose ne sait pas lire se redemande, avec ce qu'on avait écrit.
    pub(crate) fn repondre_a_la_date(
        &mut self,
        (board, ids, propose): (String, Vec<String>, String),
        reponse: Reponse,
        champ: Option<String>,
    ) {
        let ecrit = champ.unwrap_or_default();
        let date = match reponse {
            Reponse::Oui => match parse_anchor(&ecrit, DEFAULT_ERAS) {
                Some(date) => Some(date),
                None => {
                    let pourquoi = format!("Glucose ne sait pas lire « {} ». {AIDE}", ecrit.trim());
                    self.poser_la_date(board, ids, ecrit, pourquoi);
                    return;
                }
            },
            // Une ère proposée : celles que montrait la question, calculées de ce qu'elle avait lu.
            Reponse::Choix(i) => match eres(&propose).get(i) {
                Some(ere) => parse_anchor(ere.name, DEFAULT_ERAS),
                None => return,
            },
            Reponse::Non => None,
            Reponse::Annuler | Reponse::Defiler(_) => return,
        };
        self.store.begin_live_edit();
        self.store.ancrer_dans_le_temps(&board, &ids, date.as_ref());
        self.store.end_live_edit();
        self.mark_dirty();
    }
}

/// **Les ères qui répondent à ce qu'on a écrit** — sans majuscules ni accents (PLI-1) ; rien
/// d'écrit, les premières du catalogue.
fn eres(ecrit: &str) -> Vec<&'static NamedEra> {
    let cherche = pli(ecrit.trim());
    if cherche.is_empty() {
        return DEFAULT_ERAS.iter().take(ERES_SANS_RIEN_D_ECRIT).collect();
    }
    DEFAULT_ERAS
        .iter()
        .filter(|e| pli(e.name).contains(&cherche))
        .take(ERES_AU_PLUS)
        .collect()
}

/// Les réponses : « Dater », les ères proposées avec leur période, « Retirer la date » quand
/// quelque chose en porte une, « Annuler ».
fn choix(ecrit: &str, datee: bool) -> Vec<(String, Reponse)> {
    let mut choix = vec![("Dater".to_string(), Reponse::Oui)];
    for (i, ere) in eres(ecrit).into_iter().enumerate() {
        let periode = format_anchor(&glucose_core::types::TemporalAnchor {
            start: ere.start,
            end: ere.end,
            label: None,
        });
        choix.push((format!("{} · {periode}", ere.name), Reponse::Choix(i)));
    }
    if datee {
        choix.push(("Retirer la date".to_string(), Reponse::Non));
    }
    choix.push(("Annuler".to_string(), Reponse::Annuler));
    choix
}

#[cfg(test)]
mod tests;
