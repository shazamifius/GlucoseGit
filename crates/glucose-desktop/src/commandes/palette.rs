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

mod chercher;
mod defiler;
mod ouvrir;

/// **L'état de la palette** : le filtre que ses rangées montrent, la première commande de sa
/// fenêtre, et celle en évidence — des rangs dans toutes les commandes trouvées.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Etat {
    pub filtre: String,
    pub debut: usize,
    pub choisie: usize,
    /// Ce que la commande choisie devient : lancée, ou celle dont on change le raccourci.
    pub but: But,
    /// Ce que la recherche a trouvé pour ce filtre (`Ctrl+F`, fiche 89) : vide pour les commandes.
    pub trouves: Vec<glucose_core::recherche::Trouve>,
    /// Tous les documents (`Ctrl+O`, RECENTS-1), et la trouvaille de la fouille qu'ils comptent.
    pub documents: Vec<crate::persist::recents::Document>,
    pub fouillee: u64,
    /// **Ce qui vient d'arriver à un document** — renommé, dupliqué, supprimé (DOCUMENTS-2) : dit
    /// en tête de la liste, là où l'on regarde.
    pub note: Option<String>,
}

/// **Ce que la palette fait de la commande choisie** (P4, fiche 74).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum But {
    /// La lancer.
    #[default]
    Lancer,
    /// Changer son raccourci : la palette sert à la trouver, toutes les commandes comprises.
    ChangerLeRaccourci,
    /// **Chercher dans le document** (`Ctrl+F`, fiche 89) : ses rangées sont des nœuds, et Entrée y
    /// va. La même palette, une autre source — comme `Ctrl+P` et `Ctrl+Maj+P` dans VS Code.
    Chercher,
    /// **Ouvrir un document** (`Ctrl+O`, RECENTS-1, fiche 174) : ses rangées sont les documents.
    Ouvrir,
}

impl GlucoseApp {
    /// **Ouvre la palette**, le champ vide : toutes les commandes, dans l'ordre de la table.
    pub(crate) fn ouvrir_la_palette(&mut self, but: But) {
        self.poser_une_palette(Etat {
            but,
            ..Etat::default()
        });
    }

    /// **Pose une palette neuve** dans cet état, le champ vide.
    fn poser_une_palette(&mut self, mut etat: Etat) {
        let but = etat.but;
        // Au téléphone, la liste se lit d'abord : le clavier attend le doigt (RECENTS-2).
        let au_doigt = but == But::Ouvrir && self.ui.documents_ranges;
        if but == But::Ouvrir {
            // Ce qu'elle trouvera paraîtra en chemin (RECENTS-1) ; au téléphone, elle ne cherche
            // nulle part (`fouille::racine`).
            self.disque.fouille.lancer();
            etat.documents = self.composer_les_documents();
            etat.fouillee = self.disque.fouille.version();
        }
        let (choix, focus, precisions) = self.rangees_de_la_palette(&etat);
        let compte = (but == But::Ouvrir).then(|| self.compte_des_documents(&etat));
        let (titre, texte) = match but {
            But::Lancer => (
                "Palette de commandes",
                "Écris ce que tu cherches ; Entrée lance la commande en évidence.",
            ),
            But::ChangerLeRaccourci => (
                "Changer un raccourci",
                "Choisis la commande dont tu veux changer le raccourci.",
            ),
            But::Chercher => ("Chercher", chercher::AIDE),
            But::Ouvrir => ("Ouvrir un document", ouvrir::AIDE),
        };
        let question = Question {
            titre: titre.into(),
            texte: compte.unwrap_or_else(|| texte.into()),
            choix,
            focus,
            precisions,
            champ: Some(Champ::default()),
            clavier_au_toucher: au_doigt,
            ..Default::default()
        };
        self.ui.reste_de_la_molette = 0.0;
        self.demander(question, Suite::Palette(etat));
    }

    /// **La palette suit son champ** : un texte qui a changé depuis l'image d'avant refait ses
    /// rangées depuis le début, l'évidence sur la première. Celle des documents suit aussi la
    /// fouille : ce qu'elle trouve paraît, l'évidence à sa place.
    pub(crate) fn suivre_la_palette(&mut self) {
        let fouillee = self.disque.fouille.version();
        if let Some((_, Suite::Palette(etat))) = &self.ui.question {
            if etat.but == But::Ouvrir && etat.fouillee != fouillee {
                let mut etat = etat.clone();
                etat.documents = self.composer_les_documents();
                etat.fouillee = fouillee;
                self.poser_la_palette(etat);
            }
        }
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
        let Some((_, Suite::Palette(avant))) = &self.ui.question else {
            return;
        };
        let mut etat = avant.clone();
        if etat.but == But::Chercher {
            etat.trouves = self.chercher_dans_le_document(&ecrit);
        }
        (etat.filtre, etat.debut, etat.choisie) = (ecrit, 0, 0);
        self.poser_la_palette(etat);
    }

    /// **Pose cet état** : ses rangées, son évidence, et lui-même dans la question.
    pub(super) fn poser_la_palette(&mut self, etat: Etat) {
        let (choix, focus, precisions) = self.rangees_de_la_palette(&etat);
        let dit = match etat.but {
            But::Chercher => Some(chercher::compte(&etat)),
            But::Ouvrir => Some(self.compte_des_documents(&etat)),
            _ => None,
        };
        if let Some((q, suite)) = &mut self.ui.question {
            q.choix = choix;
            q.focus = focus;
            q.precisions = precisions;
            if let Some(dit) = dit {
                q.texte = dit;
            }
            *suite = Suite::Palette(etat);
        }
        self.mark_dirty();
    }

