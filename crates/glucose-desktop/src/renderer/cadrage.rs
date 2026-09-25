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

/// Quelle part de la scene ce rendu produit.
///
/// # Pourquoi la scene se coupe en deux, et exactement la
///
/// La voie graphique pose les photos elle-meme ([`crate::present::scene_gpu`]). Le processeur
/// doit alors produire ce qui les entoure -- mais pas d'un seul tenant : **une partie passe
/// dessous et une partie dessus**, et les melanger mettrait une membrane par-dessus la photo
/// qu'elle contient.
///
/// La frontiere n'est pas choisie : c'est l'ordre de `rendre_la_region`, qui suit celui de
/// Glucose Tauri. Sous les photos viennent le fond, les lueurs, les membranes et les
/// dossiers -- tous des CONTENANTS. Dessus viennent les annotations et les reperes du geste.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Couche {
    /// Tout, d'un seul tenant : la voie processeur, qui pose aussi les photos.
    Tout,
    /// Ce qui passe **sous** les photos : le fond, les lueurs, les membranes, les dossiers.
    Dessous,
    /// Ce qui passe **sur** les photos : les annotations et les reperes du geste.
    Dessus,
}

impl Couche {
    /// Cette couche porte-t-elle ce qui se dessine avant les photos ?
    pub fn porte_le_dessous(self) -> bool {
        matches!(self, Self::Tout | Self::Dessous)
    }

    /// Le processeur peint-il lui-meme le fond et les lueurs ?
    ///
    /// # Ou passe vraiment la frontiere, et pourquoi elle bouge
    ///
    /// Elle n'est pas fixee une fois pour toutes : elle recule a chaque passe qui descend sur
    /// la carte. Le fond et les lueurs y sont depuis la fiche 21 etape 3, donc la couche du
    /// dessous ne porte plus que les membranes et les dossiers -- et, sur un document qui n'en
    /// a pas, plus rien du tout.
    ///
    /// C'est ce qui rend le televersement de cette couche EVITABLE au lieu d'etre seulement
    /// mis en cache : un fond opaque existe toujours, une couche vide n'existe pas.
    pub fn porte_le_fond(self) -> bool {
        matches!(self, Self::Tout)
    }

    /// Cette couche porte-t-elle ce qui se dessine apres les photos ?
    pub fn porte_le_dessus(self) -> bool {
        matches!(self, Self::Tout | Self::Dessus)
    }

