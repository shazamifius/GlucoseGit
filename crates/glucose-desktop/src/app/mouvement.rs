//! Ce que l'image doit montrer avant d'etre peinte : le mouvement de la camera, et la
//! finesse a laquelle la scene merite d'etre rendue.
//!
//! Les deux vont ensemble et n'allaient nulle part ailleurs. Le mouvement decide de ce qu'on
//! regarde ; la finesse decide de ce que cela coute. Ils se lisent a la meme seconde de
//! l'image -- juste avant de peindre -- et une boucle de rendu qui les portait en plus du
//! reste ne laissait plus voir sa propre structure.

use super::GlucoseApp;
use tiny_skia::Pixmap;

impl GlucoseApp {
    /// Accorde le tampon de la scene reduite au palier courant. Rend vrai s'il a change.
    ///
    /// Un changement de palier rend l'image precedente inutilisable : elle n'a plus la meme
    /// taille, donc plus rien a dire de ce qui est deja a l'ecran. L'appelant le traite comme
    /// un tampon neuf, ce qui force un rendu complet -- et c'est exact.
    pub(super) fn accorder_le_tampon_reduit(&mut self, largeur: u32, hauteur: u32) -> bool {
        let f = self.resolution.facteur();
        if f == 1 {
            return self.tampon_reduit.take().is_some();
        }
        // Le plafond arrondit vers le haut : une fenetre de 1001 pixels reduite de moitie en
        // demande 501, pas 500 -- sinon la derniere colonne n'aurait pas de source.
        let (w, h) = (largeur.div_ceil(f).max(1), hauteur.div_ceil(f).max(1));
        if self
            .tampon_reduit
            .as_ref()
            .is_some_and(|p| p.width() == w && p.height() == h)
        {
            return false;
        }
        self.tampon_reduit = Pixmap::new(w, h);
        true
    }

    /// Publique pour que les bancs et les tests puissent jouer une image sans fenetre : le
    /// geste ne deplace plus rien tout seul, donc le verifier demande de jouer l'image.
    ///
    /// Deux sources de mouvement, dans cet ordre : ce que la main demande, puis le vol vers
    /// une destination decidee. L'ordre ne les departage pas -- un geste **annule** le vol au
    /// moment ou il arrive, donc les deux ne se disputent jamais la meme image.
    pub fn appliquer_l_elan(&mut self, largeur: u32, hauteur: u32) {
        let dt = self.duree_de_l_image();
        self.appliquer_la_demande(largeur, hauteur);
        self.appliquer_le_vol(largeur, hauteur, dt);
        self.mesurer_ce_que_l_oeil_voit_bouger((largeur, hauteur), dt);
    }

    /// De combien la vue vient de bouger, et ce que l'œil tolère à cette vitesse.
    ///
    /// # Pourquoi on mesure le **déplacement**, et non la vitesse de l'élan
    ///
    /// L'élan connaît sa propre vitesse, mais il n'est qu'une des sources de mouvement : un
    /// vol de caméra, un geste direct, un signet rappelé déplacent la vue sans passer par
    /// lui. Ce que l'œil voit, c'est le mouvement **résultant**, d'où qu'il vienne.
    ///
    /// Et il se mesure exactement : le déplacement d'un point de l'écran entre deux cadrages
    /// est affine en sa position, donc son maximum est atteint à un coin. C'est déjà ce que
    /// calcule [`crate::interactions::vol::ecart_max_en_pixels`], et c'est la bonne grandeur
    /// — un zoom qui ne translate rien fait pourtant glisser toute l'image sous l'œil.
    fn mesurer_ce_que_l_oeil_voit_bouger(&mut self, (largeur, hauteur): (u32, u32), dt: f64) {
        let Some(tableau) = self.store.active_board() else {
            return;
        };
        let vue = tableau.viewport;
        let ecran = glucose_core::membrane_focus::ScreenSize {
            width: f64::from(largeur),
            height: f64::from(hauteur),
        };
        let vitesse = match (self.vue_precedente, dt > 0.0) {
            (Some(avant), true) => {
                crate::interactions::vol::ecart_max_en_pixels(avant, vue, ecran) / dt
            }
            // Première image, ou durée nulle : on ne sait rien, donc on ne dégrade rien.
            _ => 0.0,
        };
        self.vue_precedente = Some(vue);
        self.perception = crate::perception::Perception::a_la_vitesse(vitesse, self.scale_factor);
    }

