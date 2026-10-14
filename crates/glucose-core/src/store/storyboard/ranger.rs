//! **Ranger un storyboard** : sa suite de cases à leur place dans la grille, numérotées de un en
//! un, ses images qui suivent leur case — la seule loi qui écrit les cases d'un storyboard
//! (STORYBOARD-2, fiche 65).
//!
//! # Plusieurs storyboards dans une liste plate (fiche 68)
//!
//! Le tableau garde toutes ses cases dans une seule liste ; chacune nomme son storyboard. Ranger
//! l'un ne touche **que ses places** dans la liste ([`remplacer_la_suite`]) : la `k`-ième place
//! de sa suite reçoit sa `k`-ième case, celles de trop partent, celles qui manquent s'ajoutent
//! au bout. Les cases des autres storyboards ne bougent pas d'un rang — le journal, qui défait
//! par rangs, ne les écrit pas.

use super::super::journal::{Edit, Slot};
use super::super::Store;
use super::{suivre_le_cadre, Disposition};
use crate::types::{BoardImage, Storyboard, StoryboardPanel};

impl Store {
    /// **La suite, rangée à la disposition** : chaque case à sa place et à sa taille, numérotée
    /// de un en un, dans le storyboard `sb` ; ses images la suivent ([`suivre_le_cadre`]) ; sa
    /// grille s'enregistre — et s'il n'existe pas encore, il naît. Ce qui change entre au
    /// journal en un geste — dans le geste ouvert, s'il y en a un.
    ///
    /// `tenue` : la case qu'une main a portée, et les images qu'elle emportait — les siennes,
    /// quoi qu'elle recouvre ; les autres images cherchent leur case sans elle.
    pub(super) fn ranger(
        &mut self,
        (board_id, sb): (&str, &str),
        d: Disposition,
        suite: Vec<StoryboardPanel>,
        tenue: Option<(&str, &[String])>,
    ) {
        let neuves: Vec<StoryboardPanel> = suite
            .iter()
            .enumerate()
            .map(|(i, c)| {
                let (x, y) = d.grille.coin(d.origine, i);
                StoryboardPanel {
                    order: i as i32,
                    x,
                    y,
                    width: d.grille.largeur,
                    height: d.grille.hauteur(),
                    ..c.clone()
                }
            })
            .collect();
        let Some(b) = self.project.boards.iter_mut().find(|b| b.id == board_id) else {
            return;
        };
        let mut edits = suivre_les_images(board_id, &mut b.images, (&suite, &neuves), tenue);
        // Le storyboard d'abord : ses cases le nomment, il existe avant elles (SEQUENCE-1).
        edits.push(regler_la_grille(board_id, &mut b.storyboards, sb, d));
        edits.extend(remplacer_la_suite(board_id, &mut b.panels, sb, neuves));
        self.record_as_one_gesture(edits);
    }
}

/// Chaque image suit la première case de la suite où son centre est posé — lue à sa place
/// d'avant, une seule fois —, sauf celles que la case tenue emportait.
fn suivre_les_images(
    board_id: &str,
    images: &mut [BoardImage],
    (suite, neuves): (&[StoryboardPanel], &[StoryboardPanel]),
    tenue: Option<(&str, &[String])>,
) -> Vec<Edit> {
    let case_de = |image: &BoardImage| match tenue {
        Some((id, emportees)) if emportees.contains(&image.id) => {
            suite.iter().position(|c| c.id == id)
        }
        _ => suite
            .iter()
            .position(|c| tenue.is_none_or(|(id, _)| c.id != id) && c.contient((image.x, image.y))),
    };
    let mut edits = Vec::new();
    for (k, image) in images.iter_mut().enumerate() {
        let Some(rang) = case_de(image) else {
            continue;
        };
        if image.locked {
            continue;
        }
        let avant = image.clone();
        suivre_le_cadre(image, &suite[rang], &neuves[rang]);
        edits.push(Edit::Image {
            board: board_id.to_string(),
            slot: Slot::changed(k, avant, image.clone()),
        });
    }
    edits
}

/// **La grille du storyboard `sb` devient celle de `d`** ; la durée et les champs restent les
/// siens. S'il n'existe pas, il naît au bout de la liste.
fn regler_la_grille(
    board_id: &str,
    storyboards: &mut Vec<Storyboard>,
    sb: &str,
    d: Disposition,
) -> Edit {
    let board = board_id.to_string();
    match storyboards.iter().position(|s| s.id == sb) {
        Some(i) => {
            let avant = storyboards[i].clone();
            storyboards[i].grille = d.grille;
            let slot = Slot::changed(i, avant, storyboards[i].clone());
            Edit::Storyboard { board, slot }
        }
        None => {
            let neuf = Storyboard::neuf(sb, d.grille);
            storyboards.push(neuf.clone());
            let slot = Slot::inserted(storyboards.len() - 1, neuf);
            Edit::Storyboard { board, slot }
        }
    }
}

/// **Les cases de `sb` remplacées par `neuves`, à leurs places dans la liste du tableau** : la
/// `k`-ième place reçoit la `k`-ième ; celles de trop partent, de la dernière à la première ;
/// celles qui manquent s'ajoutent au bout. Rend les éditions dans l'ordre où elles se rejouent —
/// une case qui ne change pas en donne une nulle, que le journal jette.
pub(super) fn remplacer_la_suite(
    board_id: &str,
    panels: &mut Vec<StoryboardPanel>,
    sb: &str,
    neuves: Vec<StoryboardPanel>,
) -> Vec<Edit> {
    let places: Vec<usize> = (0..panels.len())
        .filter(|&i| panels[i].storyboard == sb)
        .collect();
    let edit = |slot| Edit::Panel {
        board: board_id.to_string(),
        slot,
    };
    let mut edits = Vec::new();
    let mut neuves = neuves.into_iter();
    let mut de_trop = Vec::new();
    for &i in &places {
        match neuves.next() {
            Some(n) => {
                let avant = std::mem::replace(&mut panels[i], n.clone());
                edits.push(edit(Slot::changed(i, avant, n)));
            }
            None => de_trop.push(i),
        }
    }
    for &i in de_trop.iter().rev() {
        let partie = panels.remove(i);
        edits.push(edit(Slot::removed(i, partie)));
    }
    for n in neuves {
        edits.push(edit(Slot::inserted(panels.len(), n.clone())));
        panels.push(n);
    }
    edits
}
