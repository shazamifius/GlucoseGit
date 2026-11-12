//! PARU-1 : ce que le compositeur dit de chaque image, compté sans compositeur.

use super::*;
use std::sync::{Arc, Mutex};

/// Une période de 60 Hz, en nanosecondes.
const P_NS: u32 = 16_666_667;

fn vue(us: u64, battement: u64) -> Option<Vue> {
    Some(Vue {
        us,
        battement,
        periode_ns: P_NS,
    })
}

/// Une session : chaque image `(envoi, échéance)` part, puis son verdict arrive.
/// Une image : son envoi et son échéance, puis son verdict.
type Image = ((u64, Option<u64>), Option<Vue>);

fn ecran_apres(images: &[Image]) -> Ecran {
    let mut e = Ecran::default();
    for &((envoi, du), _) in images {
        e.en_attente.push_back((envoi, du));
    }
    for &((envoi_us, _), vue) in images {
        e.recevoir(Retour { envoi_us, vue });
    }
    e
}

#[test]
fn test_paru_1_compte_les_montrees_et_les_jetees() {
    let e = ecran_apres(&[
        ((1_000, None), vue(40_000, 10)),
        ((2_000, None), None),
        ((3_000, None), vue(56_667, 11)),
    ]);
    assert_eq!((e.montrees, e.jetees), (2, 1));
}

/// **Un battement sauté, c'est une image due que l'écran n'a pas montrée** — et la preuve à
/// l'envers : la même suite de battements, sans image due (une pause), ne compte rien.
#[test]
fn test_paru_1_un_battement_saute_quand_l_image_etait_due() {
    // B était due (échéance 20 000) avant que A paraisse (40 000), et paraît deux battements
    // après A : l'écran a remontré A une fois.
    let e = ecran_apres(&[
        ((10_000, Some(9_000)), vue(40_000, 10)),
        ((25_000, Some(20_000)), vue(73_333, 12)),
    ]);
    assert_eq!((e.dus, e.sautes), (2, 1));
    // C n'a été demandée qu'après que B a paru : rien n'était dû entre les deux.
    let e = ecran_apres(&[
        ((10_000, Some(9_000)), vue(40_000, 10)),
        ((25_000, Some(20_000)), vue(73_333, 12)),
        ((500_000, Some(490_000)), vue(523_333, 39)),
    ]);
    assert_eq!((e.dus, e.sautes), (2, 1), "une pause n'est pas un saut");
    // Une image sans échéance (le système l'a demandée) ne se juge pas non plus.
    let e = ecran_apres(&[
        ((10_000, Some(9_000)), vue(40_000, 10)),
        ((25_000, None), vue(73_333, 12)),
    ]);
    assert_eq!((e.dus, e.sautes), (0, 0));
}

/// Un écran qui ne compte pas ses battements : l'écart se lit sur le temps, rapporté à la
/// période et **arrondi** — 33 300 µs à 60 Hz font deux battements (1,998), 16 600 en font un
/// (0,996) ; tronqués, ils en feraient un et zéro.
#[test]
fn test_paru_1_sans_numero_le_temps_rapporte_a_la_periode() {
    let e = ecran_apres(&[
        ((10_000, Some(9_000)), vue(40_000, 0)),
        ((25_000, Some(20_000)), vue(73_300, 0)),
        ((45_000, Some(41_000)), vue(89_900, 0)),
    ]);
    assert_eq!((e.dus, e.sautes), (3, 1));
}

/// Un écran à fréquence variable n'a pas de période (zéro), mais compte ses battements : le
/// compte fait foi.
#[test]
fn test_paru_1_sans_periode_le_compte_de_l_ecran_fait_foi() {
    let sans_periode = |us, battement| {
        Some(Vue {
            us,
            battement,
            periode_ns: 0,
        })
    };
    let e = ecran_apres(&[
        ((10_000, Some(9_000)), sans_periode(40_000, 10)),
        ((25_000, Some(20_000)), sans_periode(61_000, 12)),
    ]);
    assert_eq!((e.dus, e.sautes), (2, 1));
}

