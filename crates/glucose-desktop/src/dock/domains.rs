//! Le panneau DOMAINES — une **vue** du catalogue du document, et rien d'autre.
//!
//! # DOM-UI-1 — le panneau ne détient aucune copie du document
//!
//! Il tenait auparavant sa propre `Vec<DomainItem>`, alimentée par un bouton qui poussait
//! dedans un `format!("domain-{}", len + 1)`. `store.add_domain` n'était jamais appelé :
//! l'utilisateur cliquait vingt fois et obtenait vingt lignes décoratives, absentes de tout
//! document, assignables à rien, perdues à la fermeture. C'est le constat **R-47**, et c'est
//! précisément pourquoi il n'y a plus une seule donnée de domaine ici : ce qui s'affiche est
//! lu dans `store.project.domains` à chaque frame, et ce qui se clique part en
//! [`DomainIntent`] vers le moteur de commandes.
//!
//! Ce que [`DomainsUi`] garde est ce qui **n'appartient pas** au document et disparaît avec la
//! fenêtre : quel domaine attend une confirmation de suppression, et quel nom est en cours de
//! frappe. Aucun de ces deux états n'a de sens une fois le geste terminé, et aucun ne survit à
//! un `undo` — ce qui est la définition d'un état d'interface.
//!
//! # DOM-UI-2 — dessin et test de clic lisent la MÊME liste
//!
//! [`layout_domains_panel`] produit la liste des rectangles ; [`render_domains_panel`] la
//! dessine et [`hit_domains_panel`] la teste. Un bouton ne peut pas cliquer ailleurs qu'où il
//! est dessiné (loi L4, standard § 5.1).
//!
//! # DOM-UI-3 — le coût du panneau ne dépend pas de la taille du document
//!
//! Le panneau ne parcourt que le **catalogue**, jamais les nœuds : ni comptage d'usages, ni
//! relevé des pondérations de la sélection, qui seraient l'un et l'autre en O(nœuds) par frame
//! (loi L2). La pondération d'un nœud se lit là où elle a un sens — sur le canevas, dans sa
//! réglette. Ce que le panneau affiche de la sélection tient en un `len()`.

use crate::interactions::text_entry::TextEntry;
use crate::params::{Pointer, ScaledRect};
use crate::theme::{DOMAIN_PALETTE, DOMAIN_SIGILS};
use glucose_core::store::Store;

use super::WidgetRect;

/// Les paliers de pondération offerts par le panneau.
///
/// Cinq, parce qu'un curseur continu n'apporte rien ici : une pondération sémantique se pense
/// en « un peu / moyennement / beaucoup », pas au centième. Chacun est un bouton dont l'action
/// existe (§ 5.4) : un clic assigne, immédiatement, à toute la sélection.
pub const WEIGHT_STEPS: [f64; 5] = [0.2, 0.4, 0.6, 0.8, 1.0];

/// Mesures du panneau, en **unités logiques** (§ 5.3), converties une seule fois.
#[derive(Clone, Copy, Debug)]
struct Metrics {
    pad: f32,
    title_baseline: f32,
    header_baseline: f32,
    rows_top: f32,
    row_height: f32,
    row_gap: f32,
    line_gap: f32,
    swatch: f32,
    sigil_width: f32,
    icon_button: f32,
    segment_height: f32,
    segment_gap: f32,
    unassign_width: f32,
    add_height: f32,
    hint_height: f32,
    title_size: f32,
    name_size: f32,
    hint_size: f32,
    sigil_size: f32,
    step_size: f32,
}

impl Metrics {
    const fn logical() -> Self {
        Self {
            pad: 12.0,
            title_baseline: 12.0,
            header_baseline: 32.0,
            rows_top: 50.0,
            row_height: 40.0,
            row_gap: 6.0,
            line_gap: 22.0,
            swatch: 16.0,
            sigil_width: 28.0,
            icon_button: 18.0,
            segment_height: 15.0,
            segment_gap: 4.0,
            unassign_width: 26.0,
            add_height: 26.0,
            hint_height: 16.0,
            title_size: 13.0,
            name_size: 11.5,
            hint_size: 9.0,
            sigil_size: 9.0,
            step_size: 9.0,
        }
    }

    /// L'unique conversion logique → physique du panneau (§ 5.3).
    fn scaled(s: f32) -> Self {
        let l = Self::logical();
        Self {
            pad: l.pad * s,
            title_baseline: l.title_baseline * s,
            header_baseline: l.header_baseline * s,
            rows_top: l.rows_top * s,
            row_height: l.row_height * s,
            row_gap: l.row_gap * s,
            line_gap: l.line_gap * s,
            swatch: l.swatch * s,
            sigil_width: l.sigil_width * s,
            icon_button: l.icon_button * s,
            segment_height: l.segment_height * s,
            segment_gap: l.segment_gap * s,
            unassign_width: l.unassign_width * s,
            add_height: l.add_height * s,
            hint_height: l.hint_height * s,
            title_size: l.title_size * s,
            name_size: l.name_size * s,
            hint_size: l.hint_size * s,
            sigil_size: l.sigil_size * s,
            step_size: l.step_size * s,
        }
    }
}

