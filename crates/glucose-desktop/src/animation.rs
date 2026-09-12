//! Les animations en cours, et l'horloge que le noyau n'a pas.
//!
//! # Le partage des rôles
//!
//! [`glucose_core::anim`] tient les courbes et les durées, sans jamais demander l'heure : un
//! tween y est une fonction pure du temps écoulé. Ce module-ci apporte ce qui manque — un
//! `Instant` de départ, et la boucle qui redemande une frame tant qu'il reste quelque chose à
//! montrer.
//!
//! La frontière n'est pas décorative : elle rend **toutes les courbes de Glucose testables sans
//! attendre**, et ne laisse ici que ce qui ne peut pas l'être.
//!
//! # Pourquoi l'animation écrit dans le document
//!
//! Un vol de caméra pourrait se dessiner à côté, en passant au renderer un viewport de
//! substitution. Ce serait deux caméras : celle du document et celle qu'on voit. Elles
//! finiraient par diverger — et la minimap, le picking, le magnétisme liraient la mauvaise.
//!
//! L'animation écrit donc dans le viewport, par la porte unique `Store::set_viewport`. C'est
//! sans danger : la caméra n'entre jamais dans la pile d'annulation (loi UNDO-1), et tout ce
//! qui lit la position de la caméra lit la même.

use glucose_core::anim::{timing, Curve, Tween};
use glucose_core::membrane_focus::{fit_viewport, focus_consts, ScreenSize};
use glucose_core::geometry::Rect;
use glucose_core::store::Store;
use glucose_core::types::Viewport;
use std::time::Instant;

/// Ce qu'il faut faire une fois la caméra arrivée.
#[derive(Debug, Clone, PartialEq)]
pub enum Pending {
    /// Rien : le vol était le geste entier.
    Nothing,
    /// Entrer dans le dossier nommé.
    EnterFolder(String),
    /// Remonter à cette profondeur de la pile de dossiers.
    ExitToDepth(usize),
}

/// Un vol de caméra : d'un cadrage à un autre, en un temps donné, suivi d'une action.
#[derive(Debug, Clone)]
struct Flight {
    from: Viewport,
    to: Viewport,
    start: Instant,
    duration_ms: u32,
    curve: Curve,
    then: Pending,
    /// Le tableau dont la caméra est animée. Si le tableau actif change en route — l'action
    /// différée l'a fait, ou l'utilisateur est passé ailleurs — le vol s'arrête : il n'a plus
    /// de sujet.
    board: String,
}

/// Les animations en cours. Une seule à la fois pour la caméra : deux vols simultanés
/// tireraient le même viewport dans deux directions.
#[derive(Debug, Default)]
pub struct Animator {
    flight: Option<Flight>,
}

impl Animator {
    /// Aucune animation en cours.
    pub fn new() -> Self {
        Self::default()
    }

    /// Vrai si quelque chose bouge, donc s'il faut redessiner.
    pub fn is_running(&self) -> bool {
        self.flight.is_some()
    }

    /// Lance un vol de la caméra du tableau `board` vers `to`, suivi de `then`.
    ///
    /// Un vol déjà en cours est remplacé : le nouveau part de **là où la caméra est**, et non
    /// de là où le précédent avait commencé — sans quoi un second geste ferait sauter l'image.
    pub fn fly_to(
        &mut self,
        store: &Store,
        board: &str,
        to: Viewport,
        duration_ms: u32,
        then: Pending,
    ) {
        let from = store
            .project
            .boards
            .iter()
            .find(|b| b.id == board)
            .map(|b| b.viewport)
            .unwrap_or_default();
        self.flight = Some(Flight {
            from,
            to: to.normalized(),
            start: Instant::now(),
            duration_ms,
            curve: Curve::EaseOutCubic,
            then,
            board: board.to_string(),
        });
    }

    /// Cadre la boîte monde `b` et vole jusqu'à elle.
    ///
    /// Le cadrage passe par [`fit_viewport`], qui sert déjà au mode focus des membranes :
    /// une seule géométrie de cadrage dans tout le programme.
    pub fn fly_to_box(
        &mut self,
        store: &Store,
        board: &str,
        b: Rect,
        screen: ScreenSize,
        duration_ms: u32,
        then: Pending,
    ) {
        let to = fit_viewport(b, screen, focus_consts::FIT_PADDING);
        self.fly_to(store, board, to, duration_ms, then);
    }

