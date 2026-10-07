//! Le pavé par *Direct Manipulation* (fiche 53) : la lecture de la transformation, et ce que
//! Glucose en montre — par la vraie boucle (`suivre_le_pave`, `prochain_reveil`), une source
//! factice jouant le système.

use super::*;
use std::cell::RefCell;
use std::rc::Rc;

/// Une suite de transformations telles que le système les donne, lue d'un bout à l'autre.
fn lire_tout(transformations: &[(f32, f32, f32)]) -> Vec<Mouvement> {
    let mut lecteur = Lecteur::default();
    transformations
        .iter()
        .flat_map(|t| lecteur.lire(*t))
        .collect()
}

/// **Un déplacement rend ce qui a bougé depuis l'image précédente**, et rien quand rien n'a bougé.
#[test]
fn test_un_deplacement_rend_ses_ecarts() {
    let m = lire_tout(&[(1.0, 3.0, -2.0), (1.0, 3.0, -2.0), (1.0, 10.5, -2.0)]);
    assert_eq!(
        m,
        vec![
            Mouvement::Deplacer(3.0, -2.0),
            Mouvement::Deplacer(7.5, 0.0)
        ]
    );
}

/// **Un pincement rend, au total, exactement l'échelle du geste** : la somme des octaves vaut
/// le logarithme de l'échelle finale — aucune perte en route, ce qui manquait aux paquets de
/// molette.
#[test]
fn test_un_pincement_rend_exactement_son_echelle() {
    let echelles = [1.0f32, 1.03, 1.11, 1.25, 1.6, 2.0, 1.9];
    let t: Vec<_> = echelles.iter().map(|s| (*s, 0.0, 0.0)).collect();
    let total: f64 = lire_tout(&t)
        .iter()
        .map(|m| match m {
            Mouvement::Zoomer(o) => *o,
            Mouvement::Deplacer(..) | Mouvement::Bascule(_) => {
                panic!("un pincement pur ne déplace pas, et ne bascule pas")
            }
        })
        .sum();
    assert!((total - f64::from(1.9f32).log2()).abs() < 1e-9, "{total}");
}

/// **Pendant un pincement, le décalage n'est pas un geste** : le système déplace le contenu pour
/// garder le point entre les doigts, et le lire ferait glisser la vue sous le zoom (Blender :
/// « absurde »). Mais un déplacement qui **devient** un pincement est lu des deux façons.
#[test]
fn test_un_pincement_ne_deplace_plus_jusqu_a_la_fin_du_geste() {
    let m = lire_tout(&[(1.0, 4.0, 0.0), (1.2, 40.0, 30.0), (1.2, 55.0, 31.0)]);
    assert_eq!(m.len(), 3, "{m:?}");
    assert_eq!(m[0], Mouvement::Deplacer(4.0, 0.0));
    assert!(matches!(m[2], Mouvement::Zoomer(o) if o > 0.0));
}

/// **La bascule d'un déplacement en pincement se mesure**, à son écart d'échelle — et un
/// pincement d'emblée ne bascule pas (fiche 53 § 8).
#[test]
fn test_la_bascule_se_mesure() {
    let m = lire_tout(&[(1.0, 4.0, 0.0), (1.2, 40.0, 30.0)]);
    assert!(
        matches!(m[1], Mouvement::Bascule(e) if (e - 0.2).abs() < 1e-6),
        "{m:?}"
    );
    let d_emblee = lire_tout(&[(1.2, 0.0, 0.0), (1.5, 0.0, 0.0)]);
    assert!(
        d_emblee.iter().all(|m| !matches!(m, Mouvement::Bascule(_))),
        "{d_emblee:?}"
    );
}

/// **La remise repart de l'identité** : le geste suivant ne part pas de l'échelle où celui-ci
/// s'est arrêté, et un pincement fini ne bloque plus les déplacements.
#[test]
fn test_la_remise_repart_de_l_identite() {
    let mut lecteur = Lecteur::default();
    lecteur.lire((2.0, 0.0, 0.0));
    lecteur.remettre();
    assert_eq!(lecteur, Lecteur::default());
    assert_eq!(
        lecteur.lire((1.0, 0.0, 0.0)),
        Vec::new(),
        "l'identité ne bouge rien"
    );
    assert_eq!(
        lecteur.lire((1.0, 5.0, 0.0)),
        vec![Mouvement::Deplacer(5.0, 0.0)]
    );
}

/// **L'arrondi d'un `f32` n'est pas un pincement** — mais le plus petit vrai écart en est un.
#[test]
fn test_l_arrondi_n_est_pas_un_pincement() {
    let presque = f32::from_bits(1.0f32.to_bits() - 1);
    assert_eq!(
        lire_tout(&[(presque, 1.0, 0.0)]),
        vec![Mouvement::Deplacer(1.0, 0.0)]
    );
    let m = lire_tout(&[(1.001, 0.0, 0.0)]);
    assert!(matches!(m[..], [Mouvement::Zoomer(_)]), "{m:?}");
}

/// Le système, joué : des mouvements à rendre, et un geste en cours ou non.
#[derive(Clone, Default)]
struct Factice(Rc<RefCell<(bool, Vec<Mouvement>)>>);

