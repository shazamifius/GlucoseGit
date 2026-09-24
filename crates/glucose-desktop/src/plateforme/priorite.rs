//! **CEDER-1 — le fil qui dessine passe devant l'atelier.**
//!
//! # Ce que la chronique montrait sans pouvoir le nommer
//!
//! Depuis la mémoire par étages, les pires images d'un zoom portaient 21 à 34 ms sur un poste
//! qui n'en coûte d'ordinaire qu'un : `docks`, puis `relever`, juste après. C'est dans ce poste
//! que le magasin confie à l'atelier les niveaux à offrir, à reprendre ou à redécoder — et
//! qu'il **réveille ses ouvriers**, un par ordre.
//!
//! L'atelier lance un ouvrier par cœur sauf un, « pour que celui qui reste tienne la
//! cadence ». Mais rien ne réserve ce cœur : les ouvriers ont la priorité du fil qui dessine,
//! Windows dope un fil qu'on réveille — davantage encore dans l'application au premier plan
//! —, et les bandes que chaque image lance pour se composer attendent leur tour derrière eux.
//! Un tour, c'est un quantum de l'ordonnanceur : 16 à 31 ms.
//!
//! # Mesuré avant d'être cru (`bench_ceder`)
//!
//! Sur la machine de l'utilisateur (16 fils logiques, donc 15 ouvriers), une image de 2 ms
//! pendant que l'atelier est plein : **22,4 ms en médiane, 33,9 au p99** avec des ouvriers de
//! même priorité — les chiffres de sa chronique ; **2,7 ms en médiane, 3,4 au p99** quand ils
//! cèdent. La même image, le même travail d'atelier : seul l'ordre de passage change.
//!
//! # Ce qui est demandé au système, et pourquoi pas davantage
//!
//! Un cran sous la normale, et **pas de dopage au réveil** : l'ouvrier ne passe jamais devant
//! le fil qui dessine, ni devant ceux des autres applications — c'est la charte, qui veut que
//! Glucose se niche là où il reste des ressources, et ne dispute jamais une machine à Blender.
//! Pas plus bas : le mode « arrière-plan » de Windows baisse aussi la priorité des lectures et
//! des pages, et une photo qu'on vient de zoomer doit se décoder vite. Un fil qui cède n'est
//! pas un fil qui s'arrête : il prend tout ce que les autres laissent, et Windows relève
//! lui-même un fil qui attend depuis quatre secondes.
//!
//! # Ailleurs que sous Windows
//!
//! Rien, pour l'instant : l'application ne tourne que sous Windows (fiche 36, phase 9). Linux
//! demandera `setpriority` sur le fil, macOS une classe de service `utility`.

/// Fait céder le fil courant au fil qui dessine — et aux autres applications. À appeler au
/// début d'un fil qui travaille pour plus tard : un ouvrier de l'atelier.
pub fn ceder_au_fil_qui_dessine() {
    natif::ceder();
}

/// Le fil courant cède-t-il ? Ce que le système en dit, pour que l'épreuve le lise au lieu de
/// le supposer. `None` là où l'on ne sait pas le demander.
pub fn cede() -> Option<bool> {
    natif::cede()
}

#[cfg(windows)]
mod natif {
    use windows::Win32::System::Threading::{
        GetCurrentThread, GetThreadPriority, GetThreadPriorityBoost, SetThreadPriority,
        SetThreadPriorityBoost, THREAD_PRIORITY_BELOW_NORMAL,
    };

    pub fn ceder() {
        // Le pseudo-handle du fil courant : il ne s'ouvre ni ne se ferme. Un refus laisse le
        // fil à sa priorité — il travaille, simplement sans céder.
        let moi = unsafe { GetCurrentThread() };
        let _ = unsafe { SetThreadPriority(moi, THREAD_PRIORITY_BELOW_NORMAL) };
        let _ = unsafe { SetThreadPriorityBoost(moi, true) };
    }

    pub fn cede() -> Option<bool> {
        let moi = unsafe { GetCurrentThread() };
        let priorite = unsafe { GetThreadPriority(moi) };
        let mut sans_dopage = windows::core::BOOL::default();
        unsafe { GetThreadPriorityBoost(moi, &mut sans_dopage) }.ok()?;
        Some(priorite == THREAD_PRIORITY_BELOW_NORMAL.0 && sans_dopage.as_bool())
    }
}

#[cfg(not(windows))]
mod natif {
    pub fn ceder() {}

    pub fn cede() -> Option<bool> {
        None
    }
}

#[cfg(test)]
mod tests {
    /// Le système dit que le fil cède, une fois qu'on le lui a demandé — et pas avant : un
    /// fil neuf garde la priorité normale, celle du fil qui dessine.
    #[test]
    fn test_un_fil_qui_cede_le_dit_au_systeme() {
        let (avant, apres) = std::thread::spawn(|| {
            let avant = super::cede();
            super::ceder_au_fil_qui_dessine();
            (avant, super::cede())
        })
        .join()
        .expect("le fil d'épreuve");
        if cfg!(windows) {
            assert_eq!(avant, Some(false), "un fil neuf ne cède pas");
            assert_eq!(apres, Some(true));
        }
    }
}