    /// Ce que cette image a coute decide de la finesse de la suivante.
    ///
    /// Deux questions, et il faut les deux : le budget dit ce dont on a **besoin**, la
    /// perception ce qui est **licite**. La seconde manquait, et c'est elle qui faisait
    /// persister les gros blocs pendant que l'amortissement s'eteignait.
    pub(super) fn accorder_la_finesse(&mut self, ecoule: std::time::Duration) {
        // Ce que cette image a coute decide de la finesse de la suivante. `en_cours` dit si la
        // main demande encore quelque chose : des qu'elle se tait, la nettete revient.
        let scene = crate::perf::valeur_du_compteur("img_scene_us").unwrap_or(0.0);
        let mesure = crate::resolution::Mesure {
            image: ecoule,
            scene: std::time::Duration::from_micros(scene.max(0.0) as u64),
        };
        // **Le plancher de la charte, et non la cible de cout.** `budget_rendu` vaut deux
        // millisecondes et demie : c'est ce qu'on VISE pour laisser du temps au travail de
        // fond, pas le seuil au-dela duquel on a le droit d'abimer l'image. Vise ainsi, la
        // reduction se declenchait des qu'une image depassait 2,5 ms -- c'est-a-dire presque
        // toujours -- et rendait la scene a MOITIE resolution : 22 % des images d'une session
        // reelle, a facteur 1,98, en gros blocs illisibles.
        //
        // L'utilisateur l'a tranche sur capture : « c'est ultra pixelise, sur un ecran comme
        // le mien ca passe pas ; deja ca lag, et ensuite c'est moche ». On degradait donc
        // violemment ce qui se voit, sans meme y gagner la cadence.
        //
        // A dix millisecondes, les deux leviers visent le meme plancher, et l'ordre tombe de
        // lui-meme : le filtre pixelise d'abord -- il se decide par prevision, AVANT le rendu
        // -- et la resolution ne cede que si l'image mesuree depasse malgre lui. On abime
        // d'abord ce qui se voit le moins.
        let plancher = crate::cadence::BUDGET_TOTAL;
        // Un vol compte comme un mouvement au meme titre que l'elan : la vue change sous
        // l'oeil, et c'est cela seul qui autorise a rendre plus grossier.
        let en_mouvement = self.elan.en_cours() || self.vol.en_cours();
        // Ce que l'oeil tolere a la vitesse a laquelle la vue vient de bouger. Le budget dit
        // ce dont on a BESOIN, ceci dit ce qui est LICITE -- et degrader demande les deux.
        let plafond = self.perception.facteur_admissible();
        self.resolution
            .observer(mesure, plancher, en_mouvement, plafond);
    }

    /// La duree ecoulee depuis l'image precedente, en secondes.
    ///
    /// Plafonnee : une image tres longue -- un dialogue natif ouvert, une fenetre reduite --
    /// ne doit pas faire franchir tout un vol d'un coup, ce qui serait precisement la
    /// teleportation qu'on cherche a supprimer.
    fn duree_de_l_image(&mut self) -> f64 {
        const PAS_MAX: f64 = 0.1;
        let maintenant = std::time::Instant::now();
        let dt = self
            .derniere_image
            .map_or(0.0, |avant| maintenant.duration_since(avant).as_secs_f64());
        self.derniere_image = Some(maintenant);
        dt.min(PAS_MAX)
    }

    /// Ce que la main a demande et que l'image n'a pas encore montre.
    fn appliquer_la_demande(&mut self, largeur: u32, hauteur: u32) {
        let diagonale = f64::from(largeur).hypot(f64::from(hauteur));
        let Some(m) = self.elan.avancer(std::time::Instant::now(), diagonale) else {
            return;
        };
        if m.pan != (0.0, 0.0) {
            self.store.pan(m.pan.0, m.pan.1);
        }
        if m.octaves != 0.0 {
            // Le zoom se dit en octaves et s'applique en facteur : `2^n`, et rien d'autre.
            let facteur = m.octaves.exp2();
            let bornes = crate::interactions::pan_zoom::WHEEL_SCALE_RANGE;
            self.store.zoom(facteur, m.ancre.0, m.ancre.1, bornes);
        }
    }

    /// Un pas de vol vers la destination en cours, s'il y en a une.
    fn appliquer_le_vol(&mut self, largeur: u32, hauteur: u32, dt: f64) {
        let Some(tableau) = self.store.active_board() else {
            return;
        };
        let (id, vue) = (tableau.id.clone(), tableau.viewport);
        let ecran = glucose_core::membrane_focus::ScreenSize {
            width: f64::from(largeur),
            height: f64::from(hauteur),
        };
        if let Some(suivante) = self.vol.avancer(vue, ecran, dt) {
            self.store.set_viewport(&id, suivante);
        }
    }
}
