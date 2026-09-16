//! Lire les rangs qu'un culling vient de rendre (CULL-1).
//!
//! L'index spatial numérote les nœuds ; les passes de rendu veulent des nœuds. La traduction
//! vit ici, et nulle part ailleurs : c'est ce qui fait qu'un décalage oublié ne peut plus
//! dessiner un nœud à la place d'un autre.

/// Les nœuds visibles d'un tableau, rendus **par tranches de présentation** (CULL-1).
///
/// L'index numérote les nœuds d'un tableau d'affilée — ses images, puis ses annotations, puis
/// ses dossiers — et [`super::SpatialHash::query_rect_ranks`] rend ces numéros triés. Une passe de
/// rendu ne veut ni des numéros ni des indices : elle veut **les nœuds**. Ce type est le seul
/// endroit du moteur où un rang se traduit, et il rend directement de quoi dessiner.
///
/// # Pourquoi les nœuds et non des indices
///
/// La première version rendait des indices, et chaque passe écrivait `board.annotations[i]`
/// de son côté. Deux fautes en une : le décalage s'y rejouait à chaque appel — celle des
/// annotations l'avait déjà oublié une fois, et aurait dessiné un nœud à la place d'un autre
/// sans qu'aucun test de géométrie ne s'en aperçoive — et le rendu remettait la main dans les
/// champs du modèle, contre la règle S. En rendant les nœuds, l'indexation ne se fait plus
/// qu'ici, et il n'y a plus d'occasion de se tromper.
///
/// Les bornes se trouvent par deux recherches dichotomiques : lire une tranche ne parcourt
/// rien, et le parcours qui suit est séquentiel, donc favorable au cache. Elles viennent des
/// longueurs du tableau, si bien qu'un rang resté d'un tableau plus grand est ignoré au lieu
/// de devenir un accès hors bornes.
#[derive(Debug, Clone, Copy)]
pub struct Visibles<'a> {
    rangs: &'a [u32],
    board: &'a crate::types::Board,
}

impl<'a> Visibles<'a> {
    /// Interprète des rangs bruts à la lumière du tableau qui les a présentés.
    ///
    /// Le tableau doit être **celui-là même** que le dernier [`super::SpatialHash::index_board`] a
    /// parcouru : ce sont ses longueurs qui situent les frontières entre les tranches.
    pub fn nouvelles(rangs: &'a [u32], board: &'a crate::types::Board) -> Self {
        Self { rangs, board }
    }

    /// Les images visibles, dans l'ordre du tableau.
    pub fn images(self) -> impl Iterator<Item = &'a crate::types::BoardImage> {
        let board = self.board;
        self.tranche(0, board.images.len())
            .map(move |i| &board.images[i])
    }

    /// Les annotations visibles, dans l'ordre du tableau.
    pub fn annotations(self) -> impl Iterator<Item = &'a crate::types::Annotation> {
        let board = self.board;
        let debut = board.images.len();
        self.tranche(debut, debut + board.annotations.len())
            .map(move |i| &board.annotations[i])
    }

    /// Les dossiers visibles, dans l'ordre du tableau.
    pub fn dossiers(self) -> impl Iterator<Item = &'a crate::types::CanvasFolder> {
        let board = self.board;
        let debut = board.images.len() + board.annotations.len();
        self.tranche(debut, debut + board.folders.len())
            .map(move |i| &board.folders[i])
    }

    /// La tranche `[debut, fin)` des rangs, ramenée à des indices de la liste concernée.
    fn tranche(self, debut: usize, fin: usize) -> impl Iterator<Item = usize> + 'a {
        let (debut, fin) = (debut as u32, fin as u32);
        let d = self.rangs.partition_point(|&r| r < debut);
        let f = self.rangs.partition_point(|&r| r < fin);
        self.rangs[d..f].iter().map(move |&r| (r - debut) as usize)
    }
}

/// Les rangs de **tous** les nœuds d'un tableau, dans l'ordre de présentation.
///
/// C'est le culling qui ne retient rien. Les preuves de rendu et les bancs en ont besoin :
/// ils mesurent une passe entière, et ne veulent pas que la fenêtre décide à leur place ce
/// qui compte. Associé à [`Visibles::nouvelles`], il rend un ensemble de tranches pleines.
pub fn tous_les_rangs(board: &crate::types::Board) -> Vec<u32> {
    let total = board.images.len() + board.annotations.len() + board.folders.len();
    (0..total as u32).collect()
}
