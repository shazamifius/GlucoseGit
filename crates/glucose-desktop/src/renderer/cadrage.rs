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
    /// L'oeil tolere-t-il qu'on abime cette image ? (voir [`crate::perception`])
    ///
    /// Faux a l'arret et a toute vitesse que l'oeil sait poursuivre -- donc sur toute la fin
    /// d'un amortissement. Le budget dit ce dont on a **besoin**, ceci dit ce qui est
    /// **licite**, et degrader demande les deux.
    pub degradation_permise: bool,
}

impl Cadrage {
    /// La fenetre entiere, a sa taille reelle.
    pub fn plein() -> Self {
        Self {
            origine: (0.0, 0.0),
            reduction: 1.0,
            degradation_permise: false,
        }
    }

    /// La fenetre entiere, rendue `f` fois plus petite.
    pub fn reduit(f: u32) -> Self {
        Self {
            origine: (0.0, 0.0),
            reduction: f64::from(f.max(1)),
            // Une scene deja rendue plus petite l'est parce que l'oeil le tolerait : c'est le
            // plafond de la perception qui a decide du facteur (`resolution::observer`).
            degradation_permise: true,
        }
    }

    /// Une region de la fenetre, a sa taille reelle.
    pub fn region(origine: (f32, f32)) -> Self {
        Self {
            origine,
            reduction: 1.0,
            degradation_permise: false,
        }
    }

    /// Le meme cadrage, ou l'oeil tolere -- ou ne tolere pas -- qu'on abime l'image.
    pub fn avec_degradation(self, permise: bool) -> Self {
        Self {
            degradation_permise: permise,
            ..self
        }
    }
}
