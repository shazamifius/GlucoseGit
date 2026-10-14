//! **Le storyboard dans le fichier** : ses cases, et ce qui les range.
//!
//! * Schéma 1 à 2 : des cases seules — un numéro, une description, un cadre.
//! * Schéma 3 (STORYBOARD-2) : la grille du storyboard, gardée avec le tableau.
//! * Schéma 4 (fiche 67) : une case porte sa durée, son plan et le texte de ses champs ; le
//!   storyboard, la durée d'une case qui n'a pas la sienne et le nom de ses champs.
//! * **Schéma 5** (fiche 68) : un tableau porte plusieurs storyboards, chacun son identifiant,
//!   et chaque case nomme le sien.
//!
//! Ce qu'un schéma plus ancien ne disait pas se lit comme Glucose le voyait alors : une case
//! sans durée à elle, sans plan, sans texte ; un storyboard aux réglages de naissance
//! ([`Storyboard::neuf`]) ; le seul storyboard d'un tableau, et chacune de ses cases, sous
//! [`STORYBOARD_D_AVANT`] — le nom qu'un geste de son histoire lui donne aussi.

use super::bytes::{Reader, Writer};
use super::{ANIMATIQUE_DEPUIS, SEQUENCES_DEPUIS};
use crate::error::CoreResult;
use crate::types::{Format, Grille, Plan, Storyboard, StoryboardPanel, STORYBOARD_D_AVANT};

pub(super) fn write_panel(w: &mut Writer, panel: &StoryboardPanel) {
    let StoryboardPanel {
        id,
        storyboard,
        order,
        description,
        x,
        y,
        width,
        height,
        duree_ms,
        plan,
        textes,
    } = panel;

    w.text(id);
    w.i32(*order);
    w.text(description);
    w.f64(*x);
    w.f64(*y);
    w.f64(*width);
    w.f64(*height);
    w.opt(*duree_ms, |w, ms| w.uvarint(u64::from(ms)));
    w.opt(*plan, |w, p| w.u8(rang(&Plan::TOUS, p)));
    w.seq(textes, |w, t| w.text(t));
    w.text(storyboard);
}

/// Lit une case écrite au schéma `version`.
pub(super) fn read_panel(r: &mut Reader<'_>, version: u16) -> CoreResult<StoryboardPanel> {
    let mut panel = StoryboardPanel {
        id: r.text()?,
        storyboard: STORYBOARD_D_AVANT.to_string(),
        order: r.i32()?,
        description: r.text()?,
        x: r.f64()?,
        y: r.f64()?,
        width: r.f64()?,
        height: r.f64()?,
        duree_ms: None,
        plan: None,
        textes: Vec::new(),
    };
    if version >= ANIMATIQUE_DEPUIS {
        panel.duree_ms = r.opt(duree)?;
        panel.plan = r.opt(plan)?;
        panel.textes = r.seq(Reader::text)?;
    }
    if version >= SEQUENCES_DEPUIS {
        panel.storyboard = r.text()?;
    }
    Ok(panel)
}

/// **Un storyboard** : sa grille — le format par son rang dans [`Format::TOUS`], un numéro qui
/// ne bouge jamais (PERSIST-4) —, puis la durée d'une case, ses champs, et son identifiant.
pub(super) fn write_storyboard(w: &mut Writer, storyboard: &Storyboard) {
    let Storyboard {
        id,
        grille,
        duree_ms,
        champs,
    } = storyboard;
    let Grille {
        format,
        largeur,
        colonnes,
        ecart,
    } = grille;

    w.u8(rang(&Format::TOUS, *format));
    w.f64(*largeur);
    w.uvarint(*colonnes as u64);
    w.f64(*ecart);
    w.uvarint(u64::from(*duree_ms));
    w.seq(champs, |w, c| w.text(c));
    w.text(id);
}

/// Lit un storyboard écrit au schéma `version` (3 au moins : avant, il n'y en a pas).
pub(super) fn read_storyboard(r: &mut Reader<'_>, version: u16) -> CoreResult<Storyboard> {
    let rang = r.u8()?;
    let format = *Format::TOUS
        .get(usize::from(rang))
        .ok_or_else(|| super::tags::unknown("format de storyboard", rang))?;
    let grille = Grille {
        format,
        largeur: r.f64()?,
        colonnes: r.uvarint()? as usize,
        ecart: r.f64()?,
    };
    let mut storyboard = Storyboard::neuf(STORYBOARD_D_AVANT, grille);
    if version >= ANIMATIQUE_DEPUIS {
        storyboard.duree_ms = duree(r)?;
        storyboard.champs = r.seq(Reader::text)?;
    }
    if version >= SEQUENCES_DEPUIS {
        storyboard.id = r.text()?;
    }
    Ok(storyboard)
}

/// **Les storyboards d'un tableau** : une liste au schéma 5 ; un seul, ou aucun, aux schémas 3
/// et 4 ; aucun avant.
pub(super) fn write_storyboards(w: &mut Writer, storyboards: &[Storyboard]) {
    w.seq(storyboards, write_storyboard);
}

pub(super) fn read_storyboards(r: &mut Reader<'_>, version: u16) -> CoreResult<Vec<Storyboard>> {
    if version >= SEQUENCES_DEPUIS {
        r.seq(|rr| read_storyboard(rr, version))
    } else if version >= super::STORYBOARD_DEPUIS {
        Ok(r.opt(|rr| read_storyboard(rr, version))?
            .into_iter()
            .collect())
    } else {
        Ok(Vec::new())
    }
}

/// Le rang d'une valeur dans sa liste — son numéro dans le fichier.
fn rang<T: PartialEq>(tous: &[T], v: T) -> u8 {
    tous.iter().position(|x| *x == v).unwrap_or(0) as u8
}

/// Une durée en millisecondes : un nombre qui ne tient pas dans `u32` est un fichier abîmé.
fn duree(r: &mut Reader<'_>) -> CoreResult<u32> {
    let v = r.uvarint()?;
    u32::try_from(v).map_err(|_| {
        crate::error::CoreError::DeserializationError(format!(
            "durée de case hors des bornes ({v} ms) : le document est corrompu"
        ))
    })
}

fn plan(r: &mut Reader<'_>) -> CoreResult<Plan> {
    let rang = r.u8()?;
    Plan::TOUS
        .get(usize::from(rang))
        .copied()
        .ok_or_else(|| super::tags::unknown("plan de case", rang))
}
