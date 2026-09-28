//! FLECHE-5 : le contournement, contre une recherche à plat qui connaît tout d'avance, et
//! contre un échantillonnage dense qui ne partage rien avec lui.

use super::*;

/// Un tirage déterministe (congruence linéaire de Knuth) : les scènes sont les mêmes à chaque
/// exécution.
struct Hasard(u64);

impl Hasard {
    fn suivant(&mut self) -> f64 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        (self.0 >> 11) as f64 / (1u64 << 53) as f64
    }

    fn entre(&mut self, a: f64, b: f64) -> f64 {
        a + (b - a) * self.suivant()
    }
}

fn se_touchent(r: Rect, zone: Rect) -> bool {
    r.left <= zone.right()
        && zone.left <= r.right()
        && r.top <= zone.bottom()
        && zone.top <= r.bottom()
}

/// L'index du test : une liste, interrogée exhaustivement.
fn requete_sur(obstacles: &[Rect]) -> impl FnMut(Rect, &mut Vec<Rect>) + '_ {
    move |zone, sortie| {
        sortie.extend(obstacles.iter().copied().filter(|r| se_touchent(*r, zone)));
    }
}

fn longueur(points: &[Point]) -> f64 {
    points.windows(2).map(|s| distance(s[0], s[1])).sum()
}

fn chemin_complet(a: Point, etapes: &[Point], b: Point) -> Vec<Point> {
    std::iter::once(a)
        .chain(etapes.iter().copied())
        .chain(std::iter::once(b))
        .collect()
}

/// **Le segment passe-t-il par l'intérieur de la boîte gonflée ?** Vérifié par deux mille
/// points le long du segment, sans rien emprunter au prédicat du module.
fn echantillon_dedans(p: Point, q: Point, r: Rect, ecart: f64) -> bool {
    let e = 1e-6;
    (1..2000).any(|k| {
        let t = k as f64 / 2000.0;
        let (x, y) = (p.0 + (q.0 - p.0) * t, p.1 + (q.1 - p.1) * t);
        x > r.left - ecart + e
            && x < r.right() + ecart - e
            && y > r.top - ecart + e
            && y < r.bottom() + ecart - e
    })
}

fn dedans(p: Point, r: Rect, ecart: f64) -> bool {
    Boite::de(r, ecart).contient(p)
}

/// **La recherche à plat** : tous les obstacles connus d'avance, tous les coins, Dijkstra.
/// Rend la longueur du plus court chemin et ce chemin, ou rien s'il n'y en a pas.
fn plus_court_a_plat(a: Point, b: Point, obstacles: &[Rect]) -> Option<(f64, Vec<Point>)> {
    let boites: Vec<Boite> = obstacles
        .iter()
        .map(|r| Boite::de(*r, ECART))
        .filter(|o| !o.contient(a) && !o.contient(b))
        .collect();
    let mut sommets = vec![a, b];
    for o in &boites {
        for c in o.coins() {
            if !boites.iter().any(|x| x.contient(c)) {
                sommets.push(c);
            }
        }
    }
    let voit = |i: usize, j: usize| {
        let (p, q) = (sommets[i], sommets[j]);
        !boites.iter().any(|o| o.traversee(p, q))
    };
    let n = sommets.len();
    let mut d = vec![f64::INFINITY; n];
    let mut avant = vec![0; n];
    let mut fait = vec![false; n];
    d[0] = 0.0;
    loop {
        let u = (0..n)
            .filter(|&k| !fait[k] && d[k].is_finite())
            .min_by(|&x, &y| d[x].total_cmp(&d[y]))?;
        if u == 1 {
            let mut chemin = vec![b];
            let mut k = 1;
            while k != 0 {
                k = avant[k];
                chemin.push(sommets[k]);
            }
            chemin.reverse();
            return Some((d[1], chemin));
        }
        fait[u] = true;
        for v in 0..n {
            let par_u = d[u] + distance(sommets[u], sommets[v]);
            if !fait[v] && par_u < d[v] && voit(u, v) {
                d[v] = par_u;
                avant[v] = u;
            }
        }
    }
}

/// Rien sur le chemin : la flèche va droit, et l'itinéraire est vide.
#[test]
fn test_rien_ne_gene_la_fleche_va_droit() {
    let obstacles = [Rect::new(0.0, 200.0, 100.0, 100.0)];
    let mut requete = requete_sur(&obstacles);
    let bout = |x| (x, 0.0);
    assert!(itineraire(bout(0.0), bout(1000.0), &mut requete).is_empty());
}

