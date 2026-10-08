//! **CLAVIER-1 — le clavier du système, miroir de la saisie** (fiche 57).
//!
//! Le clavier d'un téléphone ne tape pas des touches : il **réécrit un texte** qu'il tient
//! lui-même (`plateforme::clavier`). Une suggestion remplace un mot entier, la correction
//! change une lettre trois mots plus tôt, la dictée pose une phrase d'un coup. Rejouer cela en
//! frappes serait deviner ; Glucose fait l'inverse — **la saisie et le clavier tiennent le
//! même état**, et chaque différence passe de l'un à l'autre :
//!
//! * ce que le clavier a changé devient **une** commande d'écriture ordinaire — la plus petite
//!   réécriture qui mène d'un texte à l'autre ([`epissure`]) —, donc s'annule mot par mot
//!   (TEXTE-UNDO-1) et s'enregistre comme une frappe ;
//! * ce que Glucose a changé lui-même (un toucher qui déplace le curseur, `Ctrl+Z`, un collage)
//!   repart au clavier, sans quoi sa prochaine réécriture partirait d'un texte périmé et
//!   défairait le geste.
//!
//! # Comparer des états, pas écouter des moments
//!
//! Le miroir ne sait pas *quand* une saisie s'ouvre ou se ferme : un clic, l'outil Texte, la
//! reprise après un plantage, un document qu'on change le font chacun à leur façon. Il compare
//! ce que la saisie est à ce que le clavier tient, à chaque tour de boucle — aucun chemin ne
//! peut l'oublier.
//!
//! # L'état qui revient d'avant (CLAVIER-2)
//!
//! Écrire au clavier n'est pas fait quand l'appel rend la main : la demande part dans la file
//! du fil de l'interface d'Android. Relire juste après rend **l'ancien** état, qu'un miroir naïf
//! prendrait pour une réécriture — et il défairait ce que Glucose vient d'écrire. Le miroir
//! retient donc ce que le clavier tenait avant l'envoi, et l'ignore tant qu'il revient.

use super::keys::Command;
use crate::app::GlucoseApp;
use crate::plateforme::clavier::{Clavier, EtatDuClavier};
use crate::renderer::card::{text_box, TEXT_ORIGIN};
use crate::renderer::richtext::hit::line_of_offset;
use glucose_core::text::selection::{Direction, Motion};
use glucose_core::text::Selection;
use std::ops::Range;

/// Le clavier du système et ce que la saisie lui a confié.
#[derive(Default)]
pub struct Miroir {
    systeme: Option<Box<dyn Clavier>>,
    accord: Option<Accord>,
    /// Un toucher dans la saisie redemande le clavier : le geste retour l'a peut-être rentré,
    /// et Android ne le dit pas.
    redemande: bool,
}

/// **Ce que le clavier écrit** : le texte d'un nœud en saisie, ou le champ d'une question —
/// qui passe devant, puisqu'elle prend tout tant qu'elle est posée (DOCUMENTS-2).
#[derive(Clone, Debug, PartialEq, Eq)]
enum Cible {
    Noeud(String),
    Champ,
}

/// Ce que le clavier tient pour une saisie.
struct Accord {
    cible: Cible,
    /// Ce qu'il tient, ou tiendra quand la file l'aura servi.
    tient: EtatDuClavier,
    /// Ce qu'il tenait avant le dernier envoi, tant qu'il peut encore le rendre (CLAVIER-2).
    perime: Option<EtatDuClavier>,
}

impl Miroir {
    /// Le clavier de cette plateforme ; sans lui, le miroir ne fait rien.
    pub fn brancher(&mut self, clavier: Box<dyn Clavier>) {
        self.systeme = Some(clavier);
    }

    /// Ressortir le clavier au prochain tour, s'il y a une saisie.
    pub fn redemander(&mut self) {
        self.redemande = true;
    }

    /// Une saisie est-elle confiée au clavier ?
    pub fn est_ouvert(&self) -> bool {
        self.accord.is_some()
    }

    fn lire(&self) -> EtatDuClavier {
        self.systeme.as_ref().map(|c| c.lire()).unwrap_or_default()
    }

    /// Une saisie neuve : le clavier sort sur elle.
    fn ouvrir(&mut self, cible: Cible, etat: EtatDuClavier) {
        let avant = self.lire();
        if let Some(c) = &self.systeme {
            c.montrer(&etat);
        }
        let perime = (!avant.meme_saisie(&etat)).then_some(avant);
        self.accord = Some(Accord {
            cible,
            tient: etat,
            perime,
        });
        self.redemande = false;
    }

