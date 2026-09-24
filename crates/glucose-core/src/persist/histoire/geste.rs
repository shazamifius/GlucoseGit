//! Un **geste** écrit sur le disque : la transaction du journal d'annulation, telle quelle.
//!
//! # Pourquoi aucune forme nouvelle
//!
//! Le journal d'annulation enregistre déjà chaque geste comme la seule chose qu'il faut pour
//! le rejouer : une suite d'éditions qui portent chacune leur **avant** et leur **après**
//! (JRN-1). L'écrire sur le disque ne demande donc aucun modèle de plus — et c'est ce qui rend
//! le retour dans le temps possible dans les deux sens : l'avant d'une édition la défait,
//! l'après la refait.
//!
//! Les rangs (`Slot::index`) sont ceux du moment du geste. Ils restent justes à la relecture
//! parce que l'histoire se rejoue **dans l'ordre** où elle a été écrite, depuis la base ou un
//! instantané — exactement la condition que JRN-2 demande au journal en mémoire.
//!
//! # INVARIANT PERSIST-4 — les numéros des éditions ne bougent jamais
//!
//! Même règle que [`super::super::tags`] : une variante nouvelle prend le numéro suivant,
//! aucune n'est renumérotée. Un numéro inconnu refuse l'entrée au lieu de la deviner.

use super::super::annotation::{read_annotation, write_annotation};
use super::super::bytes::{Reader, Writer};
use super::super::document::{
    read_board, read_domain, read_folder, read_panel, read_preset, read_zone, write_board,
    write_domain, write_folder, write_panel, write_preset, write_zone,
};
use super::super::image::{read_image, write_image};
use super::super::tags::unknown;
use crate::error::CoreResult;
use crate::store::journal::{Bouts, Edit, Slot, Transaction, Whole};

const IMAGE: u8 = 0;
const ANNOTATION: u8 = 1;
const DOSSIER: u8 = 2;
const PANNEAU: u8 = 3;
const ZONES: u8 = 4;
const NOM_DU_TABLEAU: u8 = 5;
const TRANSLATION: u8 = 6;
const TABLEAU: u8 = 7;
const DOMAINE: u8 = 8;
const PRESET: u8 = 9;
const NOM_DU_PROJET: u8 = 10;
const TABLEAU_ACTIF: u8 = 11;

/// Un geste tel que l'histoire le garde : quand, par qui, et quoi.
#[derive(Debug, Clone, PartialEq)]
pub struct Geste {
    /// Millisecondes Unix, fournies par l'appelant : le noyau ne lit jamais l'horloge.
    pub instant: i64,
    /// Qui l'a fait : un identifiant par installation. Seul pour l'instant ; c'est ce qui
    /// distinguera les mains quand le document sera partagé (fiche 36, phase 5).
    pub auteur: u64,
    pub transaction: Transaction,
}

pub fn ecrire(w: &mut Writer, geste: &Geste) {
    w.i64(geste.instant);
    w.u64(geste.auteur);
    w.seq(&geste.transaction.edits, ecrire_edition);
}

/// Relit un geste écrit au schéma de document `version`.
pub fn lire(r: &mut Reader<'_>, version: u16) -> CoreResult<Geste> {
    Ok(Geste {
        instant: r.i64()?,
        auteur: r.u64()?,
        transaction: Transaction {
            edits: r.seq(|rr| lire_edition(rr, version))?,
        },
    })
}

fn ecrire_edition(w: &mut Writer, edit: &Edit) {
    match edit {
        Edit::Image { board, slot } => {
            w.u8(IMAGE);
            w.text(board);
            ecrire_case(w, slot, write_image);
        }
        Edit::Annotation { board, slot } => {
            w.u8(ANNOTATION);
            w.text(board);
            ecrire_case(w, slot, write_annotation);
        }
        Edit::Folder { board, slot } => {
            w.u8(DOSSIER);
            w.text(board);
            ecrire_case(w, slot, write_folder);
        }
        Edit::Panel { board, slot } => {
            w.u8(PANNEAU);
            w.text(board);
            ecrire_case(w, slot, write_panel);
        }
        Edit::Zones { board, whole } => {
            w.u8(ZONES);
            w.text(board);
            w.seq(&whole.before, write_zone);
            w.seq(&whole.after, write_zone);
        }
        Edit::BoardName { board, whole } => {
            w.u8(NOM_DU_TABLEAU);
            w.text(board);
            ecrire_textes(w, whole);
        }
        Edit::Translation {
            board,
            delta,
            images,
            annotations,
            folders,
            bouts,
        } => {
            w.u8(TRANSLATION);
            w.text(board);
            w.f64(delta.0);
            w.f64(delta.1);
            for rangs in [images, annotations, folders] {
                w.seq(rangs, |ww, r| ww.uvarint(u64::from(*r)));
            }
            w.seq(bouts, |ww, (r, b)| {
                ww.uvarint(u64::from(*r));
                ww.u8(u8::from(b.origine) | (u8::from(b.cible) << 1));
            });
        }
        Edit::Board { slot } => {
            w.u8(TABLEAU);
            ecrire_case(w, slot, write_board);
        }
        Edit::Domain { slot } => {
            w.u8(DOMAINE);
            ecrire_case(w, slot, write_domain);
        }
        Edit::Preset { slot } => {
            w.u8(PRESET);
            ecrire_case(w, slot, write_preset);
        }
        Edit::ProjectName { whole } => {
            w.u8(NOM_DU_PROJET);
            ecrire_textes(w, whole);
        }
        Edit::ActiveBoard { whole } => {
            w.u8(TABLEAU_ACTIF);
            ecrire_textes(w, whole);
        }
    }
}

