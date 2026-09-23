//! **Le geste** : ce que l'utilisateur est en train de faire quand une image se dessine.
//!
//! Sorti de [`super`] quand celui-ci a passé les six cents lignes que la fiche 05 admet. La
//! coupure tombe là où la raison de changer diffère : la chronique **agrège** des mesures, et
//! ceci **nomme** ce que la main fait — un geste de plus ne change rien à la façon d'agréger.

/// Ce que l'utilisateur est en train de faire quand l'image se dessine.
///
/// Déduit de l'état de l'application, jamais déclaré : un geste qui devrait penser à
/// s'annoncer finirait par oublier, et c'est précisément le genre d'oubli qui a coûté une
/// journée de recherche cette semaine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Geste {
    /// Rien en cours : l'image vient d'une animation, d'un survol ou du système.
    Repos,
    /// La vue se déplace — sous la main, ou sur son élan une fois lâchée.
    DeplacerLaVue,
    /// La vue change d'échelle.
    Zoomer,
    /// Un ou plusieurs nœuds suivent la main.
    GlisserUnNoeud,
    /// Un nœud change de taille par une poignée.
    Redimensionner,
    /// Un objet naît sous la main.
    Dessiner,
    /// Le rectangle de sélection élastique s'étire.
    Selectionner,
    /// Du texte s'écrit dans une carte.
    EditerDuTexte,
    /// Des images finissent de se décoder en arrière-plan.
    Decoder,
    /// La caméra vole vers une cible.
    Animer,
}

impl Geste {
    /// Tous les gestes, dans l'ordre de leur indice.
    pub const TOUS: [Geste; 10] = [
        Geste::Repos,
        Geste::DeplacerLaVue,
        Geste::Zoomer,
        Geste::GlisserUnNoeud,
        Geste::Redimensionner,
        Geste::Dessiner,
        Geste::Selectionner,
        Geste::EditerDuTexte,
        Geste::Decoder,
        Geste::Animer,
    ];

    pub(super) fn indice(self) -> usize {
        Self::TOUS.iter().position(|g| *g == self).unwrap_or(0)
    }

    /// Le nom court qui paraît dans le rapport.
    pub fn nom(self) -> &'static str {
        match self {
            Geste::Repos => "repos",
            Geste::DeplacerLaVue => "deplacer la vue",
            Geste::Zoomer => "zoomer",
            Geste::GlisserUnNoeud => "glisser un noeud",
            Geste::Redimensionner => "redimensionner",
            Geste::Dessiner => "dessiner",
            Geste::Selectionner => "selectionner",
            Geste::EditerDuTexte => "editer du texte",
            Geste::Decoder => "decoder des images",
            Geste::Animer => "animer la camera",
        }
    }
}
