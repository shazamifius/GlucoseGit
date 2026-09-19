//! Comment une image se rend : ou, a quelle finesse, et ce qu'on a le droit d'y abimer.
//!
//! Les trois reponses voyagent ensemble parce qu'elles disent la meme chose sous trois
//! angles -- comment passer du repere de la fenetre a celui du tampon, et ce que l'oeil
//! tolerera du resultat. Un rendu qui les recevrait separement pourrait les appliquer dans
//! le mauvais ordre.

/// La scene rendue plus petite que la fenetre, et de combien.
///
/// Les deux ne se separent jamais : un tampon sans son facteur ne dit pas comment l'agrandir,
/// et un facteur sans son tampon ne designe rien.
pub struct SceneReduite<'a> {
    pub tampon: &'a mut tiny_skia::Pixmap,
    pub facteur: u32,
}

/// Ce que l'oeil et la main font en ce moment -- ce que le rendu a le droit d'abimer, et ce
/// qu'il a interet a ne pas repeindre.
///
/// # Deux questions, et il faut les deux
///
/// `degradation_permise` vient de la perception : l'oeil tolere-t-il qu'on reduise la finesse
/// a la vitesse ou la vue bouge ? Elle tombe a faux des que la vitesse passe sous celle de la
/// poursuite oculaire -- environ mille pixels par seconde -- donc sur toute la fin d'un
/// freinage.
///
/// `en_mouvement` vient de l'elan et du vol : la vue bouge-t-elle encore, si peu que ce soit ?
/// C'est cette question-la qui decide si la grille de tuiles doit servir. Pendant un
/// freinage, l'oeil ne tolere plus qu'on abime -- mais repasser au rendu direct a trente
/// millisecondes ferait sauter le contenu de soixante pixels, ce qui se voit infiniment plus
/// qu'un agrandissement d'un facteur un virgule trois. La finesse revient a l'ARRET, d'un
/// coup, quand plus rien ne bouge.
///
/// **C'est un choix de ressenti, et il se juge a l'ecran**, pas sur le papier : la charte le
/// dit de la nettete en mouvement. Il est ecrit ici pour qu'on sache ou il se defait.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Regard {
    /// L'oeil tolere-t-il qu'on abime l'image ? (voir [`crate::perception`])
    pub degradation_permise: bool,
    /// La vue bouge-t-elle encore ?
    pub en_mouvement: bool,
}

impl Regard {
    /// A l'arret : rien n'est degradable, rien ne bouge.
    pub fn immobile() -> Self {
        Self::default()
    }
}

/// Ou et a quelle finesse la scene se rend dans le pixmap qu'on lui donne.
///
/// Les deux vont ensemble parce qu'ils disent la meme chose -- comment passer du repere de la
/// fenetre a celui du tampon -- et qu'un rendu qui les recevrait separement pourrait les
/// appliquer dans le mauvais ordre.
#[derive(Debug, Clone, Copy)]
pub struct Cadrage {
    /// L'origine de l'ecran, pour un rendu par region (A.1).
    pub origine: (f32, f32),
    /// De combien la scene est rendue plus petite que la fenetre (voir [`crate::resolution`]).
    pub reduction: f64,
    /// La vue a employer, quand ce n'est pas celle du document.
    ///
    /// # Pourquoi une tuile en a besoin
    ///
    /// Une tuile ne se rend pas dans le repere de l'ecran mais dans le sien : son coin est
    /// l'origine, et son echelle est celle de son niveau. Elle ne depend donc d'AUCUNE vue --
    /// c'est meme toute sa raison d'etre. Sans cette porte, la rendre demanderait d'ecrire
    /// dans le document la vue qu'on veut, puis de la remettre : un etat partage modifie le
    /// temps d'un rendu, ce qu'aucun test ne saurait rattraper.
    pub vue: Option<glucose_core::types::Viewport>,
    /// L'oeil tolere-t-il qu'on abime cette image ? (voir [`crate::perception`])
    ///
    /// Faux a l'arret et a toute vitesse que l'oeil sait poursuivre -- donc sur toute la fin
    /// d'un amortissement. Le budget dit ce dont on a **besoin**, ceci dit ce qui est
    /// **licite**, et degrader demande les deux.
    pub degradation_permise: bool,
    /// La vue bouge-t-elle encore ? (voir [`Regard::en_mouvement`])
    pub en_mouvement: bool,
}

impl Cadrage {
    /// La fenetre entiere, a sa taille reelle.
    pub fn plein() -> Self {
        Self {
            origine: (0.0, 0.0),
            reduction: 1.0,
            vue: None,
            degradation_permise: false,
            en_mouvement: false,
        }
    }

    /// La fenetre entiere, rendue `f` fois plus petite.
    pub fn reduit(f: u32) -> Self {
        Self {
            origine: (0.0, 0.0),
            reduction: f64::from(f.max(1)),
            vue: None,
            // Une scene deja rendue plus petite l'est parce que l'oeil le tolerait : c'est le
            // plafond de la perception qui a decide du facteur (`resolution::observer`).
            degradation_permise: true,
            en_mouvement: true,
        }
    }

    /// Une region de la fenetre, a sa taille reelle.
    pub fn region(origine: (f32, f32)) -> Self {
        Self {
            origine,
            reduction: 1.0,
            vue: None,
            degradation_permise: false,
            en_mouvement: false,
        }
    }

    /// Le cadrage d'une **tuile** : sa vue a elle, et rien du document.
    ///
    /// L'echelle est celle du niveau, et l'origine place le coin de la tuile sur celui du
    /// pixmap. Une tuile est degradable par construction : elle se rend a son echelle exacte,
    /// donc il n'y a rien a y abimer.
    pub fn tuile(niveau: i32, origine_monde: (f64, f64)) -> Self {
        let echelle = glucose_core::tuile::Adresse::echelle(niveau);
        Self {
            origine: (0.0, 0.0),
            reduction: 1.0,
            vue: Some(glucose_core::types::Viewport {
                scale: echelle,
                x: -origine_monde.0 * echelle,
                y: -origine_monde.1 * echelle,
            }),
            degradation_permise: false,
            en_mouvement: false,
        }
    }

    /// Le meme cadrage, ou l'oeil tolere -- ou ne tolere pas -- qu'on abime l'image.
    pub fn avec_degradation(self, permise: bool) -> Self {
        Self {
            degradation_permise: permise,
            ..self
        }
    }

    /// Le meme cadrage, sous ce que l'oeil et la main font en ce moment.
    pub fn sous_le_regard(self, regard: Regard) -> Self {
        Self {
            degradation_permise: regard.degradation_permise,
            en_mouvement: regard.en_mouvement,
            ..self
        }
    }
}
