use super::couverture::couverture;
use super::emballage::{bandes_de, emballer, Emballage};
use super::{morceaux_d_une_cubique, point_de_cubique, Contours, Point};

/// Un générateur congruentiel : des cubiques variées, toujours les mêmes.
struct Hasard(u64);

impl Hasard {
    fn suivant(&mut self) -> f64 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        (self.0 >> 11) as f64 / (1u64 << 53) as f64
    }

    fn point(&mut self, etendue: f64) -> Point {
        [self.suivant() * etendue, self.suivant() * etendue]
    }
}

fn distance(a: Point, b: Point) -> f64 {
    (a[0] - b[0]).hypot(a[1] - b[1])
}

/// Le rectangle `[x0, x1] × [y0, y1]`, parcouru dans un sens (`direct`) ou dans l'autre.
fn rectangle(c: &mut Contours, (x0, y0, x1, y1): (f64, f64, f64, f64), direct: bool) {
    let mut coins = [[x0, y0], [x1, y0], [x1, y1], [x0, y1]];
    if !direct {
        coins.reverse();
    }
    c.aller(coins[0]);
    for p in &coins[1..] {
        c.ligne(*p);
    }
    c.fermer();
}

/// Un disque de centre `(cx, cy)` et de rayon `r`, en quatre cubiques (le `κ` des arcs).
fn disque(c: &mut Contours, (cx, cy): (f64, f64), r: f64) {
    let k = 0.552_284_749_830_793_4 * r;
    c.aller([cx + r, cy]);
    c.cubique([cx + r, cy + k], [cx + k, cy + r], [cx, cy + r]);
    c.cubique([cx - k, cy + r], [cx - r, cy + k], [cx - r, cy]);
    c.cubique([cx - r, cy - k], [cx - k, cy - r], [cx, cy - r]);
    c.cubique([cx + k, cy - r], [cx + r, cy - k], [cx + r, cy]);
    c.fermer();
}

fn emballe(bati: impl FnOnce(&mut Contours), tolerance: f64) -> Emballage {
    let mut c = Contours::nouveaux(tolerance);
    bati(&mut c);
    emballer(&c.finir()).expect("une forme qui a des courbes")
}

/// Les trois points d'une courbe, lus dans l'emballage.
fn courbe(e: &Emballage, r: u32) -> [[f32; 2]; 3] {
    let (a, b) = (e.courbes[r as usize], e.courbes[r as usize + 1]);
    [[a[0], a[1]], [a[2], a[3]], [b[0], b[1]]]
}

/// Les rangs des courbes d'une bande.
fn bande(e: &Emballage, b: u32) -> &[u32] {
    let (n, ou) = (e.bandes[2 * b as usize], e.bandes[2 * b as usize + 1]);
    &e.bandes[ou as usize..(ou + n) as usize]
}

/// Toutes les courbes d'une forme : celles que nomme au moins une bande.
fn toutes(e: &Emballage) -> Vec<u32> {
    let n = e.nombre_de_bandes[0] + e.nombre_de_bandes[1];
    let mut rangs: Vec<u32> = (0..n).flat_map(|b| bande(e, b).to_vec()).collect();
    rangs.sort_unstable();
    rangs.dedup();
    rangs
}

/// CUBIQUE-1 : chaque morceau s'écarte de sa cubique de la tolérance au plus — mesuré en cent
/// points de chaque morceau, sur deux cents cubiques et trois tolérances.
#[test]
fn test_une_cubique_s_approche_a_sa_tolerance() {
    let mut h = Hasard(7);
    let mut pire = 0.0f64;
    for tolerance in [1.0, 0.25, 0.001] {
        for _ in 0..200 {
            let c = [
                h.point(2048.0),
                h.point(2048.0),
                h.point(2048.0),
                h.point(2048.0),
            ];
            let mut contours = Contours::nouveaux(tolerance);
            contours.aller(c[0]);
            contours.cubique(c[1], c[2], c[3]);
            let morceaux = contours.courant.clone();
            let n = morceaux.len();
            assert_eq!(n, morceaux_d_une_cubique(c, tolerance));
            for (i, q) in morceaux.iter().enumerate() {
                for k in 0..=100 {
                    let t = f64::from(k) / 100.0;
                    let global = (i as f64 + t) / n as f64;
                    let ecart = distance(q.point(t), point_de_cubique(&c, global));
                    pire = pire.max(ecart / tolerance);
                }
            }
        }
    }
    assert!(
        pire <= 1.0 + 1e-9,
        "un morceau s'écarte de {pire} fois sa tolérance"
    );
    // Et la borne n'est pas lâche au point de ne rien exercer.
    assert!(pire > 0.05, "l'épreuve n'exerce pas la borne ({pire})");
}

/// Une cubique qui est une droite (ses contrôles sur la droite) ne se coupe pas.
#[test]
fn test_une_cubique_droite_reste_une_courbe() {
    let c = [[0.0, 0.0], [1.0, 1.0], [2.0, 2.0], [3.0, 3.0]];
    assert_eq!(morceaux_d_une_cubique(c, 0.5), 1);
}