/// **Un bloc au milieu se contourne par deux de ses coins**, à l'écart près, et du côté le plus
/// court : le bloc est plus haut que bas, la flèche passe dessous.
#[test]
fn test_un_bloc_au_milieu_se_contourne_par_ses_coins() {
    let obstacles = [Rect::new(400.0, -150.0, 200.0, 200.0)];
    let mut requete = requete_sur(&obstacles);
    let bout = |x| (x, 0.0);
    let etapes = itineraire(bout(0.0), bout(1000.0), &mut requete);
    assert_eq!(etapes, vec![(388.0, 62.0), (612.0, 62.0)]);
}

/// **Le chemin est le plus court, et il ne traverse rien** — sur deux mille scènes tirées au
/// hasard. La recherche paresseuse trouve exactement la longueur
/// de la recherche à plat, et ne lit qu'une partie des obstacles.
#[test]
fn test_le_chemin_est_le_plus_court_et_ne_traverse_rien() {
    let mut h = Hasard(42);
    let (mut detours, mut enfermes, mut lus, mut presents) = (0, 0, 0, 0);
    for scene in 0..2000 {
        let n = 3 + (h.suivant() * 22.0) as usize;
        let obstacles: Vec<Rect> = (0..n)
            .map(|_| {
                let (w, hh) = (h.entre(20.0, 200.0), h.entre(20.0, 200.0));
                Rect::new(h.entre(0.0, 1000.0), h.entre(0.0, 1000.0), w, hh)
            })
            .collect();
        let bout = |h: &mut Hasard| (h.entre(0.0, 1000.0), h.entre(0.0, 1000.0));
        let (a, b) = (bout(&mut h), bout(&mut h));
        let mut compte = 0;
        let mut requete = |zone: Rect, sortie: &mut Vec<Rect>| {
            let avant = sortie.len();
            sortie.extend(obstacles.iter().copied().filter(|r| se_touchent(*r, zone)));
            compte += sortie.len() - avant;
        };
        let etapes = itineraire(a, b, &mut requete);
        let chemin = chemin_complet(a, &etapes, b);
        let direct = distance(a, b);
        let optimum = plus_court_a_plat(a, b, &obstacles);
        match &optimum {
            Some((optimum, le_bon)) if (optimum - direct).abs() > 1e-9 => {
                detours += 1;
                assert!(
                    (longueur(&chemin) - optimum).abs() < 1e-6,
                    "scène {scene} : {} contre l'optimum {optimum}\n trouvé {chemin:?}\n le bon \
                     {le_bon:?}\n départ {a:?}\n arrivée {b:?}\n obstacles \
                     {obstacles:?}",
                    longueur(&chemin)
                );
            }
            _ => assert!(etapes.is_empty(), "scène {scene} : droit, ou aucun chemin"),
        }
        lus += compte;
        presents += n;
        // Sans chemin — un bout enfermé —, la flèche va droit et traverse, par construction.
        if optimum.is_none() {
            enfermes += 1;
            continue;
        }
        // Rien de traversé, vérifié sans le prédicat du module.
        for (k, s) in chemin.windows(2).enumerate() {
            for r in &obstacles {
                if !dedans(a, *r, ECART) && !dedans(b, *r, ECART) {
                    assert!(
                        !echantillon_dedans(s[0], s[1], *r, ECART),
                        "scène {scene} : le tronçon {k} traverse {r:?}"
                    );
                }
            }
        }
    }
    assert!(detours > 50, "les scènes contournent vraiment : {detours}");
    assert!(enfermes < 30, "peu de bouts enfermés : {enfermes}");
    assert!(
        lus < presents,
        "la recherche ne lit pas tout : {lus} sur {presents}"
    );
}

/// **Un bloc qui contient un bout ne se contourne pas** : on ne sort pas d'où l'on est.
#[test]
fn test_un_bloc_qui_contient_un_bout_ne_se_contourne_pas() {
    let obstacles = [Rect::new(-50.0, -50.0, 100.0, 100.0)];
    let mut requete = requete_sur(&obstacles);
    let depart = (0.0, 0.0);
    let arrivee = (1000.0, 0.0);
    assert!(itineraire(depart, arrivee, &mut requete).is_empty());
}

/// **Un bout enfermé va droit** : quatre blocs qui se chevauchent l'entourent sans le
/// contenir ; aucun chemin n'existe.
#[test]
fn test_un_bout_enferme_va_droit() {
    let obstacles = [
        Rect::new(-200.0, -200.0, 400.0, 100.0),
        Rect::new(-200.0, 100.0, 400.0, 100.0),
        Rect::new(-200.0, -200.0, 100.0, 400.0),
        Rect::new(100.0, -200.0, 100.0, 400.0),
    ];
    let mut requete = requete_sur(&obstacles);
    let depart = (0.0, 0.0);
    let arrivee = (1000.0, 0.0);
    assert!(plus_court_a_plat(depart, arrivee, &obstacles).is_none());
    assert!(itineraire(depart, arrivee, &mut requete).is_empty());
}

