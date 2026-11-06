use super::*;
use crate::epreuve::Dossier;

const VERSION: &str = "2.0.3-beta.7";

#[test]
fn test_moteur_1_sans_fichier_tout_est_permis() {
    let l = Echelle::lire("", VERSION);
    assert!(l.echelle.permet("vulkan"));
    assert_eq!(l.vient_de_tomber, None);
    assert!(l.a_sonder.is_empty());
}

#[test]
fn test_moteur_1_un_nom_encore_ecrit_fait_tomber_ce_moteur() {
    let l = Echelle::lire("version=2.0.3-beta.7\nouverture=vulkan\n", VERSION);
    assert_eq!(l.vient_de_tomber.as_deref(), Some("vulkan"));
    assert!(!l.echelle.permet("vulkan"));
    assert!(l.echelle.permet("dx12"));
    assert!(
        l.a_sonder.is_empty(),
        "la même version ne repose pas la question"
    );
    assert!(
        !l.echelle.texte().contains("ouverture="),
        "la chute n'est dite qu'une fois"
    );
}

#[test]
fn test_moteur_1_une_session_survivante_ne_fait_tomber_personne() {
    let l = Echelle::lire("version=2.0.3-beta.7\n", VERSION);
    assert_eq!(l.vient_de_tomber, None);
    assert!(l.echelle.permet("vulkan"));
}

#[test]
fn test_moteur_1_l_echelle_descend_barreau_par_barreau() {
    let a = Echelle::lire(
        "version=2.0.3-beta.7\ntombe=vulkan\nouverture=dx12\n",
        VERSION,
    );
    assert!(
        !a.echelle.permet("vulkan"),
        "le barreau d'avant reste écarté"
    );
    assert!(!a.echelle.permet("dx12"));
    assert!(a.echelle.permet("gl"));
}

#[test]
fn test_moteur_1_une_nouvelle_version_sonde_sans_rouvrir() {
    let l = Echelle::lire("version=2.0.3-beta.6\ntombe=vulkan\n", VERSION);
    assert_eq!(l.a_sonder, ["vulkan"]);
    assert!(
        !l.echelle.permet("vulkan"),
        "tant que la sonde n'a pas survécu, ce lancement ne l'ouvre pas"
    );
    assert!(l.echelle.texte().starts_with("version=2.0.3-beta.7\n"));
}

#[test]
fn test_moteur_1_la_sonde_qui_survit_rend_le_moteur() {
    let mut l = Echelle::lire("version=2.0.3-beta.6\ntombe=vulkan\n", VERSION).echelle;
    l.oublier("vulkan");
    assert!(l.permet("vulkan"));
    assert!(!l.texte().contains("tombe="));
}

#[test]
fn test_moteur_1_un_fichier_illisible_n_est_pas_un_souvenir() {
    let l = Echelle::lire("\u{0}n'importe quoi\n=\ntombe=\n", VERSION);
    assert!(l.echelle.permet(""));
    assert_eq!(l.vient_de_tomber, None);
}

#[test]
fn test_moteur_1_ce_qui_est_ecrit_survit_a_l_ouverture_et_s_efface_a_la_premiere_image() {
    let dossier = Dossier::nouveau("echelle");
    let (mut ici, tombe, _) = Habitee::ouvrir(&dossier, VERSION);
    assert_eq!(tombe, None);
    ici.j_ouvre("vulkan");
    let (_, tombe, _) = Habitee::ouvrir(&dossier, VERSION);
    assert_eq!(
        tombe.as_deref(),
        Some("vulkan"),
        "aucune première image : il est tombé"
    );

    let dossier = Dossier::nouveau("echelle-survit");
    let (mut ici, _, _) = Habitee::ouvrir(&dossier, VERSION);
    ici.j_ouvre("vulkan");
    ici.survecu();
    let (suivant, tombe, _) = Habitee::ouvrir(&dossier, VERSION);
    assert_eq!(tombe, None, "la première image a effacé le nom");
    assert!(suivant.echelle.permet("vulkan"));
}

#[test]
fn test_moteur_1_les_epreuves_n_ont_aucune_echelle() {
    // `ICI` n'est posé que par le vrai lancement : tous les moteurs restent permis, et rien
    // ne s'écrit chez la personne qui lance les épreuves.
    assert!(ICI.get().is_none());
    assert_eq!(barreaux().len(), moteurs::ici().count());
}

/// **L'ordre dans le code est toute la garantie** : le nom s'écrit avant que le pilote se charge,
/// et aucune épreuve sans fenêtre ne peut ouvrir ce chemin.
#[test]
fn test_moteur_1_le_nom_s_ecrit_avant_l_instance() {
    let source = include_str!("../gpu/ouverture.rs");
    let ecrit = source
        .find("echelle::j_ouvre(nom)")
        .expect("le nom s'écrit");
    let instance = source
        .find("moteurs::instance_pour_la_fenetre(")
        .expect("l'instance");
    assert!(
        ecrit < instance,
        "le nom doit s'écrire avant d'atteindre le pilote"
    );
    assert!(
        source.contains("echelle::barreaux()"),
        "et l'échelle choisit les moteurs"
    );
}

#[test]
fn test_moteur_1_une_image_presentee_et_une_fin_propre_effacent_le_nom() {
    let presentation = include_str!("../../app/presentation.rs");
    assert!(presentation.contains("echelle::survecu()"));
    let fin = include_str!("../../app/fin.rs");
    assert!(fin.contains("echelle::survecu()"));
    let demarrage = include_str!("../../demarrage.rs");
    assert!(demarrage.contains("echelle::habiter("));
}

/// **Un vrai plantage** : ce processus d'épreuve meurt à l'ouverture, et le suivant le sait.
#[test]
fn test_moteur_1_un_processus_mort_a_l_ouverture_fait_tomber_le_moteur() {
    let dossier = Dossier::nouveau("echelle-plantage");
    let statut = std::process::Command::new(std::env::current_exe().expect("l'épreuve"))
        .args(["--exact", "present::echelle::tests::tomber_ici"])
        .args(["--ignored", "--test-threads=1", "--nocapture"])
        .env(TOMBER, "vulkan")
        .env("GLUCOSE_ECHELLE_D_EPREUVE", &*dossier)
        .stderr(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .status()
        .expect("le processus qui va tomber");
    assert!(!statut.success(), "il devait tomber");
    let (suivant, tombe, _) = Habitee::ouvrir(&dossier, VERSION);
    assert_eq!(tombe.as_deref(), Some("vulkan"));
    assert!(!suivant.echelle.permet("vulkan"));
}

#[test]
#[ignore = "lancé par test_moteur_1_un_processus_mort_a_l_ouverture_fait_tomber_le_moteur"]
fn tomber_ici() {
    let Some(dossier) = std::env::var_os("GLUCOSE_ECHELLE_D_EPREUVE") else {
        return;
    };
    let (mut ici, _, _) = Habitee::ouvrir(Path::new(&dossier), VERSION);
    ici.j_ouvre("vulkan");
    tomber_si_demande("vulkan");
}
