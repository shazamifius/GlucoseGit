//! Les lois de [`Fx`]. Chacune est une propriété, pas un exemple : ce qui est affirmé dans
//! l'en-tête du module est vérifié ici, ou n'est pas affirmé.

use super::*;
use crate::types::Viewport;

// ── La constante est déduite, pas choisie ────────────────────────────────────

/// Distance confortable entre deux nœuds voisins d'un grand canva, en pixels monde.
/// Une carte de texte en fait 260 (fiche 06 § 5) ; 5 000 laisse la place de respirer, de
/// grouper et de zoomer sans que la carte soit un pavage serré.
const MAILLAGE_PX: f64 = 5_000.0;
/// La cible de la charte : des dizaines de millions de nœuds. On dimensionne sur dix.
const NOEUDS_CIBLE: f64 = 1e7;
/// Au zoom maximal, une unité de coordonnée doit rester indiscernable : au plus un quart de
/// pixel écran. En deçà, le positionnement sous-pixel du texte perdrait de la finesse.
const SOUS_PIXEL_MAX: f64 = 0.25;

fn portee_px(bits: u32) -> f64 {
    (1i64 << 31) as f64 / (1i64 << bits) as f64
}

fn pas_ecran_au_zoom_max(bits: u32) -> f64 {
    Viewport::MAX_SCALE / (1i64 << bits) as f64
}

/// Portée à couvrir : un canva de 10⁷ nœuds au maillage confortable, de part et d'autre de
/// l'origine. √10⁷ ≈ 3 163 nœuds de côté × 5 000 px = 15,8 Mpx de large, soit ±7,9 Mpx.
fn portee_requise_px() -> f64 {
    NOEUDS_CIBLE.sqrt() * MAILLAGE_PX / 2.0
}

/// La justification de [`FRAC_BITS`], sous forme exécutable : **un seul** nombre de bits
/// satisfait les deux exigences du projet, et c'est celui que le module retient.
///
/// Ce test est la raison pour laquelle cette constante n'a pas à être « réglée ». Le jour où
/// la cible en nœuds, le maillage ou le zoom maximal changent, c'est lui qui le dira — et si
/// plus aucun `i32` ne convient, il dira ça aussi.
#[test]
fn test_la_precision_est_la_seule_qui_satisfasse_les_deux_exigences() {
    let admissibles: Vec<u32> = (0..=16)
        .filter(|&b| {
            portee_px(b) >= portee_requise_px() && pas_ecran_au_zoom_max(b) <= SOUS_PIXEL_MAX
        })
        .collect();
    assert_eq!(
        admissibles,
        vec![FRAC_BITS],
        "portée requise {:.0} px ; candidats : {:?}",
        portee_requise_px(),
        (0..=16)
            .map(|b| (b, portee_px(b) as i64, pas_ecran_au_zoom_max(b)))
            .collect::<Vec<_>>()
    );
}

/// Les deux bornes annoncées dans la documentation sont celles que le type a vraiment.
#[test]
fn test_le_domaine_est_celui_qui_est_documente() {
    assert_eq!(Fx::SPAN, 8_388_608.0);
    assert_eq!(Fx::MAX.to_f64(), 8_388_608.0 - 1.0 / 256.0);
    assert_eq!(Fx::MIN.to_f64(), -8_388_608.0);
    assert_eq!(Fx::ONE.to_f64(), 1.0);
    assert_eq!(Fx::EPSILON.to_f64(), 1.0 / 256.0);
    // La borne anti-crash de la fiche 09 (±10⁶ px) est représentable sans saturer — la
    // propriété qui compte vraiment, et qui se vérifie sur une valeur, pas sur une constante.
    assert_eq!(Fx::from_f64(1e6).to_f64(), 1e6);
    assert_eq!(Fx::from_f64(-1e6).to_f64(), -1e6);
}

// ── L'exactitude : ce que le f64 ne sait pas faire ───────────────────────────

/// L'addition est associative et commutative **au sens strict**, là où le `f64` ne l'est qu'à
/// l'arrondi près. Le contre-exemple flottant est joint : c'est lui qui rend le test utile.
#[test]
fn test_l_addition_est_associative_et_commutative_exactement() {
    // Le contre-exemple qui motive tout le module.
    assert_ne!(0.1_f64 + 0.2 + 0.3, 0.1 + (0.2 + 0.3));

    let valeurs = [-1000.0, -0.1, 0.0, 0.3, 1.0, 1234.567, 99_999.5];
    for a in valeurs {
        for b in valeurs {
            for c in valeurs {
                let (a, b, c) = (Fx::from_f64(a), Fx::from_f64(b), Fx::from_f64(c));
                assert_eq!((a + b) + c, a + (b + c), "associativité");
                assert_eq!(a + b, b + a, "commutativité");
            }
        }
    }
}

