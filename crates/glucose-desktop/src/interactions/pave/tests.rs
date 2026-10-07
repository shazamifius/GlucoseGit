//! Le pavé par *Direct Manipulation* (fiches 53 et 54) : la lecture de la transformation, et ce
//! que Glucose en montre — par la vraie boucle (`suivre_le_pave`, `prochain_reveil`), une
//! source factice jouant le système.

use super::*;
use std::cell::RefCell;
use std::rc::Rc;

/// Une suite de transformations telles que le système les donne, lue d'un bout à l'autre.
fn lire_tout(transformations: &[(f32, f32, f32)]) -> Vec<Mouvement> {
    let mut lecteur = Lecteur::default();
    transformations
        .iter()
        .filter_map(|t| lecteur.lire(*t))
        .collect()
}

fn mouvement(echelle: f64, decalage: (f64, f64)) -> Mouvement {
    Mouvement { echelle, decalage }
}

/// Applique une similitude à un point de la fenêtre : `r · q + b`.
fn appliquer(m: &Mouvement, (x, y): (f64, f64)) -> (f64, f64) {
    (m.echelle * x + m.decalage.0, m.echelle * y + m.decalage.1)
}

/// **Un déplacement rend ce qui a bougé depuis l'image précédente**, et rien quand rien n'a bougé.
#[test]
fn test_un_deplacement_rend_ses_ecarts() {
    let m = lire_tout(&[(1.0, 3.0, -2.0), (1.0, 3.0, -2.0), (1.0, 10.5, -2.0)]);
    assert_eq!(
        m,
        vec![mouvement(1.0, (3.0, -2.0)), mouvement(1.0, (7.5, 0.0))]
    );
}

/// **Le geste de sa session du 07/10** : un déplacement en biais dont l'échelle bouge de 0,128 %
/// — un biais des doigts. Il **continue de déplacer** : de (40, 30) à (55, 31) à échelle égale,
/// la similitude est la translation (15, 1). L'ancienne lecture prenait ce geste pour un
/// pincement jusqu'à la fin, et ne rendait **rien** de ce dernier pas — rejouée ici, elle
/// l'aurait bloqué.
#[test]
fn test_un_deplacement_en_biais_qui_change_un_peu_d_echelle_continue_de_deplacer() {
    let m = lire_tout(&[
        (1.0, 4.0, 0.0),
        (1.00128, 40.0, 30.0),
        (1.00128, 55.0, 31.0),
    ]);
    assert_eq!(m.len(), 3, "{m:?}");
    assert_eq!(m[0], mouvement(1.0, (4.0, 0.0)));
    // L'échelle de 1,00128 au deuxième pas : un zoom de 0,128 %, et le reste en translation.
    assert!((m[1].echelle - f64::from(1.00128f32)).abs() < 1e-12);
    assert_eq!(m[2], mouvement(1.0, (15.0, 1.0)), "le déplacement continue");
}

/// **Les similitudes d'un geste se composent exactement en la sienne** : un point du contenu,
/// montré en `c` au début du geste, est montré en `s · c + t` à la fin — et c'est exactement où
/// l'amènent, l'une après l'autre, les similitudes rendues image par image.
#[test]
fn test_les_similitudes_d_un_geste_se_composent_en_la_sienne() {
    let geste = [
        (1.0f32, 0.0f32, 0.0f32),
        (1.03, 12.0, -4.0),
        (1.11, 30.5, -9.0),
        (1.6, -40.0, 22.0),
        (2.0, -150.0, -75.0),
        (1.9, -140.0, -70.25),
    ];
    let c = (321.0, -87.5);
    let mut q = c;
    for m in lire_tout(&geste) {
        q = appliquer(&m, q);
    }
    let (s, tx, ty) = geste[geste.len() - 1];
    let attendu = (
        f64::from(s) * c.0 + f64::from(tx),
        f64::from(s) * c.1 + f64::from(ty),
    );
    assert!(
        (q.0 - attendu.0).abs() < 1e-9 && (q.1 - attendu.1).abs() < 1e-9,
        "{q:?} contre {attendu:?}"
    );
}

/// **Un pincement pur garde fixe le point du système** : autour de (300, 200), l'échelle
/// double et le décalage vaut `(1 − 2) · (300, 200)`. Le point fixe se retrouve, `b / (1 − r)` ;
/// une translation n'en a aucun.
#[test]
fn test_le_point_fixe_d_un_pincement() {
    let m = mouvement(2.0, (-300.0, -200.0));
    assert_eq!(m.point_fixe(), Some((300.0, 200.0)));
    assert_eq!(appliquer(&m, (300.0, 200.0)), (300.0, 200.0));
    assert_eq!(mouvement(1.0, (5.0, 0.0)).point_fixe(), None);
}

