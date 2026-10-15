//! **La palette de commandes** (`Ctrl+Maj+P`, P4, fiche 72) : chaque commande de la table,
//! cherchée par son nom, lancée par Entrée — celle de PureRef 2 et de VS Code.
//!
//! Elle est une **question** de Glucose (QUESTION-1) : un champ où l'on écrit, des réponses —
//! les commandes trouvées, chacune avec son raccourci lu dans le clavier —, Entrée qui lance
//! celle en évidence, Échap qui renonce ; au PC comme au téléphone, à la souris comme au doigt.
//! Ses réponses suivent le champ **à chaque image** ([`GlucoseApp::suivre_la_palette`]), comme le
//! panneau du storyboard suit le document : le clavier du PC et celui du téléphone écrivent par
//! deux chemins, la palette n'en lit qu'un.

use super::Commande;
use crate::app::GlucoseApp;
use crate::ui::question::{Champ, Question, Reponse, Suite};

impl GlucoseApp {
    /// **Ouvre la palette**, le champ vide : toutes les commandes, dans l'ordre de la table.
    pub(crate) fn ouvrir_la_palette(&mut self) {
        let question = Question {
            titre: "Palette de commandes".into(),
            texte: "Écris ce que tu cherches ; Entrée lance la commande en évidence.".into(),
            choix: self.choix_de_la_palette(""),
            champ: Some(Champ::default()),
            ..Default::default()
        };
        self.demander(question, Suite::Palette(String::new()));
    }

    /// **La palette suit son champ** : un texte qui a changé depuis l'image d'avant refait ses
    /// réponses, et l'évidence revient à la première.
    pub(crate) fn suivre_la_palette(&mut self) {
        let ecrit = match &self.ui.question {
            Some((q, Suite::Palette(filtre))) => {
                let ecrit = q
                    .champ
                    .as_ref()
                    .map(|c| c.texte.clone())
                    .unwrap_or_default();
                if ecrit == *filtre {
                    return;
                }
                ecrit
            }
            _ => return,
        };
        let choix = self.choix_de_la_palette(&ecrit);
        if let Some((q, suite)) = &mut self.ui.question {
            q.choix = choix;
            q.focus = 0;
            *suite = Suite::Palette(ecrit);
        }
        self.mark_dirty();
    }

    /// **La réponse de la palette** : la commande trouvée à ce rang pour ce filtre — celui que
    /// les réponses montraient.
    pub(crate) fn lancer_depuis_la_palette(&mut self, filtre: &str, reponse: Reponse) {
        if let Reponse::Choix(i) = reponse {
            if let Some(&commande) = trouvees(filtre).get(i) {
                self.executer(commande);
            }
        }
    }

    /// Les réponses de la palette pour ce filtre : chaque commande trouvée et son raccourci.
    fn choix_de_la_palette(&self, filtre: &str) -> Vec<(String, Reponse)> {
        trouvees(filtre)
            .into_iter()
            .enumerate()
            .map(|(i, c)| {
                let accord = self
                    .ui
                    .clavier
                    .accords_de(&c.id())
                    .first()
                    .map(|a| a.dire());
                let libelle = match accord {
                    Some(a) => format!("{}   {a}", c.nom()),
                    None => c.nom(),
                };
                (libelle, Reponse::Choix(i))
            })
            .collect()
    }
}

/// **Les commandes que ce filtre trouve**, les meilleures d'abord, au plus ce qu'un téléphone
/// tient debout ([`crate::persist::documents::MONTRES`]). La palette ne se trouve pas
/// elle-même.
pub(crate) fn trouvees(filtre: &str) -> Vec<Commande> {
    let mut v: Vec<(u8, usize, Commande)> = Commande::toutes()
        .into_iter()
        .filter(|c| *c != Commande::Palette)
        .enumerate()
        .filter_map(|(rang, c)| Some((rang_de(filtre, &c.nom(), &c.id())?, rang, c)))
        .collect();
    v.sort_by_key(|(qualite, rang, _)| (*qualite, *rang));
    v.into_iter()
        .take(crate::persist::documents::MONTRES)
        .map(|(_, _, c)| c)
        .collect()
}

/// **À quel point ce nom répond au filtre** — 0 s'il commence par lui, 1 si l'un de ses mots
/// commence par lui, 2 s'il le contient, 3 si ses lettres y sont dans l'ordre ; `None` sinon.
/// Sans majuscules ni accents : « retablir » trouve « Rétablir ». L'identifiant compte aussi :
/// celui qui a lu une couche de raccourcis le cherche par là.
pub(crate) fn rang_de(filtre: &str, nom: &str, id: &str) -> Option<u8> {
    let filtre = simple(filtre.trim());
    if filtre.is_empty() {
        return Some(0);
    }
    let (nom, id) = (simple(nom), simple(id));
    if nom.starts_with(&filtre) || id.starts_with(&filtre) {
        return Some(0);
    }
    let mots = |t: &str| {
        t.split(|c: char| !c.is_alphanumeric())
            .any(|m| m.starts_with(&filtre))
    };
    if mots(&nom) || mots(&id) {
        return Some(1);
    }
    if nom.contains(&filtre) || id.contains(&filtre) {
        return Some(2);
    }
    let dans_l_ordre = |t: &str| {
        let mut lettres = t.chars();
        filtre.chars().all(|f| lettres.any(|c| c == f))
    };
    (dans_l_ordre(&nom) || dans_l_ordre(&id)).then_some(3)
}

/// Le texte sans majuscules ni accents du français.
fn simple(t: &str) -> String {
    t.to_lowercase()
        .chars()
        .map(|c| match c {
            'à' | 'â' | 'ä' => 'a',
            'é' | 'è' | 'ê' | 'ë' => 'e',
            'î' | 'ï' => 'i',
            'ô' | 'ö' => 'o',
            'ù' | 'û' | 'ü' => 'u',
            'ç' => 'c',
            'ÿ' => 'y',
            autre => autre,
        })
        .collect()
}
