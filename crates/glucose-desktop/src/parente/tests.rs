//! Les épreuves de PARENTE-2 : ce que l'IA dit des textes, par le vrai chemin — l'application, son
//! image, et le fil qui réveille la boucle.

use super::{besoin, suffit, Parente};
use crate::app::GlucoseApp;
use glucose_core::store::Store;
use glucose_core::types::Annotation;

/// Cinq textes de musique, quatre de l'eau, et des textes qui ne parlent de rien de commun — le
/// modèle les range ainsi (mesuré en Python sur le même modèle, fiche 106).
const MUSIQUE: [&str; 5] = [
    "Le piano est un instrument de musique à cordes frappées.",
    "Mozart a composé ses premières symphonies dans son enfance.",
    "Une partition note la mélodie, le rythme et l'harmonie d'un morceau.",
    "Le violon et le violoncelle jouent dans l'orchestre symphonique.",
    "La guitare électrique a transformé la musique rock.",
];
const EAU: [&str; 4] = [
    "L'eau bout à cent degrés au niveau de la mer.",
    "La glace est de l'eau à l'état solide, elle fond au-dessus de zéro degré.",
    "La vapeur d'eau se condense en gouttelettes dans les nuages.",
    "Les océans couvrent la plus grande partie de la surface de la Terre.",
];
const SEULS: [&str; 9] = [
    "Le chat dort sur le canapé tout l'après-midi.",
    "Il faut acheter du pain et des œufs demain matin.",
    "Le train pour Lyon part à huit heures.",
    "Le football se joue avec onze joueurs par équipe.",
    "La Révolution française commence en 1789.",
    "Le béton armé résiste mieux à la traction.",
    "Les abeilles produisent du miel à partir du nectar.",
    "Un algorithme de tri range une liste dans l'ordre.",
    "Le vaccin entraîne le système immunitaire.",
];
const AUTRES: [&str; 6] = [
    "La molécule d'eau est formée de deux atomes d'hydrogène et d'un atome d'oxygène.",
    "Les impôts financent les écoles et les hôpitaux publics.",
    "La photosynthèse transforme la lumière en énergie chimique.",
    "Les élections municipales ont lieu tous les six ans.",
    "Mon téléphone n'a plus de batterie.",
    "La tour Eiffel mesure plus de trois cents mètres.",
];

/// Une application sans fenêtre dont le tableau porte ces textes, nommés `m0…`, `e0…`, `s0…`,
/// `a0…`, en colonne — puis une image.
fn application() -> GlucoseApp {
    let mut app = GlucoseApp::new();
    app.ui.current_toast = None;
    if let Some(b) = app.store.active_board_mut() {
        b.annotations.clear();
    }
    let tableau = app.store.project.active_board_id.clone();
    let groupes: [(&str, &[&str]); 4] =
        [("m", &MUSIQUE), ("e", &EAU), ("s", &SEULS), ("a", &AUTRES)];
    let mut y = 0.0;
    for (prefixe, textes) in groupes {
        for (k, t) in textes.iter().enumerate() {
            let texte = Annotation::text(format!("{prefixe}{k}"), 0.0, y, *t);
            app.store.add_annotation(&tableau, texte);
            y += 200.0;
        }
    }
    app.store.clear_selection();
    app.une_image_sans_fenetre((800, 600));
    app
}

/// La lueur d'un texte de l'application : sa couleur et sa sûreté.
fn lueur(app: &GlucoseApp, id: &str) -> ((u8, u8, u8), f32) {
    let board = app.store.active_board().expect("un tableau");
    let ann = board.annotations.iter().find(|a| a.id() == id).expect(id);
    app.renderer.sens_des_textes.lueur(ann)
}

/// La teinte d'une couleur sur le cercle d'Oklch, et sa vivacité.
fn teinte_et_vivacite(rgb: (u8, u8, u8)) -> (f64, f64) {
    let c = glucose_core::couleur::Oklch::des_octets(rgb);
    (c.h, c.c)
}

/// L'écart de deux teintes, en radians.
fn ecart(a: f64, b: f64) -> f64 {
    let e = (a - b).rem_euclid(std::f64::consts::TAU);
    e.min(std::f64::consts::TAU - e)
}

/// Les teintes des textes sûrs (plus d'une chance sur deux) de ce groupe.
fn teintes_sures(app: &GlucoseApp, prefixe: &str, n: usize) -> Vec<f64> {
    (0..n)
        .map(|k| lueur(app, &format!("{prefixe}{k}")))
        .filter(|(_, surete)| *surete > 0.5)
        .map(|(rgb, _)| teinte_et_vivacite(rgb).0)
        .collect()
}