    /// Fait avancer l'animation d'une image et rend le temps à attendre avant la suivante, en
    /// millisecondes — `None` s'il n'y a rien en cours.
    ///
    /// Applique l'action différée à l'arrivée. C'est le seul endroit où une animation touche au
    /// store, et il le fait par la porte unique du viewport.
    pub fn tick(&mut self, store: &mut Store) -> Option<u64> {
        let flight = self.flight.as_ref()?;
        // Le tableau a changé sous le vol : il n'a plus de sujet.
        if store.project.active_board_id != flight.board {
            self.flight = None;
            return None;
        }

        let elapsed = flight.start.elapsed().as_secs_f64() * 1000.0;
        let interpoler = |de: f64, vers: f64| {
            Tween::new(de, vers, flight.duration_ms, flight.curve).at(elapsed)
        };
        let vp = Viewport {
            x: interpoler(flight.from.x, flight.to.x),
            y: interpoler(flight.from.y, flight.to.y),
            // L'échelle s'interpole **géométriquement** : passer de 0,1 à 10 en ligne droite
            // passerait la moitié du vol au-dessus de l'échelle 5, et donnerait une plongée qui
            // s'arrête net. En raison, chaque pas multiplie l'échelle par le même facteur, et
            // le mouvement se lit à vitesse constante — c'est ce que fait un zoom à la molette,
            // dont chaque cran multiplie déjà.
            scale: (interpoler(flight.from.scale.ln(), flight.to.scale.ln())).exp(),
        };
        let board = flight.board.clone();
        store.set_viewport(&board, vp);

        let reste = Tween::new(0.0, 1.0, flight.duration_ms, flight.curve).remaining_ms(elapsed);
        if reste > 0.0 {
            return Some(reste.ceil() as u64);
        }

        // Arrivée : poser exactement la cible, puis exécuter ce qui suit.
        let cible = flight.to;
        let suite = flight.then.clone();
        store.set_viewport(&board, cible);
        self.flight = None;
        match suite {
            Pending::Nothing => {}
            Pending::EnterFolder(id) => {
                drop(store.try_enter_folder(&id));
            }
            Pending::ExitToDepth(depth) => {
                store.exit_to_depth(depth);
            }
        }
        Some(0)
    }

    /// Termine l'animation en cours **tout de suite** : la caméra saute à son cadrage et ce
    /// qui devait suivre a lieu.
    ///
    /// C'est ce qu'attend quelqu'un de pressé : recliquer pendant une plongée doit arriver, pas
    /// attendre. Rend `false` s'il n'y avait rien en cours.
    pub fn skip(&mut self, store: &mut Store) -> bool {
        let Some(flight) = self.flight.as_mut() else {
            return false;
        };
        flight.duration_ms = 0;
        self.tick(store).is_some()
    }

    /// Abandonne l'animation en cours **sans** exécuter ce qui devait suivre.
    ///
    /// Pour Échap, ou pour tout geste qui reprend la main sur la caméra : une plongée qu'on
    /// interrompt ne doit pas ouvrir le dossier à l'arrivée.
    pub fn cancel(&mut self) -> bool {
        self.flight.take().is_some()
    }
}

#[cfg(test)]
mod tests;

// ── Les deux gestes de navigation, animés ────────────────────────────────────

/// Plonge la caméra sur un dossier, puis y entre (fiche 07 § 1, `FOLDER_TRANSITION`).
///
/// La bascule de tableau n'a lieu qu'à l'arrivée : pendant le vol, on regarde encore le
/// tableau parent, et le dossier grandit jusqu'à occuper l'écran. C'est ce qui donne la
/// sensation d'y entrer plutôt que d'y être transporté.
///
/// Rend `false` si le dossier n'existe pas — l'appelant bascule alors sans animation plutôt
/// que de ne rien faire.
pub fn fly_into_folder(
    store: &Store,
    animator: &mut Animator,
    folder_id: &str,
    screen: ScreenSize,
) -> bool {
    let Some(f) = store
        .active_board()
        .and_then(|b| b.folders.iter().find(|f| f.id == folder_id))
    else {
        return false;
    };
    let boite = Rect::new(f.x, f.y, f.width, f.height);
    let board = store.project.active_board_id.clone();
    animator.fly_to_box(
        store,
        &board,
        boite,
        screen,
        timing::FOLDER_TRANSITION_MS,
        Pending::EnterFolder(folder_id.to_string()),
    );
    true
}

/// Remonte à une profondeur, puis fait remonter la caméra depuis le dossier quitté.
///
/// L'ordre est l'inverse de l'entrée, et il le faut : on ne peut pas voir le tableau parent
/// avant d'y être. On remonte donc d'abord, on **pose** la caméra serrée sur le dossier d'où
/// l'on sort, et on la laisse revenir au cadrage que le parent avait quand on l'a quitté.
///
/// Rend `false` si l'on était déjà à cette profondeur.
pub fn fly_out_to_depth(
    store: &mut Store,
    animator: &mut Animator,
    depth: usize,
    screen: ScreenSize,
) -> bool {
    // Le dossier par lequel on redescendrait, et le tableau où l'on va : lus avant de bouger.
    let Some((parent_board, folder_id)) = store.folder_stack.get(depth).cloned() else {
        return false;
    };
    let cadrage_du_parent = store
        .project
        .boards
        .iter()
        .find(|b| b.id == parent_board)
        .map(|b| b.viewport)
        .unwrap_or_default();
    let boite = store
        .project
        .boards
        .iter()
        .find(|b| b.id == parent_board)
        .and_then(|b| b.folders.iter().find(|f| f.id == folder_id))
        .map(|f| Rect::new(f.x, f.y, f.width, f.height));

    if !store.exit_to_depth(depth) {
        return false;
    }
    if let Some(boite) = boite {
        let serre = fit_viewport(boite, screen, focus_consts::FIT_PADDING);
        store.set_viewport(&parent_board, serre);
        animator.fly_to(
            store,
            &parent_board,
            cadrage_du_parent,
            timing::FOLDER_TRANSITION_MS,
            Pending::Nothing,
        );
    }
    true
}
