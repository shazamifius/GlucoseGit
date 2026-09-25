//! **Ce que la voie graphique doit garantir sur les lueurs** : les mêmes pixels que le
//! processeur, à un écart borné, mesuré, et qui ne bougera pas en silence.
//!
//! # Pourquoi un écart et non l'égalité au bit près
//!
//! Les deux voies calculent la **même** loi — HALO-1, `α = A · P(x) · P(y)` — mais ne
//! l'évaluent pas avec le même instrument, et c'est voulu :
//!
//! * le processeur échantillonne `Φ` dans une **table au pixel**, parce qu'un masque
//!   pré-calculé doit être discret, puis arrondit chaque niveau à l'un des trente-huit que la
//!   lueur peut prendre ;
//! * la carte évalue `Φ` **analytiquement**, en flottant, sans table ni quantification.
//!
//! L'écart est donc celui de la discrétisation du processeur, et il se borne : la lueur
//! plafonne à 38 sur 255, un niveau vaut donc au plus 38 pas de couleur sur une teinte
//! saturée, et la quantification en coûte au plus un.
//!
//! Ce test le **mesure** au lieu de le supposer, et il tient la borne — c'est ce qui
//! interdit qu'une future « optimisation » du nuanceur abîme l'image sans que personne ne
//! le voie.

use super::*;
use crate::present::banc_gpu;
use crate::renderer::halo;
use tiny_skia::Pixmap;

/// La lueur qu'une carte de 240 × 60 pose à l'échelle 1, telle que le socle la décrit.
fn boite(x: f32, y: f32) -> halo::HaloBox {
    halo::HaloBox {
        left: x - halo::HALO_SPREAD,
        top: y - halo::HALO_SPREAD,
        right: x + 240.0 + halo::HALO_SPREAD,
        bottom: y + 60.0 + halo::HALO_SPREAD,
        sigma: halo::HALO_BLUR / 2.0,
        carte: None,
    }
}

/// La même lueur, traduite pour la carte.
fn lueur(boite: halo::HaloBox, rgb: (u8, u8, u8)) -> Lueur {
    Lueur::de(boite, rgb, halo::HALO_ALPHA)
}

/// Ce que le **processeur** rend : un fond noir opaque, puis les lueurs, dans l'ordre.
fn par_le_processeur(taille: (u32, u32), lueurs: &[(halo::HaloBox, (u8, u8, u8))]) -> Pixmap {
    let mut p = Pixmap::new(taille.0, taille.1).expect("un pixmap");
    p.fill(tiny_skia::Color::BLACK);
    for (boite, rgb) in lueurs {
        halo::draw_halo(&mut p.as_mut(), *boite, *rgb, halo::HALO_ALPHA);
    }
    p
}

/// Ce que la **carte** rend, sur le même fond noir opaque.
fn par_la_carte(taille: (u32, u32), lueurs: &[Lueur]) -> Option<Pixmap> {
    let (peripherique, file) = banc_gpu::carte()?;
    let mut passe_lueurs = Lueurs::nouvelles(&peripherique, banc_gpu::FORMAT);
    passe_lueurs.preparer(
        &peripherique,
        &file,
        (taille.0 as f32, taille.1 as f32),
        lueurs,
    );
    let cible = banc_gpu::cible(&peripherique, taille);
    let vue = cible.create_view(&Default::default());
    let mut encodeur = peripherique.create_command_encoder(&Default::default());
    banc_gpu::passe(&mut encodeur, &vue, wgpu::Color::BLACK, |p| {
        passe_lueurs.poser(p);
    });
    file.submit(Some(encodeur.finish()));
    banc_gpu::relire(&peripherique, &file, &cible, taille)
}

/// La borne de l'écart entre les deux voies, en niveaux de couleur sur 255.
///
/// **Mesurée, pas choisie.** Le raisonnement donne un majorant : une lueur ne prend que
/// trente-huit niveaux d'opacité distincts, l'arrondi au plus proche en coûte au plus un
/// demi, et une seconde lueur composée par-dessus en ajoute un. La mesure le confirme et le
/// resserre — **un** niveau pour une lueur seule, **deux** pour deux superposées — et c'est
/// cette valeur-là qui est écrite ici.
///
/// La tenir serrée est tout l'intérêt : à quatre ou à dix, le test aurait continué de passer
/// pendant qu'un nuanceur dérive. Elle ne se relève pas ; si elle est franchie, c'est que le
/// nuanceur a changé de loi.
const ECART_ADMIS: u8 = 2;

