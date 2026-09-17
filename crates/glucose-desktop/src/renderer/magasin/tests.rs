//! Ce que le magasin promet : ne jamais bloquer, ne jamais redemander en vain.

use super::*;

fn photo_temoin(nom: &str) -> String {
    let dossier = std::env::temp_dir().join("glucose-magasin-tests");
    std::fs::create_dir_all(&dossier).expect("dossier de test");
    let chemin = dossier.join(nom);
    if !chemin.exists() {
        let mut brute = image::RgbaImage::new(48, 32);
        for (x, _, px) in brute.enumerate_pixels_mut() {
            *px = image::Rgba([(x * 5 % 256) as u8, 90, 160, 255]);
        }
        brute.save(&chemin).expect("écriture de la photo témoin");
    }
    chemin.to_string_lossy().to_string()
}

/// Fait tourner le magasin comme le rendu le fait — récolter, puis demander — jusqu'à ce que
/// l'image soit là, sans jamais boucler indéfiniment.
fn attendre(magasin: &mut Magasin, src: &str) -> bool {
    let depart = std::time::Instant::now();
    while depart.elapsed() < std::time::Duration::from_secs(30) {
        magasin.recolter();
        if magasin.pyramide(src).is_some() {
            return true;
        }
        std::thread::yield_now();
    }
    false
}

#[test]
fn test_la_premiere_demande_ne_bloque_pas_et_rend_rien() {
    let src = photo_temoin("magasin-premiere.png");
    let mut magasin = Magasin::nouveau();

    // C'est tout l'invariant DECODE-1 : la première demande revient immédiatement, les mains
    // vides. Une mesure de durée serait fragile ; ce qui se vérifie, c'est qu'aucune image
    // n'est rendue alors qu'aucun décodage n'a encore pu aboutir.
    assert!(
        magasin.pyramide(&src).is_none(),
        "le rendu n'attend pas le décodage"
    );
    assert_eq!(magasin.en_travail(), 1, "mais la demande est bien partie");
}

#[test]
fn test_l_image_finit_par_arriver_et_reste() {
    let src = photo_temoin("magasin-arrivee.png");
    let mut magasin = Magasin::nouveau();

    assert!(attendre(&mut magasin, &src), "l'image doit finir par arriver");
    assert_eq!(magasin.en_travail(), 0, "le chantier est vide");

    let pyramide = magasin.pyramide(&src).expect("elle est dans le cache");
    assert_eq!(
        (pyramide.native().width(), pyramide.native().height()),
        (48, 32)
    );
    assert!(magasin.octets() > 0, "elle occupe de la place, et on le sait");
}

#[test]
fn test_un_fichier_illisible_ne_se_redemande_jamais() {
    let absent = std::env::temp_dir()
        .join("glucose-magasin-tests")
        .join("rien-ici.png")
        .to_string_lossy()
        .to_string();
    let mut magasin = Magasin::nouveau();

    assert!(magasin.pyramide(&absent).is_none());
    let depart = std::time::Instant::now();
    while magasin.en_travail() > 0 && depart.elapsed() < std::time::Duration::from_secs(30) {
        magasin.recolter();
        std::thread::yield_now();
    }
    magasin.recolter();

    assert!(magasin.echecs.contains(&absent), "l'échec est retenu");
    // R-29 : sans le cache négatif, chaque image redemanderait ce fichier, et l'atelier
    // passerait sa vie à rouvrir un fichier qui n'existe pas.
    assert!(magasin.pyramide(&absent).is_none());
    assert_eq!(magasin.en_travail(), 0, "aucune nouvelle demande n'est partie");
}
