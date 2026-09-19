//! Imprime le rapport de la chronique sur une session **reconstituée** depuis le terrain.
//!
//! # Pourquoi cet aperçu existe
//!
//! Un rapport qu'on ne relit pas ne sert à personne, et on ne peut pas juger sa lisibilité
//! sans le voir en entier. Le faire en lançant l'application demande une session réelle, donc
//! du temps de l'utilisateur — pour vérifier une mise en forme.
//!
//! Les nombres ci-dessous viennent de la chronique du 19/09 : 2 282 images, 429 photos,
//! `report` dominant, une image sur deux réduite, et des durées de rendu qui vont de 6 à
//! 67 ms. Ce n'est pas une simulation de ce qu'on voudrait voir : c'est ce que la machine a
//! fait, rejoué.
//!
//! `cargo run -p glucose-desktop --example apercu_chronique`

use glucose_desktop::chronique::{Chronique, Geste, Instantane};
use std::time::{Duration, Instant};

/// Les durées de rendu observées sur le terrain, en microsecondes, et leur fréquence.
///
/// Reprises des centiles de la chronique du 19/09 pour le geste « zoomer » : 6,14 ms en
/// médiane, 12,29 au p90, 28,67 au p99, 66,83 au pire.
const TERRAIN: [(u32, usize); 5] = [
    (6_140, 500),
    (8_200, 300),
    (12_290, 120),
    (28_670, 60),
    (66_830, 20),
];

fn main() {
    let mut c = Chronique::nouvelle();
    c.rythme.observer_la_machine(
        Duration::from_micros(4_166),
        "immediate (AUCUNE synchronisation)",
    );

    let mut horloge = Instant::now();
    let mut debut_precedent: Option<Instant> = None;
    // Mille pixels par seconde : un glissement franc au pavé tactile.
    let vitesse = 1_000.0;

    // **Les durées s'entrelacent, elles ne se groupent pas.** Une première version jouait les
    // cinq durées par blocs de plusieurs centaines : l'intervalle était alors constant à
    // l'intérieur de chaque bloc et ne changeait qu'aux quatre transitions, ce qui donnait
    // « 0 % d'images irrégulières » sous une distribution de balayages pourtant étalée de un à
    // seize. Une reconstitution qui lisse ce qu'elle prétend montrer ne montre rien.
    let sequence = entrelacer();
    for tour in 0..4 {
        for duree_us in &sequence {
            {
                let duree_us = *duree_us;
                let duree = Duration::from_micros(u64::from(duree_us));
                let pas = debut_precedent.map_or(Duration::ZERO, |avant| {
                    horloge.saturating_duration_since(avant)
                });
                let presentation = horloge + duree;
                let mesure = if pas.is_zero() {
                    Default::default()
                } else {
                    c.rythme
                        .presentee(presentation, horloge, pas, vitesse, true)
                };
                debut_precedent = Some(horloge);
                horloge = presentation;

                c.enregistrer(image(duree_us, tour, mesure));
            }
        }
    }
    println!("{}", c.rapport());
}

/// Les durées du terrain, mélangées de façon déterministe.
///
/// Un pas premier avec la longueur parcourt la suite entière sans jamais repasser deux fois
/// au même endroit : le mélange est donc complet, reproductible, et n'a besoin d'aucun
/// générateur — ce qui rend l'aperçu identique d'une exécution à l'autre.
fn entrelacer() -> Vec<u32> {
    let mut brut: Vec<u32> = Vec::new();
    for (duree_us, combien) in TERRAIN {
        brut.extend(std::iter::repeat_n(duree_us, combien));
    }
    let n = brut.len();
    let pas = 397;
    (0..n).map(|i| brut[(i * pas) % n]).collect()
}

/// Une image telle que le terrain en produit : tout redessiné, la scène réduite une fois sur
/// deux, et le report qui domine.
fn image(
    duree_us: u32,
    tour: usize,
    mesure: glucose_desktop::chronique::rythme::Mesure,
) -> Instantane {
    let mut vu = Instantane {
        duree_us,
        geste: Geste::TOUS
            .iter()
            .position(|g| *g == Geste::Zoomer)
            .unwrap_or(0) as u8,
        noeuds: 429,
        photos: 429,
        region_px: 2_560 * 1_600,
        fenetre_px: 2_560 * 1_600,
        surcouverture: 290,
        // Une image sur deux se rend plus petite, comme la chronique du 19/09 le montre.
        reduction: if tour.is_multiple_of(2) { 1 } else { 2 },
        intervalle_us: mesure.intervalle_us,
        saut_px: mesure.saut_px,
        fidelite_millieme: mesure.fidelite_millieme,
        ..Default::default()
    };
    // Le report prend l'essentiel, le reste se partage — la répartition du terrain.
    vu.postes_us[0] = duree_us * 45 / 100;
    vu.postes_us[1] = duree_us * 12 / 100;
    vu.postes_us[2] = duree_us * 10 / 100;
    vu
}
