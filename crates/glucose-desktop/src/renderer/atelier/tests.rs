//! Ce que l'atelier promet : décoder ailleurs, ne pas mentir, ne pas redemander.

use super::*;

/// Écrit une image de test sur le disque et rend son chemin.
fn photo_temoin(nom: &str, w: u32, h: u32) -> std::path::PathBuf {
    let dossier = std::env::temp_dir().join("glucose-atelier-tests");
    std::fs::create_dir_all(&dossier).expect("dossier de test");
    let chemin = dossier.join(nom);
    if !chemin.exists() {
        let mut brute = image::RgbaImage::new(w, h);
        for (x, y, px) in brute.enumerate_pixels_mut() {
            *px = image::Rgba([(x % 256) as u8, (y % 256) as u8, 200, 255]);
        }
        brute.save(&chemin).expect("écriture de la photo témoin");
    }
    chemin
}

/// Attend que l'atelier ait rendu tout son travail, sans jamais boucler indéfiniment.
fn moisson_complete(atelier: &mut Atelier) -> Vec<Decodee> {
    let mut tout = Vec::new();
    let depart = std::time::Instant::now();
    while atelier.en_travail() > 0 && depart.elapsed() < std::time::Duration::from_secs(30) {
        tout.extend(atelier.recolter());
        std::thread::yield_now();
    }
    tout.extend(atelier.recolter());
    tout
}

#[test]
fn test_une_image_se_decode_sur_un_fil_de_fond() {
    let chemin = photo_temoin("temoin-64.png", 64, 48);
    let src = chemin.to_string_lossy().to_string();

    let mut atelier = Atelier::nouveau();
    assert!(atelier.demander(&src), "la demande doit partir");
    assert_eq!(atelier.en_travail(), 1);

    let moisson = moisson_complete(&mut atelier);
    assert_eq!(moisson.len(), 1, "une demande, une réponse");
    let (rendu, image, cout) = &moisson[0];
    assert_eq!(rendu, &src);
    let pyramide = image.as_ref().expect("l'image témoin est lisible");
    assert_eq!(
        (pyramide.native().width(), pyramide.native().height()),
        (64, 48)
    );
    // Ce qui revient est PRÊT À POSER : la pyramide est faite, pas seulement le décodage.
    // Sans cela, le fil de rendu la construisait lui-même — 73 ms mesurés en pleine image.
    assert!(
        pyramide.niveaux_construits() > 1,
        "l'ouvrier rend une pyramide complète, pas un seul niveau"
    );
    // Le coût du décodage est chronométré : c'est ce qui donne son utilité à l'image dans le
    // cache, et une utilité mesurée vaut mieux qu'une utilité estimée (ADAPT-1).
    assert!(
        *cout > std::time::Duration::ZERO,
        "un décodage réussi a forcément pris du temps"
    );
    assert_eq!(atelier.en_travail(), 0, "le chantier est vide");
}

#[test]
fn test_une_meme_image_ne_part_pas_deux_fois() {
    let chemin = photo_temoin("temoin-double.png", 32, 32);
    let src = chemin.to_string_lossy().to_string();

    let mut atelier = Atelier::nouveau();
    assert!(atelier.demander(&src));
    // Le rendu redemande la même image à chaque passe tant qu'elle n'est pas là. Sans ce
    // garde-fou, quelques dixièmes de seconde suffiraient à empiler des centaines de
    // décodages du même fichier.
    assert!(!atelier.demander(&src), "la deuxième demande est refusée");
    assert!(!atelier.demander(&src));
    assert_eq!(atelier.en_travail(), 1);

    let moisson = moisson_complete(&mut atelier);
    assert_eq!(moisson.len(), 1, "un seul décodage, pas trois");
}

#[test]
fn test_un_fichier_absent_revient_sans_image_et_sans_bloquer() {
    let absent = std::env::temp_dir()
        .join("glucose-atelier-tests")
        .join("ce-fichier-n-existe-pas.png")
        .to_string_lossy()
        .to_string();

    let mut atelier = Atelier::nouveau();
    assert!(atelier.demander(&absent));
    let moisson = moisson_complete(&mut atelier);

    assert_eq!(moisson.len(), 1);
    assert!(
        moisson[0].1.is_none(),
        "pas d'image, et c'est un cas normal"
    );
    assert_eq!(atelier.en_travail(), 0, "un échec libère le chantier");
}

#[test]
fn test_un_lot_entier_se_decode_sans_qu_aucune_demande_se_perde() {
    let mut atelier = Atelier::nouveau();
    let mut attendus = Vec::new();
    for i in 0..12 {
        let chemin = photo_temoin(&format!("temoin-lot-{i}.png"), 24, 24);
        let src = chemin.to_string_lossy().to_string();
        assert!(atelier.demander(&src));
        attendus.push(src);
    }
    assert_eq!(atelier.en_travail(), 12);

    let moisson = moisson_complete(&mut atelier);
    assert_eq!(moisson.len(), 12, "douze demandes, douze réponses");
    for src in &attendus {
        assert!(
            moisson
                .iter()
                .any(|(rendu, img, _)| rendu == src && img.is_some()),
            "{src} n'est pas revenu"
        );
    }
}

#[test]
fn test_un_pixel_opaque_traverse_la_premultiplication_intact() {
    // Le défaut que ce test ferme : la prémultiplication en flottant tronquait, et un canal
    // à 255 sur un pixel opaque ressortait à 254. Sur une photo, l'écart est invisible ;
    // sur un aplat de couleur voisin d'une bordure, il se voit.
    let dossier = std::env::temp_dir().join("glucose-atelier-tests");
    std::fs::create_dir_all(&dossier).expect("dossier de test");
    let chemin = dossier.join("temoin-opaque.png");
    let mut brute = image::RgbaImage::new(4, 1);
    for (i, px) in brute.pixels_mut().enumerate() {
        let v = [0u8, 1, 128, 255][i];
        *px = image::Rgba([v, v, v, 255]);
    }
    brute.save(&chemin).expect("écriture");

    let image = decoder(&chemin.to_string_lossy()).expect("décodage");
    let (pixels, _) = image.data().as_chunks::<4>();
    for (i, attendu) in [0u8, 1, 128, 255].iter().enumerate() {
        assert_eq!(
            pixels[i][0], *attendu,
            "un pixel opaque doit ressortir exactement tel quel"
        );
    }
}

#[test]
fn test_l_atelier_a_toujours_au_moins_un_ouvrier() {
    // Sur une machine qui n'annonce qu'un cœur, « tous sauf un » donnerait zéro — et rien ne
    // serait jamais décodé.
    assert!(
        ouvriers() >= 1,
        "au moins un ouvrier, quelle que soit la machine"
    );
}
