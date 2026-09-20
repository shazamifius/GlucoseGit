//! L'image d'une image : ce qu'elle a coute, ce qu'elle a montre, et pourquoi elle existe.
//!
//! Sortie de [`super`] quand celui-ci a depasse sa taille admise. Le decoupage n'est pas
//! arbitraire : une chronique **agrege**, un instantane **decrit**, et les deux ne changent
//! jamais pour les memes raisons.

use super::{Geste, POSTES};

/// Ce qu'une image a coûté, et dans quel contexte.
///
/// **Que des entiers.** Aucun texte, aucun chemin, aucun contenu : c'est ce qui rend la
/// promesse de confidentialité vérifiable par le type plutôt que par la relecture.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Instantane {
    /// Depuis le début de la session.
    pub instant_ms: u32,
    /// Ce que l'image entière a coûté.
    pub duree_us: u32,
    /// L'indice du geste, au sens de [`Geste::TOUS`].
    pub geste: u8,
    /// La durée de chaque poste, dans l'ordre où ils se sont déclarés.
    pub postes_us: PostesUs,
    /// Combien de nœuds le culling a retenus.
    pub noeuds: u32,
    /// Combien d'images ont été posées.
    pub photos: u32,
    /// L'aire redessinée, en pixels — le cœur d'A.1.
    pub region_px: u32,
    /// Combien de fois la surface de la fenêtre les images posees couvrent, en centiemes.
    ///
    /// **Le chiffre qui distingue une image chere d'une image repetee.** Soixante-dix photos
    /// qui se recouvrent repeignent soixante-dix fois le meme ecran ; une duree seule ne le
    /// separe pas de soixante-dix photos couteuses, et les deux ne se corrigent pas pareil.
    pub surcouverture: u32,
    /// L'aire de la fenêtre, pour que la précédente soit lisible en proportion.
    pub fenetre_px: u32,
    /// Ce que les images décodées occupent, en mébioctets.
    pub images_mo: u32,
    /// Combien d'images attendent encore leur décodage.
    pub en_decodage: u16,
    /// Combien de vignettes l'atelier a achevées pendant cette image (CASCADE-1).
    pub vignettes: u16,
    /// Combien de vignettes attendent encore leur tour au chantier.
    ///
    /// **C'est le chiffre qui dit si l'image est chère à bon droit.** Une photo sans vignette
    /// se dessine par le chemin général, quinze fois plus cher ; tant que ce nombre n'est pas
    /// nul, la scène est en régime transitoire. S'il ne descend jamais, c'est que le chantier
    /// se refait aussi vite qu'il se vide — et aucune durée ne le dirait.
    pub vignettes_en_attente: u16,
    /// Combien de photos ont pu se poser depuis une **vignette prête**.
    ///
    /// À comparer à [`Self::photos`] : les deux chemins diffèrent d'un facteur dix, donc une
    /// image chère ne dit pas d'elle-même si elle dessine beaucoup ou si elle dessine **mal**.
    /// Zéro sur une image de quatre-vingts photos veut dire qu'aucune n'a pu en avoir — et la
    /// seule raison possible est qu'elles débordent de la fenêtre, c'est-à-dire le zoom proche.
    pub par_vignette: u16,
    /// Combien de vignettes existaient pour ce nœud, mais à une **autre forme**.
    ///
    /// Sépare deux causes qu'aucune durée ne distingue : une vignette pas encore construite,
    /// et une vignette construite pour une forme que la vue a déjà quittée.
    pub vignettes_perimees: u16,
    /// Combien de nœuds ont une vignette prête, quelle que soit sa forme.
    pub vignettes_pretes: u16,
    /// Combien de vignettes ont été achevées pour un nœud qui n'existait plus, depuis le début
    /// de la session. Du travail intégralement perdu.
    pub vignettes_orphelines: u16,
    /// Combien de nœuds ont dû être recréés pendant cette image, faute d'entrée.
    pub vignettes_recreees: u16,
    /// Combien de chantiers ont été abandonnés depuis le début, leur forme ayant été quittée.
    pub vignettes_abandonnees: u16,
    /// Ce que le modèle de coût avait **prévu** pour cette image, en microsecondes.
    ///
    /// Zéro quand la machine n'avait pas encore démontré assez pour prévoir. L'écart avec
    /// [`Self::duree_us`] est le résidu : nul, le modèle comprend la machine ; élevé, il
    /// désigne exactement ce qu'il ne compte pas encore.
    pub prevu_us: u32,
    /// Les images de cette scène se sont-elles **pixelisées** pour tenir le budget ?
    pub pixelise: u16,
    /// De combien la scene a ete rendue plus petite que la fenetre. `1` veut dire « pas du
    /// tout » (voir [`crate::resolution`]).
    pub reduction: u16,
    /// Combien de tuiles de la grille ont été **peintes** pendant cette image (TUILE-1).
    ///
    /// C'est le travail réellement fait. Une image immobile n'en peint aucune ; un glissement
    /// n'en peint que la colonne qui entre ; un zoom d'une octave les repeint toutes.
    pub tuiles_peintes: u16,
    /// Combien de tuiles déjà peintes ont servi telles quelles — le travail épargné.
    ///
    /// C'est ce que les vignettes n'ont jamais su montrer : servies à zéro pour cent sur cinq
    /// sessions, sans qu'aucune durée ne le dise.
    pub tuiles_reprises: u16,
    /// Ce que le report des photos a réellement coûté, en microsecondes.
    ///
    /// C'est **ce que le modèle prévoit**, et donc la seule grandeur à laquelle sa prévision
    /// puisse se comparer. La rapporter à la durée entière de l'image la ferait paraître
    /// fausse alors qu'elle ne parle pas de la même chose.
    pub report_us: u32,
    /// Ce que cette image a envoyé à la carte graphique, en mébioctets.
    pub blit_mo: u16,
    /// Combien de temps l'image **précédente** est restée sous les yeux, en microsecondes.
    ///
    /// À ne pas confondre avec [`Self::duree_us`], qui dit ce que **celle-ci** a coûté. Les
    /// deux diffèrent exactement de la variation du coût d'une image à l'autre, et c'est cet
    /// écart — pas le coût — que l'œil reçoit.
    pub intervalle_us: u32,
    /// De combien de pixels le contenu s'est montré à côté de sa trajectoire (RYTHME-1).
    ///
    /// Le mouvement est intégré sur le pas qui sépare deux **débuts de rendu**, et montré
    /// pendant l'intervalle qui sépare deux **présentations**. Leur écart, multiplié par la
    /// vitesse, est la distance entre là où le contenu apparaît et là où il devrait être.
    /// C'est la seule grandeur de cette structure qui dise ce que l'œil voit.
    pub saut_px: u16,
    /// Combien de balayages le tempo visait pour cette image (TEMPO-1). Zéro hors mouvement.
    ///
    /// À comparer au nombre de balayages **observé** : si les deux divergent, le tempo vise
    /// juste mais la machine ne suit pas — ou l'inverse.
    pub tempo_balayages: u16,
    /// Ce que l'image a attendu avant de partir, en microsecondes, pour tenir ce tempo.
    pub tempo_attente_us: u32,
    /// Le rapport `temps intégré / temps montré`, en millièmes. Mille vaut « exact ».
    ///
    /// Au-dessous, le contenu a moins avancé que sa durée d'affichage ne le demandait : il
    /// traîne. Au-dessus, il a sauté. Zéro dit que la vue ne bougeait pas.
    pub fidelite_millieme: u16,
    /// Ce qui empêchait l'application de dormir — un bit par raison de réveil.
    ///
    /// Un masque et non la seule raison la plus pressée : savoir laquelle a gagné la course
    /// ne dit pas laquelle il faudrait supprimer. Les huit tiennent dans un `u16`.
    pub reveils: u16,
}