/// **PARENTE-2 — des textes parents partagent une teinte, et le reste est gris** : par le vrai
/// chemin, l'application et son image. Les textes sûrs de la musique ont une teinte, ceux de l'eau
/// une autre ; les textes sans parent ont le gris d'une membrane sans couleur. **La preuve à
/// l'envers** : avant PARENTE-2, chaque texte prenait la teinte de son voisinage sur le canevas —
/// posés en colonne, des voisins de sens différents se ressemblaient ; ici, deux familles voisines
/// dans la colonne ont deux teintes.
#[test]
fn parente_2_des_textes_parents_partagent_une_couleur() {
    let app = application();
    // Le gris d'un texte que l'IA ne connaît pas : celui d'une membrane sans couleur.
    let inconnu = Annotation::text("?", 0.0, 0.0, "");
    let gris = crate::renderer::sens::SensDesTextes::default().teinte(&inconnu);
    let (musique, eau) = (teintes_sures(&app, "m", 5), teintes_sures(&app, "e", 4));
    assert!(
        musique.len() >= 2,
        "deux textes de musique sûrs : {musique:?}"
    );
    assert!(eau.len() >= 2, "deux textes de l'eau sûrs : {eau:?}");
    for t in &musique {
        assert!(
            ecart(*t, musique[0]) < 0.05,
            "une seule teinte : {musique:?}"
        );
    }
    for t in &eau {
        assert!(ecart(*t, eau[0]) < 0.05, "une seule teinte : {eau:?}");
    }
    assert!(
        ecart(musique[0], eau[0]) > 0.3,
        "deux familles, deux teintes"
    );
    let vivacite = teinte_et_vivacite(lueur(&app, "e0").0).1;
    assert!(vivacite > 0.06, "sûre, elle est vive : {vivacite}");
    for k in 0..9 {
        let (couleur, surete) = lueur(&app, &format!("s{k}"));
        assert_eq!(couleur, gris, "le texte seul {k} est gris");
        assert!(surete < 0.5, "le texte seul {k} n'est pas sûr : {surete}");
    }
    let (_, sure) = lueur(&app, "e0");
    assert!(sure > 0.5, "« l'eau bout » a un parent sûr : {sure}");
}

/// **PARENTE-2 — la couleur choisie passe devant, sûre** : un texte de musique qu'on colore en rose
/// prend sa couleur, pleine ; ses parents gardent celle de leur famille.
#[test]
fn parente_2_la_couleur_choisie_passe_devant() {
    let mut app = application();
    if let Some(b) = app.store.active_board_mut() {
        for a in &mut b.annotations {
            if let Annotation::Text { id, color, .. } = a {
                if id == "m0" {
                    *color = Some("#f472b6".into());
                }
            }
        }
    }
    app.une_image_sans_fenetre((800, 600));
    assert_eq!(lueur(&app, "m0"), ((0xf4, 0x72, 0xb6), 1.0));
    assert_ne!(
        lueur(&app, "m1").0,
        (0xf4, 0x72, 0xb6),
        "ses parents gardent leur famille"
    );
}

/// **PARENTE-2 — déplacer un nœud ne relance rien ; écrire relance** : l'empreinte des textes ne
/// bouge pas quand la version du document avance pour un déplacement. **La preuve à l'envers** :
/// changer un texte la fait bouger.
#[test]
fn parente_2_deplacer_ne_relance_rien_ecrire_relance() {
    let mut store = Store::new("p");
    let tableau = store.project.active_board_id.clone();
    store.add_annotation(&tableau, Annotation::text("a", 0.0, 0.0, MUSIQUE[0]));
    store.add_annotation(&tableau, Annotation::text("b", 0.0, 200.0, MUSIQUE[1]));
    let mut parente = Parente::default();
    assert!(parente.suivre(&store).is_some(), "la première lecture");
    assert!(parente.suivre(&store).is_none(), "rien n'a changé");
    store.bump_version();
    if let Some(Annotation::Text { x, .. }) = store
        .active_board_mut()
        .and_then(|b| b.annotations.first_mut())
    {
        *x += 500.0;
    }
    assert!(
        parente.suivre(&store).is_none(),
        "un déplacement ne relance rien"
    );
    store.bump_version();
    if let Some(Annotation::Text { text, .. }) = store
        .active_board_mut()
        .and_then(|b| b.annotations.first_mut())
    {
        *text = EAU[0].into();
    }
    assert!(parente.suivre(&store).is_some(), "un texte écrit relance");
}

/// **PARENTE-2 — le fil rend l'avis et réveille la boucle** : au vrai lancement, rien ne se calcule
/// sur place ; le fil réveille, et l'image suivante reçoit l'avis.
#[test]
fn parente_2_le_fil_rend_l_avis_et_reveille() {
    let (reveil, reveille) = std::sync::mpsc::channel();
    let reveil = std::sync::Mutex::new(reveil);
    let mut parente = Parente::default();
    parente.demarrer(std::sync::Arc::new(move || {
        let _ = reveil.lock().map(|r| r.send(()));
    }));
    let mut store = Store::new("p");
    let tableau = store.project.active_board_id.clone();
    for (k, t) in MUSIQUE.iter().chain(&SEULS).enumerate() {
        store.add_annotation(&tableau, Annotation::text(format!("t{k}"), 0.0, 0.0, *t));
    }
    assert!(parente.suivre(&store).is_none(), "le fil calcule ailleurs");
    reveille
        .recv_timeout(std::time::Duration::from_secs(30))
        .expect("le fil réveille la boucle");
    let avis = parente.suivre(&store).expect("l'avis est là");
    assert_eq!(avis.len(), 14);
}

/// **PARENTE-2 — la règle de la mémoire** : la parenté ne prend jamais plus que la part d'un cache ;
/// sans borne connue, elle calcule.
#[test]
fn parente_2_la_regle_de_la_memoire() {
    assert_eq!(besoin(1000), 6_000_000);
    assert!(suffit(besoin(1000), Some(6_000_000)));
    assert!(!suffit(besoin(1000), Some(5_999_999)));
    assert!(suffit(besoin(1_000_000), None));
}