/// Les contours se ferment, et chaque courbe commence où la précédente finit — au bit près :
/// c'est ce qui permet de ranger chaque extrémité une seule fois.
#[test]
fn test_les_contours_sont_fermes_et_joints() {
    let mut c = Contours::nouveaux(0.5);
    disque(&mut c, (500.0, 500.0), 400.0);
    c.aller([0.0, 0.0]);
    c.ligne([10.0, 0.0]);
    c.quadratique([20.0, 10.0], [10.0, 20.0]);
    // Pas de fermeture : `finir` la fait.
    let contours = c.finir();
    assert_eq!(contours.len(), 2);
    for contour in &contours {
        for paire in contour.windows(2) {
            assert_eq!(paire[0].fin, paire[1].debut);
        }
        assert_eq!(contour.last().unwrap().fin, contour[0].debut);
    }
}

/// Une ligne de longueur nulle n'est pas une courbe, et une forme sans courbe ne se range pas.
#[test]
fn test_une_ligne_nulle_n_existe_pas() {
    let mut c = Contours::nouveaux(0.5);
    c.aller([1.0, 1.0]);
    c.ligne([1.0, 1.0]);
    c.fermer();
    let contours = c.finir();
    assert!(contours.is_empty());
    assert!(emballer(&contours).is_none());
}

/// BANDES-2 : pour toute ordonnée qu'un rayon horizontal peut avoir, chaque courbe qui la
/// croise est dans la bande que **l'une et l'autre** arithmétique lui donnent — essayé aux
/// extrémités exactes des courbes, à un cran de flottant de part et d'autre, et partout entre.
#[test]
fn test_une_courbe_est_dans_chaque_bande_qu_un_rayon_peut_calculer() {
    let e = emballe(
        |c| {
            disque(c, (0.5, 0.5), 0.4);
            rectangle(c, (0.1, 0.2, 0.33, 0.71), false);
        },
        1.0 / 4096.0,
    );
    let [_, ey, _, dy] = e.transformation;
    let nh = e.nombre_de_bandes[0];
    let rangs = toutes(&e);
    let mut ordonnees: Vec<f32> = (0..=2000).map(|k| k as f32 / 2000.0).collect();
    for &r in &rangs {
        for p in courbe(&e, r) {
            let bits = p[1].to_bits();
            ordonnees.extend([p[1], f32::from_bits(bits + 1), f32::from_bits(bits - 1)]);
        }
    }
    let mut verifiees = 0;
    for y in ordonnees {
        let (a, b) = bandes_de(y, ey, dy, nh - 1);
        for b in [a, b] {
            let dedans = bande(&e, b);
            for &r in &rangs {
                let p = courbe(&e, r).map(|q| q[1]);
                let (lo, hi) = (p[0].min(p[1]).min(p[2]), p[0].max(p[1]).max(p[2]));
                if lo != hi && lo <= y && y <= hi {
                    assert!(
                        dedans.contains(&r),
                        "la courbe {r} croise y = {y} hors de la bande {b}"
                    );
                    verifiees += 1;
                }
            }
        }
    }
    assert!(
        verifiees > 10_000,
        "l'épreuve n'a presque rien vérifié ({verifiees})"
    );
}

/// Dans une bande horizontale, les courbes vont de la plus à droite à la plus à gauche ; dans
/// une verticale, de la plus haute à la plus basse — c'est ce qui permet de s'arrêter tôt.
#[test]
fn test_les_bandes_sont_triees_du_plus_loin_au_plus_pres() {
    let e = emballe(|c| disque(c, (0.5, 0.5), 0.4), 1.0 / 4096.0);
    let [nh, nv] = e.nombre_de_bandes;
    for (b, axe) in (0..nh).map(|b| (b, 0)).chain((nh..nh + nv).map(|b| (b, 1))) {
        let maxima: Vec<f32> = bande(&e, b)
            .iter()
            .map(|&r| {
                courbe(&e, r)
                    .iter()
                    .map(|p| p[axe])
                    .fold(f32::MIN, f32::max)
            })
            .collect();
        assert!(
            maxima.windows(2).all(|m| m[0] >= m[1]),
            "bande {b} : {maxima:?}"
        );
    }
}

/// BANDES-1 : autant de bandes que de courbes ; une droite horizontale n'est dans aucune
/// bande horizontale, ni une verticale dans une bande verticale.
#[test]
fn test_les_bandes_ignorent_ce_qui_ne_croise_pas_leur_rayon() {
    let e = emballe(|c| rectangle(c, (0.0, 0.0, 1.0, 1.0), true), 0.5);
    // Quatre côtés : deux verticaux pour les bandes horizontales, deux horizontaux pour les
    // verticales.
    assert_eq!(e.nombre_de_bandes, [2, 2]);
    for b in 0..2 {
        for &r in bande(&e, b) {
            let p = courbe(&e, r);
            assert!(
                p[0][0] == p[2][0],
                "un côté horizontal dans une bande horizontale"
            );
        }
    }
}