/// **Dix mille blocs loin du trajet ne coûtent rien** : l'index n'est interrogé que le long du
/// segment, et n'en rend aucun.
#[test]
fn test_seul_le_voisinage_du_trajet_est_lu() {
    let obstacles: Vec<Rect> = (0..10_000)
        .map(|k| {
            Rect::new(
                (k % 100) as f64 * 50.0,
                5000.0 + (k / 100) as f64 * 50.0,
                40.0,
                40.0,
            )
        })
        .collect();
    let mut lus = 0;
    let mut requete = |zone: Rect, sortie: &mut Vec<Rect>| {
        let avant = sortie.len();
        sortie.extend(obstacles.iter().copied().filter(|r| se_touchent(*r, zone)));
        lus += sortie.len() - avant;
    };
    let bout = |x| (x, 0.0);
    assert!(itineraire(bout(0.0), bout(5000.0), &mut requete).is_empty());
    assert_eq!(lus, 0);
}

/// **Un bout enfermé dans une mosaïque se reconnaît en ne lisant que des pourtours** :
/// quarante mille blocs serrés, le départ dans un trou au milieu. Aucun chemin, la flèche va
/// droit — et la recherche a lu moins de boîtes que la mosaïque n'en a, là où ramasser toute
/// l'aire en lisait neuf par bloc.
#[test]
fn test_un_bout_enferme_dans_une_mosaique_se_reconnait() {
    let obstacles: Vec<Rect> = (0..40_000)
        .filter(|k| !(k % 200 == 100 && k / 200 == 100))
        .map(|k| Rect::new((k % 200) as f64 * 50.0, (k / 200) as f64 * 50.0, 45.0, 45.0))
        .collect();
    let mut lus = 0;
    let mut requete = |zone: Rect, sortie: &mut Vec<Rect>| {
        let avant = sortie.len();
        sortie.extend(obstacles.iter().copied().filter(|r| se_touchent(*r, zone)));
        lus += sortie.len() - avant;
    };
    let depart = (5022.5, 5022.5);
    let arrivee = (20_000.0, 5022.5);
    assert!(itineraire(depart, arrivee, &mut requete).is_empty());
    assert!(
        lus < obstacles.len(),
        "{lus} obstacles lus sur {}",
        obstacles.len()
    );
}

/// Un mur de `colonnes × rangees` blocs de 45 serrés à 5 d'écart, son coin en haut à gauche en
/// `(0, 0)`, et ce que la recherche en a lu.
fn mur(colonnes: usize, rangees: usize) -> Vec<Rect> {
    (0..colonnes * rangees)
        .map(|k| {
            let (i, j) = ((k % colonnes) as f64, (k / colonnes) as f64);
            Rect::new(i * 50.0, j * 50.0, 45.0, 45.0)
        })
        .collect()
}

/// **Un mur de dix mille blocs se contourne par ses coins extérieurs** — le grand tour qu'il
/// préfère — en ne lisant que son pourtour. Découvert bloc par bloc, il demandait 132 secondes.
#[test]
fn test_un_mur_se_contourne_par_ses_coins_exterieurs() {
    let blocs = mur(100, 100);
    let mut lus = 0;
    let mut requete = |zone: Rect, sortie: &mut Vec<Rect>| {
        let avant = sortie.len();
        sortie.extend(blocs.iter().copied().filter(|r| se_touchent(*r, zone)));
        lus += sortie.len() - avant;
    };
    let etapes = itineraire((-500.0, 2522.5), (5500.0, 2522.5), &mut requete);
    // Le mur va de 0 à 4 995 ; gonflé de l'écart, de −12 à 5 007.
    assert_eq!(etapes, vec![(-12.0, 5007.0), (5007.0, 5007.0)]);
    assert!(lus < blocs.len(), "le pourtour seul : {lus} lectures");
}

/// **Un mur lointain ne coûte rien à la visibilité, quelle que soit sa taille** : un segment qui
/// ne l'approche pas l'écarte d'un coup, par sa boîte englobante. Le travail d'un même test de
/// visibilité — compté, pas chronométré — est le même à côté d'un mur de dix blocs de côté et
/// d'un mur de vingt : sans l'écart, il suivrait le pourtour du mur (fiche 43).
#[test]
fn test_un_mur_lointain_ne_coute_rien_a_la_visibilite() {
    let travail_a_cote_d_un_mur = |cote: usize| {
        let proche = Rect::new(100.0, -50.0, 50.0, 100.0);
        let mut obstacles: Vec<Rect> = mur(cote, cote)
            .into_iter()
            .map(|r| Rect::new(r.left + 5000.0, r.top + 5000.0, r.width, r.height))
            .collect();
        obstacles.push(proche);
        let mut requete = requete_sur(&obstacles);
        let mut r = Recherche::nouvelle((0.0, 0.0), (300.0, 0.0), &mut requete);
        r.amasser(Boite::de(proche, ECART)).expect("dans le budget");
        r.amasser(Boite::de(obstacles[0], ECART))
            .expect("dans le budget");
        assert_eq!(r.amas.len(), 2, "le bloc proche et le mur");
        let avant = r.travail;
        assert!(r.se_voient(&[(0.0, 100.0), (300.0, 100.0)], 0, 1));
        r.travail - avant
    };
    assert_eq!(travail_a_cote_d_un_mur(10), travail_a_cote_d_un_mur(20));
}

