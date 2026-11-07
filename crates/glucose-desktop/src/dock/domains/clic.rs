//! Le test de clic du panneau DOMAINES : il traduit un clic en intention, depuis la même liste de
//! rectangles que le dessin (DOM-UI-2).

use super::{Cible, DomainIntent, DomainRowLayout, DomainsPanelLayout, WEIGHT_STEPS};
use crate::params::Pointer;
use glucose_core::store::Store;

/// Traduit un clic en intention (DOM-UI-2). Rend `None` si le clic ne touche aucun widget.
///
/// `has_selection` grise les paliers et le retrait : un bouton dont l'action n'existe pas ne
/// fait rien plutôt que de simuler (§ 5.4).
pub fn hit_domains_panel(
    layout: &DomainsPanelLayout,
    _store: &Store,
    pointer: Pointer,
    has_selection: bool,
) -> Option<DomainIntent> {
    let (mx, my) = (pointer.x, pointer.y);
    if let Some((premier, lignes)) = &layout.ou {
        if layout.add_button.contains(mx, my) {
            return Some(DomainIntent::FermerOu);
        }
        return lignes
            .iter()
            .position(|r| r.contains(mx, my))
            .map(|k| DomainIntent::Aller(premier + k));
    }
    if layout.add_button.contains(mx, my) {
        return Some(DomainIntent::Create);
    }
    for row in &layout.rows {
        if row.ou.contains(mx, my) {
            return Some(DomainIntent::Ou(row.lignee.clone()));
        }
        let intent = match &row.cible {
            Cible::Ailleurs(lignee) => hit_ailleurs(row, lignee, pointer, has_selection),
            Cible::Ici(id) => hit_ici(row, id, pointer, has_selection),
        };
        if intent.is_some() {
            return intent;
        }
    }
    None
}

/// Un domaine d'un autre fichier : ses paliers le font entrer ici. Son nom et sa couleur vivent dans
/// ce fichier-là : ils ne se changent pas d'ici.
fn hit_ailleurs(
    row: &DomainRowLayout,
    lignee: &str,
    pointer: Pointer,
    has_selection: bool,
) -> Option<DomainIntent> {
    if !has_selection {
        return None;
    }
    let step = row
        .steps
        .iter()
        .position(|r| r.contains(pointer.x, pointer.y))?;
    Some(DomainIntent::Adopter {
        lignee: lignee.to_string(),
        weight: WEIGHT_STEPS[step],
    })
}

fn hit_ici(
    row: &DomainRowLayout,
    id: &str,
    pointer: Pointer,
    has_selection: bool,
) -> Option<DomainIntent> {
    let (mx, my) = (pointer.x, pointer.y);
    if let Some((yes, no)) = row.confirm {
        if yes.contains(mx, my) {
            return Some(DomainIntent::ConfirmDelete(id.to_string()));
        }
        if no.contains(mx, my) {
            return Some(DomainIntent::CancelDelete);
        }
    }
    if let Some((yes, no)) = row.question {
        if yes.contains(mx, my) {
            return Some(DomainIntent::MemeSujet(id.to_string()));
        }
        if no.contains(mx, my) {
            return Some(DomainIntent::Preciser(id.to_string()));
        }
    }
    hit_row(row, id, pointer, has_selection)
}

fn hit_row(
    row: &DomainRowLayout,
    id: &str,
    pointer: Pointer,
    has_selection: bool,
) -> Option<DomainIntent> {
    let (mx, my) = (pointer.x, pointer.y);
    if row.swatch.contains(mx, my) {
        return Some(DomainIntent::CycleColor(id.to_string()));
    }
    if row.sigil.contains(mx, my) {
        return Some(DomainIntent::CycleSigil(id.to_string()));
    }
    if row.name.contains(mx, my) {
        return Some(DomainIntent::StartRename(id.to_string()));
    }
    if row.delete.contains(mx, my) {
        return Some(DomainIntent::AskDelete(id.to_string()));
    }
    if !has_selection || row.confirm.is_some() || row.question.is_some() {
        return None;
    }
    if row.unassign.contains(mx, my) {
        return Some(DomainIntent::Unassign(id.to_string()));
    }
    for (step, rect) in row.steps.iter().enumerate() {
        if rect.contains(mx, my) {
            return Some(DomainIntent::Assign {
                domain_id: id.to_string(),
                weight: WEIGHT_STEPS[step],
            });
        }
    }
    None
}