/// **Une lueur seule rend les mêmes pixels des deux côtés**, à la quantification près.
#[test]
fn test_une_lueur_rend_la_meme_chose_des_deux_cotes() {
    let taille = (512u32, 320u32);
    let b = boite(120.0, 110.0);
    let rgb = (96, 165, 250);
    let Some(carte) = par_la_carte(taille, &[lueur(b, rgb)]) else {
        eprintln!("aucune carte utilisable : test saute");
        return;
    };
    let processeur = par_le_processeur(taille, &[(b, rgb)]);

    let pire = banc_gpu::pire_ecart(&processeur, &carte);
    assert!(
        pire <= ECART_ADMIS,
        "les deux voies divergent de {pire} niveaux (admis {ECART_ADMIS})"
    );
}

/// **La lueur est bien là, et pas seulement « proche du noir ».**
///
/// Sans cette épreuve, un nuanceur qui ne dessinerait **rien** passerait le test précédent
/// avec un écart de zéro — le fond étant noir des deux côtés. C'est la leçon des compteurs
/// jamais remplis (fiche 17 § 3.1) : un zéro se lit comme une mesure.
#[test]
fn test_la_lueur_ecrit_vraiment_des_pixels() {
    let taille = (512u32, 320u32);
    let b = boite(120.0, 110.0);
    let rgb = (96, 165, 250);
    let Some(carte) = par_la_carte(taille, &[lueur(b, rgb)]) else {
        eprintln!("aucune carte utilisable : test saute");
        return;
    };
    let encres = carte
        .data()
        .as_chunks::<4>()
        .0
        .iter()
        .filter(|p| p[0] > 0 || p[1] > 0 || p[2] > 0)
        .count();
    assert!(
        encres > 20_000,
        "une lueur de 300 x 120 dilatee touche bien plus que {encres} pixels"
    );
}

/// **Deux lueurs qui se chevauchent se composent dans le même ordre des deux côtés.**
///
/// Le processeur les peint l'une après l'autre ; la carte les dessine comme deux instances du
/// même lot. Rien ne garantirait *a priori* qu'elles se composent dans l'ordre — et un
/// mélange qui commute mal se voit exactement là où deux cartes se touchent.
#[test]
fn test_deux_lueurs_se_composent_dans_le_meme_ordre() {
    let taille = (512u32, 320u32);
    let a = (boite(80.0, 90.0), (250u8, 96u8, 96u8));
    let b = (boite(150.0, 130.0), (96u8, 250u8, 165u8));
    let Some(carte) = par_la_carte(taille, &[lueur(a.0, a.1), lueur(b.0, b.1)]) else {
        eprintln!("aucune carte utilisable : test saute");
        return;
    };
    let processeur = par_le_processeur(taille, &[a, b]);

    let pire = banc_gpu::pire_ecart(&processeur, &carte);
    assert!(
        pire <= ECART_ADMIS,
        "deux lueurs superposees divergent de {pire} niveaux (admis {ECART_ADMIS})"
    );
}

/// **Une lueur minuscule ne produit ni `NaN` ni bord net faux.**
///
/// Vue de très loin, `σ` tombe sous le pixel et la division `1 / (σ√2)` explose. Le nuanceur
/// la borne ; ce test vérifie que le résultat reste celui d'un bord net, comme le processeur
/// le produit avec un noyau d'un seul échantillon.
#[test]
fn test_une_lueur_minuscule_ne_diverge_pas() {
    let taille = (128u32, 128u32);
    let b = halo::HaloBox {
        left: 40.0,
        top: 40.0,
        right: 88.0,
        bottom: 88.0,
        sigma: 0.0,
        carte: None,
    };
    let rgb = (255, 255, 255);
    let Some(carte) = par_la_carte(taille, &[lueur(b, rgb)]) else {
        eprintln!("aucune carte utilisable : test saute");
        return;
    };
    let processeur = par_le_processeur(taille, &[(b, rgb)]);
    let pire = banc_gpu::pire_ecart(&processeur, &carte);
    assert!(
        pire <= ECART_ADMIS,
        "une lueur d'ecart-type nul diverge de {pire} niveaux"
    );
}

/// **Aucune lueur ne se dessine quand il n'y en a pas.**
#[test]
fn test_aucune_lueur_ne_laisse_le_fond_intact() {
    let taille = (64u32, 64u32);
    let Some(carte) = par_la_carte(taille, &[]) else {
        eprintln!("aucune carte utilisable : test saute");
        return;
    };
    assert!(
        carte
            .data()
            .as_chunks::<4>()
            .0
            .iter()
            .all(|p| *p == [0, 0, 0, 255]),
        "le fond a ete touche alors qu'aucune lueur n'etait posee"
    );
}