/// **Le budget borne le travail** : un mur long de deux mille blocs, les bouts de part et
/// d'autre de son milieu. Son pourtour est trop long à longer pour un détour de cent mille
/// unités : la flèche va droit.
#[test]
fn test_le_budget_borne_le_travail() {
    let blocs = mur(2000, 3);
    let mut requete = requete_sur(&blocs);
    let etapes = itineraire((50_000.0, -500.0), (50_000.0, 700.0), &mut requete);
    assert!(etapes.is_empty(), "droit : {etapes:?}");
}

/// **Une courbe qui mordrait se resserre**, et ne mord plus : l'épingle à cheveux de
/// Catmull-Rom déborde de sa ligne brisée, sur un bloc posé juste derrière.
#[test]
fn test_une_courbe_qui_mordrait_se_resserre() {
    let bloc = Rect::new(1020.0, -60.0, 100.0, 130.0);
    let obstacles = [bloc];
    let mut requete = requete_sur(&obstacles);
    let (debut, fin) = ((0.0, 0.0), (0.0, 10.0));
    let mut etapes = vec![(1000.0, 0.0), (1000.0, 10.0)];
    let bouts = |_: &[Point]| (debut, fin);
    let avant = trace::morceaux(&chemin_complet(debut, &etapes, fin), true);
    let mordait = avant.iter().any(|m| {
        (0..=1000).any(|k| {
            let p = m.point(k as f64 / 1000.0);
            Boite::de(bloc, 0.0).contient(p)
        })
    });
    assert!(mordait, "la courbe d'origine mord le bloc");
    resserrer(&mut etapes, &bouts, [debut, fin], &mut requete);
    assert!(etapes.len() > 2, "des étapes ont été ajoutées");
    let apres = trace::morceaux(&chemin_complet(debut, &etapes, fin), true);
    for m in &apres {
        for k in 0..=1000 {
            let p = m.point(k as f64 / 1000.0);
            assert!(!Boite::de(bloc, 0.0).contient(p), "{p:?} mord encore");
        }
    }
}

/// Le prédicat de traversée : longer un bord ou toucher un coin n'est pas traverser.
#[test]
fn test_longer_n_est_pas_traverser() {
    let o = Boite {
        x0: 0.0,
        y0: 0.0,
        x1: 10.0,
        y1: 10.0,
    };
    assert!(!o.traversee((-5.0, 0.0), (15.0, 0.0)), "le long du bord");
    assert!(!o.traversee((-5.0, 5.0), (0.0, 10.0)), "jusqu'au coin");
    assert!(
        o.traversee((-1.0, 11.0), (1.0, 9.0)),
        "couper un coin, c'est traverser"
    );
    assert!(o.traversee((-5.0, 5.0), (15.0, 5.0)), "de part en part");
    assert!(o.traversee((5.0, 5.0), (5.0, 6.0)), "dedans");
    assert!(!o.traversee((-5.0, -5.0), (-1.0, 20.0)), "à côté");
}

/// **Un bord couvert par morceaux emboîtés reste couvert** : une grande voisine couvre tout le
/// bord, une petite s'emboîte dedans — l'union des intervalles ne recule pas.
#[test]
fn test_un_bord_couvert_par_morceaux_emboites() {
    let b = |x0, y0, x1, y1| Boite { x0, y0, x1, y1 };
    let x = b(0.0, 0.0, 100.0, 100.0);
    let voisines = [
        // Au-dessus : une grande, et une petite emboîtée dedans.
        b(-100.0, -50.0, 300.0, 10.0),
        b(40.0, -20.0, 60.0, 5.0),
        // Au-dessous, à gauche, à droite.
        b(-100.0, 90.0, 300.0, 150.0),
        b(-50.0, -100.0, 10.0, 300.0),
        b(90.0, -100.0, 150.0, 300.0),
    ];
    assert!(!super::amas::a_un_bord_expose(&x, &voisines));
    assert!(
        super::amas::a_un_bord_expose(&x, &voisines[1..]),
        "sans la grande, le haut s'ouvre"
    );
}
