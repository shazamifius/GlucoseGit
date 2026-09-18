//! Ce qu'une image va coûter, **avant** de la dessiner (COUT-1).
//!
//! # Pourquoi prédire, et pas mesurer les images par seconde
//!
//! Une boucle qui compte les images par seconde et corrige ensuite arrive toujours trop tard :
//! elle dégrade après avoir raté, et elle raffine après avoir dégradé. Ce qu'il faut est
//! l'inverse — **savoir ce qu'une image coûtera pendant qu'on décide encore de ce qu'elle
//! contiendra**, et n'engager que ce qui tient dans le budget.
//!
//! Le budget ne se discute pas : **cent images par seconde, quoi qu'il se passe**, donc dix
//! millisecondes par image. Quand la scène n'y tient pas, ce qui cède est la finesse du rendu,
//! jamais la cadence.
//!
//! # Comment la machine se comprend elle-même
//!
//! On ne l'interroge pas, on l'**observe**. Chaque nature de travail — un pixel recopié, un
//! pixel rééchantillonné, un pixel de vignette — a un coût par unité que la machine vient de
//! démontrer en le faisant. Ce coût dit tout ce qu'il y a à savoir d'elle : son processeur, sa
//! mémoire, sa charge du moment, et jusqu'à son bridage thermique, qui se lit comme un débit
//! qui baisse sans que personne n'ait eu à demander sa température.
//!
//! # Le dernier débit, et pourquoi pas une moyenne
//!
//! Le coût par unité varie **lentement** : une machine qui chauffe met des secondes à se
//! brider, un jeu qui démarre met des secondes à prendre la carte. Pour un tel processus, la
//! dernière mesure est le meilleur prédicteur de la suivante — et c'est le seul qui ne demande
//! aucune constante de temps, aucune fenêtre, aucun facteur d'oubli à choisir.
//!
//! Le cumul de la session sert de repli tant que rien n'a encore été observé, et seulement là.
//!
//! # Le résidu est la vraie mesure
//!
//! `mesuré − prédit` est ce que le modèle **ne comprend pas**. Nul, il dit que la machine est
//! comprise et que le budget peut être tenu par le calcul. Élevé, il désigne exactement où
//! chercher : un défaut de cache, une file de commandes pleine, un poste qu'on a oublié de
//! compter. C'est la seule grandeur de ce module qui apprenne quelque chose de neuf.

use core::time::Duration;

/// Une nature de travail dont le coût se mesure par unité.
///
/// Les trois premières se comptent en **pixels**, et leurs coûts diffèrent d'un ordre de
/// grandeur : les confondre reviendrait à prédire la moyenne de choses qui n'ont rien à voir.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Travail {
    /// Un pixel repris tel quel depuis une vignette déjà à la bonne taille.
    PixelRepris,
    /// Un pixel rééchantillonné depuis un niveau de pyramide, avec interpolation.
    PixelLisse,
    /// Un pixel rééchantillonné en prenant le texel le plus proche, sans interpoler.
    PixelPixelise,
    /// Un pixel écrit dans une vignette en cours de construction.
    PixelDeVignette,
}

impl Travail {
    /// Toutes les natures, dans l'ordre de leur indice.
    pub const TOUS: [Travail; 4] = [
        Travail::PixelRepris,
        Travail::PixelLisse,
        Travail::PixelPixelise,
        Travail::PixelDeVignette,
    ];

    fn indice(self) -> usize {
        match self {
            Travail::PixelRepris => 0,
            Travail::PixelLisse => 1,
            Travail::PixelPixelise => 2,
            Travail::PixelDeVignette => 3,
        }
    }

    /// Le nom court qui paraît dans la trace.
    pub fn nom(self) -> &'static str {
        match self {
            Travail::PixelRepris => "px repris",
            Travail::PixelLisse => "px lisse",
            Travail::PixelPixelise => "px pixelise",
            Travail::PixelDeVignette => "px vignette",
        }
    }
}

/// Le budget dur d'une image : **cent images par seconde, quoi qu'il se passe**.
///
/// Ce n'est pas une cible mais un plancher. Il ne dépend pas de l'écran : un moniteur à
/// 240 Hz demande des images plus souvent, il ne rend pas une image de vingt millisecondes
/// plus acceptable. Une image qui dépasse ce budget se **pixelise** plutôt que de durer.
pub const BUDGET: Duration = Duration::from_millis(10);

