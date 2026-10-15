//! **La table des commandes** (P4, fiche 72) : chaque action de Glucose nommée une fois.
//!
//! Un raccourci, une entrée du menu, une ligne de la palette désignent une **commande** — jamais
//! le code d'un geste. La commande a un identifiant stable (ce qu'écrit une couche de
//! raccourcis, [`glucose_core::raccourcis`]), un nom dit à l'écran, et sait si sa répétition au
//! maintien d'une touche rend un service (KEY-2). Ce qu'elle fait est dans [`executer`] : une
//! commande rend `false` quand elle ne s'applique pas — les flèches à qualifier quand aucune
//! n'est choisie —, et l'accord passe alors à la suivante de sa liste.
//!
//! **Les raccourcis de Glucose** sont la couche de base, un texte embarqué
//! (`assets/raccourcis/glucose.txt`) : exactement ceux du code d'avant la fiche 72, lus dans leur
//! table au lieu d'être écrits dans cinq `match`. Rien n'a changé à l'écran.

use crate::ui::ActiveTool;
use glucose_core::raccourcis::{Clavier, Couche, ErreurDeLigne};

mod executer;
pub(crate) mod palette;

/// Un sens de déplacement au clavier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sens {
    Gauche,
    Droite,
    Haut,
    Bas,
}

impl Sens {
    const TOUS: [Sens; 4] = [Sens::Gauche, Sens::Droite, Sens::Haut, Sens::Bas];

    fn nom(self) -> &'static str {
        match self {
            Sens::Gauche => "gauche",
            Sens::Droite => "droite",
            Sens::Haut => "haut",
            Sens::Bas => "bas",
        }
    }

    /// Le pas d'un cran dans ce sens, en unités du monde.
    pub fn pas(self) -> (f64, f64) {
        match self {
            Sens::Gauche => (-1.0, 0.0),
            Sens::Droite => (1.0, 0.0),
            Sens::Haut => (0.0, -1.0),
            Sens::Bas => (0.0, 1.0),
        }
    }
}

/// **Une commande de Glucose.**
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Commande {
    // Le document.
    NouveauDocument,
    OuvrirUnDocument,
    Enregistrer,
    EnregistrerSous,
    Exporter,
    ImporterDesImages,
    TimeMachine,
    // L'édition.
    Annuler,
    Retablir,
    Copier,
    Couper,
    Coller,
    Dupliquer,
    ToutChoisir,
    Supprimer,
    PremierPlan,
    ArrierePlan,
    RetirerLesBordures,
    Verrouiller,
    /// Pousser la sélection d'un cran, ou de dix.
    Pousser(Sens, bool),
    // Naviguer.
    ToutCadrer,
    CadrerLaSelection,
    ModeReference,
    /// `Espace` : la navigation, qui lâche ce qu'on tenait — pas l'outil Main seul.
    Naviguer,
    Echap,
    /// Un signet de vue (1 à 9) : le poser, ou y voler.
    PoserLeSignet(u8),
    AllerAuSignet(u8),
    // Les outils.
    Outil(ActiveTool),
    // Les flèches : la relation (1 à 6), ou aucune.
    Relation(Option<u8>),
    // Ce que le menu porte aussi (ACCES-1) : chaque action du menu qui ne dépend pas d'un
    // onglet est une commande — la palette les porte toutes.
    EnregistrerLImage,
    RemplacerParLImage,
    StoryboardDeLaSelection,
    LireLeStoryboard,
    ExporterLesPlanches,
    Signets,
    NouvelOnglet,
    ImporterUnDocument,
    JournalTechnique,
    VoirCeQuiPart,
    // L'interface.
    Palette,
}

/// Les outils que le clavier choisit, et le nom de chacun.
const OUTILS: [(ActiveTool, &str, &str); 7] = [
    (ActiveTool::Pan, "main", "Outil Main"),
    (ActiveTool::Select, "selection", "Outil Sélection"),
    (ActiveTool::Arrow, "fleche", "Outil Flèche"),
    (ActiveTool::Sticky, "post-it", "Outil Post-it"),
    (ActiveTool::Membrane, "membrane", "Outil Membrane"),
    (ActiveTool::Text, "texte", "Outil Texte"),
    (ActiveTool::Folder, "dossier", "Outil Dossier"),
];

