//! Ce que la sonde de la carte promet, sur la machine qui fait tourner les tests.

use super::Sonde;

/// **Sous Windows, la sonde retrouve la carte que wgpu a ouverte, et en lit un budget.**
///
/// Par ses identifiants matériels, les mêmes pour Vulkan et pour le gestionnaire de mémoire
/// vidéo de Windows : c'est ce qui relie deux mondes qui ne se connaissent pas. Sur une
/// machine sans carte utilisable, l'épreuve se saute ; ailleurs que sous Windows, la sonde ne
/// sait rien, et doit le dire.
#[test]
fn test_la_sonde_lit_le_budget_de_la_carte_ouverte() {
    let instance = wgpu::Instance::default();
    let Ok(adaptateur) =
        pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))
    else {
        eprintln!("aucune carte graphique : epreuve sautee");
        return;
    };
    let info = adaptateur.get_info();
    let sonde = Sonde::pour(info.vendor, info.device);
    if cfg!(windows) {
        let sonde = sonde.expect("Windows connait la carte que wgpu a ouverte");
        let vu = sonde.lire().expect("et il dit son budget");
        assert!(vu.budget > 0, "un budget nul ne serait pas un budget");
    } else {
        assert!(sonde.is_none(), "ailleurs, la sonde ne devine pas");
    }
}

/// **Le veilleur réveille la boucle quand le budget change, une fois par changement**, et son
/// fil s'arrête avec la sonde (ETAGES-2).
///
/// L'épreuve ne peut pas faire changer le vrai budget sans occuper la carte : elle signale
/// l'événement que Windows signalerait, et regarde ce que le fil en fait.
#[cfg(windows)]
#[test]
fn test_le_veilleur_reveille_la_boucle_quand_le_budget_change() {
    let instance = wgpu::Instance::default();
    let Ok(adaptateur) =
        pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))
    else {
        eprintln!("aucune carte graphique : epreuve sautee");
        return;
    };
    let info = adaptateur.get_info();
    let mut sonde = Sonde::pour(info.vendor, info.device).expect("Windows connait la carte");
    let (reveille, reveils) = std::sync::mpsc::channel();
    sonde.veiller(Box::new(move || reveille.send(()).is_ok()));

    sonde.signaler();
    reveils
        .recv_timeout(std::time::Duration::from_secs(5))
        .expect("le fil doit reveiller la boucle");
    assert!(sonde.a_change(), "et dire que le budget a change");
    assert!(!sonde.a_change(), "une seule fois par changement");
    drop(sonde);
}