/// Un glisser long ne dérive pas d'une unité, quel que soit le nombre de pas.
///
/// **Et le `f64` non plus, à l'œil.** Ce test a d'abord été écrit pour montrer que le flottant
/// dérivait ; il a échoué, parce que l'aller-retour flottant revenait exactement à son point de
/// départ. La mesure qui suit est donc à charge contre l'argument de la fiche 11 § RQ-1 : la
/// dérive existe, elle est bornée par 10⁻⁵ pixel sur cent mille pas, et elle est invisible à
/// tout zoom. Ce n'est **pas** ce qui justifie la virgule fixe.
///
/// L'assertion sur le `f64` est écrite comme une borne et non comme une égalité : le jour où
/// elle casserait, c'est que l'argument serait devenu vrai, et il faudrait le savoir.
#[test]
fn test_un_glisser_long_est_exact_et_la_derive_du_f64_reste_invisible() {
    const PAS: usize = 100_000;
    let depart = Fx::from_f64(1_000_000.0);
    let delta = Fx::from_raw(7); // 7/256 de pixel : un pas de souris fin, exactement représenté

    let mut p = depart;
    for _ in 0..PAS {
        p += delta;
    }
    assert_eq!(
        p,
        depart + Fx::from_raw(7 * PAS as i32),
        "cent mille pas valent exactement la somme des pas"
    );
    for _ in 0..PAS {
        p -= delta;
    }
    assert_eq!(p, depart, "l'aller-retour entier revient au point de départ");

    // Le même geste en f64 : la dérive est réelle, et sans conséquence observable.
    let mut q = 1_000_000.0_f64;
    for _ in 0..PAS {
        q += 0.1;
    }
    let derive = (q - 1_010_000.0).abs();
    assert!(derive > 0.0, "la dérive flottante existe bel et bien");
    assert!(
        derive * Viewport::MAX_SCALE < 0.001,
        "et elle reste sous le millième de pixel écran au zoom maximal : {derive:e} px monde"
    );
}

/// L'ordre dérivé sur `Fx` est celui des réels représentés — c'est ce qui rend `Ord` utilisable
/// pour trier des nœuds par abscisse sans écrire de comparateur.
#[test]
fn test_l_ordre_est_celui_des_reels_representes() {
    let mut vals: Vec<f64> = vec![
        3.5,
        -17.25,
        0.0,
        1.0 / 256.0,
        -1.0 / 256.0,
        8_000_000.0,
        -8_000_000.0,
    ];
    let mut fx: Vec<Fx> = vals.iter().map(|&v| Fx::from_f64(v)).collect();
    vals.sort_by(f64::total_cmp);
    fx.sort();
    assert_eq!(fx.iter().map(|f| f.to_f64()).collect::<Vec<_>>(), vals);

    // Et l'égalité est exacte : deux nœuds sont alignés, ou ils ne le sont pas.
    assert_eq!(Fx::from_f64(12.0), Fx::from_f64(12.0));
    assert_ne!(Fx::from_f64(12.0), Fx::from_f64(12.0) + Fx::EPSILON);
}

/// L'aller-retour avec le monde flottant perd au plus un demi-pas — la définition même d'un
/// arrondi au plus proche. C'est l'erreur *unique* que la quantification introduit ; elle ne
/// s'accumule pas, contrairement à celle du `f64`.
#[test]
fn test_l_aller_retour_avec_f64_perd_au_plus_un_demi_pas() {
    let demi_pas = 1.0 / 512.0;
    let mut echantillon = 0.0_f64;
    for i in 0..10_000 {
        // Une suite irrationnelle : aucune valeur ne tombe sur une unité par hasard.
        echantillon = (i as f64) * std::f64::consts::PI * 37.0 % 100_000.0 - 50_000.0;
        let retour = Fx::from_f64(echantillon).to_f64();
        assert!(
            (retour - echantillon).abs() <= demi_pas,
            "{echantillon} → {retour}"
        );
    }
    assert!(echantillon != 0.0, "l'échantillon doit vraiment varier");
}

// ── Le domaine est clos : aucune opération n'en sort ─────────────────────────