    /// Le processeur pose-t-il lui-meme les photos ?
    pub fn porte_les_photos(self) -> bool {
        matches!(self, Self::Tout)
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
    /// Quelle part de la scene ce rendu produit (voir [`Couche`]).
    pub couche: Couche,
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
            couche: Couche::Tout,
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
            couche: Couche::Tout,
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
            couche: Couche::Tout,
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
            couche: Couche::Tout,
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

// ── Où la vue tombe, et ce qu'elle montre ────────────────────────────────────

use super::Renderer;
use crate::canvas::screen_to_world;
use glucose_core::store::Store;

impl Renderer {
    /// Ou la vue tombe dans ce pixmap, et quels noeuds y apparaissent.
    ///
    /// # Les deux transformations, et pourquoi une seule ligne les porte
    ///
    /// La scene se rend dans le repere du pixmap cible, qui n'est pas toujours celui de la
    /// fenetre. `world_to_screen` vaut `monde x echelle + vp`, ce qui suffit a tout dire :
    ///
    /// * la **reduction** -- diviser l'ecran par `f` revient a diviser l'echelle ET la
    ///   translation par `f`. Rien d'autre n'a besoin de le savoir : le culling se resserre
    ///   tout seul, et le niveau de detail suit, ce qui est exactement ce qu'on attend d'une
    ///   image plus petite ;
    /// * l'**origine** -- decaler la vue de `-origine` deplace l'origine de l'ecran d'autant.
    ///
    /// L'index spatial se remet d'accord ici, et non chez l'appelant. Il l'etait dans
    /// `render`, si bien qu'un appelant de `rendre_la_scene` -- un banc, un temoin --
    /// dessinait un ecran VIDE sans que rien ne le dise. C'est arrive, et le banc annoncait
    /// alors un gain nul en toute bonne foi. Ne coute rien quand rien n'a change : la
    /// comparaison de version precede le balayage.
    pub(super) fn cadrer(
        &mut self,
        store: &Store,
        (width, height): (u32, u32),
        header_h: f32,
        cadrage: Cadrage,
    ) -> (glucose_core::types::Viewport, Vec<u32>, f32) {
        self.sync_spatial_index(store);
        // La vue du cadrage l'emporte : une tuile se rend dans SON repere, pas dans celui de
        // l'ecran, et elle n'a ni reduction ni origine a appliquer par-dessus.
        let mut vp = cadrage.vue.unwrap_or_else(|| store.viewport());
        // La densité suit la réduction comme le zoom : un tampon deux fois plus petit porte des
        // poignées deux fois plus petites, que l'agrandissement rend à leur taille (DPI-1).
        let mut densite = self.densite;
        if cadrage.vue.is_none() {
            let f = cadrage.reduction.max(1.0);
            vp.scale /= f;
            vp.x = vp.x / f - f64::from(cadrage.origine.0);
            vp.y = vp.y / f - f64::from(cadrage.origine.1);
            densite /= f as f32;
        }
        let (min_wx, min_wy) = screen_to_world(0.0, header_h as f64, &vp);
        let (max_wx, max_wy) = screen_to_world(width as f64, height as f64, &vp);
        let mut rangs = self.visibles_du_present(store, (min_wx, min_wy, max_wx, max_wy));
        // En focus, seules la membrane et son contenu se dessinent (MEMB-2).
        self.focus.filtrer(&mut rangs);
        crate::perf::stage("cull");
        (vp, rangs, densite)
    }

    /// **Les rangs de ce qui tombe dans la fenêtre, dans le document tel qu'il est** — geste
    /// en cours compris (GESTE-1).
    ///
    /// L'index décrit le document publié ; un geste ouvert l'a déjà changé. Une flèche qu'on
    /// tirait n'avait aucun rang dans l'index : elle restait invisible jusqu'au relâchement.
    /// Le suivi joint ce que le geste a touché, et relit les rangs de l'index au présent.
    /// Quand le geste a fait ce qu'aucun rang d'avant ne relit — un retrait —, l'index se
    /// remet d'accord ici, comme la fin du geste l'aurait fait.
    fn visibles_du_present(
        &mut self,
        store: &Store,
        (x0, y0, x1, y1): (f64, f64, f64, f64),
    ) -> Vec<u32> {
        let mut rangs = self.spatial_hash.query_rect_ranks(x0, y0, x1, y1, 200.0);
        let Some(board) = store.active_board() else {
            return rangs;
        };
        let geste = store.journal.en_cours();
        if self
            .suivi_du_geste
            .completer(geste, board, &self.spatial_hash, &mut rangs)
        {
            return rangs;
        }
        self.spatial_hash.index_board(board);
        self.suivi_du_geste.absorbe(geste, &self.spatial_hash);
        self.spatial_hash.query_rect_ranks(x0, y0, x1, y1, 200.0)
    }
}

#[cfg(test)]
mod tests_densite {
    use super::*;

    /// **DPI-1 — la densité suit la réduction comme le zoom** : une scène rendue deux fois
    /// plus petite porte des poignées deux fois plus petites, que l'agrandissement rend à leur
    /// taille. Sans cela, une scène pixelisée pour tenir la cadence aurait des poignées géantes.
    #[test]
    fn test_dpi_1_la_densite_suit_la_reduction() {
        let store = Store::new("densite");
        let mut renderer = Renderer::new();
        renderer.densite = 1.5;
        let (_, _, pleine) = renderer.cadrer(&store, (800, 600), 0.0, Cadrage::plein());
        let reduite = Cadrage {
            reduction: 2.0,
            ..Cadrage::plein()
        };
        let (_, _, divisee) = renderer.cadrer(&store, (400, 300), 0.0, reduite);
        assert_eq!((pleine, divisee), (1.5, 0.75));
    }
}