// ── L'état d'interaction, et rien de plus ───────────────────────────────────

/// Un renommage en cours : quel domaine, et ce qui a été tapé jusqu'ici.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DomainRename {
    pub domain_id: String,
    pub entry: TextEntry,
}

/// L'état d'interaction du panneau DOMAINES (DOM-UI-1). Aucune donnée de domaine ici.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DomainsUi {
    /// Domaine dont la suppression attend une confirmation.
    pub pending_delete: Option<String>,
    /// Saisie de renommage en cours.
    pub rename: Option<DomainRename>,
}

impl DomainsUi {
    /// Abandonne tout geste en cours — au changement de document, par exemple, où les
    /// identifiants retenus ici ne désignent plus rien.
    pub fn reset(&mut self) {
        self.pending_delete = None;
        self.rename = None;
    }
}

// ── Ce que l'utilisateur demande ────────────────────────────────────────────

/// Une intention issue du panneau. Aucune variante ne mute quoi que ce soit : elles nomment un
/// geste, le document est modifié ailleurs (standard § 1.7).
#[derive(Debug, Clone, PartialEq)]
pub enum DomainIntent {
    Create,
    StartRename(String),
    CycleColor(String),
    CycleSigil(String),
    AskDelete(String),
    CancelDelete,
    ConfirmDelete(String),
    Assign { domain_id: String, weight: f64 },
    Unassign(String),
}

// ── Mise en page ────────────────────────────────────────────────────────────