/// Les commandes sans paramètre : leur identifiant et leur nom.
const SIMPLES: [(Commande, &str, &str); 35] = [
    (
        Commande::NouveauDocument,
        "nouveau-document",
        "Nouveau document",
    ),
    (Commande::OuvrirUnDocument, "ouvrir", "Ouvrir un document…"),
    (Commande::Enregistrer, "enregistrer", "Enregistrer"),
    (
        Commande::EnregistrerSous,
        "enregistrer-sous",
        "Enregistrer sous…",
    ),
    (Commande::Exporter, "exporter", "Exporter…"),
    (
        Commande::ImporterDesImages,
        "importer-des-images",
        "Ajouter des images…",
    ),
    (Commande::TimeMachine, "time-machine", "Time Machine"),
    (Commande::Annuler, "annuler", "Annuler"),
    (Commande::Retablir, "retablir", "Rétablir"),
    (Commande::Copier, "copier", "Copier"),
    (Commande::Couper, "couper", "Couper"),
    (Commande::Coller, "coller", "Coller"),
    (Commande::Dupliquer, "dupliquer", "Dupliquer"),
    (Commande::ToutChoisir, "tout-choisir", "Tout choisir"),
    (Commande::Supprimer, "supprimer", "Supprimer"),
    (Commande::PremierPlan, "premier-plan", "Premier plan"),
    (Commande::ArrierePlan, "arriere-plan", "Arrière-plan"),
    (
        Commande::RetirerLesBordures,
        "retirer-les-bordures",
        "Retirer les bordures",
    ),
    (
        Commande::Verrouiller,
        "verrouiller",
        "Verrouiller ou déverrouiller",
    ),
    (Commande::ToutCadrer, "tout-cadrer", "Tout cadrer"),
    (
        Commande::CadrerLaSelection,
        "cadrer-la-selection",
        "Cadrer la sélection",
    ),
    (Commande::ModeReference, "mode-reference", "Mode référence"),
    (
        Commande::Naviguer,
        "naviguer",
        "Naviguer (lâcher la sélection)",
    ),
    (Commande::Echap, "echap", "Annuler le geste en cours"),
    (
        Commande::EnregistrerLImage,
        "enregistrer-l-image",
        "Enregistrer l'image sous…",
    ),
    (
        Commande::RemplacerParLImage,
        "remplacer-par-l-image",
        "Remplacer par l'image",
    ),
    (
        Commande::StoryboardDeLaSelection,
        "storyboard-de-la-selection",
        "Storyboard de la sélection",
    ),
    (
        Commande::LireLeStoryboard,
        "lire-le-storyboard",
        "Lire le storyboard",
    ),
    (
        Commande::ExporterLesPlanches,
        "exporter-les-planches",
        "Exporter en planches…",
    ),
    (Commande::Signets, "signets", "Signets…"),
    (Commande::NouvelOnglet, "nouvel-onglet", "Nouveau board"),
    (
        Commande::ImporterUnDocument,
        "importer-un-document",
        "Importer un document…",
    ),
    (
        Commande::JournalTechnique,
        "journal-technique",
        "Journal technique…",
    ),
    (
        Commande::VoirCeQuiPart,
        "voir-ce-qui-part",
        "Voir ce qui part",
    ),
    (Commande::Palette, "palette", "Palette de commandes"),
];