fn lire_edition(r: &mut Reader<'_>, version: u16) -> CoreResult<Edit> {
    Ok(match r.u8()? {
        IMAGE => Edit::Image {
            board: r.text()?,
            slot: lire_case(r, |rr| read_image(rr, version))?,
        },
        ANNOTATION => Edit::Annotation {
            board: r.text()?,
            slot: lire_case(r, read_annotation)?,
        },
        DOSSIER => Edit::Folder {
            board: r.text()?,
            slot: lire_case(r, read_folder)?,
        },
        PANNEAU => Edit::Panel {
            board: r.text()?,
            slot: lire_case(r, read_panel)?,
        },
        ZONES => Edit::Zones {
            board: r.text()?,
            whole: Whole::new(r.seq(read_zone)?, r.seq(read_zone)?),
        },
        NOM_DU_TABLEAU => Edit::BoardName {
            board: r.text()?,
            whole: lire_textes(r)?,
        },
        TRANSLATION => lire_translation(r)?,
        TABLEAU => Edit::Board {
            slot: lire_case(r, |rr| read_board(rr, version))?,
        },
        DOMAINE => Edit::Domain {
            slot: lire_case(r, read_domain)?,
        },
        PRESET => Edit::Preset {
            slot: lire_case(r, read_preset)?,
        },
        NOM_DU_PROJET => Edit::ProjectName {
            whole: lire_textes(r)?,
        },
        TABLEAU_ACTIF => Edit::ActiveBoard {
            whole: lire_textes(r)?,
        },
        autre => return Err(unknown("édition de l'histoire", autre)),
    })
}

fn lire_translation(r: &mut Reader<'_>) -> CoreResult<Edit> {
    let board = r.text()?;
    let delta = (r.f64()?, r.f64()?);
    let rang = |rr: &mut Reader<'_>| -> CoreResult<u32> {
        u32::try_from(rr.uvarint()?).map_err(|_| unknown("rang d'un déplacement", 0xff))
    };
    let images = r.seq(rang)?;
    let annotations = r.seq(rang)?;
    let folders = r.seq(rang)?;
    let bouts = r.seq(|rr| {
        let i = rang(rr)?;
        let quels = rr.u8()?;
        Ok((
            i,
            Bouts {
                origine: quels & 1 != 0,
                cible: quels & 2 != 0,
            },
        ))
    })?;
    Ok(Edit::Translation {
        board,
        delta,
        images,
        annotations,
        folders,
        bouts,
    })
}

/// Une case : son rang, puis son avant et son après, chacun présent ou non.
fn ecrire_case<T>(w: &mut Writer, slot: &Slot<T>, ecrire: fn(&mut Writer, &T)) {
    w.uvarint(slot.index as u64);
    w.opt(slot.before.as_deref(), ecrire);
    w.opt(slot.after.as_deref(), ecrire);
}

fn lire_case<T>(
    r: &mut Reader<'_>,
    lire: impl Fn(&mut Reader<'_>) -> CoreResult<T>,
) -> CoreResult<Slot<T>> {
    let index = usize::try_from(r.uvarint()?).map_err(|_| unknown("rang d'une case", 0xff))?;
    Ok(Slot {
        index,
        before: r.opt(&lire)?.map(Box::new),
        after: r.opt(&lire)?.map(Box::new),
    })
}

fn ecrire_textes(w: &mut Writer, whole: &Whole<String>) {
    w.text(&whole.before);
    w.text(&whole.after);
}

fn lire_textes(r: &mut Reader<'_>) -> CoreResult<Whole<String>> {
    Ok(Whole::new(r.text()?, r.text()?))
}
