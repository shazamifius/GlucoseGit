//! **La palette de commandes** (`Ctrl+Maj+P`, P4, fiche 72) : chaque commande de la table,
//! cherchée par son nom, lancée par Entrée — celle de PureRef 2 et de VS Code.
//!
//! Elle est une **question** de Glucose (QUESTION-1) : un champ où l'on écrit, des réponses —
//! les commandes trouvées, chacune avec son raccourci lu dans le clavier —, Entrée qui lance
//! celle en évidence, Échap qui renonce ; au PC comme au téléphone, à la souris comme au doigt.
//! Ses réponses suivent le champ **à chaque image** ([`GlucoseApp::suivre_la_palette`]), comme le
//! panneau du storyboard suit le document : le clavier du PC et celui du téléphone écrivent par
//! deux chemins, la palette n'en lit qu'un.
//!
//! # Tout se voit, en défilant (sa demande du 15/10)
//!
//! *« Ce serait bien que dans la palette de commandes on ait ce truc-là de pouvoir scroller et
//! voir tout. »* La palette montre une **fenêtre** de [`MONTRES`] commandes sur toutes celles
//! qu'elle trouve ([`Etat`]) : la molette et le pavé la font défiler d'une rangée par cran, les
//! flèches et Tab déplacent l'évidence et la fenêtre la suit, Page haut et Page bas sautent
//! d'une fenêtre ; au doigt, deux rangées « ↑ n plus haut » et « ↓ n plus bas » se touchent.

use super::Commande;
use crate::app::GlucoseApp;
use crate::persist::documents::MONTRES;
use crate::ui::question::{Champ, Question, Reponse, Suite};

mod defiler;

/// **L'état de la palette** : le filtre que ses rangées montrent, la première commande de sa
/// fenêtre, et celle en évidence — des rangs dans toutes les commandes trouvées.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Etat {
    pub filtre: String,
    pub debut: usize,
    pub choisie: usize,
    /// Ce que la commande choisie devient : lancée, ou celle dont on change le raccourci.
    pub but: But,
}

/// **Ce que la palette fait de la commande choisie** (P4, fiche 74).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum But {
    /// La lancer.
    #[default]
    Lancer,
    /// Changer son raccourci : la palette sert à la trouver, toutes les commandes comprises.
    ChangerLeRaccourci,
}

impl GlucoseApp {
    /// **Ouvre la palette**, le champ vide : toutes les commandes, dans l'ordre de la table.
    pub(crate) fn ouvrir_la_palette(&mut self, but: But) {
        let etat = Etat {
            but,
            ..Etat::default()
        };
        let (choix, focus) = self.rangees_de_la_palette(&etat);
        let (titre, texte) = match but {
            But::Lancer => (
                "Palette de commandes",
                "Écris ce que tu cherches ; Entrée lance la commande en évidence.",
            ),
            But::ChangerLeRaccourci => (
                "Changer un raccourci",
                "Choisis la commande dont tu veux changer le raccourci.",
            ),
        };
        let question = Question {
            titre: titre.into(),
            texte: texte.into(),
            choix,
            focus,
            champ: Some(Champ::default()),
            ..Default::default()
        };
        self.ui.reste_de_la_molette = 0.0;
        self.demander(question, Suite::Palette(etat));
    }

    /// **La palette suit son champ** : un texte qui a changé depuis l'image d'avant refait ses
    /// rangées depuis le début, l'évidence sur la première.
    pub(crate) fn suivre_la_palette(&mut self) {
        let ecrit = match &self.ui.question {
            Some((q, Suite::Palette(etat))) => {
                let ecrit = q
                    .champ
                    .as_ref()
                    .map(|c| c.texte.clone())
                    .unwrap_or_default();
                if ecrit == etat.filtre {
                    return;
                }
                ecrit
            }
            _ => return,
        };
        let but = match &self.ui.question {
            Some((_, Suite::Palette(etat))) => etat.but,
            _ => But::Lancer,
        };
        self.poser_la_palette(Etat {
            filtre: ecrit,
            debut: 0,
            choisie: 0,
            but,
        });
    }

    /// **Pose cet état** : ses rangées, son évidence, et lui-même dans la question.
    pub(super) fn poser_la_palette(&mut self, etat: Etat) {
        let (choix, focus) = self.rangees_de_la_palette(&etat);
        if let Some((q, suite)) = &mut self.ui.question {
            q.choix = choix;
            q.focus = focus;
            *suite = Suite::Palette(etat);
        }
        self.mark_dirty();
    }

    /// **La réponse de la palette** : la commande de ce rang, parmi celles que ce filtre trouve.
    pub(crate) fn lancer_depuis_la_palette(&mut self, etat: &Etat, reponse: Reponse) {
        if let Reponse::Choix(i) = reponse {
            match (trouvees_pour(&etat.filtre, etat.but).get(i), etat.but) {
                (Some(&commande), But::Lancer) => {
                    self.executer(commande);
                }
                (Some(&commande), But::ChangerLeRaccourci) => self.demander_le_raccourci(commande),
                (None, _) => {}
            }
        }
    }

    /// **Les rangées de la palette** et celle en évidence : « ↑ n plus haut » s'il y a des
    /// commandes au-dessus de la fenêtre, ses commandes avec leur raccourci, « ↓ n plus bas »
    /// s'il en reste dessous.
    fn rangees_de_la_palette(&self, etat: &Etat) -> (Vec<(String, Reponse)>, usize) {
        let toutes = trouvees_pour(&etat.filtre, etat.but);
        let fin = (etat.debut + MONTRES).min(toutes.len());
        let mut rangees = Vec::new();
        if etat.debut > 0 {
            let n = etat.debut;
            rangees.push((
                format!("↑  {n} plus haut"),
                Reponse::Defiler(-(MONTRES as isize)),
            ));
        }
        let premiere = rangees.len();
        for (i, c) in toutes.iter().enumerate().take(fin).skip(etat.debut) {
            rangees.push((self.libelle(*c), Reponse::Choix(i)));
        }
        if fin < toutes.len() {
            let n = toutes.len() - fin;
            rangees.push((
                format!("↓  {n} plus bas"),
                Reponse::Defiler(MONTRES as isize),
            ));
        }
        let focus = premiere + etat.choisie.saturating_sub(etat.debut);
        (rangees.clone(), focus.min(rangees.len().saturating_sub(1)))
    }

    /// Le nom d'une commande, et son raccourci lu dans le clavier.
    fn libelle(&self, c: Commande) -> String {
        match self.ui.clavier.accords_de(&c.id()).first() {
            Some(a) => format!("{}   {}", c.nom(), a.dire()),
            None => c.nom(),
        }
    }
}

/// **Les commandes que ce filtre trouve, pour ce but**, les meilleures d'abord : pour en lancer une, la palette ne se
/// trouve pas elle-même ; pour changer un raccourci, toutes — le sien compris.
pub(crate) fn trouvees_pour(filtre: &str, but: But) -> Vec<Commande> {
    let mut v: Vec<(u8, usize, Commande)> = Commande::toutes()
        .into_iter()
        .filter(|c| but == But::ChangerLeRaccourci || *c != Commande::Palette)
        .enumerate()
        .filter_map(|(rang, c)| Some((rang_de(filtre, &c.nom(), &c.id())?, rang, c)))
        .collect();
    v.sort_by_key(|(qualite, rang, _)| (*qualite, *rang));
    v.into_iter().map(|(_, _, c)| c).collect()
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
