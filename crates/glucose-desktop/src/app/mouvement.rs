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
        // **Le pas vient de l'horloge**, donc de ce que l'ecran a montre -- et non du temps
        // qu'il a fallu pour calculer quoi que ce soit. Une seule horloge pour l'elan, le vol
        // et la mesure : deux d'entre elles divergeaient, et c'est ce qui se voyait.
        let pas = self.horloge.pas();
        let dt = pas.as_secs_f64();
        self.appliquer_la_demande(largeur, hauteur, pas);
        self.appliquer_le_vol(largeur, hauteur, dt);
        // La vue vient de bouger : entre-t-on en focus, en sort-on (MEMB-2) ?
        self.suivre_le_focus((largeur, hauteur));
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
        // Le mouvement se **décompose**, parce que l'œil ne les traite pas pareil. Le
        // déplacement d'un point est affine en sa position : sa part **constante** est le
        // déplacement du centre de l'écran — un glissement uniforme, que la poursuite annule
        // — et sa part **linéaire** est le zoom, qui écarte le contenu radialement et
        // qu'aucune poursuite ne peut suivre ailleurs qu'au point fixe.
        let (deplacement, zoom) = match (self.vue_precedente, dt > 0.0) {
            (Some(avant), true) => {
                let centre = (ecran.width / 2.0, ecran.height / 2.0);
                let monde = (
                    (centre.0 - avant.x) / avant.scale,
                    (centre.1 - avant.y) / avant.scale,
                );
                let apres = (monde.0 * vue.scale + vue.x, monde.1 * vue.scale + vue.y);
                let deplacement = (apres.0 - centre.0).hypot(apres.1 - centre.1);
                // Au bord, le zoom écarte de la demi-diagonale fois le rapport des échelles.
                let demi_diagonale = ecran.width.hypot(ecran.height) / 2.0;
                let zoom = demi_diagonale * (vue.scale / avant.scale - 1.0).abs();
                (deplacement / dt, zoom / dt)
            }
            // Première image, ou durée nulle : on ne sait rien, donc on ne dégrade rien.
            _ => (0.0, 0.0),
        };
        self.vue_precedente = Some(vue);
        // La vue s'est immobilisee : le geste est fini, et le prochain pourra venir d'une
        // autre source. C'est le seul instant ou l'oubli est sur, et il ne coute aucune
        // constante de temps.
        if deplacement == 0.0 && zoom == 0.0 && !self.elan.en_cours() {
            self.defilement_au_doigt = false;
        }
        self.perception =
            crate::perception::Perception::a_la_vitesse(deplacement, zoom, self.scale_factor);
        // **Ce que cette image montre du mouvement**, retenu pour la presentation (RYTHME-1).
        //
        // Le pas est celui sur lequel le mouvement vient d'etre integre ; la presentation, qui
        // arrive plus tard, le comparera au temps pendant lequel l'image aura ete VUE. Les
        // deux ne coincident que si le rendu coute la meme chose d'une image a l'autre --
        // hypothese que le terrain dement d'un facteur dix.
        //
        // La comparaison precedente -- le rapport de deux vitesses apparentes successives --
        // ne mesurait pas cela. Elle avait aussi un seuil choisi, un pixel par seconde, que
        // la fin de tout amortissement traverse : d'ou un « pire ecart 655x » qui ne disait
        // rien d'autre que « la vitesse est passee pres de zero », ce qui est le contraire
        // d'un defaut.
        self.pas_et_vitesse = (
            std::time::Duration::from_secs_f64(dt.max(0.0)),
            deplacement + zoom,
        );
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
        // **Ce que le tempo vise pour descendre d'un cran, et non un plancher abstrait.**
        //
        // La premiere version visait `budget_rendu`, deux millisecondes et demie : la
        // reduction se declenchait des qu'une image depassait ce chiffre -- presque toujours
        // -- et rendait la scene a MOITIE resolution. L'utilisateur l'a tranche sur capture :
        // « c'est ultra pixelise, sur un ecran comme le mien ca passe pas ».
        //
        // La deuxieme visait le plancher de la charte, dix millisecondes. Et la chronique a
        // montre que ce n'etait toujours pas la bonne question : le TEMPO tenait quatre
        // balayages, soit 16,7 ms par image, pendant qu'on degradait 48 % des images pour en
        // tenir dix. On abimait ce qui se voit pour un budget que personne n'attendait -- et
        // l'utilisateur l'a redit : « mon ordinateur est plutot puissant, pourquoi cette
        // pixelisation ».
        //
        // La cible est donc un cran sous ce que le tempo tient DEJA : y arriver le fait
        // descendre, et la cible descend avec lui, jusqu'au plancher de la charte. L'ordre
        // des leviers ne change pas : le filtre pixelise d'abord -- il se decide par
        // prevision, AVANT le rendu -- et la resolution ne cede que si l'image mesuree
        // depasse malgre lui. On abime d'abord ce qui se voit le moins.
        let plancher = self.tempo.cible_pour_descendre();
        // Un vol compte comme un mouvement au meme titre que l'elan : la vue change sous
        // l'oeil, et c'est cela seul qui autorise a rendre plus grossier.
        let en_mouvement = self.elan.en_cours() || self.vol.en_cours();
        // Ce que l'oeil tolere a la vitesse a laquelle la vue vient de bouger. Le budget dit
        // ce dont on a BESOIN, ceci dit ce qui est LICITE -- et degrader demande les deux.
        let plafond = self.perception.facteur_admissible();
        self.resolution
            .observer(mesure, plancher, en_mouvement, plafond);
    }

    /// Ce que la main a demande et que l'image n'a pas encore montre.
    fn appliquer_la_demande(&mut self, largeur: u32, hauteur: u32, pas: std::time::Duration) {
        let diagonale = f64::from(largeur).hypot(f64::from(hauteur));
        let Some(m) = self.elan.avancer(std::time::Instant::now(), pas, diagonale) else {
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
        if let Some(boite) = self.vol.prendre_la_boite_a_poser() {
            let bandeau = f64::from(self.ui.header_height());
            let vue = crate::interactions::vol::vue_sur(boite, ecran, bandeau);
            self.store.set_viewport(&id, vue);
            return;
        }
        if let Some(suivante) = self.vol.avancer(vue, ecran, dt) {
            self.store.set_viewport(&id, suivante);
        }
    }
}
