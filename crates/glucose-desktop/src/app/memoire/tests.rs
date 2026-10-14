//! MEMOIRE-1 et 2, éprouvés par l'application elle-même : l'alerte du système passe par le
//! même chemin que celui que `winit` appelle (`memory_warning`, qui ne fait que déléguer), et ce
//! qui en sort se lit dans la boîte noire et dans le magasin.

use super::*;
use crate::present::{Issue, Presenter};
use crate::renderer::magasin::Entree;
use crate::renderer::photo::Pyramide;
use std::num::NonZeroU32;
use std::sync::{Arc, Mutex};

/// Une présentation qui tient des octets sur la carte, et retient chaque demande de les rendre
/// (vrai : derrière une autre application).
struct Carte {
    demandes: Arc<Mutex<Vec<bool>>>,
    surface: bool,
    octets: u64,
}

impl Presenter for Carte {
    fn resize(&mut self, _: NonZeroU32, _: NonZeroU32) -> crate::error::DesktopResult<()> {
        Ok(())
    }
    fn present(&mut self, _: &tiny_skia::Pixmap) -> crate::error::DesktopResult<Issue> {
        Ok(Issue::Presentee)
    }
    fn lacher_la_surface(&mut self) {
        self.surface = false;
    }
    fn a_sa_surface(&self) -> bool {
        self.surface
    }
    fn nom(&self) -> &'static str {
        "carte d'epreuve"
    }
    fn octets_sur_la_carte(&self) -> Option<u64> {
        Some(self.octets)
    }
    fn rendre_la_memoire(&mut self, arriere_plan: bool) {
        self.demandes.lock().expect("demandes").push(arriere_plan);
    }
}

/// Une application dont la boîte noire est ouverte dans le dossier de l'épreuve, et qui tient
/// trois mébioctets sur sa carte.
fn application(d: &crate::epreuve::Dossier) -> (GlucoseApp, Arc<Mutex<Vec<bool>>>) {
    let mut app = GlucoseApp::new();
    app.chronique.ouvrir_la_boite_noire(d);
    let demandes = Arc::new(Mutex::new(Vec::new()));
    app.presenter = Some(Box::new(Carte {
        demandes: demandes.clone(),
        surface: true,
        octets: 3 << 20,
    }));
    (app, demandes)
}

fn session(app: &GlucoseApp) -> String {
    let b = app.chronique.boite_noire().expect("la boîte noire ouverte");
    b.synchroniser();
    std::fs::read_to_string(b.chemin()).expect("la session")
}

/// **Au premier plan, la carte garde ce que l'écran pose**, et l'alerte s'écrit — suivie de
/// ce que Glucose tenait à cet instant, la carte comprise, **même sans record** : des records
/// imbattables sont posés d'abord, et seule une ligne forcée peut encore paraître.
#[test]
fn test_memoire_2_au_premier_plan_la_carte_garde_l_ecran_et_l_alerte_s_ecrit() {
    let d = crate::epreuve::Dossier::nouveau("memoire-premier-plan");
    let (mut app, demandes) = application(&d);
    let imbattable = Occupation {
        processus_mo: Some(u64::MAX),
        disponible_mo: Some(0),
        ..Occupation::default()
    };
    app.chronique.noter_la_memoire(imbattable, false);
    let avant = session(&app);
    app.rendre_la_memoire();
    assert_eq!(*demandes.lock().expect("demandes"), vec![false]);
    let texte = session(&app)[avant.len()..].to_string();
    let alerte = texte
        .find("{\"type\":\"alerte_memoire\"")
        .expect("l'alerte est écrite");
    assert!(
        texte[alerte..].contains("\"arriere_plan\":false"),
        "{texte}"
    );
    let memoire = texte
        .find("{\"type\":\"memoire\"")
        .expect("ce que Glucose tenait est écrit");
    assert!(alerte < memoire, "l'alerte d'abord : {texte}");
    assert!(texte[memoire..].contains("\"carte_mo\":3"), "{texte}");
}

/// **Derrière une autre application, tout se rend** : la carte entière, et le magasin offre au
/// système même ce que l'écran montrait (ETAGES-1, MEMOIRE-2).
#[test]
#[cfg_attr(
    not(any(windows, target_os = "linux", target_os = "android")),
    ignore = "l'offre de mémoire au système n'existe pas encore sur ce système (fiche 44, phase 8)"
)]
fn test_memoire_2_en_arriere_plan_tout_se_rend() {
    let d = crate::epreuve::Dossier::nouveau("memoire-arriere-plan");
    let (mut app, demandes) = application(&d);
    let photo = tiny_skia::Pixmap::new(512, 384).expect("une photo");
    app.renderer
        .magasin
        .cache
        .insert("photo".into(), Entree::pour_test(Pyramide::nouvelle(photo)));
    let tenus = app.renderer.magasin.octets_en(Etat::Tenu);
    assert!(tenus > 0, "la photo est tenue");

    app.suspendre();
    app.rendre_la_memoire();
    app.renderer.magasin.attendre_le_chantier();

    assert_eq!(*demandes.lock().expect("demandes"), vec![true]);
    assert!(
        app.renderer.magasin.octets_en(Etat::Offert) > 0,
        "le magasin a offert ce qu'il tenait"
    );
    assert!(app.renderer.magasin.octets_en(Etat::Tenu) < tenus);
    assert!(session(&app).contains("\"arriere_plan\":true"));
}
