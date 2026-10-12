//! ORDONNER-2 — la couleur perçue d'une image, et l'ordre qu'on en tire.

use super::*;

fn px(r: u8, g: u8, b: u8) -> Option<Oklch> {
    Oklch::du_pixel([r, g, b, 255])
}

fn proche(a: f64, b: f64) -> bool {
    (a - b).abs() < 5e-5
}

/// **Les valeurs publiées d'Oklab** pour les primaires de sRGB et le blanc (Ottosson ; les
/// mêmes que CSS Color 4) : les matrices et la linéarisation sont recopiées sans faute.
#[test]
fn test_oklab_rend_les_valeurs_de_reference() {
    let attendus = [
        ([1.0, 0.0, 0.0], [0.62796, 0.22486, 0.12585]),
        ([0.0, 1.0, 0.0], [0.86644, -0.23389, 0.17950]),
        ([0.0, 0.0, 1.0], [0.45201, -0.03246, -0.31153]),
        ([1.0, 1.0, 1.0], [1.0, 0.0, 0.0]),
    ];
    for (srgb, attendu) in attendus {
        let lab = oklab(srgb);
        for k in 0..3 {
            assert!(
                proche(lab[k], attendu[k]),
                "{srgb:?} → {lab:?}, attendu {attendu:?}"
            );
        }
    }
}

/// **La clarté d'Oklab est celle de l'œil** — c'est la raison de quitter HSL. Le jaune et le
/// bleu purs ont la même « luminosité » HSL (0,5) ; à l'œil, le jaune est presque blanc et le
/// bleu sombre, et c'est ce que le tri du sombre au clair doit dire.
#[test]
fn test_un_jaune_est_plus_clair_qu_un_bleu_que_hsl_disait_egaux() {
    let hsl = |r: f64, g: f64, b: f64| (r.max(g).max(b) + r.min(g).min(b)) / 2.0;
    // La preuve à l'envers : l'ancienne mesure les déclare égaux.
    assert_eq!(hsl(1.0, 1.0, 0.0), hsl(0.0, 0.0, 1.0));

    let couleurs = [px(255, 255, 0), px(0, 0, 255)];
    assert_eq!(ordonner(&couleurs, Tri::SombreVersClair), vec![1, 0]);
    assert_eq!(ordonner(&couleurs, Tri::ClairVersSombre), vec![0, 1]);
}

/// Un pixel prémultiplié se redresse : un rouge à moitié transparent est le même rouge.
#[test]
fn test_un_pixel_premultiplie_garde_sa_couleur() {
    let plein = Oklch::du_pixel([255, 0, 0, 255]).expect("une couleur");
    let moitie = Oklch::du_pixel([128, 0, 0, 128]).expect("une couleur");
    assert!(proche(plein.l, moitie.l) && proche(plein.c, moitie.c) && proche(plein.h, moitie.h));
}

/// Un pixel entièrement transparent n'a pas de couleur, et ne se range pas comme un noir.
#[test]
fn test_un_pixel_transparent_n_a_pas_de_couleur() {
    assert_eq!(Oklch::du_pixel([0, 0, 0, 0]), None);
    let couleurs = [
        Oklch::du_pixel([0, 0, 0, 0]),
        px(0, 0, 0),
        px(255, 255, 255),
    ];
    assert_eq!(ordonner(&couleurs, Tri::SombreVersClair), vec![1, 2, 0]);
}

/// **Gris : ce qu'on ne distingue pas d'un gris.** Un gris teinté de douze unités de rouge
/// reste sous la différence à peine visible ; à vingt-deux, l'œil voit le rouge.
#[test]
fn test_une_teinte_commence_a_la_difference_a_peine_visible() {
    let gris = px(128, 128, 128).expect("une couleur");
    let presque = px(140, 128, 128).expect("une couleur");
    let rose = px(150, 128, 128).expect("une couleur");
    assert!(!gris.a_une_teinte());
    assert!(presque.c > 0.0 && !presque.a_une_teinte(), "{presque:?}");
    assert!(rose.a_une_teinte(), "{rose:?}");
}