/// **Dite autour de n'importe quelle ancre, la similitude reste la même** : un déplacement `p`
/// puis un zoom autour de `a` envoie chaque point là où `r · q + b` l'envoie — l'ancre ne choisit
/// que la façon de la dire.
#[test]
fn test_dite_autour_d_une_ancre_la_similitude_est_la_meme() {
    let m = mouvement(1.37, (-58.0, 41.5));
    for a in [(0.0, 0.0), (300.0, 200.0), (-1000.0, 2500.0)] {
        let (p, octaves) = m.autour_de(a);
        let r = octaves.exp2();
        for q in [(0.0, 0.0), (640.0, 360.0), (-17.0, 900.0)] {
            let apres = (r * (q.0 + p.0 - a.0) + a.0, r * (q.1 + p.1 - a.1) + a.1);
            let attendu = appliquer(&m, q);
            assert!(
                (apres.0 - attendu.0).abs() < 1e-9 && (apres.1 - attendu.1).abs() < 1e-9,
                "ancre {a:?}, point {q:?} : {apres:?} contre {attendu:?}"
            );
        }
    }
}

/// **La remise repart de l'identité** : le geste suivant ne part pas de l'échelle où celui-ci
/// s'est arrêté.
#[test]
fn test_la_remise_repart_de_l_identite() {
    let mut lecteur = Lecteur::default();
    lecteur.lire((2.0, 0.0, 0.0));
    lecteur.remettre();
    assert_eq!(lecteur, Lecteur::default());
    assert_eq!(lecteur.lire(IDENTITE), None, "l'identité ne bouge rien");
    assert_eq!(
        lecteur.lire((1.0, 5.0, 0.0)),
        Some(mouvement(1.0, (5.0, 0.0)))
    );
}

/// **Compter ce que l'œil a vu** : un coin bouge de `diagonale / 2 · |r − 1|` sous le zoom.
/// Sur une diagonale de 2000, 1 % d'échelle fait 10 pixels : plus qu'un déplacement de 6, moins
/// qu'un de 14.
#[test]
fn test_le_zoom_domine_quand_il_bouge_plus_l_ecran() {
    let m = mouvement(1.01, (0.0, 0.0));
    assert!(m.zoom_domine((6.0, 0.0), 2000.0));
    assert!(!m.zoom_domine((0.0, 14.0), 2000.0));
}

/// Le système, joué : des mouvements à rendre, un geste en cours ou non, des signes de vie.
#[derive(Clone, Default)]
struct Factice(Rc<RefCell<Systeme>>);

/// Ce que le système joué tient : un geste en cours ou non, ce qu'il rendra, ses signes.
#[derive(Default)]
struct Systeme {
    en_geste: bool,
    mouvements: Vec<Mouvement>,
    signes: Vec<Signe>,
}

impl Pave for Factice {
    fn en_geste(&self) -> bool {
        self.0.borrow().en_geste
    }
    fn avancer(&mut self) -> Vec<Mouvement> {
        std::mem::take(&mut self.0.borrow_mut().mouvements)
    }
    fn signes(&mut self) -> Vec<Signe> {
        std::mem::take(&mut self.0.borrow_mut().signes)
    }
}

fn application(pave: &Factice) -> GlucoseApp {
    let mut app = GlucoseApp::new();
    app.pave = Some(Box::new(pave.clone()));
    app.mouse_pos = (300.0, 200.0);
    app.curseur_vu = true;
    app
}

/// Joue une similitude du pavé par la vraie boucle, et rend la vue d'avant et d'après.
fn jouer(
    m: Mouvement,
) -> (
    glucose_core::types::Viewport,
    glucose_core::types::Viewport,
    GlucoseApp,
) {
    let pave = Factice::default();
    let mut app = application(&pave);
    let avant = app.store.viewport();
    pave.0.borrow_mut().en_geste = true;
    pave.0.borrow_mut().mouvements = vec![m];
    app.bouger_la_camera(1280, 720);
    let apres = app.store.viewport();
    (avant, apres, app)
}

/// **Ce que la vue montre suit la similitude, point par point** : le point du monde montré en
/// `q` avant l'image est montré en `r · q + b` après — en un zoom, en un déplacement, et
/// mêlés, y compris quand le point fixe du système n'est pas le curseur.
#[test]
fn test_la_vue_suit_la_similitude_point_par_point() {
    for m in [
        mouvement(2.0, (-300.0, -200.0)),
        mouvement(1.0, (30.0, -12.0)),
        mouvement(1.25, (40.0, -7.0)),
        mouvement(0.8, (0.0, 0.0)),
    ] {
        let (avant, apres, _) = jouer(m);
        for q in [(0.0, 0.0), (300.0, 200.0), (1100.0, 650.0)] {
            let monde = crate::canvas::screen_to_world(q.0, q.1, &avant);
            let (x, y) = appliquer(&m, q);
            let ici = crate::canvas::screen_to_world(x, y, &apres);
            assert!(
                (ici.0 - monde.0).abs() < 1e-6 && (ici.1 - monde.1).abs() < 1e-6,
                "{m:?} : le point montré en {q:?} doit l'être en ({x}, {y})"
            );
        }
    }
}

/// **Rien ne glisse après les doigts** : la similitude se montre en entier dans l'image.
#[test]
fn test_rien_ne_glisse_apres_les_doigts() {
    let (avant, apres, app) = jouer(mouvement(2.0, (-300.0, -200.0)));
    assert!((apres.scale / avant.scale - 2.0).abs() < 1e-9);
    assert!(!app.elan.en_cours(), "rien ne glisse après les doigts");
}

