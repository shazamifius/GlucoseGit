//! Génération d'identifiants et chargement de projet.
//!
//! # R-22 — `generate_id` était en O(n) par identifiant
//!
//! L'implémentation d'origine appelait `id_exists` (scan de tous les boards × toutes les
//! images × toutes les annotations × tous les dossiers) pour chaque identifiant produit :
//! coller 100 éléments coûtait O(n²). Comme `next_id` est monotone, ce scan ne servait
//! jamais — sauf après le chargement d'un projet venu du disque, dont les identifiants n'ont
//! pas été produits par ce compteur.
//!
//! La collision est donc réglée **une fois au chargement** (INVARIANT ID-1, cf.
//! [`super::Store::next_id`]) et la création redevient O(1).

use super::{build_folder_stack, Store};
use crate::types::Project;

/// Extrait le suffixe numérique d'un identifiant produit par `generate_id`.
///
/// `"img-42"` → `Some(42)`, `"mirror-folder-7"` → `Some(7)`, `"main"` → `None`.
/// Un identifiant entièrement numérique (`"12"`) compte aussi : il entrerait en collision
/// avec le préfixe vide.
fn numeric_suffix(id: &str) -> Option<u64> {
    let tail = match id.rsplit_once('-') {
        Some((_, tail)) => tail,
        None => id,
    };
    tail.parse::<u64>().ok()
}

/// Plus grand suffixe numérique présent dans le projet.
fn max_numeric_suffix(project: &Project) -> u64 {
    let mut max = 0;
    let mut consider = |id: &str| {
        if let Some(n) = numeric_suffix(id) {
            max = max.max(n);
        }
    };
    for b in &project.boards {
        consider(&b.id);
        for i in &b.images {
            consider(&i.id);
        }
        for a in &b.annotations {
            consider(a.id());
        }
        for f in &b.folders {
            consider(&f.id);
            consider(&f.child_board_id);
        }
        for p in &b.panels {
            consider(&p.id);
        }
    }
    for p in &project.presets {
        consider(&p.id);
    }
    for d in &project.domains {
        consider(&d.id);
    }
    max
}

impl Store {
    /// Scan linéaire de collision. Conservé pour les diagnostics et les tests : il n'est plus
    /// sur le chemin chaud de `generate_id` (R-22).
    pub fn id_exists(&self, id: &str) -> bool {
        self.project.boards.iter().any(|b| {
            b.id == id
                || b.images.iter().any(|i| i.id == id)
                || b.annotations.iter().any(|a| a.id() == id)
                || b.folders.iter().any(|f| f.id == id || f.child_board_id == id)
        })
    }

    /// Produit un identifiant unique en O(1) (R-22).
    ///
    /// L'unicité repose sur l'INVARIANT ID-1 : `next_id` domine tous les suffixes déjà
    /// présents. Il est rétabli par [`Store::resync_next_id`] à chaque injection de projet.
    pub fn generate_id(&mut self, prefix: &str) -> String {
        self.next_id += 1;
        format!("{}-{}", prefix, self.next_id)
    }

    /// Rétablit l'INVARIANT ID-1 après une injection directe de `project`.
    ///
    /// C'est le **seul** endroit où le projet est scanné pour éviter les collisions
    /// d'identifiants. À appeler après toute écriture de `store.project` qui ne passe pas par
    /// [`Store::load_project`].
    pub fn resync_next_id(&mut self) {
        self.next_id = self.next_id.max(max_numeric_suffix(&self.project));
    }

    /// Adopte un projet venu de l'extérieur (disque, import) comme document courant.
    ///
    /// Rend le nombre de nœuds dont une assignation de domaine invalide a été retirée — voir
    /// [`Store::repair_domain_assignments`]. Ce n'est pas une réparation silencieuse : le
    /// compte remonte à l'appelant, à qui il revient de le dire à l'utilisateur (standard
    /// § 6.4). Il vaut zéro sur tout document écrit par une version qui fait cascader
    /// `try_remove_domain` — c'est-à-dire toutes celles qui suivent R-47 — et n'est non nul
    /// que pour un fichier antérieur, dont les références vers des domaines disparus seraient
    /// sinon réécrites au prochain `Ctrl+S`.
    pub fn load_project(&mut self, project: Project) -> usize {
        self.project = project;
        self.resync_next_id();
        let repaired = self.repair_domain_assignments();
        // JRN-2 : le document vient d'être remplacé en dehors du journal ; ses index ne
        // décrivent plus rien. On vide plutôt que de garder une pile qui ment.
        self.journal.clear();
        self.clear_selection();
        self.folder_stack = build_folder_stack(&self.project.boards, &self.project.active_board_id);
        repaired
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_numeric_suffix() {
        assert_eq!(numeric_suffix("img-42"), Some(42));
        assert_eq!(numeric_suffix("mirror-folder-7"), Some(7));
        assert_eq!(numeric_suffix("12"), Some(12));
        assert_eq!(numeric_suffix("main"), None);
        assert_eq!(numeric_suffix("img-"), None);
        assert_eq!(numeric_suffix("board-abc"), None);
    }

    #[test]
    fn test_generate_id_is_monotonic_and_unique() {
        let mut store = Store::new("P");
        let a = store.generate_id("img");
        let b = store.generate_id("img");
        assert_ne!(a, b);
        assert!(!store.id_exists(&a));
    }
}