/// Les rectangles d'une ligne de domaine. `index` désigne `store.project.domains[index]` :
/// aucune chaîne n'est recopiée par frame (§ 4.3).
#[derive(Debug, Clone, PartialEq)]
pub struct DomainRowLayout {
    pub index: usize,
    pub swatch: WidgetRect,
    pub sigil: WidgetRect,
    pub name: WidgetRect,
    pub delete: WidgetRect,
    /// Les cinq paliers, du plus faible au plus fort.
    pub steps: Vec<WidgetRect>,
    pub unassign: WidgetRect,
    /// `Some((oui, non))` quand cette ligne attend une confirmation de suppression : les deux
    /// boutons prennent alors la place de la ligne des paliers.
    pub confirm: Option<(WidgetRect, WidgetRect)>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DomainsPanelLayout {
    pub rows: Vec<DomainRowLayout>,
    pub add_button: WidgetRect,
    /// Domaines qui ne tiennent pas dans la hauteur du panneau.
    pub hidden: usize,
}

/// Produit l'unique liste de rectangles du panneau (DOM-UI-2).
pub fn layout_domains_panel(
    frame: ScaledRect,
    store: &Store,
    ui: &DomainsUi,
) -> DomainsPanelLayout {
    let s = crate::theme::clamp_ui_scale(frame.scale);
    let m = Metrics::scaled(s);
    let (px, py, pw, ph) = (frame.x, frame.y, frame.w, frame.h);

    let add_button = WidgetRect::new(
        px + m.pad,
        py + ph - m.pad - m.add_height,
        pw - 2.0 * m.pad,
        m.add_height,
    );
    let rows_bottom = add_button.y - m.hint_height - m.row_gap;
    let pitch = m.row_height + m.row_gap;
    let room = ((rows_bottom - (py + m.rows_top)) / pitch).floor().max(0.0) as usize;
    let shown = store.project.domains.len().min(room);

    let mut rows = Vec::with_capacity(shown);
    for index in 0..shown {
        let top = py + m.rows_top + index as f32 * pitch;
        let pending = ui
            .pending_delete
            .as_deref()
            .is_some_and(|id| store.project.domains.get(index).is_some_and(|d| d.id == id));
        rows.push(layout_row(index, (px, top, pw), &m, pending));
    }

    DomainsPanelLayout { rows, add_button, hidden: store.project.domains.len() - shown }
}

/// Les deux lignes d'un domaine : identité en haut, pondérations en bas.
fn layout_row(index: usize, at: (f32, f32, f32), m: &Metrics, pending: bool) -> DomainRowLayout {
    let (px, top, pw) = at;
    let right = px + pw - m.pad;
    let swatch = WidgetRect::new(px + m.pad, top, m.swatch, m.swatch);
    let sigil = WidgetRect::new(swatch.x + swatch.w + m.pad / 3.0, top, m.sigil_width, m.swatch);
    let delete = WidgetRect::new(right - m.icon_button, top, m.icon_button, m.icon_button);
    // Le nom part de la fin du sigle plutôt que d'une abscisse recopiée : deux constantes de
    // mise en page qui doivent s'accorder finissent toujours par diverger (§ 5.2).
    let name_left = sigil.x + sigil.w + m.pad / 3.0;
    let name = WidgetRect::new(name_left, top, (delete.x - name_left - m.pad / 2.0).max(0.0), m.swatch);

    let second = top + m.line_gap;
    let unassign = WidgetRect::new(right - m.unassign_width, second, m.unassign_width, m.segment_height);
    let steps_width = unassign.x - m.segment_gap - (px + m.pad);
    let step_width =
        ((steps_width - (WEIGHT_STEPS.len() - 1) as f32 * m.segment_gap) / WEIGHT_STEPS.len() as f32).max(0.0);
    let steps = (0..WEIGHT_STEPS.len())
        .map(|i| {
            WidgetRect::new(
                px + m.pad + i as f32 * (step_width + m.segment_gap),
                second,
                step_width,
                m.segment_height,
            )
        })
        .collect();

    // La confirmation occupe la moitié droite de la seconde ligne : la question reste lisible
    // à gauche, et les deux réponses ne se confondent pas avec un palier.
    let confirm = pending.then(|| {
        let half = (m.unassign_width * 2.0).max(step_width);
        let yes = WidgetRect::new(right - 2.0 * half - m.segment_gap, second, half, m.segment_height);
        let no = WidgetRect::new(right - half, second, half, m.segment_height);
        (yes, no)
    });

    DomainRowLayout { index, swatch, sigil, name, delete, steps, unassign, confirm }
}

// ── Test de clic ────────────────────────────────────────────────────────────

/// Traduit un clic en intention (DOM-UI-2). Rend `None` si le clic ne touche aucun widget.
///
/// `has_selection` grise les paliers et le retrait : un bouton dont l'action n'existe pas ne
/// fait rien plutôt que de simuler (§ 5.4).
pub fn hit_domains_panel(
    layout: &DomainsPanelLayout,
    store: &Store,
    pointer: Pointer,
    has_selection: bool,
) -> Option<DomainIntent> {
    let (mx, my) = (pointer.x, pointer.y);
    if layout.add_button.contains(mx, my) {
        return Some(DomainIntent::Create);
    }
    for row in &layout.rows {
        let id = store.project.domains.get(row.index)?.id.clone();
        if let Some((yes, no)) = row.confirm {
            if yes.contains(mx, my) {
                return Some(DomainIntent::ConfirmDelete(id));
            }
            if no.contains(mx, my) {
                return Some(DomainIntent::CancelDelete);
            }
        }
        if let Some(intent) = hit_row(row, &id, pointer, has_selection) {
            return Some(intent);
        }
    }
    None
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
    if !has_selection || row.confirm.is_some() {
        return None;
    }
    if row.unassign.contains(mx, my) {
        return Some(DomainIntent::Unassign(id.to_string()));
    }
    for (step, rect) in row.steps.iter().enumerate() {
        if rect.contains(mx, my) {
            return Some(DomainIntent::Assign { domain_id: id.to_string(), weight: WEIGHT_STEPS[step] });
        }
    }
    None
}

// ── Les palettes d'édition ──────────────────────────────────────────────────

/// La couleur qui suit `current` dans la palette du thème.
///
/// Un vrai sélecteur de couleur est un panneau à part entière ; ce bouton-ci fait ce qu'il
/// annonce et le fait entièrement : il parcourt les huit teintes du thème, choisies pour
/// rester distinctes sur le fond du canevas. Une couleur absente de la palette (document venu
/// d'ailleurs) repart de la première.
pub fn next_color(current: &str) -> &'static str {
    let at = DOMAIN_PALETTE.iter().position(|c| c.eq_ignore_ascii_case(current));
    DOMAIN_PALETTE[at.map_or(0, |i| (i + 1) % DOMAIN_PALETTE.len())]
}

/// Le sigle qui suit `current` dans la palette du thème.
pub fn next_sigil(current: &str) -> &'static str {
    let at = DOMAIN_SIGILS.iter().position(|c| c.eq_ignore_ascii_case(current));
    DOMAIN_SIGILS[at.map_or(0, |i| (i + 1) % DOMAIN_SIGILS.len())]
}

/// La couleur et le sigle à donner au `n`-ième domaine créé.
///
/// Deux domaines créés à la suite ne se ressemblent pas : c'est la seule chose qui rende la
/// réglette lisible sans aller lire le panneau.
pub fn fresh_look(rank: usize) -> (&'static str, &'static str) {
    (
        DOMAIN_PALETTE[rank % DOMAIN_PALETTE.len()],
        DOMAIN_SIGILS[rank % DOMAIN_SIGILS.len()],
    )
}

mod paint;

pub use paint::render_domains_panel;

#[cfg(test)]
mod tests;
