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