/// **Un bord droit couvre exactement l'aire** : le pixel dont le centre est à `d` pixels
/// dedans est couvert de `½ + d` — c'est le filtre-boîte, sans approximation.
#[test]
fn test_un_bord_droit_couvre_exactement_son_aire() {
    // Un carré d'une unité, vu à cent pixels par unité.
    let e = emballe(|c| rectangle(c, (0.0, 0.0, 1.0, 1.0), true), 0.5);
    let ppu = [100.0, 100.0];
    for d in [-0.75f32, -0.5, -0.3, 0.0, 0.25, 0.5, 0.9] {
        // Le centre du pixel à `d` pixels à droite du bord gauche, à mi-hauteur.
        let x = d / 100.0;
        let c = couverture(&e, [x, 0.5], ppu);
        let attendu = (0.5 + d).clamp(0.0, 1.0);
        assert!(
            (c - attendu).abs() < 1e-5,
            "à {d} px : {c} au lieu de {attendu}"
        );
    }
    assert_eq!(couverture(&e, [0.5, 0.5], ppu), 1.0);
    assert_eq!(couverture(&e, [1.5, 0.5], ppu), 0.0);
}

/// Les enroulements : un trou (un contour en sens inverse) ne couvre rien ; deux contours
/// superposés dans le même sens couvrent une fois, pas deux.
#[test]
fn test_les_enroulements_font_les_trous_et_pas_les_doubles() {
    let ppu = [100.0, 100.0];
    let troue = emballe(
        |c| {
            rectangle(c, (0.0, 0.0, 1.0, 1.0), true);
            rectangle(c, (0.25, 0.25, 0.75, 0.75), false);
        },
        0.5,
    );
    assert_eq!(couverture(&troue, [0.5, 0.5], ppu), 0.0);
    assert_eq!(couverture(&troue, [0.1, 0.5], ppu), 1.0);
    let double = emballe(
        |c| {
            rectangle(c, (0.0, 0.0, 1.0, 1.0), true);
            rectangle(c, (0.25, 0.25, 0.75, 0.75), true);
        },
        0.5,
    );
    assert_eq!(couverture(&double, [0.5, 0.5], ppu), 1.0);
}

/// **L'aire se conserve** : la somme des couvertures d'un disque de quarante pixels de rayon
/// vaut son aire, à moins d'un centième de pixel par pixel de bord.
#[test]
fn test_un_disque_couvre_son_aire() {
    let rayon = 40.0;
    let e = emballe(|c| disque(c, (0.0, 0.0), rayon), 1.0 / 512.0);
    let mut somme = 0.0f64;
    let mut pleins = 0;
    for j in -45..45 {
        for i in -45..45 {
            let p = [i as f32 + 0.5, j as f32 + 0.5];
            let c = couverture(&e, p, [1.0, 1.0]);
            somme += f64::from(c);
            let r = f64::from(p[0]).hypot(f64::from(p[1]));
            // Loin du bord, la couverture est entière.
            if r < rayon - 1.0 {
                assert_eq!(c, 1.0, "au centre {p:?}");
                pleins += 1;
            } else if r > rayon + 1.0 {
                assert_eq!(c, 0.0, "dehors {p:?}");
            }
        }
    }
    let aire = std::f64::consts::PI * rayon * rayon;
    let perimetre = 2.0 * std::f64::consts::PI * rayon;
    assert!(
        (somme - aire).abs() < perimetre * 0.01,
        "somme {somme} pour une aire de {aire}"
    );
    assert!(pleins > 4000);
}

/// **BANDES-2 : là où la multiplication et l'addition fondues ne tombent pas dans la même bande
/// que séparées, les deux bandes sont rendues** — cherché aux frontières de bandes réelles, à
/// quelques crans de flottant près, jusqu'à en trouver.
#[test]
fn test_bandes_2_les_deux_arithmetiques_sont_comptees() {
    let mut h = Hasard(11);
    let mut trouvees = 0;
    for _ in 0..200 {
        let echelle = (1.0 + h.suivant() * 999.0) as f32;
        let decalage = (-(h.suivant() * 50.0)) as f32;
        for b in 1..40u32 {
            let frontiere = (b as f32 - decalage) / echelle;
            for k in -8i32..=8 {
                let v = f32::from_bits((frontiere.to_bits() as i32 + k) as u32);
                let separees = (v * echelle + decalage) as i32;
                let fondue = v.mul_add(echelle, decalage) as i32;
                if separees == fondue {
                    continue;
                }
                trouvees += 1;
                let (lo, hi) = bandes_de(v, echelle, decalage, 100);
                assert_eq!(
                    (lo as i32, hi as i32),
                    (separees.min(fondue), separees.max(fondue))
                );
            }
        }
    }
    assert!(
        trouvees > 0,
        "aucune frontiere ou les deux arithmetiques different"
    );
}
