//! Le catalogue de domaines et les assignations qui le relient aux nœuds.
//!
//! Un **domaine** est une catégorie sémantique (Science, Art, Jeu vidéo, Conlang…). Un nœud —
//! annotation ou image — en porte zéro, un ou plusieurs, chacun avec une **pondération**.
//!
//! # DOM-1 — un seul endroit énumère les porteurs d'assignations
//!
//! [`for_each_assignment_list`] est le **seul** point du noyau qui sait où vivent les
//! `Vec<DomainAssignment>` : les annotations et les images de tous les tableaux. La cascade de
//! suppression, l'élagage des références orphelines et le comptage en dérivent tous. Ajouter un
//! porteur (un dossier, un panneau de storyboard) se fait donc ici **et nulle part ailleurs** :
//! c'est ce qui rend impossible le retour de R-47, où `remove_domain` retirait le domaine du
//! catalogue sans toucher aux nœuds et laissait derrière lui des références vers un domaine
//! disparu — références que la persistance écrivait ensuite sur disque.
//!
//! # DOM-2 — les assignations d'un nœud suivent l'ordre du catalogue
//!
//! Les assignations sont rangées dans l'ordre où leurs domaines apparaissent dans
//! `project.domains`. Ce n'est pas une coquetterie : le rendu dessine une colonne par
//! assignation, et sans cet ordre la même paire de domaines produirait deux réglettes
//! différentes sur deux nœuds voisins. Un domaine occupe la même colonne partout.
//!
//! # DOM-3 — aucune entrée d'undo pour une opération qui n'a rien changé
//!
//! [`super::Store::push_undo`] n'est appelé qu'**après** la validation complète, et jamais
//! quand la mutation se révèle sans effet. Un `try_remove_domain` sur un identifiant inconnu
//! empilait auparavant une entrée d'annulation pour une opération qui n'avait rien fait :
//! l'utilisateur devait appuyer deux fois sur `Ctrl+Z` pour défaire un seul geste.

use super::journal::{Edit, Slot};
use super::Store;
use crate::error::{CoreError, CoreResult};
use crate::types::{Board, Domain, DomainAssignment, Project};

/// Bornes admises pour une pondération, incluses.
const WEIGHT_MIN: f64 = 0.0;
const WEIGHT_MAX: f64 = 1.0;

/// Ce qu'une mise à jour de domaine change. Un champ à `None` reste tel quel.
///
/// Le patch existe parce que `update_domain` ne savait changer que le nom : `color` et `icon`
/// étaient dans le modèle, sérialisés, dessinés — et inéditables (violation de la loi L9, qui
/// veut qu'un champ arrive avec son rendu, **son édition** et sa sérialisation).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DomainPatch {
    pub name: Option<String>,
    pub color: Option<String>,
    pub icon: Option<String>,
}

impl DomainPatch {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn with_color(mut self, color: impl Into<String>) -> Self {
        self.color = Some(color.into());
        self
    }

    pub fn with_icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Le patch changerait-il quoi que ce soit à ce domaine ? (DOM-3)
    fn changes(&self, domain: &Domain) -> bool {
        let differs = |field: &Option<String>, current: &String| {
            field.as_ref().is_some_and(|wanted| wanted != current)
        };
        differs(&self.name, &domain.name)
            || differs(&self.color, &domain.color)
            || differs(&self.icon, &domain.icon)
    }

    /// Écrit les champs présents sur le domaine.
    fn apply(self, domain: &mut Domain) {
        if let Some(name) = self.name {
            domain.name = name;
        }
        if let Some(color) = self.color {
            domain.color = color;
        }
        if let Some(icon) = self.icon {
            domain.icon = icon;
        }
    }
}

/// Applique `visit` au vecteur d'assignations de **chaque nœud du projet** (DOM-1).
fn for_each_assignment_list(
    project: &mut Project,
    mut visit: impl FnMut(&mut Vec<DomainAssignment>),
) {
    for board in &mut project.boards {
        for ann in &mut board.annotations {
            visit(ann.domains_mut());
        }
        for img in &mut board.images {
            visit(&mut img.domains);
        }
    }
}