/// Ce qu'une nature de travail coûte, tel que la machine vient de le démontrer.
#[derive(Debug, Clone, Copy, Default)]
struct Debit {
    /// Le dernier coût par unité observé, en nanosecondes.
    dernier: f64,
    /// Le cumul de la session, qui sert de repli avant la première observation.
    unites: u64,
    nanos: u64,
}

impl Debit {
    fn par_unite(&self) -> Option<f64> {
        if self.dernier > 0.0 {
            return Some(self.dernier);
        }
        (self.unites > 0).then(|| self.nanos as f64 / self.unites as f64)
    }
}

/// Ce que cette machine coûte, appris de ce qu'elle vient de faire.
#[derive(Debug, Clone, Default)]
pub struct Cout {
    debits: [Debit; 4],
}

impl Cout {
    pub fn nouveau() -> Self {
        Self::default()
    }

    /// Enregistre ce qu'un travail vient de coûter réellement.
    ///
    /// Une mesure portant sur zéro unité n'apprend rien et n'est pas retenue : elle ne
    /// mesurerait que le bruit de l'horloge.
    pub fn observer(&mut self, travail: Travail, unites: u64, duree: Duration) {
        if unites == 0 {
            return;
        }
        let nanos = duree.as_nanos().min(u128::from(u64::MAX)) as u64;
        let debit = &mut self.debits[travail.indice()];
        debit.dernier = nanos as f64 / unites as f64;
        debit.unites = debit.unites.saturating_add(unites);
        debit.nanos = debit.nanos.saturating_add(nanos);
    }

    /// Ce que ce travail coûtera par unité, ou `None` tant que la machine ne l'a jamais fait.
    pub fn par_unite(&self, travail: Travail) -> Option<f64> {
        self.debits[travail.indice()].par_unite()
    }

    /// Ce que coûtera un travail de cette taille.
    pub fn prevoir(&self, travail: Travail, unites: u64) -> Option<Duration> {
        let par_unite = self.par_unite(travail)?;
        Some(Duration::from_nanos((par_unite * unites as f64) as u64))
    }

    /// Ce que coûtera un ensemble de travaux, si **tous** sont connus.
    ///
    /// `None` dès qu'une seule nature n'a jamais été observée : une prévision partielle
    /// sous-estimerait, et une sous-estimation fait tenir un budget qu'on dépasse — c'est
    /// exactement l'erreur que ce module existe pour éviter.
    pub fn prevoir_tout(&self, travaux: &[(Travail, u64)]) -> Option<Duration> {
        let mut total = Duration::ZERO;
        for (travail, unites) in travaux {
            if *unites == 0 {
                continue;
            }
            total += self.prevoir(*travail, *unites)?;
        }
        Some(total)
    }
}

/// Ce qu'il faut faire d'une scène dont on connaît le coût prévu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Finesse {
    /// La scène tient dans le budget : on la rend telle qu'elle est.
    Lisse,
    /// La scène n'y tient pas : les images se rééchantillonnent **sans interpoler**.
    ///
    /// C'est la pixelisation, et c'est ce que la charte demande — quand le budget ne tient
    /// pas, ce qui cède est la finesse du rendu, jamais la cadence.
    Pixelisee,
}

/// Ce qu'il faut faire d'une scène dont on prévoit qu'elle coûtera `prevu`.
///
/// # Pourquoi la décision se prend sur une prévision et pas sur la dernière image
///
/// Décider d'après l'image précédente, c'est pixeliser **après** avoir raté, puis redevenir
/// lisse après avoir pixelisé : la scène oscille, et l'œil voit l'oscillation. Une prévision
/// décide avant, donc une seule fois, et la scène ne change d'aspect que lorsque son contenu
/// change vraiment.
///
/// Sans prévision — la machine n'a encore rien démontré — on reste **lisse** : on ne dégrade
/// pas ce qu'on n'a pas mesuré.
pub fn finesse_pour(prevu: Option<Duration>) -> Finesse {
    match prevu {
        Some(prevu) if prevu > BUDGET => Finesse::Pixelisee,
        _ => Finesse::Lisse,
    }
}

#[cfg(test)]
mod tests;