impl Pave for Factice {
    fn en_geste(&self) -> bool {
        self.0.borrow().0
    }
    fn avancer(&mut self) -> Vec<Mouvement> {
        std::mem::take(&mut self.0.borrow_mut().1)
    }
}

fn application(pave: &Factice) -> GlucoseApp {
    let mut app = GlucoseApp::new();
    app.pave = Some(Box::new(pave.clone()));
    app.mouse_pos = (300.0, 200.0);
    app.curseur_vu = true;
    app
}

/// **Un pincement se montre en entier à l'image suivante, autour du curseur** : le système l'a
/// déjà lissé, rien ne glisse après les doigts.
#[test]
fn test_un_pincement_se_montre_en_entier_autour_du_curseur() {
    let pave = Factice::default();
    let mut app = application(&pave);
    let avant = app.store.viewport();
    let monde_sous_le_curseur = crate::canvas::screen_to_world(300.0, 200.0, &avant);

    *pave.0.borrow_mut() = (true, vec![Mouvement::Zoomer(1.0)]);
    app.suivre_le_pave();
    app.appliquer_l_elan(1280, 720);

    let apres = app.store.viewport();
    assert!(
        (apres.scale / avant.scale - 2.0).abs() < 1e-9,
        "une octave : {apres:?}"
    );
    let sous_le_curseur = crate::canvas::screen_to_world(300.0, 200.0, &apres);
    assert!(
        (sous_le_curseur.0 - monde_sous_le_curseur.0).abs() < 1e-6
            && (sous_le_curseur.1 - monde_sous_le_curseur.1).abs() < 1e-6,
        "le point sous le curseur reste sous le curseur"
    );
    assert!(!app.elan.en_cours(), "rien ne glisse après les doigts");
}

/// **Un déplacement suit les doigts, à l'image suivante.**
#[test]
fn test_un_deplacement_suit_les_doigts() {
    let pave = Factice::default();
    let mut app = application(&pave);
    let avant = app.store.viewport();
    *pave.0.borrow_mut() = (true, vec![Mouvement::Deplacer(30.0, -12.0)]);
    app.suivre_le_pave();
    app.appliquer_l_elan(1280, 720);
    let apres = app.store.viewport();
    let (a, b) = (
        crate::canvas::screen_to_world(0.0, 0.0, &avant),
        crate::canvas::screen_to_world(30.0, -12.0, &apres),
    );
    assert!(
        (a.0 - b.0).abs() < 1e-6 && (a.1 - b.1).abs() < 1e-6,
        "{avant:?} {apres:?}"
    );
}

/// **Le pavé ne réveille la boucle que pendant un geste** : au repos, zéro image.
#[test]
fn test_le_pave_ne_reveille_que_pendant_un_geste() {
    let pave = Factice::default();
    let mut app = application(&pave);
    app.prochain_reveil();
    let au_repos = app.provenance.de_l_image();
    assert_eq!(au_repos & crate::app::reveil::Raison::Pave.bit(), 0);

    pave.0.borrow_mut().0 = true;
    assert!(
        app.prochain_reveil().is_some(),
        "en geste, la boucle repasse"
    );
    let en_geste = app.provenance.de_l_image();
    assert_ne!(en_geste & crate::app::reveil::Raison::Pave.bit(), 0);

    pave.0.borrow_mut().0 = false;
    app.prochain_reveil();
    assert_eq!(
        app.provenance.de_l_image() & crate::app::reveil::Raison::Pave.bit(),
        0,
        "le geste fini, le pavé se tait"
    );
}

/// **Hors geste, le système n'avance pas** : ce qu'il aurait encore à rendre attend.
#[test]
fn test_hors_geste_le_systeme_n_avance_pas() {
    let pave = Factice::default();
    let mut app = application(&pave);
    *pave.0.borrow_mut() = (false, vec![Mouvement::Deplacer(5.0, 0.0)]);
    app.suivre_le_pave();
    assert_eq!(
        pave.0.borrow().1.len(),
        1,
        "rien n'a été demandé au système"
    );
}

/// **Ce que le pavé rend se montre dans l'image même où il est lu** : l'image fait avancer le
/// système puis bouge la caméra, d'un seul appel. Quand le système avançait au réveil de la
/// boucle, une image recevait deux pas et la suivante aucun (fiche 53 § 8).
#[test]
fn test_le_pave_se_montre_dans_l_image_qui_le_lit() {
    let pave = Factice::default();
    let mut app = application(&pave);
    let avant = app.store.viewport();
    *pave.0.borrow_mut() = (true, vec![Mouvement::Deplacer(30.0, -12.0)]);
    app.bouger_la_camera(1280, 720);
    let apres = app.store.viewport();
    assert!(
        (apres.x - avant.x - 30.0).abs() < 1e-9 && (apres.y - avant.y + 12.0).abs() < 1e-9,
        "{avant:?} -> {apres:?}"
    );
    assert!(
        pave.0.borrow().1.is_empty(),
        "le système a été lu dans cette image"
    );
}