/// Les assignations de chaque nœud du projet, en lecture seule (DOM-1).
fn each_assignment_list(project: &Project) -> impl Iterator<Item = &[DomainAssignment]> {
    project.boards.iter().flat_map(|board| {
        board
            .annotations
            .iter()
            .map(|ann| ann.domains())
            .chain(board.images.iter().map(|img| img.domains.as_slice()))
    })
}

/// Une pondération est-elle acceptable ? `NaN` et `±∞` ne le sont pas : écrits dans le modèle,
/// ils partent tels quels dans le fichier, et rien en aval ne sait plus quoi en faire.
fn check_weight(weight: f64) -> CoreResult<()> {
    if weight.is_finite() && (WEIGHT_MIN..=WEIGHT_MAX).contains(&weight) {
        Ok(())
    } else {
        Err(CoreError::InvalidWeight(weight))
    }
}

impl Store {
    // ── Lecture ─────────────────────────────────────────────────────────────

    /// Le domaine de cet identifiant, s'il est au catalogue.
    pub fn domain(&self, id: &str) -> Option<&Domain> {
        self.project.domains.iter().find(|d| d.id == id)
    }

    /// Rang d'un domaine dans le catalogue — la colonne qu'il occupe partout (DOM-2).
    pub fn domain_rank(&self, id: &str) -> Option<usize> {
        self.project.domains.iter().position(|d| d.id == id)
    }

    /// Les assignations d'un nœud, annotation ou image, du tableau `board_id`.
    pub fn node_domains(&self, board_id: &str, node_id: &str) -> CoreResult<&[DomainAssignment]> {
        let board = self
            .project
            .boards
            .iter()
            .find(|b| b.id == board_id)
            .ok_or_else(|| CoreError::BoardNotFound(board_id.to_string()))?;
        if let Some(ann) = board.annotations.iter().find(|a| a.id() == node_id) {
            return Ok(ann.domains());
        }
        if let Some(img) = board.images.iter().find(|i| i.id == node_id) {
            return Ok(&img.domains);
        }
        Err(CoreError::NodeNotFound(node_id.to_string()))
    }

    /// Nombre de nœuds du projet qui portent ce domaine.
    ///
    /// O(nœuds) : réservé aux diagnostics, aux tests et aux rapports. **Jamais** dans une
    /// frame — le coût d'une frame ne dépend pas de la taille du document (loi L2).
    pub fn domain_assignment_count(&self, domain_id: &str) -> usize {
        each_assignment_list(&self.project)
            .filter(|list| list.iter().any(|a| a.domain_id == domain_id))
            .count()
    }

    /// Identifiants de domaine référencés par un nœud sans exister au catalogue.
    ///
    /// Rend toujours une liste vide sur un document sain : c'est l'assertion de DOM-1, offerte
    /// aux tests et au diagnostic d'un fichier venu d'une version antérieure.
    pub fn orphan_domain_references(&self) -> Vec<String> {
        let mut orphans = Vec::new();
        for list in each_assignment_list(&self.project) {
            for assignment in list {
                let known = self
                    .project
                    .domains
                    .iter()
                    .any(|d| d.id == assignment.domain_id);
                if !known && !orphans.contains(&assignment.domain_id) {
                    orphans.push(assignment.domain_id.clone());
                }
            }
        }
        orphans
    }

    // ── Écriture ────────────────────────────────────────────────────────────

    /// Ajoute un domaine au catalogue. Un identifiant déjà pris est refusé (DOM-1).
    pub fn try_add_domain(&mut self, domain: Domain) -> CoreResult<()> {
        if self.project.domains.iter().any(|d| d.id == domain.id) {
            return Err(CoreError::DuplicateDomainId(domain.id));
        }
        let index = self.project.domains.len();
        self.project.domains.push(domain.clone());
        self.record_edit(Edit::Domain { slot: Slot::inserted(index, domain) });
        Ok(())
    }

