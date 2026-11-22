//! **Le tiroir DISCUSSION** (fiche 129) : ce qui s'est dit avec l'IA, et le champ où l'on écrit.
//! Sa demande du 21/11 : *« une fenêtre de chat »*. Un tiroir à droite, comme le panneau du kit
//! d'agent de tldraw (tldraw.dev/starter-kits/agent) : la discussion d'un côté, le canevas où l'IA
//! travaille sous ses yeux de l'autre. Ses gestes y sont des lignes ; leur trace, dans la Time
//! Machine, signée de son nom.
//!
//! # Une seule géométrie (L4) — les lignes se coupent à l'arrivée
//!
//! Couper un texte demande de le mesurer, donc la typographie, que le rangement du dock n'a pas.
//! La boucle coupe donc le fil quand il change — un message de plus, une autre échelle
//! ([`DiscussionUi::couper`]) — et la mise en page, le dessin et le clic ne font que compter ses
//! lignes.

use super::WidgetRect;
use crate::params::{Pointer, ScaledRect};
use crate::typography::{Face, Typography};

pub mod paint;

/// La largeur du tiroir, celle de Plugins, le plus large : une réponse se lit en lignes longues.
pub const LARGEUR: f32 = 340.0;
const MARGE: f32 = 14.0;
/// Le haut du fil, sous le titre.
const HAUT: f32 = 40.0;
/// La hauteur d'une ligne du fil.
pub const LIGNE: f32 = 15.0;
const CHAMP: f32 = 26.0;
const BOUTON: f32 = 72.0;
const ECART: f32 = 8.0;
const PIED: f32 = 14.0;
/// La taille du texte du fil, et celle du nom de qui parle.
pub const CORPS: f32 = 11.0;
pub const AUTEUR: f32 = 9.0;

/// Ce qui s'est dit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Message {
    Moi(String),
    Ia {
        modele: String,
        texte: String,
    },
    /// Un geste de l'IA sur le canevas, en mots.
    Geste(String),
    /// Une bascule, une ligne de l'essai : ce qui se dit sans être une réponse.
    Note(String),
    Erreur(String),
}

/// Ce qu'une ligne du fil est : son encre en dépend.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Genre {
    Auteur,
    Moi,
    Ia,
    Geste,
    Note,
    Erreur,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ligne {
    pub texte: String,
    pub genre: Genre,
}

/// **L'état du tiroir** : ce qui s'est dit, ses lignes coupées, le champ.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct DiscussionUi {
    pub messages: Vec<Message>,
    pub lignes: Vec<Ligne>,
    /// Pour quoi les lignes sont coupées : le nombre de messages, et l'échelle.
    coupe_pour: Option<(usize, u32)>,
    /// Le champ : ce qu'on écrit, et où — frappé par la loi de l'édition d'un nœud.
    pub entree: crate::ui::question::Champ,
    /// Le clavier va au champ.
    pub tape: bool,
    /// Le modèle qui réfléchit, pendant qu'une question est en route.
    pub en_cours: Option<String>,
}

impl DiscussionUi {
    pub fn dire(&mut self, message: Message) {
        self.messages.push(message);
    }

    /// **Coupe le fil à la largeur du tiroir**, s'il a changé depuis la dernière coupe. `s` est
    /// l'échelle déjà bornée. Rend vrai s'il a coupé.
    pub fn couper(&mut self, typo: &Typography, s: f32) -> bool {
        let cle = (self.messages.len(), s.to_bits());
        if self.coupe_pour == Some(cle) {
            return false;
        }
        let largeur = (LARGEUR - 2.0 * MARGE) * s;
        let mesure = |t: &str| typo.measure_text(t, CORPS * s, Face::Regular).0;
        self.lignes = self
            .messages
            .iter()
            .flat_map(|m| lignes_de(m, &mesure, largeur))
            .collect();
        self.coupe_pour = Some(cle);
        true
    }
}

/// Les lignes d'un message : qui parle, puis ce qu'il dit.
fn lignes_de(m: &Message, mesure: &dyn Fn(&str) -> f32, largeur: f32) -> Vec<Ligne> {
    let corps = |texte: &str, genre: Genre| {
        couper_le_texte(texte, mesure, largeur)
            .into_iter()
            .map(move |texte| Ligne { texte, genre })
    };
    let auteur = |nom: &str| Ligne {
        texte: nom.to_string(),
        genre: Genre::Auteur,
    };
    match m {
        Message::Moi(t) => std::iter::once(auteur("Toi"))
            .chain(corps(t, Genre::Moi))
            .collect(),
        Message::Ia { modele, texte } => std::iter::once(auteur(modele))
            .chain(corps(&en_clair(texte), Genre::Ia))
            .collect(),
        Message::Geste(t) => corps(&format!("→ {t}"), Genre::Geste).collect(),
        Message::Note(t) => corps(t, Genre::Note).collect(),
        Message::Erreur(t) => corps(t, Genre::Erreur).collect(),
    }
}

