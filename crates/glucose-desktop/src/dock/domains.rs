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
//!
//! # DOM-UI-4 — le panneau montre aussi ce que les autres fichiers portent (fiche 88)
//!
//! Un domaine traverse les fichiers : le panneau liste donc, sous ceux du document, ceux qu'on
//! connaît d'ailleurs. [`Connus`] est une vue **déduite** du recensement, refaite quand il change : ce
//! n'est pas une copie du document, et DOM-UI-1 tient. Un domaine dont le nom désigne aussi un autre
//! sujet ailleurs ne reçoit plus aucun palier tant qu'on n'a pas répondu : **même sujet, ou mot à
//! préciser** (sa règle du 7/11/2026).

use crate::interactions::text_entry::TextEntry;
use crate::params::ScaledRect;
use crate::theme::{DOMAIN_PALETTE, DOMAIN_SIGILS};
use glucose_core::store::{lignee_de, Store};
use glucose_core::types::Domain;

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
    ou_width: f32,
    voir_width: f32,
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
            ou_width: 22.0,
            voir_width: 26.0,
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
            ou_width: l.ou_width * s,
            voir_width: l.voir_width * s,
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

/// **Un domaine connu d'un autre fichier**, que ce document ne porte pas.
#[derive(Debug, Clone, PartialEq)]
pub struct Ailleurs {
    pub lignee: String,
    pub definition: Domain,
    pub noeuds: usize,
    pub fichiers: usize,
}

/// **Ce que les autres fichiers disent** (DOM-UI-4), déduit du recensement.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Connus {
    pub ailleurs: Vec<Ailleurs>,
    /// Les domaines de ce document dont le nom désigne aussi un autre sujet ailleurs.
    pub homonymes: Vec<String>,
    /// Pour un domaine de ce document : combien de nœuds et de fichiers le portent aussi ailleurs.
    pub aussi: Vec<(String, usize, usize)>,
}

/// **Une place où vit un domaine.**
#[derive(Debug, Clone, PartialEq)]
pub struct LigneOu {
    pub chemin: String,
    pub tableau: String,
    pub noeud: String,
    /// La première phrase du nœud, ou ce qu'il est s'il n'a pas de texte.
    pub texte: String,
    /// Dans le document ouvert ?
    pub ici: bool,
}

/// **« Où ? »** : chaque place où vit un domaine, tous fichiers confondus.
#[derive(Debug, Clone, PartialEq)]
pub struct Ou {
    pub nom: String,
    pub lignes: Vec<LigneOu>,
}

/// L'état d'interaction du panneau DOMAINES (DOM-UI-1), et ce que le recensement en déduit
/// (DOM-UI-4). Aucune copie du document ici.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct DomainsUi {
    /// Domaine dont la suppression attend une confirmation.
    pub pending_delete: Option<String>,
    /// Saisie de renommage en cours.
    pub rename: Option<DomainRename>,
    pub connus: Connus,
    /// La liste « où ? », quand elle remplace les domaines.
    pub ou: Option<Ou>,
    /// La première rangée montrée : la molette fait défiler.
    pub decalage: usize,
}

impl DomainsUi {
    /// Abandonne tout geste en cours — au changement de document, par exemple, où les
    /// identifiants retenus ici ne désignent plus rien.
    pub fn reset(&mut self) {
        *self = Self::default();
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
    Assign {
        domain_id: String,
        weight: f64,
    },
    Unassign(String),
    /// Un domaine d'un autre fichier, posé sur la sélection : il entre au catalogue de ce document.
    Adopter {
        lignee: String,
        weight: f64,
    },
    /// « Même sujet » : ce domaine rejoint son homonyme d'un autre fichier.
    MemeSujet(String),
    /// « Autre sujet » : préciser le mot.
    Preciser(String),
    /// Où vit ce domaine, tous fichiers confondus ?
    Ou(String),
    FermerOu,
    /// Aller à la place de ce rang dans la liste « où ? ».
    Aller(usize),
    /// **Regarder ce domaine**, ou ne plus le regarder s'il l'est déjà (DOMAINE-1).
    Regarder(String),
}

// ── Mise en page ────────────────────────────────────────────────────────────

/// **Ce qu'une rangée montre** : un domaine de ce document, par son identifiant, ou un domaine connu
/// d'ailleurs, par sa lignée.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Cible {
    Ici(String),
    Ailleurs(String),
}