/// Aucune opération ne déborde ni ne boucle : tout sature aux bornes du document. C'est la
/// borne anti-crash de la fiche 09 devenue propriété du type — il n'y a plus de `clamp` à ne
/// pas oublier, parce qu'il n'existe aucune valeur hors domaine.
#[test]
fn test_aucune_operation_ne_sort_du_domaine() {
    let extremes = [
        Fx::MIN,
        Fx::MAX,
        Fx::ZERO,
        Fx::ONE,
        -Fx::ONE,
        Fx::from_f64(8e6),
    ];
    for a in extremes {
        for b in extremes {
            // Le seul fait que ces appels rendent un `Fx` prouve l'absence de débordement ;
            // en `debug`, un dépassement entier non saturé ferait paniquer le test.
            let _ = a + b;
            let _ = a - b;
            let _ = a.scaled(i32::MAX, 1);
            let _ = a.scaled(1, i32::MAX);
        }
        let _ = -a;
        assert!(a.abs() >= Fx::ZERO);
    }
    assert_eq!(Fx::MAX + Fx::ONE, Fx::MAX, "la borne haute tient");
    assert_eq!(Fx::MIN - Fx::ONE, Fx::MIN, "la borne basse tient");
    assert_eq!(
        Fx::from_px(i32::MAX),
        Fx::MAX,
        "la conversion en pixels sature aussi"
    );
    assert_eq!(Fx::from_f64(1e300), Fx::MAX);
    assert_eq!(Fx::from_f64(-1e300), Fx::MIN);
    assert_eq!(Fx::from_f64(f64::INFINITY), Fx::MAX);
}

/// Une coordonnée qui n'est pas un nombre devient l'origine plutôt que de rendre le nœud
/// introuvable — même décision que [`Viewport::normalized`], pour la même raison.
#[test]
fn test_une_coordonnee_qui_n_est_pas_un_nombre_devient_l_origine() {
    assert_eq!(Fx::from_f64(f64::NAN), Fx::ZERO);
}

// ── Distance et mise à l'échelle ─────────────────────────────────────────────

/// La distance au carré est exacte jusqu'aux coins opposés du document : comparer deux
/// distances ne demande aucune tolérance, et le picking peut s'en servir tel quel.
///
/// Ce test a d'abord été écrit avec un `i64`, et le compilateur a refusé la ligne : l'écart
/// entre les deux coins vaut 2³², son carré 2⁶⁴, hors domaine. C'est ce refus qui a fixé le
/// type de retour de [`dist2`].
#[test]
fn test_la_distance_au_carre_est_exacte_aux_bornes_du_document() {
    // Le cas le plus défavorable : les deux coins du domaine.
    let d = dist2(Fx::MIN, Fx::MIN, Fx::MAX, Fx::MAX);
    let ecart = i32::MAX as i128 - i32::MIN as i128;
    assert_eq!(d, 2 * ecart * ecart);
    assert!(d > i64::MAX as i128, "c'est bien un cas qu'un i64 n'aurait pas tenu");

    // Un triplet pythagoricien, en pixels entiers : le résultat est exact, pas approché.
    let (a, b) = (Fx::from_px(3), Fx::from_px(4));
    let attendu = 5i128 * (1 << FRAC_BITS);
    assert_eq!(dist2(Fx::ZERO, Fx::ZERO, a, b), attendu * attendu);

    // Comparer sans racine ni epsilon : c'est tout ce dont le picking a besoin.
    let proche = dist2(Fx::ZERO, Fx::ZERO, Fx::from_px(3), Fx::ZERO);
    let loin = dist2(Fx::ZERO, Fx::ZERO, Fx::from_px(4), Fx::ZERO);
    assert!(proche < loin);
}

/// La mise à l'échelle par un rationnel — un rapport de deux longueurs, ce qu'est toujours un
/// facteur de redimensionnement — arrondit au plus proche et ne dérive pas de signe.
#[test]
fn test_la_mise_a_l_echelle_est_un_rationnel_arrondi_au_plus_proche() {
    let cent = Fx::from_px(100);
    assert_eq!(cent.scaled(1, 2), Fx::from_px(50));
    assert_eq!(cent.scaled(3, 1), Fx::from_px(300));
    assert_eq!(cent.scaled(0, 1), Fx::ZERO);
    assert_eq!((-cent).scaled(1, 2), Fx::from_px(-50));
    assert_eq!(cent.scaled(-1, 2), Fx::from_px(-50));
    // Un tiers de pixel : 256/3 = 85,33 → 85 unités, l'arrondi au plus proche.
    assert_eq!(Fx::ONE.scaled(1, 3), Fx::from_raw(85));
    assert_eq!((-Fx::ONE).scaled(1, 3), Fx::from_raw(-85));
    // Une demie exactement : 128 unités, sans ambiguïté de sens d'arrondi.
    assert_eq!(Fx::ONE.scaled(1, 2), Fx::from_raw(128));
    // Un dénominateur nul ne peut pas arriver, mais il ne doit pas faire tomber le programme.
    assert_eq!(cent.scaled(5, 0), cent);
}
