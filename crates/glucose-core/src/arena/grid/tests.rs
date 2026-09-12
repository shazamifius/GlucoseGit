//! Les lois de l'index spatial. La première est la seule qui compte vraiment : l'index doit
//! rendre le même résultat que le balayage, jamais « à peu près ».

use super::*;
use crate::arena::Kind;

/// Une suite congruentielle : les cas sont variés mais strictement reproductibles.
struct Lcg(u64);

impl Lcg {
    fn next(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.0 >> 11
    }
    /// Un entier dans `[-span, span]`.
    fn coord(&mut self, span: i32) -> i32 {
        (self.next() % (2 * span as u64 + 1)) as i32 - span
    }
}

fn px(v: i32) -> Fx {
    Fx::from_px(v)
}

fn boite(x: i32, y: i32, w: i32, h: i32) -> Box2 {
    Box2::new(px(x), px(y), px(w), px(h))
}

/// Trois dispositions de nœuds, parce qu'un index qui ne marche que sur une grille régulière ne
/// marche pas : une grille, un amas dense, et du désordre avec des tailles très inégales.
fn peupler(disposition: &str, n: usize) -> Arena {
    let mut a = Arena::with_capacity(n);
    let mut r = Lcg(0x51ced);
    let cote = (n as f64).sqrt().ceil() as i32;
    for i in 0..n {
        let (x, y, w, h) = match disposition {
            "grille" => (i as i32 % cote * 400, i as i32 / cote * 400, 260, 80),
            "amas" => (r.coord(500), r.coord(500), 40 + r.coord(20).abs(), 30),
            _ => {
                // Du désordre, avec un nœud sur cinquante très grand : ce sont eux qui mettent
                // la structure en défaut si la liste des grands est mal traitée.
                let grand = i % 50 == 0;
                let (w, h) = if grand {
                    (2_000 + r.coord(3_000).abs(), 1_500 + r.coord(2_000).abs())
                } else {
                    (20 + r.coord(300).abs(), 20 + r.coord(200).abs())
                };
                (r.coord(20_000), r.coord(20_000), w, h)
            }
        };
        a.spawn(if i % 3 == 0 { Kind::Image } else { Kind::Text }, boite(x, y, w, h), NodeId::NONE);
    }
    a
}

/// **La loi de l'index** : sur trois dispositions, des milliers de vues et des nœuds morts en
/// chemin, la grille rend exactement les mêmes nœuds que le balayage de l'arène.
///
/// Les ordres diffèrent — la grille rend par cellule, le balayage par identifiant — donc les
/// deux résultats sont triés avant comparaison. Tout le reste doit coïncider.
#[test]
fn test_la_grille_rend_exactement_ce_que_rend_le_balayage() {
    for disposition in ["grille", "amas", "desordre"] {
        let mut a = peupler(disposition, 2_000);
        // Un nœud sur onze meurt : l'index ne doit jamais rendre un mort.
        for i in (0..a.slots()).step_by(11) {
            a.kill(NodeId::from_index(i));
        }
        let g = Grid::build(&a);
        g.check(&a).unwrap();

        let mut r = Lcg(0xf00d);
        let (mut par_index, mut par_balayage) = (Vec::new(), Vec::new());
        for essai in 0..500 {
            // Des vues de toutes tailles, du point unique au document entier, et des vues
            // entièrement hors du document.
            let taille = match essai % 5 {
                0 => 1,
                1 => 100,
                2 => 1_920,
                3 => 40_000,
                _ => 500,
            };
            let vue = boite(r.coord(25_000), r.coord(25_000), taille, taille * 9 / 16);

            g.query(&a, vue, &mut par_index);
            a.cull(vue, &mut par_balayage);
            par_index.sort();
            par_balayage.sort();
            assert_eq!(
                par_index, par_balayage,
                "{disposition}, vue {vue:?} : {} par l'index contre {} par le balayage",
                par_index.len(),
                par_balayage.len()
            );
        }
    }
}

/// La taille de cellule n'est pas réglée : elle sort des données, et elle donne bien de l'ordre
/// d'un nœud par cellule — c'est la règle annoncée par l'en-tête du module.
#[test]
fn test_la_cellule_deduite_donne_environ_un_noeud_par_cellule() {
    for n in [100usize, 10_000, 100_000] {
        let a = peupler("grille", n);
        let g = Grid::build(&a);
        let par_cellule = n as f64 / g.cells() as f64;
        assert!(
            (0.25..=4.0).contains(&par_cellule),
            "{n} nœuds dans {} cellules : {par_cellule:.2} par cellule",
            g.cells()
        );
    }
}

/// **Le piège des grilles denses, évité par construction.** Deux nœuds aux antipodes du
/// document couvrent seize millions de pixels : une cellule de taille fixe demanderait des
/// centaines de millions d'entrées. Ici le nombre de cellules suit le nombre de nœuds.
#[test]
fn test_deux_noeuds_aux_antipodes_ne_font_pas_exploser_l_index() {
    let mut a = Arena::new();
    a.spawn(Kind::Text, Box2::new(Fx::MIN, Fx::MIN, px(10), px(10)), NodeId::NONE);
    a.spawn(Kind::Text, Box2::new(px(8_000_000), px(8_000_000), px(10), px(10)), NodeId::NONE);

    let g = Grid::build(&a);
    assert!(g.cells() <= 16, "{} cellules pour deux nœuds", g.cells());
    assert!(g.bytes() < 1_000, "{} octets d'index pour deux nœuds", g.bytes());
    g.check(&a).unwrap();

    // Et ils restent trouvables, chacun de son côté du document.
    let mut vus = Vec::new();
    g.query(&a, Box2::new(Fx::MIN, Fx::MIN, px(100), px(100)), &mut vus);
    assert_eq!(vus, vec![NodeId::from_index(0)]);
    g.query(&a, boite(7_999_000, 7_999_000, 100_000, 100_000), &mut vus);
    assert_eq!(vus, vec![NodeId::from_index(1)]);
}