    /// La saisie est finie : le clavier rentre.
    fn rentrer(&mut self) {
        self.redemande = false;
        if self.accord.take().is_some() {
            if let Some(c) = &self.systeme {
                c.cacher();
            }
        }
    }

    /// **Le clavier a-t-il réécrit la saisie ?** Un état qui revient d'avant l'envoi n'est pas
    /// une réécriture (CLAVIER-2) ; le premier qui en diffère prouve que la file est servie.
    fn nouveau(&mut self, lu: &EtatDuClavier) -> bool {
        let Some(accord) = &mut self.accord else {
            return false;
        };
        if accord.perime.as_ref().is_some_and(|p| p.meme_saisie(lu)) {
            return false;
        }
        accord.perime = None;
        if accord.tient.meme_saisie(lu) {
            return false;
        }
        accord.tient = lu.clone();
        true
    }

    /// Ce que Glucose a changé lui-même repart au clavier — et le clavier ressort si on l'a
    /// redemandé.
    fn rattraper(&mut self, voulu: EtatDuClavier) {
        let Some(accord) = &mut self.accord else {
            return;
        };
        let Some(clavier) = &self.systeme else {
            return;
        };
        if !accord.tient.meme_saisie(&voulu) {
            // S'il revient encore d'avant un envoi, c'est toujours cet état-là qui peut revenir.
            if accord.perime.is_none() {
                accord.perime = Some(accord.tient.clone());
            }
            clavier.ecrire(&voulu);
            accord.tient = voulu;
        }
        if std::mem::take(&mut self.redemande) {
            clavier.montrer(&accord.tient);
        }
    }
}

impl GlucoseApp {
    /// **Un tour du miroir** : à chaque fois que la boucle a vidé ses évènements.
    pub fn suivre_le_clavier(&mut self) {
        if self.lancement.clavier.systeme.is_none() {
            return;
        }
        let Some((cible, etat)) = self.saisie_au_clavier() else {
            self.lancement.clavier.rentrer();
            return;
        };
        let accord = self.lancement.clavier.accord.as_ref();
        if accord.is_none_or(|a| a.cible != cible) {
            self.lancement.clavier.ouvrir(cible, etat);
            return;
        }
        let lu = self.lancement.clavier.lire();
        if self.lancement.clavier.nouveau(&lu) {
            self.refleter(&cible, &lu);
        }
        if let Some((_, voulu)) = self.saisie_au_clavier() {
            self.lancement.clavier.rattraper(voulu);
        }
    }

    /// La saisie que le clavier doit tenir, s'il y en a une : le champ d'une question d'abord.
    fn saisie_au_clavier(&self) -> Option<(Cible, EtatDuClavier)> {
        if let Some(champ) = self
            .ui
            .question
            .as_ref()
            .and_then(|(q, _)| q.champ.as_ref())
        {
            return Some((Cible::Champ, etat_de(&champ.texte, champ.selection)));
        }
        let s = self.editing_session.as_ref()?;
        let etat = etat_de(&s.buffer, s.selection);
        Some((Cible::Noeud(s.ann_id.clone()), etat))
    }

    /// Ce que le clavier a écrit devient la saisie : dans un nœud, une commande d'écriture,
    /// puis sa sélection ; dans un champ, son texte — un nom ne s'annule pas mot par mot.
    fn refleter(&mut self, cible: &Cible, lu: &EtatDuClavier) {
        if *cible == Cible::Champ {
            let question = self.ui.question.as_mut();
            if let Some(champ) = question.and_then(|(q, _)| q.champ.as_mut()) {
                champ.selection = selection_de(&lu.texte, lu.selection);
                champ.texte = lu.texte.clone();
            }
            self.mark_dirty();
            return;
        }
        let Some(session) = self.editing_session.as_mut() else {
            return;
        };
        let (plage, ecrit) = epissure(&session.buffer, &lu.texte);
        if !plage.is_empty() || !ecrit.is_empty() {
            session.selection = Selection {
                anchor: plage.start,
                head: plage.end,
            };
            let commande = if ecrit.is_empty() {
                Command::Delete(Motion::Char, Direction::Backward)
            } else {
                Command::Insert(ecrit.to_string())
            };
            self.apply_text_command(commande, false);
        }
        if let Some(session) = self.editing_session.as_mut() {
            session.selection = selection_de(&session.buffer, lu.selection);
            session.goal_x = None;
        }
        self.mark_dirty();
        self.garder_la_ligne_en_vue();
    }