    /// Change le nom, la couleur ou l'icône d'un domaine.
    ///
    /// Un patch sans effet ne laisse aucune entrée d'annulation derrière lui (DOM-3).
    pub fn try_update_domain(&mut self, id: &str, patch: DomainPatch) -> CoreResult<()> {
        let domain = self
            .project
            .domains
            .iter()
            .find(|d| d.id == id)
            .ok_or_else(|| CoreError::DomainNotFound(id.to_string()))?;
        if !patch.changes(domain) {
            return Ok(());
        }
        let Some(i) = self.project.domains.iter().position(|d| d.id == id) else {
            return Ok(());
        };
        let before = self.project.domains[i].clone();
        patch.apply(&mut self.project.domains[i]);
        let after = self.project.domains[i].clone();
        self.record_edit(Edit::Domain { slot: Slot::changed(i, before, after) });
        Ok(())
    }

    /// Retire un domaine du catalogue **et toutes ses assignations** (DOM-1).
    ///
    /// Rend le nombre de nœuds dont l'assignation a été retirée. Après cet appel,
    /// [`Store::orphan_domain_references`] ne peut pas mentionner `id` : c'est la seule
    /// définition acceptable de « supprimer un domaine ». Un `undo` restaure le catalogue
    /// **et** les assignations, puisque l'instantané est pris avant la cascade.
    pub fn try_remove_domain(&mut self, id: &str) -> CoreResult<usize> {
        if !self.project.domains.iter().any(|d| d.id == id) {
            return Err(CoreError::DomainNotFound(id.to_string()));
        }
        // Le domaine et toutes ses assignations partent ensemble : un seul geste, donc un
        // seul Ctrl+Z. Seul un noeud qui portait vraiment ce domaine est clone.
        let Some(di) = self.project.domains.iter().position(|d| d.id == id) else {
            return Err(CoreError::DomainNotFound(id.to_string()));
        };
        let removed = self.project.domains.remove(di);
        let mut edits = vec![Edit::Domain { slot: Slot::removed(di, removed) }];
        let mut detached = 0usize;

        // Meme ordre de visite que `for_each_assignment_list` : annotations, puis images.
        for board in &mut self.project.boards {
            let bid = board.id.clone();
            for (i, ann) in board.annotations.iter_mut().enumerate() {
                if !ann.domains_mut().iter().any(|a| a.domain_id == id) {
                    continue;
                }
                let before = ann.clone();
                ann.domains_mut().retain(|a| a.domain_id != id);
                edits.push(Edit::Annotation {
                    board: bid.clone(),
                    slot: Slot::changed(i, before, ann.clone()),
                });
                detached += 1;
            }
            for (i, img) in board.images.iter_mut().enumerate() {
                if !img.domains.iter().any(|a| a.domain_id == id) {
                    continue;
                }
                let before = img.clone();
                img.domains.retain(|a| a.domain_id != id);
                edits.push(Edit::Image {
                    board: bid.clone(),
                    slot: Slot::changed(i, before, img.clone()),
                });
                detached += 1;
            }
        }

        self.record_as_one_gesture(edits);
        Ok(detached)
    }