/// Les nœuds trop grands pour une cellule vont à part — et restent trouvables exactement comme
/// les autres. C'est le point faible annoncé de la structure, donc celui qui est le plus testé.
#[test]
fn test_les_grands_noeuds_vont_a_part_et_restent_trouvables() {
    let mut a = Arena::with_capacity(1_001);
    for i in 0..1_000 {
        a.spawn(Kind::Text, boite(i % 40 * 300, i / 40 * 300, 260, 80), NodeId::NONE);
    }
    // Une membrane qui couvre tout le document.
    let geante = a.spawn(Kind::Membrane, boite(-500, -500, 14_000, 9_000), NodeId::NONE);

    let g = Grid::build(&a);
    assert_eq!(g.large(), 1, "la géante et elle seule est hors grille");
    assert_eq!(g.len(), 1_000);
    g.check(&a).unwrap();

    // Elle apparaît dans toute vue qu'elle croise, et dans aucune autre.
    let mut vus = Vec::new();
    for (vue, attendue) in [
        (boite(0, 0, 10, 10), true),
        (boite(6_000, 4_000, 10, 10), true),
        (boite(-10_000, -10_000, 100, 100), false),
        (boite(100_000, 100_000, 100, 100), false),
    ] {
        g.query(&a, vue, &mut vus);
        assert_eq!(vus.contains(&geante), attendue, "vue {vue:?}");
    }
}

/// Un nœud mort n'est pas indexé, et l'index ne le rend jamais.
#[test]
fn test_les_morts_ne_sont_pas_indexes() {
    let mut a = peupler("grille", 500);
    for i in 0..a.slots() {
        if i % 2 == 0 {
            a.kill(NodeId::from_index(i));
        }
    }
    let g = Grid::build(&a);
    assert_eq!(g.len() + g.large(), a.len());
    g.check(&a).unwrap();

    let mut vus = Vec::new();
    g.query(&a, boite(-100_000, -100_000, 200_000, 200_000), &mut vus);
    assert_eq!(vus.len(), a.len());
    assert!(vus.iter().all(|&id| a.alive(id)));
}

/// Une arène vide, ou dont tous les nœuds sont morts, donne un index vide qui répond sans rien
/// faire tomber.
#[test]
fn test_une_arene_sans_noeud_vivant_donne_un_index_vide() {
    let mut vus = vec![NodeId::from_index(7)];
    for a in [Arena::new(), {
        let mut a = Arena::new();
        let id = a.spawn(Kind::Text, boite(0, 0, 10, 10), NodeId::NONE);
        a.kill(id);
        a
    }] {
        let g = Grid::build(&a);
        assert!(g.is_empty());
        assert_eq!(g.cells(), 0);
        // Quatre octets : la borne unique que l'invariant GRD-1 exige même sans cellule.
        assert_eq!(g.bytes(), 4);
        g.check(&a).unwrap();
        g.query(&a, boite(0, 0, 1_000, 1_000), &mut vus);
        assert!(vus.is_empty(), "la sortie est vidée même sans résultat");
    }
}

/// Tous les nœuds au même point : la grille dégénère en une cellule, et c'est le bon
/// comportement — quand tout est au même endroit, tout balayer est ce qu'il faut faire.
#[test]
fn test_tous_les_noeuds_au_meme_point_donnent_une_seule_cellule() {
    let mut a = Arena::with_capacity(100);
    for _ in 0..100 {
        a.spawn(Kind::Text, boite(42, 42, 10, 10), NodeId::NONE);
    }
    let g = Grid::build(&a);
    assert_eq!(g.cells(), 1);
    assert_eq!(g.len(), 100);
    g.check(&a).unwrap();

    let mut vus = Vec::new();
    g.query(&a, boite(40, 40, 20, 20), &mut vus);
    assert_eq!(vus.len(), 100);
    g.query(&a, boite(1_000, 1_000, 20, 20), &mut vus);
    assert!(vus.is_empty());
}

/// L'index pèse quatre octets par nœud plus le tableau de cellules — pas huit ni seize, parce
/// qu'aucun nœud n'est inscrit deux fois.
#[test]
fn test_l_index_pese_quatre_octets_par_noeud_sans_duplication() {
    let a = peupler("grille", 10_000);
    let g = Grid::build(&a);
    assert_eq!(g.len() + g.large(), 10_000, "chaque nœud est inscrit une fois, et une seule");
    let par_noeud = g.bytes() as f64 / 10_000.0;
    assert!(par_noeud < 12.0, "{par_noeud:.1} octets par nœud");
}

/// La vérification attrape une incohérence plutôt que de la laisser passer.
#[test]
fn test_la_verification_attrape_une_incoherence() {
    let a = peupler("grille", 100);
    let mut g = Grid::build(&a);
    g.check(&a).unwrap();

    let derniere = g.starts.len() - 1;
    g.starts[derniere] += 1;
    assert!(g.check(&a).unwrap_err().contains("dernière borne"));
    g.starts[derniere] -= 1;

    g.starts[1] = u32::MAX; // bornes qui reculent
    assert!(g.check(&a).unwrap_err().contains("reculent"));
    g = Grid::build(&a);

    g.items[0] = NodeId::from_index(99); // rangé dans la mauvaise cellule
    assert!(g.check(&a).unwrap_err().contains("appartient à"));

    g.starts.pop(); // plus assez de bornes
    assert!(g.check(&a).unwrap_err().contains("bornes pour"));
}