    /// **La ligne qu'on écrit reste en vue** (CLAVIER-3) : quand le clavier sort, la fenêtre
    /// rétrécit (`adjustResize`), et ce qu'on écrit pouvait passer dessous.
    ///
    /// La vue bouge du moins possible — zéro si la ligne se voit déjà — et seulement en
    /// hauteur : le clavier ne prend que de la hauteur. Une ligne plus haute que ce qui reste
    /// montre son haut, là où le curseur commence.
    pub(crate) fn garder_la_ligne_en_vue(&mut self) {
        let Some(session) = self.editing_session.as_ref() else {
            return;
        };
        let (Some(boite), Some((mise_en_page, largeur))) =
            (self.card_box(&session.ann_id), self.editing_layout())
        else {
            return;
        };
        let rang = line_of_offset(&mise_en_page, session.selection.head);
        let hauteur = f64::from(text_box(largeur).line_height);
        let Some(board) = self.store.active_board() else {
            return;
        };
        let mut vue = board.viewport;
        let id = board.id.clone();
        let haut_monde = boite.origin.1 + f64::from(TEXT_ORIGIN.1) + rang as f64 * hauteur;
        let haut = haut_monde * vue.scale + vue.y;
        let bas = haut + hauteur * vue.scale;
        let plafond = f64::from(self.ui.header_height());
        let plancher = f64::from(self.taille_de_la_fenetre().1);
        // Le décalage le plus proche de zéro qui pose la ligne entre les deux ; le haut prime.
        let decalage = 0.0_f64.min(plancher - bas).max(plafond - haut);
        if decalage != 0.0 {
            vue.y += decalage;
            self.store.set_viewport(&id, vue);
            self.mark_dirty();
        }
    }
}

/// Ce qu'une saisie confie au clavier : son texte, et sa sélection en unités UTF-16.
fn etat_de(texte: &str, selection: Selection) -> EtatDuClavier {
    EtatDuClavier {
        selection: (
            utf8_vers_utf16(texte, selection.anchor),
            utf8_vers_utf16(texte, selection.head),
        ),
        texte: texte.to_string(),
        composition: None,
    }
}

/// La sélection que le clavier rend, en octets de ce texte.
fn selection_de(texte: &str, (ancre, tete): (usize, usize)) -> Selection {
    Selection {
        anchor: utf16_vers_utf8(texte, ancre),
        head: utf16_vers_utf8(texte, tete),
    }
}

/// **La plus petite réécriture** qui mène de `avant` à `apres` : la plage d'`avant` à remplacer,
/// et ce qui la remplace — tout ce qui précède et suit en commun reste. Les deux bords tombent
/// sur des caractères entiers.
pub fn epissure<'a>(avant: &str, apres: &'a str) -> (Range<usize>, &'a str) {
    let debut = avant
        .char_indices()
        .zip(apres.chars())
        .find(|((_, a), b)| a != b)
        .map_or(avant.len().min(apres.len()), |((i, _), _)| i);
    // La fin commune ne mord pas sur le début commun : « aa » → « aaa » ajoute à la fin.
    let place = avant.len().min(apres.len()) - debut;
    let mut fin = 0;
    for (a, b) in avant.chars().rev().zip(apres.chars().rev()) {
        if a != b || fin + a.len_utf8() > place {
            break;
        }
        fin += a.len_utf8();
    }
    (debut..avant.len() - fin, &apres[debut..apres.len() - fin])
}

/// L'octet UTF-8 d'une position en unités UTF-16. Une position au milieu d'une paire (un
/// emoji) tombe au début du caractère ; au-delà du texte, à sa fin.
pub fn utf16_vers_utf8(texte: &str, position: usize) -> usize {
    let mut unites = 0;
    for (octet, c) in texte.char_indices() {
        unites += c.len_utf16();
        if unites > position {
            return octet;
        }
    }
    texte.len()
}

/// La position en unités UTF-16 d'un octet UTF-8, posé sur un caractère entier.
pub fn utf8_vers_utf16(texte: &str, octet: usize) -> usize {
    texte
        .get(..octet.min(texte.len()))
        .map_or(0, |debut| debut.encode_utf16().count())
}

#[cfg(test)]
mod tests;