/// Les durées des postes d'une image, en microsecondes, dans l'ordre de la chronique.
///
/// Un type à part pour une seule raison : `Default` ne se dérive que jusqu'à trente-deux
/// éléments, et la borne des postes n'a pas à dépendre d'une limite de la bibliothèque
/// standard. Il se lit et s'écrit comme le tableau qu'il enveloppe.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PostesUs(pub [u32; POSTES]);

impl Default for PostesUs {
    fn default() -> Self {
        Self([0; POSTES])
    }
}

impl std::ops::Deref for PostesUs {
    type Target = [u32; POSTES];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for PostesUs {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Instantane {
    /// Cette image n'a-t-elle **rien** redessiné du tout ?
    ///
    /// La salissure était propre : l'image précédente était encore exacte, et on s'est
    /// contenté de la représenter. C'est l'issue la moins chère qui existe.
    pub fn evitee(&self) -> bool {
        self.region_px == 0
    }

    /// La part de la fenêtre qui a été redessinée, entre 0 et 1.
    pub fn part_redessinee(&self) -> f64 {
        if self.fenetre_px == 0 {
            return 1.0;
        }
        f64::from(self.region_px) / f64::from(self.fenetre_px)
    }

    pub(super) fn geste(&self) -> Geste {
        Geste::TOUS
            .get(self.geste as usize)
            .copied()
            .unwrap_or(Geste::Repos)
    }
}