/// Le tampon grandit quand l'écran demande plus de lueurs que la capacité de départ, et il
/// les pose toutes.
#[test]
fn test_le_tampon_suit_ce_que_l_ecran_demande() {
    let taille = (256u32, 256u32);
    let combien = LUEURS_AU_DEPART + 7;
    let lueurs: Vec<Lueur> = (0..combien)
        .map(|i| {
            let x = (i % 16) as f32 * 14.0;
            let y = (i / 16) as f32 * 14.0;
            lueur(
                halo::HaloBox {
                    left: x,
                    top: y,
                    right: x + 8.0,
                    bottom: y + 8.0,
                    sigma: 2.0,
                    carte: None,
                },
                (200, 200, 200),
            )
        })
        .collect();
    let Some(carte) = par_la_carte(taille, &lueurs) else {
        eprintln!("aucune carte utilisable : test saute");
        return;
    };
    let encres = carte
        .data()
        .as_chunks::<4>()
        .0
        .iter()
        .filter(|p| p[0] > 0)
        .count();
    assert!(
        encres > 1_000,
        "au-dela de la capacite de depart, les lueurs cessent d'etre posees : {encres} pixels"
    );
}

// ── LUEUR-1 — la lueur découpée par sa carte ────────────────────────────────

/// Une lueur de carte telle que la géométrie la décrit : sa boîte dilatée, **et** la carte
/// arrondie qui la découpe — posée à des positions fractionnaires, pour que ses bords coupent
/// des pixels.
fn boite_decoupee(x: f32, y: f32) -> halo::HaloBox {
    halo::HaloBox {
        carte: Some(glucose_core::membrane_forme::Arrondi::nouveau(
            x, y, 240.0, 60.0, 32.0,
        )),
        ..boite(x, y)
    }
}

/// **Une lueur découpée rend la même chose des deux côtés** : la découpe est la même loi —
/// la distance exacte du noyau, le filtre-boîte d'un pixel — sur les deux voies.
#[test]
fn test_lueur_1_une_lueur_decoupee_rend_la_meme_chose_des_deux_cotes() {
    let taille = (512u32, 320u32);
    let b = boite_decoupee(120.3, 110.6);
    let rgb = (96, 165, 250);
    let Some(carte) = par_la_carte(taille, &[lueur(b, rgb)]) else {
        eprintln!("aucune carte utilisable : test saute");
        return;
    };
    let processeur = par_le_processeur(taille, &[(b, rgb)]);
    let pire = banc_gpu::pire_ecart(&processeur, &carte);
    assert!(
        pire <= ECART_ADMIS,
        "les deux voies divergent de {pire} niveaux (admis {ECART_ADMIS})"
    );
}

/// **Sous sa carte, la lueur n'écrit rien — sur les deux voies** ; hors de sa carte, elle
/// écrit ce qu'elle écrivait. C'est l'ombre CSS de Tauri : découpée à l'intérieur de la boîte.
#[test]
fn test_lueur_1_la_carte_ne_recoit_pas_sa_propre_lueur() {
    let taille = (512u32, 320u32);
    let b = boite_decoupee(120.3, 110.6);
    let arrondi = b.carte.expect("une carte");
    let rgb = (96, 165, 250);
    let libre = par_le_processeur(taille, &[(boite(120.3, 110.6), rgb)]);
    let mut voies = vec![par_le_processeur(taille, &[(b, rgb)])];
    voies.extend(par_la_carte(taille, &[lueur(b, rgb)]));
    for image in &voies {
        let (mut dedans, mut dehors) = (0, 0);
        for y in 0..taille.1 {
            for x in 0..taille.0 {
                let d = arrondi.distance(x as f32 + 0.5, y as f32 + 0.5);
                let ici = image.pixel(x, y).expect("dans l'image");
                if d <= -0.5 {
                    dedans += 1;
                    assert_eq!(
                        (ici.red(), ici.green(), ici.blue()),
                        (0, 0, 0),
                        "({x}, {y}) est sous la carte : aucune lueur"
                    );
                } else if d >= 0.5 && image.data() == voies[0].data() {
                    dehors += 1;
                    assert_eq!(ici, libre.pixel(x, y).expect("dans l'image"));
                }
            }
        }
        assert!(dedans > 10_000, "la carte couvre {dedans} pixels");
        assert!(
            dehors == 0 || dehors > 50_000,
            "hors de la carte : {dehors}"
        );
    }
}