/// Les rectangles d'une ligne de domaine. `index` désigne `store.project.domains[index]`, puis, au-delà,
/// `connus.ailleurs` (DOM-UI-4).
#[derive(Debug, Clone, PartialEq)]
pub struct DomainRowLayout {
    pub index: usize,
    pub cible: Cible,
    /// Sa lignée : ce que « où ? » demande.
    pub lignee: String,
    pub swatch: WidgetRect,
    pub sigil: WidgetRect,
    pub name: WidgetRect,
    /// « où ? » : les places où il vit.
    pub ou: WidgetRect,
    /// « voir » : le regarder — ce qui ne lui appartient pas pâlit (DOMAINE-1).
    pub voir: WidgetRect,
    pub delete: WidgetRect,
    /// Les cinq paliers, du plus faible au plus fort.
    pub steps: Vec<WidgetRect>,
    pub unassign: WidgetRect,
    /// `Some((oui, non))` quand cette ligne attend une confirmation de suppression : les deux
    /// boutons prennent alors la place de la ligne des paliers.
    pub confirm: Option<(WidgetRect, WidgetRect)>,
    /// `Some((oui, non))` quand ce nom désigne aussi un autre sujet ailleurs : même sujet, ou mot à
    /// préciser. Tant qu'on n'a pas répondu, aucun palier (DOM-UI-4).
    pub question: Option<(WidgetRect, WidgetRect)>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DomainsPanelLayout {
    pub rows: Vec<DomainRowLayout>,
    /// « + Nouveau domaine », ou « Retour » quand la liste « où ? » est ouverte.
    pub add_button: WidgetRect,
    /// Rangées qui ne tiennent pas dans la hauteur du panneau : la molette les amène.
    pub hidden: usize,
    /// Les rangées de la liste « où ? », quand elle est ouverte : `.0` est le rang de la première.
    pub ou: Option<(usize, Vec<WidgetRect>)>,
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
    if let Some(ou) = &ui.ou {
        let pitch = m.swatch + m.row_gap;
        let room = ((rows_bottom - (py + m.rows_top)) / pitch).floor().max(0.0) as usize;
        let premier = ui.decalage.min(ou.lignes.len().saturating_sub(room));
        let montres = (ou.lignes.len() - premier).min(room);
        let lignes = (0..montres)
            .map(|k| {
                let top = py + m.rows_top + k as f32 * pitch;
                WidgetRect::new(px + m.pad, top, pw - 2.0 * m.pad, m.swatch)
            })
            .collect();
        return DomainsPanelLayout {
            rows: Vec::new(),
            add_button,
            hidden: ou.lignes.len() - montres,
            ou: Some((premier, lignes)),
        };
    }
    let pitch = m.row_height + m.row_gap;
    let room = ((rows_bottom - (py + m.rows_top)) / pitch).floor().max(0.0) as usize;
    let domaines = store.project.domains.len();
    let total = domaines + ui.connus.ailleurs.len();
    let premier = ui.decalage.min(total.saturating_sub(room));
    let shown = (total - premier).min(room);

    let mut rows = Vec::with_capacity(shown);
    for k in 0..shown {
        let index = premier + k;
        let top = py + m.rows_top + k as f32 * pitch;
        let (cible, lignee, pending, question) = match store.project.domains.get(index) {
            Some(d) => (
                Cible::Ici(d.id.clone()),
                lignee_de(d),
                ui.pending_delete.as_deref() == Some(d.id.as_str()),
                ui.connus.homonymes.contains(&d.id),
            ),
            None => {
                let a = &ui.connus.ailleurs[index - domaines];
                (
                    Cible::Ailleurs(a.lignee.clone()),
                    a.lignee.clone(),
                    false,
                    false,
                )
            }
        };
        let mut row = layout_row(index, (px, top, pw), &m, pending);
        (row.cible, row.lignee) = (cible, lignee);
        // La question prend la place de la confirmation, sauf si une suppression attend déjà.
        row.question = (question && !pending).then(|| deux_reponses(&row, &m));
        rows.push(row);
    }

    DomainsPanelLayout {
        rows,
        add_button,
        hidden: total - shown,
        ou: None,
    }
}

/// Les deux boutons d'une réponse, à droite de la seconde ligne : la question reste lisible à gauche,
/// et les deux réponses ne se confondent pas avec un palier.
fn deux_reponses(row: &DomainRowLayout, m: &Metrics) -> (WidgetRect, WidgetRect) {
    let right = row.delete.x + row.delete.w;
    let second = row.unassign.y;
    let step = row.steps.first().map_or(0.0, |r| r.w);
    let half = (m.unassign_width * 2.0).max(step);
    let yes = WidgetRect::new(
        right - 2.0 * half - m.segment_gap,
        second,
        half,
        m.segment_height,
    );
    let no = WidgetRect::new(right - half, second, half, m.segment_height);
    (yes, no)
}

/// Les deux lignes d'un domaine : identité en haut, pondérations en bas.
fn layout_row(index: usize, at: (f32, f32, f32), m: &Metrics, pending: bool) -> DomainRowLayout {
    let (px, top, pw) = at;
    let right = px + pw - m.pad;
    let swatch = WidgetRect::new(px + m.pad, top, m.swatch, m.swatch);
    let sigil = WidgetRect::new(
        swatch.x + swatch.w + m.pad / 3.0,
        top,
        m.sigil_width,
        m.swatch,
    );
    let delete = WidgetRect::new(right - m.icon_button, top, m.icon_button, m.icon_button);
    let ou = WidgetRect::new(
        delete.x - m.pad / 2.0 - m.ou_width,
        top,
        m.ou_width,
        m.icon_button,
    );
    let voir = WidgetRect::new(
        ou.x - m.pad / 2.0 - m.voir_width,
        top,
        m.voir_width,
        m.icon_button,
    );
    // Le nom part de la fin du sigle plutôt que d'une abscisse recopiée : deux constantes de
    // mise en page qui doivent s'accorder finissent toujours par diverger (§ 5.2).
    let name_left = sigil.x + sigil.w + m.pad / 3.0;
    let name = WidgetRect::new(
        name_left,
        top,
        (voir.x - name_left - m.pad / 2.0).max(0.0),
        m.swatch,
    );

    let second = top + m.line_gap;
    let unassign = WidgetRect::new(
        right - m.unassign_width,
        second,
        m.unassign_width,
        m.segment_height,
    );
    let steps_width = unassign.x - m.segment_gap - (px + m.pad);
    let step_width = ((steps_width - (WEIGHT_STEPS.len() - 1) as f32 * m.segment_gap)
        / WEIGHT_STEPS.len() as f32)
        .max(0.0);
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

    let mut row = DomainRowLayout {
        index,
        cible: Cible::Ici(String::new()),
        lignee: String::new(),
        swatch,
        sigil,
        name,
        ou,
        voir,
        delete,
        steps,
        unassign,
        confirm: None,
        question: None,
    };
    // La confirmation occupe la moitié droite de la seconde ligne.
    row.confirm = pending.then(|| deux_reponses(&row, m));
    row
}

// ── Les palettes d'édition ──────────────────────────────────────────────────

/// La couleur qui suit `current` dans la palette du thème.
///
/// Un vrai sélecteur de couleur est un panneau à part entière ; ce bouton-ci fait ce qu'il
/// annonce et le fait entièrement : il parcourt les huit teintes du thème, choisies pour
/// rester distinctes sur le fond du canevas. Une couleur absente de la palette (document venu
/// d'ailleurs) repart de la première.
pub fn next_color(current: &str) -> &'static str {
    let at = DOMAIN_PALETTE
        .iter()
        .position(|c| c.eq_ignore_ascii_case(current));
    DOMAIN_PALETTE[at.map_or(0, |i| (i + 1) % DOMAIN_PALETTE.len())]
}

/// Le sigle qui suit `current` dans la palette du thème.
pub fn next_sigil(current: &str) -> &'static str {
    let at = DOMAIN_SIGILS
        .iter()
        .position(|c| c.eq_ignore_ascii_case(current));
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

mod clic;
pub mod paint;

pub use clic::hit_domains_panel;
pub use paint::render_domains_panel;

#[cfg(test)]
mod tests;