    /// Affecte un domaine à un nœud — annotation **ou** image — avec sa pondération.
    ///
    /// Réassigner un domaine déjà porté écrase son poids ; l'assignation garde sa place, qui
    /// est celle du domaine au catalogue (DOM-2).
    pub fn try_assign_domain_to_node(
        &mut self,
        board_id: &str,
        node_id: &str,
        domain_id: &str,
        weight: f64,
    ) -> CoreResult<()> {
        check_weight(weight)?;
        let ranks = self.domain_ranks();
        let rank = ranks
            .iter()
            .position(|known| known == domain_id)
            .ok_or_else(|| CoreError::DomainNotFound(domain_id.to_string()))?;
        let at = self
            .project
            .boards
            .iter()
            .position(|b| b.id == board_id)
            .ok_or_else(|| CoreError::BoardNotFound(board_id.to_string()))?;
        if !node_exists(&self.project.boards[at], node_id) {
            return Err(CoreError::NodeNotFound(node_id.to_string()));
        }

        // La validation est terminée : à partir d'ici l'opération aboutit (DOM-3, § 3.3).
        let assignment = DomainAssignment {
            domain_id: domain_id.to_string(),
            weight,
        };
        let board = &mut self.project.boards[at];
        let bid = board.id.clone();
        let Some(place) = locate_node(board, node_id) else {
            return Ok(());
        };
        let edit = match place {
            NodeSlot::Annotation(i) => {
                let before = board.annotations[i].clone();
                insert_ranked(board.annotations[i].domains_mut(), assignment, rank, &ranks);
                Edit::Annotation {
                    board: bid,
                    slot: Slot::changed(i, before, board.annotations[i].clone()),
                }
            }
            NodeSlot::Image(i) => {
                let before = board.images[i].clone();
                insert_ranked(&mut board.images[i].domains, assignment, rank, &ranks);
                Edit::Image {
                    board: bid,
                    slot: Slot::changed(i, before, board.images[i].clone()),
                }
            }
        };
        self.record_edit(edit);
        Ok(())
    }

    /// Retire l'assignation d'un domaine sur un nœud.
    ///
    /// Un nœud qui ne porte pas ce domaine n'a rien à perdre : l'appel échoue plutôt que de
    /// laisser une entrée d'annulation pour un geste sans effet (DOM-3).
    pub fn try_unassign_domain_from_node(
        &mut self,
        board_id: &str,
        node_id: &str,
        domain_id: &str,
    ) -> CoreResult<()> {
        let carried = self
            .node_domains(board_id, node_id)?
            .iter()
            .any(|a| a.domain_id == domain_id);
        if !carried {
            return Err(CoreError::DomainNotAssigned {
                node_id: node_id.to_string(),
                domain_id: domain_id.to_string(),
            });
        }
        let Some(board) = self.project.boards.iter_mut().find(|b| b.id == board_id) else {
            return Ok(());
        };
        let bid = board.id.clone();
        let Some(place) = locate_node(board, node_id) else {
            return Ok(());
        };
        let edit = match place {
            NodeSlot::Annotation(i) => {
                let before = board.annotations[i].clone();
                board.annotations[i]
                    .domains_mut()
                    .retain(|a| a.domain_id != domain_id);
                Edit::Annotation {
                    board: bid,
                    slot: Slot::changed(i, before, board.annotations[i].clone()),
                }
            }
            NodeSlot::Image(i) => {
                let before = board.images[i].clone();
                board.images[i].domains.retain(|a| a.domain_id != domain_id);
                Edit::Image {
                    board: bid,
                    slot: Slot::changed(i, before, board.images[i].clone()),
                }
            }
        };
        self.record_edit(edit);
        Ok(())
    }

    /// Retire de chaque nœud les assignations qu'aucune écriture ne pourrait produire
    /// aujourd'hui : celles qui pointent vers un domaine absent du catalogue (DOM-1), et
    /// celles dont la pondération est hors de `0.0..=1.0`, infinie ou `NaN`.
    ///
    /// Rend le nombre de nœuds réparés. Appelé par [`Store::load_project`] : un fichier écrit
    /// par une version antérieure à DOM-1 peut porter les unes comme les autres, et les garder
    /// reviendrait à les réécrire au prochain enregistrement — et, pour un `NaN`, à donner au
    /// rendu une longueur qui n'en est pas une. La réparation n'est pas silencieuse : le compte
    /// remonte à l'appelant, à qui il revient de le dire (standard § 6.4).
    pub fn repair_domain_assignments(&mut self) -> usize {
        let known: Vec<String> = self.project.domains.iter().map(|d| d.id.clone()).collect();
        let mut repaired = 0usize;
        for_each_assignment_list(&mut self.project, |list| {
            let before = list.len();
            list.retain(|a| known.contains(&a.domain_id) && check_weight(a.weight).is_ok());
            if list.len() != before {
                repaired += 1;
            }
        });
        repaired
    }