/// **Coupe un texte en lignes** qui tiennent dans `largeur`, entre les mots ; un mot plus long
/// que la ligne se coupe où il faut. Un saut de ligne du texte en reste un.
pub fn couper_le_texte(texte: &str, mesure: &dyn Fn(&str) -> f32, largeur: f32) -> Vec<String> {
    let mut lignes = Vec::new();
    for paragraphe in texte.split('\n') {
        let mut ligne = String::new();
        for mot in paragraphe.split_whitespace() {
            let essai = if ligne.is_empty() {
                mot.to_string()
            } else {
                format!("{ligne} {mot}")
            };
            if mesure(&essai) <= largeur {
                ligne = essai;
                continue;
            }
            if !ligne.is_empty() {
                lignes.push(std::mem::take(&mut ligne));
            }
            // Le dernier morceau du mot reste la ligne en cours, même quand un seul caractère
            // déborde : une coupe qui prendrait tout laisserait une ligne vide derrière elle.
            let mut reste = mot;
            while mesure(reste) > largeur {
                let coupe = premiere_coupe(reste, mesure, largeur);
                if coupe == reste.len() {
                    break;
                }
                lignes.push(reste[..coupe].to_string());
                reste = &reste[coupe..];
            }
            ligne = reste.to_string();
        }
        lignes.push(ligne);
    }
    lignes
}

/// Où couper un mot trop long : après le plus long début qui tient — un caractère au moins, pour
/// que la coupe avance toujours.
fn premiere_coupe(mot: &str, mesure: &dyn Fn(&str) -> f32, largeur: f32) -> usize {
    let fins = mot.char_indices().map(|(i, c)| i + c.len_utf8());
    let premiere = fins.clone().next().unwrap_or(mot.len());
    fins.take_while(|&i| mesure(&mot[..i]) <= largeur)
        .last()
        .unwrap_or(premiere)
}

/// **Le Markdown d'une réponse, en clair** : le tiroir n'a pas le dessin riche des nœuds ; les
/// marques (`**`, `` ` ``, `#`) partent, une puce devient « • ».
pub fn en_clair(texte: &str) -> String {
    texte
        .lines()
        .map(|ligne| {
            let retrait = ligne.len() - ligne.trim_start().len();
            let reste = ligne.trim_start();
            let reste = if reste.starts_with('#') {
                reste.trim_start_matches('#').trim_start()
            } else {
                reste
            };
            let reste = match reste
                .strip_prefix("* ")
                .or_else(|| reste.strip_prefix("- "))
            {
                Some(r) => format!("• {r}"),
                None => reste.to_string(),
            };
            let propre = reste.replace("**", "").replace("__", "").replace('`', "");
            format!("{}{propre}", &ligne[..retrait])
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// La géométrie du tiroir — la seule, lue par le dessin et par le clic.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DiscussionLayout {
    pub fil: WidgetRect,
    pub champ: WidgetRect,
    pub bouton: WidgetRect,
    pub pied: (f32, f32),
}

pub fn layout_discussion_panel(frame: ScaledRect, ui: &DiscussionUi) -> DiscussionLayout {
    let s = crate::theme::clamp_ui_scale(frame.scale);
    let (x, w) = (frame.x + MARGE * s, frame.w - 2.0 * MARGE * s);
    let fil = WidgetRect::new(x, frame.y + HAUT * s, w, ui.lignes.len() as f32 * LIGNE * s);
    let y = fil.y + fil.h + ECART * s;
    let bouton = WidgetRect::new(x + w - BOUTON * s, y, BOUTON * s, CHAMP * s);
    let champ = WidgetRect::new(x, y, w - (BOUTON + 6.0) * s, CHAMP * s);
    DiscussionLayout {
        fil,
        champ,
        bouton,
        pied: (x, y + (CHAMP + 6.0) * s),
    }
}

/// **La hauteur du tiroir** (TIROIR-2), sans sa poignée : jusqu'au bas de sa phrase de pied.
pub fn hauteur_naturelle(ui: &DiscussionUi, largeur: f32, s: f32) -> f32 {
    let cadre = ScaledRect {
        x: 0.0,
        y: 0.0,
        w: largeur,
        h: 0.0,
        scale: s,
    };
    let s = crate::theme::clamp_ui_scale(s);
    layout_discussion_panel(cadre, ui).pied.1 + (PIED + 10.0) * s
}

/// Ce qu'un clic dans le tiroir demande.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiscussionIntent {
    Saisir,
    Envoyer,
    Arreter,
}

pub fn click_discussion_panel(
    ui: &DiscussionUi,
    layout: &DiscussionLayout,
    pointer: Pointer,
) -> Option<DiscussionIntent> {
    if layout.bouton.contains(pointer.x, pointer.y) {
        return Some(if ui.en_cours.is_some() {
            DiscussionIntent::Arreter
        } else {
            DiscussionIntent::Envoyer
        });
    }
    layout
        .champ
        .contains(pointer.x, pointer.y)
        .then_some(DiscussionIntent::Saisir)
}

#[cfg(test)]
mod tests;