    /// **La réponse de la palette** : la commande de ce rang, parmi celles que ce filtre trouve.
    pub(crate) fn lancer_depuis_la_palette(&mut self, etat: &Etat, reponse: Reponse) {
        if etat.but == But::Ouvrir {
            self.ouvrir_depuis_la_palette(etat, reponse);
            return;
        }
        if let Reponse::Choix(i) = reponse {
            if etat.but == But::Chercher {
                self.aller_au_resultat(&etat.filtre, &etat.trouves, i);
                return;
            }
            let trouvees = trouvees_pour(&etat.filtre, etat.but, &|c| self.s_applique(c));
            match (trouvees.get(i), etat.but) {
                (Some(&commande), But::Lancer) => {
                    self.executer(commande);
                }
                (Some(&commande), But::ChangerLeRaccourci) => self.demander_le_raccourci(commande),
                (Some(_), But::Chercher | But::Ouvrir) | (None, _) => {}
            }
        }
    }

    /// **Les rangées de la palette** et celle en évidence : « ↑ n plus haut » s'il y a des
    /// commandes au-dessus de la fenêtre, ses commandes avec leur raccourci, « ↓ n plus bas »
    /// s'il en reste dessous ; pour les documents, ce qui situe chacun, et « Parcourir ce PC… »
    /// tout en bas.
    fn rangees_de_la_palette(&self, etat: &Etat) -> Rangees {
        let dossier_de = self.lecteur();
        let documents = ouvrir::trouves_pour(&etat.filtre, &etat.documents, &dossier_de);
        let mut precisions = Vec::new();
        let toutes: Vec<String> = match etat.but {
            But::Ouvrir => documents
                .iter()
                .enumerate()
                .map(|(i, d)| {
                    let (nom, precision) = ouvrir::rangee(d, dossier_de(&d.chemin));
                    precisions.push((Reponse::Choix(i), precision));
                    nom
                })
                .collect(),
            // La phrase d'un résultat ne se calcule que pour les rangées montrées (fiche 89).
            But::Chercher => etat
                .trouves
                .iter()
                .enumerate()
                .map(|(i, t)| {
                    let montree = (etat.debut..etat.debut + MONTRES).contains(&i);
                    if !montree {
                        return String::new();
                    }
                    let (phrase, precision) = self.dire_le_trouve(t, &etat.filtre);
                    precisions.push((Reponse::Choix(i), precision));
                    phrase
                })
                .collect(),
            _ => trouvees_pour(&etat.filtre, etat.but, &|c| self.s_applique(c))
                .into_iter()
                .map(|c| self.libelle(c))
                .collect(),
        };
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
        for (i, libelle) in toutes.iter().enumerate().take(fin).skip(etat.debut) {
            rangees.push((libelle.clone(), Reponse::Choix(i)));
        }
        if fin < toutes.len() {
            let n = toutes.len() - fin;
            rangees.push((
                format!("↓  {n} plus bas"),
                Reponse::Defiler(MONTRES as isize),
            ));
        }
        let focus = premiere + etat.choisie.saturating_sub(etat.debut);
        let focus = focus.min(rangees.len().saturating_sub(1));
        // Au téléphone, aucun sélecteur ne voit plus loin que les documents de Glucose ; et le
        // geste retour n'est pas le seul moyen d'en sortir (DOCUMENTS-1).
        if etat.but == But::Ouvrir {
            rangees.push(match self.ui.documents_ranges {
                true => ("Annuler".into(), Reponse::Annuler),
                false => ("Parcourir ce PC…".into(), ouvrir::PARCOURIR),
            });
        }
        (rangees, focus, precisions)
    }

    /// Le nom d'une commande, et son raccourci lu dans le clavier.
    fn libelle(&self, c: Commande) -> String {
        match self.ui.clavier.accords_de(&c.id()).first() {
            Some(a) => format!("{}   {}", c.nom(), a.dire()),
            None => c.nom(),
        }
    }
}

/// Les rangées d'une palette, celle en évidence, et ce que certaines disent de plus.
type Rangees = (
    Vec<(String, Reponse)>,
    usize,
    Vec<(Reponse, crate::ui::question::Precision)>,
);

impl GlucoseApp {
    /// **Combien de rangées cet état porte** : les nœuds trouvés, ou les commandes.
    pub(crate) fn total_de_la_palette(&self, etat: &Etat) -> usize {
        match etat.but {
            But::Chercher => etat.trouves.len(),
            But::Ouvrir => {
                ouvrir::trouves_pour(&etat.filtre, &etat.documents, &self.lecteur()).len()
            }
            _ => trouvees_pour(&etat.filtre, etat.but, &|c| self.s_applique(c)).len(),
        }
    }
}

/// **Les commandes que ce filtre trouve, pour ce but**, les meilleures d'abord. Pour en lancer
/// une : celles qui s'appliquent (`s_applique`, APPLIQUE-1), sauf la palette elle-même ; pour
/// changer un raccourci, toutes — la sienne comprise, et celles qui ne s'appliquent pas encore.
pub(crate) fn trouvees_pour(
    filtre: &str,
    but: But,
    s_applique: &dyn Fn(Commande) -> bool,
) -> Vec<Commande> {
    if matches!(but, But::Chercher | But::Ouvrir) {
        return Vec::new();
    }
    let mut v: Vec<(u8, usize, Commande)> = Commande::toutes()
        .into_iter()
        .filter(|c| but == But::ChangerLeRaccourci || (*c != Commande::Palette && s_applique(*c)))
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

/// Le texte sans majuscules ni accents : la loi de la recherche (PLI-1).
fn simple(t: &str) -> String {
    glucose_core::recherche::pli(t)
}