/// **Le cercle se coupe au plus grand vide.** Des magentas, des rouges, des oranges et des
/// jaunes, rien entre le vert et le bleu : le rangement va du magenta au jaune, et le
/// magenta reste à côté du rouge.
///
/// La preuve à l'envers : couper le cercle au rouge, comme Glucose Tauri, envoyait le magenta
/// tout au bout, loin des rouges qui lui ressemblent.
#[test]
fn test_le_cercle_se_coupe_au_plus_grand_vide() {
    let (jaune, rouge, magenta, orange) = (0, 1, 2, 3);
    let couleurs = [
        px(255, 255, 0),
        px(255, 0, 0),
        px(255, 0, 255),
        px(255, 128, 0),
    ];
    assert_eq!(
        ordonner(&couleurs, Tri::Teinte),
        vec![magenta, rouge, orange, jaune]
    );

    let degres = |c: &Option<Oklch>| c.expect("une couleur").h.rem_euclid(std::f64::consts::TAU);
    let mut depuis_le_rouge: Vec<usize> = (0..couleurs.len()).collect();
    depuis_le_rouge.sort_by(|&i, &j| degres(&couleurs[i]).total_cmp(&degres(&couleurs[j])));
    assert_eq!(depuis_le_rouge, vec![rouge, orange, jaune, magenta]);
}

/// **Le plus grand vide peut être au milieu des angles** : du cyan, du bleu, du jaune et du
/// vert, rien entre le bleu et le jaune. Le rangement va du jaune au bleu par le vert et le
/// cyan, chaque couleur à côté de sa voisine.
///
/// La preuve à l'envers : partir du plus petit angle donnait cyan, bleu, jaune, vert — le vert
/// et le cyan, voisins sur le cercle, aux deux bouts, et le vide au milieu du rangement.
#[test]
fn test_le_plus_grand_vide_peut_etre_au_milieu_des_angles() {
    let (cyan, bleu, jaune, vert) = (0, 1, 2, 3);
    let couleurs = [
        px(0, 255, 255),
        px(0, 0, 255),
        px(255, 255, 0),
        px(0, 255, 0),
    ];
    assert_eq!(
        ordonner(&couleurs, Tri::Teinte),
        vec![jaune, vert, cyan, bleu]
    );

    let mut depuis_le_plus_petit: Vec<usize> = (0..couleurs.len()).collect();
    let angle = |k: usize| couleurs[k].expect("une couleur").h;
    depuis_le_plus_petit.sort_by(|&i, &j| angle(i).total_cmp(&angle(j)));
    assert_eq!(depuis_le_plus_petit, vec![cyan, bleu, jaune, vert]);
}

/// Les gris viennent après les teintes, du sombre au clair ; ce qui n'a pas de couleur vient
/// en dernier, dans l'ordre où il était.
#[test]
fn test_les_gris_puis_ce_qui_n_a_pas_de_couleur() {
    let couleurs = [
        None,
        px(200, 200, 200),
        px(0, 0, 255),
        Oklch::du_pixel([0, 0, 0, 0]),
        px(20, 20, 20),
        px(255, 0, 0),
    ];
    assert_eq!(ordonner(&couleurs, Tri::Teinte), vec![2, 5, 4, 1, 0, 3]);
}

/// À égalité, l'ordre donné reste : ranger deux fois ne change plus rien.
#[test]
fn test_a_egalite_l_ordre_donne_reste() {
    let couleurs = [px(255, 0, 0), px(10, 10, 10), px(255, 0, 0), px(10, 10, 10)];
    for tri in [Tri::Teinte, Tri::SombreVersClair, Tri::ClairVersSombre] {
        let ordre = ordonner(&couleurs, tri);
        let premiers_rouges =
            ordre.iter().position(|&i| i == 0) < ordre.iter().position(|&i| i == 2);
        let premiers_noirs =
            ordre.iter().position(|&i| i == 1) < ordre.iter().position(|&i| i == 3);
        assert!(premiers_rouges && premiers_noirs, "{tri:?} : {ordre:?}");
        let reordonnees: Vec<_> = ordre.iter().map(|&i| couleurs[i]).collect();
        let identite: Vec<usize> = (0..couleurs.len()).collect();
        assert_eq!(ordonner(&reordonnees, tri), identite, "{tri:?}");
    }
}

/// Une seule teinte, ou aucune : rien à couper.
#[test]
fn test_une_seule_teinte_ne_se_coupe_pas() {
    assert_eq!(ordonner(&[px(255, 0, 0)], Tri::Teinte), vec![0]);
    assert_eq!(ordonner(&[], Tri::Teinte), Vec::<usize>::new());
}