/// **Le pavé ne réveille la boucle que pendant un geste** : au repos, zéro image.
#[test]
fn test_le_pave_ne_reveille_que_pendant_un_geste() {
    let pave = Factice::default();
    let mut app = application(&pave);
    app.prochain_reveil();
    let au_repos = app.provenance.de_l_image();
    assert_eq!(au_repos & crate::app::reveil::Raison::Pave.bit(), 0);

    pave.0.borrow_mut().en_geste = true;
    assert!(
        app.prochain_reveil().is_some(),
        "en geste, la boucle repasse"
    );
    let en_geste = app.provenance.de_l_image();
    assert_ne!(en_geste & crate::app::reveil::Raison::Pave.bit(), 0);

    pave.0.borrow_mut().en_geste = false;
    app.prochain_reveil();
    assert_eq!(
        app.provenance.de_l_image() & crate::app::reveil::Raison::Pave.bit(),
        0,
        "le geste fini, le pavé se tait"
    );
}

/// **Hors geste, le système n'avance pas** : ce qu'il aurait encore à rendre attend — mais ses
/// signes de vie, eux, partent à chaque image.
#[test]
fn test_hors_geste_le_systeme_n_avance_pas_mais_ses_signes_partent() {
    let pave = Factice::default();
    let mut app = application(&pave);
    pave.0.borrow_mut().mouvements = vec![mouvement(1.0, (5.0, 0.0))];
    pave.0.borrow_mut().signes = vec![Signe {
        quand: Instant::now(),
        quoi: "contact",
        valeur: 0,
    }];
    app.suivre_le_pave();
    assert_eq!(
        pave.0.borrow().mouvements.len(),
        1,
        "rien n'a été demandé au système"
    );
    assert!(pave.0.borrow().signes.is_empty(), "les signes sont partis");
}

/// **À la fermeture, les signes qui attendent partent aussi** : ceux d'une coupure suivie
/// d'aucune image se seraient perdus.
#[test]
fn test_a_la_fermeture_les_signes_partent() {
    let pave = Factice::default();
    let mut app = application(&pave);
    pave.0.borrow_mut().signes = vec![Signe {
        quand: Instant::now(),
        quoi: "statut_suspendu",
        valeur: 3,
    }];
    app.vider_les_signes_du_pave();
    assert!(pave.0.borrow().signes.is_empty());
}

/// **Ce que le pavé rend se montre dans l'image même où il est lu** : l'image fait avancer le
/// système puis bouge la caméra, d'un seul appel. Quand le système avançait au réveil de la
/// boucle, une image recevait deux pas et la suivante aucun (fiche 53 § 8).
#[test]
fn test_le_pave_se_montre_dans_l_image_qui_le_lit() {
    let (avant, apres, app) = jouer(mouvement(1.0, (30.0, -12.0)));
    assert!(
        (apres.x - avant.x - 30.0).abs() < 1e-9 && (apres.y - avant.y + 12.0).abs() < 1e-9,
        "{avant:?} -> {apres:?}"
    );
    let reste = app.pave.as_ref().map(|p| p.en_geste());
    assert_eq!(reste, Some(true), "le geste continue, le système a été lu");
}

/// **Le point fixe du système se mesure, par rapport au curseur** : un pincement autour du
/// curseur (300, 200) le met à zéro pixel ; autour de l'origine, à `|(300, 200)|`, soit 361.
#[test]
fn test_le_point_fixe_se_mesure_par_rapport_au_curseur() {
    let (_, _, app) = jouer(mouvement(2.0, (-300.0, -200.0)));
    assert_eq!(app.chronique.navigation.points_fixes().centile(0.5), 0);
    let (_, _, app) = jouer(mouvement(2.0, (0.0, 0.0)));
    let ecart = app.chronique.navigation.points_fixes().centile(0.5);
    assert!(
        (ecart as f64 - 300f64.hypot(200.0)).abs() <= 361.0 * 0.05,
        "{ecart}"
    );
}

/// **Au mode référence, `Alt` + pincer agrandit la fenêtre** (REFERENCE-2) : l'octave du
/// pincement part à la fenêtre, et l'échelle du canevas ne bouge pas.
#[test]
fn test_alt_et_pincer_au_mode_reference_agrandit_la_fenetre() {
    let pave = Factice::default();
    let mut app = application(&pave);
    app.ui.reference = true;
    app.modifiers = winit::keyboard::ModifiersState::ALT;
    let avant = app.store.viewport();
    pave.0.borrow_mut().en_geste = true;
    pave.0.borrow_mut().mouvements = vec![mouvement(2.0, (-300.0, -200.0))];
    app.bouger_la_camera(1280, 720);
    assert!((app.fenetre_de_reference.pincement.en_attente() - 1.0).abs() < 1e-12);
    assert_eq!(
        app.store.viewport().scale,
        avant.scale,
        "le canevas garde son échelle"
    );
}