    /// Rang de chaque domaine du catalogue, dans l'ordre du catalogue (DOM-2).
    fn domain_ranks(&self) -> Vec<String> {
        self.project.domains.iter().map(|d| d.id.clone()).collect()
    }
}

/// Ce tableau porte-t-il un nœud de cet identifiant, annotation ou image ?
/// Ou vit un noeud dans un board : le genre de liste, et son rang.
///
/// Journaliser une assignation demande de savoir non seulement *quelle* liste de
/// pondérations modifier, mais *quelle case* de *quelle liste de noeuds* elle appartient --
/// ce que `node_assignments_mut` ne dit pas, puisqu'il ne rend que la liste.
enum NodeSlot {
    Image(usize),
    Annotation(usize),
}

/// Meme ordre de recherche que `node_assignments_mut` : annotations d'abord.
fn locate_node(board: &Board, node_id: &str) -> Option<NodeSlot> {
    if let Some(i) = board.annotations.iter().position(|a| a.id() == node_id) {
        return Some(NodeSlot::Annotation(i));
    }
    board
        .images
        .iter()
        .position(|img| img.id == node_id)
        .map(NodeSlot::Image)
}

fn node_exists(board: &Board, node_id: &str) -> bool {
    board.annotations.iter().any(|a| a.id() == node_id)
        || board.images.iter().any(|i| i.id == node_id)
}

/// Insère (ou remplace) une assignation à la place que lui donne l'ordre du catalogue (DOM-2).
fn insert_ranked(
    list: &mut Vec<DomainAssignment>,
    assignment: DomainAssignment,
    rank: usize,
    ranks: &[String],
) {
    if let Some(existing) = list
        .iter_mut()
        .find(|a| a.domain_id == assignment.domain_id)
    {
        existing.weight = assignment.weight;
        return;
    }
    let rank_of = |id: &str| {
        ranks
            .iter()
            .position(|known| known == id)
            .unwrap_or(usize::MAX)
    };
    let at = list
        .iter()
        .position(|a| rank_of(&a.domain_id) > rank)
        .unwrap_or(list.len());
    list.insert(at, assignment);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_a_weight_is_a_finite_number_between_zero_and_one() {
        for good in [0.0, 0.5, 1.0] {
            assert!(check_weight(good).is_ok(), "{good} devrait passer");
        }
        // `NaN` n'est égal à rien, pas même à lui-même : l'erreur se lit par filtrage, et on
        // vérifie que c'est bien la valeur refusée qu'elle transporte.
        for bad in [-0.01, 1.01, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            let Err(CoreError::InvalidWeight(refused)) = check_weight(bad) else {
                panic!("{bad} aurait dû être refusé");
            };
            assert_eq!(refused.is_nan(), bad.is_nan(), "{bad}");
            assert!(
                refused.is_nan() || refused == bad,
                "{bad} refusé comme {refused}"
            );
        }
    }

    #[test]
    fn test_an_empty_patch_changes_nothing() {
        let domain = Domain {
            id: "d".into(),
            name: "Science".into(),
            color: "#60a5fa".into(),
            icon: "SCI".into(),
            created_at: 0,
        };
        assert!(!DomainPatch::new().changes(&domain));
        assert!(!DomainPatch::new().with_name("Science").changes(&domain));
        assert!(DomainPatch::new().with_name("Art").changes(&domain));
        assert!(DomainPatch::new().with_color("#f472b6").changes(&domain));
        assert!(DomainPatch::new().with_icon("ART").changes(&domain));
    }
}