impl Commande {
    /// **Toutes les commandes**, dans l'ordre où la palette les montre.
    pub fn toutes() -> Vec<Commande> {
        let mut v: Vec<Commande> = SIMPLES.iter().map(|(c, _, _)| *c).collect();
        v.extend(
            Sens::TOUS
                .iter()
                .flat_map(|&s| [Commande::Pousser(s, false), Commande::Pousser(s, true)]),
        );
        v.extend(OUTILS.iter().map(|(o, _, _)| Commande::Outil(*o)));
        v.extend((1..=9).map(Commande::AllerAuSignet));
        v.extend((1..=9).map(Commande::PoserLeSignet));
        v.extend((1..=6).map(|r| Commande::Relation(Some(r))));
        v.push(Commande::Relation(None));
        v
    }

    /// **Son identifiant** : ce qu'une couche de raccourcis écrit. Stable — un fork, un preset,
    /// un fichier de l'utilisateur le nomment.
    pub fn id(self) -> String {
        match self {
            Commande::Pousser(s, dix) => {
                format!("pousser.{}{}", s.nom(), if dix { ".dix" } else { "" })
            }
            Commande::Outil(o) => format!("outil.{}", Self::outil(o).1),
            Commande::AllerAuSignet(n) => format!("signet.aller.{n}"),
            Commande::PoserLeSignet(n) => format!("signet.poser.{n}"),
            Commande::Relation(Some(r)) => format!("relation.{r}"),
            Commande::Relation(None) => "relation.aucune".to_string(),
            simple => Self::simple(simple).1.to_string(),
        }
    }

    /// **Son nom à l'écran** : ce que la palette montre.
    pub fn nom(self) -> String {
        match self {
            Commande::Pousser(s, dix) => format!(
                "Pousser la sélection vers {} ({})",
                match s {
                    Sens::Gauche => "la gauche",
                    Sens::Droite => "la droite",
                    Sens::Haut => "le haut",
                    Sens::Bas => "le bas",
                },
                if dix { "dix pas" } else { "un pas" }
            ),
            Commande::Outil(o) => Self::outil(o).2.to_string(),
            Commande::AllerAuSignet(n) => format!("Aller au signet {n}"),
            Commande::PoserLeSignet(n) => format!("Poser le signet {n}"),
            Commande::Relation(Some(r)) => format!("Flèches : relation {r}"),
            Commande::Relation(None) => "Flèches : sans relation".to_string(),
            simple => Self::simple(simple).2.to_string(),
        }
    }

    /// **La commande de cet identifiant**, si elle existe.
    pub fn de_l_id(id: &str) -> Option<Commande> {
        Self::toutes().into_iter().find(|c| c.id() == id)
    }

    /// **Sa répétition au maintien d'une touche rend-elle un service ?** (KEY-2) Pousser d'un
    /// pas de plus, défaire d'un cran de plus : oui. Coller, dupliquer, ouvrir : jamais — un
    /// `Ctrl+V` maintenu collait cinquante images par seconde.
    pub fn repetable(self) -> bool {
        matches!(
            self,
            Commande::Pousser(..) | Commande::Annuler | Commande::Retablir
        )
    }

    fn simple(c: Commande) -> (Commande, &'static str, &'static str) {
        SIMPLES
            .iter()
            .copied()
            .find(|(s, _, _)| *s == c)
            .unwrap_or((c, "", ""))
    }

    fn outil(o: ActiveTool) -> (ActiveTool, &'static str, &'static str) {
        OUTILS
            .iter()
            .copied()
            .find(|(t, _, _)| *t == o)
            .unwrap_or((o, "", ""))
    }
}

/// **Les raccourcis de Glucose**, la couche de base.
pub const RACCOURCIS_DE_GLUCOSE: &str = include_str!("../assets/raccourcis/glucose.txt");

/// **Le clavier de Glucose** : sa couche de base, et les lignes qu'elle n'a pas pu lire — une
/// épreuve exige qu'il n'y en ait aucune, et que chaque commande nommée existe.
pub fn clavier_de_glucose() -> (Clavier, Vec<ErreurDeLigne>) {
    let (couche, erreurs) = Couche::lire("glucose", RACCOURCIS_DE_GLUCOSE);
    (Clavier::de(vec![couche]), erreurs)
}

#[cfg(test)]
mod tests;