/// Les verdicts arrivent dans l'ordre des envois : un envoi resté sans verdict quand un plus
/// récent en reçoit un sort de l'attente — elle ne grandit pas sur une session de plusieurs
/// heures. Un verdict qui ne retrouve pas son envoi ne compte pas.
#[test]
fn test_paru_1_l_attente_ne_grandit_pas() {
    let mut e = Ecran::default();
    for envoi in 1..=5 {
        e.en_attente.push_back((envoi, None));
    }
    e.recevoir(Retour {
        envoi_us: 4,
        vue: vue(40_000, 10),
    });
    assert_eq!(e.en_attente, VecDeque::from([(5, None)]));
    e.recevoir(Retour {
        envoi_us: 2,
        vue: vue(40_000, 10),
    });
    assert_eq!(e.montrees, 1, "un verdict orphelin ne compte pas");
}

#[test]
fn test_paru_1_la_latence_va_de_l_envoi_a_l_ecran() {
    let e = ecran_apres(&[((10_000, None), vue(47_300, 10))]);
    let l = e.latence.centile(0.5);
    assert!((37_000..=37_600).contains(&l), "37,3 ms, lu {l} µs");
}

/// Une fausse sonde : elle date ses envois et rend les verdicts qu'on lui confie.
struct Fausse {
    envois: Arc<Mutex<Vec<u64>>>,
    verdicts: Arc<Mutex<Vec<Retour>>>,
}

impl Sonde for Fausse {
    fn envoyer(&self) -> u64 {
        let mut envois = self.envois.lock().unwrap();
        let us = 1_000_000 + 16_667 * envois.len() as u64;
        envois.push(us);
        us
    }
    fn relever(&self) -> Vec<Retour> {
        std::mem::take(&mut *self.verdicts.lock().unwrap())
    }
}

/// **Par le vrai chemin** : chaque envoi relève ce qui est arrivé, et date l'échéance sur
/// l'horloge du compositeur par son âge.
#[test]
fn test_paru_1_avant_d_envoyer_releve_puis_demande() {
    let (envois, verdicts) = (Arc::default(), Arc::default());
    let mut e = Ecran::default();
    e.brancher(Some(Box::new(Fausse {
        envois: Arc::clone(&envois),
        verdicts: Arc::clone(&verdicts),
    })));
    let due = std::time::Instant::now() - std::time::Duration::from_millis(5);
    e.avant_d_envoyer(Some(due));
    let (envoi, du) = e.en_attente[0];
    assert_eq!(envoi, 1_000_000);
    let age = envoi - du.expect("une échéance");
    assert!(
        (5_000..50_000).contains(&age),
        "l'échéance a 5 ms, lu {age} µs"
    );
    verdicts.lock().unwrap().push(Retour {
        envoi_us: envoi,
        vue: vue(1_030_000, 10),
    });
    e.avant_d_envoyer(None);
    assert_eq!(e.montrees, 1, "le verdict est relevé au prochain envoi");
    assert_eq!(envois.lock().unwrap().len(), 2);
    // La surface refuse l'image : l'envoi sort de l'attente.
    e.rien_n_est_parti();
    assert!(e.en_attente.is_empty());
}

#[test]
fn test_paru_1_le_rapport_dit_ce_que_l_ecran_a_montre() {
    let mut t = String::new();
    Ecran::default().ecrire(&mut t);
    assert!(t.is_empty(), "sans verdict, la section se tait");
    let e = ecran_apres(&[
        ((10_000, Some(9_000)), vue(40_000, 10)),
        ((25_000, Some(20_000)), vue(73_333, 12)),
        ((30_000, None), None),
    ]);
    e.ecrire(&mut t);
    assert!(
        t.contains("3 image(s) confiee(s) : 2 montree(s), 1 jetee(s)"),
        "{t}"
    );
    assert!(
        t.contains("battements sautes alors qu'une image etait due : 1 sur 2"),
        "{t}"
    );
    assert!(t.contains("de l'envoi a l'ecran : median"), "{t}");
}
