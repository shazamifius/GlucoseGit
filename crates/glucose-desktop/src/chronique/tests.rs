//! Ce que la chronique promet : une distribution juste, une mémoire bornée, aucun contenu.

use super::*;

fn image(geste: Geste, duree_us: u32) -> Instantane {
    Instantane {
        duree_us,
        geste: Geste::TOUS.iter().position(|g| *g == geste).unwrap_or(0) as u8,
        ..Default::default()
    }
}

#[test]
fn test_les_centiles_ne_se_laissent_pas_tromper_par_la_moyenne() {
    // Le défaut que ce module existe pour corriger : cent images à 2 ms et une à 200 ms
    // donnent une moyenne de 4 ms — excellente — alors que l'utilisateur a vu un gel.
    let mut c = Chronique::nouvelle();
    for _ in 0..100 {
        c.enregistrer(image(Geste::DeplacerLaVue, 2_000));
    }
    c.enregistrer(image(Geste::DeplacerLaVue, 200_000));

    let moyenne = c.moyenne_du_geste(Geste::DeplacerLaVue);
    assert!(
        moyenne < 6_000,
        "la moyenne reste basse ({moyenne} us) et c'est bien le probleme"
    );
    // Le pire, lui, dit la vérité — et il est gardé exact, pas approché.
    assert_eq!(c.pire_du_geste(Geste::DeplacerLaVue), 200_000);
    assert!(
        c.centile_du_geste(Geste::DeplacerLaVue, 0.50) < 3_000,
        "la mediane reflete la masse"
    );
}

#[test]
fn test_la_memoire_ne_grandit_pas_avec_la_session() {
    // Une session de plusieurs heures ne doit rien accumuler : c'est la condition pour
    // enregistrer en permanence chez l'utilisateur.
    let mut c = Chronique::nouvelle();
    for i in 0..10_000u32 {
        c.enregistrer(image(Geste::Repos, 1_000 + i % 500));
    }
    assert_eq!(c.rendues(), 10_000);
    assert!(
        c.pires().len() <= PIRES,
        "les pires sont bornees a {PIRES}, or il y en a {}",
        c.pires().len()
    );
}

#[test]
fn test_les_pires_sont_les_pires_et_dans_l_ordre() {
    let mut c = Chronique::nouvelle();
    for us in [5_000u32, 80_000, 1_000, 200_000, 12_000] {
        c.enregistrer(image(Geste::Zoomer, us));
    }
    let durees: Vec<u32> = c.pires().iter().map(|p| p.duree_us).collect();
    assert_eq!(durees, vec![200_000, 80_000, 12_000, 5_000, 1_000]);
}

#[test]
fn test_une_image_rapide_ne_deloge_pas_une_lente() {
    let mut c = Chronique::nouvelle();
    for _ in 0..PIRES {
        c.enregistrer(image(Geste::Repos, 50_000));
    }
    c.enregistrer(image(Geste::Repos, 10));
    assert_eq!(c.pires().len(), PIRES);
    assert!(
        c.pires().iter().all(|p| p.duree_us == 50_000),
        "une image de 10 us n'a rien a faire parmi les pires"
    );
}

#[test]
fn test_chaque_geste_a_sa_propre_distribution() {
    // Confondre les gestes a deja fait optimiser dans le vide : un deplacement de vue lent et
    // un zoom lent ne se corrigent pas au meme endroit.
    let mut c = Chronique::nouvelle();
    for _ in 0..50 {
        c.enregistrer(image(Geste::DeplacerLaVue, 1_000));
    }
    for _ in 0..50 {
        c.enregistrer(image(Geste::Zoomer, 40_000));
    }
    assert_eq!(c.rendues_du_geste(Geste::DeplacerLaVue), 50);
    assert_eq!(c.rendues_du_geste(Geste::Zoomer), 50);
    assert!(c.centile_du_geste(Geste::DeplacerLaVue, 0.99) < 2_000);
    assert!(c.centile_du_geste(Geste::Zoomer, 0.50) > 30_000);
    assert_eq!(c.rendues_du_geste(Geste::Dessiner), 0);
}

#[test]
fn test_les_postes_se_nomment_une_fois_et_gardent_leur_indice() {
    let mut c = Chronique::nouvelle();
    let a = c.poste("images").expect("un poste neuf");
    let b = c.poste("annotations").expect("un second poste");
    assert_ne!(a, b);
    assert_eq!(
        c.poste("images"),
        Some(a),
        "un nom deja vu garde son indice"
    );
    assert_eq!(c.nom_du_poste(a), Some("images"));
}

#[test]
fn test_le_nombre_de_postes_est_borne() {
    // La borne existe pour que l'enregistrement d'une image reste de taille fixe.
    let mut c = Chronique::nouvelle();
    // Plus de noms que la borne n'en admet, quelle qu'elle soit : le test ne doit pas avoir
    // à changer quand elle change.
    let noms: Vec<&'static str> = (0..POSTES + 10)
        .map(|i| Box::leak(format!("poste-{i}").into_boxed_str()) as &'static str)
        .collect();
    let acceptes = noms.iter().filter(|n| c.poste(n).is_some()).count();
    assert_eq!(acceptes, POSTES, "au-dela de {POSTES}, un poste est refuse");
}

#[test]
fn test_ou_va_le_temps_d_un_geste() {
    let mut c = Chronique::nouvelle();
    let images = c.poste("images").expect("poste");
    let ui = c.poste("ui").expect("poste");
    let mut vu = image(Geste::GlisserUnNoeud, 10_000);
    vu.postes_us[images] = 8_000;
    vu.postes_us[ui] = 1_500;
    c.enregistrer(vu);

    let parts = c.parts_du_geste(Geste::GlisserUnNoeud).expect("des parts");
    assert!(parts.contains(&("images", 8_000)));
    assert!(parts.contains(&("ui", 1_500)));
    assert!(c.parts_du_geste(Geste::Dessiner).is_none());
}

#[test]
fn test_le_rapport_dit_ce_qu_il_faut_sans_rien_reveler() {
    let mut c = Chronique::nouvelle();
    let images = c.poste("images").expect("poste");
    for _ in 0..30 {
        let mut vu = image(Geste::GlisserUnNoeud, 40_000);
        vu.postes_us[images] = 35_000;
        vu.noeuds = 36;
        vu.photos = 36;
        vu.region_px = 100_000;
        vu.fenetre_px = 1_000_000;
        c.enregistrer(vu);
    }
    let r = c.rapport();
    assert!(r.contains("glisser un noeud"), "le geste doit paraitre");
    assert!(r.contains("images"), "le poste dominant doit paraitre");
    assert!(r.contains("p99"), "la distribution doit paraitre");
    // Rien d'autre ne peut y figurer : `Instantane` ne porte que des entiers, donc aucun
    // chemin, aucun nom de fichier, aucun texte de carte ne peut traverser ce chemin.
    assert!(!r.contains(".png") && !r.contains("C:\\"));
}

#[test]
fn test_une_part_redessinee_se_lit_meme_sans_fenetre() {
    // Une division par zero ici passerait inapercue et produirait un NaN dans le rapport.
    let vu = Instantane::default();
    assert_eq!(vu.part_redessinee(), 1.0);
}
